//! The Linux seccomp-bpf program, built and checked as plain data
//! (security scope 6.7). Written from the kernel's seccomp documentation
//! (seccomp(2), the `seccomp_data` layout and the classic BPF instruction
//! set); no other project's filter is a source.
//!
//! Pure code: it compiles on every OS, so the builder is tested anywhere
//! by running its output through [`run`], a small evaluator of the BPF
//! subset it emits. `sandbox::linux` installs the result.
//!
//! **Layout.** The arch is checked first (anything else is denied).
//! Then each rule is one block: `ld nr; jeq nr` falls into the block or
//! jumps past it; the block checks the rule's argument conditions, any
//! failure jumping past the block, and ends in its own `ret action`. So
//! every jump is short (no jump offset can overflow its 8-bit field,
//! whatever the table's length), and several rules for one syscall are
//! alternatives: the first whose conditions all hold decides (D6: the
//! two shapes of `fcntl`). A call no rule allows reaches the final
//! `ret DENY`.
//!
//! **Arguments are compared on their low 32 bits.** The kernel truncates
//! the `int` arguments these rules check, so a check on the high word
//! could be bypassed by setting it; the low word is what the kernel uses
//! (seccomp(2), "Caveats").

/// One classic BPF instruction, laid out as the kernel's `sock_filter`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Insn {
    /// The opcode.
    pub code: u16,
    /// Instructions to skip when a jump's condition holds.
    pub jt: u8,
    /// Instructions to skip when it does not.
    pub jf: u8,
    /// The constant operand.
    pub k: u32,
}

/// `BPF_LD | BPF_W | BPF_ABS`: load a 32-bit word of `seccomp_data`.
const LD_W_ABS: u16 = 0x20;
/// `BPF_JMP | BPF_JEQ | BPF_K`.
const JEQ_K: u16 = 0x15;
/// `BPF_ALU | BPF_AND | BPF_K`.
const AND_K: u16 = 0x54;
/// `BPF_RET | BPF_K`.
const RET_K: u16 = 0x06;

/// `seccomp_data` offsets: `nr` (u32), `arch` (u32), `ip` (u64), then six
/// u64 arguments; on the little-endian arches below an argument's low
/// word is at its own offset.
const OFF_NR: u32 = 0;
const OFF_ARCH: u32 = 4;
const OFF_ARGS: u32 = 16;

/// `SECCOMP_RET_ALLOW`.
pub const RET_ALLOW: u32 = 0x7fff_0000;
/// `SECCOMP_RET_ERRNO`; the errno is the low 16 bits.
const RET_ERRNO: u32 = 0x0005_0000;
/// Linux `EPERM`. The deny is observable, never silent (security 6.7).
const EPERM: u32 = 1;
/// Linux `ENOSYS`: `clone3` reports "not implemented", so the C library
/// falls back to `clone`, whose flags this filter can check (`clone3`
/// passes them in memory, out of a filter's reach).
const ENOSYS: u32 = 38;
/// Deny with `EPERM`.
pub const RET_DENY: u32 = RET_ERRNO | EPERM;
/// Deny with `ENOSYS`.
pub const RET_NOSYS: u32 = RET_ERRNO | ENOSYS;

/// `AUDIT_ARCH_X86_64`.
pub const ARCH_X86_64: u32 = 0xC000_003E;
/// `AUDIT_ARCH_AARCH64`.
pub const ARCH_AARCH64: u32 = 0xC000_00B7;

/// A condition on one argument's low 32 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// Equal to the value.
    Eq(u32),
    /// `arg & mask == value`: "these bits set, those clear".
    Masked {
        /// The bits looked at.
        mask: u32,
        /// What they must be.
        value: u32,
    },
    /// One of the values.
    OneOf(&'static [u32]),
}

