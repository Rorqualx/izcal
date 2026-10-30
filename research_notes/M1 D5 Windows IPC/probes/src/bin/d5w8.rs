//! M1 D5 review probes W8 and W9 (written from Microsoft Learn reference
//! pages; no header or crate source).
//!
//! W8: with a synchronous ReadFile pending on a non-overlapped pipe end,
//! which queries on that end (the same handle, or a duplicate of it) return
//! and which wait.
//! W9: a pipe end's read mode and wait mode: which handles can change them,
//! whether a change on one end shows on the other or on a duplicate, and
//! what a byte-mode reader sees.
#![allow(clippy::missing_safety_doc)]

use std::ptr::{null, null_mut};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use d5probe::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::Pipes::*;

const MSG: u32 = PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS;

fn say(tag: &str, s: String) {
    println!("[{tag}] {s}");
}

/// A pair whose client end is opened without FILE_FLAG_OVERLAPPED.
fn sync_pair() -> (H, H) {
    let name = format!(r"\\.\pipe\izcal-d5-{}", rand_hex());
    let server = unsafe {
        CreateNamedPipeW(
            w(&name).as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            MSG,
            1,
            65536,
            65536,
            0,
            null(),
        )
    };
    assert!(server != INVALID_HANDLE_VALUE, "CreateNamedPipeW {}", le());
    let client = unsafe {
        CreateFileW(
            w(&name).as_ptr(),
            FILE_READ_DATA
                | FILE_WRITE_DATA
                | FILE_READ_ATTRIBUTES
                | FILE_WRITE_ATTRIBUTES
                | SYNCHRONIZE,
            0,
            null(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    };
    assert!(client != INVALID_HANDLE_VALUE, "CreateFileW {}", le());
    let mode = PIPE_READMODE_MESSAGE;
    unsafe { SetNamedPipeHandleState(client, &mode, null(), null()) };
    (server, client)
}

/// Runs `f` on its own thread; reports whether it returned within 2 s,
/// and keeps waiting (up to 5 s more) after `release` to see if it does.
fn timed(
    name: &str,
    f: impl FnOnce() -> String + Send + 'static,
) -> std::sync::mpsc::Receiver<(String, u128)> {
    let (tx, rx) = channel();
    let t0 = Instant::now();
    let n = name.to_owned();
    std::thread::spawn(move || {
        let r = f();
        let _ = tx.send((format!("{n}: {r}"), t0.elapsed().as_millis()));
    });
    rx
}

fn w8() {
    let (server, client) = sync_pair();
    let copy = dup(me(), client, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    let (c, s, cp) = (hv(client), hv(server), hv(copy));
    // The pending synchronous read.
    let reader = timed("ReadFile(sync)", move || {
        let mut buf = [0u8; 64];
        let mut n = 0u32;
        let ok = unsafe { ReadFile(hp(c), buf.as_mut_ptr(), 64, &mut n, null_mut()) };
        format!("ok={ok} n={n} err={}", if ok == 0 { le() } else { 0 })
    });
    std::thread::sleep(Duration::from_millis(300));
    let mut waits = Vec::new();
    for (which, hval) in [("same", c), ("dup", cp)] {
        let q: Vec<(&str, Box<dyn FnOnce() -> String + Send>)> = vec![
            (
                "GetFileType",
                Box::new(move || format!("{}", unsafe { GetFileType(hp(hval)) })),
            ),
            ("GetNamedPipeInfo", Box::new(move || pipe_info(hp(hval)))),
            (
                "GetNamedPipeHandleStateW",
                Box::new(move || {
                    let mut st = 0u32;
                    let ok = unsafe {
                        GetNamedPipeHandleStateW(
                            hp(hval),
                            &mut st,
                            null_mut(),
                            null_mut(),
                            null_mut(),
                            null_mut(),
                            0,
                        )
                    };
                    format!("ok={ok} state={st:#x}")
                }),
            ),
            (
                "GetFileInformationByHandleEx(FileNameInfo)",
                Box::new(move || file_name(hp(hval))),
            ),
            (
                "NtQueryObject(basic)",
                Box::new(move || format!("{:?}", counts(hp(hval)))),
            ),
            (
                "NtQueryObject(name)",
                Box::new(move || object_name(hp(hval))),
            ),
            (
                "PeekNamedPipe",
                Box::new(move || {
                    let mut avail = 0u32;
                    let ok = unsafe {
                        PeekNamedPipe(hp(hval), null_mut(), 0, null_mut(), &mut avail, null_mut())
                    };
                    format!("ok={ok} avail={avail}")
                }),
            ),
            (
                "DuplicateHandle",
                Box::new(move || {
                    format!(
                        "{:?}",
                        dup(me(), hp(hval), me(), 0, DUPLICATE_SAME_ACCESS).map(hv)
                    )
                }),
            ),
            (
                "CompareObjectHandles",
                Box::new(move || format!("{}", unsafe { CompareObjectHandles(hp(hval), hp(c)) })),
            ),
        ];
        for (name, f) in q {
            let rx = timed(&format!("{which} {name}"), f);
            match rx.recv_timeout(Duration::from_millis(2000)) {
                Ok((r, ms)) => say("W8", format!("{r} ({ms} ms)")),
                Err(_) => {
                    say("W8", format!("{which} {name}: STILL WAITING after 2000 ms"));
                    waits.push(rx);
                }
            }
        }
    }
    // Release the read by writing one message, then see what the waiting
    // queries do.
    let wrote = write_msg(hp(s), vec![7u8; 8]);
    say("W8", format!("server write to release the read: {wrote:?}"));
    match reader.recv_timeout(Duration::from_millis(3000)) {
        Ok((r, ms)) => say("W8", format!("{r} returned at {ms} ms")),
        Err(_) => say("W8", "the read still pending".into()),
    }
    for rx in waits {
        match rx.recv_timeout(Duration::from_millis(3000)) {
            Ok((r, ms)) => say("W8", format!("after release: {r} (returned at {ms} ms)")),
            Err(_) => say("W8", "after release: still waiting".into()),
        }
    }
    // A second pending read, then CancelIoEx from another thread.
    let reader2 = timed("ReadFile(sync) #2", move || {
        let mut buf = [0u8; 64];
        let mut n = 0u32;
        let ok = unsafe { ReadFile(hp(c), buf.as_mut_ptr(), 64, &mut n, null_mut()) };
        format!("ok={ok} n={n} err={}", if ok == 0 { le() } else { 0 })
    });
    std::thread::sleep(Duration::from_millis(300));
    let ok = unsafe { windows_sys::Win32::System::IO::CancelIoEx(hp(cp), null()) };
    say(
        "W8",
        format!(
            "CancelIoEx on the duplicate: ok={ok} err={}",
            if ok == 0 { le() } else { 0 }
        ),
    );
    match reader2.recv_timeout(Duration::from_millis(2000)) {
        Ok((r, ms)) => say("W8", format!("{r} returned at {ms} ms")),
        Err(_) => say("W8", "read #2 still pending after CancelIoEx".into()),
    }
}

/// FileModeInfo on a synchronous end with and without a read pending.
fn mode_probe() {
    let (_server, client) = sync_pair();
    let c = hv(client);
    say(
        "W8",
        format!("FileModeInfo, sync client end, idle: {}", mode_info(c)),
    );
    let _reader = timed("read", move || {
        let mut buf = [0u8; 64];
        let mut n = 0u32;
        unsafe { ReadFile(hp(c), buf.as_mut_ptr(), 64, &mut n, null_mut()) };
        String::new()
    });
    std::thread::sleep(Duration::from_millis(300));
    say(
        "W8",
        format!(
            "FileModeInfo, sync client end, read pending: {}",
            mode_info(c)
        ),
    );
}

fn state(h: H) -> String {
    let mut st = 0u32;
    let ok = unsafe {
        GetNamedPipeHandleStateW(
            h,
            &mut st,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
        )
    };
    if ok == 0 {
        format!("err={}", le())
    } else {
        let mode = if st & PIPE_READMODE_MESSAGE != 0 {
            "message"
        } else {
            "byte"
        };
        let wait = if st & PIPE_NOWAIT != 0 {
            "nowait"
        } else {
            "wait"
        };
        format!("{mode}/{wait} ({st:#x})")
    }
}

fn set(h: H, m: u32) -> String {
    let ok = unsafe { SetNamedPipeHandleState(h, &m, null(), null()) };
    if ok == 0 {
        format!("err={}", le())
    } else {
        "ok".into()
    }
}

fn w9() {
    let (server, client, _) = make_pair(None, 65536, 65536).unwrap();
    say(
        "W9",
        format!("start: server {} client {}", state(server), state(client)),
    );
    say(
        "W9",
        format!(
            "set server byte+nowait: {}",
            set(server, PIPE_READMODE_BYTE | PIPE_NOWAIT)
        ),
    );
    say(
        "W9",
        format!("now: server {} client {}", state(server), state(client)),
    );
    say(
        "W9",
        format!(
            "reset server: {}",
            set(server, PIPE_READMODE_MESSAGE | PIPE_WAIT)
        ),
    );
    say(
        "W9",
        format!("set client byte: {}", set(client, PIPE_READMODE_BYTE)),
    );
    say(
        "W9",
        format!("now: server {} client {}", state(server), state(client)),
    );
    say(
        "W9",
        format!("reset client: {}", set(client, PIPE_READMODE_MESSAGE)),
    );
    // A same-access duplicate shares the file object.
    let c2 = dup(me(), client, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    say(
        "W9",
        format!(
            "set the client's same-access duplicate byte: {}",
            set(c2, PIPE_READMODE_BYTE)
        ),
    );
    say(
        "W9",
        format!("now: client {} duplicate {}", state(client), state(c2)),
    );
    say(
        "W9",
        format!("reset via duplicate: {}", set(c2, PIPE_READMODE_MESSAGE)),
    );
    // A duplicate with the transport's move mask.
    let mv = FILE_READ_DATA | FILE_WRITE_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
    let c3 = dup(me(), client, me(), mv, 0).unwrap();
    say(
        "W9",
        format!(
            "move-mask duplicate: state {} set byte: {}",
            state(c3),
            set(c3, PIPE_READMODE_BYTE)
        ),
    );
    let s3 = dup(me(), server, me(), mv, 0).unwrap();
    say(
        "W9",
        format!(
            "move-mask server duplicate: state {} set nowait: {}",
            state(s3),
            set(s3, PIPE_READMODE_MESSAGE | PIPE_NOWAIT)
        ),
    );
    // What a byte-mode server end reads after two messages.
    say(
        "W9",
        format!("set server byte: {}", set(server, PIPE_READMODE_BYTE)),
    );
    let _ = write_msg(client, vec![1u8; 10]);
    let _ = write_msg(client, vec![2u8; 10]);
    say(
        "W9",
        format!(
            "byte-mode read of two 10-byte messages: {}",
            fmt_read(&read_msg(server, 64, 1000))
        ),
    );
    say(
        "W9",
        format!(
            "set server message+nowait: {}",
            set(server, PIPE_READMODE_MESSAGE | PIPE_NOWAIT)
        ),
    );
    let r = read_msg(server, 64, 1000);
    say(
        "W9",
        format!("nowait overlapped read on an empty pipe: {}", fmt_read(&r)),
    );
}

/// File mode flags (GetFileInformationByHandleEx FileModeInfo), with a
/// 2 s limit.
fn mode_info(hval: isize) -> String {
    let rx = timed("FileModeInformation", move || {
        // NtQueryInformationFile(FileModeInformation = 16): FILE_MODE_INFORMATION
        // is one ULONG; 0x10/0x20 mark a synchronous file object.
        type Nqif =
            unsafe extern "system" fn(H, *mut [u64; 2], *mut core::ffi::c_void, u32, i32) -> i32;
        let f: Nqif = unsafe {
            let m = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
                w("ntdll.dll").as_ptr(),
            );
            let p = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                m,
                c"NtQueryInformationFile".as_ptr().cast(),
            )
            .expect("NtQueryInformationFile");
            std::mem::transmute::<_, Nqif>(p)
        };
        let mut iosb = [0u64; 2];
        let mut m = 0u32;
        let st = unsafe { f(hp(hval), &mut iosb, (&raw mut m).cast(), 4, 16) };
        format!("status={st:#x} mode={m:#x}")
    });
    match rx.recv_timeout(Duration::from_millis(2000)) {
        Ok((r, ms)) => format!("{r} ({ms} ms)"),
        Err(_) => "FileModeInformation: STILL WAITING after 2000 ms".into(),
    }
}

/// W8b child: a synchronous read on the inherited value.
fn w8_child(v: &str) {
    let h = hp(v.parse().unwrap());
    let mut buf = [0u8; 64];
    let mut n = 0u32;
    let t0 = Instant::now();
    let ok = unsafe { ReadFile(h, buf.as_mut_ptr(), 64, &mut n, null_mut()) };
    println!(
        "[W8b child] ReadFile ok={ok} n={n} err={} after {} ms",
        if ok == 0 { le() } else { 0 },
        t0.elapsed().as_millis()
    );
}

/// W8b: another process has a synchronous read pending on its handle
/// when the broker pulls that handle with DUPLICATE_CLOSE_SOURCE.
fn w8b() {
    use std::os::windows::io::AsRawHandle;
    let (server, client) = sync_pair();
    let inh = dup(me(), client, me(), 0, DUPLICATE_SAME_ACCESS).unwrap();
    unsafe { SetHandleInformation(inh, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
    unsafe { CloseHandle(client) };
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["w8child", &hv(inh).to_string()])
        .spawn()
        .unwrap();
    unsafe { CloseHandle(inh) };
    std::thread::sleep(Duration::from_millis(500));
    say(
        "W8b",
        format!(
            "sync file object mode, server end (overlapped): {}",
            mode_info(hv(server))
        ),
    );
    let cp = hp(child.as_raw_handle() as isize);
    let pulled = dup(
        cp,
        inh,
        me(),
        0,
        DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE,
    );
    say(
        "W8b",
        format!(
            "pull with close-source while the child's read is pending: {:?}",
            pulled.map(hv)
        ),
    );
    let Ok(pulled) = pulled else { return };
    let pv = hv(pulled);
    say(
        "W8b",
        format!(
            "NtQueryObject(basic) on the pulled end: {:?}",
            counts(pulled)
        ),
    );
    say(
        "W8b",
        format!("GetFileType: {}", unsafe { GetFileType(pulled) }),
    );
    say(
        "W8b",
        format!("FileModeInfo on the pulled end: {}", mode_info(pv)),
    );
    let rx = timed("GetNamedPipeInfo(pulled)", move || pipe_info(hp(pv)));
    match rx.recv_timeout(Duration::from_millis(2000)) {
        Ok((r, ms)) => say("W8b", format!("{r} ({ms} ms)")),
        Err(_) => say(
            "W8b",
            "GetNamedPipeInfo(pulled): STILL WAITING after 2000 ms".into(),
        ),
    }
    let ok = unsafe { windows_sys::Win32::System::IO::CancelIoEx(pulled, null()) };
    say(
        "W8b",
        format!(
            "broker CancelIoEx on the pulled end: ok={ok} err={}",
            if ok == 0 { le() } else { 0 }
        ),
    );
    match rx.recv_timeout(Duration::from_millis(2000)) {
        Ok((r, ms)) => say(
            "W8b",
            format!("after CancelIoEx: {r} (returned at {ms} ms)"),
        ),
        Err(_) => say(
            "W8b",
            "after CancelIoEx: GetNamedPipeInfo still waiting".into(),
        ),
    }
    let _ = write_msg(server, vec![9u8; 4]);
    say("W8b", "wrote one message from the server end".into());
    match rx.recv_timeout(Duration::from_millis(3000)) {
        Ok((r, ms)) => say("W8b", format!("after the write: {r} (returned at {ms} ms)")),
        Err(_) => say(
            "W8b",
            "after the write: GetNamedPipeInfo still waiting".into(),
        ),
    }
    let _ = child.kill();
    let _ = child.wait();
    let rx2 = timed(
        "GetNamedPipeInfo(pulled) after the child ended",
        move || pipe_info(hp(pv)),
    );
    match rx2.recv_timeout(Duration::from_millis(2000)) {
        Ok((r, ms)) => say("W8b", format!("{r} ({ms} ms)")),
        Err(_) => say("W8b", "after the child ended: still waiting".into()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("w8child") {
        w8_child(&args[1]);
        return;
    }
    if args.iter().any(|a| a == "w8b") {
        w8b();
        std::process::exit(0);
    }
    let all = args.is_empty();
    if all || args.iter().any(|a| a == "w8") {
        w8();
        mode_probe();
    }
    if all || args.iter().any(|a| a == "w9") {
        w9();
    }
    std::process::exit(0);
}
