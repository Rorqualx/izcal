//! The probe's two jobs (bytecode note Section 7): WeakRef and
//! FinalizationRegistry across ClearKeptObjects, with canonical cycles at
//! decision points (R0-B11), and the stress modes of the moving collector
//! giving the same results and the same heap digest (JS spec 6.5).

use hg_heap::Stress;
use hg_js_ops::support::HostValue;
use hg_js_runtime::JobError;

use crate::harness::*;

const PROBE: &str = r#"
.function setup params=0 registers=8
.const k0 key "weakRef"
.const k1 key "registry"
        LoadNull
        StoreReg r0                           ; r0 = null, the prototype
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r1                           ; r1 = target
        CallIntrinsic %WeakRefCreate, r1:1
        StoreReg r2                           ; r2 = WeakRef (the target is kept until ClearKeptObjects)
        LoadInt #7
        StoreReg r3
        CallIntrinsic %CreateHostFunction, r3:1
        StoreReg r3                           ; r3 = cleanup callback: embedder function 7
        CallIntrinsic %FinalizationRegistryCreate, r3:1
        StoreReg r3                           ; r3 = registry
        Copy r1, r4                           ; r4 = target
        LoadInt #1
        StoreReg r5                           ; r5 = held value 1
        LoadUndefined
        StoreReg r6                           ; r6 = no unregister token
        CallIntrinsic %FinalizationRegistryRegister, r3:4
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r7                           ; r7 = holder
        LoadReg r2
        SetNamed r7, k0, s?
        LoadReg r3
        SetNamed r7, k1, s?
        LoadReg r7
        Return
.end

.function churn params=1 registers=3
.const k0 key "weakRef"
        LoadNull
        StoreReg r0
        LoadInt #0
        StoreReg r1                           ; i = 0
loop:   CallIntrinsic %ObjectCreate, r0:1     ; garbage: dict_units each
        LoadReg r1
        Inc s?
        StoreReg r1
        LoadInt #2000
        Lt r1, s?                             ; i < 2000
        BranchFalse @done
        LoopBack @loop, s?
done:   GetNamed a0, k0, s?                   ; the WeakRef
        StoreReg r2
        CallIntrinsic %WeakRefDeref, r2:1
        Return
.end
"#;

/// What the probe observes: the second job's deref, the cleanup jobs the
/// host was asked for, the callbacks they made, and the heap digest.
#[derive(Debug, PartialEq)]
struct Seen {
    deref: Result<HostValue, JobError>,
    jobs: usize,
    callbacks: u32,
    calls: Vec<Call>,
    digest: u64,
    cycles: u64,
}

fn probe(stress: Stress, clear_between: bool) -> (Seen, u64) {
    let units = assemble_auto(PROBE);
    let after = with_agent(stress, 1000, |m, a, host| {
        let holder = run_unit(m, a, host, &units[0], &[]).expect("setup");
        if clear_between {
            // The host ends the synchronous sequence: ClearKeptObjects.
            a.clear_kept_objects(m);
        }
        let deref = run_unit(m, a, host, &units[1], &[holder]).map(|h| a.host_value(m, h).unwrap());
        // The host runs the cleanup jobs it was asked to enqueue.
        let mut callbacks = 0;
        for job in host.cleanup_jobs.clone() {
            callbacks += a.run_cleanup_job(m, host, job).expect("cleanup");
        }
        (deref, callbacks)
    });
    let (deref, callbacks) = after.value;
    let seen = Seen {
        deref,
        jobs: after.host.cleanup_jobs.len(),
        callbacks,
        calls: after.calls,
        digest: after.digest,
        cycles: after.cycles,
    };
    (seen, after.stats.moved)
}

#[test]
fn a_weak_target_dies_after_clear_kept_objects_and_its_cleanup_runs() {
    let (s, _) = probe(Stress::None, true);
    // 9.10: once ClearKeptObjects has run, the target is only weakly held,
    // so a canonical cycle in the second job clears it; deref is undefined.
    assert_eq!(s.deref, Ok(HostValue::Undefined));
    assert!(s.cycles >= 1, "{s:?}");
    // 9.13: one cleanup job; it calls function 7 with the held value 1.
    assert_eq!(s.jobs, 1);
    assert_eq!(s.callbacks, 1);
    assert_eq!(s.calls, vec![(7, vec![HostValue::Number(1.0)])]);
}

#[test]
fn a_kept_target_survives_until_clear_kept_objects() {
    // Without ClearKeptObjects the target is in [[KeptAlive]] (AddToKeptObjects
    // in WeakRef's constructor), so the cycles keep it: deref is the object,
    // and no cleanup is enqueued.
    let (s, _) = probe(Stress::None, false);
    assert_eq!(s.deref, Ok(HostValue::Object));
    assert!(s.cycles >= 1);
    assert_eq!((s.jobs, s.callbacks), (0, 0));
    assert!(s.calls.is_empty());
}

#[test]
fn the_stress_modes_give_the_same_results_and_digest() {
    for clear in [true, false] {
        let (plain, _) = probe(Stress::None, clear);
        for stress in [Stress::MoveOnAlloc, Stress::MoveAtSafepoint] {
            let (seen, moved) = probe(stress, clear);
            assert_eq!(seen, plain, "{stress:?}, clear {clear}");
            // The stress mode really moved objects.
            assert!(moved > 1000, "{stress:?} moved only {moved}");
        }
    }
}

#[test]
fn every_opcode_program_is_stable_under_moving() {
    // A program that touches strings, ropes, heap numbers, objects and
    // properties, run with and without the moving collector.
    let consts = ".const k0 key \"s\"\n.const k1 string \"abcdefgh\"\n.const k2 number 0.25\n.const k3 key \"n\"";
    let body = " LoadNull\n StoreReg r0\n CallIntrinsic %ObjectCreate, r0:1\n StoreReg r1\n\
        LoadConst k1\n StoreReg r2\n LoadInt #0\n StoreReg r3\n\
        top: LoadReg r3\n StoreReg r4\n LoadConst k2\n Add r4, s?\n StoreReg r4\n\
        LoadReg r2\n StoreReg r5\n LoadReg r4\n Add r5, s?\n SetNamed r1, k0, s?\n\
        LoadReg r4\n SetNamed r1, k3, s?\n\
        LoadReg r3\n Inc s?\n StoreReg r3\n LoadInt #50\n Lt r3, s?\n BranchFalse @end\n LoopBack @top, s?\n\
        end: GetNamed r1, k0, s?";
    let want = eval_with(Stress::None, consts, body, false);
    assert_str(want.clone(), "abcdefgh49.25");
    for stress in [Stress::MoveOnAlloc, Stress::MoveAtSafepoint] {
        assert_eq!(eval_with(stress, consts, body, false), want, "{stress:?}");
    }
}
