//! Which received rights become handles (IPC-R1, security scope SEC-4).
//!
//! A channel end crosses as port rights: a sender end is its data
//! port's send right then its credit port's receive right, a receiver
//! end its data port's receive right ([`slots`]). Each right arrives
//! already inserted in this process, with a kind its sender claims in
//! the record's descriptor table. The transport believes nothing the
//! sender says about it: the sys module asks the kernel what the name
//! now holds (`mach_port_type`), how many send references
//! (`mach_port_get_refs`), whether a send right names a kernel object
//! (`mach_port_kobject`, P8) and whether a receive right is a plain
//! queue (`mach_port_get_attributes`, P6), and hands the answers here as
//! [`SlotFacts`].
//!
//! The rules, so that only fresh ends of channels pass:
//! - a send slot is a send right (disposition 17) to a plain port, not a
//!   task, thread, host, memory-entry or IOKit port (P8, P9); the name
//!   holds nothing else, except the receive right when this process's
//!   own receiver end is that port's; and its user references are
//!   exactly the one this record brought: more means this process
//!   already held a send right to the port (the kernel gives one port
//!   one name in a task and merges the rights, P32), which is either an
//!   end it already holds or a port outside the transport (its
//!   bootstrap port, a reply port), and neither may become a handle;
//! - a receive slot is a receive right (disposition 16) to a plain
//!   queue; the name holds nothing else, except send rights this
//!   process's own sender end holds; and no port-destroyed request rides
//!   on it (one would hand the right back to whoever asked when this
//!   process destroys it, R8);
//! - a receiver end's data port also has no send-once rights anywhere
//!   (status `sorights`, R8): a former holder that kept one could make
//!   the kernel send a notification to it, or queue past its limit
//!   (P18). The sending transport cancels its own no-senders request
//!   before moving the right, so an honest one has none;
//! - a sender end's credit port is a receive slot whose send-once rights
//!   are the reply rights of messages not yet credited, so the decision
//!   allows at most the carried end's in-flight count
//!   ([`Slot::Credit`]);
//! - no name twice in one record.
//!
//! The kernel cannot say which channel two rights belong to, so a
//! sender end built from the data send right of one channel and the
//! credit receive right of another passes. That only confuses the flow
//! of channels the forger already held both rights of: credits on the
//! wrong port are refused there without blame.

use hg_substrate::ipc::{HandleKind, MAX_MESSAGE_HANDLES};

use super::super::common::codec::WireKind;
use super::wire::{
    DESC_PORT, KOBJECT_NONE, MAX_SLOTS, NAME_DEAD, NAME_NULL, PORT_RECEIVE, PORT_SEND,
    TYPE_RECEIVE, TYPE_SEND,
};

/// What one received right is, as the kernel says. The sys module fills
/// it; a query that fails leaves a value no rule accepts (0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SlotFacts {
    /// The descriptor's type (only port descriptors pass).
    pub(crate) desc_type: u8,
    /// The right's name in this process.
    pub(crate) name: u32,
    /// Its disposition as received.
    pub(crate) disposition: u8,
    /// `mach_port_type` of the name after receipt.
    pub(crate) type_bits: u32,
    /// Send user references the name holds after receipt.
    pub(crate) send_refs: u32,
    /// For a send right, `mach_port_kobject`'s type (0 if the query
    /// failed).
    pub(crate) kobject: u32,
    /// For a receive right, whether its receive status could be read: a
    /// plain queue's can (P6).
    pub(crate) plain_queue: bool,
    /// For a receive right, the send-once rights to its port that exist
    /// anywhere (status word 5, R8); 0 if the query failed.
    pub(crate) send_once: u32,
    /// For a receive right, whether a port-destroyed request is set on it
    /// (status word 7, R8); true if the query failed.
    pub(crate) destroy_request: bool,
}

/// Which names this process's transport already holds as ends.
pub(crate) trait Held {
    /// Send user references this process's ends hold on `name`.
    fn send_refs(&self, name: u32) -> u32;
    /// Whether one of this process's receiver ends is `name`'s port.
    fn receiver(&self, name: u32) -> bool;
}

