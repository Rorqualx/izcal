# Izcal extensions

*The extensions platform spec for Izcal: the WebExtensions compatibility surface Izcal implements and refuses, the manifest model, the process model on the component host (JS-D10), permissions, storage, network APIs, native messaging, signing and distribution, CSP, privacy, determinism, observability, and the Stage 2 staffing that closes ledger gap G-20. Research notes: `research_notes/Izcal extensions/`. Relates to: HGBA-2 (W-1, D12), HGBA-3 (B1, B2, B9), Stage 1 design (D-S3, W-1, Section 6), JavaScript engine (JS-D10, JS-D15, JS-D17), DOM and bindings (DM-D16), content blocking (CB-D15, CB-D21), networking (NW 6.19), DevTools (IIP, DT-D10), editing (ED-D10, ED-D11, ED-D21), graphics (GR-D28), resource footprint (FP-D9, FP-D16).*

Date: 21 October 2026. Status: draft 0.4, phase 8 applied.

*Revision 0.4 (21 October 2026): batch-4 amendments applied, from the accepted shell spec 0.3 (§9.5: EX-D5, EX-D15) and phishing protection spec 0.3 (§9.5: EX-D7, 6.7), following the owner decision of 21 October 2026. Install and growth grants are requested by the extension, shown in the prompt layer and approved only by the broker, with the shell showing a persistent pending marker (6.5, EX-D5). All Ring A UI slots, including the action button, popup host, options page, badge, per-site grant row, new-tab/homepage override prompts, recommendation tier badge and command editor, are **Stage 2**, because Stage 1 ships no extensions (6.9, EX-D15); A-EX-12 stays open until those Stage 2 surfaces exist in the shell (shell 6.15). An extension-initiated navigation is checked against the phishing bundle before the request; no extension API exposes, suppresses or alters a verdict; an extension cannot add an allowlist entry or a removal, and extension-supplied lists cannot carry a phishing allow entry; `ext://` pages are labelled "Extension page" and are governed by EX-D10, while remote content they embed is checked as a subframe (6.7, 6.9).*

*Revision 0.3 (21 October 2026): **owner decision.** The owner accepted every recommendation in Section 9.4 (EX-D1 to EX-D16) on 21 October 2026; the decision record is `research_notes/Phase 3 amendments/batch3_decisions.md`. No decision text changed. Section 9.5's amendments are applied in the documents they amend, each with its own dated revision line: HGBA-2's W-1, X-3 and I-1, the Stage 1 design's D-S3, Section 6, Section 9 and 8.3, the JavaScript engine spec's 6.0, 6.15 and G14, the networking spec's 6.22, the content blocking spec's 6.15, the DevTools spec's IIP domain and observability row, the storage spec's 6.x extension-storage mapping and split-mode keying, the HTML and CSS spec's `hg-url` `ext://` registration, the security scope's process kind and CON-R cites, the resource footprint spec's extension lines and N-extension scenario, the editing spec's 6.16, and the accessibility spec's extension UI rows (A-EX-1 to A-EX-15). **The shell UI design row is not applied in that document**: the shell surface is being rewritten as `reports/Izcal shell.md` (batch 4, started 21 October 2026), and the Extensions page and slots land with that spec, so the shell owner takes that row. The WECG row is an upstream filing, not an Izcal document edit. Section 9.5's status is updated accordingly.*

*Revision 0.2 (20 October 2026): phase 8 applied by a fact-checker and a design critic who did not write the spec (`factcheck.md` and `critique.md` in the research notes folder). The fact-check checked 105 claims (84 confirmed, 12 partly correct, 5 wrong, 4 not re-checked) and listed 7 must-fix and 9 recommended items; all are applied. The design critique raised 22 findings (5 High, 9 Medium, 8 Low) with a 16-row reconciliation list; all are applied, several in a form different from the one proposed (Appendix A). Neither reviewer found an M-1 violation. What changed: `declarativeNetRequest` is ringed **per method** (match feedback, dynamic rules, `modifyHeaders` and extension-resource redirects refused under CB's rules), extension tier-3 grants are **observe-only** (`mock`/`block`/`modify` stay reserved to the user's pipeline and to agent and automation principals), the ring table is **closed and default-deny** with every inventoried namespace named and a fourth **staged** state, declared content scripts create an install-time "run on the sites you visit" grant class so the flagship classes are not inert under the on-click default, user scripts run in the **USER_SCRIPT world**, `split` mode is defined concretely (same ID, per-container agent, storage and grants) with the in-cage trust assumption stated, the JS spec gains a 9.5 amendment row (G6's allow-list, the embedder list, G14's closure), package and manifest parsing move to a **strictest-tier utility process** (`hg-ext-package-main`), the review operation is budgeted (**2–4† non-engineers** beside the 9–14† engineers), the footprint row gains reservation, offscreen-renderer and N-extension lines, and the identity precedent is corrected to Chrome's documented `key`-field derivation.*

*Revision 0.1 (20 October 2026): first draft, written from the seven research notes by the synthesis author.*

**The central idea: the platform is a capability system wearing the documented WebExtensions API as its published, falsifiable face.** Compatibility decides what the face looks like (three rings: implemented with published conformance suites, accepted-as-data, refused with recorded reasons); capabilities decide what the face can do. The decisions that matter most: (1) **MV3-only manifests** (EX-D2) — the only capability MV2 has that MV3 lacks is the one CB-D15 refuses anyway; (2) **background contexts run on the component host** (JS-D10), one process per extension origin, jitless, with content scripts as isolated worlds and extension pages in own-origin renderers — no extension code shares a renderer with web content (EX-D3); (3) **capabilities, not booleans** (EX-D5): install-time declaration is a ceiling, host access is runtime-granted per origin and container, revocation is destructive, growth is staged until re-grant; (4) **`webRequest` is an observational adapter over CB-D15's audited tier-3 path**, and blocking interception does not exist (EX-D7); (5) **trust is a published process**: two signatures per package, review decisions in a public log, ownership transfer as a review event, updates on the uniform grid, no install telemetry (EX-D10, EX-D12). Stage 2 staffing is **9–14 engineers† plus 2–4† non-engineering review and directory operations** over roughly 12–18 months†, off the Stage 1 critical path, which closes ledger gap G-20.

## 1. The problem and who has it

Izcal ships in Stage 1 with **no extensions** (Stage 1 design D-S3): content blocking is built in, the agent is a principal (D12), and the platform is deliberately absent. That is the right Stage 1 answer and a knowingly incomplete product: a large share of the browser's actual users — people whose password manager, blocker, translator or text expander is an extension — experience the browser as missing something they already own. Stage 2 must close the gap, and the only question is what it costs and what it buys. Buying Chrome's platform wholesale buys its documented record: an install-time all-or-nothing permission model that made "read everything, for ever" the cheapest extension to write; review that missed the DataSpii exfiltration of about 4 million people's browsing histories and a click-fraud wave of 500+ extensions that an outside researcher surfaced; The Great Suspender, sold and turned malicious on about 2 million people. Refusing the platform wholesale also has a record: Tor Browser's no-extensions policy is defensible for anonymity, and Safari's App-Store-mediated model produced a thin catalogue.

The people with the problem, precisely:

