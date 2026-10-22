# Izcal app host

*First-principles specification of the Izcal app host: sandboxed WASI and Component Model applications in the browser — the component host's app configuration, `app://` identities, capability tickets minted by the broker, a named WASI profile, signed packages installed as grant events, HTML-faced apps with a typed intent boundary, recorded host calls that replay against a virtual filesystem and network, and the Stage 3 track that closes the corpus's WASI door. Date: 21 October 2026. Status: draft 0.2. Research notes: `research_notes/Izcal app host/` (`history_postmortems.md`, `specifications_standards.md`, `security_capabilities.md`, `determinism_recording.md`, `distribution_trust.md`, `gaps_opportunities.md`, `browser_stance_ui.md`; phase 8: `factcheck.md`, `critique.md`). Relates to: HGBA-2 2.4.2 [M-1, P1–P6, I-1, I-2, I-3, D5, X-3, Sections 2.3, 4.2, 6][H2]; HGBA-3 3.3.0 [B1, B2, B9, B10, DET-1, DET-2, AG-0, AG-3][H3]; Stage 1 design 0.14.3 [Sections 1, 2.1–2.2, 4, 6, 8.1, 8.3, 9, 11][S1]; security scope 0.3.7 [CON-R1 to CON-R9, A-SH-12, IPC-R1 to IPC-R17, REC-R1 to REC-R7, AGT-R1 to AGT-R8][SS]; coding standard 0.3.6 [tier P, CS-ST-10][CS]; JavaScript engine 0.4.3 [JS-D10, JS-D13, G6, G14][JS]; extensions 0.4.1 [EX-D3, EX-D5, EX-D10, EX-D12][EX]; devices 0.3 [DV-D2, DV-D4, DV-D7, DV-D8, DV-D9][DV]; storage 0.3.2 [SG-D4, SG-D9, SG-D10, G150][SG]; networking 0.3.7 [NW-D2, NW-D4, NW-D7][NW]; resource footprint 0.4.4 [FP-1, FP-D8, FP-D9, FP-D16][FP]; shell 0.3 [SH-D2, SH-D11, SH-D22][SH]; DevTools 0.3.6 [DT-D2, DT-D5][DT].*

*Revision 0.1 (21 October 2026): first draft.*

*Revision 0.2 (21 October 2026): phase 8 applied, by an independent fact-checker and an independent design critic who did not write the spec. The fact-check raised 65 findings (43 Confirmed, 17 Corrected, 5 Overstated; 12 must-fix) and the critique raised 37 (5 Blocker, 21 Major, 11 Minor). Every finding is applied; Appendix A records each disposition. The design changes that matter: the app id is a manifest-carried random value that survives key rotation (C1); install parses package and component bytes in a tier-P `--verify` configuration, never in the broker (C2); `app://` documents are denied every web permission, storage API and in-place navigation, with AH-17 closing the UI's web surface (C3, C13, C15); the device family ceiling is not an install grant (C4); own-UI and broker-routed intent calls have separate endpoints and principals (C5); app grants have explicit session, remembered (90 days†) and low-risk install lifetimes (C7); the admission rule is three separate instruments (C8, C9); the Stage 1 refusal is read as time-scoped and narrowed to native local apps (C10); replay is jitless by default (C6); the digest, recording-store, revocation and footprint definitions are explicit (C21, C23, C17, C26); and the staffing arithmetic and milestone durations are corrected (C19, C20). Two targets are added, AH-17 (sealed UI authority) and AH-18 (declared destinations); AH-1 to AH-18 now stand.*

**Summary.** Izcal can run a web page and, from Stage 2, an extension, but not a program. The app host is the component host's app, app-replay and install-verify configurations: a tier-P process that runs signed Component Model binaries against a named WASI 0.2 profile, with the manifest as a ceiling, the broker as the only grant authority, `app://<id>` as a non-web identity, an HTML UI in a closed renderer behind typed intents, and a recorded host-call boundary that replays against a virtual filesystem and network. The decisions that matter most are the process-kind configuration (AH-D1), a manifest-carried stable id that survives key rotation (AH-D2), install as a grant event computed by a tier-P verifier (AH-D3), WASI 0.2.12 pinned over 0.3 (AH-D4), the sealed UI with no web authority (AH-D6), and a Stage 3 exposure gate that admits at most 20† named apps (AH-D8). Its cost is **16–39† incremental engineer-years** over 21† months.

## 1. The problem and who has it

Izcal can run a web page and, from Stage 2, an extension. It cannot run **a program**. A person who wants a diff tool, a password generator that never touches a network, a local Git client, an image converter or a notes app has to leave the browser, install a native application with ambient authority over their files and their machine, and accept whatever updater that application chose. The browser's own answer to "run my program" is the installed web app, which is a website with a made-up window: it has the web platform's capabilities, the site's origin, and the site's availability. It is not a program the person owns.

The corpus has said no to this three times, in three different vocabularies:

- **HGBA-2 X-3** calls native Component Model support *optional* and places it in **Stage 3**, counting toward the "native implementations in two browser engines" gate, with "stay on jco-style glue" as the fallback ([H2 6][H2]).
- **Stage 1 design, "Out of scope for Stages 1 and 2"** refuses **"the browser as a window manager for local apps"** outright ([S1 4][S1]).
- **The Stage 3 backlog** lists "native Component Model (optional)" again in its engine column; the shell column's only Stage 3 item is "Sync (if wanted)" ([S1 9][S1]).

None of the three is a design. X-3 is about the **engine** gaining native CM support; the refusal is about the **browser** becoming a window manager; neither says what a local application is, where it comes from, what it can reach, how it is recorded, or what happens when it is malicious. Meanwhile the corpus has built, in other specifications, every primitive an app platform needs and has never joined them: a per-origin sandbox with capability tokens (`hg-js-host`, JS-D10), a broker that is the only grant authority (security scope CON-R), broker-signed tickets for network and devices (NW-D4, DV-D2), signed packages with a review process and a decision log (EX-D10), a storage service that is the only process allowed to touch files (SG, G150), a recording substrate whose replay opens no real resource (DT-D2, DV-D8), and a shell that owns every prompt window and every launch surface (SH-D2, SH-D11).

The gap is that the pieces have never been pointed at third-party native code. This spec points them.

**Who has the problem.**

| Who | What they are trying to do | What happens today |
|---|---|---|
| A person with a small tool they use daily | Run a trusted program without a native install, an admin prompt and an updater | The web app is a site; the native app has ambient authority; the flatpak/snap sandbox is another OS install |
| A developer shipping a tool | Reach users without six package managers and a notarisation pipeline per OS | A web build loses filesystem, processes and determinism; an Electron build ships a whole browser and a Node runtime |
| A security-conscious user | Know what a program can reach, and revoke it | Native OS permissions are coarse; a Wasm runtime outside the browser has no UI, no broker and no recording |
| An agent, once AI is on | Call a tool's typed operations rather than drive a GUI | It scrapes a page or drives a browser automation protocol ([H3 B9][H3]) |

**What the corpus already decides, and this spec must honour.** One process per (site, container) and zero ambient authority (HGBA-2 I-1, I-2); a 16 GB† cage with a trusted region (I-3); the broker as the only grant authority (CON-R1); prompts decided on broker-observed OS input (CON-R1, A-SH-12); recording classes and the never-recorded list (DT-D2); no telemetry from installs (EX-D12); MIT-only for everything that ships (M-1 2.3.2); and no third-party source code as evidence (M-1 part 3).

**What success looks like.** A person installs a signed package, sees exactly which directories, network destinations and device families it may reach, launches it in a window that cannot paint a prompt, uses it offline, and can replay what it did from a recorded host-call log. A page cannot detect that the app host exists. An app cannot name a file, a socket or a device without a ticket the broker minted. A replay of an app opens zero real files, zero sockets and zero devices. A malicious app is bounded by its manifest, its tickets and the sandbox — not by a reviewer's reading of machine code. Those become AH-1 to AH-18.

**Why this is the right time.** WASI 0.2 shipped, WASI 0.3.0 was released on 11 June 2026 ([WASI.dev][W3]), the Component Model has a WIT type system and a canonical ABI, and no browser ships either ([H2 4.2][H2]). The engine landscape has also moved in the last six months: Mozilla resolved a **positive** standards position on the Component Model on **12 October 2026** and is prototyping an implementation; Google/V8 opened an issue to evaluate it and has committed to nothing; WebKit has filed no position on either ([BS 19, 27, 22]). Two native implementations are what the Component Model's Phase 4 gate waits for ([BS 28]); Izcal's engine work is a candidate, though this spec does not rest on the claim. WebAssembly is used by **21.2% of surveyed browser-API developers**, yet appears on **0.35% of desktop pages** and only **2.05% of the top 1,000** — the ecosystem's energy is outside the page, and the browser is the bottleneck [GO 1.1–1.2]. Izcal already owns a Wasm engine, a heap, a code generator and a tier-P discipline for exactly this kind of host (JS-D10). The alternative — a third-party runtime inside the browser — is excluded by M-1 and would reintroduce the second heap and the second code generator that JS-D10 removed.

## 2. History: what came before and why

Installed applications inside a browser are not a new idea; they are a thirty-year sequence of attempts, each with a documented reason for its shape and a documented reason for its end. The record is read here for two questions only: **which failures were technical, and which were about distribution, economics and ecosystem** — because the app host inherits the technical fixes (WASI and the Component Model) and must supply everything else itself.

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1996 | **ActiveX** (Microsoft) | Native COM/OLE controls downloaded from a site, installed, and loaded **in-process** in Internet Explorer | Reuse the Windows component ecosystem; the browser as an extension of the OS | Removed from the Edge engine; fully retired in 2022, with IE mode supported to at least 2029. Maximal local power without a sandbox left a multi-decade cleanup bill ([HP 4.4, 4.10][HP]) |
| 1995–2001 | **Java applets, then Java Web Start** | Centralised zero-install distribution: applets in a browser plug-in; Web Start as a native-app-like launcher from 2001 | The JRE was assumed present, and the plug-in was the only cross-browser code channel | Both depended on runtimes other vendors controlled: browsers removed the plug-in, and OS vendors' bundled-runtime push dismantled the JRE path; Web Start was excluded from Java SE 11 in 2018 ([HP 4.2–4.3, 4.10][HP]) |
| 2008–2013 | **NaCl, then PNaCl** (Google) | Sandboxed native code in the browser: an inner software-fault-isolation sandbox plus an outer OS system-call interceptor; Pepper/PPAPI as the API; PNaCl replaced `.nexe` with **LLVM bitcode** as the wire format | Native speed inside the browser's security model, without NPAPI's ambient trust | Chrome-only from the start — **no other browser ever shipped it**; PNaCl's bitcode was neither portable nor stable; deprecated in 2017, web-platform support removed in Q4 2019, inside Chrome Apps to 2021–22 ([HP 1.3–1.14][HP]) |
| 2008–2019 | **Adobe AIR** | A cross-platform desktop runtime for Flex, ActionScript and HTML apps, with sandboxing and auto-update | One runtime above three OSes, owned by the tool vendor | Survived a decade, then platform support was transitioned to HARMAN in 2019; a proprietary runtime with a shrinking vendor commitment ([HP 4.1, 4.10][HP]) |
| 2013–2028 | **Chrome Apps** | Packaged and hosted apps with a store, a launcher, a payment system and privileged APIs on the dominant browser | Fill the gaps the web had in 2013: offline, notifications, hardware access | **~1% of desktop users** in 2016, when the wind-down was announced; phased out on desktop in June 2021 (all platforms in June 2022), with a Chrome OS extension to at least January 2025, and final end of life **October 2028** — twelve years from announcement to last support ([HP 2.1–2.9][HP]) |
| 2011–2016 | **Firefox OS** (Boot to Gecko) | A phone OS where every app was a web app | The web stack as the platform; target the low-end smartphone in emerging markets | Low-end Android reached **under $50**; WhatsApp reportedly never shipped; carriers were indifferent. The carrier-phone strategy stopped in December 2015 and development ended in July 2016 ([HP 3.1–3.5][HP]) |
| 2014–2015 | **Electron** (Atom Shell) | Bundled Chromium and Node in every app, with native installers, auto-update and crash reporting | No browser gatekeeper, no plug-in, no store — the app owns its runtime and update path | Won desktop apps despite carrying a Chromium-sized runtime per app; the inversion of every choice that killed the in-browser platforms ([HP 4.5–4.6, 4.10][HP]) |
| 2014–2018 | **Flatpak and Snap** | Sandboxed, transactional Linux desktop apps; portals as the permission-broker UI; store and runtime layers | Package once, isolate apps from each other and from the system | Both survive; adoption was decided by the store and policy layer, not by the sandbox mechanism ([HP 4.7–4.10][HP]) |
| 2015 | **Progressive Web Apps** | Linkable, zero-install, earned-permission apps on the open web | The web's distribution model with app-like behaviour | The pattern that retired Chrome Apps; it named AIR, Chrome packaged apps, Firefox OS packaged apps, Cordova, W3C Widgets and Electron as "not of the web" because they give up linkability ([HP 4.9][HP]) |
| 2017–2019 | **KaiOS** | The same web-app stack as Firefox OS, on smart feature phones, with partner apps and carrier subsidies | A tier below Android, with the must-have apps delivered by partnership | **100 million devices shipped** by 2019, the third mobile OS by installed base; same technology as Firefox OS, different economics ([HP 3.6–3.9][HP]) |
| 2019–2026 | **WASI and the Component Model** | A standard system interface for WebAssembly (27 March 2019) and a typed component model (WASI 0.2 voted 25 January 2024), both built on a W3C-standard bytecode (Core 1.0 Recommendation, 5 December 2019) | Portability and capability-based security, cross-language and virtualizable | Fixes the two technical causes of NaCl's death — an unstable format and a plug-in API — but ships **no store, signing, update, revocation, UI or permission policy**, and no browser ships the Component Model natively in 2026 ([HP 5.1–5.7][HP]) |

Two lines cut across the table. **The technical failures were format and API failures**: PNaCl's LLVM bitcode was not designed to be stable, and Pepper was a single-vendor API surface in a market where the standard became the product ([HP 1.9–1.14][HP]). **The platform failures were distribution and ecosystem failures**: Chrome Apps had the dominant browser, a store and a payment system and still reached about 1% of desktop users; Firefox OS had the right technology at the wrong price tier with none of the must-have apps; KaiOS had the same technology with partner apps and an uncontested tier and reached 100 million devices ([HP 2.3, 2.9, 3.3–3.9][HP]). The sources attribute none of these outcomes to sandbox quality.

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption baked into older designs | Why it no longer holds | Consequence for Izcal |
|---|---|---|
| The browser's distribution is enough to make an app platform succeed | Chrome Apps reached ~1% of desktop users inside the dominant browser; the web's marginal cost beat a proprietary platform with a store ([HP 2.3, 2.9][HP]) | The app host is not a mass-market bet: it starts with file and directory installs, targets tasks the web does not serve, and AH-M5 requires **≥ 10† external authors**, not a launch |
| An existing compiler IR can be the distribution format | LLVM IR is not portable (different per architecture) or stable (changes with optimisation and language requirements); that is why Wasm exists ([HP 1.10][HP]) | Packages ship a **standard binary component**, and the format is published (AH-D17) rather than private |
| One vendor's browser share is enough to define a platform format | No other browser shipped NaCl; when Chrome removed it, the format had nowhere to go ([HP 1.14][HP]) | Publish the package layout, manifest and profile, or accept single-engine status explicitly; AH-D17 requires the former |
| A strong sandbox creates trust and adoption | NaCl had a formally studied sandbox and no adoption; ActiveX had no sandbox and a thirty-year cleanup ([HP 1.3, 1.15, 4.4][HP]) | The sandbox is table stakes; the product is the **permission UI, the capability review and the grant record** (6.4, 6.7) |
| Apps need a store to be discovered and installed | Chrome's store did not save Chrome Apps; Electron and PWAs distribute without one; KaiOS used carriers and partners ([HP 2.3, 3.7–3.9, 4.6, 4.9][HP]) | The directory is an ordinary website; install from file works from day one; no store gate (6.7, 6.12) |
| A preinstalled third-party runtime is a distribution strategy | Applets and Web Start died when the OS vendors and browser vendors removed the runtime and the plug-in ([HP 4.2–4.3][HP]) | The host is self-contained inside Izcal; there is no JRE-style system dependency, and no system-wide runtime is installed |
| A vendor will keep carrying the runtime indefinitely | AIR was transitioned to another vendor in 2019; Flash reached end of life; the JRE path was removed ([HP 4.1–4.3][HP]) | Izcal owns the runtime, the update channel and revocation as one system (6.7, 6.12), with a recorded kill switch |
| The web will not catch up to the platform's features | Service worker, push and the PWA pattern replaced Chrome Apps' founding gaps — offline, notifications, hardware — within three years ([HP 2.2, 2.4, 2.9][HP]) | The app host must answer "why not a PWA?" per feature (6.1): recorded and replayable execution, capability-granted local I/O, typed intents, and an owned update/revocation channel are the four the web does not provide |
| A browser can manage arbitrary local windows like an OS | The OS owns focus, always-on-top and window placement; Wayland denies focus stealing by design ([BS 49–57]) | The Stage 1 refusal stands; app windows are shell-owned with a PWA-grade contract (6.8, 6.15) |
| WASI fixes the platform problem | WASI and the Component Model fix format and sandbox; distribution, store economics, app supply, UI integration and permission policy are not in the standard ([HP 5.7][HP]) | The app host supplies the four missing pieces in the same document (6.4, 6.7, 6.8, 6.12) — the reason this is a spec and not an engine feature |
| Review makes an app safe | Every supply-chain incident in the store-and-signing record — CCleaner, ASUS Live Update, SolarWinds and 3CX — arrived through the legitimate signed update channel, and review missed compiler-injected malware ([DU 3.1, 3.5, 1.5][DU]) | Capability diffs, anti-rollback, staged growth and a disclosed kill switch; the sandbox and grants remain the boundary (6.7) |

The last two rows are the design's inheritance. WASI answers the technical half of the history; nothing in the standard answers the half that killed every platform in the chronology, and a spec that stopped at "WASI inside the browser" would be repeating PNaCl with better bytecode.

## 3. Today: the landscape and its gaps

**3.1 The standards are real, and unfinished.** WASI 0.2 (Preview 2) launched after the subgroup vote on **25 January 2024**, the first release based on the Component Model; WASI 0.3.0 followed on **11 June 2026** with async-native `stream<T>` and `future<T>`, `async func` and `wasi:io` removed, then **0.3.1 on 11 August** and **0.3.2 on 13 October 2026**; Preview 1 is still "widely used in production", so the ecosystem runs two worlds at once [SP 1.2, 1.5–1.6, 1.11][GO 6.1–6.2]. The Component Model itself is at **Phase 1** in the WebAssembly Community Group, with **no tagged releases**, no formal specification and no reference interpreter, and it cannot reach 1.0 "without native implementation in at least two browser engines"; WASI 1.0 depends on Component Model 1.0 [SP 1.13, 1.15, 2.1–2.2, 2.15][BS 27–28][BA]. The standard is real. The browser side is the empty half.

**3.2 The engine stances are precise, and they stop at the Component Model.** No engine ships the Component Model or WASI: the Component Model is the one structural Wasm feature with zero engine support [BS 15–16]. Mozilla resolved a **positive** standards position on the Component Model on **12 October 2026** — "the proposal still has a ways to go to be a standard, but we hope to contribute to that process with our prototype" — and is prototyping it; it has **no WASI position** at all [BS 19, 21]. Google/V8 has an issue "for evaluating Component Model implementation" and has made no commitment, and Chromium's feature tracker has no WASI entry [BS 23, 27]. WebKit has filed no position on either, and JavaScriptCore still leaves memory64 and relaxed SIMD behind flags [BS 15–16, 22]. WASI was scoped outside the browser and no engine has a position on it: its subgroup's scope targets "non-browser and non-JS engines", and a Component Model/WASI status update put it plainly — "we don't think about user agents" [BS 29–30]. The sanctioned browser path is transpilation: **jco** compiles a component to core Wasm plus JS glue and now emits a `"use components"` marker so engines can measure adoption [BS 27, 39]. Page Wasm itself is healthy — threads, GC, exception handling, tail calls and JSPI now ship in all three engines [BS 1–13] — but every capability a page Wasm module has is a **web API on a web origin**, not a WASI import. Outside the browser, Cloudflare Workers runs Wasm without WASI — experimental, partial syscalls, no threads — and the edge vendors (Fastly, Fermyon Spin) target servers [GO 6.3–6.4].

**3.3 Tooling sees files, not running programs.** `wasm-tools` is a static suite: parse, print, validate, objdump, compose, addr2line, and no execution, tracing or replay [GO 2.2]. `wit-bindgen` covers Rust, C/C++ and C#; Go is covered by `wasm-tools-go`; JavaScript and Python by the componentize siblings [GO 2.1]. There is no standard sampling profiler for Wasm, only engine-specific mechanisms [GO 2.5]. Chrome DevTools offers source breakpoints, scopes and the Memory Inspector, and states that timing under it is unreliable; wasmCloud's own debugging guide says you "can't attach a traditional debugger" [GO 3.1–3.2, 2.6]. A 2025 study found **7 code-generation defects** in WIT binding generators that produced crashes and silent data corruption, and Firefox's WebReplay is dead [GO 3.4–3.5]. Nothing shows what a running component asked its host for, what the host answered, or why a capability was denied [GO 3.3].

