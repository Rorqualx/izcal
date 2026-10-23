# The incompressible remainder: what agentic coding does not compress in Izcal

Compiled 21 October 2026. Primary sources were preferred: the Izcal specs were read in the worktree, and every external claim below was checked on the page cited. Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only. † marks the corpus's own first-principles estimates. A † number is not a measurement; the confidence grade says only that the spec states it.

## 0. The remainder at a glance

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 0.1 | The corpus is already activity-class based, not a flat staffing model | 551–719 engineers†; 553–723 people†; about 2,000–3,400+ engineer-years†; Stage 1 exit 48–60 months† | Ledger C-6 addendum, `research_notes/Phase 3 amendments/batch3_decisions.md` §5 | 2026-10-21 | High |
| 0.2 | The eight Phase 2 component specs alone, before DOM, shell, storage, security, infrastructure and DevTools | 437–560 people†, 384–502 engineers†, about 1,645–2,855 engineer-years†, plus 150–400 designer-years† | `research_notes/Phase 2 reconciliation/reconciliation_2.md` §4 | 2026-10-19 | High |
| 0.3 | The largest single non-code block is type design | 150–400 designer-years† over 5–10 years†; about 45† designers; 5–10 year calendar set by the work, not the team | `reports/Izcal fonts and text.md` §9.3 | 2026-10-19 | High |
| 0.4 | The Stage 1 calendar is set by layout and graphics, not by implementation velocity | 48–60 months†, set by LY-M2/LY-M3 and GR-M3 | `reports/Izcal layout.md` §9.3; `research_notes/Phase 2 reconciliation/reconciliation_2.md` §4 | 2026-10-19 | High |
| 0.5 | Conformance targets are large, per-directory and run on fixed farms | 6,827 html5lib tokenizer tests; 1,973 WPT tree-construction cases; about 25,000 WPT layout files; 346 IDL files; test262 ≥ 99.5%† on 3 ISAs × 3 ceilings | `reports/Izcal HTML and CSS parsing and style.md` §9.2; `reports/Izcal layout.md` §9.3; `reports/Izcal DOM, bindings and event loop.md` §6.3; `reports/Izcal JavaScript engine.md` JS-1 | 2026-10-19 | High |
| 0.6 | Fuzzing is a weekly compute commitment with wall-clock SLAs, not a task that finishes | ≥ 1,000† CPU-hours per week for SVG alone; nightly campaigns on 3 ISAs; 14†/30†-day crash-age SLAs | `reports/Izcal SVG.md` SV-14; `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-5, SEC-10, SEC-11, SEC-28, SEC-30 | 2026-10-19/21 | High |
| 0.7 | Human AT testing needs a physical lab, licences and daily users of the AT | At least 8 Windows machines, 3 Macs, 2 Linux machines, 1 riscv64 board, an Android set; at least 2 engineers who use a screen reader daily; contracted studies each release | `reports/Izcal accessibility.md` §9.3, AX-D25, AX-40 to AX-42 | 2026-10-20 | High |
| 0.8 | Licence, patent and audit work ends in a signature, not a merge | 9–17 engineer-years† once plus 8–15 people† ongoing; AVC cap $9.75 M/year; 50–70 CA owners† reviewed initially | `research_notes/Phase 2 reconciliation/licence_audit.md` Summary; `reports/Izcal images and media.md` §6.2; `reports/Izcal networking.md` NW-53 | 2026-10-19 | High |
| 0.9 | Owner decisions are serial human throughput | 51 decisions accepted in batch 2 and 38 in batch 3, all in full, each with costs and reversal triggers | `research_notes/Phase 3 amendments/batch2_decisions.md`; `batch3_decisions.md` | 2026-10-20/21 | High |

**Implications:** the remainder is not a residual percentage left over after an AI discount. It is a set of classes with different binding constraints: compute, physical devices, scarce craft, accountable signatures, external calendars and one owner's attention. A single multiplier on C-6 would misstate both the total and the date.

## 1. Method, scope and evidence limits

This note records, per activity class, the spec's own numbers and whatever published external evidence bears on whether that class is compressible by coding agents. It does not propose the per-class translation factors; the other strands do. It does say where a class is bounded by something other than code production, and what the translation must therefore carry separately.

Evidence limits, stated plainly:

- The session's shared web-search budget was exhausted before this strand ran a search. No search-engine query was made and no mirror or proxy was used. External rows are direct fetches of known primary URLs on 21 October 2026; where a page did not state the figure sought, the row says so rather than guessing.
- The corpus contains no aggregate CI machine-year figure. It states CI compute as multiples (2–4× production time†) and as seed counts (10k DST seeds in T1, 1M in T2), not as machine-years.
- The corpus contains no aggregate fuzz CPU-hour figure. The only explicit CPU-hour commitment found is SVG's SV-14.
- No public price for a browser-grade external security audit was found. The audit evidence below is hours and project counts, not dollars.

## 2. The measurement calendar: † thresholds are dates

Every † in the corpus is a placeholder until a named milestone measures it. Those milestones are on the critical path, and several require multi-quarter observation before a threshold can be confirmed.

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | Style's measurement milestone is 6–9 months before its first build | ST-M0: 6,827 tokenizer tests, a design review, and measured spikes on the corpus; ST-M1 at 15–21 months | `reports/Izcal HTML and CSS parsing and style.md` §9.2, §9.3 | 2026-10-19 | High |
| 2.2 | Layout's measurement milestone is also 6–9 months; its exit is the programme's Stage 1 date | LY-M0 at 6–9 months; LY-M2 at 36–54 months; LY-M3 at 48–60 months | `reports/Izcal layout.md` §9.2, §9.3 | 2026-10-19 | High |
| 2.3 | The JS engine's design and measurement milestone is 9–12 months, on three ISAs | JS-M0 at 9–12 months: design review, spikes on x86-64, AArch64 and riscv64, K3 verification | `reports/Izcal JavaScript engine.md` §9.2, §9.3 | 2026-10-19 | High |
| 2.4 | Graphics' measurement milestone is 6–9 months and includes named vendor spikes | GR-M0 at 6–9 months; GR-S1, GR-S2, GR-S6 and GR-S9 done, GR-S3 and GR-S4 started | `reports/Izcal graphics and rendering.md` §9.2 | 2026-10-19 | High |
| 2.5 | Accessibility brings a security measurement forward to its own M0 | AX-M0 at 6 months; the security scope's M5 broker-mitigation measurement for NVDA, JAWS, Dragon, ZoomText and Magnifier runs there | `reports/Izcal accessibility.md` §9.2, §9.4 (AX-D9) | 2026-10-20 | High |
| 2.6 | Footprint's first measurements are before the first build and include a riscv64 device | FP-M0 before M1: empty-process probe on 3 OSes, spawn cost, a riscv64 reference device and runners; FP-M3 first full corpus at M4 | `reports/Izcal resource footprint.md` §9.2 | 2026-10-19 | High |
| 2.7 | DOM's M0 includes hardware and a first fuzz grammar | DM-M0 with JS-M0: a riscv64 board procured; first fuzz grammar | `reports/Izcal DOM, bindings and event loop.md` §9.2 | 2026-10-19 | High |
| 2.8 | Some thresholds cannot be confirmed without quarters of operation | ST-D12's reference-engine reversal needs 4 quarters below 5† divergences per quarter while CI exceeds 30%† of the DST farm; fonts' risk trigger is 2 quarters below 70%† of plan, or wave velocity below half plan | `reports/Izcal HTML and CSS parsing and style.md` §9.4; `reports/Izcal fonts and text.md` §10 | 2026-10-19 | High |

**Implications:** agentic coding can shorten the engineering inside a milestone, but it cannot shorten the observation itself. A threshold that needs four quarters of divergences, a yearly CA audit, or a 30-day crash-age window is bound to the calendar by construction. The restated C-6 must keep the measurement calendar as a floor, not treat it as an artefact of slow coding.

## 3. Conformance and WPT runs

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | The HTML parser has a fixed, named corpus | 100% of 6,827 html5lib tokenizer tests, by both the production tokenizer and the reference stepper; 100% of WPT tree-construction cases, 1,973 at 19 October 2026 | `reports/Izcal HTML and CSS parsing and style.md` ST-1 | 2026-10-19 | High |
| 3.2 | Style has per-directory floors, with the best engine as a second floor | html/syntax ≥ 99%†; html/dom ≥ 97.5%†; dynamic-markup-insertion ≥ 97%†; shadow-dom/declarative ≥ 99%†; domparsing and sanitizer-api ≥ the best engine (all engines below 70%) | `reports/Izcal HTML and CSS parsing and style.md` ST-2 | 2026-10-19 | High |
| 3.3 | CSS floors are also per directory | 97%† for css-syntax, css-cascade, css-nesting, cssom; 95%† for css-conditional, selectors, css-contain, css-properties-values-api; 87%† for css-shadow, css-variables, css-values | `reports/Izcal HTML and CSS parsing and style.md` ST-3 | 2026-10-19 | High |
| 3.4 | Layout floors cover four bands | 97%† for CSS2, display, box, position, sizing, align, flexbox, grid; 92%† for 13 inline/text/table/scroll directories; 90%† for css-page and mathml; 95%† for the observer and CSSOM View directories | `reports/Izcal layout.md` LY-1 | 2026-10-19 | High |
| 3.5 | The layout suite is large and much of it is legacy | About 25,000 WPT layout files, over a third of them CSS 2.1's legacy suite | `reports/Izcal layout.md` §3, §9.3 | 2026-10-19 | High |
| 3.6 | DOM has floors and a generated-bindings corpus | ≥ 97%† in dom/, shadow-dom/, custom-elements/, webidl/, html/semantics/, html/webappapis/, html/dom/; ≥ 95%† in html/browsers/, navigation-api/, workers/, service-workers/, webmessaging/, uievents/, selection/; 346 IDL files gate every generator change | `reports/Izcal DOM, bindings and event loop.md` DM-1, §6.3 | 2026-10-19 | High |
| 3.7 | JS conformance is measured three ways at once | test262 ≥ 99.5%† at Stage 1 exit, identical pass sets under full, baseline and jitless ceilings, on x86-64, AArch64 and riscv64 | `reports/Izcal JavaScript engine.md` JS-1 | 2026-10-19 | High |
| 3.8 | CI compute is stated as a multiple of production time, not as machine-years | About 2–4× the production style time† and 2–4× the production layout time† of the same runs; 10k DST seeds in T1, 1M in T2 | `reports/Izcal HTML and CSS parsing and style.md` §9.3; `reports/Izcal layout.md` §9.3 | 2026-10-19 | High |
| 3.9 | The CI tiers are wall-clock windows | T2 nightly, at most 12 h; T3 weekly, at most 72 h, including a 72 h soak | `reports/HGBA Stage 1 architecture design.md` §7.3 | 2026-10-19 | High |
| 3.10 | The WPT dashboard publishes no suite total or runtime | The page fetched on 21 October 2026 is a dashboard shell; it gives neither a test/subtest total nor a full-run duration | [wpt.fyi](https://wpt.fyi/) | 2026-10-21 | High |
| 3.11 | No public WPT machine-year figure was found | The corpus and the dashboard checked state no machine-year total; the specs give multiples and seed counts only | [wpt.fyi](https://wpt.fyi/); specs above | 2026-10-21 | Medium |

**Implications:** conformance is compute-bound and calendar-bound. Agentic coding can write more tests and triage failures faster, but each run must execute, and more agent-written tests raise the run count. The translation should carry a farm budget measured at ST-M1 and LY-M1, not a guessed machine-year figure. Note that the specs themselves defer this: the CI multiple is a planning placeholder until the named milestone measures it.

## 4. Fuzzing CPU-years

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | SVG states the only explicit weekly CPU-hour commitment found | ≥ 1,000† CPU-hours per week across grammar, structure and amplification fuzzers, on x86-64, arm64 and riscv64 builds; 0 open crash, panic, arithmetic-trap or hang bugs older than 14† days | `reports/Izcal SVG.md` SV-14 | 2026-10-19 | High |
| 4.2 | Security requires endpoint-complete sequence fuzzing | 100% of endpoints covered by generated sequence fuzzers at M6 and every later milestone; adversarial-peer mode for every sandboxed process kind, network service and GPU service; per-OS transport fuzzer nightly; planted-bug benchmark published quarterly† | `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-5 | 2026-10-21 | High |
| 4.3 | GPU, WebGL, WGSL and display-list fuzzers run nightly | 0 crash bugs in the GPU service older than 30† days | `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-10 | 2026-10-21 | High |
| 4.4 | Fuzzilli runs nightly through the renderer's REPRL mode | 0 open crash bugs older than 30† days | `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-11 | 2026-10-21 | High |
| 4.5 | The recording bundle parser is a fuzz target | 0 open crash bugs older than 30† days | `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-21 | 2026-10-21 | High |
| 4.6 | Network and WebRTC parsers have hard crash-age SLAs | Network: 0 panics reachable from network input older than 14† days; WebRTC: 0 crash bugs older than 30† days | `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-28, SEC-30 | 2026-10-21 | High |
| 4.7 | Every parser must be registered with OSS-Fuzz, with a sequence fuzzer per IPC endpoint | T0 `cargo check` of fuzz targets and corpus replay within 120 s; T2 nightly fuzzing under an RSS limit | `izcal-wt/coding-standard/reports/Izcal coding standard.md` CS-TE-11, CS-TE-8 | 2026-10-21 | High |
| 4.8 | Fuzz engineering is costed in the security track | 12 engineer-months for adversarial-peer simulation, the planted-bug benchmark and the per-OS transport fuzzer; 7 for GPU/display-list/shader fuzzing; 5 for network fuzzing; 106 engineer-months total, about 10 of them contracted audit work | `izcal-wt/security-scope/reports/Izcal security scope.md` §9.2 | 2026-10-21 | High |
| 4.9 | OSS-Fuzz's scale is public, its compute total is not | As of May 2025, over 13,000 vulnerabilities and 50,000 bugs across 1,000 projects; the README gives no executions-per-day or CPU-time figure | [google/oss-fuzz README](https://github.com/google/oss-fuzz) | 2026-10-21 | High |
| 4.10 | ClusterFuzz publishes the farm size and bug count, not CPU-hours | Can run on 30,000 VMs; 25,000+ bugs found in Google; 36,000+ across over 550 open-source projects; no test-case or CPU-usage figure | [ClusterFuzz](https://google.github.io/clusterfuzz/) | 2026-10-21 | High |
| 4.11 | An open-source audit fund publishes review hours | 13,000+ hours of security review across 100+ partner projects; 130+ severe bugs patched (about 130 review hours per project if averaged, which the site does not itself claim) | [OSTIF](https://ostif.org/) | 2026-10-21 | High |

**Implications:** fuzzing is the clearest incompressible compute class. Campaigns are nightly or weekly, and the SLAs are 14 or 30 days of wall-clock, so a faster coding agent does not shorten the assurance window. Worse for the naive translation, agentic coding tends to increase the number of fuzz targets, which raises compute demand. The corpus has no aggregate CPU-year number beyond SV-14, so the translation should budget the farm as a measured quantity at DM-M3's "fuzz campaign at full scale" and at SV-14's weekly floor, and should state the absence of a total rather than invent one.

## 5. The AT lab and human accessibility testing

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | The lab is a fixed physical estate | At least 8 Windows machines (JAWS and NVDA, current and previous versions; two run the AX-D25 characterisation suite), 3 Macs (VoiceOver, Voice Control, Switch Control), 2 Linux machines, 1 riscv64 Linux board (Orca), and an Android set from AX-M3; JAWS and Dragon licences | `reports/Izcal accessibility.md` §9.3 | 2026-10-20 | High |
| 5.2 | The owner chose black-box AT characterisation, at a measured cost | AX-D25 option (b): no third-party source ever; behaviour learned from documentation, vendor liaison and black-box testing; +1–2† engineers and two more Windows machines | `reports/Izcal accessibility.md` AX-D25 | 2026-10-20 | High |
| 5.3 | The team must include people who actually use the AT | At least two engineers who use a screen reader daily, in a 12–15 person start at AX-M0 | `reports/Izcal accessibility.md` §9.3 | 2026-10-20 | High |
| 5.4 | User studies are contracted each release and sized by power analysis | AX-40: completion ratio ≥ 0.9† and time ratio ≤ 1.2† against Chrome with the same AT and OS; AX-41 shell tasks; AX-42 low vision and motion; each release | `reports/Izcal accessibility.md` AX-40 to AX-42, §9.3 | 2026-10-20 | High |
| 5.5 | AT compatibility is measured on the lab, not asserted | AX-M0 brings the security scope's M5 broker-mitigation measurement forward for NVDA, JAWS, Dragon, ZoomText and Magnifier | `reports/Izcal accessibility.md` §9.2; `izcal-wt/security-scope/reports/Izcal security scope.md` §9.1 | 2026-10-20/21 | High |
| 5.6 | The AT matrix is real and fragmented | Screen Reader User Survey #10: 1,539 valid responses, December 2023–January 2024; JAWS 40.5%, NVDA 37.7%, VoiceOver 9.7%, Orca 2.4%, SuperNova 3.7% | [WebAIM Screen Reader User Survey #10](https://webaim.org/projects/screenreadersurvey10/) | 2024-01 (survey) | High |

**Implications:** each AT/OS/version combination is a physical machine, a licence and a human session. Agents can drive `hg-ax-probe`, generate ablation pages and replay recordings, but they cannot be the screen-reader user the study requires, and recruiting daily users of JAWS, NVDA and VoiceOver is wall-clock. This class is human-bound. The one genuine agentic leverage is harness quality, which the spec already commits to through AX-D25's characterisation suite; that leverage reduces engineering, not the human confirmation.

## 6. Measurement rigs, hardware and exclusive timing

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Footprint's first milestone buys devices and a riscv64 reference machine | FP-M0: empty-process probe on all three OSes, DC-8 and DC-4 lab devices, a riscv64 reference device (RVA23, SpacemiT K3 class) and riscv64 CI runners | `reports/Izcal resource footprint.md` §9.2 | 2026-10-19 | High |
| 6.2 | Small-memory devices are bought, not emulated | FP-D12: add an 8 GB Windows laptop and a 4 GB Linux laptop at FP-M0; emulation is rejected because it misses real pressure behaviour | `reports/Izcal resource footprint.md` FP-D12 | 2026-10-19 | High |
| 6.3 | The first full corpus measurement is a milestone of its own | FP-M3 at M4: FP-5/6/13/14/35, † budgets confirmed or changed, energy baseline and noise bands, frozen-tab footprint per OS, swap encryption, decode memory | `reports/Izcal resource footprint.md` §9.2 | 2026-10-19 | High |
| 6.4 | Graphics keeps a permanent device-and-driver lab | Device lab and driver response 4–6† engineers, continuing after parity; the GPU matrix covers Intel and AMD on macOS and Windows 10 while those qualify | `reports/Izcal graphics and rendering.md` §9.3; GR-D21 | 2026-10-19 | High |
| 6.5 | A fifth of Android is below the GPU floor | About 21% of Android devices lack Vulkan 1.1 or the 2022 profile (dashboard snapshot of 24 November 2025), so the CPU path is a real target | `reports/Izcal graphics and rendering.md` GR-D27, §6.17 | 2026-10-19 | High |
| 6.6 | riscv64 is a procurement item with a named board and a board pool | SpacemiT K3: 8 X100 cores at 2.4 GHz, RVA23 with RVV 1.0, sold as K3 Pico-ITX and Milk-V Jupiter 2; T1 pool 16–24† boards (§6.18), 20–27† boards and +5–7† engineers (JS-D22, §9.3); performance lab 3† boards; Izcal has not measured the board | `reports/Izcal JavaScript engine.md` §6.18, JS-D22, §9.3 | 2026-10-19 | High |
| 6.7 | riscv64 has its own coherence and determinism gates | JS-28: 0 executions of stale instructions or illegal-instruction faults in 10⁶† installs on 8 threads with kernel migration; per-install coherence cost published; JS-D12 keeps cross-ISA identity only if its cost is ≤ 0.5%† on JetStream 3 | `reports/Izcal JavaScript engine.md` JS-28, JS-D12 | 2026-10-19 | High |
| 6.8 | The lab runs to fixed windows | T2 nightly at most 12 h; T3 weekly at most 72 h, including a 72 h soak | `reports/HGBA Stage 1 architecture design.md` §7.3 | 2026-10-19 | High |
| 6.9 | The corpus names no exclusive run lock | No Izcal spec uses a game-run-lock or booking mechanism; timing-sensitive runs are specified by their lab and milestone, and the project's rig-booking practice is the external operational analogue, not a spec commitment | Search of the specs; `reports/HGBA Stage 1 architecture design.md` §7.3 | 2026-10-19 | High |
| 6.10 | A vendor-gate date in the spec is not on the vendor's lifecycle page | GR-D21 says Windows 10 consumer updates run to 12 October 2027; Microsoft's lifecycle page states end of support 14 October 2025 and lists no ESU end date. The 2027 date is cited to Microsoft's ESU page and was not independently verified here | `reports/Izcal graphics and rendering.md` GR-D21, §11.2; [Windows 10 Home and Pro lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-10-home-and-pro) | 2026-10-19/21 | Medium |

**Implications:** this class is device-bound. Emulation is explicitly rejected where pressure or GPU behaviour matters, so the matrix of OSes, GPUs, memory classes and ISAs is capital and wall-clock. Agentic coding cannot manufacture a K3 board or make an Intel Mac stay supported. Procurement lead times can move a milestone (JS-D22 already says that if RVA23 hardware cannot be bought in the pool's numbers by JS-M1, T1 moves to emulation with nightly board runs), and a vendor ending security updates can drop a platform at the next release. These belong on the schedule's critical path, not in a headcount multiplier.

## 7. Licence, patent, audit and root-program work

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | Nine shipped items fail the MIT-only rule | Two large ones: the TLS crypto provider (aws-lc-rs/AWS-LC) and the font set (SIL OFL 1.1); the rest include rustls's ISC and BSD dependencies, the Mozilla root store via CCADB, the HSTS list, EasyList and non-MIT generic crates | `research_notes/Phase 2 reconciliation/licence_audit.md` Summary, class 2 | 2026-10-19 | High |
| 7.2 | The cost of building everything is dominated by fonts | About 9–17 engineer-years† once, plus 8–15 people† ongoing, plus a pan-Unicode font set of 150–400 designer-years† over 5–10 years | `research_notes/Phase 2 reconciliation/licence_audit.md` Summary | 2026-10-19 | High |
| 7.3 | Some non-MIT data cannot be replaced without diverging from the standard | Six standards-data sets, including the Public Suffix List (MPL-2.0), WHATWG data (CC BY 4.0) and the CT log list (licence not stated); the owner allowed Unicode, standards data and test-only inputs by decision | `research_notes/Phase 2 reconciliation/licence_audit.md` classes 3–4, OD-L1 to OD-L9 | 2026-10-19 | High |
| 7.4 | Codec patents are a product decision, not only a legal review | Every format and encoder has a patent row; AVC: $0 for the first 100,000 units a year, $0.20 to 5 million, $0.10 above, capped at $9.75 M a year; AAC: $15,000 plus $0.98 per unit for the first 500,000; AOM Patent License 1.0 is royalty-free with termination clauses, contested by Sisvel and Access Advance | `reports/Izcal images and media.md` §6.2, MD-D3 | 2026-10-19 | High |
| 7.5 | The root program has a credibility gate before the first public release | 100% of included roots with a review record; the distrust process run once; ≥ 99.9%† of corpus HTTPS origins and a CT sample chain to an included root; ongoing distrust review within 7† days and decision within 30† days; every CA audit renewed yearly | `reports/Izcal networking.md` NW-53 | 2026-10-19 | High |
| 7.6 | The root program is permanent staff, not a project | About 50–70 CA owners† reviewed at the start and yearly; 4–6 people† ongoing; 1–2 person-years† for the initial review; CA/Browser Forum and CCADB participation | `reports/Izcal networking.md` §6.5.2, §9.3 | 2026-10-19 | High |
| 7.7 | External audits are release blockers in four specs | Networking NW-52 before the first public release and at least yearly†; media MD-25; security SEC-24; ML service ML-21 | `reports/Izcal networking.md` NW-52; `reports/Izcal images and media.md` MD-25; `izcal-wt/security-scope/reports/Izcal security scope.md` SEC-24; `reports/Izcal ML service.md` ML-21 | 2026-10-19 | High |
| 7.8 | Security's own audit work is small but contracted, not staffed | About 10 of 106 engineer-months are contracted audit work; 96 are in-house, about 2.7–4.0 engineers over 24–36 months | `izcal-wt/security-scope/reports/Izcal security scope.md` §9.2 | 2026-10-21 | High |
| 7.9 | AI legal tools are not reliable enough to be the reviewer of record | Legal hallucinations on specific verifiable questions about random federal court cases: 58% with ChatGPT 4 to 88% with Llama 2; legal research tools from LexisNexis and Thomson Reuters hallucinated 17–33% of the time | [Dahl et al., arXiv 2401.01301](https://arxiv.org/abs/2401.01301); [Magesh et al., arXiv 2405.20362](https://arxiv.org/abs/2405.20362) | 2024-01/2024-05 | High |
| 7.10 | The Via LA page fetched does not expose the rates | The AVC/H.264 licensing page returned navigation text only, with no per-unit rates or caps; the rates above are from the spec's patent table, which cites Via LA and its own sources | [Via LA AVC/H.264](https://www.viala.com/licensing-programs/avc-h-264/) | 2026-10-21 | High |

**Implications:** legal and audit work is accountability-bound. Agents can draft a policy, check a licence file, assemble an audit scope and trace a patent table, and the MIT-only rule converts some risk into engineering that agents can compress. They cannot sign an audit opinion, accept a patent licence, certify a CA's audit, or take liability for a legal opinion. The hallucination evidence is a direct warning against substituting a model for the reviewer of record: a 17–33% error rate in specialist legal tools is not a basis for a release gate. This class sets release dates, not staffing alone.

## 8. Type design

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | The type-design programme is measured in designer-years, not engineer-years | 150–400 designer-years† over 5–10 years† | `reports/Izcal fonts and text.md` §9.3; `research_notes/Phase 2 reconciliation/licence_audit.md` | 2026-10-19 | High |
| 8.2 | The team is capped at about 45 and the calendar follows from that | About 45† type staff; ramp about 10 designer-years in year one, 22 in year two, 38 in year three, then about 45 a year; about 160† by year five and 385† by year ten; the low estimate is reached at about 4.8 years, the high at about 10.3 | `reports/Izcal fonts and text.md` §9.3 | 2026-10-19 | High |
| 8.3 | The spec's stated reason is privacy, not schedule | Every year a script stays on the interim OS-font tier, its readers' text differs by OS and the OS fonts are a fingerprint (TX-22, TX-D25), so time to the last face is a privacy cost borne by users | `reports/Izcal fonts and text.md` §9.3 | 2026-10-19 | High |
| 8.4 | The work is split by script and skill, and the hard part is Han | Aliases 2–4† designer-years; math 2–3†; emoji 0.5–1 engineer-year† plus artwork; last resort 0.25†; Han is tens of thousands of glyphs per regional form and the largest share of the designer-years | `reports/Izcal fonts and text.md` §9.2, §9.3 | 2026-10-19 | High |
| 8.5 | The spec names scarcity and sets a measured risk trigger | Designers fluent in Han, Indic, Arabic and South-East Asian type are scarce; the track is smaller than most foundries; below 70%† of plan for two quarters, or wave velocity below half plan, is the risk row | `reports/Izcal fonts and text.md` §9.3, §10 | 2026-10-19 | High |
| 8.6 | A real pan-script programme took most of a decade and many designers | Noto: creation span 2012–2020; about 1,000 languages and 162 writing systems as of November 2024; 195 fonts as of December 2020; Google commissioned it and multiple designers and foundries worked on it | [Noto fonts (Wikipedia)](https://en.wikipedia.org/wiki/Noto_fonts) | 2024-11 (page) | Low |
| 8.7 | No verified published evidence was found that AI compresses type design | The pages checked do not measure AI-assisted type design; the corpus's claim rests on the scale of the work and the scarcity of designers, not on a tool study | — | 2026-10-21 | Low |

**Implications:** type design is the largest non-engineering block and the clearest craft-bound class. The 150–400 designer-years are set by the work, not by team size; a larger team spends them sooner but cannot exceed the supply of designers who can draw a credible Han, Indic or Arabic face. Agentic coding does not touch this work, and the tooling the spec already counts (`hg-fontc`, generated tables) removes engineering around the design, not the design. The 5–10 year calendar is therefore a privacy cost, not a coding-speed artefact. This is the class most likely to set the programme's full-strength completion date even if every engineering class compresses.

## 9. Independent review, spec anchors and owner decision bandwidth

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | Every spec has an independent fact-check and a design critique, and the volume is large | Editing: 201 claims and 41 findings; media: 179 claims, 517 links and 48 findings; graphics: 140 claims, 611 links and 41 findings; style: 92 claims and 35 findings; networking: 94 claims and 29 findings; SVG: 146 claims and 36 findings; PDF: 132 claims and 36 findings; accessibility: 140 claims and 33 findings | Revision lines of the cited specs; review files under `research_notes/<spec>/factcheck.md` and `critique.md` | 2026-10-19 | High |
| 9.2 | The method makes review surface proportional to code, mechanically | G9: every crate implementing web-platform behaviour carries spec anchors on its algorithm functions; ANC-1: 100% of anchors resolve, coverage is published per specification per release and must not fall; `hg-anchor` enforces it in T0 | `reports/HGBA Stage 1 architecture design.md` §2.2 (G9); `reports/HGBA 2 browser architecture spec.md` M-1, ANC-1 | 2026-10-19 | High |
| 9.3 | Owner decisions arrive in batches and are accepted in full | Batch 2: 51 decisions (SVG 16, animation 13, editing 22); batch 3: 38 decisions (storage 12, WebCrypto 10, extensions 16) | `research_notes/Phase 3 amendments/batch2_decisions.md` §1; `batch3_decisions.md` §1 | 2026-10-20/21 | High |
| 9.4 | Each spec also carries its own decision table | Roughly 10–32 decisions per spec, each with options, a recommendation, a cost and a reversal trigger | The `§9.4` or `§9.3` decision sections of the specs | 2026-10-19 | High |
| 9.5 | The running total is restated only by an owner decision | C-6 addendum: about 531–689 people† from batch 2; batch 3 adds storage 7–10†, WebCrypto 3.5–5.5† and extensions 9–14† plus 2–4† non-engineering, giving about 551–719 engineers† and 553–723 people† | `research_notes/Phase 3 amendments/batch3_decisions.md` §5 | 2026-10-21 | High |
| 9.6 | No verified study measures AI's effect on review throughput or decision quality | The pages checked do not measure it; the nearest published evidence is the legal-hallucination work, which shows a verification tax rather than a replacement | — | 2026-10-21 | Low |

**Implications:** this is the verification-tax class. G9 and ANC-1 mean that more agent-written code creates more anchors and more review surface, not less, unless the review is also automated. The method's independence rule (a reviewer who did not write the spec) and the owner's personal acceptance of 89 decisions in two batches limit how much can be delegated. Agents can prepare options, evidence and draft text; they cannot accept the risk or be the independent reviewer of record. Owner bandwidth is serial and is the most likely non-technical constraint on how fast the corpus can absorb agent output.

## 10. Vendor gates and market admission

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | The platform floor follows vendor security support, not a guessed date | GR-D21: support an OS version while its vendor ships consumer security updates at the release decision; Windows 10 consumer updates to 12 October 2027; Intel Macs stop at macOS 26 | `reports/Izcal graphics and rendering.md` GR-D21, §11.2 | 2026-10-19 | Medium |
| 10.2 | The vendor page checked confirms the end of support, not the 2027 extension | Windows 10 Home and Pro: end of support 14 October 2025; the page lists no ESU end date, so the 12 October 2027 figure remains a spec claim from a separate Microsoft ESU page | [Windows 10 Home and Pro lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-10-home-and-pro) | 2026-10-21 | High |
| 10.3 | A CSS feature is admitted only when the standards and market process moves | ST-D18 and LY-D10: implement an editor's-draft feature when it has a WG resolution and WPT tests and either ships in two engines or is an Interop focus area | `reports/Izcal HTML and CSS parsing and style.md` ST-D18; `reports/Izcal layout.md` LY-D10 | 2026-10-19 | High |
| 10.4 | The root program's calendar is external | CA/Browser Forum and CCADB participation; every included CA's audit renewed yearly; other programs' distrust reviewed within 7† days and decided within 30† days | `reports/Izcal networking.md` §6.5.2, NW-53 | 2026-10-19 | High |
| 10.5 | Audit cadence is yearly at minimum | Re-audit after a key-schedule, record-layer or primitive change, and at least yearly† | `reports/Izcal networking.md` NW-52 | 2026-10-19 | High |

**Implications:** these gates are set by other organisations' release calendars. No amount of agentic effort makes a vendor patch an OS, a working group resolve a feature, a CA renew an audit, or two engines ship a feature. They set minimum calendar time for a release and should be on the critical path, separate from the staffing ledger.

## 11. Synthesis: a two-ledger translation

The evidence supports one structural conclusion: the corpus's engineer-year totals and its calendar are not the same quantity, and agentic coding acts on the first much more than the second.

| # | Class | Binding constraint | Agentic effect | Corpus number | Translation treatment |
|---|---|---|---|---|---|
| 11.1 | Implementation engineering | Code production | Large compression possible (other strands) | About 2,000–3,400+ engineer-years† | Compressible ledger |
| 11.2 | Conformance and CI runs | Farm compute and run windows | Builds harnesses faster; runs still execute | 2–4× production time†; 10k/1M seeds | Budget from measurement at ST-M1/LY-M1 |
| 11.3 | Fuzzing | CPU-hours and 14/30-day SLAs | More targets, not fewer hours | ≥ 1,000† CPU-h/week (SVG); nightly campaigns | Fixed farm commitment; measure at DM-M3 |
| 11.4 | Device and AT labs | Physical machines, licences, quiet time | Harness automation only | 8+ Windows, 3 Macs, 2 Linux, riscv64, Android; K3 pool | Capital and wall-clock, not headcount |
| 11.5 | Type design | Scarce human craft | No verified effect | 150–400 designer-years†; ~45† staff; 5–10 years† | Separate designer-year ledger |
| 11.6 | Legal, patent and audit | Accountable signature | Drafting and checking only | 9–17 eng-yr† + 8–15 people†; AVC cap $9.75 M/yr | Release gate; external cost |
| 11.7 | Root program | Permanent operations and external audits | Tooling only | 4–6 people† ongoing; 50–70 CA owners†/yr | Permanent staff, post-parity |
| 11.8 | Owner decisions | One person's serial attention | Prepares options | 51 + 38 decisions | Serial calendar constraint |
| 11.9 | Vendor and standards gates | Other organisations' calendars | None | Windows 10 ESU; two-engine rule; yearly audits | Critical-path dates |

**Floor arithmetic.** Three numbers survive any plausible agentic compression:

- **Stage 1's exit is dated at 48–60 months†** on the corpus's own critical path, because layout and graphics set it and their milestones are measurement and hardware milestones, not coding milestones (`reports/Izcal layout.md` §9.3; `reconciliation_2.md` §4).
- **The type-design programme is dated at about 5 years† on the low estimate and about 10 on the high**, because 150–400 designer-years are reached at about 4.8 or 10.3 years with the ~45† staff the owner chose (`reports/Izcal fonts and text.md` §9.3).
- **After parity, 38–54† people remain** in the root program (8–13†), type maintenance (6–10†), codec maintenance (20–25†) and the graphics device lab (4–6†), because these are operations, audits and craft, not a project that ends (`reconciliation_2.md` §4).

**Proposed owner decision (C-6R, 21 October 2026).** Restate C-6 as two ledgers rather than one multiplied total. **Ledger A** is compressible engineering: the implementation engineer-years, translated per activity class by the other strands. **Ledger B** is the incompressible remainder, carried as its own numbers and not multiplied by an agentic factor: type design 150–400 designer-years† at about 45† staff over 5–10 years†; the Stage 1 calendar floor of 48–60 months†; conformance and fuzz farm compute, measured at ST-M1, LY-M1, SV-14 and DM-M3; the AT lab and its contracted studies; external audits and legal review; the root program's 4–6† ongoing staff; owner decision throughput; and the vendor and standards gates. **Reversal trigger:** at the first measured milestone (ST-M1 or LY-M1), if measured compute and review effort fall below Ledger B's floors for two consecutive quarters, restate the floor; if type-design wave velocity falls below 70%† of plan for two quarters, move the date, not the design.

**Implications:** this keeps the corpus falsifiable. It says what agentic coding is expected to compress (implementation), what it is not (the remainder), and where the claim will be tested (the named measurement milestones). It also prevents the common error of applying a headline speed-up to a total that mixes engineer-years, designer-years, CPU-hours, audit hours and calendar gates.

## 12. What is not known

- **No aggregate CI machine-year figure** exists in the corpus. The specs give a 2–4× production-time multiplier† and seed counts. A machine-year total would have to be built from measured run times at ST-M1 and LY-M1.
- **No aggregate fuzz CPU-hour figure** exists. SV-14's ≥ 1,000† CPU-hours per week is the only explicit weekly commitment found; the security scope and coding standard specify targets, coverage and SLAs, not farm size.
- **No public audit price** was found. OSTIF's 13,000+ hours across 100+ projects gives a scale, not a rate; Izcal's own contracted audit is about 10 of 106 security engineer-months, which is effort, not cost.
- **No verified evidence that AI compresses type design, AT testing, independent review or owner decisions.** The only directly relevant published evidence found is negative for unsupervised legal work: 17–33% hallucination in specialist legal tools, and 58–88% on federal-case questions.
- **Several primary pages did not state the figures sought.** wpt.fyi publishes no suite total or runtime; CCADB publishes no root or CA count on the pages checked; Via LA's AVC page returned no rates. The rows say so rather than filling the gaps.
- **The Windows 10 consumer ESU date of 12 October 2027** is a spec claim cited to Microsoft's ESU page; the Microsoft lifecycle page checked states the 14 October 2025 end of support and no ESU date. It is marked Medium here, not High.
- **The web-search budget was exhausted before this strand could search.** Every external row above comes from a direct fetch of a known primary URL on 21 October 2026. A later pass with search available should look specifically for published fuzz-farm CPU totals, audit prices, and any measured effect of AI tools on type design or accessibility testing.
