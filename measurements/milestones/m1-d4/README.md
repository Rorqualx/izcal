# M1 phase D4: the macOS IPC transport, first measurements

Measured 25 October 2026 on the local Mac. This is the first baseline for the transport in `crates/hg-substrate-os/src/ipc/macos.rs` (Mach ports). As for D3, no spec sets a raw IPC latency target yet, so these numbers are evidence for whichever spec sets one, not a pass or fail against one.

## Setup

| | |
|---|---|
| Host | Huey the Sixth: Apple M6 (Mac18,5), 12 cores (2 Super, 4 Performance, 6 Efficiency), 32 GB, macOS 27.0.1 (26A434) |
| Build | `rustc 1.99.1`, `--release`, branch `ipc/d4-macos` at `d41be6e` (every D4 review fix in) |
| Booking | `rig check OK: local kind=exclusive session=m1-substrate \| load 0.29/cpu (3.45 / 12 cpus) \| bookings: b-80c018 exclusive cpu by m1-substrate until 19:50 \| game lock: free q0 \| 2026-10-25T18:59:19-06:00` (one check before each run; the other two read the same but for the time, 18:59:19 and 18:59:20) |
| Background | The machine's own services ran throughout (load 3.45 on 12 CPUs); they were not stopped |
| Test | `cargo test --release -p hg-substrate-os --lib measure_round_trip_and_throughput -- --ignored --nocapture` |

The test starts a second process: the test binary started again with `IZCAL_D4_CHILD=echo`, spawned suspended with a bootstrap port of the parent's own, identified by its audit token and then resumed. The child takes its launch channels from the parent's answer to its hello. Both sides then listen with the peer bound (`listen_bound`), so every number crosses a process boundary and the full receive path. That path covers the IO thread's port set, the audit trailer, the acceptance and decision rules, the credit records on send-once reply rights, and a delivery task on the receiver's sequence.

- **Round trip:** a 64-byte message sent and echoed back, one at a time, 10,000 times.
- **Throughput:** 2,000 messages of 64 KiB echoed, with up to 32 in flight, counting bytes in both directions.

## Results

| Run | Round trip p50 | p99 | max | 64 KiB echo (both ways) |
|---|---|---|---|---|
| 1 | 12.1 µs | 19.8 µs | 62 µs | 4,849 MiB/s |
| 2 | 12.6 µs | 30.0 µs | 607 µs | 5,050 MiB/s |
| 3 | 12.1 µs | 26.5 µs | 204 µs | 4,939 MiB/s |

The three runs agree within 4% at p50 and in throughput. The p99 and the maxima vary more (20–30 µs, 62–607 µs). These are single outliers beside the machine's other load, not investigated further.

For scale, probe P14 (round 1, a noisy machine) measured a bare Mach round trip, with no transport, at 3.8–4.1 µs p50 for an empty message. The transport's 12 µs adds the IO thread's wake-up, the decision and the delivery task on each side. The D3 numbers in `../m1-d3/` were taken on hueyj, a different machine (Intel Core i7-7700K, Linux), so the two are not a comparison of the transports.

## Correctness evidence on the same build

- **macOS tests:** the workspace passed five runs in a row, 153 tests and 0 failures each, including the transport's 28. Those 28 cover hostile peers, children across processes, a child inside the sandbox and the review-fix tests. They also passed in the same binary ad-hoc signed with Enhanced Security (`com.apple.security.hardened-process`, `platform-restrictions-string` "2", `enhanced-security-version-string` "2"; keys as in probe P30). After the measurement, scenario C's child was found to die by SIGKILL in that build: an `EXC_GUARD` (`INVALID_NAME`) raised because the test child released a port name it had moved away behind the transport's back (`mach_facts.md` R13). The test was fixed in `80c91c7`, and the child now exits 0 in both builds.
- **Each fix's test fails without its fix:** with the fix undone in a scratch copy, the tests for review findings 1 (scenario C), 5, 7 and 8 failed. The finding-3 test passed without its fix: the defect released rights after the waiting message had taken them, which this test cannot observe. Finding 2's test failed before its fix on Linux. The data-port no-senders design shows in the commit history: six cross-process tests fail at `d33ef36` and pass from `484a0ff` on.
- **Determinism:** 60 seeds of `dst-smoke --ipc` (seeds 0–59, outputs concatenated) hash to `f06e8ea9488ab8cae1eb4054119ae52ddbebb41da8940781a9c0f57fe449911b`, as expected.
- **Linux:** the workspace tests passed on hueyj; clippy `-D warnings` passed for macOS, Linux and Windows.

## Formal verification, same code

The 11 D4 Kani harnesses (`ipc::mach`) on hueyj, one at a time under `prlimit --as=10G`, heavy booking `b-064729`: all verified, and every cover property was satisfied. Each took 1–97 s.
- `accept`: `acceptance_is_exactly_the_stated_set` (now with the send-once, port-destroyed and credit-slot rules) and `distinct_refuses_repeats`.
- `notify`: `classify_trusts_only_the_kernel_s_exact_notifications`, new.
- `wire`: `layout_reads_any_bytes`, `encode_then_layout` and `unframe_reads_any_bytes_and_inverts_frame`.
- `decide`: `decisions_are_sound` for 0/0, 1/1, 2/1, 2/2 and 3/2. It now asserts that a bound receiver's message from any other process is a WrongPeer violation naming the bound peer, whatever its layout, and that an accepted credit port has no more send-once rights than its messages in flight.

## Open

- **Kernel memory per queued message.** Queued inline data is charged to neither process's footprint (probe R3), so how much kernel memory a channel's queue pins is not measured. The credit window bounds it per channel (capacity × 64 KiB at most), but no number exists yet.
- **One `hg-substrate-sim` hang (not D4 code).** A sim suite binary built on hueyj at 16:15 hung six times in `ipc::a_bound_receiver_hears_only_the_peer_it_names`: once in the workspace run and five times alone. Both threads were waiting on a futex with no CPU. That binary was built from sources that already include the sim fix in `19d33df` (the branch head then, `2831d20`, descends from it), so the hang does not predate the fix. The binary was overwritten by a rebuild before it could be inspected, and the hang has not recurred: 0 in 350 runs that afternoon, then 0 in 1,000 runs of the whole suite under gdb on hueyj (ready to print every thread's backtrace on a 60 s timeout) and 0 in 1,000 runs on the Mac (ready to `sample` on a 60 s timeout). The cause is unknown.