/// One allowed syscall: its number, the conditions on its arguments (all
/// must hold), what happens when it matches, and why it is there.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    /// The syscall number on this arch.
    pub nr: u32,
    /// `(argument index, condition)`, all required.
    pub args: &'static [(u32, Cond)],
    /// [`RET_ALLOW`], or an errno return such as [`RET_NOSYS`].
    pub action: u32,
    /// The written reason (security 6.7: every entry has one).
    pub reason: &'static str,
}

const fn allow(nr: u32, reason: &'static str) -> Rule {
    Rule {
        nr,
        args: &[],
        action: RET_ALLOW,
        reason,
    }
}

const fn allow_if(nr: u32, args: &'static [(u32, Cond)], reason: &'static str) -> Rule {
    Rule {
        nr,
        args,
        action: RET_ALLOW,
        reason,
    }
}

// Constants for the argument conditions (Linux values, both arches).
const AF_UNIX: u32 = 1;
/// `SOCK_SEQPACKET | SOCK_CLOEXEC | SOCK_NONBLOCK`: the only socket the
/// IPC transport makes.
const IPC_SOCKET_TYPE: u32 = 5 | 0o2_000_000 | 0o4000;
const MSG_DONTWAIT: u32 = 0x40;
const MSG_NOSIGNAL: u32 = 0x4000;
const SOL_SOCKET: u32 = 1;
/// The socket options the transport reads: `SO_TYPE`, `SO_ERROR`,
/// `SO_SNDBUF`, `SO_PASSCRED`, `SO_PEERCRED`, `SO_PROTOCOL`, `SO_DOMAIN`.
const GETSOCKOPT_NAMES: &[u32] = &[3, 4, 7, 16, 17, 38, 39];
/// The ones it sets: `SO_SNDBUF`, `SO_PASSCRED`.
const SETSOCKOPT_NAMES: &[u32] = &[7, 16];
/// `F_GETFD`, `F_SETFD`, and `F_GET_SEALS` (D6: region acceptance).
const FCNTL_CMDS: &[u32] = &[1, 2, 1034];
/// `F_ADD_SEALS` (fcntl(2) "File sealing").
const F_ADD_SEALS: u32 = 1033;
/// The two seal sets a sandboxed process may add to a region it holds:
/// `F_SEAL_WRITE | F_SEAL_SEAL` (sealing a writer) and
/// `F_SEAL_FUTURE_WRITE | F_SEAL_SEAL` (sharing it as a producer).
const REGION_SEAL_SETS: &[u32] = &[0x8 | 0x1, 0x10 | 0x1];
/// `PROT_EXEC`: never granted, so no executable mapping can be made
/// (jitless tiers; D6 review M5, since `MFD_NOEXEC_SEAL` does not stop a
/// `PROT_EXEC` mapping, fact L13).
const PROT_EXEC: u32 = 0x4;
/// `SHUT_WR`.
const SHUT_WR: u32 = 1;
/// `CLONE_VM | CLONE_THREAD`: a thread, never a new process.
const CLONE_THREAD_BITS: u32 = 0x100 | 0x1_0000;
/// Every `CLONE_NEW*` flag `clone` accepts (mount, cgroup, UTS, IPC,
/// user, PID, network): none may be set, so a sandboxed process can
/// neither fork nor make namespaces (which would let it forge the
/// credentials the IPC transport relies on, unix(7)).
const CLONE_NEW_BITS: u32 =
    0x2_0000 | 0x0200_0000 | 0x0400_0000 | 0x0800_0000 | 0x1000_0000 | 0x2000_0000 | 0x4000_0000;

