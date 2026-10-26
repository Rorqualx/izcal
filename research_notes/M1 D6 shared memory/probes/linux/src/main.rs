//! M1 D6 black-box probes: memfd sealing, identification and passing on
//! Linux. Sources: the man pages memfd_create(2), fcntl(2) "File
//! Sealing", mmap(2), mprotect(2), madvise(2), unix(7), proc(5) only.
//! Each probe prints one `Lnn` line per observation; nothing here is
//! product code.
#![allow(clippy::all)]

use std::ffi::CString;
use std::io::Error;
use std::ptr::null_mut;
use std::time::Instant;

const KB: usize = 1024;

fn errno() -> i32 {
    Error::last_os_error().raw_os_error().unwrap_or(-1)
}

fn rc(name: &str, r: libc::c_long) -> String {
    if r < 0 {
        format!(
            "{name}=-1 errno {} ({})",
            errno(),
            Error::from_raw_os_error(errno())
        )
    } else {
        format!("{name}={r}")
    }
}

fn memfd(name: &str, flags: libc::c_uint) -> i32 {
    let n = CString::new(name).unwrap();
    // SAFETY: probe; a valid C string and flags.
    unsafe { libc::memfd_create(n.as_ptr(), flags) }
}

fn sized(fd: i32, len: usize) {
    // SAFETY: probe.
    let r = unsafe { libc::ftruncate(fd, len as libc::off_t) };
    assert_eq!(r, 0, "ftruncate {}", errno());
}

fn add_seals(fd: i32, seals: i32) -> libc::c_long {
    // SAFETY: probe.
    unsafe { libc::fcntl(fd, libc::F_ADD_SEALS, seals) as libc::c_long }
}

fn get_seals(fd: i32) -> libc::c_long {
    // SAFETY: probe.
    unsafe { libc::fcntl(fd, libc::F_GET_SEALS) as libc::c_long }
}

fn map(fd: i32, len: usize, prot: i32, flags: i32) -> *mut u8 {
    // SAFETY: probe.
    let p = unsafe { libc::mmap(null_mut(), len, prot, flags, fd, 0) };
    if p == libc::MAP_FAILED {
        null_mut()
    } else {
        p.cast()
    }
}

fn map_rc(fd: i32, len: usize, prot: i32, flags: i32) -> String {
    let p = map(fd, len, prot, flags);
    if p.is_null() {
        format!(
            "mmap failed errno {} ({})",
            errno(),
            Error::from_raw_os_error(errno())
        )
    } else {
        // SAFETY: probe; unmapping what we mapped.
        unsafe { libc::munmap(p.cast(), len) };
        "mmap ok".into()
    }
}

fn reopen(fd: i32, flags: i32) -> i32 {
    let p = CString::new(format!("/proc/self/fd/{fd}")).unwrap();
    // SAFETY: probe.
    unsafe { libc::open(p.as_ptr(), flags) }
}

fn seals_str(s: libc::c_long) -> String {
    if s < 0 {
        return format!("F_GET_SEALS=-1 errno {}", errno());
    }
    let mut v = vec![];
    for (b, n) in [
        (libc::F_SEAL_SEAL, "SEAL"),
        (libc::F_SEAL_SHRINK, "SHRINK"),
        (libc::F_SEAL_GROW, "GROW"),
        (libc::F_SEAL_WRITE, "WRITE"),
        (libc::F_SEAL_FUTURE_WRITE, "FUTURE_WRITE"),
        (0x20, "EXEC"),
    ] {
        if s as i32 & b != 0 {
            v.push(n);
        }
    }
    format!("{s:#x} [{}]", v.join("|"))
}

