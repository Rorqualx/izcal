# Izcal security scope

*Overarching security design and threat model for Izcal. Date: 18 October 2026. Status: draft 0.3. Research notes: `research_notes/Izcal security scope/` (five research strands, plus `factcheck.md` and `critique.md`). Relates to: about forty existing IDs in HGBA-2, HGBA-3, the Stage 1 design, the DevTools spec, the shell UI design, `CONTRIBUTING.md` and `tools/checks.sh`; Section 6.4 maps each one.*

*Revision 0.1 (18 October 2026): first draft.*

*Revision 0.2 (18 October 2026): draft 0.1 was independently fact-checked (157 claims; `factcheck.md`) and design-critiqued (39 findings: 8 high, 20 medium, 11 low; `critique.md`), and this revision applies both. Fact-check: the fork's pipeline state now separates `main`, the fork's `hgba` branch and the open fork PR #2; the V8 tally, the "logic bugs were all escapes" claim, the Stage 1 codec and PDF sandbox, the existing patch-lag metric, the jitless cost, the advisory counts, the attributions and the citations are corrected; the broken cookie-encryption reference, the retention double-decision and the threat-model timing are fixed. Critique: the broker, not the shell, now decides consent (CON-R1 to CON-R5, SEC-29); the replay host and automation gateway have sandbox tiers that match what they do; SEC-1 tracks a Chrome-patched branch, starts its clock at the upstream fix and has a failure action that bites; an emergency release class and a delegated TUF `releases` role keep the 72-hour path open; every process kind is treated as a possible attacker (IPC-R17); IPC-R2 gains bound and free endpoints and per-frame identity; WebRTC, the native PlayReady path and the updater get models and targets; the GPU plan relies on wgpu-core's validation and names WebRender display lists as an IPC exception (IPC-R16). New: a per-process index (6.3a), rule IDs for the prose rules (CON-R, AGT-R, REC-R, SH-R, NET-R, FFI-R, GPU-R, UPD-R), TB-13, SEC-29 to SEC-35, SEC-D11 to SEC-D14, Q-SEC-13 to Q-SEC-19 and a review-disposition appendix. SEC-D8 and SEC-D9 are retired as decisions and became rules. Appendix B lists every ID change.*

*Revision 0.3 (18 October 2026): the owner decided the panic strategy (coding standard CS-D1 (a)). Every shipping process aborts on panic, the network service included: IPC-R5, NET-R3, SEC-28 and the network service's mitigations now require that no panic be reachable from network input, in place of a per-connection `catch_unwind`.* *The owner accepted the recommendation for SEC-D1 to SEC-D5, SEC-D7 and SEC-D10 to SEC-D14; SEC-D7's sources are chosen (CERT Polska, Phishunt and URLAbuse for phishing; URLhaus for malware URLs once licensed), with the licensing shelved until the M8 pre-release gate. SEC-D6 is decided (b) after the macOS 27.0.1 measurement: custom profiles still apply and only named profiles are killed, but Apple does not call custom profiles supported, so the evaluation stays and adds Endpoint Security. IPC-R5 and SEC-7 (b) name the allowlist file, `tools/coding-standard/tier-p-exceptions.toml` (coding standard amendment).*

