# Izcal security scope: history and postmortems of browser security architecture

Compiled 2026-10-18. Primary sources were preferred (papers, vendor security blogs, maintainers' posts, release notes, Project Zero and GTIG reviews). No other browser's source files are cited (M-1).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data, or a well-known fact not re-fetched this session; **Low** = secondary source only.

Scope: phase 2 of the first-principles method (history: what was built, why, its limits, mismatches, its fate and the actual reasons). Ideas only; nothing here is a template.

## 0. Chronology at a glance

| Year | Step | Driving threat or constraint |
|---|---|---|
| 1996–1997 | ActiveX and Browser Helper Objects: native COM code in-process | Extend a thin platform; no sandbox concept |
| ~2000–2006 | Monolithic single-process browsers (IE6, Firefox 1–3, Safari 3) | Single-core, low-RAM PCs; one address space for everything |
| 2006–2007 | IE7 Protected Mode on Vista (Low integrity + brokers) | Drive-by malware installs; Vista's new MIC/UAC primitives |
| 2008 | Chrome: multi-process, sandboxed renderers; JITs arrive | Renderer RCE bugs; multi-core PCs; "browser as OS" |
| 2008–2009 | NaCl (native code in an SFI sandbox) | Native-speed apps without plugins |
| 2009–2016 | Firefox Electrolysis (e10s): started, paused, shipped in Firefox 48 | Add-on compatibility with a shared-memory model |
| 2010 | WebKit2 split process model (Safari) | Same as Chrome; Apple's own sandbox (Seatbelt) |
| 2011 | DigiNotar compromise; CA distrust; key pinning | A single CA can mint any certificate |
| 2013–2015 | NPAPI phased out (Chrome), ActiveX dropped (Edge) | Plugins the top crash, hang and exploit source |
| 2013–2018 | Certificate Transparency, enforced in Chrome 2018 | Detect misissuance after DigiNotar, Symantec |
| 2016–2017 | Firefox Quantum, Stylo in Rust | Parallelism plus memory safety |
| 2017–2020 | Flash end of life (2020), Java plugin deprecated, PNaCl removed | Open standards caught up (HTML5, WebGL, Wasm) |
| 2018 | Spectre/Meltdown; Chrome Site Isolation on by default (Chrome 67) | Same-process memory is readable by JS |
| 2019–2021 | Site Isolation against compromised renderers; COOP/COEP; Fission | Treat the whole renderer as untrusted |
| 2020–2024 | MiraclePtr, hardened libc++, V8 sandbox, RLBox | Memory-safety bugs ~70% of severe bugs |
| 2021–2025 | CET, MTE research, Apple MIE (2025); JIT-off modes; Lockdown Mode | Exploit chains for targeted surveillance |
| 2019–2025 | Manifest V3 (Chrome), MV2 off by 2025 | Extension abuse, review, performance; ad-blocking controversy |
| 2023–2026 | Rust in Chromium (third-party libs, fonts; FreeType removed Chrome 145) | Unsafe parsers of untrusted input |
| 2025–2026 | Agentic browsers; indirect prompt injection; agent origin sets | A new principal acting on the user's behalf |

## 1. The single-process era and why it failed

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | In 2008 "most" browsers still used "the original monolithic architecture": one process held parsing, rendering, JS, cookies, passwords and file access, so any rendering-engine RCE gave the attacker the user's full privileges ("can steal sensitive files or install malware"). | — | [Barth, Jackson, Reis, "The Security Architecture of the Chromium Browser"](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf) | 2008 | High |
| 1.2 | Analysis of the previous year's disclosed IE, Firefox and Safari vulnerabilities: **67.4% (87 of 129)** were in the rendering engine; **70.4% (38 of 54)** of the arbitrary-code-execution bugs were there. Placing the engine in a sandbox would have mitigated those. 72.7% (8 of 11) of browser-kernel bugs would not have been helped by more sandboxing. | 87/129; 38/54; 8/11 | [Barth et al.](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf) | 2008 | High |
| 1.3 | Why single-process existed: early-2000s PCs were single-core with little RAM, plugins and add-ons (ActiveX, BHOs, XPCOM) assumed in-process access to browser internals, and the threat was mostly malicious content rather than organized exploit chains. BHOs were "COM objects that were loaded in-process with the browser". | BHO introduced 1997 | [Microsoft Edge team, "A break from the past, part 2"](https://blogs.windows.com/msedgedev/2015/05/06/a-break-from-the-past-part-2-saying-goodbye-to-activex-vbscript-attachevent/) | 2015-05-06 | High (BHO); Medium (era constraints, inference) |
| 1.4 | **Fate:** the monolithic model failed because one bug equals full compromise, and because crashes and hangs from plugins took the whole browser down. Every major browser moved to multi-process between 2008 (Chrome) and 2016 (Firefox 48). | — | [Computerworld on Firefox 48](https://www.computerworld.com/article/1668630/mozilla-supplies-multi-process-firefox-to-1-of-users.html) | 2016-08 | Medium |

**Implications:** Izcal starts multi-process. The 2008 numbers are the founding evidence that the parser and renderer, not the kernel, carry most of the risk; that is still true (§9).

## 2. IE7 Protected Mode (2006–2007): the first OS-assisted browser sandbox

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Protected Mode ran IE7 at **Low integrity** on Vista, using three new Vista primitives: UAC (least privilege), Mandatory Integrity Control (lower-integrity processes cannot write higher-integrity objects) and UIPI (blocks window messages to higher-integrity processes, against "shatter attacks"). | 4 integrity levels: Low, Medium, High, System | [IEBlog, "Protected Mode in Vista IE7"](https://learn.microsoft.com/en-us/archive/blogs/ie/protected-mode-in-vista-ie7) | 2006 | High |
| 2.2 | Goal: exploited code "wouldn't have enough privileges to install software, put files in the user's Startup folder, hijack browser settings". It targeted **persistence and drive-by install**, not data theft. | — | [IEBlog](https://learn.microsoft.com/en-us/archive/blogs/ie/protected-mode-in-vista-ie7) | 2006 | High |
| 2.3 | Medium-integrity work went through a **User Broker**, admin work (e.g. ActiveX install) through an **Admin Broker** that asks for consent. Add-ons could create their own brokers. | — | [MSDN, Application Compatibility: IE Protected Mode](https://learn.microsoft.com/id-id/previous-versions/bb756991(v=msdn.10)) | 2007 | Medium |
| 2.4 | **Limits:** Low integrity blocks writes but **not reads** of the user's files; one process still held all sites; ActiveX and add-ons still ran in the tab, so add-on bugs were browser bugs; XP (no MIC) got no protection. Read access was the gap Chrome's restricted token closed. | — | Inference from 2.1–2.3 and [Chrome sandbox announcement](https://blog.chromium.org/2008/10/new-approach-to-browser-security-google.html) | — | Medium |

**Implications:** a sandbox defined by "what the attacker cannot persist" is weaker than one defined by "what the attacker cannot read". Izcal's renderer must have no read access to user data, not just no write access.

## 3. Chrome's multi-process sandbox (2008) and its evolution

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | Architecture: a **browser kernel** (trusted, holds user data and OS access) and **rendering engines** (HTML, JS, images) in sandboxed processes, talking over IPC. The kernel enforces policy on files and network; the renderer enforces the same-origin policy. | — | [Barth et al.](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf) | 2008 | High |
| 3.2 | **Explicitly out of scope in 2008:** "The architecture does not prevent an attacker who compromises the rendering engine from attacking other web sites (for example, by reading their cookies)." Cross-site isolation was deferred for compatibility. | — | [Barth et al.](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf) | 2008 | High |
| 3.3 | Compatibility was a first-class constraint: passed 99% of 10,115 WebKit layout tests. | 99% of 10,115 | [Barth et al.](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf) | 2008 | High |
| 3.4 | **Plugins ran outside the sandbox with the user's full privileges** because vendors expected one instance per browser and Flash needed camera, microphone and file writes. A `--safe-plugins` option was experimental. This was the biggest hole in the 2008 design. | — | [Barth et al.](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf) | 2008 | High |
| 3.5 | Windows sandbox "does not reinvent any security model": restricted token with all privileges removed and groups disabled, job objects, alternate desktop. Known gaps: FAT32 volumes without security descriptors, and third-party objects misconfigured with world access. | — | [Chromium Blog, "A new approach to browser security: the Google Chrome sandbox"](https://blog.chromium.org/2008/10/new-approach-to-browser-security-google.html) | 2008-10 | High |
| 3.6 | Today's exploit shape is a **two-bug chain**: renderer RCE then sandbox escape, either through the OS (syscalls left reachable) or through a bug in the privileged browser process (IPC). Of 100 recent high-severity bugs, the largest classes were V8 bugs in the renderer and use-after-free in the browser process. | sample of 100 bugs | [Google Security Blog, "Use-after-freedom: MiraclePtr"](https://security.googleblog.com/2022/09/use-after-freedom-miracleptr.html) | 2022-09-13 | High |
| 3.7 | **Rule of 2:** pick at most two of untrustworthy input, unsafe language, high privilege. The policy sometimes blocks features because a new process is too expensive. | — | [Chromium docs, The Rule of 2](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/rule-of-2.md); [Chromium memory safety page](https://www.chromium.org/Home/chromium-security/memory-safety/) | living doc | High |
| 3.8 | Sandbox escapes are now often **logic bugs at the IPC/OS boundary**, not memory corruption: CVE-2025-2783 (Operation ForumTroll, March 2025) was a Mojo handle-validation logic error on Windows used against Russian targets; fixed in Chrome 134.0.6998.177. | KEV-listed 2025-03-27 | [Dark Reading](https://www.darkreading.com/cyberattacks-data-breaches/google-patches-chrome-zero-day-exploited-apt); [CyberInsider](https://cyberinsider.com/google-patches-actively-exploited-chrome-zero-day-used-in-espionage-campaign/) | 2025-03 | Low (secondary; Kaspersky primary not read) |

**Implications:** the 2008 boundary (renderer vs kernel) is still the main one, but its two deferred items, cross-site isolation and plugins, became the next decade's work. Izcal's IPC is the escape surface: memory-safe implementation does not remove logic bugs in capability checks.

## 4. Firefox: e10s, Fission, RLBox

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | Multi-process Firefox was discussed from 2009, paused for several years, resumed mid-2015, and shipped first in Firefox 48 (August 2016) to about 1% of eligible users, with only two processes (chrome + one content). | 7-year effort | [Computerworld](https://www.computerworld.com/article/1668630/mozilla-supplies-multi-process-firefox-to-1-of-users.html) | 2016-08 | Low (secondary) |
| 4.2 | **Actual reason for the delay: add-ons.** The XUL/XPCOM extension model "assumed that the browser and content used the same memory space"; only 8 of the top 20 extensions were e10s-compatible (Firebug and NoScript were not). Users without add-ons got e10s first. | 8 of top 20 | [Computerworld](https://www.computerworld.com/article/1668630/mozilla-supplies-multi-process-firefox-to-1-of-users.html) | 2016-08 | Low |
| 4.3 | As of April 2021 Firefox ran a **fixed** pool: 8 web content processes, up to 2 semi-privileged content processes, and 4 utility processes (extensions, GPU, network, media). Mozilla said this "does not meet the security standards of Mozilla" because two sites can share a process. | 8 + 2 + 4 | [Mozilla Hacks, "Introducing Firefox's new Site Isolation security architecture"](https://hacks.mozilla.org/2021/05/introducing-firefox-new-site-isolation-security-architecture/) | 2021-05 | High |
| 4.4 | Fission's stated rationale is Spectre: "the only way to provide memory protections necessary to defend against Spectre-like attacks is to rely on ... the operating system's process separation." Shipped by default in Firefox 95 (Dec 2021). | — | [Mozilla Hacks](https://hacks.mozilla.org/2021/05/introducing-firefox-new-site-isolation-security-architecture/) | 2021-05 | High (rationale); Medium (Firefox 95 date) |
| 4.5 | **RLBox** (with UCSD and UT Austin): compile a C/C++ library to Wasm, then wasm2c back to C, giving in-process isolation without new processes. Firefox 95 sandboxed Graphite, Hunspell, Ogg; Expat and Woff2 in 96. Why not processes: "processes have a fixed memory overhead". Bounty pays for sandbox bypass even without a library bug. | 5 libraries | [Mozilla Hacks, "WebAssembly and Back Again"](https://hacks.mozilla.org/2021/12/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/) | 2021-12 | High |
| 4.6 | RLBox limits: not suitable for components that "depend too much on sharing memory with the rest of the program" or are too performance-sensitive. | — | [Mozilla Hacks](https://hacks.mozilla.org/2021/12/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/) | 2021-12 | High |
| 4.7 | Firefox is 30M+ lines across six platforms; Pwn2Own 2024 chain (Manfred Paul, two critical bugs: OOB read/write and privileged code execution, CVE-2024-29943/29944) was patched in under 21 hours; an in-the-wild exploit (ESET report, Oct 2024) in 25 hours. Fast response is a deliberate capability, with release trains planned around Pwn2Own. | 21 h; 25 h | [Mozilla Security Blog, "Rapidly Leveling up Firefox Security"](https://blog.mozilla.org/security/2024/04/04/rapidly-leveling-up-firefox-security/); [Mozilla, "Behind the Scenes: Fixing an In-the-Wild Firefox Exploit"](https://blog.mozilla.org/security/2024/10/11/behind-the-scenes-fixing-an-in-the-wild-firefox-exploit/) | 2024-04-04; 2024-10-11 | High |

**Implications:** an extension API that shares memory with the engine freezes the process model for a decade. Izcal's extension and agent APIs must be message-based from day one. RLBox shows a second isolation tier (in-process, Wasm-based) for libraries too small to merit a process.

## 5. Safari / WebKit: process model, WebContent sandbox, Lockdown Mode, MIE

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | WebKit2 (announced April 2010, shipped in Safari 5.1 / Lion 2011) split WebKit into a UI process and a WebContent process, built into the framework so every WebKit client got it. | — | [WebKit blog, "Announcing WebKit2"](https://webkit.org/blog/1001/announcing-webkit2/) (fetch failed this session) | 2010-04 | Medium |
| 5.2 | Apple's hardening pattern is **attack-surface removal from WebContent**: GPU work moved to a separate GPU process and WebContent lost IOKit user clients (e.g. a 2021 change removed IOSurfaceAcceleratorClient access when the GPU process is on). Synacktiv (OffensiveCon 2024) describes a 2023 full chain that escaped WebContent via the GPU process, followed by allocator and deserialization hardening. | WebContent IOKit ≈ 0 (slide table; ambiguous) | [WebKit r273169](https://trac.webkit.org/r273169); [Synacktiv slides](https://www.synacktiv.com/sites/default/files/2024-05/escaping_the_safari_sandbox_slides.pdf) | 2021; 2024-05 | Medium |
| 5.3 | **Site isolation in WebKit is late and partial.** Bug 287102 (process isolation of cross-site frames) was filed publicly in Feb 2025 "tracked internally for quite some time"; in July 2025 the implementer reported no progress outside Cocoa ports for about two years, with cross-process compositing on Linux unsolved. | — | [WebKit bug 287102](https://bugs.webkit.org/show_bug.cgi?id=287102); [webkit-dev, July 2025](https://lists.webkit.org/pipermail/webkit-dev/2025-July/032768.html) | 2025 | Medium |
| 5.4 | **Lockdown Mode** (iOS 16 / macOS Ventura, 2022): optional extreme mode for "the very small number of users who face grave, targeted threats". For the web it **disables JIT** "unless the user excludes a trusted site". Bounties doubled, up to $2M. | $2,000,000 max | [Apple Newsroom](https://www.apple.com/newsroom/2022/07/apple-expands-commitment-to-protect-users-from-mercenary-spyware/) | 2022-07-06 | High |
| 5.5 | **Memory Integrity Enforcement (MIE)**, iPhone 17 / Air (Sept 2025): type-segregated secure allocators (kalloc_type, xzone malloc, **WebKit's libpas**) plus **EMTE in synchronous mode** plus Tag Confidentiality Enforcement, always on. Apple judged original Arm MTE "exhibited weaknesses that were unacceptable" as a defense; async mode leaves "a race window". A Spectre V1 mitigation at "virtually zero CPU cost" protects tag secrecy. Five years of work. | PAC first shipped 2018 (A12) | [Apple Security Research, "Memory Integrity Enforcement"](https://security.apple.com/blog/memory-integrity-enforcement/) | 2025-09 | High |

**Implications:** the most effective recent defenses (MIE, PAC) come from owning the hardware and the allocator together. Izcal cannot own hardware, but it can own its allocator's type segregation and opt into sync MTE where the OS offers it.

## 6. The death of plugins and native-code-on-the-web

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **ActiveX** (1996): "a binary extension model ... to embed native Windows technologies (COM/OLE) in web pages". Edge dropped ActiveX, BHOs, VBScript, VML and document modes; 220,000+ lines of MSHTML removed. Reason given: HTML5 capabilities and interoperability. | 220,000 lines | [Microsoft Edge team](https://blogs.windows.com/msedgedev/2015/05/06/a-break-from-the-past-part-2-saying-goodbye-to-activex-vbscript-attachevent/) | 2015-05-06 | High |
| 6.2 | **NPAPI** (Netscape era): its "90s-era architecture has become a leading cause of hangs, crashes, security incidents, and code complexity". Chrome blocked page-instantiated NPAPI from Jan 2014, removed it on Linux in Chrome 35, harder-to-bypass UI in Chrome 37. Only six plugins had >5% monthly users (Silverlight 15%, Unity 9.1%, Google Earth 9.1%, Java 8.9%, Google Talk 8.7%, Facebook Video 6.0%). NPAPI was never on mobile. | 6 plugins >5% | [Chromium Blog, "Saying Goodbye to Our Old Friend NPAPI"](https://blog.chromium.org/2013/09/saying-goodbye-to-our-old-friend-npapi.html) | 2013-09 (updates to 2014-07) | High |
| 6.3 | **Flash:** Jobs' 2010 objections were closedness, security ("Symantec recently highlighted Flash for having one of the worst security records in 2009"), reliability ("the number one reason Macs crash"), battery (software video decode) and touch. Adobe announced end of life for end of 2020, citing HTML5, WebGL and Wasm maturity, in collaboration with Apple, Facebook, Google, Microsoft, Mozilla. | EOL 2020-12-31 | [Steve Jobs, "Thoughts on Flash" (archive)](https://web.archive.org/web/20100501010616/http://www.apple.com/hotnews/thoughts-on-flash/); [Adobe, "Flash & The Future of Interactive Content"](https://blog.adobe.com/en/publish/2017/07/25/adobe-flash-update) | 2010-04; 2017-07-25 | High |
| 6.4 | **Java applets:** Java Plug-in deprecated in JDK 9 (JEP 289 / "Deprecate the Java Plug-in"); Chrome already blocked Java by default "for security reasons" in 2013. | — | [Oracle, JDK 9 deprecated features](https://www.oracle.com/java/technologies/javase/9-deprecated-features.html); [Chromium Blog NPAPI](https://blog.chromium.org/2013/09/saying-goodbye-to-our-old-friend-npapi.html) | 2017; 2013 | High |
| 6.5 | **Silverlight 5** support ended 2021-10-12 (fixed lifecycle). | — | [Microsoft Lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/silverlight-5) | — | High |
| 6.6 | **Common cause of death:** plugins were a second, vendor-controlled platform with its own update channel, own security record and full user privileges, outside the browser's sandbox (3.4). Each died when (a) open standards covered its use cases, (b) mobile never supported it, and (c) browsers stopped paying the crash and exploit cost. Security was a cause, not the only one. | — | Synthesis of 6.1–6.4 | — | Medium |
| 6.7 | **NaCl** (Dec 2008): an SFI sandbox for native x86 code (Yee et al., IEEE S&P 2009). **PNaCl** (2013) made it portable. Removed because it "did not provide a solution that worked seamlessly across all browsers" and "usage of PNaCl is sufficiently low"; WebAssembly won as the cross-browser path. PNaCl removed from the open web Q1 2018; NaCl deprecated 2020; ChromeOS 138 ends NaCl. | — | [Chromium Blog, "Goodbye PNaCl, Hello WebAssembly!"](https://blog.chromium.org/2017/05/goodbye-pnacl-hello-webassembly.html); [Chrome docs, Native Client](https://developer.chrome.com/docs/native-client); [NaCl paper page](https://research.google/pubs/native-client-a-sandbox-for-portable-untrusted-x86-native-code/) | 2017-05-30; 2024-10 | High |

**Implications:** a security mechanism that only one engine implements dies even if it works (NaCl). Anything Izcal exposes to pages must be a web standard or staged as one; and no component may run outside the sandbox "for compatibility" (the 2008 plugin decision cost a decade).

## 7. Spectre (2018), Site Isolation and cross-origin isolation

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | V8's conclusion: "speculative vulnerabilities on today's hardware defeat all language-enforced confidentiality with no known comprehensive software mitigations"; untrusted code can build "a universal read gadget". Chrome and V8 therefore "shifted the security model ... to process isolation". | — | [McIlroy et al., "Spectre is here to stay" (arXiv 1902.05178)](https://arxiv.org/abs/1902.05178) | 2019-02 | High |
| 7.2 | Early mitigations (timer coarsening, JIT changes) only reduce leak rates. Shared memory plus resolution recovery yields nanosecond timers, enough to see L1 hits. | performance.now() single-digit µs | [V8 blog, "A year with Spectre"](https://v8.dev/blog/spectre) | 2019-04-23 | High |
| 7.3 | Site Isolation on by default in **Chrome 67** for 99% of desktop users (1% holdback); "about a 10-13% total memory overhead in real workloads". It needed out-of-process iframes (years of work "independently of Spectre", first shipped for extensions in 2017) and **CORB** to keep cross-site HTML/JSON/XML out of the renderer. Site = scheme + registrable domain. | 10–13% memory | [Google Security Blog, "Mitigating Spectre with Site Isolation in Chrome"](https://security.googleblog.com/2018/07/mitigating-spectre-with-site-isolation.html) | 2018-07-11 | High |
| 7.4 | Reis et al. frame the threat as compromised renderers plus transient execution; deployed to all desktop Chrome users. | — | [Reis, Moshchuk, Oskov, USENIX Security 2019](https://www.usenix.org/conference/usenixsecurity19/presentation/reis) | 2019-08 | Medium (abstract) |
| 7.5 | Chrome 77 extended Site Isolation to defend against **fully compromised renderers** and UXSS on desktop (browser-side enforcement), and enabled it on Android for sites users log into. | — | [Google Security Blog, Oct 2019](https://security.googleblog.com/2019/10/improving-site-isolation-for-stronger.html); [chromium.org Site Isolation](https://www.chromium.org/Home/chromium-security/site-isolation/) | 2019-10 | High |
| 7.6 | **Memory budget limit:** on Android, only a subset of sites is isolated, and Site Isolation is unavailable on devices with **less than 2 GB RAM** and in Android WebView. Chrome 92 added OAuth-login sites and COOP sites. | 2 GB threshold | [chromium.org Site Isolation](https://www.chromium.org/Home/chromium-security/site-isolation/) | living doc | High |
| 7.7 | Leaky.page (2021): practical Spectre PoC leaking **1 kB/s** on Chrome 88/Skylake, also worked on Apple M1; variants leak 8 kB/s with a 5 µs timer and **60 B/s with 1 ms timers**. Isolation features "don't prevent the exploitation of Spectre; rather, they protect sensitive data from being present" in readable memory. | 1 kB/s; 60 B/s | [Google Security Blog, "A Spectre proof-of-concept for a Spectre-proof web"](https://security.googleblog.com/2021/03/a-spectre-proof-of-concept-for-spectre.html) | 2021-03 | High |
| 7.8 | **Cross-origin isolation** (COOP + COEP `require-corp`) gates SharedArrayBuffer and precise timers. SAB returned on desktop Chrome 68 (with Site Isolation), then was restricted to cross-origin-isolated pages from Chrome 92; origin trials ran to Chrome 113 and a deprecation trial to Chrome 124. COEP `credentialless` (Chrome 96) was added because `require-corp` was hard to deploy. | Chrome 68 → 92 → 124 | [web.dev, "Why you need cross-origin isolated"](https://web.dev/articles/why-coop-coep); [Chrome blog, SharedArrayBuffer updates](https://developer.chrome.com/blog/enabling-shared-array-buffer) | 2020-05; updated 2024-01 | High |

**Implications:** process isolation per site is the only accepted Spectre defense, and it has a measured memory cost and a RAM floor (2 GB) below which Chrome gives up. Izcal's I-1 rule (no two sites per process) must come with a budget plan for low-memory devices, not an exception that silently weakens it. Opt-in headers (COOP/COEP) took four years and still needed a softer mode; defaults beat opt-ins.

## 8. Memory safety in large C++ codebases: MiraclePtr, hardened libc++, V8 sandbox

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | ~70% of Chrome's high/critical security bugs are memory-safety bugs (912 bugs since 2015); half of those are use-after-free. "This volume of bugs holds steady despite years of investment into developer education, fuzzing, Vulnerability Reward Programs". | 70%; 912 bugs | [Chromium memory safety](https://www.chromium.org/Home/chromium-security/memory-safety/); [chromium.org Site Isolation](https://www.chromium.org/Home/chromium-security/site-isolation/) | 2020; living | High |
| 8.2 | **MiraclePtr / BackupRefPtr**: `raw_ptr<T>` with a hidden refcount in PartitionAlloc; freed memory with live references is quarantined and poisoned. ~10 algorithms compared. 15,000+ raw pointers rewritten; on in the browser process on Windows/Android in Chrome 102. Cost: browser-process memory +4.5–6.5% (Windows), +3.5–5% (Android), main-thread contention +6–7%. Only class/struct pointer fields covered. | 15,000 pointers | [Google Security Blog, "Use-after-freedom: MiraclePtr"](https://security.googleblog.com/2022/09/use-after-freedom-miracleptr.html) | 2022-09-13 | High |
| 8.3 | MiraclePtr results: all non-renderer processes by Sept 2022; ChromeOS, macOS, Linux June 2023. Of 168 UAF reports since launch it mitigated **57%** in privileged processes (estimate was 50%). Renderers excluded. | 57% of 168 | [Google Security Blog, Jan 2024](https://security.googleblog.com/2024/01/miracleptr-protecting-users-from-use.html) | 2024-01 | High |
| 8.4 | **Hardened libc++** in Chrome since 2022; across Google's servers it costs **0.30%** average, found 1,000+ bugs, and cut baseline segfault rate 30%. Spatial-safety bugs are **40%** of in-the-wild memory-safety exploits over a decade (Project Zero data, July 2014–Dec 2023). | 0.30%; 40% | [Google Security Blog, "Retrofitting spatial safety"](https://security.googleblog.com/2024/11/retrofitting-spatial-safety-to-hundreds.html) | 2024-11 | High |
| 8.5 | **V8 sandbox (why):** all in-the-wild Chrome exploits 2021–2023 began with renderer memory corruption; **60% were in V8**. V8 bugs are often logic bugs in the JIT that produce memory corruption, so memory-safe languages and MTE are poor fits; disabling JIT costs 1.5–10× on compute-heavy work and covers only ~half of exploited V8 bugs. | 60%; 1.5–10× | [V8 blog, "The V8 Sandbox"](https://v8.dev/blog/sandbox) | 2024-04-04 | High |
| 8.6 | **V8 sandbox (what):** only the V8 heap (not the stack) sits in a reserved region (1 TB reservation, 40-bit offsets, size "currently limited to 32GB"); off-heap objects via a pointer table (`external_ptr_t`). Threat model: arbitrary read/write inside the sandbox. Overhead ~1% on Speedometer/JetStream. Build-time only, 64-bit only. In the VRP from Chrome 123, described then as a "beta". | ~1% | [V8 blog](https://v8.dev/blog/sandbox) | 2024-04-04 | High |
| 8.7 | **V8 sandbox (bypass history):** early violations were "simple first-order bugs, such as a missing bounds check"; recent exploits "had to get past the sandbox". VRP bypass scope and rewards updated 2024-11-13. 2025 V8 CVEs (e.g. CVE-2025-8880) give code execution inside the sandbox, i.e. attackers now need a separate bypass. No public statement found that it is a complete security boundary as of 2026-10. | — | [V8 blog](https://v8.dev/blog/sandbox); [Chrome VRP FAQ](https://chromium.googlesource.com/chromium/src.git/+show/lkgr-android-internal/docs/security/vrp-faq.md) | 2024–2025 | Medium |

**Implications:** retrofits in C++ work but cost memory and cover partial classes (MiraclePtr 57% of UAFs, not renderers). Izcal avoids the retrofit by being Rust, but it embeds V8: the V8 heap sandbox and its build-time flag become Izcal's responsibility to keep enabled and to test against.

## 9. In-the-wild exploit data: where attackers actually go

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | 2020: "25% of the 0-days detected in 2020 are closely related to previously publicly disclosed vulnerabilities" (incomplete patches). Memory corruption still dominant. | 25% | [Project Zero, 2020 year in review](https://projectzero.google/2021/02/deja-vu-lnerability.html) | 2021-02 | High |
| 9.2 | 2022: 41 ITW 0-days (down from 69 in 2021); over 40% were variants of earlier bugs; browser 0-days fell, partly because "all major browsers also implemented new defenses". | 41; 17 variants | [Google TAG, "The ups and downs of 0-days"](https://blog.google/threat-analysis-group/0-days-exploited-wild-2022/) | 2023 | High |
| 9.3 | 2024: 75 ITW 0-days (98 in 2023); **browser 0-days fell from 17 to 11**; Chrome was the main browser target; UAF stayed prevalent; attackers shifted toward enterprise security and networking appliances. | 17 → 11 | [GTIG, 2024 zero-day exploitation analysis](https://cloud.google.com/blog/topics/threat-intelligence/2024-zero-day-trends) | 2025 | High |
| 9.4 | Android memory-safety vulnerabilities fell from **76% (2019) to 24% (2024)** after new code moved to memory-safe languages, though most code is still unsafe; vulnerabilities decay with code age (5-year-old code 3.4–7.4× lower density). Removing a now-unneeded sandbox made Chromium's Rust QR generator 20× faster. | 76% → 24% | [Google Security Blog, "Eliminating Memory Safety Vulnerabilities at the Source"](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html) | 2024-09-25 | High |

**Implications:** layered mitigations measurably moved attackers away from browsers; variants of known bugs are a quarter to two fifths of 0-days, so root-cause fixes matter more than patch speed. New code in a safe language is where the gain comes from, which is Izcal's position by construction.

## 10. Rust in browsers

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | **Stylo** (Firefox Quantum CSS, shipped Firefox 57, Nov 2017) after two failed C++ parallelization attempts. Of 69 historical security bugs in the style component, **51 (73.9%)** would have been impossible in Rust; 32 of 34 critical/high were memory-related. The rewrite still **reintroduced a previously fixed logic bug** (history leak via SVG, bug 1420001). | 51/69; 32/34 | [Mozilla Hacks, "Implications of Rewriting a Browser Component in Rust"](https://hacks.mozilla.org/2019/02/rewriting-a-browser-component-in-rust/) | 2019-02 | High |
| 10.2 | Chromium's 2023 Rust policy: support **third-party Rust libraries from C++ only**, **one-way interop (C++→Rust)**, because the browser process is "a giant interconnected system of (mutable) pointers" and passing references across languages is risky. Investment in Crubit for richer interop. | — | [Google Security Blog, "Supporting the Use of Rust in the Chromium Project"](https://security.googleblog.com/2023/01/supporting-use-of-rust-in-chromium.html) | 2023-01-12 | High |
| 10.3 | As of 2021 Rust was "not yet used in production versions of Chrome"; preferred plan was C++ safety (MiraclePtr, hardened STL) plus Rust. | — | [Google Security Blog, "An update on memory safety in Chrome"](https://security.googleblog.com/2021/09/an-update-on-memory-safety-in-chrome.html) | 2021-09 | High |
| 10.4 | **Fonts:** Chrome replaced FreeType with Rust **Skrifa** (Fontations); FreeType fuzz findings cost Google ≥0.25 FTE; system fonts on Linux/Android moved in Chrome 136; **FreeType removed from Blink in Chrome 145** (post updated 2026-04-24). | 0.25 FTE | [Chrome for Developers, "Memory safety for web fonts"](https://developer.chrome.com/blog/memory-safety-fonts) | 2025-03-19; upd. 2026-04-24 | High |

**Implications:** Rust removes most of the bug classes that dominated parser CVEs, but rewrites re-introduce old logic bugs; regression tests for every historical CVE class in the spec area are part of a rewrite. Chromium's interop pain is a C++-hosted problem; Izcal's equivalent is the Rust↔V8 C++ boundary, which deserves the same narrowness.

## 11. Hardware and compiler mitigations

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | **Clang CFI** in official Chrome on Linux x86-64: virtual calls from M54, indirect calls from M68. | M54; M68 | [chromium.org Control Flow Integrity](https://www.chromium.org/developers/testing/control-flow-integrity/) | living | High |
| 11.2 | **Intel CET shadow stacks** in Chrome 90 on Windows (20H1+, Intel 11th gen / Zen 3) for most processes but **not the renderer**: V8's writable JIT code lets attackers skip ROP, so V8 CET compatibility was not first priority. Incompatible: old security software that hooks with `push; ret`. | Chrome 90 | [Google Security Blog, "Enabling Hardware-enforced Stack Protection"](https://security.googleblog.com/2021/05/enabling-hardware-enforced-stack.html) | 2021-05 | High |
| 11.3 | **ARM PAC**: Apple first deployed Pointer Authentication in the A12 (2018). | 2018 | [Apple, MIE post](https://security.apple.com/blog/memory-integrity-enforcement/) | 2025-09 | High |
| 11.4 | **MTE** in Chrome: experimental from M115 (PartitionAlloc, 15 tags). Project Zero: async MTE is bypassable; for JS → renderer, "Spectre-type speculative side-channels can be used to break tag confidentiality, so known-tag-bypasses are a certainty". TIKTAG (IEEE S&P 2025) leaked tags across a whole Chrome renderer. | 15 tags | [Project Zero, "MTE as implemented, part 2"](https://projectzero.google/2023/08/mte-as-implemented-part-2-mitigation.html); [Arm blog](https://developer.arm.com/community/arm-community-blogs/b/operating-systems-blog/posts/enhancing-chromiums-memory-safety-with-armv9); [TIKTAG](https://github.com/compsec-snu/tiktag) | 2023-08; 2024 | High (P0); Medium (others) |
| 11.5 | **JIT vs mitigations:** Edge's "Super Duper Secure Mode" (2021) disabled the JIT so that CET and ACG (no RWX pages) could be enabled. ~45% of V8 CVEs after 2019 were JIT-related; over half of ITW Chrome exploits abused a JIT bug (Mozilla analysis). Measured: most tests unchanged; page-load regressions averaging ~17% in regressed tests. | 45%; ~17% | [Microsoft Edge VR team, "Super Duper Secure Mode"](https://microsoftedge.github.io/edgevr/posts/Super-Duper-Secure-Mode/) | 2021-08-04 | High |

**Implications:** every hardware mitigation so far has been weakened in the renderer by the JIT (W^X, CET) or by Spectre (MTE tags). Izcal should treat "JIT on" as a per-site capability with a measured cost, and plan its renderer to run well with JIT off.

## 12. Browser extensions: MV2 → MV3

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | Chrome's MV3 rationale: privacy, security, performance. Changes: background pages → service workers; **no remotely hosted code** ("unreviewed code"); blocking `webRequest` deprecated for `declarativeNetRequest` because filtering required extensions to "proxy all network traffic". | — | [Chrome docs, "What is Manifest V3"](https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3) | living | High |
| 12.2 | Timeline: deprecation paused Dec 2022 after feedback; MV2 disabled in pre-stable channels from June 2024 (Chrome 127); enterprise policy extension until June 2025. | — | [Chrome blog, "Resuming the transition to Manifest V3"](https://developer.chrome.com/blog/resuming-the-transition-to-mv3) | 2023-11 | High |
| 12.3 | **Criticism:** EFF called MV3 "outright harmful to privacy efforts", "doubtful" for security (a better review process would help more), cited a 2020 Princeton/UChicago study that privacy extensions improve performance, and pointed to Google's conflict of interest (trackers on 75% of top 1M sites). | 75% | [EFF, "Chrome Users Beware"](https://www.eff.org/deeplinks/2021/12/chrome-users-beware-manifest-v3-deceitful-and-threatening) | 2021-12 | High |
| 12.4 | Firefox implemented MV3 but **kept blocking webRequest and event pages**, and has "no plans to deprecate MV2". | — | [Mozilla Add-ons blog, March 2024](https://blog.mozilla.org/addons/2024/03/13/manifest-v3-manifest-v2-march-2024-update/) | 2024-03-13 | High |
| 12.5 | **The real extension threat is the update channel, not the API version:** Cyberhaven's MV3 extension was hijacked on 2024-12-24 via a phishing email posing as Chrome Web Store support and a malicious OAuth app; version 24.10.4 stole cookies and tokens for ~25 hours via auto-update; part of a wider campaign hitting other extensions. | ~25 h; ~400k (unverified) | [Cyberhaven preliminary analysis](https://www.cyberhaven.com/engineering-blog/cyberhavens-preliminary-analysis-of-the-recent-malicious-chrome-extension); [DomainTools](https://dti.domaintools.com/cyberhaven-breach-likely-part-of-a-long-term-criminal-campaign/) | 2024-12 | Medium |

**Implications:** MV3 traded user capability for reviewability, and the controversy shows that a browser's extension API is also a statement about whose interests the browser serves. Izcal should give content blocking a first-class, declarative and powerful API, and treat extension updates as a supply-chain channel (signing, staged rollout, permission-diff prompts).

## 13. Update systems and update-channel attacks

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 13.1 | Chrome's **Omaha** protocol: XML over HTTP POST, secured by **CUP** (client update protocol, signed request/response) or TLS; SHA-256 fingerprints of binaries. Silent auto-update is why Chrome could drop NPAPI and ship mitigations on a fixed cadence. | — | [Omaha Server Protocol V3](https://github.com/google/omaha/blob/main/doc/ServerProtocolV3.md) | living | High (protocol doc) |
| 13.2 | **TUF** threat model goes beyond "signed file": freeze attacks (same old file forever), rollback to an older vulnerable version, indefinite-version attacks, and key compromise; it uses separated roles and thresholds "to minimize the impact of key compromises". | — | [TUF overview](https://theupdateframework.io/docs/overview/) | living | High |
| 13.3 | **Notepad++ 2025:** state-linked attackers held the updater's shared hosting from ~June to 2 Sept 2025 (credentials until 2 Dec) and redirected selected users to malicious manifests; before 8.8.9 the updater did not verify integrity and authenticity. Shows that a signed binary is not enough if the manifest or host is unauthenticated. | ~6 months | [Help Net Security](https://www.helpnetsecurity.com/2026/02/02/2025-notepad-supply-chain-compromise/); [Unit 42](https://origin-unit42.paloaltonetworks.com/notepad-infrastructure-compromise/) | 2026-02 | Low–Medium |
| 13.4 | Browser release cadence is itself a security control: Mozilla plans release trains around Pwn2Own and ships chemspill fixes in about a day (4.7); but a user stuck on 123 had to update twice to reach 124.0.1. | — | [Mozilla Security Blog](https://blog.mozilla.org/security/2024/04/04/rapidly-leveling-up-firefox-security/); [The Register](https://www.theregister.com/2024/03/25/mozilla_fixes_firefox_zerodays/) | 2024-03 | Medium |

**Implications:** an updater is a remote-code-execution channel run as the user. Izcal's updater needs TUF-class metadata (rollback and freeze protection, threshold keys) and the same treatment for extensions and agent tools.

## 14. Certificates and PKI failures

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 14.1 | **DigiNotar (2011):** 531 fraudulent certificates for 344 domains incl. `*.google.com`, used for MITM mostly against users in Iran; Fox-IT found no basic security controls. Chrome users were protected because Chrome **detected** the fraudulent cert (static key pinning); Chrome and Firefox then disabled the CA. DigiNotar went bankrupt. | 531 certs; 344 domains | [Google Security Blog, Aug 2011](https://security.googleblog.com/2011/08/update-on-attempted-man-in-middle.html); [Fox-IT Black Tulip report](https://zoek.officielebekendmakingen.nl/blg-191225.pdf) | 2011-08; 2012-08 | High (Google); Medium (Fox-IT figures via secondary) |
| 14.2 | **Symantec distrust:** after a "continuing pattern of issues" (2015 incident, Jan 2017 misissuance), Chrome 66 (April 2018) distrusted pre-June-2016 certs and Chrome 70 (Oct 2018) all old Symantec infrastructure; issuance moved to DigiCert. Staged over ~18 months to avoid breaking sites. | Chrome 66; 70 | [Google Security Blog, "Chrome's Plan to Distrust Symantec Certificates"](https://security.googleblog.com/2017/09/chromes-plan-to-distrust-symantec.html) | 2017-09 | High |
| 14.3 | **Entrust distrust (2024):** "a pattern of concerning behaviors" in incident reports; Chrome 131 stopped trusting new certs whose earliest **SCT** is after 2024-11-11, a CT timestamp used as the cut-over. | Chrome 131 | [Google Security Blog, "Sustaining Digital Certificate Security"](https://security.googleblog.com/2024/06/sustaining-digital-certificate-security.html) | 2024-06 | High |
| 14.4 | **Certificate Transparency:** append-only, publicly auditable Merkle logs; monitors let domain owners see every cert for their names. Chrome requires SCTs from qualified logs (CT Compliance); before Chrome 148 it also accepted SCTs in stapled OCSP. CT turned distrust from guesswork into evidence. | — | [certificate.transparency.dev](https://certificate.transparency.dev/howctworks/); [Chrome CT Policy](https://googlechrome.github.io/CertificateTransparency/ct_policy.html) | living | High |

**Implications:** the PKI lesson is that trust anchors fail by organizational behavior, not cryptography, and that browsers enforce by **staged, evidence-based distrust** with transparency logs as the evidence. Izcal needs a root-store policy process and CT enforcement, not just a root list.

## 15. Agents as a new principal (2025–2026)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 15.1 | Brave showed **indirect prompt injection** in Perplexity Comet: page content fed to the LLM without separating it from the user's instruction; hidden text could make the agent read the user's email in another tab. It "demonstrates how easy it is to manipulate AI assistants into performing actions that were prevented by long-standing Web security techniques". | — | [Brave, "Agentic Browser Security: Indirect Prompt Injection in Perplexity Comet"](https://brave.com/blog/comet-prompt-injection/) | 2025-08-20 | High |
| 15.2 | Chrome's agent design: a **User Alignment Critic** (separate model, isolated from untrusted content, can veto actions) and **Agent Origin Sets** (read-only vs read-writable origins per task, chosen by a gating function not exposed to web content; non-task iframes are hidden from the model); plus user confirmations and red-teaming. Chrome calls prompt injection "the primary new threat facing all agentic browsers". | — | [Google Security Blog, "Architecting Security for Agentic Capabilities in Chrome"](https://security.googleblog.com/2025/12/architecting-security-for-agentic.html) | 2025-12-08 | High |
| 15.3 | OpenAI (ChatGPT Atlas, Dec 2025): prompt injection is "unlikely to ever be fully 'solved'"; agent mode "expands the security threat surface"; defense is automated RL red-teaming and adversarial training. | — | [CyberScoop](https://cyberscoop.com/?p=87276); [Fortune](https://www.fortune.com/2025/12/23/openai-ai-browser-prompt-injections-cybersecurity-hackers) (OpenAI post returned 403) | 2025-12-22 | Low–Medium |

**Implications:** the agent is a confused deputy that holds the user's ambient authority across all origins; origin isolation was built for code, not for a model that reads everything. Izcal's B9 agent principal should be deterministic-capability-first (origin sets, data-flow limits) with model-based checks as a second layer, since vendors themselves say the model layer cannot be complete.

## Lessons for Izcal

| Project | What happened | Lesson |
|---|---|---|
| Monolithic browsers (to 2008) | 67–70% of severe bugs were in the rendering engine, which ran with full user privilege. | Put untrusted parsing and rendering in a process with no ambient authority from day one. |
| IE7 Protected Mode | Low integrity blocked writes, not reads, and kept all sites and add-ons in one process. | Define the sandbox by what the attacker cannot read or reach, not only what they cannot persist. |
| Chrome 2008 | Deferred cross-site isolation and ran plugins unsandboxed "for compatibility"; both took about ten years to fix. | Compatibility exceptions to the security model become the next decade's work; refuse them early or bound them in time. |
| Firefox e10s | Seven years of delay because the add-on API assumed shared memory. | Extension and agent APIs must be asynchronous and message-based so the process model can change later. |
| Chrome Site Isolation | 10–13% memory, and disabled on Android under 2 GB RAM. | Per-site isolation needs an explicit memory budget and a degradation story that keeps I-1, not one that silently drops it. |
| Spectre / V8 | "No known comprehensive software mitigations" for same-address-space secrets. | Treat the process (address space) as the only confidentiality boundary; in-process language checks give integrity, not secrecy. |
| COOP/COEP | Opt-in headers took years and needed a softer `credentialless` mode. | Prefer safe defaults over developer opt-ins; design opt-ins to be deployable incrementally. |
| NPAPI, ActiveX, Flash | Second platforms with their own update channels and full privilege died once standards covered their use cases. | Never host a second, privileged code platform inside the browser; extend the standard platform instead. |
| NaCl/PNaCl | Worked, but single-engine; lost to cross-browser Wasm with low usage. | Page-visible capabilities must be standards-track or they are dead ends. |
| MiraclePtr | 57% of privileged-process UAFs mitigated at 3.5–6.5% memory; renderers excluded. | Retrofits are partial and costly; a memory-safe implementation language is the cheaper path, and the remaining C++ (V8) needs its own containment. |
| V8 heap sandbox | 60% of ITW Chrome exploits were V8; JIT logic bugs defeat language safety. | Izcal's Rust safety ends at the V8 boundary; keep the V8 sandbox enabled, fuzz the boundary, and treat V8 as hostile to the rest of the renderer. |
| Rule of 2 | Clear rule, but blocks features when a process is too expensive. | Give Izcal a cheap second isolation tier (Wasm/RLBox-style) so the rule does not force bad trade-offs. |
| RLBox | In-process Wasm isolation for libraries, with bypass bounties. | Libraries that parse untrusted input can be sandboxed without a process; pay for boundary bypasses, not only bugs. |
| Stylo | 73.9% of historical style bugs impossible in Rust, but the rewrite reintroduced a fixed logic bug. | Carry every historical CVE as a regression test when writing a new component. |
| Android Safe Coding | Memory-safety share fell 76% → 24% by writing new code safely. | The gain comes from new code; avoid importing large unsafe libraries, since vulnerability density is highest in new code. |
| CET / Edge SDSM / Lockdown Mode | JIT blocks W^X, CET and ACG; vendors ship JIT-off modes for high-risk users. | Make JIT a per-site capability and keep a well-performing no-JIT mode. |
| MTE / MIE | Async MTE bypassable, tags leak via Spectre; Apple needed sync EMTE plus allocator type segregation plus tag confidentiality. | Hardware tagging helps only with sync mode and a typed allocator; design Izcal's allocators to be type-segregated so they benefit. |
| 0-day data | 25–40% of ITW 0-days were variants of fixed bugs. | Fix root causes and bug classes, not single reports; track variants. |
| Cyberhaven, Notepad++ | Auto-update of a trusted extension or app delivered malware within hours. | Treat every update channel (browser, extensions, agent tools) as TUF-class supply chain with rollback and freeze protection. |
| DigiNotar, Symantec, Entrust | CAs failed through organizational behavior; browsers distrusted them in stages using CT evidence. | Run a root-store policy with CT enforcement and staged distrust; pin nothing permanently. |
| MV3 | Reviewability bought at the cost of user capability; Firefox kept blocking webRequest. | The extension API states whose agent the browser is; give users strong declarative blocking. |
| Comet, Chrome agents, Atlas | Prompt injection turns the agent into a confused deputy; vendors say it cannot be fully solved. | Bound the agent principal with deterministic origin and data-flow capabilities first; model checks are a second layer. |

## Technology mismatches

Assumptions baked into today's browser security that no longer hold, or never held everywhere:

| Assumption | Where it comes from | Why it no longer holds | Evidence |
|---|---|---|---|
| A process per site is affordable | Chrome 2018 desktop, 10–13% overhead | Low-end phones and WebView cannot afford it; Chrome gives up under 2 GB RAM and isolates only login sites on Android. | 7.3, 7.6 |
| Language-level checks give confidentiality | Same-origin policy inside one JS heap | Spectre reads any same-process memory; tags and pointers leak too. | 7.1, 11.4 |
| JIT is free | 2008 JIT race | JIT bugs dominate V8 exploits and block CET, ACG and W^X; vendors now ship JIT-off modes. | 8.5, 11.2, 11.5, 5.4 |
| The human is the only principal | Every permission prompt and origin model | Agents act across origins with the user's authority and read attacker text as input. | 15.1–15.3 |
| The renderer is the main attack target | 2008 analysis | Escapes now often come from logic bugs in IPC and OS boundaries; attackers moved to GPU process and appliances. | 3.8, 5.2, 9.3 |
| Signed binaries make updates safe | Classic code signing | Manifest, host and developer-account compromise bypass it (Notepad++, Cyberhaven). | 12.5, 13.3 |
| CAs are trustworthy by audit | 1990s PKI | Trust fails through behavior; transparency logs are what make enforcement possible. | 14.1–14.4 |
| A fixed process pool is "multi-process enough" | Firefox e10s 2016 | Spectre made any site sharing a process a leak. | 4.3, 4.4 |
| Extensions are trusted extensions of the user | XUL, MV2 | Extensions are supply-chain targets and data-harvesting vectors; the API now encodes policy. | 12.1–12.5 |
| Memory safety can be retrofitted to C++ | MiraclePtr, hardened libc++ | Retrofits cover partial classes at a memory cost; new safe code is what lowers the share. | 8.2–8.4, 9.4 |
| Hardware mitigations are universal | CET, MTE, PAC launches | They depend on CPU generation, OS version and vendor (CET needs 11th-gen Intel/Zen 3; MIE only on iPhone 17), so a browser must work without them. | 11.2, 5.5 |

## Gaps and flagged claims

- **V8 sandbox status in 2026** (whether declared a full security boundary) is unconfirmed; only the 2024 "beta" statement and the Nov 2024 VRP update were found.
- **Safari site isolation shipping status** (which Safari version, if any, enables it by default) is unconfirmed; only WebKit bug and mailing-list evidence.
- **Chromium Rust share of shipped code and current memory-safety bug share** after 2023 were not found in a primary source; only the 2020 "70%" and the fonts post.
- **Firefox e10s pause dates** come from press, not Mozilla.
- The **Cyberhaven ~400k** figure and **Notepad++ attribution** are secondary and contested.
- **OpenAI Atlas** wording is from press quotes; the OpenAI post returned 403.
- **WebKit2 2010** and **Fission default in Firefox 95** dates are from prior knowledge, not re-fetched.
- **MTE default status in Chrome on Android** in 2025–2026 not found.

Finding count: 79 numbered findings in sections 1–15.