fn ident(label: &str, fd: i32) {
    // SAFETY: probe; zeroed plain-data structs filled by the kernel.
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::fstat(fd, &mut st) };
    let mut fs: libc::statfs = unsafe { std::mem::zeroed() };
    let r2 = unsafe { libc::fstatfs(fd, &mut fs) };
    let fl = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    let link = std::fs::read_link(format!("/proc/self/fd/{fd}"))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|e| format!("readlink err {e}"));
    println!(
        "L5 {label}: fstat={r} mode={:#o} type={} nlink={} size={} dev={:#x} ino={} | fstatfs={r2} f_type={:#x} | F_GETFL acc={} | seals {} | link {link}",
        st.st_mode,
        match st.st_mode & libc::S_IFMT {
            libc::S_IFREG => "REG",
            libc::S_IFSOCK => "SOCK",
            libc::S_IFIFO => "FIFO",
            libc::S_IFDIR => "DIR",
            libc::S_IFCHR => "CHR",
            _ => "other",
        },
        st.st_nlink,
        st.st_size,
        st.st_dev,
        st.st_ino,
        fs.f_type,
        match fl & libc::O_ACCMODE {
            libc::O_RDONLY => "RDONLY",
            libc::O_WRONLY => "WRONLY",
            libc::O_RDWR => "RDWR",
            _ => "?",
        },
        seals_str(get_seals(fd))
    );
}

fn maps_count() -> usize {
    std::fs::read_to_string("/proc/self/maps")
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

fn l1_seal_write_vs_mappings() {
    let fd = memfd("l1", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let w = map(
        fd,
        64 * KB,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
    );
    println!(
        "L1 F_SEAL_WRITE with a writable shared mapping: {}",
        rc("add", add_seals(fd, libc::F_SEAL_WRITE))
    );
    let r = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
    // SAFETY: probe.
    unsafe { libc::munmap(w.cast(), 64 * KB) };
    println!(
        "L1 ... after unmapping it (a read-only shared mapping still exists): {}",
        rc("add", add_seals(fd, libc::F_SEAL_WRITE))
    );
    let p = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_PRIVATE);
    // SAFETY: probe.
    let mp = unsafe { libc::mprotect(r.cast(), 64 * KB, libc::PROT_READ | libc::PROT_WRITE) }
        as libc::c_long;
    println!(
        "L1 WRITE-sealed: mprotect(shared RO map -> RW): {}",
        rc("mprotect", mp)
    );
    println!(
        "L1 WRITE-sealed: new shared RW mmap: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED
        )
    );
    println!(
        "L1 WRITE-sealed: new private RW mmap: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE
        )
    );
    // SAFETY: probe.
    let mp2 = unsafe { libc::mprotect(p.cast(), 64 * KB, libc::PROT_READ | libc::PROT_WRITE) }
        as libc::c_long;
    println!(
        "L1 WRITE-sealed: mprotect(private RO map -> RW): {}",
        rc("mprotect", mp2)
    );
    let buf = [7u8; 16];
    // SAFETY: probe.
    let wr = unsafe { libc::pwrite(fd, buf.as_ptr().cast(), 16, 0) } as libc::c_long;
    println!("L1 WRITE-sealed: pwrite: {}", rc("pwrite", wr));
    // SAFETY: probe.
    let fa = unsafe {
        libc::fallocate(
            fd,
            libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
            0,
            4096,
        )
    } as libc::c_long;
    println!(
        "L1 WRITE-sealed: fallocate(PUNCH_HOLE): {}",
        rc("fallocate", fa)
    );
    // SAFETY: probe.
    let mr = unsafe { libc::madvise(r.cast(), 4096, libc::MADV_REMOVE) } as libc::c_long;
    println!(
        "L1 WRITE-sealed: madvise(MADV_REMOVE) on the RO shared map: {}",
        rc("madvise", mr)
    );
    unsafe {
        libc::munmap(r.cast(), 64 * KB);
        libc::munmap(p.cast(), 64 * KB);
        libc::close(fd);
    }
}

