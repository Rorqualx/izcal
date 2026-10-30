//! M1 phase E, round 4: the full Linux tier (E-Q1), measured once the
//! owner's AppArmor profile `izcal-spawner` grants `userns` to the
//! spawner's path. Written from man pages only: clone(2),
//! user_namespaces(7), mount(2), mount_namespaces(7), pivot_root(2),
//! umount(2), execveat(2), stat(2), proc(5).
//!
//! The profile attaches by path, so this probe is copied to the spawner's
//! path for the run (see the README beside the profile) and started from
//! there: `hg-spawner-main <child-exe> [bind]`.
//!
//! The parent opens `<child-exe>` O_PATH before the clone (as the spawner
//! does), clones with new user, PID, network and mount namespaces, writes
//! the child's id maps (its own uid and gid only), and the child:
//! 1. makes the mount tree private (`MS_REC | MS_PRIVATE` on `/`);
//! 2. mounts a tmpfs on `/mnt`, makes `/mnt/old`, `pivot_root(/mnt,
//!    /mnt/old)`, `chdir("/")`, detaches `/old` and removes it: an empty
//!    root;
//! 3. without `bind`: `execveat(exe_fd, "", AT_EMPTY_PATH)` across it;
//!    with `bind`: before the pivot, bind-mounts the executable onto
//!    `/mnt/exe`, then after the pivot execs `/exe` by path, having
//!    checked that its device and inode equal the pre-opened descriptor's.
//! Each step prints `step=<name> rc=<rc> errno=<e>`; the exec'd child
//! (any static program) runs in the empty root.
#![allow(unsafe_code, clippy::all)]

use std::ffi::CString;

const CLONE_PIDFD: u64 = 0x1000;
const CLONE_NEWNS: u64 = 0x0002_0000;
const CLONE_NEWUSER: u64 = 0x1000_0000;
const CLONE_NEWPID: u64 = 0x2000_0000;
const CLONE_NEWNET: u64 = 0x4000_0000;

#[repr(C)]
#[derive(Default)]
struct CloneArgs {
    flags: u64,
    pidfd: u64,
    child_tid: u64,
    parent_tid: u64,
    exit_signal: u64,
    stack: u64,
    stack_size: u64,
    tls: u64,
}

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
}

fn step(name: &str, rc: i64) -> bool {
    let e = if rc < 0 { errno() } else { 0 };
    println!("step={name} rc={rc} errno={e}");
    rc >= 0
}

fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn write_file(path: &str, data: &str) -> i64 {
    let p = c(path);
    unsafe {
        let fd = libc::open(p.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC);
        if fd < 0 {
            return -1;
        }
        let rc = libc::write(fd, data.as_ptr().cast(), data.len());
        libc::close(fd);
        rc as i64
    }
}

fn dev_ino_fd(fd: i32) -> (u64, u64) {
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    unsafe { libc::fstat(fd, &mut st) };
    (st.st_dev as u64, st.st_ino as u64)
}

fn dev_ino_path(p: &str) -> Option<(u64, u64)> {
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let cp = c(p);
    (unsafe { libc::stat(cp.as_ptr(), &mut st) } == 0).then(|| (st.st_dev as u64, st.st_ino as u64))
}

