# Izcal accessibility: frame

Written 19 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

Izcal needs its own accessibility stack, end to end. HGBA-3 B8 and Stage 1 Section 5.8 planned to feed the operating systems' accessibility APIs through AccessKit (`accesskit` 0.25, MIT OR Apache-2.0), used unmodified. HGBA-2 2.3.0 excludes every library that does browser-engine work, and the JavaScript engine spec's owner-accepted **JS-D11** classifies AccessKit as engine work, with "build" as M-1 part 1's default; the Phase 3 ledger records the gap as **G-10** (B8 and 5.8 rely on a superseded dependency) and **G-17** (the accessibility spec is unowned). Every other accepted spec already assumes an accessibility consumer: layout supplies bounds, line records, `reading-flow` order and the `content-visibility` rule (A-S1-45); the text stack supplies per-character bounds and word boundaries (A-S1-44, R24); the graphics spec puts forced colours and backplates in paint (GR-D25); the media spec requires accessible native controls (MD-12) and OS caption preferences (6.12); the DevTools spec builds its accessibility view from "the AccessKit update stream" (DT-D6); the shell design says the shell "exposes its tree through the same AccessKit path as content" (shell 4.9). None of them has a producer to call.

Scope of this spec:
- **The accessibility tree**: computed from the DOM, computed style and the fragment tree; roles, names and descriptions per accname; states, properties and relations; HTML-AAM, Core-AAM, WAI-ARIA 1.2 and 1.3, SVG-AAM, MathML-AAM, DPUB-ARIA and Graphics ARIA mappings; shadow DOM, custom elements (`ElementInternals`), element reflection and reference target; text, caret and selection exposure.
- **Incremental updates and events**: invalidation from the mutation log and the flush, event generation (focus, state, value, tree, text, live regions, `ariaNotify`), coalescing.
- **Izcal's own platform adapters** (JS-D11): macOS NSAccessibility, Windows UI Automation (and the IAccessible2/MSAA decision), Linux AT-SPI2 over D-Bus, Android `AccessibilityNodeInfo`.
- **Process placement and the cross-process protocol**: the tree is computed in the per-site renderer; the broker exposes it to the OS; out-of-process iframes are stitched; the protocol's security.
- **Performance**: large trees, lazy computation, memory when AT is off and on.
- **Interaction**: screen readers (browse and focus modes), switch and voice control, the focus and caret navigation boundary between shell and content, AT-initiated actions.
- **Style hooks**: forced colours and high contrast, reduced motion, reduced transparency, contrast preference, zoom and text scaling, captions (with the media spec).
- **The shell's own accessibility** (with the shell UI design), automated testing (WPT ARIA/AAM, black-box AT testing), privacy (AT use must not be a fingerprint), observability (the DevTools accessibility domain).

Not in scope: the repair overlay's rules and study (HGBA-3 B10, A11Y-1 to A11Y-4 stay where they are; this spec gives them a home in the pipeline), the on-device ML models for generated descriptions (MD-D25, future ML spec), editing commands (future editing spec; this spec owns their accessibility exposure), the PDF viewer's internals (future PDF spec; this spec owns the interface PDF content uses to reach the tree).

## Who has the problem

- **Assistive-technology users** (screen reader, magnifier, switch, voice control, braille): pages, the shell and every prompt must be fully usable, fast (no lag between action and speech), and must not reveal that they use AT.
- **Low-vision, motion-sensitive and cognitively disabled users without AT**: forced colours, zoom and text scaling, reduced motion, captions.
- **Web developers and accessibility practitioners**: "why does the screen reader say this?", "why is this not announced?", "what did the screen reader actually receive?", "does it differ by platform?".
- **Agents** (B9, AI on): the semantic tree is their observation format; that use must never distort what AT users get.
- **Izcal's engineers**: the tree must be deterministic (B1, DET-1), checkable against a from-scratch reference (B5), memory-bounded (FP), and its producer and its mirrors must survive a compromised renderer (I-1, IPC-R17).

## What Izcal's specs already decide or constrain

