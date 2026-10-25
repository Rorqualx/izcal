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

Every run has a maximum near 13 ms. It is one outlier in 10,000, and it recurs at about the same size each time, which an unrelated background spike would not explain. It was not investigated. It is recorded here as an open question for the measurement that sets a target, not as a property of the transport.

Moving a handle costs about 45 µs more per round trip at the median: two broker relays and two redemptions, about 11 µs each.

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

A single `decide` harness with the message length and the descriptor count both symbolic did not finish in 30 minutes. It is split here the way the Unix one is (D1): one harness per descriptor count, plus one for the rules that come before the decoder. The descriptor harnesses need unwind 17, because the prover's `memcmp` loops over the 16-byte ticket.

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