**3.4 Distribution and trust are solved in shape, not in substance.** The two-signature shape — author key for identity, platform signature after review — is where the field converged; Chrome's 2025 "Verified uploads" reached it independently, and Android's v3 proof-of-rotation is the prior art for key rotation, which Izcal deliberately replaces with transfer-as-review-event [DU 3.6, 2.4]. Signatures prove key possession, integrity since signing and continuity — not safety: CCleaner, ASUS Live Update, SolarWinds and 3CX all shipped valid, legitimate signatures, and every detection came from behavioural telemetry or incident response [DU 2.7–2.11]. Store review is triage at industrial scale (Apple: 9.1M submissions and 2.09M rejections in 2025; about 500 human reviewers (2021 trial testimony); Google: 2.36M apps prevented, 92% of reviews AI-assisted) and it did not stop XcodeGhost or the four store-and-signing incidents above [DU 1.2–1.6]. On the Wasm side, WARG's federated registry was deprecated and the Bytecode Alliance moved to OCI artifacts via `wkg`; names are an attack surface — 1,233,000-plus cumulative malicious packages by the end of 2025, and an average 209 days to public detection in a 2020 study of 174 packages — while digests and author keys are not [DU 4.1–4.7].

**3.5 Determinism exists in pieces, not as a product.** The public record for Wasm is one shipped post-mortem snapshot (Wasmtime coredump), one in-tree record/replay prototype (`RRConfig`), one academic boundary-recording technique (Wasm-R3, OOPSLA 2024) and no toolchain support [DR 3.1–3.4]. Whole-machine recorders pay **10–20×** (Microsoft TTD) and produce gigabytes per few minutes; deterministic database simulation finds real bugs at **~700× on 1,000 cores** (TigerBeetle's VOPR) and **~10:1** (FoundationDB), while whole-VM simulation (Antithesis) needs a simulated CPU, has no GPU and is not a browser feature [DR 5.1–5.7]. Nobody ships a WASI-boundary recorder, and no published number exists for its cost [DR 3.5].

**3.6 The gaps, in one table.**

| Gap | Who has it today | Why it persists | What this spec does |
|---|---|---|---|
| A program with no ambient authority and a real UI | Native sandboxes and installed web apps, separately | The browser has no program model; the OS has no broker | Two sandboxes and typed intents (6.8) |
| Capabilities a person can see, and revoke per use | OS dashboards and Chrome site settings | Dashboards are post-hoc and coarse, show grants not use, and have no denial log; only 17% of participants paid attention to Android permission warnings during installation in the 2012 SOUPS study | Install view, live tickets and a reason-coded denial log (6.4, 6.13, AH-12) |
| A portable package with a reviewable capability surface | Flatpak, Snap, MSIX, app stores | Native capabilities are not computable from the artifact | Manifest ceiling plus computed imports (6.4, 6.7) |
| Record/replay that opens nothing | rr (native), Antithesis (VM); nothing for WASI | WASI runtimes do not record; browsers do not run WASI | Host-call log and a virtual filesystem/network (6.13) |
| Host-boundary observability for Wasm | Nothing; DevTools stops at memory and instructions | No embedder owns both the boundary and the log | The IIP `app` domain (6.13, AH-12) |
| A named, conformance-tested WASI profile | "WASI" varies by host: Workers' partial syscalls, Preview 1 vs 0.2 vs 0.3 | No browser vendor has committed to one | `izcal:app@0.1` and AH-5 (6.5) |
| Typed, auditable agent access to a tool | Pixel automation, and MCP tools with no capability record | Agents were bolted onto UIs, not onto capability systems | The intent registry with per-task grants (6.14) |
| A UI/backend boundary that can state what each side reaches | Electron-like apps, where both share one process and ambient authority | The UI and the logic were never separated by a typed, recorded channel | The sealed `app://` UI plus intents (6.8) |

The gap is not a missing standard. It is that no embedder combines a capability broker, a typed runtime boundary, a package pipeline and a recording substrate in one product. Izcal has built each piece for other reasons; this spec joins them.

## 4. First principles

### 4.1 Users' real tasks

A person does not want "a WASI runtime". They want to:

1. **Run a program they chose**, offline, without giving it their whole home directory or their machine's ambient authority.
2. **Know what it can reach** before and after installing it: which folders, which hosts, which devices, which other apps.
3. **Take their data with them**: a program's data is files in a folder they picked, not a proprietary store they cannot export.
4. **See what it did** when something goes wrong — including after the fact, from a recording.
5. **Let a tool be driven by another tool** (a script, another app, later an agent) through operations the tool declares, not by faking clicks.

A developer wants: one portable artifact, one capability model, the same artifact on every OS, a debugger that can show a host call and a denial, and no per-OS notarisation pipeline. An agent wants: typed operations with schemas, and a grant that names the person who gave it ([H3 B9][H3]).

### 4.2 The platform primitives, set out

| Primitive | What it actually guarantees | What it does not |
|---|---|---|
| Core Wasm | Deterministic computation over a validated module; no ambient authority; memory isolated by bounds checks | Nothing about I/O, time, randomness, files or the network |
| WASI 0.2 worlds | Typed imports that *name* authority: clocks, random, preopens, sockets, HTTP | No guarantee that an embedder mediates them well; no origin model; no UI; no package format |
| Component Model | A composition model (worlds, interfaces, resources) and a canonical ABI with one copy per crossing ([BA][BA]) | No linking policy, no distribution, no signatures in the core spec; async adds roughly 3.5× overhead ([BA][BA]) |
| WIT | A machine-readable interface language: packages, worlds, resources | No version resolution policy; no notion of a trusted publisher |
| Izcal broker | The only grant authority; prompts on OS input; the prompt ledger | Not a runtime; it cannot execute a guest |
| Izcal component host (JS-D10) | Izcal's Wasm engine with a WASI embedding, tier-P build, one process per extension origin, jitless | Built for extension agents, not for third-party programs with files and windows |
| Izcal storage service | The only process that touches files (G150); per-container keys; quota and eviction | Has no WASI filesystem surface; picked-file I/O is a broker-minted capability into its file path ([SG 6.14][SG]) |
| Izcal devices | Broker-signed, expiring device tickets; arbiter/session split; virtual device for replay | No app identity; grants are keyed by frame origin and top-level site |
| Izcal extensions | Signed packages, review decisions in a public log, capability growth staged, no install telemetry (EX-D10) | Extensions extend pages; they are not programs with a lifetime, a window or a filesystem |

The primitives compose. The design work is in the joins: identity, lifetime, capability translation, UI, recording and trust.

### 4.3 Izcal's principles that bind this design

- **P1, isolation.** One app never shares a process with another app or with web content. Its renderer (if it has one) is keyed like any renderer; its component process is keyed by app and container.
- **P2, zero ambient authority.** The guest starts with no file, socket, device, clock or random capability beyond what its manifest declares and the broker grants.
- **P3, the person decides.** Every capability that reaches outside the app's own sandbox is a broker prompt with a consequence line and a ledger entry, on broker-observed input.
- **P5, deterministic by construction.** The guest is deterministic; every host call is an input and is recorded.
- **P6 and B10, AI off by default.** The intent registry, the agent path and the model host do not exist with AI off ([H3 B10][H3]).
- **M-1.** Izcal writes the component decoder, the WIT parser, the linker, the canonical ABI and every WASI world from the specifications; no Wasmtime, no wasm-tools, no jco in the shipped path.

### 4.4 Assumptions inherited, kept or broken

| Assumption from existing app platforms | Decision | Why |
|---|---|---|
| An app is a website with a window | **Break** | That is an installed site; it has the web's origin model and the site's availability, not a program's capabilities |
| An app is native code with OS permissions | **Break** | Ambient authority, per-OS packaging, an updater the user did not choose |
| An app runtime must ship its own browser (Electron) | **Break** | Izcal is the browser; the component host is the runtime and the renderer is the UI |
| A sandboxed app cannot have a real UI | **Break** | The UI is the engine's own strength: a sealed HTML document in an own-origin renderer, talking to the component over typed intents |
| A package store must audit code | **Partly** | Review the **capability diff** mechanically and the identity by hand; the sandbox, not the review, is the boundary |
| WASI apps are headless servers | **Partly** | Service apps are headless; windowed apps pair a component with a sealed UI |
| Raw sockets are needed for real apps | **Break** | HTTP through the partition service is enough for v1; raw sockets bypass routes, LNA, blocking and the partition key |
| Component Model linking means dynamic linking | **Break** | One package is one component; dependencies are linked at build time by the publisher and covered by the signature |
| An app's data belongs to the app | **Break** | Data lives in a storage bucket keyed by app and container, exportable, deletable, and in user-picked folders the person owns |
| A browser app platform must manage windows like an OS | **Break** | Chrome Apps' window stack and launcher died with the platform; the browser owns its own windows and treats OS window policy as a request — Wayland denies always-on-top and focus stealing outright ([BS 49–57]) |
| The app host is the web's route to running components | **Break** | Web content gets jco-transpiled core Wasm on a page origin; the app host is not reachable from a page and has no web API (6.15) ([BS 27, 39]) |
| A WASI world is a capability model | **Partly** | A world import is ambient to whatever host is linked; Izcal adds identity, broker tickets, prompts, packaging, updates and admission on top (6.4, 6.7, 6.15) ([BS 35–37]) |

**No assumption is kept whole.** The method asks a keep-or-break question of each; here the closest to a keep are the four **Partly** rows — capability review, headless service apps, WASI's handle model and the app's ownership of its data — and each is kept only with the qualification in its row. Every assumption that would make the app host a smaller project (native packaging, an Electron-style runtime, web origins as app identity, a display-protocol UI, raw sockets, OS-style window management) is broken, and the reasons are the ones the history in 2 and 5 supplies.

## 5. Lessons learned and how Izcal applies them

| Project or precedent | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| **Native Client and PNaCl** (2008–2021) | A researched, two-sandbox native-code runtime inside Chrome; PNaCl's wire format was LLVM bitcode; no other browser ever shipped it, and it was removed after Wasm arrived ([HP 1.9–1.14][HP]) | The **distribution format is the fragile half**: a compiler IR is neither stable nor portable, and a single-vendor format dies when the vendor loses patience | The package carries a **standard binary component** and the format is published (AH-D17); the engine implements the specification, not a vendor dialect (6.6) |
| **Firefox OS and KaiOS** (2011–2020) | The same web-app stack failed on low-end smartphones (Android under $50, no WhatsApp, indifferent carriers) and succeeded on feature phones with partner apps and carrier subsidies — 100 million devices ([HP 3.1–3.9][HP]) | The ecosystem problem is not solved by the runtime; **tier and partners decide**, and a pure-web platform must beat the incumbent's price at the same features | The app host targets tasks the web does not serve rather than a price tier (6.1); AH-M5 requires external authors before the platform is called viable; distribution partnerships are a milestone question, not marketing |
| **Java applets and Web Start** (1995–2018) | Zero-install centralised distribution that depended on a **preinstalled third-party runtime** and a browser plug-in; both were removed by their owners ([HP 4.2–4.3][HP]) | A runtime that depends on someone else's runtime dies with it | The app host is self-contained in Izcal's own binary (6.2); nothing is installed system-wide, and no JRE-style layer may appear |
| **ActiveX** (1996–2022) | Native COM controls loaded in-process with the page's full authority; retired over decades, with IE mode to at least 2029 ([HP 4.4][HP]) | Full local power without a sandbox creates a generation-long cleanup bill | Every app capability is a broker-issued grant, never ambient (6.4, AH-1); no in-process native code, ever (M-1, 6.17) |
| **Adobe AIR** (2008–2019) | A proprietary cross-platform runtime with sandboxing and auto-update; support was transitioned to another vendor after a decade ([HP 4.1][HP]) | A runtime survives only while its owner keeps paying for it; **the update and trust story must belong to the platform that ships it** | Izcal owns the runtime, the signing service, the directory and the kill switch as one system (6.7, 6.12); the dependency is Izcal's own engine, not a third party's |
| **Flatpak and Snap** (2014–) | Sandboxed, transactional Linux app systems; both survive, and adoption tracked the **store and policy layer**, not the sandbox mechanism ([HP 4.7–4.10][HP]) | The sandbox is necessary; the store, signing and permission policy decide adoption | The directory, review, signing and revocation are in this spec (6.7, 9.2), not left to an ecosystem to invent |
| **Progressive Web Apps** (2015–) | Linkable, zero-install, earned-permission apps; the pattern that made Chrome Apps redundant ([HP 2.4, 4.9][HP]) | The web's install and permission model is the **incumbent**, not an ally; a new app platform must justify itself per feature | 6.1 answers "why not a PWA?" with the four things the web does not provide; install is explicit and never store-gated; the earned-permission pattern is reused, not reinvented (6.4, 6.7) |
| **Format governance** (all of the above) | Every platform in this table either published a format others could implement (the web), or depended on one vendor's continued investment until that vendor stopped (NaCl, Chrome Apps, AIR) ([HP 1.14, 2.9, 4.1][HP]) | **Publish the format, or knowingly accept single-engine status** — and know that every failed platform in this table depended on one vendor's continued investment, though that is not the only cause of any of them | AH-D17: the package layout, manifest schema and `izcal:app@0.1` profile are published as an open, versioned MIT specification, so another host could implement them; the reversal trigger names the twelve-month test |
| **Chrome Apps** (2013–2022; kiosk tail to 2028) | A browser-native packaged-app platform with real windows, offline storage, hardware APIs and an app launcher; Google announced its end in January 2020, ended desktop support in June 2021 and all-platform support in June 2022, with Chrome OS extended and a kiosk tail to 2028 ([BS 42, 44, 49–50]) | A browser app platform dies when (a) the web platform catches up for most uses and (b) the vendor must maintain a parallel window, packaging and API stack; the launcher's fate tracked the platform's ([BS 50, 67]) | The app host is **not** a new platform: no storefront, no launcher process, no window manager, no general native-binary support (6.1, 6.12, 6.20). Its windows are browser windows; its launch surface is the shell's app panel; its package pipeline is the extensions directory's (A23) |
| **Flash** (until 2020) | A capability-rich plugin with its own runtime, updater and security model; browser vendors ended support on 31 December 2020 ([BS 41]) | A second execution stack with its own patch channel becomes a second security program; the platform that hosts it inherits the patching burden | The app host is a **configuration of Izcal's existing component host** (6.2), not an embedded third-party runtime: one engine, one updater, one signed-update channel for the host itself (M-1, 6.7) |
| **The extension model** (current) | Survived where app platforms died: review-gated stores, mandatory signing, a manifest of permissions, a kill switch ([BS 43]) | What survives is the **governance stack** — review, signing, manifest, revocation — not the runtime novelty | Apps share the extensions directory, signing service, review process and decision log (6.7, A23); the manifest ceiling is the app's permission declaration, checked mechanically against the component's imports (G225) |
| **Electron and PWAs** (2013–) | Two survivors split the problem: Electron is a native shell with rich OS windows and a whole browser inside; PWAs are browser windows with a thin contract (display modes, title-bar overlay) ([BS 51–56]) | Rich OS integration belongs in a native shell; a browser's honest offer is rendering plus a **thin, request-based** window contract — Wayland denies always-on-top and focus stealing outright ([BS 57]) | The app window is PWA-grade with app chrome; OS integration is requested through the OS's own consent; no window-manager promises (6.8, 6.12) |
| **WASI in the browser** (as of 2026) | WASI was scoped outside browsers by its own subgroup ("we don't think about user agents"), has no origin model, no permission UI, no package format, no update channel and no store policy; no engine ships it or the Component Model ([BS 29–30, 34–37, 40]) | A standard interface is not an app model. Hosting WASI means **supplying the five missing pieces** — permission UI, identity, packaging, updates, admission — not shipping a runtime | The app host supplies exactly those: broker tickets and prompts (6.4), `app://` identity (6.3), signed packages (6.7), an update path with anti-rollback (6.7), and the DV-D9 admission rule (6.15) |
| **jco transpilation** (current) | The sanctioned browser path compiles components to core Wasm plus JS glue on a page origin, and now carries a `"use components"` adoption marker ([BS 27, 39]) | For **web content**, the ecosystem runs components through the page's origin and capabilities — and that is the correct compatibility posture, not a deficiency to fix | The app host does not replace jco for web content; it has no web API and no page reachability (6.15). The two paths coexist: pages use jco, installed apps use the app host |
| **Native app stores** (2008–) | Review at industrial scale is triage (Apple: 9.1M submissions and 2.09M rejected in 2025; about 500 human reviewers (2021 trial testimony)); every major supply-chain compromise shipped a valid signature, and every one was caught by telemetry, a customer report or incident response, not by review ([DU 1.2–1.6, 2.7–2.11, 5.1–5.2]) | Signatures prove possession, integrity and continuity, **not safety**; review bounds the obvious and buys response time; the update channel is the attack surface | Two signatures with the identity pinned to the author key; ownership transfer is a review event; a public decision log; no install telemetry, so the substitutes are named and committed (6.7, 10) |
| **Whole-machine record/replay and deterministic simulation** (2010s–) | rr records at the syscall boundary; Microsoft TTD pays 10–20× and gigabytes per few minutes; TigerBeetle simulates at ~700× on 1,000 cores and FoundationDB at ~10:1; no WASI-boundary recorder ships ([DR 3, 5]) | Record the **smallest boundary that explains the program's external behaviour**; replay must open nothing real; simulation is a test harness, not a product mode | The recording unit is the WASI host call, not the instruction stream; entropy is recorded as values; replay opens zero files, sockets and devices (6.13, AH-15) |
| **Agent permission work** (2025–2026) | The field converges on short-lived, per-action approvals, signed single-use consent and presence signals (relock-on-input); typed app capabilities are shipping in agent hosts ([BS 61–65]) | An agent acting for a person must be a **designed principal** with per-action grants, not a script that clicks | Typed intents, per-task grant sets, person-present writes (CON-R1, A-SH-12), and a recorded grant per call (6.14) |
| **Wasm tooling** (2017–2026) | wasm-tools is static; no sampling profiler standard; DevTools declares timing unreliable; wasmCloud says a debugger cannot be attached; WebReplay is dead ([GO 2.1–2.6, 3.1–3.6]) | The **host boundary** is the observable seam, and no embedder owns it today | The IIP `app` domain records and presents host calls, tickets and denials (6.13, DT-D5, AH-12) |

## 6. Design

### 6.0 Architecture at a glance

```
                      broker (only grant authority, prompt owner, app registry)
                        │ app launch · install grant · tickets · intent routing
        ┌───────────────┼───────────────────────────┬──────────────────────┐
        │               │                           │                      │
  app host          app UI renderer            storage service        partition service
  (component host   (own-origin `app://`       (only file toucher,    (app's own key:
   app config)       renderer, sealed UI)       app bucket + grants)   no cookies, routes,
  per (app,          per (app, container)                              LNA, bad ports)
   container)            │                           ▲                      ▲
  W0/W1/W2 tier          │ typed intents             │                      │
  tier-P engine          └──────────► broker ────────┘                      │
        │                                                                    │
        └── wasi:http ─────────────────────────────────────────────────────►─┘
        └── device tickets ──► device service (DV-D2)         │
        └── host-call log ──► recording / replay host ◄───────┘
```

The app host is **one process per (app, container)** running Izcal's own Wasm engine in its **component configuration**. It holds no OS handle. Every capability it exercises is a typed request to a service that the broker has already authorized, or a broker-signed ticket it presents to that service. It has no DOM, no JavaScript engine, no web API and no window.

An app that has a user interface pairs the component with a **sealed UI**: HTML, CSS and project JavaScript from the same signed package, served on `app://<id>` and rendered by an ordinary renderer keyed by the app origin and container. The UI holds the web capabilities of its own origin and nothing else; the component holds the app's WASI capabilities. The only channel between them is a **typed intent channel** routed by the broker.

The central idea: **an app is a signed package with two sandboxes and one capability surface, and the broker is the only authority between them.**

### 6.1 Why an app host at all

Three alternatives, and why none is this design:

| Alternative | What it is | Why not |
|---|---|---|
| **Installed web app only** | A site with a standalone window | The site's origin, the site's availability, no filesystem beyond the picker, no deterministic program, no portable artifact |
| **Native app with OS sandbox** | A flatpak/snap/MSIX package | Ambient authority at the OS level, per-OS packaging and signing, an updater the browser does not control, no host-call recording, no typed intents |
| **Third-party Wasm runtime in a renderer** | Wasmtime/wasmer embedded in the browser | M-1 2.3.0 (a Wasm engine is engine work); a second heap and code generator; no tier discipline; JS-D10 removed the dependency for extensions and this spec must not reintroduce it |
| **jco-transpiled components in a page** | Components compiled to core Wasm plus JS glue, run as page script — the ecosystem's sanctioned browser path [BS 39] | The page's origin and capabilities; no filesystem, no process lifetime, no isolation from the site, no recording at a WASI boundary |
| **The app host (this spec)** | Izcal's component host in its app configuration, plus `app://` UI and a package pipeline | Costs a component decoder, a WIT model, a canonical ABI and a WASI profile; all four are Izcal's anyway under M-1 |

### 6.2 The process model

**One process kind, four roles.** The component host is already a process kind after the extensions amendment: the security scope counts it, with one threat model. The app host adds **three configurations** to the same kind — the same source and IPC schema, and a **larger review surface than the extension configuration**: new parsers (component, WIT, canonical ABI, package) and a tier-P codegen build — exactly as the device service is one kind in the arbiter, session and enumeration configurations ([DV 6.1][DV]).

| Configuration | Binary | Runs | Tier ceiling | Links |
|---|---|---|---|---|
| **Extension** | `hg-ext-host-main` | Extension background agents (JS, jitless) | Jitless | `hg-wasm` (tier P), `hg-heap`, `hg-js-*`; no codegen |
| **App** | `hg-app-host-main` | Third-party components | Baseline (W0/W1) by default; Full (W2) and Jitless per policy | `hg-wasm`, `hg-component`, `hg-wit`, `hg-wasi-host`; tier-P build of `hg-codegen` when the ceiling allows W1/W2 |
| **App replay** | `hg-app-host-main --replay` | A recorded app | Jitless by default (REC-R6); recorded tier only in the engineer-only engine-triage mode | As app, with `hg-app-replay` in place of the service clients; no codegen on the default path |
| **Install verify** | `hg-app-host-main --verify` | One package, at install | N/A (parse and validate only) | `hg-app-package`, `hg-component`, `hg-wit`; no codegen, no service clients, exit after one summary |