- **HGBA-2 2.3.2**: P1–P6; M-1 (no engine libraries, MIT only, spec anchors, no engine source as evidence); I-1 (process per (site, container), cross-site iframes out of process); IPC model.
- **HGBA-3**: B1, B2, B4 (restated by JS-D4), B5, B8 (semantic plane: stable IDs, provenance, visibility flags, snapshots and diffs, repair overlay), B9 (agent principal), B10 (repair classes R0–R4; "accessibility unaffected" by the AI switch; R0 OS preferences never normalized), A11Y-1 (repairs invisible to content), A11Y-1b, A11Y-2, A11Y-3, A11Y-4, Q-10 (disclosure of inferred names).
- **Stage 1 design**: G1 (`hg-semantic` reads `hg-tree-traits`), G4 (agent extras only with feature `ai`), G8 (`hg-semantic-repair` cannot write content state), G9 (anchors); 5.8 (semantic plane; AccessKit now void); 13.2 (screen-reader support in Stage 1 shell scope).
- **JavaScript engine spec**: JS-D4 (one-copy DOM; workers read node columns only during a flush with the script thread blocked), JS-D15 (timing-dependent decisions are recorded B1 inputs), JS-D11 (AccessKit is engine work).
- **Style spec**: the environment snapshot (one per document per frame, D-S1 quantization in one place), ST-D17 (accessibility overrides through the snapshot's quantized preferences), the computed values the semantic plane needs (display, visibility, `content`, `speak`).
- **Layout spec**: LY-16 (AT leaves no layout trace; Containment 2 §4.6), LY-D11 (Izcal scrollbar metrics, OS accessibility setting as default), G38 (consumers read `hg-fragment`'s API), fragment handles carry an epoch.
- **Fonts and text**: per-character bounds, word boundaries, glyph-to-character map through `hg-text`.
- **Graphics**: GR-D25 (forced colours and backplates in paint, flash analysis, reduce-motion interventions proposed to the shell).
- **Images and media**: MD-12 (accessible controls), 6.12 (OS caption preferences), MD-D19 (reduced motion for animated images), MD-D25 (generated captions on device, opt-in).
- **Footprint**: semantic tree is a class (a) store, built only when AT is active; FP-1 broker ≤ 20 MiB† idle; generational handles.
- **Security scope 0.3**: CON-R1 (broker accepts approval only for broker-observed OS input), CON-R3 (broker owns native windows and routes input), IPC-R1 to R17, IPC-R15 (no web-format parsing in the broker), the broker's CIG and extension-point blocking "to be measured at M5 against the IME and assistive-technology compatibility list".
- **DevTools**: DT-D6 (semantic history), DT-8 (DevTools accessible), DT-9, IIP.
- **Shell UI design**: UP4 (shell is a page), R5, R6, R20, 4.9 (keyboard model, F6 cycling), UI-1, UI-2.
- **G-rule range for this spec**: G100–G109. ID prefix: AX-, AX-D#, AX-M#.

## What success looks like (measurable)

Candidate target dimensions, to become AX- targets:
- **Correctness**: WPT `accname/`, `wai-aria/`, `html-aam/`, `core-aam/`, `svg-aam/`, `graphics-aam/`, `dpub-aam/`, `mathml-aam` pass rates against Izcal's floors; platform-tree tests per OS; ARIA-AT verdicts with NVDA, JAWS, VoiceOver and Orca at or above the best shipping engine.
- **Verification**: incremental tree equals from-scratch tree on every CI frame; broker mirror equals renderer tree after every update; adapters answer equal to a reference model.
- **Performance**: screen-reader query latency; time from DOM change to AT event; cost with AT off (zero) and on; large-tree budgets.
- **Memory**: bytes per semantic node in the renderer and in the broker mirror.
- **Privacy**: AT on and off indistinguishable to content (timing, events, layout work).
- **Security**: hostile tree updates cannot crash or confuse the broker; no cross-site data in another site's tree; no consent without a person.
- **Users**: screen-reader task success in the shell and on the corpus, measured with users.