fn l2_future_write() {
    let fd = memfd("l2", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let w = map(
        fd,
        64 * KB,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
    );
    let all =
        libc::F_SEAL_FUTURE_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    println!(
        "L2 FUTURE_WRITE|GROW|SHRINK|SEAL with a writable shared mapping: {} -> seals {}",
        rc("add", add_seals(fd, all)),
        seals_str(get_seals(fd))
    );
    // SAFETY: probe; w maps 64 KiB.
    unsafe { *w = 0x5a };
    let r = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
    // SAFETY: probe.
    println!(
        "L2 producer's earlier RW mapping still writes; a new RO mapping reads {:#x}",
        unsafe { *r }
    );
    unsafe { *w = 0x5b };
    println!(
        "L2 ... a later write is visible in the RO mapping: {:#x}",
        unsafe { *r }
    );
    println!(
        "L2 new shared RW mmap: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED
        )
    );
    println!(
        "L2 new private RW mmap: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE
        )
    );
    // SAFETY: probe.
    let mp = unsafe { libc::mprotect(r.cast(), 64 * KB, libc::PROT_READ | libc::PROT_WRITE) }
        as libc::c_long;
    println!("L2 mprotect(shared RO map -> RW): {}", rc("mprotect", mp));
    let buf = [7u8; 16];
    let wr = unsafe { libc::pwrite(fd, buf.as_ptr().cast(), 16, 0) } as libc::c_long;
    println!("L2 pwrite: {}", rc("pwrite", wr));
    let fa = unsafe {
        libc::fallocate(
            fd,
            libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
            0,
            4096,
        )
    } as libc::c_long;
    println!("L2 fallocate(PUNCH_HOLE): {}", rc("fallocate", fa));
    let mr = unsafe { libc::madvise(r.cast(), 4096, libc::MADV_REMOVE) } as libc::c_long;
    println!(
        "L2 madvise(MADV_REMOVE) on the RO shared map: {} (byte now {:#x})",
        rc("madvise", mr),
        unsafe { *r }
    );
    let ft = unsafe { libc::ftruncate(fd, 0) } as libc::c_long;
    println!("L2 ftruncate shrink: {}", rc("ftruncate", ft));
    let ft2 = unsafe { libc::ftruncate(fd, 128 * KB as libc::off_t) } as libc::c_long;
    println!("L2 ftruncate grow: {}", rc("ftruncate", ft2));
    println!(
        "L2 add WRITE after SEAL: {}",
        rc("add", add_seals(fd, libc::F_SEAL_WRITE))
    );
    // The holder of a read-only descriptor: reopen through /proc.
    let ro = reopen(fd, libc::O_RDONLY | libc::O_CLOEXEC);
    let rw = reopen(fd, libc::O_RDWR | libc::O_CLOEXEC);
    println!(
        "L2 reopen /proc/self/fd O_RDONLY -> {ro}, O_RDWR -> {rw} (errno {})",
        errno()
    );
    if rw >= 0 {
        println!(
            "L2 via the reopened O_RDWR fd: new shared RW mmap: {}",
            map_rc(
                rw,
                64 * KB,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED
            )
        );
        unsafe { libc::close(rw) };
    }
    if ro >= 0 {
        let fl = unsafe { libc::fcntl(ro, libc::F_SETFL, libc::O_RDWR) };
        let acc = unsafe { libc::fcntl(ro, libc::F_GETFL) } & libc::O_ACCMODE;
        println!(
            "L2 F_SETFL(O_RDWR) on an O_RDONLY fd: rc {fl}, access mode after: {}",
            if acc == libc::O_RDONLY {
                "RDONLY"
            } else {
                "changed"
            }
        );
        unsafe { libc::close(ro) };
    }
    // mprotect PROT_EXEC on the read-only shared mapping.
    let mx = unsafe { libc::mprotect(r.cast(), 64 * KB, libc::PROT_READ | libc::PROT_EXEC) }
        as libc::c_long;
    println!("L2 mprotect(RO shared -> R|X): {}", rc("mprotect", mx));
    unsafe {
        libc::munmap(r.cast(), 64 * KB);
        libc::munmap(w.cast(), 64 * KB);
        libc::close(fd);
    }
    // Without F_SEAL_SEAL, a holder can add seals of its own.
    let fd = memfd("l2b", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let w = map(
        fd,
        64 * KB,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
    );
    add_seals(
        fd,
        libc::F_SEAL_FUTURE_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK,
    );
    let ro = reopen(fd, libc::O_RDONLY | libc::O_CLOEXEC);
    println!(
        "L2 no SEAL: holder's O_RDONLY fd adds SEAL: {}",
        rc("add", add_seals(ro, libc::F_SEAL_SEAL))
    );
    let rw = reopen(fd, libc::O_RDWR | libc::O_CLOEXEC);
    println!(
        "L2 no SEAL: holder's reopened O_RDWR fd adds EXEC|SEAL: {} -> {}",
        rc("add", add_seals(rw, 0x20 | libc::F_SEAL_SEAL)),
        seals_str(get_seals(fd))
    );
    unsafe {
        libc::munmap(w.cast(), 64 * KB);
        libc::close(ro);
        libc::close(rw);
        libc::close(fd);
    }
}

