# M1 phase D5: the Windows IPC transport, first measurements

Measured 25 October 2026 on rig253. This is the first baseline for the transport in `crates/hg-substrate-os/src/ipc/windows.rs`. As with D3, no spec sets a raw IPC latency target yet, so these numbers are evidence for whichever spec sets one, not a pass or fail against one.

## Setup

| | |
|---|---|
| Host | rig253: AMD Ryzen AI Max+ 395, 16 cores and 32 threads, Windows 11 Pro (10.0.26300) |
| Build | `rustc 1.99.1`, `--release`, branch `ipc/d5-windows` at `7bfa455` |
| Booking | `rig check OK: rig253 kind=exclusive session=m1-substrate \| CPU% 2% (32 cpus) \| bookings: b-1f3af9 exclusive cpu by m1-substrate until 18:00 \| game lock: free q0 \| 2026-10-25T17:01:01-06:00` (run 1; runs 2 and 3 were checked the same way at 17:01:06 and 17:01:08) |
| Background | The host's own services ran throughout (2% CPU); they were not stopped |
| Test | `cargo test --release -p hg-substrate-os --lib measure_round_trip_and_throughput -- --ignored --nocapture` |

The test launches a second process (the test binary started again with `IZCAL_D5_CHILD=echo`) through the transport's own launcher, and talks to it over the launch channels. Every number therefore crosses a process boundary and the full receive path: the IO thread, the completion routines, the decision, the credit records and a delivery task on the receiver's sequence.

- **Round trip:** a 64-byte message sent and echoed back, one at a time, 10,000 times.
- **Round trip with a handle:** a 64-byte message carrying a channel's receiver end, echoed back with the end, 1,000 times. A trip costs two relays through the broker (one each way) and two redemptions, so it measures the ticket path that a Unix transport does not have.
- **Throughput:** 2,000 messages of 64 KiB echoed, with up to 32 in flight, counting bytes in both directions.

## Results

| Run | Round trip p50 | p99 | max | With a receiver end, p50 | p99 | 64 KiB echo (both ways) |
|---|---|---|---|---|---|---|
| 1 | 33.4 µs | 95.1 µs | 13,516 µs | 78.1 µs | 131.7 µs | 5,233 MiB/s |
| 2 | 33.0 µs | 94.2 µs | 12,667 µs | 77.5 µs | 130.0 µs | 5,328 MiB/s |
| 3 | 32.9 µs | 94.0 µs | 12,730 µs | 79.6 µs | 130.4 µs | 4,389 MiB/s |

The percentiles agree within 2% across the runs. Throughput varies more (4.4 to 5.3 GiB/s).

Every run has a maximum near 13 ms. It is one outlier in 10,000, and it recurs at about the same size each time, which an unrelated background spike would not explain. The section below finds its cause: it is the first trip, which waits out the child's start-up.

Moving a handle costs about 45 µs more per round trip at the median: two broker relays and two redemptions, about 11 µs each.

## After the review fixes (25 Oct 2026)

Measured again on rig253 under an exclusive booking (b-c98449), with `rig check` passing before each run. Each build was cleaned before it was measured, because extracting an older tree leaves file times that make cargo reuse the previous binary.

**The ~13 ms maximum is the child's start-up.** The test now prints every trip over 1 ms. In every run the only such trip was the first one (#0, 10.9 to 12.9 ms); the other 9,999 stayed under 1 ms. Waiting 500 ms after the launch removed it (0 trips over 1 ms, max 330 to 342 µs), so the first message had been waiting for the child to reach its echo loop: `launch` returns before then. The measurement now makes one untimed round trip before it starts timing.