**The review delta is named, not waved.** Compared with the extension configuration, the app configuration adds four attacker-facing parser families — the ZIP package and canonical digest (`hg-app-package`), the component binary (`hg-component`), WIT text on the build and test path (`hg-wit`) and the canonical-ABI lift/lower path — plus the tier-P codegen build and the WASI host's own argument decoders. The component-host threat model gains one app-profile section with a **subsection per family**, and each subsection states the input, the parse stages, every bound, the memory-safety argument (safe Rust, no `unsafe` in the parsers; G223/G229), and the failure mode (typed refusal, never partial acceptance). Each family has its own fuzz target (AH-4) and its own malformed-corpus seed set. That is the precise sense in which this is one kind with a larger surface, and it is why AH-D1's split trigger is measurable rather than a matter of taste.

**No new process kind.** The app host is not the 38th kind. After the devices amendment the map has **37 kinds and 39 threat-model files** ([DV 9.5][DV]); this spec adds no kind and no file: the component-host threat model gains an **app-profile section** covering the app, replay and install-verify configurations, and the security scope's 6.3 row for the component host gains the profile.

Why not a separate kind: the process has the same trust shape as the extension configuration (parses untrusted module bytes, holds no OS handle, all authority through typed requests), the same binary family, the same cage and the same tier rules. A separate kind would duplicate the threat model and the review surface for no new boundary. The reversal trigger is in AH-D1: if the app profile's threat surface grows a class the extension profile does not have (for example a display protocol or a GPU device), the **threat-model file** splits — still not a new process kind unless the sandbox tier changes.

**Lifetime.** The broker launches `hg-app-host-main` when an app's component is first needed: a windowed app's launch, a service app's first intent, or an agent's invocation with a grant. It exits after **30 s†** with no live intent and no open ticket, and always when its last window closes and its last ticket expires. A profile that runs no app runs no app-host process; the footprint row is zero (6.16).

**Keying.** One app host per (app id, container). Apps are principals like sites: I-1's rule applies unchanged, and the app's storage key and network partition use the same pair. Two apps never share an app host; an app and web content never share a process.

**What the process holds.** A 16 GB† cage with a trusted region, the app's instantiated component and its linear memory (bounded by quota, 6.11), the tier-P engine and (when the ceiling allows) the tier-P codegen build, the canonical ABI, and the WASI host implementation. It holds **no** OS handle, no socket, no file descriptor, no device handle, no DOM, no JS heap, no window and no broker endpoint other than the typed app endpoint set.

**Rule of Two.** The app host parses attacker-controlled bytes (component binaries, WIT types, canonical ABI payloads) in a memory-safe tier-P build, and it holds no OS authority. The process that holds OS handles (storage, network services, device arbiter) parses none of the guest's bytes. The broker decides and parses no guest format (IPC-R15's rule). The three-way split is the same one the devices spec applies ([DV 6.1][DV]).

### 6.3 Identity, origin and lifetime of an app

**App id.** An app has a **stable id assigned when it is first signed**: a 128-bit random value generated by the signing service (or by the author in developer mode), encoded as 26 base32 characters and carried in the manifest as `app_id`. The id is **not derived from the key**, so a key can rotate without renaming the app. The **author key** is the Ed25519 key that signs the package and is recorded in the manifest alongside the key history (6.7); the platform signature is separate, and identity never derives from it, so a platform-key incident cannot re-point an app (the 2019 extension lesson, EX-D10). What the person sees as the publisher is the current key fingerprint, shown with the app id in the install prompt, the app panel and the directory.

**Name and icon are directory properties, not manifest claims.** People do not compare fingerprints, so a name the package supplies would be an impersonation tool: a malicious signed app could present a known app's name and icon and the install prompt would show them as the app's own. Display name and icon are therefore bound to the **directory entry and the author key**: the directory stores them against the app id, refuses an entry whose Unicode-normalized name duplicates an existing one or whose icon hash matches another entry's, and the shell shows the directory name in the app window chrome, the app panel and the prompt. A manifest's name is used only for a developer-mode or file install with no directory entry, and the broker labels that app **unverified publisher** with the file name; the **verified publisher** badge requires the platform signature. The rule is the extension directory's, applied with app ids (EX-D10), and the impersonation risk with its domain-claim trigger is in 10.

**Origin.** An app's UI runs on `app://<id>`. The scheme is an internal scheme: web content cannot navigate to it (a link to `app://` is refused as a non-web scheme at navigation), agents cannot reach it (AGT-R3), and an `app://` document cannot be fetched across origins. It is a tuple origin; its app data has the storage key (app id, container) in the app bucket, and it has no web storage (6.8).

**Not a web origin.** An app has no registrable domain, no cookies, no `document.cookie`, no referrer, no same-site relationship to any website. Its HTTP requests carry no cookies and no web credentials; authentication is the app's own, through its intents or its HTTP headers. This is a privacy property, not a limitation: an app cannot ride the person's web session, and a website cannot recognise the app as a visitor beyond the request itself.

**Containers.** An app is installed per profile and may be launched in any container. Its data, grants and tickets are per container; a container's app data is cleared with the container. The app's UI renderer is keyed by (app origin, container), so two containers never share a renderer.

**Lifetime and state.** The component's lifetime is a **session**: launch to exit. There is no bfcache, no suspend spike, no snapshot of app memory. Durable state lives in the app's storage bucket (6.11) or in files the person granted; everything else is lost at exit, by design. A windowed app's UI may outlive the component (the component restarts on the next intent), and the broker shows a typed "component exited" state in the app panel.

### 6.4 The capability model

**Three layers, and only the third is authority.**

1. **The manifest ceiling.** The package declares the app's capabilities in a machine-readable manifest: filesystem scopes, network destinations, device families, host functions, piped stdin, intents it exports and accepts, whether it has a UI, and its resource limits. The manifest is a **ceiling**, never a grant.
2. **The computed surface.** The component's own imports are read from its type section and **must be covered by the manifest**. The loader refuses a component whose imports exceed the declaration — the same "declared ≥ computed" discipline the coding standard applies to tiers (CS-ST-10). The pair (declared, computed) is the review artifact.
3. **The grant.** At install and at runtime, the broker grants specific capabilities, as **tickets** where a different process verifies them and as **broker-held grants** where the broker is the only verifier.

**What WASI supplies, and what it does not.** WASI's model is the right foundation: access is an **unforgeable handle**, never a name, and the design principles state that it "aims to have no ambient authorities" ([SC 1.1–1.3]). But a world import is ambient to whatever host is linked — the host's act of providing the import is the grant ([SP 2.5]). WASI standardizes neither a **budget** nor a **descriptor-revocation** operation ([SC 1.9, 4.4]). Izcal supplies both: fuel and epoch budgets are part of the grant (6.13, 6.16), and revocation is ticket closure at the next call or **destruction of the app host** — the only revocation WASI can express ([SC 7.5]).

**The manifest, as a sketch** (names illustrative, not an interface freeze; the canonical serialization is `hg-app-manifest`'s and is inside the package digest):

```json
{
  "schema": "izcal.app.manifest/1",
  "name": "com.example.diff",
  "version": "1.4.0",
  "author_key": "ed25519:9f2c...",
  "profile": "izcal:app@0.1",
  "component": { "path": "component.wasm", "sha256": "b6a1..." },
  "ui": { "entry": "ui/index.html", "network": false },
  "capabilities": {
    "filesystem": [{ "preopen": "/data", "mode": "rw" }],
    "directories": [{ "scope": "documents", "mode": "ro", "prompt": "app-file-read" }],
    "network": [{ "hosts": ["api.example.com"], "transport": "https" }],
    "devices": ["hid"],
    "host": ["clipboard-write", "notify", "open-url"],
    "stdio": ["stdin"]
  },
  "intents": {
    "export": { "version": "1.0.0", "writes": true }
  },
  "limits": { "memory_mib": 256, "intent_deadline_s": 10 }
}
```

The manifest names only capabilities; the **computed imports** are the other half of the pair, and an install is refused when the component imports more than the manifest declares (G225). A manifest that declares more than the component imports is allowed — a ceiling, not a promise — and the install prompt shows what the component actually uses, marked against the ceiling. A `limits` value may only lower the broker's policy value: `intent_deadline_s` above **10 s†** is refused unless the app holds the `app-long-intent` grant (6.14).

**The ticket format.** An app ticket is the device ticket's shape, not a new one ([DV-D2][DV]): versioned, fixed-layout, Ed25519-signed by the broker, bound to the app id, container, capability class, scope digest, sequence and expiry. Filesystem directory tickets and network route tickets are verified by the storage service and the partition service respectively; the app host cannot mint, only present. Ticket expiries are DV-D2's: **24 h†** for a session ticket and **60 s†** for a one-shot open, renewed without a prompt while the grant lives.

**Grant lifetimes are explicit, and no class defaults to "until uninstall".** The install prompt offers a lifetime per class, and the app panel shows and changes it:

| Class | Default | Remembered option |
|---|---|---|
| **Session** | Ends when the app host exits, the window closes and no intent remains, or the person revokes | — |
| **Remembered** | — | Expires **90 days†** after the grant, or on revoke; use before expiry does not renew it silently, and a renewal is a visible re-grant (CON-R9's cap, applied to apps) |
| **Install-lifetime** | Only for `app-notify`, `app-open-url` and `app-autostart`, which are meaningless as session grants | — |

High-risk classes (`app-file-read`, `app-file-write`, `app-clipboard` read, device writes) are **session by default** and are never install-lifetime. A remembered grant for a high-risk class is offered only with A-SH-12's person-present evidence, and its 90-day† expiry cannot be extended without a new prompt. App-private storage is not a grant at all: it is the app's bucket, and it exists while the app is installed. Revocation is immediate and closes live tickets at their next use.

**What is never ambient.**

| Capability | Grant unit | Prompt class | Verified by |
|---|---|---|---|
| App-private storage | The app's bucket, quota-charged | None (implicit in install) | Storage service |
| A user directory | One directory, read or read-write, per grant | `app-file-read`, `app-file-write` (high risk) | Storage service, on a ticket |
| Network destination | A host or origin set, per app and container | `app-network` (medium) | Partition service, on a ticket |
| A device family | Family plus device identity digest, per open | `device-*` (DV-D4) | Device arbiter, on a ticket |
| An intent | Sender class (shell, another app, agent) and intent name | `app-intent` (medium; high for writes) | Broker |
| Launch at login | One app, one container | `app-autostart` (medium) | Broker |
| Clipboard read/write | Per app, per container | `app-clipboard` (high for read, medium for write) | Broker |
| Opening a web page | A destination class, per app | `app-open-url` (medium) | Broker |
| Notifications | Per app, per container | `app-notify` (medium) | Broker |
| Piped stdin | One file, one launch | `app-stdin` (low) | Broker |
| Long intent | One intent declaration, above 10 s† | `app-long-intent` (medium; policy ceiling 5 min†) | Broker |

**Prompt classes.** The classes above are registered in the security scope's consent section (9.5). Each is drawn by the prompt renderer, approved only on broker-observed input (CON-R1), and recorded in the prompt ledger (SH-D11). High-risk classes (`app-file-read`, `app-file-write`, `app-clipboard`, device-write) follow A-SH-12: an AT-invoked approval requires person-present evidence.

**Revocation.** Revoking a grant closes every live ticket at its next use and terminates the app host if the app holds no other grant of that class. A revoked directory grant makes the next WASI call fail with `EACCES`; the app sees a typed error, not a crash. Uninstall revokes everything and deletes the app's bucket after a confirmation that names what will be lost (6.12).

### 6.5 The WASI profile: `izcal:app@0.1`

The profile is a **named world** with pinned interfaces. It is a subset of WASI 0.2, chosen world by world from the authority each opens. A component that imports anything outside the profile is refused at load; the error names the import and the profile line that excludes it.

| Interface | Ships | Authority it names | Decision |
|---|---|---|---|
| `wasi:cli/command@0.2.0` (`run`, `exit`) | **Yes** | None; the entry point | One export per component |
| `wasi:cli/environment@0.2.0` | **Yes**, host-provided | None | `IZCAL_APP_ID`, `IZCAL_CONTAINER`, `IZCAL_DATA_DIR` (the value is always `/data`, never a host path), `IZCAL_LOCALE`; no secrets, no host environment |
| `wasi:cli/stdin@0.2.0` | **Closed** | `app-stdin` (low) | Reads return end-of-stream unless the app declares `stdio` in its manifest and the person pipes a file in through the shell's open-with path; the pipe is one launch's grant, never a standing capability |
| `wasi:cli/stdout@0.2.0`, `stderr` | **Yes** | None | To the app log and the IIP `app` domain; capped at 4 MiB† per session, rotated |
| `wasi:clocks/monotonic-clock@0.2.0` | **Yes** | None | Substrate clock; recorded |
| `wasi:clocks/wall-clock@0.2.0` | **Yes** | None | Substrate clock; recorded, never shifted: an app that gets the wrong local time is a bug in the app, not a hook in the browser |
| `wasi:random/random@0.2.0` | **Yes** | Entropy | `hg-entropy` crypto; the draw is a **sensitive internal draw** (recorded, encrypted at rest, redactable) |
| `wasi:random/insecure-random@0.2.0`, `insecure-seed` | **Yes** | None | Recorded as values |
| `wasi:filesystem/preopens@0.2.0`, `types@0.2.0` | **Yes** | A directory ticket per preopen | `/data` is the app's bucket; `/grant/<n>` are person-granted directories; no other path exists |
| `wasi:sockets/network@0.2.0`, `tcp`, `udp`, `ip-name-lookup` | **No** | Raw sockets | Refused. HTTP only (6.9). A guest that imports them fails the load check |
| `wasi:http/outgoing-handler@0.2.0`, `types@0.2.0` | **Yes**, client only | A network ticket per destination set | Through the app's own partition service; no cookies, no referrer |
| `wasi:http/incoming-handler@0.2.0` | **No** | A listening socket | Refused: no app listens on a port in v1 |
| `wasi:io/error@0.2.0`, `poll@0.2.0`, `streams@0.2.0` | **Yes** | None | Pollables over the broker's typed endpoints; the guest stays single-threaded |

**Why 0.2 and not 0.3.** The profile pins WASI **0.2.12** (2 June 2026), the last patch of the 0.2 line; the interface versions remain `@0.2.0` [SP 1.10]. WASI 0.3.0 (11 June 2026) moves HTTP and streams onto `stream<T>`/`future<T>` with the async ABI, which the Bytecode Alliance measured at roughly **3.5× overhead even on synchronous call paths** ([BA][BA]), and it makes the host a task scheduler whose wake-ups and completions would all have to be recorded. The 0.2 worlds are stable, poll-based and a smaller recording surface; 0.3.2 shipped eight days ago and the train continues through 0.3.9 in December 2027 [SP 1.11]. The component binary format and the WIT type model are the same in both; `izcal:app@0.2` adds the 0.3 worlds when the async ABI's overhead is measured on Izcal's engine and the worlds stop changing. The profile is versioned; an app declares the version it targets.

**The host's own interfaces are named too.** WASI does not describe everything a browser app needs, so the profile adds one host package, `izcal:app/host@0.1`, versioned with the profile and reviewed with it. Each function is a capability with a prompt class; none is ambient.

| Function | Effect | Grant |
|---|---|---|
| `open-url(url)` | Opens a new tab in the app's container, `http(s)` only, after the broker's scheme check; never navigates the app window | `app-open-url` (medium; granted at install for a manifest-declared destination class) |
| `clipboard-read()` | Reads the clipboard as text, after the shell's clipboard rules | `app-clipboard` (high) |
| `clipboard-write(text)` | Writes text; the person sees the shell's copy indicator | `app-clipboard` (medium for write) |
| `notify(title, body, tag)` | Posts a notification through the shell's notification path with the app's identity | `app-notify` (medium) |
| `log(fields)` | Structured app log fields to the app log and the IIP `app` domain | None |

The host package is deliberately small. A function is added only when a real app corpus needs it and the capability can be granted, recorded and denied (AH-6); a "generic browser API" in this package would undo the point of the profile.

**What is excluded from the Wasm profile itself.** No threads or shared memory (no `SharedArrayBuffer` equivalent, no atomics across instances); no relaxed SIMD (nondeterministic by design); no tail calls, exception handling or GC types in `@0.1` unless the engine's conformance target requires them; no memory64 in `@0.1` (a 32-bit memory is 4 GiB, more than the app's quota). The exclusion list is a compatibility cost recorded in 10.

**The host is threaded; the guest is not.** The guest sees one thread and no shared memory. Inside the app host, one **guest thread** executes the component, and a **supervisor thread** owns the broker endpoint, the ticket refresh, the deadline and the cancel flag. A host call that needs a service posts a typed request and parks the guest thread on a completion; `poll` waits on the app host's completion queue and the cancel flag together, so a revocation or a window close wakes a blocked guest instead of leaving it parked. The threaded host does not relax the guest profile: no guest thread, no `SharedArrayBuffer` equivalent, no atomics across instances (6.5).

**The host side is not WASI.** WIT describes the guest's interface; it does not describe how the host enforces it. The app host's implementation of each world is Izcal's, anchored to the pinned WASI specification, and every call passes through one funnel: `CallContext { app, container, call_id, ticket?, scope, deadline }`. A host function without a `CallContext` is a compile error (G222).

### 6.6 The Component Model subset

**What the engine implements.**

- **The component binary format**: sections, type definitions, imports and exports, instantiations, aliases and canonical function declarations, decoded by `hg-component` from the pinned specification documents. Validation is a separate pass, and the decoder treats every length as untrusted. Because the Component Model has **no tagged releases**, the pin is by **content digest**: the SHA-256 of `Binary.md`, `WIT.md`, `CanonicalABI.md` and `Explainer.md` is recorded in the anchor table with the retrieval date (21 October 2026), and the dated snapshot is archived. A `// spec:` anchor cites the digest, so a silent upstream edit fails the anchor check instead of drifting; the pin is re-checked at each milestone [SP 1.15, 2.2].
- **The WIT type model**: packages, interfaces, worlds, resources, records, variants, tuples, lists, options, results, flags, enums, `use` and `include`. The text format is parsed by `hg-wit` for the build and test path only; **the shipped path never parses WIT text** — a package ships a binary component whose types are in its type section, and a host that links by canonical names need not consume WIT at runtime [SP 2.17].
- **The canonical ABI**: lift and lower for every type, `cabi_realloc`, string encoding (UTF-8, with the `utf16` and `latin1+utf16` options implemented for compatibility and the profile pinning UTF-8), resources with handle tables, post-return, and the realloc ownership rules. One copy per crossing; no zero-copy in `@0.1`. The ABI's own limits are enforced as decoder bounds: flat values are only `i32`, `i64`, `f32` and `f64`; string and list byte lengths are at most **2^28−1**; resource handles are `i32` indices into the instance's handle table with own/borrow ownership transfer [SP 2.7, 2.10, 2.13].
- **Linking, restricted**: a component's imports are resolved from the host's provided interfaces only. **A package is one component; there is no runtime component-to-component linking** in `@0.1`. A publisher who uses a library links it at build time into the shipped component, and the signature covers the result. This removes the registry-resolution and diamond-dependency problems from the trust boundary, makes the capability surface a property of one binary, and means **install performs no dependency resolution** — so the dependency-confusion class (a private name shadowed by a public package) and the malicious-transitive-package class have no surface here: a package's bytes are all of its code [SC 5.9].

**What the engine does not implement in `@0.1`.** Dynamic composition of multiple components at load, `future<T>` and `stream<T>` async calls, the lazy ABI, and nested component imports. There is **no Component Model conformance suite yet** — only a growing WAST suite in the component-model repository — so the operative conformance bar is the shared **`wasi-testsuite`** at a pinned revision, the same suite Wasmtime and jco run, plus Izcal's own corpus built from the specification's examples. The component-model WAST suite is pinned by **commit hash**, because it has no releases either; a suite run whose hash is not the pinned one is not an AH-5 result. Both suites run as **test-only oracles** under M-1's test-only class and are Apache-2.0 [SP 4.2–4.4].

**No precompiled code, ever.** The app host compiles every component from the **validated source bytes of the package on each install and each update**, and caches only the host-produced compilation keyed by the package digest and the host's own build identity [SC 5.9, 7.8]. A precompiled or AOT artefact is never accepted from a package, because a compiler output reaches past the decoder and validator that the rest of the design depends on: the runwasi/containerd-shim-wasmtime **CVE-2026-47218** is exactly that class — attacker-supplied precompiled bytes in an image layer were treated as trusted — and `CVE-2026-27572`'s lesson is that a host-API panic path is a denial-of-service surface: the fix returns a guest trap, and every host call is fuzzed with malformed inputs [SC 5.9, 4.6]. The cost is that install pays compilation once; the benefit is that the only bytes the sandbox trusts are the ones it decoded itself. `@0.1` therefore has no compilation-cache sharing between machines, and no developer-supplied `.cwasm` or equivalent.

**Why the subset is honest.** The component model's value here is the **typed interface and the capability surface**, not dynamic linking. A single-component package with host-provided interfaces gives the person a reviewable import list, gives the broker a computed capability surface, and gives the developer WIT bindings from the standard toolchain. Dynamic composition can arrive later without changing the package format or the sandbox.

### 6.7 Packages, signatures and the install grant

**What a package is.** A `.izapp` file is a ZIP container with a fixed layout:

```
manifest.json         typed manifest: id inputs, version, profile, capabilities, intents, UI entry, limits
component.wasm        one Component Model binary (the app's logic)
ui/                   optional sealed UI: HTML, CSS, JS, assets (served on app://<id>/)
icons/                icons and metadata
LICENSES/             the app's own licence texts
signatures/author.sig detached Ed25519 over the canonical package digest
signatures/platform.sig detached Ed25519 over the same digest (directory/enterprise only)
```

The **canonical package digest** is SHA-256 over a canonical serialization, and the serialization is part of the published format (AH-D17), not an implementation detail:

- entries sorted by **UTF-8 path bytes, bytewise** (not locale collation), with `/` as the separator;
- each entry framed as path length, path bytes, content length, content bytes, over **decompressed** bytes, so the compression level cannot change the digest;
- **no ZIP metadata** in the digest: timestamps, permissions, extra fields, comments and the archive comment are excluded;
- `signatures/` excluded, everything else included; the manifest is an ordinary entry with no special ordering, so the digest never depends on which file a reader considers primary.

`hg-app-package` refuses a package with an **absolute path**, a `..` component, an empty path, a **duplicate path** after Unicode NFC normalization, a **symlink entry**, or a path whose canonical form differs from its bytes. Those are parser invariants as well as digest rules: they are what keeps a package from escaping the sealed image (ZIP-slip), so they are fuzzed under AH-4 and AH-16 like every other bound, not only checked at signature time. **Test vectors** with known digests for a fixed set of entries are published with the format, and AH-D17's second implementation must reproduce them; a digest two implementations compute differently is a format bug, not a compatible extension. The manifest is inside the digest, and it carries both `app_id` and the author key; the id does not derive from the key (6.3).

**Parser bounds, before anything is decompressed.** A package is attacker-supplied, so `hg-app-package` enforces fixed limits: **512 MiB†** compressed, **2 GiB†** uncompressed in total, **65,536†** entries, a **200:1†** compression-ratio cap per entry, no path traversal and no duplicate names after Unicode normalization, and no zip64 entry over the total budget. Limits are refused with a typed error naming the limit, never truncated. They are fuzz targets (G223) and each has a Kani harness for the arithmetic (G229). ZIP is the container because it is universal, every OS ships a way to produce one, and the format has a published specification (APPNOTE); the parser is Izcal's own (7), because a package parser that trusts a library's decompression limits is a parser that trusts a library's bugs.

**Two signatures, one identity** (EX-D10's process, reused):
- The **author signature** proves possession of the author key and defines the app id.
- The **platform signature** is applied by Izcal's signing service after review. Directory and enterprise installs require it. Developer-mode installs verify the author signature only and are labelled **developer mode** in the app panel, per session and per container; with the product gate on, a developer-mode install is still allowed only for a named-list app (6.15), so the list is a real exposure bound.

**No standard signature format exists, so this one is Izcal's and is published.** Neither the Component Model nor WASI defines how a component is signed, and no current registry format supplies a signature — WARG, before its archive, signed its package-log entries with publisher keys, while `wkg`/OCI carry **digests**, which prove integrity and nothing about who signed [SC 6.12, 7.2–7.4][SP 5.4–5.7]. The design therefore borrows properties from the schemes with a production record rather than inventing a trust story: **default-deny claims**, so a valid signature with no explicit grant grants nothing — the wasmCloud pattern, whose JWT claims carry `nbf`/`exp` and must be accompanied by an explicit capability set [SC 6.11]; **short validity plus a transparency record**, so a signature is evidence of a moment rather than a permanent permission — the Sigstore keyless/OIDC and transparency-log pattern that npm provenance and PyPI attestations adopted [SC 6.6–6.8]; and **roots that do not live on the distribution server**, so a compromised directory cannot re-root trust — TUF's offline-root discipline, already the update system's (UPD-R1/R3) [SC 6.9]. What a signature does **not** prove is stated once and relied on everywhere: possession of a key, the identity tied to it, the integrity of the bytes and continuity with prior versions — not the absence of malice (6.7's update section, 10). Because there is no standard to conform to, the package layout, the digest construction and the signature envelope are part of the open specification AH-D17 requires.

**Key rotation and revocation.** An author key rotates by a **key-rotation statement**: a record naming the app id, the old key, the new key and an effective version, signed by both keys and countersigned by the platform. Because the app id is a manifest field and not a hash of the key (6.3), a valid rotation **keeps the id**, and grants, tickets, storage keys and the directory entry continue under it; the key history is what proves continuity to the person. If the first package signed by the new key arrives before the rotation statement, continuity cannot be proven: the update is refused, the signing service will not issue a platform signature for an id whose key history does not connect, and the package can be installed only as a different publisher with a **new id issued at signing**, with no data following (6.3). A compromised author key is revoked through the directory's TUF channel (UPD-R1's data-channel discipline): the app is delisted, a **signed revocation record** naming the app id and the affected version range is published as a TUF target, and installed copies obey it — launch is refused, live host calls fail with a typed revocation error, and tickets are closed — with a person's explicit per-session override recorded as a degraded install grant. The record is separate from delisting: delisting stops new installs, the revocation record stops an installed copy, is delivered by the update channel, and reaches an offline profile at its next update check. A compromised platform key is the update system's emergency case and follows UPD-R5's closed switch list; it never silently re-signs a public key.

