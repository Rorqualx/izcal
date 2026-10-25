# Izcal feature inventory audit: final ledger

Compiled 24 October 2026 by the synthesis author from the four inventories, the four maps and the five checks in this folder. The counts at the end are computed by a script from the table rows below, not by hand. The planning document that uses this ledger is `reports/Izcal feature inventory audit.md`.

## How to read this

**One row per inventory row (1,117 rows).** Each row has exactly one final status:

| Status | Meaning | Gap? |
|---|---|---|
| **Designed** | A spec specifies Izcal's behaviour or architecture for the feature | No (covered) |
| **Owned** | A spec names the feature, or a standard that contains it, as in scope, with an owner and a milestone, floor or target; it is built from the standard as written | No (covered) |
| **Held** | Single-engine or unstable, held by a recorded admission rule (LY-D10, ST-D18, DM 6.3, DM 9.1, AH 6.15) or a per-feature hold | No, but rows marked "rule only" have no per-feature record (FI-37) |
| **Refused** | A recorded decision or non-goal says no; "(implicit)" means a general rule covers it without naming it | No; implicit refusals should be written down (FI-38) |
| **Deferred** | A later stage, with an owner spec | No |
| **Deferred, no owner** | A later stage is named, but no spec owns the feature | Yes, for later |
| **Mentioned** | Named or implied, with no owner, milestone, floor or design | Yes |
| **Partial** | Partly designed; the missing part is in the map's evidence cell | Yes, for the missing part |
| **Missing** | No spec mentions it | Yes |

**Override order.** A later check replaces an earlier verdict: map < `verify_missing_bf_po.md` (**VMB**) < `verify_missing_wa_cg.md` (**VMW**) < `verify_covered_excluded.md` (**VCE**) < `regrade_bf_po.md` (**RBP**) < `regrade_covered_wa_cg.md` (**RWC**). `factcheck.md` (**FC**) changes no status; where it corrects a row's framing or a licence fact, the row says so. The "Source of verdict" column shows every verdict the row received, in order.

**Mapping rules the synthesis applied.**

- Map **Covered** rows that no check re-graded do not exist: RBP and RWC re-graded every Covered row. Map **Partial** rows were not re-checked and stay Partial.
- VMW's **Held (rule)** rows are Held. On the stricter reading (every hold needs its own ledger entry) they would be Missing; they carry "rule only" in the owner column and FI-37 turns them into records.
- VMB's "Actually partial" rows that are named only (BF-4.10, BF-6.11, PO-3.7) and VMW's WA-12.10 are **Mentioned**. VMB's "Actually excluded" rows (BF-17.7, PO-1.2, PO-8.14, PO-9.22) are **Deferred, no owner**, because each names a later stage or a reversal trigger that no spec owns.
- VMW's "Actually covered" rows (WA-1.18, WA-2.14) are **Owned**: ST 9.1 implements the pinned parser behaviour as written.
- VCE's verdicts on the WA and CG Excluded rows are final, because no later check covers them. Confirmed deferrals are **Deferred**, confirmed admission holds are **Held**, and confirmed refusals are **Refused**. CG's "Deferred (admission)" rows are **Held**.
- RBP's "Refused (implicit)" is **Refused** with "(implicit)" in the owner column.
- **Two synthesis adjustments (SYN)** apply VMW's admission-rule reading to rows VMW never saw because the map filed them Excluded: WA-4.6 interest invokers (Chromium-only) and WA-14.16 IP Protection (retired, Chromium-only) are **Held**, not Missing as VCE has them. Each row says so.
- **→ FI-n** in the owner column names the work item in the planning document that closes the row.

**Tags.** [SB] ship blocker (PO), [TS] table stakes (BF), [Diff] differentiator (BF), [1E] single-engine (WA map).

## WA: Web platform APIs


### WA 1. HTML: documents, browsing contexts, navigation and history

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-1.1 | Document and browsing-context model | **Designed** | DM 6.8, 6.4 (`hg-agents`, DM-D6, DM-D7) | DM-M2; DM-1 `html/browsers/` 95% | map: Covered; RWC: Designed |
| WA-1.2 | Navigation and session history | **Designed** | DM 6.9 (`hg-navigation`, DM-D20) | DM-M2; DM-1 `html/browsers/` 95% | map: Covered; RWC: Designed |
| WA-1.3 | Navigation API | **Designed** | DM 6.9 (`hg-window` renderer, broker commits) | DM-M2 names it; DM-4; DM-1 `navigation-api/` 95% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-1.4 | Back/forward cache (bfcache) behaviour | **Designed** | DM 6.9 (DM-D9, DM-D14), DM 6.12 exception | DM-M2; DM-28, DM-31 | map: Covered; RWC: Designed |
| WA-1.5 | Page Visibility and Page Lifecycle | **Designed** | DM 6.14 (FP-D4), DM 6.9; DM-D15 exception | DM-M2/M3 | map: Covered; RWC: Designed |
| WA-1.6 | `beforeunload` / `unload` | **Designed** | DM 6.9 (DM-D9) | DM-M2; DM-1 `html/browsers/` 95% | map: Covered; RWC: Designed |
| WA-1.7 | iframes and embedding | **Designed** | DM 6.12 (`hg-html`), DM 6.9; LY 6.15 | DM-M2/M3; DM-1 `html/semantics/` 97% | map: Covered; RWC: Designed |
| WA-1.8 | Credentialless iframes [1E] | **Partial** | NW 6.8 (nonce n), NW 6.7 (COEP `credentialless`), ST 6.7 **→ FI-12** | — | map: Partial |
| WA-1.9 | Fenced frames [1E] | **Partial** | NW 6.8 (nonce n) only **→ FI-12** | — | map: Partial |
| WA-1.10 | `Location.ancestorOrigins` | **Owned** | DM 6.3 corpus, DM-D15; `hg-window` (DM 6.17) | DM-M2 (`Location`); DM-1 `html/browsers/` 95% | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-1.11 | Origin API | **Mentioned** | — (HTML claim only) **→ FI-14** | — | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins). *HTML pin date unrecorded* |
| WA-1.12 | Text fragments | **Designed** | ED 6.12 (`hg-find`, ED-D17) | ED-M2 | map: Covered; RWC: Designed |
| WA-1.13 | Top-navigation restrictions | **Owned** | DM 6.9 (broker sandbox and history-action checks, DM-D20) | DM-M2; DM-1 `html/browsers/` 95% (sandboxing) | map: Covered; RWC: Owned |
| WA-1.14 | Initial `about:blank` behaviour | **Owned** | DM 6.9 (renderer `hg-window`, "Document creation and initialization") | DM-M2; DM-1 `html/browsers/` 95% | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-1.15 | Printing | **Designed** | LY 6.7 (LY-D8); GR 6.27 | LY-M2 paged media; LY-1 `css-page` 90%; print UI Stage 2 | map: Covered; RWC: Designed |
| WA-1.16 | Simple dialogs | **Mentioned** | DM 6.7 (`hg-script` pause); rendering assigned to shell, unclaimed **→ FI-5** | DM-1 `html/webappapis/` 97% (DOM side only) | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins) |
| WA-1.17 | Window and screen properties | **Designed** | DM 6.16, 6.18 (broker environment-values record, D-S1, OD-6) | DM-M2 | map: Covered; RWC: Designed |
| WA-1.18 | Out-of-order HTML streaming [1E] | **Owned** | — | — | map: Missing; VMW: Actually covered |
| WA-1.19 | XSLT and `XSLTProcessor` | **Refused** | ST-D4; DM-D15 | — | map: Excluded; VCE: Confirmed → Refused |