/// Why a right was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Refusal {
    /// Not a port descriptor, or a null or dead name.
    NotAPort,
    /// Not the disposition the slot needs.
    Disposition,
    /// A kernel object, not a plain port.
    KernelObject,
    /// The name holds a right it should not (or lacks the one it should).
    Rights,
    /// A send right to a port this process already held one to.
    AlreadyHeld,
    /// A receive right that is not a plain queue.
    NotAQueue,
    /// A receive right whose port has a port-destroyed request set.
    DestroyRequest,
    /// A receive right whose port has send-once rights a sender could
    /// still use: any for a receiver end, more than the messages in
    /// flight for a credit port.
    SendOnce,
    /// The same name twice in one record.
    Duplicate,
}

/// What a right must be in its slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    /// A data port's send right (the sender end's first right).
    Send,
    /// A receiver end's data port's receive right.
    Receive,
    /// A sender end's credit port's receive right: as [`Slot::Receive`],
    /// except that its send-once rights are checked against the carried
    /// end's counters, in the decision.
    Credit,
}

/// The slots a claimed kind takes on the wire, and how many.
pub(crate) const fn slots(kind: WireKind) -> ([Slot; 2], usize) {
    match kind {
        WireKind::Sender | WireKind::BoundSender => ([Slot::Send, Slot::Credit], 2),
        WireKind::Receiver => ([Slot::Receive, Slot::Receive], 1),
    }
}

/// The handle kind a claim maps to.
pub(crate) const fn handle_kind(claimed: WireKind) -> HandleKind {
    match claimed {
        WireKind::Sender => HandleKind::ChannelSender,
        WireKind::BoundSender => HandleKind::BoundChannelSender,
        WireKind::Receiver => HandleKind::ChannelReceiver,
    }
}

/// Decides one right in its slot. The checks run in a fixed order, so
/// the reason names the first rule broken.
///
/// # Errors
/// The [`Refusal`]; the caller then refuses the whole record.
pub(crate) fn accept_slot<H: Held>(slot: Slot, f: &SlotFacts, held: &H) -> Result<(), Refusal> {
    if f.desc_type != DESC_PORT || f.name == NAME_NULL || f.name == NAME_DEAD {
        return Err(Refusal::NotAPort);
    }
    match slot {
        Slot::Send => {
            if f.disposition != PORT_SEND {
                return Err(Refusal::Disposition);
            }
            if f.kobject != KOBJECT_NONE {
                return Err(Refusal::KernelObject);
            }
            if held.send_refs(f.name) != 0 {
                return Err(Refusal::AlreadyHeld);
            }
            let allowed = TYPE_SEND
                | if held.receiver(f.name) {
                    TYPE_RECEIVE
                } else {
                    0
                };
            if f.type_bits & TYPE_SEND == 0 || f.type_bits & !allowed != 0 {
                return Err(Refusal::Rights);
            }
            if f.send_refs != 1 {
                return Err(Refusal::AlreadyHeld);
            }
            Ok(())
        }
        Slot::Receive | Slot::Credit => {
            if f.disposition != PORT_RECEIVE {
                return Err(Refusal::Disposition);
            }
            let ours = held.send_refs(f.name);
            let allowed = TYPE_RECEIVE | if ours > 0 { TYPE_SEND } else { 0 };
            if f.type_bits & TYPE_RECEIVE == 0 || f.type_bits & !allowed != 0 {
                return Err(Refusal::Rights);
            }
            if f.send_refs != ours {
                return Err(Refusal::Rights);
            }
            if !f.plain_queue {
                return Err(Refusal::NotAQueue);
            }
            if f.destroy_request {
                return Err(Refusal::DestroyRequest);
            }
            if slot == Slot::Receive && f.send_once != 0 {
                return Err(Refusal::SendOnce);
            }
            Ok(())
        }
    }
}