| Build | Round trip p50 | p99 | max | With a receiver end, p50 | p99 | 64 KiB echo |
|---|---|---|---|---|---|---|
| `cbda210` (before M8 moved work off the IO thread) | 33.4 to 35.2 µs | 89.8 to 98.4 µs | 10.6 to 11.2 ms (first trip) | 81.4 to 83.3 µs | 130.5 to 133.5 µs | 4,688 to 5,416 MiB/s |
| `088de21` (relays classified off the IO thread) | 35.2 to 36.1 µs | 95.2 to 100.6 µs | 11.4 to 14.4 ms (first trip) | 129.5 to 149.8 µs | 202.0 to 280.7 µs | 3,980 to 5,223 MiB/s |
| `310bc85` (relays, redeems and binds off the IO thread) | 34.2 to 38.6 µs | 94.6 to 101.3 µs | 13.7 to 14.7 ms (first trip) | 172.8 to 211.1 µs | 278.2 to 302.5 µs | 4,250 to 5,025 MiB/s |
| `3ec0f88` (a thread per request, with the warm-up trip) | 33.2 to 36.2 µs | 82.7 to 84.4 µs | 130.9 to 151.5 µs | 180.2 to 183.7 µs | 270.1 to 291.8 µs | 4,081 to 5,181 MiB/s |
| `40ea55f` (one worker per child) | 33.6 to 36.5 µs | 94.9 to 98.1 µs | 135.3 to 206.4 µs | 104.0 to 105.3 µs | 155.3 to 182.9 µs | 3,869 to 4,664 MiB/s |

Three runs each. The plain round trip and throughput did not change beyond run-to-run variation.

**A trip that carries a handle got about 100 µs slower, and M8 was the cause.** Item 1 (M8) moved the kernel queries of each relay, redemption and bind a child asks for off the IO thread, so that a child whose pipe blocks those queries cannot stall the broker. Of a trip's two relays and two redemptions, two run for the child (its redemption of the end and its relay of it back); the broker handles its own locally. Moving the child's relays to a new thread per request added about 50 µs at the median (`cbda210` to `088de21`), and moving its redemptions and binds about 50 µs more (`088de21` to `310bc85`): about 50 µs per request for starting a thread and handing its result back. The end-closed record added after `310bc85` changed nothing measurable.

**One worker per child (`40ea55f`) brings it to 104 to 105 µs, not back to 82 µs.** A temporary probe (not committed) timed each of the 2,000 worker requests in three runs: the worker took 3.5 to 4.1 µs to wake once a request was queued, did the request's work in 7.5 µs (work the IO thread used to do itself), and its wake of the IO thread (an APC into `SleepEx`) took 4.4 to 5.0 µs to return there. The two wakes add about 8 µs per request, about 16 µs per trip of the 22 µs gap; the other 6 µs is not attributed. Any design that runs these queries off the IO thread pays the two wakes, so closing the rest of the gap needs a different design, not tuning this one.