const CLONE_ARGS: &[(u32, Cond)] = &[(
    0,
    Cond::Masked {
        mask: CLONE_THREAD_BITS | CLONE_NEW_BITS,
        value: CLONE_THREAD_BITS,
    },
)];
const SOCKETPAIR_ARGS: &[(u32, Cond)] = &[
    (0, Cond::Eq(AF_UNIX)),
    (1, Cond::Eq(IPC_SOCKET_TYPE)),
    (2, Cond::Eq(0)),
];
const SENDMSG_ARGS: &[(u32, Cond)] = &[(
    2,
    Cond::Masked {
        mask: MSG_DONTWAIT | MSG_NOSIGNAL,
        value: MSG_DONTWAIT | MSG_NOSIGNAL,
    },
)];
const RECVMSG_ARGS: &[(u32, Cond)] = &[(
    2,
    Cond::Masked {
        mask: MSG_DONTWAIT,
        value: MSG_DONTWAIT,
    },
)];
const GETSOCKOPT_ARGS: &[(u32, Cond)] = &[
    (1, Cond::Eq(SOL_SOCKET)),
    (2, Cond::OneOf(GETSOCKOPT_NAMES)),
];
const SETSOCKOPT_ARGS: &[(u32, Cond)] = &[
    (1, Cond::Eq(SOL_SOCKET)),
    (2, Cond::OneOf(SETSOCKOPT_NAMES)),
];
const FCNTL_ARGS: &[(u32, Cond)] = &[(1, Cond::OneOf(FCNTL_CMDS))];
const ADD_SEALS_ARGS: &[(u32, Cond)] = &[
    (1, Cond::Eq(F_ADD_SEALS)),
    (2, Cond::OneOf(REGION_SEAL_SETS)),
];
const NO_EXEC_ARGS: &[(u32, Cond)] = &[(
    2,
    Cond::Masked {
        mask: PROT_EXEC,
        value: 0,
    },
)];
const SHUTDOWN_ARGS: &[(u32, Cond)] = &[(1, Cond::Eq(SHUT_WR))];

/// The x86_64 table: the empty process's needs, then the IPC
/// transport's (M1 phase D3), each with its reason.
pub const RULES_X86_64: &[Rule] = &[
    allow(0, "read — the pre-opened IPC endpoint"),
    allow(1, "write — IPC, stderr"),
    allow(3, "close"),
    allow(5, "fstat — std's fd queries; IPC handle acceptance"),
    allow(8, "lseek"),
    allow_if(
        9,
        NO_EXEC_ARGS,
        "mmap — the allocator, IPC regions; never executable",
    ),
    allow_if(
        10,
        NO_EXEC_ARGS,
        "mprotect — the allocator; never executable",
    ),
    allow(11, "munmap — the allocator"),
    allow(12, "brk — the allocator"),
    allow(13, "rt_sigaction — thread setup"),
    allow(14, "rt_sigprocmask — thread setup"),
    allow(15, "rt_sigreturn — signal return"),
    allow(16, "ioctl — std's fd queries"),
    allow(24, "sched_yield"),
    allow(28, "madvise — the allocator"),
    allow(35, "nanosleep"),
    allow(39, "getpid"),
    allow(44, "sendto — the IPC ready byte (std uses MSG_NOSIGNAL)"),
    allow_if(
        46,
        SENDMSG_ARGS,
        "sendmsg — IPC records, never blocking, no SIGPIPE",
    ),
    allow_if(47, RECVMSG_ARGS, "recvmsg — IPC records, never blocking"),
    allow_if(
        48,
        SHUTDOWN_ARGS,
        "shutdown — closing a transferred IPC sender for writing",
    ),
    allow_if(
        53,
        SOCKETPAIR_ARGS,
        "socketpair — new IPC channels, Unix seqpacket only",
    ),
    allow_if(
        54,
        SETSOCKOPT_ARGS,
        "setsockopt — IPC credentials and buffer size",
    ),
    allow_if(
        55,
        GETSOCKOPT_ARGS,
        "getsockopt — IPC handle acceptance and errors",
    ),
    allow_if(
        56,
        CLONE_ARGS,
        "clone — threads only (the sanctioned thread spawn)",
    ),
    allow(60, "exit"),
    allow(63, "uname — std's platform queries"),
    allow_if(
        72,
        FCNTL_ARGS,
        "fcntl — close-on-exec of received IPC handles; region seals read",
    ),
    allow_if(
        72,
        ADD_SEALS_ARGS,
        "fcntl — sealing an IPC region (write or future-write, and seal)",
    ),
    allow(97, "getrlimit — std's resource queries; IPC fd headroom"),
    allow(131, "sigaltstack — signal handling"),
    allow(157, "prctl — PR_SET_NO_NEW_PRIVS and friends"),
    allow(158, "arch_prctl — thread TLS"),
    allow(186, "gettid"),
    allow(202, "futex — thread parking"),
    allow(204, "sched_getaffinity — std's CPU queries"),
    allow(218, "set_tid_address — thread teardown"),
    allow(228, "clock_gettime"),
    allow(230, "clock_nanosleep — std::thread::sleep"),
    allow(231, "exit_group"),
    allow(233, "epoll_ctl — the IPC IO thread's interest list"),
    allow(234, "tgkill — thread signalling"),
    allow(262, "newfstatat — std's fd queries"),
    allow(273, "set_robust_list — glibc thread teardown"),
    allow(281, "epoll_pwait — the IPC IO thread's wait"),
    allow(290, "eventfd2 — waking the IPC IO thread"),
    allow(291, "epoll_create1 — the IPC IO thread's epoll instance"),
    allow(302, "prlimit64 — std's resource queries"),
    allow(318, "getrandom — std's hash seeds"),
    allow(334, "rseq — glibc's restartable sequences"),
    Rule {
        nr: 435,
        args: &[],
        action: RET_NOSYS,
        reason: "clone3 — refused as unimplemented, so glibc uses clone",
    },
];