fn l3_unseal_able_kinds() {
    let a = memfd("l3-noseal", libc::MFD_CLOEXEC);
    ident("memfd without ALLOW_SEALING", a);
    let b = memfd("l3-seal", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(b, 64 * KB);
    ident("memfd ALLOW_SEALING, unsealed", b);
    add_seals(
        b,
        libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL,
    );
    ident("memfd sealed WRITE|GROW|SHRINK|SEAL", b);
    let ro = reopen(b, libc::O_RDONLY | libc::O_CLOEXEC);
    ident("same memfd, reopened O_RDONLY", ro);
    let c = memfd(
        "l3-noexec",
        libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC | 0x8, /* MFD_NOEXEC_SEAL */
    );
    if c >= 0 {
        ident("memfd MFD_NOEXEC_SEAL", c)
    } else {
        println!("L5 memfd MFD_NOEXEC_SEAL: errno {}", errno())
    }
    let d = memfd(
        "l3-huge",
        libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC | libc::MFD_HUGETLB,
    );
    if d >= 0 {
        ident("memfd MFD_HUGETLB", d)
    } else {
        println!("L5 memfd MFD_HUGETLB: errno {}", errno())
    }
    let shm_name = CString::new("/izcal-d6-probe").unwrap();
    let e = unsafe {
        libc::shm_open(
            shm_name.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )
    };
    if e >= 0 {
        sized(e, 64 * KB);
        ident("shm_open file in /dev/shm", e);
        unsafe { libc::shm_unlink(shm_name.as_ptr()) };
        ident("shm_open file after shm_unlink", e);
    }
    let tmp = CString::new("/tmp/izcal-d6-probe.bin").unwrap();
    let f = unsafe {
        libc::open(
            tmp.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC,
            0o600,
        )
    };
    ident("regular file in /tmp", f);
    unsafe { libc::unlink(tmp.as_ptr()) };
    let g = unsafe { libc::open(c"/dev/shm".as_ptr(), libc::O_TMPFILE | libc::O_RDWR, 0o600) };
    if g >= 0 {
        ident("O_TMPFILE on /dev/shm (tmpfs)", g)
    } else {
        println!("L5 O_TMPFILE /dev/shm: errno {}", errno())
    }
    let mut sv = [0i32; 2];
    unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0, sv.as_mut_ptr()) };
    ident("seqpacket socket", sv[0]);
    let h = unsafe { libc::open(c"/proc/self/fd".as_ptr(), libc::O_PATH) };
    let op = reopen(b, libc::O_PATH | libc::O_CLOEXEC);
    ident("O_PATH fd of the sealed memfd", op);
    println!(
        "L5 O_PATH fd of the sealed memfd: mmap RO: {}",
        map_rc(op, 64 * KB, libc::PROT_READ, libc::MAP_SHARED)
    );
    for x in [a, b, ro, c, d, e, f, g, sv[0], sv[1], h, op] {
        if x >= 0 {
            unsafe { libc::close(x) };
        }
    }
}

/// A memfd crosses SCM_RIGHTS from a forked child (the producer) to the
/// parent (the consumer); the consumer checks what it got and maps it.
fn l7_pass_between_processes() {
    let mut sv = [0i32; 2];
    unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            sv.as_mut_ptr(),
        )
    };
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        let fd = memfd("l7", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
        sized(fd, 64 * KB);
        let w = map(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
        );
        unsafe { *w = 1 };
        add_seals(
            fd,
            libc::F_SEAL_FUTURE_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL,
        );
        let ro = reopen(fd, libc::O_RDONLY | libc::O_CLOEXEC);
        send_fd(sv[1], ro);
        let mut b = [0u8; 1];
        unsafe { libc::read(sv[1], b.as_mut_ptr().cast(), 1) };
        unsafe { *w = 2 };
        unsafe { libc::write(sv[1], b"w".as_ptr().cast(), 1) };
        unsafe { libc::read(sv[1], b.as_mut_ptr().cast(), 1) };
        unsafe { libc::_exit(0) };
    }
    let fd = recv_fd(sv[0]);
    ident("received over SCM_RIGHTS", fd);
    let r = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
    println!("L7 consumer reads {:#x}", unsafe { *r });
    unsafe { libc::write(sv[0], b"g".as_ptr().cast(), 1) };
    let mut b = [0u8; 1];
    unsafe { libc::read(sv[0], b.as_mut_ptr().cast(), 1) };
    println!(
        "L7 after the producer's later write the consumer reads {:#x}",
        unsafe { *r }
    );
    println!(
        "L7 consumer: shared RW mmap {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED
        )
    );
    let rw = reopen(fd, libc::O_RDWR | libc::O_CLOEXEC);
    println!(
        "L7 consumer: reopen O_RDWR -> {rw}; shared RW mmap through it: {}",
        if rw >= 0 {
            map_rc(
                rw,
                64 * KB,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
            )
        } else {
            format!("errno {}", errno())
        }
    );
    // Producer exits; the mapping stays.
    unsafe { libc::write(sv[0], b"x".as_ptr().cast(), 1) };
    let mut st = 0;
    unsafe { libc::waitpid(pid, &mut st, 0) };
    println!(
        "L7 producer exited (status {st}); consumer still reads {:#x}",
        unsafe { *r }
    );
    unsafe {
        libc::munmap(r.cast(), 64 * KB);
        libc::close(fd);
        if rw >= 0 {
            libc::close(rw);
        }
    }
}

