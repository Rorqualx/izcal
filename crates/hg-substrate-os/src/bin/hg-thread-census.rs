//! `hg-thread-census PID`: a test tool (it ships nowhere) that lists the
//! threads of a process and where each one starts, read from outside by
//! the same user, without elevation. `hg-launch-tool`'s idle-thread test
//! runs it on Windows.
//!
//! Windows puts thread-pool workers into a process on its own: the
//! loader's at start, and one more in every process that registered an
//! ETW provider when any session enables that provider (the worker
//! factory reacting to the enable notification; DiagTrack does this on
//! its own schedule). Those threads start in `ntdll`; the threads a
//! process creates start in its own code. So each line says which.
//!
//! One line per thread: `thread TID ntdll 0xOFFSET` or `thread TID
//! other 0xADDRESS`. A thread that ends while it is read is left out.
//! The system-wide process query reports every start address as 0 to a
//! standard user, so each thread is opened for `THREAD_QUERY_INFORMATION`
//! (the limited right is refused for a sandboxed child's threads).
#![cfg_attr(
    windows,
    expect(
        unsafe_code,
        reason = "G5: src/bin/hg-thread-census.rs (Toolhelp thread list, OpenThread, NtQueryInformationThread)"
    )
)]

use std::process::ExitCode;

#[cfg(windows)]
fn main() -> ExitCode {
    let Some(pid) = std::env::args().nth(1).and_then(|a| a.parse::<u32>().ok()) else {
        eprintln!("usage: hg-thread-census PID");
        return ExitCode::from(2);
    };
    let census = win::ntdll_range().and_then(|nt| Ok((nt.clone(), win::threads(pid)?)));
    match census {
        Ok((nt, threads)) => {
            for (tid, start) in threads {
                if nt.contains(&start) {
                    println!("thread {tid} ntdll {:#x}", start - nt.start);
                } else {
                    println!("thread {tid} other {start:#x}");
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("hg-thread-census: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("hg-thread-census: Windows only");
    ExitCode::from(2)
}

#[cfg(windows)]
mod win {
    use std::io;
    use std::ops::Range;
    use std::ptr::null_mut;

    use windows_sys::Wdk::System::Threading::{
        NtQueryInformationThread, ThreadQuerySetWin32StartAddress,
    };
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_INVALID_PARAMETER, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, OpenThread, THREAD_QUERY_INFORMATION,
    };

    /// A handle closed when dropped.
    struct Owned(HANDLE);

    impl Drop for Owned {
        fn drop(&mut self) {
            // SAFETY: `self.0` is a handle this tool opened and closes once.
            unsafe { CloseHandle(self.0) };
        }
    }

    /// `ntdll`'s image in this process. Windows maps it at one address in
    /// every process of a boot, so the range holds for the target too.
    pub fn ntdll_range() -> io::Result<Range<usize>> {
        let name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        // SAFETY: `name` is NUL-terminated UTF-16 and outlives the call.
        let module = unsafe { GetModuleHandleW(name.as_ptr()) };
        if module.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: takes nothing; the pseudo-handle needs no closing.
        let me = unsafe { GetCurrentProcess() };
        let mut info = MODULEINFO {
            lpBaseOfDll: null_mut(),
            SizeOfImage: 0,
            EntryPoint: null_mut(),
        };
        let size = u32::try_from(size_of::<MODULEINFO>()).map_err(io::Error::other)?;
        // SAFETY: `module` is loaded in this process for its whole life
        // (ntdll is never unloaded); `info` is writable and `size` long.
        let ok = unsafe { GetModuleInformation(me, module, &raw mut info, size) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        let base = info.lpBaseOfDll as usize;
        let len = usize::try_from(info.SizeOfImage).map_err(io::Error::other)?;
        Ok(base..base + len)
    }

    /// Each thread of `pid` and its start address, in Toolhelp's order.
    pub fn threads(pid: u32) -> io::Result<Vec<(u32, usize)>> {
        // SAFETY: plain flags; the result is checked before use.
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snap == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let snap = Owned(snap);
        let mut entry = THREADENTRY32 {
            dwSize: u32::try_from(size_of::<THREADENTRY32>()).map_err(io::Error::other)?,
            ..THREADENTRY32::default()
        };
        let mut tids = Vec::new();
        // SAFETY: `snap.0` is an open snapshot; `entry` is writable with
        // `dwSize` set, as Thread32First requires.
        let mut more = unsafe { Thread32First(snap.0, &raw mut entry) } != 0;
        while more {
            if entry.th32OwnerProcessID == pid {
                tids.push(entry.th32ThreadID);
            }
            // SAFETY: as for Thread32First, on the same snapshot.
            more = unsafe { Thread32Next(snap.0, &raw mut entry) } != 0;
        }
        let mut out = Vec::with_capacity(tids.len());
        for tid in tids {
            if let Some(start) = start_of(tid)? {
                out.push((tid, start));
            }
        }
        Ok(out)
    }

    /// Thread `tid`'s start address, or `None` if it has ended.
    fn start_of(tid: u32) -> io::Result<Option<usize>> {
        // SAFETY: plain values; the result is checked before use.
        let h = unsafe { OpenThread(THREAD_QUERY_INFORMATION, 0, tid) };
        if h.is_null() {
            let e = io::Error::last_os_error();
            // An ended thread's id no longer names a thread.
            return match e.raw_os_error() {
                Some(c) if u32::try_from(c) == Ok(ERROR_INVALID_PARAMETER) => Ok(None),
                _ => Err(e),
            };
        }
        let h = Owned(h);
        let mut start: usize = 0;
        let size = u32::try_from(size_of::<usize>()).map_err(io::Error::other)?;
        // SAFETY: `h.0` is open with THREAD_QUERY_INFORMATION, which this
        // class needs; `start` is a writable pointer-sized value of `size`.
        let st = unsafe {
            NtQueryInformationThread(
                h.0,
                ThreadQuerySetWin32StartAddress,
                (&raw mut start).cast(),
                size,
                null_mut(),
            )
        };
        if st < 0 {
            return Err(io::Error::other(format!(
                "NtQueryInformationThread({tid}): {:#010x}",
                st.cast_unsigned()
            )));
        }
        Ok(Some(start))
    }
}
