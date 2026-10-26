//! OI-M0's exit test (OS integration Appendix C item 1): a DST run in
//! which the scripted OS delivers a 100-launch burst, a `Settings`
//! change and a `Power` will-sleep deadline input to a consumer in the
//! broker, which answers the deadline with `OsCommand.reply`. The trace
//! hash is the same on two runs and on every OS: the constant below was
//! measured once and every OS's CI must reproduce it.

use std::sync::{Arc, Mutex};

use hg_ipc::{BoundedString, BoundedVec, decode_exact, encode_to_vec};
use hg_os_types::os::{
    ColorScheme, Contrast, LaunchIntent, LaunchOrigin, LaunchRequest, Operand, OsCommand, OsInput,
    PowerEvent, SettingsValues, Tri, Verdict,
};
use hg_substrate::os::{OsDelivery, OsError};
use hg_substrate::{Memory, Priority};
use hg_substrate_os::memory::OsMemory;
use hg_substrate_sim::{Limits, OsScript, RunEnd, SimConfig, World};
use hg_types::{Duration, Instant};

const LAUNCHES: usize = 100;
const SLEEP_TOKEN: u64 = 7;

fn backing() -> Arc<dyn Memory> {
    Arc::new(OsMemory::new())
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn launch(i: usize) -> Vec<u8> {
    let url = BoundedString::try_new(format!("https://example.org/{i}")).unwrap();
    encode_to_vec(&OsInput::Launch {
        request: LaunchRequest {
            origin: LaunchOrigin::CommandLine,
            intent: LaunchIntent::Open,
            operands: BoundedVec::try_new(vec![Operand::Url { url }]).unwrap(),
            working_dir: None,
            profile: None,
            activation_token: None,
            sender: None,
        },
    })
}

fn settings() -> Vec<u8> {
    encode_to_vec(&OsInput::Settings {
        call: None,
        values: SettingsValues {
            scheme: ColorScheme::Dark,
            accent: Some(0x3366_ccff),
            contrast: Contrast::Normal,
            forced_colors: Tri::No,
            system_colors: BoundedVec::try_new(Vec::new()).unwrap(),
            reduced_motion: Tri::Yes,
            reduced_transparency: Tri::Unknown,
            inverted_colors: Tri::No,
            differentiate_without_color: Tri::No,
            text_scale: BoundedVec::try_new(Vec::new()).unwrap(),
            caret_width_px: None,
            focus_ring_width_px: Some(2),
        },
    })
}

fn will_sleep() -> Vec<u8> {
    encode_to_vec(&OsInput::Power {
        call: None,
        event: PowerEvent::WillSleep {
            token: SLEEP_TOKEN,
            deadline_ms: 3_000,
        },
    })
}

/// What the consumer saw, in order: "launch N", "settings", "will-sleep",
/// and what the OS saw: "reply N".
type Log = Arc<Mutex<Vec<String>>>;

fn run(seed: u64) -> (u64, Vec<String>) {
    let log: Log = Arc::default();
    let mut inputs: Vec<(Duration, Vec<u8>)> =
        (0..LAUNCHES).map(|i| (Duration::ZERO, launch(i))).collect();
    inputs.push((ms(50), settings()));
    inputs.push((ms(100), will_sleep()));
    let os_log = Arc::clone(&log);
    let script = OsScript {
        inputs,
        answer: Box::new(move |_id, frame| {
            if let Ok(OsCommand::Reply { token, verdict }) = decode_exact::<OsCommand>(frame) {
                assert_eq!(verdict, Verdict::Allow);
                os_log.lock().unwrap().push(format!("reply {token}"));
            }
            Vec::new()
        }),
    };
    let w = World::new(SimConfig::new(seed));
    let consumer = Arc::clone(&log);
    let _broker = w.spawn_broker(
        "browser",
        backing(),
        false,
        move |s| {
            let os = s.os.clone().expect("the broker has an OsPort");
            let seq = s.exec.sequence("browser.os", Priority::UserVisible);
            let log = Arc::clone(&consumer);
            let port = Arc::clone(&os);
            let guard = os
                .listen(
                    seq,
                    Box::new(move |d| {
                        let OsDelivery::Frame(frame) = d else { return };
                        let input = decode_exact::<OsInput>(&frame).unwrap();
                        let (line, reply) = match input {
                            OsInput::Launch { request } => match request.operands.as_slice() {
                                [Operand::Url { url }] => {
                                    (format!("launch {}", url.as_str()), None)
                                }
                                other => panic!("{other:?}"),
                            },
                            OsInput::Settings { .. } => ("settings".to_owned(), None),
                            OsInput::Power {
                                event: PowerEvent::WillSleep { token, .. },
                                ..
                            } => ("will-sleep".to_owned(), Some(token)),
                            OsInput::Power { .. }
                            | OsInput::DefaultStatus { .. }
                            | OsInput::Locale { .. }
                            | OsInput::TimeZone { .. }
                            | OsInput::Hardware { .. }
                            | OsInput::Displays { .. } => panic!("not scripted: {input:?}"),
                        };
                        log.lock().unwrap().push(line);
                        if let Some(token) = reply {
                            let answer = OsCommand::Reply {
                                token,
                                verdict: Verdict::Allow,
                            };
                            port.command(encode_to_vec(&answer)).unwrap();
                        }
                    }),
                )
                .unwrap();
            // A second listener is refused: one broker dispatcher.
            assert_eq!(
                os.listen(seq, Box::new(|_d| {})).err(),
                Some(OsError::Refused)
            );
            std::mem::forget(guard);
        },
        script,
    );
    let other = w.spawn("renderer", backing(), false, |s| {
        assert!(s.os.is_none(), "only the broker has an OsPort");
    });
    let _ = other;
    let end = w.run(Limits {
        max_steps: 100_000,
        max_time: Instant::from_nanos(60_000_000_000),
    });
    assert_eq!(end, RunEnd::Idle);
    let lines = log.lock().unwrap().clone();
    (w.summary().trace_hash, lines)
}

/// The trace hash every OS must reproduce (measured on Linux x86-64).
const EXPECTED_TRACE: u64 = 0x867e_66b9_2c4d_b0e9;

#[test]
fn the_scripted_os_delivers_a_burst_settings_and_a_sleep_deadline() {
    let (hash, lines) = run(42);
    assert_eq!(lines.len(), LAUNCHES + 3, "{lines:?}");
    for (i, line) in lines.iter().take(LAUNCHES).enumerate() {
        assert_eq!(line, &format!("launch https://example.org/{i}"));
    }
    assert_eq!(
        &lines[LAUNCHES..],
        ["settings", "will-sleep", &format!("reply {SLEEP_TOKEN}")]
    );
    // Two runs, one hash; and the hash every OS reproduces.
    assert_eq!(run(42).0, hash);
    assert_eq!(hash, EXPECTED_TRACE, "trace hash {hash:#018x}");
}