Izcal's security rules are spread over six documents and about forty IDs, and no document says which attacker each rule stops or which process holds what. This spec unifies them into one threat model and one map, and supplies the per-process threat models that Stage 1 requires before M6. Its central finding is that **Izcal's own code removes the bug class that dominated the last decade, and the risk moves to two places.** The first is code Izcal ships but does not own: V8 (21 of 37 Chrome in-the-wild 0-days from 2022 to September 2026, 57%, counting CVE-2024-0519 as V8), ANGLE and GPU drivers, C codecs, and the OS engines in the protected-media host. There the exposure is **patch latency and containment**. The second is **logic at the boundaries Izcal does own**: handle relay and identity in IPC (CVE-2025-2783 escaped Chrome's sandbox with no memory corruption), consent drawn by a renderer, grants for agents and automation clients, and the release and update pipeline. The decisions that matter most: (1) **severity-tiered patch-latency targets** for V8 security fixes (SEC-1), on a pinned V8 branch that Chrome still patches, with a clock that starts at the upstream fix and an emergency path that keeps them reachable (SEC-D1, SEC-D11, SEC-D12); (2) **IPC rules enforced by the generator**, with identity taken from bound endpoints, handles typed, broker-minted and only narrowed, and **no process trusting any other**, tested by the deterministic simulator acting as each compromised process kind (SEC-3 to SEC-6); (3) **consent decided by the broker** from OS input it hit-tests itself, never by the shell renderer (SEC-29); (4) a **GPU-service target** that relies on wgpu-core's validation and adds ownership checks, clamps and fuzzing (SEC-10), and Stage 1's existing **library sandbox for codecs and PDF made a Stage 1 target** (SEC-12); (5) a **verifiable artifact chain** from V8 build to user update, with controls sized for a project that has one maintainer (SEC-13 to SEC-17, SEC-27); (6) **agent and automation entry points, memory and URL schemes treated as untrusted channels**, plus a Rule-of-Two task policy (SEC-18 to SEC-21). The work adds a **security track of 3–4 engineers†** to Stage 1's 23–30.5.

**How to read this document.** Every engineer reads Sections 6.1–6.3a; then:
- **IPC and service authors:** 6.6 (IPC-R1 to IPC-R17), 6.6a (consent), SEC-3 to SEC-7, the process's row in 6.3a and its model in 6.5.
- **FFI and bindings authors:** 6.8 (FFI-R1 to FFI-R4), SEC-1, SEC-11.
- **GPU and media authors:** 6.9, 6.10, 6.10a (GPU-R, WebRTC, CDM), SEC-10, SEC-12, SEC-30, SEC-33.
- **Network authors:** 6.11 (NET-R), SEC-28, SEC-31.
- **Release maintainers:** 6.16 (UPD-R), 6.17, SEC-13 to SEC-17, SEC-27, SEC-D4, SEC-D11.
- **DevTools, automation and agent authors:** 6.12 and 6.13 (AGT-R, REC-R), SEC-18 to SEC-21.
- **Shell authors:** 6.6a (CON-R), 6.14 and 6.14a (SH-R), SEC-22, SEC-23, SEC-29, SEC-32.

---

## 1. The problem and who has it

**Users.** Most Izcal users are ordinary users who want to browse without being tracked or robbed. A small number will be high-risk users. Commercial surveillance vendors were behind 64% of browser and mobile 0-days in 2023 (24 of 37, [TAG 2023 review](https://blog.google/technology/safety-security/a-review-of-zero-day-in-the-wild-exploits-in-2023/)), and a new browser chosen for its privacy stance attracts exactly the people they target. Both groups need the same thing: a compromised page must not reach other sites' data, the user's files, or the user's credentials, and a fix for a known exploit must arrive before attackers use it. Ordinary users are also hit most often by the plainest routes: a malicious download, an external protocol handler, a phishing page (Sections 6.14 and 6.14a).

**Developers and automation users.** DevTools, WebDriver BiDi, CDP, DAP and MCP clients drive the browser from outside. Each is a principal with power over the user's sessions.

**Agents.** With AI switched on (B10), an agent acts for the user across origins and reads attacker-written text as input.

**The project.** Izcal has one maintainer, who owns both repositories ([`CONTRIBUTING.md`](../CONTRIBUTING.md), "Code review": self-merge is allowed). The maintainer's account, workstation and signing keys are the largest single asset an attacker could take. Until a second person and a distinct security owner exist, every "security owner approves" gate in this spec is a self-approval; Section 9.2 makes that visible.

**What the existing specs already decide.** HGBA-2 ranks security first (P1) and fixes the isolation model (I-1 to I-6), the targets S-1 to S-8 and the FFI policy (Section 4.8). HGBA-3 adds the deterministic substrate (B1), typed capability IPC (B2), the agent principal (B9), AI off by default (B10) and the proof target VER-1. The Stage 1 design fixes the process map (Section 1), including utility processes with an RLBox-style library sandbox for C decoders and PDF; the crate rules (G1–G9); IPC (Section 4); fuzzing (7.5); the upstream pipeline with a V8 lag metric and an escalation at more than two weeks (7.7); and the requirement for **one threat model per process, written before M6** (13.2). The DevTools spec sets the session and consent model (6.5). M-1 and D-S10 forbid copying another browser's code and make "build our own, following our specs" the standing answer.

**What is missing.** No single attacker model; no map from assets to processes; no severity-tiered target for patch latency; no target for the GPU service, IPC logic, consent integrity, the release pipeline or AI-scale bug volume; no models for WebRTC, the native DRM path or the updater; and several findings from 2025–2026 that no spec addresses (Section 3.3).

**Success, in terms someone can measure later.** Every attacker class and every process has a written model with named mitigations and residual risk (SEC-26). Every research finding has a decision, an open question or an explicit "out of scope" (Sections 6, 9.3 and 11). Every new rule has an ID, and every target has a threshold and a failure action (Section 8). The plan fits Stage 1's milestones with a stated staffing cost (Section 9).

---

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1996–2006 | ActiveX, BHOs, single-process browsers | Native code and all sites in one process with the user's full privilege | Single-core PCs with little RAM; add-ons assumed in-process access ([Edge team](https://blogs.windows.com/msedgedev/2015/05/06/a-break-from-the-past-part-2-saying-goodbye-to-activex-vbscript-attachevent/)) | Replaced: one bug gave full compromise. Of the IE, Firefox and Safari bugs Barth et al. studied, 67.4%, and 70.4% of the code-execution bugs, would have fallen in Chromium's rendering engine ([Barth et al. 2008](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf)) |
| 2006–2007 | IE7 Protected Mode | Low integrity plus brokers on Vista | Vista's new integrity levels; the threat was drive-by installs ([IEBlog](https://learn.microsoft.com/en-us/archive/blogs/ie/protected-mode-in-vista-ie7)) | Superseded: it blocked writes, not reads, and kept all sites and add-ons in one process |
| 2008 | Chrome | Sandboxed renderers, trusted browser kernel, IPC ([Barth et al.](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf)) | Renderer RCE was the dominant bug class; multi-core PCs | Survived. Two deferred items, cross-site isolation and unsandboxed plugins, took about ten years to fix |
| 2009–2016 | Firefox e10s | Multi-process Firefox, shipped in Firefox 48 to about 1% of users | — | Seven years late because the add-on API assumed shared memory; 8 of the top 20 extensions were compatible ([Computerworld](https://www.computerworld.com/article/1668630/mozilla-supplies-multi-process-firefox-to-1-of-users.html), secondary) |
| 2008–2020 | NPAPI, Flash, NaCl | Second, privileged code platforms inside the browser | Native speed and features the web lacked | Removed once standards covered the use cases; NaCl lost to cross-browser Wasm ([Chromium blog, NPAPI](https://blog.chromium.org/2013/09/saying-goodbye-to-our-old-friend-npapi.html); [PNaCl](https://blog.chromium.org/2017/05/goodbye-pnacl-hello-webassembly.html)) |
| 2010 | Capsicum on Chromium | Capability-mode sandbox in about 100 lines | The browser already passed descriptors and used RPC everywhere ([Watson et al.](https://www.usenix.org/legacy/event/sec10/tech/full_papers/Watson.pdf)) | Linux port never merged; the idea survives in Fuchsia, WASI and Landlock |
| 2011–2018 | DigiNotar, Symantec; Certificate Transparency | Misissued certificates; staged distrust; public logs | CAs fail through behavior, not cryptography ([Google 2011](https://security.googleblog.com/2011/08/update-on-attempted-man-in-middle.html); [Symantec plan](https://security.googleblog.com/2017/09/chromes-plan-to-distrust-symantec.html)) | CT made distrust evidence-based ([CT](https://certificate.transparency.dev/howctworks/)) |
| 2017 | EdgeHTML ACG | Moved the JIT out of process so content processes could forbid writable code | A JIT needs exactly what ACG forbids ([Edge blog](https://blogs.windows.com/msedgedev/2017/02/23/mitigating-arbitrary-native-code-execution/)) | EdgeHTML retired; the trade-off between JIT and code-integrity mitigations remains |
| 2018–2021 | Site Isolation, Fission | One process per site against Spectre and compromised renderers | "No known comprehensive software mitigations" for same-process secrets ([McIlroy et al.](https://arxiv.org/abs/1902.05178)) | Shipped. 10–13% memory on desktop with all sites isolated (about 3–5% for Android's partial isolation); below 2 GB of RAM on Android it is "not yet supported" ([Google 2018](https://security.googleblog.com/2018/07/mitigating-spectre-with-site-isolation.html); [chromium.org](https://www.chromium.org/Home/chromium-security/site-isolation/)) |
| 2021–2022 | RLBox; Firefox Win32k lockdown | In-process Wasm sandboxes for C libraries; renderers with no OS UI | Processes have a fixed memory cost ([Mozilla Hacks](https://hacks.mozilla.org/2021/12/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/)); win32k was a kernel escape route ([Mozilla Hacks](https://hacks.mozilla.org/2022/05/improved-process-isolation-in-firefox-100/)) | Shipped. The wider security-boundary re-architecture behind Win32k lockdown took about four years |
| 2022–2024 | MiraclePtr, hardened libc++, V8 heap sandbox | Retrofit memory safety into C++ | 70% of Chrome's severe bugs are memory safety ([Chromium](https://www.chromium.org/Home/chromium-security/memory-safety/)) | Partial: MiraclePtr mitigated 57% of 168 eligible use-after-frees, not in renderers ([Google 2024](https://security.googleblog.com/2024/01/miracleptr-protecting-users-from-use.html)); in 2024 the V8 team listed "a number of issues to resolve before it becomes a strong security boundary" ([V8](https://v8.dev/blog/sandbox)) |
| 2021–2025 | CET, Lockdown Mode, Apple MIE | Hardware and mode-level mitigations | Targeted exploit chains | In 2021 (Chrome 90) CET was off in Chrome's renderer because of the JIT ([Google 2021](https://security.googleblog.com/2021/05/enabling-hardware-enforced-stack.html)); Apple reports that six real-world exploit chains it evaluated could not be rebuilt under MIE (vendor claim, [Apple](https://security.apple.com/blog/memory-integrity-enforcement/)) |
| 2023–2026 | Rust in browsers | Fontations replaced FreeType in Chrome; Android new code in Rust | Unsafe parsers of untrusted input | Fontations: 39 fuzzer bugs, none security-critical ([Chrome](https://developer.chrome.com/blog/memory-safety-fonts)); Android memory-safety share below 20% in 2025 ([Google](https://blog.google/security/rust-in-android-move-fast-fix-things/)) |
| 2024–2026 | xz, tj-actions, Clinejection, arrayref | Attacks on maintainers, CI and registries | Owning a person, a token or a builder is cheaper than breaking crypto | Ongoing; in the arrayref compromise (August 2026) three republished crates pulled in the typosquat `proc-macro1`, whose build script ran a payload at `cargo build` ([Rust blog](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/)) |
| 2025–2026 | Agentic browsers | An agent acts on the user's behalf and reads pages | A new principal | Prompt injection "unlikely to ever be fully solved" (OpenAI, via press); Chrome added origin sets and a critic ([Google](https://security.googleblog.com/2025/12/architecting-security-for-agentic.html)) |

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption | Where it came from | Why it no longer holds | Evidence |
|---|---|---|---|
| The renderer is the main target | 2008 analysis | Escapes now go through IPC logic, GPU drivers and OS services | Section 3.2 |
| Memory safety removes the escape class | Rust adoption | The 2025 handle-relay escapes in Chrome and Firefox were logic bugs | [Securelist](https://securelist.com/forumtroll-apt-hacking-team-dante-spyware/117851/); [MFSA 2025-19](https://www.mozilla.org/en-US/security/advisories/mfsa2025-19/) |
| JIT is free | The 2008 JIT race | A JIT conflicts with ACG and CET, and Edge shipped a JIT-off mode to get them | [Edge SDSM](https://microsoftedge.github.io/edgevr/posts/Super-Duper-Secure-Mode/) |
| In-process checks give confidentiality | Same-origin policy in one heap | Spectre reads any same-process memory | [McIlroy et al.](https://arxiv.org/abs/1902.05178) |
| The human is the only principal | Every permission model | Agents and automation clients act with the user's authority | Section 3.4 |
| A signed binary makes an update safe | Classic code signing | Manifest, host and account compromise bypass it | [Notepad++](https://notepad-plus-plus.org/news/hijacked-incident-info-update/); [Cyberhaven](https://www.cyberhaven.com/engineering-blog/cyberhavens-preliminary-analysis-of-the-recent-malicious-chrome-extension) |
| A tag names a fixed version | Git conventions | Attackers moved tags of popular actions | [Wiz on tj-actions](https://threats.wiz.io/all-incidents/tj-actionschanged-files-supply-chain-attack); [The Hacker News on actions-cool](https://thehackernews.com/2026/05/github-actions-supply-chain-attack.html) |
| The OS sandbox API is stable | Seatbelt since 2007 | Apple calls SBPL "not API" and SDK 27 kills some profiles | [Apple DTS](https://developer.apple.com/forums/thread/777509); [openssh-unix-dev](https://lists.mindrot.org/pipermail/openssh-unix-dev/2026-August/042690.html) |
| Bugs in a new engine stay hidden for a while | Obscurity | AI models find hundreds of bugs per release in mature engines | [Mozilla, 271 fixes](https://blog.mozilla.org/en/firefox/ai-security-zero-day-vulnerabilities/) |
| Only the renderer gets compromised | 2008 analysis | Every sandboxed service parses hostile input (GPU commands, network bytes, media), so any of them can be the attacker's foothold | Section 6.1 (ATK-2) |
| The browser's own UI is trusted | Single-process browsers | Izcal's shell is a renderer that shows every site's strings (D-S2) | Section 6.6a |
| Hardware mitigations are universal | CET, MTE, PAC launches | They depend on CPU generation, OS and vendor | [Google 2021](https://security.googleblog.com/2021/05/enabling-hardware-enforced-stack.html); [Apple MIE](https://security.apple.com/blog/memory-integrity-enforcement/) |

---
## 3. Today: the landscape and its gaps

### 3.1 Where attackers go

- **Volume is down, price is up.** GTIG counted browser 0-days falling from 17 (2023) to 11 (2024) to "less than 10%" of 90 in 2025, a historical low. GTIG does not decide whether hardening or better attacker secrecy explains it ([GTIG 2024](https://cloud.google.com/blog/topics/threat-intelligence/2024-zero-day-trends); [GTIG 2025](https://cloud.google.com/blog/topics/threat-intelligence/2025-zero-day-review/)).
- **V8 is the entry point.** Of 37 Chrome in-the-wild 0-days from 2022 to September 2026 in Project Zero's sheet, **21 (57%) were V8**, 14 of them type confusion; that count includes CVE-2024-0519, which the sheet lists only as "Out-of-bounds memory access". Graphics (GPU, Skia, ANGLE, Dawn) were 6 (16%); media codecs and WebRTC 4 (11%) ([P0 sheet](https://docs.google.com/spreadsheets/d/1lkNJ0uQwbeC1ZTRrxdtuPLCIl7mlUreoKfSIgajnSyY/), tally in the research notes).
- **Logic bugs reach the broker.** 3 of the 37 were logic or validation bugs: CVE-2022-3075 ("Insufficient data validation in Mojo"), CVE-2022-2856 ("Insufficient validation of untrusted input in Intents") and CVE-2025-2783. The sheet labels only CVE-2025-2783 a sandbox escape; the other two have no escape analysis there (same tally).
- **Variants recur.** 25% of 2020's 0-days and over 40% of 2022's (17 of 41) were variants of known bugs ([Project Zero 2020](https://projectzero.google/2021/02/deja-vu-lnerability.html); [TAG 2022](https://blog.google/threat-analysis-group/0-days-exploited-wild-2022/)).
- **Defenders ship fast when they plan for it.** Mozilla fixed a Pwn2Own chain in under 21 hours and an in-the-wild exploit in 25 hours ([Mozilla 2024-04](https://blog.mozilla.org/security/2024/04/04/rapidly-leveling-up-firefox-security/); [Mozilla 2024-10](https://blog.mozilla.org/security/2024/10/11/behind-the-scenes-fixing-an-in-the-wild-firefox-exploit/)).

### 3.2 How chains escape

A full chain is a renderer bug, usually V8, then an escape through a GPU user-mode driver, an OS service or a browser IPC logic bug, then a kernel bug on mobile. **None of the 2025 escapes GTIG analysed was generic to the browser sandbox**: the mobile chains went through ART or the Adreno and Mali libraries, CVE-2025-6558 was a Mali-driver route fixed in ANGLE, and CVE-2025-14174 reached Metal through ANGLE in an Apple chain ([GTIG 2025](https://cloud.google.com/blog/topics/threat-intelligence/2025-zero-day-review/)). GTIG files CVE-2025-2783 separately, as an OS-based Windows escape. It relayed Windows pseudo-handle `-2` through ipcz; the broker checked only `-1` and "converted -2 into a real handle to its own thread" ([Securelist](https://securelist.com/forumtroll-apt-hacking-team-dante-spyware/117851/)). Mozilla then fixed the same pattern in Firefox (CVE-2025-2857, [MFSA 2025-19](https://www.mozilla.org/en-US/security/advisories/mfsa2025-19/); the advisory does not give the timing). Firefox's parent once trusted a renderer-sent origin (`"__proto__"`, CVE-2022-1529) and a renderer-sent URL (CVE-2019-11708) ([ZDI](https://www.zerodayinitiative.com/blog/2022/8/23/but-you-told-me-you-were-safe-attacking-the-mozilla-firefox-renderer-part-2); [Mozilla A&D](https://blog.mozilla.org/attack-and-defense/?p=217)). At Pwn2Own Berlin 2026, Edge's sandbox fell to four chained logic bugs, and no team entered Chrome on any of the three days ([ZDI day 1](https://www.zerodayinitiative.com/blog/2026/5/13/pwn2own-berlin-2026-day-one-results); [day 2](https://www.thezdi.com/blog/2026/5/15/pwn2own-berlin-2026-day-two-results); [day 3](https://www.zerodayinitiative.com/blog/2026/5/16/pwn2own-berlin-2026-day-three-results-and-master-of-pwn)).

### 3.3 Rust, supply chain and AI-scale discovery

- **Rust code fails differently.** In 2026 rustls had five advisories, none memory corruption, among them a downgrade to classical key exchange, ECH data leakage and a 1 GiB pre-authentication allocation ([RustSec rustls](https://github.com/rustsec/advisory-db/tree/main/crates/rustls)). quinn-proto has 10 advisories, 8 of them from 2026, all denial of service ([RustSec quinn-proto](https://github.com/rustsec/advisory-db/tree/main/crates/quinn-proto)). Most of Deno's 40 advisories (about 35 under a broad definition) are permission and logic bypasses; its two memory bugs sit exactly at the Rust↔V8 boundary: `v8::External` lifetimes ([GHSA-3j27-563v-28wf](https://github.com/denoland/deno/security/advisories/GHSA-3j27-563v-28wf)) and a resizable ArrayBuffer shrunk under an async Rust operation ([GHSA-c25x-cm9x-qqgx](https://github.com/denoland/deno/security/advisories/GHSA-c25x-cm9x-qqgx)).
- **Rust's registry is a target.** RustSec lists 75 malicious-crate advisories, 46 since 2024 ([advisory-db](https://github.com/rustsec/advisory-db), tally). In the arrayref compromise, three crates were republished with a dependency on the typosquat `proc-macro1`, whose build script ran a payload; they were live for 86–107 minutes ([Rust blog](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/)).
- **CI is a target.** Every 2025–2026 GitHub Actions compromise in the research used a moved tag, a privileged trigger reading untrusted text (including an AI agent's prompt), a shared cache, or a long-lived token ([Wiz on tj-actions](https://threats.wiz.io/all-incidents/tj-actionschanged-files-supply-chain-attack); [PyPI on Ultralytics](https://blog.pypi.org/posts/2024-12-11-ultralytics-attack-analysis/); [Clinejection](https://adnanthekhan.com/posts/clinejection)).
- **AI finds bugs at volume.** Claude Opus 4.6 found 22 Firefox vulnerabilities in two weeks ([Anthropic](https://www.anthropic.com/news/mozilla-firefox-security)); Firefox 150 fixed 271 found with Claude Mythos Preview, and Mozilla found "no category or complexity of vulnerability that humans can find that this model can't" ([Mozilla](https://blog.mozilla.org/en/firefox/ai-security-zero-day-vulnerabilities/)). RustSec advisories rose to 256 in 2026 from 73–81 a year in 2023–2025, and Wasmtime's to 51 from about 4 a year (8 in 2022) (tallies; the AI cause is an inference).

### 3.4 New surfaces

- **Agents.** Every 2025–2026 agent attack entered through a channel the vendor trusted: an agent-invocation URL parameter (CometJacking, [The Hacker News](https://thehackernews.com/2025/10/cometjacking-one-click-can-turn.html)), persistent memory written by CSRF ([The Hacker News](https://thehackernews.com/2025/10/new-chatgpt-atlas-browser-exploit-lets.html)), local files through `view-source:file://` and a password-manager extension (PleaseFix, [Zenity](https://zenity.io/research/pleasefix-vulnerabilities)), and malicious extensions with basic permissions taking over the AI panel (CVE-2026-0628, [Unit 42](https://unit42.paloaltonetworks.com/gemini-live-in-chrome-hijacking/)). Adaptive attacks broke 12 published defenses, most at over 90% success ([arXiv 2510.09023](https://arxiv.org/abs/2510.09023)); Anthropic reports 1% success against its own adaptive attacker with 100 attempts, and calls that "meaningful risk" ([Anthropic](https://www.anthropic.com/research/prompt-injection-defenses)).
- **Debug and automation ports.** Infostealers start Chrome in remote-debugging mode to read cookies after App-Bound Encryption ([Elastic](https://www.elastic.co/security-labs/threat-command/katz-and-mouse-game)); Chrome 136 ignores debugging flags on the default data directory ([Chrome](https://developer.chrome.com/blog/remote-debugging-port)).
- **Local servers reachable from pages.** Local agent servers were reachable from web pages through DNS rebinding (Playwright MCP, CVE-2025-9611; MCP Inspector, CVE-2025-49596; research note `new_surfaces_privacy.md` 4.7, from the DevTools research).
- **Recordings and debug artifacts.** HAR files with session tokens let attackers hijack sessions at 5 of Okta's customers ([BleepingComputer](https://www.bleepingcomputer.com/news/security/okta-breach-134-customers-exposed-in-october-support-system-hack/)).
- **The DRM host is another browser.** A WebView2 host app can export the cookies of the pages it hosts ([BleepingComputer](https://www.bleepingcomputer.com/news/security/clever-phishing-method-bypasses-mfa-using-microsoft-webview2-apps/)), and WKWebView is patched only with the OS ([Hive Pro](https://hivepro.com/threat-advisory/apple-webkit-zero-days-exploited-in-the-wild/)).

### 3.5 What Izcal's specs do not yet cover

| Finding | Existing ID | Covered today? | Where this spec decides |
|---|---|---|---|
| V8 patch lag through the fork | D2 policy ("one Chrome stable cycle"), S-4e (ship blocker with a backport fallback), Stage 1 7.7 (lag metric "Days behind Chrome stable V8 security fixes", escalation at more than two weeks) | **Partly.** A lag metric and escalation exist. There is no latency target by severity, no per-advisory publication, no rule that the pinned V8 stays on a branch Chrome still patches, and no failure action when the fix itself is late | SEC-1, SEC-2, SEC-D1, SEC-D11, SEC-D12 |
| V8 archives not verified | Stage 1 7.1 (mirror); CONTRIBUTING ("commit their `SHA256SUMS`") | **No.** `RUSTY_V8_ARCHIVE_SHA256` is never set; the binding file is never checked | SEC-13 |
| IPC handle and identity logic | I-2, S-2, VER-1 | **No.** S-2 tests the renderer's syscalls; VER-1 proves parsers, not handle policy | IPC-R1 to IPC-R17; SEC-3 to SEC-7 |
| Consent drawn by the shell renderer | D-S2, UI-5, UI-10 | **No.** The shell draws and clicks every prompt; nothing checks that a person did | CON-R1 to CON-R5; SEC-29 |
| GPU service | S-7 (MTE where present), blocklist | **No target** for sandbox, ownership checks, fuzzing or the display-list format | SEC-10, IPC-R16 |
| C codecs and PDF in Stage 1 | Stage 1 Section 1 and 13.2 (utility processes, "libvpx in the library sandbox"), Stage 1 Section 6 (PDF in a utility process); S-5 (Stage 2 target) | **Design yes, target no.** The library sandbox is designed for Stage 1; the target and gate are Stage 2 | SEC-12, SEC-D3 |
| WebRTC and the native PlayReady path | Stage 1 M7, 13.1a item 2, 13.2 (OpenH264) | **No** process placement, model or target | Section 6.10a, SEC-30, SEC-33 |
| Rust↔V8 boundary | 4.8, S-4c, S-4e | **Partly.** No rule for `External` lifetimes or ArrayBuffers held across async work | FFI-R1 to FFI-R4, SEC-11 |
| macOS sandbox API | HGBA-2 Section 3 ("Mach ports under Seatbelt") | **No.** SDK 27 risk not recorded | SEC-8, SEC-D6 |
| Windows JIT versus ACG and CET | S-4a (Hardened: jitless, CET/BTI) | **Partly.** ACG not named; "every platform" assumes hardware Izcal cannot require | SEC-8, SEC-D5 |
| Linux kernel surface | S-2 (io_uring, ptrace, `pkey_mprotect` in renderers) | **Partly.** Only renderers; no architecture check; no reduced-mode reporting | SEC-9 |
| Web content reaching the local network | Stage 1 Section 1 (network service enforces LNA) | **Partly.** Enforced, but no boundary, attacker mapping or target | TB-13, SEC-31 |
| Downloads, external protocols, File System Access | Stage 1 13.2 (downloads with file-type and reputation checks; File System Access permission) | **Partly.** No tagging, launch or handle rules; no reputation source | SH-R6 to SH-R9, SEC-32, SEC-D7 |
| Agent entry points, memory, schemes | B9, AG-3 | **No.** B9 covers web origins only | AGT-R1 to AGT-R8, SEC-18, SEC-19 |
| Automation launched by local malware | DT-11, DT-13 | **Partly.** Launch flags not covered | SEC-20 |
| Imported and saved recordings | DevTools 6.5 rules 8 and 10 | **Partly.** No opaque-origin replay rule for imports, no retention limit, user-scoped key | REC-R1 to REC-R5, SEC-21 |
| Secrets at rest (cookies, recordings) | 13.2 (passwords in the OS keychain) | **No** rule for cookies at rest | SEC-34 |
| DRM host bridges and patch level | I-2 note, D-S7 | **Partly.** No no-bridge rule, no restart on runtime update, no minimum version | SH-R4, SEC-22 |
| Importer versus App-Bound Encryption | 13.2 everyday features | **No** | SH-R2, SEC-23 |
| Phishing and malware URL protection | B10 classifier carve-out; shell UI 4.5 and UI-9 | **No** protection is specified, although the shell designs its interstitial | SEC-D7 |
| Release governance, signing, provenance, updater privilege | 7.8 ("signed updates"), 13.2 (TUF, packaging) | **Partly.** Single-account control, attestation, reproducibility, key custody, the emergency path and the updater's process are unstated | SEC-13 to SEC-17, SEC-27, SEC-D4, SEC-D14 |
| AI-scale bug discovery | None | **No** | SEC-24, SEC-25 |
| Per-process threat models | 13.2 (before M6) | **Required, not written** | Section 6.5, SEC-26 |

---

## 4. First principles

### 4.1 Users' real security tasks

Users do not configure security; they expect a few outcomes to hold whatever page they open:
1. A page cannot read another site's data or the user's files.
2. A page cannot install anything or survive a restart.
3. A known exploit is fixed before it reaches them.
4. The browser they installed is the browser the project built.
5. Nothing acts in their name without their say: not a page, an extension, an agent, a debugging client or local malware posing as one.
6. What they did is not linkable across sites, and what they typed is not stored or sent unless they chose it.

Each outcome maps to one or more targets in Section 8.

### 4.2 Platform primitives

- **The process is the only confidentiality boundary.** Spectre defeats in-process checks ([McIlroy et al.](https://arxiv.org/abs/1902.05178)). In-process layers (V8 cage, RLBox, MTE) give integrity, not secrecy (I-3 to I-6).
- **OS sandboxes reduce kernel reach; they do not define policy.** "System call filtering isn't a sandbox" ([Linux seccomp](https://docs.kernel.org/userspace-api/seccomp_filter.html)). Windows has "no practical way to prevent code in the sandbox from calling a system service" ([Chromium sandbox design](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/design/sandbox.md)).
- **Capabilities attenuate only if something enforces it.** On Zircon a duplicated handle's rights must be "strictly lesser than" the source's, or equal ([Fuchsia `zx_handle_duplicate`](https://fuchsia.dev/reference/syscalls/handle_duplicate)); no desktop OS gives Izcal that for its own handles, so `hg-ipc` must.
- **Update integrity needs signed freshness, not only signed files** ([TUF specification](https://theupdateframework.github.io/specification/latest/)).
- **Signatures prove who built a binary, not what was built.** Only reproducible rebuilds or an isolated builder address a compromised build machine ([CrowdStrike on SUNSPOT](https://www.crowdstrike.com/blog/sunspot-malware-technical-analysis/)).

### 4.3 Izcal's principles that bind this design

P1 ranks security first. M-1 and D-S10 forbid copying another browser's code; designs are derived here, and other browsers are cited only as evidence. I-1 keys isolation by (site, container). I-2 gives renderers zero ambient authority. B2 makes endpoints capabilities. B9 makes agents a principal. B10 makes AI off mean not loaded. The DevTools spec makes every external session a scoped principal. Build-versus-reuse is not an open question: the standing answer is "build our own, following our specs", with unmodified dependencies allowed.

### 4.4 Assumptions kept and broken

| Assumption | Kept or broken | Reason |
|---|---|---|
| Renderer is untrusted; broker is trusted | **Kept**, sharpened: the broker is trusted to hold authority, never to believe a sender | CVE-2025-2783 crossed the broker; the other 2025 escapes went through OS and GPU-driver code that the broker grants reach to (Section 3.2) |
| Only renderers are compromised | **Broken** | Every sandboxed process parses hostile input; no process trusts another (IPC-R17, Section 6.6) |
| The browser's own UI decides consent | **Broken** for Izcal's shell | The shell is a cross-site renderer; the broker decides that a user approved (CON-R1 to CON-R5) |
| Memory-safe code needs fewer controls | **Broken** for boundaries | Logic bugs remain; IPC-R rules aim at them |
| A sandbox profile is a separate per-OS file | **Broken** where the OS allows | Dual-coding drifts ([Capsicum](https://www.usenix.org/legacy/event/sec10/tech/full_papers/Watson.pdf)); policy is generated with the process type |
| JIT is always on | **Broken** | JIT is a per-container capability (Hardened profile, SEC-D5) |
| One sandbox tier everywhere | **Broken** | Linux namespaces and macOS SBPL may be unavailable; the achieved tier is reported, never silently lowered |
| The model can be trusted to refuse | **Broken** | Engine enforcement first (AG-3, DT-13); model defenses are a second layer |
| Signing equals integrity | **Broken** | Hash pins, attestations, reproducible V8 archives, TUF |
| A single maintainer can be secured by process | **Kept with limits** | Hardware keys and server-side rules reduce the risk; only a second person removes it (SEC-D4). Until then a key threshold held by one person protects against key loss, not against compromise or coercion of that person |
| Deterministic simulation is a correctness tool | **Extended** | No published case uses DST with an adversarial peer; Izcal tries it and measures it (SEC-5) |

---

## 5. Lessons learned

Ideas and evidence only; no code and no template (M-1).

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Chrome 2008 | Deferred cross-site isolation; ran plugins unsandboxed "for compatibility" | Exceptions to the model become the next decade's work | No process runs outside its sandbox for compatibility; reduced modes are visible (SEC-8) |
| IE7 Protected Mode | Blocked writes, not reads | Define a sandbox by what the attacker cannot read | Renderers get no read access to user data (I-2, S-2) |
| Firefox e10s | Shared-memory add-on API froze the process model for years | Extension and agent APIs must be message-based | Agent host and W-1 components talk only over IPC |
| Chrome CVE-2025-2783; Firefox CVE-2025-2857 | Broker turned a relayed pseudo-handle into a real one | Handle transfer is the most dangerous IPC operation | IPC-R1: typed, broker-minted handles; transport rejects the rest (SEC-4) |
| Firefox CVE-2019-11708, CVE-2022-1529 | Parent trusted renderer-sent URL and origin | Identity must not come from the sender | IPC-R2: identity from a bound, non-transferable endpoint per frame and worker (SEC-3) |
| FORCEDENTRY over NSXPC | Subclass-accepting decoding reached the class library | The reachable surface must be "minimal, enumerable and reviewable" ([Project Zero](https://projectzero.google/2022/03/forcedentry-sandbox-escape.html)) | IPC-R3 closed types; IPC-R11 surface inventory (SEC-6) |
| Chrome FileWriter UAF | Renderer controlled a broker object's lifetime | Lifetime is part of the protocol ([Project Zero](https://projectzero.google/2019/04/virtually-unlimited-memory-escaping.html)) | IPC-R8; endpoint-closure faults in simulation (SEC-5) |
| Mozilla Nyx; MojoLPM | IPC fuzzing needed deterministic resets and call sequences ([Mozilla](https://blog.mozilla.org/attack-and-defense/2024/06/24/ipc-fuzzing-with-snapshots/); [MojoLPM](https://chromium.googlesource.com/chromium/src/+/HEAD/mojo/docs/mojolpm.md)) | Izcal's simulator is that platform | Compromised-renderer sequence fuzzer (SEC-5) |
| Edge 2017 ACG; Chrome CET | Code-integrity mitigations conflict with an in-process JIT | JIT placement is a sandbox decision | Jitless tier with ACG and CET on Windows (SEC-D5) |
| Firefox Win32k lockdown | Four-year retrofit | Design renderers without OS UI from the start | Renderers link no OS UI or font libraries (Section 6.7) |
| kCTF io_uring | 60% of kernel exploits through one subsystem ([Google](https://security.googleblog.com/2023/06/learnings-from-kctf-vrps-42-linux.html)) | Deny new kernel surface by default | Syscall allowlists for every sandboxed process (SEC-9) |
| Apple SBPL status | Vendor calls the sandbox language "not API" | Sandbox backends must be replaceable | `hg-sandbox` backend trait; SDK 27 measured (SEC-D6) |
| Deno CVE-2024-27934, CVE-2023-28445 | Rust↔V8 lifetime and buffer bugs | Rust safety ends at the V8 boundary | Typed `External` wrapper; buffer-pinning rule (SEC-11) |
| Firefox WebGPU | 2 sandbox escapes in one release ([MFSA 2025-87](https://www.mozilla.org/en-US/security/advisories/mfsa2025-87/); the advisory does not say whether wgpu or C++ glue was at fault) | Rust in the GPU service covers only its top layer | wgpu-core validates; Izcal adds ownership checks, clamps, fuzzing and the sandbox (SEC-10) |
| libwebp, libvpx | One bug each hit four platforms ([TAG 2023 review](https://storage.googleapis.com/gweb-uniblog-publish-prod/documents/Year_in_Review_of_ZeroDays.pdf)) | Third-party C parsers are the fastest route to many targets | Stage 1's library sandbox for codecs and PDF becomes a Stage 1 target; Rust-first images and fonts (SEC-12) |
| WebRTC CVE-2022-2294, CVE-2023-7024 | Two in-the-wild Chrome 0-days in the WebRTC stack (P0 sheet tally) | Peer-controlled protocol parsing is a renderer-class surface | WebRTC placement, socket grants and fuzzing (SEC-30) |
| Stylo | Rust rewrite reintroduced a fixed history leak ([Mozilla](https://hacks.mozilla.org/2019/02/rewriting-a-browser-component-in-rust/)) | Rewrites need regression tests for historical bug classes | Security regression suite that is never disabled (SEC-24) |
| Mozilla 2024 | Fixed chains in under a day | Fast response is an engineered capability | Patch pipeline with rehearsals (SEC-1, SEC-2) |
| Firefox 150 | 271 AI-found fixes in one release | A young engine gets no protection from obscurity | Pre-release AI-assisted audits; triage staffing (SEC-24, SEC-25) |
| xz, SUNSPOT | Release differed from source; builder altered the build | Reproducibility is the only check against a compromised builder | Two-builder V8 archive (SEC-15) |
| tj-actions, Ultralytics, Clinejection | Moved tags; poisoned caches; an AI triager with a shell | Each door is small and closable | SHA pins, no release caches, no CI agent with shell on untrusted input (SEC-13, SEC-27) |
| Notepad++, Zen CVE-2026-41431 | Unsigned metadata; verification removed in a fork | Updater code is security-critical | TUF client with attack vectors in CI (SEC-17) |
| Storm-0558 | Key rotation was switched off ([CSRB report](https://www.cisa.gov/sites/default/files/2025-03/CSRBReviewOfTheSummer2023MEOIntrusion508.pdf)) | Unrehearsed rotation stops happening | Rotation drills (SEC-16) |
| Comet, Atlas, PleaseFix | Agents took tasks from URLs, memory and local files | Agent entry points are injection channels | SEC-18 |
| Chrome agent origin sets | Only the writable set was enforced at rollout | Origin sets bound reach, not intent | Rule-of-Two task policy, counting third-party text inside writable origins as untrusted (SEC-19) |
| Okta HAR | Debug artifacts carried tokens | Recordings will be shared and stolen | DT-10 plus SEC-21 |
| WebView2 cookie export | The host can read every cookie | The DRM host is a second engine with its own authority | SEC-22 |
| Password-manager clickjacking | 10 of 11 extensions leaked through DOM-injected UI ([The Hacker News](https://thehackernews.com/2025/08/dom-based-extension-clickjacking.html)) | Credential UI must be out of the page's reach | Out-of-DOM autofill; the shell never holds the secret (SEC-23, CON-R4) |

---

## 6. Design

### 6.1 Attackers

Attacker classes carry the prefix `ATK-`, trust boundaries `TB-` and IPC rules `IPC-R`. Prose rules engineers must follow carry `CON-R` (consent), `FFI-R` (Rust↔V8), `GPU-R`, `NET-R`, `AGT-R` (agents and automation), `REC-R` (recordings), `SH-R` (shell features) and `UPD-R` (updates). None collides with an existing prefix in `reports/`.

| ID | Attacker | Capability assumed | Examples from the record | First boundary it meets |
|---|---|---|---|---|
| **ATK-1** | **Malicious web content**: any page, ad, iframe, email or calendar text | Arbitrary HTML, CSS, JS, Wasm, WebGL and WebGPU calls, media, fonts and PDFs; timing measurement; navigation and redirects; user-gesture lures; requests to local-network and loopback addresses | Every renderer 0-day; Leaky.page Spectre leak at 1 kB/s ([Google](https://security.googleblog.com/2021/03/a-spectre-proof-of-concept-for-spectre.html)); WebGPU cache attacks ([Giner et al.](https://www.gruss.cc/files/WebGPUAttacks.pdf)); DNS rebinding of local agent servers (Section 3.4) | TB-1, TB-13 |
| **ATK-2** | **Compromised sandboxed process**: code execution in **any** sandboxed Izcal process after a bug reached through ATK-1, ATK-3 or ATK-5 (a renderer, the shell renderer, a DevTools pane, the GPU, network, storage or utility service, the replay host, the gateway, the agent host) | Arbitrary code in that process; sends any byte string on any endpoint it holds; chooses message order, timing and endpoint closure; reads its own memory, including the V8 cage; in the shell renderer, draws any UI and "clicks" anything that has no broker-observed input behind it | Chrome FileWriter UAF; CVE-2025-2783; CVE-2022-1529; a page title that triggers a shell-renderer bug (Section 6.6a) | TB-2, TB-3 |
| **ATK-3** | **Hostile input to a privileged parser or driver**: crafted media, font, PDF, shader, GPU command sequence or WebRTC packet | Reaches C/C++ libraries, ANGLE, shader compilers and drivers from content | libwebp CVE-2023-4863; WebRTC CVE-2022-2294 and CVE-2023-7024; ANGLE-route CVE-2025-6558 and CVE-2025-14174 ([GTIG 2025](https://cloud.google.com/blog/topics/threat-intelligence/2025-zero-day-review/)) | TB-5, TB-6 |
| **ATK-4** | **Injected instructions and hostile tools**: text an agent or automation client reads, a poisoned MCP or WebMCP tool description, a malicious W-1 extension (Stage 2) | Steers a model that holds a session's grants; cannot run code in Izcal unless another bug exists | Comet, CometJacking, PleaseFix; MCPTox 36.5% average success ([arXiv 2508.14925](https://arxiv.org/abs/2508.14925)); Cyberhaven | TB-8 |
| **ATK-5** | **Network attacker**: hostile Wi-Fi, ISP, DNS, a misissuing CA, a hostile server or WebRTC peer | Reads and alters unprotected traffic; serves hostile protocol bytes; replays old update metadata | DigiNotar; rustls downgrade (RUSTSEC-2026-0349); Notepad++ update redirection | TB-7, TB-11 |
| **ATK-6** | **Supply-chain attacker** | Publishes a malicious crate version, takes over an upstream maintainer, moves an action tag, serves a tampered toolchain, LLVM package or V8 archive | arrayref; xz; tj-actions; Trivy | TB-11 |
| **ATK-7** | **Local attacker**: same-user malware, another local user, someone with the disk | Runs as the user; kills and relaunches Izcal with any flags; reads the profile directory; reads or injects into same-user process memory where the OS allows; connects to local sockets and pipes; synthesizes input where the OS allows | Infostealers using debug ports and App-Bound Encryption bypasses by injection ([Elastic](https://www.elastic.co/security-labs/threat-command/katz-and-mouse-game)); Lumma, over 394,000 machines in two months ([Microsoft](https://blogs.microsoft.com/on-the-issues/2025/05/21/microsoft-leads-global-action-against-favored-cybercrime-tool/)) | TB-10 |
| **ATK-8** | **Compromise of Izcal's own pipeline**: the maintainer's account, workstation, tokens, CI runners, signing or update keys | Pushes code, tags, workflows and releases as the project | Clinejection; GhostAction; s1ngularity using local AI CLIs ([Wiz](https://www.wiz.io/blog/s1ngularity-supply-chain-attack)) | TB-11 |
| **ATK-9** | **Tracker**: ad tech, data brokers, any site linking identities | Fingerprinting, cross-site navigation, UID smuggling, shared-resource timing, WebRTC address leaks | Google allowed advertiser fingerprinting from 16 February 2025 ([ICO](https://ico.org.uk/about-the-ico/media-centre/news-and-blogs/2024/12/our-response-to-google-s-policy-change-on-fingerprinting)); pool-party channels ([Snyder et al.](https://www.usenix.net/system/files/usenixsecurity23-snyder.pdf)) | TB-1, TB-7 |

**Against same-user code execution (ATK-7), Izcal raises cost and leaves evidence; it does not prevent theft.** Same-user malware can read or inject into a medium-integrity broker on Windows, and App-Bound Encryption was bypassed by injection within months (Elastic, above). Linux has no app-bound key store: Secret Service is user-scoped. Section 6.14 and SEC-34 state the residual per OS.

**Out of scope, stated so the residual risk is honest.** An attacker with administrator or root rights on the machine; an attacker with an unlocked device in hand; hardware implants; bugs in GPU drivers, OS media decoders, OS DRM components and the OS kernel themselves (Izcal limits who can reach them, but cannot fix them); and the OS web engine inside the protected-media host beyond what Izcal exposes to it (I-2 note).

**Which attacker meets which boundary.** ● first boundary; ○ reached after another boundary falls.

| | TB-1 | TB-2 | TB-3 | TB-4 | TB-5 | TB-6 | TB-7 | TB-8 | TB-9 | TB-10 | TB-11 | TB-12 | TB-13 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ATK-1 | ● | ○ | ○ | ○ | ○ | ○ | | | ● | | | ○ | ● |
| ATK-2 | | ● | ● | ● | ○ | ○ | | | | | | ● | ○ |
| ATK-3 | | ○ | ○ | | ● | ● | | | | | | | |
| ATK-4 | | | | | | | | ● | | | | | |
| ATK-5 | | ○ | | | | ○ | ● | | | | ● | | |
| ATK-6 | | | | | | | | | | | ● | | |
| ATK-7 | | | | | | | | ● | ● | ● | | | |
| ATK-8 | | | | | | | | | | | ● | | |
| ATK-9 | ● | | | | | | ● | | | | | | |

### 6.2 Trust boundaries

| ID | Boundary | What crosses it | Who enforces | Main existing IDs |
|---|---|---|---|---|
| **TB-1** | Web content ↔ renderer | Page bytes, script, timing | Renderer (same-origin policy) for correctness; never relied on against ATK-2, except between same-site origins below the origin-keying RAM threshold (Section 6.5, renderer) | I-6, D-S1, PRIV-2 |
| **TB-2** | Sandboxed process ↔ broker, and sandboxed process ↔ sandboxed process (IPC) | Typed messages, endpoints, handles, shared memory; the IPC-R16 exception payloads | The receiver, always, from the endpoint binding the broker created (IPC-R17) | I-2, B2, VER-1, IPC-R1 to IPC-R17 |
| **TB-3** | Sandboxed process ↔ OS kernel | Syscalls, ioctls, Mach and COM services | OS sandbox per process type | S-2, S-6, SEC-8, SEC-9 |
| **TB-4** | V8 cage ↔ rest of the renderer (Rust↔V8 FFI) | Handles, `External` pointers, backing stores, `cppgc` objects | V8 sandbox; Izcal's FFI wrappers | I-3, S-4b, S-4c, 4.8, FFI-R1 to FFI-R4, SEC-11 |
| **TB-5** | GPU service ↔ ANGLE, wgpu-core, shader compilers, drivers | Validated commands, translated shaders | wgpu-core (WebGPU), Izcal's WebGL validation, ownership checks; OS sandbox | S-7, GPU-R1 to GPU-R8, SEC-10 |
| **TB-6** | Utility ↔ C/C++ libraries and vendor binaries | Encoded media, fonts, PDF streams, OpenH264 | Process sandbox, RLBox-style wasm2c | I-3, S-5, SEC-12 |
| **TB-7** | Network ↔ network service | Wire bytes, certificates, DNS, WebRTC transport | rustls, CT, revocation, partitioning | I-1(b), 4.7, 13.2 TLS and certificate rows, NET-R1 to NET-R6 |
| **TB-8** | External principals ↔ broker: agents, automation, DevTools clients | Commands, observations, grants | `hg-inspect` sessions; B9 origin sets | B9, B10, AG-3, DevTools 6.5, DT-11, DT-13, AGT-R1 to AGT-R8 |
| **TB-9** | Izcal ↔ protected-media host and the OS DRM path | A page handed to the OS engine; EME messages to the native CDM; storage per (site, container) | Broker-mediated switch; generated EME decoders | I-2 note, D-S7, SH-R4, SEC-22, SEC-33 |
| **TB-10** | Izcal ↔ the local user environment | Profile files, keychain, command line, environment, local IPC, other browsers' profiles, downloaded files, external protocol handlers | OS file permissions, OS-bound keys, consent prompts, download tagging | D-S4, DevTools 6.5 rules 3 and 10, SEC-20, SEC-21, SEC-23, SEC-32, SEC-34 |
| **TB-11** | Source, dependencies, build, release, update | Commits, crates, actions, toolchains, V8 archives, signed packages, TUF metadata, data channels | Hooks, CI, cargo-deny and cargo-vet, attestations, TUF | 7.1, 7.7, 7.8, 13.2, UPD-R1 to UPD-R5, SEC-13 to SEC-17, SEC-27 |
| **TB-12** | Shell renderer ↔ broker, and OS input ↔ consent | Shell endpoints (tabs, settings, prompt requests); OS input events | Broker: shell endpoints bound only to the shell origin; **the broker decides consent from OS input it hit-tests itself** (CON-R1) | D-S2, UI-5, UI-10, IPC-R12, CON-R1 to CON-R5 |
| **TB-13** | Web content ↔ local network and loopback | Requests to private and loopback addresses; DNS answers that rebind to them | Network service, from the endpoint's context (IPC-R2), with Local Network Access permission | Stage 1 Section 1 (LNA), DevTools 6.5 rule 4, NET-R7, SEC-31 |

### 6.3 Assets and the processes that hold them

Stage 1 design Section 1, the four DevTools process kinds (DevTools spec 6.1), the crash handler (Stage 1 Section 6) and two kinds this spec adds (the prompt renderer, Section 6.6a, and the updater, Section 6.16): **seventeen process kinds**. Two components that span processes, WebRTC and the native DRM path, get their own rows. "Compromise yields" assumes ATK-2-class code execution inside that process and no further bug.

| Process | Trust | Assets it holds | Compromise yields |
|---|---|---|---|
| **Browser (broker)** | Trusted, unsandboxed | Every capability grant, the site-to-process map, the permission store, profile and policy, session grants, consent state and the record of which prompts a person approved, the agent principal's origin sets, the top-level native windows and OS input routing | Everything the user can do. The design goal is that no web-format parsing, no TUF metadata parsing and no interpreter run here (IPC-R10, IPC-R15, UPD-R2) |
| **Renderer** (per site, container) | Untrusted | One (site, container)'s DOM, script state, cookies it was given, form input in progress | That site's data in that container only, if I-1 holds; below the origin-keying threshold, every origin of that site |
| **Shell renderer** | Untrusted; cross-site by design | Tab list and titles of all sites, navigation, settings, import and download UI; usernames for autofill pickers, never passwords (CON-R4) | UI spoofing and any action its endpoints allow; **not** consent, because the broker approves only on broker-observed input (CON-R1). Without CON-R1 it would equal approving every pending prompt |
| **Prompt renderer** | Untrusted, project code only, jitless | The text of one security prompt at a time, formatted by the broker | A spoofed prompt rectangle; still no approval without broker-observed input |
| **GPU service** | Sandboxed, privileged by reach | Every site's composited pixels, textures and WebGPU buffers; driver access | Cross-site pixel reads; a path into drivers and the kernel; a foothold to attack other services (IPC-R17) |
| **Network service** | Sandboxed (Stage 1 tier per SEC-D13) | All cookies in flight, HTTP cache, connection pools, TLS session state, HSTS, DNS, content-blocking engine, WebRTC transport and UDP sockets | Every site's sessions and traffic |
| **Storage service** | Sandboxed | All sites' IndexedDB, Cache API, localStorage and cookies at rest | Every site's stored data; partition checks must live here, not only in the broker |
| **Utility** (pooled) | Sandboxed | The inputs of the current decode job, including WebRTC media | That job's inputs and outputs; must not mix sites if output could leak (I-1) |
| **Protected-media host** | Outside Izcal's trust boundary | One (site, container)'s DRM-site session in the OS engine | That session; and if the Izcal side of the host is compromised, every protected-media container's cookies (WebView2's cookie manager) |
| **ML service** (AI or accessibility model on) | Sandboxed | Model sessions, inputs sent for inference | Inference inputs; a path to NPU and GPU drivers |
| **Agent host** (AI on) | Sandboxed | Task state, model context, tool registry view | What the task's origin sets allow; nothing more if B9 enforcement is in the broker |
| **DevTools workspace renderer** | Untrusted, project code only | Structural metadata only | Frame IDs, sites and timings of inspected pages |
| **DevTools pane renderer** (per site, container) | Untrusted, project code only | One inspected (site, container)'s DOM, console, sources, network detail | The same as compromising that site's renderer (DevTools spec 6.1) |
| **Replay host** (per site, container) | Sandboxed like a renderer, no network | One recorded process's recording | Everything that recording holds, including values never on the wire (Section 6.13) |
| **Automation gateway** | Sandboxed | One external session's endpoints and its accepted connection handle | That session's grant |
| **Updater** | Unsandboxed by necessity, no network, started on demand | Write access to the install directory; the pinned TUF root and verification state | Code execution at the install's privilege on the next launch; per-user under SEC-D14 |
| **Crash handler** | Separate process | Minidumps of crashed processes | Stack memory of crashed processes, which can hold page strings (D-S4) |
| *WebRTC* (spans network service and utility) | Component | Peer connections, ICE candidates and local addresses, decoded media | The hosting process's assets; local-address exposure to peers (ATK-9) |
| *Native DRM path* (Windows PlayReady through Media Foundation) | Vendor CDM and OS protected media path | Encrypted media sessions, licence messages | Whatever the hosting process holds; the CDM and the OS path are outside Izcal's code |

### 6.3a Per-process index

One row per process kind: what applies to it, where its model lives and who owns it. "Track" is the Stage 1 design 8.3 track (or the Security track, SEC-D2). Every process also falls under IPC-R1 to IPC-R9, IPC-R11, IPC-R13, IPC-R14, IPC-R17, SEC-3 to SEC-7, SEC-8, SEC-9 and SEC-26; the table lists what is specific.

| Process | ATK | TB | Specific rules | Sandbox (6.7 row) | Specific targets | Threat model file | Track |
|---|---|---|---|---|---|---|---|
| Broker | ATK-2 (all peers), ATK-4, ATK-5, ATK-7 | TB-2, TB-8, TB-10, TB-12 | IPC-R10, IPC-R12, IPC-R15, IPC-R16, CON-R1, CON-R3, AGT-R3, UPD-R2 | Broker | SEC-29, SEC-20, SEC-18 | `security/threat-models/broker.md` | Substrate and IPC |
| Renderer | ATK-1, ATK-2 | TB-1, TB-2, TB-3, TB-4, TB-13 | FFI-R1 to FFI-R4 | Renderer | SEC-1, SEC-11 | `renderer.md` | JS, bindings and DOM |
| Shell renderer | ATK-1, ATK-2 | TB-2, TB-12 | CON-R1 to CON-R4, IPC-R12 | Renderer (jitless) | SEC-29 | `shell-renderer.md` | Shell |
| Prompt renderer | ATK-2 | TB-12 | CON-R2, CON-R5 | Renderer (jitless) | SEC-29 | `prompt-renderer.md` | Shell |
| GPU service | ATK-2, ATK-3 | TB-2, TB-3, TB-5 | GPU-R1 to GPU-R8, IPC-R16 | GPU | SEC-10 | `gpu-service.md` | Graphics |
| Network service | ATK-2, ATK-5 | TB-2, TB-3, TB-7, TB-13 | NET-R1 to NET-R7 | Network | SEC-28, SEC-30, SEC-31 | `network-service.md` | Network and security |
| Storage service | ATK-2 | TB-2, TB-3 | Storage mitigations (6.5) | Storage | SEC-34 | `storage-service.md` | Network and security |
| Utility | ATK-3 | TB-2, TB-6 | Section 6.10 rules | Utility | SEC-12, SEC-30 | `utility.md` | JS, bindings and DOM (media) |
| Protected-media host | ATK-1, ATK-7 | TB-9 | SH-R4 | Host engine's own | SEC-22 | `protected-media-host.md` | Shell |
| ML service | ATK-2, ATK-3 | TB-2, TB-3 | B10, G4 | ML | — | `ml-service.md` | Accessibility |
| Agent host | ATK-4 | TB-8 | AGT-R1 to AGT-R6 | Utility | SEC-18, SEC-19 | `agent-host.md` | Security (Stage 2: agent owner) |
| DevTools workspace | ATK-2 | TB-2 | DevTools 6.1 | Renderer (jitless) | — | `devtools-workspace.md` | DevTools |
| DevTools pane | ATK-1, ATK-2 | TB-2 | DevTools 6.5 rule 7 | Renderer (jitless) | — | `devtools-pane.md` | DevTools |
| Replay host | ATK-1, ATK-7 | TB-2, TB-10 | REC-R1, REC-R6 | Renderer (recorded tier) | SEC-21 | `replay-host.md` | DevTools |
| Automation gateway | ATK-4, ATK-7, ATK-1 | TB-8, TB-13 | AGT-R7, AGT-R8, IPC-R16 | Utility (accepted-connection handle) | SEC-20, SEC-31 | `automation-gateway.md` | DevTools |
| Updater | ATK-5, ATK-8 | TB-11 | UPD-R1 to UPD-R5 | Updater | SEC-16, SEC-17 | `updater.md` | Infrastructure |
| Crash handler | ATK-2, ATK-7 | TB-10 | SH-R5 | Broker | SEC-35 | `crash-handler.md` | Infrastructure |
| *WebRTC* | ATK-3, ATK-5, ATK-9 | TB-6, TB-7 | NET-R1, Section 6.10a | Network, Utility | SEC-30 | `webrtc.md` | Network and security |
| *Native DRM path* | ATK-3 | TB-9 | Section 6.10a | Dedicated CDM process | SEC-33 | `native-drm.md` | Shell |

### 6.4 One map of the existing security IDs

IDs are **not renumbered**. This table says where each one sits in the threat model and where this spec proposes an amendment (amendments are listed in Section 9.4 and are not applied by this document).

| ID | Defined in | Attacker and boundary | What it does | Amended here |
|---|---|---|---|---|
| P1 | HGBA-2 §2 | All | Security first in every trade-off | — |
| M-1, D-S10 | HGBA-2 §2; Stage 1 13.1 | ATK-6 (provenance), method | No other browser's code; specs are the source; fork changes written against V8's public interfaces | Applied to SBPL profiles, importers, PDF, WebRTC, crash handler and GPU blocklist (Section 7) |
| I-1, I-1 extensions | HGBA-2 §4.1 | ATK-2, ATK-9 / TB-2, TB-7 | Process per (site, container); network-state partitioning | Adds DevTools panes and replay hosts to the I-1 checks; connection-pool and DNS partitioning stays as specified |
| I-2, I-2 note | HGBA-2 §4.1 | ATK-2 / TB-2, TB-3; TB-9 | Zero ambient authority; protected-media host outside the boundary | SH-R4 and SEC-22 rules for the host |
| I-3 | HGBA-2 §4.1 | ATK-2, ATK-3 / TB-4, TB-6 | V8 cage, wasm2c, MTE as layers | wasm2c and Wasmtime as code generators in the trusted base (SEC-12) |
| I-4, I-5 | HGBA-2 §4.1 | ATK-2 | PKU and MTE limits | — |
| I-6 | HGBA-2 §4.1 | ATK-1 (Spectre) / TB-1 | COOP and COEP gate shared memory and timers | Extended to WebGPU timers (GPU-R7) |
| 4.8 | HGBA-2 | ATK-2 / TB-4, TB-6 | `unsafe` inventory, tainted FFI, Rule of 2, FFI fuzzing | FFI-R1 to FFI-R4 (SEC-11) |
| S-1, S-3 | HGBA-2 §6 | ATK-2 | Site isolation and its memory cost | — |
| S-2 | HGBA-2 §6 | ATK-2 / TB-3 | Renderer syscall and open tests | Extended to all sandboxed processes with an architecture check (SEC-9) |
| S-4a, D2 Hardened model | HGBA-2 §6, §7 | ATK-2 / TB-4 | Jitless profile | Windows tier adds ACG; "every platform" becomes "where the hardware supports it" (SEC-D5); Izcal-only renderers jitless by default (Section 6.7) |
| S-4b | HGBA-2 §6 | ATK-2 / TB-4 | V8 sandbox on | SEC-1 keeps it patched |
| S-4c | HGBA-2 §6 | ATK-2 / TB-4 | GC and threading discipline | Referenced by FFI-R3 |
| S-4e, D2 ownership | HGBA-2 §6, §7 | ATK-1 via known V8 bugs | Ship-blocker rebase target with a backport fallback; one-cycle window | SEC-1 adds severity-tiered latency targets and the Chrome-patched-branch rule (SEC-D12) |
| S-5 | HGBA-2 §6 (Stage 2) | ATK-3 / TB-6 | Library sandboxing target | Stage 1's existing library-sandbox design becomes a Stage 1 target (SEC-12, SEC-D3) |
| S-6 | HGBA-2 §6 (Stage 2) | ATK-2, ATK-5 / TB-3 | Network service sandboxed | Required in Stage 1 on Linux and macOS (SEC-D13) |
| S-7 | HGBA-2 §6 (Stage 2) | ATK-2 | MTE, PKU, CET, BTI where present | — |
| S-8 | HGBA-2 §6 (Stage 2) | ATK-2 | Android reduced isolation | — |
| B1, DET-1 | HGBA-3 | Tool | Deterministic substrate | Used as an adversarial IPC fuzzer (SEC-5) |
| B2 | HGBA-3 | ATK-2 / TB-2 | Typed capability IPC | IPC-R1 to IPC-R17 |
| VER-1 | HGBA-3 | ATK-2 / TB-2 | Proven or Kani-checked (de)serializers | Names the properties; adds a handle-policy harness (SEC-4, SEC-7) |
| DST-1 | HGBA-3 | Tool | DST must earn its CPU | A security variant measured on IPC endpoints with a planted-bug benchmark (SEC-5) |
| B8, B9, AG-3, AG-5 | HGBA-3 | ATK-4 / TB-8 | Provenance, agent principal, origin sets, visibility flags | AGT-R1 to AGT-R6 (SEC-18, SEC-19) |
| B10, AG-0 | HGBA-3 | ATK-4, ATK-7 | AI off means not loaded | Phishing classifier route (SEC-D7) |
| PRIV-1, PRIV-2, D-S1 | HGBA-3; Stage 1 §11 | ATK-9 | Fingerprinting defenses and their cost | Adaptive de-noising and GPU timing (Section 6.15) |
| G4, G5, G6, G7 | Stage 1 §2.2 | ATK-2, ATK-4 | AI crates not linked; `unsafe` inventory; V8 surface bounded; renderer reaches services only through IPC | — |
| G3 | Stage 1 §2.2 | Method | Only `hg-substrate-os` names `std::process`, `std::fs` and `std::net` | `hg-sandbox` sits inside `hg-substrate-os`; `hg-update` reaches files and network only through it (Section 9.4) |
| G10–G12 (proposed) | DevTools spec 6.1, pending (DevTools 9.5) | ATK-1, ATK-7 | DevTools crate rules; G12 limits the replay host to `hg-substrate-sim` | Cited as proposed, not as applied |
| Stage 1 §4 | Stage 1 | ATK-2 / TB-2 | `.hgidl`, endpoints, sync allowlist, VER-1 wire format | IPC-R rules |
| Stage 1 7.5 | Stage 1 | ATK-1 to ATK-3 | Fuzzing plan | Adds the adversarial-peer, transport, GPU, Rust↔V8, WebRTC and network protocol campaigns |
| Stage 1 7.7, 7.8 | Stage 1 | ATK-1 (known bugs), ATK-6, ATK-8 | Upstream pins, V8 bot, lag metric with escalation at more than two weeks, signed updates | SEC-1, SEC-13 to SEC-17 |
| Stage 1 13.2 rows | Stage 1 | All | Threat models before M6; disclosure policy; TUF updates; signed packages | Section 6.5 is the template and first pass; SEC-25, SEC-17, SEC-16 |
| D-S2 | Stage 1 §11 | ATK-2 / TB-12 | Shell is sandboxed web content, CSS-only theming | IPC-R12; CON-R1 to CON-R5 |
| D-S4 | Stage 1 §11 | ATK-7, privacy | Crash reports opt-out, minidumps only | SH-R5, SEC-35 |
| D-S7 | Stage 1 13.1a | TB-9 | Protected media mode | SH-R4, SEC-22 |
| D-S11 | Stage 1 13.1 (18 October 2026 update) | ATK-4, ATK-7 / TB-8 | Own DevTools; IIP with adapters | SEC-20 |
| D-S12 | Stage 1 13.1 | Method | Compatibility oracles: black-box observations for `hg-cdp` and HAR | — |
| DevTools 6.5, DT-10, DT-11, DT-13 | DevTools spec | ATK-4, ATK-7 / TB-8, TB-10 | Scoped sessions, consent, no ports, redaction | SEC-20, SEC-21; the gateway receives accepted connections as handles (Section 6.7) |
| UI-5, UI-10 | Shell UI | ATK-1 / TB-12 | Prompts need a gesture; prompts cannot be spoofed | CON-R1 to CON-R5 make the broker the judge of the gesture |
| UI-9 | Shell UI | ATK-1 | Interstitials are heeded | Depends on SEC-D7 (no phishing source exists yet) |
| `security` and `secrets` checks, AI review | `CONTRIBUTING.md`, `tools/checks.sh` | ATK-6, ATK-8; developer error | `security`: `// SAFETY:` comments, risky patterns, zizmor, cargo-deny advisories. `secrets`: gitleaks. AI review with a security checklist | Stated as quality gates, not a boundary (Section 6.16) |

### 6.5 Per-process threat models

Stage 1 design 13.2 requires one threat model per process before M6. This section supplies the template and a first pass for all seventeen process kinds in Section 6.3 and the two cross-process components; the Stage 1 list names eight and omits the protected-media host, the ML service, the crash handler and the four DevTools kinds. **At M1 all nineteen files are created from the template under `security/threat-models/` (names in Section 6.3a); the broker, renderer and network-service models are filled at M1, and the rest by M6.** Each is owned by the track that owns the process, reviewed by the security owner, and updated in the same pull request as any change to that process's endpoint set or sandbox policy (SEC-26).

**Template.**

```
# Threat model: <process>
Owner: <track>      Reviewed: <date, security owner or "self-approved">      Surface inventory: <generated file, SEC-6>
1. Assets: what this process holds, per (site, container) where relevant.
2. Entry points: every endpoint it serves (from the surface inventory), every IPC-R16 exception payload,
   every file or device it opens, every parser it runs on input it did not create.
3. Attacker capabilities assumed: which ATK classes reach each entry point, and what they control.
   Every peer process is assumed compromised (IPC-R17).
4. Mitigations: IDs (IPC-R, CON-R, FFI-R, GPU-R, NET-R, AGT-R, REC-R, SH-R, UPD-R, SEC-, S-, I-) and how each is tested.
5. Residual risk: what remains if every mitigation works, stated plainly.
6. Open items: missing tests, unverified OS behavior, decisions pending.
```

**First pass.**

**Browser (broker).**
| Item | Content |
|---|---|
| Assets | All grants, permission store, site-to-process map, profile and policy, session grants and consent records, agent origin sets, native windows and OS input |
| Entry points | Every renderer-, shell-, prompt-, pane-, gateway-, agent- and service-facing endpoint; OS input events; command line and environment; OS integration (file associations, protocol handlers, default-browser APIs); verified update results from the updater |
| Attackers | ATK-2 from every peer; ATK-4 through gateways and the agent host; ATK-7 on the command line, environment and profile |
| Mitigations | IPC-R1 to IPC-R17 enforced by `hg-ipc-codegen` and the transports; VER-1 harnesses; adversarial simulation with every process kind as the peer (SEC-5); consent decided from broker-observed input (CON-R1); no web-format parsing, no TUF metadata parsing and no interpreter (IPC-R10, IPC-R15, UPD-R2); `panic = "abort"` with handler lints (IPC-R5); Hardened Runtime and Enhanced Security on macOS; CET and CFG on Windows; CET or IBT where available on Linux (Section 6.7); launch flags cannot reach a user container (AGT-R8) |
| Residual | Logic bugs in policy and handle relay (Section 3.2); OS bugs reachable as the user; it runs unsandboxed by necessity; same-user code can inject into it (ATK-7) |
| Open | Which Windows mitigations (CIG, extension-point blocking) the broker can carry while it owns the native window, third-party IMEs, assistive technology and file dialogs: measured at M5 against an IME and assistive-technology compatibility list; if incompatible, a separate unsandboxed, non-broker UI-host process owns the window (Q-SEC-6 is answered by the separate updater, Section 6.16) |

**Renderer (per site, container).**
| Item | Content |
|---|---|
| Assets | One (site, container)'s DOM, JS heap, session cookies it was handed, form data |
| Entry points | All web content; one initial broker endpoint and the endpoints the broker grants (one bound endpoint per frame and per worker, IPC-R2); V8 |
| Attackers | ATK-1 fully; after a bug, ATK-2 within the process |
| Mitigations | I-1, I-2, I-3, I-6; S-1, S-2, S-4a to S-4c; V8 sandbox and SEC-1 patch latency; FFI-R1 to FFI-R4 (SEC-11); per-OS sandbox (Section 6.7); no OS UI or font libraries linked |
| Residual | V8 bugs contained only by the cage and the process sandbox; kernel bugs reachable through allowed syscalls (Linux), system services (Windows has no syscall filter) and the SBPL allowlist (macOS); Spectre within the process; **below the origin-keying RAM threshold, subdomains of one site share a renderer, so the same-origin policy is relied on between them** (Stage 1 Section 1, citing Kilger et al.); the threshold decision is the mitigation |
| Open | macOS: a real Xcode 27 build of the SEC-D6 probe, and sandbox extensions on macOS 27 (SEC-D6); Linux reduced tier when user namespaces are unavailable; the Windows token combination (Section 6.7) measured at M5 |

**Shell renderer.**
| Item | Content |
|---|---|
| Assets | Tab list and titles of all sites, settings, import and download flows, usernames shown in autofill pickers |
| Entry points | The shell's own HTML and project JS; page titles, favicons and URLs from every site; notification text; user CSS themes |
| Attackers | ATK-1 through page-supplied strings; ATK-2 if the shell itself is exploited. **It is the one cross-site renderer, so one bug triggered by a page title puts ATK-2 next to the most powerful endpoints** |
| Mitigations | D-S2 (CSS-only theming); jitless, with the full Windows mitigation set (Section 6.7); Trusted Types and a strict CSP for shell code; page strings rendered as text; shell endpoints bound to the shell origin only (IPC-R12); **the shell cannot approve anything: security prompts are drawn by the prompt renderer, and the broker accepts an approval only for OS input it hit-tested itself (CON-R1, CON-R2)**; passwords never reach it (CON-R4); every privileged request is re-checked in the broker |
| Residual | A shell compromise can spoof non-security UI, reorder tabs and request anything its endpoints allow; the endpoint set is the blast radius |
| Open | Publish the shell's endpoint inventory and keep it the smallest the UI needs (SEC-6) |

**Prompt renderer.**
| Item | Content |
|---|---|
| Assets | One security prompt's text and layout at a time |
| Entry points | A broker endpoint that sends prompt content: the broker-formatted origin, the action, fixed project strings; nothing else derived from a page |
| Attackers | ATK-2 only through a bug in project code fed broker-formatted data; ATK-1 cannot reach it with free text |
| Mitigations | CON-R2 (project JS only, jitless, no page strings); CON-R5 (minimum visible time, no approval while occluded or moved); the broker composites the prompt above page content and owns its rectangle |
| Residual | A compromise can draw a misleading prompt, but approval still needs broker-observed input on the rectangle the broker chose |
| Open | Whether the native recovery-window path (D-S2) should draw the credential prompts instead (CON-R2 alternative (c)) |

**GPU service.**
| Item | Content |
|---|---|
| Assets | Every site's composited output, textures, WebGPU buffers; driver handles |
| Entry points | WebRender display lists from every renderer (an IPC-R16 exception); WebGL commands (ANGLE on Windows, native GL on macOS and Linux); WebGPU commands and WGSL shaders through wgpu-core and Naga; shared-texture handles |
| Attackers | ATK-3 through command sequences and shaders; ATK-2 from any renderer sending arbitrary display lists and commands |
| Mitigations | GPU-R1 to GPU-R8 (SEC-10): sandbox on all desktop platforms; wgpu-core as the WebGPU validator; Izcal's WebGL validation; per-renderer resource ownership (a renderer names only textures and buffers it created); size and limit clamps; display-list fuzzing with a size cap and copy-then-validate; call-sequence and shader fuzzing; Naga translation checks; I-6 timer rules; blocklist |
| Residual | Driver and shader-compiler bugs below validation (the shader compilation pipeline is "only weakly sandboxed if at all", [Bernhard, OffensiveCon 2025](https://citation.thinkst.com/talk/96285)); cross-site pixel exposure inside one process; ANGLE is C++ in a privileged process; a compromised GPU service attacks its peers (IPC-R17) |
| Open | Which Windows tier the GPU drivers tolerate (Low IL, LPAC, Q-SEC-9); WebGPU on by default per platform (Q-SEC-14); where WebGL lives and which translator macOS and Linux use (Q-SEC-15) |

**Network service.**
| Item | Content |
|---|---|
| Assets | Cookies in flight, HTTP cache, connection and socket pools, DNS, TLS state, HSTS, content-blocking lists, WebRTC transport |
| Entry points | Network bytes (HTTP/1.1, HTTP/2, HTTP/3 through neqo, TLS through rustls, DNS through hickory-dns, WebRTC transport protocols); fetch requests from renderers; storage requests it makes to the storage service |
| Attackers | ATK-5 on the wire; ATK-2 sending requests with forged context; ATK-1 targeting the local network (TB-13) |
| Mitigations | NET-R1 to NET-R7: request context from the endpoint (IPC-R2); ORB filtering before data reaches a renderer; partitioning by (top-level site, container) (I-1(b)); LNA and DNS-rebinding checks (SEC-31); CT enforcement and CRL-based revocation (13.2); protocol differential fuzzing and resource caps (SEC-28); `panic = "abort"` with no panic reachable from network input, and the broker restarts a crashed service (IPC-R5); sandbox tier per SEC-D13 |
| Residual | Protocol state-machine and resource-exhaustion bugs (Section 3.3); a compromise exposes every site's sessions |
| Open | Windows sandbox tier (AppContainer with `internetClient`, or LPAC) |

**Storage service.**
| Item | Content |
|---|---|
| Assets | All sites' stored data and cookies at rest |
| Entry points | Storage requests from renderers and the network service; SQLite |
| Attackers | ATK-2 from a renderer **or from the network service** asking for another partition's data |
| Mitigations | Partition key from the endpoint (IPC-R2), checked in this process on every query, not only in the broker, and the network service treated as hostile (IPC-R17); quota per (site, container); no network; file access limited to the profile directory; cookies at rest encrypted (SEC-34); **SQLite hardening**: bound parameters only (no SQL text from any peer), defensive mode on, no extension loading, FTS tokenizer pointers and similar features off; the IndexedDB key and value encodings fuzzed |
| Residual | SQLite bugs; a compromise exposes every partition |
| Open | — |

**Utility (pooled).**
| Item | Content |
|---|---|
| Assets | Inputs of the current job |
| Entry points | Encoded images, fonts, media containers and bitstreams (including WebRTC media), PDF documents |
| Attackers | ATK-3 |
| Mitigations | Strictest sandbox tier on every OS; one (site, container) per process at a time when outputs could leak across sites; C libraries and the OpenH264 binary in RLBox-style wasm2c or a single-purpose process (SEC-12); Rust decoders by default for images and fonts; outputs normalized before crossing (bitmaps, glyph outlines) |
| Residual | Library bugs give the job's inputs; OS decoders (H.264, AAC, HEVC) run inside OS frameworks Izcal cannot patch |
| Open | Where OS decoders run, and with what sandbox, per OS (Q-SEC-4) |

**Protected-media host.**
| Item | Content |
|---|---|
| Assets | A DRM site's session in one (site, container) user-data folder |
| Entry points | The page the switch hands over; the host's configuration from Izcal |
| Attackers | ATK-1 against the OS engine; ATK-7 against the user-data folder |
| Mitigations | SH-R4 (SEC-22): no bridge (zero host objects, web messages or injected scripts), its own process, one data folder per (site, container), host restarted when the OS runtime updates, refusal below a minimum runtime or OS WebKit version, label stating whose patch level applies; never for agent tabs (I-2 note) |
| Residual | The OS engine's own bugs on Microsoft's or Apple's schedule; the Izcal process that hosts WebView2 can read every hosted cookie, so it is never the broker |
| Open | Whether WebView2 user-data folders get any OS-bound encryption (unverified) |

**ML service.**
| Item | Content |
|---|---|
| Assets | Inference inputs (page text, images) and outputs |
| Entry points | Requests from the agent host, the accessibility model setting and task APIs; OS ML runtimes |
| Attackers | ATK-3 through crafted inputs; ATK-2 through requests |
| Mitigations | Launched only when its switch is on (B10, G4); inputs from one (site, container) per session; GPU and NPU access only as the OS runtime requires; Low IL on Windows (Section 6.7) |
| Residual | OS ML runtimes and NPU drivers are opaque (HGBA-2 4.5) |
| Open | Per-OS sandbox profile once the runtimes' needs are measured |

**Agent host.**
| Item | Content |
|---|---|
| Assets | Task context, model context, observations within the task's origin sets |
| Entry points | Observations from the broker (provenance-filtered), tool results, MCP and WebMCP tool descriptions, the user's task |
| Attackers | ATK-4 fully; the model is assumed compromised by injection |
| Mitigations | B9 enforcement lives in the broker, not here; AGT-R1 to AGT-R6 (SEC-18, SEC-19); per-action confirmations through CON-R1 and the prompt renderer; AI off means not launched (AG-0) |
| Residual | In-set misuse: an agent with write access to a mail origin can be told to forward mail through that origin; engine enforcement bounds reach, not intent (B9) |
| Open | Stage 2 design detail; AG-3 suite contents |

**DevTools workspace renderer.**
| Item | Content |
|---|---|
| Assets | Structural metadata (frame IDs, sites, containers, timings, status codes, sizes) |
| Entry points | IIP metadata messages |
| Attackers | ATK-2 after a bug triggered by metadata strings |
| Mitigations | No URLs beyond the site, no bodies, no write capability (DevTools 6.1); page strings as text under Trusted Types; jitless (Section 6.7) |
| Residual | Leak of which sites are open and their timings |
| Open | — |

**DevTools pane renderer.**
| Item | Content |
|---|---|
| Assets | One (site, container)'s DOM, console, sources and network detail as granted |
| Entry points | IIP messages for that site's frames; page strings shown in views; V8 inspector payloads (IPC-R16) |
| Attackers | ATK-1 through page strings, source text and console messages; ATK-2 after a bug |
| Mitigations | One pane per (site, container) (I-1); write rights only as granted; Trusted Types and strict CSP (DevTools 6.5 rule 7); jitless (Section 6.7) |
| Residual | Equal to compromising that site's renderer |
| Open | — |

**Replay host (per site, container).**
| Item | Content |
|---|---|
| Assets | One recorded process's recording, including values computed in memory |
| Entry points | A recording bundle, saved or imported; probes and queries from panes |
| Attackers | ATK-1 as the author of an imported bundle (an attacker-authored page plus an attacker-authored input log), **equal to opening a page**; ATK-7 against saved bundles |
| Mitigations | `hg-substrate-sim` only (G12, proposed in the DevTools spec); renderer-tier sandbox with exactly the recorded engine configuration, so a JIT recording replays with the JIT and a jitless replay exists only for jitless recordings (REC-R6); no network, storage service or GPU device; imported bundles replay under an opaque origin with no user container and no write capability, and the bundle parser is fuzzed (REC-R1, SEC-21); agents get query results, never bundles |
| Residual | A leaked bundle exposes every value the page computed, not only what was displayed (replay as an oracle) |
| Open | Checkpointing on Windows (DevTools Q-DT-3) |

**Automation gateway.**
| Item | Content |
|---|---|
| Assets | One external session's grant and endpoints |
| Entry points | CDP, BiDi, DAP and MCP messages from an external client over a pipe, OS-authenticated local IPC, or a connection accepted on a user-enabled loopback listener; V8 inspector payloads (IPC-R16) |
| Attackers | ATK-7 (local malware as a client); ATK-4 (an injected agent); ATK-1 through DNS rebinding if a listener is on |
| Mitigations | DevTools 6.5 (consent naming the container, origin sets, sensitive classes, no ports by default, `Origin` and `Host` checks, tokens); **the broker owns the listening socket and passes each accepted connection to the gateway as a handle, so the gateway keeps "no network"** (Section 6.7); decodes page data only through generated B2 decoders; consent through CON-R1; AGT-R7, AGT-R8 (SEC-20) |
| Residual | Same-user malware can still drive the UI where the OS allows synthetic input (ATK-7: cost and evidence, not prevention) |
| Open | Per-OS protection of the consent prompt against synthetic input (Q-SEC-7) |

**Updater.**
| Item | Content |
|---|---|
| Assets | Write access to the install directory; the pinned TUF root; verification state |
| Entry points | Update bytes and TUF metadata fetched by the network service and passed through the broker as opaque bytes; the apply request from the broker |
| Attackers | ATK-5 (hostile mirror or metadata); ATK-8 (stolen keys below threshold); ATK-2 from the network service or broker peers |
| Mitigations | UPD-R1 to UPD-R5: `hg-update` verification against the TUF attack classes (SEC-17) runs here, not in the broker; no network; verify-then-apply on a locked file; per-user install with no elevation (SEC-D14); delegated `releases` role (SEC-16) |
| Residual | A verification bug gives code execution at the install's privilege; per-machine installs would add an elevated component with the long local-privilege-escalation record of Chrome-class updaters (not researched, Q-SEC-18) |
| Open | macOS and Linux packaging interplay (notarization, distribution packages) |

**Crash handler.**
| Item | Content |
|---|---|
| Assets | Minidumps (thread stacks and registers) of crashed processes |
| Entry points | Crash signals and exception ports of every Izcal process; the upload channel |
| Attackers | ATK-2 crafting a crash to plant data; ATK-7 reading pending dumps; crash-server breach |
| Mitigations | D-S4 (minidumps only, viewable before sending); SH-R5 (SEC-35): filtered-stack dumps for renderer, shell, prompt, pane and replay-host processes ([`MiniDumpFilterMemory`-class](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/ne-minidumpapiset-minidump_type) behavior on every OS); recordings never attached (DevTools 6.5 rule 10); dumps deleted after 30 days† |
| Residual | Stack memory can still hold page strings |
| Open | Per-OS filtering method |

**WebRTC (component in the network service and utility).**
| Item | Content |
|---|---|
| Assets | Peer connections, ICE candidates and local addresses, SRTP keys, encoded and decoded media |
| Entry points | Peer-controlled RTP, RTCP, SCTP, STUN and DTLS over UDP; signalling data from the page; media for encoding |
| Attackers | ATK-5 and ATK-3 as a hostile peer; ATK-1 through the page's API calls; ATK-9 collecting local addresses |
| Mitigations | Section 6.10a: transport in the network service, UDP sockets as broker-minted handles per (site, container), media in utility processes, no WebRTC parser in the broker; host ICE candidates exposed only as mDNS names unless the page has a camera or microphone grant; fuzz targets per protocol (SEC-30); dependency advisories under SEC-25 |
| Residual | Protocol bugs in the network service expose its assets; IP exposure through relayed or server-reflexive candidates is inherent to the protocol |
| Open | Stack provenance is settled by the standing answer (Section 7); encode-side codec placement |

**Native DRM path (Windows PlayReady through Media Foundation).**
| Item | Content |
|---|---|
| Assets | Encrypted media sessions and licence messages for one (site, container) |
| Entry points | EME messages from a renderer; licence-server responses; the OS CDM and protected media path |
| Attackers | ATK-3 through licence messages and encrypted media; ATK-2 from the renderer |
| Mitigations | Section 6.10a (SEC-33): the CDM is never loaded into the broker; Izcal loads it only in a dedicated sandboxed process per (site, container), at the tier the CDM tolerates; EME messages cross through generated decoders; the protected-media host stays the fallback |
| Residual | The vendor CDM and the OS protected media path are outside Izcal's code and patched on Microsoft's schedule |
| Open | Which OS processes host the CDM and the protected media path, and which sandbox the CDM tolerates (Q-SEC-17) |

### 6.6 IPC: the escape boundary

Stage 1 Section 4 fixes the model: `.hgidl` IDL, endpoints as capabilities, sequence affinity, a reviewed sync allowlist, a fixed wire format with a Kani harness per message (VER-1), and OS transports written from OS documentation. Many of Chromium's IPC bugs were "object lifetime management issues" ([GitHub Security Lab](https://securitylab.github.com/resources/chromium-ipc-vulnerabilities)); this spec infers that Rust plus broker-owned async lifetimes removes most of that class, which the source does not claim. What remains is logic. The rules below amend B2 and Stage 1 Section 4. Rules marked **[gen]** are enforced by `hg-ipc-codegen` or the transport, so no reviewer has to remember them. They are derived from I-2 and the evidence cited; no other engine's IPC code is a source (M-1). IDs IPC-R1 to IPC-R15 keep their 0.1 meaning; 0.2 adds detail to R1, R2, R5, R7 and R8 and adds R16 and R17 (Appendix B).

1. **IPC-R1. Handles are typed, broker-minted and only narrowed. [gen]** A message field carries a handle of a declared kind (`Endpoint<T>`, read-only shared memory, read-only file, writable file inside a granted directory, accepted socket). The transport rejects any OS handle not of that kind, and rejects pseudo-handles and sentinels (`-1`, `-2`, `INVALID_HANDLE_VALUE`, descriptors 0–2 unless declared). The broker never widens a handle's rights. **Per OS:** on Windows the broker is the relay for every handle between sandboxed processes (Stage 1 4.2: named pipes plus `DuplicateHandle` brokered by the browser process), so it duplicates **from** the source process into its own table, determines the kind **from the object** (`GetFileType`, the `NtQueryObject` type name), never from the message, compares it with the declared kind, and duplicates onward with an explicit, narrowed access mask, never `DUPLICATE_SAME_ACCESS`. On macOS the transport rejects task, thread and host ports by kernel-object type and accepts only port kinds the broker created. On Linux `SCM_RIGHTS` descriptors are checked with `fstat` and, for memory, their seals. **Read-only shared memory** is defined per backend: a memfd sealed with `F_SEAL_WRITE`, `F_SEAL_GROW` and `F_SEAL_SHRINK`; a Windows section duplicated without write access; a Mach memory entry created read-only. A writable mapping can never cross as the read-only kind. Evidence: CVE-2025-2783, CVE-2025-2857, and the Windows rule that "any leaked handle can be abused" ([Chromium sandbox design](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/design/sandbox.md)).
2. **IPC-R2. Identity comes from a bound endpoint, never from the message. [gen]** Site, origin, container, frame, worker and process identity are bound to an endpoint when the broker creates it. (a) `.hgidl` declares every endpoint **`bound`** (identity-bearing, non-transferable: the generator gives it no send path and the transport refuses to carry it) or **`free`** (a data channel with no identity, such as a MessagePort's data pipe). (b) The broker creates a separate bound endpoint **per frame and per worker**, so frame identity is as sound as process identity. (c) Generated handlers receive a `CallerIdentity` value that only `hg-ipc` can construct, from the binding. Messages may carry origins and URLs **as data**, in dedicated types (`TargetOrigin` for `postMessage`, `NavigationUrl` for a navigation target); no authorization API accepts those types. This holds on every edge, service to service included (IPC-R17). Evidence: "use the origin the browser already knows" ([Mojo security notes](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)); CVE-2019-11708; CVE-2022-1529.
3. **IPC-R3. Closed type universe. [gen]** No `any`, no inheritance or subtype acceptance, no self-describing nested payloads, no strings the privileged side later parses as URLs, paths, JSON or expressions. URLs and paths are dedicated validated types. Payloads outside `.hgidl` exist only on the IPC-R16 list. Evidence: FORCEDENTRY ([Project Zero](https://projectzero.google/2022/03/forcedentry-sandbox-escape.html)).
4. **IPC-R4. Valid by construction. [gen]** Closed enums (an unknown discriminant is a bad message), unions instead of optional-field combinations, no parallel arrays, explicit maximum lengths, checked arithmetic in generated size calculations.
5. **IPC-R5. A bad message closes the endpoint and terminates the sender; it never panics the receiver. [gen]** Handlers return `Result`. In broker and service handler crates, `clippy::unwrap_used`, `expect_used`, `panic`, `indexing_slicing` and `arithmetic_side_effects` are denied outright, with an allowlist file reviewed like the `unsafe` inventory, the generated `tools/coding-standard/tier-p-exceptions.toml` (coding standard CS-ER-3) (a lint cannot tell which values are message-derived, so the rule covers all of them). The Kani harness proves "no panic for any byte string" for the generated parser; handler-side evidence is the sequence fuzzing of IPC-R14. **Panic strategy:** `panic = "abort"` in every shipping process, the broker and the network service included (owner decision, coding standard CS-D1 (a)). A process that holds every site's state is not trusted to continue after a bug; the network service instead makes panics unreachable from network input (NET-R3, SEC-28), and the broker restarts a crashed service. Evidence: Chromium's rule against `CHECK` on untrusted content ([Mojo security notes](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)).
6. **IPC-R6. Round trip. [gen]** Every accepted message re-serializes to the identical bytes, so two components cannot disagree about a message (the property Vest proves, [Cai et al.](https://www.usenix.org/conference/usenixsecurity25/presentation/cai-yi)).
7. **IPC-R7. Copy, then validate, what the receiver parses. [gen]** Data in shared memory from another process that the receiver **parses or interprets** is copied once into private memory before it is validated or used; the generated reader never reads shared memory twice (double-fetch freedom, as EverParse3D proves for Hyper-V, [Swamy et al.](https://www.microsoft.com/en-us/research/publication/hardening-attack-surfaces-with-formally-proven-binary-format-parsers/)). Opaque pixel and sample buffers that are only sampled are exempt when the receiver fixed their size and format at creation.
8. **IPC-R8. The broker owns lifetime, and identifiers are never reused.** Objects reachable from an endpoint are owned by its state; in-flight async work holds weak references; endpoint closure cancels it. Frame, request, resource and object IDs carry a generation and are never reused, so a stale ID cannot reach a new object (the Rust form of a use-after-free). Endpoint closure during every pending call is a standard simulator fault ([Project Zero](https://projectzero.google/2019/04/virtually-unlimited-memory-escaping.html)).
9. **IPC-R9. Policy and feature gates on the privileged side.** The broker decides; senders request. An unshipped feature's endpoints are not bound at all for processes that should not have them.
10. **IPC-R10. No interpreter on the privileged side.** No script engine, template evaluator or expression language runs on IPC-derived data in the broker, network service or storage service. Firefox's escapes in 2019 and 2022 ran in privileged parent-process script.
11. **IPC-R11. The attack surface is an artifact. [gen]** CI generates, per process type, the list of interfaces and methods it can reach, whether each is sync, which handle kinds it can send, which endpoints are bound or free, and which IPC-R16 payloads it receives. Every pull request that changes it shows the diff, and the security owner approves it (SEC-6).
12. **IPC-R12. Privileged endpoints are bound to a kind of content.** Shell endpoints go only to the shell renderer, prompt endpoints only to the prompt renderer, DevTools endpoints only to panes for a granted target, agent endpoints only to the agent host. No general renderer can request them.
13. **IPC-R13. Sync calls stay on the reviewed allowlist (Stage 1 4.1); nested sync is forbidden** (new in this spec; Stage 1 has no nested-sync rule).
14. **IPC-R14. Fuzz the protocol, not only the parser.** Every endpoint reachable from a sandboxed process gets a generated sequence fuzzer, run in the simulator or under `cargo-fuzz`, driven by a harness that can send anything that process's endpoint set allows; no new endpoint merges without it (SEC-5).
15. **IPC-R15. No parsing of web formats in the broker,** even though it is Rust: parse in a site-locked renderer or a utility process and send normalized results ([Rule of 2](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/rule-of-2.md)).
16. **IPC-R16. Payloads outside `.hgidl` are a named exception list.** Each entry names the payload, its receiver, an owner, a size cap, a fuzz target that deserializes it in the receiver, and copy-then-validate. Entry 1: **WebRender display lists** (renderer → GPU service), WebRender's own serialized format and the largest renderer-to-privileged parser. Entry 2: **V8 inspector JSON** (`script.v8`, DevTools 6.1), which the broker routes as opaque bytes and never parses; parsing and redaction happen in the gateway or the pane. Entry 3: **TUF metadata and update bytes**, routed by the broker as opaque bytes to the updater (UPD-R2). A new entry needs the security owner's approval through SEC-6.
17. **IPC-R17. No process trusts another.** The broker trusts no other process; every service trusts no other process, the broker's identity bindings excepted. Identity on a service-to-service endpoint (network service → storage service, renderer → GPU service) is bound by the broker at creation, so IPC-R2 holds on every edge. "Less privileged" is not used as a test anywhere. Each sandboxed process kind gets an adversarial harness as the compromised peer (SEC-3 (c), SEC-5 (a)), starting with the network service and the GPU service.

**The simulator as a compromised peer.** Mozilla built whole-VM snapshots because restarting the browser was "simply too slow", the lack of a cheap reset broke determinism, and it was hard to know when async handling had finished ([Mozilla A&D](https://blog.mozilla.org/attack-and-defense/2024/06/24/ipc-fuzzing-with-snapshots/)). B1 gives Izcal those properties by construction: the whole multi-process browser runs in one process under a seeded scheduler, and "dispatch returned" is observable. `hg-dst` gets an **adversarial-peer mode**: a simulated process of any sandboxed kind whose endpoints are driven by a structure-aware generator from `.hgidl` (well-formed and malformed messages, sequences, endpoint closure racing in-flight replies, reordering within what sequence affinity allows, attempts to forward bound endpoints, stale IDs). Oracles are the receiver invariants: no panic, no grant outside the peer's (site, container), no handle of an undeclared kind returned, no approval without broker-observed input, every bad message ends in sender termination. In simulation, handles are not OS handles, so IPC-R1 and IPC-R7 are tested at the real transports by a separate **per-OS multi-process transport fuzzer** for handle passing. No published case uses deterministic simulation with an adversarial peer (research notes, absence), so SEC-5 measures the mode's yield on a **planted-bug benchmark** (a pseudo-handle relay, an identity taken from the message, closure during a pending reply, a double fetch, a forwarded bound endpoint) against `cargo-fuzz` on the same endpoints, as DST-1 does for correctness bugs.

**What VER-1 proves and what it does not.** VER-1's harnesses cover the parser. They should name three properties: no panic on any input, round trip (IPC-R6), and bounded allocation. Handle validation (IPC-R1) is policy, outside a parser proof, so each OS backend gets its own Kani harness over the handle-acceptance function, and Kani's bounds are stated with the result ([Kani on Firecracker](https://model-checking.github.io/kani-verifier-blog/2023/08/31/using-kani-to-validate-security-boundaries-in-aws-firecracker.html)).

### 6.6a Consent and trusted UI

Every consent the spec relies on (automation attach, agent per-action confirmations, B9's credential and payment confirmations, permission prompts, the protected-media switch, autofill) was drawn and clicked in 0.1 by the shell renderer, which takes page titles, favicons, URLs and notification text from every site. A re-check that "a grant matches a pending prompt the broker issued" does not help: a compromised shell can approve a real pending prompt with no person present. The rules below make the broker the judge of consent. They are derived from D-S2, UI-5 and UI-10 and from first principles; no other browser's prompt code is a source.

- **CON-R1. The broker decides that a user approved.** It accepts an approval only for a real OS input event that the broker itself received and hit-tested against the prompt rectangle it asked to be drawn, while the prompt was unoccluded and had been visible for at least 500 ms†. A keyboard approval needs the broker-observed focus in the prompt and a broker-observed key event. Messages from the shell or the prompt renderer that say "approved" are ignored.
- **CON-R2. Security prompts are drawn by a separate prompt renderer.** It runs project JS only, jitless, and receives no page-derived strings except the origin as the broker formats it. Credential, payment, agent confirmations, automation attach, permission prompts and the protected-media switch all use it. Options considered: (a) broker input hit-testing alone; (b) a prompt renderer; (c) the native recovery-window code path (D-S2). **(a) plus (b) is chosen for all security prompts;** (c) stays the fallback if the prompt renderer cannot meet the shell UI latency targets.
- **CON-R3. The broker owns the native windows and routes OS input.** Content and shell processes receive input from the broker, never from the OS directly, so the broker sees every event before any renderer does. The broker also decides where the prompt is composited (above page and shell content).
- **CON-R4. The shell never holds secrets.** Autofill pickers show usernames; after the person chooses one through CON-R1, the broker writes the password into the target renderer's form directly. Passkey UI follows the same path.
- **CON-R5. No approval from a moved or new prompt.** An approval within 500 ms† of the prompt appearing, moving or changing text is rejected, and the prompt's rectangle cannot be covered by page or shell content.

Rationale for 500 ms†: long enough that an input aimed at page content cannot land on a prompt that appears under the pointer, short enough to be below typical deliberate reaction times; measured against the shell UI targets before M6.

### 6.7 OS sandboxes per platform

`hg-sandbox` has one backend per OS behind a trait and lives inside `hg-substrate-os` (G3). Each process type's policy is generated next to the code that defines the process type, so policy and code cannot drift. Every process **opens what it needs before it locks down, then receives everything else as pre-opened handles from the broker**; no process opens a path after start-up. Capsicum's 100-line Chromium port worked only because the browser already passed descriptors ([Watson et al.](https://www.usenix.org/legacy/event/sec10/tech/full_papers/Watson.pdf)). At start-up each process probes and records the **sandbox tier it achieved**, shown on an `izcal://sandbox` page and in local logs. A tier below the process type's floor is a visible reduced mode, never silent; on the four reference devices the floor for the renderer, shell renderer, prompt renderer and GPU service cannot be waived (SEC-8).

**JIT tiers are separate executables.** On macOS `allow-jit` is a Hardened Runtime entitlement on the executable, not an SBPL rule, so jitless processes run from a helper binary that lacks it. On Linux jitless processes get a seccomp rule that denies `PROT_EXEC` in `mmap` and `mprotect`, which makes S-4a's "JIT-region allocation attempts fail" testable. **Processes that run only Izcal's code are jitless by default:** the shell renderer, the prompt renderer, the DevTools workspace and the panes, with the full Windows mitigation set (ACG, CET), measured against the shell UI latency targets.

| Process | Linux | macOS | Windows |
|---|---|---|---|
| Renderer; replay host (at the recorded tier) | New user, PID, network and mount namespaces with an empty root where available; seccomp-bpf allowlist with an architecture check, denying `io_uring`, `bpf`, `userfaultfd`, `keyctl`, `ptrace`, `perf_event_open` and namespace creation after setup; Landlock (ABI ≥4: no filesystem, no TCP) | Custom deny-default SBPL applied at start (not a built-in named profile); no Mach lookups except the broker's; no WindowServer, no IOKit user clients; JIT helper binary only for the Standard profile | Untrusted integrity, lockdown token, own job (one process), alternate desktop, win32k lockdown, AppContainer without capabilities, CIG applied after Izcal's own DLLs load (the combination is measured at M5); **ACG and CET only in the jitless tier** |
| Shell renderer, prompt renderer, DevTools workspace and panes | As renderer, jitless: plus the `PROT_EXEC` denial | As renderer, jitless helper binary | As renderer, plus ACG and CET |
| GPU service | seccomp allowlist including DRM ioctls; Landlock `IOCTL_DEV` (ABI ≥5) limited to the render node; no network | SBPL allowing only the GPU's IOKit clients and Metal services | Low integrity; LPAC if drivers allow (Q-SEC-9); win32k lockdown is not possible |
| Network service | seccomp allowing socket calls; Landlock TCP and UDP rules (ABI ≥4 and ≥10); read-only certificate store | SBPL allowing network, no user files | AppContainer with `internetClient`, or LPAC with an ACL'd cache directory |
| Storage service | No network; Landlock limited to the profile directory | SBPL limited to the profile directory | LPAC with an ACL on the profile directory; ACG, CIG |
| Utility, automation gateway, agent host, native-DRM CDM process (at the tier the CDM tolerates) | Strictest seccomp (compute plus IPC); no filesystem, no network; the gateway gets accepted connections as handles from the broker | Custom deny-default computation-only SBPL profile (the built-in named profiles such as PureComputation are among those SDK 27 kills), or an Enhanced Security helper extension | Untrusted, lockdown, win32k lockdown, AppContainer without capabilities, ACG, CIG, CET |
| ML service | Like GPU, with NPU or GPU device nodes | SBPL allowlist for the Core ML services it needs | Low integrity |
| Updater | Unsandboxed, no network, writes only the install directory | Hardened Runtime | Medium integrity, per-user install (SEC-D14); CET, CFG |
| Broker, crash handler | Unsandboxed; CET or IBT where available | Hardened Runtime; Enhanced Security (`com.apple.security.hardened-process`) | Medium integrity; CET, CFG; CIG and extension-point blocking **to be measured at M5** against the IME and assistive-technology compatibility list, because the broker owns the native window (CON-R3) |

Sources: [Linux seccomp](https://docs.kernel.org/userspace-api/seccomp_filter.html); [Landlock](https://docs.kernel.org/userspace-api/landlock.html); [Chromium sandbox design](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/design/sandbox.md) as evidence of which Windows mitigations combine; [Apple Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime); [Apple Enhanced Security](https://developer.apple.com/documentation/xcode/enabling-enhanced-security-for-your-app) (which also needs the `…enhanced-security-version-string` entitlement). Landlock ABI-to-kernel mappings for ABI 8–11 were not checked (Section 11).

**macOS: the sandbox API is the largest per-OS risk.** Apple's developer support says SBPL "has never been documented for third-party use, and thus is not considered API", and that this is why `<sandbox.h>` is deprecated ([Apple Developer Forums](https://developer.apple.com/forums/thread/777509)). The SDK 27 `sandbox_init(3)` page, as quoted on the OpenSSH list, says "these profiles must not be used when building against macOS SDK >= 27.0. Processes opting into them will be killed" ([openssh-unix-dev](https://lists.mindrot.org/pipermail/openssh-unix-dev/2026-August/042690.html)). In that thread "these profiles" are the built-in named `sandbox_init` profiles (`kSBXProfile*`, including PureComputation), so custom profiles applied through `sandbox_init_with_parameters` are not obviously covered. **Measured in October 2026** on macOS 27.0.1 (`research_notes/Izcal security scope/measurements/sec-d6/`): custom deny-default profiles apply and enforce whatever SDK version the binary declares (26.2, 27.0, 28.0), and named profiles kill the process inside `sandbox_init` once the declared SDK is 27.0 or later, with the SDK field alone deciding it. This host had only SDK 26.2, so the SDK 27 builds were simulated by setting the Mach-O SDK field; at M1, on the Mac build machine, a real Xcode 27 build confirms the result and checks that the SDK 27 headers still declare the functions Izcal uses. Sandbox extensions (`sandbox_extension_issue_file_to_process`), which a broker may use to hand file access to children, reportedly fail on macOS 27 for Firefox and others and are not yet measured. In parallel, evaluate Enhanced Security helper extensions ([Apple](https://developer.apple.com/documentation/xcode/creating-enhanced-security-helper-extensions)), which Apple runs "in a sandbox that restricts access to most system services and frameworks", for utility, storage and agent-host processes first; their JIT support and instance limits are undocumented (SEC-D6). If SDK 27 forces a reduced renderer tier, that is an owner decision under SEC-D6, not a log entry. Izcal writes its SBPL from Apple's manual pages and its own black-box tests, never from another browser's profile files (M-1).

**Windows: JIT versus code-integrity mitigations.** ACG forbids writable code pages and CET protects return addresses; both conflict with a JIT that keeps writable and executable code. In 2021 (Chrome 90) Chrome enabled neither in its renderer, and Edge's 2017 answer was to move the JIT out of process ([Google](https://security.googleblog.com/2021/05/enabling-hardware-enforced-stack.html); [Edge blog](https://blogs.windows.com/msedgedev/2017/02/23/mitigating-arbitrary-native-code-execution/)). Izcal makes this an explicit tier: Standard renderers run V8 with the JIT and without ACG or CET; **jitless renderers (the Hardened profile, S-4a) and Izcal-only renderers add ACG and CET**, and every other non-JIT process carries the full set. An out-of-process JIT would need deep changes inside V8, which the rule that fork changes are written against V8's public interfaces (HGBA-2 M-1, Stage 1 D-S10, `CONTRIBUTING.md`) excludes. CET needs Windows 10 20H1 or later and a CPU such as Intel 11th generation or AMD Zen 3 (Google, above), so "CET on every platform" in S-4a becomes "where the hardware and OS support it", with the achieved state shown on `izcal://sandbox`.

**Linux: the kernel surface is the residual risk.** In Google's kCTF program, 60% of submissions in the year to June 2023 used `io_uring`, and Google calls it "safe only for use by trusted components" ([Google](https://security.googleblog.com/2023/06/learnings-from-kctf-vrps-42-linux.html)). Canonical cites a Google figure that 44% of exploits needed unprivileged user namespaces ([Ubuntu](https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces)), and Ubuntu now restricts them per application through AppArmor, a restriction Qualys bypassed three ways ([Qualys](https://www.qualys.com/2025/three-bypasses-of-Ubuntu-unprivileged-user-namespace-restrictions.txt)). Izcal's Linux packages ship an AppArmor profile that permits namespace creation for the Izcal binary only, and the renderer drops the ability to create namespaces after setup. Where namespaces are unavailable (Flatpak, hardened distributions), the renderer runs on seccomp plus Landlock alone, reported as a reduced tier; these are not reference-device configurations, so SEC-8 allows the waiver and counts it. The Flatpak build (Stage 2, 13.2) documents its reduced isolation.

### 6.8 V8 and the Rust↔V8 boundary

**Patch latency is Izcal's real V8 exposure.** Izcal inherits every V8 bug, and the window between an upstream fix and an Izcal release is the time a public bug can be used against Izcal users. Today D2 says "ship V8 security fixes within one Chrome stable cycle", S-4e is a ship-blocker rebase target with a backport fallback, and Stage 1 7.7 tracks "Days behind Chrome stable V8 security fixes" with an escalation at more than two weeks of lag. What is missing is narrower: a **latency target by severity**, per-advisory publication, a rule that keeps backports possible, and a failure action when the fix itself is late. One cycle is about four weeks, against fixes in about a day at Mozilla. SEC-1 sets the bound by severity (72 hours† for exploited bugs, 7 days† for critical and high, one cycle for the rest).

Three conditions make that bound reachable:
- **The pinned V8 is always on a branch Chrome still patches** (stable, or extended stable where Chrome maintains it). Cherry-picks apply cleanly only to a maintained branch; on an abandoned branch Izcal would be doing V8 security engineering itself, which no staffing row covers. Izcal tracks Deno's `rusty_v8` tags when they are on a patched branch; when they are not, the fork rebases its own commits onto the V8 branch directly (SEC-D12). Chrome's branch policy is taken from the critique and still needs a primary source (Q-SEC-16).
- **The clock cannot be gamed.** Time zero is the earliest of the first public V8 commit containing the fix (found afterwards from the fix commit's history), the first Chrome stable release containing it, and public disclosure. It is computed after the fact by a script over upstream history, so not noticing a commit cannot improve the score.
- **The pipeline is measured where releases build.** The M0 figures (17.5–46 minutes per release build, Stage 1 7.1) came from the M4 Mac, huey and the Windows laptop, which SEC-27 keeps out of CI. SEC-2's drill runs on the actual release runners, records each stage's time, and the 72-hour bound is checked against the sum of measured stages.

The pipeline is: the V8 bot opens the update PR within 24 hours† (7.7); if the full rebase is not green in time, the `rusty_v8` owner cherry-picks the V8 security commits onto the pinned branch (S-4e backport rule); the fork builds release-sandbox and debug-sandbox archives on all three platforms; T0 to T2 pass; the release is published as an **emergency release** (UPD-R4) signed by the `releases` role (UPD-R3) and delivered through TUF. **When an exploited-bug window is missed**, the emergency mitigation channel (UPD-R5, SEC-D11) switches on the narrowest signed configuration that removes the exploited path (jitless renderers, WebAssembly off, a feature off) for all users until the fix ships; an advisory alone barely bites, because jitless covers about half of exploited V8 bugs and few users act on advisories. Whether embedders can get advance notice was not researched (Q-SEC-1).

**The V8 sandbox is a layer, not a proven boundary.** In 2024 the V8 team listed "a number of issues to resolve before it becomes a strong security boundary"; recent exploits "had to get past the sandbox" ([V8](https://v8.dev/blog/sandbox)). In October 2026 Chrome pays a flat $5,000 for V8 sandbox issues (secondary report, [Help Net Security](https://www.helpnetsecurity.com/?p=387768)). S-4b keeps it on. Izcal counts it as one layer under the process sandbox, and **publishes a "which layer stops which class" table** in its security documentation at M7 (Section 9.1), rather than calling any single layer the boundary.

**Rust↔V8 rules** (amend HGBA-2 Section 4.8; the FFI crates are `hg-dom`, `hg-bindings` and `hg-script` per G6; SEC-11 measures them):
- **FFI-R1. `v8::External` only through a typed wrapper.** Raw `*const c_void` in an `External` elides Rust lifetimes; Deno's CVE-2024-27934 was a use-after-free and code execution from exactly that ([GHSA-3j27-563v-28wf](https://github.com/denoland/deno/security/advisories/GHSA-3j27-563v-28wf)). Izcal's wrapper stores an index into a per-isolate table owned by Rust, checked on every access, with a type tag and a generation (IPC-R8's rule, applied in-process); a lint forbids `External::new` outside it.
- **FFI-R2. No Rust borrow of a JS-visible buffer across an await or a callback into script.** Resizable and transferable ArrayBuffers can shrink or detach while Rust holds a slice; Deno's CVE-2023-28445 gave out-of-bounds read and write that way ([GHSA-c25x-cm9x-qqgx](https://github.com/denoland/deno/security/advisories/GHSA-c25x-cm9x-qqgx)). Async operations copy the bytes, or pin the backing store and re-check its length after every point where script can run.
- **FFI-R3. `cppgc` discipline stays S-4c**, including the debug-V8 CI configuration that catches missing write barriers, which release V8 misses silently.
- **FFI-R4. Fuzz the boundary.** Fuzzilli through the renderer's REPRL mode (Stage 1 7.5) plus targeted tests that resize, detach and transfer buffers during every async binding.

**DrumBrake.** The Hardened profile runs Wasm on V8's DrumBrake interpreter, and an OffensiveCon 2026 talk claims 23 RCE bugs in it (unverified, already in HGBA-2 4.2). D2's rule stands: an in-the-wild DrumBrake exploit turns Hardened's Wasm default off, and that switch is one of the emergency channel's closed list (UPD-R5).

### 6.9 GPU service

The GPU service is Izcal's widest privileged surface. It is reached from every renderer through WebRender display lists, WebGL (ANGLE on Windows, native GL on macOS and Linux, Stage 1 5.7) and WebGPU (wgpu-core and Naga), and it reaches vendor drivers. Graphics was the second-largest source of Chrome in-the-wild 0-days (6 of 37), Firefox's WebGPU had two sandbox escapes in one release ([MFSA 2025-87](https://www.mozilla.org/en-US/security/advisories/mfsa2025-87/)), and 2025's Android and Apple chains passed bad sizes and call sequences through ANGLE and GPU user-mode drivers (Mali, Adreno, Metal) ([GTIG 2025](https://cloud.google.com/blog/topics/threat-intelligence/2025-zero-day-review/)). Rust in wgpu, Naga and WebRender removes memory corruption in those layers and nothing below them. No existing target covers this, so SEC-10 adds one. The rules:
- **GPU-R1. Sandbox on every desktop platform,** built at M3–M4 and gated at M8 (Section 6.7), with policy tests like S-2's.
- **GPU-R2. wgpu-core is the WebGPU validator.** Izcal does not duplicate it. Izcal adds per-renderer resource ownership (a renderer names only resources it created, with generation-tagged IDs, IPC-R8), size and limit clamps below the adapter's limits, and fuzzing. The W3C WebGPU conformance test suite's validation tests are the metric.
- **GPU-R3. WebGL is validated in the GPU service, inside Izcal's WebGL implementation,** before ANGLE or the GL driver sees a call. Where that implementation lives and which shader translator macOS and Linux use is an open question with this recommendation (Q-SEC-15); the Khronos WebGL conformance suite's negative tests are the metric. Writing the WebGL implementation is engine work in the Graphics track, not in the security estimate.
- **GPU-R4. Display lists are IPC-R16 entry 1:** a fuzz target that deserializes them in the GPU service, a size cap per frame, copy-then-validate, and the Graphics track as owner.
- **GPU-R5. Fuzz the layer that 2025 used.** A WebGL call-sequence fuzzer and a WGSL shader fuzzer run in T2. DarthShader found 39 bugs in shader pipelines of three browsers, 15 with CVEs ([arXiv 2409.01824](https://arxiv.org/abs/2409.01824)).
- **GPU-R6. Check translations.** A wrong Naga translation is a security bug if it drops a bounds check; a differential test compares generated HLSL, MSL and SPIR-V against the WGSL semantics for bounds-checked accesses.
- **GPU-R7. Timers.** WebGPU timing exposes keystroke timing and device identity ([Giner et al.](https://www.gruss.cc/files/WebGPUAttacks.pdf); [DRAWNAPART](https://arxiv.org/abs/2201.09956)); WebGPU timestamp queries follow the same coarsening as `performance.now()` outside cross-origin isolation (I-6).
- **GPU-R8. Blocklist updates** ship as a TUF data channel (UPD-R1) within 7 days† of a driver advisory. The blocklist is built from Izcal's own device testing and vendor advisories (Section 7).

### 6.10 Codecs, images, fonts and PDF

libwebp and libvpx each hit four platforms with one bug ([TAG 2023 review](https://storage.googleapis.com/gweb-uniblog-publish-prod/documents/Year_in_Review_of_ZeroDays.pdf)), and media codecs and WebRTC were 11% of Chrome's in-the-wild 0-days. **The Stage 1 design already puts C decoders and PDF in sandboxed utility processes** (Section 1: "RLBox-style for C libraries … PDF parsing and rendering"; 13.2: "libvpx in the library sandbox"; Section 6: the PDF renderer runs in a utility process). What is missing is a target and a gate: S-5 is a Stage 2 target. This spec recommends **making Stage 1's library-sandbox design a Stage 1 target**, gated at M7 (SEC-12, SEC-D3):
- C decoders and the OpenH264 binary run in a single-purpose utility process at the strictest tier, or in RLBox-style wasm2c inside a utility process; the wasm2c translator is pinned and treated as trusted code (I-3), and Izcal writes its own tainted-type wrapper (Section 7).
- **Images and fonts are Rust by default.** Chrome's Rust font stack had 39 fuzzer bugs and none security-critical ([Chrome](https://developer.chrome.com/blog/memory-safety-fonts)); the decoder choice is open with a recommendation (Q-SEC-13).
- **OS decoders** (H.264, AAC, HEVC through Media Foundation, VideoToolbox, VA-API or GStreamer) run in the utility process where the OS allows; where an OS framework forces in-process decoding, the residual risk is stated in that process's threat model (Q-SEC-4).
- **PDF: no code generated from document data.** PDF.js CVE-2024-4367 turned document contents into executed JavaScript ([Codean Labs](https://codeanlabs.com/?p=1431)), as did CVE-2018-5158 ([Red Hat](https://access.redhat.com/security/cve/CVE-2018-5158)). The PDF engine is built by Izcal in Rust from ISO 32000 (Section 7), never evaluates or compiles anything from the document, renders in a utility process, and its output is shown in an opaque-origin renderer, never in the shell.
- **Historical bug classes become tests.** Each decoder gets regression inputs for its library's past CVE classes, because rewrites reintroduce old bugs (Stylo).

### 6.10a WebRTC and the native DRM path

**WebRTC** ships in Stage 1 (Stage 1 M7, "PQC for WebRTC DTLS"; 13.2, OpenH264 for WebRTC) and has an in-the-wild record (CVE-2022-2294, CVE-2023-7024). It parses peer-controlled RTP, RTCP, SCTP, STUN and DTLS over UDP and holds local addresses. Design:
- **Placement.** Transport (ICE, DTLS, SRTP, SCTP) runs in the network service; encode and decode run in utility processes; the broker runs no WebRTC parser. UDP sockets are broker-minted handles per (site, container) (IPC-R1), granted only to the network service.
- **Stack.** Built by Izcal in Rust from the IETF RFCs and the W3C WebRTC specification, using unmodified non-browser crates for primitives (the standing answer; Section 7). OpenH264 is Cisco's binary, in a utility process under SEC-12.
- **ICE candidate policy.** Host candidates are exposed to the page only as mDNS names unless the page holds a camera or microphone grant in that (site, container); candidate gathering is per (site, container).
- **Fuzzing.** A fuzz target per protocol parser (RTP, RTCP, SCTP, STUN, DTLS) in T2 (SEC-30).

**Native PlayReady** (Stage 1 13.1a item 2, Windows, M7) brings a vendor CDM and the Windows protected media path into Izcal's own engine. Design: the CDM is never loaded into the broker or a renderer; Izcal loads it only in a dedicated sandboxed process per (site, container), at the tier the CDM tolerates; EME messages reach it through generated decoders; licence traffic goes through the network service. Which OS processes host the CDM and the protected media path is to be measured at M7 (Q-SEC-17). Protected media mode remains the fallback (SEC-33).

### 6.11 Network service and PKI

The network service holds every site's cookies and parses hostile bytes. Its own code is Rust, so its failures will look like 2026's rustls and quinn advisories: downgrades, state-machine confusion, pre-authentication allocations and panics (Section 3.3). Rules:
- **NET-R1. Request context comes from the endpoint** (IPC-R2): initiator, site, container and partition key are bound to the requester's endpoint, never taken from the request.
- **NET-R2. Filter before delivery.** Cross-site responses a renderer must not read are withheld in the network service. Chrome's site isolation needed CORB for this ([Google 2018](https://security.googleblog.com/2018/07/mitigating-spectre-with-site-isolation.html)); Izcal implements the Fetch standard's successor mechanism, **ORB** (opaque-response blocking), written from the Fetch standard (M-1).
- **NET-R3. Panics are a denial-of-service class.** The network service aborts on panic like every shipping process (IPC-R5), so a panic reachable from network input ends every site's connections until the broker restarts the service. No panic may be reachable from network input: the tier P lints apply, and T2 fuzzing finds what they miss (SEC-28). Allocations sized by network input are fallible, and their failure ends that connection, not the service; allocations before authentication are bounded (SEC-28).
- **NET-R4. Differential protocol fuzzing.** HTTP/2, HTTP/3, TLS handshake and WebRTC transport fuzzing in T2, with a state-machine oracle written from the RFCs.
- **NET-R5. Dependency advisories.** Advisories against rustls, rustls-webpki, neqo, hickory-dns or any shipped dependency follow SEC-25's response times.
- **NET-R6. PKI as decided in 13.2**: platform verifier first, CT enforced in Stage 1, CRLs pushed as a TUF data channel (UPD-R1). A root-store policy process with staged, CT-evidenced distrust, as browsers used for Symantec ([Google 2017](https://security.googleblog.com/2017/09/chromes-plan-to-distrust-symantec.html)) and Entrust ([Google 2024](https://security.googleblog.com/2024/06/sustaining-digital-certificate-security.html)), is needed for the fallback root list (Q-SEC-5).
- **NET-R7. Local network and loopback (TB-13).** Requests from web content to private or loopback addresses, including names that resolve there after a DNS change, need a Local Network Access grant for that (site, container); the check uses the endpoint's context and the address actually connected to, not the name. DNS-rebinding attacks on local agent servers (Section 3.4) are the reason (SEC-31).

### 6.12 Agents, automation and DevTools

The DevTools spec already makes every external session a scoped principal with consent, origin sets, sensitive classes, no ports by default and a session log (6.5), and B9 bounds agents by origin sets. The 2025–2026 record adds channels those rules do not name (Section 3.4). Additions (amend B9, AG-3 and DevTools 6.5):

- **AGT-R1. Entry points.** An agent task starts only from the trusted agent UI. No URL scheme, deep link, query parameter, address-bar fallback (a string that fails URL parsing), notification or page script creates, extends or re-targets a task. CometJacking used the agent's own invocation URL; the Atlas omnibox report (single secondary source) used a failed parse.
- **AGT-R2. Memory.** Agent memory is per task by default. Anything kept across tasks needs confirmation through CON-R1 and is never writable through a web request ("Tainted Memories").
- **AGT-R3. Schemes and tools on parsed URLs.** The broker checks the **parsed** scheme and origin of every agent navigation and fetch: only `https:` origins in the task's sets (plus `http:` only by explicit grant). `file:`, `view-source:`, `blob:` and `data:` navigations, internal `izcal:` pages and extension pages are refused. PleaseFix bypassed a string filter twice with `view-source:file://` and a `#.pdf` suffix ([Zenity](https://zenity.io/research/pleasefix-vulnerabilities)).
- **AGT-R4. Credentials and extensions.** The built-in password manager never fills for an agent without B9's confirmation through CON-R1, and no extension is reachable from an agent task.
- **AGT-R5. No extension messages to the agent host or AI UI.** CVE-2026-0628 let malicious extensions with basic permissions take over Chrome's Gemini panel ([Unit 42](https://unit42.paloaltonetworks.com/gemini-live-in-chrome-hijacking/)).
- **AGT-R6. Rule-of-Two task policy.** Meta's rule: a session should hold at most two of untrusted input, access to sensitive data, and the ability to change state or communicate externally ([Willison on Meta](https://simonw.substack.com/p/new-prompt-injection-papers-agents)). Izcal applies it per task. **Untrusted input** is any origin outside the task's writable set **and** any content in a sensitive-class origin that third parties can write (mail, messages, issues, comments), unless a site policy or B8 provenance marks that content first-party; an incoming email in the mail origin is the commonest injection. If a task has read untrusted input **and** holds a read-write origin in a sensitive class (mail, banking, payments, identity, code hosting, as a container or site policy marks them), every write to that origin needs per-action confirmation through CON-R1 (SEC-19). Confirmation counts per task are published, because a policy users click through is not a control (SEC-19 (b)).
- **Residual risk, stated in the UI and docs.** Origin sets bound reach, not intent. An agent allowed to write to a mail origin can still be steered by the user's own first-party content, and confirmation fatigue erodes AGT-R6. Model defenses reduce these in percentages that an adaptive attacker raises ([arXiv 2510.09023](https://arxiv.org/abs/2510.09023); Gray Swan's 4.7% single-attempt and 33.6% ten-attempt figures are a secondary report). AG-3 and DT-13 therefore test with model defenses off.
- **AGT-R7. MCP and adapters.** The MCP server exists only with AI on (B10). CDP, BiDi and DAP are parsers of untrusted client input; they run in the automation gateway, outside the broker, and decode only through generated B2 decoders (DevTools 6.1). MCPTox's 36.5% average success for poisoned tool metadata, with more capable models "often more susceptible" ([arXiv 2508.14925](https://arxiv.org/abs/2508.14925)), supports DevTools 6.5 rule 9 (hashed descriptions; a change revokes the grant). Third-party MCP servers the user connects are outside Izcal's code; their residual is stated in the agent-host model.
- **AGT-R8. Local malware launching Izcal.** Infostealers usually kill the browser and relaunch it in debugging mode ([Elastic](https://www.elastic.co/security-labs/threat-command/katz-and-mouse-game)). The control is that **no command-line flag, environment variable or project file attaches anything to a user container**; pipe launches always get a fresh ephemeral container (DevTools 6.5 rule 3). As defense in depth, Izcal also refuses an automation launch against a profile directory another Izcal instance holds; that does not stop a kill-and-relaunch. The consent prompt goes through CON-R1 and should resist synthetic OS input where the platform allows (Q-SEC-7) (SEC-20).

### 6.13 Recordings

A recording holds every response, timer and input event of a session; credential fields are never recorded, but other keystrokes are, with timestamps (DevTools DT-D2, 6.5). Replay can read any value the page computed, so a leaked bundle exposes more than any HAR file. Additions to DevTools 6.5 rules 8 and 10 (SEC-21):
- **REC-R1. Imported bundles are hostile input.** They replay in a replay host under an opaque origin, with no network, no user container and no write capability; opening one equals opening a page (ATK-1). The bundle parser is a Tier 1 surface with its own fuzz target. No published attack on trace importers was found; the closest analogue is a crafted trace file that ran script in a diagnostics UI (CVE-2025-40943, [NVD](https://nvd.nist.gov/vuln/detail/cve-2025-40943)).
- **REC-R2. Saved bundles use an OS-bound key** under SEC-34. Chrome's DPAPI-protected cookies were readable by any program running as the user, which is why Chrome added App-Bound Encryption ([Google](https://security.googleblog.com/2024/07/improving-security-of-chrome-cookies-on.html)).
- **REC-R3. Retention.** Saved bundles expire after 30 days† unless the user pins them. Rationale: long enough to cover a support or bug-report cycle, short enough that a forgotten bundle stops being a standing token store; a legal review may change the period (this was SEC-D9 in 0.1).
- **REC-R4. Typing rhythm.** Exported bundles quantize the **intervals between keystrokes in text-entry fields** to 100 ms† buckets unless the user keeps exact timing, because inter-keystroke timing is a behavioral biometric; the WebGPU cache attack recovered it at F1 82–98% ([Giner et al.](https://www.gruss.cc/files/WebGPUAttacks.pdf)). Other input keeps its timing. Quantizing changes how input interleaves with timers, so quantized exports run through DevTools rule 6's redact-and-replay divergence check and are labelled "approximate from the first divergence". The 100 ms bucket is near typical inter-key intervals and is unjustified by data so far; a measurement plan sets it (Q-SEC-19).
- **REC-R5. Fingerprinting seed.** A bundle carries the session's anti-fingerprinting seed (Stage 1 Section 6); the export manifest says so.
- **REC-R6. Replay uses exactly the recorded engine configuration.** JIT on or off, V8 flags and the worker-task mode are recording fields (DT-D2); the replay host runs at the matching sandbox tier, and a jitless replay is available only for recordings made jitless.

### 6.14 Shell features: credentials, importers, phishing, DRM host, crash reports

- **SH-R1. Password manager and passkeys.** Autofill and passkey UI are drawn by the browser outside the page DOM, after a CON-R1 approval; the shell sees usernames only and the broker writes the secret into the target form (CON-R4). DOM-injected autofill leaked credentials from 10 of 11 password-manager extensions tested ([The Hacker News](https://thehackernews.com/2025/08/dom-based-extension-clickjacking.html)). Stage 2 extensions cannot intercept WebAuthn ([SquareX](https://labs.sqrx.com/recap-of-our-passkeys-pwned-talk-at-def-con-2a56356e3835)). Credentials live in the OS keychain (13.2). Proofpoint's user-agent downgrade of FIDO sign-in (research note `new_surfaces_privacy.md` 12.3) is recorded for the user-agent work (UA-1), not decided here.
- **SH-R2. Importer.** On Windows, Chrome binds cookie decryption to Chrome's own identity; a third-party program can decrypt it only with the techniques infostealers use ([Google](https://security.googleblog.com/2024/07/improving-security-of-chrome-cookies-on.html); [Red Canary](https://redcanary.com/blog/threat-intelligence/google-chrome-app-bound-encryption/)). Izcal's importer therefore never imports cookies, takes Chrome passwords only from a CSV file the user exports from Chrome, and imports bookmarks and history from disk. If Chrome documents a supported export interface, the importer uses it. Its file access is documented for EDR vendors so it is not mistaken for an infostealer. Google's 2024 post says passwords were *planned* for App-Bound Encryption; whether they are in 2026 is unverified. Importer formats remain black-box observations (13.2). This was SEC-D8 in 0.1; direct decryption was never a real option.
- **Phishing and malware URL protection is missing.** The Stage 1 specs mention only download reputation checks and B10's carve-out for security classifiers, while the shell UI designs phishing interstitials and UI-9 measures them. An agent navigates wherever it is told, and LayerX estimated Atlas users were "up to 90% more exposed than traditional browsers" (vendor estimate, about phishing, [The Hacker News](https://thehackernews.com/2025/10/new-chatgpt-atlas-browser-exploit-lets.html)). The choice of data source and protocol, for URLs and for download reputation, is an owner decision (SEC-D7).
- **SH-R3. Extensions (Stage 2).** Wasmtime is one layer: CVE-2026-34971 was a sandbox escape through a Cranelift aarch64 miscompile ([NVD](https://nvd.nist.gov/vuln/detail/CVE-2026-34971)), and Wasmtime published 51 advisories in 2026 ([Wasmtime](https://github.com/bytecodealliance/wasmtime/security/advisories)). Each extension also gets its own OS-sandboxed process, Wasmtime's Spectre mitigations and signal-based traps stay on, Izcal tracks a Wasmtime LTS line, updates are TUF targets and show a capability diff, and no extension gets a debugger, cookie-jar, WebAuthn or DOM-injection capability ([Chrowned](https://dspace.networks.imdea.org/handle/20.500.12761/1704)).
- **SH-R4. Protected-media host rules** (amend the I-2 note and D-S7). Microsoft's own guidance is to treat all web content as insecure and turn off host objects, web messages and script injection when not needed ([Microsoft Learn](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security)). Izcal's host: (a) exposes **no bridge**: zero host objects, zero web-message handlers, zero injected scripts; (b) runs in its own process, never the broker, because the WebView2 host can read every hosted cookie; (c) restarts when the OS runtime updates, because an app that runs continuously "will continue to use the previous version of the WebView2 Runtime" ([Microsoft Learn, Distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)); (d) refuses protected media mode below a minimum WebView2 runtime or OS WebKit version, published per release; (e) states in its label that the mode has Microsoft's or Apple's patch level. Per-(site, container) data folders (D-S7) already give each container its own WebView2 process group ([Microsoft Learn](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)) (SEC-22). The switch into the mode is a security prompt (CON-R2).
- **SH-R5. Crash reports.** Renderer, shell, prompt, pane and replay-host dumps use filtered stacks; dump access is restricted; dumps are deleted after 30 days† (SEC-35). Rationale: long enough to triage a crash cluster across one release cycle. A Mozilla engineer described dumps as "highly sensitive private data" ([Bugzilla 1177121](https://bugzilla.mozilla.org/show_bug.cgi?id=1177121), a comment, not a policy). This was part of SEC-D9 in 0.1; a legal review may change the period.

### 6.14a Downloads, external protocols and File System Access

Delivering a malicious file and launching an external protocol handler are among the plainest routes to compromising ordinary users, this spec's main population (Section 1). Stage 1 13.2 plans a downloads manager "with file-type and reputation checks" and a File System Access permission, without rules. The rules below are first-principles; the research notes hold no incident data for them, which Section 11.2 records.
- **SH-R6. Downloads are tagged.** Every downloaded file gets the OS origin mark: Mark-of-the-Web (the `Zone.Identifier` stream) on Windows, `com.apple.quarantine` on macOS. Linux has no common equivalent; the residual is stated.
- **SH-R7. Dangerous types need a second step.** Executable and script types, and archives containing them, are never opened automatically; opening needs a CON-R1 approval. Reputation checks depend on SEC-D7's source.
- **SH-R8. External protocol launches need a gesture and a per-origin prompt.** A page can launch an external scheme handler only after a user gesture and a CON-R1 approval naming the scheme, the handler and the origin; there is no launch from cross-origin iframes and no automatic repeat. A remembered approval is per (origin, scheme, container).
- **SH-R9. File System Access handles are narrow.** A renderer receives a read-only file, a writable file inside the granted directory, or a directory handle limited to the grant, all as typed broker-minted handles (IPC-R1); the broker refuses system and profile directories.

### 6.15 Privacy surfaces

PRIV-1, PRIV-2 and D-S1 stay. Additions:
- **PRIV-2 must include adaptive de-noising.** Randomized canvas defenses in Brave and nine extensions were predicted or removed in every mechanism tested (WWW 2025, abstract only); Firefox 145's protections cut the share of unique users "by almost half" ([Mozilla](https://blog.mozilla.org/en/firefox/fingerprinting-protections/)).
- **GPU and NPU timing** join the surface list: GPU timing re-identified about 70% of 500 devices (WiSec 2025, Medium confidence), and D-S1 normalizes adapter information, not timing.
- **Partitioning** covers connection pools and DNS by the same key (I-1(b) already lists them; pool-party channels are why, [Snyder et al.](https://www.usenix.net/system/files/usenixsecurity23-snyder.pdf)); bounce-tracking mitigation is on by default. CHIPS and FedCM versus the retired Privacy Sandbox APIs are a privacy-spec question, out of scope here.
- **WebRTC addresses** are covered by Section 6.10a's ICE policy.
- **GPC wording.** Stage 1 Section 6 says GPC is enforceable in three states. Vendor blogs count about twelve by 2026, with conflicting dates (Low confidence); California's AG settled with Healthline for $1.55M partly over GPC ([California AG](https://oag.ca.gov/node/604800)). The sentence needs a legal check before M7.
- **Crash reports** follow SH-R5.

### 6.16 Supply chain, build, release and update

**Current state, read from the repositories on 18 October 2026.** The project repository is on `main` at `781ac97`. The fork's release branch `hgba` is at `8ba5648`. **Fork PR #2 (Rorqualx/rusty_v8-izcal#2, branch `ci/security-hardening`) was open on 18 October 2026 with CI running**; the rows it fixes are marked "fixed by fork PR #2 (pending merge)", and none of them is true on `hgba` until it merges.

| Item | State | Evidence |
|---|---|---|
| Actions pinned to full commit SHAs | **Project: fixed** in `checks.yml`. **Fork: open on `hgba`** (8 tag-pinned `uses:` in `izcal-v8-build.yml`, such as `actions/checkout@v4` and `softprops/action-gh-release@v2`, and 2 in `izcal-checks.yml`); **fixed by fork PR #2 (pending merge)** | Workflow files on `main`, `hgba` and the PR branch |
| CI checker-tool downloads verified | **Project: fixed**: gitleaks, actionlint, zizmor and cargo-deny pinned and SHA-256-checked. **Fork: open on `hgba`** (`izcal-checks.yml` downloads gitleaks with no hash check and runs `bash <(curl …download-actionlint.bash)`); **fixed by fork PR #2 (pending merge)** | `tools/ci-install-tools.sh`; `izcal-checks.yml` |
| Toolchain downloads in the V8 release build | **Open on both branches, including PR #2**: Chromium `tools/win` is fetched from a commit-pinned googlesource `+archive` URL with no hash; LLVM comes from `apt.llvm.org` (`llvm-toolchain-noble-21`) and `brew install llvm@21` with no exact version. libclang's version shapes the bindgen output, so these affect the binding file SEC-13 hashes and the comparison SEC-15 makes. V8's own `tools/rust_toolchain.py` and `tools/clang/scripts/update.py` show no hash check (Q-SEC-11) | `izcal-v8-build.yml` lines 93–130 on the PR branch; research note L5 |
| Caches on tag builds | **Fork: open on `hgba`** (`sccache` and `actions/cache` run on tag builds); **fixed by fork PR #2 (pending merge)**, which skips them on `refs/tags/` | `izcal-v8-build.yml` |
| Template injection | **Project: fixed** (event fields passed through `env:`); zizmor runs in the `security` check | `checks.yml`; `tools/checks.sh` |
| V8 archive hash | **Open**: `build.rs` verifies the archive only if `RUSTY_V8_ARCHIVE_SHA256` is set, and nothing sets it | `m0/rusty_v8/build.rs` `pinned_archive_sha256()`; repository search |
| Generated binding file | **Open**: `src_binding*.rs` is never hash-checked, and it is Rust source included into every consumer build | `build.rs` `print_prebuilt_src_binding_path()` |
| `SHA256SUMS` | **Detects corruption, not tampering**: computed and published by the same pipeline beside the archives | `izcal-v8-build.yml` publish job |
| Release provenance and signing | **Open**: no attestation, no signature; immutable releases not confirmed (repository settings were not checked, Q-SEC-10) | Workflow files |
| Third-party action with write rights | **Open**: `softprops/action-gh-release` runs with `contents: write` (SHA-pinned only in PR #2) | `izcal-v8-build.yml` |
| Required check runs PR code | **Open**: `checks.yml` runs the PR's own `tools/checks.sh`; the fork fetches it from the unpinned default branch | `checks.yml`; `izcal-checks.yml` |
| Self-hosted runners | **Open**: `IZCAL_RUNNER_*` variables can point `pull_request` jobs of a public repository at self-hosted machines | `izcal-v8-build.yml` matrix |
| Governance | **Open**: one maintainer owns both repositories; self-merge allowed; no commit or tag signing; release-tag protection client-side (`pre-push`); step 4 of the S-4e procedure lifts force-push protection on `hgba` each rebase | `CONTRIBUTING.md` "Code review", "Pushing", "S-4e upstream rebase" |
| Reproducible builds | **Not measured**: M0 measured runtime determinism and kept rebuild logs, not bit-for-bit archive equality across builders | `m0/determinism_crossplatform.md` |
| AI review | **Quality gate, not a boundary**: local, read-only tools, structured verdict, but fails open and reads diff text that can carry instructions | `tools/checks.sh` review step |

**Controls.** Priority P0 means before any outside contributor or public binary; P1 before Stage 1 release (M7); P2 Stage 2. Costs are estimates†.

| Control | Threat | Priority | Cost† | Target |
|---|---|---|---|---|
| Merge fork PR #2 after its CI passes (SHA pins, verified checker downloads, no caches on tag builds) | Moved tags, tampered tools, cache poisoning | P0 | review time | SEC-13 |
| Hash-pin Chromium `tools/win`; pin LLVM to an exact version or a hashed archive in the release build; verify or pin the toolchains V8's own scripts fetch (Q-SEC-11) | Compromised download source; unreproducible bindings | P0 | 1–2 days | SEC-13, SEC-15 |
| Commit the V8 archive and binding hashes in the project repository per platform and configuration; set `RUSTY_V8_ARCHIVE_SHA256` from them; extend `build.rs` to verify the binding file (offer upstream); build fails on mismatch | Replaced archive or binding (ATK-6, ATK-8) | P0 | 1–2 days | SEC-13 |
| Enable immutable releases on the fork; publish as draft → assets → publish ([GitHub](https://github.blog/changelog/2025-10-28-immutable-releases-are-now-generally-available/)) | Asset replacement, tag moves | P0 | 1 hour | SEC-13 |
| Build provenance attestations from a reusable build workflow (which GitHub says helps "achieve SLSA v1.0 Build Level 3", [GitHub docs](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-and-reusable-workflows-to-achieve-slsa-v1-build-level-3)); consumers verify before use; replace the third-party release action with `gh release` | Builder or release compromise | P0 | 2–4 days | SEC-13 |
| Required checks run the checker from the protected base branch, not the PR; the fork pins the project's `checks.sh` by commit | Malicious PR disables its own checks | P0 | 0.5 day | SEC-27 |
| Server-side rulesets: protect `main`, `hgba` and `v*-izcal.*` tags; require signed commits and signed release tags; perform the S-4e force-push as a logged ruleset bypass, not by disabling protection; enable GitHub's SHA-pinning policy and a pin updater (Renovate or Dependabot), so pins do not go stale | Account misuse, history rewrite, rogue tag, stale pins | P0 | 1 day | SEC-27 |
| Maintainer accounts with passkeys or hardware keys only; no SMS or TOTP; no long-lived tokens (maintainer phishing led to the chalk and debug compromise, [Endor Labs](https://endorlabs.com/learn/major-supply-chain-attack-compromises-popular-npm-packages-including-chalk-and-debug)) | Phishing, token theft | P0 | small | SEC-27 |
| No self-hosted runner on any `pull_request` job; build machines (huey, the Windows laptop) serve only as independent rebuilders, not CI runners ([SecurityWeek on PyTorch](https://www.securityweek.com/new-class-of-ci-cd-attacks-could-have-led-to-pytorch-supply-chain-compromise/)) | Runner takeover | P0 | none | SEC-27 |
| No AI agent with a shell, write token, secrets or cache write runs on untrusted input in CI; the local AI review stays read-only and is documented as advisory | Prompt injection into code execution ([Clinejection](https://adnanthekhan.com/posts/clinejection)) | P0 | none | SEC-27 |
| Builds and tests run with no secrets in the environment; publishing is a separate job with short-lived OIDC credentials | Build-script payloads (arrayref) | P0 | 0.5 day | SEC-27 |
| cargo-deny extended from advisories to bans, licenses and sources (crates.io only; git dependencies need a recorded decision) | Unapproved crates, slopsquatting ([arXiv 2406.10279](https://arxiv.org/abs/2406.10279)) | P0 | 1 day | SEC-14 |
| cargo-vet with imported audits from organizations that audit Izcal's shared dependencies; safe-to-deploy for shipped crates, safe-to-run for tools; exemptions shrink each milestone ([cargo-vet](https://mozilla.github.io/cargo-vet/)) | Malicious or compromised versions | P1 (start P0) | 1 week, then ongoing | SEC-14 |
| Dependency cooldown: no crate version younger than 7 days† without a recorded security reason. Rationale: arrayref was live 86–107 minutes and removed within about two hours; seven days covers that by a wide margin plus weekends and slower detection, while staying shorter than SEC-25's advisory times | Fresh malicious releases | P1 | none | SEC-14 |
| Vendored dependencies for release builds (13.2, 7.1); release builds use no `sccache` or other cache (amends Stage 1 7.1) | Registry games; cache poisoning | P1 | 1 day | SEC-13 |
| Reproducible V8 archive: GitHub-hosted runners and one independent builder produce identical hashes before a normal tag is published; emergency releases follow UPD-R4; the browser binary follows in Stage 2 ([Tor](https://blog.torproject.org/deterministic-builds-part-two-technical-details); [Chromium](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/deterministic_builds.md)) | SUNSPOT-class builder compromise; xz-class source mismatch | P1 / P2 | 1–2 weeks | SEC-15 |
| OS code signing (Apple Developer ID and notarization; Windows signing in a dedicated, environment-protected job); certificates rotate at least yearly† under the 460-day limit (Low confidence) | Spoofed binaries; key theft | P1 | fees | SEC-16 |
| Izcal's own TUF client (`hg-update`) written from the TUF specification, with the roles of UPD-R3; pinned initial root; timestamp expiry 3 days†; consistent snapshots; an "updates are stale" UI state; verify-then-apply on a locked file ([TUF](https://theupdateframework.github.io/specification/latest/); [MFSA 2013-83](https://www.mozilla.org/security/advisories/mfsa2013-83/)) | Update hijack, rollback, freeze, key compromise (Notepad++) | P1 | 4–8 weeks | SEC-17 |
| Update metadata and binaries served from storage Izcal does not trust (GitHub or a CDN), with TUF as the only integrity root | Hosting compromise | P1 | design | SEC-17 |
| Key-rotation drills for TUF and code-signing keys twice a year†; alerts on key age | Silent key decay (Storm-0558) | P1 | 1 day per drill | SEC-16 |
| OpenSSF Scorecard alongside zizmor and actionlint | Dangerous workflow patterns returning | P1 | 0.5 day | SEC-27 |
| Binary transparency: release hashes and attestations in a public log, and the updater checks inclusion | Targeted malicious updates to a few users | P2 | 1–2 weeks | — |

**Update rules.**
- **UPD-R1. Every data channel is a TUF target.** Browser updates, content-blocking lists, CRLs, the GPU blocklist, a future phishing list, extension updates and the emergency configuration are TUF targets with expiry, under SEC-17's tests. Nothing the browser fetches to change its behavior bypasses TUF.
- **UPD-R2. Fetch, verify and apply are separate.** The network service downloads; the broker routes the bytes as opaque data (IPC-R16 entry 3) and parses no TUF metadata; the **updater** process runs `hg-update`'s verification and applies the update, with no network and write access only to the install directory. Its privilege follows the install model (SEC-D14: per-user, no elevation, recommended). This answers Q-SEC-6.
- **UPD-R3. TUF roles are split for speed and safety.** **Root** is offline, threshold-held (2-of-3† hardware keys in separate places). **Targets** delegates release signing to a **`releases`** role whose hardware keys release managers carry, so a release can be signed within hours. **Timestamp and snapshot** keys are online. Until SEC-D4 is met, all these keys are held by one person, so the thresholds protect against key loss, not against compromise or coercion of that person.
- **UPD-R4. Emergency release class.** A security release may publish with a verified build-provenance attestation (SEC-13 (b)) while the reproducibility comparison (SEC-15) runs in parallel; the comparison must be green within 7 days†, or the release is re-issued from a build that passes. Each use is recorded in the security-response log and the quarterly report. Rationale for 7 days†: the same as SEC-1's critical bound, so a release is never unverified for longer than the next bound would allow.
- **UPD-R5. Emergency mitigation channel** (SEC-D11). A TUF-signed configuration, signed by the `releases` role, can switch on only a closed list of security switches: jitless renderers, WebAssembly off, WebGPU off, one named codec off, DrumBrake off. Each switch carries an expiry, is shown on `izcal://sandbox`, and never sends data or changes privacy settings.

**Single-account governance.** With one maintainer and self-merge allowed, one stolen GitHub session controls source, CI and releases. Server-side rules and hardware keys reduce that risk; they do not remove it, because the same person holds every approval. The options and a recommendation are SEC-D4.

**Build-our-own applied.** The TUF client is built by Izcal from the TUF specification, using audited cryptographic primitives as dependencies, because the standing answer to build-or-reuse is to build following the specs. Its correctness oracle is a test vector for every attack class the specification names, plus a black-box differential against an existing TUF implementation's behavior on the same metadata. This costs more than using a TUF library as a dependency (estimated 3–6 weeks with a library, research notes); the difference is in the estimate above.

### 6.17 Security response and AI-scale discovery

Stage 1 13.2 commits to a disclosure policy, advisories, an embargo process and a later bug bounty. The record adds a volume problem: AI models found 22 Firefox vulnerabilities in two weeks and then 271 in one release; Chrome cut its base rewards in May 2026 ([SecurityWeek](https://www.securityweek.com/google-adjusts-bug-bounties-chrome-payouts-drop-as-android-rewards-rise-amid-ai-surge/)) and restructured them in October 2026 ([Help Net Security](https://www.helpnetsecurity.com/?p=387768)), as AI tools raised report volume (secondary reports). A young engine should expect a high early advisory rate. Design:
- **Pre-release audits** (SEC-24). Before each Beta and Release milestone, an AI-assisted audit and a fuzzing campaign run over every Izcal crate changed since the last audit, with findings triaged before release; an external audit runs before the first public binary.
- **Triage capacity** (SEC-25). Acknowledgement and fix times by severity, published, with an exploited-in-the-wild tier at SEC-1's 72 hours† for **every** shipped component; the security track (Section 9.2) staffs triage and the on-call rotation.
- **Variant analysis.** Each fixed security bug gets a search for its variants across Izcal's code, because 25–40% of in-the-wild 0-days were variants (Section 3.1).
- **A security regression suite** that carries a test for every fixed security bug and every historical CVE class in each component's area, and that is never disabled to speed up CI (the Stylo lesson). The suite is part of the D7 equivalence gate, so a rewritten component cannot pass equivalence while failing it.
- **Disclosure policy** published before the first public binary, with a security contact, an embargo process for upstream fixes, and advisories in a machine-readable format.

### 6.18 Alternatives

| Question | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|
| V8 patch latency | (a) Keep D2's one cycle with 7.7's lag metric; (b) severity-tiered targets on a Chrome-patched branch (SEC-1) | **(b)** | A rehearsed backport path, a two-person on-call rotation, and self-rebases when Deno lags | Two quarters† where drills show (b) cannot be met without skipping T2: restate the thresholds in the decision log with the measured pipeline time |
| Consent integrity | (a) Shell draws and decides; (b) broker hit-tests OS input; (c) (b) plus a separate prompt renderer; (d) native prompts | **(c)** (CON-R1, CON-R2) | One more process kind; input routing through the broker | The prompt renderer misses the shell UI latency targets: draw credential prompts natively (d) |
| IPC verification depth | (a) Kani per message, Stage 1 4.1's choice under VER-1; (b) Vest-class proofs (Vest emits Rust) or EverParse-class proofs (EverParse emits Rust only through the experimental EverCDDL; 3D emits C) for renderer-to-broker messages | **(a) in Stage 1, (b) as a Stage A stretch**, as Stage 1 4.1 already states; VER-1 allows both ([Vest](https://www.usenix.org/conference/usenixsecurity25/presentation/cai-yi); [EverParse](https://project-everest.github.io/everparse/)) | (b) adds a format-description language to the toolchain | A parser bug Kani's bounds missed |
| Windows renderer JIT | (a) JIT without ACG and CET (Standard) plus a jitless tier with ACG and CET; (b) out-of-process JIT; (c) jitless everywhere | **(a)** | Jitless is about 40% slower on Speedometer 2 and about 80% slower on the Web Tooling Benchmark ([V8 jitless](https://v8.dev/blog/jitless)), and compiler bugs are about half of exploited V8 bugs ([V8](https://v8.dev/blog/sandbox)) | V8 offers an out-of-process JIT through public interfaces |
| macOS sandbox backend | (a) Seatbelt SBPL; (b) Enhanced Security helper extensions; (c) App Sandbox | **(a) with (b) evaluated from M1** | Two backends for a time | SDK 27 kills custom profiles: (b) becomes primary where it fits, and a reduced tier is an owner decision where it does not |
| C codec isolation | (a) Separate utility processes; (b) RLBox-style wasm2c in a utility process; (c) Rust replacements | **(c) for images and fonts, (a) or (b) for media codecs per measured cost** | Process memory or a wasm2c slowdown | A codec whose isolated cost breaks playback on a reference device: record the exception, time-bounded |
| IPC fuzzing platform | (a) Coverage-guided `cargo-fuzz` per message; (b) adversarial simulation with sequences | **Both**, measured against each other on a planted-bug benchmark (SEC-5) | DST CPU-hours | If (b) finds fewer planted bugs than (a) for two quarters†, keep it at regression level, as DST-1 does |
| Emergency path | (a) Normal release only; (b) emergency release class (UPD-R4); (c) (b) plus a mitigation channel (UPD-R5) | **(c)**, with the channel as SEC-D11 | A delegated signing role; a closed switch list | — (decided in SEC-D11) |
| TUF client | (a) Izcal's own from the specification; (b) a maintained TUF library as a dependency | **(a)**, the standing build-our-own answer | 1–2 extra weeks (estimate) | A security bug in `hg-update` that a library would not have had, found before release: review whether (b) is safer for this component, by recorded decision |
| Release governance | (a) Second maintainer with release authority; (b) single maintainer with compensating controls | **(a) before the first public binary; (b) until then** | Funding (D-S8) | Decided in SEC-D4 |

---

## 7. M-1 compliance

**Dependencies used unmodified, through their public interfaces.** V8 (through the `rusty_v8` fork; security backports are maintenance of a dependency, which M-1 allows, S-4e), rustls, rustls-platform-verifier, neqo, hickory-dns, wgpu-core and Naga, WebRender, ANGLE (Windows), dav1d, libvpx, Opus, Vorbis, FLAC, Cisco's OpenH264 binary, SQLite, Wasmtime (Stage 2), the wasm2c translator, `adblock`, AccessKit, the Windows PlayReady CDM through Media Foundation's documented APIs; tools: cargo-deny, cargo-vet, zizmor, actionlint, gitleaks, Kani, `cargo-fuzz`, Fuzzilli, GitHub artifact attestations, Renovate or Dependabot. Cryptographic primitives for `hg-update` and the WebRTC stack come from audited, non-browser crates.

**Built new by Izcal, following its specifications** (the standing answer to build-versus-reuse; none of these is an owner question).
- `hg-ipc` handle validation and the IPC-R generator rules: derived from I-2 and B2 and from OS documentation (unix(7) and `SCM_RIGHTS`, `memfd_create(2)` seals, Mach ports, Win32 `DuplicateHandle`, `NtQueryObject`); Mojo's security notes, Mozilla's IPC write-ups and the CVE analyses are cited as evidence only.
- The consent path (CON-R1 to CON-R5) and the prompt renderer: from D-S2, UI-5, UI-10 and OS input documentation.
- `hg-sandbox` backends: written from OS documentation (Linux seccomp and Landlock documentation, Apple's `sandbox_init(3)` and `sandbox(7)` manual pages and Hardened Runtime and Enhanced Security documentation, Microsoft's AppContainer, job object and process-mitigation documentation). **SBPL is the hard case**: Apple does not document it, so Izcal writes its profiles from the manual pages and its own black-box tests and never reads or adapts another browser's `.sb` profile files.
- The adversarial-peer mode of `hg-dst`, the per-OS transport fuzzer, the surface inventory generator, the Rust↔V8 `External` wrapper.
- The **RLBox-style tainted-type wrapper** for wasm2c-sandboxed libraries: RLBox itself is Firefox's C++ framework, so Izcal writes its own Rust wrapper from the idea, not from RLBox's code.
- The **PDF engine**: in Rust, from ISO 32000. PDF.js and PDFium are browser components and are not used.
- The **WebRTC stack**: in Rust, from the IETF RFCs (ICE, STUN, DTLS-SRTP, RTP and RTCP, SCTP over DTLS) and the W3C WebRTC specification. libwebrtc is a browser component and is not used.
- The **crash handler and minidump writer**: from Microsoft's minidump format documentation and the OS crash APIs. Crashpad and Breakpad are Chromium projects and are not used.
- The **GPU driver blocklist**: from Izcal's own device testing and vendor advisories, not from another browser's blocklist data.
- **ORB** response filtering: from the Fetch standard.
- `hg-update`: from the TUF specification.
- Importers: from black-box observations of profile formats in `compat/observations/` (13.2).
- Threat models, the security regression suite and the audit process.
- If SEC-D7 chooses one: the phishing-protection client, from a published protocol or Izcal's own specification.

**What each part is anchored to.** IPC-R rules to I-2, B2 and VER-1; sandbox policy to OS documentation; the network filtering and partitioning to the Fetch standard and the Privacy CG storage-partitioning work (I-1(b)); WebRTC to the IETF and W3C specifications; EME and the CDM path to the W3C Encrypted Media Extensions specification and Microsoft's Media Foundation documentation (Stage 1 13.1a); the updater to the TUF specification; release provenance to SLSA v1.2 ([SLSA](https://slsa.dev/spec/v1.2/whats-new)); automation surfaces to the WebDriver, BiDi, DAP and MCP specifications (DevTools spec Section 7). Other browsers' design documents and postmortems are cited as evidence in Sections 2–6; none is a template.

---

## 8. Targets

† marks thresholds this document sets from first principles; none is measured. "Release" means an Izcal Stable release available through the update service. Existing targets keep their IDs; these are new. SEC-1 to SEC-28 keep their 0.1 subjects; Appendix B lists what each 0.2 change adds. The coding standard cites SEC-3, SEC-4, SEC-7 and SEC-11, whose subjects are unchanged.

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **SEC-1** | **V8 security patch latency** | Time zero is the **earliest** of the first public V8 commit containing the fix (found afterwards from the fix commit's history), the first Chrome stable release containing it, and public disclosure. Time from it to an Izcal release containing the fix: **≤ 72 h†** for a bug known to be exploited in the wild; **≤ 7 days†** for critical or high severity; **≤ one Chrome stable cycle** (D2) for the rest. The pinned V8 is on a branch Chrome still patches on **100%** of days. Published per advisory, per quarter | Security-response log; time zero computed by a script over upstream history, not entered by hand; daily V8-bot check of the pinned branch; quarterly report | Each miss gets a postmortem. A missed exploited-bug window triggers the emergency mitigation channel (UPD-R5) for all users until the fix ships. A pinned branch outside Chrome's patched set for more than 7 days† is a ship blocker. Two misses in a rolling six months† escalate to the D2 review and the staffing review |
| **SEC-2** | **Backport path is rehearsed** | Each quarter†, a chosen V8 security commit is cherry-picked onto the pinned branch and taken through fork build **on the actual release runners** (three platforms, release and debug sandbox), T0 to T2, signing by the `releases` role and TUF publication to a staging channel as an emergency release, plus one emergency-configuration push to staging, in **≤ 48 h†** of elapsed time; per-stage times recorded | Drill log with per-stage times | The stage that exceeded its share is fixed before the next Chrome stable release; SEC-1 cannot be claimed while the drill fails |
| **SEC-3** | **IPC identity comes from bound endpoints** | (a) Only `hg-ipc` can construct `CallerIdentity` (type rule), and no authorization API accepts `TargetOrigin`, `NavigationUrl` or another data type (lint), on **every** edge, service to service included; (b) **100%** of `bound` endpoints are refused by the generator and the transport when sent; the broker creates one bound endpoint per frame and per worker (property test); (c) for **each** sandboxed process kind, a compromised-peer suite that tries to act as another (site, container), frame or process through every endpoint it holds, including forwarded endpoints, is blocked **100%**; (d) frame, request, resource and object IDs are never reused (property test over generations) | T0 type rule and lints; T1 suite in simulation | Ship blocker |
| **SEC-4** | **Handles are typed and only narrowed** | (a) Each OS transport rejects **100%** of handles of an undeclared kind, pseudo-handles and sentinel values, and writable mappings offered as the read-only kind, in a per-OS test set; on Windows the kind is determined from the object and onward duplication uses an explicit narrowed mask (test); (b) a Kani harness per OS backend proves the acceptance function rejects every undeclared kind within stated bounds; (c) property test: no broker operation returns a handle with rights wider than the one it was derived from, **100%**; (d) only the `hg-ipc` backend crates may call `DuplicateHandle`, `sendmsg` with `SCM_RIGHTS` or `mach_msg` with port descriptors (crate-graph lint, **0** violations) | T0 Kani, property tests and graph lint; T1 per-OS test set | Ship blocker |
| **SEC-5** | **Simulation acts as a compromised peer** | (a) T0 rule: no endpoint merges without a generated sequence fuzzer, run under the simulator or `cargo-fuzz`; **100%** of endpoints covered at M6 and at every later milestone; the adversarial-peer mode covers every sandboxed process kind, network service and GPU service first, with endpoint-closure faults against every in-flight call; (b) a planted-bug benchmark (pseudo-handle relay, identity from the message, closure during a pending reply, double fetch, forwarded bound endpoint) measures time-to-find for the simulator and for `cargo-fuzz`, published quarterly†; (c) a per-OS multi-process transport fuzzer for handle passing runs nightly | T0 merge check; T2 nightly in the DST farm; shared bug database (Stage 1 7.4) | (a) blocks the milestone. (b) simulator slower than `cargo-fuzz` on the benchmark for two quarters†: keep the mode at regression level (as DST-1); never drop IPC fuzzing |
| **SEC-6** | **Attack surface is an artifact** | **100%** of pull requests that change a process type's reachable interfaces, sync methods, sendable handle kinds, bound or free endpoints, or IPC-R16 payloads show the generated diff and carry the security owner's approval (or a recorded self-approval, Section 9.2) | T0 check on the generated inventory | The change cannot merge |
| **SEC-7** | **Receivers never panic on input** | (a) Kani "no panic on any byte string" for every generated message parser (extends VER-1); (b) broker and service handler crates deny `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` and `arithmetic_side_effects` with a reviewed allowlist (`tools/coding-standard/tier-p-exceptions.toml`), **0** unlisted uses; the panic strategy per process is stated in its threat model; (c) **0** broker or service crashes attributable to message content in T2 fuzzing, open for more than 14 days† | T0 harnesses and lints; T2 crash triage | Ship blocker for the message type; fix within 14 days† |
| **SEC-8** | **Sandbox tier is achieved and visible** | On the four reference devices, the renderer, shell renderer, prompt renderer and GPU service reach the Section 6.7 floor, **100%**, **with no waiver**; other process types and non-reference configurations (Flatpak, kernels without namespaces) reach the floor or report a named reduced tier on `izcal://sandbox` and in local logs, **100%**; policy tests like S-2's for every sandboxed process type; on macOS, a recorded SDK 27 result for custom profiles by M1; the number of waivers and affected platforms published per release | T1 policy tests per OS; startup probe; release report | A reference-device floor missed: ship blocker for that platform; a forced reduced renderer tier on macOS goes to SEC-D6, not the decision log |
| **SEC-9** | **Kernel surface is allowlisted** | On Linux, every sandboxed process type runs a seccomp allowlist with an architecture check; attempts to call `io_uring_setup`, `bpf`, `userfaultfd`, `keyctl`, `ptrace`, `perf_event_open` or create namespaces after setup fail in **100%** of runs; jitless processes fail `PROT_EXEC` mappings, **100%**; a new syscall in an allowlist needs security-owner review (extends S-2 beyond renderers) | T1 policy tests | Ship blocker |
| **SEC-10** | **GPU service is contained and fuzzed** | (a) Sandboxed on Windows, macOS and Linux at the Section 6.7 tier, built at M3–M4 and gated at M8, with policy tests (no files beyond the render node, no network) passing **100%**; (b) WebGPU: wgpu-core validation plus per-renderer ownership checks and limit clamps, passing **100%** of the W3C WebGPU conformance suite's validation tests in the GPU service; WebGL: **100%** of the Khronos WebGL conformance suite's negative tests rejected in the GPU service before ANGLE or a GL driver; (c) WebGL call-sequence, WGSL shader and display-list fuzzers run nightly, with **0** crash bugs in the GPU service older than 30 days†; display lists capped per frame (IPC-R16); (d) Naga translation differential: generated HLSL, MSL and SPIR-V keep every bounds check on a published corpus, **100%**; (e) driver blocklist updates ship within 7 days† of a driver advisory | T1 tests; T2 fuzzing; release report | (a) or (b) missing on a platform: WebGPU ships disabled there and Stage 1 exit is blocked for that platform; (c) to (e): fix before the next release |
| **SEC-11** | **Rust↔V8 boundary rules hold** | (a) **0** uses of raw `External` outside the typed wrapper (lint, FFI-R1); (b) a published suite resizes, detaches and transfers ArrayBuffers during every async binding and every callback into script, and passes **100%** under debug V8 and ASan (FFI-R2); (c) Fuzzilli through the renderer's REPRL mode runs nightly with **0** open crash bugs older than 30 days† (FFI-R4) | T0 lint; T1 suite; T2 fuzzing | Ship blocker for the binding |
| **SEC-12** | **Stage 1's library sandbox is a Stage 1 target** | At M7, **100%** of C and C++ decoders, the OpenH264 binary and the PDF path that handle untrusted input run in a utility process at the strictest tier or in RLBox-style wasm2c inside one (S-5's metric, applied to Stage 1's existing design); images and fonts use Rust decoders; the PDF path generates no code from document data; OS-framework decoders are listed with their placement | Inventory check in T1; policy tests | Ship blocker for the decoder; a time-bounded exception needs a decision-log record |
| **SEC-13** | **The artifact chain is verifiable** | (a) **100%** of builds consuming a V8 archive verify the archive and binding hashes committed in the project repository, and fail on mismatch; (b) **100%** of release assets carry a build-provenance attestation that verifies against the reusable workflow's identity; (c) **100%** of releases are immutable; (d) **0** release or publish jobs restore a cache; (e) **100%** of `uses:` references are full SHAs; (f) **100%** of downloads in the release build (`tools/win`, LLVM, toolchains V8's scripts fetch) are pinned by hash or exact version | T0 check on workflows; build-time verification; release checklist script | Ship blocker for the release |
| **SEC-14** | **Every dependency is reviewed** | (a) cargo-deny advisories, bans, licenses and sources pass in CI; (b) cargo-vet passes, and its exemption list is shorter at each milestone than at the previous one; (c) **0** crate versions younger than 7 days† without a recorded reason; (d) **0** new crates without a recorded human decision | T0 | The change cannot merge |
| **SEC-15** | **V8 archive is reproducible** | From M7, each normal fork tag's release archives are built on GitHub-hosted runners and on one independent builder with **identical** hashes per platform, and the comparison is published with the release; an emergency release (UPD-R4) publishes with a verified attestation and its comparison is green within 7 days† | Release job plus independent rebuild | Normal tag: not published until the difference is explained and fixed. Emergency release: re-issued from a passing build |
| **SEC-16** | **Releases are signed and keys are kept** | (a) **100%** of release packages are OS-signed (and notarized on macOS); (b) TUF root keys are offline and threshold-held (2-of-3† hardware keys in separate places); targets delegates to a `releases` role on carried hardware keys; timestamp and snapshot keys are online (UPD-R3); (c) a rotation drill for TUF and code-signing keys passes twice a year†; (d) key-age alerts exist for every key | Release checklist; drill log | No release until fixed |
| **SEC-17** | **The update client and every data channel resist the TUF attack classes** | (a) `hg-update` rejects **100%** of a test-vector suite covering the TUF specification's attacks (arbitrary installation, endless data, extraneous dependencies, fast-forward, indefinite freeze, malicious mirrors preventing updates, mix-and-match, rollback, wrong software) and its key-compromise resilience goal below threshold; (b) timestamp expiry 3 days†; stale state shown to the user; verify-then-apply on a locked file; (c) **100%** of data channels (UPD-R1) are TUF targets with expiry; (d) verification runs in the updater process, and the broker parses **0** TUF metadata (UPD-R2) | T1 suite; fuzzing of metadata parsing; channel inventory | Ship blocker for the updater or the channel |
| **SEC-18** | **Agent entry points, memory and schemes are closed** | With AI on: **0** tasks created, extended or re-targeted from a URL, deep link, query parameter, address-bar fallback, notification or page script; **0** writes to persistent agent memory from a web request; **100%** of a published scheme-bypass suite (`file:`, `view-source:file:`, fragment-suffix tricks, `blob:`, `data:`, internal and extension pages) refused on parsed URLs in the broker; **0** extension messages reach the agent host or AI UI (AGT-R1 to AGT-R5). Added to AG-3's suite, run with model defenses off | Suite per release once the agent host exists | Ship blocker for the agent principal |
| **SEC-19** | **Rule-of-Two task policy** | (a) In a published task suite, **100%** of writes to a sensitive-class read-write origin, in tasks that have read untrusted input (outside the writable set, or third-party-writable content inside a sensitive origin, AGT-R6), require per-action confirmation through CON-R1; (b) the median confirmations per task on a published task corpus is published per release and stays **≤ 3†** | Suite and corpus per release | (a) ship blocker for the agent principal; (b) above the ceiling: redesign the policy or the task UI before the next release |
| **SEC-20** | **Automation cannot be borrowed by local malware** | **0** command-line flags, environment variables or project files that attach a session to a user container; automation consent goes through CON-R1 and survives a published synthetic-input suite on each OS where the OS offers a defense (extends DT-11 and DT-13); as defense in depth, **100%** of automation launches against a profile directory held by another Izcal instance refused | Red-team suite each release from M6 | Ship blocker for the affected surface |
| **SEC-21** | **Recordings are contained** | (a) **100%** of imported bundles replay under an opaque origin with no network, no user container and no write capability (REC-R1); (b) the bundle parser has a fuzz target in T2 with **0** open crash bugs older than 30 days†; (c) saved bundles are encrypted under SEC-34 and expire after 30 days† unless pinned (REC-R2, REC-R3); (d) exported bundles quantize inter-keystroke intervals in text-entry fields to 100 ms† buckets by default and carry the divergence label (REC-R4); (e) **100%** of replays run at the recorded engine configuration (REC-R6) | T1 tests; T2 fuzzing | Ship blocker for recording import or save |
| **SEC-22** | **Protected-media host has no bridge** | **0** host objects, web-message handlers and injected scripts in the host (checked by an API-call audit test); the host runs outside the broker; **100%** of runtime updates restart the host before the next protected session; protected media mode refuses runtimes below the published minimum version, **100%** (SH-R4) | T1 test on Windows and macOS; release check | Protected media mode is disabled for the release on that OS |
| **SEC-23** | **Credential UI is out of the page's reach** | **100%** of autofill and passkey UI is drawn outside the page DOM, after a CON-R1 approval (property test); the importer imports **0** cookies and reads **0** App-Bound-protected data directly; Chrome passwords only from a user-exported CSV (SH-R1, SH-R2) | T1 property test; importer test | Ship blocker for the password manager or importer |
| **SEC-24** | **Pre-release audit** | Before each Beta and Release milestone: an AI-assisted audit and a fuzzing campaign over every Izcal crate changed since the last audit; **100%** of critical and high findings fixed or formally waived before release; an external audit before the first public binary; the security regression suite has **0** disabled tests and is part of the D7 equivalence gate | Audit report per milestone | The milestone does not pass |
| **SEC-25** | **Advisories are handled on time** | Acknowledge external reports within 3 days†; a bug known to be exploited in the wild in **any** shipped component (Izcal's code, crates, C and C++ libraries, vendor binaries such as OpenH264 and the PlayReady CDM where Izcal ships the fix path) within 72 h†; critical issues in Izcal's own code within 14 days† and high within 30 days†; upstream advisories against **every shipped dependency** (advisory sources listed per dependency in the Stage 1 7.7 table): critical or high within 7 days†, others in the next release; published per quarter | Security-response log | A miss gets a postmortem; repeated misses trigger the staffing review |
| **SEC-26** | **Threat models are current** | All nineteen files (Section 6.3a) exist at M1; broker, renderer and network service filled at M1; **100%** filled by M6 (the Stage 1 13.2 requirement); **100%** of pull requests that change a process's endpoint set or sandbox policy update its threat model; each model is reviewed at every milestone | T0 check (inventory diff without a threat-model change fails); milestone review | M6 does not pass |
| **SEC-27** | **The pipeline resists account and CI compromise** | A checklist, run monthly† and at every release: rulesets on `main`, `hgba` and release tags enforced server-side; signed release tags; maintainer accounts on hardware keys only; no long-lived tokens; required checks run from the base branch; no self-hosted runner on `pull_request`; no CI agent with a shell on untrusted input; builds run without secrets; SHA-pinning policy and a pin updater on; OpenSSF Scorecard checks Pinned-Dependencies, Dangerous-Workflow, Token-Permissions and Binary-Artifacts score **10**, with Code-Review and Branch-Protection listed as known exceptions until SEC-D4 is met; zizmor reports **no findings of medium or higher**† | Checklist script and Scorecard | Release blocked until the failing item is fixed |
| **SEC-28** | **Network parsers degrade, not crash** | **0** panics in the network service reachable from network input in T2 fuzzing (HTTP/1.1, HTTP/2, HTTP/3, TLS, DNS, WebRTC transport), open for more than 14 days†; every pre-authentication allocation bounded by a published cap; no panic reachable from network input | T2 fuzzing; cap review | Fix before the next release |
| **SEC-29** | **Consent is decided by the broker** | (a) A compromised-shell harness and a compromised-prompt-renderer harness in the simulator try to approve every pending prompt type without broker-observed input: **0** succeed; (b) **100%** of security prompt types are drawn by the prompt renderer, which receives **0** page-derived strings except the broker-formatted origin (surface inventory check); (c) **0** endpoints to the shell carry a password or passkey secret type (CON-R4); (d) **100%** of approvals within 500 ms† of a prompt appearing, moving or changing are rejected (CON-R5) | T1 harnesses; T0 inventory check | Ship blocker for the affected prompt type |
| **SEC-30** | **WebRTC is contained and fuzzed** | (a) Transport in the network service, media in utility processes, UDP sockets only as broker-minted handles, **0** WebRTC parsers in the broker (inventory); (b) fuzz targets for RTP, RTCP, SCTP, STUN and DTLS in T2 with **0** crash bugs older than 30 days†; (c) **100%** of host candidates exposed as mDNS names without a camera or microphone grant (test suite); (d) WebRTC dependencies listed under SEC-25 | T0 inventory; T1 suite; T2 fuzzing | WebRTC disabled for the release on the affected platform |
| **SEC-31** | **Local network and loopback are guarded** | **100%** of a published Local Network Access and DNS-rebinding suite (private ranges, loopback, rebinding after resolution, the DevTools loopback listener) blocked without a grant for that (site, container) (NET-R7) | T1 suite per release | Ship blocker for the network service release |
| **SEC-32** | **Downloads, external protocols and file handles are controlled** | (a) **100%** of downloaded files carry Mark-of-the-Web on Windows and `com.apple.quarantine` on macOS (SH-R6); (b) **0** automatic opens of dangerous types (SH-R7); (c) **0** external protocol launches without a gesture and a CON-R1 approval, and **0** from cross-origin iframes (SH-R8); (d) **100%** of File System Access handles are typed, broker-minted and limited to the grant (SH-R9) | T1 suite per OS | Ship blocker for downloads, protocol handling or File System Access |
| **SEC-33** | **The native DRM path is contained** | **0** CDM loads into the broker or a renderer; **100%** of EME messages to the CDM pass generated decoders; the CDM process runs at the tier recorded for it at M7; its threat model names the OS processes involved (Q-SEC-17) | T1 test on Windows; inventory | Native PlayReady disabled for the release; protected media mode remains |
| **SEC-34** | **Secrets at rest use an OS-bound key where the OS has one** | **100%** of cookies at rest, saved recordings and password-manager entries are encrypted under a key the renderer never holds: on macOS a Keychain item whose access is bound to Izcal's code signature; on Windows DPAPI user scope under the per-user install of SEC-D14 (same-user code can decrypt it, stated as residual); on Linux Secret Service (user-scoped, residual stated) | T1 test per OS | Ship blocker for the storage that fails |
| **SEC-35** | **Crash dumps are filtered and expire** | **100%** of renderer, shell, prompt, pane and replay-host dumps use filtered stacks; **100%** of dumps are deleted after 30 days†; **0** recordings attached (SH-R5) | T1 test; server retention check | Crash upload disabled until fixed |

Phishing protection gets a target once SEC-D7 is decided.

**Rationale for the † thresholds** (one line each; all are to be revisited with measured data).

| Threshold | Where | Reasoning |
|---|---|---|
| 72 h† | SEC-1, SEC-25 | Two working days plus a weekend margin, matching Mozilla's demonstrated 21–25 hours with room for a three-platform V8 build |
| 7 days† | SEC-1 (critical/high, branch), SEC-10 (e), SEC-15, UPD-R4 | One week: short against a four-week Chrome stable cycle |
| 48 h† | SEC-2 | Leaves a day of the 72-hour bound for triage and publication |
| 24 h† | V8 bot (Stage 1 7.7) | Unchanged from Stage 1 |
| 14 days† | SEC-7, SEC-25, SEC-28 | Two weeks: one fix cycle for a crash a fuzzer can reproduce |
| 30 days† | SEC-10, SEC-11, SEC-21, SEC-25, SEC-30, REC-R3, SH-R5 | One release cycle for fuzz crashes; one support cycle for recordings and dumps |
| 3 days† | SEC-25 acknowledgement; SEC-17 timestamp expiry | Acknowledgement inside a working week; a freeze cannot hide an emergency release longer than SEC-1's 72-hour bound |
| 2-of-3† | SEC-16 | The smallest threshold that survives the loss of one key |
| Twice a year†; monthly†; each quarter† | SEC-16, SEC-27, SEC-2, SEC-5 | Often enough that the procedure is still remembered by the person who runs it |
| Two misses in six months†; two quarters† | SEC-1, SEC-5, SEC-D1 | One miss can be bad luck; two in the same window is a pattern |
| 500 ms† | CON-R1, CON-R5 | Longer than an input aimed at page content takes to land; shorter than deliberate reaction |
| 100 ms† | REC-R4 | Placeholder until Q-SEC-19's measurement; near typical inter-key intervals |
| ≤ 3† confirmations | SEC-19 (b) | A task that asks more than about three times trains users to click through |
| 5%† | SEC-D5 | PRIV-1's style of corpus-breakage tolerance |
| 7-day cooldown† | SEC-14 (c) | arrayref was removed within about two hours; seven days covers slower detection with margin |
| Under one engineer† | SEC-D2 | Below one engineer the track is not a track |

---

## 9. Milestones and staffing

### 9.1 How the work fits Stage 1's milestones

| Milestone | Security work (exit criterion added) |
|---|---|
| **M0** (start conditions) | SEC-13 (a), (c), (f): archive and binding hashes committed and verified, immutable releases, `tools/win` and LLVM pinned; SEC-13 (d), (e): done in the project (`main`), **pending fork PR #2 merge** for the fork; SEC-27 P0 items: rulesets, hardware keys, checks from the base branch, no self-hosted runners on `pull_request`, SHA-pinning policy and pin updater; SEC-14 (a); the V8 bot's daily patched-branch check (SEC-1, SEC-D12); `rusty_v8` owner, a named backup and the security owner named; threat-model template adopted (this document) |
| **M1** (substrate and IPC) | IPC-R rules in `hg-ipc-codegen` and the transports, including bound and free endpoints, `CallerIdentity` and ID generations (SEC-3 (a), (b), (d); SEC-4 (a), (b), (d); SEC-6; SEC-7 (a), (b)); SEC-5 (a)'s merge rule; adversarial-peer mode skeleton; broker-owned input routing designed (CON-R3); per-OS sandbox probe and `izcal://sandbox`; **macOS SDK 27 measurement** (SEC-8); all nineteen threat-model files created, broker, renderer and network service filled (SEC-26) |
| **M2** (DOM and bindings) | FFI-R1 to FFI-R4 (SEC-11); first SEC-2 drill, on the release runners; first planted-bug benchmark (SEC-5 (b)); cargo-vet started (SEC-14 (b)) |
| **M3–M4** (style, layout, pixels) | GPU service sandbox, ownership checks and clamps built (SEC-10 (a), (b)); GPU and display-list fuzzers start (SEC-10 (c), (d)) |
| **M5** (network, navigation, Windows) | Network service sandbox per SEC-D13 (SEC-8); SEC-28; SEC-31; Windows tiers including the jitless ACG and CET tier and the jitless Izcal-only renderers (SEC-D5); the Windows renderer token combination and the broker's CIG versus IME and assistive-technology compatibility measured; per-OS transport fuzzer (SEC-5 (c)) |
| **M6** (interaction) | **All threat models complete** (SEC-26, Stage 1 13.2); SEC-5 (a) at 100%; SEC-3 (c) for every sandboxed process kind; **prompt renderer and broker-decided consent (SEC-29)**; SEC-20 and DT-13's CDP, BiDi and DAP parts |
| **M7** (breadth) | SEC-12 (library sandbox gate); SEC-22 (DRM host rules); SEC-33 (native PlayReady); SEC-30 (WebRTC); SEC-23 (password manager, importer); SEC-32 (downloads, protocols, File System Access); SEC-34 (secrets at rest); SEC-35 (crash dumps); SEC-17 (`hg-update`, the updater process, data channels); SEC-16 (roles, signing, first drill); SEC-15 (reproducible V8 archive, emergency class); SEC-21 (recordings); "which layer stops which class" table published; first SEC-24 audit; disclosure policy published |
| **M8** (Stage 1 exit) | Every SEC target met or formally waived in the decision log (SEC-8's reference-device floors cannot be waived); SEC-10 gated; SEC-1 measured over at least one quarter; the emergency mitigation channel (SEC-D11) drilled in SEC-2; external audit before the first public binary (SEC-24); SEC-D4 governance condition met; SEC-D7's shelved licensing done before the first public binary (URLhaus commercial licence, a global phishing feed, the URLAbuse account, PhishTank's answer), or malware-URL protection ships as a stated gap |

SEC-18 and SEC-19 apply when the agent principal ships (Stage 2 / Stage C). The tests are written in Stage 1 so that the broker's B9 enforcement is built against them.

### 9.2 What the work costs

The 23–30.5† engineers of Stage 1 design 8.3 include a **Network and security** track of 2 engineers covering the network service, per-platform sandboxing and VER-1, and the DevTools work in other tracks includes 8 engineer-months in that track for `hg-net-observe`, qlog and four DevTools threat models. The Stage 1 design refers to a "security owner" (4.1, 7.7) but does not staff one. The work this spec adds does not fit inside those rows. Engine work this spec depends on but does not cost (Izcal's WebGL implementation, the WebRTC stack, the PDF engine, the prompt renderer's UI) belongs to the owning tracks.

| Stage 1 security work beyond the existing 8.3 rows | Engineer-months† |
|---|---|
| Threat models for nineteen kinds and components, kept current (beyond the DevTools spec's four) | 7 |
| IPC-R generator rules, bound endpoints, transport handle validation, per-backend Kani harnesses | 5 |
| Adversarial-peer simulation for every process kind, planted-bug benchmark, per-OS transport fuzzer (SEC-5) | 12 |
| Consent path: broker hit-testing, prompt-renderer security, compromised-shell harness (SEC-29) | 4 |
| Sandbox work beyond the existing track: macOS SDK 27 measurement and extension backend evaluation, Windows tiers, Linux allowlists and reduced-tier reporting | 8 |
| V8 patch pipeline, drills and self-rebases when Deno lags (SEC-1, SEC-2) | 7 |
| On-call rotation for SEC-1 and SEC-25 (two people, all year) | 6 |
| Rust↔V8 boundary rules, suite and fuzzing (SEC-11) | 4 |
| GPU service ownership checks, clamps, display-list, call-sequence and shader fuzzing, translation checks (SEC-10) | 7 |
| Library-sandbox gate for codecs and PDF (SEC-12) | 4 |
| WebRTC placement, ICE policy and fuzz targets; native DRM containment (SEC-30, SEC-33) | 4 |
| Network protocol fuzzing, resource caps, LNA suite (SEC-28, SEC-31) | 5 |
| Supply chain: P0 and P1 controls, `hg-update` and the updater process, roles, emergency release class and mitigation channel, signing, reproducible V8 archive, key drills (SEC-13 to SEC-17, SEC-27) | 12 |
| Downloads, external protocols, File System Access, secrets at rest (SEC-32, SEC-34) | 3 |
| Security response, triage at AI-era volume, pre-release audits (SEC-24, SEC-25), over 24–36 months | 15 |
| Agent and automation additions in Stage 1 (SEC-18 to SEC-20 suites; broker scheme allowlist) | 3 |
| **Total** | **106, of which about 10 are contracted audit work (SEC-D10): 96 in-house, about 2.7–4.0 engineers over 24–36 months; 3–4† in the track** |

**Recommendation (SEC-D2):** add a **Security track of 3–4 engineers†**, led by a named security owner, which takes the review and approval duties the Stage 1 design already assigns to that role and staffs the on-call rotation. Stage 1's minimum staffing becomes **26–34.5 engineers†**, about 52–104 engineer-years over 24–36 months† (previously 46–92). External audits and a later bug bounty are money, not engineers, and depend on D-S8. The `rusty_v8` owner can sit in the JS, bindings and DOM track, but SEC-1's 72-hour bound needs a named backup who can run the backport path.

**Self-approval is visible.** Until a security owner distinct from the maintainer exists, every gate this spec gives that role (SEC-6, SEC-9, the sync allowlist, SEC-26's reviews, IPC-R16 entries) is recorded as "self-approved" in the pull request and counted in the quarterly report, so the gap is visible rather than hidden.

### 9.3 Decisions for the owner

A bare "yes" to any of these means the recommendation. Build-versus-reuse is not on this list: the standing answer is "build our own, following our specs" (M-1, D-S10).

| ID | Decision | Options | Recommendation | Reversal trigger |
|---|---|---|---|---|
| **SEC-D1** | V8 patch-latency bound | (a) Keep D2's one Chrome stable cycle with Stage 1 7.7's lag metric; (b) SEC-1's 72 h† for exploited bugs, 7 days† for critical and high, one cycle for the rest, with the clock at the earliest upstream fix or disclosure | **Decided (0.3).** **(b)** | Two quarters† of SEC-2 drills show (b) cannot be met without skipping T2: restate the bound from measured pipeline time |
| **SEC-D2** | Security staffing | (a) Keep security inside the 2-engineer Network and security track; (b) a Security track of 3–4† with a named security owner and the on-call rotation | **Decided (0.3).** **(b)** | Measured security work at the M2 review is under one engineer†: fold the track back |
| **SEC-D3** | Library-sandbox target in Stage 1 | (a) Stage 1's library-sandbox design ships with S-5 still a Stage 2 target; (b) make it a Stage 1 target gated at M7 (SEC-12) | **Decided (0.3).** **(b)**: libvpx and WebRTC have in-the-wild exploits, and Stage 1 ships both | Isolation costs break playback on a reference device and no Rust decoder exists: time-bounded exception in the decision log |
| **SEC-D4** | Release governance with one maintainer | (a) Single maintainer with SEC-27's compensating controls indefinitely; (b) those controls now, and a **second person with release authority before the first public binary**, with two-person approval for release tags and for TUF root signing and a second `releases` key holder | **Decided (0.3).** **(b)** | D-S8 funds a second person earlier: the date moves forward. If D-S8 cannot fund one, the first public binary waits or ships labelled as a preview without auto-update |
| **SEC-D5** | JIT policy per tier | (a) Standard everywhere, Hardened opt-in only; (b) as (a), plus the banking container's default set to Hardened; (c) Hardened by default for all users | **Decided (0.3).** **(b)**: one default container gets the jitless tier with ACG and CET on Windows, at a cost users of that container can see (Izcal-only renderers are jitless regardless, Section 6.7) | PRIV-1-style corpus breakage for the banking container above 5%†: revert that default |
| **SEC-D6** | macOS sandbox backend | (a) Seatbelt SBPL only; (b) Seatbelt plus an Enhanced Security helper-extension backend evaluated from M1 | **Decided (0.3): (b)**, after an early measurement on macOS 27.0.1 (`research_notes/Izcal security scope/measurements/sec-d6/`): custom deny-default profiles through `sandbox_init` and `sandbox_init_with_parameters` apply and enforce (file, network and Mach-lookup denials) with the binary's SDK field at 26.2, 27.0 and 28.0, while the named `kSBXProfile*` profiles get the process killed inside `sandbox_init` once the SDK field is 27.0 or later. Only half the reversal trigger is met: Apple does not state that custom profiles are supported, and names Endpoint Security (`es_new_descendants_client`, new in macOS 27), App Sandbox and Virtualization as the supported options, so the (b) evaluation adds Endpoint Security. The SDK 27 results come from a simulated link (26.2 headers, SDK field set to 27.0); a real Xcode 27 build is still required at M1 | SDK 27 measurement shows custom profiles still apply and Apple states they are supported: drop the extension evaluation. If SDK 27 forces a reduced renderer tier, this decision is reopened with the measured options |
| **SEC-D7** | Phishing, malware URL and download-reputation protection | (a) None in Stage 1 (document the gap; disable UI-9's phishing interstitial); (b) a local-list protection written by Izcal from a published privacy-preserving lookup protocol, with a list source chosen by the owner, also used for download reputation; (c) an Izcal-operated list service | **Decided (0.3).** **(b)**, and required for agent tasks (B9) when they ship; the protocol and source need a recorded privacy analysis under P1 and B10's classifier rules; the list is a TUF data channel (UPD-R1). **Sources (owner, 0.3), from an assessment of 25 candidates with five feed samples over 3.4 hours (`research_notes/Izcal security scope/measurements/sec-d7/`):** the protocol and list format follow Google's published Safe Browsing v5 local-list design, filled with Izcal's own data, never Google's (its terms ban commercial use and serving its data from a mirror). Phishing: **CERT Polska**'s warning list as primary (any use, no attribution, about 124,500 domains, rebuilt about every 5 minutes), plus **Phishunt** (CC0) and **URLAbuse** (CC BY 4.0). The free sources barely overlap (8% of PhishTank's hosts are in CERT Polska) and are weakest on global, non-Polish phishing. Malware URLs: **abuse.ch URLhaus**, whose free terms are not-for-profit only, so malware URLs are a **documented gap until a commercial licence is signed**. Download reputation: the download URL and every redirect host are checked against URLhaus, plus SH-R7; no public hash-reputation service exists. Matching is by full URL, never by whole domain, and the list build applies an allowlist of popular and shared-hosting domains. **Shelved until close to release (owner, 0.3):** creating the URLAbuse account, asking Cisco Talos for PhishTank redistribution rights, and licensing URLhaus (Spamhaus) and a global phishing feed (Netcraft or APWG eCX), which together form the M8 pre-release gate; the privacy analysis covers what the TUF fetch reveals, on-device matching, any later prefix check through a non-colluding relay, user-initiated reporting only and no hit telemetry | The chosen list source requires sending browsing data the privacy analysis rejects: fall back to (a) for browsing and keep it required for agent tasks only. CERT Polska changes its terms or stops publishing for more than 7 days, or a licensor refuses or withdraws: replace that source. A popular domain outside the allowlist is listed: that list version does not ship |
| ~~SEC-D8~~ | *Retired in 0.2.* Importer on Windows | — | Settled rule SH-R2: no cookie import; Chrome passwords only from a user-exported CSV | Chrome documents a supported export interface: use it (kept in SH-R2) |
| ~~SEC-D9~~ | *Retired in 0.2.* Crash-report and recording retention | — | Settled rules REC-R3 (SEC-21 (c)) and SH-R5 (SEC-35): 30 days† | A legal review sets a different period (kept as a note in both rules) |
| **SEC-D10** | Audit and bounty budget | (a) Bounty and audits after funding; (b) an external audit before the first public binary regardless, about 10 engineer-months† of contracted audit over Stage 1; bounty when D-S8 allows | **Decided (0.3).** **(b)** | The pre-release AI-assisted audits (SEC-24) find fewer than one high finding per milestone for two milestones†: reduce the contracted share |
| **SEC-D11** | Emergency mitigation channel | (a) None: emergency releases only (UPD-R4); (b) a TUF-signed configuration that can switch on only a closed list of security switches (jitless, Wasm off, WebGPU off, one codec off, DrumBrake off), each with an expiry and shown on `izcal://sandbox` (UPD-R5) | **Decided (0.3).** **(b)**: it is the only failure action for SEC-1 that protects users who do not read advisories. It changes behavior without a user action, which B10's "no surprise changes" spirit and P1 must weigh, so the switch list is closed and no switch sends data | A use of the channel outside the closed list, or a privacy review that rejects it: fall back to (a) |
| **SEC-D12** | V8 tracking source | (a) Deno's `rusty_v8` tags only; (b) Deno's tags while they sit on a branch Chrome patches, and the fork's own rebase onto the patched V8 branch whenever they do not | **Decided (0.3).** **(b)** | Self-rebases cost more than 5 engineer-days† per Chrome cycle for two quarters†: revisit the fork's scope with measured costs |
| **SEC-D13** | Network-service sandbox (S-6) in Stage 1 | (a) Best effort per OS, reported (Stage 1 today); (b) required at M8 on Linux and macOS at the Section 6.7 tier, best effort on Windows until its tier is measured at M5 | **Decided (0.3).** **(b)**: the service holds every cookie and parses every byte from the network | The Windows measurement at M5 shows a workable tier: extend (b) to Windows. A Linux or macOS tier that breaks HTTP/3 or DNS on a reference device: time-bounded exception |
| **SEC-D14** | Updater install model | (a) Per-user install, no elevated component; (b) per-machine install with an elevated update service or privileged helper | **Decided (0.3).** **(a) for Stage 1**: no elevated component exists to attack, at the cost of DPAPI-user-scope keys on Windows (SEC-34) and per-user installs only | Enterprise deployment (Stage 2) needs per-machine installs, or an app-bound Windows key proves worth an elevated service: decide (b) with its own threat model |

### 9.4 Amendments this spec proposes to other documents (not applied)

This document edits no other file. The owner or the spec owners apply these, with revision lines as `CONTRIBUTING.md` requires.

| Document | Proposed amendment |
|---|---|
| HGBA-2 D2 ("Owning `rusty_v8`"), S-4e and Stage 1 7.7 | Replace "within one Chrome stable cycle" with SEC-1's severity bounds and clock; add the Chrome-patched-branch rule (SEC-D12) and SEC-2 drills on release runners; list advisory sources for every shipped dependency in the 7.7 table (SEC-25) |
| HGBA-2 S-2 | Extend to every sandboxed process type and add the architecture check and the jitless `PROT_EXEC` rule (SEC-9) |
| HGBA-2 S-4a | "CET/BTI on every platform" becomes "where hardware and OS support it, reported per device"; add ACG on Windows for the jitless tier; Izcal-only renderers jitless by default |
| HGBA-2 S-5 | Stage 1's library-sandbox design for codecs and PDF becomes a Stage 1 target (SEC-12; SEC-D3) |
| HGBA-2 S-6 | Required at M8 on Linux and macOS (SEC-D13) |
| HGBA-2 Section 3 | Record the macOS Seatbelt risk and the backend trait |
| HGBA-2 Section 4.8 | Add FFI-R1 to FFI-R4 |
| HGBA-2 I-2 note | Add SH-R4's no-bridge, separate-process, restart and minimum-version rules |
| HGBA-3 B2 and Stage 1 Section 4 | Add IPC-R1 to IPC-R17, including bound and free endpoints, per-frame endpoints and the new nested-sync rule |
| HGBA-3 VER-1 | Name the three proven properties; add the per-backend handle harness |
| HGBA-3 DST-1 | Add the adversarial-peer measurement and the planted-bug benchmark (SEC-5) |
| HGBA-3 B9 and AG-3 | Add AGT-R1 to AGT-R6 (SEC-18, SEC-19) |
| Stage 1 Section 1 | Add the DevTools process kinds, the crash handler, the prompt renderer and the updater; place WebRTC (network service and utility) and the native CDM process; state that the broker owns native windows and input routing (CON-R3); record the GPU and network service sandbox tiers per OS |
| Stage 1 Section 2.2 (G3) | Place `hg-sandbox` inside `hg-substrate-os`; `hg-update` uses files and network only through it |
| Stage 1 7.1 | Release builds use no `sccache` or other cache; release-build downloads are hash- or version-pinned |
| Stage 1 7.5 | Add the adversarial-peer, transport, GPU and display-list, Rust↔V8, WebRTC and network protocol campaigns |
| Stage 1 7.8 and 13.2 (Updates, Packaging) | Attestations, immutable releases, reproducible V8 archive, TUF roles and the emergency release class, data channels as TUF targets, the updater process, key custody and drills (SEC-13 to SEC-17, UPD-R1 to UPD-R5) |
| Stage 1 8.3 | Add the Security track (SEC-D2) |
| Stage 1 13.2 (Threat model) | List all seventeen process kinds and the two components; this document is the template and first pass |
| Stage 1 13.2 (Everyday features) | Importer per SH-R2; out-of-DOM autofill (SEC-23); downloads, external protocols and File System Access per SH-R6 to SH-R9 |
| Stage 1 Section 6 (GPC row; credential sentence) | Replace the three-state claim after legal review; say "credentials stored only in the OS keychain" |
| DevTools spec 6.1 and 6.5 rules 4, 8 and 10 | Replay host at the recorded renderer tier (REC-R6); the broker owns the loopback listener and passes accepted connections to the gateway; `script.v8` routed as opaque bytes (IPC-R16); imported bundles, OS-bound key, retention, timestamp quantization (REC-R1 to REC-R4, SEC-21); launch-flag rule (AGT-R8) |
| Shell UI design (UI-5, UI-10, 4.5, UI-9) | Security prompts drawn by the prompt renderer and approved only on broker-observed input (CON-R1 to CON-R5); phishing interstitials depend on SEC-D7 |
| `CONTRIBUTING.md` | Rulesets instead of client-side tag rules; S-4e force-push as a logged bypass; checks from the base branch; V8 hash pins verified at build time; the AI review documented as advisory; self-approvals recorded |

---

## 10. Risks

| Risk | Where | Mitigation |
|---|---|---|
| SEC-1's 72-hour bound depends on learning of fixes; if Izcal learns only at Chrome stable release, exploited V8 bugs are public n-days for Izcal | 6.8 | The clock starts at the earliest upstream signal; SEC-2 drills shorten the pipeline; Q-SEC-1 on advance notice; the emergency mitigation channel during a window |
| Deno's tags lag Chrome's patched branches and self-rebases are costly | 6.8 | SEC-D12's trigger measures the cost; the staffing row covers self-rebases |
| The emergency channel is itself an attack or a surprise | 6.16 | Closed switch list, TUF signing by the `releases` role, expiry, visible state (UPD-R5, SEC-D11) |
| One person holds every key and approval | 6.16 | SEC-D4; hardware keys; offline root threshold; self-approvals counted |
| The prompt renderer or broker input routing misses the shell UI latency targets | 6.6a | Measured before M6; native prompts as the fallback (CON-R2 (c)) |
| macOS SDK 27 kills custom sandbox profiles | 6.7 | Measured October 2026: custom profiles apply on macOS 27.0.1, named profiles are killed (simulated SDK 27); confirmed with a real Xcode 27 build at M1; extension and Endpoint Security backends evaluated (SEC-D6); a reduced renderer tier becomes an owner decision |
| GPU drivers below validation | 6.9 | Ownership checks, clamps, fuzzing and the sandbox reduce reach; the residual is stated |
| Adversarial simulation finds little | 6.6 | The planted-bug benchmark measures it; `cargo-fuzz` and the transport fuzzer run regardless |
| The Rust WebRTC stack and PDF engine are large new engine work | 6.10, 6.10a | Costed in their owning tracks; the standing answer applies; fuzz targets from the first commit |
| Broker mitigations break IMEs and assistive technology | 6.7 | M5 measurement; a separate UI-host process if needed |
| Security track cannot be funded (D-S8 deferred) | 9.2 | The P0 items cost days; the rest waits with the milestones it gates |
| AI-found bug volume exceeds triage | 6.17 | Pre-release audits move the volume before release; SEC-25 publishes misses |
| The jitless tier breaks sites in the banking container | SEC-D5 | Per-site exception mechanism (S-4a); reversal trigger |
| Phishing protection's list source conflicts with privacy | SEC-D7 | Privacy analysis before choosing; fallback (a) |
| Building `hg-update` in-house introduces bugs a library would not | 6.16 | Attack-class test vectors, differential testing, fuzzing; reversal trigger |
| Threat models go stale | 6.5 | SEC-26 ties them to endpoint diffs |
| The broker's own logic is wrong despite all rules | 6.6 | Rules shrink the space; the adversarial suites and Kani cover what they can; the residual is the broker's stated risk |

---

## 11. Open questions and flagged claims

### 11.1 Open questions

| ID | Question | Owner | Answered by |
|---|---|---|---|
| **Q-SEC-1** | Can an independent V8 embedder receive advance notice of V8 security fixes, and from whom? Not researched; it would move SEC-1's time zero earlier | Security owner | M2 |
| **Q-SEC-2** | Does a custom SBPL profile applied with `sandbox_init_with_parameters` still work when linking against SDK 27, and does notarization require building against SDK 27? | Network and security | M1 (measured) |
| **Q-SEC-3** | Do Enhanced Security helper extensions support JIT, per-site instance counts, Mach-port handoff and broker-owned policy? Undocumented | Network and security | M2 |
| **Q-SEC-4** | Where do OS media decoders run on each OS, and can they run inside Izcal's utility sandbox? | JS, bindings and DOM (media) | M7 |
| **Q-SEC-5** | What root-store policy process applies to the fallback root list (Mozilla's list, used through `webpki-roots`)? | Network and security | M5 |
| **Q-SEC-6** | Should the update client run in its own process rather than in the broker? **Answered in 0.2: yes** (UPD-R2, SEC-D14) | — | 0.2 |
| **Q-SEC-7** | Which OS mechanisms protect a consent prompt from synthetic input on Windows, macOS and Linux (X11 and Wayland)? | Shell | M6 |
| **Q-SEC-8** | Is the V8 sandbox declared a security boundary in 2026? No primary statement after April 2024 was found | Security owner | M7 (layer table) |
| **Q-SEC-9** | Which Windows tier (Low integrity, LPAC) do the GPU drivers on the reference devices tolerate? | Graphics | M5 |
| **Q-SEC-10** | Are the fork's repository settings (immutable releases, rulesets, required signatures, disabled upstream workflows) already set? Settings are not in git and were not checked | Maintainer | M0 |
| **Q-SEC-11** | Do toolchain downloads inside the V8 build (`tools/rust_toolchain.py`, clang's `update.py`, CIPD for gn and ninja) verify hashes? A grep found no check; CIPD instance IDs may be content hashes. **Now a P0 item**, because SEC-15 depends on it | `rusty_v8` owner | M0 |
| **Q-SEC-12** | Does self-hosted push (autopush, Stage 2) need its own threat model and retention rules? Not researched | Security owner | Stage 2 |
| **Q-SEC-13** | Which image and font decoders? **Recommendation:** Rust decoders, as unmodified non-browser crates where they exist, otherwise built new; Chrome's Fontations record (Section 2.1) supports Rust fonts | JS, bindings and DOM | M4 |
| **Q-SEC-14** | Is WebGPU on by default per platform? **Recommendation:** on only where SEC-10 (a) and (b) pass on that platform's reference device | Graphics | M8 |
| **Q-SEC-15** | Where does Izcal's WebGL implementation live, and which shader translator do macOS and Linux use? **Recommendation:** in the GPU service with validation before ANGLE or GL (GPU-R3); ANGLE unmodified on Windows; the macOS and Linux translator chosen by measurement against the Khronos conformance suite | Graphics | M4 |
| **Q-SEC-16** | Which Chrome branches receive V8 security backports (stable, extended stable), and for how long? The critique states the policy; no primary source is in the research notes | `rusty_v8` owner | M0 |
| **Q-SEC-17** | Which OS processes host the PlayReady CDM and the Windows protected media path, and what sandbox does the CDM tolerate? | Shell | M7 |
| **Q-SEC-18** | What is the local-privilege-escalation record of elevated browser updaters, and what would a per-machine Izcal install need? Not researched; it informs SEC-D14's reversal | Infrastructure | Stage 2 |
| **Q-SEC-19** | What quantization of inter-keystroke intervals defeats keystroke-biometric identification while keeping replays useful? Measurement plan for REC-R4 | DevTools | M7 |

### 11.2 Claims that are single-source, unverified or conflicting

| Claim | Status |
|---|---|
| CVE-2025-2783 mechanism | High via Kaspersky's Securelist; some exploitation details in the history note came from secondary press |
| V8 = 21 of 37 | Holds only if CVE-2024-0519 ("Out-of-bounds memory access" in the sheet) is counted as V8 |
| CVE-2022-2856 and CVE-2022-3075 as sandbox escapes | Not supported by the P0 sheet; 0.2 no longer claims it |
| The SDK 27 `sandbox_init(3)` text | Quoted second-hand on the OpenSSH list; in that thread "these profiles" are the built-in named profiles |
| Chrome's V8 backport branch policy (stable, extended stable) | From the design critique; no primary source (Q-SEC-16) |
| Landlock ABI 8–11 kernel versions; ABI 1–7 kernel versions | Not checked; recalled |
| Ubuntu 24.04 enabling the user-namespace restriction by default | From the Qualys advisory, not Canonical's release notes |
| Flatpak weakening browser sandboxes | Community posts only |
| Chrome VRP 2025 and 2026 figures, the May 2026 cut and the October 2026 restructuring | Secondary reports (SecurityWeek for the cut; Help Net Security for the restructuring and the $5,000 V8 sandbox reward) |
| Gray Swan's 4.7% and 33.6% | Secondary report |
| Atlas omnibox injection; LayerX's "up to 90% more exposed" | Single secondary source; vendor estimate; "to phishing" is not in the quote |
| GPC enforceable in about twelve states | Vendor blogs with conflicting dates (Low); Stage 1 says three |
| CrumbCruncher's 8% of navigations | Recalled, not rechecked |
| AI-assisted auditing as the cause of the 2026 RustSec and Wasmtime spikes | Inference |
| Rust removing most of Chromium's IPC lifetime bug class | The spec's inference; GitHub Security Lab says only that many issues were lifetime issues |
| DrumBrake "23 RCE bugs" | Researchers' claim |
| Lockdown Mode disabling the JIT | Not stated on Apple's page |
| Whether Rust (wgpu) or C++ glue caused Firefox's WebGPU escapes | Unknown; the advisory does not name wgpu |
| MiraclePtr memory cost | Both figures confirmed, for different scopes: +4.5–6.5% on Windows (2022 post); +5.5–8% desktop browser process (2024 post) |
| Apple MIE and six chains | Vendor claim: "six of the real-world exploit chains that we evaluated" |
| Cyberhaven user counts; Notepad++ attribution; Shai-Hulud, Trivy and chalk counts; slopsquatting rates | Vendor or secondary sources (Low) |
| CA/B Forum 460-day code-signing limit | Secondary (Low) |
| Workflow lockfiles for free public repositories | Enterprise Cloud preview only; availability unknown |
| Chrome passwords under App-Bound Encryption in 2026 | Google's 2024 post says "planned"; current state not confirmed |
| WebView2 two-week cadence from version 153; VoidStealer | Single secondary sources |
| Project Zero sheet "last updated 2023-04-20" while holding 2026 rows | Contents used as found |
| Bit-for-bit reproducibility of the V8 archive | Not measured |
| No published case of DST with an adversarial peer; no academic Mojo fuzzing paper | Absence in searches, not proof |
| Download, external-protocol and File System Access attack prevalence (Section 6.14a) | The research notes hold no incident data; the rules are first-principles |
| Elevated-updater privilege-escalation record | Not researched (Q-SEC-18) |
| ICE candidate policy and the CDM process placement | Design choices, not evidence-backed claims; CDM hosting to be measured (Q-SEC-17) |

### 11.3 Contradictions and gaps found between existing specs

Eleven items. The fact-check found 8 real as stated (#2–#6, #9–#11) and 3 partly right (#1, #7, #8); 0.2 restates those three.

1. **Patch latency has a metric but no severity target.** HGBA-2 D2 says "ship V8 security fixes within one Chrome stable cycle", Stage 1 7.7 tracks "Days behind Chrome stable V8 security fixes" with an escalation at more than two weeks, and S-4e is a ship blocker with a backport fallback. No target sets latency by severity, publishes it per advisory, or acts when a fix itself is late. *(Partly right in 0.1, which said there was no measurement.)*
2. **The security owner is referenced but not staffed.** Stage 1 4.1 and 7.7 give the security owner review and SLA duties; Section 8.3 has no such role.
3. **The threat-model list is incomplete.** Stage 1 13.2 lists eight processes and omits the protected-media host, the ML service and the crash handler; the DevTools spec's four process kinds are a proposed, unapplied amendment.
4. **Phishing interstitials without phishing detection.** The shell UI designs phishing and malware interstitials (4.5) and measures them (UI-9), but no spec provides a phishing or malware URL source; B10 only defines when a classifier would be exempt, and the ML inventory is empty.
5. **CONTRIBUTING asks for committed `SHA256SUMS`, but the build verifies nothing.** `RUSTY_V8_ARCHIVE_SHA256` is unset, and the binding file is never checked.
6. **Branch protection is described as server-side, yet S-4e lifts it** each Chrome cycle for the force-push; release-tag protection is client-side (`pre-push`).
7. **S-4a says "CET/BTI on every platform"**, but CET needs specific CPU generations and Windows versions, so the target cannot hold on every device Izcal supports. *(Partly right in 0.1: "Apple Silicon uses PAC and BTI, not CET" does not contradict "CET/BTI".)*
8. **Stage 1 7.1 uses `sccache` for builds generally**, and no project-level rule excludes release builds; fork PR #2 (pending merge) skips caches on tag builds, but `hgba` still uses them. *(Partly right in 0.1, which treated the fork's skip as merged.)*
9. **GPC.** Stage 1 Section 6 says three states; the research finds about twelve (Low confidence).
10. **Credential store wording.** Stage 1 Section 6 says "no custom credential store in Stage 1"; 13.2 adds a basic password manager in the OS keychain. Both can hold, but the first sentence should say "credentials stored only in the OS keychain".
11. **Recording start.** Stage 1 3.4 says recording is "user-initiated per tab" with replays in the simulator; the DevTools spec's pending amendment changes this to process launch and one process per replay host.

### 11.4 Research lessons placed in 0.2

| Lesson (research note) | Placement |
|---|---|
| Type-segregated allocators so that MTE and MIE pay off (history) | **Out of scope for Stage 1**: Izcal's code is Rust and S-7 (MTE) is Stage 2; revisit with S-7 |
| Publish which layer stops which class (exploits, lesson 6) | Deliverable at M7 (Section 6.8) |
| SHA-pin updates through Renovate or Dependabot; GitHub's SHA-pinning policy (supply chain) | P0 control (Section 6.16), checked in SEC-27 |
| D7 equivalence gating includes security regression tests (exploits, lesson 9) | SEC-24 |
| FIDO downgrade by user-agent spoofing (new surfaces 12.3) | Recorded for UA-1 in SH-R1 |
| Document importer behavior for EDR vendors (new surfaces) | SH-R2 |
| Residual from third-party MCP servers (new surfaces) | AGT-R7 |
| Wasmtime LTS tracking (new surfaces) | SH-R3 |
| CHIPS and FedCM versus Privacy Sandbox (new surfaces 7) | **Out of scope**: a privacy-spec question (Section 6.15) |

---

## Appendix A. Review disposition

Every finding of the 0.1 design critique (`critique.md`). The fact-check's required corrections 1–10 are all applied; they are summarized in the revision line and marked in Sections 3, 6.16, 8, 9.1 and 11.

| Finding | Disposition | Where, or why |
|---|---|---|
| **H1** Trusted region drawn by an untrusted renderer | Applied | CON-R1 to CON-R5 (broker-verified OS input hit-test, separate prompt renderer, native fallback); shell in ATK-2's examples; shell and prompt-renderer models; SEC-29 with compromised-shell harness; CON-R4 (shell holds no secrets) |
| **H2** Replay host and gateway sandbox tiers | Applied | Replay host at the recorded renderer tier per (site, container) (REC-R6, Section 6.7); gateway gets accepted connections from the broker and keeps "no network" |
| **H3** SEC-1 unachievable and gameable | Applied | Chrome-patched-branch rule and SEC-D12; clock at the earliest upstream fix or disclosure, computed after the fact; SEC-2 on release runners with per-stage times; emergency mitigation channel as the failure action (SEC-D11) |
| **H4** Release-chain targets block the emergency path | Applied | UPD-R3 (root offline, delegated `releases` role, online timestamp and snapshot); UPD-R4 emergency release class with reproducibility within 7 days†; key-loss-only statement until SEC-D4 |
| **H5** One compromised process only | Applied | ATK-2 redefined as any sandboxed process; IPC-R17 privilege lattice; SEC-3 (c) and SEC-5 (a) for every process kind |
| **H6** IPC-R2 identity holes | Applied | IPC-R2 (a)–(c): bound and free endpoints, per-frame and per-worker endpoints, `CallerIdentity` versus `TargetOrigin` and `NavigationUrl`; IPC-R8 ID generations; SEC-3 (a), (b), (d) |
| **H7** WebRTC and native PlayReady missing | Applied | Section 6.10a; rows in 6.3, 6.3a and 6.5; SEC-30, SEC-33; M-1 provenance in Section 7; WebRTC dependencies under SEC-25 |
| **H8** GPU plan overstated; display lists uncovered | Applied | GPU-R2 (wgpu-core validates), GPU-R3 and Q-SEC-15 (WebGL location), GPU-R4 and IPC-R16 (display lists entry 1); SEC-10 (b), (c) with conformance-suite metrics; staffing row reduced |
| **M1** Rule of Two misses in-origin injection | Applied | AGT-R6 counts third-party-writable content as untrusted; SEC-19 (b) confirmation ceiling |
| **M2** SEC-25 lacks exploited tier and C/C++ deps | Applied | SEC-25 72 h† tier for any component; every shipped dependency; advisory sources in 7.7 |
| **M3** Staffing math; self-approval | Applied | Recomputed (106, 96 in-house, 3–4†); on-call row; self-approval recorded and counted |
| **M4** SEC-8 always passable | Applied | Non-waivable floors on reference devices; waiver count published; macOS reduced tier goes to SEC-D6 |
| **M5** Broker mitigations versus IMEs and assistive tech | Applied | Broker owns the native window (CON-R3); CIG and extension-point blocking measured at M5; UI-host fallback |
| **M6** CI downloads overclaimed | Applied | `tools/win` and LLVM row marked open on both branches; P0 control; SEC-13 (f); Q-SEC-11 made P0 |
| **M7** Updater privilege and data channels | Applied | Updater process (UPD-R2), SEC-D14, rows in 6.3 and 6.5; UPD-R1 every data channel a TUF target; SEC-17 (c), (d) |
| **M8** IPC-R5 lint and panic strategy | Applied | Outright denies with an allowlist; abort in the broker, unwind per connection in the network service (0.3: the owner chose abort in the network service too, coding standard CS-D1 (a)); SEC-7 split into parser and handler evidence |
| **M9** Copy-then-validate and read-only memory | Applied | IPC-R7 limited to parsed data; per-backend read-only definitions in IPC-R1; SEC-4 (a) test |
| **M10** Windows broker handle checks | Applied | IPC-R1 per-OS mechanism (kind from the object, narrowed mask, macOS port types); SEC-4 (a) |
| **M11** SEC-20 and same-user malware | Applied | Profile-in-use clause demoted to defense in depth; ATK-7 "cost and evidence"; Linux residual in SEC-34 |
| **M12** Timestamp quantization breaks replay | Applied | REC-R4: only inter-keystroke intervals in text fields, divergence check and label; Q-SEC-19 measurement plan |
| **M13** Izcal-only processes jitless | Applied | Section 6.7: shell, prompt renderer, workspace and panes jitless with ACG and CET, `PROT_EXEC` denied on Linux |
| **M14** Provenance gaps under M-1 | Applied | Section 7: PDF engine, WebRTC stack, crash handler, GPU blocklist and tainted-type wrapper built new; CORB renamed ORB (NET-R2) |
| **M15** Missing and non-real decisions | Applied | Added SEC-D11 (emergency channel), SEC-D12 (V8 source), SEC-D13 (S-6), SEC-D14 (install model); image/font decoders and WebGPU default as Q-SEC-13 and Q-SEC-14; SEC-D8 and SEC-D9 retired into rules; SEC-D6 option (c) removed; triggers added to SEC-D4 and SEC-D10 |
| **M16** No local-network boundary | Applied | TB-13, NET-R7, SEC-31 |
| **M17** Downloads, protocols, File System Access | Applied | Section 6.14a, SH-R6 to SH-R9, SEC-32; download reputation tied to SEC-D7 |
| **M18** SEC-5 on the critical path; transports untested | Applied | M6 gated on generated harnesses under either tool, as a T0 merge rule; per-OS transport fuzzer (SEC-5 (c)); planted-bug benchmark (SEC-5 (b)) |
| **M19** Scorecard criterion undefined | Applied | SEC-27 names the checks that must score 10 and the known exceptions |
| **M20** Not yet usable as the first document | Applied | Reading guide by role; per-process index (6.3a); rule IDs (CON-R, FFI-R, GPU-R, NET-R, AGT-R, REC-R, SH-R, UPD-R); Q-SEC owners and milestones; shortened "Relates to"; ATK × TB matrix |
| **L1** G12 cited as existing; G3 placement | Applied | G12 marked proposed (6.4, 6.5); `hg-sandbox` inside `hg-substrate-os`; 9.4 amendment |
| **L2** † thresholds without rationale | Applied | Rationale table in Section 8; one timestamp-expiry value (3 days†) |
| **L3** Threat-model timing | Applied | Section 6.5 and SEC-26: all files at M1, three filled at M1, the rest by M6 |
| **L4** SEC-10 (a) milestone | Applied | "Built at M3–M4, gated at M8" |
| **L5** Vague measurement methods | Applied | SEC-3 (a) type rule; SEC-4 (d) crate-graph lint; SEC-11 (c) 30-day† threshold |
| **L6** JIT tiers need separate binaries | Applied | Section 6.7 per-tier helper executables; `PROT_EXEC` denial (SEC-9) |
| **L7** Research lessons without placement | Applied | Section 11.4 |
| **L8** Same-site co-location | Applied | Renderer model residual; TB-1 note |
| **L9** SQLite hardening | Applied | Storage-service model |
| **L10** Windows renderer token combination | Applied | Section 6.7: measured at M5; CIG after Izcal's DLLs load |
| **L11** V8 inspector JSON through the broker | Applied | IPC-R16 entry 2 (opaque bytes; parsed in the gateway or pane) |

No finding was rejected. One was applied differently in detail: H1's recommendation of a separate prompt renderer "for credential and agent confirmations" is extended to all security prompts, because the same compromised shell could approve any of them.

## Appendix B. ID changes from 0.1

No 0.1 ID was renumbered. The coding standard's references (SEC-3, SEC-4, SEC-7, SEC-11, IPC-R1 to IPC-R15) keep their subjects; the additions below are the clauses an implementer must follow.

| ID | 0.1 | 0.2 |
|---|---|---|
| ATK-2 | Compromised renderer | Compromised **sandboxed process** of any kind (broadened) |
| TB-2 | Sandboxed process ↔ broker and services | Adds sandboxed ↔ sandboxed edges and the IPC-R16 payloads |
| TB-12 | Shell renderer ↔ broker | Adds OS input ↔ consent (CON-R1) |
| IPC-R1 | Typed, broker-minted, only narrowed | Adds per-OS checking mechanism, read-only memory definitions, accepted-socket kind |
| IPC-R2 | Identity from the endpoint | Adds bound and free endpoints, per-frame and per-worker endpoints, `CallerIdentity` versus data types; holds on every edge |
| IPC-R3 | Closed type universe | Points non-`.hgidl` payloads to IPC-R16 |
| IPC-R5 | No panic on message-derived values | Outright lint denies with an allowlist; panic strategy per process |
| IPC-R7 | Copy, then validate | Limited to data the receiver parses; opaque sampled buffers exempt |
| IPC-R8 | Broker owns lifetime | Adds never-reused, generation-tagged IDs |
| IPC-R11 | Surface inventory | Adds bound or free endpoints and IPC-R16 payloads to the inventory |
| IPC-R12 | Privileged endpoints bound to a content kind | Adds prompt endpoints |
| IPC-R13 | Sync allowlist; nested sync forbidden "(Stage 1 4.1)" | Same rule; nested-sync ban marked as new in this spec |
| IPC-R14 | Sequence fuzzer in the simulator | Simulator or `cargo-fuzz`; no endpoint merges without it |
| IPC-R16, IPC-R17 | — | New: non-`.hgidl` exception list; no process trusts another |
| SEC-1 | Latency from first Chrome stable or an Izcal-identified commit | Clock at the earliest upstream fix or disclosure; patched-branch condition; emergency channel as failure action |
| SEC-2 | Quarterly drill | On release runners, per-stage times, emergency release and configuration push |
| SEC-3 | Renderer-scope identity | Every edge; type rule; bound endpoints; per-kind suites; ID reuse (subject unchanged) |
| SEC-4 | Handle tests | Adds read-only-kind test, Windows mechanism test, crate-graph lint (subject unchanged) |
| SEC-5 | Adversarial renderer; yield per CPU-hour | Every process kind; merge rule; planted-bug benchmark; transport fuzzer |
| SEC-7 | Kani plus T2 crashes | Adds handler lint clause (b); old (b) is now (c) (subject unchanged) |
| SEC-8 | Floor or reduced tier | Non-waivable floors on reference devices; waiver count |
| SEC-10 | Izcal validates every command | wgpu-core validates WebGPU; WebGL validation; conformance metrics; display lists |
| SEC-11 | Rust↔V8 rules | Adds 30-day† threshold to (c); rules named FFI-R1 to FFI-R4 (subject unchanged) |
| SEC-12 | "C parsers sandboxed in Stage 1" | "Stage 1's library sandbox is a Stage 1 target" (design already existed) |
| SEC-13 | (a)–(e) | Adds (f) pinned release-build downloads |
| SEC-15 | Reproducible before publish | Adds the emergency release class |
| SEC-16 | Root and targets offline | Root offline; delegated `releases` role; online timestamp and snapshot |
| SEC-17 | Update client | Adds every data channel and verification outside the broker; one expiry value |
| SEC-19 | Out-of-set trigger | Adds in-origin third-party content; confirmation ceiling (b) |
| SEC-20 | Profile-in-use refusal as a control | Kept only as defense in depth; consent through CON-R1 |
| SEC-21 | All input quantized | Only inter-keystroke intervals in text fields; (e) recorded engine configuration |
| SEC-25 | Crates only | Every shipped dependency; exploited tier |
| SEC-26 | Files "at M1" | All files at M1, three filled; nineteen models |
| SEC-27 | "Scorecard clean at medium" | Named Scorecard checks and exceptions |
| SEC-29 to SEC-35 | — | New: consent, WebRTC, local network, downloads and handles, native DRM, secrets at rest, crash dumps |
| SEC-D2 | Track of 2–3† | Track of 3–4† |
| SEC-D3 | "S-5 moves to Stage 1" | "Stage 1's library-sandbox design becomes a Stage 1 target" |
| SEC-D6 | Options (a)–(c) | Option (c) "wait for Apple" removed |
| SEC-D8, SEC-D9 | Decisions | **Retired**: now SH-R2 and REC-R3/SH-R5; IDs not reused |
| SEC-D11 to SEC-D14 | — | New: emergency channel, V8 tracking source, S-6 in Stage 1, updater install model |
| Q-SEC-6 | Open | Answered (UPD-R2) |
| Q-SEC-13 to Q-SEC-19 | — | New |
| TB-13; CON-R, FFI-R, GPU-R, NET-R, AGT-R, REC-R, SH-R, UPD-R | — | New; the FFI-R, GPU-R, NET-R, AGT-R, REC-R and SH-R rules give IDs to 0.1's prose bullets without changing their content except as listed above |