### WA 2. HTML: elements, attributes and document metadata

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-2.1 | HTML parser and serializer | **Designed** | ST 6.2–6.5 (`hg-html-syntax`, `hg-html-driver`, ST-D1–D3) | ST-M1; ST-1 100% html5lib/tree-construction; ST-2 | map: Covered; RWC: Designed |
| WA-2.2 | Sectioning, grouping and text-level elements | **Owned** | DM 6.12 (`hg-html`) | DM-M3 full element set; DM-1 `html/semantics/` 97% | map: Covered; RWC: Owned |
| WA-2.3 | Heading level adjustment [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| WA-2.4 | Links and `<a>` behaviours | **Owned** | DM 6.12 (`hg-html`); CB 6.2; NW 6.14 | DM-M2/M3; DM-1 `html/semantics/` 97% (links) | map: Covered; RWC: Owned |
| WA-2.5 | `<link>` relation types | **Owned** | DM 6.12 (`hg-html`); NW 6.3, 6.11; ST 6.6a | DM-M3; NW-M2 | map: Covered; RWC: Owned |
| WA-2.6 | `<meta>` names and pragmas | **Owned** | DM 6.12 (`hg-html`); NW 6.7; DM 6.9 | DM-M3; DM-1 `html/semantics/` 97% (document-metadata) | map: Covered; RWC: Owned |
| WA-2.7 | `<template>` and `<slot>` | **Designed** | DM 6.6, 6.12 (`hg-dom`) | DM-M1; DM-1 `shadow-dom/` 97% | map: Covered; RWC: Designed |
| WA-2.8 | Responsive images markup | **Owned** | DM 6.12 (selection in `hg-dom`); MD 6.20 | DM-M3; DM-1 `html/semantics/` 97% (embedded-content) | map: Covered; RWC: Owned |
| WA-2.9 | Lazy loading | **Designed** | NW 6.3; LY 6.15 (`hg-layout-query` lazy-load observer) | NW-M2; LY-M2; DM-1 `html/semantics/` 97% | map: Covered; RWC: Designed |
| WA-2.10 | `<details>` and `<summary>` | **Owned** | DM 6.12 (`hg-html`); ED 6.12 | DM-M2; DM-4 `details` area 99% | map: Covered; RWC: Owned |
| WA-2.11 | `hidden="until-found"` | **Designed** | ED 6.12 (`hg-find`); AX-O11 | ED-M2 | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-2.12 | Global attributes | **Owned** | DM 6.5, 6.12; ED 6.8, 6.13 (ED-D18) | DM-M2; ED-M1/M2; DM-1 `html/dom/` 97% | map: Covered; RWC: Owned |
| WA-2.13 | `autocorrect` attribute | **Designed** | ED 6.8 (ED-D19), 6.13 | ED-M3 | map: Covered; RWC: Designed |
| WA-2.14 | Processing instructions in HTML [1E] | **Owned** | — | — | map: Missing; VMW: Actually covered |
| WA-2.15 | Permission and capability elements | **Held** | S1 13.2 Permissions; SHUI 4.4 (adopted once standardized) | When standardized | map: Excluded; VCE: Confirmed → Held |
| WA-2.16 | `<model>` element [1E] | **Held** | — — rule only, no per-feature record **→ FI-36** | — | map: Missing; VMW: Held (rule) |
| WA-2.17 | MathML Core | **Designed** | LY 6.5.9 (`hg-layout-math`, LY-D21); DM 6.12 | LY-M2; LY-1 `mathml` 90% | map: Covered; RWC: Designed |
| WA-2.18 | Obsolete and removed features | **Designed** | DM 6.12 (DM-D15); ST-D4 | — | map: Covered; RWC: Designed |

### WA 3. HTML: forms and form controls

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-3.1 | Form submission | **Owned** | DM 6.12 (`hg-forms`) | DM-M2 forms; DM-1 `html/semantics/` 97% (forms) | map: Covered; RWC: Owned |
| WA-3.2 | Input types | **Owned** | DM 6.12 (`hg-forms`); LY-D11 | DM-M2; DM-1 `html/semantics/` 97% | map: Covered; RWC: Owned |
| WA-3.3 | Constraint validation | **Owned** | DM 6.12 (`hg-forms`) | DM-M2 "forms and validation"; DM-1 `html/semantics/` 97% | map: Covered; RWC: Owned |
| WA-3.4 | `<select>`, `<option>`, `<datalist>`, `<textarea>`, `<output>`, `<progress>`, `<meter>`, `<fieldset>` | **Designed** | DM 6.12 (`hg-forms`); LY 6.3 (LY-D11) | DM-M2; LY-M2 | map: Covered; RWC: Designed |
| WA-3.5 | Customizable `<select>` | **Partial** | DM 6.12 (`selectedcontent`; forms track "select and customizable select"); AX-O12; LY-D10 holds `appearance: base-select` **→ FI-37** | — | map: Partial. *LY-D10 holds `base-select`, but Chrome 135 and Safari 27.0 ship it, so the hold's trigger looks met* |
| WA-3.6 | `showPicker()` | **Mentioned** | — (HTML claim; LY-D11 says pickers are browser-process UI) **→ FI-5** | — | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins) |
| WA-3.7 | Autofill and `autocomplete` tokens | **Designed** | DM 6.12; ED 6.6 (autofill door); SS CON-R4 | Stage 1 passwords; Stage 2 beyond (S1 13.2) | map: Covered; RWC: Designed |
| WA-3.8 | Directory upload | **Missing** | — **→ FI-5** | — | map: Missing; VMW: Confirmed missing |
| WA-3.9 | HTML Media Capture | **Missing** | — **→ FI-5** | — | map: Missing; VMW: Confirmed missing |
| WA-3.10 | `<input type=checkbox switch>` [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| WA-3.11 | Color input `alpha` and `colorspace` | **Mentioned** | — (HTML claim; LY-D11 picker) **→ FI-5** | — | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins) |
| WA-3.12 | Form-associated custom elements | **Owned** | DM 6.6, 6.12 (`hg-forms`, `ElementInternals`) | DM-M2; DM-1 `custom-elements/` 97% | map: Covered; RWC: Owned |
| WA-3.13 | OpaqueRange [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |

### WA 4. HTML: interactive UI, focus, dialogs and drag and drop

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-4.1 | `<dialog>` | **Owned** | DM 6.12 (`hg-html`, top layer) | DM-M2 dialog; DM-4 dialogs area 99% | map: Covered; RWC: Owned |
| WA-4.2 | Dialog light dismiss (`closedby`) and `requestClose()` | **Owned** | DM 6.12 (`hg-html`) | DM-M2; DM-4 dialogs area 99% | map: Covered; RWC: Owned |
| WA-4.3 | Popover API | **Owned** | DM 6.12 (`hg-html`) | DM-M2 popover; DM-4 popovers area 99% | map: Covered; RWC: Owned |
| WA-4.4 | `popover="hint"` | **Owned** | DM 6.12 (`hg-html`) | DM-M2; DM-4 | map: Covered; RWC: Owned |
| WA-4.5 | Invoker commands | **Owned** | DM 6.12 (`hg-html`) | DM-M2; DM-4 | map: Covered; RWC: Owned |
| WA-4.6 | Interest invokers | **Held** | — (AX-O13 maps it only "while admitted"; no decision holds `interestfor`) — rule only, no per-feature record **→ FI-37** | When admitted | map: Excluded; VCE: Misread → Missing; SYN: Held (Chromium-only (inventory: 142 / N / N); VMW's admission-rule reading holds single-engine rows, and VMW did not see this one because the map filed it Excluded) (checks differ; the later check wins) |
| WA-4.7 | `CloseWatcher` | **Owned** | DM 6.3 corpus; DM 6.12 light dismiss | DM-M2 (top layer) | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-4.8 | Light-dismiss input timing change | **Owned** | DM 6.12 (`hg-html`) | DM-M2; DM-4 popovers area | map: Covered; RWC: Owned |
| WA-4.9 | Focus management | **Designed** | DM 6.5 (`hg-html::focus`); AX 6.6 (`hg-ax-focus`) | DM-M2 "input, focus and activation" | map: Covered; RWC: Designed |
| WA-4.10 | `inert` | **Owned** | DM 6.5 (`hg-html::focus`) | DM-M2 focus | map: Covered; RWC: Owned |
| WA-4.11 | Focusgroup [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| WA-4.12 | User activation | **Designed** | DM 6.5 (DM-D20) | DM-M2 | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-4.13 | HTML drag and drop | **Designed** | ED 6.11 (`hg-dnd`, ED-D13) | ED-M2 | map: Covered; RWC: Designed |
| WA-4.14 | Fullscreen API | **Designed** | DM 6.7 (element side; broker window side), 6.18 (`hg-webapi`); MD 6.12; shell 6.8 | SH-M1 fullscreen; DM-M2 | map: Covered; RWC: Designed |
| WA-4.15 | Pointer Lock | **Missing** | — **→ FI-28** | — | map: Missing; VMW: Confirmed missing |
| WA-4.16 | Keyboard Lock and Keyboard Map [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-4.17 | `ariaNotify()` | **Designed** | AX 6.7; DM 6.18 | AX milestones | map: Covered; RWC: Designed |
| WA-4.18 | Editing hooks | **Designed** | ED 6.2 (ED-D1), ED-D2 | ED-M2 | map: Covered; RWC: Designed |

### WA 5. HTML: scripting, modules and resource loading

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-5.1 | Classic scripts | **Designed** | DM 6.12 (`hg-script`); ST 6.4 (ST-D2); JS 6.14 | DM-M2; ST-M1; DM-1 `html/semantics/` 97% | map: Covered; RWC: Designed |
| WA-5.2 | Module scripts | **Designed** | JS 6.14; DM 6.18 (`hg-script`) | JS-M1 ES modules | map: Covered; RWC: Designed |
| WA-5.3 | JSON and CSS module scripts | **Owned** | JS 6.14; DM 6.3 corpus | JS-M2; DM-1 `html/semantics/` 97% (scripting-1) | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-5.4 | Text module imports | **Owned** | JS 6.14 (loader names text modules); JS-1, JS-70 | When TC39 Stage 4 (stage unrecorded) | map: Excluded; VCE: Misread → Owned |
| WA-5.5 | Import maps | **Owned** | DM 6.18 (`hg-script`); JS 6.14 | DM-1 `html/semantics/` 97% (import-maps) | map: Covered; RWC: Owned |
| WA-5.6 | `modulepreload` | **Owned** | DM 6.12 (`hg-html` link) | DM-M3 | map: Covered; RWC: Owned |
| WA-5.7 | Preload and resource hints | **Owned** | DM 6.12; NW 6.3, 6.11; ST 6.4 | NW-M2; DM-M3 | map: Covered; RWC: Owned |
| WA-5.8 | Speculation Rules | **Designed** | DM 6.9 (prerendering), DM 6.7; NW 6.11 | DM-M3 (S1 M7) vs S1 §9 Stage 2 conflict | map: Covered; RWC: Designed. *contradiction: DM-M3 has prerender in Stage 1, S1 §9 says Stage 2* |
| WA-5.9 | No-Vary-Search | **Missing** | — **→ FI-16** | — | map: Missing; VMW: Confirmed missing |
| WA-5.10 | Render-blocking | **Owned** | ST 6.6a; DM 6.7 (render-blocked filter) | ST-M1; DM-1/ST-2 `html/dom/` 97% | map: Covered; RWC: Owned |
| WA-5.11 | Web bundles and signed exchanges [1E] | **Held** | — — rule only, no per-feature record **→ FI-16** | — | map: Missing; VMW: Held (rule) |
| WA-5.12 | Integrity Policy | **Missing** | — **→ FI-16** | — | map: Missing; VMW: Confirmed missing |
| WA-5.13 | Signature-based integrity [1E] | **Held** | — — rule only, no per-feature record **→ FI-16** | — | map: Missing; VMW: Held (rule) |

### WA 6. DOM, Web Components and DOM-adjacent APIs

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-6.1 | DOM core | **Designed** | DM 6.1 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | map: Covered; RWC: Designed |
| WA-6.2 | Events | **Designed** | DM 6.5 (`hg-dom` events core) | DM-M1; DM-1 `dom/` 97% | map: Covered; RWC: Designed |
| WA-6.3 | Aborting | **Owned** | DOM claim (DM 6.3, DM 4.2); `hg-dom` | DM-M1; DM-1 `dom/` 97% (`dom/abort/`) | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-6.4 | MutationObserver | **Designed** | DM 6.6, 6.1 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | map: Covered; RWC: Designed |
| WA-6.5 | Ranges, traversal and `StaticRange` | **Designed** | DM 6.1, 6.6 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | map: Covered; RWC: Designed |
| WA-6.6 | `moveBefore()` | **Designed** | DM 6.1 (`hg-dom`) | DM-M1; DM-1 `dom/` 97% | map: Covered; RWC: Designed |
| WA-6.7 | Shadow DOM | **Designed** | DM 6.6 (`hg-dom`) | DM-M1; DM-1 `shadow-dom/` 97% | map: Covered; RWC: Designed |
| WA-6.8 | Declarative shadow DOM | **Designed** | ST 6.4–6.5 (`hg-html-syntax`); DM 6.6 | ST-M1; ST-2 `shadow-dom/declarative` 99% | map: Covered; RWC: Designed |
| WA-6.9 | Custom elements | **Designed** | DM 6.6 (`hg-dom`) | DM-M1; DM-1 `custom-elements/` 97% | map: Covered; RWC: Designed |
| WA-6.10 | Scoped custom element registries | **Designed** | DM 6.6 (`hg-dom`) | DM-M1; DM-4 Interop; DM-1 `custom-elements/` | map: Covered; RWC: Designed |
| WA-6.11 | Reference Target [1E] | **Partial** | AX 6.2 (IDREF resolution); DM owns the resolver **→ FI-37** | — | map: Partial |
| WA-6.12 | ARIA reflection | **Designed** | DM 6.18, 6.3 (reflection codegen); AX 6.2 (IDREF resolver in `hg-dom`) | DM-M2; DM-1 `html/dom/` 97% (aria reflection tests) | map: Covered; RWC: Designed |
| WA-6.13 | DOM Parsing and Serialization | **Designed** | ST 6.4–6.5 (`FragmentSink`, serializer) | ST-M1; ST-2 `domparsing` ≥ best engine; ST-8 | map: Covered; RWC: Designed |
| WA-6.14 | HTML Sanitizer API | **Designed** | ST 6.5 (`SanitizingSink`); DM 6.16 | ST-M1; ST-2 `sanitizer-api` ≥ best engine; ST-8(c) | map: Covered; RWC: Designed |
| WA-6.15 | XPath and XML documents | **Partial** | DOM claim (DM 6.3 corpus); ST (XML parsing) **→ FI-14** | — | map: Partial |
| WA-6.16 | Selection API | **Designed** | DM 6.6 (data model); ED 6.4 (`hg-caret`) | DM-M2; ED-M1; DM-1 `selection/` 95% | map: Covered; RWC: Designed |
| WA-6.17 | UI Events | **Designed** | DM 6.5 (`hg-input`) | DM-M2; DM-1 `uievents/` 95% | map: Covered; RWC: Designed |
| WA-6.18 | Pointer Events | **Designed** | DM 6.5 (`hg-input`) | DM-M2 ("input"); no `pointerevents/` floor | map: Covered; RWC: Designed |
| WA-6.19 | Touch Events | **Designed** | DM 6.5 (`hg-input`, `hg-dom`) | DM-M2 ("input"); no `touch-events/` floor | map: Covered; RWC: Designed |
| WA-6.20 | Ink API | **Held** | GR 6.9, 9.1 (LY-D10's process) | When admitted | map: Excluded; VCE: Confirmed → Held |
| WA-6.21 | Observers | **Designed** | LY 6.15 (`hg-layout-query`) | LY-M2; LY-1 `resize-observer/`, `intersection-observer/` 95% | map: Covered; RWC: Designed |
| WA-6.22 | Scrolling and caret APIs | **Designed** | LY 6.15 (`hg-layout-query`, `hg-scroll`); DM 6.7 | LY-M1; LY-1 `cssom-view` 95%; DM-4 (`scrollend`) | map: Covered; RWC: Designed |
| WA-6.23 | Observable API [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| WA-6.24 | View Transitions, script side | **Designed** | AN 6.x (`hg-view-transition`, `hg-anim-api`); DM 6.7 | AN-M2, AN-M3; AN-1 `css/css-view-transitions` 92%; AN-2 | map: Covered; RWC: Designed |
| WA-6.25 | Geometry interfaces | **Mentioned** | SV 6.x (Geometry Interfaces 1 pinned as input); DM 6.18 "geometry APIs" to layout **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-6.26 | Visual Viewport | **Mentioned** | LY 6.1, LY-D13; GR 6.28 (compositor mechanics) **→ FI-14** | — | map: Covered; RWC: Mentioned |

### WA 7. Event loop, timers, scheduling, workers and messaging

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-7.1 | Event loop, tasks and microtasks | **Designed** | DM 6.7 (`hg-script`, DM-D1) | DM-M1; DM-1 `html/webappapis/` 97%; DM-22, DM-24 | map: Covered; RWC: Designed |
| WA-7.2 | Timers and animation frames | **Designed** | DM 6.7 (DM-D10, AN-D3) | DM-M1; DM-14; DM-1 `html/webappapis/` | map: Covered; RWC: Designed |
| WA-7.3 | `requestIdleCallback` | **Designed** | DM 6.7 (`hg-script`) | DM-M1 (rIC named) | map: Covered; RWC: Designed |
| WA-7.4 | Prioritized Task Scheduling | **Designed** | DM 6.7 (`hg-script`) | DM-M1 (`postTask` named) | map: Covered; RWC: Designed |
| WA-7.5 | `isInputPending` | **Refused** | DM 6.7; DM-D15 | — | map: Excluded; VCE: Confirmed → Refused |
| WA-7.6 | Error reporting | **Owned** | HTML claim (DM 6.3, DM-D15); DM 6.7 (`hg-script`) | DM-M1; DM-1 `html/webappapis/` 97% | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-7.7 | Dedicated workers | **Designed** | DM 6.10 (`hg-workers`, `hg-agents`) | DM-M2; DM-1 `workers/` 95%; DM-26 | map: Covered; RWC: Designed |
| WA-7.8 | Shared workers | **Designed** | DM 6.10, DM-D8 (`hg-agents`) | DM-M3; DM-1 `workers/` 95% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-7.9 | Worklets | **Designed** | DM 6.7, 6.8 (`hg-workers`); JS 6.12 | — (no milestone names worklets; `worklets/` no floor) | map: Covered; RWC: Designed |
| WA-7.10 | Cross-document messaging | **Designed** | DM 6.11, DM-D13 (`hg-messaging`, broker) | DM-M2; DM-1 `webmessaging/` 95%; DM-25 | map: Covered; RWC: Designed |
| WA-7.11 | Channel messaging | **Designed** | DM 6.11, DM-D13 (`hg-messaging`) | DM-M2; DM-1 `webmessaging/` 95% | map: Covered; RWC: Designed |
| WA-7.12 | Broadcast Channel | **Designed** | DM 6.11 (`hg-agents` routing) | DM-M2; DM-1 `webmessaging/` 95% (`broadcastchannel/`) | map: Covered; RWC: Designed |
| WA-7.13 | Structured clone and transfer | **Designed** | DM 6.11, DM-D12 (`hg-clone-wire`) | DM-M2; DM-1 `html/webappapis/` 97%; DM-12 | map: Covered; RWC: Designed |
| WA-7.14 | SharedArrayBuffer and Atomics in browsers | **Designed** | JS 6.12 (`hg-js-runtime`); DM 6.8 | JS-M2; JS-1 test262 `Atomics` | map: Covered; RWC: Designed |
| WA-7.15 | Web Locks | **Designed** | DM 6.17 (`hg-webapi`, `hg-agents`); SG 6.3, SG-D12 | — (no milestone; `web-locks/` no floor) | map: Covered; RWC: Designed |

### WA 8. Fetch, networking and data formats

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-8.1 | Fetch API | **Designed** | NW 6.7 (partition service); DM 6.18 (`hg-webapi`) | NW-M1; NW-1 `fetch/` ≥ 95% | map: Covered; RWC: Designed |
| WA-8.2 | Fetch upload streaming and ranges | **Partial** | NW 6.2 (bodies as data pipes); NW 6.10 (ranges cached) **→ FI-16** | — | map: Partial |
| WA-8.3 | `fetchLater()` | **Owned** | NW 6.11 (partition service) | NW-M4 (not itemized); NW-1 `fetch/` (`fetch/fetch-later/`) | map: Covered; RWC: Owned |
| WA-8.4 | Body helpers | **Owned** | DM 6.18 (`hg-webapi`, body mixin) | NW-M1 Fetch core; NW-1 `fetch/` (`fetch/api/`) | map: Covered; RWC: Owned |
| WA-8.5 | XMLHttpRequest | **Designed** | DM 6.18, 6.7 (`hg-webapi`); NW 6.7 | DM-M2; NW-1 `xhr/` ≥ 95% | map: Covered; RWC: Designed |
| WA-8.6 | Server-sent events | **Mentioned** | DM 6.18 (`hg-webapi`, "`EventSource` objects") **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-8.7 | Beacon | **Partial** | NW 6.11 (`keepalive` owned by the partition service); CB 6.2 (`sendBeacon` as a channel) **→ FI-14** | — | map: Partial |
| WA-8.8 | WebSockets | **Designed** | NW 6.11 (partition service) | NW-M2 (H1, H2), NW-M3 (H3); NW-7 | map: Covered; RWC: Designed |
| WA-8.9 | WebSocketStream [1E] | **Held** | — — rule only, no per-feature record **→ FI-16** | — | map: Missing; VMW: Held (rule) |
| WA-8.10 | WebTransport | **Designed** | NW 6.11, NW-D13 | NW-M4, gate rule (WPT `webtransport/` + two servers) | map: Covered; RWC: Designed |
| WA-8.11 | Direct Sockets | **Refused** | S1 §6 out-of-scope list; NW §1 | — (Stages 1–2) | map: Excluded; VCE: Confirmed → Refused |
| WA-8.12 | Streams | **Mentioned** | DM 6.18, 6.17 (`hg-webapi`, "bindings and algorithms") **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-8.13 | Compression Streams | **Mentioned** | DM 6.18 (`hg-webapi` over `hg-compress`) **→ FI-14** | — (NW-M1 ships `hg-compress` decoders only) | map: Covered; RWC: Mentioned |
| WA-8.14 | Encoding API | **Designed** | ST 6.x, ST-D16 (`hg-encoding`); DM 6.18 | ST-M0; ST-15 `encoding/` ≥ 97% | map: Covered; RWC: Designed |
| WA-8.15 | URL and URLSearchParams | **Designed** | ST (`hg-url`, G31); DM 6.18; JS 6.13 (UTS 46) | ST-M0; ST-15 `url/` ≥ 97% | map: Covered; RWC: Designed |
| WA-8.16 | URL Pattern | **Missing** | — **→ FI-14** | — | map: Missing; VMW: Confirmed missing |
| WA-8.17 | MIME Sniffing | **Owned** | NW 6.2 (`hg-net-wire`) | NW-M2 (ORB); NW-1 `fetch/` (`fetch/orb/`) | map: Covered; RWC: Owned |
| WA-8.18 | Cookies as page-visible state | **Designed** | NW 6.9, NW-D10, NW-D16 (`hg-cookies`) | NW-M2; NW-2 `cookies/` ≥ 95% | map: Covered; RWC: Designed |
| WA-8.19 | Client Hints [1E] | **Partial** | S1 13.2 (client-hint brands), S1 §6 (normalized), DM 6.16 ("client-hint mirrors"), NW-D12 **→ FI-12** | — | map: Partial |
| WA-8.20 | Network Information and Save-Data [1E] | **Held** | — — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule) |

### WA 9. Storage, files and quota

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-9.1 | Web Storage | **Designed** | DM 6.13, DM-D11 (`hg-storage-dom`); SG 6.3, 6.7 | SG-M2; SG-3 | map: Covered; RWC: Designed |
| WA-9.2 | IndexedDB 3 | **Designed** | SG 6.2–6.6 (`hg-store-engine`); DM 6.13 | SG-M2 ("Full IndexedDB surface"); SG-2 `IndexedDB/` recorded floor | map: Covered; RWC: Designed |
| WA-9.3 | Cache API | **Designed** | SG 6.3, 6.6 (storage service) | SG-M3; SG-2 cache subset recorded floor | map: Covered; RWC: Designed |
| WA-9.4 | Storage Standard | **Designed** | SG 6.4, 6.5 (SG-D4, SG-D5) | SG-M2; SG-2 `storage/` ≥ 90%; SG-10 | map: Covered; RWC: Designed |
| WA-9.5 | Storage Buckets | **Held** | SG 6.4, SG-D4 (default bucket only until the draft stabilizes) | When the draft stabilizes | map: Excluded; VCE: Confirmed → Held |
| WA-9.6 | Cookie Store API | **Designed** | NW 6.9, NW-D10 | NW-M2; NW-2 (Cookie Store event cases 100%) | map: Covered; RWC: Designed |
| WA-9.7 | File API | **Designed** | DM 6.18 (`hg-webapi`); SG-D10 | SG-M3 (blob value-log backing) | map: Covered; RWC: Designed |
| WA-9.8 | Origin private file system (OPFS) | **Designed** | SG-D9, SG 6.7; DM 6.18 | SG-M3 (OPFS with sync handles) | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-9.9 | File System Access | **Partial** | SG 6.14 (byte transport); SS 6.14a (grant rules); S1 13.2 (Stage 1 permission) **→ FI-5** | Stage 1 | map: Partial |
| WA-9.10 | File System Observer [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| WA-9.11 | File and Directory Entries API | **Missing** | — **→ FI-14** | — | map: Missing; VMW: Confirmed missing |
| WA-9.12 | Removed storage | **Refused** | DM-D15; SG §2 | — | map: Excluded; VCE: Confirmed → Refused |

### WA 10. Service workers, background work, push and notifications

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-10.1 | Service Workers | **Designed** | DM 6.10, 6.8 (DM-D8), `hg-agents`/`hg-workers`; NW 6.7 routing | DM-M3; DM-1 `service-workers/` 95%; DM-27 | map: Covered; RWC: Designed |
| WA-10.2 | Module service workers | **Owned** | DM 6.3 corpus; DM 7 anchors (Service Workers); `hg-workers` | DM-M3; DM-1 `service-workers/` 95% | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-10.3 | Navigation preload | **Owned** | DM 6.10, 6.18 (allocated to NW partition service) | DM-M3; DM-1 `service-workers/` 95% (`navigation-preload/` tests) | map: Covered; RWC: Owned |
| WA-10.4 | Static routing API | **Designed** | DM 6.10; NW partition service | DM-M3; DM-27 "static routes never start the worker"; `service-workers/` 95% | map: Covered; RWC: Designed |
| WA-10.5 | Push API | **Deferred** | S1 13.2; NW 6.14 (push client), NW-M6 | Stage 2 | map: Excluded; VCE: Confirmed → Deferred |
| WA-10.6 | Declarative Web Push [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-10.7 | Notifications API | **Deferred** | SH 6.13, SH-D17; DM 6.18 (DOM side) | Stage 2 | map: Excluded; VCE: Confirmed → Deferred |
| WA-10.8 | Background Sync | **Deferred, no owner** | S1 13.2 names it only; Chromium-only, so the admission rule would hold it **→ FI-9** | Stage 2 | map: Excluded; VCE: Confirmed → Deferred, no owner |
| WA-10.9 | Periodic Background Sync [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-10.10 | Background Fetch | **Refused** | NW 6.11 | — | map: Excluded; VCE: Confirmed → Refused |
| WA-10.11 | Content Index [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |

### WA 11. Installable web apps and OS integration

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-11.1 | Web App Manifest | **Missing** | — **→ FI-9** | — | map: Missing; VMW: Confirmed missing |
| WA-11.2 | Install prompts [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-11.3 | Launch Handler and File Handling [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-11.4 | Protocol handlers | **Partial** | HTML claim (custom scheme handlers); SH 6.4 (external-protocol prompt, outgoing); EX (`protocol_handlers` key, Stage 3) **→ FI-9** | — | map: Partial |
| WA-11.5 | Web Share Target [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-11.6 | Badging API | **Missing** | — **→ FI-9** | — | map: Missing; VMW: Confirmed missing |
| WA-11.7 | Window Controls Overlay [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-11.8 | Window Management [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-11.9 | Document Picture-in-Picture | **Designed** | MD 6.12, MD 6.20, MD-M2 | MD-M2 (Stage 1); S1 13.2 says Stage 2 (contradiction) | map: Excluded; VCE: Misread → Designed. *contradiction: MD 6.12 builds it at MD-M2, S1 13.2 says Stage 2* |
| WA-11.10 | Isolated Web Apps and their APIs | **Refused** | DV 6.3, 6.10 | — (Stages 1–2) | map: Excluded; VCE: Confirmed → Refused |
| WA-11.11 | Get Installed Related Apps [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |
| WA-11.12 | PWA origin migration [1E] | **Held** | — — rule only, no per-feature record **→ FI-9** | — | map: Missing; VMW: Held (rule) |

### WA 12. Permissions, clipboard, sharing and other user-mediated APIs

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-12.1 | Permissions API | **Owned** | DM 6.18 (`hg-webapi`); S1 13.2 store; DV 6.10; SH 6.4 | S1 "detailed design is written before M6" | map: Covered; RWC: Owned |
| WA-12.2 | Permissions Policy | **Partial** | Consumed by DV 6.10, MD-D13, MD (`speaker-selection`), AX 6.7 (`aria-notify`); CB cites the spec **→ FI-15** | — | map: Partial |
| WA-12.3 | Document Policy [1E] | **Held** | — — rule only, no per-feature record **→ FI-15** | — | map: Missing; VMW: Held (rule) |
| WA-12.4 | Async Clipboard | **Designed** | ED 6.10 (`hg-clipboard`, ED-D10, ED-D11); DM 6.18 | ED-M1/ED-M2; ED-1 `clipboard-apis/` 95%; ED-11 | map: Covered; RWC: Designed |
| WA-12.5 | Clipboard events and custom formats | **Designed** | ED 6.10 (ED-D12) | ED-M2; ED-1 `clipboard-apis/` 95% | map: Covered; RWC: Designed |
| WA-12.6 | Web Share | **Missing** | — **→ FI-4** | — | map: Missing; VMW: Confirmed missing |
| WA-12.7 | Contact Picker [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-12.8 | EyeDropper [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-12.9 | Local Font Access | **Refused** | TX-D17 | — | map: Excluded; VCE: Confirmed → Refused |
| WA-12.10 | Web Speech API | **Mentioned** | MD 6.20 ("a separate spec" that does not exist; recognition under B10) **→ FI-29** | — | map: Missing; VMW: Actually partial. *VMW: MD 6.20 hands it to "a separate spec" that does not exist* |
| WA-12.11 | Virtual Keyboard API | **Held** | ED-D18 | When admitted | map: Excluded; VCE: Confirmed → Held |
| WA-12.12 | Selective permissions intervention [1E] | **Missing** | — **→ FI-12** | — | map: Missing; VMW: Confirmed missing |

### WA 13. Security and isolation (page-visible)

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-13.1 | Same-origin policy and `document.domain` | **Designed** | DM 6.8 (DM-D6), `hg-agents` | DM-M2; DM-11; DM-1 with named exclusions | map: Covered; RWC: Designed |
| WA-13.2 | Content Security Policy Level 3 | **Partial** | NW 6.7 (renderer pre-request checks including `<meta>`; network-side fetch directives); ST (CSP for style requests, nonce hidden); JS 6.14 (`HostEnsureCanCompileStrings`) **→ FI-15** | NW-M2 | map: Partial |
| WA-13.3 | Trusted Types | **Designed** | DM 6.16, DM 7 (`hg-dom` sinks); JS 6.14 host hook; ED 9.5 | Trusted Types WPT (DM 6.16 "Checked by"); `trusted-types/` test input | map: Covered; RWC: Designed |
| WA-13.4 | Cross-origin isolation | **Designed** | NW 6.7 (CORP, COEP); DM 6.8 (COOP, isolation modes) | NW-M2 exit (CORP, COEP); NW-1; DM-M2 | map: Covered; RWC: Designed |
| WA-13.5 | Document-Isolation-Policy [1E] | **Held** | — — rule only, no per-feature record **→ FI-16** | — | map: Missing; VMW: Held (rule) |
| WA-13.6 | Fetch Metadata | **Owned** | NW 6.7 (partition service) | NW-M2; NW-1 `fetch/` ≥ 95% (`fetch/metadata/`) | map: Covered; RWC: Owned |
| WA-13.7 | Referrer Policy | **Owned** | NW 6.7 (partition service) | NW-M1/M2; NW-1 `referrer-policy/` ≥ 95% | map: Covered; RWC: Owned |
| WA-13.8 | Mixed content and HTTPS upgrades | **Designed** | NW 6.7 (HTTPS-First), 6.5.3 (NW-D25) | NW-M2; NW-1 `mixed-content/` | map: Covered; RWC: Designed |
| WA-13.9 | Secure contexts | **Owned** | DM 6.3 (`[SecureContext]` descriptors); DV 6.10 | DM-M1; DM-3 idlharness | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-13.10 | iframe sandbox and X-Frame-Options | **Mentioned** | DM 6.9, 6.16 (broker sandbox flags); XFO by HTML claim **→ FI-15** | DM-11 (sandbox); none for XFO | map: Covered; RWC: Mentioned |
| WA-13.11 | Clear-Site-Data | **Designed** | NW 6.8 (NW-D18); SG 6.9 clearing path | NW-M2; NW-19 cross-container case; SG-M3 | map: Covered; RWC: Designed |
| WA-13.12 | Local Network Access | **Designed** | NW 6.1, 6.7 (NW-D4 tickets), 6.11 | NW-M2; NW-16; NW-43 | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-13.13 | Connection Allowlists [1E] | **Held** | — — rule only, no per-feature record **→ FI-16** | — | map: Missing; VMW: Held (rule) |
| WA-13.14 | Web Cryptography | **Designed** | WC spec (`hg-webcrypto`, WC-D1, WC-D2) | WC milestones (WC-M2); NW-49/50/52 | map: Covered; RWC: Designed |

### WA 14. Privacy, tracking protection and advertising APIs

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-14.1 | Storage Access API | **Designed** | NW 6.8, 6.9 (NW-D16); DM 6.5 activation | NW-M2; NW-2; NW-19 | map: Covered; RWC: Designed |
| WA-14.2 | `requestStorageAccessFor` and Related Website Sets | **Refused** | NW 6.9 | — | map: Excluded; VCE: Confirmed → Refused; FC. *FC 2b: retired 17 October 2025* |
| WA-14.3 | Storage and cookie partitioning | **Designed** | NW 6.8 (NW-D7), 6.9; S1 §6 | NW-M2 (partition table complete); NW-19, NW-20, NW-46 | map: Covered; RWC: Designed |
| WA-14.4 | Bounce tracking mitigations | **Designed** | NW 6.9; CB 6.5 (`hg-nav-protect`, CB-D8) | NW-M4; CB-10 | map: Covered; RWC: Designed |
| WA-14.5 | Topics API [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule); FC. *FC 2b: retired by Google on 17 October 2025* |
| WA-14.6 | Protected Audience [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule); FC. *FC 2b: retired 17 October 2025* |
| WA-14.7 | Shared Storage and selectURL [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule); FC. *FC 2b: retired 17 October 2025* |
| WA-14.8 | Private Aggregation [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule); FC. *FC 2b: retired 17 October 2025* |
| WA-14.9 | Attribution Reporting [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule); FC. *FC 2b–2c: retired; a W3C attribution standard continues* |
| WA-14.10 | Private Click Measurement [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule) |
| WA-14.11 | Private State Tokens [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule); FC. *FC 2c: Google keeps Private State Tokens* |
| WA-14.12 | Private Verification Tokens [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-12** | — | map: Missing; VMW: Held (rule) |
| WA-14.13 | Global Privacy Control | **Designed** | S1 §6 (GPC row); NW 6.7 (`Sec-GPC: 1`); SH settings | Stage 1 | map: Covered; RWC: Designed |
| WA-14.14 | User-Agent reduction and string freezes | **Designed** | S1 13.2 (UA); NW 6.20 (NW-D12) | NW-M1; NW-23; UA-1 | map: Covered; RWC: Designed |
| WA-14.15 | Fingerprinting protections | **Designed** | S1 §6 (D-S1, OD-6); DM 6.16; SS 6.15 | Stage 1 baseline, Stage 2 full; PRIV-2 | map: Covered; RWC: Designed |
| WA-14.16 | IP Protection | **Held** | — (S1 §6 refuses a browser-operated VPN, a different thing; implicit refusal only) — rule only, no per-feature record **→ FI-12** | — | map: Excluded; VCE: Misread → Missing; SYN: Held (retired by Google and Chromium-only; held like WA-14.5 to 14.9 under VMW's reading; the privacy spec (FI-12) should record the refusal) (checks differ; the later check wins); FC. *FC 2b: retired 17 October 2025* |

### WA 15. Identity, credentials and payments

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-15.1 | Credential Management | **Missing** | — **→ FI-13** | — | map: Missing; VMW: Confirmed missing |
| WA-15.2 | Web Authentication (passkeys) | **Partial** | S1 §6 ("WebAuthn through platform authenticators", Stage 1); SS SH-R1, SEC-23; SH 6.4 (credential prompt class); DV 6.14 **→ FI-13** | Stage 1 (no milestone) | map: Partial |
| WA-15.3 | FedCM [1E] | **Held** | No decision recorded — rule only, no per-feature record **→ FI-13** | — | map: Missing; VMW: Held (rule); FC. *FC 2c: Google keeps FedCM* |
| WA-15.4 | Digital Credentials API | **Missing** | — **→ FI-13** | — | map: Missing; VMW: Confirmed missing |
| WA-15.5 | WebOTP [1E] | **Held** | — — rule only, no per-feature record **→ FI-13** | — | map: Missing; VMW: Held (rule) |
| WA-15.6 | Device Bound Session Credentials [1E] | **Held** | — — rule only, no per-feature record **→ FI-13** | — | map: Missing; VMW: Held (rule) |
| WA-15.7 | Email Verification Protocol [1E] | **Held** | — — rule only, no per-feature record **→ FI-13** | — | map: Missing; VMW: Held (rule) |
| WA-15.8 | Payment Request API | **Partial** | DV 6.14 ("Payments stay in Payment Request"); DM 6.5 (payment among activation-gated capabilities); SH 6.9 (payment autofill Stage 2) **→ FI-13** | — | map: Partial |
| WA-15.9 | Payment Handler [1E] | **Partial** | DT 6.5 (redacts "payment-handler results") **→ FI-13** | — | map: Partial |
| WA-15.10 | Secure Payment Confirmation [1E] | **Held** | — — rule only, no per-feature record **→ FI-13** | — | map: Missing; VMW: Held (rule) |
| WA-15.11 | Apple Pay JS and merchant validation [1E] | **Held** | — — rule only, no per-feature record **→ FI-13** | — | map: Missing; VMW: Held (rule) |

### WA 16. Device and hardware APIs

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-16.1 | Geolocation | **Partial** | DM 6.18 (DOM side, activation checks); S1 13.2 (location permission); SH 6.4 (capability class) **→ FI-28** | — | map: Partial |
| WA-16.2 | Device orientation and motion events | **Missing** | — **→ FI-28** | — | map: Missing; VMW: Confirmed missing |
| WA-16.3 | Generic Sensor family [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.4 | Battery Status [1E] | **Refused** | — | — | map: Missing; VMW: Actually excluded |
| WA-16.5 | Device Memory | **Held** | DM 6.16 lists `deviceMemory` only in fingerprinting row — rule only, no per-feature record **→ FI-12** | — | map: Covered; VCE: Only partial; RWC: Held (checks differ; the later check wins). *exposed by DM 6.16's environment record with no DM-D15 exception; VCE said Partial [1E]* |
| WA-16.6 | Compute Pressure [1E] | **Refused** | — | — | map: Missing; VMW: Actually excluded (weak). *exclusion is implied (FP 6.15 "pressure state … Never"), not named* |
| WA-16.7 | CPU Performance API [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.8 | Screen Wake Lock | **Partial** | DM 6.18 (DOM side) **→ FI-28** | — | map: Partial |
| WA-16.9 | Vibration | **Missing** | — **→ FI-28** | — | map: Missing; VMW: Confirmed missing |
| WA-16.10 | Screen Orientation | **Missing** | — **→ FI-28** | — | map: Missing; VMW: Confirmed missing |
| WA-16.11 | Idle Detection [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.12 | Gamepad | **Partial** | DV 6.14 ("Gamepads stay with the Gamepad API") **→ FI-28** | — | map: Partial |
| WA-16.13 | Web MIDI | **Designed** | DV 6.6 (DV-D13); DM 6.18 bindings | DV-M2 (experimental setting); DV-E5 at DV-M3 | map: Covered; RWC: Designed |
| WA-16.14 | Web Serial | **Designed** | DV 6.5; DM 6.18 | DV-M2; DV-E6 at DV-M3 | map: Covered; RWC: Designed |
| WA-16.15 | WebUSB | **Designed** | DV 6.3; DM 6.18 | DV-M2; DV-E1 at DV-M4 | map: Covered; RWC: Designed |
| WA-16.16 | WebHID | **Designed** | DV 6.4; DM 6.18 | DV-M2; DV-E2 at DV-M4 | map: Covered; RWC: Designed |
| WA-16.17 | Web Bluetooth | **Designed** | DV 6.7; DM 6.18 | DV-M2 (GATT only); DV-E3 at DV-M4 | map: Covered; RWC: Designed |
| WA-16.18 | Web NFC | **Designed** | DV 6.8 (DV-D9); DM 6.18 | Built, not shipped; DV-E4 at DV-M5 | map: Covered; RWC: Designed |
| WA-16.19 | Device Posture [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.20 | Viewport Segments [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.21 | Shape Detection [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.22 | Presentation API [1E] | **Held** | — — rule only, no per-feature record **→ FI-32** | — | map: Missing; VMW: Held (rule) |
| WA-16.23 | Input Device Capabilities [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| WA-16.24 | Managed device attributes [1E] | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |

### WA 17. Performance, measurement and reporting

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-17.1 | High Resolution Time | **Designed** | DM 6.7 (DM-D10), `hg-script` | DM-M1; DM-14 (WPT `hr-time/`) | map: Covered; RWC: Designed |
| WA-17.2 | Performance Timeline | **Mentioned** | DM 6.18 (`hg-webapi`) **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-17.3 | Navigation Timing 2 | **Mentioned** | DM 6.18 (`hg-webapi`, "Navigation") **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-17.4 | Resource Timing 3 | **Owned** | DM 6.18 (`hg-webapi`); NW 6.8 table row | NW-19 (every 6.8 row) | map: Covered; RWC: Owned |
| WA-17.5 | User Timing 3 | **Mentioned** | DM 6.18 (`hg-webapi`) **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-17.6 | Server Timing | **Missing** | — **→ FI-16** | — | map: Missing; VMW: Confirmed missing |
| WA-17.7 | Paint Timing | **Mentioned** | DM 6.7 (`hg-script` "mark paint timing"); DM 6.18; GR 6.x **→ FI-14** | — | map: Covered; RWC: Mentioned |
| WA-17.8 | Largest Contentful Paint | **Partial** | LY 6.1 (LCP among clock reads); MD (LCP render times as a timing channel); DM 6.18 (Performance Timeline family) **→ FI-14** | — | map: Partial |
| WA-17.9 | Event Timing and INP | **Owned** | DM 6.18 (`hg-webapi`); DM 6.7 input-first policy | GR-44 (cross-site Event Timing channel ≤ 0.1 bit/s) | map: Covered; RWC: Owned |
| WA-17.10 | Layout Instability | **Designed** | LY 6.15 (`hg-layout-query`, LY-D10 exception) | LY milestones; WPT `layout-instability/` (LY test inputs) | map: Covered; RWC: Designed |
| WA-17.11 | Long Tasks and Long Animation Frames | **Owned** | DM 6.7, 6.12 (DM-D15 exception), `hg-script` | DM-1 per-directory rule, `long-animation-frame/` test input | map: Covered; RWC: Owned |
| WA-17.12 | Element Timing and Container Timing [1E] | **Partial** | DM 6.18 lists Element Timing **→ FI-37** | — | map: Partial. *Element Timing is listed by DM 6.18 with no DM-D15 exception* |
| WA-17.13 | Soft navigations [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| WA-17.14 | Memory and profiling [1E] | **Refused** | — | — | map: Missing; VMW: Actually excluded |
| WA-17.15 | Reporting API | **Designed** | NW 6.8, 6.11; DM 6.18; FP 6.13 | NW-19 (Reporting row of 6.8) | map: Covered; VCE: Confirmed covered; RWC: Designed |
| WA-17.16 | Network Error Logging | **Refused** | NW-D15 | — | map: Excluded; VCE: Confirmed → Refused |
| WA-17.17 | Crash Report API [1E] | **Partial** | FP 6.7 (WICG crash reporting: `oom` reports only when the site's own process crossed the threshold) **→ FI-11** | — | map: Partial |
| WA-17.18 | Scroll performance timing [1E] | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |

### WA 18. ECMAScript language (coarse grain)

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-18.1 | ES5 core and Annex B | **Designed** | JS 6.1–6.4, 6.13 (`hg-regex`); JS-5 | JS-M1/M2; JS-1 ≥99% | map: Covered; RWC: Designed |
| WA-18.2 | ES2015 syntax | **Owned** | JS (ES2026 claim) | JS-M2 "ES2026 plus Temporal"; JS-1 | map: Covered; RWC: Owned |
| WA-18.3 | ES2015 library | **Owned** | JS (ES2026 claim); JS 6.5 ephemerons | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.4 | Proper tail calls | **Partial** | ES2026 claim (JS-1) **→ FI-26** | — | map: Partial |
| WA-18.5 | Modules | **Designed** | JS 6.14 (`hg-js-host`) | JS-M1 "ES modules" | map: Covered; RWC: Designed |
| WA-18.6 | Async functions and async iteration | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.7 | ES2018–2021 additions | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.8 | Weak references | **Designed** | JS 6.5 (`hg-heap`) | JS-M2; JS-50 (WeakRef order) | map: Covered; RWC: Designed |
| WA-18.9 | ES2022 classes | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.10 | ES2022–2024 library | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.11 | Shared memory | **Designed** | JS 6.12 | JS-M2; JS-1 | map: Covered; RWC: Designed |
| WA-18.12 | Resizable and transferable ArrayBuffers | **Owned** | JS (ES2026 claim); DM 6.11 clone | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.13 | Iterator helpers | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.14 | Iterator sequencing, joint iteration and new methods | **Partial** | ES2026 claim for `Iterator.concat`; others under JS-1's Stage 4 scope **→ FI-26** | JS-M2 (concat) | map: Partial |
| WA-18.15 | Set methods | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.16 | ES2025 batch | **Owned** | JS (ES2026 claim); JS 6.14 | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.17 | Explicit resource management | **Missing** | — (not named; no recorded hold) **→ FI-26** | — | map: Excluded; VCE: Misread → Missing |
| WA-18.18 | Upsert | **Mentioned** | — (ES claim only) **→ FI-26** | — (JS-M2 only if in ES2026) | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins). *ES2026 membership rests on secondary sources* |
| WA-18.19 | `Math.sumPrecise` | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.20 | Uint8Array base64/hex | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; VCE: Confirmed covered; RWC: Owned |
| WA-18.21 | `Error.isError`, stack APIs | **Partial** | JS-1 Stage 4 scope (`Error.isError`) **→ FI-26** | — | map: Partial |
| WA-18.22 | `Array.fromAsync` | **Owned** | JS (ES2026 claim) | JS-M2; JS-1 | map: Covered; RWC: Owned |
| WA-18.23 | JSON.parse source text access | **Mentioned** | — (ES claim only) **→ FI-26** | — (JS-M2 only if in ES2026) | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins). *ES2026 membership rests on secondary sources* |
| WA-18.24 | `Promise.allKeyed` / `allSettledKeyed` | **Missing** | — (not named; no recorded hold) **→ FI-26** | — | map: Excluded; VCE: Misread → Missing |
| WA-18.25 | Source phase imports and `import defer` | **Partial** | JS 6.14 (designed); Q-JS-12 open **→ FI-26** | When Stage 4 (Q-JS-12) | map: Excluded; VCE: Misread → Partial |
| WA-18.26 | Not yet shipped | **Held** | JS 6.14, JS 9.1 (not until Stage 4) | At TC39 Stage 4 | map: Excluded; VCE: Confirmed → Held |

### WA 19. Intl and Temporal

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-19.1 | Intl core | **Designed** | JS 6.13 (`hg-intl`, `hg-cldr-build`) | JS-M3; JS-2 ≥95%/99% | map: Covered; RWC: Designed |
| WA-19.2 | `Intl.Segmenter` | **Designed** | JS 6.13; TX `hg-unicode` | JS-M3; JS-2 | map: Covered; RWC: Designed |
| WA-19.3 | `Intl.DurationFormat` | **Owned** | JS 6.13 (ECMA-402 claim) | JS-M3 "all of ECMA-402"; JS-2 | map: Covered; RWC: Owned |
| WA-19.4 | `Intl.Locale` info and `variants` | **Partial** | ECMA-402 claim (JS-2) **→ FI-26** | — | map: Partial |
| WA-19.5 | `Intl.PluralRules` notation | **Owned** | JS 6.13 (ECMA-402 claim) | JS-M3; JS-2 | map: Covered; RWC: Owned |
| WA-19.6 | Temporal | **Designed** | JS 6.14, 6.13 (`hg-tz`) | JS-M2 "plus Temporal" | map: Covered; RWC: Designed |

### WA 20. WebAssembly

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-20.1 | Wasm MVP and JS API | **Designed** | JS 6.6 (`hg-wasm`) | JS-M2 (W0, JS API); JS-3 100% .wast | map: Covered; RWC: Designed |
| WA-20.2 | Wasm Web API | **Owned** | JS 6.6; JS-3 | JS-M2; JS-3 `wasm/webapi/` C-1 floor | map: Covered; RWC: Owned |
| WA-20.3 | Post-MVP core features | **Owned** | JS 6.6; JS-3 | JS-M2; JS-3 100% Wasm 3.0 .wast | map: Covered; RWC: Owned |
| WA-20.4 | Threads | **Owned** | JS 6.12; JS-3 | JS-M2; JS-3 "Wasm 3.0 plus threads" | map: Covered; RWC: Owned |
| WA-20.5 | Exception handling | **Designed** | JS 6.6 | JS-M2; JS-3 | map: Covered; RWC: Designed |
| WA-20.6 | GC and tail calls | **Designed** | JS 6.5, 6.6 | JS-M2; JS-3 | map: Covered; RWC: Designed |
| WA-20.7 | Memory64 and multi-memory | **Designed** | JS 6.6 | JS-M2; JS-3 | map: Covered; RWC: Designed |
| WA-20.8 | Relaxed SIMD | **Designed** | JS 6.9, 6.18; JS-D12 | JS-M2; JS-3 | map: Covered; RWC: Designed |
| WA-20.9 | JS String Builtins | **Designed** | JS 6.6 | JS-M2; JS-3 | map: Covered; RWC: Designed |
| WA-20.10 | JavaScript Promise Integration (JSPI) | **Designed** | JS 6.6 (JS-D19); JS 9.1 | JS-M2 "JSPI" | map: Covered; RWC: Designed |
| WA-20.11 | Resizable memory buffers | **Mentioned** | — **→ FI-26** | — | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins) |
| WA-20.12 | Newer proposals | **Held** | JS 9.1 (Wasm phase-4 rule); Q-JS-12 | At Wasm phase 4 | map: Excluded; VCE: Confirmed → Held |

### WA 21. On-device AI and agent-facing APIs

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-21.1 | Translator and Language Detector | **Deferred** | ML 6.5, ML-D2; HGBA-2 4.5 | Stage 2, under the AI grant | map: Excluded; VCE: Confirmed → Deferred |
| WA-21.2 | Summarizer, Writer, Rewriter, Proofreader | **Deferred** | ML-D2; HGBA-2 4.5 | Stage 2, under the AI grant | map: Excluded; VCE: Confirmed → Deferred |
| WA-21.3 | Prompt API | **Deferred** | ML-D2; HGBA-3 B10 | Stage 2, under the AI grant | map: Excluded; VCE: Confirmed → Deferred |
| WA-21.4 | WebNN | **Deferred** | HGBA-2 4.5; ML-D2 (behind a flag, not shipped in Stage 1) | Stage 2 at the earliest | map: Excluded; VCE: Confirmed → Deferred |
| WA-21.5 | WebMCP | **Deferred** | S1 §9; HGBA-3 B9, B10 (inert surface) | Stage 3 | map: Excluded; VCE: Confirmed → Deferred |

### WA 22. Browser-shipped protocols and infrastructure standards

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| WA-22.1 | WebDriver (classic) | **Designed** | DT 6.3, 6.5 (`hg-bidi`, `izcaldriver`) | S1 M2 slice; M7; DT-5 `webdriver/tests/classic/` | map: Covered; RWC: Designed |
| WA-22.2 | WebDriver BiDi | **Designed** | DT 6.3 (`hg-bidi` over IIP); NW 6.18 | M7; DT-5 `webdriver/tests/bidi/` | map: Covered; RWC: Designed |
| WA-22.3 | WebDriver extension commands | **Partial** | DV 6.13 (virtual devices), ED 6.16 (virtual IME) as DevTools oracles **→ FI-7** | — | map: Partial |
| WA-22.4 | Chrome DevTools Protocol | **Designed** | DT 6.3 (`hg-cdp`), DT-D12 | M7; DT-6; JS-63 | map: Covered; RWC: Designed |
| WA-22.5 | Browser MCP servers | **Designed** | DT 6.3 (`hg-agent-mcp`, `izcal-mcp`) | Stage 2 (with B9); DT-13, DT-14 | map: Covered; RWC: Designed |
| WA-22.6 | Web IDL | **Designed** | DM 6.3 (`hg-webidl`, DM-D2, DM-D21) | DM-M0; DM-1 `webidl/` ≥97% | map: Covered; RWC: Designed |
| WA-22.7 | Infra | **Missing** | — **→ FI-14** | — | map: Covered; VCE: Not there; RWC: Not there |
| WA-22.8 | Console | **Owned** | JS 6.10 (`hg-js-observe`), JS-63; DT 9.1 | JS-M1; DT M6 | map: Covered; RWC: Owned |
| WA-22.9 | Compatibility and Quirks Mode | **Partial** | ST (quirks, limited-quirks and no-quirks modes; quirks UA sheet) **→ FI-23** | ST milestones | map: Partial |
| WA-22.10 | Test Utils | **Missing** | — **→ FI-7** | — | map: Missing; VMW: Confirmed missing |
| WA-22.11 | WebExtensions | **Designed** | EX 6.0–6.19 (EX-D1–D16) | Stage 2; EX-M0 seams in Stage 1 | map: Covered; RWC: Designed |
| WA-22.12 | Origin trials | **Missing** | — **→ FI-7** | — | map: Missing; VMW: Confirmed missing |

## CG: CSS, graphics and media


### CG 1. CSS: syntax, cascade and selectors

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-1.1 | CSS 2.1 core | **Designed** | LY 6.5.1, 6.5.7 (`hg-layout`); GR 6.3 (`hg-paint`) | LY-M1, GR-M1; LY-1 `CSS2` 97% | map: Covered; RWC: Designed |
| CG-1.2 | CSS Snapshot 2026 | **Missing** | — **→ FI-23** | — | map: Missing; VMW: Confirmed missing |
| CG-1.3 | CSS Syntax 3 | **Designed** | ST 6.6 (`hg-css-syntax`) | ST-M1; ST-3 `css-syntax` 97% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-1.4 | Cascade and inheritance 3/4 | **Designed** | ST 6.9 (`hg-style`) | ST-M1; ST-3 `css-cascade` 97% | map: Covered; RWC: Designed |
| CG-1.5 | Cascade layers | **Designed** | ST 6.9, 6.6a (`hg-style`) | ST-M1 exit names layers; ST-3 `css-cascade` 97% | map: Covered; RWC: Designed |
| CG-1.6 | `@scope` | **Owned** | ST 6.9 (`hg-style`) | ST-M1 exit names `@scope`; ST-3 `css-cascade` 97% | map: Covered; RWC: Owned |
| CG-1.7 | `revert-rule` keyword | **Owned** | ST 6.9 (ST-D8 match keys) | ST-M1; ST-3 `css-cascade` 97% | map: Covered; RWC: Owned |
| CG-1.8 | CSS Nesting | **Designed** | ST 6.6, 6.7 | ST-M1; ST-3 `css-nesting` 97% | map: Covered; RWC: Designed |
| CG-1.9 | CSS Namespaces | **Partial** | — **→ FI-23** | — | map: Missing; VMW: Actually partial |
| CG-1.10 | Style attributes | **Designed** | ST 6.11, 6.6 (`hg-dom`, `hg-style`) | ST-M1 | map: Covered; RWC: Designed |
| CG-1.11 | Selectors 3 | **Designed** | ST 6.7 (ST-D9) | ST-M0 reference, ST-M1; ST-3 `selectors` 95% | map: Covered; RWC: Designed |
| CG-1.12 | Selectors 4 | **Designed** | ST 6.7, 6.7.1, 6.7.2 (ST-D11) | ST-M1, `:has()` ST-M2; ST-3 `selectors` 95% | map: Covered; RWC: Designed |
| CG-1.13 | State pseudo-classes for UI | **Designed** | ST 6.7.1; DM 6.12 (`hg-html`) | ST-M1, DM-M2; DM-1 Interop release gates | map: Covered; RWC: Designed |
| CG-1.14 | Media pseudo-classes | **Owned** | ST 6.7.1 (Selectors 4 anchor) | ST-3 `selectors` 95% (`css/selectors/media/`) | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-1.15 | Selectors 5 | **Held** | ST-D18 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-1.16 | Pseudo-elements 4 | **Designed** | LY 6.3 (`hg-box-tree`); ED 6.7 highlights | LY-M1; ED-1 `css-highlight-api` 93% | map: Covered; RWC: Designed |
| CG-1.17 | Shadow DOM styling | **Designed** | ST 6.9, 6.7 | ST-M1; ST-3 `css-shadow` 87% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-1.18 | Custom properties | **Designed** | ST 6.9 (ST-D14) | ST-M2 by reference; ST-3 `css-variables` 87% | map: Covered; RWC: Designed |
| CG-1.19 | Registered custom properties | **Designed** | ST 6.9 | ST-M2; ST-3 `css-properties-values-api` 95% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-1.20 | Custom functions and mixins | **Held** | ST 9.1, ST-D18 ("`if()`, `@function`, `@mixin` … wait for ST-D18's admission rule"); interpreter built general enough (ST 6.9) | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-1.21 | CSS Speech | **Held** | ST-D18 (rule applies); AX 6.3 ("`speak-as` if admitted") — rule only, no per-feature record **→ FI-29** | — | map: Excluded; VCE: Confirmed as hold, wrong status |

### CG 2. CSS: values, units, functions and conditional rules

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-2.1 | Values and Units 3 | **Designed** | ST 6.9 | ST-M1; ST-3 `css-values` 87% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-2.2 | Values and Units 4 | **Designed** | ST 6.9, 6.10 | ST-M1; ST-3 `css-values` 87% | map: Covered; RWC: Designed |
| CG-2.3 | Typed `attr()` | **Designed** | ST 6.9 (substitution interpreter) | ST-M2 exit names typed `attr()` | map: Covered; RWC: Designed |
| CG-2.4 | `if()` | **Held** | ST 9.1, ST-D18 | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-2.5 | Tree-counting functions | **Owned** | ST 6.9 (substitution interpreter) | ST-M2; ST-3 `css-values` 87% (`tree-counting`) | map: Covered; RWC: Owned |
| CG-2.6 | `random()` | **Held** | ST-D18 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-2.7 | `progress()`, `*-mix()`, `calc-size()` | **Held** | LY 6.15, LY-D10 and AN-D9 ("`interpolate-size`, `calc-size()` … Held") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-2.8 | Environment variables | **Partial** | ST 6.10 (`env()` from the environment snapshot); AX 6.9 (`env(preferred-text-scale)` bucketed) **→ FI-23** | ST-M1 | map: Partial |
| CG-2.9 | Linked parameters | **Held** | SV 9.1, SV-D13 ("link parameters" held) | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-2.10 | Conditional rules 3 | **Designed** | ST 6.10 | ST-M1; ST-3 `css-conditional` 95% | map: Covered; RWC: Designed |
| CG-2.11 | Conditional rules 4 | **Owned** | ST 7 anchor (Conditional 5) | ST-3 `css-conditional` 95% | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-2.12 | Conditional rules 5 | **Owned** | ST 7 anchor (Conditional 5) | ST-3 `css-conditional` 95% | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-2.13 | Container size queries | **Designed** | ST 6.10; LY 6.9 (LY-D12) | ST-M2, LY-M2; LY-6 | map: Covered; RWC: Designed |
| CG-2.14 | Container style queries | **Designed** | ST 6.10 | ST-M2 container queries; ST-3 `css-conditional` 95% | map: Covered; RWC: Designed |
| CG-2.15 | Container scroll-state queries | **Held** | ST 9.1, ST-D18; LY 9.1, LY-D10; snapshot point designed (LY 6.9) | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-2.16 | Media Queries 3/4 | **Designed** | ST 6.10; MD 6.5; GR 6.20 | ST-M1 | map: Covered; RWC: Designed |
| CG-2.17 | Media Queries 5 | **Partial** | ST 6.10 (`prefers-*`, `forced-colors`); AX 6.9 (`prefers-reduced-transparency`, `inverted-colors`, contrast buckets) **→ FI-23** | ST-M1, AX-M2 | map: Partial |
| CG-2.18 | Viewport rule | **Partial** | ST 6.4 (speculative parser applies `<meta name=viewport>`) **→ FI-33** | — | map: Partial |
| CG-2.19 | CSS `zoom` | **Partial** | LY 6.2 (zoom factors converted once at the boundary) **→ FI-23** | — | map: Partial |
| CG-2.20 | Easing functions | **Designed** | AN 6.2 (AN-D2, `hg-anim-time`) | AN-M1; AN-1 `css-easing` 97% | map: Covered; RWC: Designed |

### CG 3. CSS: colour, images, backgrounds and borders

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-3.1 | Colour 3 | **Designed** | ST 6.9; MD 6.5 (`hg-colour`) | ST-M1, GR-M1; GR-1 `css-color` C-1 floor | map: Covered; RWC: Designed |
| CG-3.2 | Colour 4 | **Designed** | ST 6.9; MD 6.5; GR 6.4 | ST-M1, GR-M1; GR-1 `css-color` | map: Covered; RWC: Designed |
| CG-3.3 | Colour 5 | **Designed** | ST 6.9 | ST-M1; GR-1 `css-color` | map: Covered; RWC: Designed |
| CG-3.4 | `contrast-color()` | **Owned** | GR 7 anchor (Color 4 and 5) | GR-1 `css-color` C-1 floor | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-3.5 | Colour HDR | **Designed** | MD 6.5 (MD-D9); ST 6.9 | MD-M2, GR-M2 | map: Covered; RWC: Designed |
| CG-3.6 | Colour adjustment | **Designed** | AX 6.9; GR 6.23 (GR-D25, `hg-paint`); ST 6.6 | AX-M2, GR-M2 | map: Covered; RWC: Designed |
| CG-3.7 | Backgrounds and borders 3 | **Designed** | GR 6.4 (GR-D17) | GR-M1 exit; GR-1 `css-backgrounds`, `css-borders` | map: Covered; RWC: Designed |
| CG-3.8 | Backgrounds 4 | **Partial** | ST-M1 webref parsing only **→ FI-24** | — | map: Partial |
| CG-3.9 | Borders and box decorations 4 | **Owned** | GR 7 anchor (Borders 4); GR-D17; LY 6.14 | GR-M1; GR-1 `css-borders` | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-3.10 | Gap decorations | **Held** | LY 9.1, LY-D10 | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-3.11 | Images 3 | **Designed** | LY 6.5.8; GR 6.4; MD 6.6 | LY-M1, MD-M1; GR-1 `css-images` | map: Covered; RWC: Designed |
| CG-3.12 | Images 4 | **Designed** | GR 6.4; SV 6.8 | GR-M1; GR-1 `css-images` | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-3.13 | `object-view-box` | **Held** | ST-D18/LY-D10 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-3.14 | Image animation control | **Held** | MD 6.6, MD-D19 ("Izcal follows CSSWG issue 1615 so authors can opt specific images in when it is specified") | When specified | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-3.15 | Fill and stroke 3 | **Held** | ST-D18 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-3.16 | Legacy `-webkit-` features | **Partial** | TX 6.18 (`-webkit-text-stroke` owned by paint and SVG) **→ FI-23** | — | map: Partial |

### CG 4. CSS: box model, display and layout modes

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-4.1 | Box model 3 | **Designed** | LY 6.5.1 (`hg-layout`) | LY-M1; LY-1 `css-box` 97% | map: Covered; RWC: Designed |
| CG-4.2 | Box model 4 | **Held** | LY 6.5.1, LY-D10 | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-4.3 | Box sizing 3 | **Designed** | LY 6.6 | LY-M1/M2; LY-1 `css-sizing` 97% | map: Covered; RWC: Designed |
| CG-4.4 | Box sizing 4 | **Designed** | LY 6.6, 6.9 | LY-M2; LY-1 `css-sizing` 97% | map: Covered; RWC: Designed |
| CG-4.5 | Display 3 | **Designed** | LY 6.3 (LY-D2, `hg-box-tree`) | LY-M1; LY-1 `css-display` 97% | map: Covered; RWC: Designed |
| CG-4.6 | Display 4 | **Held** | LY 9.1, LY-D10; AX 6.3 uses it "when the property is admitted" | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-4.7 | Flexbox | **Designed** | LY 6.5.3 | LY-M2; LY-1 `css-flexbox` 97% | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-4.8 | Grid 1 | **Designed** | LY 6.5.4 | LY-M2; LY-1 `css-grid` 97% | map: Covered; RWC: Designed |
| CG-4.9 | Grid 2 (subgrid) | **Designed** | LY 6.5.4 | LY-M2 exit names subgrid; LY-1 `css-grid` 97% | map: Covered; RWC: Designed |
| CG-4.10 | Grid 3 (Grid Lanes / masonry) | **Held** | LY 6.5.4, LY-D10 ("not admitted"; sizing written lanes-ready) | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-4.11 | Box alignment 3 | **Owned** | LY 7 anchor (`hg-layout`) | LY-1 `css-align` 97% | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-4.12 | Multi-column 1 | **Designed** | LY 6.5.6 | LY-M2; LY-1 `css-multicol` 92%; LY-24 | map: Covered; RWC: Designed |
| CG-4.13 | Multi-column 2 | **Held** | LY 6.5.6, LY-D10 | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-4.14 | Tables 3 | **Designed** | LY 6.5.5 (LY-D17) | LY-M2; LY-1 `css-tables` 92% | map: Covered; RWC: Designed |
| CG-4.15 | Inline layout 3 | **Designed** | LY 6.5.2; TX 6.8 | LY-M1; LY-1 `css-inline` 92% | map: Covered; RWC: Designed |
| CG-4.16 | Positioned layout 3 | **Designed** | LY 6.5.7, 6.10 | LY-M1; LY-1 `css-position` 97% | map: Covered; RWC: Designed |
| CG-4.17 | Positioned layout 4 (top layer) | **Designed** | LY 6.5.7, 6.9; DM 6.12; AN 6.9 (AN-D9) | LY-M1, DM-M2 | map: Covered; RWC: Designed |
| CG-4.18 | Anchor positioning 1 | **Designed** | LY 6.9 rule 5 | LY-M2; LY-1 `css-anchor-position` 92% | map: Covered; RWC: Designed |
| CG-4.19 | Anchor positioning 2 | **Held** | LY-D10 ("anchored container queries") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-4.20 | Overflow 3 | **Designed** | LY 6.10 (LY-D13) | LY-M1; LY-1 `css-overflow` 92% | map: Covered; RWC: Designed |
| CG-4.21 | Overflow 4 | **Owned** | LY 6.7, 7 anchor (Overflow 4) | LY-M2 fragmentation; LY-1 `css-overflow` 92% | map: Covered; RWC: Owned |
| CG-4.22 | Overflow 5 (CSS carousels) | **Held** | LY-D10 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-4.23 | Logical properties | **Partial** | ST-M1 webref parsing; LY 6.8 (algorithms in logical coordinates) **→ FI-23** | ST-M1 | map: Partial |
| CG-4.24 | Writing modes 3/4 | **Designed** | LY 6.8; TX 6.6 | LY-M2; LY-1 `css-writing-modes` 92% | map: Covered; RWC: Designed |
| CG-4.25 | Containment 1/2 | **Designed** | LY 6.9 | LY-M2; LY-16 | map: Covered; RWC: Designed |
| CG-4.26 | `will-change` | **Designed** | GR 6.3 (GR-D5) | GR-M1 | map: Covered; RWC: Designed |
| CG-4.27 | Shapes 1 | **Owned** | LY 6.5.1 (float area) | LY-M1 exit names shapes; LY-1 `css-shapes` 92% | map: Covered; RWC: Owned |
| CG-4.28 | `shape()` function | **Partial** | — **→ FI-24** | — | map: Missing; VMW: Actually partial |
| CG-4.29 | Exclusions | **Held** | LY-D10 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-4.30 | Regions | **Held** | LY-D10 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-4.31 | Page floats, line grid, rhythmic sizing, template layout, round display | **Refused** | LY-D9 (page floats explicitly not admitted); others LY-D10 (rule applies, not named) | — | map: Excluded; VCE: Confirmed |
| CG-4.32 | CSS Layout API | **Held** | LY-D10 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-4.33 | Quirks mode | **Designed** | ST 6.6, 6.6a; LY 6.3 | ST-M1, LY-M1 | map: Covered; RWC: Designed |

### CG 5. CSS: scrolling, fragmentation and paged media

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-5.1 | Scroll snap 1 | **Designed** | LY 6.10; GR 6.8 | LY-M2; LY-1 `css-scroll-snap` 92% | map: Covered; RWC: Designed |
| CG-5.2 | Scroll snap 2 | **Held** | LY 6.10 ("`scrollsnapchange` and `scrollsnapchanging` are admitted per LY-D10") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-5.3 | Scroll anchoring | **Designed** | LY 6.10 | LY-M2; LY-1 `css-scroll-anchoring` 92% | map: Covered; RWC: Designed |
| CG-5.4 | Overscroll behaviour | **Partial** | GR 7 anchor table ("Overscroll Behavior") **→ FI-24** | GR-M1 | map: Partial |
| CG-5.5 | Scrollbar styling | **Partial** | LY-D11 (`scrollbar-width` thin 8†, none 0); GR 6.28 (scrollbars painted as layers) **→ FI-24** | LY-M1 | map: Partial |
| CG-5.6 | Smooth scrolling and scroll APIs | **Designed** | LY 6.15, 6.10; GR 6.8, 6.28 | LY-M2; LY-1 `cssom-view` 95% | map: Covered; RWC: Designed |
| CG-5.7 | Spatial navigation | **Held** | — — rule only, no per-feature record **→ FI-37** | — | map: Missing; VMW: Held (rule) |
| CG-5.8 | Fragmentation 3 | **Designed** | LY 6.7 (LY-D8) | LY-M2; LY-1 `css-break` 92% | map: Covered; RWC: Designed |
| CG-5.9 | Fragmentation 4 | **Held** | LY 6.7 ("`margin-break` … admitted by LY-D10's rule") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-5.10 | Paged media 3 | **Designed** | LY 6.7 (LY-D9) | LY-M2; LY-1 `css-page` 90% | map: Covered; RWC: Designed |
| CG-5.11 | Generated content for paged media | **Refused** | LY-D9 (not admitted; reserved-area slots from LY-M0) | — | map: Excluded; VCE: Confirmed |

### CG 6. CSS: text, fonts, lists and generated content

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-6.1 | Text 3 | **Designed** | TX 6.6 (`hg-unicode`, `hg-ucd`); LY 6.5.2 (`hg-layout`) | TX-M1, LY-M1; TX-2 `css-text`; LY-1 `css-text` 92% | map: Covered; RWC: Designed |
| CG-6.2 | Text 4 | **Designed** | LY 6.5.2, LY-D16; TX 6.6, 6.18 | LY-M1 (`balance`), LY-M3 (`pretty`), TX-M2 | map: Covered; RWC: Designed |
| CG-6.3 | Text decoration 3 | **Partial** | TX 6.18 (owner "Layout and paint"; TX supplies underline metrics, emphasis glyphs) **→ FI-24** | — | map: Partial |
| CG-6.4 | Text decoration 4 | **Partial** | TX 6.18 (glyph outlines for skip-ink) **→ FI-24** | — | map: Partial |
| CG-6.5 | Text size adjust | **Held** | — — rule only, no per-feature record **→ FI-33** | — | map: Missing; VMW: Held (rule) |
| CG-6.6 | Fonts 3 | **Designed** | TX 6.4 (`hg-font-match`) | TX-M1; TX-2 `css-fonts` | map: Covered; RWC: Designed |
| CG-6.7 | Fonts 4 | **Designed** | TX 6.4, 6.5 | TX-M1/M2; TX-2 `css-fonts` | map: Covered; RWC: Designed |
| CG-6.8 | Fonts 5 | **Owned** | TX 4.2, 6.4, 6.8 (`hg-font-match`, metrics query) | TX-M2; TX-2 `css-fonts` | map: Covered; VCE: Confirmed covered; RWC: Owned |
| CG-6.9 | Font loading API | **Designed** | TX 6.5 (`hg-font-load`) | TX-M2; TX-2 `css-font-loading` | map: Covered; RWC: Designed |
| CG-6.10 | Web font formats | **Designed** | TX 6.2 (`.hgfont`, `hg-fontgen`) | TX-M1; TX-5 100% WOFF2 suites | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-6.11 | Variable fonts | **Designed** | TX 6.2 | TX-M2; avar2 TX-M4 | map: Covered; RWC: Designed |
| CG-6.12 | Colour font formats | **Designed** | TX 6.2, 6.3; GR 6.6 | TX-M2; TX-5 COLRv1 reftests 100% | map: Covered; RWC: Designed |
| CG-6.13 | Text shaping and layout tables | **Designed** | TX 6.7, TX-D8 (`hg-shape`) | TX-M1/M2; TX-3, TX-4 | map: Covered; RWC: Designed |
| CG-6.14 | Unicode text algorithms | **Designed** | TX 6.6, TX-D11 (`hg-unicode`) | TX-M0/M1; TX-1 100% | map: Covered; RWC: Designed |
| CG-6.15 | Hyphenation dictionaries | **Designed** | TX 6.6, TX-D14 (`hg-hyphen`) | TX-M2; TX-3 review gate | map: Covered; RWC: Designed |
| CG-6.16 | Incremental Font Transfer | **Owned** | TX 6.2, 9.1 (`hg-font-load`) | TX-M4 "IFT shipped if its gate passed" | map: Covered; RWC: Owned |
| CG-6.17 | Local Font Access | **Refused** | TX-D17 ("not implemented; per-site local-font grants from site settings") | — | map: Excluded; VCE: Confirmed |
| CG-6.18 | Ruby layout | **Owned** | LY 6.5.2, LY 7 anchor "Ruby 1" (`hg-layout`) | LY-1 `css-ruby` 92% | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-6.19 | Lists and counters 3 | **Mentioned** | LY 7 anchor "CSS Lists 3"; LY 9.3 box-tree track **→ FI-23** | — | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins) |
| CG-6.20 | Counter styles 3 | **Partial** | Implied by Lists 3 (LY) and webref parsing **→ FI-23** | — | map: Partial |
| CG-6.21 | Generated content 3 | **Partial** | LY 6.3 (`content` in the box-tree "affects" category; pseudo-element boxes); AX 6.3 (CSS Content 3 alt text) **→ FI-23** | LY-M1 | map: Partial |
| CG-6.22 | Custom Highlight API | **Designed** | ED 6.7 (`hg-caret`; map cites 6.10, which is clipboard) | ED-M2; ED-1 `css/css-highlight-api/` 93% | map: Covered; RWC: Designed |

### CG 7. CSS: transforms, animation and visual effects

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-7.1 | Transforms 1 | **Designed** | GR 6.4 (`hg-raster`); AN 6.2 (`hg-interp`) | GR-M1; GR-1 `css-transforms`; AN-1 subtests 95% | map: Covered; RWC: Designed |
| CG-7.2 | Transforms 2 | **Designed** | GR 6.3 (`hg-paint`) | GR-M2 "3D transforms"; GR-1 `css-transforms` | map: Covered; RWC: Designed |
| CG-7.3 | Transitions 1 | **Designed** | AN 6.4 (`hg-anim::css`); ST 6.9 | AN-M1; AN-1 `css-transitions` 97% | map: Covered; RWC: Designed |
| CG-7.4 | Transitions 2 | **Designed** | AN 6.4, AN-D9; ST 6.10 | AN-M1; AN-1 `css-transitions` 97% | map: Covered; RWC: Designed |
| CG-7.5 | Animations 1 | **Designed** | AN 6.4 (`hg-anim::css`) | AN-M1; AN-1 `css-animations` 97% | map: Covered; RWC: Designed |
| CG-7.6 | Animations 2 | **Designed** | AN 6.4, AN-D9 | AN-M1; AN-1 `css-animations` 97% | map: Covered; RWC: Designed |
| CG-7.7 | Web Animations 1 | **Designed** | AN 6.1, 6.2 (`hg-anim`, `hg-anim-time`) | AN-M1; AN-1 `web-animations` 97% | map: Covered; RWC: Designed |
| CG-7.8 | Web Animations 2 | **Held** | AN-D9 ("`GroupEffect`, `SequenceEffect` … Held"; iteration composite internal for SMIL) | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-7.9 | Scroll-driven animations | **Designed** | AN 6.6, AN-D8; LY 6.10; GR 6.8 | AN-M2; AN-1 `scroll-animations` 92%; AN-2 Interop | map: Covered; RWC: Designed |
| CG-7.10 | Animation Worklet | **Refused** | AN-D9 ("Not implemented") | — | map: Excluded; VCE: Confirmed |
| CG-7.11 | Motion path | **Designed** | AN 6.2, 6.5 (class M), AN-D4 | AN-M1/M2; AN-1 `css/motion` subtests 95%; AN-7 | map: Covered; RWC: Designed |
| CG-7.12 | View transitions 1 (same-document) | **Designed** | AN 6.7, AN-D7 (`hg-view-transition`); GR 6.8 | AN-M2, GR-M3; AN-1 `css-view-transitions` 92% | map: Covered; RWC: Designed |
| CG-7.13 | View transitions 2 | **Designed** | AN 6.7 | AN-M3; AN-53 | map: Covered; RWC: Designed |
| CG-7.14 | Filter effects 1 | **Designed** | GR 6.4, GR-D18 | GR-M2; GR-1 `filter-effects`; SV-2 | map: Covered; RWC: Designed |
| CG-7.15 | Filter effects 2 | **Designed** | GR 6.3, 6.7 (`hg-layerize`) | GR-M2 "Filter Effects 1 and 2"; GR-1 `filter-effects` | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-7.16 | Compositing and blending | **Designed** | GR 6.3, 6.4 (`hg-raster`) | GR-M2; GR-1 `compositing` | map: Covered; RWC: Designed |
| CG-7.17 | Masking | **Designed** | GR 6.4, 6.16; SV 6.5 | GR-M2 "CSS Masking"; GR-1 `css-masking`; SV-2 | map: Covered; RWC: Designed |
| CG-7.18 | Geometry interfaces | **Partial** | SV 4.2 (pinned input), SV 6.6 (`DOMRect`, `DOMMatrix` as double) **→ FI-14** | — | map: Partial |

### CG 8. CSS: object models, Houdini and observers

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-8.1 | CSSOM | **Designed** | ST 6.6, 6.11 (`hg-cssom`) | ST-M2 "CSSOM complete"; ST-3 `cssom` 97% | map: Covered; RWC: Designed |
| CG-8.2 | CSSOM View | **Designed** | LY 6.15, 6.1 (`hg-layout-query`) | LY-M1 (getters, forcing table); LY-1 `cssom-view` 95% | map: Covered; RWC: Designed |
| CG-8.3 | CSS Typed OM | **Partial** | ST 9.3 ("Typed OM if admitted"); Q-ST-14 open **→ FI-23** | — | map: Partial |
| CG-8.4 | CSS Painting API | **Held** | ST-D18 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-8.5 | Worklets | **Designed** | DM 6.7, 6.8 (`hg-workers`, `hg-agents`) | — (no milestone names worklets; `worklets/` not a DM-1 floor) | map: Covered; RWC: Designed |
| CG-8.6 | CSS parser API | **Held** | ST-D18 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-8.7 | Resize Observer | **Designed** | LY 6.15; GR 6.3 | LY-M2; LY-1 `resize-observer` 95% | map: Covered; RWC: Designed |
| CG-8.8 | Intersection Observer | **Designed** | LY 6.15, LY-D10 owner exception for v2 | LY-M2; LY-1 `intersection-observer` 95% | map: Covered; RWC: Designed |
| CG-8.9 | Rendering timing hooks | **Designed** | DM 6.7 (`hg-script::rendering`); GR 6.9 | DM-M1/M2; DM-1 `html/webappapis/` 97% | map: Covered; RWC: Designed |

### CG 9. CSS and HTML: forms, UI and interactivity

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-9.1 | Basic UI 3 | **Partial** | ED (`caret-color`); LY 6.5.2 and TX 6.18 (`text-overflow`); GR-D17 (outline) **→ FI-23** | — | map: Partial |
| CG-9.2 | Basic UI 4 | **Partial** | LY-D11 (`appearance`, `field-sizing`); ED 6.x (`user-select`, `caret-shape`, `caret-animation`); LY 6.14 (`pointer-events`) **→ FI-23** | — | map: Partial |
| CG-9.3 | Customisable form controls | **Held** | LY 6.3, LY-D10 ("held by LY-D10 until it meets the admission rule") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold). *contradiction: DM 9.3 staffs customizable select and AX-O12 maps it; LY-D10 holds it* |
| CG-9.4 | Native form control rendering | **Designed** | LY 6.3, LY-D11 (`hg-box-tree`); DM 6.12 (`hg-forms`) | LY-M1 "form controls", DM-M2 | map: Covered; RWC: Designed |
| CG-9.5 | HTML rendering section | **Designed** | LY 6.3; ST 6.6, 6.11 | LY-M1 "Box tree complete for HTML Rendering", ST-M1 | map: Covered; RWC: Designed |
| CG-9.6 | `inert` | **Owned** | DM 6.1, 6.5 (`hg-html::focus`); LY 6.14 (`hg-hit`); AX 6.x | DM-M2; DM-1 `html/semantics/` 97% (`interactivity/inert`) | map: Covered; RWC: Owned |
| CG-9.7 | `interactivity: inert` | **Held** | ST-D18 (rule applies, not named) — rule only, no per-feature record **→ FI-37** | — | map: Excluded; VCE: Confirmed as hold, wrong status |
| CG-9.8 | Forced colours and high contrast | **Designed** | GR 6.23, GR-D25 (`hg-paint`); AX 6.11, AX-D18; ST 6.10 | GR-M2, AX-M2; AX-42 | map: Covered; RWC: Designed |
| CG-9.9 | Viewport and device environment | **Partial** | Visual viewport: GR 6.28, LY-D13; VirtualKeyboard and `interactive-widget` held (ED-D18) **→ FI-33** | — | map: Partial |
| CG-9.10 | EyeDropper API | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| CG-9.11 | Ink and pen rendering | **Held** | GR 6.9 and 9.1 ("OS-delegated ink … admission questions under … LY-D10's process") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |

### CG 10. SVG and MathML

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-10.1 | SVG 1.1 | **Designed** | SV 6.2–6.11; GR 6.16 (`hg-svg-paint`) | SV-M1/M2; SV-1 per-`svg/` floors | map: Covered; RWC: Designed |
| CG-10.2 | SVG 2 | **Designed** | SV (SVG 2 CR pin), SV-D14, SV-D13 | SV-M1/M2; SV-1 | map: Covered; RWC: Designed |
| CG-10.3 | SVG animation (SMIL) | **Designed** | AN 6.8, AN-D6; SV 6.9, SV-D4 | AN-M2, SV-M2; AN-1 `svg/animations` 90%; SV-5 | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-10.4 | SVG integration and image modes | **Designed** | SV 6.8, SV-D10–D12; MD-D20 | SV-M1/M2; SV-7 | map: Covered; RWC: Designed |
| CG-10.5 | SVG fonts | **Refused** | SV-D7 ("not supported") | — | map: Excluded; VCE: Confirmed |
| CG-10.6 | SVG markers and strokes | **Designed** | GR 6.16, 6.4 (`hg-path`, `hg-svg-paint`) | GR-M2, SV-M2 "markers"; SV-1 `painting` 90%; GR-7 | map: Covered; RWC: Designed |
| CG-10.7 | MathML Core | **Designed** | LY 6.5.9, LY-D21; TX 6.8 (`MATH`), TX-W2 | LY-M2; LY-1 `mathml` 90% | map: Covered; RWC: Designed |
| CG-10.8 | MathML 4 full | **Refused** | LY-D21 ("MathML outside Core is not implemented") | — | map: Excluded; VCE: Confirmed |

### CG 11. 2D and 3D graphics APIs

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-11.1 | Canvas 2D | **Designed** | GR 6.15 (GR-D12), `hg-canvas` | GR-M1 "Canvas 2D"; GR-1 `html/canvas/`; GR-8, GR-9 | map: Covered; RWC: Designed |
| CG-11.2 | Path2D | **Owned** | GR 7 anchor "HTML Standard canvas chapter", `hg-canvas` | GR-M1; GR-1 `html/canvas/` (path-objects tests live there) | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-11.3 | OffscreenCanvas | **Designed** | GR 6.15, `hg-canvas` | GR-M3 "OffscreenCanvas"; GR-1 `html/canvas/` | map: Covered; RWC: Designed |
| CG-11.4 | ImageBitmap | **Designed** | GR 6.15 surface table | GR-M1; GR-1 `html/canvas/`; GR-8 | map: Covered; RWC: Designed |
| CG-11.5 | Canvas HDR and float pixels | **Owned** | GR 6.15, 6.5 (E16) | GR-M1/M2; GR-1 `html/canvas/` | map: Covered; RWC: Owned |
| CG-11.6 | HTML-in-canvas | **Held** | GR 6.15, 9.1 ("if admitted under the layout spec's admission rule") | When admitted | map: Deferred (admission); VCE: Confirmed (admission hold) |
| CG-11.7 | WebGL 1.0 | **Designed** | GR 6.14 (GR-D9), `hg-webgl`, `hg-glsl-es` | GR-M2; GR-4 Khronos ≥ 99.5% | map: Covered; RWC: Designed |
| CG-11.8 | WebGL 2.0 | **Designed** | GR 6.14 (GR-D9), `hg-webgl` | GR-M2; GR-4 Khronos WebGL 2 ≥ 99.5% | map: Covered; RWC: Designed |
| CG-11.9 | WebGL extensions | **Partial** | — **→ FI-24** | — | map: Missing; VMW: Actually partial |
| CG-11.10 | WebGPU | **Designed** | GR 6.13 (GR-D11), `hg-webgpu` | GR-M3; GR-3 CTS ≥ 99%; GR-S6 | map: Covered; RWC: Designed |
| CG-11.11 | WGSL | **Designed** | GR 6.12 (GR-D7), `hg-wgsl`, `hg-shader-ir` | GR-M1 (internal), GR-M3; GR-3 | map: Covered; RWC: Designed |
| CG-11.12 | WebGPU optional features | **Designed** | GR 6.13 (GR-D11) | GR-M3; GR-3 | map: Covered; RWC: Designed |
| CG-11.13 | Shape Detection | **Held** | — — rule only, no per-feature record **→ FI-28** | — | map: Missing; VMW: Held (rule) |
| CG-11.14 | Image decode and lazy decode | **Designed** | MD 6.6 `hg-imgpipe`; LY 6.15 lazy-load observer `hg-layout-query` | MD-M1, LY-M2; LY anchor "HTML lazy loading" WPT | map: Covered; RWC: Designed |

### CG 12. Image formats and colour management

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-12.1 | JPEG (baseline and progressive) | **Designed** | MD 6.2 (MD-D1) | MD-M1; MD-2 (T.83 tolerance) | map: Covered; RWC: Designed |
| CG-12.2 | PNG (3rd edition) | **Designed** | MD 6.2, 6.5 | MD-M1; MD-1 PngSuite | map: Covered; RWC: Designed |
| CG-12.3 | APNG | **Designed** | MD 6.2, 6.6 (MD-D19) | MD-M1; MD-1 PngSuite | map: Covered; RWC: Designed |
| CG-12.4 | GIF | **Designed** | MD 6.2, 6.6 (MD-D19) | MD-M1 | map: Covered; RWC: Designed |
| CG-12.5 | WebP | **Owned** | MD 6.2 (MD-D1) | MD-M1; MD-1 RFC 9649 lossless cases | map: Covered; VCE: Confirmed covered; RWC: Owned |
| CG-12.6 | AVIF | **Designed** | MD 6.2, MD-D2 gate | MD-M1 (still), MD-M2; MD-1 Argon/AOM | map: Covered; RWC: Designed; FC. *FC 6e: as AV1* |
| CG-12.7 | JPEG XL | **Designed** | MD-D2 | MD-M3; 18181-3, MD-50 | map: Covered; RWC: Designed; FC. *FC 8a–8c: JPEG XL is on in Chrome 155 (stable about 6 October 2026); two engines now* |
| CG-12.8 | HEIC/HEIF | **Refused** | MD-D1 ("Not exposed … HEIC (HEVC patents; Safari only, through the OS)") | — | map: Excluded; VCE: Confirmed; FC. *FC 6c: HEVC pools; no software exemption stated* |
| CG-12.9 | TIFF | **Refused** | MD-D1 (TIFF decoded only for `sbix` glyphs) | — | map: Excluded; VCE: Confirmed |
| CG-12.10 | JPEG 2000 | **Refused** | MD-D1 (not exposed; `hg-jpx` for PDF only) | — | map: Excluded; VCE: Confirmed |
| CG-12.11 | BMP, ICO, CUR | **Designed** | MD 6.2 (MD-D1) | MD-M1 | map: Covered; RWC: Designed |
| CG-12.12 | SVG as image | **Designed** | SV 6.8 (SV-D10–D12); MD 6.1 (MD-D20) | SV-M1 (renderer), SV-M2 (media process); SV-53 | map: Covered; RWC: Designed |
| CG-12.13 | HDR gain-map images | **Designed** | MD 6.5 (MD-D9) | MD-M2; MD-6 | map: Covered; RWC: Designed |
| CG-12.14 | ICC colour management | **Designed** | MD 6.5 (MD-D8), `hg-colour` | MD-M1; MD-5 ΔE00 ≤ 0.5 | map: Covered; RWC: Designed |
| CG-12.15 | Wide-gamut and HDR output | **Designed** | MD 6.5 (MD-D8, MD-D9); GR 6.5 (GR-D16) | MD-M2, GR-M2; MD-6 | map: Covered; RWC: Designed |
| CG-12.16 | Image density correction | **Mentioned** | MD 6.6 (one clause) **→ FI-25** | — | map: Covered; VCE: Only partial; RWC: Mentioned (checks differ; the later check wins) |
| CG-12.17 | Image MIME sniffing | **Designed** | NW 6.2 `hg-net-wire` (MIME Sniffing); MD 6.14 | NW-M1, MD-M1; MD-20 | map: Covered; RWC: Designed |

### CG 13. Video and audio formats

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-13.1 | MP4 / ISO BMFF container | **Owned** | MD 6.2 container table | MD-M2; MD-3; WPT `media-source/` | map: Covered; RWC: Owned |
| CG-13.2 | WebM / Matroska container | **Owned** | MD 6.2 container table | MD-M2; MD-3 | map: Covered; RWC: Owned; FC. *FC 6f: the VP9 grant question applies to WebM's VP9 payloads* |
| CG-13.3 | Ogg container | **Owned** | MD 6.2 container table | MD-M2; MD-3 | map: Covered; RWC: Owned |
| CG-13.4 | HLS native playback | **Designed** | MD 6.7 (MD-D21) | MD-M3 | map: Covered; RWC: Designed |
| CG-13.5 | MPEG-DASH | **Refused** | MD 6.20 ("DASH stays in page script (no browser ships native DASH)") | — | map: Excluded; VCE: Confirmed |
| CG-13.6 | H.264 / AVC | **Designed** | MD 6.2 (MD-D3), `hg-h264` | MD-M2 (OS), MD-M3 (Linux per licence); MD-1 H.264.1 | map: Covered; RWC: Designed; FC. *FC 6a: Via LA pool; no stated patent expiry* |
| CG-13.7 | H.265 / HEVC | **Designed** | MD 6.2 (MD-D3), G57 | MD-M2 | map: Covered; RWC: Designed; FC. *FC 6c: Access Advance and Via LA pools* |
| CG-13.8 | H.266 / VVC | **Refused** | — | — | map: Missing; VMW: Actually excluded; FC. *FC 6d: VVC pools exist; stays not exposed* |
| CG-13.9 | VP8 | **Designed** | MD 6.2, MD-D4 | MD-M2; MD-1 RFC 6386 vectors | map: Covered; RWC: Designed |
| CG-13.10 | VP9 | **Owned** | MD 6.2 | MD-M2; MD-1 VP9 test vectors | map: Covered; RWC: Owned; FC. *FC 6f: Google's WebM grant does not cover an independent VP9 decoder; Sisvel runs a VP9 pool. Legal review needed* |
| CG-13.11 | AV1 | **Designed** | MD 6.2; S1 13.2 | MD-M1 (intra), MD-M2; MD-1 Argon | map: Covered; RWC: Designed; FC. *FC 6e: AOM licence covers any implementation; Sisvel claims are a business risk* |
| CG-13.12 | Theora | **Refused** | MD-D1 (not exposed) | — | map: Excluded; VCE: Confirmed |
| CG-13.13 | AAC (LC, HE-AAC) | **Designed** | MD 6.2 (MD-D3) | MD-M2 (OS), MD-M3; MD-2 | map: Covered; RWC: Designed; FC. *FC 6b: Via LA pool; no stated patent expiry* |
| CG-13.14 | MP3 | **Owned** | MD 6.2 | MD-M2; MD-2 ISO tolerance | map: Covered; VCE: Confirmed covered; RWC: Owned |
| CG-13.15 | Opus | **Designed** | MD 6.2, MD-D4 | MD-M2; MD-2 RFC 8251 vectors | map: Covered; RWC: Designed; FC. *FC 6g: royalty-free grants confirmed* |
| CG-13.16 | Vorbis | **Owned** | MD 6.2 | MD-M2; MD-2 (−90 dBFS of float64 reference) | map: Covered; RWC: Owned |
| CG-13.17 | FLAC | **Owned** | MD 6.2 | MD-M2; MD-1 FLAC files | map: Covered; RWC: Owned |
| CG-13.18 | PCM / WAV, G.711, G.722 | **Owned** | MD 6.2; MD 6.10 (MD-D30) | MD-M2 ("PCM"; WebRTC "G.711") | map: Covered; RWC: Owned |
| CG-13.19 | AC-3 / E-AC-3 / Dolby | **Refused** | — | — | map: Missing; VMW: Actually excluded (needs a record). *excluded by MD-D1's closed set, but no licence or OS-decoder route is recorded* |
| CG-13.20 | ALAC | **Refused** | — | — | map: Missing; VMW: Actually excluded |
| CG-13.21 | Media MIME and codec strings | **Designed** | MD 6.15 (MD-D11) | MD-M2; MD-30 | map: Covered; RWC: Designed |

### CG 14. HTML media elements and streaming

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-14.1 | `<video>` and `<audio>` | **Designed** | MD 6.7 (`hg-media`, MD-D12) | MD-M2; MD-4 `html/semantics/embedded-content/media-elements/`; DM-1 `html/semantics/` | map: Covered; RWC: Designed |
| CG-14.2 | Default media controls | **Designed** | MD 6.12 (`hg-media` UA shadow tree); shell UI design (exit affordance) | MD-M2 "native controls (MD-12)"; MD-12 | map: Covered; RWC: Designed |
| CG-14.3 | Text tracks | **Designed** | MD 6.12 (`hg-media`, generated WebVTT parser) | MD-M2; MD-4 media-elements/ (track tests); DM-1 `html/semantics/` | map: Covered; RWC: Designed |
| CG-14.4 | WebVTT | **Designed** | MD 6.12; ST 6.9 (`::cue`) | MD-M2 "WebVTT with OS caption preferences"; MD-4 `webvtt/` | map: Covered; RWC: Designed |
| CG-14.5 | Media Fragments | **Partial** | SV Q-SV-11 (SVG `xywh=`, `t=` only) **→ FI-25** | — | map: Partial |
| CG-14.6 | Media Source Extensions | **Designed** | MD 6.7 (MD-D22, `hg-media`) | MD-M2; MD-4 `media-source/` | map: Covered; RWC: Designed |
| CG-14.7 | MSE in workers and ManagedMediaSource | **Owned** | MD 6.7, 6.20 (MD-D21) | MD-M2 "MSE, ManagedMediaSource"; MD-4 `media-source/` (incl. dedicated-worker tests) | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-14.8 | Encrypted Media Extensions | **Designed** | MD 6.11; S1 13.1a (D-S7) | MD-M2 (EME Clear Key, PlayReady); S1 M7; MD-4 `encrypted-media/` | map: Covered; RWC: Designed |
| CG-14.9 | Content Decryption Modules | **Designed** | S1 13.1a (D-S7); MD 6.11 (MD-D32) | S1 M7; MD-M2 PlayReady exit (needs MD-S5) | map: Covered; RWC: Designed |
| CG-14.10 | Media Capabilities | **Designed** | MD 6.7, 6.15 (MD-D11 policy table) | MD-M2 (MD-30 measured); MD-30 | map: Covered; RWC: Designed |
| CG-14.11 | Media Playback Quality | **Partial** | MD 3 (cited as "the only standard health metric") **→ FI-25** | — | map: Partial |
| CG-14.12 | `requestVideoFrameCallback` | **Designed** | GR 6.11 (GR-D26); MD 6.7, 6.15 | GR-M2 "video layers"; — no rVFC floor | map: Covered; RWC: Designed |
| CG-14.13 | Media Session | **Designed** | MD 6.12 (broker to OS controls) | MD-M2 "Media Session"; MD-4 `mediasession/` | map: Covered; RWC: Designed |
| CG-14.14 | Picture-in-Picture (video) | **Designed** | MD 6.12 (shell-owned OS window) | MD-M2 "Picture-in-Picture"; MD-4 `picture-in-picture/` | map: Covered; RWC: Designed |
| CG-14.15 | Document Picture-in-Picture | **Designed** | MD 6.12 | MD-M2 "Picture-in-Picture" (implicit); anchor "Document PiP" | map: Covered; RWC: Designed |
| CG-14.16 | Remote Playback | **Missing** | — **→ FI-32** | — | map: Missing; VMW: Confirmed missing |
| CG-14.17 | Presentation API | **Held** | — — rule only, no per-feature record **→ FI-32** | — | map: Missing; VMW: Held (rule) |
| CG-14.18 | Audio Output Devices | **Designed** | MD 6.8 (audio service, broker chooser) | MD-M2 "audio service"; — no `audio-output/` floor | map: Covered; VCE: Confirmed covered; RWC: Designed |
| CG-14.19 | Audio Session | **Held** | — — rule only, no per-feature record **→ FI-25** | — | map: Missing; VMW: Held (rule) |
| CG-14.20 | Autoplay policy | **Designed** | MD 6.12 (MD-D13) | MD-M2 "autoplay rule"; MD-4 `autoplay-policy-detection/` | map: Covered; RWC: Designed |
| CG-14.21 | WebCodecs | **Designed** | MD 6.7 | MD-M2 decode, MD-M3 encode; MD-4 `webcodecs/` | map: Covered; RWC: Designed |
| CG-14.22 | ImageDecoder | **Owned** | MD 6.20 (WebCodecs row, image decoders in media process) | MD-M2 "WebCodecs decode"; MD-4 `webcodecs/` (image-decoder tests) | map: Covered; VCE: Only partial; RWC: Owned (checks differ; the later check wins) |
| CG-14.23 | Web Audio 1.0/1.1 | **Designed** | MD 6.8 (`hg-webaudio`, MD-D14) | MD-M2; MD-4 `webaudio/`; MD-30, MD-40 | map: Covered; RWC: Designed |
| CG-14.24 | AudioWorklet | **Designed** | MD 6.8 (MD-D31) | MD-M2 "Web Audio with AudioWorklet"; MD-9, MD-10; gated by MD-S8 | map: Covered; RWC: Designed |
| CG-14.25 | Web MIDI | **Designed** | DV 6.6, 6.10 (device-access service, DV-D9) | DV-M1, DV-M2 (behind Experimental setting); DV-M3 exposure decision DV-E5 | map: Covered; RWC: Designed |

### CG 15. Media capture and real-time communication

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-15.1 | Media Capture and Streams | **Designed** | MD 6.10 (capture service, broker) | MD-M2 "capture service"; MD-4 `mediacapture-streams/` | map: Covered; RWC: Designed |
| CG-15.2 | Media Capture extensions | **Missing** | — **→ FI-27** | — | map: Missing; VMW: Confirmed missing |
| CG-15.3 | MediaStream Image Capture | **Held** | — — rule only, no per-feature record **→ FI-27** | — | map: Missing; VMW: Held (rule) |
| CG-15.4 | Capture from DOM elements | **Partial** | GR 6.15, 6.27 (`canvas.captureStream()`) **→ FI-27** | GR-M3 | map: Partial |
| CG-15.5 | Screen Capture | **Designed** | MD 6.10; GR 6.27 (capture split) | MD-M2; GR-M3 "capture"; GR-24 | map: Covered; RWC: Designed |
| CG-15.6 | Capture Handle | **Held** | — — rule only, no per-feature record **→ FI-27** | — | map: Missing; VMW: Held (rule) |
| CG-15.7 | Region Capture | **Owned** | GR 6.27 (compositor table row, `cropTo`) | GR-M3; GR-24 | map: Covered; RWC: Owned |
| CG-15.8 | Element Capture | **Designed** | GR 6.27 (`restrictTo`) | GR-M3; GR-24 | map: Covered; RWC: Designed |
| CG-15.9 | Captured Surface Control and captured mouse events | **Held** | — — rule only, no per-feature record **→ FI-27** | — | map: Missing; VMW: Held (rule) |
| CG-15.10 | MediaStreamTrack insertable streams | **Mentioned** | MD 6.20 (`MediaStreamTrackProcessor` listed); MD 6.7 **→ FI-27** | — (no milestone names it; `mediacapture-insertable-streams/` not floored) | map: Covered; RWC: Mentioned |
| CG-15.11 | MediaRecorder | **Designed** | MD 6.7 | MD-M3 "MediaRecorder"; MD-4 `mediacapture-record/` | map: Covered; RWC: Designed |
| CG-15.12 | Content hints | **Missing** | — **→ FI-27** | — | map: Missing; VMW: Confirmed missing |
| CG-15.13 | HTML Media Capture | **Missing** | — **→ FI-5** | — | map: Missing; VMW: Confirmed missing |
| CG-15.14 | WebRTC 1.0 | **Partial** | NW 6.12 (ICE, DTLS, SRTP, SCTP); MD 6.10 (depacketizing, jitter buffers, encode, FEC); MD anchor table cites "WebRTC 1.0" **→ FI-27** | NW-M4, MD-M2 | map: Partial |
| CG-15.15 | RTCDataChannel | **Owned** | NW 6.12 (`hg-sctp`, RFC 8831) | NW-M4; NW-42 "data channels complete", WPT `webrtc/` floors at NW-M4 | map: Covered; RWC: Owned |
| CG-15.16 | WebRTC statistics | **Designed** | MD 6.10 | MD-M2 "WebRTC media"; MD-4 `webrtc/`; anchor WebRTC Statistics | map: Covered; RWC: Designed |
| CG-15.17 | WebRTC Encoded Transform | **Mentioned** | MD 6.10, 6.20 (one sentence) **→ FI-27** | — (MD-M2 says "WebRTC media" only; `webrtc-encoded-transform/` not floored) | map: Covered; RWC: Mentioned |
| CG-15.18 | WebRTC extensions, SVC, priority, ICE | **Missing** | — **→ FI-27** | — | map: Missing; VMW: Confirmed missing |
| CG-15.19 | WebRTC Identity | **Held** | — — rule only, no per-feature record **→ FI-27** | — | map: Missing; VMW: Held (rule) |
| CG-15.20 | WebRTC mandatory codecs and media stack | **Partial** | MD-D30 (VP8, Opus, G.711, H.264 per MD-D3); MD-D16 (`hg-voice` echo cancellation); MD 6.10 (jitter buffers, FEC, retransmission) **→ FI-27** | MD-M2/M3 | map: Partial |

### CG 16. XR, speech, printing and other rendering features

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| CG-16.1 | WebXR Device API | **Missing** | — **→ FI-36** | — | map: Missing; VMW: Confirmed missing |
| CG-16.2 | WebXR modules | **Missing** | — **→ FI-36** | — | map: Missing; VMW: Confirmed missing |
| CG-16.3 | Web Speech: synthesis | **Partial** | MD 6.20 ("speech synthesis and recognition (a separate spec …)"); extensions `tts` Stage 3 **→ FI-29** | — | map: Partial |
| CG-16.4 | Web Speech: recognition | **Partial** | MD 6.20; ML 1 ("speech recognition is 'an ML service feature under B10'") **→ FI-29** | — | map: Partial |
| CG-16.5 | Printing | **Designed** | LY 6.7 (LY-D8); GR 6.27; S1 13.2 (print UI Stage 2) | LY-M2 "paged media"; GR-M3 print raster; GR-28 | map: Covered; RWC: Designed |
| CG-16.6 | Save as PDF output | **Deferred** | PDF 6.19 (writer core Stage 1 for saving and printing PDFs); PD-M4 "Writer (Stage 2, with the print UI)"; Stage 1 13.2 | PD-M4 (Stage 2) | map: Deferred (Stage 2); VCE: Confirmed |
| CG-16.7 | Built-in PDF viewer | **Designed** | PD (PD-D1 to PD-D26; 6.15 viewer) | PD-M1, PD-M2 (Stage 1 M7) | map: Covered; RWC: Designed |
| CG-16.8 | Fullscreen rendering | **Designed** | DM 6.9 (element side), 6.18 `hg-webapi`; MD 6.12; ST 6.7.1 | DM-M2 "top layer"; MD-M2 | map: Covered; RWC: Designed |
| CG-16.9 | Scroll to text fragment rendering | **Designed** | ED 6.12 (`hg-find`); DM 6.9; LY 6.17 | ED-M2; ED anchor WPT `scroll-to-text-fragment/` | map: Covered; RWC: Designed |
| CG-16.10 | Find in page highlighting | **Designed** | ED 6.12 (ED-D17, `hg-find`); LY 6.9 | ED-M2 "find in page across frames"; WPT `html/editing/the-hidden-attribute/` | map: Covered; RWC: Designed |
| CG-16.11 | Reader view and text-only rendering | **Deferred, no owner** | Stage 1 §6 ("Save for offline … Reader rendering on top", Stage 2); PDF 9.1 ("Stage 2 with the reader view") **→ FI-30** | Stage 2 | map: Deferred (Stage 2); VCE: Confirmed (weak) (no spec has a reader-view row) |

## BF: Browser product features


### BF 1. Windows, tabs and sessions

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-1.1 | Multiple windows [TS] | **Designed** | Shell 6.3 `shell.window`, 6.8 | Stage 1, SH-M1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-1.2 | Tab strip basics [TS] | **Designed** | Shell 6.8 "The tab strip" | Stage 1, SH-M1 | map: Covered; RBP: Designed |
| BF-1.3 | Pinned tabs [TS] | **Mentioned** | Shell 6.8, SH-D27 **→ FI-17** | Stage 1, SH-M1 | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| BF-1.4 | Tab groups [TS] | **Deferred** | Shell 9.1, SH-D27; Q-SH-9 | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred |
| BF-1.5 | Saved and synced tab groups | **Deferred, no owner** | Shell 9.1 (groups Stage 2; sync Stage 3 or never) **→ FI-35** | Stage 2 / Stage 3 or never | map: Excluded; VCE: Confirmed-implicit; RBP: Deferred, no owner. *VCE: implicit refusal; RBP: saving a group is named nowhere and sync is open* |
| BF-1.6 | Two-level tab stacks [Diff] | **Refused** | (implicit) UI §8 Non-goals, UP5 **→ FI-38** | — | map: Excluded; VCE: Confirmed-implicit; RBP: Refused (implicit) |
| BF-1.7 | Vertical tabs | **Deferred** | Shell 9.1, SH-D27; UI R18 | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred. *contradiction: S1 §6 out-of-scope list vs SH-D27 Stage 2* |
| BF-1.8 | Split view / tab tiling [Diff→TS] | **Deferred** | Shell 9.1, SH-D27 | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred. *contradiction: as BF-1.7* |
| BF-1.9 | Workspaces / Spaces [Diff] | **Refused** | S1 §6 out-of-scope list | Out of scope Stages 1–2 | map: Excluded; VCE: Confirmed; RBP: Refused |
| BF-1.10 | Tab search / tab switcher [TS] | **Designed** | Shell 6.8 "Recently closed and tab search" | Stage 1, SH-M3 | map: Covered; RBP: Designed |
| BF-1.11 | Tab hover preview | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-1.12 | Mute tab or site [TS] | **Mentioned** | Shell 6.3, 6.8 **→ FI-17** | Stage 1, SH-M1 | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| BF-1.13 | Reopen closed tab or window [TS] | **Designed** | Shell 6.8 | Stage 1, SH-M3 | map: Covered; RBP: Designed |
| BF-1.14 | Session restore at startup [TS] | **Designed** | Shell 6.8 "Session store and restore", SH-21 | Stage 1, SH-M3 | map: Covered; RBP: Designed |
| BF-1.15 | Named saved sessions [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-1.16 | Crash recovery [TS] | **Designed** | Shell 6.8, 6.14, SH-21, SH-30 | Stage 1, SH-M3/SH-M4 | map: Covered; RBP: Designed |
| BF-1.17 | Link peek / glance [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-1.18 | Auto-archive of idle tabs [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-1.19 | Tab emoji / custom tab names | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-1.20 | Journeys / topic history [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-1.21 | Full screen and toolbar-in-full-screen [TS] | **Designed** | Shell 6.8 "Fullscreen and zoom" | Stage 1, SH-M1 | map: Covered; RBP: Designed |
| BF-1.22 | UI auto-hide / compact mode [Diff] | **Partial** | Shell 6.2, 6.9 Appearance **→ FI-17** | Stage 1 (density) | map: Partial |
| BF-1.23 | Tabs from other devices | **Deferred, no owner** | Shell 9.1, SH-D20; Q-SH-5 **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed; RBP: Deferred, no owner. *VCE read it as a confirmed exclusion; RBP: shell 9.1 says "Stage 3 or never" and Q-SH-5 is open, so not a refusal* |
| BF-1.24 | Open-tab list in the OS app switcher (mobile) | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |

### BF 2. Address bar, search and navigation

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-2.1 | Unified address and search bar [TS] | **Designed** | Shell 6.8 "The address bar" | Stage 1, SH-M1 | map: Covered; RBP: Designed |
| BF-2.2 | Suggestions [TS] | **Designed** | Shell 6.8; Q-UI-1 | Stage 1, SH-M1 | map: Covered; RBP: Designed |
| BF-2.3 | Default search engine choice [TS] | **Partial** | Shell 6.7, 6.8 ("chosen search engine"); EX 6.9 ("Izcal's search settings are the user's") **→ FI-18** | — | map: Partial |
| BF-2.4 | Search engine choice screen (EU DMA) [TS in EU] | **Missing** | — **→ FI-18** | — | map: Missing; VMB: Confirmed missing; FC. *FC 4c: not a legal duty for Izcal; only gatekeeper browsers (Chrome, Safari) must show it. Reframed as a first-run default-engine choice* |
| BF-2.5 | Site search keywords / shortcuts | **Missing** | — **→ FI-18** | — | map: Missing; VMB: Confirmed missing |
| BF-2.6 | Calculator and unit conversion in the address bar | **Missing** | — **→ FI-18** | — | map: Missing; VMB: Confirmed missing |
| BF-2.7 | Quick actions / command palette [Diff] | **Deferred** | Shell 9.1, SH-D27; Q-SH-8 | Stage 2 (trial) | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred |
| BF-2.8 | Voice search | **Missing** | — **→ FI-29** | — | map: Missing; VMB: Confirmed missing |
| BF-2.9 | Visual search (Lens) | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-2.10 | Security indicator and URL display [TS] | **Designed** | Shell 6.8, SH-D14; UI 4.1–4.2 | Stage 1, SH-M1/SH-M2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-2.11 | Address bar position (mobile) | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-2.12 | AI search mode in the address bar | **Refused** | (implicit) SH-D16; Shell 6.8 **→ FI-31** | — | map: Excluded; VCE: Confirmed-implicit; RBP: Refused (implicit) |
| BF-2.13 | Sponsored and contextual suggestions, with an off switch | **Refused** | Shell 6.8, 6.7 | — | map: Excluded; VCE: Confirmed; RBP: Refused |
| BF-2.14 | Home button / homepage [TS] | **Partial** | EX 6.9, EX-D15 (homepage override prompt, Stage 2) **→ FI-18** | — | map: Partial |
| BF-2.15 | Mouse gestures [Diff] | **Missing** | — **→ FI-21** | — | map: Missing; VMB: Confirmed missing |
| BF-2.16 | Search engine bundled with the browser [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-2.17 | Desktop search bar outside the browser | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |

### BF 3. Bookmarks, history, reading list and start page

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-3.1 | Bookmarks and bookmark bar [TS] | **Designed** | Shell 6.7 "Surfaces", SH-D8, SH-D9 | Stage 1, SH-M3/SH-M5 | map: Covered; RBP: Designed |
| BF-3.2 | Bookmark manager with tags | **Partial** | Shell 6.7, 6.12 **→ FI-17** | Stage 1, SH-M5 | map: Partial |
| BF-3.3 | History view and search [TS] | **Designed** | Shell 6.7, SH-19, SH-20 | Stage 1, SH-M3/SH-M5 | map: Covered; RBP: Designed |
| BF-3.4 | History statistics [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-3.5 | Reading list / read later | **Partial** | S1 §6 "Save for offline" (Stage 2); UI R18 **→ FI-17** | Stage 2 (offline save only) | map: Partial |
| BF-3.6 | Collections / pinboards [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-3.7 | New tab page / start page [TS] | **Designed** | Shell 6.7 "Surfaces" | Stage 1, SH-M5 | map: Covered; RBP: Designed |
| BF-3.8 | Speed dial tiles | **Partial** | Shell 6.7 **→ FI-17** | Stage 1 | map: Partial |
| BF-3.9 | News feed on the new tab page | **Refused** | (implicit) SH-D29; Shell 6.7 **→ FI-38** | — | map: Excluded; VCE: Confirmed-implicit; RBP: Refused (implicit) |
| BF-3.10 | Dashboard widgets on the start page [Diff] | **Missing** | — (Shell 6.7, SH-D29 refuse networked only) **→ FI-17** | — | map: Excluded; VCE: Misread; RBP: Misread → Missing |
| BF-3.11 | Start page shows bookmarks, reading list and history | **Partial** | Shell 6.7 **→ FI-17** | Stage 1 | map: Partial |
| BF-3.12 | Page change notifications [Diff] | **Partial** | Shell 6.7 "Content fingerprint and health", SH-D9 **→ FI-17** | Stage 1 | map: Partial |
| BF-3.13 | Wayback Machine fallback on a 404 [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-3.14 | Recent tabs and "continue where you left off" cards | **Partial** | Shell 6.7 (new-tab page recently closed), 6.8 **→ FI-17** | Stage 1 | map: Partial |

### BF 4. Profiles, accounts, sync and cross-device

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-4.1 | Multiple profiles [TS] | **Partial** | Shell 4.1 ("profiles later"), 6.2 ("One shell renderer per profile"), SH-D26; S1 13.2 "Profiles" (format) **→ FI-10** | — | map: Partial |
| BF-4.2 | Guest mode | **Missing** | — **→ FI-10** | — | map: Missing; VMB: Confirmed missing |
| BF-4.3 | Browser sign-in / account [TS] | **Deferred, no owner** | SH-D20; Shell 9.1; Q-SH-5 **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed; RBP: Deferred, no owner. *VCE: confirmed exclusion; RBP: not a refusal while Q-SH-5 is open* |
| BF-4.4 | Sync [TS] | **Deferred, no owner** | Shell 9.1, SH-D20; Q-SH-5; S1 §9 **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred, no owner |
| BF-4.5 | End-to-end encrypted sync [Diff] | **Deferred, no owner** | As BF-4.4 **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed-implicit; RBP: Deferred, no owner |
| BF-4.6 | Send tab to device | **Deferred, no owner** | Shell 9.1 "Sync and multi-device" **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed; RBP: Deferred, no owner. *VCE: confirmed exclusion; RBP: not a refusal while Q-SH-5 is open* |
| BF-4.7 | Send files and notes between devices [Diff] | **Deferred, no owner** | Shell 9.1 "Sync and multi-device" **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed; RBP: Deferred, no owner. *VCE: confirmed exclusion; RBP: not a refusal while Q-SH-5 is open* |
| BF-4.8 | Profile-based work/personal separation | **Partial** | SH-D26 (named containers personal/work/banking/shopping); NW-D20 (ambient authentication allowlist) **→ FI-10** | Stage 1 (containers, read-only) | map: Partial |
| BF-4.9 | Sync troubleshooting and data reset | **Deferred, no owner** | As BF-4.4 **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed-implicit; RBP: Deferred, no owner |
| BF-4.10 | Third-party sign-in prompts (FedCM UI) | **Mentioned** | — **→ FI-13** | — | map: Missing; VMB: Actually partial. *named only (VMB)* |
| BF-4.11 | Export my data | **Designed** | Shell 6.12 "Export", SH-D20 | Stage 1, SH-M3 (M7) | map: Covered; RBP: Designed |

### BF 5. Downloads, files, saving, printing and PDF

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-5.1 | Downloads UI [TS] | **Designed** | Shell 6.10, 6.3 `shell.downloads`, SH-D18 | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-5.2 | Download location and ask-where-to-save [TS] | **Designed** | Shell 6.9 Downloads group | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-5.3 | Dangerous download warnings [TS] | **Designed** | Shell 6.10, SH-D18, SH-R7; PH-D7 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-5.4 | Save page [TS] | **Mentioned** | S1 §6 "Save for offline"; PH 6.4.10 **→ FI-17** | Stage 2 (snapshot only) | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| BF-5.5 | Print and print preview [TS] | **Designed** | LY 6.7, GR 6.27 (GR-M3); Shell 6.19 print UI | Stage 1 layout/raster; Stage 2 UI | map: Excluded; VCE: Misread; RBP: Misread → Designed. *engine half Designed (LY 6.7, GR 6.27); the print UI half is PO-9.14 (FI-8). VCE said Partial* |
| BF-5.6 | Save as PDF [TS] | **Designed** | PD 6.19, PD-D19–D21, PD-M4 | Stage 2 (writer core Stage 1) | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-5.7 | Built-in PDF viewer [TS] | **Designed** | PD 6.15, PD-D2 | Stage 1 (M7) | map: Covered; RBP: Designed |
| BF-5.8 | PDF form fill, annotate and sign | **Designed** | PD-D12, PD-D17, PD 9.5 | Stage 1 forms/annotate (PD-M3); Stage 2 sign | map: Covered; RBP: Designed. *contradiction: S1 §6 says annotate is Stage 2, PD 9.5 moves it to Stage 1 (PD-M3)* |
| BF-5.9 | PDF image alt text (AI) | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-5.10 | PDF summarise and ask | **Missing** | — (ML-D2 defers web Summarizer API only) **→ FI-31** | — | map: Excluded; VCE: Misread; RBP: Misread → Missing |
| BF-5.11 | Easy file picker [Diff] | **Missing** | — **→ FI-5** | — | map: Missing; VMB: Confirmed missing |

### BF 6. Passwords, passkeys, autofill and payments

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-6.1 | Password manager [TS] | **Designed** | Shell 6.12, 6.3 `shell.password`; S1 §6 (A-S1-51) | Stage 1 | map: Covered; RBP: Designed |
| BF-6.2 | Password generator [TS] | **Missing** | — **→ FI-13** | — | map: Missing; VMB: Confirmed missing |
| BF-6.3 | Breached and weak password alerts [TS] | **Mentioned** | Shell 6.12 (no owner) **→ FI-13** | — | map: Excluded; VCE: Misread; RBP: Misread → Mentioned. *VCE said Missing; RBP Mentioned because shell 6.12 names the breach check* |
| BF-6.4 | Passkey manager [TS] | **Owned** | S1 §6 "Passkeys and credentials" (A-S1-51); shell 6.12 | Stage 1 | map: Covered; RBP: Owned |
| BF-6.5 | Password and passkey import/export | **Designed** | Shell 6.12, SH-D19, SH-R2 | Stage 1, SH-M3 (M7) | map: Covered; RBP: Designed |
| BF-6.6 | Password manager PIN / device unlock | **Designed** | Shell 6.12; 6.4 Credential class | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-6.7 | Third-party password manager integration | **Missing** | — **→ FI-13** | — | map: Missing; VMB: Confirmed missing |
| BF-6.8 | Address autofill [TS] | **Deferred** | Shell 9.5 (confirms D-S3/S1 13.2); ED 6.6 preview | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred |
| BF-6.9 | Payment card autofill [TS] | **Deferred** | Shell 6.9, 9.5; D-S3 | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred |
| BF-6.10 | Virtual cards, BNPL and card benefits | **Missing** | — **→ FI-13** | — | map: Missing; VMB: Confirmed missing |
| BF-6.11 | Wallet payment sheet | **Mentioned** | — **→ FI-13** | — | map: Missing; VMB: Actually partial. *named only (VMB)* |
| BF-6.12 | Shopping: price tracking and insights | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-6.13 | Store reviews | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-6.14 | Crypto wallet [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-6.15 | Creator rewards [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |

### BF 7. Privacy controls

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-7.1 | Private browsing window [TS] | **Designed** | NW 6.1; SG 6.9; shell 6.7 | Stage 1 | map: Covered; RBP: Designed |
| BF-7.2 | Lock private tabs with biometrics | **Missing** | — **→ FI-10** | — | map: Missing; VMB: Confirmed missing |
| BF-7.3 | Private window over Tor [Diff] | **Mentioned** | NW 6.13, NW-D6 **→ FI-19** | Stage 2 (container routes) | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| BF-7.4 | Cross-site tracker blocking [TS] | **Designed** | CB (whole spec), CB-D19 | Stage 1 | map: Covered; RBP: Designed |
| BF-7.5 | Cookie partitioning / Total Cookie Protection | **Designed** | NW 6.9, NW-D7, NW-D16 | Stage 1 | map: Covered; RBP: Designed |
| BF-7.6 | Third-party cookie blocking setting [TS] | **Partial** | NW 6.9 **→ FI-12** | Stage 1 | map: Partial |
| BF-7.7 | Fingerprinting protection | **Designed** | D-S1, OD-6; S1 §6 "Anti-fingerprinting" | Stage 1 baseline, Stage 2 full | map: Covered; RBP: Designed |
| BF-7.8 | Tracking-parameter removal from URLs | **Designed** | CB 6.5, CB-D8 | Stage 1 | map: Covered; RBP: Designed |
| BF-7.9 | Privacy report / protections dashboard | **Partial** | CB 6.12–6.13, CB-D20; shell 6.6 Blocking row; NW 6.18 "who received what" **→ FI-19** | Stage 1 | map: Partial |
| BF-7.10 | Per-site shields panel [Diff] | **Designed** | CB 6.12, CB-D19; Shell 6.6 | Stage 1, SH-M2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-7.11 | Built-in ad blocking [Diff] | **Designed** | CB 6.x, CB-D5 (Ads list), CB-D1 | Stage 1, CB-M3 | map: Covered; RBP: Designed |
| BF-7.12 | Cookie banner handling [Diff] | **Designed** | CB 6.15, CB-D22 | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-7.13 | Clear browsing data [TS] | **Partial** | Shell 6.4 (ledger retention), 6.7 (history clearing), 6.9 ("clear-on-close choices") **→ FI-19** | Stage 1 | map: Partial |
| BF-7.14 | Site data manager [TS] | **Designed** | Shell 6.6 Storage row, SG-D5/SG-D6; Shell 6.9 | Stage 1, SH-M2 | map: Covered; RBP: Designed |
| BF-7.15 | Global Privacy Control / Do Not Track setting | **Designed** | S1 §6 GPC row; Shell 6.9 Privacy group | Stage 1 | map: Covered; RBP: Designed |
| BF-7.16 | Zero telemetry / telemetry controls | **Designed** | D-S4, SH-D29; Shell 6.3, 6.14 | Stage 1 | map: Covered; RBP: Designed |
| BF-7.17 | IP masking in private browsing | **Refused** | S1 §6 out-of-scope ("browser-operated VPN service") | — | map: Excluded; VCE: Confirmed; RBP: Refused |
| BF-7.18 | Distraction control / element hiding | **Partial** | CB 6.10, CB-D14 ("write their own rules"); CB 6.7 **→ FI-19** | Stage 1 | map: Partial |

### BF 8. Security and safety UI

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-8.1 | Safe browsing interstitials [TS] | **Designed** | PH-D6, PH-12; Shell 6.4; UI 4.5 | Stage 1 | map: Covered; RBP: Designed |
| BF-8.2 | Enhanced protection modes | **Partial** | HGBA-2 S-4a "Hardened profile"; JS-D13 (Jitless for the banking container and per site); SEC-D5 **→ FI-19** | Stage 1 (engine) | map: Partial |
| BF-8.3 | Scareware / tech-support-scam blocker | **Partial** | CB 6.8, CB-D23 Security list ("scam redirectors") **→ FI-19** | Stage 1 | map: Partial |
| BF-8.4 | HTTPS-only / always use secure connections [TS] | **Designed** | NW 6.7, NW 6.5.3; Shell 6.9 | Stage 1 | map: Covered; RBP: Designed |
| BF-8.5 | Secure DNS / DNS-over-HTTPS settings | **Designed** | NW 6.6, NW-D8; NW 6.14 | Stage 1 | map: Covered; RBP: Designed |
| BF-8.6 | Certificate viewer [TS] | **Partial** | Shell 6.6 Identity row ("certificate details expander"); NW 6.18 **→ FI-19** | Stage 1 | map: Partial |
| BF-8.7 | Certificate error interstitials [TS] | **Designed** | Shell 6.4; NW 6.5.3, 6.18; UI 4.5 | Stage 1 | map: Covered; RBP: Designed |
| BF-8.8 | Safety check | **Missing** | — **→ FI-19** | — | map: Missing; VMB: Confirmed missing |
| BF-8.9 | Unwanted software cleanup | **Missing** | — **→ FI-19** | — | map: Missing; VMB: Confirmed missing |
| BF-8.10 | "Managed by your organisation" indicator [TS] | **Designed** | Shell 6.9, 6.14; NW 6.14 | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-8.11 | Lookalike-domain warning | **Partial** | Shell 6.4 (interstitial class includes "lookalike domain"), SH-11; UI Q-UI-5 / Q-PH-16 (open) **→ FI-19** | Stage 1 (class) | map: Partial |
| BF-8.12 | Auto-update [TS] | **Designed** | Shell 6.11, SH-24; S1 13.2 Updates (TUF), 7.8 | Stage 1 | map: Covered; RBP: Designed |

### BF 9. Site permissions and page info

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-9.1 | Page info panel [TS] | **Designed** | Shell 6.6, SH-D14 | Stage 1, SH-M2 | map: Covered; RBP: Designed |
| BF-9.2 | Permission prompts [TS] | **Designed** | Shell 6.4, 6.5, SH-D2, SH-D12 | Stage 1, SH-M2 | map: Covered; RBP: Designed |
| BF-9.3 | Quieter notification prompts | **Designed** | Shell 6.4, SH-D17 | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-9.4 | Site settings manager [TS] | **Designed** | Shell 6.9, SH-D15 | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-9.5 | Auto-revoke unused permissions | **Designed** | Shell 6.4 | Stage 1 | map: Covered; RBP: Designed |
| BF-9.6 | Device access choosers | **Designed** | Shell 6.4 device-pairing class; DV 6.18 | Stage 1 (experimental setting) | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-9.7 | Popup blocker [TS] | **Partial** | DOM 6.5 (activation record gates popups); CB 6.2 (`$popup`) **→ FI-19** | Stage 1 (engine) | map: Partial |
| BF-9.8 | Autoplay controls | **Designed** | MD 6.12, MD-D13; S1 §6 | Stage 1 | map: Covered; RBP: Designed |
| BF-9.9 | In-use indicators | **Partial** | Shell 6.6 device indicator (SH-35); SEC 6.5 capture threat model (screen capture "shows an OS-level capture indicator") **→ FI-19** | Stage 1 (devices) | map: Partial |
| BF-9.10 | Screen-share picker | **Partial** | MD 6.10 Capture ("the OS pickers"); GR 6.27 (tab capture in the compositor) **→ FI-19** | Stage 1 | map: Partial |
| BF-9.11 | Per-site JavaScript, images and content toggles | **Partial** | S1 §6 "JavaScript modes" (Stage 2), UI R9; shell 6.6 JavaScript row **→ FI-19** | Stage 2 (JavaScript) | map: Partial |
| BF-9.12 | Protocol handler and external app prompts | **Designed** | Shell 6.4; SEC 6.14a; PH 6.4.6 | Stage 1 | map: Covered; RBP: Designed |

### BF 10. Content, reading and page tools

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-10.1 | Find in page [TS] | **Designed** | Shell 6.8 find bar, ED-D17, SH-32 | Stage 1, SH-M5 | map: Covered; RBP: Designed |
| BF-10.2 | Page zoom [TS] | **Designed** | Shell 6.8; AX-D19; GR 6.28 | Stage 1, SH-M1 | map: Covered; RBP: Designed |
| BF-10.3 | Text-only zoom and minimum font size | **Partial** | AX-D19 (text scale), ST-D17 (minimum font size); AX 6.11 **→ FI-20** | Stage 1 (mechanism) | map: Partial |
| BF-10.4 | Reader mode [TS] | **Deferred, no owner** | UI R18/§6; S1 §6; PD 9.1 (reflow with reader view) **→ FI-30** | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred, no owner |
| BF-10.5 | Read aloud / listen to page | **Partial** | — **→ FI-30** | — | map: Missing; VMB: Actually partial |
| BF-10.6 | Page translation [TS] | **Missing** | — (ML-D2 defers web Translator API only) **→ FI-30** | — | map: Excluded; VCE: Misread; RBP: Misread → Missing |
| BF-10.7 | Spellcheck [TS] | **Designed** | ED 6.8, ED-D7, ED-D8, ED-D22 | Stage 2 (ED-D8 restaged 21 Oct) | map: Covered; RBP: Designed. *stage corrected: ED-D8 restaged to Stage 2 on 21 October 2026* |
| BF-10.8 | Cloud / enhanced spellcheck and grammar | **Refused** | ED §4.4, ED-D7 | — | map: Excluded; VCE: Confirmed; RBP: Refused |
| BF-10.9 | Share page [TS] | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| BF-10.10 | Link to highlighted text | **Partial** | ED 6.12 "Text directives" **→ FI-20** | Stage 1 (navigation) | map: Partial |
| BF-10.11 | QR code for page | **Missing** | — **→ FI-20** | — | map: Missing; VMB: Confirmed missing |
| BF-10.12 | Screenshot tool | **Partial** | — **→ FI-20** | — | map: Missing; VMB: Actually partial |
| BF-10.13 | Eyedropper | **Missing** | — **→ FI-20** | — | map: Missing; VMB: Confirmed missing |
| BF-10.14 | Notes in sidebar [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-10.15 | Highlights / page key facts | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-10.16 | Request desktop site (mobile) [TS on mobile] | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-10.17 | Dark mode for web content | **Missing** | — **→ FI-30** | — | map: Missing; VMB: Confirmed missing |
| BF-10.18 | View page source [TS] | **Partial** | PH 6.2 and HGBA-3 B9 (AGT-R3) (refer to `view-source:` URLs) **→ FI-20** | — | map: Partial. *PH 6.2 and AGT-R3 assume `view-source:` exists; no owner* |

### BF 11. Media

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-11.1 | Picture-in-picture [TS] | **Designed** | MD 6.12 | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-11.2 | Automatic PiP on tab switch | **Missing** | — **→ FI-20** | — | map: Missing; VMB: Confirmed missing |
| BF-11.3 | Global media controls | **Partial** | MD 6.12 Media Session **→ FI-20** | Stage 1 | map: Partial |
| BF-11.4 | Casting [TS on Chrome] | **Missing** | — **→ FI-32** | — | map: Missing; VMB: Confirmed missing |
| BF-11.5 | Live captions for media | **Designed** | ML-D2 (ML-ASR), MD-D25 | Stage 1 (ML-M3) or Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-11.6 | Media playlist / offline play [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-11.7 | Built-in music player [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-11.8 | Volume booster and video enhancement [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-11.9 | DRM content playback [TS] | **Designed** | D-S7, S1 13.1a, MD-D32 | Stage 1 (M7) | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-11.10 | Microphone volume adjustment | **Partial** | — **→ FI-20** | — | map: Missing; VMB: Actually partial |

### BF 12. Performance and resource controls

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-12.1 | Tab sleeping / memory saver [TS] | **Designed** | Shell 6.8 cold-tab discard; FP-D15 | Stage 1 | map: Covered; RBP: Designed |
| BF-12.2 | Always-keep-active site list | **Designed** | FP-D15; UI 10 (A-UI-1) | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-12.3 | Energy / battery saver | **Partial** | AN 6.11, AN-D11 (30 Hz under the OS low-power signal); FP-D15 "limit this site on battery" (Stage 2, gated by FP-30) **→ FI-21** | Stage 1 (follows the OS), Stage 2 (per site) | map: Partial |
| BF-12.4 | Performance issue alerts | **Refused** | (implicit) Shell 6.8 cold-tab discard; UP2 **→ FI-38** | — | map: Excluded; VCE: Confirmed-implicit; RBP: Refused (implicit) |
| BF-12.5 | Browser task manager | **Partial** | Shell 6.14 `about:diagnostics` ("process list with PIDs and budgets"); shell 6.6 Resources row (Stage 2); FP 6.14 **→ FI-21** | Stage 1 (diagnostics), Stage 2 (meter) | map: Partial |
| BF-12.6 | Preloading settings | **Partial** | S1 §9 ("Speculation Rules and prerender", Stage 2) **→ FI-21** | Stage 2 (engine) | map: Partial |
| BF-12.7 | Startup boost / background running | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| BF-12.8 | Hardware acceleration toggle | **Mentioned** | Shell 6.14 recovery window, SH-D22 **→ FI-21** | Stage 1, SH-M4 | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |

### BF 13. Extensions, themes and customisation

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-13.1 | Extension support [TS] | **Designed** | EX spec (EX-D1–EX-D16), D-S3 | Stage 2 | map: Covered; RBP: Designed |
| BF-13.2 | Extension store [TS] | **Designed** | EX 6.12, EX-D10 | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-13.3 | Extension management UI [TS] | **Designed** | Shell 6.3 `shell.extensions`, 6.15; EX 6.12 Extensions page | Stage 2 | map: Covered; RBP: Designed |
| BF-13.4 | Cross-ecosystem extension support [Diff] | **Designed** | EX 6.1 Ring A, EX-D1, EX-D4 | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-13.5 | Legacy (MV2) extension support [Diff] | **Refused** | EX-D2 | — | map: Excluded; VCE: Confirmed; RBP: Refused |
| BF-13.6 | Extensions on mobile | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-13.7 | AI-generated extensions [Diff] | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-13.8 | Themes [TS] | **Designed** | Shell 6.2, SH-D6 | Stage 1 | map: Covered; RBP: Designed |
| BF-13.9 | AI theme generator | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-13.10 | Light/dark mode for the browser UI [TS] | **Designed** | Shell 6.2; UI 4.10 | Stage 1 | map: Covered; RBP: Designed |
| BF-13.11 | Toolbar customisation | **Missing** | — **→ FI-21** | — | map: Missing; VMB: Confirmed missing |
| BF-13.12 | Deep UI customisation / mods [Diff] | **Partial** | Shell 6.2 (user CSS limited to custom properties and ordinary properties), D-S2/UI R2 (W-1 components), SH-D6 **→ FI-21** | Stage 1 (CSS) | map: Partial |
| BF-13.13 | Keyboard shortcuts [TS] | **Designed** | Shell 6.16, SH-4 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-13.14 | Remappable keyboard shortcuts [Diff] | **Partial** | EX-D15 (extension "command editor", Stage 2); shell 6.9 ("keyboard options") **→ FI-21** | Stage 2 (extensions only) | map: Partial |
| BF-13.15 | Command chains / macros [Diff] | **Missing** | — **→ FI-21** | — | map: Missing; VMB: Confirmed missing |
| BF-13.16 | Peripheral lighting integration [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-13.17 | Experimental feature flags / labs | **Designed** | Shell 6.9 "No hidden settings", SH-D15 | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-13.18 | Language settings [TS] | **Designed** | Shell 6.9 "Language and writing"; ED-D8, ED 6.8 | Stage 2 (hidden until ED-D8 ships) | map: Excluded; VCE: Misread; RBP: Misread → Designed |
| BF-13.19 | Built-in game [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-13.20 | Touch Bar / hardware control customisation | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |

### BF 14. Accessibility

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-14.1 | Screen reader support [TS] | **Designed** | AX spec (AX-D3, D4, D9, D12); shell 6.16; UI-1 | Stage 1 | map: Covered; RBP: Designed |
| BF-14.2 | High contrast / forced colours [TS] | **Designed** | AX 6.11; shell 6.16 "Colour and contrast" | Stage 1 | map: Covered; RBP: Designed |
| BF-14.3 | Focus highlight and caret browsing | **Designed** | AX 6.12, AX-D20; shell 6.16 "Focus" | Stage 1 | map: Covered; RBP: Designed |
| BF-14.4 | Keyboard-only browsing [TS] | **Designed** | Shell 6.16, SH-4; UI-2 | Stage 1 | map: Covered; RBP: Designed |
| BF-14.5 | AI image descriptions | **Designed** | ML 6.x ML-IMG, ML-D2, ML-D4; MD-D25; HGBA-3 B10 | Stage 1, ML-M1 | map: Covered; RBP: Designed |
| BF-14.6 | Live caption (see BF-11.5) | **Designed** | ML-ASR (ML model table), MD-D25 | Stage 1 ML-M3, or Stage 2 | map: Covered; RBP: Designed |
| BF-14.7 | Magnifier support / full-page zoom | **Designed** | AX 6.8; shell 6.16 "Zoom, reflow and text", 6.8 | Stage 1 | map: Covered; RBP: Designed |
| BF-14.8 | Reduced motion and default fonts | **Partial** | AN-D10, shell 6.16 "Motion", SH-D25 **→ FI-21** | Stage 1 | map: Partial |
| BF-14.9 | Talk to type / voice dictation | **Owned** | AX 6.9; ED-D5; ED-4 | Stage 1 (OS hand-off) | map: Covered; VCE: Confirmed covered; RBP: Owned |

### BF 15. AI features and their off switches

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-15.1 | AI assistant panel | **Deferred, no owner** | UI §6 Stage 3 "Agent surfaces"; HGBA-3 §7, B10 **→ FI-31** | Stage 3 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred, no owner |
| BF-15.2 | Choice of AI provider [Diff] | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.3 | Page summarisation | **Mentioned** | ML-D2 (ML 6.x "Stage 2 tasks") **→ FI-31** | Stage 2 (task API only) | map: Excluded; VCE: Misread; RBP: Misread → Mentioned. *VCE said Missing or Partial; RBP Mentioned* |
| BF-15.4 | Ask about multiple tabs / compare tabs | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.5 | AI tab organiser | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.6 | Agentic browsing [Diff] | **Partial** | HGBA-3 B9, B10, §7; SEC AGT-R1–AGT-R8; shell 6.4 "Agent and automation" class; S1 §9 Stage 2 "agent principal" **→ FI-31** | Stage 2 (principal), Stage 3 (surfaces) | map: Partial |
| BF-15.7 | AI history search | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.8 | Writing help | **Deferred** | ED 6.8 (ED-D9); ML-D2; Shell 6.9 Language and writing | Stage 2, under AI grant | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred |
| BF-15.9 | AI memory / personalisation | **Partial** | — **→ FI-31** | — | map: Missing; VMB: Actually partial |
| BF-15.10 | AI access to personal data sources | **Partial** | — **→ FI-31** | — | map: Missing; VMB: Actually partial |
| BF-15.11 | Image generation and editing | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.12 | Voice / live AI conversation and screen sharing | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.13 | Virtual try-on | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.14 | On-device models and their management | **Designed** | ML 6.3 model manager, ML-D4, ML-D10, ML-D11 | Stage 1, ML-M3 | map: Covered; RBP: Designed |
| BF-15.15 | AI master off switch [Diff→TS] | **Designed** | HGBA-3 B10, D13; SH-D16, SH-8; S1 §6 "AI switch" | Stage 1 | map: Covered; RBP: Designed |
| BF-15.16 | AI-generated content creation from browsing [Diff] | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |
| BF-15.17 | Agent / automation server for external AI tools | **Designed** | DT spec IIP and adapters, D-S11, DT-D10 | Stage 1 (BiDi/CDP); MCP when AI on | map: Covered; RBP: Designed |
| BF-15.18 | Link previews with AI summary | **Missing** | — **→ FI-31** | — | map: Missing; VMB: Confirmed missing |

### BF 16. Built-in apps and services

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-16.1 | Sidebar / side panel [TS] | **Partial** | EX-D15 ("sidebar Stage 3"), EX 6.9 ("pending the shell's panel design") **→ FI-17** | Stage 3 (extension slot) | map: Partial. *EX-D15 waits on a shell panel design that is not scheduled* |
| BF-16.2 | Web panels / sidebar apps [Diff] | **Missing** | — **→ FI-17** | — | map: Missing; VMB: Confirmed missing |
| BF-16.3 | Integrated messengers [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.4 | Mail client [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.5 | Calendar [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.6 | Feed reader (RSS/Atom) [Diff] | **Deferred, no owner** | S1 §9 Stage 2 "optional W-1 components (feeds, …)" **→ FI-17** | Stage 2 | map: Excluded; VCE: Confirmed (deferral); RBP: Deferred, no owner |
| BF-16.7 | Built-in VPN / secure proxy | **Refused** | S1 §6 out-of-scope list | — | map: Excluded; VCE: Confirmed; RBP: Refused |
| BF-16.8 | Proxy settings | **Designed** | NW 6.13 "Proxies", NW 6.14 | Stage 1 (OS), Stage 2 (per-container editing) | map: Covered; RBP: Designed |
| BF-16.9 | Video calling [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.10 | Google / productivity service shortcuts | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.11 | Live work destinations and meeting prep [Diff] | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.12 | Remote desktop access | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.13 | Location and directions cards | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-16.14 | Phone-number click-to-call | **Partial** | Shell 6.4 External protocol class ("`mailto:`, `tel:`") **→ FI-4** | Stage 1 | map: Partial |

### BF 17. Web apps and operating-system integration

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-17.1 | Install site as app | **Partial** | PH 6.4.9 (assumes "An installed web app launches in a window with no address bar"); AH 6.8 and 6.12 (app windows for `app://` packages, Stage 3) **→ FI-9** | — | map: Partial |
| BF-17.2 | Web app management | **Partial** | Shell 6.3 `shell.apps` (for `app://` packages); AH 6.12 **→ FI-9** | Stage 3 (packages) | map: Partial |
| BF-17.3 | Site shortcuts on desktop | **Missing** | — **→ FI-9** | — | map: Missing; VMB: Confirmed missing |
| BF-17.4 | Default browser setting and prompt [TS] | **Designed** | Shell 6.11, SH-D21; S1 §6 | Stage 1 | map: Covered; RBP: Designed |
| BF-17.5 | OS notifications for web notifications [TS] | **Designed** | Shell 6.13, SH-D17; Q-SH-10 open | Stage 2 | map: Excluded; VCE: Misread; RBP: Misread → Designed. *same feature as PO-9.7 (shell 6.13, Stage 2); both have an owner and a stage* |
| BF-17.6 | OS share sheet integration [TS] | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| BF-17.7 | Handoff / continuity | **Deferred, no owner** | Shell 9.1 "Stage 3 or never" (sync and multi-device); Q-SH-5 **→ FI-35** | Stage 3 or never | map: Missing; VMB: Actually excluded. *VMB said Excluded (shell 9.1 "Stage 3 or never"); filed Deferred, no owner to match RBP's reading of the same row for sync* |
| BF-17.8 | Mobile widgets and home-screen shortcuts | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-17.9 | In-car browser | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-17.10 | Password autofill in other apps (mobile) | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-17.11 | Default app installs from the browser (iOS EU) | **Refused** | (implicit) S1 13.2 "iOS" **→ FI-38** | — (Stages 1–2 out of scope) | map: Excluded; VCE: Confirmed-implicit; RBP: Refused (implicit) |
| BF-17.12 | Calendar event add (mobile) | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |

### BF 18. Developer and power-user entry points

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-18.1 | Developer tools [TS] | **Designed** | DT spec, D-S11, DT-D10 | Stage 1 | map: Covered; RBP: Designed |
| BF-18.2 | DevTools entry points [TS] | **Partial** | Shell 6.2 ("open the shell's inspector from the about:diagnostics page"), 6.8 (Why? "open DevTools with the same query") **→ FI-20** | Stage 1 | map: Partial |
| BF-18.3 | Responsive design / device mode | **Missing** | — **→ FI-7** | — | map: Missing; VMB: Confirmed missing |
| BF-18.4 | Report a broken site / send feedback | **Designed** | CB 6.11, CB-D18; shell 6.6 | Stage 1 | map: Covered; RBP: Designed |
| BF-18.5 | Internal status pages | **Designed** | Shell 6.14 "About pages", 6.9 | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-18.6 | Remote debugging and automation | **Designed** | DT spec (IIP, BiDi, CDP, DAP); S1 M2 | Stage 1 | map: Covered; RBP: Designed |
| BF-18.7 | Beta / preview channels | **Mentioned** | S1 7.8; shell 6.9 "Update channel" row **→ FI-2** | Stage 1 (named) | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |

### BF 19. Setup, onboarding and maintenance

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-19.1 | Import from other browsers [TS] | **Designed** | Shell 6.12, SH-D19, SH-R2; S1 13.2 | Stage 1, SH-M3 (M7) | map: Covered; RBP: Designed |
| BF-19.2 | First-run onboarding [TS] | **Designed** | Shell 6.14, SH-D23, SH-6; UI 4.6 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-19.3 | What's new page | **Partial** | Shell 6.11 ("no ... new-version page re-asks") **→ FI-22** | — | map: Partial |
| BF-19.4 | Reset / refresh settings [TS] | **Designed** | Shell 6.14 recovery table, 6.9 | Stage 1 | map: Covered; RBP: Designed |
| BF-19.5 | Crash reporting with consent [TS] | **Designed** | D-S4; S1 §6 "Crash reports"; shell 6.14 | Stage 1 | map: Covered; RBP: Designed |
| BF-19.6 | Troubleshooting help | **Designed** | Shell 6.14 recovery window, SH-D22; D-S2 | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| BF-19.7 | In-browser help / AI help | **Partial** | — **→ FI-22** | — | map: Missing; VMB: Actually partial |
| BF-19.8 | Lost-device lock and erase | **Refused** | (implicit) SH-D20; Shell 9.1 "Stage 3 or never"; Q-SH-5 **→ FI-35** | Stage 3 or never | map: Excluded; VCE: Confirmed-implicit; RBP: Refused (implicit) |

### BF 20. Enterprise, family and special modes

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-20.1 | Enterprise policies | **Partial** | Shell 6.9 (policy rows, "A policy can set only registry settings"), 6.14 `about:policies`; NW 6.14 "Enterprise policy surface"; HGBA-3 B10 (AI lock) **→ FI-3** | Stage 1 | map: Partial |
| BF-20.2 | Kiosk mode | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| BF-20.3 | Parental controls / family safety | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-20.4 | Kids mode | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |
| BF-20.5 | Data-saver / lite mode (mobile) | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-20.6 | Shared device / multi-user | **Partial** | SG §1 (shared-device risk), SG 6.9 and SG-D7 (per-container keys in the OS key store); shell 4.1 ("profiles later") **→ FI-10** | — | map: Partial |

### BF 21. Mobile-specific product features

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| BF-21.1 | Tab grid / switcher [TS] | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-21.2 | Swipe gestures between tabs and back [TS] | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-21.3 | Pull to refresh [TS] | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-21.4 | Offline page saving | **Designed** | S1 §6 "Save for offline" | Stage 2 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| BF-21.5 | Mobile privacy hub | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |
| BF-21.6 | Camera search / QR scan | **Missing** | — **→ FI-33** | — | map: Missing; VMB: Confirmed missing |

## PO: OS integration and operations


### PO 1. Packaging and installation

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-1.1 | Windows consumer installer [SB] | **Partial** | S1 13.2 "Packaging"; SEC-D14; FP-25 **→ FI-2** | Stage 1 | map: Partial |
| PO-1.2 | Windows enterprise MSI [SB] | **Deferred, no owner** | SEC-D14 (per-user installs only in Stage 1); no Stage 2 owner **→ FI-3** | Stage 2 (SEC-D14's reversal trigger) | map: Missing; VMB: Actually excluded. *VMB said Excluded: SEC-D14 refuses per-machine installs for Stage 1 only; its reversal trigger names Stage 2 enterprise, which nobody owns* |
| PO-1.3 | MSIX / Microsoft Store package | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-1.4 | macOS disk image and pkg [SB] | **Partial** | S1 13.2 "Packaging"; SEC-16 **→ FI-2** | Stage 1 | map: Partial |
| PO-1.5 | Mac App Store distribution | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-1.6 | Linux deb and rpm packages [SB] | **Partial** | S1 13.2 "Packaging"; SEC 6.5 updater threat model **→ FI-2** | Stage 1 | map: Partial. *S1 13.2 ships deb/rpm, while SEC-D14 and UPD-R2 assume the per-user TUF updater; SEC 6.5 leaves it Open* |
| PO-1.7 | Flatpak (Flathub) | **Deferred** | S1 13.2 "Packaging"; SEC-8 (reduced tier on `izcal://sandbox`) | Stage 2 | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred |
| PO-1.8 | Snap | **Partial** | GR G67 **→ FI-2** | — | map: Partial |
| PO-1.9 | Tarball / portable build | **Mentioned** | S1 13.2 "Packaging" **→ FI-2** | Stage 1 | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| PO-1.10 | Side-by-side channels | **Partial** | S1 7.8 **→ FI-2** | Stage 1 | map: Partial |
| PO-1.11 | Initial preferences / partner distributions | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-1.12 | Install-time options | **Missing** | — (AH 6.12 covers apps only) **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-1.13 | Uninstall, repair and leftover cleanup [SB] | **Missing** | — (AH 6.12, AH-13 cover installed apps only) **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-1.14 | First-run experience and import | **Designed** | SH 6.12, 6.14, SH-D23; SEC SH-R2 | Stage 1, SH-M5 (M6) | map: Covered; RBP: Designed |
| PO-1.15 | Profile management | **Partial** | S1 13.2 "Profiles"; SH 4.1, SH-D26; SH 6.8 **→ FI-10** | Stage 1 (format) | map: Partial |
| PO-1.16 | VDI, roaming profiles and multi-session | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-1.17 | Kiosk mode | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |

### PO 2. Code signing and platform trust

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-2.1 | Windows Authenticode signing [SB] | **Owned** | SEC 6.16 controls, SEC-16 (a) | Stage 1, P1 (before M7) | map: Covered; RBP: Owned |
| PO-2.2 | SmartScreen app reputation [SB] | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-2.3 | macOS Developer ID signing and hardened runtime [SB] | **Owned** | SEC 6.16, 6.7 table; SEC-16 | Stage 1, P1 | map: Covered; RBP: Owned |
| PO-2.4 | macOS notarisation and stapling [SB] | **Owned** | S1 13.2; SEC-16 (a) | Stage 1, P1 | map: Covered; VCE: Confirmed covered; RBP: Owned |
| PO-2.5 | macOS JIT entitlement | **Designed** | SEC 6.7; JS 6.8 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-2.6 | Linux repository and package signing [SB] | **Partial** | SEC-16 (a); SEC 6.5 updater "Open" **→ FI-2** | Stage 1 | map: Partial |
| PO-2.7 | Signed update payloads | **Designed** | SEC 6.16 UPD-R1–R3, SEC-17; S1 7.8 | Stage 1 | map: Covered; RBP: Designed |
| PO-2.8 | Third-party DLL injection blocking | **Partial** | SEC 6.7 table **→ FI-11** | Stage 1 (M5 measurement) | map: Partial |
| PO-2.9 | Reproducible builds and build provenance (SBOM, SLSA) | **Designed** | S1 7.8; SEC-13, SEC-15 | Stage 1 P1 (engine images M7; browser binary Stage 2) | map: Covered; RBP: Designed |
| PO-2.10 | iOS/iPadOS alternative engine entitlement (later, EU only) | **Refused** | S1 13.2 "iOS" | — (revisit if DMA enforcement changes) | map: Excluded; VCE: Confirmed; RBP: Refused; FC. *FC 3a–3h: Apple's EU engine rules confirmed* |

### PO 3. Updates and release channels

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-3.1 | Background auto-updater [SB] | **Partial** | S1 1 ("Updater … On update check"); SEC UPD-R2; SH 6.11 **→ FI-2** | Stage 1 | map: Partial |
| PO-3.2 | Elevation-free updates on Windows | **Designed** | SEC-D14 (SEC 6.7) | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-3.3 | Delta (differential) updates | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-3.4 | Release channels [SB] | **Mentioned** | S1 7.8; SH 6.9 **→ FI-2** | Stage 1 | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| PO-3.5 | Release cadence | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-3.6 | Extended support channel | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-3.7 | Staged (percentage) rollout | **Mentioned** | — (AH 6.7 mentions staged rollout for apps only) **→ FI-2** | — | map: Missing; VMB: Actually partial. *named only (VMB)* |
| PO-3.8 | Version pinning and rollback | **Missing** | — **→ FI-2** | — | map: Missing; VMB: Confirmed missing |
| PO-3.9 | Update policy controls | **Partial** | SH 6.9 "Updates and recovery"; SH 6.11 **→ FI-3** | Stage 1 | map: Partial |
| PO-3.10 | Relaunch notification | **Designed** | SH 6.11 "Updates"; SH-24 | Stage 1 | map: Covered; RBP: Designed |
| PO-3.11 | Update while running | **Designed** | S1 1 (staged updates); SH 6.11; SH-24 | Stage 1 | map: Covered; RBP: Designed |
| PO-3.12 | Emergency (out-of-band) security release | **Designed** | SEC UPD-R4, UPD-R5, SEC-D11 | Stage 1 | map: Covered; RBP: Designed |
| PO-3.13 | Weekly security updates | **Partial** | SEC-D1 (JS-16), SEC-25 **→ FI-2** | Stage 1 | map: Partial |
| PO-3.14 | Release notes publishing | **Partial** | H3 B10 (ML inventory); S1 13.4 UA-1; SEC 6.17 **→ FI-2** | Stage 1 | map: Partial |
| PO-3.15 | Supported-OS policy and end-of-support handling | **Mentioned** | GR-D21 **→ FI-1** | Stage 1 | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |

### PO 4. Component and data updates (pushed outside the binary)

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-4.1 | Component update channel [SB] | **Designed** | SEC UPD-R1; NW-D17 (6.13); S1 13.2 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-4.2 | Root certificate store [SB] | **Designed** | NW-D5, NW-D24 (NW 6.5.2) | Stage 1, NW-M2 | map: Covered; RBP: Designed |
| PO-4.3 | Root program governance | **Designed** | NW 6.5.2 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-4.4 | Revocation data | **Designed** | NW 6.5.1, NW-D21; S1 13.2 | Stage 1, NW-M2 (both CRLs and filters) | map: Covered; RBP: Designed |
| PO-4.5 | Certificate Transparency policy and log list | **Designed** | S1 13.2; NW 6.5.1 | Stage 1 | map: Covered; RBP: Designed |
| PO-4.6 | Safe-browsing lists [SB] | **Designed** | PH (PH-D1–D15); SEC-D7 | Stage 1 | map: Covered; RBP: Designed |
| PO-4.7 | Public Suffix List | **Owned** | H2 2.3.2; S1 13.2; NW-D7 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Owned; FC. *FC 7a: the MPL-2.0 attribution is from secondary sources only; confirm before release* |
| PO-4.8 | HSTS preload list | **Designed** | NW-D25 (NW 6.5.3) | Stage 1 | map: Covered; RBP: Designed |
| PO-4.9 | Extension and add-on blocklist | **Partial** | EX 6.10 (distribution, review); SEC UPD-R1 **→ FI-2** | Stage 2 | map: Partial |
| PO-4.10 | DRM content decryption module | **Designed** | D-S7 (S1 13.1a); MD-D32 | Stage 1 (M7) | map: Covered; RBP: Designed |
| PO-4.11 | Codec plug-ins | **Refused** | S1 13.2 "Codecs" (A-S1-85); MD-D3 | — | map: Excluded; VCE: Confirmed; RBP: Refused |
| PO-4.12 | Spellcheck dictionaries | **Designed** | ED 6.8, ED-D7, ED-D8, ED-D22; TX-D14 | Stage 2 for interim OS tier (ED-D8 restaged 21 Oct) | map: Covered; RBP: Designed. *stage corrected: ED-D8 restaged to Stage 2 on 21 October 2026* |
| PO-4.13 | Fonts and font fallback data | **Designed** | TX-D1, TX-D14, TX-D24; FP-D13 | Stage 1 (TX-W1 onward) | map: Covered; RBP: Designed |
| PO-4.14 | On-device ML models | **Designed** | ML-D7, ML-D10, ML-11 | Stage 1 (AX grant); Stage 2 (AI switch) | map: Covered; RBP: Designed |
| PO-4.15 | Locale and Unicode data | **Designed** | H2 2.3.2; TX (`hg-ucd`, `hg-unicode`) | Stage 1 | map: Covered; RBP: Designed; FC. *FC 7b: Unicode License v3 (permissive, notice required)* |
| PO-4.16 | Tracking-protection and filter lists | **Designed** | CB-D13, CB-D17; NW-D26 | Stage 1 | map: Covered; RBP: Designed |
| PO-4.17 | Remote kill switches and config | **Designed** | SEC UPD-R5, SEC-D11 | Stage 1 | map: Covered; RBP: Designed |

### PO 5. Crash reporting, telemetry and experimentation

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-5.1 | Crash capture and minidumps [SB] | **Designed** | S1 1 (crash handler), D-S4; SEC SH-R5 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-5.2 | Crash report submission and consent | **Designed** | D-S4 (S1 6 "Crash reports"); SH 6.14 | Stage 1 | map: Covered; RBP: Designed |
| PO-5.3 | Crash server and symbolication | **Partial** | S1 7.8 **→ FI-11** | Stage 1 | map: Partial |
| PO-5.4 | OS crash and hang reporting integration | **Missing** | — **→ FI-11** | — | map: Missing; VMB: Confirmed missing |
| PO-5.5 | Session restore after a crash | **Designed** | SH 6.8 "Session store and restore"; SH 6.14 | Stage 1, SH-M3 | map: Covered; RBP: Designed |
| PO-5.6 | Safe mode / troubleshoot mode | **Designed** | D-S2, SH-D22 (SH 6.14) | Stage 1, SH-M4 | map: Covered; RBP: Designed |
| PO-5.7 | Usage telemetry | **Partial** | D-S4; S1 6 "Usage telemetry" **→ FI-11** | Stage 1 | map: Partial |
| PO-5.8 | Telemetry consent and deletion | **Partial** | D-S4; S1 13.2 "Privacy and legal"; SH 6.14 **→ FI-11** | Stage 1 (before M7) | map: Partial |
| PO-5.9 | Field trials and server-side experiments | **Missing** | — **→ FI-11** | — | map: Missing; VMB: Confirmed missing |
| PO-5.10 | Local feature flags page | **Designed** | SH 6.9 "No hidden settings" | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-5.11 | Troubleshooting and support pages | **Designed** | SH 6.14 "About pages"; SEC 6.7 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-5.12 | User feedback reporting | **Partial** | SH 6.14 (`about:diagnostics` "record this UI bug"); CB-D18 **→ FI-11** | Stage 1 | map: Partial |
| PO-5.13 | Network and performance trace capture | **Designed** | DT 6.3 "Exports" (`hg-export`, `hg-redact`) | HAR Stage 1; qlog, Perfetto Stage 2; OTLP Stage 3 | map: Covered; RBP: Designed |

### PO 6. Enterprise policy and management

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-6.1 | Windows Group Policy (ADMX/ADML) [SB] | **Missing** | — (S1 1 gives the broker "profile and policy" ownership) **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.2 | MDM on Windows (Intune / OMA-URI / ADMX ingestion) [SB] | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.3 | macOS configuration profiles [SB] | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.4 | Apple declarative device management for Safari | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.5 | Linux policy files [SB] | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.6 | Cross-platform policy file | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.7 | Cloud browser management | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.8 | Policy precedence and `about:policy` | **Partial** | SH 6.14 (`about:policies`); NW 6.14 (`izcal://policy`); SH 6.9 **→ FI-3** | Stage 1 | map: Partial. *policy page is `about:policies` in shell 6.9/6.14 and `izcal://policy` in NW 6.14* |
| PO-6.9 | Policy breadth (the policy catalogue) | **Partial** | SH 6.9 registry ("whether it can be set by policy"); NW 6.14; H3 B10; DV 6.15; JS-D13; EX 6.6; AH 6.7 **→ FI-3** | Stage 1 | map: Partial |
| PO-6.10 | Managed bookmarks | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.11 | Extension allow/block lists and force-install | **Partial** | EX 6.6, 6.10; Q-EX-5 **→ FI-3** | Stage 2 | map: Partial |
| PO-6.12 | URL allow/block lists | **Partial** | PH 6.11 (enterprise policy); NW 6.14 **→ FI-3** | Stage 1 | map: Partial |
| PO-6.13 | Lockable preferences | **Mentioned** | SH 6.9; H3 B10; ML-D4 **→ FI-3** | Stage 1 (display only) | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| PO-6.14 | Enterprise reporting | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.15 | Data loss prevention / content-analysis connectors | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.16 | Enterprise sign-in and profile separation | **Partial** | SH-D26 (containers); SH 6.12 (no account) **→ FI-3** | Stage 1 keying, Stage 2 container UI | map: Partial |
| PO-6.17 | Legacy browser support / IE mode | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.18 | Native messaging hosts | **Designed** | EX 6.11, EX-D9 | Stage 2 (EX-M3) | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-6.19 | Enterprise admin documentation set | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-6.20 | AI and generative-feature controls | **Designed** | H3 B10, D13; ML-D4 | Stage 1 | map: Covered; RBP: Designed |

### PO 7. Enterprise networking, authentication and certificates

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-7.1 | Integrated Windows authentication (Negotiate, Kerberos, NTLM) [SB] | **Designed** | NW 6.1, NW-D20; NW 6.14 | Stage 1 (NW-M4) | map: Covered; RBP: Designed |
| PO-7.2 | Kerberos on macOS and Linux | **Designed** | NW 6.1, NW-D20 | Stage 1 | map: Covered; RBP: Designed |
| PO-7.3 | OS single sign-on to cloud identity | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-7.4 | Basic and Digest auth dialogs | **Designed** | NW 6.14 "HTTP authentication" | Stage 1 (NW-M4) | map: Covered; RBP: Designed |
| PO-7.5 | Client certificates (mutual TLS) [SB] | **Designed** | NW 6.14, 6.1 | Stage 1 (NW-M4) | map: Covered; RBP: Designed |
| PO-7.6 | Smart cards and PKCS#11 modules | **Partial** | NW 6.1 ("client-key signing (smart cards and OS key stores)"); EX 6.3 (refuses `pkcs11`) **→ FI-16** | Stage 1 | map: Partial |
| PO-7.7 | Proxy support: manual, PAC, WPAD, SOCKS [SB] | **Designed** | NW 6.13, 6.1 | Stage 1 | map: Covered; RBP: Designed |
| PO-7.8 | Enterprise and locally installed roots (TLS inspection compatibility) [SB] | **Designed** | NW 6.5, 6.1 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-7.9 | TLS version and cipher policies | **Partial** | — **→ FI-3** | — | map: Missing; VMB: Actually partial |
| PO-7.10 | DNS-over-HTTPS policy and resolver choice | **Designed** | NW 6.6, NW-D8; NW 6.14 | Stage 1 | map: Covered; RBP: Designed |
| PO-7.11 | Encrypted Client Hello controls | **Designed** | NW 6.5; NW 6.14 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-7.12 | Intranet and single-word navigation | **Missing** | — **→ FI-16** | — | map: Missing; VMB: Confirmed missing |
| PO-7.13 | Private network access controls | **Designed** | NW 6.1 (LNA tickets); NW 6.14 | Stage 1 | map: Covered; RBP: Designed |
| PO-7.14 | Captive portal detection | **Designed** | NW 6.13 "Captive portals" | Stage 1 | map: Covered; RBP: Designed |
| PO-7.15 | Browser VPN / proxy privacy service | **Refused** | S1 §6 out-of-scope list; S1 §6 "Containers"; NW-D6 | — | map: Excluded; VCE: Confirmed (weak); RBP: Refused |
| PO-7.16 | OS VPN and per-app VPN compatibility | **Partial** | — **→ FI-16** | — | map: Missing; VMB: Actually partial |
| PO-7.17 | Reputation and phishing protection for enterprise | **Partial** | PH 6.11 "Enterprise policy"; PH (Chrome Enterprise lesson) **→ FI-13** | Stage 1 | map: Partial |

### PO 8. Default browser and OS shell integration

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-8.1 | Default-browser registration on Windows [SB] | **Partial** | SH 6.11; S1 6 "Default-browser handling"; UI R19 **→ FI-4** | Stage 1 | map: Partial |
| PO-8.2 | Default browser on macOS [SB] | **Partial** | SH 6.11 **→ FI-4** | Stage 1 | map: Partial |
| PO-8.3 | Default browser on Linux [SB] | **Partial** | SH 6.11 **→ FI-4** | Stage 1 | map: Partial |
| PO-8.4 | Default browser agent and prompts | **Designed** | SH 6.11; UI R19, UP3 | Stage 1 | map: Covered; RBP: Designed |
| PO-8.5 | Enterprise default-browser assignment | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-8.6 | Protocol and file-type associations | **Partial** | SEC SH-R8; SH 6.10; PH 6.4.6 **→ FI-4** | Stage 1 | map: Partial |
| PO-8.7 | Taskbar and dock integration | **Partial** | TX-16 (TX 6.x) **→ FI-4** | Stage 1 | map: Partial |
| PO-8.8 | Downloads: Mark of the Web and quarantine | **Designed** | SH 6.10; SEC SH-R6; NW 6.14 | Stage 1 | map: Covered; RBP: Designed |
| PO-8.9 | Antivirus scan on download | **Partial** | SH 6.10 (verdict source "the OS reputation interface") **→ FI-4** | Stage 1 | map: Partial |
| PO-8.10 | Open download / show in folder | **Designed** | SH 6.10; SH 6.3 `shell.downloads` | Stage 1 | map: Covered; RBP: Designed |
| PO-8.11 | Background mode and start at login | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| PO-8.12 | OS scripting and automation hooks | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| PO-8.13 | Desktop wallpaper and OS actions from context menus | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| PO-8.14 | Continuity and cross-device handoff | **Deferred, no owner** | Shell 9.1; S1 §9 Stage 3 "Sync (if wanted)"; Q-SH-5 **→ FI-35** | Stage 3 or never | map: Missing; VMB: Actually excluded. *VMB said Excluded; filed Deferred, no owner as for BF-17.7* |

### PO 9. OS service integration

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-9.1 | OS accessibility API: Windows [SB] | **Designed** | AX (AX-D3, AX-D4, AX-D9) | Stage 1 | map: Covered; RBP: Designed |
| PO-9.2 | OS accessibility API: macOS [SB] | **Designed** | AX §2/§6 OS interfaces | Stage 1 | map: Covered; RBP: Designed |
| PO-9.3 | OS accessibility API: Linux [SB] | **Designed** | AX, AX-D12 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.4 | Accessibility API mappings conformance | **Designed** | AX 6.2 (role tables) | Stage 1 | map: Covered; RBP: Designed |
| PO-9.5 | IME and text services [SB] | **Designed** | ED 6.x (ED-D5 to ED-D7); S1 13.2 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.6 | Touch keyboard and handwriting | **Designed** | ED 6.13, ED-D18 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.7 | OS notifications [SB] | **Deferred** | Shell 6.13, SH-D17; Q-SH-10 | Stage 2 | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred. *same feature as BF-17.5* |
| PO-9.8 | System media controls | **Partial** | MD 6.12 (MD-D13, MD-D25) **→ FI-4** | Stage 1 | map: Partial |
| PO-9.9 | OS share sheet | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| PO-9.10 | OS credential store for secrets [SB] | **Designed** | SH 6.12; SEC SH-R1, SEC-34 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.11 | Passkeys and platform authenticators | **Partial** | S1 6 "Passkeys and credentials"; SEC SH-R1; DV 6.x ("FIDO/CTAP remains in the credential subsystem") **→ FI-13** | Stage 1 | map: Partial |
| PO-9.12 | OS biometric re-authentication | **Designed** | SH 6.12 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-9.13 | OS spellchecker | **Designed** | ED 6.8, ED-D8 (b) | Stage 2 (restaged 21 Oct; map said Stage 1) | map: Covered; VCE: Only partial (wrong stage); RBP: Designed (checks differ; the later check wins). *VCE said Only partial (wrong stage); RBP Designed, Stage 2* |
| PO-9.14 | OS printing [SB] | **Deferred, no owner** | S1 §6 "Printing"; GR 6.27 (raster/vector only); PD 6.16 **→ FI-8** | Stage 1 layout; Stage 2 print UI | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred, no owner. *VCE: confirmed deferral; RBP: shell 6.19 and PD-M4 each point to the other, so nobody owns the print UI* |
| PO-9.15 | Save as PDF | **Deferred** | PD 6.19 (`hg-pdf-write`/`hg-pdf-print`), PD-D19–21 | Stage 2 (PD-M4) | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred |
| PO-9.16 | OS file pickers [SB] | **Partial** | S1 13.2 "Permissions" (File System Access); SEC SH-R9; SH 6.9 ("ask where to save") **→ FI-5** | Stage 1 | map: Partial |
| PO-9.17 | Clipboard and drag-and-drop | **Designed** | ED 6.10 (ED-D10–D12), 6.11 (ED-D13) | Stage 1 | map: Covered; RBP: Designed |
| PO-9.18 | Screen capture backends | **Designed** | MD capture service (MD 6.x); GR 6.27 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.19 | Camera and microphone stacks and OS privacy prompts | **Designed** | MD capture service | Stage 1 | map: Covered; RBP: Designed |
| PO-9.20 | OS location services | **Partial** | S1 13.2 "Permissions"; DM 6.x (Geolocation bindings) **→ FI-28** | Stage 1 | map: Partial |
| PO-9.21 | Audio output and device routing | **Designed** | MD 6.8 audio service | Stage 1 | map: Covered; RBP: Designed |
| PO-9.22 | OS speech synthesis and recognition | **Deferred, no owner** | EX 9.1 (TTS, Stage 3, needs a host surface); no owner **→ FI-29** | Stage 3 (EX 9.1) | map: Missing; VMB: Actually excluded. *VMB said Excluded: EX 9.1 names TTS for Stage 3 with no owner* |
| PO-9.23 | OS device stacks: USB, HID, serial, Bluetooth, game controllers | **Partial** | DV (DV-D1 to DV-D15) **→ FI-28** | Stage 1 boundary | map: Partial |
| PO-9.24 | Screen and system wake locks | **Partial** | DM 6.x (Screen Wake Lock binding, OS side assigned to shell and security) **→ FI-28** | Stage 1 | map: Partial |
| PO-9.25 | OS proxy and network-change notifications | **Designed** | NW 6.1 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.26 | OS certificate stores and trust evaluation | **Designed** | NW 6.1, 6.5.2 | Stage 1 | map: Covered; RBP: Designed |
| PO-9.27 | Parental controls integration | **Missing** | — **→ FI-38** | — | map: Missing; VMB: Confirmed missing |

### PO 10. Graphics, media hardware, power and display

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-10.1 | GPU driver blocklist and workarounds [SB] | **Designed** | GR 6.18, GR-D19; SEC GPU-R8 | Stage 1 | map: Covered; RBP: Designed |
| PO-10.2 | GPU process isolation and device-loss recovery | **Designed** | GR 6.17, 6.18; H2 5.5 | Stage 1 | map: Covered; RBP: Designed |
| PO-10.3 | Hardware acceleration toggle | **Mentioned** | SH 6.14, SH-D22 (recovery window) **→ FI-21** | Stage 1 (recovery only) | map: Covered; RBP: Mentioned |
| PO-10.4 | Graphics API backends | **Designed** | GR (`hg-gpu-hal`), H2 D3 | Stage 1, GR-M1 | map: Covered; RBP: Designed |
| PO-10.5 | Hardware video decode and encode | **Designed** | MD 6.x (MD-D10) | Stage 1 | map: Covered; RBP: Designed |
| PO-10.6 | Codec patent licensing via OS decoders | **Owned** | S1 13.2 "Codecs"; MD-D3 | Stage 1 (legal review before M7) | map: Covered; VCE: Confirmed covered; RBP: Owned |
| PO-10.7 | Hardware-backed DRM paths | **Designed** | S1 13.1a (D-S7) | Stage 1 (M7) | map: Covered; RBP: Designed |
| PO-10.8 | Power management and OS QoS | **Designed** | H2 4.3; FP 6.7, 6.8; GR 6.9 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-10.9 | Battery and memory saver modes | **Designed** | FP-D15, FP 6.8; S1 6 | Stage 1 (discard), Stage 2 ("limit on battery") | map: Covered; RBP: Designed |
| PO-10.10 | HiDPI and per-monitor scaling [SB] | **Partial** | GR-D17 (device scale as a rational); GR presentation table (fractional-scale); SH 5 **→ FI-4** | Stage 1 | map: Partial |
| PO-10.11 | HDR and wide colour | **Designed** | GR 6.5 (GR-D16); MD-D9 | Stage 1 | map: Covered; RBP: Designed |
| PO-10.12 | Variable refresh rate and vsync source | **Designed** | GR 6.9 (GR-D15); AN | Stage 1 | map: Covered; RBP: Designed |
| PO-10.13 | Dark mode, system theme and accent colour [SB] | **Designed** | SH 6.2 theming, 6.9 Appearance; ST UA variants | Stage 1 | map: Covered; RBP: Designed |
| PO-10.14 | High contrast and forced colours | **Designed** | AX 6.11 (AX-D18); UI 6 | Stage 1 | map: Covered; RBP: Designed |
| PO-10.15 | OS reduced motion and text scaling | **Designed** | AX 6.11 (AX-D18, AX-D19); AN 6.11 | Stage 1 | map: Covered; RBP: Designed |
| PO-10.16 | Window management integration | **Partial** | SH 6.2 "Native window frame" **→ FI-4** | Stage 1 | map: Partial |
| PO-10.17 | Touch, pen and precision touchpad gestures | **Partial** | GR 6.x (pinch zoom, LY-D13) **→ FI-4** | Stage 1 | map: Partial |
| PO-10.18 | Multi-monitor window placement | **Partial** | SH 6.8 (session store keeps "window bounds and display"); SH 5 **→ FI-4** | Stage 1 | map: Partial |

### PO 11. Platform targets and OS security primitives

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-11.1 | Windows x64 [SB] | **Owned** | S1 D-S5; GR-D21 | Stage 1 (sandbox M5, complete before M8) | map: Covered; RBP: Owned |
| PO-11.2 | Windows on ARM64 | **Partial** | FP-25 (per architecture: x86-64, AArch64, riscv64); JS ISA targets **→ FI-2** | Stage 1 | map: Partial |
| PO-11.3 | macOS on Apple silicon and Intel [SB] | **Owned** | GR-D21; S1 D-S5 | Stage 1 | map: Covered; RBP: Owned |
| PO-11.4 | Linux x86-64 on Wayland and X11 [SB] | **Designed** | GR 6.10 presentation table; ED 6.x (ED-D5); GR-D21 | Stage 1 | map: Covered; RBP: Designed |
| PO-11.5 | Linux ARM64 | **Mentioned** | FP-25; JS ISAs **→ FI-2** | — (no Linux ARM64 target) | map: Covered; VCE: Only partial; RBP: Mentioned (checks differ; the later check wins) |
| PO-11.6 | Windows sandbox primitives [SB] | **Designed** | SEC 6.7; S1 D-S5 | Stage 1 (M5) | map: Covered; RBP: Designed |
| PO-11.7 | Windows exploit mitigations | **Designed** | SEC 6.7; SEC-D5 | Stage 1 (broker CIG measured at M5) | map: Covered; RBP: Designed |
| PO-11.8 | macOS sandbox primitives [SB] | **Designed** | SEC 6.7; SEC-D6 | Stage 1 | map: Covered; RBP: Designed |
| PO-11.9 | Linux sandbox primitives [SB] | **Designed** | SEC 6.7; FP-D1 | Stage 1 | map: Covered; RBP: Designed |
| PO-11.10 | Android (later) | **Deferred, no owner** | H2 S-8; S1 §1, §9; CB "Interfaces for later stages" **→ FI-33** | Stage 2 | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred, no owner. *VCE: confirmed deferral; RBP: only H2 S-8 and S1 name it; no Android shell or packaging spec* |
| PO-11.11 | iOS/iPadOS (later) | **Refused** | S1 13.2 "iOS" | — (Stages 1–2) | map: Excluded; VCE: Confirmed; RBP: Refused |

### PO 12. Web apps installed into the OS

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-12.1 | Web app manifest processing | **Missing** | — **→ FI-9** | — | map: Missing; VMB: Confirmed missing |
| PO-12.2 | PWA install into the OS | **Partial** | PH 6.4.9 (assumes installed web apps); AH 2–3 (PWA as "the incumbent") **→ FI-9** | — | map: Partial |
| PO-12.3 | PWA uninstall through the OS | **Missing** | — **→ FI-9** | — | map: Missing; VMB: Confirmed missing |
| PO-12.4 | OS-level app integration | **Missing** | — **→ FI-9** | — | map: Missing; VMB: Confirmed missing |
| PO-12.5 | Enterprise force-installed web apps | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |
| PO-12.6 | Store packaging of PWAs | **Missing** | — **→ FI-9** | — | map: Missing; VMB: Confirmed missing |
| PO-12.7 | Site-specific shortcuts and "taskbar tabs" | **Missing** | — **→ FI-9** | — | map: Missing; VMB: Confirmed missing |

### PO 13. Embedding, automation, headless and command line

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-13.1 | Embedding API (WebView) | **Deferred, no owner** | S1 13.2 "Embedding API"; CB (lists as data) **→ FI-34** | Stage 2 | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred, no owner. *VCE: confirmed deferral; RBP: one S1 row, no spec designs the WebView API* |
| PO-13.2 | Embedded runtime distribution and update | **Missing** | — (S1 13.2 defers only the API) **→ FI-34** | — | map: Deferred (Stage 2); VCE: Misread; RBP: Misread → Missing |
| PO-13.3 | WebDriver classic [SB] | **Designed** | DT 6.3 (`hg-bidi`, `izcaldriver`); DT-5 | Stage 1 (M2 slice, M7 full) | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-13.4 | WebDriver BiDi | **Designed** | DT 6.3; DT-5 | Stage 1 (M7) | map: Covered; RBP: Designed |
| PO-13.5 | CDP-compatible remote debugging endpoint | **Designed** | DT 6.3 (DT-D10, DT-D12); S1 D-S11 | Stage 1 (JS, DOM, CSS, Network, Page, Overlay); Stage 2 rest | map: Covered; RBP: Designed |
| PO-13.6 | Safari remote automation and inspection | **Designed** | DT 6.3 "WebDriver's HTTP server" | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-13.7 | Headless mode [SB] | **Partial** | DT 6.3 ("headless embedders" over `--automation-pipe`); H3 B7 (pixel-optional tabs) **→ FI-7** | Stage 1 / Stage 2 | map: Partial |
| PO-13.8 | Testing builds | **Missing** | — **→ FI-7** | — | map: Missing; VMB: Confirmed missing |
| PO-13.9 | Command-line switches | **Partial** | SEC-20, AGT-R8; DT 6.3 (`--automation-pipe`) **→ FI-4** | Stage 1 | map: Partial |
| PO-13.10 | Single-instance and remote control of a running browser | **Missing** | — **→ FI-4** | — | map: Missing; VMB: Confirmed missing |
| PO-13.11 | Developer tools enablement policy | **Missing** | — **→ FI-3** | — | map: Missing; VMB: Confirmed missing |

### PO 14. Localisation and the browser's own accessibility

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-14.1 | UI localisation at scale [SB] | **Mentioned** | Shell 4.4 (T12), 6.17 "Strings"; S1 13.2 "Text and input" **→ FI-6** | — (Stage 1 English-only; no later stage) | map: Excluded; VCE: Misread; RBP: Misread → Mentioned |
| PO-14.2 | Locale selection and language packs | **Partial** | SH 6.9 "Language and writing" (Stage 2); FP 6.x optional components **→ FI-6** | Stage 2 | map: Partial |
| PO-14.3 | Right-to-left UI mirroring | **Partial** | SH 6.16 "Strings and bidirectional text" **→ FI-6** | Stage 1 (bidi isolation) | map: Partial |
| PO-14.4 | Translation and localisation pipeline | **Partial** | SH-D24; SH 9.3 track T12; SH 6.16 **→ FI-6** | Stage 1 (string layer) | map: Partial |
| PO-14.5 | Regional builds and defaults | **Missing** | — **→ FI-6** | — | map: Missing; VMB: Confirmed missing |
| PO-14.6 | Browser UI accessibility conformance [SB] | **Partial** | SH 6.16; UI-2; S1 13.2 "Shell"; AX targets **→ FI-1** | Stage 1 | map: Partial |
| PO-14.7 | Full keyboard access and focus management in UI | **Designed** | UI-2; SH 6.16; AX 6.12 (AX-D20, AX-D21) | Stage 1 (ship blocker) | map: Covered; RBP: Designed |
| PO-14.8 | Accessibility settings in the browser | **Designed** | SH 6.9 "Accessibility"; AX 6.11–6.12 | Stage 1 | map: Covered; RBP: Designed |

### PO 15. Legal, regulatory and compliance

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-15.1 | Privacy notice and terms of use [SB] | **Partial** | S1 13.2 "Privacy and legal" **→ FI-1** | Stage 1 (before M7) | map: Partial |
| PO-15.2 | Third-party licence notices [SB] | **Designed** | SH 6.14 About pages; SH G209 | Stage 1 | map: Covered; RBP: Designed |
| PO-15.3 | GDPR data access and export | **Partial** | SH 6.12 (`izcal-export` v1); S1 13.2 "Privacy and legal" **→ FI-1** | Stage 1 | map: Partial |
| PO-15.4 | Data deletion | **Partial** | SH 6.9, 6.12; SEC SH-R5 **→ FI-1** | Stage 1 | map: Partial |
| PO-15.5 | DMA browser choice screens | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing; FC. *FC 4a–4d: Izcal need not show a choice screen; the work is eligibility for Apple's and Android's screens (an iOS build, about 5,000 EU iPhone users a year)* |
| PO-15.6 | DMA data portability | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing |
| PO-15.7 | Age assurance and age signals | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing |
| PO-15.8 | Encryption export classification | **Owned** | S1 13.2 "Privacy and legal" | Stage 1 (before first public binary) | map: Covered; VCE: Confirmed covered; RBP: Owned |
| PO-15.9 | EU Cyber Resilience Act [SB] | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing; FC. *FC 1a–1e: Annex III Class I item 2; Art. 14 reporting applies from 11 September 2026 (24 h / 72 h / 14 days, via ENISA's platform); support period ≥ 5 years (Art. 13(8)); Art. 32(2) conformity route* |
| PO-15.10 | European Accessibility Act exposure | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing; FC. *FC 5a: the EAA never names browsers; exposure is indirect* |
| PO-15.11 | Trademark and brand rules | **Partial** | D-S9; S1 13.2 "Privacy and legal" **→ FI-1** | Stage 1 (before first public binary) | map: Partial |
| PO-15.12 | Codec, DRM and font licences | **Owned** | S1 13.2 "Codecs"; S1 13.1a licensing track; TX-D24 | Stage 1 (codec review before M7) | map: Covered; RBP: Owned |
| PO-15.13 | Search engine agreements and default search | **Partial** | D-S8 (deferred); SH 6.8 ("the chosen search engine") **→ FI-18** | — | map: Partial |

### PO 16. Security response and operations

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-16.1 | Vulnerability disclosure policy and intake [SB] | **Owned** | S1 13.2 "Security response"; SEC 6.17 | Stage 1 (before first public binary) | map: Covered; RBP: Owned |
| PO-16.2 | Bug bounty | **Deferred** | SEC-D10 (b); S1 13.2 "Security response" | After funding (D-S8), not a stage | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred |
| PO-16.3 | CVE assignment (CNA) | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing |
| PO-16.4 | Public security advisories | **Owned** | S1 13.2; SEC 6.17; SEC-25 | Stage 1 | map: Covered; RBP: Owned |
| PO-16.5 | Fix-time targets [SB] | **Designed** | SEC-D1, SEC-1, SEC-25 | Stage 1 | map: Covered; RBP: Designed |
| PO-16.6 | Security severity rating scale | **Partial** | SEC-25 **→ FI-1** | Stage 1 | map: Partial |
| PO-16.7 | Coordination and embargoes | **Owned** | S1 13.2; SEC 6.17 | Stage 1 (before first public binary) | map: Covered; RBP: Owned |
| PO-16.8 | Continuous fuzzing and sanitiser builds | **Designed** | S1 7.5, 7.2; SEC-24 | Stage 1 | map: Covered; RBP: Designed |
| PO-16.9 | Release engineering and on-call | **Partial** | S1 7.8; SEC UPD-R3; JS-16 (24-hour on-call); SEC-D2, SEC-D4 **→ FI-2** | Stage 1 | map: Partial |
| PO-16.10 | Security notes for enterprise | **Missing** | — **→ FI-1** | — | map: Missing; VMB: Confirmed missing |

### PO 17. Back-end services a shipping browser depends on

| # | Feature | Final status | Owner (spec §) / decision | Lands in | Source of verdict |
|---|---|---|---|---|---|
| PO-17.1 | Update and download servers and CDN [SB] | **Owned** | SEC 6.16 (SEC-17); NW-D17; S1 13.2 "Updates" | Stage 1 | map: Covered; RBP: Owned |
| PO-17.2 | Web Push service [SB] | **Deferred** | NW 6.14 "Web Push" (client); S1 13.2 "Service workers" (autopush) | Stage 2 | map: Deferred (Stage 2); VCE: Confirmed; RBP: Deferred |
| PO-17.3 | Sync service | **Mentioned** | Shell 9.1 "Stage 3 or never"; SH-D20; Q-SH-5 open **→ FI-35** | — (open owner question) | map: Deferred (Stage 3); VCE: Misread; RBP: Misread → Mentioned. *VCE and RBP agree it is not a deferral; RBP grades it Mentioned (the same gap as the sync rows)* |
| PO-17.4 | Extension store and signing service | **Designed** | EX 6.12, EX-D10 | Stage 2 | map: Deferred (Stage 2); VCE: Misread; RBP: Misread → Designed |
| PO-17.5 | Safe-browsing and reputation service | **Designed** | PH 6.0–6.2 (`hg-phish-ingest`, `hg-phish-compile`); PH 9.2 | Stage 1 | map: Covered; VCE: Confirmed covered; RBP: Designed |
| PO-17.6 | Network geolocation provider | **Missing** | — **→ FI-28** | — | map: Missing; VMB: Confirmed missing |
| PO-17.7 | Speech, translation and suggestion services | **Partial** | ML (Stage 2 translate task); SH 6.8 (search suggestions from the chosen engine, after Enter) **→ FI-29** | Stage 2 | map: Partial |
| PO-17.8 | Crash and telemetry ingestion back end | **Partial** | S1 7.8; D-S4 **→ FI-11** | Stage 1 | map: Partial |
| PO-17.9 | Support site and community | **Missing** | — **→ FI-22** | — | map: Missing; VMB: Confirmed missing |

## Counts

Computed on 24 October 2026 by the synthesis script from the rows above, then re-checked by a second script that parses this table and recounts every status per inventory.

### By inventory and status

| Inventory | Rows | Designed | Owned | Held | Refused | Deferred | Deferred, no owner | Mentioned | Partial | Missing | **Covered** (Designed + Owned) | **Gaps** (Dn + Mentioned + Partial + Missing) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| WA Web platform APIs | 333 | 121 | 63 | 61 | 12 | 7 | 1 | 18 | 28 | 22 | 184 (55%) | 69 (21%) |
| CG CSS, graphics and media | 277 | 151 | 30 | 40 | 14 | 1 | 1 | 4 | 28 | 8 | 181 (65%) | 41 (15%) |
| BF Browser product features | 270 | 89 | 2 | 0 | 12 | 7 | 12 | 10 | 48 | 90 | 91 (34%) | 160 (59%) |
| PO OS integration and operations | 237 | 87 | 13 | 0 | 4 | 5 | 6 | 9 | 59 | 54 | 100 (42%) | 128 (54%) |
| **All** | **1117** | **448** | **108** | **101** | **42** | **20** | **20** | **41** | **163** | **174** | **556 (50%)** | **398 (36%)** |

Of the 101 Held rows, **77** are held by the rule only, with no per-feature record. Of the 42 Refused rows, **6** are implicit. On the strict reading, where a rule-only hold counts as a gap, the gap total is **475**.

### Gaps by group

| Group | Rows | Covered | Held | Refused | Deferred | Deferred, no owner | Mentioned | Partial | Missing | **Gaps** |
|---|---|---|---|---|---|---|---|---|---|---|
| WA 1. HTML: documents, browsing contexts, navigation and history | 19 | 14 | 0 | 1 | 0 | 0 | 2 | 2 | 0 | **4** |
| WA 2. HTML: elements, attributes and document metadata | 18 | 15 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | **0** |
| WA 3. HTML: forms and form controls | 13 | 6 | 2 | 0 | 0 | 0 | 2 | 1 | 2 | **5** |
| WA 4. HTML: interactive UI, focus, dialogs and drag and drop | 18 | 14 | 3 | 0 | 0 | 0 | 0 | 0 | 1 | **1** |
| WA 5. HTML: scripting, modules and resource loading | 13 | 9 | 2 | 0 | 0 | 0 | 0 | 0 | 2 | **2** |
| WA 6. DOM, Web Components and DOM-adjacent APIs | 26 | 20 | 2 | 0 | 0 | 0 | 2 | 2 | 0 | **4** |
| WA 7. Event loop, timers, scheduling, workers and messaging | 15 | 14 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | **0** |
| WA 8. Fetch, networking and data formats | 20 | 10 | 2 | 1 | 0 | 0 | 3 | 3 | 1 | **7** |
| WA 9. Storage, files and quota | 12 | 7 | 2 | 1 | 0 | 0 | 0 | 1 | 1 | **2** |
| WA 10. Service workers, background work, push and notifications | 11 | 4 | 3 | 1 | 2 | 1 | 0 | 0 | 0 | **1** |
| WA 11. Installable web apps and OS integration | 12 | 1 | 7 | 1 | 0 | 0 | 0 | 1 | 2 | **3** |
| WA 12. Permissions, clipboard, sharing and other user-mediated APIs | 12 | 3 | 4 | 1 | 0 | 0 | 1 | 1 | 2 | **4** |
| WA 13. Security and isolation (page-visible) | 14 | 10 | 2 | 0 | 0 | 0 | 1 | 1 | 0 | **2** |
| WA 14. Privacy, tracking protection and advertising APIs | 16 | 6 | 9 | 1 | 0 | 0 | 0 | 0 | 0 | **0** |
| WA 15. Identity, credentials and payments | 11 | 0 | 6 | 0 | 0 | 0 | 0 | 3 | 2 | **5** |
| WA 16. Device and hardware APIs | 24 | 6 | 10 | 2 | 0 | 0 | 0 | 3 | 3 | **6** |
| WA 17. Performance, measurement and reporting | 18 | 6 | 2 | 2 | 0 | 0 | 4 | 3 | 1 | **8** |
| WA 18. ECMAScript language (coarse grain) | 26 | 17 | 1 | 0 | 0 | 0 | 2 | 4 | 2 | **8** |
| WA 19. Intl and Temporal | 6 | 5 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | **1** |
| WA 20. WebAssembly | 12 | 10 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | **1** |
| WA 21. On-device AI and agent-facing APIs | 5 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 | **0** |
| WA 22. Browser-shipped protocols and infrastructure standards | 12 | 7 | 0 | 0 | 0 | 0 | 0 | 2 | 3 | **5** |
| CG 1. CSS: syntax, cascade and selectors | 21 | 16 | 3 | 0 | 0 | 0 | 0 | 1 | 1 | **2** |
| CG 2. CSS: values, units, functions and conditional rules | 20 | 11 | 5 | 0 | 0 | 0 | 0 | 4 | 0 | **4** |
| CG 3. CSS: colour, images, backgrounds and borders | 16 | 10 | 4 | 0 | 0 | 0 | 0 | 2 | 0 | **2** |
| CG 4. CSS: box model, display and layout modes | 33 | 21 | 9 | 1 | 0 | 0 | 0 | 2 | 0 | **2** |
| CG 5. CSS: scrolling, fragmentation and paged media | 11 | 5 | 3 | 1 | 0 | 0 | 0 | 2 | 0 | **2** |
| CG 6. CSS: text, fonts, lists and generated content | 22 | 15 | 1 | 1 | 0 | 0 | 1 | 4 | 0 | **5** |
| CG 7. CSS: transforms, animation and visual effects | 18 | 15 | 1 | 1 | 0 | 0 | 0 | 1 | 0 | **1** |
| CG 8. CSS: object models, Houdini and observers | 9 | 6 | 2 | 0 | 0 | 0 | 0 | 1 | 0 | **1** |
| CG 9. CSS and HTML: forms, UI and interactivity | 11 | 4 | 4 | 0 | 0 | 0 | 0 | 3 | 0 | **3** |
| CG 10. SVG and MathML | 8 | 6 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | **0** |
| CG 11. 2D and 3D graphics APIs | 14 | 11 | 2 | 0 | 0 | 0 | 0 | 1 | 0 | **1** |
| CG 12. Image formats and colour management | 17 | 13 | 0 | 3 | 0 | 0 | 1 | 0 | 0 | **1** |
| CG 13. Video and audio formats | 21 | 16 | 0 | 5 | 0 | 0 | 0 | 0 | 0 | **0** |
| CG 14. HTML media elements and streaming | 25 | 20 | 2 | 0 | 0 | 0 | 0 | 2 | 1 | **3** |
| CG 15. Media capture and real-time communication | 20 | 7 | 4 | 0 | 0 | 0 | 2 | 3 | 4 | **9** |
| CG 16. XR, speech, printing and other rendering features | 11 | 5 | 0 | 0 | 1 | 1 | 0 | 2 | 2 | **5** |
| BF 1. Windows, tabs and sessions | 24 | 7 | 0 | 2 | 3 | 2 | 2 | 1 | 7 | **12** |
| BF 2. Address bar, search and navigation | 17 | 3 | 0 | 2 | 1 | 0 | 0 | 2 | 9 | **11** |
| BF 3. Bookmarks, history, reading list and start page | 14 | 3 | 0 | 1 | 0 | 0 | 0 | 6 | 4 | **10** |
| BF 4. Profiles, accounts, sync and cross-device | 11 | 1 | 0 | 0 | 0 | 6 | 1 | 2 | 1 | **10** |
| BF 5. Downloads, files, saving, printing and PDF | 11 | 7 | 0 | 0 | 0 | 0 | 1 | 0 | 3 | **4** |
| BF 6. Passwords, passkeys, autofill and payments | 15 | 4 | 0 | 0 | 2 | 0 | 2 | 0 | 7 | **9** |
| BF 7. Privacy controls | 18 | 11 | 0 | 1 | 0 | 0 | 1 | 4 | 1 | **6** |
| BF 8. Security and safety UI | 12 | 6 | 0 | 0 | 0 | 0 | 0 | 4 | 2 | **6** |
| BF 9. Site permissions and page info | 12 | 8 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | **4** |
| BF 10. Content, reading and page tools | 18 | 3 | 0 | 1 | 0 | 1 | 0 | 5 | 8 | **14** |
| BF 11. Media | 10 | 3 | 0 | 0 | 0 | 0 | 0 | 2 | 5 | **7** |
| BF 12. Performance and resource controls | 8 | 2 | 0 | 1 | 0 | 0 | 1 | 3 | 1 | **5** |
| BF 13. Extensions, themes and customisation | 20 | 9 | 0 | 1 | 0 | 0 | 0 | 2 | 8 | **10** |
| BF 14. Accessibility | 9 | 8 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | **1** |
| BF 15. AI features and their off switches | 18 | 3 | 0 | 0 | 1 | 1 | 1 | 3 | 9 | **14** |
| BF 16. Built-in apps and services | 14 | 1 | 0 | 1 | 0 | 1 | 0 | 2 | 9 | **12** |
| BF 17. Web apps and operating-system integration | 12 | 2 | 0 | 1 | 0 | 1 | 0 | 2 | 6 | **9** |
| BF 18. Developer and power-user entry points | 7 | 4 | 0 | 0 | 0 | 0 | 1 | 1 | 1 | **3** |
| BF 19. Setup, onboarding and maintenance | 8 | 5 | 0 | 1 | 0 | 0 | 0 | 2 | 0 | **2** |
| BF 20. Enterprise, family and special modes | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 4 | **6** |
| BF 21. Mobile-specific product features | 6 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 5 | **5** |
| PO 1. Packaging and installation | 17 | 1 | 0 | 0 | 1 | 1 | 1 | 6 | 7 | **15** |
| PO 2. Code signing and platform trust | 10 | 6 | 0 | 1 | 0 | 0 | 0 | 2 | 1 | **3** |
| PO 3. Updates and release channels | 15 | 4 | 0 | 0 | 0 | 0 | 3 | 4 | 4 | **11** |
| PO 4. Component and data updates (pushed outside the binary) | 17 | 15 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | **1** |
| PO 5. Crash reporting, telemetry and experimentation | 13 | 7 | 0 | 0 | 0 | 0 | 0 | 4 | 2 | **6** |
| PO 6. Enterprise policy and management | 20 | 2 | 0 | 0 | 0 | 0 | 1 | 5 | 12 | **18** |
| PO 7. Enterprise networking, authentication and certificates | 17 | 10 | 0 | 1 | 0 | 0 | 0 | 4 | 2 | **6** |
| PO 8. Default browser and OS shell integration | 14 | 3 | 0 | 0 | 0 | 1 | 0 | 6 | 4 | **11** |
| PO 9. OS service integration | 27 | 15 | 0 | 0 | 2 | 2 | 0 | 6 | 2 | **10** |
| PO 10. Graphics, media hardware, power and display | 18 | 13 | 0 | 0 | 0 | 0 | 1 | 4 | 0 | **5** |
| PO 11. Platform targets and OS security primitives | 11 | 7 | 0 | 1 | 0 | 1 | 1 | 1 | 0 | **3** |
| PO 12. Web apps installed into the OS | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 6 | **7** |
| PO 13. Embedding, automation, headless and command line | 11 | 4 | 0 | 0 | 0 | 1 | 0 | 2 | 4 | **7** |
| PO 14. Localisation and the browser's own accessibility | 8 | 2 | 0 | 0 | 0 | 0 | 1 | 4 | 1 | **6** |
| PO 15. Legal, regulatory and compliance | 13 | 3 | 0 | 0 | 0 | 0 | 0 | 5 | 5 | **10** |
| PO 16. Security response and operations | 10 | 5 | 0 | 0 | 1 | 0 | 0 | 2 | 2 | **4** |
| PO 17. Back-end services a shipping browser depends on | 9 | 3 | 0 | 0 | 1 | 0 | 1 | 2 | 2 | **5** |

### Ship-blocker and table-stakes rows

| Tag | Rows | Designed | Owned | Held | Refused | Deferred | Deferred, no owner | Mentioned | Partial | Missing | Gaps |
|---|---|---|---|---|---|---|---|---|---|---|---|
| [SB] (PO) | 55 | 22 | 7 | 0 | 0 | 2 | 2 | 2 | 13 | 7 | 24 |
| [TS] (BF) | 82 | 54 | 1 | 0 | 0 | 3 | 3 | 4 | 10 | 7 | 24 |

[SB] rows that are gaps: PO-1.1 (Partial, FI-2), PO-1.2 (Deferred, no owner, FI-3), PO-1.4 (Partial, FI-2), PO-1.6 (Partial, FI-2), PO-1.13 (Missing, FI-2), PO-2.2 (Missing, FI-2), PO-2.6 (Partial, FI-2), PO-3.1 (Partial, FI-2), PO-3.4 (Mentioned, FI-2), PO-6.1 (Missing, FI-3), PO-6.2 (Missing, FI-3), PO-6.3 (Missing, FI-3), PO-6.5 (Missing, FI-3), PO-8.1 (Partial, FI-4), PO-8.2 (Partial, FI-4), PO-8.3 (Partial, FI-4), PO-9.14 (Deferred, no owner, FI-8), PO-9.16 (Partial, FI-5), PO-10.10 (Partial, FI-4), PO-13.7 (Partial, FI-7), PO-14.1 (Mentioned, FI-6), PO-14.6 (Partial, FI-1), PO-15.1 (Partial, FI-1), PO-15.9 (Missing, FI-1).

### Rows per work item

| Work item | Type | Rows | WA | CG | BF | PO | Missing | Partial | Mentioned | Deferred, no owner | Held | Refused (to record) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| FI-1 Compliance and security-response duties | New spec | 14 | 0 | 0 | 0 | 14 | 7 | 6 | 1 | 0 | 0 | 0 |
| FI-2 Distribution, installers, channels and updates | New spec | 27 | 0 | 0 | 1 | 26 | 10 | 12 | 5 | 0 | 0 | 0 |
| FI-3 Enterprise policy delivery and management | New spec | 29 | 0 | 0 | 2 | 27 | 19 | 8 | 1 | 1 | 0 | 0 |
| FI-4 OS integration (default browser, handlers, share, taskbar, lifecycle) | New spec | 22 | 1 | 0 | 4 | 17 | 9 | 13 | 0 | 0 | 0 | 0 |
| FI-5 Script dialogs, form pickers and the file chooser | Amendment (shell 6.4, LY-D11, SHUI) | 9 | 6 | 1 | 1 | 1 | 4 | 2 | 3 | 0 | 0 | 0 |
| FI-6 Localisation of the UI | Amendment (shell 4.4, 6.17) | 5 | 0 | 0 | 0 | 5 | 1 | 3 | 1 | 0 | 0 | 0 |
| FI-7 Automation, headless and testing builds | Amendment (DevTools) | 6 | 3 | 0 | 1 | 2 | 4 | 2 | 0 | 0 | 0 | 0 |
| FI-8 Print UI and the OS print path: name one owner | Amendment (shell 6.19 or PD 9) | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| FI-9 Installed web apps | New spec (or app host amendment) | 23 | 14 | 0 | 3 | 6 | 8 | 4 | 0 | 1 | 10 | 0 |
| FI-10 Profiles, guest mode and shared devices | Amendment (shell; SG) | 6 | 0 | 0 | 5 | 1 | 2 | 4 | 0 | 0 | 0 | 0 |
| FI-11 Crash, telemetry, feedback and experiment operations | Amendment (S1 7.8) plus a short new spec | 9 | 1 | 0 | 0 | 8 | 2 | 7 | 0 | 0 | 0 | 0 |
| FI-12 Privacy spec (the one the security scope points to) | New spec | 16 | 15 | 0 | 1 | 0 | 1 | 4 | 0 | 0 | 11 | 0 |
| FI-13 Identity, credentials and payments | New spec | 19 | 11 | 0 | 6 | 2 | 5 | 5 | 3 | 0 | 6 | 0 |
| FI-14 DOM web-API table: owners, WPT floors and milestones | Amendment (DM 6.18, DM 8, DM 9) | 17 | 16 | 1 | 0 | 0 | 3 | 4 | 10 | 0 | 0 | 0 |
| FI-15 Web security policy: CSP 3, Permissions Policy, X-Frame-Options | Amendment (security scope; NW 6.7) | 4 | 4 | 0 | 0 | 0 | 0 | 2 | 1 | 0 | 1 | 0 |
| FI-16 Networking: headers, loading checks and enterprise network edges | Amendment (NW) | 12 | 9 | 0 | 0 | 3 | 4 | 3 | 0 | 0 | 5 | 0 |
| FI-17 Tabs, sessions, library and the side panel | Amendment (shell 6.7, 6.8; panel design for EX-D15) | 24 | 0 | 0 | 24 | 0 | 12 | 8 | 3 | 1 | 0 | 0 |
| FI-18 Address bar and search settings | Amendment (shell 6.8, 6.9) | 6 | 0 | 0 | 5 | 1 | 3 | 3 | 0 | 0 | 0 | 0 |
| FI-19 Privacy, permission and security surfaces | Amendment (shell 6.4, 6.6, 6.9; CB, PH, NW, MD) | 14 | 0 | 0 | 14 | 0 | 2 | 11 | 1 | 0 | 0 | 0 |
| FI-20 Page tools and media surfaces | Amendment (shell menus; MD 6.12) | 10 | 0 | 0 | 10 | 0 | 3 | 7 | 0 | 0 | 0 | 0 |
| FI-21 Performance controls and customisation | Amendment (FP; shell 6.2, 6.9, 6.16) | 11 | 0 | 0 | 10 | 1 | 3 | 6 | 2 | 0 | 0 | 0 |
| FI-22 Help, support and what's new | Amendment (shell 6.11, 6.14) | 3 | 0 | 0 | 2 | 1 | 1 | 2 | 0 | 0 | 0 | 0 |
| FI-23 CSS completeness and the Compat Standard | Amendment (ST 4.2, 7, ST-3) | 14 | 1 | 13 | 0 | 0 | 1 | 12 | 1 | 0 | 0 | 0 |
| FI-24 Paint and GPU API coverage | Amendment (GR 4.2, 7; LY) | 7 | 0 | 7 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | 0 |
| FI-25 Media API coverage | Amendment (MD 6.20, MD-D1) | 4 | 0 | 4 | 0 | 0 | 0 | 2 | 1 | 0 | 1 | 0 |
| FI-26 JS and Wasm proposal-stage rule | Amendment (JS 6.14, JS 9.1) | 10 | 10 | 0 | 0 | 0 | 2 | 5 | 3 | 0 | 0 | 0 |
| FI-27 WebRTC API layer and real-time media | New spec | 12 | 0 | 12 | 0 | 0 | 3 | 3 | 2 | 0 | 4 | 0 |
| FI-28 Sensors, screen and input APIs | New spec | 24 | 18 | 2 | 0 | 4 | 5 | 6 | 0 | 0 | 13 | 0 |
| FI-29 Speech: synthesis and recognition | New spec (MD 6.20 names it) | 7 | 1 | 3 | 1 | 2 | 1 | 3 | 1 | 1 | 1 | 0 |
| FI-30 Reading tools: reader view, read aloud, translation | New spec | 5 | 0 | 1 | 4 | 0 | 2 | 1 | 0 | 2 | 0 | 0 |
| FI-31 AI features with the switch on | New spec (under ML, HGBA-3 §7, B10) | 21 | 0 | 0 | 21 | 0 | 15 | 3 | 1 | 1 | 0 | 1 |
| FI-32 Casting and remote playback | Decision, then a small spec | 4 | 1 | 2 | 1 | 0 | 2 | 0 | 0 | 0 | 2 | 0 |
| FI-33 Mobile shell (Android, Stage 2) | New spec (Stage 2) | 17 | 0 | 3 | 13 | 1 | 13 | 2 | 0 | 1 | 1 | 0 |
| FI-34 Embedding API and shared runtime | Decision, then a Stage 2 spec | 2 | 0 | 0 | 0 | 2 | 1 | 0 | 0 | 1 | 0 | 0 |
| FI-35 Sync and accounts | Decision needed from the owner | 12 | 0 | 0 | 10 | 2 | 0 | 0 | 1 | 10 | 0 | 1 |
| FI-36 XR | Decision needed from the owner | 3 | 1 | 2 | 0 | 0 | 2 | 0 | 0 | 0 | 1 | 0 |
| FI-37 Admission records for single-engine features | Decision, then amendments (DM 6.3, ST 9, LY 9) | 24 | 12 | 12 | 0 | 0 | 0 | 3 | 0 | 0 | 21 | 0 |
| FI-38 Product non-goals record | Amendment (shell 4.3 or UI 8) | 28 | 0 | 0 | 27 | 1 | 24 | 0 | 0 | 0 | 0 | 4 |
| **All** | | **481** | | | | | | | | | | |

Every gap row, every rule-only hold and every implicit refusal is assigned to exactly one work item (unassigned: 0).