/// Refuses a record that carries one name twice: two slots would become
/// two handles to one right.
///
/// # Errors
/// [`Refusal::Duplicate`].
pub(crate) fn distinct(facts: &[SlotFacts]) -> Result<(), Refusal> {
    // Bounded by the constant (VER-1 cost); a record has at most
    // MAX_SLOTS rights, and more are the count check's to refuse.
    for (i, a) in facts.iter().enumerate().take(MAX_SLOTS) {
        for b in facts.iter().take(i) {
            if a.name == b.name {
                return Err(Refusal::Duplicate);
            }
        }
    }
    Ok(())
}

/// The slots `kinds` take, in wire order; `None` past [`MAX_SLOTS`].
pub(crate) fn wire_slots(kinds: &[WireKind]) -> Option<([Slot; MAX_SLOTS], usize)> {
    let mut out = [Slot::Send; MAX_SLOTS];
    let mut n = 0usize;
    for &k in kinds.iter().take(MAX_MESSAGE_HANDLES.saturating_add(1)) {
        let (s, c) = slots(k);
        for &one in s.iter().take(c) {
            *out.get_mut(n)? = one;
            n = n.checked_add(1)?;
        }
    }
    (kinds.len() <= MAX_MESSAGE_HANDLES).then_some((out, n))
}

/// VER-1 for handle acceptance (SEC-4 (b): "a per-OS Kani proof covers
/// handle acceptance").
#[cfg(kani)]
pub(crate) mod proofs {
    use super::*;

    /// Any kernel facts.
    pub(crate) fn any_facts() -> SlotFacts {
        SlotFacts {
            desc_type: kani::any(),
            name: kani::any(),
            disposition: kani::any(),
            type_bits: kani::any(),
            send_refs: kani::any(),
            kobject: kani::any(),
            plain_queue: kani::any(),
            send_once: kani::any(),
            destroy_request: kani::any(),
        }
    }

    /// Any slot.
    pub(crate) fn any_slot() -> Slot {
        match kani::any::<u8>() % 3 {
            0 => Slot::Send,
            1 => Slot::Receive,
            _ => Slot::Credit,
        }
    }

    /// A process that holds `refs` send references and maybe the
    /// receive right on one name, `held_name`.
    pub(crate) struct OneHeld {
        pub(crate) held_name: u32,
        pub(crate) refs: u32,
        pub(crate) receiver: bool,
    }

    impl Held for OneHeld {
        fn send_refs(&self, name: u32) -> u32 {
            if name == self.held_name { self.refs } else { 0 }
        }
        fn receiver(&self, name: u32) -> bool {
            name == self.held_name && self.receiver
        }
    }

    /// Exactly the stated set passes: a port descriptor with a real name;
    /// for a send slot, a fresh send right (one reference, none held
    /// before) to a plain port, with at most this process's own receive
    /// right beside it; for a receive or credit slot, a receive right to
    /// a plain queue with no port-destroyed request and at most this
    /// process's own send references beside it, and for a receive slot
    /// no send-once right anywhere.
    /// Everything else is refused.
    #[kani::proof]
    fn acceptance_is_exactly_the_stated_set() {
        let f = any_facts();
        let slot = any_slot();
        let held = OneHeld {
            held_name: kani::any(),
            refs: kani::any(),
            receiver: kani::any(),
        };
        let ok = accept_slot(slot, &f, &held).is_ok();
        let port = f.desc_type == DESC_PORT && f.name != NAME_NULL && f.name != NAME_DEAD;
        let expected = port
            && match slot {
                Slot::Send => {
                    f.disposition == PORT_SEND
                        && f.kobject == KOBJECT_NONE
                        && held.send_refs(f.name) == 0
                        && f.type_bits & TYPE_SEND != 0
                        && (f.type_bits == TYPE_SEND
                            || (held.receiver(f.name) && f.type_bits == TYPE_SEND | TYPE_RECEIVE))
                        && f.send_refs == 1
                }
                Slot::Receive | Slot::Credit => {
                    f.disposition == PORT_RECEIVE
                        && f.plain_queue
                        && !f.destroy_request
                        && (slot == Slot::Credit || f.send_once == 0)
                        && f.send_refs == held.send_refs(f.name)
                        && (f.type_bits == TYPE_RECEIVE
                            || (held.send_refs(f.name) > 0
                                && f.type_bits == TYPE_RECEIVE | TYPE_SEND))
                }
            };
        assert!(ok == expected);
        kani::cover!(ok && slot == Slot::Send, "a send right accepted");
        kani::cover!(ok && slot == Slot::Receive, "a receive right accepted");
        kani::cover!(
            ok && slot == Slot::Credit && f.send_once > 0,
            "a credit port with reply rights out accepted"
        );
        kani::cover!(
            ok && slot == Slot::Send && f.type_bits != TYPE_SEND,
            "a send right to this process's own receiver accepted"
        );
    }

