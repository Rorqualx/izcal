# Izcal extensions: frame

Written 20 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

A browser without an extension platform asks its users to accept the features it ships and nothing else. A browser with the wrong extension platform hands third-party code the user's browsing history, every page's content, and the browser's own privileges, and inherits a decade of documented failures: sold extensions that turned malicious, blockers disguising data exfiltration, review processes that missed malware, and a manifest-version migration (MV2 to MV3) that broke working software and did not stop the abuse it targeted. No accepted Izcal spec owns extensions. The Phase 3 amendment ledger records the gap as **G-20** ("Extensions (Stage 2 surface, JS-D10 component host)"), named by CB-D15, CB 6.15, NW 6.19 and Stage 1 design D-S3. Stage 1 design D-S3 (signed off 14 October 2026) says Stage 1 ships **no extensions** at all — content blocking is built in — and extensions arrive in Stage 2. This spec designs the Stage 2 platform and is written now so that Stage 1 builds the seams (worlds, the component host, the IIP interception path, the prompt layer) without rework.

Scope of this spec, as assigned:
- **The compatibility surface**: which WebExtensions API surface Izcal implements and which it deliberately does not, as a published, falsifiable list, decided from vendor and MDN prose only.
- **Manifest version**: the MV3 model and whether any v2 surface is supported, decided from first principles and the deprecation record.
- **Process model**: extension background contexts on the JS engine's component host (JS-D10); content scripts in isolated worlds per the DOM spec's realm design (DM-D16).
- **Permissions**: install-time vs runtime prompts, host permissions, the shell's prompt surface under GR-D28's OS prompt layer.
- **Extension storage**: pointed at the storage spec being written in parallel (ledger G-19).
- **Network APIs**: `webRequest` vs `declarativeNetRequest`, coordinated with the content-blocking spec's DNR design (CB-D15).
- **Native messaging**: broker-mediated, capability-scoped.
- **Signing, distribution and update policy**: Izcal's own signing and distribution rules, by the root-program analogy (NW 6.5.2); sandbox and capability rules rather than a licence requirement.
- **CSP for extension pages.**
- **Privacy**: extension fingerprinting, install telemetry.
- **Security scope rows** (proposed as amendments; this spec does not edit the security scope).
- **Determinism and recording** for extension actions (B1, JS-D15 pattern).
- **Observability**: the about:debugging equivalent, DevTools extension inspection, IIP.
- **Staffing, timeline and testing targets (EX-M\*)** as Stage 2 work off the Stage 1 critical path, closing G-20 explicitly.

Not in scope: the extension host engine itself (JS-D10, JS spec), the WASI embedding (JS spec), the isolated-world wrapper machinery (DM-D16, DOM spec), the DNR rule syntax and compiler (CB), the IIP transport and audit schema (DevTools spec, NW 6.19), the storage service and quota accounting (storage spec, G-19), permission-prompt rendering (shell UI design, GR-D28), socket brokers and LNA (networking spec), the agent principal's own tool surface (HGBA-3 B9; agents are not extensions, D12).

## Who has the problem

- **Power users** whose workflow depends on an extension (password managers, ad blockers, translation, developer tools): Stage 1 will ship without them, and D-S3 accepts that loss knowingly; Stage 2 must close it without buying Chrome's platform's documented failures.
- **Extension authors**: porting cost is the single largest determinant whether the platform has any ecosystem at all; Firefox and Safari both chose near-total Chrome API compatibility for exactly this reason, and Izcal's model differs enough (Wasm components, no blocking interception) that the compatibility question must be answered explicitly, per API.
- **Privacy- and security-conscious users**: extensions are the largest third-party code surface in a browser and a documented fingerprinting channel; they must be auditable and their capability growth must re-prompt.
- **Enterprises and administrators**: force-install, block and policy configuration are load-bearing in the installed base.
- **Izcal's own engineers**: Stage 1 must leave the seams; the Stage 2 team must be able to build without touching the renderer-critical path.

## What Izcal's specs already decide (the binding set)

