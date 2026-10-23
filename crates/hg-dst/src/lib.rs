//! Izcal's deterministic simulation test runner (Stage 1 design 7.4;
//! coding standard CS-ER-2, CS-T5). Tier T.
//!
//! A *simulation binary* takes `--seed N`, runs a world under
//! `hg-substrate-sim`, and on success prints the report line
//! (`hg_substrate_sim::Summary::report_line`) last. `hg-dst` runs **one
//! seed per child OS process**, so a panic aborts only its seed: the
//! runner records the crash signature, the seed and a one-line replay
//! command, and goes on to the next seed. Failures are grouped by
//! signature, so one bug found by many seeds is one entry.
//!
//! The rerun check (CS-T5) runs every seed twice, in separate processes,
//! and reports any seed whose two traces differ: a determinism bug.
//!
//! Not yet here (later DST farm work, Stage 1 7.4): spreading seeds
//! across machines, shrinking a failing seed's fault schedule, and the
//! shared bug database.

use std::collections::BTreeMap;
use std::io::Read as _;
use std::ops::Range;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// A simulation binary and the arguments it gets besides `--seed`.
#[derive(Debug, Clone)]
pub struct Target {
    /// The simulation binary.
    pub bin: PathBuf,
    /// Arguments after `--seed N`.
    pub args: Vec<String>,
    /// Wall-clock time a seed may take before it counts as hung.
    pub timeout: Duration,
}

/// What a passing seed reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Tasks run.
    pub steps: u64,
    /// Virtual time reached, in nanoseconds.
    pub time_ns: u64,
    /// Crashes injected.
    pub crashes: u64,
    /// The trace hash.
    pub trace: u64,
}

/// How one seed ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Exited cleanly with a report line.
    Pass(Report),
    /// Anything else.
    Fail {
        /// What identifies the bug: the panic location, or how the
        /// process ended.
        signature: String,
        /// The end of stderr, for the bug report.
        detail: String,
    },
}

/// One seed's result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedResult {
    /// The seed.
    pub seed: u64,
    /// How it ended.
    pub outcome: Outcome,
}

impl Target {
    /// The command that replays `seed`, for a bug report.
    pub fn replay_command(&self, seed: u64) -> String {
        let mut parts = vec![
            quote(&self.bin.display().to_string()),
            "--seed".to_owned(),
            seed.to_string(),
        ];
        parts.extend(self.args.iter().map(|a| quote(a)));
        parts.join(" ")
    }

    /// Runs one seed in a child process.
    pub fn run_seed(&self, seed: u64) -> SeedResult {
        let outcome = match self.spawn_and_wait(seed) {
            Err(e) => Outcome::Fail {
                signature: format!("could not run: {}", e.kind()),
                detail: e.to_string(),
            },
            Ok(None) => Outcome::Fail {
                signature: "timeout".to_owned(),
                detail: format!("no exit within {:?}", self.timeout),
            },
            Ok(Some((status, stdout, stderr))) => judge(status, &stdout, &stderr),
        };
        SeedResult { seed, outcome }
    }

    fn spawn_and_wait(&self, seed: u64) -> std::io::Result<Option<(ExitStatus, String, String)>> {
        let mut child = Command::new(&self.bin)
            .arg("--seed")
            .arg(seed.to_string())
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // Drain both pipes while waiting, so a chatty child cannot block.
        let mut out = child.stdout.take();
        let mut err = child.stderr.take();
        let out_reader = std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(o) = out.as_mut() {
                let _ = o.read_to_string(&mut s);
            }
            s
        });
        let err_reader = std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(e) = err.as_mut() {
                let _ = e.read_to_string(&mut s);
            }
            s
        });
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break Some(status);
            }
            if start.elapsed() > self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let stdout = out_reader.join().unwrap_or_default();
        let stderr = err_reader.join().unwrap_or_default();
        Ok(status.map(|s| (s, stdout, stderr)))
    }

    /// Runs `seeds` on `jobs` parallel child processes; results in seed
    /// order.
    pub fn run_seeds(&self, seeds: Range<u64>, jobs: usize) -> Vec<SeedResult> {
        let next = AtomicU64::new(seeds.start);
        let results = Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            for _ in 0..jobs.max(1) {
                scope.spawn(|| {
                    loop {
                        // ordering: Relaxed — a work counter; results are
                        // collected under the mutex.
                        let seed = next.fetch_add(1, Ordering::Relaxed);
                        if seed >= seeds.end {
                            break;
                        }
                        let r = self.run_seed(seed);
                        results.lock().unwrap().push(r);
                    }
                });
            }
        });
        let mut v = results.into_inner().unwrap();
        v.sort_by_key(|r| r.seed);
        v
    }

    /// CS-T5: runs every seed twice and returns the seeds whose two runs
    /// did not end the same way (different traces, or one failing).
    pub fn rerun_mismatches(
        &self,
        seeds: Range<u64>,
        jobs: usize,
    ) -> Vec<(SeedResult, SeedResult)> {
        let first = self.run_seeds(seeds.clone(), jobs);
        let second = self.run_seeds(seeds, jobs);
        first
            .into_iter()
            .zip(second)
            .filter(|(a, b)| a != b)
            .collect()
    }
}