/// Without F_SEAL_SHRINK the producer can truncate under the consumer.
fn l8_shrink_without_seal() {
    let fd = memfd("l8", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        let r = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
        sized(fd, 0);
        let v = unsafe { std::ptr::read_volatile(r.add(8 * KB)) };
        println!("L8 read after shrink returned {v}");
        unsafe { libc::_exit(0) };
    }
    let mut st = 0;
    unsafe { libc::waitpid(pid, &mut st, 0) };
    println!(
        "L8 consumer reading past a shrink (no SHRINK seal): signalled={} sig={}",
        libc::WIFSIGNALED(st),
        if libc::WIFSIGNALED(st) {
            libc::WTERMSIG(st)
        } else {
            0
        }
    );
    unsafe { libc::close(fd) };
}

fn send_fd(sock: i32, fd: i32) {
    let mut cmsg = [0u8; 64];
    let mut data = *b"f";
    let mut iov = libc::iovec {
        iov_base: data.as_mut_ptr().cast(),
        iov_len: 1,
    };
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg.as_mut_ptr().cast();
    msg.msg_controllen = unsafe { libc::CMSG_SPACE(4) } as _;
    unsafe {
        let c = libc::CMSG_FIRSTHDR(&msg);
        (*c).cmsg_level = libc::SOL_SOCKET;
        (*c).cmsg_type = libc::SCM_RIGHTS;
        (*c).cmsg_len = libc::CMSG_LEN(4) as _;
        std::ptr::write_unaligned(libc::CMSG_DATA(c).cast::<i32>(), fd);
        libc::sendmsg(sock, &msg, 0);
    }
}

fn recv_fd(sock: i32) -> i32 {
    let mut cmsg = [0u8; 64];
    let mut data = [0u8; 1];
    let mut iov = libc::iovec {
        iov_base: data.as_mut_ptr().cast(),
        iov_len: 1,
    };
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg.as_mut_ptr().cast();
    msg.msg_controllen = cmsg.len() as _;
    unsafe {
        libc::recvmsg(sock, &mut msg, libc::MSG_CMSG_CLOEXEC);
        let c = libc::CMSG_FIRSTHDR(&msg);
        std::ptr::read_unaligned(libc::CMSG_DATA(c).cast::<i32>())
    }
}

