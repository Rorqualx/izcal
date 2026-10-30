# M1 Kani: memory limits and harness cost

Measured 25–26 October 2026 on hueyj (Intel Core i7-7700K, 8 threads,
31 GiB, Linux 6.8.0-147-generic), Kani 0.68.0 with CBMC 6.11.0 and
CaDiCaL 3.0.0, branch `substrate/m1e-pure`. Each run was the only Kani
job on the host, under `prlimit --as=<limit>`, timed with
`/usr/bin/time -v` (its "Maximum resident set size" is the largest
process, CBMC), with a 15-minute limit per harness.

## The address-space cap

`verify_os_input_power_round_trip` (before the layout split), run twice:

| `--as` limit | Result | Peak RSS | Wall time |
|---|---|---|---|
| 10 GiB | "Solver ran out of memory during propositional reduction" | 9,334,792 KiB (8.9 GiB) | 2:24 |
| 16 GiB | CBMC timed out (15 min) | 9,345,108 KiB (8.9 GiB) | 15:03 |

The out-of-memory failure was real memory use, not reserved address
space: resident memory reached 8.9 GiB, and CBMC's address space runs
about 1 GiB above that, which met the 10 GiB cap. With the cap raised,
the same run used no more memory but did not finish in 15 minutes, so a
higher cap would not have made it pass. Linux has no per-process cap on
resident memory short of a cgroup (not used on shared rigs), so
`--as` stays the cap; `tools/kani.sh` applies it when `KANI_AS_LIMIT`
is set. Peak RSS of passing runs, for scale: `hg-ipc` (8 harnesses)
6.3 GiB, `hg-ipc-gen` (41) 2.2 GiB, one `string<8192>` validator on 62
bytes 4.2 GiB.

## What made the hg-os-types harnesses expensive

| Harness kind | Cause, measured | Change | Result |
|---|---|---|---|
| Round trip, `OsCommand.launch` | Unwind bound 8,194 (the type's largest `N`) for a 22-byte input: the UTF-8 loop unrolled 8,194 times; not finished in 72 min at 7 GB | Bound by the input: `min(N, body)` per loop | 219 s |
| Unknown ordinal, `OsInput` | Every message validator explored on paths the ordinal rules out; timed out | Validators stubbed (any result) | 0.4 s, 10,589 steps |
| Round trip, `OsInput.power` | Symbolic length: every offset after the first variable field symbolic. Decode alone: 4 s; decode and encode: 57 s, 2.6 GiB; with the byte comparison: out of memory, at a fixed 14-byte length too | Split by layout: option presence, union case, string length and vec count fixed, contents free | One layout: 6.8 s, 30,356 steps; 5 of 6 layouts pass |
| Validator, two strings (`OsCommand.launch`, `post`) | Each string's UTF-8 check on a symbolic-length slice costs about 1.1 M steps and 4.2 GiB (one `string<8192>`, 62 bytes: 158 s); two in one formula exceed 10 GiB | Not changed (see below) | Open |

A byte-at-a-time UTF-8 check (experiment only, not committed) took the
`OsCommand.launch` validator from out of memory (2.5 M steps) to a pass
in 181 s at 1.7 GiB (463 k steps), and `validate_agrees_with_core` still
passes with it (12 s). It changes `hg-ipc`'s inventoried UTF-8 code, so
it waits for a decision.
