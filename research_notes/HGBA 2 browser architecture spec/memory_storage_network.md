# Memory, Storage and Networking Hardware/OS: What a Client Browser Can Actually Use (as of Oct 2026)

Method note: these findings come from web searches run on 2026-10-13. Items marked "[canonical doc, not re-fetched]" cite long-standing primary documentation that this session's searches did not directly return. Verify those before quoting exact wording. Dates are given for older data.

## 1. CXL: spec features, shipping devices, deployment, client support, latency, Linux tiering

### Takeaway
CXL memory expansion is a data-center technology. CXL 2.0 expanders (Samsung CMM-D, SK hynix CMM-DDR5, Micron CZ120, Astera Leo-based) are in validation and early deployment. The first publicly announced cloud deployment is Azure M-series in preview (Nov 2025). No shipping consumer laptop or desktop supports CXL. Measured latency is about 2-3x local DRAM, around 200-400 ns. HGBA-2.0 should not assume a CXL tier on clients.

### Cited Findings
- The CXL 3.2 spec was released on 3 Dec 2024. It adds a CXL Hot-Page Monitoring Unit (CHMU) for memory tiering, common event records, PCIe MMPT compatibility, online firmware activation, Post Package Repair (PPR) enhancements, extra perf-monitoring events, and Trusted Security Protocol (TSP) features (meta-bits storage for HDM-H, expanded IDE). It is fully backward compatible. — [CXL Consortium 3.2 announcement PDF](https://computeexpresslink.org/wp-content/uploads/2024/12/CXL_3.2-Spec-Announcement_FINAL-1.pdf); [insideHPC](https://insidehpc.com/2024/12/cxl-consortium-releases-3-2-specification); [CXL Consortium About](https://computeexpresslink.org/about-cxl)
- CXL 3.x overview (fabrics, multi-level switching, memory sharing): [CXL Consortium "Overview of the CXL 3.X Specification"](https://computeexpresslink.org/?p=3508)
- **Latency (HEIMDALL / "Hitchhiker's Guide", arXiv 2411.02814, 2024):** CXL latency is about 50-300% above local DDR5 DIMMs. ASIC CXL devices measured about 200-300 ns and FPGA devices about 400 ns. — [arXiv 2411.02814](https://arxiv.org/pdf/2411.02814)
- **LiteSwitch (OSDI '26)** cites measured CXL device load latencies of 214-394 ns, typically about 3x or more slower than local memory. — [USENIX OSDI '26](https://www.usenix.org/system/files/osdi26-li-nanqinqin.pdf)
- "Dissecting CXL Memory Performance at Scale" (arXiv 2409.14317) found local and NUMA DRAM tail latency stable (p99.9 minus p50 = 45-61 ns), while CXL tail latency varies widely across vendors. — [arXiv 2409.14317](https://arxiv.org/pdf/2409.14317)
- Early prototypes measured CXL at about 2-3x local DDR latency. — [CXLMemSim, arXiv 2303.06153](https://arxiv.org/html/2303.06153v2)
- **SK hynix:** completed customer validation of the 96 GB CMM-DDR5 (CXL 2.0) in April 2025, with a 128 GB version in validation at that time. — [SK hynix PR via StreetInsider](https://www.streetinsider.com/PRNewswire/SK+hynix+Completes+Customer+Validation+of+CXL+2.0-based+DDR5/24670318.html)
- **Samsung CMM-D 2.0:** 128/256 GB, up to 36 GB/s, CXL 2.0, PCIe Gen5, with samples available. Commercialization was deprioritized behind HBM/DRAM demand. — [Korea Herald](https://www.koreaherald.com/article/10737182). A July 2026 report says Samsung targets mass production of CXL 3.2 modules by year-end but notes a possible delay to CXL 3.1 CMM-D. This is single-source and should be treated cautiously. — [techora.ru](https://techora.ru/news/samsung-nachnet-massovyy-vypusk-pamyati-cxl-2026-07-21)
- **Micron CZ120:** E3.S 2T, PCIe Gen5 x8, up to 36 GB/s. It was demoed on Intel Xeon 6 engineering samples (2024). — [ServeTheHome](https://www.servethehome.com/micron-cz120-cxl-memory-module-spotted-at-intel-vision-2024/); [Micron infographic](https://www.micron.cn/content/dam/micron/global/public/infographics/memory-expansion-module-infographic-micron-cz120.pdf)
- **Astera Labs Leo:** On 18 Nov 2025, Astera announced that Leo CXL 2.0 controllers (up to 2 TB per controller) can be evaluated in the Azure M-series VM preview. Astera calls this the first announced CXL-attached memory deployment. — [Astera Labs](https://www.asteralabs.com/news/astera-labs-leo-cxl-smart-memory-controllers-on-microsoft-azure-m-series-virtual-machines-overcome-the-memory-wall/)
- Micron reportedly began rolling out CXL 2.0 expansion modules (TrendForce citing Chosun Daily). — [TrendForce](https://www.trendforce.com/news/?p=46925)
- **Server-only CPU support:** Intel Sapphire Rapids and AMD EPYC Genoa were the first CPUs with CXL support. Around 2023-24, AMD estimated client CXL was 3-5 years away. — [Notebookcheck](https://www.notebookcheck.net/AMD-estimates-that-CXL-technology-with-improved-storage-performance-could-come-to-consumer-platforms-in-3-5-years.664216.0.html)
- A 2024 FMS talk proposed a "FleX" M.2-style CXL/PCIe socket for client motherboards. This is a proposal, not a product. — [FMS 2024 Gervasi](https://files.futurememorystorage.com/proceedings/2024/20240807_CXLT-201-1_Gervasi.pdf)
- **Linux tiering:** Meta's TPP (Transparent Page Placement) demotes cold pages to CXL and promotes hot pages. Most of it landed in Linux v5.18. — [TPP paper](https://deepai.org/publication/tpp-transparent-page-placement-for-cxl-enabled-tiered-memory)
- The kernel also uses NUMA balancing in tiering mode, which finds hot pages through hint faults and promotes them, and demotes on reclaim. — [patchew DAMON NUMA series](https://patchew.org/linux/20250420194030.75838-1-sj@kernel.org)
- **DAMON-based tiering (SK hynix):** the DAMOS actions `migrate_hot` / `migrate_cold` with a target node. — [LKML RFC v2](https://lkml.iu.edu/hypermail/linux/kernel/2402.3/02113.html); [RFC v3](https://patchew.org/linux/20240405060858.2818-1-honggyu.kim@sk.com)
- **DAMON auto-tuning for tiering:** reported about 4.42% better than NUMA balancing in tiering mode on its test. — [LWN](https://lwn.net/Articles/1018191/)

### Inferences
- CXL appears to a host as a CPU-less NUMA node. Any benefit to a browser would come from OS-level tiering, not browser code. On a client, the "far tier" is effectively compressed RAM (zram/zswap, macOS/Windows memory compression) plus SSD swap, not CXL.
- CHMU (CXL 3.2) is a hardware hot-page tracker for the OS/hypervisor. Userspace apps don't use it.

### Gaps
- I found no evidence of any 2026 consumer laptop or desktop platform with CXL. Absence is inferred from search results, not confirmed by a vendor statement.
- Samsung's 2026 CMM-D mass-production status is unverified (single source).

## 2. Persistent memory (Optane, CXL PMem) and why "persistent DOM in PMem" is impractical

### Takeaway
Intel wound down Optane in Q2 2022 and took a $559M write-off. No byte-addressable persistent memory ships on client hardware. A PMem-resident DOM therefore has no target hardware. Fast restore must use DRAM plus SSD serialization.

### Cited Findings
- Intel began winding down the Optane business in Q2 2022 and recorded a $559M inventory impairment. It ceased future development and continued to meet existing customer commitments, citing strategic priorities and its embrace of CXL. — [AnandTech](https://www.anandtech.com/show/17515); [Network World](https://www.networkworld.com/article/3668569/intel-kills-optane-amid-cost-cutting.html); [Club386](https://www.club386.com/intel-opts-out-of-optane-in-559-million-write-off/)
- Consumer Optane storage products had already ended around the turn of 2021-22. — [Blocks & Files](https://blocksandfiles.com/2022/07/29/intel-at-3d-exit-point-alas-poor-optane-i-knew-it-well/)
- Optane persistent memory (DCPMM) was a server-only product (Xeon). — [TechTarget](https://www.techtarget.com/it-infrastructure/opinion/Understand-the-Intel-Optane-shutdown)
- CXL Type-3 devices can in principle expose persistent memory, but every shipping expander found here (CMM-D, CMM-DDR5, CZ120, Leo) is volatile DRAM. — sources in section 1.

### Inferences
- A persistent DOM is impractical on clients for three reasons. There is no hardware. Crash-consistent PMem programming needs cache-line flush/fence discipline (CLWB/SFENCE) across a heap full of pointers, including JS GC-managed objects. And on-disk formats would have to stay ABI-stable across browser updates. Snapshot-to-SSD (bfcache-like serialization, V8 snapshots) is the realistic substitute.

### Gaps
- I found no source for any 2025-26 CXL persistent-memory product shipping in volume.

## 3. Client memory trends: unified memory, on-package LPDDR5X, LPCAMM2, bandwidth

### Takeaway
Unified CPU/GPU/NPU memory is now standard on client SoCs, so zero-copy CPU-GPU sharing is physically possible. Bandwidth spans about 120 GB/s (mainstream 128-bit) to 546 GB/s (M4 Max). Intel's on-package memory (Lunar Lake) was a one-off. The industry trend is soldered or LPCAMM2 LPDDR5X, not new tiers.

### Cited Findings
- **Apple M4 Max** (Oct 2024): up to 128 GB unified memory and 546 GB/s. **M4 Pro:** 273 GB/s. — [Apple Newsroom](https://www.apple.com/au/newsroom/2024/10/apple-introduces-m4-pro-and-m4-max/); [Wccftech](https://wccftech.com/m4-max-first-non-ultra-apple-silicon-to-exceed-500gb-s-memory-bandwidth/amp/)
- **AMD Strix Halo (Ryzen AI Max+ 395):** 256-bit LPDDR5X-8000, 256 GB/s theoretical. The memory must be soldered to fan out the 256-bit bus. Mainstream parts use 128-bit buses. — [Framework deep dive](https://frame.work/blog/framework-desktop-deep-dive-ryzen-ai-max)
- Strix Halo has no memory-on-package. It uses discrete LPDDR5X packages. This comes from a forum post, so it is weaker evidence. — [TechPowerUp forum](https://www.techpowerup.com/forums/goto/post?id=5411159)
- **Intel Lunar Lake** has on-package memory but is a packaging "one-off." Panther Lake and Nova Lake place memory off-package (Gelsinger, Q3 2024 earnings call). — [Thurrott](https://www.thurrott.com/?p=312447); [Wccftech](https://wccftech.com/intel-reaffirms-commitment-to-arc-gpus-panther-lake-nova-lake-sticking-to-non-on-package-memory/amp/)
- **LPCAMM2:** first used in the Lenovo ThinkPad P1 Gen 7 (2024). Crucial modules run up to 7500 MT/s and cost $330 for 64 GB. Micron projected 9600 MT/s modules by 2026. The format originated as Dell's proprietary CAMM (Precision 7670, 2022). It trades bandwidth and socket cost against modularity. — [AnandTech](https://www.anandtech.com/show/21390); [Liliputing](https://liliputing.com/microns-lpddr5x-based-lpcamm2-brings-compact-efficient-modular-memory-to-laptops/); [Wccftech](https://wccftech.com/micron-lpcamm2-lpddr5x-memory-debuts-lenovo-thinkpad-p1-gen7-workstation-laptop/amp/)

### Inferences
- On UMA SoCs, CPU-GPU "zero-copy" is a driver and API question (shared or imported GPU buffers, IOSurface, dma-buf, AHardwareBuffer, D3D shared handles) rather than a hardware one. Browsers already use these paths for video and canvas (see section 6).
- At 120-550 GB/s, memory bandwidth is rarely a browser bottleneck. The binding constraints are capacity per tab and power.

### Gaps
- The Lunar Lake bandwidth figure (LPDDR5X-8533, 128-bit, about 136 GB/s theoretical) was not confirmed by a primary source this session.

## 4. Fast restore mechanisms browsers already ship

### Takeaway
Browsers already combine an in-memory page snapshot (bfcache) with persisted compiled code (V8 code cache, explicit compile hints, Wasm code caching, SpiderMonkey JSBC), heap snapshots for startup (V8 snapshots), session restore and memory-pressure tab discarding. Storage Buckets is still Chromium-only.

### Cited Findings
- **bfcache** ships in Chrome (since 96), Firefox and Safari. — [web.dev bfcache](https://web.dev/articles/bfcache)
- Back/forward navigations are about 1 in 10 on desktop and 1 in 5 on mobile in Chrome. — [web.dev](https://web.dev/bfcache/)
- CrUX has reported a `navigation_types` breakdown, including back_forward_cache, since March 2024. An example origin served 6.77% of navigations from bfcache (Mar-Apr 2024). This is an example, not a global rate. — [Chrome for Developers: CrUX navigation types](https://developer.chrome.com/blog/crux-navigation-types)
- `Cache-Control: no-store` is the largest bfcache blocker, affecting about 17% of history navigations on mobile and 7% on desktop. Chrome experimented with allowing CCNS pages into bfcache when safe from Chrome 116, with 100% rollout projected for Mar-Apr 2025. The page was last updated Sept 2025. This exception is Chrome-only. — [Chrome for Developers: bfcache-ccns](https://developer.chrome.com/docs/web-platform/bfcache-ccns)
- One-site example: 17% of navigations were back/forward but only 2% were bfcache hits. — [DebugBear](https://www.debugbear.com/blog/back-forward-cache)
- Only Chromium exposes bfcache "notRestoredReasons." — [Sentry docs](https://docs.sentry.io/platforms/javascript/configuration/integrations/bfcache/)
- **V8 explicit compile hints:** the `//# allFunctionsCalledOnLoad` magic comment shipped in Chrome 136 (Apr 2025). It triggers eager background-thread compilation. In tests, 17 of 20 pages improved, with foreground parse/compile time down about 630 ms on average. V8 advises using it sparingly because of memory and time costs. — [V8 blog](https://v8.dev/blog/explicit-compile-hints); [DevClass](https://devclass.com/2025/05/02/v8-javascript-engine-gets-eager-compilation-hints-but-will-devs-use-sparingly-as-advised/)
- **V8 code caching and Wasm code caching** cache compiled code alongside the HTTP cache. — [V8 blog: Wasm code caching](https://v8.dev/blog/wasm-code-caching) [canonical doc, not re-fetched]; [V8 blog: Improved code caching](https://v8.dev/blog/improved-code-caching) [canonical doc, not re-fetched]
- **V8 startup snapshots** deserialize a pre-initialized heap instead of running JS setup. — [V8 blog: custom startup snapshots](https://v8.dev/blog/custom-startup-snapshots) [canonical doc, not re-fetched]
- **SpiderMonkey JS Startup Bytecode Cache (JSBC, 2017):** stores bytecode for executed functions in the network cache and decodes it instead of parsing, trading speed for memory. Newer work moves to Stencil-based in-memory caching (2022). — [Mozilla JS blog](https://blog.mozilla.org/javascript/2017/12/12/javascript-startup-bytecode-cache/); [SpiderMonkey newsletter FF106-107](https://spidermonkey.dev/blog/2022/10/24/newsletter-firefox-106-107.html)
- **Chrome Memory Saver** (Chrome 108) discards inactive background tabs. Discarded tabs keep their title and favicon and reload on focus. No event fires on discard, so developers should save state on `visibilitychange`. — [Chrome for Developers](https://developer.chrome.com/blog/memory-and-energy-saver-mode)
- Tab discarding under memory pressure dates to Chrome 46 (2015), when a tab used about 50 MB. — [Chrome blog 2015](https://developer.chrome.com/blog/tab-discarding/)
- Probabilistic Memory Saver (tested from Chrome 121) predicts revisit likelihood. Users revisit Memory Saver tabs within 24 h about 60% of the time. This comes from secondary coverage. — [Windows Report](https://windowsreport.com/chromes-memory-saver-mode-gets-smarter-at-discarding-tabs/)
- **Storage Buckets API:** Chromium 122+ (Chrome, Edge, Chrome Android). It is unsupported in Firefox (through 155) and Safari (through 26.x/27) per caniuse. — [Chrome for Developers](https://developer.chrome.com/docs/web-platform/storage-buckets); [caniuse](https://caniuse.com/wf-storage-buckets); [Chromium buckets README](https://chromium.googlesource.com/chromium/src/+/HEAD/content/browser/buckets/README.md)
- **OPFS** (Origin Private File System, with sync access handles in workers) is supported across Chrome, Firefox and Safari. — [MDN OPFS](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system) [canonical doc, not re-fetched]

### Inferences
- bfcache is the closest existing thing to "persistent live DOM," but it lives only in memory and is evicted on memory pressure or after a timeout. Across sessions, browsers persist only compiled code and session URLs/form state, never live heaps.
- A spec that wants fast cross-session restore would need DOM/heap serialization, which no browser does for arbitrary pages today. V8 snapshots show it is possible for controlled, deterministic heaps.

### Gaps
- No browser-wide bfcache hit rates were found for Firefox or Safari, and no current Chrome-wide hit rate was found either. The CrUX BigQuery `navigation_types` data would provide the Chrome figure.
- Whether the CCNS bfcache rollout reached 100% was not independently confirmed.
- Chrome session-restore internals (SNSS command files) were not researched this session.

## 5. Crash-consistent storage in browsers

### Takeaway
Browsers rely on proven embedded-store crash consistency: SQLite in WAL or rollback-journal mode, plus LevelDB's log-structured LSM in Chromium. They don't build custom persistence layers.

### Cited Findings
- SQLite WAL mode appends changes to a separate WAL file and checkpoints them back into the database. This gives atomic commits and lets readers run concurrently with a writer. — [SQLite WAL docs](https://www.sqlite.org/wal.html) [canonical doc, not re-fetched]
- LMDB uses copy-on-write B+trees with no WAL, so a crash leaves the previous committed root intact. — [LMDB docs](http://www.lmdb.tech/doc/) [canonical doc, not re-fetched]

### Inferences
- Chromium stores IndexedDB and Local Storage in LevelDB (an LSM with a write-ahead log). Firefox stores IndexedDB and session data in SQLite. These are well-known facts, but this session did not source them; see Gaps.

### Gaps
- This session did not find primary sources for Chromium's current IndexedDB backend (an SQLite-backed IndexedDB migration has been reported as in progress) or for Firefox's storage internals. Re-check the Chromium `content/browser/indexed_db` README and Mozilla source docs.

## 6. Memory tiering, madvise and huge pages: userspace control; shared-memory IPC; io_uring

### Takeaway
Userspace can only give hints: madvise (MADV_COLD, MADV_PAGEOUT, MADV_DONTNEED, MADV_HUGEPAGE), NUMA mbind, and mlock. Hot/cold page migration, DAMON and TPP are kernel decisions, and nothing lets an app pin data into the CPU cache. Chromium IPC uses Mojo with shared-memory regions, and video frames travel renderer to GPU via shared memory and GPU mailboxes. Google restricts io_uring on ChromeOS (disabled), Android (blocked for apps via seccomp) and its servers, so a sandboxed browser cannot rely on it.

### Cited Findings
- Transparent huge pages and `MADV_HUGEPAGE` are documented in kernel docs. — [kernel.org transhuge](https://docs.kernel.org/admin-guide/mm/transhuge.html) [canonical doc, not re-fetched]
- DAMON lets the kernel or privileged admins monitor data access and act on it (DAMOS). Tiering actions migrate pages between nodes. — [LKML](https://lkml.iu.edu/hypermail/linux/kernel/2402.3/02113.html); [kernel.org DAMON](https://docs.kernel.org/mm/damon/index.html) [canonical doc, not re-fetched]
- **io_uring:** In June 2023, Google reported that 60% of kCTF VRP submissions exploited io_uring, including all mitigation bypasses. It disabled io_uring on ChromeOS, blocks it for Android apps via seccomp-bpf (with future SELinux limits to select system processes), disabled it on production servers, and called it safe only for trusted components. Docker's default seccomp profile also blocks it. — [Phoronix](https://www.phoronix.com/news/Google-Restricting-IO_uring); [Schneier](https://www.schneier.com/blog/archives/2025/04/new-linux-rootkit.html); [Wikipedia io_uring](https://en.wikipedia.org/wiki/Io_uring)
- **Chromium network service:** the browser decides whether to run it in-process or in a utility process. Out-of-process is preferred and the default on most platforms. — [Chromium services/network README](https://chromium.googlesource.com/chromium/src/+show/cda6795da170a81f278d586636cc110d14db2a7a/services/network/README.md)
- As of the M128-era doc, the network process is unsandboxed on Android, Windows and Linux, and sandboxed on macOS and Fuchsia. — [Chromium process-sandboxes-by-platform](https://chromium.googlesource.com/chromium/src/+/b5470ac602dd94de3fa37f35c8a2a3649c6d013d/docs/security/process-sandboxes-by-platform.md)
- Mojo core brokers shared-memory allocation for sandboxed processes on some platforms. Unsandboxed processes skip brokering. — [initialize_mojo_core.cc mirror](https://gitcode.com/openharmony-tpc/chromium_src/blob/master/content/app/initialize_mojo_core.cc)
- Video frames pass from renderer to the GPU process via shared memory and become GPU resources (mailboxes, sync tokens) for the compositor. This comes from an informal slide deck. — [SlideShare: Isolating GPU access](https://www.slideshare.net/PatriciaAas/isolating-gpu-access-in-its-own-process)

### Inferences
- An HGBA-2.0 "memory tiers" layer should be an advisory policy (hot/warm/cold per tab or frame) that maps onto madvise or OS memory-pressure APIs and on tab discard/freeze. It should not claim to control physical placement or caches.
- io_uring cannot be part of a portable renderer or network-service design on ChromeOS or Android. At most it could serve an optional, unsandboxed, Linux-desktop-only browser-process path.

### Gaps
- No primary source was found this session for Chromium's VideoFrame/GpuMemoryBuffer zero-copy path. Check the Chromium `media/` and `gpu/` docs.

## 7. Network acceleration: kTLS, crypto instructions, QUIC GSO/GRO, offload, SmartNICs

### Takeaway
On clients, the realistic accelerations are CPU crypto instructions (AES-NI, ARMv8 Crypto Extensions) and, for QUIC, UDP GSO/GRO batching on Linux. kTLS and NIC TLS/QUIC offload are server and data-center features. kTLS doesn't apply to QUIC. Chromium's QUIC is Google's QUICHE, running in the network service.

### Cited Findings
- kTLS hardware offload has been demonstrated with NGINX on NVIDIA BlueField-2 DPUs, limited to TLS 1.2 and AES-128 (netdev 0x16, 2022). — [netdev 0x16 slides](https://netdevconf.info/0x16/slides/20/TLS-0x16.pdf); [kernel TLS offload docs](https://docs.kernel.org/networking/tls-offload.html) [canonical doc, not re-fetched]
- Cloudflare sped up QUIC UDP transmission by moving from one `sendmsg()` per packet to `sendmmsg()` and GSO. — [Cloudflare blog](https://blog.cloudflare.com/accelerating-udp-packet-transmission-for-quic/)
- GSO lets an app pass a buffer of up to 64 KB that the kernel splits into MTU-sized datagrams. It needs Linux kernel 4.18 or later (quic-go v0.36). — [Golang Weekly / quic-go](https://golangweekly.com/link/141593/34e40b1103)
- A 2026 study (NOMS 2026, TUM) found GSO/GRO cut packet I/O overhead substantially, but many QUIC stacks lack support. Only 1 of 4 tested implementations used both, reaching about 8.3 Gbit/s goodput. — [TUM NOMS 2026 paper](https://www.net.in.tum.de/fileadmin/bibtex/publications/papers/spaethj-noms2026.pdf)
- QUIC receiver-side CPU cost: implementations analyzed in 2024 did not use UDP GRO, and ACK generation in userspace is a bottleneck. This comes from a secondary summary. — [youngju.dev (2026)](https://www.youngju.dev/split/2026-07-16-quic-receiver-cpu-cost-kernel-quic)
- Tailscale measured large UDP throughput gains from GSO/GRO on Linux (2023). — [Tailscale blog](https://tailscale.com/blog/quic-udp-throughput)
- LWN covers in-kernel QUIC proposals. — [LWN](https://lwn.net/Articles/953031)
- QUICHE (Google) powers Chromium's QUIC. A 2020 QUICHE commit adds a GSO batch writer marked "do not add to BUILD.gn," which suggests it isn't in the Chromium build. — [QUICHE commit](https://quiche.googlesource.com/quiche/+/e0b2bb09e1a8654d564084bf0bba1757f8113c3f)

### Inferences
- A client browser gains little from NIC offload. TLS and QUIC crypto run at multi-GB/s per core with AES-NI or ARMv8 crypto, so client links (≤10 Gbit/s) are rarely crypto-bound. Packet I/O syscall overhead (GSO/GRO) matters more.
- SmartNICs and DPUs (e.g., BlueField) appear only in data-center and server contexts in every source found.

### Gaps
- Whether Chromium's client QUIC uses UDP GSO/GRO on Linux, Windows (USO/URO) or Android was not confirmed.
- No primary source was found for client AES-NI or ARMv8 crypto throughput figures, or for the status of the in-kernel QUIC merge as of Oct 2026.
