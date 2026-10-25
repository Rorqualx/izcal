# M1 phase D3: the Linux IPC transport, first measurements

Measured 25 October 2026 on hueyj. This is the first baseline for the transport in `crates/hg-substrate-os/src/ipc/linux.rs`. No spec sets a raw IPC latency target yet: the nearest are ED-32's 1 ms† p99 for the broker's synchronous IME answers, which come from a local snapshot rather than a channel round trip, and the app host's in-process host-call budget. These numbers are evidence for whichever spec sets one, not a pass or fail against one.

## Setup

| | |
|---|---|
| Host | hueyj: Intel Core i7-7700K, 4 cores and 8 threads at 4.2 GHz, Linux 6.8.0-147-generic |
| Build | `rustc 1.99.1`, `--release`, branch `ipc/d3-linux` |
| Booking | `rig check OK: hueyj kind=exclusive session=m1-substrate \| load 0.43/cpu (3.46 / 8 cpus) \| bookings: b-7418f5 exclusive cpu by m1-substrate until 05:10 \| game lock: free q0 \| 2026-10-25T04:10:12-06:00` |
| Background | The host's own services ran throughout (load 3.46 on 8 CPUs); they were not stopped |
| Test | `cargo test --release -p hg-substrate-os --lib measure_round_trip_and_throughput -- --ignored --nocapture` |

The test starts a second process (the test binary started again with `IZCAL_D3_CHILD=echo`) and talks to it over the channels handed to it at start, so every number crosses a process boundary and the full receive path. That path includes the IO thread, the control-data parser, the decision, the credit records and a delivery task on the receiver's sequence.

- **Round trip:** a 64-byte message sent and echoed back, one at a time, 10,000 times.
- **Throughput:** 2,000 messages of 64 KiB echoed, with up to 32 in flight, counting bytes in both directions.

## Results

| Run | Round trip p50 | p99 | max | 64 KiB echo (both ways) |
|---|---|---|---|---|
| 1 | 43.5 µs | 124.8 µs | 4,606 µs | 2,391 MiB/s |
| 2 | 35.2 µs | 69.0 µs | 1,451 µs | 3,330 MiB/s |
| 3 | 37.7 µs | 73.7 µs | 1,398 µs | 3,060 MiB/s |

Run 1 is the first after the build and includes warm-up. Runs 2 and 3 agree within 10%. The maxima of 1.4–4.6 ms are single outliers that coincide with the host's other load; they were not investigated further.

## Also measured: `Closed` is eventual

After a receiver is dropped, a send on its channel was accepted, and the message lost, in **6 of 40** runs of the transport's tests when they ran beside the tests that start child processes. It happened in **0 of 200** runs of the test alone, and in **0 of 40** full-suite runs without the child-starting tests.

The cause is process creation: a child being started by another thread holds a copy of every descriptor until it execs (posix_spawn(3), fork(2)), so the dropped socket is briefly still open. The next send reports `Closed`.

The transport documents this, and the test now checks that `Closed` follows within a deadline. Any process that starts children while dropping channel ends inherits this window. The broker does both, so it is an input to phase E (the launcher).

## Formal verification, same build

`tools/kani.sh` on hueyj: 68 of 68 proofs verified (`hg-ipc` 8, `hg-ipc-gen` 26, `hg-substrate-os` 34), with no unreachable cover property. It took 3,078 s (51 min) and peaked at 6.7 GB. The slowest proof is `ipc::unix::cmsg::proofs::a_step_keeps_the_invariant`, at 1,066 s.
