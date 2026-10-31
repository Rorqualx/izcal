# JS-M0 probe: one seed, one state, on three OSes

Measured 31 October 2026. This is the JS-M0 probe that M1's decision-point criterion now names (Stage 1 §3.3 and the M1 row, as amended in 0.14.9; JS spec §9.2, JS-M0 row). It is not a timing run, so the bookings were `heavy`, not `exclusive`.

## What runs

`js-probe` (`crates/hg-dst-sim/src/bin/js-probe.rs`) runs two reference-interpreter units over `hg-heap` inside the simulator:
- **`setup`** makes a target, a `WeakRef` to it, and a `FinalizationRegistry` whose cleanup callback is host function 7; it registers the target with a held value drawn from `Math.random`, and stamps `Date.now`.
- **`churn`** loops `floor(Math.random() * 900)` times, making an object and a string each turn, then derefs the `WeakRef`.

`Math.random` draws from the substrate's seeded entropy, and `Date.now` from the simulated clock. The seeded scheduler orders six rounds: each round is a task posted with a seeded delay. Odd rounds run `ClearKeptObjects` between `setup` and `churn`, so their targets are held only weakly while `churn` allocates. Collections start when the operations' canonical charges reach the trigger (4,000 units; R0-B10). Each canonical cycle records a `GC_COMPLETE` decision point, which the simulator writes into its trace.

The report line is the `hg-dst` line plus `state=`, a hash of:
- each round's deref result;
- each cleanup job and its result;
- each host-function call and its arguments;
- the heap's address-independent digest and the cycle count.

`JS_PROBE_EXPLAIN=1` prints each item the hash covers. For seed 1 it shows two rounds whose target was cleared (`deref Undefined`), their two cleanup calls with the held values, and five cycles.

## Results

Built at `dffc55e` (`js/m0-probe`), release, rustc 1.99.1 on every host.

| Host | OS, ISA | `hg-dst repeat --seed 1 --times 1000` | seed 1 | seed 2 |
|---|---|---|---|---|
| Huey the Sixth (local) | macOS 27.0.1, AArch64 | `distinct=1` for `--stress none`, `move` and `safepoint` | `trace=c439ef76d31b5da5 state=63d8ca16c293f0f2` | `trace=37bb313951a5ce55 state=8c3ec24730e6e53a` |
| Huey the Destroyer (hueyj) | Linux 6.8.0, x86-64 | `distinct=1` for `none` and `move` | `trace=c439ef76d31b5da5 state=63d8ca16c293f0f2` | `trace=37bb313951a5ce55 state=8c3ec24730e6e53a` |
| Huey the Sojourn (rig253) | Windows 11 build 26300, x86-64 | `distinct=1` for `none` and `move` | `trace=c439ef76d31b5da5 state=63d8ca16c293f0f2` | `trace=37bb313951a5ce55 state=8c3ec24730e6e53a` |

Every repeat's one line was `steps=8 time_ns=20050927 crashes=0 trace=c439ef76d31b5da5 state=63d8ca16c293f0f2`.

**The pass criteria (JS-M0 plan, Phase 7):**
- **1,000 runs of a seed give one state hash:** PASS on all three hosts.
- **The hash does not depend on where objects live:** PASS. The moving test collector (`move`, every object on every allocation) and the GC-stress mode (`safepoint`) give the same hashes as `none`.
- **Seeds that change the decisions give different hashes:** PASS. Seeds 1, 2 and 3 give three traces and three states.
- **The GC decisions are in the trace:** PASS. With the trigger raised so that no cycle runs (`--trigger 1000000000`), seed 1's trace changes from `c439ef76d31b5da5` to `66be02e363bd3939`.
- **Identical on macOS, Linux and Windows:** PASS, across two ISAs.

**Open.** The recording header's `DecisionLogRef` from real engine entries (plan Phase 0, step 3) is not yet filled; the decisions are in the trace hash, not yet in a replayable recording. The corpus-page version of this probe is JS-50's first run at JS-M1.

## Evidence

- **hueyj:** booking b-954e54 (heavy, 15:15–16:15), build in a `systemd-run --user --scope -p MemoryMax=6G -p MemorySwapMax=0` scope; released at the end of the run.
- **rig253:** booking b-f6ee76 (heavy, 15:15–16:15); released at the end of the run.
- `rig status` reported both hosts unreachable over its own ssh, while direct ssh worked; the bookings were made without its memory check.