/// Failing seeds grouped by signature, each group in seed order.
pub fn group_failures(results: &[SeedResult]) -> BTreeMap<String, Vec<u64>> {
    let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for r in results {
        if let Outcome::Fail { signature, .. } = &r.outcome {
            groups.entry(signature.clone()).or_default().push(r.seed);
        }
    }
    groups
}

fn judge(status: ExitStatus, stdout: &str, stderr: &str) -> Outcome {
    if let Some(sig) = panic_signature(stderr) {
        return Outcome::Fail {
            signature: sig,
            detail: tail(stderr),
        };
    }
    if !status.success() {
        return Outcome::Fail {
            signature: describe(status),
            detail: tail(stderr),
        };
    }
    match parse_report(stdout) {
        Some(r) => Outcome::Pass(r),
        None => Outcome::Fail {
            signature: "no report line".to_owned(),
            detail: tail(stdout),
        },
    }
}

/// `panic at <file>:<line>:<col>` from the standard panic message. The
/// message text is left out: it often carries seed-specific values. The
/// path uses `/` on every OS, so one bug has one signature everywhere.
pub fn panic_signature(stderr: &str) -> Option<String> {
    let line = stderr.lines().find(|l| l.contains("panicked at "))?;
    let at = line.split("panicked at ").nth(1)?;
    Some(format!(
        "panic at {}",
        at.trim_end_matches(':').trim().replace('\\', "/")
    ))
}

fn describe(status: ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(sig) = status.signal() {
            return format!("signal {sig}");
        }
    }
    status
        .code()
        .map_or_else(|| "killed".to_owned(), |c| format!("exit {c}"))
}

/// Parses the last report line of `stdout`.
pub fn parse_report(stdout: &str) -> Option<Report> {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.starts_with("hg-dst: seed="))?;
    let field = |name: &str| {
        line.split_whitespace()
            .find_map(|kv| kv.strip_prefix(name)?.strip_prefix('='))
    };
    Some(Report {
        steps: field("steps")?.parse().ok()?,
        time_ns: field("time_ns")?.parse().ok()?,
        crashes: field("crashes")?.parse().ok()?,
        trace: u64::from_str_radix(field("trace")?, 16).ok()?,
    })
}

/// The last lines of `s`, with run-specific noise removed (the OS
/// thread ID in `thread 'main' (12345) panicked`), so the same failure
/// reads the same on every run.
fn tail(s: &str) -> String {
    let lines: Vec<String> = s.lines().map(drop_thread_id).collect();
    let from = lines.len().saturating_sub(8);
    lines.get(from..).unwrap_or_default().join("\n")
}

fn drop_thread_id(line: &str) -> String {
    if let Some((head, rest)) = line.split_once("' (")
        && head.starts_with("thread '")
        && let Some((id, after)) = rest.split_once(") ")
        && !id.is_empty()
        && id.chars().all(|c| c.is_ascii_digit())
    {
        return format!("{head}' {after}");
    }
    line.to_owned()
}

fn quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./\\=:,+".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_report_line_and_panic_signatures() {
        let out = "noise\nhg-dst: seed=3 steps=10 time_ns=42 crashes=1 trace=00000000000000ff\n";
        assert_eq!(
            parse_report(out),
            Some(Report {
                steps: 10,
                time_ns: 42,
                crashes: 1,
                trace: 255
            })
        );
        assert_eq!(parse_report("nothing"), None);
        assert_eq!(
            drop_thread_id("thread 'main' (84472848) panicked at a.rs:1:2:"),
            "thread 'main' panicked at a.rs:1:2:"
        );
        let err = "\nthread 'main' (123) panicked at src/bin/x.rs:10:5:\nbug: seed 7\n";
        assert_eq!(
            panic_signature(err).as_deref(),
            Some("panic at src/bin/x.rs:10:5")
        );
        assert_eq!(
            panic_signature("thread 'main' (9) panicked at src\\bin\\x.rs:10:5:").as_deref(),
            Some("panic at src/bin/x.rs:10:5")
        );
    }

    #[test]
    fn replay_commands_quote_what_needs_it() {
        let t = Target {
            bin: PathBuf::from("/a b/sim"),
            args: vec!["--bug".to_owned()],
            timeout: Duration::from_secs(1),
        };
        assert_eq!(t.replay_command(9), "'/a b/sim' --seed 9 --bug");
    }
}
