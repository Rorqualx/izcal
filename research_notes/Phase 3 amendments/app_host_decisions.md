# Phase 3, batch 6: record of the owner's decisions of 21 October 2026

Written 21 October 2026. Most of the work lands on branch `spec/phase2c`; the security-scope amendments land on `spec/security-scope` and the coding-standard amendments on `spec/cs-amendments`, each in its own worktree and published when that branch merges. This file records what the owner decided on 21 October 2026 and where each decision is applied. It is a decision record, not a research note: it cites the documents it changes, and every change it names carries its own dated revision line in the document it lands in.

## 1. Every batch-6 recommendation is accepted

The owner accepted, in full and as written, every recommendation in `reports/Izcal app host.md` §9.4, **AH-D1 to AH-D17** (the spec is now draft 0.3, the corpus's 24th specification). The decision section carries the stamp "**Owner decision (21 October 2026): every recommendation below is accepted**", in the form `reports/Izcal JavaScript engine.md` §9.4 uses, and the spec carries a dated Revision 0.3 line (0.2 → 0.3, a notable revision under `CONTRIBUTING.md`). Notable consequences:

- **AH-D1** runs apps as **the component host's app configuration** — no new process kind: the security scope stays at **37 kinds and 39 threat-model files**, and `component-host.md` gains an app-profile section with one subsection per parser family (package/ZIP, component, WIT, canonical ABI). A `--replay` and an install-verify configuration join the App configuration.
- **AH-D2** makes the app id a **manifest-carried random `app_id`, signed by the author key and kept across key rotation by a two-key succession record**; `app://<id>` is a non-web identity with no web origin and no cookies.
- **AH-D3** is the capability model: **manifest ceiling + computed imports + broker-signed tickets**, with **install as a grant event** computed by a tier-P install verifier so the broker never parses guest bytes.
- **AH-D4** pins the profile to **WASI 0.2 worlds at 0.2.12** — cli, clocks, random, filesystem, http client and io; **no sockets, no incoming, no threads, no relaxed SIMD** — with 0.3 async as a named reversal trigger re-decided at AH-M2.
- **AH-D5** is **one component per package, host interfaces only, canonical ABI complete**; no runtime composition, publishers vendor dependencies. **AH-D6** is the **sealed HTML UI in an own-origin renderer plus typed intents** — no display protocol — which is two sandboxes per windowed app.
- **AH-D7** shares **the extensions directory, signing service and review process**: two signatures, mechanical capability-diff review, no telemetry and uniform updates. **AH-D8** is the exposure gate: **off by default behind the product gate; a named list of at most 20† apps that may install or launch, developer mode included; per-capability owner exceptions expiring after 12 months†; no web API.**
- **AH-D9** records host calls against a **virtual filesystem and network plus recorded clocks and entropy**; replay opens nothing. **AH-D10** sends app HTTP only through the **app's own partition**, with no cookies and no content blocking on app traffic, and names AH-18 (declared destinations) as the reversal trigger.
- **AH-D11** reuses **DV-D2/DV-D4 unchanged** with a manifest family ceiling; **AH-D12** is the **app bucket plus picked-directory tickets through the storage service**; **AH-D13** gives agents **intents only, per-task grants and person-present writes**.
- **AH-D14** keeps **Baseline (W0/W1) default, Full (W2) optional, Jitless available, tier-P builds**. **AH-D15** makes the app host a **Stage 3 deliverable with Stage 2 seams, starting at JS-M4**. **AH-D16** limits review claims to **capability review, not code audit** — the sandbox is the boundary. **AH-D17** publishes the **package layout, manifest schema and `izcal:app@0.1` profile as an open, versioned MIT specification**; the host implementation stays Izcal's.
- **The nine owner-facing decisions** are flagged for attention in the spec's 9.4 and this record: **AH-D1 to AH-D8 and AH-D17** — every starred decision is in this group. The rest are engineering decisions the spec made and the owner reviewed.

## 2. The WASI door closes

The door had three parts: HGBA-2's X-3 row offered native Component Model support as optional and said nothing about running a program; Stage 1's out-of-scope bullet refused "the browser as a window manager for local apps"; and Stage 1's Stage 3 backlog row offered "native Component Model (optional)" with nothing behind it. The acceptance closes all three on 21 October 2026:

- **HGBA-2 2.4.3 (A1; A2 subsumed).** The X-3 row now says native CM counts toward the "two engines" gate **for web exposure only**, names **the app host (*Izcal app host*, AH-D15) as the Stage 3 deliverable that consumes this engine work**, and states that components executed inside the app host have no web surface and do not count toward the gate. **A2 proposed a second replacement of the same row; it is recorded as subsumed by A1 and no second edit is applied.**
- **Stage 1 0.14.4 (A4), the narrowed refusal.** The out-of-scope bullet now reads: **"the browser as a window manager for local apps"** — a refusal the section itself scopes to **Stages 1 and 2** — narrowed to **arbitrary native local apps** (native binaries, OS processes and ambient-authority applications), not to every way of running a program, with the parenthetical: "(Stages 1–2. A Stage 3 app host runs signed WASI components in browser-owned windows under the broker's consent — *Izcal app host*, AH 6.15; the refusal's object, arbitrary native local apps, is unchanged.)"
- **Stage 1 0.14.4 (A5, A31), the promotion.** The Stage 3 backlog row becomes "native Component Model (optional) **and the app host (AH-M0–AH-M5; *Izcal app host*)**", with the staffing note — **15–21† engineers**, **16–39† incremental engineer-years**, **2–4† non-engineers** — and the directory and signing tracks shared with extensions. The door's closure is therefore a promotion with an owner (named at AH-M0), a gate (**AH-D8**, at most 20† named apps), a cost and an exit audit (**AH-M5**: a corpus of ≥ 25† apps, of which ≥ 10† are from authors other than Izcal), not a new permission: **no app model is added to Stages 1–2.**

## 3. How the batch lands

- **The app host edits no other file itself.** Every §9.5 application lands in its target document with its own dated revision line: the security-scope rows on `spec/security-scope`, the coding-standard rows on `spec/cs-amendments`, and the rest on `spec/phase2c`. The source spec carries the decision stamp, its Revision 0.3 line (status `draft 0.3, owner decisions accepted`), and §9.5's Status column reads **applied, 21 October 2026** on all 31 rows.
- **Where an earlier batch already changed a target, the amendment was integrated, not appended twice.** Shell 0.3.5, DevTools 0.3.7, footprint 0.4.5, DOM 0.2.6 and accessibility 0.2.6 build on the devices and phase-3 revisions already in those files, and each file's current text was read before editing. HGBA-2's X-3 row was replaced once (A1), with A2 recorded beneath it; Stage 1's out-of-scope bullet and Stage 3 backlog row were edited in place (A4, A5), on top of 0.14.3.
- **One process kind in every count.** The app configuration is not a new kind: the security scope's map stays at 37 kinds / 39 files, and the app-profile section lives inside `component-host.md`. **G220–G229 are registered** for the app host in Stage 1 2.2 and the coding standard; G230 onward belongs to the next specification.

## 4. Where the accepted decisions are applied

With the source decisions accepted, the §9.5 amendments follow from them. Applied on 21 October 2026, each with its own dated revision line:

| Document | Version | What landed |
|---|---|---|
| `reports/Izcal app host.md` | 0.3 | The owner stamp, the Revision 0.3 line, §9.5's applied note and statuses, and the decision record link (AH-D1–AH-D17) |
| `reports/HGBA 2 browser architecture spec.md` | 2.4.3 | A1/A2: the X-3 row replaced once, A2 recorded as subsumed; A3: the M-1 compliance record and the component/WASI suites as test-only oracles |
| `reports/HGBA Stage 1 architecture design.md` | 0.14.4 | A4: the narrowed, time-scoped refusal; A5/A31: the Stage 3 backlog row and staffing; A6: the app-host crates and layers; A7: G220–G229; A8: the Stage 2 seams |
| `reports/Izcal security scope.md` (`spec/security-scope`) | 0.3.8 | A9: the app, app-replay and install-verify configurations and the app-profile threat-model section; A10: `app.intent`, `app.log`, `app.control` and `app.invoke` with their rules; A11: ten `app-*` prompt classes and the A-SH-12 extension; A12: REC-R8; A13: AGT-R3/AGT-R6; A14: SEC-18/SEC-19 |
| `reports/Izcal coding standard.md` (`spec/cs-amendments`) | 0.3.7 | A15: G220–G229 registration and the tier-P app crates; A16: CS-ST-10's app-configuration ceiling rules |
| `reports/Izcal JavaScript engine.md` | 0.4.4 | A17: the app, `--replay` and `--verify` configurations; A18: G6/G14 and the jitless closure; A19: JS-D13's app tier ceilings |
| `reports/Izcal devices.md` | 0.3.1 | A20: DV-D2's app-id ticket, DV-D4's chooser principal, DV-D8's replay, DV-D9's template |
| `reports/Izcal storage.md` | 0.3.3 | A21: SG-D4's app bucket, the directory grant, the five path-resolution rules, G150 |
| `reports/Izcal networking.md` | 0.3.8 | A22: NW-D4's app-id LNA tickets, NW-D7's app partition tuple, app HTTP out of content blocking |
| `reports/Izcal extensions.md` | 0.4.2 | A23: the shared directory, signing service and review process, the mechanical capability diff and EX-D12 |
| `reports/Izcal shell.md` | 0.3.5 | A24: `shell.apps`, app windows and their chrome, the ledger's app entries and SH-D11 |
| `reports/Izcal DevTools spec.md` | 0.3.7 | A25: the `app` IIP domain and DT-D5; the `app.wasi` class and the two-record join in DT-D2 |
| `reports/Izcal phishing protection.md` | 0.3.1 | A26: 6.4.9's app window and the no-chrome-imitation review refusal |
| `reports/Izcal resource footprint.md` | 0.4.5 | A27: FP-1's exclusion and two charged lines, FP-D8's app origins and FP-D9's no-app-ids rule |
| `reports/Izcal content blocking.md` | 0.3.5 | A28: app HTTP out of scope, with AH-D10's reason and AH-18's reversal trigger |
| `reports/Izcal accessibility.md` | 0.2.6 | A29: 6.12's app surfaces |
| `reports/Izcal DOM, bindings and event loop.md` | 0.2.6 | A30: 6.9's app navigation rules |
| `research_notes/Phase 2 reconciliation/licence_audit.md` | updated 21 October 2026 | A31 part 1: the Component Model WAST suite and `wasi-testsuite` as Apache-2.0 test-only oracles |
| `research_notes/Phase 3 amendments/ledger.md` | updated 21 October 2026 | A31 part 2: the Stage 3 plan; G220–G229; the C-6 addendum 5; A-AH-1 to A-AH-31 |
| `README.md` | updated 21 October 2026 | The app host spec added (draft 0.3, decisions accepted); the changed revisions bumped (HGBA-2 2.4.3, Stage 1 0.14.4, DevTools 0.3.7, JavaScript engine 0.4.4, footprint 0.4.5, networking 0.3.8, content blocking 0.3.5, DOM 0.2.6, accessibility 0.2.6, storage 0.3.3, extensions 0.4.2, shell rev 0.3.5, phishing 0.3.1, devices 0.3.1) |
| `research_notes/Phase 3 amendments/app_host_decisions.md` | this file | The acceptance, the door closure, the application map, staffing and the items left open |

## 5. What is left open

- **Q-AH-1 to Q-AH-9 stay open**, with the owners and needed-by points in the app host spec's Section 11:
  - **Q-AH-1** — the measured canonical-ABI overhead per type class (AH-M0; feeds AH-7 and the 0.3 decision).
  - **Q-AH-2** — whether any browser team has an unstated WASI-in-the-browser position that would change the two-engine gate.
  - **Q-AH-3** — whether the storage service's file path can meet WASI filesystem semantics, or needs a WASI-specific view (spike at AH-M0).
  - **Q-AH-4** — the smallest set of apps that proves the profile (AH-6's 25†).
  - **Q-AH-5** — whether the directory needs domain-bound identity to prevent impersonation; no user study is planned.
  - **Q-AH-6** — how an app declares a default file handler without ambient authority (a `file-handler` intent plus a per-type grant is proposed; OS/desktop integration is out of scope).
  - **Q-AH-7** — whether a per-intent CPU deadline suffices for legitimately long service apps, or a background budget class is needed (measure on the AH-6 corpus; decide at AH-M3).
  - **Q-AH-8** — whether the WAST suite covers canonical-ABI transcoding well enough to be the conformance bar (failure action is AH-5's).
  - **Q-AH-9** — whether the package signature moves to a standard envelope (Sigstore bundle or OCI referrer) if one emerges, or stays Izcal's published format (W3C WASI subgroup and Bytecode Alliance watch).
- **AH-D8's named-app gate and per-capability owner exceptions stand as written.** The named list admits at most 20† apps to install or launch, developer mode included; each per-capability owner exception **expires after 12 months†**. The acceptance grants no exception beyond these, and AH-M5 reviews them.
- **The owner-facing decisions are accepted, not left open.** AH-D1 to AH-D8 and AH-D17 are flagged because they carry product, cost, identity or governance consequences; their reversal triggers are stated in the 9.4 table and the ledger records them as accepted on 21 October 2026.

## 6. Staffing, as the batch recomputes it

The running total (ledger C-6, addendum 5 of 21 October 2026): C-6's full-strength figure after devices is about **613–809† engineers†**. Batch 6 adds the app host's **15–21† engineers at peak** (the app host spec's 9.3, five roles, the runtime track's 6–9† overlapping the JavaScript engine's Wasm team by 2–3†), taking the full-strength total to about **628–830 engineers†**; with the **7.5–14.5† non-engineers** (5.5–10.5† before this batch plus the app host's 2–4†), the corpus total is about **635–845† people†**. The track's **22–42† engineer-years** at 18–24 months† include the 2–3† shared engineers, so the **incremental figure is 16–39† engineer-years**; the non-engineering share is a separate **3–8† staff-years**. The **Stage 1 8.3 total is unchanged at 67.5–96.5† engineers** with 5.5–10.5† non-engineers: the app host is a Stage 3 deliverable and adds nothing to Stage 1.