/// The aarch64 table, the same operations.
pub const RULES_AARCH64: &[Rule] = &[
    allow(19, "eventfd2 — waking the IPC IO thread"),
    allow(20, "epoll_create1 — the IPC IO thread's epoll instance"),
    allow(21, "epoll_ctl — the IPC IO thread's interest list"),
    allow(22, "epoll_pwait — the IPC IO thread's wait"),
    allow_if(
        25,
        FCNTL_ARGS,
        "fcntl — close-on-exec of received IPC handles; region seals read",
    ),
    allow_if(
        25,
        ADD_SEALS_ARGS,
        "fcntl — sealing an IPC region (write or future-write, and seal)",
    ),
    allow(29, "ioctl"),
    allow(57, "close"),
    allow(62, "lseek"),
    allow(63, "read"),
    allow(64, "write"),
    allow(79, "newfstatat"),
    allow(80, "fstat — IPC handle acceptance"),
    allow(93, "exit"),
    allow(94, "exit_group"),
    allow(96, "set_tid_address"),
    allow(98, "futex"),
    allow(99, "set_robust_list — glibc thread teardown"),
    allow(101, "nanosleep"),
    allow(113, "clock_gettime"),
    allow(115, "clock_nanosleep"),
    allow(123, "sched_getaffinity"),
    allow(124, "sched_yield"),
    allow(131, "tgkill"),
    allow(132, "sigaltstack"),
    allow(134, "rt_sigaction"),
    allow(135, "rt_sigprocmask"),
    allow(139, "rt_sigreturn"),
    allow(160, "uname"),
    allow(163, "getrlimit — IPC fd headroom"),
    allow(167, "prctl"),
    allow(172, "getpid"),
    allow(178, "gettid"),
    allow_if(
        199,
        SOCKETPAIR_ARGS,
        "socketpair — new IPC channels, Unix seqpacket only",
    ),
    allow(206, "sendto — the IPC ready byte (std uses MSG_NOSIGNAL)"),
    allow_if(
        208,
        SETSOCKOPT_ARGS,
        "setsockopt — IPC credentials and buffer size",
    ),
    allow_if(
        209,
        GETSOCKOPT_ARGS,
        "getsockopt — IPC handle acceptance and errors",
    ),
    allow_if(
        210,
        SHUTDOWN_ARGS,
        "shutdown — closing a transferred IPC sender for writing",
    ),
    allow_if(
        211,
        SENDMSG_ARGS,
        "sendmsg — IPC records, never blocking, no SIGPIPE",
    ),
    allow_if(212, RECVMSG_ARGS, "recvmsg — IPC records, never blocking"),
    allow(214, "brk"),
    allow(215, "munmap"),
    allow_if(
        220,
        CLONE_ARGS,
        "clone — threads only (the sanctioned thread spawn)",
    ),
    allow_if(222, NO_EXEC_ARGS, "mmap — never executable"),
    allow_if(226, NO_EXEC_ARGS, "mprotect — never executable"),
    allow(233, "madvise"),
    allow(261, "prlimit64"),
    allow(278, "getrandom"),
    allow(293, "rseq"),
    Rule {
        nr: 435,
        args: &[],
        action: RET_NOSYS,
        reason: "clone3 — refused as unimplemented, so glibc uses clone",
    },
];