- **HGBA-2**: P1–P6; M-1 (no engine libraries, MIT only, no third-party source as evidence, 2.3.3); I-1 isolation key (site, container); **W-1** row (extensions as internal Wasm components; no extension code in renderer processes — host text superseded by JS-D10 per ledger C-17/G-5); 4.2 note that JS-in-Wasm is an isolation tool "used only to isolate untrusted script, such as extension sandboxes"; **D12** (agent is a principal, not a privileged extension).
- **HGBA-3**: B1 (deterministic substrate, recorded inputs), B2 (typed capability IPC), B9 (agent principal), B10 (AI off by default).
- **Stage 1 design**: **D-S3** (none in Stage 1; Stage 2 via the Wasm host); D-S2 (the shell is sandboxed web content; scripted customization goes through W-1 components with declared capabilities); 8.1–8.3 (milestones, tracks); Section 6 rows; W-1 row in the 5.x process table; Section 9 Stage 2 backlog row "extensions in Wasmtime" (host text superseded by C-17).
- **JavaScript engine (accepted, JS-D1–D21)**: **JS-D10** — internal components run on Izcal's own Wasm engine in a **component host process** with a WASI embedding, tier-P build; the component host is one of the DOM-less embedders that run the JS engine **in jitless form**; JS-D15 (timing-dependent decisions are recorded B1 inputs); JS-D20 (reduced-isolation mode boundary); the per-agent heap and one-cage-per-process rules.
- **DOM, bindings and event loop (accepted)**: **DM-D16** — content scripts (Stage 2), DevTools evaluation and agents see the DOM through **their own realms** with per-world side tables and **Xray-style views**; mutation provenance carries the world (page, extension, DevTools, agent); service-worker contexts live in renderers with a 30 s† idle-termination default for *page* service workers.
- **Content blocking (accepted, CB-D1–D24)**: **CB-D15** — Stage 2 extension tiers: (1) list providers with **zero read capability**, lists as data; (2) typed interventions as data; (3) **scoped, audited interception through IIP** with re-prompting on capability growth; **no blocking request-interception API**; DNR-compatible API as data; extension lists count against the container's compiled-size budget; CB-D21 (matching is a pure function; recorded decisions).
- **Networking (accepted, NW-D1–D26)**: NW 6.19 — agent and automation interception is scoped to an origin set and container, typed by intent, badged, audited, with bodies/cookies/credentials as sensitive classes needing a grant; "Extensions (Stage 2) get request interception through the same scoped, audited path"; NW 6.22 marks extensions as Stage 2 (host text says "Wasmtime"; superseded by C-17); NW 6.5.2 the root program as the governance analogy; "who received what" ledger.
- **DevTools spec (accepted)**: **no WebExtension `devtools.*` API**; author views come through the IIP `views` domain or W-1 components; every inspection session is a scoped, audited principal over IIP; the frontend respects site isolation.
- **Editing (accepted batch 2, ED-D1–D21)**: **ED-D21** — agents, AT and extensions edit only through typed edit intents producing trusted `beforeinput`/`input`, never minting user activation; **ED-D11** — clipboard provenance and a warning when a page plants a shell command; broker-owned clipboard with per-read paste confirmation (ED-D10).
- **Graphics (accepted, GR-D1–D28)**: **GR-D28** — security prompts are presented in their own OS layer the shared compositor cannot draw over, rastered by the shell's own content GPU process, input routing through broker-clamped renderer regions.
- **SVG (accepted batch 2, SV-D1)**: scriptless static host over the shared node store — no bearing on extension pages, which run as ordinary documents.
- **Shell UI design**: the shell is a page; the prompt surface; the settings and site-panel model.
- **Resource footprint (accepted)**: FP-1 broker budget; FP-D16 regenerable-cache class (compiled filter files); FP-D9 (no telemetry by default; opt-in aggregates only under a privacy review); the Great Suspender lesson (footprint control is first-party work).
- **Storage (G-19)**: storage service, quota, IndexedDB, Cache API — a spec being written in parallel; extension storage points there and sets no storage-design text here.

## What would make this a success

- A published, per-API compatibility list that an extension author can read and act on, and that a test suite can falsify (each supported API has conformance tests; each refused API has a recorded reason).
- The three CB-D15 tiers are the security spine: an extension that only blocks can never read; one that must read is scoped, badged, audited and re-prompted on growth.
- No extension code in a renderer process that also renders web content for a different (site, container); content scripts confined to isolated worlds with Xray views; background contexts on the component host.
- Install, grant, upgrade-with-capability-growth and removal are all explicit, recorded, and presented in the OS prompt layer (GR-D28); nothing updates silently into more privilege.
- Determinism: an extension's decisions are recorded B1 inputs, so a recording replays with the same extension behaviour.
- Observability at least equal to `about:debugging`, through IIP, for every extension context.
- Staffing and milestones as Stage 2 work off the Stage 1 critical path, with the Stage 1 seams named and costed, closing G-20.