**The install grant.** Installation is a **grant event**, not a file copy. The broker verifies both signatures and then hands the package to the **install verifier** — a short-lived, tier-P `hg-app-host-main --verify` process bound to the broker at launch (6.2) — which parses the ZIP container, decodes and validates the component and returns a typed **capability summary** over that closed IPC channel: the imports found, the manifest's declared set, the diff, and the UI's declared surface. The broker consumes the summary, refuses a component whose imports exceed the manifest, stores the sealed image and the summary, and **never parses the guest format itself** (IPC-R15). It then draws an install prompt that names:

- the app's name, id and author key fingerprint;
- the capabilities requested, in plain language with consequences (`read your Documents folder`, `connect to api.example.com`, `use your YubiKey`);
- the source (directory, file, enterprise policy) and whether the package is developer-signed;
- what the app can do with each granted item, and what it cannot.

On approval the broker records the install at profile scope — the sealed package image (content-addressed, read-only, mapped clean), the verifier's capability summary and the registry entry — and a **grant set scoped to (app, container)** for the container where the install started, with that container's bucket created. A **first launch in a different container** draws a first-launch prompt with the same consequence lines, defaulting to session grants only; the person may repeat their remembered choices or narrow them, and no remembered grant silently follows the app into a container where it was not granted. A denial records a `denied` decision and installs nothing.

**Review is a capability review, and its economics decide its shape.** The only operator publishing full volumes is Apple: **9.1 million submissions and 2.09 million rejections in 2025**, with about **500 human reviewers** (2021 trial testimony) and roughly **12 minutes** of reviewer time per app, on top of automated pre-screening; Google reports 2.36 million policy-violating apps prevented in 2024 with **92% of human reviews AI-assisted**, and Microsoft certifies in about **three business days** [DU 1.1–1.4, 1.6, 1.10, 5.1–5.2]. At Izcal's **2–4 review staff** shared with extensions, manual review of everything is arithmetically impossible — 15–45 minutes per submission at plausible volume is the ceiling [DU 5.5]. The design follows the evidence: **automation triages, a human reviews the escalated tail** (capabilities that can write outside the app, reach a device, run an intent that changes state, or use a native-messaging class), known publishers get curation, and the **sandbox plus runtime grants remain the boundary** [DU 5.6]. Reviewers see the manifest, the computed import diff against the previous version, the identity history and the requested capability growth. They do not audit machine code, and the spec does not claim they do — Apple's own record shows review missing compiler-injected malware (XcodeGhost) and each of the four documented store-and-signing incidents — CCleaner, ASUS Live Update, SolarWinds and 3CX — passing the gate [DU 1.5, 2.7–2.10]. The **capability diff is mechanical**: a new import, a widened directory scope, a new host or a new device family is a diff line, and the review question is whether the growth is justified. This is the one place where a Wasm app platform is cheaper to operate than a native store: the security-relevant surface is computable from the artifact, and the decision log can publish what was tested per decision [DU 1.11].

**Updates.** Updates ride the extensions' uniform update grid (EX-D10). Three cases:
- **No capability growth, same author key**: automated checks only — both signatures, the digest, the computed import diff and the structural scan — then install; no prompt and no human review.
- **Capability growth**: staged until the person re-grants, with the same consequence lines as install; review before the platform signature.
- **Ownership transfer without key continuity** (a different publisher, not a rotation): full re-review and a **new app id**; the old app's data does not follow. A key rotation with a valid succession statement is not this case and keeps the id (6.3).

**Updates are where trust is actually exercised.** Every supply-chain incident in the store-and-signing record — CCleaner, ASUS Live Update, SolarWinds Orion and 3CX — arrived through the legitimate, correctly signed update channel, so install-time review was structurally blind to it, and every detection came from behavioural telemetry or incident response, not a signature [DU 3.1, 3.5]. The app host therefore treats updates as a cryptographic channel: the broker refuses any version older than the installed one (**anti-rollback**), pins the **package digest** as the version's identity, and verifies the signed metadata chain (UPD-R1/R3's TUF semantics) [DU 3.3]. Staged rollout is a quality control, not a security control; the adversarial-aware form is capability-growth staging, which no malicious update can skip [DU 3.2]. Because Izcal collects **no install telemetry** (EX-D12), the detection substitutes are named commitments rather than an absence: the local app log and prompt ledger the person can read, the public decision log, review triggers on ownership change and unusual capability deltas, and a remote kill switch whose use is disclosed [DU 3.6, 5.6].

**No install telemetry.** Installs and launches are local events; the directory is an ordinary website and the install action makes no report (EX-D12, FP-D9). The only aggregate channel is the opt-in footprint statistics of FP-D9, and app ids never appear in them.

**Where packages come from, and how an install starts.** Three sources, all explicit: the **Izcal directory** (a website with signed packages and a published decision log), a **local file** the person chose (developer mode if self-signed; platform-signed packages install normally), and **enterprise policy** (a signed policy that allowlists apps and pins versions). A directory install starts from the directory page's **Install** button, which is an ordinary download of a `.izapp` file with a registered media type; the download shelf offers **Install app**, and the broker draws the install prompt. Nothing installs from a download alone. There is no silent install, no bundle-with-another-installer and no store-initiated push.

**The distribution protocol.** The package artifact is shared with extensions (ZIP plus a detached Ed25519 signature), so the directory, the signing service and the update client are one piece of infrastructure, not two. The directory's index is a **TUF-signed metadata file**; packages are addressed by **digest**, and names are mutable labels checked by machine rules (no lookalikes, no confusables) rather than trusted — the npm/PyPI record of **1,233,000-plus cumulative malicious packages and a 2020 study's average of 209 days to public detection (174 packages)** is the evidence that names are an attack surface, while identity from the author key plus a content digest is not [DU 4.5–4.7]. If distribution needs more than a website and a signed index, the package travels as an **OCI artifact** with a Wasm media type, addressed by SHA-256 digest: that is the direction the Bytecode Alliance settled on after its federated WARG registry was archived on **28 July 2025**, with `wkg` as the surviving tool, and it avoids inventing a registry protocol when a general one exists [DU 4.1–4.2][SP 5.4–5.6]. WARG's one load-bearing idea, a maintainer-signed release log, survives here as the public decision log plus the two signatures.

**Data-only updates.** An app may ship a data update signed by the author key alone if the manifest declares the data as non-executable (a dictionary, a model, a list). The broker verifies that the update touches only `data/` and never `component.wasm` or `ui/`; this is the extensions spec's data-only update class (EX-D10, EX-9), applied to app packages.

### 6.8 The app's user interface

**The UI is a sealed web document.** A windowed app's `ui/` is served on `app://<id>/` from the sealed package image, by the broker's resource path, with a CSP that forbids remote script (`default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'none'` unless the manifest grants UI network). The document runs in an ordinary renderer keyed by (app origin, container), with the full engine: layout, accessibility, input, IME, DevTools, recording. It is not a special UI toolkit, and it is not a second browser.

**The app origin is not a web origin, and its authority surface is closed.** `app://` documents get the engine's rendering and script, not the web platform's authority-bearing surface. On `app://` the following are refused, `navigator.permissions.query` answers `denied` for them, and there is no prompt path:

- **Every permission-gated API** — camera, microphone, geolocation, notifications, clipboard, screen capture, USB, Bluetooth, Serial, HID, file system access, idle detection, wake lock, MIDI, payment, and the rest of the permission registry. Authority in the UI is exactly the manifest's `host` lines (clipboard, notify, open-url), each granted as an `app-*` capability through the broker's prompt renderer (6.4), never through a web permission prompt.
- **Origin-scoped storage** — cookies, `localStorage`, IndexedDB, the Cache API and OPFS. An app has one data store, its bucket, and the engine's storage service would otherwise become a second one with different quota, export and clear paths (6.11).
- **Service workers and shared workers**, and any registration that outlives the document. An app's lifetime is its component's; a window is not a principal, and nothing in the app may run when the app is not running.
- **Downloads** (`a[download]` is refused; writing a file is an `app-file-write` grant through the storage service) and **in-place navigation** (below).

What remains is the document: DOM, CSS, canvas, media decode, WebAudio, dedicated workers, and `fetch` only when `ui-network` is declared, through the app's partition. The manifest is therefore the UI's ceiling too, not only the component's, and the install prompt shows the `host` lines as part of what the app may do. A UI that needs another capability gets it from its component through an intent, which keeps the component the app's one authority. AH-17 measures this closure.

**Why the UI is HTML and not a display protocol.** A display-list or canvas-only app UI would make every app author responsible for accessibility, text input, IME, high-DPI and platform conventions, and would need a widget toolkit Izcal does not have and no local-app platform has made succeed. The engine already has the best UI stack in the product. The honest split is: **logic in Wasm, face in HTML**, with a typed boundary between them.

**The boundary is typed intents, on two sender paths.** The component exports intents (WIT functions). They are reachable in exactly two ways, and the broker binds the sender's **principal**, not a page-supplied field, on both:

- **`app.intent`, the app's own UI.** Bound to documents of the app's own origin: the broker derives the app id from the frame's committed `app://<id>/` origin and refuses a document of any other app. The endpoint exists for no other caller.
- **`app.invoke`, every other sender.** The shell (open-with, default handler), another app and an agent reach an intent through broker-routed calls, not through any document endpoint. The sender principal is `shell`, `app:<id>` of the calling app's host process or `agent:<task>`; the broker checks the per-sender, per-intent grant on every call (6.14). A renderer has no `app.invoke` endpoint at all, so no web page and no app UI can impersonate the shell or another app.

On both paths the broker validates the intent name against the manifest, enforces the grants, and routes the call to the app host. An intent's result is a typed value; an intent declared as streaming opens a **typed channel**: the broker fixes the channel's schema at setup, returns a channel id to both ends, and relays **opaque frames** without decoding them (it parses no guest format, IPC-R15). A terminal or an editor is therefore bounded by frame copying, not by per-byte marshalling in the broker, and the recording of a channel is a frame-count and sequence digest, not a frame log. The UI never gets a handle to the component's memory, its files or its tickets.

**The UI's own network surface.** By default the sealed UI has **no network**: its CSP sets `connect-src 'none'`, and `app://` responses carry no cookies. A manifest that declares `ui-network` (for example an app whose UI loads map tiles) makes the UI's `fetch` go through the **same app partition** as the component's HTTP, with `connect-src` limited to the granted destinations and the same no-cookie stripping. The component, not the UI, is the app's authority: a UI-only network grant is a separate manifest line and is reviewed as one.

**The app window.** The shell creates an app window: a window with app chrome (name, icon, container, an app menu) and no address bar. The app menu shows the capabilities, the live tickets, the app log, "Reload UI", "Restart component", "Open data folder" (for a granted directory), "Uninstall". The window contract is **PWA-grade, not a window manager**: the shell draws display modes and its own title-bar chrome, and always-on-top, focus stealing and cross-display placement are requests the OS may refuse — Wayland denies the first two outright — never promises ([BS 49–57]).

**The app window navigates nowhere.** A renderer hosting an `app://<id>` document refuses every navigation that would replace the document — `location` assignment, meta refresh, form submission, `window.open` targeting the window — when the target is a web URL or a **different app id**. A link to the web opens a **new browser tab**; a link to another app opens **that app's window or tab, with the target's chrome, launch prompt and grants**, or fails if the target is not installed. The app origin therefore never becomes a web origin, and app A's name and icon never frame app B's content. The rule is enforced in the navigation layer against the frame's app id and the target's scheme, not by inspecting page text (A30). App content must not imitate shell or browser chrome: the window keeps a shell-drawn identity strip that page content cannot cover or overlay, and a package that draws a fake address bar or another app's identity is refused at review, the same rule that keeps an installed site from imitating browser UI (A26).

**Service apps.** A package with no `ui/` is a **service app**: it has no window, is launched by an intent or at login (with `app-autostart`), and appears in the app panel with its state. A service app is how a local engine (a Git implementation, a converter) is shared by several UIs without any of them linking it.

**Several windows, one component.** An app's windows share its one app-host process and one component instance: the app's state is the app's, not the window's. Each window gets a **UI session id**, and an intent carries it, so the component can answer the right window; closing one window closes its UI session and leaves the component running while another window or a live intent remains. A window opened after the component exited restarts it, and the UI re-queries the app's state rather than assuming it survived. This is the same relationship a service app has with several UIs, and it is why the intent schema carries a session id from the first milestone.

**Accessibility.** The UI is web content, so it inherits the engine's accessibility, the shell's checks and the AT rules. The app window itself is shell chrome and passes the shell's accessibility targets. The component has no accessibility surface of its own; an app whose logic produces text for the person must render it in the UI.

### 6.9 Network

**HTTP only, through the app's own partition.** `wasi:http/outgoing-handler` is implemented as a typed request to a partition service keyed by (app id, container), not by a web site. The app request uses the networking spec's full partition tuple ([NW-D7][NW]) with every field fixed, not only T: **T = F = O = `app://<id>`** (schemeful; an app key can never equal a web key), **C** the app's container, **X = 0** and **N = 0** (an app request has no cross-site ancestor and no main-frame navigation), **n = 0**, **Cr = `same-origin`** (the app's own partition; no cookies are sent or stored), and **R = the declared route** that matched the destination. The cache key is (T, C, F, X, N, n); every connection-level key adds R and O. The app therefore gets its own caches, its own connection pool and no shared state with any website, and two apps in one container cannot share a cache entry, a connection or a DNS result.

- **No cookies, no referrer, no web credentials.** Requests carry the app's own headers; the partition service strips `Cookie`, `Referer` and `Origin` web semantics. An app that needs authentication manages its own tokens, stored in its bucket or a granted directory.
- **Bad ports, routes, LNA and blocking.** Bad-port checks and container routes apply unchanged. A local or private destination requires an **LNA ticket** the broker issues only for a granted address class (NW-D4). Content blocking is **not** applied to app traffic: the lists target page subresources and third-party requests, and an app's own HTTP calls are its declared purpose; the app panel shows the destinations, and a route or a network grant is the control. This is a decision with a measurable reversal trigger: **AH-18** audits contacted against declared destinations on the corpus and runs a known-bad endpoint corpus, and a failure reopens AH-D10.
- **Redirects, TLS and certificate policy** are the network stack's, unchanged: Izcal's TLS, CT, and the same error taxonomy. An app cannot disable certificate checks; there is no "ignore TLS errors" flag.
- **DNS names resolve through the network stack's resolver**, never through an OS call the app can see. `wasi:sockets/ip-name-lookup` is therefore absent with the rest of sockets.
- **Listening is absent.** `wasi:http/incoming-handler` is refused; an app that needs to receive a callback uses a broker-brokered loopback channel (a future profile line) or a webhook through a hosted service. No app opens a port.

### 6.10 Devices

Device access is the devices spec's, not a parallel path ([DV][DV]). An app's manifest declares a **device family ceiling** (for example `hid`, `serial`); the ceiling is a review fact, not a grant. Every device open is a **per-device grant through the existing chooser, ticket and session** (DV-D4), exactly as for a renderer. There is no install-time family grant: installing an app authorizes no device, and the first open in a container draws the chooser. The app host presents the ticket to the device arbiter exactly as a renderer does; the arbiter enforces the same ticket format and rules for an app principal, `app:<id>` (A20), and a ticket minted for one app is refused for another. Two consequences:

- An app cannot reach a device the person did not choose, and the ticket binds the app id and container, so a stolen ticket is useless to another app.
- The device session's parser (USB descriptors, HID reports, GATT) stays in the device service's session role; the app receives typed reports, not raw descriptors.

Apps do not get **WebUSB-style arbitrary control transfers** in `@0.1`: the manifest's family ceiling is enforced by the device service's capability sets, and an app that needs a vendor-specific protocol uses HID reports or serial bytes as the existing APIs do.

### 6.11 Storage

Two kinds of storage, both through the storage service, which remains the only process that touches files ([SG, G150][SG]):

1. **The app's private tree.** A bucket keyed by (app id, container) in the storage service, holding a directory tree the app sees as `/data`. Quota comes from the storage spec's quota model (SG-D4) and is charged to the app; the person sees it on the app panel and can clear it. Files are ordinary app data: exportable as a folder, never trapped in a proprietary store.
2. **Person-granted directories.** A directory the person picked, held as a **directory ticket**, with byte transport through the storage service's file path — the same coordination boundary the storage spec draws for picked files ([SG 6.14][SG]). Read and write are separate grants; write is high-risk; a write-on-close atomicity rule applies.

The WASI filesystem implementation is a **translation layer over the storage service's bucket tree API** (the ownership SG-D9 fixes: the storage spec owns the store, the byte mapping and the locks), not a POSIX emulation: `open` becomes a capability-scoped open on a preopen, `readdir` a bounded listing, `stat` a metadata query. Path resolution happens in the storage service under the ticket's scope, with symlink resolution confined to the preopen (openat-equivalent semantics, no `..` escape). The app bucket has **no symlinks and no hard links** (`ENOTSUP`); a granted user directory may contain them, and they resolve only inside the preopen. The app host never sees a host path and never holds an fd. The compatibility cost — an app that expects POSIX links, permissions or `flock` fails — is recorded in 10 and measured by AH-6.

