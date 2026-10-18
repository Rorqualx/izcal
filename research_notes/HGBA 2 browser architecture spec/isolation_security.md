# Hardware and Software Isolation Primitives for Browser Security Architecture (HGBA-2.0), state as of Oct 2026

Note on sourcing: items marked [verified 2026-10] were fetched or seen in search results during this research session. Items marked [ref] cite stable primary docs from prior knowledge (cutoff mid-2026) that were not re-fetched in this session. The report writer should treat [ref] items as reliable but spot-check version numbers.

## 1. Browser sandboxing today (Chromium, Firefox, V8 heap sandbox)

### Takeaway
Shipping browsers rely on OS process sandboxes (seccomp-bpf plus namespaces on Linux, restricted tokens, job objects and AppContainer/untrusted integrity on Windows, Seatbelt on macOS), site-per-process isolation, and in-process software sandboxes (V8 heap sandbox, RLBox/wasm2c). A compromised renderer is expected to have no direct filesystem, network, device, GPU or PCIe access. Everything goes through IPC to a privileged browser process.

### Cited Findings
- The V8 Sandbox is "a lightweight, in-process sandbox for V8." Its threat model assumes the attacker has arbitrary read/write inside the sandbox, and its job is to stop corruption reaching other process memory. It was published 4 Apr 2024 by Samuel Groß after about 3 years of work. From Chrome 123 it was added to the Chrome VRP. It is enabled by default on 64-bit x64/arm64 Chrome on Android, ChromeOS, Linux, macOS and Windows [verified 2026-10] — [V8 blog: The V8 Sandbox](https://v8.dev/blog/sandbox)
- V8 sandbox mechanics: in-sandbox pointers become offsets from the sandbox base (the example is 40-bit offsets for a 1 TB reservation, with sizes "currently limited to 32GB"). Out-of-sandbox objects are reached through an `external_ptr_t` pointer-table indirection, which works like an fd table. It needs about 1 TB of virtual address space and is a build-time option (`v8_enable_sandbox`). Overhead is about 1% or less on Speedometer and JetStream [verified 2026-10] — [V8 blog](https://v8.dev/blog/sandbox)
- Motivation: about 60% of in-the-wild Chrome exploits from 2021 to 2023 were V8 bugs, all starting as renderer memory corruption. Disabling the JIT would cost 1.5 to 10x or more on compute-heavy work and would only address about half of exploited V8 bugs. MTE was judged inadequate here because tags can leak via side channels and pointer compression leaves no room for tag bits [verified 2026-10] — [V8 blog](https://v8.dev/blog/sandbox)
- V8 has a "memory corruption API" and `--sandbox-testing` mode to emulate attacker primitives for fuzzing the sandbox boundary [verified 2026-10] — [V8 blog](https://v8.dev/blog/sandbox)
- Chromium sandbox design: renderers run with a restricted token, a job object, an alternate desktop/window station and low/untrusted integrity level on Windows. Each renderer can only talk to its broker (browser process) over IPC [ref] — [Chromium Sandbox design doc](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/design/sandbox.md)
- Linux Chromium uses a layered sandbox: "layer-1" namespaces (user/PID/net) and "layer-2" seccomp-bpf syscall filtering [ref] — [Chromium Linux sandboxing](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/linux/sandboxing.md)
- macOS Chromium uses the Seatbelt sandbox (sandbox profiles in SBPL) [ref] — [Chromium mac sandbox README](https://chromium.googlesource.com/chromium/src/+/HEAD/sandbox/mac/README.md)
- Site Isolation (one renderer process per site) was enabled by default on desktop Chrome in 2018 (Chrome 67) as a Spectre response [ref] — [Google Security Blog, Jul 2018](https://security.googleblog.com/2018/07/mitigating-spectre-with-site-isolation.html); [chromium.org Site Isolation](https://www.chromium.org/Home/chromium-security/site-isolation/)
- Reis et al. (USENIX Security 2019) report about 9–13% memory overhead for Site Isolation in Chrome desktop [ref] — [USENIX Security 2019, "Site Isolation: Process Separation for Web Sites within the Browser"](https://www.usenix.org/conference/usenixsecurity19/presentation/reis)
- Firefox site isolation ("Fission") was announced in May 2021 and enabled by default from Firefox 95 (Dec 2021) [ref] — [Mozilla Security Blog, May 2021](https://blog.mozilla.org/security/2021/05/18/introducing-site-isolation-in-firefox/)
- RLBox compiles C/C++ libraries to Wasm and then back to native code (wasm2c/lucet) for in-process isolation. It first shipped for the Graphite font library in Firefox 74 on Linux (Feb 2020) [ref] — [Mozilla Hacks, Feb 2020](https://hacks.mozilla.org/2020/02/securing-firefox-with-webassembly/); [USENIX Security 2020 paper](https://www.usenix.org/conference/usenixsecurity20/presentation/narayan)
- Firefox 95 (Dec 2021) extended RLBox to all platforms for Graphite, Hunspell, Ogg, Expat and Woff2 [ref] — [Mozilla Hacks, Dec 2021](https://hacks.mozilla.org/2021/12/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/)

### Inferences
- A renderer in HGBA-2.0 should be modeled with **zero ambient authority**: no device enumeration, PCIe config space, raw sockets or GPU MMIO. All of these are brokered. This matches every shipping browser.
- Three layers stack: OS process sandbox (inter-process), V8 sandbox (intra-process, JS heap) and RLBox (intra-process, third-party C libraries). A new architecture should keep all three, because each covers a different bug class.

### Gaps
- Exact 2026 Chromium renderer-on-Windows status (e.g. AppContainer usage for renderers vs. GPU/network services) was not re-verified this session.
- Whether the V8 sandbox has been declared "feature complete" by Oct 2026 was not confirmed.

## 2. Intel MPK/PKU and Arm POE

### Takeaway
PKU gives 16 protection keys per address space, switched by the unprivileged `WRPKRU` instruction, so any attacker with code execution can flip permissions unless WRPKRU sites are controlled (binary scanning plus CFI). V8 uses PKU for JIT write protection and CFI hardening. Arm's POE (FEAT_S1POE, Armv8.9/v9.4) is the arm64 equivalent and is upstream in Linux 6.12 behind the same pkeys syscalls.

### Cited Findings
- Linux pkeys: 16 keys per process, with permissions held in the per-thread PKRU register and set by unprivileged WRPKRU/RDPKRU. The kernel doc notes the feature was introduced on Skylake "Scalable Processor" server CPUs and slated for future non-server Intel and AMD parts [ref] — [Linux kernel docs: Memory Protection Keys](https://docs.kernel.org/core-api/protection-keys.html)
- V8 uses PKU for in-process isolation to enforce CFI for its JIT. The threat model is an attacker with arbitrary read/write in the process, and the goal is a "safe/isolated" thread environment with dedicated pkeys. Jeff Xu's kernel patches (May 2023, crediting Stephen Röttger) proposed extending PKU to protect memory mappings (which became `mseal`-related work) [verified 2026-10] — [LKML: Memory Mapping (VMA) protection using PKU, May 2023](https://lkml.rescloud.iu.edu/2305.1/07904.html); [LWN 932433](https://lwn.net/Articles/932433/)
- V8 source: `RwxMemoryWriteScope` has a PKU path (`SetWritable()`/`SetExecutable()`) that toggles JIT code page write access per thread [verified 2026-10] — [v8 code-memory-access-inl.h](https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/common/code-memory-access-inl.h)
- V8 CFI blog: the problem is split into forward-edge CFI, backward-edge CFI and "JIT memory integrity", meaning all data written to executable memory is validated [verified 2026-10] — [V8 blog: Control-flow Integrity in V8](https://v8.dev/blog/control-flow-integrity)
- Cloudflare Workers: V8 already uses PKU so that compiler threads can write JIT code while execution threads can only execute it. Cloudflare added internal V8 changes that use PKU to isolate isolates from one another as defense in depth [verified 2026-10] — [Cloudflare blog: Safe in the sandbox](https://blog.cloudflare.com/safe-in-the-sandbox-security-hardening-for-cloudflare-workers/)
- PKU bypasses: compromised code can bypass PKU via unprivileged instructions (WRPKRU/XRSTOR) or OS APIs. Defenses need CFI and/or syscall filtering (Garmr, arXiv 2110.04788; "You Shall Not (by)Pass!", EuroSys '22) [verified 2026-10] — [Garmr arXiv](https://www.arxiv.org/abs/2110.04788); [Telecom SudParis: You Shall Not (by)Pass!](https://www.inf.telecom-sudparis.eu/?p=1053)
- "PKU Pitfalls" (USENIX Security 2020) showed kernel-mediated attacks on ERIM/Hodor-style PKU sandboxes, e.g. via `/proc/self/mem`, `mprotect` and signal handling [ref] — [USENIX Security 2020, Connor et al.](https://www.usenix.org/conference/usenixsecurity20/presentation/connor)
- ERIM (USENIX Security 2019) gave the canonical design: binary inspection to ensure no unauthorized WRPKRU/XRSTOR, with low switching cost [ref] — [ERIM, USENIX Security 2019](https://www.usenix.org/conference/usenixsecurity19/presentation/vahldiek-oberwagner)
- RockJIT (prior JIT-hardening work on V8) reported about 14.6% overhead [verified 2026-10, search snippet] — [RockJIT paper](https://static.aminer.org/pdf/20170130/pdfs/ccs/slm6eypktda19zo0hfhrwgxrceujfzc2.pdf)
- Arm POE constrains permissions on memory regions. It was introduced in the 2022 VMSA enhancements and can be switched from EL0 without a syscall or TLB invalidation. Linux uses it to implement the pkeys syscalls, with x86-like default policy (deny all except pkey 0). The patch series v4 (May 2024) was merged for Linux 6.12 [verified 2026-10] — [arm64 POE patch v4](https://yhbt.net/lore/linux-mm/20240503130147.1154804-1-joey.gouly@arm.com/); [arm64 6.12 pull](https://lkml2.uits.iu.edu/hypermail/linux/kernel/2409.1/08571.html); [LWN 972323](https://lwn.net/Articles/972323)
- Kernel-space pkeys on arm64 ("kpkeys", POE at EL1) are RFC work by Kevin Brodsky (2025) [verified 2026-10] — [LKML kpkeys RFC v4](https://lkml.iu.edu/hypermail/linux/kernel/2504.1/06817.html)

### Inferences
- With 16 keys, PKU cannot give each site or iframe its own domain in a multi-tenant process. It works for a few coarse trust domains (JIT code, sandbox metadata, trusted runtime state), not as a replacement for process-per-site.
- PKU is only safe when paired with W^X, CFI (CET/IBT on x86, BTI/PAC on Arm) and syscall filtering that blocks `pkey_mprotect`, `/proc/self/mem` writes and similar paths.
- Client availability (inference, not verified this session): PKU is present on recent Intel client cores and AMD Zen 3+. POE needs Armv8.9/v9.4 cores, and its deployment in shipping phones and laptops in 2026 is unclear. Apple Silicon has its own proprietary APRR/SPRR mechanism, which is not exposed as POE.

### Gaps
- Authoritative per-SKU list of client CPUs with PKU (Intel client began around Tiger Lake/11th gen, per common reports, not verified here) and which 2025–26 Arm SoCs implement FEAT_S1POE.
- Whether Chrome on arm64 Linux/Android uses POE for V8 as of Oct 2026: no source found.
- Windows does not expose PKU to user mode in a documented API (prior knowledge, not verified here). This would limit PKU-based designs on Windows.

## 3. Arm MTE and Apple MIE/EMTE

### Takeaway
MTE is on shipping phones: Pixel 8 and later as opt-in (from 2023), and Apple A19 (iPhone 17/Air, Sept 2025) as always-on EMTE in sync mode for the kernel and over 70 processes. Speculative tag-leak attacks (TikTag, StickyTags) weaken standard MTE as a security boundary. Apple's design explicitly adds tag confidentiality to address this. V8's own team judged MTE insufficient to protect the JS heap.

### Cited Findings
- Apple Memory Integrity Enforcement (announced Sept 2025) combines secure typed allocators (including WebKit's libpas), EMTE in synchronous mode and Tag Confidentiality Enforcement. It is "always-on" on all iPhone 17 and iPhone Air models (A19/A19 Pro) and covers the kernel plus "over 70 userland processes" [verified 2026-10] — [Apple Security Research: Memory Integrity Enforcement](https://security.apple.com/blog/memory-integrity-enforcement/)
- EMTE vs. MTE: Apple rejects asynchronous mode. EMTE checks accesses from tagged into non-tagged memory, which original MTE did not. Arm released EMTE in 2022 with Apple's input [verified 2026-10] — [Apple Security Research](https://security.apple.com/blog/memory-integrity-enforcement/)
- Tag Confidentiality Enforcement: tag storage is protected by the Secure Page Table Monitor even against a compromised kernel, tag values cannot influence speculation, and the tag PRNG is reseeded frequently. Apple cites StickyTags and TikTag as showing Pixel MTE is vulnerable to speculative bypass [verified 2026-10] — [Apple Security Research](https://security.apple.com/blog/memory-integrity-enforcement/)
- Apple gives no numeric overhead, only "minimal performance impact". EMTE is restricted to smaller allocations within type buckets to control cost. Intra-allocation overflows are acknowledged as uncovered [verified 2026-10] — [Apple Security Research](https://security.apple.com/blog/memory-integrity-enforcement/)
- EMTE is available to third-party developers through Xcode "Enhanced Security" (WWDC 2025) on capable hardware [verified 2026-10] — [Apple Security Research](https://security.apple.com/blog/memory-integrity-enforcement/); [The Hacker News, Sept 2025](https://thehackernews.com/2025/09/apple-iphone-air-and-iphone-17-feature.html)
- Pixel 8/8 Pro (Oct 2023) were the first production handsets that let users enable MTE (developer option) [verified 2026-10] — [Project Zero, Nov 2023](https://projectzero.google/2023/11/first-handset-with-mte-on-market.html)
- In late 2023 GrapheneOS enabled MTE on Pixel 8 for everything except its Chromium-based browser, because "Chromium currently disables it" [verified 2026-10] — [GrapheneOS forum](https://discuss.grapheneos.org/d/8439-mte-support-status-for-grapheneos/40)
- TikTag (2024) showed speculative leakage of MTE tags on Pixel 8, including an attack path on Chrome [verified 2026-10] — [HackerNoon summary of TikTag](https://hackernoon.com/arm-mte-found-vulnerable-to-speculative-tag-leakage-on-pixel-8-devices); original paper: [arXiv 2406.08719](https://arxiv.org/abs/2406.08719) [ref]
- Android MTE docs describe SYNC/ASYNC/ASYMM modes, 4-bit tags and 16-byte granules [ref] — [Android source: Arm MTE](https://source.android.com/docs/security/test/memory-safety/arm-mte)
- V8's position: memory tagging is limited for V8 because side channels can leak tags, and pointer compression leaves no room for tag bits, so all heap objects would share one tag [verified 2026-10] — [V8 blog](https://v8.dev/blog/sandbox)

### Inferences
- MTE/EMTE is a strong probabilistic mitigation (4-bit tags, so 1/16 chance per guess) for allocator-level C/C++ bugs in browser-process and native code. It does not replace process isolation or the V8 sandbox. HGBA-2.0 should use it opportunistically (sync mode where available) for privileged processes and native libraries.
- Only Apple A19-class silicon has a vendor-documented speculative tag-confidentiality design. Treat MTE elsewhere as "detect and deter", not a confidentiality boundary.

### Gaps
- No source found on whether Chrome for Android enables MTE by default in 2025–26, or on its link to Android Advanced Protection mode.
- No independent benchmark of MIE overhead. Apple's claims are self-reported.
- MTE on laptops/desktops: no x86 equivalent exists. Arm Windows/Linux laptop MTE availability was not verified.

## 4. CHERI (Morello, CHERI-RISC-V, CHERIoT)

### Takeaway
CHERI provides deterministic, fine-grained memory safety and compartmentalization via hardware capabilities. Chromium has been ported to Morello with small code changes. As of Oct 2026, though, CHERI exists only on research boards (Morello), FPGA/IP (Codasip CHERI-RISC-V) and embedded CHERIoT parts. It is not available on any client laptop, desktop or phone CPU.

### Cited Findings
- Chromium on CHERI (AsiaBSDCon, Mar 2026 slides): Chromium base (27 MLoC) needed about 0.045% LoC changed and V8 (2 MLoC) about 0.8%. CHERI "deterministically mitigates" CVE-2023-4863 (libwebp, BLASTPASS) without prior knowledge of the bug. Figures are self-reported [verified 2026-10] — [AsiaBSDCon 2026 FreeBSD/CHERI slides](https://2026.asiabsdcon.org/media/entry/question_uploads/20260322-asiabsdcon-freebsd-cheri_BQqZECW.pdf)
- A 2023 paper notes that the most complete JS VM port to Morello was JavaScriptCore, and that "no meaningful performance results are publicly available" for VMs on Morello [verified 2026-10, search snippet] — [arXiv 2308.05076](https://arxiv.org/pdf/2308.05076)
- The early CheriBSD software stack for Morello included adapted OpenSSH, nginx and WebKit [verified 2026-10, search snippet] — [Light Blue Touchpaper: SRI and Cambridge release CHERI software stack for Morello](https://www.lightbluetouchpaper.org/?p=55876)
- CHERI Alliance founding members are University of Cambridge, Capabilities Limited, Codasip, FreeBSD Foundation, lowRISC and SCI Semiconductor. Google joined. EnSilica joined in Feb 2026 [verified 2026-10] — [BankInfoSecurity](https://www.bankinfosecurity.com/cheri-backers-form-alliance-to-promote-memory-safety-chip-a-25604); [CHERI Alliance: EnSilica joins](https://cheri-alliance.org/?p=1056)
- Codasip sells commercially available CHERI-RISC-V IP and development platforms (May 2026 article) [verified 2026-10] — [Design & Reuse: Why Codasip went all-in on CHERI](https://www.design-reuse.com/news/202530469-why-codasip-went-all-in-on-cheri/)
- Morello is a 2022 Arm research prototype SoC/board from the UK DSbD program, not a product [ref] — [Arm Morello](https://www.arm.com/architecture/cpu/morello); [Cambridge CHERI](https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/)
- CHERIoT is a CHERI-RISC-V profile for microcontrollers (Microsoft-originated, now open), and SCI Semiconductor's ICENI line targets it [ref] — [cheriot.org](https://cheriot.org/)

### Inferences
- For HGBA-2.0, CHERI belongs in a "future hardware" profile. The architecture should keep pointer provenance clean (no integer/pointer laundering, offset-based handles like the V8 sandbox) so a CHERI build is cheap later. No consumer device in 2026 can run it.

### Gaps
- No confirmed 2026 shipment of application-class (MMU, Linux-capable) CHERI silicon from any vendor.
- No published Chromium/V8 performance numbers on Morello.

## 5. Confidential computing (Intel TDX, AMD SEV-SNP, Arm CCA)

### Takeaway
TEE-VMs protect a guest from a malicious host, hypervisor or firmware. That is the opposite direction from browser content sandboxing, which protects the host from malicious content. They are server-class features (Xeon/EPYC) with no general client availability. They are also still exposed to host-driven side channels.

### Cited Findings
- AMD SEV-SNP threat model: the hypervisor, host BIOS, device drivers, other VMs and PCI devices are untrusted. Earlier SEV/SEV-ES assumed a "benign but vulnerable" hypervisor, and SNP adds integrity protection (replay, remapping) [verified 2026-10] — [enclaive mirror of AMD SEV-SNP threat model](https://docs.enclaive.cloud/confidential-cloud/technology-in-depth/amd-sev/technology/threat-model)
- TDXDown and CounterSEVeillance (2024): a malicious hypervisor can single-step TDX guests and use performance counters to leak secret-dependent branches. Intel updated TDX in response [verified 2026-10] — [SecurityWeek](https://www.securityweek.com/new-counterseveillance-and-tdxdown-attacks-target-amd-and-intel-tees/amp/)
- Neither SEV-SNP nor TDX removes host-induced side-channel risk (scheduling, I/O timing) [verified 2026-10, secondary blog, lower confidence] — [Safeguard blog](https://safeguard.sh/resources/blog/technical-comparison-of-amd-sev-snp-and-intel-tdx-security-guarantees)
- Intel TDX is a Xeon (4th Gen+) server feature [ref] — [Intel TDX overview](https://www.intel.com/content/www/us/en/developer/tools/trust-domain-extensions/overview.html). AMD SEV-SNP is an EPYC (Milan+) feature [ref] — [AMD SEV developer page](https://www.amd.com/en/developer/sev.html)
- Arm CCA introduces Realms managed by the Realm Management Monitor (RMM) and is an Armv9-A architecture feature [ref] — [Arm CCA](https://www.arm.com/architecture/security-features/arm-confidential-compute-architecture)
- Client-side analogue: Android Virtualization Framework (pKVM) runs "protected VMs" isolated from the Android host on supported Pixel devices [ref] — [Android Virtualization Framework](https://source.android.com/docs/core/virtualization)

### Inferences
- A browser does not need protection from its own OS. It needs protection of the OS and user data from web content. Confidential VMs add memory-encryption overhead and attestation complexity without addressing renderer compromise, Spectre between sites, or IPC-broker bugs. At most, a TEE could protect specific secrets such as passkeys or DRM keys, which is already the job of Secure Enclave, TPM, Pluton and StrongBox.

### Gaps
- No source found on TDX or SEV-SNP on client (Core/Ryzen) SKUs. Search returned only server/cloud material. Published client overhead numbers were not found.

## 6. MicroVMs / unikernels (Firecracker, crosvm, gVisor) and VM-based browser isolation

### Takeaway
MicroVMs boot in about 100 ms with a few MiB of VMM overhead, but they need hardware virtualization (KVM, or Hyper-V/HVF equivalents) and duplicate a guest kernel per instance. Mainstream VM-isolated browsing on clients has retreated: Microsoft deprecated Defender Application Guard in Dec 2023 and removed it in Windows 11 24H2. Qubes disposable VMs remain the main enthusiast deployment.

### Cited Findings
- Microsoft Defender Application Guard (MDAG) for Edge, including the Windows Isolated App Launcher APIs, is deprecated (announced Dec 2023) and "no longer available" starting with Windows 11 24H2. Existing installs are maintained. The Chrome/Firefox extensions ended after May 2024. Microsoft recommends Windows Sandbox or Azure Virtual Desktop instead [verified 2026-10] — [Microsoft Learn: Edge and MDAG](https://learn.microsoft.com/deployedge/microsoft-edge-security-windows-defender-application-guard); [ElevenForum summary](https://www.elevenforum.com/t/microsoft-defender-application-guard-for-edge-no-longer-available-in-windows-11-24h2.30140)
- Firecracker: microVMs launch in under about 125 ms with under 5 MiB memory overhead per microVM. It is KVM-based (Linux host only) [ref] — [firecracker-microvm.github.io](https://firecracker-microvm.github.io/); [NSDI 2020 Firecracker paper](https://www.usenix.org/conference/nsdi20/presentation/agache)
- crosvm is the Rust VMM used by ChromeOS (e.g. Crostini Linux containers) and underpins Android's AVF [ref] — [crosvm book](https://crosvm.dev/book/)
- gVisor is a user-space application kernel (Sentry) that intercepts syscalls (ptrace/systrap/KVM platforms) to sandbox containers [ref] — [gVisor docs](https://gvisor.dev/docs/)
- Qubes OS provides disposable VMs for one-off browsing sessions [ref] — [Qubes: How to use disposables](https://www.qubes-os.org/doc/how-to-use-disposables/)

### Inferences
- A per-site microVM is far heavier than process-per-site: a guest kernel, a separate GPU path (needing virtio-gpu/venus or paravirtualization) and around 100 ms cold start versus single-digit ms for a process. That makes it unrealistic for iframes. A per-profile or per-"untrusted session" VM tier is plausible on KVM/Hyper-V/HVF hosts. iOS forbids it for third-party apps, and Android's AVF is restricted.
- MDAG's deprecation is evidence that VM-isolated browsing was hard to sustain at consumer scale (GPU, UX, file transfer, memory).

### Gaps
- No 2025–26 data found on Edge or Chrome VM-isolation features beyond MDAG. Remote browser isolation (cloud-rendered) vendors were not researched.
- Firecracker numbers are vendor-stated, and their date was not re-verified this session.

## 7. Capability-based security in practice (Zircon, Capsicum, WASI)

### Takeaway
Capability systems (unforgeable handles plus rights) map naturally onto a browser kernel. Each renderer, utility or service gets only the handles it needs (Mojo pipes, shared-memory handles, specific fds), with no global namespace access.

### Cited Findings
- Fuchsia/Zircon: processes hold handles to kernel objects with per-handle rights, and there is no ambient global namespace [ref] — [Fuchsia: Zircon handles](https://fuchsia.dev/fuchsia-src/concepts/kernel/handles)
- Capsicum (FreeBSD) adds capability mode (`cap_enter`), where global namespaces are disallowed and fds carry fine-grained rights. The 2010 USENIX Security paper included a Chromium sandbox port [ref] — [Cambridge Capsicum](https://www.cl.cam.ac.uk/research/security/capsicum/); [USENIX Security 2010 Capsicum paper](https://www.usenix.org/conference/usenixsecurity10/capsicum-practical-capabilities-unix)
- WASI uses capability-based preopened directories and handles, and modules cannot open arbitrary paths [ref] — [WASI GitHub](https://github.com/WebAssembly/WASI)
- V8's External Pointer Table is explicitly an fd-table-like indirection, which is a capability-style handle design inside a process [verified 2026-10] — [V8 blog](https://v8.dev/blog/sandbox)

### Inferences
- An HGBA-2.0 "browser kernel" can express every privilege as a typed handle: Mojo/IPC endpoint, read-only shmem, a pre-opened file, a GPU command buffer. On Linux these become fds under seccomp, on Windows handles under restricted tokens/AppContainer, and on macOS Mach ports and XPC under Seatbelt. Revocation is the hard part, and it needs broker-side indirection.

### Gaps
- No current (2026) data on Fuchsia's consumer footprint or Capsicum use in shipping Chromium on FreeBSD.

## 8. Network security invariants

### Takeaway
Web content must never get raw network/NIC paths. The browser's network stack enforces the same-origin policy, CORS, CSP, cookie/SameSite rules, HSTS, mixed-content blocking, proxy and enterprise policy, and (since 2025) Local Network Access permission. Kernel-bypass (DPDK/vfio) also needs exclusive device ownership and privileged setup, which conflicts with sandboxing.

### Cited Findings
- Chrome Local Network Access (blog dated 9 Jun 2025) adds a permission prompt for sites connecting to the user's local network, to stop CSRF against routers and devices and to limit local-network fingerprinting. It covers fetch(), subresources and subframe navigation. WebSockets, WebTransport and WebRTC were "not yet" gated. It was opt-in from Chrome 138 via flag. The blog credits Private Network Access feedback as guiding the new permission approach [verified 2026-10] — [Chrome for Developers: Local Network Access](https://developer.chrome.com/blog/local-network-access)
- Secondary sources report LNA enforcement by default from Chrome 142 (Edge 143), and that LNA replaces PNA's server-side CORS-preflight approach [verified 2026-10, secondary] — [OpenReplay](https://blog.openreplay.com/chrome-local-network-access-lna-permission/); [Cognite docs](https://docs.cognite.com/cdf/access/guides/private_link_local_network_access_browser_permission.md)
- Mixed content: HTTPS pages block or upgrade HTTP subresources [ref] — [MDN: Mixed content](https://developer.mozilla.org/en-US/docs/Web/Security/Mixed_content)
- DPDK requires binding NICs to vfio-pci/uio drivers, which removes them from the kernel stack and requires root/IOMMU setup [ref] — [DPDK Linux drivers guide](https://doc.dpdk.org/guides/linux_gsg/linux_drivers.html)

### Inferences
- Policy enforcement (cookies, CORS, CSP, LNA, proxy/PAC, certificate validation, HSTS, Safe Browsing) has to live in a trusted network service. A sandboxed renderer with a direct NIC queue would bypass all of them and would also enable LAN scanning and DDoS amplification. Any kernel-bypass optimization belongs only inside the trusted network process.

### Gaps
- Exact Chrome milestone at which LNA became enforced by default was not verified against Chromium release notes.

## 9. Side channels: Spectre and site isolation rationale

### Takeaway
Spectre means any data in the same address space as attacker-controlled code should be treated as readable. Hence process-per-site, cross-origin isolation (COOP/COEP) as the gate for high-resolution timers and SharedArrayBuffer, and caution about sharing memory across processes.

### Cited Findings
- Google moved to Site Isolation as the primary Spectre mitigation in 2018 [ref] — [Google Security Blog, Jul 2018](https://security.googleblog.com/2018/07/mitigating-spectre-with-site-isolation.html)
- V8 team (2019) concluded that software mitigations within a process could not fully defend against Spectre, so process isolation is required [ref] — [V8 blog: A year with Spectre](https://v8.dev/blog/spectre)
- SharedArrayBuffer and precise timers require cross-origin isolation (COOP + COEP) [ref] — [web.dev: COOP/COEP](https://web.dev/articles/coop-coep)
- Apple's MIE includes a Spectre V1 mitigation at "virtually zero CPU cost". Apple estimates attackers would need 25 or more V1 gadgets for 95%+ exploitability. Speculative tag leaks (TikTag/StickyTags) defeat standard MTE confidentiality [verified 2026-10] — [Apple Security Research](https://security.apple.com/blog/memory-integrity-enforcement/)
- TEE side channels persist even in TDX/SEV-SNP (TDXDown, CounterSEVeillance) [verified 2026-10] — [SecurityWeek](https://www.securityweek.com/new-counterseveillance-and-tdxdown-attacks-target-amd-and-intel-tees/amp/)

### Inferences
- In-process isolation (V8 sandbox, PKU, MTE, RLBox) protects integrity, not confidentiality against Spectre. HGBA-2.0 should not co-locate different sites in one process even when these are present. Shared memory across processes (shared caches, deduplicated pages, shared GPU memory) should be read-only or limited to same-site, with timer precision gated by cross-origin isolation.

### Gaps
- No 2025–26 data gathered on new transient-execution classes affecting browsers (e.g., recent Apple SLAP/FLOP, 2024–25). Separate research is recommended.

## 10. Post-quantum TLS in browsers

### Takeaway
Hybrid X25519MLKEM768 key exchange is on by default in Chrome (131+), Edge and Firefox desktop (132+), and in Apple platforms from iOS 26/macOS Tahoe (Sept 2025). It runs in software on classical CPUs and needs no special hardware. Most human TLS traffic to Cloudflare was PQ-hybrid by 2026.

### Cited Findings
- Chrome switched from Kyber draft to ML-KEM (X25519MLKEM768) as default in Chrome 131 (Nov 2024) [ref] — [Google Security Blog, Sept 2024: A new path for Kyber on the web](https://security.googleblog.com/2024/09/a-new-path-for-kyber-on-the-web.html); corroborated [verified 2026-10] by [Palo Alto Networks](https://live.paloaltonetworks.com/t5/quantum-security-articles/the-quantum-countdown-how-hybrid-encryption-is-quietly/ta-p/1230276)
- Cloudflare lists Firefox 132+ desktop, Firefox 135+ for QUIC/HTTP3 and Firefox 145+ on Android as supporting X25519MLKEM768 [verified 2026-10] — [Cloudflare PQC support docs](https://developers.cloudflare.com/ssl/post-quantum-cryptography/pqc-support/index.md)
- iOS 26 (Sept 2025) began advertising X25519MLKEM768. Cloudflare PQ share from iOS rose from under 2% to 11% in four days [verified 2026-10, secondary] — [SoftwareSeni](https://www.softwareseni.com/post-quantum-cryptography-is-already-running-in-production-and-your-stack-may-already-be-using-it/); [postquantum.com](https://postquantum.com/security-pqc/rfc-10024-hybrid-mlkem-tls/) (that page's RFC-number claim is unverified)
- More than two-thirds of human-generated TLS traffic to Cloudflare used hybrid ML-KEM as of April 2026 [verified 2026-10, secondary] — [Encryption Consulting](https://www.encryptionconsulting.com/pqc-support-in-web-browsers/)
- Chrome's enterprise policy `PostQuantumKeyAgreementEnabled` lets admins disable it temporarily and is reportedly being phased out [verified 2026-10, secondary] — [Encryption Consulting deployment guide](https://www.encryptionconsulting.com/x25519mlkem768-tls-deployment-guide/)

### Inferences
- PQ key exchange is a solved, software-only baseline for HGBA-2.0. It adds about 1.1 KB to ClientHello (ML-KEM-768 public key; prior knowledge) and needs no new hardware. PQ signatures/certificates (ML-DSA) in the WebPKI remain the open item.

### Gaps
- No Apple/WebKit primary source was found confirming Safari's default PQ group. Evidence is secondary.
- Status of PQ certificates/ML-DSA (or Merkle Tree Certificates) in browsers as of Oct 2026 was not researched.