    /// `distinct` refuses exactly the records with a repeated name.
    #[kani::proof]
    #[kani::unwind(4)]
    fn distinct_refuses_repeats() {
        let facts = [any_facts(), any_facts(), any_facts()];
        let n: usize = kani::any_where(|&n: &usize| n <= 3);
        let r = distinct(&facts[..n]);
        let repeat = (n >= 2 && facts[0].name == facts[1].name)
            || (n >= 3 && (facts[0].name == facts[2].name || facts[1].name == facts[2].name));
        assert!(r.is_err() == repeat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `mach_port_type` bits for a send-once right and a dead name (P7).
    const TYPE_SEND_ONCE: u32 = 1 << 18;
    const TYPE_DEAD_NAME: u32 = 1 << 20;

    struct Nothing;
    impl Held for Nothing {
        fn send_refs(&self, _n: u32) -> u32 {
            0
        }
        fn receiver(&self, _n: u32) -> bool {
            false
        }
    }

    /// Holds the receive right of port 0x500, and 2 send references on
    /// port 0x600 (a sender end).
    struct Some2;
    impl Held for Some2 {
        fn send_refs(&self, n: u32) -> u32 {
            if n == 0x600 { 2 } else { 0 }
        }
        fn receiver(&self, n: u32) -> bool {
            n == 0x500
        }
    }

    fn send(name: u32) -> SlotFacts {
        SlotFacts {
            desc_type: DESC_PORT,
            name,
            disposition: PORT_SEND,
            type_bits: TYPE_SEND,
            send_refs: 1,
            kobject: KOBJECT_NONE,
            plain_queue: false,
            send_once: 0,
            destroy_request: false,
        }
    }

    fn recv(name: u32) -> SlotFacts {
        SlotFacts {
            desc_type: DESC_PORT,
            name,
            disposition: PORT_RECEIVE,
            type_bits: TYPE_RECEIVE,
            send_refs: 0,
            kobject: 0,
            plain_queue: true,
            send_once: 0,
            destroy_request: false,
        }
    }

    #[test]
    fn fresh_rights_pass() {
        assert_eq!(accept_slot(Slot::Send, &send(0x103), &Nothing), Ok(()));
        assert_eq!(accept_slot(Slot::Receive, &recv(0x207), &Nothing), Ok(()));
    }

    #[test]
    fn kernel_objects_and_wrong_rights_are_refused() {
        // A task port (kobject 2), a thread port (1), a memory entry (28).
        for k in [1, 2, 3, 28, 29] {
            let f = SlotFacts {
                kobject: k,
                ..send(0x103)
            };
            assert_eq!(
                accept_slot(Slot::Send, &f, &Nothing),
                Err(Refusal::KernelObject)
            );
        }
        let f = SlotFacts {
            type_bits: TYPE_SEND | TYPE_SEND_ONCE,
            ..send(0x103)
        };
        assert_eq!(accept_slot(Slot::Send, &f, &Nothing), Err(Refusal::Rights));
        let f = SlotFacts {
            type_bits: TYPE_DEAD_NAME,
            ..send(0x103)
        };
        assert_eq!(accept_slot(Slot::Send, &f, &Nothing), Err(Refusal::Rights));
        let f = SlotFacts {
            disposition: 18,
            ..send(0x103)
        };
        assert_eq!(
            accept_slot(Slot::Send, &f, &Nothing),
            Err(Refusal::Disposition)
        );
        let f = SlotFacts {
            plain_queue: false,
            ..recv(0x207)
        };
        assert_eq!(
            accept_slot(Slot::Receive, &f, &Nothing),
            Err(Refusal::NotAQueue)
        );
        let f = SlotFacts {
            desc_type: 1,
            ..recv(0x207)
        };
        assert_eq!(
            accept_slot(Slot::Receive, &f, &Nothing),
            Err(Refusal::NotAPort)
        );
        assert_eq!(
            accept_slot(Slot::Send, &send(NAME_DEAD), &Nothing),
            Err(Refusal::NotAPort)
        );
    }

    /// A receive right with a port-destroyed request would go back to
    /// whoever asked; one with send-once rights out lets their holder
    /// aim notifications at it (R8). A credit port may have its
    /// in-flight messages' reply rights.
    #[test]
    fn requests_and_send_once_rights_on_a_received_port() {
        let f = SlotFacts {
            destroy_request: true,
            ..recv(0x207)
        };
        for slot in [Slot::Receive, Slot::Credit] {
            assert_eq!(
                accept_slot(slot, &f, &Nothing),
                Err(Refusal::DestroyRequest)
            );
        }
        let f = SlotFacts {
            send_once: 1,
            ..recv(0x207)
        };
        assert_eq!(
            accept_slot(Slot::Receive, &f, &Nothing),
            Err(Refusal::SendOnce)
        );
        assert_eq!(accept_slot(Slot::Credit, &f, &Nothing), Ok(()));
    }

    #[test]
    fn a_right_this_process_already_held_is_refused() {
        // A second send reference: the port's send right was here before
        // (its bootstrap port, a reply port, an end it holds).
        let f = SlotFacts {
            send_refs: 2,
            ..send(0x103)
        };
        assert_eq!(
            accept_slot(Slot::Send, &f, &Nothing),
            Err(Refusal::AlreadyHeld)
        );
        // A sender end this process already holds.
        let f = SlotFacts {
            send_refs: 3,
            ..send(0x600)
        };
        assert_eq!(
            accept_slot(Slot::Send, &f, &Some2),
            Err(Refusal::AlreadyHeld)
        );
    }

    #[test]
    fn this_process_s_own_other_end_may_join() {
        // The sender of a channel whose receiver is here.
        let f = SlotFacts {
            type_bits: TYPE_SEND | TYPE_RECEIVE,
            ..send(0x500)
        };
        assert_eq!(accept_slot(Slot::Send, &f, &Some2), Ok(()));
        assert_eq!(accept_slot(Slot::Send, &f, &Nothing), Err(Refusal::Rights));
        // The receiver of a channel whose sender is here.
        let f = SlotFacts {
            type_bits: TYPE_SEND | TYPE_RECEIVE,
            send_refs: 2,
            ..recv(0x600)
        };
        assert_eq!(accept_slot(Slot::Receive, &f, &Some2), Ok(()));
        assert_eq!(
            accept_slot(Slot::Receive, &f, &Nothing),
            Err(Refusal::Rights)
        );
    }

    #[test]
    fn repeats_are_refused_and_slots_follow_kinds() {
        assert_eq!(distinct(&[send(1), recv(2)]), Ok(()));
        assert_eq!(distinct(&[send(1), recv(1)]), Err(Refusal::Duplicate));
        let (s, n) = wire_slots(&[WireKind::Sender, WireKind::Receiver]).unwrap();
        assert_eq!(&s[..n], &[Slot::Send, Slot::Credit, Slot::Receive]);
        assert!(wire_slots(&[WireKind::Receiver; 17]).is_none());
    }
}