- **People whose workflow includes an extension** — password managers first. They want what they had, without inheriting a decade of incidents.
- **Extension authors**, who decide with their effort whether this platform has an ecosystem; Firefox and Safari both chose near-total Chrome API compatibility because porting cost is everything.
- **Privacy-conscious people**, for whom extensions are simultaneously the best tool (blockers) and a documented superidentifier (two detected extensions made 77.5% of a field study's users unique — [Carnus, NDSS 2020](https://www.ndss-symposium.org/ndss-paper/carnus-exploring-the-privacy-threats-of-browser-extension-fingerprinting)).
- **Administrators**, who need force-install, block and audit to work without becoming the malware channel the force-install policy historically was.
- **Izcal's engineers**, who need Stage 1 to leave the right seams (worlds, the component host, the audited interception path, the prompt layer) so Stage 2 is a platform, not a retrofit.

The Phase 3 amendment ledger records the gap as **G-20**; this spec closes it with the design, the targets and the staffing (9.3).

## 2. History: what came before and why it was built that way

Full chronology and sources: [history_postmortems][HP], [distribution_signing][DS], [privacy][PV].

### 2.1 Chronology

| Year | Project / event | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 2009–10 | Chrome extensions, then the Web Store | A JS API surface over a multi-process browser, all host permissions granted at install, store-hosted | Multi-process isolation made in-process add-on models unnecessary; the store was the anti-abuse mechanism (a ~75% drop in uninstall-support requests followed Windows install restrictions in 2014–15) | Won. Install-time host grants became the platform's load-bearing weakness |
| 2005–2017 | Firefox XUL/XPCOM add-ons | Chrome-privileged code, complete UI control | A single-process browser with an add-on model older than the sandbox era | Killed by Firefox 57 (2017): e10s made the model unsafe and slow; about two-thirds of the catalogue was lost at the cutoff |
| 2015 | Firefox WebExtensions + mandatory signing | Chrome-compatible API, signed packages | e10s, spyware defence, cross-browser compat | Became Firefox's platform; the signing intermediate's 2019 expiry disabled the ecosystem overnight and was hotfixed in under 9 hours ([Mozilla Hacks](https://hacks.mozilla.org/2019/05/technical-details-on-the-recent-firefox-add-on-outage/)) |
| 2018–2026 | Chrome Manifest V3 | Service workers, declarativeNetRequest, remote-code ban, promise APIs | Security (no remote code), privacy (declarative rules), performance (no resident background) | Won by managed decline over 4.5 years; capability (blocking interception) removed at vendor pace; uBlock Origin left Chrome; Firefox kept MV2 and its own MV3 shape |
| 2019–2021 | The abuse waves | DataSpii (4M histories), 500+ click-fraud extensions (1.7M installs, found by an outside researcher), The Great Suspender (sold, 2M users), 197 remote-code Firefox bans | Update-time capability in an install-time review model | Every incident was an update-channel or business-model failure, not an install-time one |
| 2020– | Safari Web Extensions | The Chrome/Firefox API inside App-Store-mediated native apps (Developer ID + notarization outside the store) | User demand met with Apple's distribution model | Strongest tamper resistance, thinnest catalogue; WebKit moved the API engine-level in 2025 |
| 2024– | Honey/PayPal litigation; 2025's trojanized-extension wave | A policy-compliant extension whose business model was affiliate expropriation; 16 extensions, 3.2M users, whose operator likely bought them | Review checks code, not incentives; ownership transfer is not a review event | The scandal class moved from malware to legitimate code with misaligned incentives |

### 2.2 Technology mismatches: assumptions that no longer hold

- **One process, one principal**: the era the XUL model assumed is gone; today's engines isolate per site, and an extension platform that ignores process boundaries is unbuildable on a modern engine. Izcal's isolation key is (site, container) (I-1), so an extension platform designed for it can be container-scoped — something no incumbent platform was positioned to do retroactively.
- **Install-time grants for ever**: falsified by a decade of incidents; every vendor has since moved to runtime, per-origin grants. A new platform has no migration excuse to keep the old shape.
- **The background page that never sleeps**: falsified by footprint reality; MV3's service-worker fix created a lifetime model whose main product is a workaround (the offscreen document with 15 documented reasons). A designed-from-scratch platform can declare wake intents instead of retrofitting them.
- **The store as the only trust boundary**: falsified by the incidents — the kill switch and update channel are where abuse is actually caught. Trust must be a published process with a record, not a gate.
- **JavaScript-only extension code**: a historical accident of the platforms, not a requirement; a component-model engine makes typed, non-JS components possible — useful, but not the ecosystem's need today ([GO 5][GO]).

## 3. Today: the landscape and its gaps

Full evidence: [surface_manifest][SM], [security_robustness][SR], [permissions_prompts][PP], [gaps_opportunities][GO].

**What the platforms are.** Three implementations of one documented model: Chrome's MV3 (service workers, declarativeNetRequest, remote-code ban, install-time ceiling over runtime host grants), Firefox's MV3-with-differences (event pages, blocking `webRequest` retained, MV2 alive), and Safari's (the same API inside App-Store apps, per-site grants by default). The surface is large — about 85 documented MV3 namespaces in Chrome, with roughly 16 doing the security- and privacy-relevant work — and converging: promises and `browser.*` are now the cross-browser norm.

**What today's platforms cannot do or see** (each with its evidence in the notes):

| Gap | Evidence |
|---|---|
| No audit surface: nobody can answer "what did this extension do?" | Live worker inspection is the ceiling; no platform ships a record of requests read, pages touched or grants used ([GO 2][GO]) |
| No per-container capability scoping | Containers scope site identity only; an extension granted broad hosts reads every container ([GO 3][GO]) |
| The installed set is effectively public | Resource-URL probing identifies >50% of top extensions ([Sjösten et al., CODASPY 2017](https://doi.org/10.1145/3029806.3029821)); 2 detected extensions made 77.5% of field-study users unique; interaction-triggered probing reaches extensions static probing misses ([PV 1][PV]) |
| Review is latency reduction, not a gate | The store operator's own paper ([Jagpal et al., USENIX 2015](https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/jagpal)): 96.5% detection, half within 25 minutes — yet fewer than 100 evasive extensions reached 50M+ users; an independent 2025 audit ([Palant](https://palant.info/2025/01/20/108-malicious-browser-extensions-on-chrome-web-store/)) found 96 of 108 reported extensions still live 19 months later; no operator publishes decisions, statistics or precedent ([SR 5][SR], [DS 5][DS]) |
| Update channels, not install review, are the attack surface | Copyfish, Web Developer, the 2017 sprees, The Great Suspender — every major incident arrived by update or sale ([HP 4][HP], [DS 4][DS]) |
| No conformance contract | No WPT-equivalent for extensions; MDN's compat data is the only shared record; testing is manual ([GO 4][GO]) |
| The lifetime model needs its workaround API | MV3's service worker has no DOM; the answer is a 15-reason offscreen document with justification strings ([GO 1][GO]) |
| Native messaging is the highest bridge with the least ceremony | One registry key converts any matching extension into OS code execution; no connect-time prompt is documented on any platform ([SR 3][SR]) |
| Multi-condition API checks fail in practice | The documented CVE class is contract confusion — header rewrite in the wrong context, content scripts in the wrong world, a missing prompt — not sandbox escapes ([SR 8][SR]) |

**What Izcal can do that no incumbent can** (its own architecture, per `izcal-capabilities.md`): scope every capability by container because the broker mints all authority (B2, I-1); audit extension behaviour because every cross-process message is typed and recordable (B1, NW 6.19); inspect extension contexts through the same protocol pages use (IIP); publish a falsifiable compatibility contract because the platform has spec anchors and a conformance harness pattern (DT-4/DT-6); and make extension decisions replay because decision points are recorded inputs (JS-D15, CB-D21). None of these requires a new invention; each requires the extension platform to be designed as a client of machinery the engine already has — which is this spec's shape.

## 4. First principles

### 4.1 The users' real tasks

An extension is third-party code a person installs to change what their browser does. Behind the install lie a small number of real tasks:

- **Change what is shown or sent** (block ads and trackers, restyle pages, rewrite links, add a sticky note UI), usually on every site.
- **Change what the browser is for the person** (password management, a different new tab, keyboard shortcuts, a reading mode), which is shell-shaped, not page-shaped.
- **Automate a personal workflow** (clip a page, post to a service, fill a form) — the task today's agent products are re-entering through the front door.
- **Watch and record** (developer tools, privacy dashboards, price trackers) — the task that is also the biggest privacy hazard, because watching every request and reading every page's content is indistinguishable from spyware at the API level.
- **Connect the browser to the person's machine** (native messaging to a password manager or editor).

The recurring design failure of existing platforms is that the fourth task — watching — has the same API shape as the first three, so a blocker and a spyware kit are the same program with different rulesets and honesty. CB-D15's evidence chapter is the record of what that failure costs.

### 4.2 Platform primitives

- **The web's own primitives**: HTML documents, the DOM, events, service workers, messaging. An extension is not a new kind of object; it is a principal holding capabilities over ordinary platform objects.
- **Izcal's primitives**: the broker mints all authority as typed capabilities over typed IPC (HGBA-3 B2); the component host (JS-D10) runs internal Wasm components with an embedded engine; isolated worlds with per-world side tables and Xray-style views (DM-D16) give a second principal a private view of the DOM; the compiled, sealed content-blocking pipeline (CB 6.3) turns rule text into data with zero read capability; IIP (DevTools spec) is a typed, auditable inspection protocol; the wake grid and the recorded-input substrate (B1, JS-D15) make any decision replayable.
- **The record**: MDN's, Chrome's, Firefox's and Safari's extension documentation, read as prose (M-1 part 3), and the deprecation history of MV2.

### 4.3 Izcal's principles that bind this design

**P1** (security and privacy of the user): an extension is hostile code until reviewed, and even reviewed code is a third party whose capability growth must be visible. **P2** (web compatibility and correctness): the documented API is the compatibility contract, and conformance gates it (EX-1) — a faster, smaller surface that silently diverges is a regression. **P4** (performance and energy): extension wake-ups, alarms and content-script injection are budgeted (EX-4). **P6** (evidence over aspiration): the fingerprinting and cost claims are measured, and every † threshold names its instrument. Plus D-S1: the engine's anti-fingerprinting posture (noise by default, uniformity as opt-in) cannot be weakened by an extension, and its surfaces are not for sale in the directory. **D12**: an agent is a principal, never a privileged extension — this platform is not the agent integration. Staffing is this spec's output (9.3), and G-20 closes with it.

### 4.4 Assumptions kept and broken

| Assumption inherited from existing platforms | Keep or break | Reason |
|---|---|---|
| Extensions are JavaScript packages with a manifest and a Chrome/Firefox-shaped API surface | **Keep the shape**, as a compatibility contract | Porting cost decides whether the platform has an ecosystem at all; Firefox and Safari both chose near-total Chrome compatibility for this reason (surface_manifest note). The shape is a published interface, not a licence to inherit its failure modes |
| An extension's capabilities are a flat list of booleans granted at install | **Break** | The booleans are the read-everything disguise CB-D15 documents; capabilities become (API, origin set, container, sensitive class) tuples granted by the broker |
| Host access is granted at install time and lasts forever | **Break** | Host access is runtime-granted, scoped per container, and revocable; Chrome's own 2021–2025 host-permission reform and Safari's per-site model are the direction of travel ([PP][PP]) |
| One manifest version for ever (MV2) | **Break**: MV3 only | MV2's load-bearing difference — blocking `webRequest` — is refused here anyway (CB-D15); supporting it buys only the refusals. A published migration mapping covers the rest |
| Extensions can define browser chrome (themes, settings overrides, search providers) | **Break, except where the shell designs a slot** | Search-provider and settings overrides are the top documented hijack surface; Izcal's shell is a sandboxed page (D-S2) with no scripted customization except through W-1 components |
| Silent updates of code | **Keep the update flow, break the silence** | Updates stay on a uniform schedule (CB 6.9's pattern) but an update that widens capabilities is staged until the user re-grants; every install and growth event is a recorded, explained decision |
| The extension store is the trust root and its review is invisible | **Break** | Trust follows the root-program analogy (NW 6.5.2): published decisions with evidence, signed packages, a public decision log; review is a process with a record, not a rite |
| Extensions run where the browser happens to run them | **Break** | Extension background code runs in a dedicated component-host process (JS-D10); content scripts run in isolated worlds; extension pages run in renderers of their own origin. No extension code shares a renderer with web content |
| An extension sees all containers | **Break** | Containers are first-class policy contexts (I-1 (a)); extension capabilities are scoped per container by default |
| `devtools.*` is how tools extend the browser | **Break** (already refused by the DevTools spec) | Tool integration goes through IIP as a client, which is typed and auditable, not through a privileged page |
| Storage is an API detail | **Keep the API, delegate the engine** | `storage` maps onto the storage service (G-19); this spec owns only the mapping and the grants |

## 5. Lessons learned

Ideas and evidence only; no code, no templates (M-1).

| Project / event | What happened | The lesson | How Izcal applies it |
|---|---|---|---|
| Firefox XUL/XPCOM add-ons (−2017) | Complete chrome-privileged access; multiprocess Firefox made it unsafe and slow; the 2015 rebuild announcement drew hundreds of angry comments, and Firefox 57 shipped with roughly two-thirds of the catalogue lost | A capability model that was never a capability model cannot be patched into one; and an API break is survivable only if the replacement reaches parity for the majority's uses first | Extensions get typed capabilities from the first commit (EX-D5); Ring A covers the majority's uses (blockers, passwords, content tools) before any exotic surface |
| Chrome's install-time host permissions (2010–2024) | All hosts granted at install created the read-everything-forever class every later incident exploited | Install expresses intent; **running expresses grant** | Host access is runtime, per-origin, per-container (EX-D5), the end state every vendor has since converged on |
| DataSpii (2019) | Extensions with ordinary-looking permissions exfiltrated full browsing histories of ~4M people, and the store did not act for an extended period | Broad read capability is indistinguishable from spyware; review latency is the failure | Tier 3 is scoped, badged, audited and per-container; sensitive classes are separately granted; the audit view answers "who saw what" without a researcher's reverse engineering |
| The Great Suspender (2021) | Sold; the update added remote code execution; ~2M users; the fix was a remote kill | Install-time review cannot see future updates; the update channel is the attack | Re-review before re-signing (EX-D10); ownership transfer is a review event; capability growth staged until re-grant (G175); a remote kill exists and its use is disclosed in the decision log |
| Honey/PayPal (2024–) | A policy-compliant, store-listed extension whose business model was invisible expropriation of affiliate commissions | The scandal class is legitimate code with misaligned incentives, which permission models cannot catch | The review states what was tested and the decision log is public; monetization disclosures are review questions, not policy footnotes |
| Chrome MV2→MV3 (2018–2026) | Capability (blocking interception) removed by one vendor over 4.5 managed years; the critique's own concession was the remote-code ban; Firefox kept MV2 and its own MV3 shape | De-capability-ization at vendor pace substitutes the vendor's schedule for the ecosystem's; the declarative model's *privacy* rationale was right even where its limits were wrong | Blocking is Izcal's own declarative pipeline (CB), so no extension capability is load-bearing for it; the rings are published, revisable, and refusals carry reasons and reversal triggers — deprecation as policy, not as a rollout |
| Firefox 57 aftermath | Deep UI-modification add-ons died with the privileged surface; content-level tools thrived | The surviving ecosystem is the one the safe API can express | UI slots are typed and few (EX-D15); what a slot cannot safely express is Ring C, published |
| Firefox signing and the 2019 certificate expiry | Mandatory signing (right) arrived with a single intermediate whose expiry disabled the ecosystem overnight; recovery took a Normandy hotfix in under 9 hours and back-ports completed 10–11 days later | Signing is necessary and is a product-grade single point of failure with a physical clock | Two signatures (author identity, platform trust), offline roots, rotation with overlap through the update system, and an emergency channel that does not depend on opt-in telemetry |
| Safari's extensions-as-apps (2020–) | Strongest tamper resistance, thinnest catalogue; the API later moved into the engine itself | Distribution rigidity trades directly against ecosystem size; engine-level API stability is the endgame | Directory + developer mode + enterprise keys on one verification path; the published contract is engine-level and falsifiable (EX-1) |
| Chromium-fork store fragmentation | Each fork re-reviews the same packages; trust does not fork with code | The trust layer is a component, not a side effect of a store | Signing, review records and the decision log are separable (`hg-ext-store`), so a downstream build can operate its own trust tier over the same packages |
| The extension-less cost | Users pushed off extensions land on DNS proxies and userscript managers — strictly worse channels (one userscript manager shipped analytics on by default) | Refusing the platform does not remove the need; it moves it to unguarded channels | Stage 2 ships the platform; the rings say what it will not do and why, so the need is served where it can be watched |

## 6. Design

### 6.0 Architecture at a glance

An extension is a **signed package** with an **MV3 manifest**, installed into a **profile**, granted **typed capabilities by the broker**, and running as three kinds of context:

| Context | Where it runs | What it holds |
|---|---|---|
| **Background contexts** (the service worker of the MV3 model, and any DOM-needing offscreen equivalent) | The **component host process** (JS-D10), one process per extension origin per profile; each context is one agent on the host's embedded engine (jitless, as JS-D10 fixes) | Its capability tokens; no DOM of any page; no direct network sockets (requests go through the network service under its grants) |
| **Extension pages** (action popup, options page, new-tab override, sandboxed pages) | Renderers keyed to the extension's own origin, per (origin, container), exactly like a site under I-1 | Full platform Web APIs as any document has; extension APIs through its capability tokens |
| **Content scripts** | Isolated worlds (DM-D16) inside the web-content renderers of the (site, container) they match | Xray-style DOM views; no capability tokens at all — every privileged API call is a typed request to the extension's broker-side grant set |

```
 broker (grants, prompts in the GR-D28 layer, update client, native-messaging spawner)
   | typed capability IPC (B2)
   +-- component host (JS-D10): extension agents (service workers), hg-ext-host
   +-- extension-page renderers (origin-keyed, per (origin, container))
   +-- web-content renderers: content scripts as isolated worlds (DM-D16)
   +-- partition services: tier-1/tier-2 lists as data (CB 6.3); tier-3 interception (NW 6.19)
   +-- storage service: extension storage (G-19 mapping)
   +-- IIP: the `extensions` domain; inspection of every extension context
```

The decision that shapes everything else: **the platform is a capability system wearing the WebExtensions API as its published face.** Compatibility decides what the face looks like; capabilities decide what the face can do.

### 6.1 The compatibility surface: three rings, published and falsifiable (EX-D1)

Izcal publishes one table — in this spec and as a machine-readable index shipped with the documentation — that lists every WebExtensions API namespace in one of three rings:

| Ring | Meaning | Contract |
|---|---|---|
| **A — implemented** | Izcal implements the documented behaviour | A conformance suite (`hg-ext-conformance`) covers it; pass rates are published per release (the DT-4/DT-6 pattern); removal needs the same deprecation discipline as an IIP domain (12 months, two releases) |
| **B — accepted as data** | Izcal reads the format but no code runs | The CB 6.10 list formats (ABP-family, DNR JSON, Safari content-blocker JSON, hosts); each parsed by `hg-cb-syntax`, compiled to the sealed file, never executed |
| **C — refused** | Izcal does not implement it | Each entry carries the refusal reason, the user need it fails, and the reversal trigger that would reopen it |

Compatibility is decided per **namespace**, then per **method** where a namespace is only partly safe (the table records the split; `tabs` for example is Ring A for tab metadata and messaging and Ring C for anything that reads cross-origin URL state without a host grant — the URL is page data and follows the host grant).

The rule that makes the table exhaustive: **the published ring list is closed and default-deny.** A namespace not named in it is refused with the reason "unlisted" until this spec rings it, and EX-1's conformance duty follows the table exactly as published. The entries (the inventory of [SM 2][SM], ringed):

- **Ring A (implemented)**: `runtime` (messaging, `getManifest`, `id`, URL, platform info, `externally_connectable` per-site messaging), `storage` (local, session, managed; sync in Stage 3), `tabs`/`windows` (metadata, messaging, create/update within host grants; cross-origin URL state is page data and follows the host grant), `scripting` (injection into granted hosts), `action`, `pageAction` (a published divergence: one implementation behind `action`, where Chrome and Firefox keep the namespace separate), `alarms`, `commands`, `contextMenus`/`menus`, `permissions` (the runtime-grant protocol), `i18n`, `notifications`, `omnibox` (keyword-gated), `downloads` (observe and manage, broker-mediated), `idle`, `bookmarks`, `topSites` (sensitive class), `history` (sensitive class), `sessions`, `offscreen` (a real hidden document in the extension-page renderer, not a 15-reason workaround), `nativeMessaging`, `search`, `webNavigation` (tier-3 granted), `cookies` (sensitive class, container-scoped), `webRequest` (an observational adapter over tier 3), `management` (restricted to self and to the user's settings page), `userScripts` (developer-mode gated, USER_SCRIPT world, 6.8), `activeTab` (the ephemeral action-scoped grant of 6.5), `pageCapture` (sensitive class: capture of a granted tab into a packaged document, per the documented `activeTab`-style gate), `tabGroups`, `dom` (`openOrClosedShadowRoot` for open roots; closed roots need the scripting grant), `browsingData` (**sensitive class**: wiping history, cookies and storage is a destructive act on the person's data, runtime-granted with a plain-language prompt), `contextualIdentities` (Izcal's own containers as the documented API — the one namespace that maps onto the engine natively), and the plumbing trio `events`, `extensionTypes`, `types`, plus `extension` (legacy aliases) and `declarativeNetRequest` — **ringed per method below**, like `tabs`.
- **Ring B (accepted as data)** — CB's surface, not a new one: DNR JSON rulesets, ABP-family lists, Safari content-blocker JSON, hosts files, each parsed by `hg-cb-syntax` and compiled to the sealed file (CB 6.10). The `declarative_net_request` manifest key's **static ruleset resources** ride this path: data in, compiled, zero read.
- **Ring C (refused)**, each with its reason: `devtools` and `devtools_page` (tool extension goes through IIP; the DevTools spec's decision stands); `debugger` (an unscoped full-page inspection grant dressed as an API); `proxy` (an extension-owned network path defeats container routes, NW's partitioning and the audit trail); `vpnProvider`, `dns`; `privacy` and `browserSettings` and `contentSettings` and `accessibilityFeatures` (browser-wide feature toggles would break D-S1's posture and P2); `chrome_settings_overrides` and search-provider keys (the hijack channel); `getMatchedRules`, `onRuleMatchedDebug`, `updateDynamicRules`, `updateSessionRules` and `modifyHeaders` under `declarativeNetRequest` (match feedback is refused by CB 6.15 tier 1; dynamic ruleset mutation has no tier home; header removal is forbidden by G81 — each a Ring C row with the shared reversal trigger "an owner decision amending CB-D15 with a defined data-only semantics"); DNR `redirect` to extension-packaged resources (redirects go only to Izcal's neutral-resource catalogue); `webAuthenticationProxy` (a credential-relay for enterprise WebAuthn; refused until the security owner designs the flow, reversal trigger named); `desktopCapture`, `tabCapture` (Stage 3, behind per-session broker grants, with the capture pipeline); `fileSystemProvider`, `identity` and `oauth2` keys, `gcm`, `pkcs11`, `certificateProvider`, `platformKeys`, `enterprise.*`, `captivePortal`, `power`, `fontSettings`, `input.ime` (OS and vendor surfaces without an Izcal owner yet); `declarativeContent` (a deprecated abstraction over what `scripting` and DNR-as-data do better); `wallpaper`, `system.*`, `loginState`, `printing*`/`printerProvider` (Stage 3, with the print pipeline), `readingList`, `tts`/`ttsEngine` (Stage 3, OS voices), `theme`, `sidebar_action`/`sidePanel` (Stage 3, pending the shell's panel design), Firefox's `urlbar`, `find`, `clipboard` and `menus.overwrite` experiments (the clipboard goes through the editing spec's broker path, not an extension API), and every vendor-experimental namespace.

- **Staged, not refused**: `tabCapture`, `desktopCapture`, `tts`/`ttsEngine`, `printing*`/`printerProvider`, `sidebar_action`/`sidePanel`, `protocol_handlers` and the file-handler keys carry a **staged** marker with their target stage. The published table has four states — implemented, accepted-as-data, staged, refused — so a staged namespace is never mistaken for a refusal.

Manifest keys follow the same closed-list rule. Beyond 6.2's named keys: `update_url` is **refused** for directory installs (updates ride Izcal's uniform grid; a self-hosted `update_url` is honoured only for developer-mode installs, labelled as such); `chrome_url_overrides` covers `newtab` only (the bookmarks and history pages are shell surfaces); `protocol_handlers`, `file_handlers`, `file_browser_handlers`, `mime_types_handler`, `input_components` (Stage 3, with the OS-integration design); `cross_origin_embedder_policy`/`cross_origin_opener_policy` (not meaningful for same-origin-only extension documents; ignored with a report); `export`/`import` (refused); `browser_specific_settings` read for metadata and ignored otherwise; `differential_fingerprint`, `offline_enabled`, `requirements`, `version_name`, `short_name` accepted as metadata; `nacl_modules`, `event_rules` (refused); `minimum_chrome_version` honoured as a floor against the documented version map, and an Izcal version below the floor fails the check with the extension disabled and the reason shown.

Refusal is not erasure: each Ring C entry is a row a user or author can read. The two refusals most likely to be reversed on evidence are named where their triggers are stated: the DNR method refusals and `webRequest`'s blocking forms (EX-D7's shared CB-D15 trigger, 6.20) and the MV2 refusal (EX-D2's measured trigger, 6.20).

### 6.2 Manifest model: MV3 only, with a published migration mapping (EX-D2)

Izcal parses `manifest_version: 3` and refuses version 2 with a page that names the reason: the only capabilities MV2 has that MV3 lacks — blocking request interception, persistent background pages with full DOM, remotely hosted code — are exactly the capabilities this platform refuses (CB-D15; G176). Everything else MV2 names is a rename (`browser_action` → `action`, background `scripts`/`page` → `service_worker`, `web_accessible_resources` array → object, `content_security_policy` string → object), per [Chrome's MV2 deprecation timeline](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline).

What makes this more than a store policy: Izcal publishes the **migration mapping as data** — a table of MV2 construct → MV3 construct → Izcal ring — and ships `hg-ext-migrate`, a tool that rewrites a manifest by that table and reports what it could not translate. The mapping is falsifiable: every row has a conformance test on the MV3 side.

Manifest keys are accepted or refused per key, not per version: `incognito` maps onto containers (below), `content_scripts` follows DM-D16, `host_permissions` and `optional_host_permissions` become broker grants, `web_accessible_resources` gets per-session random URLs by default (6.14), `externally_connectable` is enforced broker-side, `sandbox` marks documents that run in a unique-origin extension-page renderer with no extension APIs, `storage.managed_schema` reads enterprise policy, `minimum_chrome_version` is honoured as a floor, `key` is not needed (identity derives from the signature, 6.4).

### 6.3 Process model (EX-D3)

**One component host process per extension origin per profile**, running Izcal's Wasm engine with the WASI embedding (JS-D10) and the embedded JS engine in **jitless form**, as the JS spec fixes for this embedder. Each background context (per container where the extension spans containers) is one agent: one agent heap, one `Mutator`, its heap in its own blocks of the process's one cage (JS-D17). The process holds only that extension's contexts, so cross-extension isolation is process-granular and cross-container isolation is agent-granular plus grant-granular.

Why jitless is acceptable and how it is checked: extension background work is event-driven and light — the jitless tier costs the workloads that full-tier JITs exist for, which are page benchmarks, not alarm handlers. EX-4 measures real extension workloads on the jitless tier; if a measured corpus shows extensions whose latency or CPU budgets cannot be met jitless, the finding goes to the owner with the cost (open question Q-EX-2), not to a silent tier raise.

**Content scripts** run inside web-content renderers as isolated worlds (DM-D16) — extension code does execute in a renderer process, in a world with an Xray view, no capability tokens and no way to reach the page's JS objects. This is the deliberate exception to W-1's "no extension code in renderer processes"; its wording is amended to the structural rule (Section 9.5): *no extension code shares a renderer with web content except content scripts in isolated worlds, which hold no authority.*

**Extension pages** render in their own renderers, keyed to the extension origin per (origin, container). They are ordinary documents: full Web platform, no ambient authority, their extension APIs broker-granted. The never-share rule holds **absolutely**: I-1's reduced-isolation mode never co-locates extension-page renderers with any other origin (EX-2 states it as an invariant, not a default).

**Lifecycle.** Background contexts are event-driven with a documented idle shutdown (30 s†, the number the DOM spec already sets for page service workers and Chrome documents for extension workers), woken by declared wake intents: events on granted hosts, alarms (minimum period 30 s†, aligned to the wake grid where the event allows batching), runtime messages, tier-3 interception events, update and grant events. A wake intent is a recorded B1 input. The `offscreen` API maps to a real hidden document in the extension-page renderer with the extension's own grants and no user interaction.

### 6.4 Identity, origins and packaging (EX-D4)

- **Identity**: an extension's ID is derived from its publisher signing key, the documented pattern behind Chrome's manifest `key` field and CRX identity (a hash of the public key); Firefox instead documents store-assigned IDs. The ID never changes on update. The ID is not a secret; `runtime.id` exposes it in extension contexts only.
- **Origin**: `ext://<id>/` — one origin per extension, one site key for I-1 purposes, so extension pages never share a renderer with each other's origins or with web content.
- **Package**: a ZIP with a detached **Ed25519 signature from Izcal's signing service** over a canonical manifest digest (the package verification is `hg-ext-package`, G174; primitives from `hg-crypto`, G36). Sideloading in developer mode uses a **self-signature** that the UI labels as developer-mode, per session, per container.
- **Reproducibility**: the package hash is a recorded B1 input in every recording (6.16); a recording pins the extension versions it ran with.
- **No licence requirement on extension code.** Extensions are third-party code running in Izcal, not dependencies Izcal ships; the M-1 licence rule binds Izcal's own artifacts. The platform defines sandbox and capability rules instead, and the directory's listing displays each extension's own licence as a user-facing fact.

### 6.5 Permissions: capabilities, not booleans (EX-D5)

Every API permission becomes a **capability tuple** the broker holds: (extension, API, origin set, container, sensitive classes). The manifest's `permissions` and `host_permissions` are **the install-time declaration of what the extension may ask for** — not what it gets. Three grant states per host capability, following the documented Chrome model but defaulting stricter:

| State | Meaning | Default |
|---|---|---|
| **on click** | The capability is exercised only while the user interacts with the extension's entry point on that site | Default for new host grants |
| **on this site** | Per-origin grant, per container | User-set |
| **on all sites (this container)** | Broad grant, per container | Never default; requires the explicit broad prompt |

API permissions split into three classes:

- **Structural** (alarms, i18n, runtime, storage, action, commands, contextMenus, notifications): declared at install, shown in plain language, granted at install, no prompt fatigue. They confer no data access.
- **Sensitive** (history, cookies, bookmarks, topSites, downloads of observed data, `webNavigation`, `webRequest` observational): runtime-granted per container, each prompt naming the data in words a person can act on ("your full browsing history on this device"), badged in the site panel, auto-revoked after documented disuse (the Stage 1 permission store's rule).
- **Escalated** (tier-3 interception, native messaging): a separate grant tier with its own human review and connect-time prompts.

The ceiling rule: **the extension can never be granted more than it declared** — the effective grant is always the intersection of the declared set and what the user has allowed, the invariant the platform with the most users converged on ([PP 8][PP], [PP 1.7][PP], High), alongside the opposite-direction convergence of Firefox and Safari, both of which default host access to per-origin runtime grants ([MDN, `host_permissions`](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json/host_permissions)). Install expresses intent; running expresses grant.

Warnings are generated mechanically from the capability tuples, collapse where several permissions share one message, and stay rare by design: comprehension research shows warning fatigue above roughly a 50% ask rate, so the platform pushes asks to the time of use — a runtime, per-origin prompt attached to a user action — instead of a wall of text at install: warnings inform only where the request is anomalous, and history and all-hosts warnings are so commonly requested that nobody reads them ([PP 4][PP], High). `activeTab` is Ring A and maps to an **ephemeral, action-scoped grant**: invoking the extension's action on a tab grants it that tab's host access until navigation, with no prompt beyond the invocation itself.

Revocation is **destructive and durable**: removing a grant removes it from the granted set, so a re-request always prompts again — the granted-versus-active split that lets one platform silently re-approve a removed permission is not reproduced ([PP 7][PP], High). Host grants auto-expire after documented disuse (the Stage 1 permission store's rule; no vendor ships this for extensions — an opportunity, not evidence against).

Enterprise grants are visible: policy may force-install or block, but any **host capability granted by policy** surfaces in the same UI as a user grant, labelled with its source — the invisible-enterprise-grant gap the standards debate left open ([PP 6.34][PP], [PP 8.2][PP], High). The directory pairs prompts with a trust layer: a recommendation tier whose vetting criteria are published, and a periodic hygiene pass that flags rarely used broad-host extensions for review, in the pattern the largest vendor shipped in 2024 for web permissions ([PP 7][PP]).

Prompts are presented by the broker in the **OS prompt layer** (GR-D28), which means the security scope's consent rules hold in full: approval counts only on broker-observed input (CON-R1), the broker owns the window and routes the input (CON-R3), and the input is unoccluded (CON-R5). The prompt names the extension, the capability, the container and the site. **Install and growth grants are requested by the extension, shown in the prompt layer, and approved only by the broker**; the extension and the shell's Extensions page can request a grant but cannot approve one, and the shell shows a **persistent pending marker** until the person decides or disables the extension (shell 6.15). **Capability growth on update is staged, not applied** (G175): an update whose declared grants exceed the installed set is held until the user re-grants, and the re-grant event is recorded — the update-discoverability hole one platform documented in its own tracker (an update adding host permissions without showing it) is named and refused here.

**Declared content scripts and the default.** A manifest `content_scripts` declaration is not silent until a click: at install, the declared matches create an **install-time "run on the sites you visit" grant** — a distinct grant class from API host access, shown plainly at install, scoped per container, viewable and editable per site in the settings page, and subject to the same auto-expiry as every host grant. This is the grant the restyler, translator and link-rewriter classes live on; the on-click default remains the default for API host access requested later. The formal evaluator of EX-6 models this grant class as a first-class tuple member, so the manifest semantics and the evaluator agree.

`incognito` manifest keys map to containers: `spanning` (default) spans the containers the extension holds grants for; `split` runs the extension per container — same ID and `ext://` origin, but a per-container agent, per-container storage keying, and per-container grants; `not_allowed` confines it to the default container. This replaces Chrome's two-profile model with Izcal's first-class one.

The `permissions` API itself (`chrome.permissions.request`) is the documented runtime path for optional permissions, implemented over the same broker protocol; a page cannot distinguish an Izcal grant from a documented Chrome grant except where this spec's rings say so.

### 6.6 Extension storage (EX-D6)

The `storage` API maps onto the storage service (G-19); this spec owns the mapping and the grants, not the engine:

| Area | Mapping | Keying |
|---|---|---|
| `storage.local` | The storage service, quota per extension | Per (extension, profile); per (extension, container) in `split` mode |
| `storage.session` | In-memory in the component host, cleared at profile exit | Per (extension, profile); per (extension, container) in `split` mode |
| `storage.sync` | **Stage 3**; Izcal has no sync service until the Stage 3 row exists; until then the call resolves with an empty store and the incompatibility is published | — |
| `storage.managed` | Enterprise policy read by the broker's configuration reader, read-only | Per profile |
| IndexedDB / Cache API in extension pages | The storage service like any origin, keyed to `ext://<id>` | Per (extension origin, container) |

Quotas are the storage spec's; per-extension quota consumption is reported against that spec's measures, and against FP's canonical bytes where it defines none. Extension storage never stores page data implicitly: only what the extension explicitly writes, under its own grants.

### 6.7 Network: the three tiers, and `webRequest` as an adapter (EX-D7)

CB-D15 fixes the security spine; this section places the API names on it:

1. **Tier 1 — list providers, zero read.** An extension supplies rule lists (Izcal filter syntax, the ABP-family subset, **DNR JSON**) in its package or from a declared update URL, fetched like a user-added list (CB 6.9/6.10): through the container's route, no cookies, on the daily slot. Izcal compiles them with the same compiler, budgets and sealed file; the extension gets no match feedback; the user's explanations name the extension's list. `declarativeNetRequest`'s JSON ruleset format is Ring B — accepted as data, never executed. Updates to lists are data and need no re-review (G81 makes code-carrying rules impossible).
2. **Tier 2 — typed interventions as data**: cosmetic rules and redirects to Izcal's neutral resources, same vocabulary, same pipeline.
3. **Tier 3 — scoped, audited interception through IIP** (NW 6.19), **observe-only for extensions**: granted per (origin set, container), badged on affected tabs, audited (principal, request, partition, action), with bodies, cookies and credentials as sensitive classes needing their own grant. The `mock`, `block` and `modify` intents of NW 6.19's vocabulary stay reserved to the user's own declarative pipeline (CB tiers 1–2) and to agent and automation principals under NW 6.19 — an extension holding a block-or-modify grant through the interception path would be MV2's blocking `webRequest` renamed, which CB-D15 refuses. Admitting one is an owner decision amending CB-D15 with its own reversal trigger, never a consequence of the shared vocabulary. An update that widens a tier-3 grant asks again (CB-D15).

On this sits the compatibility adapter: **`webRequest` is implemented as an observational adapter over tier 3**, whose grants carry the `observe` intent only. A ported extension that only watches requests keeps its code shape and gets exactly the tier-3 authority: scoped, badged, audited, re-prompted on growth. The blocking forms (`webRequestBlocking`, `onBeforeRequest` with `cancel`/`redirect` semantics) are refused — they are the MV2 surface CB-D15 names, and DNR plus tier 3 are their replacements. There is **no extension-owned request stream**: requests never leave the network service's path except as typed, recorded observations through the partition service.

**Not in any tier**: an extension cannot alter Izcal's own lists, unblock a Security entry, or touch the protected-media host (CB 6.16's table applies unchanged to extension-supplied lists, which are attacker-controlled input like any user-added list).

**Phishing and malware reputation** (phishing spec PH-D8, §6.7): an **extension-initiated navigation is checked** against the phishing bundle before the request, exactly as any other navigation; no extension API exposes, suppresses or alters a verdict; an extension cannot add an allowlist entry or a removal, and an extension-supplied list cannot carry an allow entry for phishing. `ext://` pages are labelled "Extension page" in the UI and are governed by EX-D10's review and signing rules, while remote content an extension page embeds is checked as a subframe.

### 6.8 Content scripts and user scripts (EX-D8)

- **Worlds**: content scripts run in the extension world of the (site, container) renderer — own realm, own globals, per-world side table, Xray-style views (DM-D16). They never see page JS objects, and the page never sees theirs. `world: "MAIN"` is supported: it runs the script in the page's world with page-level authority and no extension API access beyond what `externally_connectable` messaging permits — the documented semantics, made safe by being ordinary page script.
- **Injection**: `manifest content_scripts` and the `scripting` API, into hosts the extension holds grants for. Match patterns use the documented grammar (`<scheme>://<host><path>`), parsed by the one tier-P parser (G170). `run_at` (`document_start`/`document_end`/`document_idle`), `all_frames`, `match_about_blank` and `exclude_matches` follow their documented meanings.
- **Injection is a recorded event**: every injection records (extension, world, site, container, pattern) as a B1 input; the mutation log tags the world (DM 6.x provenance).
- **Document hosts without injection**: content scripts never inject into the SVG-as-image scriptless static host (SV-D1), the PDF viewer's host, or DevTools panes — these hosts either render no script or hold privileged views, and no documented extension surface requires injection there.
- **Editing**: a content script that inserts or replaces text does it through typed edit intents producing trusted `beforeinput`/`input` (ED-D21); it never mints user activation. Clipboard reads need the person's paste press; clipboard writes are labelled with the extension's identity in the provenance log (ED-D10, ED-D11).
- **User scripts** (`userScripts` API): code the user, not the store, installs. Supported in a **developer mode, per container**, with per-script consent at install, a distinct listing in the settings page. User scripts run in a **USER_SCRIPT world** — a per-extension isolated world on DM-D16's side tables with zero capability tokens, opt-in messaging (`configureWorld`) and `worldId` sub-worlds, the documented model; page-world execution is an explicit per-script option the consent flow names. This is the honest version of a surface every engine either omits or smuggles in.

### 6.9 Extension pages and the UI surface (EX-D15)

Extension pages are documents rendered by the engine in the extension's origin renderers. **Every slot below is Stage 2**: Stage 1 ships no extensions (D-S3; Stage 1 13.2), so the action button and popup, the options page, the badge, the per-site grant row, the new-tab and homepage override prompts, the recommendation tier badge and the command editor are all Stage 2, and the shell specifies their Stage 2 surface in shell 6.15. A-EX-12 stays open until those surfaces exist. UI slots, each broker-mediated and each refusing script in the shell's renderer (D-S2):

| Slot | Design | Abuse answer |
|---|---|---|
| **Action button + popup** | A toolbar slot per extension, popup as an extension-origin overlay panel positioned by the broker; keyboard and screen-reader access required (AX bar) | Popup is never drawn over content prompts; it cannot cover the prompt layer (GR-D28) |
| **Options page** | Opens as a tab or embedded page | Ordinary origin |
| **New-tab override** | `chrome_url_overrides.newtab` supported; install requires an explicit prompt naming the change, and the settings page shows and reverts it | New-tab hijack is a prompt-level event, revocable |
| **Homepage / startup override** | Same prompt path | Same |
| **Search-provider override** | **Refused** (Ring C). Izcal's search settings are the user's; the documented extension key is a hijack channel | — |
| **Context menus** | Typed menu items registered into the shell's menu model, scoped by (extension, container, context type) | Menu items are data; no script runs in the shell |
| **Commands** (keyboard shortcuts) | Registered with the broker's key registry; conflicts resolve by documented rules, shown in settings | No global keylogging: commands are declared key sequences, not listeners |
| **Omnibox** | Keyword-gated: the extension sees input only after the user types its keyword; the suggestion list is labelled with the extension | Keystroke capture gated by an explicit act |
| **Notifications** | Through the Stage 1 notification pipeline: activation-gated quiet UI, per-site style mute, auto-revocation of unused grants | Notification spam controls apply to extensions as to sites |
| **Badges, title, icon state** | Data on the action slot | — |
| **Sidebar/side panel** | **Stage 3**, after the shell's panel design lands (Ring C today) | — |

Themes are refused: the shell's theming is CSS-only user theming (D-S2), and an extension cannot restyle browser chrome.

### 6.10 Editing, clipboard and activation (EX-D16)

Already stated in 6.8 because it lives there: extensions get **one labelled edit channel** (ED-D21) and **broker-mediated clipboard** (ED-D10/ED-D11). Two additions on the browser side: a tier-3 interception grant never implies an editing grant, and no extension API may synthesize OS-level input events — input synthesis is an agent-platform surface with its own principal, not an extension capability (D12).

### 6.11 Native messaging (EX-D9)

Native messaging connects an extension to a program on the person's machine; it is the platform's highest-risk bridge and is **broker-mediated end to end**:

- **Host registration**: a native-messaging host manifest (documented JSON, OS-specific install locations — registry on Windows, fixed directories on macOS and Linux), validated by the broker: it names the executable path, a description, and the allowed extension IDs. Only `hg-ext-native` parses host manifests (G170's parser family, G178).
- **Spawning**: only the broker spawns the host process — never a renderer, never the component host — with a scrubbed environment (the declared set only), no inherited credentials, and the extension's identity on the command line for the host's own logging.
- **Session**: messages are length-prefixed JSON per the documented wire format; the broker proxies the byte streams with size caps and timeouts, and the session is auditable: the settings page shows connected hosts, and the audit log records connect, disconnect and message volumes (not contents — the payload is the host's and the extension's business; the fact of the pipe is the user's).
- **Grant**: native messaging is an escalated capability: install-time declaration, a **connect-time grant naming the host program in plain language** (no platform documents one today — the gap that lets one registry key turn any matching extension into OS code execution), re-granted per container. A compromised host can do what it could do anyway on the person's machine — the browser's job is not to make it easier, and never to hold the person's secrets in the pipe.
- **No browser APIs for the host**: the host speaks to its extension and nothing else.

### 6.12 Signing, distribution, review and updates (EX-D10)

The root-program analogy (NW 6.5.2): a trust decision made by an operator is published with its evidence and can be challenged. The rules:

- **Two signatures per package**: the **author's key**, from which the extension's ID is derived (6.4), and **Izcal's platform signature**, applied after review. Directory and enterprise installs verify the **platform** signature; developer-mode installs verify the **author self-signature** and are labelled per session and container. In both cases identity follows the author's key. A platform-key incident therefore cannot silently re-point an extension's identity or push updates once the TUF root rotates — the 2019 lesson where one vendor's intermediate signing certificate expired and disabled every extension on every channel at once is answered structurally: keys rotate through the update system's root with an overlap window, the roots are offline, and the response playbook is the CA incident playbook ([DS 6.34][DS], High).
- **Ownership transfer is a review event.** The documented malware wave of 2025 (trojanized extensions bought from their benign authors, millions of users affected) makes the sale the control point: a change of author key, a transfer of the directory listing, or a re-registration of the update URL triggers full re-review before the platform signature is renewed; grants are not transferable, so the bought extension's readers re-prompt from scratch ([DS 4.23–26][DS], [PV 4.22–23][PV], High).
- **Distribution**: the Izcal extension directory is the primary channel: a public listing (an ordinary website, not a privileged surface), published review criteria, and a published **decision log** — each accept, reject or removal entry with its reason, in the pattern of NW's witnessed list governance (CB 6.9) rather than an unexplained takedown. Removals for security reasons propagate through the update system's emergency channel (UPD-R5).
- **Self-distribution**: signed-by-author packages install in developer mode only (per container, labelled, unlisted). Enterprises install via policy from their own signed source, labelled "installed by your organization", in a distinct trust tier that never mixes with the store tier — the G84 discipline (one trust domain never input to another), applied to code.
- **Review is not the safety boundary.** The store operator's own published evaluation found high detection rates and fast median takedowns for known-bad packages, while a few dozen evasive extensions reached tens of millions of users, and an independent 2025 audit found most reported extensions still live many months later ([SR 5][SR], High). Review is latency reduction. **Containment is the capability model** — the grants, tiers and audit of this spec — and **detection is signals**: user abuse reports through the OHTTP channel (the CB 6.11 pattern), the local audit view that lets a person see what an extension did, behavioural analysis of packages in the review lab, and a remote kill whose use is disclosed in the decision log. Static analysis plus behavioural review in a recorded harness states what was tested; tier 3, native messaging and sensitive classes require human review; data-only updates (tier 1 lists) do not (G81).
- **Updates**: package updates ride the uniform update grid (CB 6.9's schedule pattern): a public fetch identical for every client. The broker verifies signatures and hashes only (G174), then hands the sealed package to a **parsing utility process** (`hg-ext-package-main`, strictest tier, per IPC-R15's pattern as CB applies it to list text) — the only place manifest, match-pattern, i18n and native-host-manifest bytes are parsed (G170). An update that changes code requires re-review sign-off before the signing service signs it. Capability growth stages the update until re-grant (G175).
- **Install telemetry: none.** Installs are local events. The directory is an ordinary website whose traffic rules are the web's own; the install action itself performs no report (FP-D9; EX-7 verifies in the lab).

### 6.13 CSP for extension pages (EX-D11)

Extension pages ship under an **enforced CSP that no manifest can relax**: `default-src 'self'; script-src 'self'; object-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'`, plus `connect-src` from the extension's declared cross-origin needs (fetches remain subject to CORS; CSP does not grant read access to anything). No `unsafe-eval`, no inline event handlers in package HTML (the verifier rejects them at packaging, G176), no remote code of any kind — the MV3 rule, made structural. Wasm components are the platform's code form and are unaffected. Sandboxed pages (manifest `sandbox`) run in a unique-origin extension renderer with no extension APIs, where a developer may relax their own CSP for their own documents — the documented Chrome model, scoped so the relaxed CSP can never reach a document that holds a capability token.

Content-script CSP: Chrome and Firefox divide these loads differently (MDN documents the divergence), and Izcal **fixes one semantics and publishes the divergence** — the content script's own code runs under the extension CSP's script floor; DOM-inserted loads it causes are subject to the page's CSP. The Xray view, not CSP, is the isolation boundary.

### 6.14 Privacy: fingerprinting, telemetry, and the extension list (EX-D12)

An installed-extension set is a superidentifier, and the measurement record is blunt: resource-URL probing identifies more than 50% of the most-installed extensions from a page; two detected extensions gave 77.5% of users a unique identifier in a field study of about 84,000 users, four gave 94.5%, and the inferred sets reveal religion, health conditions and politics; interaction-triggered fingerprinting reaches extensions invisible to static probing ([PV 1.x][PV], High). Chrome's only shipped countermeasure, per-session resource URLs, covers the static vector and none of the behavioural ones; Tor Browser's answer is to ship no third-party extensions. Izcal's defences are structural:

- **Resource URLs**: `web_accessible_resources` URLs carry a **per-(top-level site, session) random component by default** — the `use_dynamic_url` behaviour made default — so a probe that succeeds on one site fails on the next and yields no cross-site set. A manifest may opt out (some embed workflows need stable URLs); the opt-out is a listed, explained setting.
- **`runtime.id`** is exposed only in extension contexts; pages learn an extension exists only through `externally_connectable`, which the extension must have declared for that site — an intended, auditable channel.
- **Behavioural and interaction-triggered probes** (DOM changes a page can read back, timing and side effects around a user action) are not eliminable; they are measured in the fingerprinting harness (EX-8), whose probe corpus includes the interaction-triggered class, and the results are published so authors know their exposure — the same measured-residual policy as D-S1 and CB 6.16. The settings page says plainly that an installed extension can be detectable by the sites it runs on.
- **`management` is restricted**: an extension can see only itself; the full installed set is shown in the user's settings page and to the user's own IIP clients, never to script or to remote inspection clients.
- **No install telemetry, no extension census.** The only aggregate channels are: crash attribution by extension version, riding **D-S4's opt-out crash-report channel** with the same minidump-only limits and disclosure (not a new telemetry stream), and footprint statistics under FP-D9's opt-in rule. Both are published.
- **Incognito/containers**: container-scoped grants (6.5) mean a leaking extension leaks at most the containers it holds grants for, and `split` mode gives it separate identities per container.

### 6.15 Security scope

The attacker table, per the security scope's format (rows proposed to the security owner in 9.5):

| Attacker | Goal | What stops it |
|---|---|---|
| A malicious extension (in the directory, undiscovered) | Read pages, history, credentials; intercept traffic | Capability tuples; host grants default on-click; tier 3 scoped/badged/audited; sensitive classes separately granted; no extension code in a process with web content (G172); audit log the user can read |
| A compromised extension (supply chain, update hijack) | Turn a good extension into a reader | Signed packages; re-review before re-signing; capability growth staged (G175); update fetches signature-verified before parsing (G174); witnessed update metadata via the TUF root |
| A malicious native host | Attack the browser from outside | Broker-only spawning (G178); scrubbed environment; no browser APIs for hosts; audited sessions |
| A compromised component host | Read across extensions or reach web content | One extension origin per process; the cage's H-2 untrusted-read rule (JS-D16) applies; typed IPC with no ambient authority; the process holds no page data and no cookie jars |
| A compromised web-content renderer | Escape through a content script into extension authority | Content scripts hold no tokens; every privileged call is a broker-checked typed request; Xray views block prototype and expando confusion (DM-D16) |
| A hostile page | Enumerate extensions; confuse a content script; ride an extension's grant | Resource-URL randomization; `externally_connectable` enforcement; worlds; grants are keyed to (origin set, container) and never borrowed across frames without a matching grant |
| The directory operator (Izcal) | Target a user with a special build | One signed package per version, one decision log, published review criteria; the update system's witnessed log pattern is proposed for review decisions (open question Q-EX-4) |
| A malicious page planted in an extension's own UI (DOM-injection XSS in an extension page) | Run script in a document holding capability tokens | The enforced CSP floor (EX-D11) stops remote and eval'd script; it does **not** stop DOM-injection XSS — the residual is named, measured in the review harness, and treated as the author's vulnerability class since Carlini's 70-vulnerabilities-in-40-extensions study, not as a platform defence |
| Contract confusion (the documented CVE class) | A permission check passes with one condition wrong: wrong world, wrong context, wrong timing | Deny-by-default on every multi-condition check: the formal grant evaluator (EX-6) makes each decision a total function of recorded state, and a check with any missing input denies |

**In-cage trust assumption**: where one extension spans containers, its per-container agent heaps share one cage (JS-D17). The heaps hold that one extension's data, whose visibility its grants already cap; the escalation of a compromised heap is therefore bounded by the extension's own grants, not by the container boundary. Where a measured corpus shows that bound is too wide, the fallback is one component host per (extension, container) — EX-D3's stated reversal.

**Residual channels, stated**: content-script DOM side effects (measured, 6.14); the existence of an installed extension visible to a site it declares `externally_connectable` for; native-messaging session volumes (metadata only); the timing of tier-3 observations (typed, recorded, and themselves auditable). Extension-jitless performance is a footprint, not a privacy channel: the engine's D-S1 posture is not extension-dependent.

### 6.16 Determinism and recording (EX-D13)

The extension platform is a recorded participant in B1:

- **Recorded inputs**: wakes and their triggering events, alarms, messages received, injections performed, permission grants and revocations, package installs/updates/enables/disables/removals, tier-3 interception decisions, list-update versions and compiled-file hashes (CB-D21's pattern, carried over).
- **Pinned inputs**: the package hash and manifest digest of every installed extension the recording touches; list source URLs (encrypted in the recording, redacted in reports, as CB 6.14 does).
- **Replay unit**: extension component hosts record as **their own replay hosts**, joined by IDs per process — the ledger's C-19 recommendation; if C-19's resolution lands otherwise, extension hosts follow it, and EX-5's corpus is defined over whatever unit C-19 fixes.
- **Replay**: the component host re-executes; tier-1 matching is a pure function (G177) so decisions recompute; divergence is reported at the first differing recorded decision, with the extension named. **Tier-3 modifications** that change page-visible bytes are recorded as inputs, on NW-D2's recorded-bytes pattern, with a per-session size bound and the credential-input exclusion the DevTools recording rules set: no credential input in any extension recording, and the sensitive classes of 6.5 redacted on export. Tier-1 and tier-2 decisions are never logged per decision; they are recomputed (G177).
- **No hidden timing**: extension-visible time outside the wake grid exists only as the coarse timed APIs; a decision that depends on wall-clock time is a recorded decision point (JS-D15's pattern).

### 6.17 Observability (EX-D14)

- **The shell's Extensions page** (settings): installed extensions, their grants per container, their tier, their audit summaries, connected native hosts, developer mode. This is the about:extensions and about:debugging equivalent in one place — no second secret page.
- **IIP `extensions` domain**: list extensions and their contexts, their processes, capability state, and recent recorded decisions; subscribe to grant and lifecycle events. It is a normal IIP domain: scoped principal, consent, audit log, sensitive classes.
- **Inspection**: any extension context (background agent, extension page, content-script world) is inspectable through the existing IIP script and DOM domains exactly like a page — the component host's engine is a normal embedder with the debugger interfaces of JS-M1's scope; content-script worlds appear as targets in the page's pane (the DOM spec's world model made visible).
- **The audit view**: tier-3 interception records join the "who received what" ledger (NW 6.18), so "which extension saw which request" is answerable per session, per container.
- **Conformance and corpus**: `hg-ext-conformance` publishes Ring A pass rates per release (DT-6's pattern); a reference corpus of ported extensions runs in CI as the compatibility canary (EX-1).

### 6.18 Crates and rules (G170–G179)

| Layer | Crate | Role | Tier |
|---|---|---|---|
| L5 | `hg-ext-manifest` | The only manifest and match-pattern parser: strict MV3 manifests, match patterns, i18n message files, host-manifest JSON; structural verifier and bounds-checked readers; linked by the parsing utility only (G170) | P |
| L2s | `hg-ext-capability` | Capability tuple types, grant state machine, plain-language grant descriptions; L2s because the broker, the component-host binary and the settings UI all link it | P |
| L2s | `hg-ext-package` | Package format: ZIP reader, canonical manifest digest, Ed25519 verification, update manifests; L2s because the broker (verification) and the parsing utility (reading) both link it | P |
| L5 | `hg-ext-host` | In the component host: extension agents, wake scheduler, lifecycle, `offscreen` documents, the extension-API host bindings | P |
| L5 | `hg-ext-api` | The typed Ring A surface, generated from `.hgidl`: tabs, storage, alarms, scripting, action, commands, contextMenus, notifications, runtime, i18n, downloads, and the rest | P |
| L5 | `hg-ext-content` | In renderers: content-script world management, injection, match evaluation, the scripting API's renderer side | R |
| L5 | `hg-ext-native` | Native messaging broker side: host-manifest validation, spawning, stream proxying with caps | P |
| L5 | `hg-ext-observe` | The IIP `extensions` domain backend; audit records for grants, lifecycle and tier-3 use | P |
| L6 | `hg-ext-host-main` | The component-host binary (serves the per-extension-origin component host) | P |
| L6 | `hg-ext-package-main` | The parsing utility binary (package, manifest and native-host-manifest parsing, strictest tier, per IPC-R15) | P |
| tools | `hg-ext-conformance` | Ring A conformance suite, reference-corpus harness, fingerprinting probe page | T |
| tools | `hg-ext-store` | The directory's backend records: review decisions, decision log, signing requests | T |
| tools | `hg-ext-migrate` | MV2→MV3/Izcal manifest migration tool driven by the published mapping table | T |

Rules, in the range assigned to this spec:

| Rule | Statement | Why |
|---|---|---|
| **G170** | Only `hg-ext-manifest` parses manifests, match patterns, i18n message files and native-host manifests; it runs at tier P and is fuzzed; an `ast-grep` check rejects other parsers | One hostile-input parser per format |
| **G171** | Every capability check is made by the broker against `hg-ext-capability` state; renderers and the component host present capability tokens and hold no grant truth; an `ast-grep` rule rejects grant-state reads outside the broker | Zero ambient authority; a compromised context escalates nothing |
| **G172** | No extension code runs in a process that renders web content, except content scripts in isolated worlds, which hold no capability tokens; the process-map test fails the build on a violation | Structural isolation; W-1's rule, restated |
| **G173** | No extension API call may return page data, browsing data or request data without a recorded grant covering (origin set, container, sensitive class); every tier-3 observation emits an audit record | The read path is always typed and always logged |
| **G174** | Package bytes are signature- and hash-verified by `hg-ext-package` in the broker before any other crate reads them; the broker then hands sealed bytes to the parsing utility process only, where `hg-ext-manifest` runs (G170) | The parser never sees unverified bytes; the broker parses no hostile text |
| **G175** | An update whose declared capabilities exceed the installed grant set is staged, not applied, until the broker records a re-grant; enforcement is in the broker's update client | Silent capability growth is the platform's named failure mode |
| **G176** | No extension surface executes text as code: CSP is enforced structurally (6.13), the packager rejects inline handlers, and `hg-ext-api` refuses string-code parameters; the only exception is the developer-mode `userScripts` path, per-script consented, which runs in the isolated USER_SCRIPT world (page-world execution only as a named per-script option) | The remote-code ban as a rule, not a store policy |
| **G177** | Tier-1 and tier-2 matching paths are pure functions of (context, sealed file, settings): no clock, no entropy, no allocation, no hash-map iteration, no panic — the G82 rule by reference | Replay; explanations that match decisions |
| **G178** | Native hosts are spawned only by the broker, only from `hg-ext-manifest`-validated registrations, with the declared environment set only; the broker parses no extension payload bytes | The bridge has one guarded door |
| **G179** | Install, grant, revocation, enable/disable, update, wake and removal events are recorded B1 decision points; no extension-visible decision may depend on time outside the wake grid without being recorded | Determinism and replay |

### 6.19 Ownership boundaries with other specs

| Boundary | Owned by | This spec's share |
|---|---|---|
| Isolated worlds, wrappers, Xray views | DOM spec (DM-D16) | Consumes; adds the extension-world identity and injection events |
| The component host, Wasm engine, WASI embedding, jitless embedder rule | JS spec (JS-D10) | Consumes; adds per-extension-origin process placement and the extension agent model |
| DNR/filter formats, the compiler, the matcher, the sealed file | Content blocking (CB-D15, CB 6.10) | Consumes; extension lists are just list providers |
| The IIP transport, session and consent model | DevTools | Consumes; adds the `extensions` domain backend |
| Storage service, quotas, eviction | Storage spec (G-19) | Owns the API mapping and per-extension keying |
| Prompt rendering and the OS prompt layer | Graphics (GR-D28), shell UI | Consumes; extension prompts are prompt-layer clients |
| Permission store, auto-revocation | Stage 1 13.2 permissions row; security scope | Consumes; extension grants live in the same store |
| Edit intents, clipboard | Editing (ED-D10, ED-D11, ED-D21) | Consumes; extension content scripts are one more labelled client |
| The update system, TUF root, emergency channel | Stage 1 13.2 | Consumes; extension packages are targets in it |
| The agent principal and its tools | HGBA-3 B9; future agent spec | Deliberately separate: agents are principals, not extensions (D12) |

### 6.20 Alternatives

| Decision | Alternatives | Recommendation | Reversal trigger |
|---|---|---|---|
| Compatibility strategy | (a) Chrome MV3 API surface as published; (b) a smaller "Izcal-native" API with a thin compatibility shim; (c) full surface including refused namespaces behind grants | **(a) as a ring-filtered subset of (c)**: the documented surface is the contract; the rings filter it. A native API (b) doubles the surface developers must learn and ships no ecosystem; full surface (c) re-creates the read-everything disguise | A top-need extension class is blocked by a ring decision and no data-only path exists (the CB-D15 reversal trigger, shared) |
| MV2 support | (a) MV3 only; (b) MV2 accepted with its blocking webRequest refused; (c) MV2 fully | **(a)**. (b) buys a parser and semantics for packages whose load-bearing difference is refused anyway; (c) re-creates the refused capability | Measured evidence that a needed class of top extensions is MV2-only for reasons Izcal supports (then (b), never (c)) |
| Background host placement | (a) one component host per extension origin per profile; (b) one host for all extensions; (c) extension SWs in renderers | **(a)**. (b) shares a cage across mutually distrusting extensions (JS-D17 makes it one cage; the blast radius is wrong); (c) violates W-1's restated rule and mixes authority | Footprint measurement shows per-extension hosts breach FP budgets (then one host per extension *publisher*, a middle cut, to the owner) |
| JIT tier for extension JS | (a) jitless as the embedder rule fixes; (b) full tier in the component host | **(a)**; measured, revisited at EX-4 | A measured corpus of real extensions misses its latency/CPU budget jitless (Q-EX-2, to the owner) |
| webRequest | (a) observational adapter over tier 3; (b) refuse the name, ship only a new API; (c) implement blocking forms | **(a)**. (b) breaks porting for no safety gain; (c) is the refused surface | Abuse data shows the adapter is used to disguise broad reading beyond tier-3 controls (then tighten to per-intent grants, never remove the audit) |
| Store model | (a) Izcal directory with published decisions; (b) no directory, author-signed packages everywhere; (c) multiple third-party directories | **(a)** with developer mode for (b)-style sideloading. A trust root with a published record is the whole lesson of the root program; (c) fragments accountability | The directory becomes a bottleneck reviewers document (review-median targets in EX-9 miss two quarters running) |

## 7. M-1 compliance

**Built new by Izcal, from the documented record**: every crate of 6.18; the Ring A API surface is written from the published extension documentation of the three vendors (read as prose; M-1 part 3), with Izcal's conformance suite and reference corpus as the oracles; the capability model, the grant state machine, the prompt flows, the review decision log, the package format and the migration mapping are Izcal's own designs anchored to the published documentation plus HGBA-2, CB-D15, NW 6.19, DM-D16, ED-D21 and JS-D10.

**Dependencies**: none beyond the engine's existing set. Package compression uses the standard ZIP container and Izcal's own deflate reader/writer (`hg-compress`, already shipped for HTTP). Signatures use `hg-crypto`'s pinned Ed25519 (G36). No extension-platform library from a browser project or doing engine work is used; the compatibility *contract* is documentation, not code, and the D-S12 class (recorded black-box observations in `compat/observations/`) covers behaviours the documentation leaves silent.

**Licences**: no new shipped dependency or data set is introduced, so no new licence question beyond the audit's standing rules. Extensions themselves are third-party code running in Izcal under the sandbox and capability rules; they are not Izcal's shipped dependencies and carry no MIT requirement (the directory displays each extension's licence as a user-facing fact, 6.4). Rule formats accepted as data (ABP-family text, DNR JSON) are formats, not works: the syntax documentation is cited as documentation, as CB 6.10 already does.

**Engine-source rule**: no Chromium, Gecko, WebKit, add-on or uBlock-class project source file, diff or commit was read or cited anywhere in this spec or its notes; values only source could state are marked as open questions or set from first principles with †.

**Spec anchors**: every Ring A namespace's conformance suite carries anchors to the published documentation section it tests (`// doc:` anchors, the G9 pattern applied to a prose record instead of a standard); the anchor index joins each to the reference-corpus runs that exercise it.

## 8. Targets

Thresholds marked † are set by this document from first principles, not measured elsewhere; each names what would falsify it.

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **EX-1** | Compatibility contract | 100% of Ring A entries — per namespace, and per method where the table splits (6.1) — have conformance suites with published pass rates; the reference corpus (the 50 extensions the review pipeline ported first) runs its declared core flows; 0 silent divergences from the documented behaviour (each divergence is either fixed, published as an incompatibility, or ringed down) | `hg-ext-conformance` in CI per release; corpus runs per release | The failing namespace is ringed down with a published note, or fixed before Stage 2 exit |
| **EX-2** | Structural isolation | 0 extension code in a web-content renderer outside isolated worlds; 0 extension-origin documents in a renderer with another extension's origin or any web origin; enforced by the process-map test on every build | G172's process-map test in CI | Ship blocker: fix the placement |
| **EX-3** | World isolation | The Xray-view differential fuzz suite (page mutates prototypes, expandos, getters; extension world reads the DOM) finds 0 leaks of page objects into the extension world over 10⁹† operations per release | `hg-ax-probe`-style differential fuzz in the DST farm | Ship blocker: fix, then re-run the full suite |
| **EX-4** | Host cost and latency | Idle extension (installed, no events): 0 wakeups/hour†, ≤ 2 MiB† resident share of the component host (FP's CC/CT measures, cited not redefined); event dispatch to a background context p95 ≤ 5 ms† jitless on the DC-4 reference device; content-script injection p95 ≤ 1 ms† per world | Lab, per release, on the footprint spec's reference devices | Lifecycle and injection work before new Ring A surface; jitless shortfall escalates as Q-EX-2 |
| **EX-5** | Determinism | Recordings that include extension activity replay with identical decision logs (0 divergences) on the extension corpus; every tier-3 decision, grant change and wake is in the recording | Replay comparison in CI, CB-D21's method | Fix the unrecorded input; ship blocker for the feature that introduced it |
| **EX-6** | Grant correctness | Every Ring A call's grant decision is derivable from the recorded capability state (a Kani-checked evaluator over `hg-ext-capability`); 0 calls granted outside the declared ∩ granted intersection in the DST farm's capability oracle runs | Formal evaluator + black-box oracle per release | Ship blocker on the grant path |
| **EX-7** | No install or update telemetry | 0 network requests outside the declared update fetches in the lab install/update harness, per release | Lab harness with a network recorder | Ship blocker |
| **EX-8** | Fingerprinting resistance | The enumeration probe suite (static resource-URL probing, `externally_connectable` probing, DOM side-effect diffing, interaction-triggered probes) identifies ≤ 1† installed Ring A extension beyond the extension's declared `externally_connectable` sites, across a corpus of 25† probe techniques, with resource-URL randomization opt-outs disabled; results published per release | Probe harness in the lab, PRIV-2's methods | Publish the measured channel, then fix or ring down the surface that leaks it |
| **EX-9** | Review throughput | Median review decision ≤ 10 working days† for a data-only or Ring A-core update; every decision in the public log with its reason | Store records | Add review capacity before adding Ring A surface |
| **EX-10** | Parser robustness | 100% coverage-guided fuzz saturation for `hg-ext-manifest` and `hg-ext-package` (G170, G174), 0 high-severity findings open at exit | Fuzz farm, SEC-28's method | Fix before the surface ships |

## 9. Milestones and staffing

### 9.1 What is staged, and why

Staging follows correctness, security and measurement only (M-1; no date-driven staging):

| Staged item | Stage | First-principles reason |
|---|---|---|
| `storage.sync` | Stage 3 | Izcal has no sync service; a fake sync that silently does not sync is a correctness lie. Published incompatibility until then |
| Sidebar/side panel, tab capture, TTS, printing APIs | Stage 3 | Each needs a host surface (shell panel design, capture pipeline, OS voices, print pipeline) that lands later; refusing them earlier is honest, shipping them half-built is not |
| `identity`, `proxy`, `vpnProvider`, `privacy`, `debugger`, `devtools.*` | Refused (Ring C), not staged | These are capability refusals with recorded reasons and reversal triggers, not schedule items |
| Native messaging | EX-M3, not EX-M1 | The highest-risk bridge is built after the grant system it depends on is verified (EX-6 gates it) |
| Tier-3 interception and the `webRequest` adapter | EX-M3 | Depends on NW 6.19's audited interception path and on tier-1/2 being live so most blockers never need tier 3 |

**Stage 1 seams** (built by Stage 1's owners, consumed here): DM-D16's worlds and per-world side tables; JS-D10's component host; the broker's typed capability IPC and permission store; the GR-D28 prompt layer; the wake grid and recording format; the TUF update root. EX-M0 verifies them by integration test. This creates **no new Stage 1 staffing and no extension surface** (D-S3 untouched): the tests are verification work counted in the Substrate and DOM tracks' existing rows, under the reconciliation's no-double-counting discipline (the ledger's C-6 itself remains unresolved and needs its owner decision).

### 9.2 Milestones

All Stage 2; the critical path (Stage 1 M0→M8) is untouched. Dates are offsets from Stage 1 exit, which the ledger's Q29 places at 48–60 months†.

| ID | Milestone | Exit criterion | Depends on |
|---|---|---|---|
| **EX-M0** | Seams verified | Integration tests (written by this track during late Stage 1, passing at Stage 1 exit): a content-script world injects into a test page; a stub component-host agent wakes on a recorded event and answers over typed IPC; a broker grant is minted, enforced, audited and revoked; a prompt renders in the GR-D28 layer | Stage 1 exit (DM, JS, broker, shell) |
| **EX-M1** | Packages and grants | Package format, manifest parser, capability store, developer mode, the Extensions settings page, tier-1 list providers end to end (DNR JSON and ABP-family through the CB pipeline); G170, G174 live; EX-10 fuzz targets running | EX-M0 |
| **EX-M2** | Background and content | Background contexts on the component host (lifecycle, alarms, messaging, storage, offscreen documents), `scripting`/content scripts with Xray views, tabs/windows/action/commands/contextMenus/notifications/i18n; Ring A core conformance green; EX-2, EX-4 measured | EX-M1 |
| **EX-M3** | Read paths and the bridge | Sensitive classes, tier-3 interception with the `webRequest` adapter, `webNavigation`, `cookies`/`history`/`topSites` under grants, native messaging, omnibox, downloads; EX-6 formal evaluator live; audit view in "who received what" | EX-M2, NW 6.19 path, ED-D21 channel |
| **EX-M4** | Distribution | Directory, signing service, review pipeline with the published decision log, update grid, hygiene pass, migration tool published; fingerprinting harness (EX-8) and determinism corpus (EX-5) measured | EX-M2 (EX-M3 parallel) |
| **EX-M5** | Stage 2 exit | All targets met or owner-waived in the decision log; the compatibility table published at its 1.0; G-20 closed in the ledger | EX-M1–EX-M4 |

### 9.3 Staffing and duration

The estimate is a first-principles guess†, in the same spirit as the other component specs: it is an output, set from the crate and surface inventory, to be re-set at EX-M1 with measured velocity. **9–14 engineers†**, on one track plus shares of others:

| Work | Scope | Engineers† |
|---|---|---|
| Platform core | `hg-ext-host`, lifecycle, wake intents, the component-host embedding, offscreen documents | 2–3 |
| Ring A surface | `hg-ext-api` codegen and the ~30 namespaces (deduplicated: `pageAction` behind `action`, `contextMenus`/`menus` one), storage mapping, `hg-ext-migrate` | 2–3 |
| Content scripts | `hg-ext-content`, injection, match evaluation, world identity, ED-D21 channel | 1–2 |
| Capability system | `hg-ext-capability`, the formal grant evaluator (EX-6), prompt flows with the shell track | 1–2 |
| Bridge and read paths | `hg-ext-native`, tier-3 adapter work with the networking track, `hg-ext-observe` | 1 |
| Distribution and trust | `hg-ext-package`, `hg-ext-store`, signing service, review pipeline, decision log | 1–1.5 |
| Conformance and privacy measurement | `hg-ext-conformance`, corpus, fingerprinting harness, EX-5 determinism corpus | 1–1.5 |
| **Engineering total** | | **9–14†** |
| Review and directory operations (non-engineering) | Reviewers, abuse triage, decision-log operation, recommendation-tier curation, signing-service operation (the CB 9.3 precedent: curators, reviewers and policy staff are budgeted, not absorbed into engineering) | **2–4† non-engineers** |

Duration: roughly **12–18 months†** from EX-M0 for EX-M5, running entirely inside Stage 2 and parallel with the other Stage 2 programs (incremental layout, containers UI, Android). Counted elsewhere and not here: the component host itself (JS spec), the world machinery (DOM spec), the storage service (storage spec), the prompt layer (graphics/shell), the IIP transport (DevTools), the update root (Stage 1). No engineer is counted twice.

**G-20 closes with this staffing**: the ledger's extensions gap ("Stage 2 surface, JS-D10 component host") is owned by this spec, is Stage 2, and is budgeted in the table above — 9–14† engineers plus 2–4† non-engineering review and directory operations; the ledger's pointer rows (CB-D15, CB 6.15, NW 6.19, Stage 1 D-S3) now point here.

### 9.4 Decisions for the owner

**Owner decision (21 October 2026): every recommendation below is accepted.** (EX-D1 to EX-D16.) Each row's options, cost and reversal trigger are in Section 6.20. A bare "yes" to any of these means the recommendation. Build-versus-reuse is not on this list: the standing answer is to build Izcal's own (M-1, D-S10).

| ID | Question | Recommendation | What it costs | Reversal trigger |
|---|---|---|---|---|
| **★ EX-D1** | The compatibility surface | Three published rings (implemented / accepted-as-data / refused), each Ring A namespace under a published conformance contract | The refusal list is public; some power users lose a favourite tool | A needed extension class is blocked and no data-only path exists |
| **★ EX-D2** | Manifest version | MV3 only; MV2 refused with a published migration mapping and a tool | MV2-only extensions do not run | Measured evidence a needed class is MV2-only for supported reasons (then MV2 manifests minus the refused capabilities, never full MV2) |
| **★ EX-D3** | Process model | One component host per extension origin per profile; background contexts as agents on the jitless embedded engine; content scripts as isolated worlds; extension pages in own-origin renderers | Process count scales with installed extensions (bounded by EX-4's budget) | Footprint breach (then per-publisher hosts, to the owner) |
| **EX-D4** | Identity and packaging | Signature-derived IDs, `ext://` origins, Izcal-signed packages, developer-mode self-signing | A signing service to operate | — |
| **★ EX-D5** | Permission model | Capability tuples; install-time declaration as ceiling only; runtime per-origin per-container grants; destructive revocation; auto-expiry; policy grants visible; prompts in the GR-D28 layer; the extension requests install and growth grants, the broker alone approves, and the shell shows a persistent pending marker | More prompts than install-time-all (by design, and rare by structure) | — |
| **EX-D6** | Extension storage | Map to the storage service; sync staged to Stage 3 | `storage.sync` is a published incompatibility until sync exists | — |
| **★ EX-D7** | Network APIs | The three CB-D15 tiers; DNR JSON as data; `webRequest` as an observational adapter over the audited tier-3 path; blocking forms refused | Blocking-interception blockers (scriptlet-dependent ones) cannot port | The CB-D15 reversal trigger, shared |
| **EX-D8** | Content scripts and user scripts | Isolated worlds per DM-D16; `world: "MAIN"` supported; `userScripts` behind developer mode with per-script consent | Developer-mode surface needs review care | — |
| **★ EX-D9** | Native messaging | Broker-mediated end to end; single guarded door; audited sessions; escalated grant | One more broker path to build and fuzz | — |
| **★ EX-D10** | Distribution and updates | Izcal directory with published review decisions and a decision log; signing after review; uniform update grid; capability growth staged; no install telemetry | An operations commitment (reviewers, signing infra) | Review throughput misses EX-9 two quarters running |
| **EX-D11** | CSP for extension pages | Enforced MV3-class CSP no manifest can relax; sandboxed pages the only relaxation, unique-origin | Rare author requests for eval are refused | — |
| **EX-D12** | Privacy | No install telemetry; resource-URL randomization default-on; `management` restricted; enumeration measured and published (EX-8) | Opt-out w.a.r. URLs remain a leak channel, measured and labelled | Measured enumeration above EX-8's threshold forces the opt-out's removal |
| **EX-D13** | Determinism | Extension decisions as recorded B1 inputs; replayed and diffed (EX-5) | Recording format work with the substrate track | — |
| **EX-D14** | Observability | One Extensions page; IIP `extensions` domain; audit view joining "who received what"; every context inspectable | A domain backend and frontend views | — |
| **EX-D15** | UI surface | The slot table of 6.9, **all Stage 2** (toolbar action, popup host, options page, badge, per-site grant row, new-tab/homepage override prompts, recommendation tier badge, command editor); search-provider and theme overrides refused; sidebar Stage 3 | Some hijack-adjacent but legitimate uses (themes) do not port | Shell panel design landing may open the sidebar slot in Stage 3 |
| **EX-D16** | Editing and input | Extensions edit through ED-D21 intents; no input synthesis; no activation minting | Extensions that simulated typing break | — |

### 9.5 Amendments this spec proposes to other documents (applied, 21 October 2026; the shell row is handed to the shell owner)

This document edits no other file. Every Izcal-document row below was applied by its target spec's owner on 21 October 2026, with a dated revision line as `CONTRIBUTING.md` requires, after the owner accepted Section 9.4's decisions; the applications are recorded as ledger rows **A-EX-1 to A-EX-15**, of which **A-EX-12** (the shell UI design row) is handed to the shell owner with `reports/Izcal shell.md` (batch 4) and is the one row not applied in this batch. The WECG row is an upstream position statement filed outside this repository and edits no Izcal document.

| Document | Proposed amendment | Reason |
|---|---|---|
| HGBA-2, W-1 and X-3 (G-5, C-17) | Restate the host from Wasmtime to the component host (JS-D10), already settled in C-17, and restate the rule structurally: **no extension code shares a renderer with web content except content scripts in isolated worlds, which hold no capability tokens; background contexts run in per-extension-origin component-host processes** | EX-D3, G172; the renderer exception must be stated once, not implied |
| Stage 1 design, D-S3 and Section 6 extensions row, and Section 9 Stage 2 row | "extensions arrive in Stage 2 via Wasmtime" → "in Stage 2 via the component host (JS-D10), per `reports/Izcal extensions.md`"; the Stage 2 backlog row gains "with EX-M0's seams verified in Stage 1 integration tests" | C-17; EX-M0 makes the Stage 1 ask explicit and small |
| Stage 1 design 8.3 | Add one row: "Extensions (Stage 2 program): 9–14† engineers plus 2–4† non-engineering review and directory operations, per the extensions spec 9.3; EX-M0's integration tests are verification work counted in the Substrate and DOM tracks' existing rows, and create no extension surface" | The reconciliation's double-counting discipline (ledger C-6 itself is unresolved and awaits its owner); G-20 closes with the staffing |
| JavaScript engine spec 6.0 (embedder list), 6.15 G6 and G14 | `hg-ext-host` (reachable only through the `hg-ext-host-main` entry point) joins the allowed `hg-js-host` dependents as the component host's extension-agent host crate; G6's exhaustive allow-list names it; the jitless embedder enumeration of 6.0 gains "and the extension agents of the component host" as a DOM-less embedder; G14's jitless loaded-library closure names `hg-ext-host-main` | EX-D3; the extension agents are one more embedder under the JS spec's rules, and no row existed for it |
| Networking spec 6.22 | "extensions (Stage 2, Wasmtime)" → "extensions (Stage 2, the component host)"; add: tier-3 interception is the only extension read path, and the `webRequest` adapter is a client of the NW 6.19 path with no new network-side authority | EX-D7; C-17's stale text |
| Content blocking spec 6.15 | "Stage 2's Wasmtime extensions" → "Stage 2's extensions (the component host)"; add that tier-1 extension list updates ride the container route per CB 6.10, and extension **package** updates ride the uniform update grid (CB 6.9's channel), so no per-user variation enters either | EX-D10; keeps CB's fetch rules intact and closes the code-vs-data channel question |
| DevTools spec 6.x (IIP domains) and Section 6 | Add the IIP `extensions` domain (list contexts, capability state, recorded decisions, lifecycle events; scoped principal, consent, audit) and one line in the observability row: the about:debugging equivalent is the shell's Extensions page plus this domain; no WebExtension `devtools.*` API (the existing decision stands) | EX-D14; keeps the DevTools spec's refusal undisturbed |
| Storage spec (G-19, in progress) | Extension storage mapping: `storage.local` per (extension, profile), `session` in the component host, `managed` from policy, extension-origin documents keyed like any origin per (origin, container); quota lines belong to the storage spec's measures | EX-D6; one owner for the engine, one for the mapping |
| HTML and CSS parsing and style spec (`hg-url`) | The `ext://` scheme and its origin grammar are registered with `hg-url` (the A-ST-2 ownership), and the site-key derivation for extension origins is stated: one extension origin = one site key for I-1 | EX-D4; the scheme must exist in the one URL crate |
| HGBA-2 I-1 | Extension origins are their own site keys; extension-page renderers never co-locate with any other origin, including in reduced-isolation mode | EX-D3, EX-2; the never-share rule as an I-1 note |
| Security scope | New process kind: the per-extension-origin component host (assets: that extension's tokens; no page data, no cookie jars, no grant truth); threat-model document per Stage 1 13.2's rule; IPC-R17 compromised-host harness for extension-API and native-messaging messages; G171–G175, G178, G179 as IPC/security rules; native-messaging host registration paths (registry, launch directories) added to the broker's inventoried inputs | 6.15; the security owner owns the final rows |
| Shell UI design (handed to the shell owner, batch 4) | The Extensions settings page (grants per container, tiers, audit summaries, native hosts, developer mode, hygiene pass); the action slot and popup placement; the new-tab and homepage override prompts; the recommendation tier badge; the command editor; the **tier-3 tab badge and site-panel extension state** | EX-D15; these are shell surfaces. The shell UI design is being rewritten as `reports/Izcal shell.md` (started 21 October 2026), so this row lands there; A-EX-12 stays open until it does |
| Resource footprint spec 8.2 and FP-1 class | Budget lines for the extension process kinds charged in CC/CT: the per-extension component host (per-extension share ≤ 2 MiB†, EX-4) **plus its per-process cage reservation and page-table line** (JS-D17's one reservation per process, multiplied by per-extension hosts); the **offscreen-document extension-page renderer** as a charged process kind with its own idle line; and an **N-extension scenario** (5/15/30 installed) added to the footprint lab runs beside FP-S30 | EX-4; footprint owns the canonical measures |
| Editing spec 6.16 | The extension client of ED-D21 is named: content scripts edit through typed intents with an `extension` principal label; clipboard writes carry extension provenance | ED-D21 already names extensions; the principal label closes the loop |
| Accessibility spec 6.x | Extension action popups, options pages and menus are in the AT bar: every Ring A UI slot is reachable and labelled through the AT-action and labelling rules (AX 6.9, AX-D11); AX-32's name-explanation rule applies to extension-provided names as to shell UI | 6.9's access requirement, owned by AX |
| Phase 3 ledger | G-20 closed by this spec (Stage 2, JS-D10 component host, 9–14† + 2–4†); the C-17/G-5 stale-text rows are discharged by the HGBA-2, Stage 1, NW and CB rows above; the G-rule registry registers **G170–G179 to this spec**, records **G150–G159 as storage's** and **G160–G169 as the WebCrypto spec's**, and notes that the registry's "G88 onward the next spec" line is superseded by the batch's allocations | — |
| WECG (filed upstream, not an Izcal document) | Position statement: per-container capability scoping, destructive revocation, extension-grant auto-expiry, and published compatibility contracts as proposals | 6.5, 6.1; the gaps note's record of the standards debate |

## 10. Risks

| Risk | Where | Mitigation |
|---|---|---|
| The ecosystem does not port: Ring A is large and Izcal is small | 9.3 | The rings make the surface bounded and honest; tier-1 data compatibility carries the flagship use (blockers) without code; the corpus canary (EX-1) measures real progress, not vibes; the migration tool cuts the mechanical cost |
| Chrome changes the surface under Izcal | 6.1 | The conformance contract is against the published documentation at a pinned date, re-diffed per release; divergence is published, not silently absorbed |
| Jitless background execution is too slow for real extensions | EX-4, Q-EX-2 | Measured before it matters; the owner decision path is named |
| The component host becomes an attack magnet | 6.15 | One origin per process; tier-P crates; fuzz targets (EX-10); the cage's H-2 rules; the SEC harness |
| Native messaging reintroduces the OS as attack surface | 6.11, G178 | Broker-only spawning, validated manifests, scrubbed environment, audited sessions; no browser APIs for hosts |
| Review throughput or quality fails, and the directory becomes either a bottleneck or a rubber stamp | EX-9, EX-D10 | Published medians and decisions make failure visible; data-only updates skip review by construction (G81); the reversal trigger names the escalation |
| Per-container scoping confuses ported extensions | 6.5 | The documented grant model is per-origin-per-container by default with `spanning` as the compatible mode; divergence is published; container semantics are tested in the conformance suite |
| Extension-based fingerprinting moves to channels Izcal does not measure | 6.14 | EX-8's probe corpus is adversarial and published; the measured-channel policy (publish, then fix or ring down) keeps the residual honest |
| MV3-only refusal breaks a load-bearing extension class | EX-D2 | The reversal trigger is measured, not anecdotal; the migration tool reports untranslatable constructs, which is the evidence collector |
| Staffing lands on a small team and the platform ossifies | 9.3 | Stage 2 placement keeps it off the critical path; the ring structure allows shipping a smaller honest surface rather than a larger dishonest one |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-EX-1.** Which namespaces the installed base actually calls (the store operators hold usage data Izcal lacks); the ring table is designed to be revisable per release when Izcal's own corpus produces evidence.
- **Q-EX-2.** Whether real extension workloads meet EX-4's budgets on the jitless embedded engine; measured at EX-M2; the escalation is the owner's (EX-D3's reversal trigger).
- **Q-EX-3.** The exact DNR feature set to accept as data beyond CB 6.10's list (rule priorities, dynamic rulesets set from an extension's tier-3 context); decided with the content-blocking owner at EX-M1.
- **Q-EX-4.** Witness cosigning for the review decision log (the CB 6.9 pattern applied to code review records); a non-staff governance cost, put to the owner with the directory launch.
- **Q-EX-5.** Enterprise distribution details (managed-device attestation, force-install surface), decided with the security owner when the enterprise profile is designed.
- **Q-EX-6.** Whether `hg-ext-store` runs inside Izcal's infrastructure or as a separate operated service; a governance and funding question (D-S8), named here so the staffing does not presume the answer.
- **Q-EX-7.** The `externally_connectable` allowance for first-run handshakes (an installer page talking to the freshly installed extension) — supported per the documented model, but its fingerprinting residual needs EX-8's first measurement.
- **Q-EX-8.** Whether content-script CSS isolation (`world`-scoped styles) is needed beyond the page's cascade; decided with the style and DOM owners if the corpus shows extension styles breaking pages.

### 11.2 Claims that are single-source, unverified or conflicting

- The 30 s† idle-termination and alarm-floor numbers are set from the documented Chrome/DOM-spec values and this document's first principles; no standard states them (EX-4, G179).
- EX-4's 2 MiB†, 5 ms† and 1 ms† thresholds are set by this document (†); the footprint spec's canonical measures are the instruments, not the source of the numbers.
- The fingerprinting claim in 6.14 (enumeration in minutes at scale) rests on the privacy note's papers; the numbers there are graded Medium where the source was a snippet, and EX-8 re-measures everything on Izcal before any claim ships in user-facing text.
- The claim that no vendor scopes extension capabilities per container is an argument from documented silence ([GO 3.3][GO], Medium); a vendor feature shipped after 20 October 2026 would not change the design, only its "no one ships this" framing.
- Review-median targets (EX-9) have no public comparable from any store operator; they are † values by construction.
- Three claims rest on Medium or Low note rows and are stated as narrative, not as targets: the store's latency on DataSpii ("an extended period", note Medium); the Web Developer extension's sale-versus-compromise mechanism (note Low, sources conflict); and the userscript manager's default-on analytics (note Low, community reports). None carries a design decision alone.



## Appendix A. Review disposition (revision 0.2)

### Fact-check: must-fix items

| Item | Disposition |
|---|---|
| FC-M1 (identity precedent misattributed to Firefox) | Applied: 6.4 now cites Chrome's documented `key`-field/CRX derivation and notes Firefox's store-assigned IDs. Design unchanged |
| FC-M2 (permissions-note row citations wrong) | Applied: warning-fatigue sentence now cites section 4 with the corrected claim (see FC-R3); enterprise-grant citation now points at the WECG #260 material |
| FC-M3 (ceiling-convergence citation range does not exist) | Applied: cites the note's convergence row and the intersection row |
| FC-M4 (decision-log pattern cited to CB's staffing section) | Applied: 6.12 cites CB 6.9's governance pattern |
| FC-M5 (amendment cites CB 6.10 for the container route) | Applied: 9.5's CB row cites CB 6.9 for the route and 6.10 for formats |
| FC-M6 (content-script CSP stated as one documented semantics) | Applied: 6.13 states the divergence and Izcal's chosen semantics as a choice, published |
| FC-M7 ("uniformity (D-S1)" misstates the accepted decision) | Applied: all three occurrences reworded to D-S1's actual posture (noise by default, uniformity as opt-in) |

### Fact-check: recommended items

| Item | Disposition |
|---|---|
| FC-R1 (no inline primary links) | Applied: inline links added at the heaviest dependence points (MV2 timeline, Carnus, Mozilla Hacks outage post, Sjösten CODASPY 2017, Jagpal USENIX 2015, Palant 2025, MDN `host_permissions`); the notes remain the full evidence trail |
| FC-R2 (Tor overstated) | Applied: "no third-party extensions" |
| FC-R3 (50% fatigue threshold is an inference) | Applied: reworded to the anomaly-baseline claim |
| FC-R4 (hotfix latency conflict; "11 days") | Applied: "Normandy hotfix in under 9 hours, back-ports completed 10–11 days later". The distribution note's "~24 h" row (6.34) is the note owner's to fix — flagged here rather than edited (this spec edits no other file) |
| FC-R5 (Safari not App-Store-only) | Applied: "App-Store-mediated", with the Developer ID/notarization path named |
| FC-R6 (distribution-note section citations) | Applied: 6.34 for the cert-expiry lesson; 4.23–26 with privacy 4.22–23 for ownership transfer; DS 5 for the no-published-decisions evidence |
| FC-R7 (G84 analogy loose) | Applied: reworded as the G84 discipline (one trust domain never input to another), applied to code |
| FC-R8 ("9.4 lists the two refusals") | Applied: 6.1 names the two refusals where their triggers are stated (6.20) |
| FC-R9 (trojanized-wave hedge dropped) | Applied: "likely bought" restored |

### Fact-check: notes items

| Item | Disposition |
|---|---|
| Ring C mixes refusals with staging | Applied: a fourth **staged** state published in 6.1 |
| `activeTab` absent from the Ring A list | Applied: named |
| `pageAction` alias is a divergence | Applied: published as a divergence like `storage.sync` |
| Three Medium/Low claims stated as fact | Applied: covered by a new sentence in 11.2 |

### Design critique

| Item | Disposition |
|---|---|
| **C1** (DNR ring row contradicts CB; not decidable) | **Applied as proposed.** Per-method rings: static rulesets as data (Ring B via the CB pipeline); `getMatchedRules`, `onRuleMatchedDebug`, `updateDynamicRules`/`updateSessionRules`, `modifyHeaders` and extension-resource redirects Ring C with the shared CB-D15 amendment trigger. EX-1's duty follows the table per method |
| **C2** (tier-3 intents re-create blocking interception) | **Applied as proposed.** Extension tier-3 grants are observe-only; `mock`/`block`/`modify` reserved to the user's pipeline and agent/automation principals; admission of any modify intent is an owner decision amending CB-D15 |
| **C3** (no JS-spec amendment row; G6 violated by construction) | **Applied as proposed.** New 9.5 row: G6's allow-list, the 6.0 embedder enumeration and G14's closure gain the extension host |
| **C4** (ring table not exhaustive) | **Applied with a stronger mechanism than proposed.** The table is declared **closed and default-deny**; every namespace the inventory surfaces is named with a ring or a staged marker, including `browsingData` (sensitive class), `contextualIdentities` (mapped onto Izcal's containers), `webAuthenticationProxy`, `desktopCapture`, `pageCapture`, `dom`, `tabGroups`, the plumbing trio and the Firefox experiments; the manifest-key fates are listed (`update_url` refused for directory installs, honoured only in developer mode) |
| **C5** (on-click default makes declared content scripts inert) | **Applied by the first proposed option.** Declared `content_scripts` matches create an install-time, auto-expiring, per-container "run on the sites you visit" grant class, shown plainly at install; EX-6's evaluator models it as a first-class tuple member |
| C6 (`split` contradiction; in-cage assumption) | Applied: `split` = same ID and origin, per-container agent, storage keying and grants; storage table keyed accordingly; the in-cage trust assumption stated in 6.15 with the per-(extension, container) host as fallback |
| C7 (userScripts page-world contradicts documented model) | Applied: USER_SCRIPT world with `configureWorld` messaging and `worldId` sub-worlds; page-world execution a named per-script option; G176's exception text updated |
| C8 (EX-M0 contradicts "no new Stage 1 work"; C-6 mis-cited) | Applied: 9.1 and the 9.5 Stage 1 row reworded (no new staffing, no extension surface, verification work in existing rows, C-6 named as unresolved) |
| C9 (recording unit under C-19; credential rule; tier-3 record size) | Applied: extension hosts record as their own replay hosts under C-19's resolution with the fallback named; the credential-input exclusion and sensitive-class redaction extended to tier-3 records; modifications recorded as inputs with a per-session size bound; tiers 1–2 recomputed per G177 |
| C10 (crash attribution vs D-S4) | Applied: attribution rides D-S4's opt-out crash channel with its limits; FP-D9 kept for footprint statistics only |
| C11 (review operation unbudgeted) | Applied: a **2–4† non-engineers** operations line in 9.3 (CB 9.3 precedent), and G-20's closure states both numbers |
| C12 (parser process placement unspecified) | Applied: `hg-ext-package-main`, strictest tier, per IPC-R15's CB application; G174 restated; the crate joins 6.18's table |
| C13 (footprint accounting one line where the model needs several) | Applied: the 9.5 footprint row gains the reservation/page-table line, the offscreen renderer kind and the 5/15/30-extension scenario |
| C14 (missing cross-references) | Applied: new 9.5 rows for `hg-url`/ST (the `ext://` scheme and site-key derivation) and HGBA-2 I-1 (extension origins as site keys, reduced-isolation exception); the shell row gains the tier-3 badge; CON-R1/3/5 cited at the prompt surface; 6.8 names the hosts content scripts never inject into (SVG-as-image per SV-D1, the PDF viewer host, DevTools panes) |
| C15 (sections 4 and 5 out of order) | Applied (also caught in self-review before the critique landed) |
| C16 (Q-EX-6 cited where Q-EX-4 meant) | Applied |
| C17 (platform-signature scope) | Applied: directory/enterprise verify the platform signature; developer mode the author self-signature |
| C18 (G-range registration) | Applied: the ledger row registers G170–G179, records G150–G159 as storage's and G160–G169 as WebCrypto's, and supersedes the registry's "G88 onward" line |
| C19 (L2s placement) | Applied: `hg-ext-manifest` moved to L5; `hg-ext-capability` and `hg-ext-package` kept in L2s with their multi-consumer justifications stated in the table |
| C20 (AX-32 mis-cited) | Applied: AX 6.9/AX-D11 for actions and labelling; AX-32 for name explanations |
| C21 (reduced-isolation contradiction) | Applied: the never-share rule stated as absolute, with the I-1 amendment row |
| C22 (namespace count) | Applied: ~30, deduplication stated |

[FR]: ../research_notes/Izcal%20extensions/frame.md
[HP]: ../research_notes/Izcal%20extensions/history_postmortems.md
[SM]: ../research_notes/Izcal%20extensions/surface_manifest.md
[SR]: ../research_notes/Izcal%20extensions/security_robustness.md
[PP]: ../research_notes/Izcal%20extensions/permissions_prompts.md
[PV]: ../research_notes/Izcal%20extensions/privacy.md
[DS]: ../research_notes/Izcal%20extensions/distribution_signing.md
[GO]: ../research_notes/Izcal%20extensions/gaps_opportunities.md