/// Why a program could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildError {
    /// A rule's block is too long for an 8-bit jump.
    BlockTooLong,
    /// The program is longer than the kernel's limit (`BPF_MAXINSNS`).
    TooLong,
}

/// The kernel's per-program instruction limit.
const MAX_INSNS: usize = 4096;

/// One rule's argument checks and returns: everything after its `jeq nr`.
fn block(rule: &Rule) -> Result<Vec<Insn>, BuildError> {
    // Build the checks with jf placeholders aimed past the block, then
    // fix the distances once the length is known.
    let mut b: Vec<Insn> = Vec::new();
    // Indices of jumps whose false branch leaves the block.
    let mut to_deny: Vec<usize> = Vec::new();
    for &(arg, cond) in rule.args {
        let off = OFF_ARGS + arg * 8;
        b.push(Insn {
            code: LD_W_ABS,
            jt: 0,
            jf: 0,
            k: off,
        });
        match cond {
            Cond::Eq(v) => {
                to_deny.push(b.len());
                b.push(Insn {
                    code: JEQ_K,
                    jt: 0,
                    jf: 0,
                    k: v,
                });
            }
            Cond::Masked { mask, value } => {
                b.push(Insn {
                    code: AND_K,
                    jt: 0,
                    jf: 0,
                    k: mask,
                });
                to_deny.push(b.len());
                b.push(Insn {
                    code: JEQ_K,
                    jt: 0,
                    jf: 0,
                    k: value,
                });
            }
            Cond::OneOf(values) => {
                // `jeq v_i` jumps past the rest of the list on a match;
                // the last one's false branch leaves the block.
                let n = values.len();
                for (i, v) in values.iter().enumerate() {
                    let rest = n - 1 - i;
                    if rest == 0 {
                        to_deny.push(b.len());
                        b.push(Insn {
                            code: JEQ_K,
                            jt: 0,
                            jf: 0,
                            k: *v,
                        });
                    } else {
                        b.push(Insn {
                            code: JEQ_K,
                            jt: u8::try_from(rest).map_err(|_e| BuildError::BlockTooLong)?,
                            jf: 0,
                            k: *v,
                        });
                    }
                }
            }
        }
    }
    b.push(Insn {
        code: RET_K,
        jt: 0,
        jf: 0,
        k: rule.action,
    });
    // A failed condition skips the rest of the block, to the next rule.
    let past = b.len();
    for at in to_deny {
        let dist = past - at - 1;
        if let Some(insn) = b.get_mut(at) {
            insn.jf = u8::try_from(dist).map_err(|_e| BuildError::BlockTooLong)?;
        }
    }
    Ok(b)
}