fn child(exe_fd: i32, exe_path: &str, bind: bool, sync_r: i32) -> ! {
    // Wait until the parent has written the id maps.
    let mut b = [0u8; 1];
    unsafe { libc::read(sync_r, b.as_mut_ptr().cast(), 1) };
    let (u, g) = unsafe { (libc::getuid(), libc::getgid()) };
    println!("child pid={} uid={u} gid={g}", unsafe { libc::getpid() });
    let none = std::ptr::null::<libc::c_char>();
    let ok = step("private", unsafe {
        libc::mount(
            none,
            c("/").as_ptr(),
            none,
            libc::MS_REC | libc::MS_PRIVATE,
            std::ptr::null(),
        )
    } as i64)
        && step("tmpfs", unsafe {
            libc::mount(
                c("tmpfs").as_ptr(),
                c("/mnt").as_ptr(),
                c("tmpfs").as_ptr(),
                0,
                std::ptr::null(),
            )
        } as i64)
        && step(
            "mkdir-old",
            unsafe { libc::mkdir(c("/mnt/old").as_ptr(), 0o700) } as i64,
        );
    if !ok {
        unsafe { libc::_exit(10) }
    }
    if bind {
        let fdp = unsafe {
            libc::open(
                c("/mnt/exe").as_ptr(),
                libc::O_CREAT | libc::O_WRONLY,
                0o700,
            )
        };
        unsafe { libc::close(fdp) };
        if !step("bind", unsafe {
            libc::mount(
                c(exe_path).as_ptr(),
                c("/mnt/exe").as_ptr(),
                none,
                libc::MS_BIND,
                std::ptr::null(),
            )
        } as i64)
        {
            unsafe { libc::_exit(11) }
        }
    }
    let ok = step("pivot_root", unsafe {
        libc::syscall(
            libc::SYS_pivot_root,
            c("/mnt").as_ptr(),
            c("/mnt/old").as_ptr(),
        )
    }) && step("chdir", unsafe { libc::chdir(c("/").as_ptr()) } as i64)
        && step(
            "umount-old",
            unsafe { libc::umount2(c("/old").as_ptr(), libc::MNT_DETACH) } as i64,
        )
        && step("rmdir-old", unsafe { libc::rmdir(c("/old").as_ptr()) }
            as i64);
    if !ok {
        unsafe { libc::_exit(12) }
    }
    println!("etc-hosts-visible={}", dev_ino_path("/etc/hosts").is_some());
    let name = c("probe-child");
    let argv = [name.as_ptr(), std::ptr::null()];
    let envp = [std::ptr::null::<libc::c_char>()];
    if bind {
        let want = dev_ino_fd(exe_fd);
        let got = dev_ino_path("/exe");
        println!(
            "bind dev/ino fd={want:?} path={got:?} same={}",
            got == Some(want)
        );
        if got != Some(want) {
            unsafe { libc::_exit(13) }
        }
        let rc = unsafe { libc::execve(c("/exe").as_ptr(), argv.as_ptr(), envp.as_ptr()) };
        step("exec-path", rc as i64);
    } else {
        let rc = unsafe {
            libc::syscall(
                libc::SYS_execveat,
                exe_fd,
                c("").as_ptr(),
                argv.as_ptr(),
                envp.as_ptr(),
                libc::AT_EMPTY_PATH,
            )
        };
        step("execveat-fd", rc);
    }
    unsafe { libc::_exit(14) }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exe = args
        .get(1)
        .expect("usage: ftier <child-exe> [bind]")
        .clone();
    let bind = args.get(2).is_some_and(|a| a == "bind");
    let ce = c(&exe);
    let exe_fd = unsafe { libc::open(ce.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    assert!(exe_fd >= 0, "open {exe}");
    let mut sync = [0i32; 2];
    unsafe { libc::pipe2(sync.as_mut_ptr(), libc::O_CLOEXEC) };
    let mut pidfd = -1i32;
    let a = CloneArgs {
        flags: CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET | CLONE_NEWNS | CLONE_PIDFD,
        pidfd: (&mut pidfd as *mut i32) as u64,
        exit_signal: libc::SIGCHLD as u64,
        ..Default::default()
    };
    let pid = unsafe {
        libc::syscall(
            libc::SYS_clone3,
            &raw const a,
            std::mem::size_of::<CloneArgs>(),
        )
    };
    if pid == 0 {
        unsafe { libc::close(sync[1]) };
        child(exe_fd, &exe, bind, sync[0]);
    }
    if !step("clone3", pid) {
        return;
    }
    let (u, g) = unsafe { (libc::getuid(), libc::getgid()) };
    step(
        "setgroups-deny",
        write_file(&format!("/proc/{pid}/setgroups"), "deny"),
    );
    step(
        "uid_map",
        write_file(&format!("/proc/{pid}/uid_map"), &format!("0 {u} 1")),
    );
    step(
        "gid_map",
        write_file(&format!("/proc/{pid}/gid_map"), &format!("0 {g} 1")),
    );
    unsafe { libc::close(sync[1]) };
    let mut status = 0;
    unsafe { libc::waitpid(pid as i32, &mut status, 0) };
    println!(
        "child exited={} code={} signal={}",
        libc::WIFEXITED(status),
        libc::WEXITSTATUS(status),
        if libc::WIFSIGNALED(status) {
            libc::WTERMSIG(status)
        } else {
            0
        }
    );
}