**Path containment is enforced twice, and the OS boundary is the one that must be right.** `hg-wasi-host` refuses absolute paths, `..` traversal and any component that would leave the preopen before it translates anything; the storage service then enforces the same rule at the OS boundary, because a translation layer is a decoder and decoders have bugs. The second enforcement carries the weight on macOS: the 2023–2026 path-resolution vulnerability class — cap-std's trailing-slash handling (GHSA-hp8f-xmx4-4qrg), Wasmtime's own path-resolution advisory at CVSS 8.8 (GHSA-vqjp-4c8c-hfgg) and a further macOS advisory (GHSA-m9f7-9w4p-gv38) — shows that trailing separators, a symlink swapped between check and open, and unchecked rename or link targets let a guest escape a directory that an `openat`-style API appeared to confine ([SC 2.1–2.11]). The storage service therefore (a) resolves with directory handles, never with string concatenation; (b) opens the final component with `O_NOFOLLOW` and verifies each intermediate component against the preopen's identity; (c) canonicalises trailing separators before every open, rename and link; (d) checks symlink permission at access time, not at open time; and (e) rejects a rename or link whose source or target resolves outside the bucket. AH-16 is measured at this boundary, not at the WASI one, because this is where a bypass becomes a host compromise.

App storage is not web storage, and it is the app's **only** store: `app://` documents are denied cookies, `localStorage`, IndexedDB, the Cache API, OPFS and service workers (6.8), so the bucket is not one of two stores with different quota, export and clear paths but the one. It is not visible to a website and no website can read it. It is included in "clear site data" only through its own app entry, because an app is not a site. One store is what makes AH-13's complete removal true rather than approximate.

### 6.12 Install, launch, update, uninstall

**The app panel** is the shell's app surface: installed apps with name, id, author fingerprint, version, capabilities, live state, data usage and last launch; a search field; "Install from file"; and the directory entry point. It is a shell document on `izcal://shell`, driven by a `shell.apps` endpoint (9.5). It is not a storefront: the directory is a website, and the panel lists what is installed.

**Launch.** Three paths: the app panel, an intent from another principal (with a grant), and `app-autostart` at profile start. A launch is a broker decision: it checks the **(app, container) grant set**, drawing the first-launch prompt when the container has none, creates the app host, and (for a windowed app) creates the app window and loads `app://<id>/`. A launch is recorded in the prompt ledger only when it involved a grant; ordinary launches are app-log events.

**OS integration is requested, not assumed.** Install offers a desktop shortcut where the platform allows one. A manifest may declare file-handler and protocol-handler intents for the app's own file types; they are shown at install and registered only through the OS's own consent UI. The app panel's search and pinning are the in-browser launch surface. Izcal does not build a separate launcher process, and the directory is not a discovery storefront: the Chrome App Launcher was removed from desktop platforms in 2016 as the app platform under it declined, and the lesson is that a launch surface is worth building only because the platform underneath it is ([BS 66–69]).

**First-run.** An app with no persisted state starts at its entry point with `/data` empty. There is no "welcome" web page and no remote onboarding: the package is the app, and it runs offline by default. An app that needs a network grant asks for it as a capability at install; a later request is a growth prompt.

**Uninstall.** The broker revokes every grant and ticket, terminates the app host, deletes the app's bucket after a confirmation that names the size and the granted directories that will be left untouched, removes the package image, and records the uninstall in the app log. A person-granted directory is never deleted; uninstalling removes only the grant. The app's UI renderer is destroyed with its windows.

**Update failure.** A failed signature check, a malformed package or a capability mismatch leaves the installed version untouched, records a typed error in the app log and shows it in the app panel. There is no partial update: the package image is swapped atomically after verification.

**Rollback.** The previous package image is kept for **7 days†** after an update. Rolling back is an install of that signed image and passes the same capability check as any update: a rollback can never restore a grant the person has since revoked, and a rollback to a version with wider capabilities requires the growth prompt. After the window, the previous image is deleted and only the installed version remains; the package is always re-downloadable from the directory.

**Crash and restart.** An app-host crash is contained: the broker marks the app **crashed**, the app's windows show a typed "component stopped" state with **Restart** and **Open app log**, and the broker records a filtered-stack crash report under SH-R5 (deleted after 30 days†). Automatic restart is allowed at most **3† times in 10 minutes†**; beyond that the app stays stopped until the person acts, so a crash loop cannot become a wakeup or CPU loop. A crash never loses data the app committed to its bucket or to a granted file; anything in the component's memory is lost by design (6.3).

**Revocation of an installed copy.** A signed revocation record (6.7) is the update channel's payload for an app id and version range, distinct from delisting, which only stops new installs. When an online profile receives it, the broker immediately refuses launch, fails live host calls with a typed revocation error, closes every ticket, revokes remembered grants, and shows the app panel entry and the app window as **revoked**, with the reason and the person's explicit per-session override recorded as a degraded install grant. Data and the package image are kept, so the person can export; revocation deletes nothing. An offline profile applies the record at its next update check, so the kill switch's reach is the update channel's: AH-10 measures revocation only **after the record has been applied**, and the delivery latency from publication to application is published, not assumed. A revoked app's recording and app log stay readable, because the record of what the app did is evidence the person keeps.

### 6.13 Determinism, recording and replay

**The determinism contract.** Hosted apps run under the Wasm 3.0 **Deterministic profile**: all NaN values generated by floating-point instructions are canonical and positive, relaxed SIMD has one fixed lowering per ISA (and is excluded from `izcal:app@0.1` altogether), and no threads or shared memory exist, so the profile's silence on data races cannot bite [DR 1.3–1.4]. Two in-core sources remain, and both are closed by construction: `memory.grow` and `table.grow` failure is made deterministic by fixing the app's memory and table maxima at instantiation (a growth past them fails at the same point on every run), and resource exhaustion is a typed trap, not a different answer [DR 1.4]. Everything else that can differ crosses the host boundary — which is exactly what is recorded. The argument is structural: a WASI host-call boundary is a closed, typed interface of a few dozen functions, not rr's open, kernel-version-dependent syscall boundary, and it needs no simulated CPU [DR 5.6, 5.8].

**Budgets are the second dimension of the grant, and the deterministic one is the recorded one.** WASI defines **no budget interface at all** — neither Preview 1 nor the 0.2 worlds have fuel or a deadline — so this is Izcal's own mechanism and it is designed as a grant, not a runtime afterthought ([SC 4.4]). The two proven mechanisms have different characters: **fuel** is an instruction-count limit whose interruption point is completely deterministic for the same program and the same amount, at the cost of higher execution overhead; **epoch** is a wall-clock deadline with a measured **~10%** slowdown, and it is explicitly **non-deterministic** — the same program may stop at different points or finish on different runs ([SC 4.1–4.3]). `@0.1` therefore uses **fuel as the budget that is recorded**: a budget trap is a typed host result, identical on replay. Epoch interruption is a **backstop only**, for a recording that would otherwise pin a core indefinitely; when it fires during a recording the trap is recorded as the run's result, with the wall time noted as an observation, so replay reproduces the stop without a real clock. Limits are **on by default** — the CVE-2026-27204 lesson that a protective default left off is a protective default that does not exist — and every preemption point is fuzzed as an audit surface of its own, because adding budget checks to bulk memory and table operations has itself produced a runtime vulnerability (CVE-2026-104855) [SC 4.5–4.8].

**The guest is deterministic; the host boundary is the only nondeterminism.** Recording therefore records **host calls**, not instructions:
- the call id, interface, function, lowered arguments (with redaction by class), the result or typed error, and the clock reading the app saw;
- filesystem effects: opens, reads, writes, metadata, renames, deletes, with content hashes and sizes;
- network exchanges: the request, the response bytes, the decision fields (route, LNA, bad port), and the timing;
- entropy draws, recorded as **values, never a seed**: replay consumes the recorded draws through the substrate's entropy interface, so the production CSPRNG path stays unseedable (Q-CS-1) and a draw log is credential-class, encrypted at rest and never exported [DR 6.4–6.6];
- intent invocations and their results;
- the app's exit code and its final `/data` digest.