/// The whole program for `arch`'s table.
///
/// # Errors
/// [`BuildError`] if a block or the program is over a kernel limit.
pub fn program(arch: u32, rules: &[Rule]) -> Result<Vec<Insn>, BuildError> {
    let mut p = vec![
        Insn {
            code: LD_W_ABS,
            jt: 0,
            jf: 0,
            k: OFF_ARCH,
        },
        // This program's arch: on to the syscall number; else DENY.
        Insn {
            code: JEQ_K,
            jt: 1,
            jf: 0,
            k: arch,
        },
        Insn {
            code: RET_K,
            jt: 0,
            jf: 0,
            k: RET_DENY,
        },
    ];
    for rule in rules {
        let body = block(rule)?;
        // The number again: the last block's checks loaded an argument.
        p.push(Insn {
            code: LD_W_ABS,
            jt: 0,
            jf: 0,
            k: OFF_NR,
        });
        p.push(Insn {
            code: JEQ_K,
            jt: 0,
            jf: u8::try_from(body.len()).map_err(|_e| BuildError::BlockTooLong)?,
            k: rule.nr,
        });
        p.extend(body);
    }
    p.push(Insn {
        code: RET_K,
        jt: 0,
        jf: 0,
        k: RET_DENY,
    });
    if p.len() > MAX_INSNS {
        return Err(BuildError::TooLong);
    }
    Ok(p)
}

/// The input a filter sees: the arch, the syscall number and its six
/// arguments (`seccomp_data`, without the instruction pointer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Call {
    /// `AUDIT_ARCH_*`.
    pub arch: u32,
    /// The syscall number.
    pub nr: u32,
    /// The arguments.
    pub args: [u64; 6],
}

/// Why [`run`] stopped without a return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunError {
    /// A jump or the end of the program left the program.
    OutOfRange,
    /// An opcode this builder never emits.
    UnknownOpcode,
    /// A load outside `seccomp_data`.
    BadLoad,
}

/// Runs `prog` on `call` with the subset of classic BPF the builder
/// emits: the reference the builder is tested against.
///
/// # Errors
/// [`RunError`] for a malformed program.
pub fn run(prog: &[Insn], call: &Call) -> Result<u32, RunError> {
    let mut acc: u32 = 0;
    let mut pc: usize = 0;
    // A program with only forward jumps ends within its own length.
    for _ in 0..=prog.len() {
        let insn = prog.get(pc).ok_or(RunError::OutOfRange)?;
        let next = pc.checked_add(1).ok_or(RunError::OutOfRange)?;
        match insn.code {
            LD_W_ABS => {
                acc = load(call, insn.k).ok_or(RunError::BadLoad)?;
                pc = next;
            }
            AND_K => {
                acc &= insn.k;
                pc = next;
            }
            JEQ_K => {
                let skip = if acc == insn.k { insn.jt } else { insn.jf };
                pc = next
                    .checked_add(usize::from(skip))
                    .ok_or(RunError::OutOfRange)?;
            }
            RET_K => return Ok(insn.k),
            _ => return Err(RunError::UnknownOpcode),
        }
    }
    Err(RunError::OutOfRange)
}

/// A 32-bit word of `seccomp_data` at byte offset `off` (little-endian).
fn load(call: &Call, off: u32) -> Option<u32> {
    match off {
        OFF_NR => Some(call.nr),
        OFF_ARCH => Some(call.arch),
        _ if off >= OFF_ARGS && (off - OFF_ARGS).is_multiple_of(4) => {
            let word = (off - OFF_ARGS) / 4;
            let arg = *call.args.get(usize::try_from(word / 2).ok()?)?;
            // Truncation is the point: the low or high 32-bit word.
            #[expect(clippy::cast_possible_truncation, reason = "taking a 32-bit word")]
            let w = if word.is_multiple_of(2) {
                arg as u32
            } else {
                (arg >> 32) as u32
            };
            Some(w)
        }
        _ => None,
    }
}

