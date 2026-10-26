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

## The byte-at-a-time UTF-8 check (commit 11dcf0f)

`hg-ipc`'s validator now reads one byte per step. Measured with it:
`OsCommand.launch` and `post` validators pass (they ran out of memory
before); `validate_agrees_with_core` 12 s and the new six-byte proof
30 s; `hg-ipc` 9 of 9 and `hg-ipc-gen` 41 of 41 pass.

Throughput, `measure_validate_throughput` (release, 1 MiB inputs, three
runs, hueyj under a light booking at load 1.9 to 4), side by side with
the previous algorithm:

| Input | By byte (now) | By sequence (before) |
|---|---|---|
| ASCII | 900–1,010 MiB/s | 1,158–1,355 MiB/s |
| Mixed (Latin, euro, emoji) | 917–1,056 MiB/s | 854–1,009 MiB/s |
| CJK | 784–907 MiB/s | 562–658 MiB/s |

## The register round trip

Every layout of `OsCommand.register` runs out of memory, though all its
structural bytes are fixed (only the 8-byte id is free). Measured piece
by piece, each a separate harness on the fixed 25-byte layout:

| Harness | Steps | Time |
|---|---|---|
| Decode only | 52,175 | 4 s |
| Decode and encode, length only | 566,024 | 73 s |
| Decode, encode and compare bytes | 577,341 | out of memory |
| An empty `string<512>`, round trip | 201,771 | 13 s |
| `RegistrationKind::NotificationIdentity` (no payload), round trip | 238,953 | 18 s |
| `u64`, `Option<u64>` (none), `RegistrationOwner::Browser` | 7–10 k | under 1 s |

Encoding, not decoding, is the cost: an encoder reads back a value the
decoder built on the heap, and CBMC does not carry the fixed layout
through it. An empty string's encoder costs 200 k steps (its length and
pointer come from the decoded `String`), and the union's encoder costs
as much although the case is fixed, because every case's encoder (three
strings for `Launcher`) is in the formula. Register has two strings and
that union, so its encoders alone are 566 k steps, and the byte
comparison on top runs out of memory. (That the union's cost comes from
its other cases' encoders is an inference from the step counts, not
measured case by case.) `default_status` layouts 1, 2 and 4 also run
out of memory; they hold a decoded `Vec` of two-enum structs (no
strings), and were not measured piece by piece. `power` layout 0 (no call id, the `State` case: four
enums, no heap values) runs out of memory too, at 426 k steps; its cause
is not measured yet.

## The output buffer, and the Kani writer

`power` layout 0 has no decoded heap value, yet ran out of memory. Measured
piece by piece (fixed 21-byte layout):

| Harness | Steps | Result |
|---|---|---|
| Decode only | 19,798 | 0.8 s |
| Decode and encode, length check only | 416,754 | 32 s |
| Decode, encode and compare | 426,258 | out of memory |
| The same, with `Writer::with_capacity(21)` | 406,118 | out of memory |
| `PowerEvent::State` alone, round trip | 132,662 | 15 s |
| `Tri` alone / `Thermal` alone | 18,591 / 29,203 | 1–2 s |
| `vec<TypeDefault, 7>`, one element: decode / encode, length / full | 15,261 / 250,156 / 255,574 | 1 / 17 / 23 s |

The heap value is the encoder's output: every `put` goes through
`Vec::extend_from_slice` and its growth code, about 60–70 k steps a put,
whatever the capacity. With a fixed-buffer writer in its place (an
experiment, 96 bytes), the full layout-0 round trip took 89,512 steps
and passed (237 s, 8.4 GiB).

So under `cfg(kani)` `hg_ipc::wire::Writer` now appends to a fixed
buffer of `KANI_WRITER_CAPACITY` bytes, with the same `new`, `put` and
`into_bytes`. The trust boundary: the Kani proofs prove this crate's and
the generated encoders and decoders against that buffer; std is trusted
for `Vec::extend_from_slice` appending a slice, which is all the
production writer does. A put past the capacity fails an assertion, and
every generated round-trip harness checks at compile time that its
longest message fits. The `the_kani_buffer_appends_as_a_vec` test runs
20,000 random put sequences (empty puts, puts to exactly the capacity)
through both and compares the bytes, in every build.

The buffer's size matters to CBMC: at 128 bytes `power` layout 0 ran
out of memory again (92,788 steps); the capacity is 64, the largest
harness input plus its ordinal.