fn l10_costs() {
    for len in [64 * KB, 4 * KB * KB] {
        let n = 2000;
        let t = Instant::now();
        for _ in 0..n {
            let fd = memfd("l10", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
            sized(fd, len);
            add_seals(
                fd,
                libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL,
            );
            unsafe { libc::close(fd) };
        }
        let create = t.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        let fd = memfd("l10", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
        sized(fd, len);
        let t = Instant::now();
        for _ in 0..n {
            let p = map(fd, len, libc::PROT_READ, libc::MAP_SHARED);
            unsafe { libc::munmap(p.cast(), len) };
        }
        let mapc = t.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        let pages = len / 4096;
        let t = Instant::now();
        for _ in 0..(n / 10) {
            let p = map(fd, len, libc::PROT_READ, libc::MAP_SHARED);
            let mut s = 0u64;
            for i in 0..pages {
                s = s.wrapping_add(unsafe { std::ptr::read_volatile(p.add(i * 4096)) } as u64);
            }
            std::hint::black_box(s);
            unsafe { libc::munmap(p.cast(), len) };
        }
        let touched = t.elapsed().as_nanos() as f64 / (n / 10) as f64 / 1000.0;
        let before = maps_count();
        let p = map(fd, len, libc::PROT_READ, libc::MAP_SHARED);
        let after = maps_count();
        unsafe { libc::munmap(p.cast(), len) };
        println!(
            "L10 {} KiB: create+size+seal+close {create:.2} µs; map+unmap {mapc:.2} µs; map+read every page+unmap {touched:.2} µs; /proc/self/maps lines +{}",
            len / KB,
            after - before
        );
        unsafe { libc::close(fd) };
    }
}

fn l1b_what_blocks_seal_write() {
    // (a) A writable shared mapping, unmapped before sealing.
    let fd = memfd("l1b", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let w = map(
        fd,
        64 * KB,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
    );
    unsafe {
        *w = 9;
        libc::munmap(w.cast(), 64 * KB)
    };
    println!(
        "L1b (a) RW shared map, unmapped, no other mapping: {}",
        rc("add WRITE", add_seals(fd, libc::F_SEAL_WRITE))
    );
    unsafe { libc::close(fd) };
    // (b) Only a PROT_READ shared mapping, made through the O_RDWR fd.
    let fd = memfd("l1b", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let r = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
    println!(
        "L1b (b) only a PROT_READ shared map from the O_RDWR fd: {}",
        rc("add WRITE", add_seals(fd, libc::F_SEAL_WRITE))
    );
    unsafe { libc::munmap(r.cast(), 64 * KB) };
    println!(
        "L1b (b) ... after unmapping it: {}",
        rc("add WRITE", add_seals(fd, libc::F_SEAL_WRITE))
    );
    unsafe { libc::close(fd) };
    // (c) A PROT_READ shared mapping made through an O_RDONLY reopen.
    let fd = memfd("l1b", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let ro = reopen(fd, libc::O_RDONLY | libc::O_CLOEXEC);
    let r = map(ro, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
    println!(
        "L1b (c) only a PROT_READ shared map from an O_RDONLY fd: {}",
        rc("add WRITE", add_seals(fd, libc::F_SEAL_WRITE))
    );
    unsafe {
        libc::munmap(r.cast(), 64 * KB);
        libc::close(ro);
        libc::close(fd)
    };
    // (d) A MAP_PRIVATE RW mapping.
    let fd = memfd("l1b", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    let p = map(
        fd,
        64 * KB,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE,
    );
    println!(
        "L1b (d) only a private RW map: {}",
        rc("add WRITE", add_seals(fd, libc::F_SEAL_WRITE))
    );
    unsafe {
        libc::munmap(p.cast(), 64 * KB);
        libc::close(fd)
    };
    // (e) Fill by pwrite, seal everything, then try every way to write.
    let fd = memfd("l1b", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC);
    sized(fd, 64 * KB);
    unsafe { libc::pwrite(fd, b"filled".as_ptr().cast(), 6, 0) };
    let all = libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    println!(
        "L1b (e) WRITE|GROW|SHRINK|SEAL with no mapping: {} -> {}",
        rc("add", add_seals(fd, all)),
        seals_str(get_seals(fd))
    );
    let r = map(fd, 64 * KB, libc::PROT_READ, libc::MAP_SHARED);
    println!(
        "L1b (e) sealed: RO shared map {}",
        if r.is_null() {
            format!("failed {}", errno())
        } else {
            "ok".into()
        }
    );
    let mp = unsafe { libc::mprotect(r.cast(), 64 * KB, libc::PROT_READ | libc::PROT_WRITE) }
        as libc::c_long;
    println!(
        "L1b (e) sealed: mprotect(shared RO map -> RW): {}",
        rc("mprotect", mp)
    );
    println!(
        "L1b (e) sealed: new shared RW mmap: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED
        )
    );
    println!(
        "L1b (e) sealed: new private RW mmap: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE
        )
    );
    let wr = unsafe { libc::pwrite(fd, b"x".as_ptr().cast(), 1, 0) } as libc::c_long;
    println!("L1b (e) sealed: pwrite: {}", rc("pwrite", wr));
    let fa = unsafe {
        libc::fallocate(
            fd,
            libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
            0,
            4096,
        )
    } as libc::c_long;
    println!(
        "L1b (e) sealed: fallocate(PUNCH_HOLE): {}",
        rc("fallocate", fa)
    );
    let mr = unsafe { libc::madvise(r.cast(), 4096, libc::MADV_REMOVE) } as libc::c_long;
    println!(
        "L1b (e) sealed: madvise(MADV_REMOVE): {} (first byte now {:#x})",
        rc("madvise", mr),
        unsafe { *r }
    );
    let rw = reopen(fd, libc::O_RDWR | libc::O_CLOEXEC);
    println!(
        "L1b (e) sealed: reopen O_RDWR -> {rw}; shared RW mmap through it: {}",
        if rw >= 0 {
            map_rc(
                rw,
                64 * KB,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
            )
        } else {
            format!("errno {}", errno())
        }
    );
    let mx = unsafe { libc::mprotect(r.cast(), 64 * KB, libc::PROT_READ | libc::PROT_EXEC) }
        as libc::c_long;
    println!(
        "L1b (e) sealed: mprotect(RO shared -> R|X): {}",
        rc("mprotect", mx)
    );
    unsafe {
        libc::munmap(r.cast(), 64 * KB);
        if rw >= 0 {
            libc::close(rw);
        }
        libc::close(fd);
    };
    // (f) MFD_NOEXEC_SEAL: can it be mapped executable?
    let fd = memfd("l1b-nx", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC | 0x8);
    sized(fd, 64 * KB);
    println!(
        "L1b (f) MFD_NOEXEC_SEAL: mmap R|X shared: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_EXEC,
            libc::MAP_SHARED
        )
    );
    unsafe { libc::close(fd) };
}

/// L12: which cgroup is charged for a memfd's pages: the creator's,
/// the mapper's or the toucher's? Run inside a delegated cgroup v2
/// subtree (`l12` mode), with two leaf cgroups A and B.
fn l12_memcg(base: &str) {
    let rd = |cg: &str| {
        std::fs::read_to_string(format!("{base}/{cg}/memory.current"))
            .map(|s| s.trim().parse::<u64>().unwrap_or(0) >> 10)
            .unwrap_or(0)
    };
    let join = |cg: &str| {
        std::fs::write(
            format!("{base}/{cg}/cgroup.procs"),
            format!("{}", std::process::id()),
        )
        .expect("join cgroup")
    };
    join("A");
    let len = 32 << 20;
    let fd = memfd("l12", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC | 0x8);
    sized(fd, len);
    println!(
        "L12 created 32 MiB memfd in A: A={} KiB B={} KiB",
        rd("A"),
        rd("B")
    );
    let mut sv = [0i32; 2];
    unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            sv.as_mut_ptr(),
        )
    };
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        std::fs::write(
            format!("{base}/B/cgroup.procs"),
            format!("{}", std::process::id()),
        )
        .expect("join B");
        let p = map(
            fd,
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
        );
        for i in (0..len).step_by(4096) {
            unsafe { std::ptr::write_volatile(p.add(i), 1) };
        }
        unsafe { libc::write(sv[1], b"t".as_ptr().cast(), 1) };
        let mut b = [0u8; 1];
        unsafe { libc::read(sv[1], b.as_mut_ptr().cast(), 1) };
        unsafe { libc::munmap(p.cast(), len) };
        unsafe { libc::write(sv[1], b"u".as_ptr().cast(), 1) };
        unsafe { libc::read(sv[1], b.as_mut_ptr().cast(), 1) };
        unsafe { libc::_exit(0) };
    }
    let mut b = [0u8; 1];
    unsafe { libc::read(sv[0], b.as_mut_ptr().cast(), 1) };
    println!(
        "L12 a process in B mapped it and wrote every page: A={} KiB B={} KiB",
        rd("A"),
        rd("B")
    );
    let p = map(fd, len, libc::PROT_READ, libc::MAP_SHARED);
    let mut x = 0u64;
    for i in (0..len).step_by(4096) {
        x = x.wrapping_add(unsafe { std::ptr::read_volatile(p.add(i)) } as u64);
    }
    println!(
        "L12 the creator in A then read every page (sum {x}): A={} KiB B={} KiB",
        rd("A"),
        rd("B")
    );
    unsafe { libc::write(sv[0], b"g".as_ptr().cast(), 1) };
    unsafe { libc::read(sv[0], b.as_mut_ptr().cast(), 1) };
    println!("L12 B unmapped: A={} KiB B={} KiB", rd("A"), rd("B"));
    unsafe { libc::write(sv[0], b"x".as_ptr().cast(), 1) };
    let mut st = 0;
    unsafe { libc::waitpid(pid, &mut st, 0) };
    println!(
        "L12 B's process exited: A={} KiB B={} KiB",
        rd("A"),
        rd("B")
    );
    unsafe {
        libc::munmap(p.cast(), len);
        libc::close(fd)
    };
    println!(
        "L12 the creator closed and unmapped: A={} KiB B={} KiB",
        rd("A"),
        rd("B")
    );
}

/// L13: MFD_NOEXEC_SEAL with the D6 seal sets; the raw fstat syscall.
fn l13_noexec_sets() {
    let fd = memfd("l13", libc::MFD_ALLOW_SEALING | libc::MFD_CLOEXEC | 0x8);
    println!(
        "L13 MFD_NOEXEC_SEAL|ALLOW_SEALING: initial seals {}",
        seals_str(get_seals(fd))
    );
    sized(fd, 64 * KB);
    println!(
        "L13 add GROW|SHRINK: {} -> {}",
        rc(
            "add",
            add_seals(fd, libc::F_SEAL_GROW | libc::F_SEAL_SHRINK)
        ),
        seals_str(get_seals(fd))
    );
    let w = map(
        fd,
        64 * KB,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
    );
    println!(
        "L13 writer seals with a live RW map: add WRITE|SEAL: {}",
        rc("add", add_seals(fd, libc::F_SEAL_WRITE | libc::F_SEAL_SEAL))
    );
    unsafe { libc::munmap(w.cast(), 64 * KB) };
    println!(
        "L13 after unmapping: add WRITE|SEAL: {} -> {}",
        rc("add", add_seals(fd, libc::F_SEAL_WRITE | libc::F_SEAL_SEAL)),
        seals_str(get_seals(fd))
    );
    println!(
        "L13 sealed NOEXEC region: mmap PROT_READ|PROT_EXEC shared: {}",
        map_rc(
            fd,
            64 * KB,
            libc::PROT_READ | libc::PROT_EXEC,
            libc::MAP_SHARED
        )
    );
    println!(
        "L13 fchmod(0755) under the EXEC seal: {}",
        rc("fchmod", unsafe { libc::fchmod(fd, 0o755) } as libc::c_long)
    );
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::syscall(libc::SYS_fstat, fd, &mut st) };
    println!(
        "L13 raw SYS_fstat: rc={r} mode={:#o} size={} nlink={}",
        st.st_mode, st.st_size, st.st_nlink
    );
    unsafe { libc::close(fd) };
}

fn main() {
    if let Some(base) = std::env::args()
        .nth(2)
        .filter(|_| std::env::args().nth(1).as_deref() == Some("l12"))
    {
        l12_memcg(&base);
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("l13") {
        l13_noexec_sets();
        return;
    }
    let un = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    println!(
        "L0 kernel {}; vm.memfd_noexec={}",
        un.trim(),
        std::fs::read_to_string("/proc/sys/vm/memfd_noexec")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|e| format!("({e})"))
    );
    println!(
        "L0 constants (libc crate): F_ADD_SEALS={} F_GET_SEALS={} SEAL={} SHRINK={} GROW={} WRITE={} FUTURE_WRITE={}",
        libc::F_ADD_SEALS,
        libc::F_GET_SEALS,
        libc::F_SEAL_SEAL,
        libc::F_SEAL_SHRINK,
        libc::F_SEAL_GROW,
        libc::F_SEAL_WRITE,
        libc::F_SEAL_FUTURE_WRITE
    );
    l1_seal_write_vs_mappings();
    l1b_what_blocks_seal_write();
    l2_future_write();
    l3_unseal_able_kinds();
    l7_pass_between_processes();
    l8_shrink_without_seal();
    l10_costs();
}