/// What the table says about `call`, decided directly from the rules:
/// the oracle [`run`]'s result must equal.
pub fn expected(arch: u32, rules: &[Rule], call: &Call) -> u32 {
    if call.arch != arch {
        return RET_DENY;
    }
    let holds = |rule: &Rule| {
        rule.args.iter().all(|&(i, cond)| {
            let Some(&raw) = call.args.get(usize::try_from(i).unwrap_or(usize::MAX)) else {
                return false;
            };
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the low 32 bits, as the kernel"
            )]
            let a = raw as u32;
            match cond {
                Cond::Eq(v) => a == v,
                Cond::Masked { mask, value } => a & mask == value,
                Cond::OneOf(vs) => vs.contains(&a),
            }
        })
    };
    rules
        .iter()
        .find(|r| r.nr == call.nr && holds(r))
        .map_or(RET_DENY, |r| r.action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> [(u32, &'static [Rule]); 2] {
        [(ARCH_X86_64, RULES_X86_64), (ARCH_AARCH64, RULES_AARCH64)]
    }

    /// Every rule has a reason; a syscall named twice is `fcntl` only,
    /// whose two shapes are alternatives with the same action.
    #[test]
    fn every_rule_has_a_reason_and_only_fcntl_has_two() {
        for (arch, rules) in tables() {
            let fcntl = if arch == ARCH_X86_64 { 72 } else { 25 };
            let mut seen = std::collections::BTreeMap::new();
            for r in rules {
                assert!(!r.reason.is_empty());
                if let Some(action) = seen.insert(r.nr, r.action) {
                    assert_eq!(r.nr, fcntl, "duplicate rule for {}", r.nr);
                    assert_eq!(action, r.action);
                }
            }
        }
    }

    /// Every rule's own number with interesting argument values, plus
    /// unlisted numbers and the other arch: the program agrees with the
    /// table on all of them.
    #[test]
    fn program_agrees_with_the_table() {
        let probes: [u64; 17] = [
            0,
            1,
            2,
            5,
            u64::from(IPC_SOCKET_TYPE),
            u64::from(MSG_DONTWAIT),
            u64::from(MSG_DONTWAIT | MSG_NOSIGNAL),
            u64::from(CLONE_THREAD_BITS),
            u64::from(CLONE_THREAD_BITS | 0x1000_0000),
            // The high word set, the low word valid: still allowed.
            (1 << 32) | u64::from(IPC_SOCKET_TYPE),
            u64::MAX,
            39,
            // fcntl's seal commands and sets, and PROT_* with PROT_EXEC.
            1033,
            1034,
            9,
            17,
            7,
        ];
        let mut checked = 0u32;
        for (arch, rules) in tables() {
            let prog = program(arch, rules).unwrap();
            let mut nrs: Vec<u32> = rules.iter().map(|r| r.nr).collect();
            nrs.extend([2, 41, 57, 59, 101, 999, 0x4000_0000]);
            for other in [arch, ARCH_X86_64 ^ ARCH_AARCH64 ^ arch, 0] {
                for &nr in &nrs {
                    for &a in &probes {
                        for &b in &probes {
                            for &c in &probes {
                                let call = Call {
                                    arch: other,
                                    nr,
                                    args: [a, b, c, 0, 0, 0],
                                };
                                assert_eq!(
                                    run(&prog, &call),
                                    Ok(expected(arch, rules, &call)),
                                    "{call:?}"
                                );
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(checked > 100_000);
    }

    #[test]
    fn the_sandbox_cannot_fork_or_make_namespaces() {
        for (arch, rules) in tables() {
            let prog = program(arch, rules).unwrap();
            let clone_nr = if arch == ARCH_X86_64 { 56 } else { 220 };
            let call = |flags: u64| Call {
                arch,
                nr: clone_nr,
                args: [flags, 0, 0, 0, 0, 0],
            };
            // A thread: allowed.
            assert_eq!(
                run(&prog, &call(u64::from(CLONE_THREAD_BITS))),
                Ok(RET_ALLOW)
            );
            // fork-like, and every namespace flag on a thread: denied.
            assert_eq!(run(&prog, &call(0x11)), Ok(RET_DENY));
            for ns in [
                0x2_0000,
                0x0200_0000,
                0x0400_0000,
                0x0800_0000,
                0x1000_0000,
                0x2000_0000,
                0x4000_0000,
            ] {
                assert_eq!(
                    run(&prog, &call(u64::from(CLONE_THREAD_BITS | ns))),
                    Ok(RET_DENY),
                    "{ns:#x}"
                );
            }
            let clone3 = Call {
                arch,
                nr: 435,
                args: [0; 6],
            };
            assert_eq!(run(&prog, &clone3), Ok(RET_NOSYS));
        }
    }

    #[test]
    fn only_unix_seqpacket_socketpairs() {
        for (arch, rules) in tables() {
            let prog = program(arch, rules).unwrap();
            let nr = if arch == ARCH_X86_64 { 53 } else { 199 };
            let call = |d: u64, t: u64, p: u64| Call {
                arch,
                nr,
                args: [d, t, p, 0, 0, 0],
            };
            assert_eq!(
                run(&prog, &call(1, u64::from(IPC_SOCKET_TYPE), 0)),
                Ok(RET_ALLOW)
            );
            assert_eq!(run(&prog, &call(1, 1, 0)), Ok(RET_DENY)); // stream
            assert_eq!(run(&prog, &call(1, 5, 0)), Ok(RET_DENY)); // blocking
            assert_eq!(
                run(&prog, &call(2, u64::from(IPC_SOCKET_TYPE), 0)),
                Ok(RET_DENY)
            ); // inet
        }
    }

    /// D6: a sandboxed process reads seals and adds only the two region
    /// seal sets; nothing else through `fcntl`.
    #[test]
    fn fcntl_seals_only_the_two_region_sets() {
        for (arch, rules) in tables() {
            let prog = program(arch, rules).unwrap();
            let nr = if arch == ARCH_X86_64 { 72 } else { 25 };
            let call = |cmd: u64, arg: u64| Call {
                arch,
                nr,
                args: [5, cmd, arg, 0, 0, 0],
            };
            for ok in [
                call(1, 0),
                call(2, 1),
                call(1034, 0),
                call(1033, 9),
                call(1033, 17),
            ] {
                assert_eq!(run(&prog, &ok), Ok(RET_ALLOW), "{ok:?}");
            }
            // F_SETFL (4), F_DUPFD (0), seals alone or with grow, shrink
            // or exec, any other set.
            for no in [
                call(4, 0),
                call(0, 10),
                call(1033, 8),
                call(1033, 16),
                call(1033, 1),
                call(1033, 0x26),
                call(1033, 9 | 2),
                call(1033, 0),
            ] {
                assert_eq!(run(&prog, &no), Ok(RET_DENY), "{no:?}");
            }
        }
    }

    /// D6 review M5: no executable mapping, by `mmap` or `mprotect`.
    #[test]
    fn nothing_is_mapped_executable() {
        for (arch, rules) in tables() {
            let prog = program(arch, rules).unwrap();
            let (mmap, mprotect) = if arch == ARCH_X86_64 {
                (9, 10)
            } else {
                (222, 226)
            };
            for nr in [mmap, mprotect] {
                let call = |prot: u64| Call {
                    arch,
                    nr,
                    args: [0, 4096, prot, 1, 0, 0],
                };
                for ok in [0, 1, 3] {
                    assert_eq!(run(&prog, &call(ok)), Ok(RET_ALLOW));
                }
                for no in [4, 5, 7, (1 << 32) | 4] {
                    assert_eq!(run(&prog, &call(no)), Ok(RET_DENY), "{nr} {no:#x}");
                }
            }
        }
    }

    #[test]
    fn the_programs_fit_the_kernel_limits() {
        for (arch, rules) in tables() {
            let prog = program(arch, rules).unwrap();
            assert!(prog.len() <= MAX_INSNS);
        }
    }
}