**A recording is per app session and per segment.** A segment starts at process launch (DT-D2's rule), ends at exit or crash, and the segments of one app session are joined by an app-session id. Live recordings follow DT-D2's buffer and segment rules; a saved bundle is encrypted under REC-R2 and expires after **30 days†** unless the person pins it (REC-R3). The app's record is visible in DevTools and in the app panel.

**The recording store has an owner, a quota and a deletion rule.** Bundles are held by `hg-app-observe` in the same encrypted store as DT-D2's other recording classes, under a **per-app quota of 1 GiB†** and a store-wide cap; the oldest unpinned bundles are evicted first and the eviction is recorded in the app log. **DevTools may display the host-call log** (interface, function, arguments after redaction, result, denial reason) and may export a **redacted text trace** under DT's export rules; it may not export the bundle's content-addressed payloads or entropy values, so "never exported" is true of the credential-class data and false only of the trace a developer already sees. **Uninstall deletes the app's unpinned recordings**; the uninstall confirmation names their size and offers to pin them, and a pinned bundle keeps the app id as a tombstone with no grants and no launch path. The at-rest risk is recorded in 10 with its trigger.

**Replay opens nothing, and it is jitless by default.** `hg-app-host-main --replay` runs the component with `hg-app-replay` in place of the service clients: the virtual filesystem is an in-memory tree seeded from the recording's preopen snapshots, the virtual network replays recorded exchanges, clocks and entropy return recorded values, and the broker endpoint answers from the log. A replay **provably opens zero files, sockets and devices**: the replay configuration links no service client, and a probe asserts no OS call is reachable from it (the DV-D8 pattern, extended to the WASI boundary). The replay configuration is **jitless** (REC-R6): the recorded tier does not lift the default, because replaying an untrusted app is one more place codegen is not needed, and the recording stores the tier it ran at. The one exception is the engineer-only **engine-triage mode**, which runs at the recorded tier under REC-R1's isolation conditions — a dedicated process per recording, no network, no documents, no ambient input, and the mode itself recorded — and it is entered only with the recording open in DevTools by a developer. A replay of an app that used a device ticket is refused unless the recording includes the device session's input log, in which case it replays against the virtual device.

**Large payloads.** File contents and HTTP bodies are stored **once, content-addressed**, and a host call references them by hash; an item over **64 MiB†** is recorded as its size and hash with the content absent, and the replay is labelled **approximate from that call** (DT-D2's segment-start rule is kept: the recording never pretends to be exact after a gap). **AH-3's ship-blocker corpus contains no approximate sessions**; the share of recorded sessions with a gap is measured and published against its own threshold, so a large-payload app cannot make the replay target unmeetable by being counted in it. Deduplication means an editor that rewrites the same buffer twice does not double the recording, and a recording of a large download is a hash, not a copy.

**What cannot be recorded.** A component that calls an interface outside the profile cannot run, so there is no unrecorded host surface. GPU work in the UI renderer is outside the app recording (it is a renderer recording, DT-D2); the recording joins the two by app-session id when both exist. A recording of a windowed app is therefore two records: the renderer's and the app host's, joined.

**Digests and divergence.** Each host call is hashed into a running digest over an explicit field list: **the call id, the interface, the function, the lowered arguments as recorded (after redaction), and the result or typed error**. Timing, host clock readings, ticket serials, segment ids and process ids are **outside** the digest, because they differ across runs by construction; a denial's **reason code** is inside it, because the decision is behaviour and a changed decision must diverge. The recording stores checkpoint digests at intent boundaries and at exit. A replay that diverges reports the first differing call with both values, the same "first differing task" discipline DT-D2 uses. Determinism is a target (AH-2), not a hope: the same package, the same recorded inputs and the same tier produce identical digests across runs and across the three ISAs.

**The debugging product this makes possible.** Today, browser Wasm observability stops at the module's memory and instructions: Chrome DevTools offers breakpoints, scopes and the Memory Inspector, but declares timing unreliable while attached, and no browser tool traces host calls, explains a denial with a reason, shows which real file a handle maps to (Chromium's own feature record admits the gap), or resolves a preopen; wasmCloud's own debugging guide tells developers they "can't attach a traditional debugger"; Firefox's WebReplay is dead [GO 3.1–3.6, 2.6]. The app host's recorded host calls and reason-coded denials answer the three questions those tools cannot: **why was this denied, which ticket covered it, and what did this app actually touch** [GO 4.1–4.7]. That is the differentiator the design should be judged on (AH-12), not a panel count.

### 6.14 Intents, the shell and agents

**Intents are the app's public surface.** An intent is a WIT function the component exports, named in the manifest with a version and a human description: `convert-image`, `open-repository`, `summarize-notes`. Intents are typed, versioned and discoverable without an app being launched: the broker reads them from the package manifest at install.

**Who may invoke an intent.**

| Sender | Grant | Prompt | Endpoint and principal |
|---|---|---|---|
| The app's own UI | Implicit for its own app | None | `app.intent`, principal `app:<id>` derived from the frame's committed origin |
| The shell (open-with, default handler) | A per-app, per-intent grant at install | Once, at install | `app.invoke`, principal `shell` |
| Another app | An `app-intent` grant naming sender and intent | `app-intent` prompt, per sender | `app.invoke`, principal `app:<sender id>` from the sender's host process |
| An agent, AI on | A per-task intent set, person-granted | Per task; person-present for writes (CON-R1, A-SH-12) | `app.invoke`, principal `agent:<task>` |
| A web page | **Never** | — | No endpoint exists |

The broker derives the sender principal from the channel, never from a field in the call: an `app.intent` call on an `app://<id>` frame is that app's, an `app.invoke` from a host process is that app's, and the shell and agent brokers are the only sources of `shell` and `agent:<task>`. A call whose principal lacks the grant is denied with a typed reason and recorded; there is no shared endpoint that lets one app name another as sender.

**Agents.** The agent principal of B9 reaches an app only through the intent registry: the broker gives an agent task a set of (app, intent) grants, the agent's calls are typed, schema-validated and recorded, and any intent classed as a **write** (it names a filesystem write, a network POST/PUT/DELETE, a device write or a spend) requires confirmation through CON-R1 with person-present evidence. The Rule of Two applies per task exactly as AGT-R6 states: untrusted input, sensitive access and the ability to change state are counted, and an app intent that changes state counts as state change. AGT-R3's scheme rules extend to `app://`: an agent never navigates to an app origin; it invokes intents.

**Every intent has a deadline, and the ceiling is broker policy, not manifest text.** The broker's typed endpoint carries a deadline per call: **10 s†** for a UI intent and **10 s†** for an agent or app intent. The manifest may request a **shorter** deadline and never a longer one. A declared long operation (a build, an export) is the `app-long-intent` grant: the broker's policy ceiling is **5 min†**, the grant names the intent, is shown in the app panel and the prompt ledger, and is revocable while the call runs. The engine's interrupt budget enforces the deadline carried in the grant: a guest that computes past it is stopped at the next safepoint, the call returns a typed timeout, and the event is a denial fact. A service app therefore cannot become an unbounded CPU consumer just because an intent woke it, and a UI that blocks on a component has a bounded wait.

**Why intents rather than automation.** Driving an app's UI through the accessibility tree or a CDP-like protocol gives an agent the app's pixels and clicks but no schema, no grant granularity and no record of intent. A typed intent is a capability: it can be granted, denied, revoked, versioned and recorded. It is also cheaper for the app author than exposing an automation surface, and it is the only path that satisfies P3 (the person decides) for an agent acting on their behalf.

**This is convergence, not a local dialect.** Typed capabilities are the shape the field is arriving at from the agent side: OpenAI's Apps SDK exposes named app capabilities to an agent, WebDriver BiDi is becoming the cross-browser automation standard, and the 2026 agent-permission work converges on short-lived, per-action approvals with a presence signal — relock-on-input, signed single-use consent [BS 61–65]. Izcal adopts the shape and supplies what the others do not: a broker that already owns every prompt and a recording substrate that records the grant, not only the call.

### 6.15 Exposure, admission and the WASI door

**No web content can reach the app host.** There is no JavaScript API, no HTML element, no URL scheme a page can navigate to and no feature-detectable surface. A page cannot tell whether the app host exists. This is not a policy; it is the design: the app host has no renderer-facing endpoint; the document intent endpoint is bound to `app://` origins of the same app, and the broker-routed invocation path is reachable only from the shell, an app host and an agent task (6.8, 6.14).

**The admission rule, applied as written.** The corpus's rule (ST-D18 as applied by LY-D10, and DV-D9 for APIs [DV 6.15][DV]) admits a web-facing feature when it has a WG resolution and tests and ships in two engines or is an Interop focus area. WASI has **no engine** shipping it, the Component Model has none either ([H2 4.2][H2]), and WASI's own subgroup scopes browsers out of its target set ([BS 29–30]); the app host cannot pass that test and does not claim to. The rule's purpose is to protect web compatibility from a single-engine API; the app host adds no web API. What the rule does govern is **default exposure**, and this spec applies the same discipline. Three **separate instruments** bound it, and each is named because they do different jobs (the DV-D9 instruments, applied to apps):

1. **The product gate.** The app host ships **off by default**, behind an **App host (experimental)** setting decided by the product owner. The gate decides whether the feature runs at all; it authorizes no app.
2. **The named-app list, at most 20†.** With the gate on, only a listed app may **install or launch** — from the directory, from a file, or in developer mode; a developer-mode package that is not listed is refused at install, so the list is the exposure bound and not a review convenience. Each entry names the app id, publisher key fingerprint, purpose, the **owner name** who accepts it and the date. Entries are decided by the product owner with the security owner's sign-off. The list does not bypass the install prompt or any grant.
3. **Per-capability owner exceptions.** If a listed app needs a capability class the profile has not admitted (a device family DV-D9 has not reviewed for app principals, or a host function under `izcal:app/host@0.1` beyond the first set), the security owner grants a named exception for that app and that capability, with conditions, the owner's name and an expiry **12 months† after grant**. An exception is never a general enablement, is recorded with the capability diff, and lapses into a re-decision, not a silent renewal.

An app whose list entry or exception lapses stops launching (or loses the capability) until the decision is made again. The list and every exception are published with the decisions, and a count of both appears in the Stage 3 exit audit (AH-M5). **Native Component Model in the engine** remains HGBA-2 X-3, Stage 3, optional, counting toward the two-engine gate. This spec does not change X-3's status; it consumes the same engine work from the app side.

**The two doors this spec closes.** It promotes the Stage 2/Stage 3 backlog row **"native Component Model (optional)"** from an optional engine target to a **Stage 3 platform deliverable with a named owner role filled at AH-M0, scope, targets and cost** (AH-D15). And it reads Stage 1's refusal **"the browser as a window manager for local apps"** for what it is: **time-scoped to Stages 1 and 2** (line 420 says "Out of scope for Stages 1 and 2"), so a Stage 3 app host is outside its scope, and **object-scoped to native applications** — native binaries, OS processes, ambient authority. The app host reads the refusal **narrowly** — its object is arbitrary native local apps, not every way of running a program — and does not claim to manage them: the browser still does not manage arbitrary local applications or OS windows. It runs **WASI components from signed packages inside its own sandbox, in browser-owned windows, with the broker's consent**, which is the capability-sandboxed path the refusal left open. 9.5's door item (A4) records both the time scope and the object, so no later revision can mistake a Stage 3 platform for the Stage 1 refusal or turn a time-scoped line into a permanent one.

### 6.16 Footprint, performance and wakeups

**The app host is a site-class process, not a baseline service.** It runs only while an app is live; at idle it costs zero. Its budget:

| Line | Budget | Notes |
|---|---|---|
| App host base, no app | ≤ 4 MiB† **charged memory** | Private anonymous resident plus owned shared regions charged once, clean file-backed code reported as shared and not charged — the footprint spec's charged metric (§6.2). The tier-P engine image is mapped clean and shared; the base is a separate line from FP-1's ≤ 2 MiB† per-extension share because the app configuration links the component, WIT, WASI and package decoders instead of `hg-js-*` |
| Per app instance | ≤ 8 MiB† charged **incremental** | Engine runtime, instance, tables and host state for one live component; when the ceiling allows W1/W2, the tier-P codegen build is charged to this line at ≤ 2 MiB† of it; the guest's linear memory is charged to the app's quota, not this line |
| Guest memory default | ≤ 512 MiB† | Raised per app to the quota cap by a person setting, never by the app |
| App UI renderer | An ordinary renderer's budget | FP-1/FP-2 apply unchanged; one per (app origin, container) |
| Launch to first host call | ≤ 50 ms† p95 | Parse, validate, instantiate, call `run` |
| Host-call overhead | ≤ 2 µs† p50, ≤ 10 µs† p99 | Local calls (clocks, random, io); network and storage calls are service round trips |
| Wakeups when idle | 0 | A service app with no pending intent and no timer is frozen; a windowed app's component sleeps when its window is hidden unless it holds a declared timer |
| Recording overhead | ≤ 5%† wall clock and ≤ 50 MiB† per app-hour on the AH-6 corpus | No published number exists for a WASI-boundary recorder: CPU-level TTD pays 10–20× with gigabytes per few minutes, while whole-VM simulation needs a simulated CPU and has no GPU [DR 5.4, 5.7]; the probe publishes both values |

The app host never wakes for a hidden window without a declared reason (FP's hidden-page rule applies). A recording adds the DT-D2 recording cost, measured by the recording probe; an app recording is off by default.

### 6.17 Crates, layers and rules G220–G229

**New crates.** L2e gains the component engine; L2s gains the pure models; L5 gains the host and services; L6 gains the binary.

| Layer | Crate | Contents | Tier |
|---|---|---|---|
| L2s | `hg-wit` | WIT type model, package/world/interface model, text parser (build and test path), canonical type interning | P |
| L2s | `hg-app-manifest` | Manifest model, canonical serialization, capability vocabulary, version rules | P |
| L2s | `hg-app-ticket` | App ticket format, signing/verification over `hg-crypto`, scope canonicalization | P |
| L2e | `hg-component` | Component binary decoder, validator, type canonicalizer, linker (host interfaces only), canonical ABI lift/lower | P |
| L5 | `hg-wasi-host` | The `izcal:app@0.1` world implementations: cli, clocks, random, io/poll, filesystem translation, http client, and the `izcal:app/host@0.1` package | P |
| L5 | `hg-app-host` | App lifecycle, intent dispatch, `CallContext`, grant checks, log and IIP facts | P |
| L5 | `hg-app-package` | Package ZIP parsing, canonical digest, signature verification, sealed image assembly | P |
| L5 | `hg-app-registry` | Installed-app metadata, package image store, update staging, uninstall | P |
| L5 | `hg-app-replay` | Virtual filesystem, virtual network and recorded clock/entropy injection for replay | P |
| L5 | `hg-app-observe` | IIP `app` domain facts: host calls, denials, grants, tickets (metadata), intents | P |
| L6 | `hg-app-host-main` | The app binary: argument parsing, sandbox entry, app, replay and install-verify configurations | P |
| tools | `hg-component-check`, `hg-wit-check`, `hg-app-sign`, `hg-app-fuzz`, `hg-app-corpus` | Verification and test tools; `hg-app-sign` is the signing service's client, not shipped in the browser | T |

**Rules.**

| Rule | Statement | Why |
|---|---|---|
| **G220** | Only `hg-wasi-host`, `hg-app-host` and `hg-app-host-main` may depend on `hg-component`; no renderer, broker or page-facing crate may link the component decoder or the canonical ABI | Keeps the component engine out of every process that parses web content; the app host is the only consumer |
| **G221** | `hg-wasi-host` never names an OS API or `hg-substrate-os`; every effect is a typed request to a service or a ticket presented to one | The app host holds no OS handle (6.2); G150's discipline extended to apps |
| **G222** | Every WASI host function takes a `CallContext`; a function without one is a compile error, and every call is recorded | One funnel for authority, recording and denial (6.5, 6.13) |
| **G223** | `hg-app-package` parses untrusted bytes at tier P: no panics, every length and count bounded before use, and a registered fuzz target per parser | A package is attacker-supplied (6.7) |
| **G224** | The app configuration links no DOM, no `hg-js-*`, no `hg-bindings`, no web API crate and no window crate; the extension configuration links no `hg-wasi-host`; replay and install-verify link no codegen; every configuration is checked on the loaded-library closure | Four configurations, one binary source, no accidental merge (6.2) |
| **G225** | The manifest is the ceiling: the loader refuses a component whose imports are not covered, and no runtime grant may exceed a declared capability | Declared ≥ computed, applied to capabilities (6.4) |
| **G226** | Only the broker mints app tickets; `hg-app-ticket` verifies and never signs; a ticket's scope is enforced by the verifying service, not by the app host | The issuer/verifier split of DV-D2 |
| **G227** | No app crate names a socket, file or device API; the only outbound paths are `hg-wasi-host`'s typed requests and the intent endpoint | Zero ambient authority (P2, I-2) |
| **G228** | Every canonical-ABI, WIT and component-format algorithm carries a `// spec:` anchor at a pinned revision, and every value the specification does not state is † or measured | M-1, G9 |
| **G229** | Every parser and every host-call decoder is a T1 fuzz target with a corpus, and every size/length invariant has a Kani harness | VER-1; every length in a component and a canonical-ABI payload is untrusted input |

### 6.18 Alternatives considered

| Decision | Alternatives | Why the recommendation wins | Reversal trigger |
|---|---|---|---|
| Process kind | (a) component-host configuration; (b) 38th kind; (c) renderer | (a): same trust shape, one threat model, no duplicate review, though with a larger parser surface (6.2) | Any confirmed memory-safety escape in an app parser family, or two new finding classes in the app-profile threat model within one release cycle: split the file; a kind only if the sandbox tier changes |
| Identity | (a) manifest-carried random `app_id` signed by the author key; (b) key-derived id; (c) domain-bound app | (a): stable across installs and key rotations, no domain to own, and the id is not a hash of a key that can be lost (6.3) | A directory requires domain-bound identity for anti-impersonation; then a domain claim is an added manifest field, not a new id |
| UI | (a) sealed HTML UI; (b) display-list protocol; (c) headless only | (a): accessibility, IME, layout and DevTools are the engine's strengths; (b) needs a toolkit and makes authors responsible for accessibility (6.8) | A standard WASI UI world with an accessible-tree contract exists and an Izcal prototype meets the interaction budget |
| WASI version | (a) 0.2 worlds; (b) 0.3 async worlds | (a): stable, poll-based, simpler to record; 0.3 async is 3.5× overhead on sync paths (6.5) | 0.3 async overhead measured below 1.2×† on Izcal's engine and the worlds stop changing |
| Sockets | (a) HTTP only; (b) raw sockets behind a grant | (a): routes, LNA, bad ports and the partition key stay outside the guest (6.9) | A standard per-destination socket capability the network stack can enforce on a connected socket, proven by a design review |
| Linking | (a) one component per package; (b) runtime component linking and a registry | (a): the capability surface is one binary's imports; no resolution trust boundary (6.6) | A component registry with signed resolution and a measured need from real apps |
| Distribution | (a) shared directory with extensions; (b) separate app store; (c) no directory, file install only | (a): one review process, one signing service, one update grid (6.7) | Review throughput or app volume makes a shared queue starve extensions |
| Stage | (a) Stage 3 deliverable; (b) Stage 2 | (a): the component host lands at JS-M4 (Stage 2 exit) and the directory is an extensions Stage 2 deliverable; apps need both (6.19) | Extensions land early and the directory has measured spare capacity |

### 6.19 Staging

**Stage 2 seams.** Stage 2 builds the pieces the app host depends on and does not build the app host:

- the component host at JS-M4, with the **configuration mechanism** (extension configuration first) and the tier-P build;
- the extension package format, signing service, directory and review process (EX-D10);
- `hg-wit` and `hg-component`'s **decoder and validator** as part of the component host's foundation, with the canonical ABI scoped to the interfaces extensions need;
- the `app://` scheme registered in the broker's resource path but serving nothing;
- the storage service's app-bucket keying and the picked-directory capability path.

**Stage 3 deliverable.** The app host proper: the full canonical ABI, the WASI profile, the app binary, packages, the install grant, the app panel and window chrome, intents, recording and replay, devices and agents. AH-M0 starts when JS-M4 exits and the extension directory is live.

### 6.20 What this spec does not do

- It does not let a web page run a Wasm app, detect the app host, or reach an app's data.
- It does not run native binaries, OS processes, shell commands or dynamic libraries.
- It does not manage arbitrary local applications or their windows; the Stage 1 refusal stands for everything outside a signed Izcal package.
- It does not implement raw sockets, listening sockets or a local server.
- It does not give an app the web platform: no DOM in the component, no cookies, no web storage, no browser APIs.
- It does not replace extensions: extensions extend pages; apps are programs.
- It does not claim that review makes an app safe; the sandbox and the grants are the boundary.

## 7. M-1 compliance

**What is built new, by Izcal, from the specifications.** The component binary decoder, validator and linker; the WIT type model and text parser; the canonical ABI; every line of the WASI profile; the package format's parser and verifier; the app host, registry, replay and observation crates; the app panel and window chrome. No Wasmtime, wasmer, wasm-tools, wit-bindgen, jco, wasi-libc, wasi-sdk or any other runtime, linker or toolchain is linked into a shipped binary or used as a template.

**What is depended on, and why it is allowed.** The engine's own `hg-wasm`, `hg-heap` and `hg-codegen` (JS-D10's tier-P build); `hg-crypto`'s MIT-licensed Ed25519 and SHA-256; the substrate; the IPC generator; the storage, network and device services through their typed endpoints. No new external dependency of any kind.

**Anchors and pinned revisions.** The Component Model documents are pinned by **SHA-256 content digest** plus the retrieval date — `WIT.md`, `Binary.md`, `CanonicalABI.md`, `Explainer.md` at 21 October 2026, with a dated snapshot archived — because the repository has **no tagged releases**; a digest is what freezes content, and `hg-component-check` verifies the digests before it trusts an anchor [SP 1.15, 2.2]. The component-model WAST suite is pinned by commit hash; the profile pins WASI **0.2.12** (2 June 2026) and the `wasi-testsuite` by tag [SP 1.10]; the ZIP format is pinned to APPNOTE; the JSON manifest schema is Izcal's own, versioned. Every algorithm function in `hg-component`, `hg-wit`, `hg-wasi-host` and `hg-app-package` carries a `// spec:` anchor at a pinned revision (G228), and for the un-tagged Component Model documents the anchor's revision includes the content digest. Values the specifications do not state are † or measured.

**Two things the standards do not supply, so this spec does.** There is **no standard component signature format** — no Component Model or WASI document defines one — so the package signature envelope, the canonical digest construction and the key-rotation statement are Izcal's own, and AH-D17 puts them in the published format specification rather than leaving them private [SC 6.12, 7.2–7.7]. There is also **no published Component Model threat model**: the runtime's own security scope is a corpus gap, and 9.5's A9 supplies the app-profile section rather than borrowing another project's document. Both are recorded here because a reader should not infer a standard's authority for a design decision the standard never made.

**Test-only and standards data.** The shared `wasi-testsuite` and the component-model WAST suite are test-only oracles under M-1's test-only class; both are Apache-2.0, and test fixtures keep their headers [SP 4.2–4.5]. The Component Model documents are Apache-2.0; the WASI specification is published under the W3C Community Contributor License Agreement; the Wasm core specification document is under the W3C Software and Document Notice and License [SP 5.1–5.3]. All are permissive for implementation and quotation, and no text is shipped. The WIT text parser is tested against the specifications' grammar and the test suites, never against a third-party parser's source.

**Evidence rule (M-1 part 3).** No research note or section of this spec links to a third-party project's code file, header, commit, diff or review; the prose documents cited are papers, specifications, official documentation, release notes and blog posts. Where a value is stated only beside code, it is marked as a question or measured black-box (11.2).

## 8. Targets

† marks a threshold set by this document from first principles; none is measured on Izcal yet. Each target names its measurement and its failure action.

| ID | Target | Threshold | Measurement | If it fails |
|---|---|---|---|---|
| **AH-1** | Zero ambient authority | 0 successful host calls without a live grant; 0 file descriptors, sockets or device handles held by an app host (probe); 0 forged tickets accepted in the compromised-host harness | SEC-31-style probe suite; sandbox probe per OS | Ship blocker |
| **AH-2** | Determinism | 1,000 of 1,000 runs of a corpus app produce an identical host-call digest, in-process and across fresh processes; identical across macOS, Linux and Windows and across W0/W1/W2. Runs stopped by the epoch backstop are excluded from the 1,000, recorded, and their rate reported and ≤ 1%† | Determinism harness, three ISAs | Ship blocker at the ceiling that fails; the ceiling is withdrawn |
| **AH-3** | Replay fidelity | ≥ 200† recorded app sessions **with no recording gap** replay to their exit with every checkpoint digest matching and **0** real files, sockets or devices opened (probe); the share of recorded sessions with a gap is reported and ≤ 5%† | `hg-app-replay` + probe | Ship blocker |
| **AH-4** | No panics | 0 panics or aborts from fuzzed packages, components, WIT types and canonical-ABI payloads; 72 h† per parser with **≥ 90%† line and ≥ 80%† branch coverage** of each parser crate measured from the fuzzer's coverage map | T1 fuzzers, `hg-app-fuzz` | Ship blocker |
| **AH-5** | Conformance | 100% of the shared `wasi-testsuite` at the pinned tag/revision and every case in the component-model WAST suite **at the pinned commit hash**, for the profile's interfaces; 0 unexplained failures | Conformance runner | Ship blocker for the profile version |
| **AH-6** | Compatibility | ≥ 25† real apps built with the reference toolchain run unmodified, including 5† with filesystem, 5† with HTTP and 3† with devices; the corpus (apps, versions, digests) is **frozen at AH-M0** | App corpus | Every failure is published; a replacement is a different app of the same class and is recorded with its reason; cutting a class needs the owner's written decision, never a re-scope that removes the failure |
| **AH-7** | Performance | Launch to first host call ≤ 50 ms† p95; local host call ≤ 2 µs† p50 and ≤ 10 µs† p99; W1 within 3×† of W2 on the app corpus | Reference devices | The ceiling changes (W1 → W2) only with the security review's sign-off |
| **AH-8** | Footprint | App host base ≤ 4 MiB† **charged memory**; per-app overhead ≤ 8 MiB† incremental, with tier-P codegen charged at ≤ 2 MiB† of it; 0 wakeups when idle; the footprint spec's per-process budget holds on three ISAs | Ledger; FP-M-style measurement on the charged metric of FP §6.2 | The budget is re-cut, not isolation |
| **AH-9** | Install trust | 0 packages installed without a platform or developer signature; 0 installs without an explicit prompt; 0 network requests during install; 0 telemetry | Lab install harness; network recorder | Ship blocker |
| **AH-10** | Update and revocation | 100% of capability-growth updates staged until re-grant; after a revocation record is applied, 100% of revocations effective at the next host call and 0 tickets surviving; 0 partial updates; publication-to-application latency measured and published per channel | Update harness | Ship blocker |
| **AH-11** | Agent intents | 100% of agent writes require person-present confirmation; 0 intents reachable without a grant; 0 `app://` navigations from an agent | SEC-18/19 suite extended | Ship blocker |
| **AH-12** | Observability | 100% of denials, grants and intents appear in the IIP `app` domain with a reason code; the app panel answers **why denied, which ticket, what it touched**, and ≥ 80%† of participants in a study of **n ≥ 20†, within-subject, three recorded sessions each in counterbalanced order, success = all three answers correct for a session** | IIP conformance; task study | The missing facts are added before Stage 3 exit |
| **AH-13** | Uninstall | 100% of app data removed; granted directories untouched; 0 orphan tickets | Uninstall harness | Ship blocker |
| **AH-14** | No web reachability | 0 web APIs, elements or navigations reach the app host; a page cannot detect it | Feature-detect corpus; browser fuzzer | Ship blocker |
| **AH-15** | Recording cost | ≤ 5%† wall-clock overhead and ≤ 50 MiB† per app-hour on the AH-6 corpus; the values are published, not estimated | Recording probe, three ISAs (DT-15's method) | Recording stays off by default and the cost is published as measured; a threshold miss is not a ship blocker |
| **AH-16** | Filesystem containment | 0 escapes in a **90-day†** adversarial corpus of **≥ 10,000† generated and ≥ 500† seeded cases per OS** covering path, symlink, rename and link cases, trailing-slash open and rename, device names and Unicode confusables; 0 host files read or written outside the app bucket or a granted directory | Storage-service adversarial suite, seeded from the 2023–2026 escape CVEs ([SC 2.1–2.11]) | Ship blocker; the storage-service path is rewritten or the grant class is withdrawn |
| **AH-17** | Sealed UI authority | 0 permission-gated web APIs and 0 web storage APIs reachable from an `app://<id>` document, with or without a manifest `host` line; 0 web permission prompts; the `izcal:app/host@0.1` functions reachable only through their `app-*` grants; 100% of the permission registry and the storage-API list probed, on every engine configuration | Probe corpus against every permission-registry entry and storage API; a page that tries each | Ship blocker; the renderer's `app://` surface is closed before release |
| **AH-18** | Declared destinations | 0 of the AH-6 corpus contacts a host outside its declared routes; ≥ 99%† of app connections carry a destination matching a declared route; a known-bad endpoint corpus is contacted 0 times by listed apps | Partition-service destination audit; known-bad endpoint corpus | AH-D10 reopens: content blocking applies to app traffic |

## 9. Staging, milestones, staffing, decisions and amendments

### 9.1 Staging

Stage 2 seams (6.19) precede AH-M0. The app host is a **Stage 3 deliverable**, starting at JS-M4's exit and the extension directory's launch. It does not compete with Stage 2's extension track for the same reviewers until the directory exists.

### 9.2 Milestones

| Milestone | Months† | Exit | Depends on |
|---|---|---|---|
| **AH-M0** | 2 | Foundations measured: `hg-component` decodes and validates the conformance corpus; `hg-wit` round-trips every type; the canonical ABI passes the suite's lift/lower cases; a hello-world component runs under `hg-app-host-main`; the tier-P build's overhead measured; the app-bucket and picked-directory paths spike; design review fixes the `CallContext`, ticket and recording schemas; the AH-6 corpus is frozen; the app host owner is named | JS-M4; storage and network seams |
| **AH-M1** | 5 | Profile `@0.1` complete for cli, clocks, random, io and the private filesystem; recording and replay of a headless app; AH-1, AH-2, AH-3, AH-4, AH-5 pass on the profile | AH-M0 |
| **AH-M2** | 5 | HTTP through the app partition; the sealed UI and `app://` (AH-17); the intent channel; the app panel and window; install/update/uninstall with two signatures and the install verifier; AH-6, AH-9, AH-10, AH-15 (measured, not estimated) | AH-M1 |
| **AH-M3** | 4 | Devices, agents, person-present confirmations; the `app` IIP domain and the app panel's facts; AH-7, AH-8, AH-11, AH-12 | AH-M2 |
| **AH-M4** | 3 | Security review and the compromised-host harness; the 90-day† adversarial filesystem corpus (AH-16); directory review live with capability diffs; the named-app list and exception mechanism; AH-13, AH-14, AH-18 | AH-M3 |
| **AH-M5** | 2 | Stage 3 exit audit: the corpus of ≥ 25† apps, of which ≥ 10† are from authors other than Izcal; the published decisions; the measured thresholds replaced; the owner exceptions reviewed; the format specification published and versioned (AH-D17) | AH-M4 |

The durations sum to **21† calendar months at the plan's midpoint**; the range is 18–24 by ±1 on AH-M1 and AH-M2. They are month counts, not dates, because AH-M0's start depends on JS-M4 and the extension directory. **The owner role** is filled by a named engineer at AH-M0 — that name is AH-M0's entry condition, and until it exists no list entry or owner exception in 6.15 may open — and each milestone's exit is signed by that owner and the security owner.

### 9.3 Staffing

| Track | People† | Notes |
|---|---|---|
| Component runtime (decoder, validator, linker, canonical ABI, WIT) | 6–9 | The long pole; overlaps the JS engine's Wasm team by 2–3†, stated not double-counted |
| WASI profile and host services | 3–4 | cli, clocks, random, io, filesystem translation, http client |
| App platform (packages, registry, intents, UI, panel) | 4–5 | Includes the shell's app surfaces |
| Security, fuzzing and replay | 2–3 | Tier-P discipline, harnesses, the compromised-host suite |
| Review and directory operations | 2–4 non-engineering | Shared with extensions; capability-diff tooling |
| **Total** | **15–21† engineers, 2–4† non-engineering** | **22–42 engineer-years†** at 18–24 months† (15–21 engineers × 1.5–2 years); non-engineering is **3–8† staff-years**, counted separately |

The arithmetic, per track: 15–21 engineers × 1.5–2 years = **22–42 engineer-years†**. Of the runtime track's 6–9, **2–3† are shared with the JS engine's Wasm team** and already in that team's plan; their 3–6† engineer-years are not an incremental app-host cost, so the **incremental figure is 16–39† engineer-years**. The 2–4 non-engineering staff are a separate 3–8† staff-years, not folded into the engineer total. The **ramp**: 50% of the team at AH-M0, 80% by AH-M2, full at AH-M3, and the last six months are security review and the exit audit; the midpoints, not the endpoints, are the planning numbers.

The overlap with extensions is real and named: the directory, signing service, review process and component host are extensions deliverables. The app host adds the runtime, the profile, packages and the app surfaces. No number is counted twice.

### 9.4 Decisions

**The owner most needs to see AH-D1, AH-D2, AH-D3, AH-D4, AH-D5, AH-D6, AH-D7, AH-D8 and AH-D17** — the starred decisions, identity and linking included; there is no unstarred decision in this first group.

| ID | Decision | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|
| **★ AH-D1** | Process kind for apps | **(a) the component host's app configuration; no new kind** | One threat-model file with one app-profile section and four parser subsections; two link configurations | Any confirmed memory-safety escape in an app parser family (package, component, WIT or canonical ABI), or two new finding classes in the app-profile threat model within one release cycle: split the file into a kind; a kind only if the sandbox tier changes |
| **★ AH-D2** | App identity | **(a) `app://<id>` with a manifest-carried random `app_id` signed by the author key, kept across key rotation by a two-key succession record; no web origin, no cookies** | A new scheme; a directory-visible id; a succession record | A directory needs domain-bound identity: add a domain claim to the manifest |
| **★ AH-D3** | Capability model | **(a) manifest ceiling + computed imports + broker-signed tickets; install is a grant event; a tier-P install verifier computes the surface so the broker never parses guest bytes** | Ticket types for FS and network; prompt classes; the `--verify` configuration | A capability that cannot be expressed as a ticket and a grant |
| **★ AH-D4** | WASI profile | **(a) 0.2 worlds pinned at 0.2.12: cli, clocks, random, filesystem, http client, io; no sockets, no incoming, no threads, no relaxed SIMD** | Compatibility gaps for socket apps and 0.3 async apps | 0.3 async measured under 1.2×† and stable for two patch releases; re-decided at AH-M2; a socket capability the stack can enforce |
| **★ AH-D5** | Component linking | **(a) one component per package; host interfaces only; canonical ABI complete** | No runtime composition; publishers vendor dependencies | A signed component registry with resolution, and real apps that need it |
| **★ AH-D6** | App UI | **(a) sealed HTML UI in an own-origin renderer + typed intents; no display protocol** | Two sandboxes per windowed app; an intent channel | A standard accessible UI world with a prototype meeting AH-7 |
| **★ AH-D7** | Distribution | **(a) shared directory with extensions; two signatures; capability-diff review; no telemetry; uniform updates** | Review capacity; signing infrastructure | Review throughput starves one package type |
| **★ AH-D8** | Exposure | **(a) off by default behind the product gate; only a named-list app ≤ 20† installs or launches, developer mode included; per-capability owner exceptions expire after 12 months†; no web API** | A gated feature that most people cannot use at first | The two-engine gate is met by other engines; then default-on is reconsidered |
| **AH-D9** | Determinism and recording | **(a) host-call log + virtual FS/network + recorded clocks/entropy; replay opens nothing** | Recording cost; a replay configuration to maintain | A host surface that cannot be virtualized |
| **AH-D10** | Networking | **(a) HTTP only through the app's own partition; no cookies; no content blocking on app traffic** | Apps that need raw sockets or a local server | AH-18 fails — an undeclared destination in the AH-6 corpus, or a listed app contacts a known-bad endpoint — or the app-destination audit shows a privacy or security problem; then content blocking applies to app traffic |
| **AH-D11** | Devices | **(a) reuse DV-D2/DV-D4 unchanged; manifest family ceiling** | No new device work | A device family whose grants cannot be keyed by app id |
| **AH-D12** | Storage | **(a) app bucket + picked-directory tickets through the storage service** | A filesystem translation layer | A WASI filesystem requirement the storage service cannot express |
| **AH-D13** | Agents | **(a) intents only; per-task grants; person-present for writes** | An intent registry and its schema | An agent task that needs a capability no intent expresses |
| **AH-D14** | Tiers | **(a) Baseline (W0/W1) default; Full (W2) optional; Jitless available; tier-P builds** | A second codegen build; a G14 amendment | W1 misses AH-7 on the app corpus: W2 becomes the default |
| **AH-D15** | Stage | **(a) Stage 3 deliverable, Stage 2 seams; starts at JS-M4** | A long dependency chain | Extensions land early and the directory has measured spare capacity: start at Stage 2 exit + 6 months† |
| **AH-D16** | What review claims | **(a) capability review, not code audit; the sandbox is the boundary** | Honest limits in the directory UI | A review failure that the capability diff could have caught: add the check |
| **★ AH-D17** | Format governance | **(a) publish the package layout, the manifest schema and the `izcal:app@0.1` profile as an open, versioned specification under MIT, so another host can implement the format; the host implementation stays Izcal's** | Writing and maintaining the document; versioning discipline | No second implementation after 12 months† and the published spec costs more than it returns: keep it published, stop treating it as a standard; a profile change would break published packages: version the profile, keep the layout stable |

### 9.5 Amendments to other documents

This spec does not edit other documents. Each item below names the document, the location and the replacement or insertion, for the owner or the document's next revision to apply. Items marked **door** close the corpus's WASI question.

| # | Document (revision read) | Where | Amendment |
|---|---|---|---|
| A1 **door** | HGBA-2 2.4.2 | 6, Stage 3 table, X-3 row | Replace the X-3 row with: "\| X-3 \| Native Component Model (optional) \| Native CM in the engine passing the CM conformance suite, counting toward the \"two engines\" gate **for web exposure only**. The app host (*Izcal app host*, AH-D15) is the Stage 3 deliverable that consumes this engine work; components executed inside the app host have no web surface (AH 6.15) and do not count toward the gate \| Stay on jco-style glue; the app host does not depend on this row \|" |
| A2 **door** | HGBA-2 2.4.2 | 6, Stage 3 table, row X-3 | **Subsumed by A1; A1 is the single replacement and no second edit is applied.** A1's replacement text already names the app host (AH-M0–AH-M5) and the Stage 3 staffing is carried by A5 (Stage 1's backlog row) and AH 9.3. |
| A3 | HGBA-2 2.4.2 | 2.3 (M-1) | No text change; record that *Izcal app host* Section 7 shows compliance, and that its component/WASI test suites are test-only oracles. |
| A4 **door** | Stage 1 design 0.14.3 | 6, out-of-scope bullet (line 420) | Record the **narrowed refusal**: **"the browser as a window manager for local apps"** — a refusal the section itself scopes to **Stages 1 and 2** — is narrowed to **arbitrary native local apps** (native binaries, OS processes and ambient-authority applications), not to every way of running a program. Add the parenthetical: "(Stages 1–2. A Stage 3 app host runs signed WASI components in browser-owned windows under the broker's consent — *Izcal app host*, AH 6.15; the refusal's object, arbitrary native local apps, is unchanged.)" |
| A5 **door** | Stage 1 design 0.14.3 | 9, Stage 3 backlog row (line 567) | "native Component Model (optional)" becomes "native Component Model (optional) **and the app host (AH-M0–AH-M5; *Izcal app host*)**"; add the app-host staffing note, 15–21† engineers overlapping extensions' directory and signing tracks. |
| A6 | Stage 1 design 0.14.3 | 2.1, crate layers | L2s gains `hg-wit`, `hg-app-manifest`, `hg-app-ticket`; L2e gains `hg-component`; L5 gains `hg-wasi-host`, `hg-app-host`, `hg-app-package`, `hg-app-registry`, `hg-app-replay`, `hg-app-observe`; L6 gains `hg-app-host-main`. |
| A7 | Stage 1 design 0.14.3 | 2.2, G-rule registry (line 142) | Assign **G220–G229** exactly as written in AH 6.17. |
| A8 | Stage 1 design 0.14.3 | 8.1 milestones and 9 backlog | Insert the Stage 2 seams of AH 6.19 before the Stage 3 app-host row. |
| A9 | Security scope 0.3.7 | 6.3 process map | The component host's row gains the **app, app-replay and install-verify configurations**, and the renderer row's keying admits `app://<id>` as its site component (an app UI is an ordinary renderer). The map's count is unchanged by this spec: the post-devices map is **37 kinds, 39 threat-model files**, and the app host adds neither a kind nor a file; the component-host threat model gains an **app-profile section with a subsection per parser family** (package/ZIP, component binary, WIT, canonical ABI), each naming its bounds, memory-safety argument, failure mode and fuzz target. |
| A10 | Security scope | 6.6 IPC table and rules | Add document endpoint `app.intent`, bound to `app://<id>` documents **of the same id** (the principal is the frame's committed app origin, checked by IPC-R12's binding and origin rules), plus `app.log` and `app.control`; and the non-document endpoint `app.invoke`, reachable only from the shell's broker channel, an app host process (principal `app:<id>`) and an agent task (principal `agent:<task>`). Add rules: no web document may call an `app.*` endpoint; no `app://` document may call `app.invoke` or a `shell.*` endpoint; an `app.intent` call from a document of another app id is refused. |
| A11 | Security scope | 6.6a, CON-R rules | Add prompt classes `app-file-read`, `app-file-write` (high), `app-network` (medium), `app-intent` (medium; high for writes), `app-autostart` (medium), `app-clipboard` (high for read, medium for write), `app-open-url` (medium), `app-notify` (medium), `app-stdin` (low), `app-long-intent` (medium). Extend A-SH-12's person-present rule to every `app-*` high class. |
| A12 | Security scope | 6.13, REC-R rules | Add class `app.wasi`: host-call log, virtual FS/network, recorded clocks/entropy; app-session id joins the app-host record with the UI renderer's record. Retention follows DT-D2. Replay is jitless by default; the recorded-tier **engine-triage mode** is REC-R6's exception and REC-R1's isolation applies unchanged. |
| A13 | Security scope | AGT-R3, AGT-R6 | AGT-R3's scheme rule extends to `app://` (agents never navigate to an app origin; they invoke intents through `app.invoke` with principal `agent:<task>`). AGT-R6 counts an app intent that changes state as state change. |
| A14 | Security scope | SEC-18, SEC-19 | Extend to app intents: an agent's app-intent confirmation is person-present, and a synthesized approval is rejected. |
| A15 | Coding standard 0.3.6 | Generated rule index (CS-GA-4) and Stage 1 2.2's range registry | Register G220–G229 as in AH 6.17. |
| A16 | Coding standard 0.3.6 | CS-ST-10 | The app configuration states a ceiling (Baseline default; Full or Jitless per policy); replay is jitless by default; install-verify links no codegen; the loaded-library closure is checked per configuration (G224); the app host's parsers are tier P. |
| A17 | JavaScript engine 0.4.3 | JS-D10, 6.0 embedders, 9.4 | Add the component host's **app configuration**, its `--replay` configuration (jitless by default) and its `--verify` configuration beside the extension configuration: same binary source, same IPC schema, app profile. The tier-P build's `hg-codegen` is linked only when the ceiling allows W1/W2, and never in `--verify`. |
| A18 | JavaScript engine 0.4.3 | G6, G14 | G14's jitless loaded-library closure includes `hg-app-host-main` **in the Jitless configuration only**; the App configuration at Baseline/Full links the tier-P codegen and is outside the jitless closure by declaration. G6's L2e scope is unchanged. |
| A19 | JavaScript engine 0.4.3 | JS-D13 | App tier ceilings: Baseline default, Full per app or policy, Jitless selectable; the ceiling is a policy input, and the per-configuration closure check is G224 (AH 6.2). |
| A20 | Devices 0.3 | DV-D2, DV-D4, DV-D8, DV-D9 | DV-D2's verifier accepts an app-id-bound ticket (same format; the principal field carries `app:<id>` where a site ticket carries the top-level site); DV-D4's chooser accepts `app://<id>` as a requesting principal; DV-D8 replays app device sessions through the virtual device; DV-D9 notes that app exposure follows the same template (AH 6.15). |
| A21 | Storage 0.3.2 | SG-D4, 6.14, G150 | The quota model admits an app bucket keyed by (app id, container); the picked-file path admits a **directory** grant with read/write split; G150 is unchanged and this spec adds that the app host holds no file handle. **Path resolution at the OS boundary follows AH 6.11's five rules** — handle-based resolution, `O_NOFOLLOW` plus intermediate-component identity checks, trailing-separator canonicalization before open/rename/link, access-time symlink permission checks, and rejection of renames/links that cross the preopen. This is a requirement on the storage service, where AH-16 is measured, not on the app host. |
| A22 | Networking 0.3.7 | NW-D4, NW-D7, content-blocking section | NW-D7's partition tuple for app traffic: T = F = O = `app://<id>`, C the container, X = 0, N = 0, n = 0, Cr = `same-origin`, R the matched declared route; the cache key is (T, C, F, X, N, n), connection-level keys add R and O, so no app key can equal a web key and two apps share no cache, connection or DNS result. NW-D4 mints LNA tickets for app ids with the same MAC and expiry; content blocking is **not** applied to app HTTP (AH-D10), and the app panel shows destinations instead. |
| A23 | Extensions 0.4.1 | EX-D10, EX-D12 | The directory, signing service and review process are shared with apps; app review adds the mechanical capability diff; EX-D12's no-telemetry rule applies to app installs and launches. |
| A24 | Shell 0.3 | 6.3 endpoint list, 6.8 windows, SH-D11 | Add `shell.apps` (the app panel: list, install from file, uninstall, open data folder) bound to `izcal://shell`; the shell owns app windows and their chrome (6.8); the prompt ledger records install grants, growth grants and revocations. |
| A25 | DevTools 0.3.6 | DT-D2, DT-D5 | DT-D2 gains the `app.wasi` class and the two-record join; DT-D5 gains the IIP `app` domain (host calls, grants, denials, ticket metadata, intents). |
| A26 | Phishing protection 0.3 | installed web apps section | Distinguish an **app window** (app chrome, no address bar, `app://` origin, no web navigation in place, a shell-drawn identity strip that app content cannot cover) from an installed site's standalone window. Add the rule that app content must not imitate shell or browser chrome — a fake address bar, a fake prompt or another app's identity is a review refusal. |
| A27 | Resource footprint 0.4.4 | FP-1, FP-2, FP-D8, FP-D9 | FP-1 excludes the app host (zero at idle) and gains two lines on the **charged metric of the footprint spec §6.2**: app host base ≤ 4 MiB† (a separate line from the ≤ 2 MiB† per-extension share, because the app configuration's linked decoders differ) and per live app ≤ 8 MiB† incremental, with the tier-P codegen build charged at ≤ 2 MiB† of that line and guest linear memory charged to the app's quota; FP-D8 keys app origins; FP-D9's statistics never contain app ids. |
| A28 | Content blocking (current revision) | Scope | App HTTP is out of scope for blocking, with the reason and the reversal trigger of AH-D10. |
| A29 | Accessibility (current revision) | App surfaces | App window chrome meets the shell's targets; an app's sealed UI inherits the engine's checks; the component has no accessibility surface of its own. |
| A30 | DOM, bindings and event loop (current revision) | Navigation | A web document's navigation to `app://` is refused as a non-web scheme; `app://` documents are not web documents and have no registrable domain, no web storage and no web permissions (AH 6.8). A navigation **from** an `app://<id>` document that would replace it — `location`, meta refresh, form submission, `window.open` on the window — is refused when the target is a web URL or a different app id, at the navigation layer and against the frame's app id; it opens a new tab or the target app's window instead (AH 6.8). |
| A31 | Licence audit and the ledger | Dependencies; Stage 3 plan | Record the Component Model and WASI test suites as test-only oracles with their licences; add AH-M0–AH-M5 to the Stage 3 plan.

### 9.6 Open questions

See 11.1; flagged claims in 11.2.

## 10. Risks

| Risk | Likelihood | Impact | Mitigation | Trigger |
|---|---|---|---|---|
| The component decoder and canonical ABI are larger than estimated | High | Schedule | AH-M0 measures the conformance corpus; the profile can ship with fewer interfaces (cli, clocks, random, io, private FS) before HTTP | AH-M0 exceeds its exit by 6 months† |
| The app host is built on a standard that was never aimed at browsers | High | Strategy | The host runs **installed apps**, not page content, and supplies the five things a browser WASI host would otherwise lack — a permission UI, app identity, a package format, an update channel and an admission rule (6.4, 6.7, 6.12, 6.15); `izcal:app@0.1` is an internal profile that can move if the Component Model's web interactions land differently ([BS 29–30, 36–37]) | A W3C/CG decision that defines a different component/UI boundary for browsers |
| W1 is too slow for real apps | Medium | Product | Baseline default; W2 per app; the corpus in AH-6 measures | AH-7 misses on the app corpus |
| Review capacity starves | Medium | Trust | Capability diffs are mechanical; the named-app list bounds volume; shared with extensions | Review p95 > 10 days† for two quarters |
| The directory has no apps | **High** | Platform | Chrome Apps reached only ~1% of desktop users inside the dominant browser before being wound down over twelve years ([HP 2.3, 2.9]); mitigations are developer mode and file install from day one, a seed corpus of Izcal-built tools, and **≥ 10† external authors** in the AH-M5 corpus | Fewer than 25† apps, or fewer than 10† external authors, at AH-M5 |
| A guest-to-host escape in the component runtime | Low | Severe | The 2019–2026 record puts every guest-to-host memory escape in the compiler, allocator or canonical ABI — zero in the WASI interfaces, whose bugs are permission bypasses and DoS ([SC 3.5, 6.4]); tier-P builds, fuzzing, the compromised-host harness, W^X, the machine-code validator, and W0 as the fallback ceiling (AH-D14) | Any escape: W0 for all apps until fixed; the case joins AH-16's corpus |
| The macOS storage path escapes the app's bucket | Medium | Severe | The app host holds no OS file handle (G150); the storage service implements the manual `openat` + `O_NOFOLLOW` walk on macOS (no `openat2`/`RESOLVE_BENEATH`), with trailing-slash open and rename canonicalization, access-time symlink checks and rename/link permission checks, fuzzed per AH-16 ([SC 2.1–2.11, 7.1]) | Any escape: the path implementation is rewritten or the directory grant class is withdrawn |
| The intent protocol becomes a de facto API that apps depend on | Medium | Design | Intents are versioned in the manifest; the profile version gates them | A breaking intent change with no version path |
| App storage grows without bound | Medium | Footprint | Quota, eviction and the app panel's data line | Quota misses on the reference device |
| The named-app list becomes permanent | Medium | Strategy | Every per-capability exception has a 12-month† expiry; the decision log lists the list and every exception | A per-capability exception renewed three times without a gate review |
| No install telemetry removes the channel that detected CCleaner, ASUS and 3CX | Medium | Trust | The named substitutes are commitments, not an absence: local audit the person can read, the public decision log, review triggers on ownership change and capability deltas, and a disclosed kill switch (6.7, [DU 3.5–3.6]) | A signed malicious update is found late and none of the substitutes surfaced it |
| A signed app impersonates a known app's name or icon | Medium | Trust | Names and icons are directory-bound, duplicate and confusable names are refused, developer-mode apps are labelled unverified, and the window chrome is shell-drawn (6.3, 6.8) | A confirmed impersonation install: add a domain claim to the manifest (Q-AH-5) and require it for the directory |
| Apps expect POSIX semantics the bucket does not have (links, locks, permissions, `mmap`) | High | Compatibility | `ENOTSUP` is explicit; the tier-1 corpus measures the gap; granted directories carry the richer semantics | AH-6 misses and > 30%† of corpus failures are filesystem semantics: widen the translation or add a per-app ext4-image class |
| The sealed UI and the component drift into two stateful apps | Medium | Design | One component, session ids on intents, UI re-queries state after a restart (6.8) | A sev-2 divergence bug between the two records of one app session |
| The recording store becomes an at-rest target holding app files and credential-class randomness | Medium | Severe | Encrypted at rest under REC-R2, per-app quota and eviction, entropy values never exported, unpinned recordings deleted on uninstall with a pin option (6.13) | Any path that exports an entropy value or a content-addressed payload, or a bundle readable after the 30-day† window without a pin: the store is re-keyed and the path is closed |

## 11. Open questions and flagged claims

### 11.1 Open questions

1. **Q-AH-1.** What is the measured overhead of the canonical ABI on Izcal's engine, per type class? Needed for AH-7 and for the 0.3 decision. Measured at AH-M0.
2. **Q-AH-2.** Does any browser team have an unstated position on WASI in the browser that would change the two-engine gate? The published positions are in [BS] (Mozilla positive on the Component Model, none on WASI; V8 evaluating; WebKit silent); a direct statement would be evidence.
3. **Q-AH-3.** Can the storage service's file path meet WASI filesystem semantics (rename, directory iteration, metadata) within its quota model, or does it need a WASI-specific view? Spike at AH-M0.
4. **Q-AH-4.** What is the smallest set of apps that proves the profile (AH-6's 25†)? The corpus is named in the notes.
5. **Q-AH-5.** Does the directory need domain-bound identity to prevent impersonation, or is directory-bound naming plus the key fingerprint enough for people? The name/icon binding and duplicate refusal are decided (6.3); a domain claim is the reversal trigger of the impersonation risk in 10, and a user study is not planned.
6. **Q-AH-6.** How does an app declare a default handler for a file type without becoming ambient authority (open-with)? A `file-handler` intent plus a per-type grant is proposed; the OS/desktop integration is out of scope here.
7. **Q-AH-7.** Is a per-intent CPU deadline enough for service apps whose work is legitimately long (a build, a large conversion), or does the profile need a background budget class with its own accounting? Measure on the AH-6 corpus and decide at AH-M3.
8. **Q-AH-8.** Does the WAST suite cover the **canonical-ABI transcoding** surface well enough to be the conformance bar, given the 2026 batch of transcoding CVEs in other implementations? If not, Izcal needs its own cross-type transcoding corpus before AH-M1's exit; the failure action is AH-5's.
9. **Q-AH-9.** Should the package signature move to a standard envelope (a Sigstore bundle or an OCI referrer) if one emerges, or stay Izcal's own published format? Watch the W3C WASI subgroup and the Bytecode Alliance; the format is versioned either way (AH-D17).

### 11.2 Flagged claims

- **Single-source:** the 3.5× async overhead figure is the Bytecode Alliance's, not independently measured on Izcal (Medium).
- **Unverified:** the exact set of WASI 0.2 interfaces that a real app corpus imports; AH-6 measures it.
- **No public benchmark:** no published overhead exists for a WASI-boundary record/replay host. The nearest public numbers are CPU-level TTD (10–20×) and whole-VM simulation (Antithesis), neither of which is this cost model; AH-15 measures and publishes.
- **Abstract-only numbers:** Wasm-R3's 99.53% trace reduction and 27 applications are taken from the OOPSLA 2024 abstract, not a reproduction (DR 3.3, Medium).
- **Report-via-summary:** Apple's 2025 App Store figures (9.1M reviewed, 2.09M rejected) and Google's 2024 prevention counts were read through search summaries of the official reports, not the reports' own files; the orders of magnitude are safe, the last digit is not (DU 1.2, 1.6, Medium).
- **Contested:** whether an app platform without raw sockets is useful enough; the corpus decides, not this document.
- **Absence by search:** "WebKit has no position" and "Chromium has no WASI feature entry" are established by searching the public trackers on 21 October 2026, not by an exhaustive listing; they are exactly as strong as the trackers (BS 22–23, Medium).
- **Different units, different years:** Web Almanac 2022 measured 0.06% of domains using Wasm, while a July 2025 crawl measured 0.35% of desktop pages; the units and years differ, and this spec cites only the page-share figure, not a trend line (GO 1.2, BS 45, Medium).
- **No stable release exists for the Component Model:** the repository has no tagged releases, so the spec's pin is by document path, SHA-256 content digest and retrieval date (21 October 2026), with a dated snapshot archived; the digest freezes content but not the upstream document's identity, and a future tagged release is the better pin; the WASI side is pinned to the 0.2.12 tag (SP 1.15, 2.2, Medium).
- **Deliberately behind the current preview:** the profile pins WASI 0.2.12 while 0.3.2 shipped on 13 October 2026; the choice is a recording-surface and stability decision with a named reversal trigger (AH-D4), not a claim that 0.3 is unusable (SP 1.11, Medium).
- **Inferred release date:** the WASI 0.2.12 date (2 June 2026) is inferred from the 0.2 release train, not read from a dated release page; the pin is the tag, not the date (SP 1.10, Medium).
- **Unmeasured on Izcal:** the macOS containment rules in 6.11 are specified from the published vulnerability class, not yet from Izcal's implementation; AH-16 is the measurement, and until it passes the risk row in 10 stays at Medium/Severe (SC 2.1–2.11, Medium).
- **No standard to conform to:** the package signature format, the canonical digest and the key-rotation statement are Izcal's own design; "no standard component signature format exists" is a statement about the specifications as of 21 October 2026, and it may change (SC 6.12, 7.2–7.4, Medium).
- **No CM threat model exists:** Izcal's app-profile threat-model section (9.5 A9) is written by this spec, not inherited from the Component Model or WASI communities; it is therefore an Izcal claim, reviewable as such (SC 6.3, 6.12, Medium).
- **Recent CVE class:** the 2026 canonical-ABI transcoding vulnerabilities were found in other implementations; Izcal's `hg-component` is new code and has no history, so the absence of a CVE in it is not evidence of absence (SC 6.4, Q-AH-8, Medium).

### 11.3 References

| Key | Source | Where |
|---|---|---|
| [H2] | *HGBA 2 browser architecture spec*, 2.4.2 | `reports/HGBA 2 browser architecture spec.md` |
| [H3] | *Future software browser engine* (HGBA-3), 3.3.0 | `reports/Future software browser engine.md` |
| [S1] | *HGBA Stage 1 architecture design*, 0.14.3 | `reports/HGBA Stage 1 architecture design.md` |
| [SS] | *Izcal security scope*, 0.3.7 | `reports/Izcal security scope.md` |
| [CS] | *Izcal coding standard*, 0.3.6 | `reports/Izcal coding standard.md` |
| [JS] | *Izcal JavaScript engine*, 0.4.3 | `reports/Izcal JavaScript engine.md` |
| [EX] | *Izcal extensions*, 0.4.1 | `reports/Izcal extensions.md` |
| [DV] | *Izcal devices*, 0.3 | `reports/Izcal devices.md` |
| [SG] | *Izcal storage*, 0.3.2 | `reports/Izcal storage.md` |
| [NW] | *Izcal networking*, 0.3.7 | `reports/Izcal networking.md` |
| [FP] | *Izcal resource footprint*, 0.4.4 | `reports/Izcal resource footprint.md` |
| [SH] | *Izcal shell*, 0.3 | `reports/Izcal shell.md` |
| [DT] | *Izcal DevTools spec*, 0.3.6 | `reports/Izcal DevTools spec.md` |
| [DR] | *Izcal app host: determinism and record/replay for WASI applications* (research note) | `research_notes/Izcal app host/determinism_recording.md` |
| [DU] | *Izcal app host: distribution and trust* (research note) | `research_notes/Izcal app host/distribution_trust.md` |
| [GO] | *Izcal app host: gaps and opportunities* (research note) | `research_notes/Izcal app host/gaps_opportunities.md` |
| [BS] | *Izcal app host: browser stances on WASI/Component Model, and app windows, launch, automation* (research note) | `research_notes/Izcal app host/browser_stance_ui.md` |
| [SP] | *Izcal app host: WASI and Component Model specifications an app host must implement* (research note) | `research_notes/Izcal app host/specifications_standards.md` |
| [HP] | *Izcal app host: history and post-mortems of prior app platforms* (research note) | `research_notes/Izcal app host/history_postmortems.md` |
| [SC] | *Izcal app host: security, capability and supply-chain record* (research note) | `research_notes/Izcal app host/security_capabilities.md` |
| [BA] | Bytecode Alliance, "The road to Component Model 1.0" | https://bytecodealliance.org/articles/the-road-to-component-model-1-0 |
| [W3] | WASI.dev, WASI 0.3.0 release page (11 June 2026) | https://wasi.dev/releases/wasi-p3 |

Two references — [SS] and [CS] — are at the stated revisions in the `security-scope` and `cs-amendments` worktrees respectively, not yet merged into this branch; every other path resolves from this worktree.

## Appendix A. Phase 8 disposition

## Appendix A. Phase 8 disposition

**Independence.** Both reviews were run by agents that did not write the specification and did not take part in phases 4–7; neither was given the other's findings. The fact-checker checked draft 0.1 at SHA-256 `25a61f14…`, traced every load-bearing claim to the research notes and, where needed, to the published prose source; the design critic checked method, M-1, consistency with the accepted corpus, security, process model, admission, recording, staging and targets. The reviewers' own files are `research_notes/Izcal app host/factcheck.md` and `critique.md`.

### A.1 Design critique: 37 findings, all applied

| ID | Severity | Disposition | Finding (short) | Where applied |
|---|---|---|---|---|
| C1 | Blocker | Applied | §6.3 line 246 derives the app id as `base32(SHA-256("izcal-app-v1" \|\| author_public_key \|\| package_name))[0..26]`. | 6.3 stable `app_id`; 6.7 succession; 6.18 alternatives; AH-D2 |
| C2 | Blocker | Applied | §6.7 line 394 has the broker verify both signatures, compute the capability surface from the component's imports and ... | 6.2 install-verify configuration; 6.7 install grant; A9; A17 |
| C3 | Blocker | Applied | §6.8 lines 420-424 runs the UI in an ordinary renderer with the full engine on `app://`, a tuple origin with storage. | 6.8 closed UI surface; 6.4 `host` lines; AH-17 |
| C4 | Blocker | Applied | §6.10 line 448 says the person grants a device family at install. | 6.10 removed the install-time family grant |
| C5 | Blocker | Applied | §6.8 line 426 makes `app.intent` available only to documents of the app's own origin. | 6.8 `app.intent` and `app.invoke`; 6.14 principal table; A10 |
| C6 | Major | Applied | §6.2 line 230 runs app replay at "the recorded ceiling", while REC-R6 makes replay jitless by default with an engine-... | 6.2 replay row; 6.13 jitless default and engine-triage; A12 |
| C7 | Major | Applied | §6.4 line 293 says an app grant lasts while installed, and a remembered grant's expiry "follows CON-R2's per-class ex... | 6.4 grant-lifetime table; 6.14 `app-long-intent` |
| C8 | Major | Applied | §6.15 lines 540-543 applies ST-D18 and LY-D10 to a feature the same section says is not web-facing, then borrows "the... | 6.15 three instruments (gate, named list, per-capability exception) |
| C9 | Major | Applied | §6.15 line 543 says the 20† allowlist bounds exposure. | 6.7 developer mode bound by the list; 6.15; AH-D8 |
| C10 | Major | Applied differently | §6.15 line 547 and A4 line 735 treat "the browser as a window manager for local apps" as a permanent refusal and "nar... | 6.15 prose cites the line as Stages 1-2 scoped; A4 records the time scope and narrows only the object. Applied in a different form: the refusal is kept and the amendment no longer implies it was permanent |
| C11 | Major | Applied | §6.2 lines 224-234 claims one process kind with "the same source, the same IPC schema, the same review surface", whil... | 6.2 review-delta paragraph; A9 threat-model subsections; AH-D1 trigger |
| C12 | Major | Applied | §6.14 line 530 raises the 10 s† intent deadline only by the app's manifest, and §6.4's manifest sketch at line 274 ca... | 6.14 deadline ceiling in policy; 6.4 manifest note; `app-long-intent` |
| C13 | Major | Applied | §6.3 line 250 refuses only web-to-`app://` navigation. | 6.8 no in-place navigation and no chrome imitation; A26; A30 |
| C14 | Major | Applied | §6.3 binds identity to the author key and displays a fingerprint. | 6.3 directory-bound names and icons; 10 impersonation risk; Q-AH-5 |
| C15 | Major | Applied | §6.11 presents the app bucket as not web storage, but §6.8's ordinary renderer on `app://` has localStorage, IndexedD... | 6.8 web storage denied; 6.11 one store; A30 |
| C16 | Major | Applied | §6.13 line 508 stores an item over 64 MiB† as size and hash with the content absent and labels the replay approximate. | 6.13 approximate sessions excluded from AH-3; AH-3 gap rate |
| C17 | Major | Applied | §6.7 line 392 revokes a compromised author key through the directory's TUF channel by delisting the app. | 6.7 revocation record; 6.12 installed-copy revocation; AH-10 |
| C18 | Major | Applied | §8 AH-6 line 660: if 25† apps fail, "the profile's gaps are named and the apps are re-scoped, never silently patched". | AH-6 corpus frozen at AH-M0; replacement and publication rules; AH-M0 |
| C19 | Major | Applied | §9.3 line 698 totals 15-21† engineers over 18-24 months as 30-50 engineer-years†; the multiplication gives 22.5-42. | 9.3 arithmetic per track, overlap subtracted, non-engineering separate |
| C20 | Major | Applied | §6.15 line 547 claims the Stage 3 deliverable has "a named owner, scope, targets and cost". | 9.2 months column and owner role; AH-M0 entry condition |
| C21 | Major | Applied | §6.7 says the canonical digest construction is Izcal's own and AH-D17 publishes it, but the spec does not state its p... | 6.7 canonicalization and ZIP invariants; AH-4 and AH-16 fuzzing |
| C22 | Major | Applied | §6.9 line 440 sets T = `app://<id>` in NW-D7's partition key but leaves F, R, X, N, n, O and Cr unassigned for app tr... | 6.9 full tuple; A22 exact values |
| C23 | Major | Applied | §6.13 records file contents content-addressed, network bytes and credential-class entropy draws in an encrypted bundl... | 6.13 recording store owner, quota, export and uninstall; 10 risk |
| C24 | Major | Applied | §7 line 641 pins the Component Model documents by path and retrieval date because the repository has no tagged releases. | 6.6 content digest pin; 6.6 WAST commit; 7; 11.2 |
| C25 | Major | Applied | §6.3 line 252 installs an app per profile and makes grants and tickets per container. | 6.7 install grant per (app, container); 6.12 first-launch prompt |
| C26 | Major | Applied | §8 AH-8 line 662 sets app host base at most 4 MiB† and per-app at most 8 MiB†. | 6.16 charged-metric definition and base; AH-8; A27 |
| C27 | Minor | Applied | The header (lines 1-5) omits the template's summary paragraph and the Date, Status, Research notes and Relates to line. | Header summary paragraph; this appendix |
| C28 | Minor | Applied | §4.4's assumptions table (lines 142-156) records Break and Partly only; no row is a whole Keep. | 4.4 statement that no assumption is kept whole |
| C29 | Minor | Applied | Amendment locators and ordering: A1 and A2 both rewrite HGBA-2's X-3 row with different replacement text and no merge... | A2 subsumed by A1; A4 to Section 6; A24 to 6.3 and 6.8 |
| C30 | Minor | Applied | A13 line 744 says AGT-R8's grant set includes (app, intent) pairs. | A13 drops the AGT-R8 clause |
| C31 | Minor | Applied | §9.4 line 704 names seven decisions the owner most needs to see (AH-D1, D3, D4, D6, D7, D8, D17), but nine are starre... | 9.4 signpost names all nine starred decisions |
| C32 | Minor | Applied | Targets leave parameters open: AH-4's "coverage floor" is not a number (line 658), AH-16's "90-day adversarial corpus... | AH-4 coverage floor; AH-12 study n and protocol; AH-16 case counts |
| C33 | Minor | Applied | §6.9 line 444 and AH-D10 turn content blocking off for app traffic. | 6.9 measurement; AH-18; AH-D10 measurable trigger |
| C34 | Minor | Applied | §6.5 line 320 lets stdin read when "the app declares `stdio` and the person pipes a file in", but `stdio` is not in t... | 6.5 `app-stdin` capability and `/data`; 6.4 manifest and prompt class; A11 |
| C35 | Minor | Applied | §6.13 records "the timing" for network exchanges (line 496) and hashes each host call's result into a running digest. | 6.13 explicit digest field list |
| C36 | Minor | Applied | §6.10 line 450 says the device arbiter "does not know or care that the client is an app". | 6.10 arbiter enforces the app principal |
| C37 | Minor | Applied | §6.13 line 492 makes epoch interruption a recorded backstop and says replay reproduces the stop without a real clock. | AH-2 excludes and reports epoch-backstop runs |

**Counts: 5 Blocker, 21 Major, 11 Minor.** No finding was rejected. **C10 is applied in a different form**: the critic is right that Stage 1's line is time-scoped, so the spec now cites it as such, and A4 records the time scope explicitly while narrowing only the refusal's object to native local apps. The other 36 are applied as proposed or more strictly than proposed.

### A.2 Fact-check: 65 findings, all applied

| ID | Verdict | Disposition | Claim (short) | Where applied |
|---|---|---|---|---|
| F1 | Corrected | Applied as must-fix | §1 "Those become AH-1 to AH-14." | 1 |
| F2 | Corrected | Applied as must-fix | §1 "A second native implementation is what the Component Model's Phase 4 gate waits for [BS 28]." | 1 |
| F3 | Confirmed | Confirmed; no change | §1 and §3.2: no browser ships the Component Model or WASI natively, as of 21 October 2026. | - |
| F4 | Confirmed | Confirmed; no change | §1 and §3.2: Mozilla resolved a positive Component Model position on 12 October 2026 and is prototyping; qu... | - |
| F5 | Confirmed | Confirmed; no change | §1 and §3.2: Google/V8 opened an issue to evaluate the Component Model and has committed to nothing; WebKit... | - |
| F6 | Confirmed | Confirmed; no change | §1: "21.2% of surveyed browser-API developers"; "0.35% of desktop pages"; "2.05% of the top 1,000". | - |
| F7 | Confirmed | Confirmed; no change | §2 chronology: ActiveX retired in 2022 with IE mode to at least 2029; Java Web Start excluded from Java SE ... | - |
| F8 | Confirmed | Confirmed; no change | §2 chronology: NaCl deprecated in 2017, web-platform support removed in Q4 2019, inside Chrome Apps to 2021... | - |
| F9 | Confirmed | Confirmed; no change | §2 chronology: Chrome Apps launched 2013, "~1% of desktop users" in 2016, final end of life October 2028. | - |
| F10 | Corrected | Applied | §2 "phased out through 2020"; §2 table "ended desktop support in June 2022". | 2, 5 |
| F11 | Overstated | Applied | §2 table: "WhatsApp never shipped" on Firefox OS. | 2 |
| F12 | Confirmed | Confirmed; no change | §2 table: Firefox OS carrier strategy stopped in December 2015, development ended in July 2016. | - |
| F13 | Confirmed | Confirmed; no change | §2 table: KaiOS shipped 100 million devices by 2019, third mobile OS. | - |
| F14 | Overstated | Applied | §2 "Sandbox quality decided none of these outcomes." | 2, 5 |
| F15 | Overstated | Applied | §1 table and §3.4: "Every documented supply-chain incident arrived through the legitimate signed update cha... | 3, 6.7 |
| F16 | Overstated | Applied | §2 table: "single-engine status is the one cause every postmortem shares". | 5 |
| F17 | Confirmed | Confirmed; no change | §3.1 and §1: WASI 0.2 vote on 25 January 2024. | - |
| F18 | Confirmed | Confirmed; no change | §3.1 and §6.5: WASI 0.3.0 on 11 June 2026, 0.3.1 on 11 August 2026, 0.3.2 on 13 October 2026. | - |
| F19 | Confirmed | Confirmed; no change | §3.1: Component Model at Phase 1, no tagged releases, no formal specification, no reference interpreter. | - |
| F20 | Confirmed | Confirmed; no change | §3.1: Component Model "cannot reach 1.0 'without native implementation in at least two browser engines'"; W... | - |
| F21 | Confirmed | Confirmed; no change | §3.2: the WASI subgroup scopes itself to "non-browser and non-JS engines"; "we don't think about user agents". | - |
| F22 | Confirmed | Confirmed; no change | §3.2: jco now emits a `"use components"` marker. | - |
| F23 | Overstated | Applied | §3.2: "WASI itself was never proposed for the web." | 1, 6.15, 11.2 |
| F24 | Corrected | Applied | §3.3: "`wit-bindgen` covers Rust, C/C++, C#, Go and the componentize siblings [GO 2.1]." | 3 |
| F25 | Confirmed | Confirmed; no change | §3.3: wasm-tools is static; no standard sampling profiler; DevTools timing unreliable; wasmCloud "can't att... | - |
| F26 | Confirmed | Confirmed; no change | §3.3: "7 code-generation defects" in WIT binding generators producing crashes and silent data corruption. | - |
| F27 | Corrected | Applied | §3.4: "Apple: 9.1M submissions, 2.09M rejections in 2025, ~500 reviewers". | 3 |
| F28 | Confirmed | Confirmed; no change | §3.4: Google prevented 2.36M policy-violating apps in 2024; 92% of human reviews AI-assisted. | - |
| F29 | Corrected | Applied as must-fix | §3.4 and §6.7: "1.2M-plus cumulative malicious packages, 209-day median detection" / "1.2 million-plus cumu... | 3 |
| F30 | Confirmed | Confirmed; no change | §3.4: WARG's federated registry was deprecated and the Bytecode Alliance moved to OCI artifacts via `wkg`; ... | - |
| F31 | Confirmed | Confirmed; no change | §3.5: Microsoft TTD pays 10–20× and produces gigabytes per few minutes. | - |
| F32 | Corrected | Applied as must-fix | §3.5: "whole-VM simulation finds real bugs at 700–1,000× acceleration (TigerBeetle VOPR-1000, FoundationDB'... | 3, 5, 6.16 |
| F33 | Corrected | Applied as must-fix | §5 comparison table: "TigerBeetle and FoundationDB simulate at 700–1,000× to find bugs [DR 3, 5]". | 3, 5, 6.16 |
| F34 | Corrected | Applied | §3.6: "only 17% of people read install prompts in the 2012 study". | 3 |
| F35 | Confirmed | Confirmed; no change | §6.5: "WASI 0.2.12 (2 June 2026)". | 6.5, 11.2 |
| F36 | Corrected | Applied as must-fix | §6.5: "0.3.2 is three weeks old". | 6.5 |
| F37 | Confirmed | Confirmed; no change | §6.5: "the Bytecode Alliance measured at roughly 3.5× overhead even on synchronous call paths". | - |
| F38 | Corrected | Applied as must-fix | §6.6: "Wasmtime's CVE-2026-47218 is exactly that class — a crafted cached compilation bypassed validation". | 6.6 |
| F39 | Corrected | Applied as must-fix | §6.6: "`CVE-2026-27572`'s lesson is that the runtime's own panic path is a denial-of-service surface a host... | 6.6 |
| F40 | Confirmed | Confirmed; no change | §6.6: flat values are only `i32`, `i64`, `f32`, `f64`; string and list lengths at most 2^28−1; handles are ... | - |
| F41 | Confirmed | Confirmed; no change | §6.6: "no Component Model conformance suite yet — only a growing WAST suite". | - |
| F42 | Corrected | Applied as must-fix | §6.7: "WARG (before its archive) and `wkg`/OCI carry digests, which prove integrity and nothing about who s... | 6.7 |
| F43 | Corrected | Applied as must-fix | §6.7: the wasmCloud default-deny pattern is cited [SC 6.9]. | 6.7 |
| F44 | Corrected | Applied as must-fix | §6.7: TUF's offline-root discipline is cited [SC 6.10–6.11]. | 6.7 |
| F45 | Confirmed | Confirmed; no change | §6.7: no standard component signature format exists. | - |
| F46 | Confirmed | Confirmed; no change | §6.7: no published Component Model threat model exists. | - |
| F47 | Confirmed | Confirmed; no change | §6.7: updates are the attack surface; anti-rollback, digest-as-identity, TUF metadata, staged rollout not a... | - |
| F48 | Confirmed | Confirmed; no change | §6.10–6.11: path-resolution class — cap-std trailing slash (GHSA-hp8f-xmx4-4qrg), Wasmtime's own advisory a... | - |
| F49 | Confirmed | Confirmed; no change | §10: "every guest-to-host memory escape in the compiler, allocator or canonical ABI — zero in the WASI inte... | - |
| F50 | Confirmed | Confirmed; no change | §6.10: CVE-2026-27204 lesson — limits on by default; a protective default left off does not exist. | - |
| F51 | Confirmed | Confirmed; no change | §6.10: adding budget checks to bulk memory and table operations produced CVE-2026-104855. | - |
| F52 | Confirmed | Confirmed; no change | §6.10: epoch interruption is a measured ~10% slowdown and non-deterministic; fuel is deterministic. | - |
| F53 | Confirmed | Confirmed; no change | §4.3 and §6.10: WASI defines no budget interface and no descriptor-revocation operation; destruction is the... | - |
| F54 | Confirmed | Confirmed; no change | §6.10 and §6.13: Deterministic profile — canonical positive NaN, relaxed SIMD one fixed lowering per ISA (e... | - |
| F55 | Confirmed | Confirmed; no change | §6.13: no published overhead or log size for a WASI-boundary record/replay host; nearest public numbers are... | - |
| F56 | Confirmed | Confirmed; no change | §6.13 and §11.2: Wasm-R3's 99.53% trace reduction and 27 applications are abstract-only. | - |
| F57 | Confirmed | Confirmed; no change | §2: WASI announced 27 March 2019; WebAssembly Core 1.0 became a W3C Recommendation on 5 December 2019. | - |
| F58 | Corrected | Applied | §9.3: "30–50 engineer-years† over 18–24 months† at the stated ramp" from 15–21† engineers plus 2–4† non-eng... | 9.3 |
| F59 | Confirmed | Confirmed; no change | §9.3: total of 15–21† engineers and 2–4† non-engineering from the four engineering rows. | - |
| F60 | Corrected | Applied as must-fix | §11.2: "Recent CVE class ... (SC 3.6, Q-AH-8, Medium)". | 11.2 |
| F61 | Confirmed | Confirmed; no change | Header: HGBA-2 2.4.2, HGBA-3 3.3.0, Stage 1 0.14.3, JS engine 0.4.3, extensions 0.4.1, devices 0.3, storage... | - |
| F62 | Confirmed | Confirmed; note added | Header and §11.3: [SS] *Izcal security scope* 0.3.7 at `reports/Izcal security scope.md`; [CS] *Izcal codin... | 11.3 |
| F63 | Confirmed | Confirmed; no change | §6.8–6.9 and §2 table: Wayland denies always-on-top and focus stealing; Chrome App Launcher removed from de... | - |
| F64 | Confirmed | Confirmed; no change | §3.2: Cloudflare Workers runs Wasm without WASI — experimental, partial syscalls, no threads; edge vendors ... | - |
| F65 | Confirmed | Confirmed; no change | §11.2: "WebKit has no position" and "Chromium has no WASI feature entry" are absence-by-search, exactly as ... | - |

**Counts: 43 Confirmed, 17 Corrected, 5 Overstated; 12 must-fix.** The 43 confirmed claims needed no text change. All 17 corrections and 5 overstatements are applied, including the 12 must-fix items; the fact-checker's recommended items (F10, F11, F14, F15, F16, F23, F24, F27, F34, F35, F58, F62) are applied, and the remaining confirmed rows are recorded so the next revision can re-check them against the same anchors.