**A candidate for later: the broker creates every pipe (decided 26 Oct: not now).** The per-child worker stays. Recorded here because the measurements above bound what it would buy:
- **The design.** The broker would make both ends of every channel: overlapped, message mode, one instance. Children would receive the ends through the move access mask, which lacks `FILE_WRITE_ATTRIBUTES`, so no child could make a synchronous handle or change an end's modes. The broker would keep a reference to each end and identify a relayed handle with `CompareObjectHandles`, which never waits (probe W8).
- **What it would remove.** Every classification query that can wait (W8), and with them the per-child workers and their two wakes per request: about 16 µs of the 22 µs gap per handle-carrying trip, measured above. The return to about 82 µs is an estimate, not measured.
- **What it would cost.**
  - Creating a channel becomes one round trip to the broker (about 35 µs at today's p50), unless children fetch channels ahead of time.
  - The ledger is keyed by an id the broker assigns, and `accept` checks identity and a handle count of 2 (the holder's handle plus the broker's).
  - The broker's handle table grows by two entries per live channel, which needs a creation quota.
  - Because the broker holds a copy of each end, a peer sees `Closed` only once the broker drops its copy (on the end-closed record, or when the holder dies). A slow broker would delay every channel's `Closed`.

## Also checked under the same build

- **Deterministic simulation:** the 60 `dst-smoke --ipc` seeds (0 to 59, stdout and stderr together) hash to `f06e8ea9488ab8cae1eb4054119ae52ddbebb41da8940781a9c0f57fe449911b` on rig253, the same value as on macOS and Linux.
- **Tests:** the Windows suite passes on rig253, 44 tests, including the transport's 15 run ten times in a row. Those 15 include children launched into their own AppContainer sandboxes. The workspace passes on rig253 and hueyj, and `clippy -D warnings` passes for the Windows, Linux and macOS targets.

## Formal verification

The Kani harnesses for the transport's pure core (`ipc::win`), run on hueyj one by one:

| Harness | Result | Time |
|---|---|---|
| `accept::proofs::no_pseudo_handle_passes` | verified | 3 s |
| `accept::proofs::accepts_exactly_the_stated_handles` | verified, 7 of 7 covers | 4 s |
| `frame::proofs::data_split_keeps_every_byte` | verified | 8 s |
| `frame::proofs::a_relay_decodes` | verified, 2 of 2 covers | 12 s |
| `frame::proofs::ctl_round_trips_32` | verified | 239 s |
| `frame::proofs::ctl_round_trips_40` | verified | 227 s |
| `decide::proofs::decisions_are_sound_early` | verified, 1 of 1 cover | 5 s |
| `decide::proofs::decisions_are_sound_0` | verified, 1 of 1 cover | 355 s |
| `decide::proofs::decisions_are_sound_1` | verified, 1 of 1 cover | 518 s |
| `decide::proofs::decisions_are_sound_2` | verified, 1 of 1 cover | 240 s |
| `frame::proofs::ctl_round_trips_24` | verified | 117 s |
| `frame::proofs::ctl_round_trips_56` | verified | 332 s |
| `frame::proofs::a_two_item_relay_decodes` | verified, 1 of 1 cover | 19 s |
| `accept::proofs::accept_matches_its_spec` | verified, 1 of 1 cover | 2 s |
| `ledger::proofs::relay_then_redeem_moves_one_end` | verified (over `model`) | 16 s |
| `ledger::proofs::a_redeemed_ticket_frees_its_carriers_slot` | verified (over `model`) | 7 s |
| `ledger::proofs::gone_then_sweep_reclaims` | verified (over `model`) | 12 s |

A single `decide` harness with the message length and the descriptor count both symbolic did not finish in 30 minutes. It is split here the way the Unix one is (D1): one harness per descriptor count, plus one for the rules that come before the decoder. The descriptor harnesses need unwind 17, because the prover's `memcmp` loops over the 16-byte ticket.

The rows from `ctl_round_trips_24` down were added after the design review (item 12). The 24- and 56-byte round trips cover the shortest records and a two-item record, and `accept_matches_its_spec` proves `accept` against an independent statement of its rules.

**The ledger proofs run on a model of the ledger, not on the ledger.** Kani could not check the ledger itself, and this was measured on hueyj. CBMC unrolls each loop inside `BTreeMap` to the proof's full bound, because those loops' trip counts depend on node lengths held in memory. One `place` on a concrete key grew from 0.40 million steps at bound 6 to 2.46 million at bound 14, and did not finish at bound 18. Symbolic keys cost no more than concrete ones. A fixed-capacity map under the real ledger fixed `place`, but `relay`, `redeem` and `gone` still ran CBMC out of 10 GiB.

So `ledger::model` follows the ledger's code over fixed arrays, over the earlier harnesses' domain: three processes, six channels and two tickets. The three properties are proved on the model. The test `the_model_agrees_with_the_ledger` holds the model to the real ledger: 3,000 random sequences of 60 operations, comparing every result and every observable value after each step; a deliberately broken model fails it at once. The model leaves out bindings, quotas (this domain never reaches them, and unit tests cover them) and relays of more than one end. That the real ledger matches the model is tested, not proved.

**W3's ledger state is in the model too** (merged on m1/integration, 26 Oct). W3 gave the ledger old pipes queued for a receiver and owed by holders (parks), charged to the channel's creator, a `lost` mark, and releases in `gone`, `close_end` and `sweep`. These change the per-creator charge, which `gone_then_sweep_reclaims` observes, so the model now carries each channel's queued and owed pipes by key (at most `MAX_OLD_PIPES`), and the differential test adds `queue_old`, `expect_park`, `park`, `cancel_park` and `take_old` to its operations and compares `has_old`, the old-pipe count and the exact per-creator charge after every step. Every fourth sequence mostly queues and parks, so the 16-pipe limit is reached (over 100 refusals per run). Removing the model's uncharge on release, or its release on the creator's death, fails the test. Pipe aliases are not modelled: the model names channels by index. A new harness, `old_pipes_are_charged_and_released`, proves on the model that two old pipes are charged to the creator and released when the receiver closes or its holder or creator dies, and that after the creator's death the receiver must end.

All 51 harnesses in `hg-substrate-os` were also run together, the way CI runs them (26 Oct, hueyj, 15 minutes per harness). 50 verified, and no cover went unreached. One Unix harness, `ipc::unix::cmsg::proofs::a_step_keeps_the_invariant`, hit the 15-minute limit. That is a timeout, not a counterexample, in code D5 does not touch.

## What Windows attests, and what it does not

These are properties of the transport as built, and the tests above exercise them.

The broker attests, for every handle it moves:

- its kind: a named pipe, the claimed end, message mode, one instance;
- that the sender kept no copy (the pipe object has one handle after the broker pulls it);
- which process holds each channel end, from its ledger;
- which process created a channel, from the pipe's name under that process's AppContainer;
- that a bound sender is used only by its grantee. When the sender moves anywhere else the binding breaks, and the receiver reports `Identity`, naming the bound peer.

Windows does not attest:

- **the sender of each message.** A pipe does not report who wrote a message. Faults a receiver detects therefore carry no sender (`Fault::sender` is `None`). Identity rests on the broker's record of who holds the channel's single sender end instead.
- **process generations.** Process IDs carry none, so `OsPeer` generation is 0, as on Linux.

It also has these limits:

- **A former holder's pending reads (W3, the bound).** A process can relay an end while reads are still pending on it, and the broker can neither cancel them nor stop them taking records (probes W3, W8b, W13d). Since W3 (below), every move puts the channel on a new pipe before the end is pulled, and nothing is written to the old pipe after the switch. A former holder can take records written before the switch, any number, never one after; a whole record taken gives Violation(Sequence), a split one Malformed; both unattributed.
- **Kernel memory pinned by pending reads (W13e, a residual).** Each pending read charges about 430 bytes of nonpaged pool to the process that posts it. Job memory limits don't bound pending-read nonpaged pool: under a 32 MB `ProcessMemoryLimit` or `JobMemoryLimit` a child still held 56.6 MB, and one process without a job held 262,144 reads, 113 MB. The job object documents no nonpaged-pool limit, and Izcal has no cap of its own. Any pipe end a sandboxed child holds allows this, not only a moved one.

The design review's item 6 had chosen to accept the first limit for M1 (option 3 of the three considered: splice every moved channel through the broker, a fresh pipe per holder, or accept and document). On 26 Oct the user chose a fresh pipe per move instead (option 2, with the broker keeping the old pipes), which W3 below builds.

## W3: a fresh pipe per move (26 Oct 2026)

**What changed.** After every move, the moved end and its peer are on a new pipe the broker makes (with every channel's flags; children never choose them):

1. The broker peeks the moving end (no close-source) and claims its channel (one relay per channel at a time).
2. It switches the peer first: locally if the broker or a ticket holds it; in a child, by `RebindValue` or `RebindId`, waiting at most 5 s for `Rebound`.
3. Only then does it pull the moving end, and checks with `CompareObjectHandles` that it is the object it peeked.
4. A receiver reads each old pipe to its end before its current one, so records keep their order and their `seq`. A moving receiver's old pipe stays with the broker until the next holder listens (`TakeDrains`).
5. Every data message ends with its own length, so a stale read that splits a record leaves a suffix the reader refuses.

**Limits.**
- At most 16† old pipes are queued or owed per channel. The move that would make the 17th is refused (code 19), and the channel ends in `Closed`.
- A peer that does not answer a rebind within 5 s fails the move (code 20).
- A pulled end that is not the object peeked is refused (code 21).
- An end whose file object does synchronous I/O is refused at the peek (code 22, below).

Old pipes count against the channel creator's quota, and are released when the receiver closes, when its holder or the creator dies, or when the channel is swept. († = set from first principles, not measured.)

**The three causes of the first build's 12 failing Windows tests** (measured with temporary tracing on rig253, one test at a time; each fix has a test that fails without it):
- A read on a pipe whose writer closed with nothing left in it fails as it starts (`ReadFileEx`, 109), not through a completion. A drain whose old sender had already closed was never dropped, so the receiver never read its current pipe. That caused 10 timeouts.
- The relay queued its `Rebind` records but did not wake the IO thread, so a `Rebind` was written only after the 5 s deadline had refused the move.
- A sender moving off a pipe the broker made at an earlier move lost its pipe's alias at the switch, and was refused as `Changed`. That caused the slow reader's 120 of 121.

**No relay holds a channel across a query that can wait (review finding).** The kernel's queries on a synchronous file object wait behind a pending read (W8), and a child chooses its own pipes' flags. Every such query now runs at the peek, on the child's own worker, before the channel is claimed; a synchronous end is refused there. After the pull only the object peeked is queried again. Probe W14 (`windows_facts.md`, round 6) shows why that is safe:
- a file object's mode cannot be changed (`NtSetInformationFile` returns 0xC000000D both ways);
- an overlapped end's queries return in 31 µs even with a read pending.

A pulled value that names another object is refused without any query.

**The stale-read conformance tests on the pre-W3 transport.** The three tests were ported to a scratch copy of `3f4afb7` (the last pre-W3 commit, where all 33 Windows tests passed) and run on rig253:

| Test | Pre-W3 (`3f4afb7`) | W3 |
|---|---|---|
| `an_old_holders_pending_read_gets_nothing_after_a_move` | **fails**: the stale read takes the first record sent after the move, and the new holder never receives it (no delivery in 20 s) | passes: the stale read ends with STATUS_PIPE_BROKEN, and the new holder gets the record |
| `a_record_split_by_a_stale_read_is_refused` | **fails**: the forged suffix is delivered as a message (`forged`) | passes: `Malformed`, unattributed |
| `a_record_taken_by_a_stale_read_shows_as_a_gap` | passes (the `seq` check predates W3) | passes: `Violation(Sequence)`, unattributed |

### Measurements

rig253 under an exclusive booking (b-99c04f), with `rig check` passing before every run (CPU 1 to 2%, `2026-10-26T13:25:20-06:00` to `13:26:04`). Release builds of `ipc/d5-w3` (`a68dfbc`, the measurement test included) and of `3f4afb7` with the same measurement tests ported, run alternately in the same window. Other sessions' light bookings were on the rig and its CPU stayed at 1 to 2%. The pre-W3 numbers are lower than the 33 µs and 104 µs above, which were measured on 25 Oct; today's runs are the comparison.

| | Pre-W3 (`3f4afb7`) p50 | p99 | W3 p50 | p99 |
|---|---|---|---|---|
| Round trip, 64 B (10,000) | 37.9 to 38.3 µs | 90.5 to 103.3 µs | 38.3 to 38.5 µs | 96.4 to 100.9 µs |
| Round trip with a receiver end each way (1,000) | 119.7 to 124.1 µs | 164.1 to 166.2 µs | 192.6 to 199.0 µs | 265.8 to 296.5 µs |
| Sender to a child and back (2 moves), receiver in the broker (500) | 131.0 to 151.8 µs | 201.7 to 207.2 µs | 201.8 to 231.5 µs | 314.0 to 364.9 µs |
| The same, receiver held by another child (500) | 150.0 to 175.8 µs | 224.7 to 246.1 µs | 316.4 to 357.8 µs | 475.0 to 546.7 µs |
| Listen with one old pipe queued, until it arrives (`TakeDrains`, 500) | n/a | n/a | 94 to 124 µs | 151 to 185 µs |

Two to four runs each. The plain round trip and 64 KiB throughput (3.2 to 4.3 GiB/s both before and after) did not change.

**The cost of a move.**
- **Peer in the broker:** about 35 µs per move at the median. That is half the 70 µs added to each two-move trip: a new pipe pair made and the peer switched in place.
- **Peer in another child:** about 80 µs per move. The extra is a `Rebind` and its `Rebound`, a round trip over the holder's control link.
- **Listen:** a receiver whose channel has an old pipe queued pays one `TakeDrains` round trip (94 to 124 µs at the median) before its first read. A receiver that never moved pays nothing.

### End checks (W3)

- **Clippy:** `clippy --workspace --all-targets -D warnings` passes for the Windows, Linux and macOS targets.
- **Windows transport suite:** 38 tests, run 10 times in a row on rig253 (all passed every time, 5.1 s each). Most children run in their own AppContainer sandboxes.
- **Workspace tests:** pass on rig253 and on hueyj.
- **Deterministic simulation:** the 60 `dst-smoke --ipc` seeds (0 to 59) still hash to `f06e8ea9488ab8cae1eb4054119ae52ddbebb41da8940781a9c0f57fe449911b`.
- **Kani, the frame harnesses:** all 11 verify. Run on hueyj (booking b-d8ea54, 27 Oct 03:54 to 04:07) at `2fa3a02`, one at a time when no other cbmc ran, `prlimit --as=10737418240`, 15-minute limit each, each on its own attributes. Steps are CBMC's symex steps.

| Harness | Result | Steps | Variables / clauses | Time |
|---|---|---|---|---|
| `ctl_round_trips_16` | verified | 823,638 | 2.74 M / 8.03 M | 66 s |
| `ctl_round_trips_24` | verified | 1,098,522 | 2.90 M / 8.38 M | 74 s |
| `ctl_round_trips_32` | verified | 1,250,159 | 3.01 M / 8.55 M | 87 s |
| `ctl_round_trips_40` | verified | 1,348,309 | 3.09 M / 8.77 M | 101 s |
| `ctl_round_trips_48` | verified | 1,425,248 | 3.20 M / 8.98 M | 98 s |
| `ctl_round_trips_56` | verified | 1,477,906 | 3.28 M / 9.21 M | 106 s |
| `a_relay_decodes` | verified, 3 of 3 covers | 808,132 | 0.38 M / 0.53 M | 36 s |
| `a_two_item_relay_decodes` | verified, 1 of 1 cover | 933,689 | 0.69 M / 1.12 M | 40 s |
| `the_w3_records_decode` | verified, 2 of 2 covers | 942,755 | 1.09 M / 2.64 M | 49 s |
| `data_split_checks_the_trailer` | verified | 19,944 | 0.08 M / 0.33 M | 3 s |
| `a_message_round_trips_and_no_suffix_is_accepted` | verified | 56,102 | 3.52 M / 14.59 M | 61 s |

  **Why the W3 build first ran out of memory, and the fix.** At `a68dfbc`, `ctl_round_trips_40`, `_48` and `_56` ran out of memory under the same limit, before solving (no counterexample). CBMC's statistics locate the cause in the encoder's `Vec` growth:
  - **Before W3** (`3f4afb7`): `ctl_round_trips_40` took 1,882,726 steps (5.49 M variables, 14.1 M clauses) and verified in 173 s.
  - **W3** (`a68dfbc`): it took 3,261,034 steps and ran out of memory. The encoder appended every field into a growing `Vec`, across W3's added record kinds.
  - **The fix** (`2fa3a02`): under `cfg(kani)` the encoder writes into a fixed `MAX_CTL_LEN` buffer with the same append semantics, as hg-ipc's wire `Writer` does, and writes reserved zeros as fixed slices instead of an iterator. That cut `_40` to 1,348,309 steps, and all six lengths now verify in under two minutes each.

  The unit test `the_kani_buffer_appends_as_a_vec` checks that the buffer gives the same bytes as a `Vec` on the same random puts, and a put past its capacity fails an assertion.

## The `decide` harnesses on integration (27 Oct 2026)

`decisions_are_sound_0`, `_1` and `_2` failed on `m1/integration` (at `46111ce` and `ac2d594`; they verified at `55bc624`, the `ipc/d5-w3` tip). Kani listed two unwinding assertions, at `decide.rs:142` and `:275` under unwind 17, but no check failed ("0 of 439 failed"). **This was not an unwinding failure.** Every check had status `ERROR`, not `FAILURE`, and CBMC printed "Solver ran out of memory during propositional reduction". Measured on hueyj (booking b-b1b6d2), Kani 0.68.0, one run at a time, `prlimit --as=10737418240`:

| Build | Steps | Variables / clauses | Peak RSS | Result |
|---|---|---|---|---|
| `55bc624` (`ipc/d5-w3`) | 243,310 | 4.89 M / 23.0 M | 9.24 GB | verified, 72 s |
| `ac2d594` (integration) | 259,497 | 5.77 M / 27.3 M | 9.30 GB | out of memory, 44 s |
| fix, `_0` | 80,319 | 2.90 M / 13.6 M | 3.10 GB | verified, 1 of 1 cover, 33 s |
| fix, `_1` | 85,139 | 2.92 M / 13.7 M | 5.22 GB | verified, 1 of 1 cover, 91 s |
| fix, `_2` | 88,526 | 2.93 M / 13.7 M | 5.23 GB | verified, 1 of 1 cover, 102 s |
| fix, `_undecodable` (new) | 9,136 | 0.01 M / 0.01 M | 0.43 GB | verified, 1 of 1 cover, 1 s |
| fix, `_early` | 17,347 | 0.05 M / 0.09 M | 0.43 GB | verified, 1 of 1 cover, 1 s |

**The cause.** The harness chose the decoder's result with `kani::any()`: `Ok(view)` with `ND` descriptors, or `Err`. CBMC merges the two into one value, so on the accept path the view's descriptor count is no longer the constant `ND`. Both descriptor loops (`decide_with`'s, at line 142, and the harness's own) were therefore unrolled to `MAX_MESSAGE_HANDLES` (16) iterations for every `ND`, 0 included. The CBMC log shows this at `55bc624` as well: 16 unrollings of each loop, plus the check at iteration 17. The loop bound did not change. Unwind 17 was already the real bound (16 handles plus one), and the unwinding assertions hold. What changed is the cost of each iteration. D6 added the region kinds: a third arm in `counters`, six wire kinds in `any_desc` with an `assume` for regions, and the region checks in the codec. Over 32 unrolled iterations that took the formula from 23.0 M to 27.3 M clauses, and the run went over the 10 GiB cap. At `55bc624` it had already peaked at 9.24 GB.

**The fix** (harness only; `decide` is unchanged). The decoder's outcome is a const parameter, `decisions_are_sound::<ND, DECODES>`. `_0`, `_1` and `_2` take the `Ok` view, so `decide_with`'s loop unrolls exactly `ND` times (CBMC log: 0, 1 and 2). The new `decisions_are_sound_undecodable` takes the `Err` and asserts a violation that changes nothing. Together the four cover the same inputs as the three before, with the same assertions. The harness's own loop over the accepted view still unrolls 16 times, because `Decision` merges `Accept` with `Violation`; at 5.2 GB peak this leaves room under the cap.

**All 24 `ipc::win` harnesses** were re-run after the fix, one at a time under the same limits, and all verify with every cover satisfied. Times: `accept` (3) under 2 s; `frame` (11) 3 to 129 s, `ctl_round_trips_56` the longest, at 6.7 GB peak; `ledger` (4) 22 to 127 s, `old_pipes_are_charged_and_released` peaking at 6.3 GB; `regions::the_registry_follows_its_state_machine` 1 s with 4 of 4 covers; `decide` (5) as in the table above.
