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

So `ledger::model` follows the ledger's code over fixed arrays, over the earlier harnesses' domain: three processes, six channels and two tickets. The three properties are proved on the model. The test `the_model_agrees_with_the_ledger` holds the model to the real ledger: 3,000 random sequences of 40 operations, comparing every result and every observable value after each step; a deliberately broken model fails it at once. The model leaves out bindings, quotas (this domain never reaches them, and unit tests cover them) and relays of more than one end. That the real ledger matches the model is tested, not proved.

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

It also has one known limit, from probe W3 (D5 review item 6):

- **a compromised holder can keep reading an end it gives away.** A well-behaved process cancels an end's reads before it relays the end (the M9 move protocol). A compromised process need not: it can relay an end while a `ReadFile` is still pending on it. The broker's `DUPLICATE_CLOSE_SOURCE` pull does not cancel that read, which keeps completing with records meant for the new holder (W3), and the broker cannot cancel another process's read (W8b). `HandleCount == 1` does not detect it, since a pending read is not a handle. The reach is confined to channels the process already held.

  Options considered, and their costs:

  1. **The broker splices every moved channel.** On a move, the broker makes a fresh pipe pair, keeps the old ends itself, and copies bytes between old and new for the channel's life. This removes the old holder from the data path, but the broker then proxies every message of every once-moved channel, forever: a large, permanent latency and CPU cost, and it abandons the direct-pipe design.
  2. **A fresh pipe per holder (rebuild on move).** On a move, the broker re-establishes the channel on a new pipe, which both ends' holders must adopt, and migrates any buffered and in-flight data. A move becomes a two-sided channel rebuild needing the receiver holder's cooperation and careful data migration; complex, and still has a window during the rebuild.
  3. **Accept the limit and document it (chosen for M1).** The exposure is a process stealing records on a channel it already belonged to, after it gives an end away — no worse in reach than what it could already read while it held the end, and reaching no channel it never held. Zero cost. The broker's other checks (one copy, the ledger, one container per child) are unaffected.

  The recommendation is option 3 for M1, revisited if a channel is ever moved between two mutually untrusting processes where the giver is assumed hostile.
