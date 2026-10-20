# Izcal JavaScript engine: riscv64 as a first-class ISA

Compiled 19 October 2026 for the JS engine spec's draft 0.4, after the owner decided (19 October 2026) that riscv64 is first-class from the start, as HGBA-2 X-1 promised. Primary sources were preferred: the RISC-V ISA manual and profile specifications, the Linux kernel's documentation and mailing-list patches, vendor datasheets and announcements, distribution release notes, and peer-reviewed or preprint papers.
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary snippet, a vendor claim without data, a technical secondary source, or a mailing-list patch whose merge state is reported by press; **Low** = secondary source only (press, single user benchmark runs).

**Method notes.**
- No JavaScript engine source code was read or cited (M-1 part 3). The V8 and SpiderMonkey ports are described from blog posts, wikis and project announcements only. Linux kernel patches and documentation are OS interface descriptions, not engine source, and are cited as such.
- The ISA manual's CFI chapter (`modules/unpriv/pages/unpriv-cfi.adoc`) was read directly; the profile facts come from the profile specification's source text (`rva23-profile.adoc`, `profiles.adoc`).
- Arithmetic on address-space fit (Section 2) is this note's own.
- What could not be verified is listed in Section 8.

---

## 1. The profile: RVA23U64 as the baseline

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **RVA23 was ratified in October 2024**, with the vector extension V and the hypervisor extension H as the headline additions | — | [RISC-V International](https://riscv.org/blog/risc-v-announces-ratification-of-the-rva23-profile-standard/); [HPCwire](https://www.hpcwire.com/off-the-wire/risc-v-announces-ratification-of-the-rva23-profile-standard/) | 2024-10-21 | High |
| 1.2 | **RVA23U64 mandatory set**: RV64I, M, A, F, D, C, B (Zba, Zbb, Zbs), Zicsr, Zicntr, Zihpm, Ziccif, Ziccrse, Ziccamoa, Zicclsm, Za64rs, Zihintpause, Zic64b, Zicbom, Zicbop, Zicboz, Zfhmin, Zkt; new in RVA23: **V**, Zvfhmin, Zvbb, Zvkt, Zihintntl, **Zicond**, **Zimop**, **Zcmop**, **Zcb**, **Zfa**, Zawrs and Supm | — | [rva23-profile.adoc](https://github.com/riscv/riscv-profiles/blob/main/src/rva23-profile.adoc) | 2024 | High |
| 1.3 | **Optional in RVA23U64**: Zicfilp and Zicfiss (expansion options, so CFI is not mandatory); Zacas, Zabha, Zama16b, Zvbc (development options); Zfh, Zbc, Zvfh | — | same | 2024 | High |
| 1.4 | **RVA23S64** mandates Sv39 as the only paging mode (Sv48, Sv57 optional) and Zifencei, which the profile calls "the only standard way to support instruction-cache coherence"; a replacement tentatively named Zjid is in development | Sv39 only | same | 2024 | High |
| 1.5 | RVA22U64 mandated Zba, Zbb, Zbs, Zkt, Zicbom and Zicclsm but left V optional; RVA23 makes V mandatory | — | [profiles.adoc](https://github.com/riscv/riscv-profiles/blob/main/src/profiles.adoc) | 2022–2024 | High |
| 1.6 | **Ztso** is ratified but in no profile, so RVWMO is the memory model an engine must assume | — | [ISA manual](https://github.com/riscv/riscv-isa-manual) | — | High |
| 1.7 | RVB23 (ratified with RVA23) explicitly does not aim at one binary interface for distributions | — | [RVB23](https://docs.riscv.org/reference/rvb23/v1.0/index.html) | 2024-10-17 | High |

**Implications:** RVA23U64 is the only RISC-V profile that guarantees what the engine's lowering needs without CPU-feature dispatch: V for Wasm SIMD, Zba's `.uw` forms for decompression, Zicond for branchless selects and index masking, Zfa's `fcvtmod.w.d` for ToInt32, and Zimop/Zcmop, which make CFI instructions safe no-ops on hardware without CFI. One build per ISA keeps JS-D12's "one lowering per ISA independent of CPU features". RVA22 hardware (with or without V) is not a target.

## 2. Address space and the cage

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **User address space by paging mode** on Linux riscv64 | Sv39: 256 GiB (of 512 GiB); Sv48: 128 TiB; Sv57: 64 PiB | [kernel vm-layout](https://docs.kernel.org/arch/riscv/vm-layout.html) | 2026 | High |
| 2.2 | Linux 6.6 capped default `mmap` placement at 47 bits and treated the hint as an upper bound, because JITs keep data in high pointer bits; Linux 6.11 stopped restricting the search by the hint, while default top-down placement still starts below 2^47 | 47 bits | [linux-riscv, 2023](https://lists.infradead.org/pipermail/linux-riscv/2023-July/035493.html); [Sv57 doc patch](https://lkml.iu.edu/hypermail/linux/kernel/2401.2/04946.html); commit 2116988d5372 (v6.11, kernel) | 2023–2024 | Medium |
| 2.3 | **The 16 GB cage fits on Sv39**, the only mode RVA23 requires; what does not fit is many guard-elided Wasm memory reservations | 256 GiB user; a 16 GiB cage plus guards, the trusted region and a code region take about 20–24 GiB; 8 GiB guard reservations per Wasm 32-bit memory leave room for about 25 before libraries and stacks, about 20 in practice | This note's arithmetic | — | — |

**Implications:** One cage per process (JS-D17) is safe on every RVA23 device. Guard-elided Wasm memories need a per-process reservation count that depends on the paging mode; beyond it, memories use explicit bounds checks, which change speed, never results (an out-of-bounds access traps either way).

## 3. Control-flow integrity

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Zicfilp landing pads**: `lpad` is encoded in AUIPC's HINT space with rd = x0; an indirect jump or call must land on a **4-byte-aligned** `lpad`, or a software-check exception (cause 18) is raised; a landing pad may carry a **20-bit label** compared with bits 31:12 of **x7**; label 0 disables the check; `jalr` through x7 (and x1, x5 as returns) is a software-guarded branch exempt from the landing-pad requirement | 20-bit label; 4-byte alignment | [ISA manual, unpriv CFI](https://github.com/riscv/riscv-isa-manual/blob/main/modules/unpriv/pages/unpriv-cfi.adoc) | 2024 | High |
| 3.2 | **Zicfiss shadow stacks**: `sspush`, `sspopchk`, `c.sspush`, `c.sspopchk` and `ssrdp` are encoded in the Zimop/Zcmop may-be-operation space and "revert to their Zimop- or Zcmop-defined behavior when the Zicfiss extension is not implemented or … not activated", so a program built with them "can thus continue to operate correctly, but without backward-edge CFI"; `ssamoswap.w/.d` are **not** MOP-encoded and trap where Zicfiss is absent | — | same | 2024 | High |
| 3.3 | The shadow-stack pointer `ssp` is a **user read-write CSR** (address 0x011); writes "should be mostly limited to" context switches, unwinding, longjmp and green threads, guarded by a **token** saved on the shadow stack and checked before switching; a `sspopchk` mismatch is delivered as `SIGSEGV` with `SEGV_CPERR` | — | [kernel zicfiss doc](https://docs.kernel.org/arch/riscv/zicfiss.html) | 2026 | High |
| 3.4 | Shadow stacks are enabled per task with the arch-generic `PR_SET_SHADOW_STACK_STATUS` / `PR_LOCK_SHADOW_STACK_STATUS` prctls, normally by the dynamic loader | — | same | 2026 | High |
| 3.5 | **Linux user-space CFI for riscv was merged in Linux 7.0** (merge window February 2026), Kconfig `RISCV_USER_CFI`, boot parameter `riscv_nousercfi` | — | [Phoronix, 30 Jan 2026](https://www.phoronix.com/news/RISC-V-User-Space-CFI); [Phoronix, 16 Feb 2026](https://www.phoronix.com/news/Linux-7.0-RISC-V) | 2026 | Medium |
| 3.6 | The same series implements `map_shadow_stack()` for riscv; the kernel needs no token because user mode can write its own with `ssamoswap`, and a flag requests one at the base for portability | — | [v16 patch](https://lists.infradead.org/pipermail/linux-riscv/2025-May/070968.html); [v26 patch](https://lkml.iu.edu/2512.1/07736.html) | 2025 | Medium |
| 3.7 | **No shipping silicon with Zicfiss or Zicfilp was found**; the implementations found are research ones (CVA6-CFI) and emulation | 0 shipping parts found | [arXiv 2602.04991](https://arxiv.org/pdf/2602.04991) | 2026-02 | Medium |
| 3.8 | GCC Zicfiss code generation was committed in early 2025; glibc's `--enable-cfi` series was at v5 in June 2026 | — | [GCC patch](https://patchwork.sourceware.org/project/gcc/patch/20250116081334.95540-1-monk.chiang@sifive.com/); [glibc v5](https://patchwork.sourceware.org/project/glibc/patch/20260628070241.88310-2-jesse.huang@sifive.com/) | 2025–2026 | Medium |
| 3.9 | RISC-V has **no pointer authentication**; pointer masking (Supm, mandatory in RVA23U64) ignores high address bits and is off by default in Linux 6.13's `PR_SET_TAGGED_ADDR_CTRL` support | — | [Phoronix, Linux 6.13](https://phoronix.com/news/Linux-6.13-RISC-V); 1.2 | 2024 | Medium |

**Implications:** The engine can emit landing pads and shadow-stack pushes and pops in every generated function on every RVA23 device at no correctness cost, and enforcement switches on wherever silicon and the kernel provide it. Backward-edge protection on riscv64 is Zicfiss alone (no PAC). Because `ssp` is user-writable, the machine-code validator, not the hardware, must keep shadow-stack switching inside the substrate's stub (JS-D19's rule restated per ISA). Until silicon ships, the JS-D19 discipline on riscv64 can be exercised only under emulation.

## 4. Hardware and CI

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **SpacemiT K3** (X100 cores, 8 at 2.4 GHz) claims RVA23 with RVV 1.0 and ships on boards (K3 Pico-ITX, Milk-V Jupiter 2); vendor claim >9 SPECint2006/GHz | Phoronix: 2.1× SiFive P550 single-thread PHP, behind the Raspberry Pi 500+; one user Geekbench 6 run, 347 single-core | [K3 datasheet](https://cdn.sipeed.com/public/k3/Introduction%20to%20K3%20Series%20Chips.pdf); [Phoronix](https://www.phoronix.com/review/spacemit-k3-pico-itx); [GB6](https://browser.geekbench.com/v6/cpu/18694031) | 2026-05 | Medium (vendor), Low (GB6) |
| 4.2 | SpacemiT K1 (BPI-F3) is RVA22 plus 256-bit RVV 1.0, in-order dual issue | — | [Banana Pi docs](https://docs.banana-pi.org/en/BPI-F3/SpacemiT_K1) | — | Medium |
| 4.3 | **Tenstorrent Ascalon** (RVA23 IP) licensable since December 2025, claims >22 SPECint2006/GHz; **XuanTie C930** shipping since 2025 and C950 IP announced March 2026, both RVA23 | Vendor claims | [Tenstorrent](https://tenstorrent.com/newsroom/tenstorrent-announces-availability-of-tt-ascalon); [CNX](https://www.cnx-software.com/2026/03/25/alibaba-xuantie-c950-a-powerful-rva23-complaint-64-bit-risc-v-core-for-edge-ai-computing/) | 2025–2026 | Medium |
| 4.4 | UltraRISC UR-DP1000 (Milk-V Titan) is "RVA23 excluding V"; Sophgo SG2044 has RVV 1.0 but no RVA23 claim was found; SG2042 has RVV 0.7.1 | — | [CNX](https://cnx-software.com/2026/01/12/milk-v-titan-a-329-octa-core-64-bit-risc-v-mini-itx-motherboard-with-a-pcie-gen4-x16-slot/); [arXiv 2508.13840](https://arxiv.org/html/2508.13840v1); [arXiv 2309.00381](https://arxiv.org/html/2309.00381v2) | 2023–2026 | Medium |
| 4.5 | **RISE RISC-V Runners**: free GitHub Actions runners on real hardware since 24 March 2026 (`ubuntu-24.04-riscv`), on TH1520 boards, not RVA23; an `ubuntu-26.04-riscv` label waits for RVA23 hardware | — | [RISE](https://riseproject.dev/2026/03/24/announcing-the-rise-risc-v-runners-free-native-risc-v-ci-on-github/); [six weeks in](https://riseproject.dev/2026/05/12/rise-risc-v-runners-six-weeks-in/) | 2026 | High |
| 4.6 | Scaleway Elastic Metal RV1 runs TH1520 (C910-class) boards | — | [Scaleway](https://www.scaleway.com/en/news/scaleway-launches-its-risc-v-servers-in-the-cloud-a-world-first-and-a-firm-commitment-to-technological-independence) | 2024 | High |

**Implications:** One RVA23 part with RVV 1.0 is buyable in volume today (K3); no hosted CI runs RVA23. Izcal must own its riscv64 lab and per-change hardware pool, and use emulation for scale (fuzzing and the DST farm) until RVA23 runners exist. Every performance figure on riscv64 is unmeasured by Izcal and marked †.

## 5. Distributions, Android and browsers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **Ubuntu requires RVA23 from 25.10**; 26.04 LTS (23 April 2026) supports it | — | [Canonical](https://canonical.com/blog/canonical-and-ubuntu-risc-v-a-2025-retro-and-looking-forward-to-2026); [Phoronix](https://www.phoronix.com/news/Ubuntu-25.10-To-Require-RVA23); [26.04](https://canonical.com/blog/canonical-releases-ubuntu-26-04-lts-resolute-raccoon) | 2025–2026 | High |
| 5.2 | Debian 13 (9 August 2025) is the first Debian release with official riscv64 support; Fedora riscv64 is still an alternative architecture | — | [Debian](https://www.debian.org/News/2025/20250809); [FOSDEM 2026](https://fosdem.org/2026/schedule/event/fedora-on-riscv) | 2025–2026 | High |
| 5.3 | Google removed RISC-V from the Android Common Kernel in April 2024 while saying it was not abandoning the ISA; RVA23 is described as the basis of the Android RISC-V ABI | — | [The Register](https://www.theregister.com/2024/05/01/riscv_support_android_pulled/); [Embedded Computing](https://embeddedcomputing.com/technology/open-source/risc-v-open-source-ip/risc-v-international-reveals-rva23-profile-ratification-in-time-for-the-risc-v-summit-north-america) | 2024 | Medium |
| 5.4 | SpiderMonkey gained an upstream riscv64 JIT and Wasm backend, contributed and maintained by PLCT Lab; no official Google Chrome riscv64 release was found | — | [SpiderMonkey newsletter](https://spidermonkey.dev/blog/2023/02/16/newsletter-firefox-110-111.html) | 2023 | High (blog) |

**Implications:** Linux distributions are the only riscv64 platform Izcal can target now; Android riscv64 waits for the Android work (HGBA-2 S-8). The "best shipping engine" on a riscv64 device is the best of the browsers that run there (Firefox and distribution Chromium builds), measured by Izcal.

## 6. Speculation, timing and instruction-cache coherence

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Spectre on RISC-V silicon**: SiFive P550 and XuanTie C910/C920 are vulnerable, including an arbitrary kernel-memory leak; the ISA "lacks a dedicated speculation barrier"; in-order U74 did not appear vulnerable | — | [Gerlach et al., USENIX Security 2026](https://cispa.de/en/research/publications/213214-spectre-on-risc-v-silicon-attacks-and-defenses-on-commercial-out-of-order-processors) | 2026 | Medium (abstract) |
| 6.2 | A speculation-barrier extension is proposed in an RVI task group; ratification not confirmed | — | [RVI wiki](https://lf-riscv.atlassian.net/wiki/x/A4DTEQ) | — | Low |
| 6.3 | **GhostWrite**: XuanTie C910/C920 vector stores write physical memory from user mode; the only mitigation is disabling V | — | [CISPA](https://cispa.de/en/ghostwrite) | 2024-08 | High |
| 6.4 | `fence.i` "does not ensure that other RISC-V harts' instruction fetches will observe the local hart's stores" | — | [ISA manual, Zifencei](https://github.com/riscv/riscv-isa-manual/blob/main/modules/unpriv/pages/zifencei.adoc) | — | High |
| 6.5 | **Linux's ABI for code modification**: user space should use the `riscv_flush_icache()` syscall, or set `PR_RISCV_SET_ICACHE_FLUSH_CTX` (per thread or per process) and then emit `fence.i` itself, so the kernel keeps the context coherent across migration | — | [kernel CMODX doc](https://docs.kernel.org/next/arch/riscv/cmodx.html) | 2024–2026 | High |
| 6.6 | Zkt (data-independent latency for a subset of integer instructions) is mandatory in RVA22U64 and RVA23U64 | — | [profiles.adoc](https://github.com/riscv/riscv-profiles/blob/main/src/profiles.adoc) | — | High |

**Implications:** Process isolation remains the only Spectre mitigation, as on the other ISAs; in-process hardening for co-located sites (JS-D20) cannot rely on a barrier, so riscv64 processes never co-locate sites. Code installation needs an explicit coherence protocol per thread, not only per hart.

## 7. Instruction-set facts the backend depends on

| # | Finding | Source | Conf. |
|---|---|---|---|
| 7.1 | **Zfa `fcvtmod.w.d`** "always rounds towards zero. Bits 31:0 are taken from the rounded, unbounded two's complement result, then sign-extended… ±inf and NaN are converted to zero"; rm must be RTZ; it "was added principally to accelerate the processing of JavaScript Numbers" (exactly ToInt32); plain `fcvt.w.d` saturates | [zfa.adoc](https://github.com/riscv/riscv-isa-manual/blob/main/modules/unpriv/pages/zfa.adoc); [f-st-ext.adoc](https://github.com/riscv/riscv-isa-manual/blob/main/modules/unpriv/pages/f-st-ext.adoc) | High |
| 7.2 | **Canonical NaN**: "If the result of a floating-point operation is NaN, it is the canonical NaN" (`0x7ff8000000000000` for D, `0x7fc00000` for F): sign clear, no payload propagation; loads, stores and moves preserve payloads; narrower values are NaN-boxed in wider registers | [f-st-ext.adoc](https://github.com/riscv/riscv-isa-manual/blob/main/modules/unpriv/pages/f-st-ext.adoc) | High |
| 7.3 | **Misaligned access (Zicclsm)** is mandatory for ordinary loads and stores but "might execute extremely slowly… assume their existence only for correctness, not for performance"; it does not cover AMOs | [profiles.adoc](https://github.com/riscv/riscv-profiles/blob/main/src/profiles.adoc) | High |
| 7.4 | Atomics: A is mandatory; Zacas and Zabha are only development options in RVA23, so byte and halfword atomics and compare-and-swap are LR/SC sequences on the baseline | [rva23-profile.adoc](https://github.com/riscv/riscv-profiles/blob/main/src/rva23-profile.adoc) | High |
| 7.5 | Branch reach: JAL ±1 MiB, B-type ±4 KiB, AUIPC+JALR ±2 GiB; 32-bit instructions plus 16-bit C and Zcb forms; **no flags register**, so overflow checks are explicit | [rv32.adoc](https://github.com/riscv/riscv-isa-manual/blob/main/modules/unpriv/pages/rv32.adoc) | High |
| 7.6 | The **Sail RISC-V model** is a formal specification of the ISA "adopted by RISC-V International", covering encodings, decoders and semantics, with generation of theorem-prover definitions (Isabelle, Rocq, Lean) and support for proofs about binary code (Islaris); it is a test-only input under OD-L2 | [sail-riscv README](https://github.com/riscv/sail-riscv) | High |

**Implications:** Decompression is two instructions (`sh2add.uw`, then a load at −4), the same count as AArch64. Tagged small-integer addition needs an explicit overflow check (about three instructions instead of one plus a branch), a per-ISA cost to measure, not a reason to change the tag. ToInt32 is one instruction. Under JS-D12 (a), riscv64's canonical NaN is covered by canonicalization at observable boundaries. Possibly misaligned accesses need an alignment test and a split path. Q-JS-4 is answered for riscv64: the Sail model plays the role Arm's ASL plays for AArch64.

## 8. Porting cost evidence, and what could not be verified

| # | Finding | Source | Conf. |
|---|---|---|---|
| 8.1 | V8's riscv64 port was upstreamed in February 2021 by RIOS Lab, Futurewei and ISCAS/PLCT; its wiki says upstream changes "may break the RISC-V port", an ongoing maintenance cost; no person-year figure is published | [RIOS Lab](https://www.rioslab.org/2023/04/14/chrome-browser-v8-engine-porting/); [v8-riscv wiki](https://github.com/v8-riscv/v8/wiki) | Medium |
| 8.2 | SpiderMonkey's port began as a 2021 PLCT mentorship with two interns and was upstreamed in early 2023; no person-year figure is published | [Mozilla Discourse](https://discourse.mozilla.org/t/meta-project-porting-spidermonkey-to-risc-v-rv64gc/85538) | Medium |

**Implications:** No published figure sizes a JIT port; the spec derives riscv64's staffing from its own deliverables per track. Both ports were done by small external teams after the engines existed, and both carry a maintenance tail; Izcal's generated tiers (JS-D5) move most per-ISA work into the generator, the backend's rules and the validator.

**Could not verify.** The exact Zicfiss and Zicfilp ratification date (patches from September 2024 call them ratified); any shipping silicon with either extension; whether glibc's CFI series has merged; ratification of a speculation barrier; the final state of `membarrier` `SYNC_CORE` on riscv; Chrome's riscv64 status beyond the absence of an official build; an independent SPEC run on the K3; whether the Sail model already covers V, Zicfilp and Zicfiss in full (checked at JS-M0); hosted CI on RVA23 hardware.
