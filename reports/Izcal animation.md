# Izcal animation

*First-principles specification of Izcal's animation engine: the Web Animations model (timelines, animations, effects, timing, the effect stack), CSS Transitions and CSS Animations on that model with the style spec (before-change and after-change styles, `@starting-style`, per-property interpolation, additive and accumulate composition, the two animation cascade origins), scroll-driven animations and view timelines with the layout spec's scroll ownership (LY-D13), View Transitions (same-document and cross-document: capture, snapshots, the pseudo-element tree, compositing and the security of cross-document snapshots), compositor-driven animation and which properties can run off the main thread deterministically, `requestAnimationFrame` and frame timing, SMIL timing (with the SVG spec), the numeric contract of animation across x86-64, arm64 and riscv64, reduced motion and user controls, energy, recording and replay, and animation observability for DevTools, tests and agents. It is the animation owner the ledger's G-13 asks for, and it owns `hg-interp`. Date: 20 October 2026. Status: draft 0.2. Research notes: `research_notes/Izcal animation/` (`frame.md`, `history_postmortems.md`, `specs_semantics.md`, `compositor_frame_timing.md`, `determinism_math_replay.md`, `security_robustness.md`, `gaps_opportunities.md`, `motion_accessibility_energy.md`; phase 8: `factcheck.md`, `critique.md`). Relates to: HGBA-2 2.3.3 [M-1 (another project's source is never evidence), P1–P6, I-1, C-1, ANC-1][H2]; HGBA-3 [B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, DET-1, PRIV-2][H3]; Stage 1 design [Sections 2 (G1–G9), 3.3, 5.1, 8][S1]; JavaScript engine spec 0.4.1 [JS-D12, JS-D15, Sections 6.9, 6.12, `hg-math`][JS]; resource-footprint spec 0.4.1 [FP-1, FP-11, FP-12, FP-17, FP-18, FP-21, FP-22, FP-D4, FP-D15, Sections 6.4, 6.6, 6.8, 6.13, 6.14][FP]; HTML and CSS parsing and style spec 0.3.4 [ST-D7, ST-D18, Sections 6.9–6.12, 6.15, G19–G24][ST]; layout spec 0.4.1 [LY-D3, LY-D10, LY-D13, Sections 6.10, 6.19][LY]; images and media spec 0.4 [MD-D19, MD-D20, MD-D28][MD]; graphics and rendering spec 0.4.1 [GR-D5, GR-D13, GR-D15, GR-D18, GR-D23, GR-D25, G61, G63, GR-18, Sections 6.1, 6.3, 6.4, 6.8, 6.9, 6.21, 6.28][GR]; DOM, bindings and event loop spec draft 0.2.1 [DM-D1, DM-D10, DM-D18, Sections 6.7, 6.15, 6.18, G90–G99][DM]; accessibility spec draft 0.2.1 [Section 6.11, `hg-ax-prefs`, R0 exposure, AX-42][AX]; SVG spec draft 0.2 [SV-D4, SV-D16, Section 6.9, G120, G122, amendment R10][SV]; DevTools spec [T1, T3, T8, DT-D2, DT-D3][DT]; Phase 2 reconciliation 2 [Q15, Q20][RN2]; Phase 3 ledger [G-12, G-13, A-FUT-1, A-FUT-4][LG].*

*Revision 0.2 (20 October 2026): phase 8 applied, by a fact-checker and a design critic who did not write the spec. The [fact-check][FK] checked 97 claims (77 correct, 6 partly correct, 7 wrong, 1 unverifiable, 6 not checked) and 513 distinct URLs (none broken; two note fragments fixed) and listed 8 must-fix and 17 recommended items; all are applied. The [design critique][CR] raised 29 findings (7 High, 14 Medium, 8 Low), 6 engine-source and licence items and 18 reconciliation items; all are applied, three in a different form than proposed (Appendix A). Neither found a link to any project's source code; two WPT constants read from test files are removed. What changed: **Chrome's two composited-colour trials conflict**, so classes G′ and U are held because the evidence conflicts and no figure was published, built behind a flag by AN-M2 and shipped at AN-M3 if AN-S2 is positive (AN-D4); **`hg-interp` is split** so the compositor links no `hg-path`, motion-path animations stay on the main thread, and every hand-off record and each renderer key's compositor bytes are capped (6.5, G134); **one tick per frame, each document's frame time relative to its own time origin**, with `performance.now()` never below it (AN-D3); **transitions are started in a sequential post-pass after style's merge**, behind an `AnimationSource` trait (6.4); **time is i128 nanoseconds**, saturating, with no floating-point path (6.2); the easing bound grows with the curve's y range; the `then`-getter reordering is an Izcal choice, not "spec order" (6.3); invisible animations are skipped only when their **swept bounds** cannot reach the viewport, and skipped `content-visibility` subtrees follow Contain 2 (6.11); **SMIL follows the SVG spec's agreed split** (SV-D4, R10: attribute effects into an overlay, internal iteration accumulate, child timelines, image timelines) (6.8); **R0 holds in the Hardened profile**, and only the opt-in uniformity profile reports defaults (6.10); level X is a UA pause flag that sets no API override; cadence is decimated per renderer from its own votes (6.11); inactive scroll timelines follow the CSSWG's May and June 2026 resolutions (6.6); 18 amendments added to 9.5; engineer-years corrected to about 105–175†.*

*Revision 0.1 (20 October 2026): first draft, written by the synthesis author from seven research notes by independent researchers and a frame by the synthesis author.*

Animation is where three of Izcal's commitments meet at once: determinism (every animated value is a function of time, and time is an input), isolation (View Transitions copy pixels across a navigation, and the frame clock is the timer most rendering attacks use), and energy (the most common way a page wastes battery is motion nobody sees). The record says four things. **First, every engine ended up with two animation engines**: a complete one on the main thread and a partial one beside the compositor that silently falls back, with a different boundary in each browser ([HP 6.8][HP]); WebKit spent 2023 to 2026 replacing Core Animation with its own threaded sampler because an OS animator's feature set, not the web's, limited what could be accelerated ([CF 1.14–1.15][CF]), and Firefox had to make its compositor sample at the main thread's frame time to stop a one-vsync skew ([CF 3.6][CF]). **Second, animation is an attack surface out of proportion to its size**: three in-the-wild exploits in eight years (one against Tor users, one by two state groups, one zero-click) were use-after-frees in the animation graph, the latest triggered by script running from a promise settlement in the middle of a timeline tick ([SR 3.1–3.3, 2.1][SR]); layout had none in the same record. **Third, the specifications leave the decisive numbers open**: the timing of style change events, the cubic-Bézier solver, time precision, the "ready" moment, snapshot limits and timeouts are all implementation-defined ([SP 14.4][SP]), and two easing levels disagree on `steps()` ([SP 5.5][SP]). **Fourth, users have no control over author motion**: shipped browser controls stop at animated images, although UAAG 2.0, a W3C Note, asks for a pause of any animation lasting three seconds or more ([MA 3.6–3.7, 4.7][MA]). **The central idea: one animation model, one numeric contract and one tick per frame, evaluated identically wherever it runs.** Every animation kind (CSS transitions, CSS animations, script animations, SMIL, scroll and view timelines, View Transitions, animated-image frame timing) is a generator of Web Animations objects in one core; the model is a pure function of recorded inputs (frame times, scroll offsets, preferences, script calls); time is exact integer nanoseconds and progress is exact fixed point, so boundaries are decided by mathematics, not rounding; and the main thread, the content GPU process and the compositor all evaluate the same `hg-interp` functions at the same frame time, so what is drawn equals what script reads for the same inputs, bit for bit, on all three ISAs. The decisions that matter most: (1) one timing core for every front end (AN-D1); (2) the exact numeric contract (AN-D2); (3) one recorded tick per frame, from which every document's unjittered frame time and every off-main-thread sample derive (AN-D3); (4) the off-main-thread classes derived from first principles, with colour and other uniform paint properties held for measurement because Chrome's two field trials conflict and published no figure (AN-D4); (5) View Transition snapshots that no renderer can read and whose cross-document hand-off is decided in the broker (AN-D7); (6) graded user motion controls that never remove an animation (AN-D10); (7) "invisible animation costs zero" as an energy rule with tests (AN-D11). The work is about **29–39 engineers† for about 4–5 years† to Stage 1 exit**, about 105–175 engineer-years† (Section 9).

Note sources are cited as **HP** ([history and postmortems][HP]), **SP** ([specifications and semantics][SP]), **CF** ([compositor and frame timing][CF]), **DR** ([determinism, maths and replay][DR]), **SR** ([security and robustness][SR]), **GO** ([gaps and opportunities][GO]) and **MA** ([motion, accessibility and energy][MA]), with the row number; the frame is [FR][FR]. Primary sources are linked inline for the claims a decision or target depends on. Phase 8 is cited as **FK** ([fact-check][FK]) and **CR** ([critique][CR]).

## 1. The problem and who has it

Izcal has no animation engine and no owner for one. The ledger lists it as a component with no spec, blocking GR-M1 ([G-13][LG]); the DOM spec fires transition and animation events from a placeholder module, `hg-anim-events`, until this spec exists ([DM 6.7][DM]); the graphics spec writes `hg-interp` on this spec's behalf ([GR 6.24][GR]); the style spec leaves "scroll-driven timelines" and "compositor-run animations" for this spec to define ([ST 6.10][ST]); the layout spec hands it scroll ranges, view-transition geometry and `interpolate-size` ([LY 6.19][LY]). HGBA-2 2.3.0 excludes every library that does this work, so Izcal builds its own, from the specifications.

| Who | What they need | What fails today (evidence) |
|---|---|---|
| **People browsing** | Motion that starts promptly, stays smooth when the page is busy, stops when they ask, and costs nothing when they cannot see it | No browser lets a user pause author animation (UAAG 2.10.5 asks for it) ([MA 3.6–3.7][MA]); a hidden CSS spinner once cost 60% CPU in Firefox until animations on invisible elements stopped updating ([MA 7.4][MA]); CSS and SVG animations drew 100–500 mW against about 25 mW for an idle browser on one M1 laptop, 4–20× idle ([MA 7.1][MA]; [FK][FK]) |
| **Motion-sensitive and photosensitive users** | Reduced or stopped motion without breaking pages; protection from flashing | Half of sites contain `prefers-reduced-motion`, and nobody checks what it does ([MA 2.11–2.12][MA]); flashing content has been used as a weapon against people with epilepsy ([SR 8.5–8.6][SR]); no browser detects flashing in page animation ([SR 8.9][SR]) |
| **Web developers** | "Why didn't my transition start?", "which animation wins?", "why is this not off the main thread?", "why does it differ in another browser?" | The two most-viewed animation questions on Stack Overflow (about 4.6 million views) ask why `display` and `height: auto` do not transition ([GO 1.11][GO]); "Animations" is the top Interactions pain point in State of CSS 2026 ([GO 1.1][GO]); scroll-driven animations and View Transitions are the second and third most-named incompatibilities in 2025 ([GO 1.2][GO]); no tool shows why a transition did not start or which effect won ([GO 2.16][GO]) |
| **Test and automation authors** | Screenshots of animated states that are identical every run | Animation timing caused 26 of 235 flaky UI tests in one study ([DR 9.3][DR]); tools turn animation off rather than control it ([GO 3.1–3.5][GO]); WebDriver BiDi has no animation command ([GO 3.6][GO]) |
| **Agents** (only with AI on, B9, B10) | "Has the page settled?" | Playwright's "stable" is two frames with the same bounding box, blind to opacity and colour ([GO 3.3][GO]) |
| **Izcal's engineers** | A model that replays exactly (B1, DET-1), is identical on three ISAs (JS-D12), cannot be re-entered by script mid-update, fits FP-11, and has one interpolation for the main thread and the compositor (GR-D13) | Engines' compositor and main-thread samplers have diverged by a frame ([CF 3.6][CF]); floating-point step boundaries were decided by rounding in 6–14% of exact-boundary cases in one experiment, depending on the durations ([DR 4.8][DR]; [FK][FK]) |

**Success, measurably** (Section 8): WPT floors on every animation directory at or above the best shipping engine; animation outputs bit-identical across 1,000 of 1,000 replays and across x86-64, arm64 and riscv64; compositor-run animations at full frame rate with the main thread blocked; zero frames and zero wakeups for animations nobody can see; every running animation explainable (why it started or did not, which effect won, where it ran); a published motion-control breakage rate.

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1996–2011 | **DHTML timers; jQuery `.animate()`** | Script moved elements from `setTimeout`; jQuery stepped every animation from one 13 ms timer | No frame callback, no shared clock; 60 Hz displays | O'Callahan: "there is no 'right' timeout value", and timers keep sampling when nothing is visible ([HP 1.4][HP]); jQuery adopted rAF in 1.6 and removed it in 1.6.3 because animations queued in hidden tabs played all at once on return ([HP 1.3][HP]) |
| 1998–2001 | **SMIL 1.0, HTML+TIME, SMIL Animation** | Timed presentations; then a subset for XML hosts with the **sandwich model**: animation writes a presentation value over an unchanged base value | Multimedia players; XML everywhere | The sandwich model survived into Web Animations ([HP 2.3][HP]); HTML+TIME died with IE's behaviors ([HP 2.2][HP]) |
| 2001–2015 | **SVG animation (SMIL)** | `animate`, `set`, `animateMotion`, `animateTransform`: any attribute, including path data, inside images where script cannot run | Declarative graphics for a script-less world | IE never shipped it ("Never", 2015) ([HP 2.5][HP]); Blink's 2015 intent to deprecate cited 0.0403% usage and complexity; the SVG WG listed what nothing else could do (path morphing, chaining, event-driven begin, animation inside images) and the deprecation was suspended in 2016 ([HP 2.6–2.10][HP]); usage is now 2.90% of Chrome page loads, with a counting change in between ([HP 2.12][HP]) |
| 1996–2020 | **Flash** | Vector animation, timeline authoring, scripting, video in a plug-in | Browsers had none of these | Battery, touch and security decided its fate on mobile; Adobe ended it at the end of 2020 ([HP 3.2–3.3][HP]) |
| 2007–2009 | **WebKit CSS Transitions, Animations, Transforms** | "Implicit animations" when a computed value changes; keyframes; 2D and 3D transforms | Apple's iPhone work, where Core Animation animated layers' transform and opacity on the GPU ([HP 4.1–4.2][HP]) | Standardized; CSS Transitions 1 is still a Working Draft in 2026 ([HP 4.4][HP]); an element with no previous style could not transition in until 2023 ([HP 4.5][HP]) |
| 2010–2011 | **`requestAnimationFrame`** | The browser chooses the frame rate; all animations in a frame share one start time; hidden pages stop | Timers could not know the frame rate or visibility ([HP 5.1–5.2][HP]) | On 65.1% of Chrome page loads ([HP 5.3][HP]); HTML still leaves cadence and throttling to each browser ([HP 5.4][HP]) |
| 2012–2020 | **Web Animations** | One timing model for CSS, SVG and script, designed so CSS could be defined on it "without any observable change" ([HP 7.1][HP]) | Three incompatible models | `Element.animate` shipped 2014 (Chrome) to 2020 (Safari); WebKit needed "well over 2 years" to rebuild on a shared engine ([HP 7.4–7.5][HP]); groups and sequences were cut in 2015 "in the interests of iterating more quickly" and are unshipped eleven years later ([HP 7.2–7.3][HP]) |
| 2012–2026 | **Compositor animation** in each engine | Transform and opacity (Chrome also filter) sampled beside the compositor | The compositor held layers, not boxes; only a layer's matrix, opacity or filter could change without paint ([HP 6.2–6.4][HP]) | Firefox 41 (2015) with exceptions for geometry animations, because desync "is much more noticeable" for geometry ([CF 1.10–1.11][CF]); Chrome's two 2021 field trials of composited `background-color` conflict: the first saw **no improvement in dropped frames** and a memory regression, the second (1% of stable users) "very good results on throughput metrics" with about 1.5% more memory on Windows and no published figure ([HP 6.6][HP]; [CF 1.7][CF]); composited `clip-path` still unconfirmed as shipped five years on ([CF 1.8][CF]); WebKit's own threaded sampler replaced Core Animation, enabled by default in WebKit's main branch on 2026-02-13 ([CF 1.15][CF]) |
| 2016–2024 | **Animation Worklet** | Script producing an effect's local time off the main thread | Scroll-linked and stateful effects without main-thread jank | Never left origin trial; its use counter reads 0; no published post-mortem ([HP 8.3][HP]; [CF 6.4][CF]); its scroll use case went to declarative scroll timelines |
| 2019–2024 | **`@property`** | Typed custom properties that interpolate | Custom properties flipped discretely | On 19.4% of Chrome page loads; in all three engines by 2024 ([HP 8.4–8.5][HP]) |
| 2020–2025 | **Scroll-driven animations** | Scroll and view progress as animation timelines | Scroll handlers on the main thread lagged the compositor's scroll ([HP 9.1][HP]) | An at-rule design was replaced by properties in 2022 ([HP 9.2][HP]); Chrome 115 (2023), Safari 26 (2025), Firefox in preview; Interop 2026 ([HP 9.4, 9.6][HP]); WebKit's threaded version once rendered continuously on idle pages ([CF 5.4][CF]) |
| 2021–2026 | **View Transitions** | Capture the old state as images, suppress rendering, update the DOM once, animate a pseudo-element tree | SPA and MPA transitions needed both states at once ([HP 10.1][HP]) | Same-document in all three engines (Firefox 144, October 2025), cross-document in Chrome and Safari ([HP 10.4][HP]); the declarative opt-in is on 12.3% of Chrome page loads ([HP 10.6][HP]); two Chrome CVEs in 2026, one a cross-origin leak ([SR 1.5][SR]) |
| 2023–2024 | **Entry and exit animation**: `allow-discrete`, `@starting-style`, `overlay`, `display` in keyframes; **`linear()`** | Closed the 2007 model's hole for elements with no previous style | Patch on the implicit-transition model ([HP 11.1–11.3][HP]) | In all three engines; Interop 2024 ([HP 11.2][HP]) |
| 2017–2026 | **JS libraries** (GSAP, Motion), **Lottie**, **Rive** | Sequencing, springs, layout animation, morphing; designer-authored formats | Platform gaps and per-engine compositor differences | Motion: 4.5 million weekly downloads ([HP 13.4][HP]); Motion's engine uses Web Animations for compositor-capable values and JS for the rest ([HP 13.5][HP]) |

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption | Where it came from | Why it no longer holds |
|---|---|---|
| A fixed 60 Hz frame | Timers of 13 ms and 16.7 ms | Displays run at 90–144 Hz or vary (ProMotion 10–120 Hz); HTML uses 60 Hz only as an example ([HP 14.1][HP]; [CF 4.7][CF]); Android's adaptive refresh rate lets each view vote, Normal ≈ 60 Hz by default, because maximum rate for small animations "results in high power consumption" ([CF 4.12][CF]) |
| "Only transform and opacity are cheap" | The 2008–2014 layer compositor, where only a layer's matrix and opacity could change without repaint ([HP 6.1, 6.4][HP]) | Raster from display lists on the GPU changes the cost of re-rastering small areas; the line is a measured trade-off, not a law ([HP 14.4][HP]) |
| One main thread | CSS and SMIL specified as if style, layout and sampling run on one thread | Every engine moved a subset off-thread and created two engines with silent fallbacks ([HP 14.3][HP]) |
| Time is the only clock | Wall-clock animation | Scroll position, view position and navigation now drive animations ([HP 14.5][HP]) |
| Frames always arrive | Timer animation | Hidden-tab throttling and battery modes make frame delivery a policy ([HP 14.6][HP]; [MA 8.6–8.9][MA]) |
| An OS animator can do the work | Core Animation on iOS | Its feature set capped `steps()`, `linear()`, additive composition, motion path and `playbackRate ≠ 1` ([CF 1.14][CF]) |
| Floating-point time is good enough | `double` milliseconds in every API | Deterministic across ISAs for basic operations, but boundaries become rounding artefacts, and f32 time drifts by seconds per hour ([DR 4.7–4.8][DR]) |
| Coarsening `performance.now()` defends timing attacks | Spectre-era mitigations | Attackers count frames; the frame boundary is itself a clock ([SR 4.8][SR]) |

## 3. Today: the landscape and its gaps

**Specifications.** Nothing in the stack is a Recommendation except SMIL Animation (2001); Web Animations 1 has not been snapshotted since June 2023 while its Editor's Draft changed normative behaviour ([SP 1.1][SP]); Animation Triggers 1 had its first public draft on 20 October 2026 ([SP 1.6][SP]). The open choices an implementation must make, and record, are: when a style change event happens ([SP 6.2][SP]), the "ready" moment ([SP 2.7][SP]), the cubic-Bézier solver and precision ([SP 5.2][SP]), time precision (microseconds recommended, [SP 14.1][SP]), the `steps()` algorithm where Easing 1 and Easing 2 disagree ([SP 5.5][SP]), stale scroll timelines, for which HTML has no step any more ([SP 13.1][SP]), hidden-document timelines, which neither spec states ([SP 13.9][SP]), view-transition limits and timeouts ([SP 14.4][SP]), and what reduced motion does ([SP 14.5][SP]).

**Engines.** Off-main-thread sets differ per engine and fall back silently ([HP 6.8][HP]); 40% of mobile pages run at least one non-composited animation ([GO 1.6][GO]). Throttling margins for offscreen frames differ between engines and between in-process and out-of-process frames ([CF 7.5][CF]). Battery modes reduce frame rates with unpublished rules ([MA 8.9–8.10][MA]).

**Tools.** Chrome's Animations panel captures only while open and does not show `requestAnimationFrame` animation; Firefox alone explains why a property is not on the compositor; Safari's timeline misses script-created animations; the CDP Animation domain is experimental and has no field for composite order, effect stack or start cause ([GO 2.1–2.14][GO]).

**What no current tool or engine shows** ([GO 2.16][GO]; [GO 5][GO]):

| Invisible today | What Izcal can show, and from what |
|---|---|
| Why a transition did **not** start | The style change event's before-change and after-change values, the transitionable check and the spec step (B4, ANC-1) |
| Which effect won a property | The effect stack in composite order with each effect's composite operation, fill and replace state |
| Where an animation ran and why | The off-main-thread eligibility decision and the rule that kept it on the main thread |
| Animations from before the panel opened | Recorded sessions (DT-D2): every animation since navigation |
| Exact animated states for tests | Seek every timeline to *t* under the virtual clock and get identical pixels on every OS and ISA |
| Energy per animation | Frames requested, wakeups, raster time per animation, and whether anyone could see it |
| What reduced motion actually changes | A replay with the preference on and off, diffed as values |
| Whether the page has settled, animation-wise | A settled signal defined from the model, not from bounding boxes |

## 4. First principles

### 4.1 Users' real tasks

| ID | Task | Who |
|---|---|---|
| U1 | See motion that is smooth while the page works, and that starts when it should | Everyone |
| U2 | Make motion stop or calm down, on every site or one, without breaking it | Motion-sensitive users, people who are distracted by motion |
| U3 | Not pay battery for motion they cannot see | Everyone on battery |
| U4 | Navigate with continuity (View Transitions) without leaking anything across origins | Everyone |
| D1 | Make an element animate as designed in every engine, and find out why it does not | Web developers |
| D2 | Know what runs where and what it costs | Web developers |
| D3 | Test animated UI deterministically | Test authors |
| A1 | Know when the page has settled | Agents, with AI on |

### 4.2 Platform primitives

The specifications define the model exactly and leave the numbers open. **Web Animations 1** defines a small closed state machine per animation (start time, hold time, pending play and pause tasks, pending playback rate, previous current time) and the six-step "update animations and send events" ([SP 2.4–2.10][SP]); the effect value pipeline (local time → active time → overall progress → simple iteration progress → current iteration → directed progress → transformed progress) ([SP 3.3][SP]); the effect stack folding from the base value with replace, add and accumulate ([SP 3.7][SP]); and the result entering the cascade at the Animation or Transition origin ([SP 3.8][SP]). **CSS Cascade 5** places transitions above every `!important` declaration and animations above every normal one ([SP 7.1][SP]). **CSS Transitions 1 and 2** define before-change and after-change styles and the start, reverse and completion rules ([SP 6.3–6.11][SP]). **CSS Values 4, Transforms 1 and 2 and Color 4** fix the per-type interpolation arithmetic: integer rounding, log-space ratios, quaternion slerp after decomposition, premultiplied Oklab by default but gamma-encoded sRGB for legacy colours ([SP 8.2–8.6][SP]). **HTML** fixes the order of every step of the rendering update and passes one `frameTimestamp` to the timelines and to rAF ([SP 11.1–11.4][SP]; [CF 4.1][CF]). **Scroll-driven Animations 1** samples scroll timelines once per frame and allows one extra style and layout pass for stale timelines ([SP 9.5][SP]). **View Transitions 1 and 2** hook into HTML's "perform pending transition operations" and "reveal" ([SP 10.2, 10.9][SP]). **SVG 2** makes animation optional and defers to SVG Animations 2, which defers to SMIL Animation (2001) ([SP 12.1–12.2][SP]).

### 4.3 Izcal's principles that bind this design

P1 (security and privacy: no script inside the model's walk; snapshots never readable; the frame clock never depends on another origin's secret-dependent work), P2 (compatibility outranks speed: off-main-thread paths must change no observable result), P3 (every accelerated path has a main-thread fallback with identical results), P4 (energy: invisible animation costs zero), P6 (evidence: off-main-thread classes added only when measured). B1 and JS-D15 (every timing-dependent decision is a recorded input), DET-1, JS-D12 (cross-ISA identity), B4 (the mutation log explains causes), B7 (pixel-optional rendering still runs the model), B9 and B10 (agents only with AI on), M-1 (no code and no constants from any other project).

### 4.4 Assumptions kept and broken

| Assumption | Keep or break | Why |
|---|---|---|
| CSS transitions, CSS animations, SMIL and script animations are separate subsystems | **Break** | WebKit paid two years to unify late ([HP 7.5][HP]); one core is cheaper from the start (AN-D1) |
| The compositor runs a second, partial animation engine | **Break** | One sampler, one function, one time, every thread ([HP 6.8][HP]; [CF 1.15][CF]) (AN-D4) |
| Time is `double` milliseconds internally | **Break** inside, **keep** at the API | Exact integer time decides boundaries by mathematics ([DR 4.8][DR]); script still sees doubles (AN-D2) |
| "Only transform and opacity can be cheap" | **Break as a rule; keep as the starting set** | Derive the classes; widen with measurement, because Chrome's two trials of composited colour conflict and published no figure ([HP 6.6][HP]; [CF 1.7][CF]) (AN-D4) |
| The compositor resolves the start time | **Break** | The renderer resolves "ready" from recorded frame times, so no compositor timing enters the model (AN-D5) |
| Frame timestamps are jittered like `performance.now()` | **Break** | Jitter per realm would desynchronize documents of one event loop and the compositor; frame times are already frame-granular (AN-D3) |
| Refresh rate is always the display's maximum | **Break** | Per-animation votes, Normal ≈ 60 Hz unless motion needs more ([CF 4.12][CF]) (AN-D11) |
| Offscreen and invisible animations are sampled | **Break** | Skip them when their swept bounds cannot become visible and skipping cannot change a script-observable value ([MA 7.4][MA]); skipped `content-visibility` subtrees follow Contain 2 ([MA 8.5][MA]) (AN-D11) |
| Reduced motion is the author's job alone | **Break** | UAAG 2.0 (a W3C Note) asks the UA for it; no browser does it ([MA 3.6–3.7][MA]) (AN-D10) |
| A UA intervention may remove animations | **Break** | Removal fires no end event and can leave content invisible ([MA 6.3–6.4][MA]) (AN-D10) |
| Script may run whenever a promise settles | **Break inside the walk** | CVE-2024-9680 ([SR 2.1–2.2][SR]); in the spec's order a `then` getter can run inside the timeline update; Izcal settles every promise after the walk, an Izcal choice the open ready moment permits (G130, 6.3) |
| Implementation-defined limits are secret | **Break** | Every limit is a published constant with a deterministic fallback ([SR 7.7][SR]) |
| Tools capture animation live, from when they open | **Break** | Recorded sessions and replay (DT-D2) |
| SMIL can be dropped | **Keep it** | Deprecation failed once; 2.90% of Chrome page loads ([HP 2.10–2.12][HP]; [SV 6.9][SV]) |
| Groups and sequences can wait | **Keep the wait, design for them** | Unshipped anywhere; the timing tree holds them so Level 2 is not a rewrite ([HP 7.2–7.3][HP]) (AN-D9) |

## 5. Lessons learned

Lessons are ideas and evidence, never code or a template to reproduce (M-1).

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| jQuery 1.6 → 1.6.3 | rAF removed because hidden-tab animations queued and fired together ([HP 1.3][HP]) | Libraries depend on exact hidden-page semantics | Hidden documents get no frames, and timelines then jump forward exactly as HTML implies (6.11, [SP 13.9][SP]) |
| Blink's SMIL deprecation | Reversed because replacements could not express path morphing, chaining or animation inside images ([HP 2.8–2.10][HP]) | Removal fails without a full replacement | SMIL on the same core (AN-D6) |
| WebKit's 2-year rebuild | Unified engine for CSS and the API, late ([HP 7.5][HP]) | Unify from day one | AN-D1 |
| Web Animations groups | Cut in 2015 for speed; still unshipped ([HP 7.2–7.3][HP]) | Scope cuts stay cut | Timing tree designed for groups; feature held by admission (AN-D9) |
| Firefox OMTA | One vsync of skew between threads; fixed by sampling at the main thread's frame time ([CF 3.6][CF]) | One time per frame, written down | AN-D3 |
| Firefox geometry rule | Transform on the compositor drifted from size changes on the main thread ([CF 1.10–1.11][CF]) | Animations of one element run on one side | Group rule (6.5) |
| Chrome composited `background-color` | Eligibility checked twice and disagreed; a missed input crashed; system colours fell back; one trial found no dropped-frame gain and a memory regression, a later one "very good" throughput and +1.5% memory ([CF 1.6–1.7][CF]; [HP 6.6][HP]) | Decide eligibility once from one snapshot; measure before widening | Hand-off snapshot rule; colour class built behind a flag and decided by AN-S2 (AN-D4) |
| WebKit and Core Animation | An OS animator's feature set capped acceleration; three years to replace ([CF 1.14–1.15][CF]) | Own the threaded sampler | `hg-interp` in the compositor, every easing and composite mode (AN-D4) |
| WebKit threaded scroll-driven animations | Continuous rendering on idle pages (fixed 2025) ([CF 5.4][CF]) | Idle must be tested | AN-30: zero frames on an idle page with a scroll-driven animation |
| Animation Worklet | Never shipped; zero use ([HP 8.3][HP]) | Keep the pure "input → local time" contract; do not run author script off-thread | Sampler is a pure function; no worklet (AN-D9) |
| Firefox hidden spinner | 60% CPU until invisible animations stopped updating ([MA 7.4][MA]) | Invisible animation must cost nothing | AN-D11, AN-30 |
| CVE-2024-9680 and kin | Promise `then` getter cancelled an animation during the timeline tick ([SR 2.1][SR]); 35 of the 43 SMIL, CSS and Web Animations and View Transitions CVEs were lifetime bugs ([SR 2.8][SR]) | No script during the walk; arena with generational handles | G130, G131 |
| CVE-2019-9796 | Double registration with the refresh driver ([SR 2.5][SR]) | Registration is a set | Frame-demand registry keyed by owner (6.11) |
| Firefox's reduced-motion spoof | The media query was hidden but smooth-scroll behaviour leaked the bit ([SR 6.4][SR]) | Every behaviour must read the one exposed value | AN-D10's single exposed value |
| CSSWG #5321 and #7440 | "No motion" mode should make motion discrete; forced reduction can make content invisible ([MA 5.2–5.5][MA]) | Transform timing, keep events and end states | AN-D10 |
| Box2D v3 | Bit-identical across ISAs by banning contraction, own transcendentals, hash in CI ([DR 7.4][DR]) | The recipe works | G133, AN-20 |
| Playwright and Chromatic | Turn animation off for screenshots ([DR 9.1–9.2][DR]) | Tools lack exact time | Seek-to-*t* and identical pixels (6.13) |

## 6. Design

### 6.0 Architecture at a glance

```
 RENDERER per renderer key (site, container; origin under FP-D8)
  hg-script::rendering (DM 6.7) -- one BeginFrame tick T (recorded, GR-D15)
     each document: F = quantize(T - its time origin) on its own grid, no jitter (6.3)
    step "update animations and send events" --> hg-anim::update(doc, F)        [no script runs]
       timelines: document = F | scroll (offsets after the frame's scroll step, LY-D13)
                  | view (subject geometry from the last layout) | SMIL fragment timelines (6.8)
       animations: state machine, pending tasks released by the ready rule (AN-D5)
       effect stacks per (element, pseudo, property or SVG attribute); replaced-animation removal
       output: a CHANGE LIST (promise settlements, events with scheduled times)
    hg-script: re-validate, settle promises -> microtask checkpoint -> sort -> dispatch
    style pass (ST, parallel): base style; animation and transition origin blocks read from an
       immutable effect-stack snapshot through the AnimationSource trait; before-change and
       after-change pairs written as transition candidates (6.4)
    sequential post-pass after style's merge (hg-anim::css): start, cancel, reverse, complete
       transitions in tree order; generations; arena allocation
    layout (LY): scroll ranges, view-timeline subjects; stale timelines -> one re-run (AN-D8)
    "perform pending transition operations" -> hg-view-transition (capture, pseudo-tree rules)
    paint (GR) -> hand-off records (6.5): class C to the compositor, class G to the content GPU
       process; each bounded (6.5 rule 6)
    frame demand (6.11): next visible change; swept-bounds visibility; votes; wake-ups that
       request a rendering opportunity
  hg-anim-api: Animation, KeyframeEffect, DocumentTimeline, ScrollTimeline, ViewTimeline,
    ViewTransition, CSSAnimation, CSSTransition, getAnimations(), SmilControl (behind DM's bindings)
  hg-anim-observe: decision records -> IIP (DT)
        | class G records                              | class C records
        v                                              v
 CONTENT GPU PROCESS per key (GR 6.1)           COMPOSITOR GPU SERVICE (GR 6.1)
  hg-anim-time, hg-interp, hg-interp-geom        hg-anim-time and hg-interp only (no hg-path,
  at the frame's tick; filter amounts and         G63); transform family and opacity; scroll-
  clip-path shapes; re-raster damaged tiles;      driven ones from the offset it draws;
  snapshot surfaces (6.7)                         snapshots with cross-origin surfaces (GR 6.28)
 BROKER: hg-navigation decides cross-document View Transition eligibility and moves snapshot
  capabilities between documents (G136)
 MEDIA PROCESS (MD-D20): hg-anim's scriptless core for SMIL in SVG images, ticks from the embedder
```

**One model, one numeric contract, one tick per frame, evaluated identically in three places.**

### 6.1 One model, many front ends (AN-D1)

`hg-anim` implements Web Animations 1's model once: timelines, animations, the playback state machine, animation effects with their timing, keyframe effects, the effect stack and composite order, replaced-animation removal, events and promises, anchored step by step to the Editor's Draft revision pinned at AN-M0 ([SP 1.1][SP]). Every animation kind is a **front end that creates Web Animations objects**:

| Front end | Creates | Composite-order class | Owner of the front end |
|---|---|---|---|
| CSS Transitions 1 and 2 | `CSSTransition` per (element, pseudo, longhand) | Transitions (lowest), by tree order, generation, property name ([SP 6.11][SP]) | This spec, with style's before-change and after-change styles |
| CSS Animations 1 and 2 | `CSSAnimation` per `animation-name` entry | CSS animations, by tree order then list position ([SP 7.7][SP]) | This spec, with style's keyframe cascade |
| `Element.animate()`, `new Animation()` | `Animation` + `KeyframeEffect` | Script animations, by global animation list position | This spec |
| SMIL (`animate`, `set`, `animateMotion`, `animateTransform`) | One effect per animation element's active interval, on its fragment's timeline | A SMIL class whose position is measured black-box first, then decided (6.8); within it, SMIL's priority (interval begin time, then document order) | This spec (timing, AN-D6); SVG spec (elements, attributes, geometry, value interpolation, SV-D4) |
| View Transitions | CSS animations on the `::view-transition-*` pseudo-elements, from UA-origin dynamic rules | CSS animations | This spec, with graphics |
| Scroll and view timelines | Timelines other front ends attach to | — | This spec, with layout |
| Animated images (MD-D19) | No effect; frame selection reads the document timeline | — | Media spec |

**Effect targets** are a CSS property (longhand or registered custom property) or, for SMIL, an **SVG attribute** that is not a property (`viewBox`, `points`, gradient and filter attributes): attribute effects interpolate through the SVG spec's `InterpolateValue` implementations (`hg-svg-interp`) and write the **animated-attribute overlay**, never the DOM (6.8; [SV 6.9][SV]).

**Groups are designed for, not shipped.** The timing tree carries a parent link and a local-time transform per node from AN-M0, so Web Animations 2's `GroupEffect`, `SequenceEffect` and effect playback rate are an admission decision, not a rewrite ([HP 7.2–7.3][HP]; AN-D9). Iteration composite `accumulate` is implemented internally because SMIL's `accumulate="sum"` needs it ([SV 6.9][SV]); its script API waits for admission. Animation Worklet is not implemented: no engine ships it and its use counter reads 0 ([HP 8.3][HP]); its lasting idea, a pure "input → local time" function, is how `hg-anim-time` is written.

**Model liveness.** Every API call takes effect immediately in the model, as Web Animations requires ([SP 3.12][SP]); `getComputedStyle` and `getAnimations()` sample the model at the document timeline's current time, which does not change within a task ([SP 2.3][SP]); `getAnimations()` triggers a style change event first, so pending CSS animations and transitions exist before it returns ([SP 3.11][SP]).

### 6.2 Numbers: time, progress, easing and interpolation (AN-D2)

Web Animations leaves the internal representation of time to the implementation and recommends microsecond precision ([DR 4.1][DR]); CSS Easing leaves the cubic-Bézier solver and its precision open ([DR 3.1][DR]). The basic IEEE operations are correctly rounded and bit-identical on x86-64, arm64 and riscv64 when nothing is contracted into FMA and flush-to-zero is off; NaN bits and every libm function are not ([DR 1.1–1.11, 2.1][DR]). Within those facts Izcal chooses an exact contract, so that every boundary is decided by mathematics; conformance tests that tolerate small differences are run black-box and pass either way.

**Time.** `AnimTime` (in `hg-types`, the integer time quantum the SVG spec also uses, [SV 6.18][SV]) is **i128 nanoseconds, saturating at ±2¹²⁶ ns**, with unresolved, +∞ and −∞ as explicit variants. Every finite double a script can usefully pass fits: a double converts by **one round-half-even** of its exact value × 10⁶, computed from its mantissa and exponent with a 256-bit intermediate; magnitudes beyond 2¹²⁶ ns (about 1.7 × 10³² ms) saturate, a recorded deviation no session can reach. A read converts the integer to the correctly rounded double, except that a **start time or hold time set directly by script is read back as the double script wrote** (the model keeps it beside the integer), so `a.startTime = x; a.startTime === x` holds for every x; other reads are within 0.5 ns of the real-number value. Nothing is accumulated across frames: an animation's current time is always `(timeline time − start time) × playback rate`, the product of an i128 and a double formed **exactly in 256 bits and rounded once** (half to even), saturating; a seek with rate r sets `start = timeline − x / r` with the exact quotient rounded half to even. Durations below 0.5 ns round to 0 and take the zero-duration rules, which the spec's precision clause permits ([DR 4.1][DR]). There is no floating-point path in `hg-anim-time`, so it is integer-only under G61 except its boundary conversions.

**Progress.** Overall progress, the current iteration and simple iteration progress come from exact integer arithmetic: `iterationStart` converts once to Q32 (2⁻³² units), the quotient `active time / iteration duration` is taken exactly, and the **boundary tests are exact** (the remainder is zero or it is not), so "an interval ending exactly on an iteration end yields 1.0" and every phase boundary follow the real-number reading of the spec ([SP 3.2–3.3][SP]). A double carried through the same steps put 6–14% of exact step boundaries in the wrong step in one experiment, depending on the durations tried ([DR 4.8][DR]; [FK][FK]). The current iteration is a saturating u64. Progress handed to easing is **Q32, floored**, so a progress strictly below 1 inside an iteration stays below 1, with the before flag.

**Easing.** `cubic-bezier()` is solved by **integer bisection** on t with the control points quantized to Q32 and x(t) evaluated exactly with a 256-bit helper; x is monotone because both x control values are in [0, 1], so bisection always converges ([DR 3.2, 5.4–5.5][DR]). The y control values are unbounded in CSS, so the step count grows with the curve: k = 24 + ⌈log₂ max(1, |y₁|, |y₂ − y₁|, |1 − y₂|)⌉ steps, at most 64, fix t to 2⁻ᵏ, and y is evaluated exactly at that t and rounded once to Q32. The output error is then at most 3 · 2⁻²⁴ + 2⁻³³ for every curve, below 2⁻²¹† when both y control values are in [0, 1] (under 0.0005 px on a 1,000 px move), and independent of the animation's duration ([FK][FK]). Newton iteration and closed-form cubics are not used (no epsilon, no transcendental, [DR 5.6][DR]). `linear()` is exact piecewise-linear arithmetic on Q32 stops, and `steps()` is exact on the Q32 input. **Easing 1 and Easing 2 disagree on `steps()`** with the before flag and out-of-range inputs ([SP 5.5][SP]): Izcal anchors its algorithm to one level's text, records which, checks the WPT result black-box (pass or fail), and files the difference (Q-AN-3). Easing output is a signed Q32.32 (i64), saturating at ±2³¹ as a recorded bound.

**Interpolation, in two numeric domains, each stated once per value type:**

| Domain | Value types | Arithmetic | Why |
|---|---|---|---|
| **Fixed point (`hg-interp`, and `hg-interp-geom` for shapes)** | Transform lists, matrices and the individual `translate`, `rotate`, `scale` properties; `opacity`; filter-function amounts; basic-shape and `path()` geometry for `clip-path` (`hg-interp-geom`) | Q32.32 in i64 with i128 intermediates, the matrix representation the graphics spec already uses for paint; the Transforms 1 and 2 decomposition, slerp and recomposition algorithms as written, with exact integer square root and **correctly rounded fixed-point sine, cosine and arccosine of degrees**; the spec's exact-equality tests (`determinant == 0`, `abs(product) == 1`) become exact integer tests ([DR 6.4–6.5][DR]) | These values reach composition; the compositor and the content GPU process must produce exactly what the main thread computes, and G61 makes `hg-interp` integer. Degree-based trigonometry gives exact zeros at multiples of 90°, where `sin(deg2rad(180))` does not ([DR 3.6][DR]; [DR 3][DR]) |
| **IEEE f64 (`hg-anim-values`)** | Every other computed-value type: lengths, percentages and `calc()` sums, numbers and integers (rounded per spec), ratios (log space), colours (premultiplied, in the interpolation space), shadows, positions, registered custom properties by syntax; motion-path (`offset-*`) sampling, through the SVG spec's path functions so CSS `offset-path` and SMIL `animateMotion` share one numeric domain ([SV 6.6, 6.18][SV]) | Style's computed-value type (ST 6.9: IEEE, no FMA contraction); the spec's two-product form `(1 − p)·A + p·B`, which returns B exactly at p = 1 where `A + p(B − A)` missed in 8.7% of random pairs ([DR 3.10, 3.12][DR]); transcendentals (`pow`, `cbrt`, `atan2`, `sin`, `cos`, `log`, `exp`) only through `hg-math` | These are style's values, serialized to script in f64 (ST 6.9, MD 6.5), consumed by layout's boundary conversion (LY G39) and by paint |

**One trigonometry for static and animated transforms.** A static `rotate(30deg)` must paint exactly like an animation filled at 30°, or zero-fuzz reftests and the "`getComputedStyle` shows what is drawn" rule break at the boundary ([CR C15][CR]). So **every** transform function's trigonometry, static or animated, goes through `hg-interp`'s degree functions, and style's computed matrices for transform functions come from them (an amendment to the style and graphics specs, 9.5). The fixed-point result of a transform or opacity becomes style's computed value by one conversion to f64, so `getComputedStyle` shows exactly what the compositor draws. Values beyond the canonical range saturate exactly as static transforms do at paint's boundary (GR 6.3). Colour interpolation stays in the f64 domain on the main thread (off-main-thread colour is held, AN-D4), so `hg-interp` needs no floating point and G61 stands.

**`hg-math`'s functions must be identical on every ISA**, which the JavaScript spec already requires (JS 6.9). This spec asks the JS owner, as an amendment, to make them **correctly rounded** in binary64 (and to add degree variants): correct rounding is the one specification that is implementation-independent and checkable against an oracle, and published work shows it is achievable (the CORE-MATH project's papers and worst-case lists are usable evidence; its code is not, under M-1) ([DR 2.2–2.3][DR]; [DR 2][DR]). Identity across ISAs does not depend on the amendment; checkability does. The fixed-point degree functions of `hg-interp` are checked **exhaustively**: sine and cosine over the reduced domain [0°, 90°) in Q32.32 (about 3.9 × 10¹¹ inputs) and arccosine over [−1, 1] in Q32 (2³³ inputs) against MPFR through `rug` (test only), so "correctly rounded" is proved, not sampled ([CR C24][CR]).

**Hygiene** (G133): no FMA except explicit `mul_add`; no `algebraic_*` operations, no SIMD float reductions, no estimate instructions; NaN canonicalized wherever its bits are observable (JS-D12 (a)); the floating-point control register (MXCSR, FPCR, fcsr) is **asserted at each frame start** in the renderer, the content GPU process and the compositor, because a loaded library can turn on flush-to-zero for a whole process ([DR 1.11][DR]); a failed assertion is a crash report, not a silent fix.

### 6.3 The frame: one tick, the order of steps, and "ready" (AN-D3, AN-D5)

**One tick, a frame time per document.** HTML passes each document the frame timestamp (the event loop's last render opportunity time) as a **relative high resolution time** for that document's global object, both to "update animations and send events" and to the animation frame callbacks ([CF 4.1][CF]; [FK][FK]); the document timeline's current time equals the rAF argument for the frame ([SP 2.3][SP]). In Izcal the rendering opportunity is the compositor's BeginFrame (GR-D15, DM 6.7), whose vsync time arrives as an integer **tick T** on the substrate's monotonic clock and is recorded (DM's decision-point register). For each document, **F = (T − the document's time origin), quantized down to that document's coarsening grid** (100 µs, or 5 µs for a document with cross-origin isolated capability; one event loop can hold both), **with no jitter**. Every timeline, every rAF callback and every off-main-thread sample of that document in that frame uses that F; every document of the event loop derives its F from the same T. A **dedicated worker** running rAF for `OffscreenCanvas` (DM 6.7) receives T on its own sequence (recorded) and derives F from its own time origin the same way.

**Why no jitter, and the order invariant.** DM-D10 jitters every `DOMHighResTimeStamp` from a per-realm stream. Applied to F, it would give the compositor (which holds no realm entropy) no way to compute the sample time, breaking the one-function rule; sending a per-cluster jitter offset in every hand-off record was considered and rejected, because it adds per-record state for no defensive value. The frame time is a sample taken once per frame and already frame-granular; jitter defends against an attacker spinning on a free-running clock to find its edges, which a once-per-frame value does not offer, and the frame boundary is itself a clock that coarsening cannot remove ([SR 4.8][SR]). The defences that matter are elsewhere (6.14). `performance.now()` keeps DM-D10's jitter, with one invariant this spec asks for: **`performance.now()` never returns less than the realm's latest F**, so `now − timestamp ≥ 0` holds in every rAF callback (an amendment to DM-D10, 9.5).

**Sample time of a presented frame.** Every presented frame belongs to one tick T: the BeginFrame that produced it. The compositor samples class C animations, and the content GPU process class G animations, at each document's F for that T (the hand-off record carries the document's time origin and grid); the main thread's content in the frame was produced at the same T, or at an earlier T when the main thread missed the deadline, in which case the frame record marks it **partial** (GR 6.9). The group rule (6.5) keeps every animation of one element on one side, so a partial frame never splits one element across two times. This is the rule Firefox reached after its one-vsync skew bug ([CF 3.6][CF]), written down.

**The animation step.** Izcal's "update animations and send events" ([SP 2.4][SP]) is split so that **no script runs while the animation graph is walked** (G130), which is the class of the in-the-wild exploits ([SR 2.1–2.2, 3.1–3.3][SR]):

1. **Walk** (`hg-anim`, no script reachable): set every timeline's current time (document: F; scroll: from `hg-scroll`'s offsets after the frame's scroll step, LY-D13; view: from the subject geometry of the last layout; SMIL fragment timelines, 6.8); run pending play and pause tasks that the ready rule releases; update each animation's finished state; remove replaced animations. The walk writes a **change list**: promises to resolve or reject, with their values, and events with their scheduled event times and composite-order keys. It iterates over handles into the document's animation arena (G131); script cannot free anything during it because no script runs.
2. **Settle** (`hg-script`): for each change-list entry in order, **re-validate its handle and generation** (an animation cancelled, removed or retargeted by script that ran for an earlier entry is handled by the spec's own rules, and stale entries are dropped), then resolve or reject the promise. Resolving a promise with the `Animation` object reads its `then` property synchronously, so an author `then` getter runs here, during settlement, and its reactions are queued as microtasks.
3. **Checkpoint, sort, dispatch** (`hg-script`): the microtask checkpoint, then the pending event queue sorted by scheduled event time (unresolved first) and composite order, then dispatch ([SP 2.4][SP]).

**This reorders one thing, deliberately.** In the specification's order a `then` getter can run synchronously **inside** the timeline update, when a pending play task resolves a ready promise with its animation, which is exactly how CVE-2024-9680 freed an animation mid-tick ([SR 2.1][SR]; [FK][FK]). In Izcal it runs after every timeline has been updated and before the microtask checkpoint. The specification leaves the ready moment to the implementation ([SP 2.7][SP]), so the order is permitted; it is still an **Izcal choice**, tagged under G135, filed with the CSSWG, and observed black-box in other engines (Q-AN-17). `hg-anim-fuzz` has cases in which the getter cancels the animation of a later entry, with AN-5's invariants asserted.

**The ready rule (AN-D5).** Web Animations leaves the ready moment to the implementation: the pending play task runs when the user agent has "completed any setup required to begin the playback ... including rendering the first frame", never synchronously ([SP 2.7][SP]; [CF 3.1–3.2][CF]). Izcal's rule: **a pending task is released in the first animation step after the rendering update that committed the animation's first frame to paint, and its ready time is that committed frame's F.** The start time is then `F_commit − hold time / playback rate`, so the first displayed frame showed the value at the hold time and the next frame continues from it with no jump. An animation whose effect has nothing to paint (no target, target not rendered) commits trivially and is released in the next step. The renderer resolves the start time, never the compositor: the compositor receives it in the hand-off record. So "ready" is a pure function of the recorded frame sequence and needs no decision point of its own. A document with no rendering opportunities (hidden, FP-D4) releases nothing until it has one.

**Hidden documents.** HTML filters hidden documents out of the rendering update, so their timelines do not advance; when the document is shown, the next frame's F jumps forward and animations jump with it ([SP 13.9][SP]). Izcal implements exactly that, which is what libraries were built around ([HP 1.3][HP]). Throttled navigables (GR-D15) are the same: a navigable with no rendering opportunity runs no animation step.

**Rendering-update order.** The DOM spec owns the steps and calls owners in HTML's order ([DM 6.7][DM]); this spec owns "update animations and send events" (above), the stale-timeline re-run inside the style and layout loop (6.6), and, with graphics, "perform pending transition operations" (6.7). `hg-anim-events`, the DOM spec's interim module, is retired at AN-M1.

### 6.4 CSS Transitions and CSS Animations with style

**Style change events.** CSS Transitions does not say when a style change event happens and warns that timing-dependent code varies between implementations ([SP 6.2][SP]). Izcal defines it: **one style change event per style flush**, which happens at each rendering update's style step and at each forced flush that computes style (`getComputedStyle`, layout getters, `getAnimations()`). Flush points are already deterministic functions of script order and recorded frames (ST 6.10), so no new decision point is needed. **A flush that rebuilds style from scratch after a discard (FP-12, ST-33) is not a style change event**: it restores the same computed styles and the retained before-change styles, so it can start no transition. The decision record of each style change event (6.13) is what answers "why did my transition not start?".

**Per element, per flush, without breaking phase purity.** Style traverses in parallel and publishes only through its deterministic merge (ST 6.10, 6.12); creating animations, numbering transition generations and allocating arena entries inside that pass would make them depend on worker interleaving (Stage 1 3.3) ([CR C4][CR]). So the work is split:
1. **Before the pass**, `hg-anim` takes an **immutable snapshot of the effect stacks** at the timeline's current time.
2. **During the pass** (parallel, read-only): style computes the **base style** (the cascade without the Animation and Transition origins); keyframe values are computed against the element with that context, live, as Web Animations requires (`em` after `font-size`, `var()`, logical-to-physical mapping by writing mode) ([SP 3.5][SP]); the effect stack is folded from the base value with replace, add and accumulate ([SP 3.7][SP]); the result enters the cascade as two declaration blocks, at the Animation origin and the Transition origin, whose precedence Cascade 5 fixes (transitions above every `!important`, animations above every normal declaration) ([SP 7.1][SP]). Style reaches all of this through an **`AnimationSource` trait** that `hg-anim` implements (the pattern `hg-cssom` uses with `ComputedStyleSource`), so `hg-style` does not depend on `hg-anim`. Each worker writes, per element whose transition-relevant values changed, a **transition-candidate record** (before-change and after-change values per longhand, or the starting style) into its task-local output.
3. **After style's merge, a sequential post-pass in tree order** (`hg-anim::css`) runs the start, interrupt, reverse-and-shorten and completion rules on the candidates, assigns the per-document **current transition generation**, allocates `CSSTransition` and `CSSAnimation` objects in the arena, updates the running and completed sets, and starts or updates CSS animations from the merged `animation-*` values. If a newly started transition changes a value in this frame (a transition with a negative delay), the affected elements are restyled once more by the same rules; the post-pass runs at most once per flush per element.

An element whose animated properties do not affect selector matching restyles only the affected style groups (ST 6.10). Style supplies three primitives this spec needs and does not own (9.5): the base value of a property, "compute a property value against element E", and "`display` computed ignoring the animation origins", which CSS Animations 2 uses to terminate animations ([SP 7.6][SP]). `hg-anim` and `hg-anim-values` depend on style's read API and `hg-tree-traits`, never on `hg-dom` (G1, and SVG G120 for the scriptless core).

**Transitions.** `hg-anim::css` keeps, per element and longhand, the running transition and the completed transition with the six fields Transitions 1 names ([SP 6.1–6.11][SP]); the **before-change styles** and the "had a computed style" bit stay style's retained primary state, inside ST-30's bytes (ST 6.11). `@starting-style` gives the starting style for elements with no before-change style, and `transition-behavior: allow-discrete` makes discrete pairs transitionable, flipping at 50% ([SP 6.9–6.10][SP]); `display`, `content-visibility` and `overlay` interpolate by their special rules ([SP 8.9][SP]). Completing transitions all move to the completed set before any event fires ([SP 6.7][SP]). Elements no longer rendered cancel their transitions ([SP 6.11][SP]).

**Animations.** The `@keyframes` cascade, keyframe grouping, the three keyframe orders and animation-name matching (last to first, keeping playback time) follow CSS Animations 1 and 2 ([SP 7.3–7.5][SP]); API overrides stop CSS from updating a `CSSAnimation` after a successful `setKeyframes()`, `updateTiming()`, `play()` or `pause()` ([SP 7.7][SP]); custom properties in `@keyframes` are animation-tainted (style) ([SP 7.10][SP]).

**Events** for transitions and animations are derived from the phase change between frames with the specified elapsed times, and dispatched through the change list (6.3) ([SP 6.8, 7.9][SP]).

### 6.5 Off the main thread: what can run where, derived (AN-D4)

An animated value can be produced away from the main thread **and stay identical to what the main thread would compute** only if three things hold ([CF 8][CF]):

- **(L) Layout independence**: the value changes no box geometry, or the off-thread result drifts from layout ([CF 1.10–1.11][CF]).
- **(P) Paint independence**: either the value changes only how existing rasters are composed, or the repaint can be done off the main thread from the recorded display list, with no style resolution and no script ([CF 1.6, 2.1][CF]).
- **(D) Determinism of sampling**: the value is a pure function of the hand-off record (start time, timing, keyframes resolved at hand-off, timeline) and the frame's sample time, with no late-resolved inputs such as system colours ([CF 1.7, 2.2–2.3][CF]).

A fourth condition comes from Izcal's process model: **(S) the sampling process may hold the code and the data**. The compositor links no path rasterizer and parses no page-controlled filter, path or ICC data (G63, GR-18), so anything that needs a page path is sampled in the content GPU process or on the main thread.

| Class | Properties | Where | Status | Evidence |
|---|---|---|---|---|
| **C: compose-only** | `transform`, `translate`, `rotate`, `scale`, `opacity`; scroll-driven animations of these (class S) | Compositor GPU service (GR-D13), linking `hg-anim-time` and `hg-interp` only | **On** | Every engine started here ([CF 1][CF]); geometry stays in sync because the compositor samples at the frame's F with the same function |
| **G: geometric paint** | `filter` (amounts; no colour that changes), `clip-path` (basic shapes and `path()` with matching vertex counts, reference box frozen at hand-off) | Content GPU process (links `hg-interp-geom` and `hg-path`): substitute display-item parameters, re-raster damaged tiles at each BeginFrame (GR-D13); never the compositor (G63) | **On** (GR-D13 already chose it), with a re-raster budget of **one viewport area† per frame per document**, revised by AN-S2 | WebKit users asked for `clip-path` because reveals stutter when the main thread is blocked ([CF 1.9][CF]) |
| **G′: further geometric paint** | `mask-position`, `mask-size`, `background-position`, `border-radius`, `box-shadow` offsets and blur | Content GPU process | **Built behind a flag by AN-M2; shipped at AN-M3 if AN-S2 is positive** | Work scales with painted area per frame, so the benefit must be measured ([CF 2][CF]) |
| **U: uniform paint** | `background-color`, `color`, `border-*-color`, `outline-color`, `fill`, `stroke`, shadow colours, `drop-shadow()` colour | Content GPU process (colour interpolation then needs `hg-math` there, under ST 6.9's f64 contract) | **Built behind a flag by AN-M2; shipped at AN-M3 if AN-S2 is positive** | Chrome's two 2021 trials conflict: no dropped-frame improvement and a memory regression in the first; "very good results on throughput metrics" and about +1.5% memory on Windows in the second, with no figure published ([HP 6.6][HP]; [CF 1.7][CF]; [FK][FK]) |
| **M: motion path** | `offset-path`, `offset-distance`, `offset-rotate`, `offset-anchor`, `offset-position` | Main thread (paint), sampled with the SVG spec's path functions (6.2) | **Main thread**; moving it off would need a bounded, pre-flattened transform table that changes the sampled values, which P2 forbids unless the main thread uses the same table; revisited with AN-S2 | G63, GR-18 ([CR C1][CR]) |
| **L: layout** | `width`, `height`, insets, margins, padding, `font-size`, grid tracks; percentage transforms on a box whose size changes every frame | Main thread only | — | — |

**Rules** (normative, each with a test):
1. **Group rule.** All animations of one element whose results meet in one composited node run on one side; a class L or M animation on an element keeps its class C and G animations on the main thread ([CF 3.7][CF]; [CF 8][CF]).
2. **Hand-off snapshot.** Eligibility and every resolved input (keyframes in `hg-interp` form, percentages resolved against the current box, the reference box, the underlying value for `add` and `accumulate`, the timeline's range snapshot) are decided **once**, at the commit that hands the animation over. Any later change (keyframes, box size, timing, a composite operation's underlying value, a timeline range) re-commits a new record version; nothing is patched in place ([CF 2.2][CF]).
3. **One time, one function.** Off-main-thread samples use `hg-anim-time` and `hg-interp` at the frame's F (6.3), so their values equal the main thread's bit for bit for the same inputs. The compositor reports per frame the sample time and, for class S, the scroll offset it used, and when an animation reached its end (GR 6.28); those reports feed frame records, DevTools and AN-6's check, **never** the model, which computes ends analytically.
4. **The main thread never waits.** A main-thread read (`getComputedStyle`, `getAnimations()`, `currentTime`) samples the model; there is nothing to fetch from another process.
5. **Caps and demotion.** GR's 16† compositor-mutable nodes per document bound class C (GR 6.3); class G's re-raster budget bounds class G. Over a bound, the lowest-priority animations (by GR's priority order, then by area) are demoted to the main thread; each demotion is a decision record with its rule (6.13), because silent fallback is the two-engines failure ([HP 6.8][HP]).
6. **Bounded records.** The compositor holds every site's pixels and runs within a fixed CPU budget (FP-1), so a page cannot be allowed to push unbounded animation state into it ([CR C5][CR]): a hand-off record carries at most **64† keyframes** and **32† transform functions per keyframe**, encodes to at most **4 KiB†**, and the compositor holds at most **256 KiB†** of animation records per renderer key, charged to that key in the GPU ledger (FP 6.6). An animation over any cap is demoted to the main thread with a decision record. The record IDL's size is published with its fuzz target as GR-18 requires, and its growth beyond GR 6.28's record goes through GR-18's security review (9.5).

**The hand-off record** extends GR 6.28's: record id and version; property and composited node; keyframes in `hg-interp` form with per-keyframe easing and composite operation; effect timing (start and end delay, iteration duration, iteration count, iteration start in Q32, direction, fill, effect easing); playback rate; start time (`AnimTime`); the document's time origin and coarsening grid; timeline (document, or scroll node with axis and the range snapshot, or view timeline with the subject's range snapshot); for `add` and `accumulate`, the static underlying value.

**Events without the main thread.** A class C or G animation needs the main thread only for its events. The renderer computes each animation's next event time analytically (phase and iteration boundaries) and arms a substrate timer for it (6.11) **only if the event could be observed**: a promise is pending, or a listener or event-handler attribute for that event type exists on the event's path (target, ancestors, `Document`, `Window`). The DOM spec's listener ledger is a DevTools-attached record, so this spec asks it for an **always-on, constant-time summary** per event type of whether any listener exists on a node's event path (9.5) ([CR C13][CR]). When the timer fires, it **requests a rendering opportunity** for that document; the events are dispatched in that frame's "update animations and send events", as HTML requires. An infinite compositor-only spinner with no `animationiteration` listener therefore never wakes the main thread.

### 6.6 Scroll-driven animations and view timelines (AN-D8)

A scroll progress timeline's current time is the scroll offset over the scroll range; a view progress timeline's is the subject's position within a named range of its scrollport ([SP 9.1–9.2][SP]). Izcal computes both as exact Q32 rationals of layout's 1/64 px integers (LY-D3). **Sampling point**: at step 1 of the animation step, from `hg-scroll`'s offsets after the frame's scroll step has applied the compositor's recorded deltas (LY-D13), and from the scroll ranges and subject geometry of the last layout ([SP 9.5][SP]). Named timeline lookup and `timeline-scope` follow the spec's global-with-scoping rules ([SP 9.3][SP]); finite attachment, `animation-range` and range-named keyframes follow Scroll-driven Animations 1 ([SP 9.4][SP]).

**Stale timelines.** The spec allows one extra style and layout pass for timelines that are new or whose ranges changed, and cites HTML steps that no longer exist ([SP 9.5, 13.1][SP]). Izcal's rule: after the frame's style and layout, a timeline is **stale** if it was created in this frame, its named ranges changed, or (Izcal's addition, as CSSWG #14337 argues) its view-timeline subject's geometry changed; if any is stale, all are updated and **exactly one** further style and layout pass runs; later changes wait for the next frame. The rule is a pure function of layout outputs, so it replays. The addition and the missing HTML step are filed (Q-AN-4).

**Off the main thread.** The compositor samples class S animations from **the offset it draws in that frame** and the range snapshot of the last commit, which is stricter than the specification's permission to be inconsistent ([SP 9.6][SP]; GR 6.8). A layout change that moves a range re-commits (rule 2 of 6.5). The frame record carries the offset each class S sample used, so AN-6 compares like with like.

**Zero scroll range.** The CSSWG resolved on 27 May 2026 that scroll and view timelines **do not become inactive** when there is no scroll range (#9256), and on 3 June 2026 that auto-rewind has no effect while a timeline is unresolved (#13817); the Editor's Draft text has not been updated and WPT is moving to the resolutions ([FK][FK]). Izcal implements the resolutions, as it implements the reduced-motion cross-fade resolution (6.7), and files for the text (Q-AN-5).

**Idle.** A scroll-driven animation requests frames only while its timeline's time changes; an idle page with one produces **zero frames** (AN-30), the bug WebKit's threaded version shipped and fixed ([CF 5.4][CF]).

### 6.7 View Transitions (AN-D7)

**Same-document** (View Transitions 1, and Level 2's types, classes and `waitUntil()`). `hg-view-transition` runs the lifecycle as specified: `startViewTransition` → capture the old state → rendering suppressed → update callback (always called, even when skipped) → capture the new state → pseudo-element tree → animate → clean up ([SP 10.1–10.2][SP]). **Capture**: in "perform pending transition operations", it walks connected elements in paint order through `hg-paint-order`, skips the ones the spec skips, fails on duplicate names, and records each element's border-box size, transform from the snapshot origin, writing mode, direction, text orientation, blend mode, backdrop filter and colour scheme from the fragment store (LY 6.19) ([SP 10.3][SP]). **Snapshots are never pixels in the renderer**: the content GPU process rasters each captured element into a snapshot surface as the spec's "capture the image" defines (ancestor transforms and effects ignored, own opacity and filter applied, cropped to ink overflow), and an element whose subtree contains a cross-origin surface is captured **by the compositor** into a texture only the compositor samples (GR 6.8, 6.28) ([SP 10.4][SP]). The renderer holds capability ids. Capture completes asynchronously; its completion and budget result arrive from the GPU process as recorded inputs (6.12).

**Effects on compositor-held snapshots.** The `::view-transition-old` and `::view-transition-new` pseudo-elements carry author styles (`filter`, `mask`, `clip-path`, `mix-blend-mode`, `backdrop-filter`), and the default cross-fade uses `mix-blend-mode: plus-lighter`. A compositor-held snapshot can take only what the compositor may apply under G63: opacity, transforms, rectangular and rounded-rectangle clips, and `plus-lighter` if the graphics spec admits that blend in the compositor (9.5). **Any other effect on a pseudo-element whose image is compositor-held skips the transition**, with `InvalidStateError` and a recorded reason (A7) ([CR C11][CR]). The content GPU process never receives those pixels (G56), and GR-D18's rules apply because such a snapshot is a cross-origin surface (G137).

**The pseudo-element tree** is generated as UA-origin dynamic style rules and keyframes, fed to style like any other UA sheet, and its animations are ordinary CSS animations on `hg-anim` ([SP 10.5][SP]): opacity cross-fades run on the compositor (class C); the group's `width` and `height` animation is class L, but its layout is a top-layer, out-of-flow box with replaced content, so it cannot dirty the page's layout. **Per frame**, "handle transition frame" ends the transition when nothing on the tree runs, skips it if the snapshot containing block changed size, and otherwise updates the new elements' geometry and group styles ([SP 10.6][SP]).

**Limits, published** (the specification leaves them all to the implementation, [SP 14.4][SP]; [SR 5.7][SR]): at most **1,024† named elements** per transition; snapshot bytes per transition within GR-D23's View Transition line, beyond which the transition is skipped with `InvalidStateError` and a recorded reason; each captured element rastered at full resolution within the viewport plus one viewport of margin† and at half resolution† beyond, which the spec allows ("at the very least, some low-quality representation", [SP 10.4][SP]); the update-callback timeout and the inbound cross-document timeout are **4 s†** each, to be confirmed by black-box observation (Q-AN-6; the figure circulating for Chrome is unverified, [SR 5.8][SR]). Timeouts run on the substrate clock, so they replay.

**Cross-document** (View Transitions 2). Both documents opt in with `@view-transition { navigation: auto }`; the transition is eligible only if the navigation is same-origin, not a reload, has no cross-origin redirect (unless the new document's latest session-history entry is non-null, a back-forward-cache restore), and, for push and replace, was not initiated from browser UI; a user agent may also decline when it shows its own navigation experience ([SP 10.9][SP]; [SR 5.2, 5.5][SR]; [FK][FK]). The old document fires `pageswap` and captures; the new document resolves the inbound transition in HTML's "reveal" step, fires `pagereveal`, and waits for render-blocking resources ([SP 10.9][SP]). Izcal's placement, because P1 and I-1 apply:

| Question | Izcal's answer |
|---|---|
| Who decides eligibility? | **`hg-navigation` in the broker** (G136). The old renderer's opt-in and capture are requests; the broker checks the committed origins of both documents, the redirect chain it saw, the navigation type and initiator, and, as an **Izcal choice** stricter than the spec (tagged, filed, and observed black-box, Q-AN-6), that both documents have the **same cross-origin isolation state**; the spec relies on its cross-origin-iframe rule for that case ([SR 5.3][SR]). A mismatch skips the transition |
| Where do old snapshots live between `pageswap` and `pagereveal`? | Where they were rastered: the **content GPU process of the renderer key**, or the compositor for cross-origin-surface snapshots. Same-origin documents have the same key under FP-D8, so the snapshots never cross a site boundary; the broker re-grants the capability ids to the new document only after its checks |
| Can any process read the pixels? | No (G137): View Transitions 1 says "the implementations must ensure this data can not be accessed by the Document", and Izcal makes that a mechanism: no API converts a snapshot capability into readable pixels, it is not a `CanvasImageSource`, and a snapshot containing a cross-origin surface is itself a cross-origin surface for GR-D18's effect rules ([SR 5.1][SR]) |
| What does the new document learn? | `PageSwapEvent.activation` (in the old document) is null unless the new document is same-origin and was not created through cross-origin redirects; `PageRevealEvent` carries only the view transition; the new document's `navigation.activation.from` is null when the previous document was not same-origin, as HTML defines ([SR 5.6][SR]; [FK][FK]); plus the transition types the old document set |
| When are snapshots freed? | When the transition ends, is skipped or times out, and when either document is destroyed; they are charged to the page throughout (GR-D23) |

**Element-scoped transitions and nested groups** (Level 2) are held by admission (AN-D9). **Reduced motion**: under `prefers-reduced-motion: reduce`, the UA stylesheet replaces the group's size and position animation with a cross-fade, as the CSSWG resolved on 24 September 2024 ([MA 5.8][MA]); the text is not yet in the drafts ([MA 5.9][MA]), so Izcal implements the resolution and files for the text (Q-AN-7).

### 6.8 SMIL (AN-D6, with the SVG spec's SV-D4)

SVG 2 makes animation optional, and SVG Animations 2 defers to SMIL Animation (2001) for the timing of `animate`, `set`, `animateMotion` and `animateTransform` ([SP 12.1–12.2][SP]). No specification maps SMIL onto Web Animations' effect stack ([SP 12.5][SP]). Izcal implements SMIL because the web uses it: its deprecation was reversed for lack of a replacement, and it is on 2.90% of Chrome page loads, with SMIL inside SVG images on 1.67% ([HP 2.10–2.12][HP]; [SV 6.9][SV]).

**The split, agreed with the SVG spec** (its draft 0.2 adopts this spec's AN-D6 and withdraws its own `hg-smil`, [SV 6.9][SV]). **This spec owns SMIL timing**, in `hg-anim`'s scriptless core: the interval engine (begin and end lists with offset, syncbase, event, repeat, accessKey, wallclock and `indefinite` values), the fragment time container (`timelinebegin`, `playbackorder`), restart, `min`/`max`, fill, repeat, the dependency graph and its cycle rules, **child timelines per SVG fragment** with pause and seek (`pauseAnimations`, `setCurrentTime`, implemented as the SVG spec's `SmilControl` trait), and the mapping of each active interval to an effect in the effect stack (the sandwich model is the effect stack with `additive="sum"` and `accumulate="sum"`, the latter through the internal iteration composite `accumulate`, [SP 12.3][SP]). **The SVG spec owns** the elements, the parsing of every timing and value attribute into `AnimTime` and typed values, which attributes are presentation attributes (CSS properties) and which are XML attributes, the SVG DOM's `baseVal` and `animVal` (SVG 2 makes `animVal` an alias of `baseVal`), `animateMotion`'s path geometry, `rotate="auto"` tangents, `keyPoints`, **paced offsets** (supplied to this spec), value interpolation of SVG types through **`hg-svg-interp`** behind the `InterpolateValue` trait (one function for CSS `d` and `transform` and for SMIL attribute effects), and the cloning of animation elements into `use` shadow trees ([SP 12.4][SP]; [SV 6.9][SV]). The reason for the split: timing is the part that must share one model, one clock and one composite order with CSS and script animations; attribute semantics belong with the attributes.

**Where values go.** SMIL effects on **CSS properties** enter the Animation origin as a SMIL composite class, ordered within the class by SMIL's priority (interval begin time, then document order). The class's position against CSS animations and script animations is **measured black-box first** (the SVG spec's Q-SV-7 and this spec's Q-AN-1) and then decided; until then the provisional position is after CSS animations and before script animations, tagged as an Izcal choice. SMIL effects on **XML attributes** write the **animated-attribute overlay** (a side table keyed by node slot and attribute), which `hg-tree-traits`' attribute accessors return to every rendering consumer (style, the reference graph, the instance builder, layout, text), while the DOM's `getAttribute`, serialization and `animVal` do not see it; a change in the overlay is an invalidation input like an attribute mutation ([SV 6.9][SV]). No animation writes the DOM. Before-change styles include SMIL values updated to the current time, so SMIL never starts a CSS transition ([SP 6.3, 12.5][SP]).

**Timelines and energy.** A fragment timeline begins at the SVG load event (or at load begin with `timelinebegin="loadbegin"`), recorded as a time on the parent timeline, so replay never re-measures network timing; event-based begins exist only in scripted documents, each event's time being the recorded time of its task; `wallclock()` values resolve against the substrate clock, recorded ([SV 6.9][SV]). **SVG images** get one fragment timeline per (image document, embedding document), starting when the image is first used for rendering there and shared by every use in that document ([SV 6.9][SV]); opaque SVG images render in the media process (MD-D20), where `hg-anim`'s scriptless core runs with a null event sink and receives the embedding renderer's ticks for visible animated images (an interface amendment to the media spec, 9.5). Off-screen or hidden fragments are not sampled for rendering; in scripted documents a fragment whose SMIL events have observers, or on which another fragment's syncbase depends, still resolves its intervals at each rendering opportunity so `beginEvent`, `endEvent` and `repeatEvent` fire on time; catch-up after a gap has its own work budget, separate from the per-frame interval budget, and never makes an animation inert ([SV 6.9][SV]).

**Robustness** (the SMIL code paths were the densest crash area in one DOM fuzzer and the origin of 18 lifetime CVEs, [SR 1.2, 1.8][SR]): the dependency graph lives in the document's animation arena with generational handles; closed cycles are detected per SMIL 3.0 and their change propagation ignored; open cycles get a **per-frame interval budget** (6.14); no recursion on graph depth (G138). `accessKey` begin values receive only key events delivered to the document while it has focus, because `accessKey` timing once captured keystrokes without script ([SR 4.10][SR]). The scriptless core depends on no `hg-dom`, `hg-heap`, `hg-js-host` or `hg-script` crate (SVG G120); listener facts reach it through an interface (6.5).

### 6.9 Feature admission (AN-D9)

ST-D18's rule binds animation features: a working-group resolution, WPT tests, and two shipping engines or an Interop focus area.

| Feature | State (October 2026) | Decision |
|---|---|---|
| Web Animations 1; CSS Transitions 1 and 2 (`transition-behavior`, `@starting-style`); CSS Animations 1 and 2 (`animation-composition`, `animation-timeline`); `linear()`; `@property` interpolation; `display`, `content-visibility`, `overlay` transitions | All three engines ([HP 7.4, 8.4, 11.1–11.3][HP]) | **Admitted** |
| Scroll-driven Animations 1 | Chrome 115, Safari 26; Interop 2026 ([HP 9.4, 9.6][HP]) | **Admitted**, with the 2026 zero-range resolutions (6.6) |
| View Transitions 1 (same-document), `view-transition-class`, types | All three engines; Interop 2025–2026 ([HP 10.4–10.5][HP]) | **Admitted** |
| Cross-document View Transitions | Chrome 126, Safari 18.2; Interop 2026 ([HP 10.4–10.5][HP]) | **Admitted** |
| SMIL in SVG | All three engines ([HP 2.11][HP]) | **Admitted** (with the SVG spec) |
| Iteration composite `accumulate` | No engine exposes it to script ([HP 7.3][HP]) | **Implemented internally** for SMIL; the script API is **held** |
| `GroupEffect`, `SequenceEffect`, effect playback rate | No engine ([HP 7.3][HP]) | **Held**; the timing tree is ready (6.1) |
| Custom effects | Spec says it "needs to be revisited" ([SP 4.6][SP]) | **Held** |
| Animation Triggers 1 (`animation-trigger`) | FPWD 20 October 2026 ([SP 1.6][SP]) | **Held** |
| Element-scoped View Transitions, nested view-transition groups | Chrome only ([HP 10.4][HP]) | **Held** |
| `interpolate-size`, `calc-size()` | Chrome only | **Held** (LY-D10) |
| Animation Worklet | No engine; zero use ([HP 8.3][HP]) | **Not implemented** |
| Per-animation frame-rate hints | Open CSSWG issue ([SP 13.7][SP]) | **Held**; Izcal's internal votes (6.11) need no syntax |

### 6.10 Reduced motion and user controls (AN-D10)

No specification says what a user agent should do to running animations when the user prefers reduced motion ([SP 14.5][SP]), and no browser does anything to author animation ([MA 4.7][MA]). The constraints are clear: removing an animation fires no end event and can leave content invisible, while changing its **timing or interpolation** keeps events, promises and end states ([MA 6.1–6.6][MA]); the CSSWG resolved in 2021 to specify a mode that makes motion-inducing animations **discrete between keyframes** and in 2022 to allow "more granular animation control" ([MA 5.2–5.4][MA]); WCAG defines motion animation as change of perceived size, shape, position or distance and depth, not of colour or opacity, and an erratum removed blur from that exclusion ([MA 3.3–3.4][MA]; [FK][FK]); UAAG 2.0, a W3C Note, asks for a user control to pause animation lasting three seconds or more ([MA 3.6][MA]).

**One record, one exposed value.** The accessibility spec's `hg-ax-prefs` carries the OS reduce-motion boolean, Android's duration scale where present (internal only), Apple's separate image and flash settings, and Izcal's global and per-site motion level and flash-protection setting ([AX 6.11][AX]). **The page sees only `prefers-reduced-motion` at its specification values**, exposed under the accessibility spec's R0 policy (HGBA-3 B10: OS accessibility preferences are exposed as the specs require and not normalized by anti-fingerprinting), and **every behaviour this section describes reads that one exposed value or a level the user set**, never the raw OS value, because a hidden media query was once leaked by smooth-scroll behaviour ([SR 6.4][SR]).

| Level | When | What the engine does | Visible to the page |
|---|---|---|---|
| **R: Reduce** | OS reduce-motion on, or the user's Izcal setting | `prefers-reduced-motion: reduce`; the accessibility spec's UA behaviours (no smooth scrolling, `marquee` paused, MD-D19's animated images and muted looping videos on a first frame with a play control); the SVG spec's SV-D16 (SVG images render their t = 0 sample paused with MD-D19's play control, and inline SMIL fragments pause at t = 0 with a page-level play control, because authors cannot gate SMIL with a media query) ([SV 6.9][SV]); View Transitions use the cross-fade (6.7). CSS and script animations are untouched | The media query, as specified |
| **S: Reduce strongly** | User opt-in, global or per site | As R, plus: animations of **motion-inducing properties** (transform family, `offset-*`, insets, `width`, `height`, margins, `filter` and `backdrop-filter` blur, and scroll-driven animations of these) that are **not marked safe** use **discrete interpolation between keyframes**. Within each interval the value is held and flips at the interval's midpoint, as the discrete animation type does; a run of consecutive keyframes spanning less than **1 s†** of time is **merged**, and the run shows its last keyframe's value from the run's start ("snap forward"), so a 300 ms transition jumps to its end value at once; scroll timelines step once per **scrollport length†** along the timeline's axis. Colour and opacity animations are unchanged; timing, events, promises, fills and end states are unchanged. An animation is **marked safe** if its `@keyframes` rule or its `animation` declaration was inside a matching `prefers-reduced-motion: reduce` media block, or (if the CSSWG specifies it) declared reduced-motion-safe in script ([MA 5.7][MA]) | The media query; and the discreteness, which script can observe through computed values (counted in PRIV-2, 6.14) |
| **X: Stop motion now** | One command and keyboard shortcut in the shell, for the current page (all its frames), until undone | A **UA pause flag** per document: each animation's current time is held as the specification's pause procedure holds it (with its events and promises), applied to animations created while the flag is on, **without** setting the API override, so CSS keeps control of `animation-play-state` after undo ([CR C14][CR]); running View Transitions are **skipped** (finished), not paused, so no pseudo-element overlay stays over the page; SMIL fragments pause; animated images and muted looping videos pause (MD-D19); `marquee` stops. rAF keeps running; the shell offers to throttle the page's rAF to **1 Hz†** with a visible indicator, because script-driven motion is outside any style-level control ([MA 6.5][MA]). Undo releases the flag | Pause states (a user action, like pausing a video); the rAF throttle if accepted |

**Infinite animations** under S step between their keyframes like any other; a spinner whose two keyframes are 0° and 360° therefore holds still, which is the intent.

**Profiles and privacy.** Under R0, the Hardened profile (a security profile, S-4a) exposes the preference exactly as the Standard profile does. Only D-S1's **opt-in full-uniformity profile** reports `no-preference`, uniformly, with no per-site exception; in that profile the UA behaviours of level R follow the reported value, and the shell says that reduced motion then stops working, while levels S and X remain available as explicit user choices with a warning that they are detectable ([AX 6.11][AX]; [CR C7][CR]).

**Flash protection.** The graphics spec already measures WCAG 2.3.1 flashes per frame over damaged regions (GR-D25) and proposed a user flash-protection mode to the shell. This spec adopts it as a motion setting: when the user turns it on, the compositor dims the **damaged region** that exceeds three general or red flashes in any second, for the rest of the flashing, invisibly to script. The method uses WCAG 2.3.1's definitions only; a patent and royalty search runs at AN-M0 under MD-D3's precedent, because automated flash analysis is a field with commercial tools ([CR C25][CR]). No browser does this today, and Apple's shipped mitigation covers video only ([SR 8.8–8.9][SR]; [MA 4.8][MA]).

**Breakage is measured, not assumed**: no published breakage rate exists for any forced-motion mode ([MA 6.7][MA]), so S is never suggested by default anywhere until AN-S3's corpus shows ≤ 1%† breakage (AN-70). Every level's exact rules are published and offered to the CSSWG as an implementation report for #7440.

### 6.11 Energy: frames only for visible change (AN-D11)

**Frame demand.** Each document keeps a **frame-demand registry**: a set of owners (animation ids, rAF, View Transitions, media, scroll, SMIL fragments) that need a frame, keyed by owner, so double registration cannot exist ([SR 2.5][SR]). Every animation computes its **next visible change time** analytically from its timing and easing: the next frame for continuous easing; the next step boundary for `steps()`, discrete interpolation and flat `linear()` segments; the end of the delay; never, for a fill-held value. The renderer asks the compositor for BeginFrames only while some owner needs the next frame, and otherwise arms a substrate timer at the earliest next change (with a tolerance of one frame interval) that **requests a rendering opportunity** when it fires. So a `steps(1)` blinking caret with a 1 s period wakes twice a second, not 120 times, the case that once cost an editor 13% CPU ([GO 4.11][GO]). When no owner needs a frame, the document has no rendering update, as HTML's "unnecessary rendering" skip allows, and its document timeline does not advance until the next update ([SP 2.3][SP]).

**Invisible animation costs nothing.** An animation is **not sampled for paint and does not register frame demand** while all of these hold ([CR C3][CR]):
1. its target is not painted now: not rendered, `visibility: hidden` with no visible descendant, or outside the viewport plus the prepaint margin, judged by the animation's **swept bounds**: the union of the target's ink bounds under every keyframe value over the whole active interval, computed conservatively at commit by `hg-interp` (for rotations, the bounding circle about the transform origin), and recomputed when layout moves the target or a scroll moves the viewport;
2. no running animation on the target or an ancestor can change that: no animated `visibility`, `display` or `content-visibility`, no ancestor `opacity` animating up from 0, no `clip-path` or `offset-*` animation, no transform family animation whose swept bounds reach the viewport plus margin;
3. skipping cannot change a script-observable value: the animation is not class L (its layout can change scrollable overflow and other boxes), and either its animated properties are not inherited or no element in the subtree lists them in `transition-property`, so no descendant's transition depends on per-frame style change events.

Its timeline still advances whenever the document has a rendering update, its values are computed on demand for any script read or style change event, and its events are scheduled analytically (6.5), so under these conditions skipping changes no value script can read. This is the regression test for Firefox's hidden spinner (60% CPU → 1%, [MA 7.4][MA]). Separately, three cases are the specifications' own behaviour, not this optimization: **hidden and frozen documents** and **throttled navigables** get no animation step at all (6.3; FP-D4; GR-D15); **skipped `content-visibility` subtrees** follow CSS Contain 2: their animations do not advance or end and no new ones start, a style query samples them at that style change event, and they resume from that point when the subtree stops being skipped ([MA 8.5][MA]; [FK][FK]).

**Rate by vote, per renderer.** The compositor drives the display at the highest rate any visible surface needs (GR-D15), and **decimates each renderer's BeginFrames to that renderer's own vote**, so one site's animations never change another site's rAF cadence ([CR C17][CR]). Votes follow Android's adaptive refresh rate ([CF 4.12][CF]): an animation that moves or resizes something, a scroll-driven or input-driven animation, and any document with rAF callbacks vote **High** (the display's rate); opacity, colour and filter-amount animations vote **Normal** (≈ 60 Hz, or the display's rate if lower); the highest vote of a renderer's documents wins. Under the OS low-power or battery-saver signal and under FP-D15's "limit this site on battery", a renderer's rate is capped at **30 Hz†**, the published iOS precedent ([MA 8.7][MA]); user scrolling and input-driven updates are **exempt** from the cap, because the user is acting; Chrome and Edge publish no rule ([MA 8.9–8.10][MA]), Izcal publishes its rates, triggers and exemptions and logs each change in FP's intervention log. The Hardened profile's **60 Hz** clamp (GR-D15) composes with the votes as a ceiling. rAF cadence follows the renderer's rate (an amendment to GR-D15's wording, 9.5); its bits are the display model's bits plus the page's own content, counted in PRIV-2.

**Measured, not assumed.** The literature has one lab measurement of animation power (about 25 mW idle against 100–500 mW for CSS and SVG animations on one laptop, [MA 7.1][MA]; [FK][FK]) and no peer-reviewed phone measurement ([MA 7.7][MA]). Izcal publishes its own table per platform (AN-35).

### 6.12 Recording, replay and determinism

Every input the model reads is recorded or derived from recorded inputs ([DR 8.9][DR]; [CR C12][CR]):

| Input | How it is recorded |
|---|---|
| Frame ticks T | BeginFrame ticks (GR-D15, DM's register); each document's F derived deterministically (6.3); ticks forwarded to dedicated workers on their sequence |
| Which frames were produced, skipped or throttled; each renderer's rate | Frame records and the compositor's timing stream (GR 6.9) |
| Scroll offsets for scroll timelines | LY-D13's recorded deltas; the compositor's drawn offset per frame, in the frame record |
| Off-main-thread sample times, offsets and ends | Reported per frame (GR 6.28); used only by tools and AN-6 |
| Preferences (reduced motion, motion level, flash protection), the OS low-power signal | ST's environment snapshot (a mutation-log entry on change) |
| Level X and its undo | Control-thread commands from the broker (DM's register row for control-thread commands) |
| View Transition capture completion and budget results | Replies from the content GPU process or compositor, recorded at the position they take effect (DM's IPC row) |
| SMIL `wallclock()` values and event-begin times | Substrate clock and task times (with the SVG spec) |
| Ticks sent to the media process for SVG images | Recorded by the embedding renderer as it sends them |
| Device scale, refresh rate | Environment snapshot |
| Script calls (times set, keyframes, play, pause) | JS-observable inputs already under B1 |
| Analytic wake-up timers, View Transition timeouts | Substrate timers (DM's timer row) |

| Output | Deterministic because | Check |
|---|---|---|
| Timeline times, start and hold times, phases, iterations, events and their order | Exact integer arithmetic on recorded inputs (6.2); one change list per frame | DET-1 replays; AN-21 |
| Effect values and computed styles of animated properties | `hg-anim-time`, `hg-interp`, `hg-anim-values`, `hg-math`; no platform libm | Cross-ISA corpus hash (AN-20) |
| Transitions started, generations, arena handles | A sequential post-pass in tree order after style's merge (6.4) | AN-24: identical across style worker counts |
| Off-main-thread presented values | Same functions, same F, same offset (6.3, 6.5) | AN-6 |
| Ready times, stale-timeline re-runs, style change events, frame demand, demotions | Pure functions of recorded frame sequences and layout outputs | DET-1; decision records under replay |
| Which device draws, raster thread count | Free to vary because outputs are identical (GR 6.21) | GR-2 |

Recording costs a share of the one whole-renderer budget (HGBA-3 DET-1, JS-52: ≤ 1%† of p95 frame time), measured jointly and attributed per subsystem ([RN2 Q15][RN2]). Because animation time is an input, **a recording replays at any frame rate**, and "slow down", "scrub" and "step one frame" in DevTools are different input sequences given to a replay, not edits to the live page.

### 6.13 Observability, tests and agents (AN-D12)

**Decision records.** `hg-anim-observe` writes, keyed by stable node and animation ids, with spec anchors: each style change event's transition decisions per element and property (before-change and after-change values, the transitionable check, whether the element was rendered in both events, start, cancel, reverse or nothing, and the step that decided); each property's effect stack on demand (effects in composite order with composite operation, fill, replace state and the resulting value); each animation's off-main-thread class and every demotion with its rule; each ready decision; each event with its scheduled time and sort key; frame demand, votes and the reason each invisible animation was skipped; the motion level applied; each View Transition's captured elements, name matching, limits and skip reason; SMIL intervals created and cycles ignored. Cost classes follow the DOM spec's: counters always; records while DevTools is attached; everything derivable on replay ([DM 6.15][DM]).

**Why-queries** (DT-D3, answered from the production model re-run under replay with probes on):

| ID | Question | Answer |
|---|---|---|
| A1 | Why didn't this transition start? | The style change event, the before-change and after-change values (or starting style), which of the six start conditions failed, and the spec step ([GO 5.1][GO]) |
| A2 | Which animation wins this property? | The effect stack at time *t*, with the winner, replaced animations and the cascade origin it entered ([GO 5.3][GO]) |
| A3 | Why is this not off the main thread? | The class, the rule (group, cap, record bound, late-resolved value, held class), and the cost per frame ([GO 5.4][GO]) |
| A4 | Why did this animation start late, or jump? | Its creation cause (mutation, script stack, navigation), the commit frame, the ready time, and the frames in between |
| A5 | What does this animation cost? | Frames requested, main-thread and raster time, wakeups, the renderer's rate vote, invisible-skip status ([GO 5.9][GO]) |
| A6 | What does reduced motion change here? | A replay at each motion level, diffed as values; WCAG 2.2.2 and 2.3.3 candidates (moving content over five seconds without a pause control; parallax-like scroll-linked transforms) ([GO 5.10][GO]) |
| A7 | Why was this view transition skipped? | The failed condition (duplicate name, size change, limit, timeout, eligibility, isolation mismatch, an effect a compositor-held snapshot cannot take) and the captured and matched names ([GO 5.15][GO]) |
| A8 | Which scroll range drives this? | The timeline, its scroller, its range and the current progress, drawn on the scroller ([GO 5.15][GO]) |

**Tools.** The DevTools animation view starts from the recording, not from when the panel opened ([GO 5.5][GO]), covers rAF-driven motion through the style writes its callbacks make, and draws CSS, script, SMIL, scroll-driven and View Transition animations on one timeline. **Automation**: Izcal offers WebDriver BiDi an `animation` module (list running effects, finish all finite animations, seek every timeline of a navigable to *t* under virtual time, set the motion level, capture a screenshot at timeline time *t*) as proposals, and translates the CDP Animation domain in the DevTools spec's compatibility profile ([GO 3.6, 5.13][GO]). With exact time and canonical pixels (GR-D3), a screenshot of any animation at any timeline time is identical on every OS and ISA, so tests need not turn animation off ([DR 9][DR]). **The settled signal**: the DOM spec's settled signal ([DM 6.15][DM]) gains its animation clause. In simulation and replay: no running finite effect targets subtree S, no pending style change can start a transition there, and T is no earlier than the latest finite end time. In production, as a statement of fact: the running finite effects that target S, each with its end time, and the running infinite effects, listed, not waited for. Agents get it only with AI on and consent (B9, B10); the agent-facing parts compile only with feature `ai` (Stage 1 G4) ([GO 5.12][GO]).

### 6.14 Security, privacy and robustness (AN-D13)

**Memory safety and re-entrancy.** All crates of this spec are `#![forbid(unsafe_code)]`. The animation graph (timelines, animations, effects, SMIL intervals and fragment timelines, View Transition state) lives in one **arena per document** addressed by **generational handles**; JS wrappers hold handles, never owning pointers; a stale handle fails closed (G131). The walk runs no script (G130, 6.3), and settlement re-validates every change-list entry. Frame-demand and timeline registrations are sets (6.11). These close the lifetime classes behind 35 of the 43 SMIL, CSS and Web Animations and View Transitions CVEs of the security note's tally, including the three exploited in the wild ([SR 2.8, 3.1–3.3][SR]); `hg-anim-fuzz` drives `then` getters, `cancel()`, `remove()`, `persist()`, element removal and GC from inside every callback and promise reaction, and is a top-tier fuzz target with the JS engine's ([SR 3][SR]).

**Published limits** (the specifications set none, [SR 7.7][SR]); each is tested at the limit minus one, the limit and above (AN-52):

| Limit | Value | Over the limit |
|---|---|---|
| Live animations per document (including persisted replaced ones) | 65,536† | New animations are created idle with a console message naming the limit; CSS animations are not created |
| Keyframes per effect; `linear()` stops | 10,000†; 10,000† | `TypeError` from `setKeyframes()` or `animate()`; invalid declaration for CSS |
| Hand-off record: keyframes; transform functions per keyframe; encoded bytes; compositor bytes per renderer key | 64†; 32†; 4 KiB†; 256 KiB† | Demoted to the main thread (6.5 rule 6) |
| SMIL instance times per element; intervals created per document per frame | 1,024†; 4,096† | Further instance times dropped in document order; further intervals deferred to the next frame, in order |
| SMIL dependency edges per document | 65,536† | The animation element that would add the edge is inert |
| Named elements per View Transition | 1,024† | Transition skipped (`InvalidStateError`) |
| Style re-runs for stale timelines per frame | 1 | Spec rule (6.6) |

Infinite iteration counts and durations are legal and handled by the exact time types ([SR 7.1–7.2][SR]); the iteration index saturates and never wraps. Event volume is bounded by the event model: CSS Animations 2 fires one `animationiteration` when the current iteration has changed since the previous animation frame ([SP 7.9][SP]; [FK][FK]), and the animation limit bounds the rest.

**Timing channels.** The frame clock is the timer most rendering attacks use, and coarsening cannot remove a frame boundary ([SR 4.1–4.8][SR]). Izcal's defences are those of the specs it builds on: render time must not depend on secret state (the graphics spec's GR-D18 rules for cross-origin surfaces and filters; the style spec's visited-link rules, which animations cannot bypass because they interpolate only computed values visible to script); out-of-process frames run on their own schedule and the compositor's fixed per-surface slices bound cross-site frame timing (GR 6.9); each renderer's cadence depends only on its own votes (6.11); every document's frame time derives from one recorded tick (6.3). A view timeline on an `<iframe>` element reports the frame box's position in the embedder, which the embedder already knows ([SR 4.12][SR]).

**View Transitions**: Section 6.7's eligibility in the broker, snapshot placement, effect rule for compositor-held snapshots, no readback (G136, G137).

**Privacy.** The exposed bits: `prefers-reduced-motion` (one bit, exposed under R0 as the accessibility spec exposes it); each renderer's presentation rate (the display model's bits plus the page's own content, GR-D15); and, only for users who opt in, the observable effects of levels S and X and of the rAF throttle, which script can detect through computed values, pause states and frame cadence, so they are opt-in and never inferred ([MA 5.5][MA]). CSS Transitions itself names this trade-off ([SR 6.2][SR]). In D-S1's opt-in uniformity profile, the media query reports `no-preference` uniformly and every UA behaviour of level R follows the reported value (6.10). PRIV-2's accounting lists each of these (9.5).

**UI integrity.** Content animations cannot overlap browser UI, and shell prompts measure their input delay from when they are fully shown and unobscured (security scope CON-R rules; GR-D28), which closes the prompt-timing attacks that used animations ([SR T29–T30][SR]).

### 6.15 Memory and footprint

Animations, transitions and SMIL state live in **side tables keyed by node slot**, as FP 6.4 already lists for animations, so an element with none pays nothing. Because only animated elements pay, this state is a per-animation cost, not a per-node one: this spec asks the footprint spec for an **"animation state" category** in the page-content ledger, outside FP-11's per-node split, with these budgets† (9.5): ≤ **256 B** per animation (model state, timing, handle, composite-order key) plus its keyframes; ≤ **96 B** per running or completed transition record; ≤ **4 KiB** per hand-off record and ≤ **256 KiB** per renderer key in the compositor, charged to that key in the GPU ledger (6.5). **Before-change styles stay style's** primary state inside ST-30's 80 B (ST 6.11), so nothing is counted twice. Interned keyframe sets are shared by content hash (ST-D7's content-addressed identities), so a thousand elements with one `@keyframes` share one keyframe set. Nothing in this spec is a derived cache that needs discarding; View Transition snapshots are on GR-D23's own FP-14 line and freed within one frame of the transition's end (AN-41).

### 6.16 Crate layout and rules

| Layer | Crate | Role | Tier |
|---|---|---|---|
| L2 (`hg-types`) | time module | `AnimTime` (i128 ns, saturating), shared with the SVG spec's parsers and the frame clock; owned by this spec | P |
| L2g | `hg-anim-time` | Q32 progress, timing calculations (phases, iterations, directed and transformed progress), easing functions (exact cubic-Bézier with its 256-bit helper, `steps()`, `linear()`), double conversions; integer only except boundary conversions (G61) | P (compositor, content GPU process, media process) |
| L2g | `hg-interp` | Fixed-point interpolation, addition and accumulation of transform lists and matrices (decomposition, slerp, recomposition), correctly rounded fixed-point trigonometry in degrees (also used for static transform functions, 6.2), integer square root, opacity, filter amounts; **the only interpolation crate the compositor links**; owned by this spec (was written by the graphics track, GR 6.24) | P |
| L2g | `hg-interp-geom` | Fixed-point interpolation of basic shapes and `path()` geometry for `clip-path`, over `hg-path`; linked by the renderer and the content GPU process, **never the compositor** (G63) | P |
| L4 | `hg-anim` | The model (6.1), CSS transitions and animations with the sequential post-pass (6.4), SMIL timing and fragment timelines (6.8), scroll and view timelines (6.6), the change list, hand-off records (6.5), frame demand (6.11), motion levels (6.10); implements style's `AnimationSource` and SVG's `SmilControl`; a **scriptless core** with no dependency on `hg-dom`, `hg-heap`, `hg-js-host` or `hg-script` (SVG G120), reaching the event loop only through `hg-script-api` | R |
| L4 | `hg-anim-values` | Per-property interpolate, add and accumulate over style's computed values, generated from the property descriptions' animation-type column (the generator belongs to style's overlay toolchain), plus hand-written "see prose" types and motion-path sampling through the SVG spec's path functions | R |
| L4 | `hg-view-transition` | View Transition state machine, capture orchestration, pseudo-tree UA rules, the renderer side of cross-document transitions | R |
| L4 | `hg-anim-api` | Script-facing objects behind the DOM spec's generated bindings (G90): `Animation`, `KeyframeEffect`, `AnimationTimeline` and its subclasses, `ViewTransition`, `CSSAnimation`, `CSSTransition`, `getAnimations()`, animation events | R |
| L4 | `hg-anim-observe` | Decision records, counters, IIP animation domain types (with the DevTools track); agent-facing parts behind feature `ai` | R |
| L5 | `hg-navigation` (DOM spec's crate) | Gains cross-document View Transition eligibility and snapshot capability re-grant, written to this spec's requirements | P (broker) |
| tools | `hg-anim-reference` | Spec-literal reference model in exact rationals (`num-rational`, MIT OR Apache-2.0, tools only), with MPFR through `rug` (LGPL-3.0-or-later, test only) as the oracle for trigonometry and square root; shares no code with `hg-anim`, `hg-anim-time` or `hg-interp` (G139) | T |
| tools | `hg-anim-fuzz`, `hg-wpt-anim`, `hg-anim-isa` | Re-entrancy and SMIL fuzzers on the security scope's fuzzing infrastructure; WPT runner and black-box observation harness; cross-ISA corpus hashing on x86-64, arm64 and riscv64 | T |

`hg-anim-events` (DOM spec, interim) is removed at AN-M1. Rules, continuing the registry (this spec's range is **G130–G139**; the SVG spec has G120–G129):

| Rule | Statement | Why |
|---|---|---|
| **G130** | No crate of this spec can name `ScriptToken` or call an API that runs script; `hg-anim::update` returns a change list, and promise settlement (with re-validation of each entry), the microtask checkpoint and event dispatch happen in `hg-script` after the walk returns | The in-the-wild exploits freed the animation graph from script during a tick ([SR 2.1–2.2][SR]) |
| **G131** | Timelines, animations, effects, SMIL intervals and fragment timelines and View Transition state live in a per-document arena addressed by generational handles; JS wrappers and other crates hold handles only; a stale handle is a no-op plus a debug assertion; registrations with the frame-demand registry and timelines are sets keyed by owner; arena allocation and transition generations happen only in the sequential post-pass and the animation step, never in a style worker | Lifetime bugs were 35 of the 43 SMIL, CSS and Web Animations and View Transitions CVEs ([SR 2.8][SR]); phase purity (Stage 1 3.3) |
| **G132** | Animation time is held only in `AnimTime`; `f32` and `f64` never hold time except at the IDL boundary; no time is accumulated across frames (current times are always computed from start time, timeline time and rate) | Exactness and replay (6.2; [DR 4.7–4.8][DR]) |
| **G133** | Interpolation runs only through `hg-interp`, `hg-interp-geom`, `hg-anim-values` and the SVG spec's `hg-svg-interp`; transcendental functions only through `hg-math` (floating point) or `hg-interp`'s fixed-point functions; no FMA except explicit `mul_add`, no `algebraic_*` operations, no SIMD float reductions, no estimate instructions; the floating-point control register is asserted at each frame start in the renderer, the content GPU process and the compositor | Cross-ISA identity ([DR 1–2][DR]) |
| **G134** | The compositor links `hg-anim-time` and `hg-interp` and no other crate of this spec, and never `hg-path`; the compositor and the content GPU process sample only validated, bounded hand-off records; they receive no style values, run no style code, and never send sampled values back as model inputs; eligibility is decided once per record version | One function, one time; G63 and GR-18; the two-engines failure ([HP 6.8][HP]; [CF 2.2][CF]) |
| **G135** | Every user-agent-defined choice of this spec (ready, style change event points, the settlement order, stale re-runs, demotions, frame demand, votes, timeouts, motion levels, the isolation-state rule) is a function of recorded inputs, reads clocks only through the substrate, and carries its spec anchor or an "Izcal choice" tag with the issue it was filed under | B1, JS-D15, ANC-1 |
| **G136** | Cross-document View Transition eligibility and the re-grant of snapshot capabilities are decided only by `hg-navigation` in the broker; a renderer's opt-in or capture is a request | A compromised renderer must not decide a cross-origin question (P1; DM G94's principle) |
| **G137** | No API in this spec or the graphics spec converts a View Transition snapshot capability into readable pixels; a snapshot containing a cross-origin surface is a cross-origin surface for GR-D18 | View Transitions 1: the data "can not be accessed by the Document", made a mechanism ([SR 5.1][SR]) |
| **G138** | No crate of this spec recurses on content-controlled depth (SMIL dependency graphs, timeline-scope lookup, effect stacks, keyframe lists); work-lists with the published budgets of 6.14 | DoS and stack exhaustion ([SR 7.5–7.6][SR]) |
| **G139** | `hg-anim-reference` shares no project-local code with `hg-anim`, `hg-anim-time`, `hg-interp`, `hg-interp-geom` or `hg-anim-values`; it shares only IDL types, `hg-math`, the style read API and the test-only MPFR oracle; CI compares them on WPT, the corpus and generated timing cases; a licence lint fails the build if an LGPL crate is reachable from any shipped binary | An independent oracle, as LY G41; the test-only licence exception enforced |

### 6.17 Interfaces with other specs

| Interface | Counterpart | Contract |
|---|---|---|
| Animation step and rendering order | DOM (`hg-script::rendering`) | `hg-anim::update(doc, F) → change list`; `hg-script` re-validates, settles, checkpoints, sorts, dispatches (6.3); one tick per frame, F per document, unjittered, with `performance.now() ≥ F` (amendment to DM-D10) |
| Listener facts | DOM (`hg-dom`) | An always-on, constant-time summary per event type of whether a listener exists on a node's event path, including `Document` and `Window` (6.5) |
| Cascade inputs | Style | `AnimationSource` trait; base value; compute against element; Animation and Transition origin blocks from the effect-stack snapshot; transition candidates from workers; the sequential post-pass; `display` ignoring animation origins; animation-type and motion-inducing columns in the property overlay; static transform trigonometry through `hg-interp` (6.2, 6.4, 9.5) |
| Scroll and view timelines | Layout (LY-D13, LY 6.19) | Offsets after the scroll step; scroll ranges and view-timeline subject geometry per layout; the stale-timeline re-run as one more pass of the style and layout loop (6.6) |
| Hand-off records | Graphics (GR 6.28, GR-D13, G63, GR-18) | 6.5's bounded record; the sample-time rule (6.3); class G parameter substitution in the content GPU process; per-frame reports to tools only |
| View Transition snapshots | Graphics (GR 6.8, 6.28, GR-D23) | Capture requests by element; capability ids; compositor capture for cross-origin surfaces with the effect rule of 6.7; budgets on the View Transition line |
| Cross-document eligibility | DOM (`hg-navigation`, broker) | G136; same origin, redirect rule, navigation type, both opted in, same isolation state |
| Presentation rate | Graphics (GR-D15) | Per-renderer decimation from its own votes; low-power cap with input exemption; Hardened 60 Hz ceiling (6.11) |
| Preferences | Accessibility (`hg-ax-prefs`), style's environment snapshot | Motion level per site; R0 exposure; one exposed value (6.10) |
| SMIL | SVG spec (SV-D4, R10) | Timing, fragment timelines and `SmilControl` here; elements, attributes, `hg-svg-interp` through `InterpolateValue`, the animated-attribute overlay, paced offsets, motion-path geometry there (6.8) |
| SVG images and animated images | Media (MD-D19, MD-D20) | Scriptless `hg-anim` in the media process; ticks from the embedder; level X pauses images |
| Maths | JS (`hg-math`) | Identical functions on every ISA; correctly rounded and degree variants requested (6.2) |
| Observability | DevTools (DT-D2, DT-D3, IIP) | Decision records and A1–A8 (6.13) |
| Settled signal | DOM (6.15) | The animation clause (6.13) |
| Energy and memory | Footprint (FP-D4, FP-D15, FP 6.4, 6.6, 6.13) | Frame demand, invisible skip, battery cap; the "animation state" ledger category; compositor record bytes per key |

### 6.18 Alternatives

| Decision | Alternative | Why not (now) | What would reverse it |
|---|---|---|---|
| AN-D1 one core | Separate engines per front end | WebKit's late unification ([HP 7.5][HP]) | — |
| AN-D2 exact integer time | (b) IEEE f64 time as specified literally, deterministic on every ISA | Boundaries decided by rounding (6–14% of exact step boundaries in one experiment, [DR 4.8][DR]); still deterministic, so this is the fallback | AN-S1 shows exact time costs > 5%† of the animation step on the corpus and no conformance or interop gain |
| AN-D2 fixed-point geometry | (b) f64 everywhere with `hg-math`, amending G61 | Identity would still hold, but G61 (accepted) and the compositor's integer composition would need a second conversion rule | AN-S1 shows fixed-point decomposition or trigonometry fails WPT tolerances or costs > 2×† the f64 path |
| AN-D3 unjittered frame time | (b) jittered per realm; (c) jittered per agent cluster with the offset sent in every record | (b) breaks the one-function rule; (c) adds per-record state for no defensive value | A published attack that uses frame-time edges against Izcal's grid |
| AN-D4 G′ and U behind a flag | Ship now; or never | Chrome's two trials conflict and published no figure ([HP 6.6][HP]; [CF 1.7][CF]) | AN-S2: ≥ 10%† fewer dropped frames on the corpus under main-thread load at ≤ 1%† memory ships them at AN-M3 |
| AN-D4 motion path on the main thread | A pre-flattened transform table in the compositor | Changes sampled values unless the main thread uses it too; G63 | AN-S2 shows motion-path jank on the corpus and a table whose error is below 1/64 px† |
| AN-D5 renderer-resolved ready | Compositor resolves the start time (on first draw) | Puts a compositor timing decision into the model, needing a new decision point | AN-33 worse than the best engine |
| AN-D6 SMIL timing here | SVG owns SMIL entirely (its draft 0.1); or no SMIL | Timing must share the model; the SVG spec's draft 0.2 agreed; dropping SMIL failed once ([HP 2.10][HP]) | — |
| AN-D7 eligibility in the broker | In the renderer | A compromised renderer could claim same-origin | — |
| AN-D10 levels | Expose the bit only (every browser today) | UAAG 2.0 2.10.5; nobody controls author motion ([MA 3.7][MA]) | AN-70 fails for S after fixes: S becomes per-site only, never suggested |
| AN-D11 votes per renderer | Display-wide rate from all votes | One site would set another's rAF cadence | — |

### 6.19 Representation choices, derived (M-1 part 3)

| Choice | Derived from |
|---|---|
| i128 nanoseconds, exact 256-bit products, one rounding | Web Animations' precision clause, the f32 and f64 drift arithmetic ([DR 4.1–4.9][DR]); WebCodecs' integer timestamps and Temporal's nanoseconds as public precedents ([DR 4.6][DR]) |
| Q32 progress and integer bisection for cubic-Bézier, with a curve-dependent step count | CSS Easing's monotone x, unbounded y and open solver ([DR 3.1–3.2][DR]); first-principles error bound ([DR 5.4–5.5][DR]; [FK][FK]) |
| Fixed-point geometry with correctly rounded degree trigonometry, exhaustively checked | G61; Transforms' own algorithms; numerical analysis in published papers ([DR 6.1–6.6][DR]) |
| The change list, settlement re-validation and arena | Published CVE prose and exploit analyses ([SR 2][SR]) |
| The off-main-thread classes and record bounds | Engines' published prose and field results ([CF 1–3, 8][CF]); Izcal's process model (G63, GR-18, FP-1) |
| Frame demand, swept bounds and per-renderer votes | Android's published adaptive-refresh model ([CF 4.12][CF]); first principles |
| Every limit marked † | First principles; black-box measurement of shipping engines is a check, never a source (Q-AN-8) |

No value in this spec was taken from any project's source code; where only source would state a value (solver tolerances, engine caps, timeouts), it is a black-box question in Section 11.

## 7. M-1 compliance

**Dependencies used unmodified (shipped).** None new. The crates of this spec depend on Rust `std` and on Izcal crates (`hg-types`, `hg-math`, `hg-path`, `hg-display-list`, the style read API, `hg-tree-traits`, `hg-script-api`, the SVG spec's `hg-svg-interp` and `hg-svg-geom`, the IPC generator), every one under Izcal's own MIT licence. **Tools and tests only**: `num-rational` 0.4 (MIT OR Apache-2.0) in `hg-anim-reference`; `rug` (LGPL-3.0-or-later, MPFR bindings) as a test-only oracle for `hg-math` and the fixed-point trigonometry, never linked into a shipped binary, as the licence exception for test-only inputs allows ([DR 2.7][DR]), enforced by G139's licence lint; CORE-MATH's published worst-case lists as test inputs (test-only whatever their licence); WPT (BSD-3-Clause) run as a black-box oracle; the fuzzing engines of the security scope's fuzzing infrastructure, under that scope's licence review.

**Built new.** Everything in Section 6.16: the timing core, the exact time and easing types, the fixed-point interpolation kernels, the per-property interpolation code, SMIL timing, scroll and view timelines, View Transitions, the hand-off records, frame demand, motion levels, decision records, the reference model and the fuzzers.

**Evidence rule.** No source code of any project was read or cited for this spec or its notes (HGBA-2 2.3.3): no code files, headers, commits, diffs or code reviews of any browser, engine, library, tool or OS component. The researchers discarded search results that pointed at engine commits, cited two Mozilla bugs by title only because their threads link to code reviews, skipped a slide quoting browser code, and read CORE-MATH's project page and papers but none of its code ([SR][SR] and [DR][DR] evidence notes; [HP][HP] M-1 note). Phase 8 found no link to any project's source code ([FK 4][FK]; [CR E1–E6][CR]) and three items needing care, all fixed in the notes: two tolerance constants that had been read from WPT test-harness files are removed (the spec now uses WPT only black-box, pass or fail); a patch title quoted from a Mozilla bug is replaced by the bug's prose comments and milestone; and a sentence that rested on a security firm's reading of a source diff is removed. The cubic-Bézier lineage is known only from a library's prose description of engines, and the method Izcal uses (integer bisection) is derived from the specification's monotonicity, not from it ([DR 5.2–5.5][DR]). Values that only source states are black-box questions (Section 11).

**Anchors** (ANC-1, G9, G135):

| Part | Anchored to |
|---|---|
| Timing model, effect stack, events, `getAnimations()` | [Web Animations 1 ED](https://drafts.csswg.org/web-animations-1/) at the revision pinned at AN-M0; [Web Animations 2](https://drafts.csswg.org/web-animations-2/) for admitted and internal parts |
| Easing | [CSS Easing 1](https://drafts.csswg.org/css-easing-1/) and [2](https://drafts.csswg.org/css-easing-2/) |
| Transitions, animations, cascade origins | [CSS Transitions 1](https://drafts.csswg.org/css-transitions-1/) and [2](https://drafts.csswg.org/css-transitions-2/); [CSS Animations 1](https://drafts.csswg.org/css-animations-1/) and [2](https://drafts.csswg.org/css-animations-2/); [CSS Cascade 5](https://drafts.csswg.org/css-cascade-5/) |
| Interpolation | [CSS Values 4](https://drafts.csswg.org/css-values-4/#combining-values); [CSS Transforms 1](https://drafts.csswg.org/css-transforms-1/) and [2](https://drafts.csswg.org/css-transforms-2/); [CSS Color 4](https://drafts.csswg.org/css-color-4/#interpolation); [Properties and Values API 1](https://drafts.css-houdini.org/css-properties-values-api-1/); CSS Display 4, Contain 2, Position 4 for discrete flips |
| Scroll and view timelines | [Scroll-driven Animations 1](https://drafts.csswg.org/scroll-animations-1/), with the CSSWG resolutions of 27 May and 3 June 2026 on [#9256](https://github.com/w3c/csswg-drafts/issues/9256) and [#13817](https://github.com/w3c/csswg-drafts/issues/13817) |
| View Transitions | [CSS View Transitions 1](https://drafts.csswg.org/css-view-transitions-1/) and [2](https://drafts.csswg.org/css-view-transitions-2/); HTML's [update the rendering](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering) and [reveal](https://html.spec.whatwg.org/multipage/browsing-the-web.html#reveal) |
| Frame timing | HTML's [event loop processing model](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model); [High Resolution Time](https://w3c.github.io/hr-time/) |
| SMIL | [SVG Animations 2 ED](https://svgwg.org/specs/animations/); [SMIL Animation](https://www.w3.org/TR/smil-animation/); [SMIL 3.0 Timing](https://www.w3.org/TR/SMIL3/smil-timing.html) for cycles |
| Reduced motion | [Media Queries 5](https://drafts.csswg.org/mediaqueries-5/); [WCAG 2.2](https://www.w3.org/TR/WCAG22/) 2.2.2, 2.3.1, 2.3.3 with its errata; [UAAG 2.0](https://www.w3.org/TR/UAAG20/) (a W3C Note) 2.10.5 |
| Izcal choices with no anchor | Tagged "Izcal choice" with the CSSWG or WHATWG issue each is filed under (G135): style change event points, the settlement order, the ready rule, the stale-timeline subject rule, the provisional SMIL composite position, the isolation-state rule, motion levels, limits |

## 8. Targets

† marks thresholds set by this document from first principles; none is measured on Izcal yet. "Best shipping engine" is the best result among the stable releases of Chrome, Firefox and (on macOS) Safari on the same reference device and day, as in the other specs; **B is fixed** at each Izcal release's branch date. Speed is compared black-box. **Statistics**: a timing comparison uses at least 30† runs per browser, reports p50 and p95 with 95% confidence intervals, and counts a tie when the intervals overlap. **riscv64**: no shipping engine runs on riscv64, so every relative performance target has a riscv64 form: Izcal on the riscv64 reference boards (the JavaScript spec's RVA23U64 profile, JS 6.12) is compared with Izcal on the arm64 reference device, normalized by the ratio of a fixed published CPU benchmark on the two machines, and must be within 1.25×†; every count, determinism, security and memory target applies unchanged on all three ISAs. Correctness, determinism, security and memory targets bind from AN-M1 for the parts built by then; performance and energy targets bind at Stage 1 exit.

### 8.1 Correctness

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **AN-1** | **Animation conformance** | Stage 1 exit, per WPT directory, floor **max(F, B)**: `web-animations`, `css/css-transitions`, `css/css-animations`, `css/css-easing` **97%†**; `scroll-animations`, `css/css-view-transitions` **92%†**; `svg/animations` **90%†**; the animation subtests of `css/css-transforms`, `css/css-color`, `css/motion`, `css/css-properties-values-api` **95%†**; held features excluded and listed; tests that assert text the CSSWG has since resolved otherwise are listed with the resolution; then C-1's non-decreasing floors | `hg-wpt-anim` in T1 | C-1's failure action |
| **AN-2** | **Interop focus areas** | Scroll-driven Animations and View Transitions (Interop 2026) at or above the best shipping engine at Stage 1 exit (ties within one percentage point) | Interop dashboard subsets | Fix before any new admission in the area |
| **AN-3** | **Reference model agrees** | `hg-anim` versus `hg-anim-reference` on WPT, the corpus and 10⁷† generated timing cases (phases, iterations, boundaries, huge and tiny playback rates and exponents, easing curves with large y values, events and their order, infinities): **0** divergences in timing and events; fixed-point values equal to the reference rounded once | CI on every change | Block the change |
| **AN-4** | **Implementation-defined choices observed** | Every black-box question of Section 11 answered for all three engines and recorded before AN-M2 | `hg-wpt-anim` observation harness | AN-M2 review holds |
| **AN-5** | **Event invariants** | 100% of fuzz and WPT runs: every `transitionrun`/`transitionstart` ends in exactly one `transitionend` or `transitioncancel`; every animation's events follow the event tables of Animations 2 and Transitions 2, including when a `then` getter cancels a later change-list entry's animation | `hg-anim-fuzz`, assertions | Block the change |
| **AN-6** | **What is drawn equals what script reads** | For every frame of the CI corpus with off-main-thread animations: presented values (compositor and content GPU process) equal the main-thread function's values **given the same (F, scroll offset)** recorded for that sample, **bit for bit, 100%** | Recomputation from frame records in DST and the device lab | Disable the class for the failing property until fixed |
| **AN-7** | **Static equals animated** | Every transform and opacity animation paused at its end state, and every SMIL and CSS motion-path animation at its end, equals the static style of that end state with zero fuzz (100%) | Reftests on the canonical CPU path | Fix the shared function |

### 8.2 Determinism

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **AN-20** | **Cross-ISA identity** | An animation corpus sampled at fixed timeline times (steps boundaries, near-parallel slerp, near-singular matrices, hue wrap, out-of-range easing, infinite durations, SMIL syncbase chains): the hash of computed values, `hg-interp` outputs and event logs is **identical on x86-64, arm64 and riscv64** on every CI change | `hg-anim-isa` on three ISAs | Block the change |
| **AN-21** | **Replay** | DET-1: animation outputs (timeline times, events, computed values, hand-off records) identical in **1,000 of 1,000** replays | DST farm | Fix the unrecorded input |
| **AN-22** | **Maths** | `hg-interp`'s degree trigonometry correctly rounded over its whole reduced domain (exhaustive against MPFR, 6.2); every `hg-math` function used here identical on the three ISAs on 10⁷† inputs plus published worst cases, and equal to MPFR if the correct-rounding amendment is accepted | CI with `rug` (test only) | Block the change |
| **AN-23** | **Recording is cheap** | This spec's share of the whole-renderer recording budget (≤ 1%† of p95 frame time, shared) reported per release | Joint measurement ([RN2 Q15][RN2]) | Reduce the recorded data |
| **AN-24** | **Parallel style changes nothing** | Transitions starting on 10,000 elements, and CSS animations created on them, with 1, 2, 4 and 8 style workers: identical generations, arena handles, events and hashes (100%) | DST | Fix the post-pass boundary |

### 8.3 Performance and energy

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **AN-30** | **Idle is idle** | A page whose only animation is scroll-driven and not scrolling, or whose only animations are offscreen (by swept bounds), hidden or in skipped content: **0** BeginFrames requested and **0** renderer wakeups over **60 s†**; and, as correctness cases, a slide-in from offscreen arrives on time, an animated `visibility` shows, and a fade-in under an ancestor animating from opacity 0 shows (100%) | Substrate counters in DST; OS counters in the lab | Fix frame demand |
| **AN-31** | **Compositor-only animations do not wake the main thread** | An infinite class C animation with no event listeners: **0** main-thread wakeups per **minute†** after start; with a `window` listener added late, events dispatched in a rendering update at their scheduled frame (100%) | Substrate counters; DST | Fix event scheduling |
| **AN-32** | **Smooth while the page works** | Class C and G animations with the main thread blocked for **500 ms†**: **0** frames missing the animation update at the renderer's rate on every reference device | Frame records (GR 6.9) | Investigate the compositor and content GPU process schedule |
| **AN-33** | **Start latency** | From the task that creates an animation to the first presented frame showing it: p50 and p95 no worse than the best shipping engine (black-box, Q-AN-11) | Lab capture, statistics as stated | Revisit the ready rule (AN-D5's reversal) |
| **AN-34** | **Animation step and style cost** | On an animation micro-suite (1,000 class L animations; 10,000 class C animations; 1,000 SMIL elements; 200 transitions starting per frame): p95 time per frame ≤ **1.25×†** the best shipping engine at Stage 1 exit, ≤ 1.0× at parity | Black-box suite | Profile; representation work |
| **AN-35** | **Energy table** | Per platform, published per release: idle; a hidden spinner and an offscreen spinner within the FP-21 noise band of idle (upper 95% CI of the difference ≤ **2%†**); a compositor spinner and a painting spinner ≤ the best shipping engine; a `steps(1)` caret blinking at 1 s ≤ **2.2†** wakeups per second | FP-21's method, platform meters | Fix the frame-demand path at fault |
| **AN-36** | **Battery rule** | Under the OS low-power signal or FP-D15's limit, each renderer's rate ≤ **30 Hz†** except user scrolling and input-driven updates; each change logged in FP's intervention log (100%) | Lab | Fix the vote path |
| **AN-37** | **Cadence is per renderer** | Two cross-site documents on one display, one with a High-voting animation: the other's rAF cadence is unchanged (100%) | Lab and DST | Fix decimation |

### 8.4 Memory

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **AN-40** | **Animation state** | ≤ **256 B†** per animation plus keyframes; ≤ **96 B†** per transition record; zero bytes for elements with no animation; in the "animation state" ledger category on the corpus | Footprint ledger | Representation work |
| **AN-41** | **Snapshots** | View Transition snapshots within GR-D23's line; freed within one frame of the end, skip or timeout in **100%** of runs | GPU ledger | Fix lifetime |
| **AN-42** | **Compositor animation bytes** | ≤ **4 KiB†** per hand-off record and ≤ **256 KiB†** per renderer key, charged to the key; over a cap the animation is demoted (100%) | GPU ledger; DoS suite | Fix the cap |

### 8.5 Security and privacy

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **AN-50** | **No unsafe** | 0 `unsafe` in this spec's crates (`forbid`) | Compiler | — |
| **AN-51** | **No re-entrancy** | `hg-anim-fuzz` (promise `then` getters, `cancel()`, `remove()`, `persist()`, element removal and GC from every callback): 0 panics and 0 sanitizer reports over the security scope's top-tier fuzzing hours | Continuous fuzzing | Fix; a regression test per finding |
| **AN-52** | **Bounded work** | Every limit of 6.14 at the limit minus one, the limit and above (**100%**); a DoS suite (10⁶ animations, 10⁴ keyframes, records at every cap, SMIL open cycles, 1 ns iterations, infinite durations, 10³ named view-transition elements) completes each case within **10×†** a benign case of the same size | CI | Bound the offending path |
| **AN-53** | **Cross-document snapshots never cross** | Probes (cross-origin navigation, cross-origin redirect, isolation mismatch, browser-UI initiation, a compromised renderer claiming eligibility or asking for another key's capability, a forbidden effect on a compositor-held snapshot): **100%** refused or skipped; an audit finds no API that reads snapshot pixels | Security suite; review | Block release |
| **AN-54** | **One tick per frame** | Every document of an event loop derives its rAF argument and timeline time from the same recorded tick (equal after adding each document's time origin, up to its grid); the off-main-thread sample uses the same tick; `performance.now() ≥ F` in every rAF callback (**100%**) | DST assertions | Fix |
| **AN-55** | **Exposure** | Under R0 the media query reports the user's preference in the Standard and Hardened profiles; only the opt-in uniformity profile reports `no-preference`, uniformly; S, X and the rAF throttle are listed in PRIV-2's accounting with their observable effects; every UA motion behaviour follows the reported value, verified by the Firefox smooth-scroll leak test ([SR 6.4][SR]) | Privacy suite | Fix the leaking behaviour |

### 8.6 Users and observability

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **AN-70** | **Level S breaks little** | On a 1,000-page† corpus (forms, carousels, menus, scroll-linked sites), pages with content left invisible or a blocked flow under S: ≤ **1%†** | AN-S3 corpus with manual review | S stays per-site only and is never suggested |
| **AN-71** | **Level X stops motion** | On a motion suite (CSS, script, SMIL, View Transitions, scroll-driven, animated images, muted loops): no pixel change over 5 s without input in **100%** of cases, except rAF-driven canvas, which the throttle offer covers; after undo, `animation-play-state` changes from CSS take effect (100%) | Canonical-pixel diff | Fix |
| **AN-72** | **Why-queries are right** | A1–A8 on a planted suite: the planted cause named in **100%** | DST | Fix the record |
| **AN-73** | **No silent fallback** | Every demotion and every held-class animation carries its rule in the decision record (**100%**) | Assertions | Fix |
| **AN-74** | **Settled means settled** | In DST, after the settled signal fires for S at T, no animation-caused change in S before T (**100%**) | DST | Fix the clause |
| **AN-75** | **Records are cheap** | DevTools-attached decision records ≤ **1%†** of frame time on the animation corpus | Lab | Sample, then move work to replay |

## 9. Milestones and staffing

### 9.1 What is staged, and why

| Item ordered or held | Reason | Kind |
|---|---|---|
| The numeric contract, `hg-anim-time`, `hg-interp`'s kernels, the reference model and the hand-off record IDL come first (AN-M0) | The graphics track needs `hg-interp` and the record before GR-M1; oracles before code | Correctness |
| Off-main-thread classes G′ and U: built behind a flag by AN-M2 and shipped at AN-M3 if AN-S2 is positive | The evidence conflicts and no figure was published ([HP 6.6][HP]; [CF 1.7][CF]); the build is not delayed, only the default | Measurement (AN-S2) |
| Motion path off the main thread | Would change sampled values unless the main thread uses the same table | Correctness (P2), revisited with AN-S2 |
| Level S suggested to anyone | Breakage unmeasured ([MA 6.7][MA]) | Measurement (AN-S3) |
| Groups, the iteration-composite script API, custom effects, triggers, scoped and nested View Transitions, `interpolate-size` | Admission (AN-D9) | Correctness (compatibility) |
| Cross-document View Transitions after same-document | They share capture and the pseudo-tree; the broker path needs `hg-navigation` (DM-M2) | Build order, not a hold |

Nothing else is staged: the full model, CSS transitions and animations, scroll-driven animations, SMIL timing, both kinds of View Transition, off-main-thread classes C and G, motion levels R, S (as an opt-in) and X, flash protection, energy rules and observability are Stage 1 work.

### 9.2 Milestones

| ID | Milestone | Exit criterion (demo plus measurement) | Needed by |
|---|---|---|---|
| **AN-M0** | Foundations and the whole design (0–6 months†) | `AnimTime`, `hg-anim-time` (Q32, exact easing), `hg-interp` kernels with the fixed-point trigonometry proved exhaustively against MPFR, `hg-interp-geom`, the bounded hand-off record IDL agreed with the graphics owner through GR-18's review, `hg-anim-reference` for the timing model, `hg-anim-isa` running on three ISAs; the WA1 ED revision pinned; Q-AN-1, -2, -3, -6, -8, -11, -17 observed; AN-S1 measured on all three ISAs; the flash-protection patent search; **design review** of Section 6 with the style, layout, graphics, DOM, SVG and accessibility owners and a provenance review | **GR-M1** (the owner the ledger asked for, [A-FUT-4][LG]) |
| **AN-M1** | The model on the main thread (18–24 months†, at least 3 months after ST-M1 and DM-M1) | WA1 core, the change list with re-validation, CSS transitions and animations with the sequential post-pass, all interpolation types, `getAnimations()`, events; `hg-anim-events` removed; AN-1 measured for `web-animations`, `css-transitions`, `css-animations`, `css-easing`; AN-3, AN-5, AN-7, AN-20, AN-21, AN-24 green | DM-M2 |
| **AN-M2** | Off the main thread, scroll, SMIL, same-document transitions (30–36 months†) | Classes C and G with the graphics track (AN-6, AN-31, AN-32, AN-42); G′ and U built behind a flag; scroll and view timelines with LY-M2; SMIL timing with the SVG spec's SV-M2; same-document View Transitions; level R; frame demand and votes (AN-30, AN-35, AN-37 measured); AN-S2 and AN-S4 measured | LY-M2, GR-M2, SV-M2 |
| **AN-M3** | Stage 1 exit (48–60 months†) | Cross-document View Transitions (AN-53); levels S and X (AN-70, AN-71); flash protection; G′ and U on if AN-S2 was positive; decision records, A1–A8 and the IIP domain (AN-72 to AN-75); every Stage 1 threshold of Section 8 | Stage 1 exit |
| **AN-M4** | Stage 2 | Admitted Level 2 features within one release of meeting ST-D18 | Stage 2 |
| **AN-M5** | Parity | AN-34 at ≤ 1.0×; AN-2 held for two Interop years | Stage 3 |

**Spikes.** **AN-S1** (AN-M0, on all three ISAs): throughput of exact time, Q32 progress, integer cubic-Bézier and fixed-point decomposition and slerp against an f64 implementation of the same algorithms, per sample and per frame on the micro-suite. **AN-S2** (AN-M2): classes G′ and U (behind their flag) and motion path off the main thread versus on, on the corpus under main-thread load: dropped frames, memory, energy; it also revises class G's re-raster budget. **AN-S3** (AN-M3): level S breakage corpus. **AN-S4** (AN-M2): perceived smoothness and energy of Normal-voted animations at 60 Hz on 120 Hz panels. **AN-S5** (AN-M3): decision-record cost.

### 9.3 Staffing and duration

No primary source gives the size of an animation team. Indirect data: WebKit's unification took "well over 2 years" for an existing team ([HP 7.5][HP]); its threaded sampler took three years from filing to default ([CF 1.15][CF]); the WPT animation directories are large and churn with Interop ([GO 3.9][GO]). The rows are first-principles estimates† from each track's deliverables, every track staffed from AN-M0:

| Track | Scope | Engineers† |
|---|---|---|
| Timing core and API | `hg-anim` model, change list, events and promises, `getAnimations()`, `hg-anim-api` with the DOM track's bindings | 4–5 |
| CSS transitions and animations | Style change events, the sequential post-pass, transition sets, keyframe cascade with the style track, discrete and entry/exit rules | 3–4 |
| Numerics and interpolation | `AnimTime`, `hg-anim-time`, `hg-interp` and `hg-interp-geom` (fixed-point decomposition, slerp, trigonometry and its exhaustive proof), `hg-anim-values` and its generator columns, colour with `hg-math` | 4–5 |
| Off the main thread and energy | Hand-off records and caps, compositor and content GPU sampling with the graphics track, G′ and U behind the flag, frame demand, swept bounds, votes and decimation | 3–4 |
| Scroll-driven animations | Scroll and view timelines, ranges, stale re-run with the layout track | 2–3 |
| View Transitions | Same- and cross-document, capture with graphics, broker eligibility with the DOM track, pseudo-tree with style | 3–4 |
| SMIL timing | Interval engine, fragment timelines and `SmilControl`, dependency graph, composite mapping, attribute effects, media-process build, with the SVG spec (which counts the element and attribute layer) | 2–3 |
| Motion controls and accessibility | Levels, UA sheets, flash protection with graphics, breakage corpus, shell work with the shell track | 1–2 |
| Observability | Decision records, why-queries, IIP domain, BiDi proposals | 2–3 |
| Verification | `hg-anim-reference`, `hg-wpt-anim`, cross-ISA CI, black-box observations, fuzzers | 4–5 |
| Security | Fuzzing triage, limits, security review share | 1 |
| **Total at full strength** | | **29–39†** |

With management and test triage, about **34–45 people†**. **Hiring ramp†**: 12–15 at AN-M0 including every track lead; full strength by month 18, linearly. **Duration†**: AN-M0 at 6 months, AN-M1 at 18–24, AN-M2 at 30–36, AN-M3 with Stage 1 exit at **48–60 months**; this spec does not lengthen Stage 1 (layout and graphics set the critical path, [RN2 Section 4][RN2]), but AN-M0 must precede GR-M1. **Engineer-years†** from the ramp: year one averages about 18–23 and year two about 28–37, then 29–39 a year, so about 18 + 28 + 2 × 29 ≈ **104** to a 48-month exit and 23 + 37 + 3 × 39 ≈ **177** to a 60-month exit: about **105–175 engineer-years†** ([FK][FK]).

**Overlap, line by line, and one counting rule** ([CR C21][CR]): a deliverable is counted by the spec that owns its crate. Counted elsewhere, not here: the compositor's frame loop, layerization, the 16-node cap and the integration of sampled values into composition (graphics' compositor track); bindings generation (DOM); the property overlay's generator and the cascade origin slots (style); scroll offsets and ranges (layout); SVG elements, attributes, `hg-svg-interp`, the animated-attribute overlay and the SMIL DOM (SVG spec, whose SMIL rows say timing is this spec's); the DevTools frontend (DevTools); `hg-math`'s functions (JS engine; the correct-rounding amendment, if accepted, adds about 1–2† engineers there); the shell's motion settings UI (shell). The graphics track wrote `hg-interp`'s first version; that work, about 1† engineer, moves here at AN-M0, and the graphics total is reduced by the same amount (9.5). Stage 1 8.3 had no animation track; reconciliation 2 counted animation nowhere ([RN2 Section 4][RN2]). This spec adds **29–39 engineers†** to the running total, less the one moved from graphics.

### 9.4 Decisions for the owner

A bare "yes" means the recommendation. Build versus reuse is not on this list: the standing answer is "build our own" (M-1, D-S10). **The owner most needs to see AN-D2, AN-D3, AN-D4, AN-D7 and AN-D10.**

| ID | Decision | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|---|
| **AN-D1** | One timing core | (a) one Web Animations core with every front end generating its objects, group-ready, with internal iteration composite for SMIL; (b) separate subsystems | **(a)** (6.1) | — | — |
| **★ AN-D2** | Numeric contract | (a) i128 ns saturating time with exact boundaries and no floating-point path, floored Q32 progress, integer cubic-Bézier with a curve-dependent step count, fixed-point geometry in `hg-interp` with exhaustively proved degree trigonometry used for static and animated transforms alike, f64 strict for style's other types, `hg-math` correctly rounded (requested); (b) IEEE f64 throughout, literal spec arithmetic, identical across ISAs, boundaries by rounding; (c) (a) without the correct-rounding request | **(a)** (6.2) | AN-S1's measured cost; 1–2† engineers in the JS track for correct rounding | AN-S1: > 5%† of the animation step with no conformance gain (time), or fixed-point fails WPT tolerances or costs > 2×† (geometry): fall back to (b) for that part |
| **★ AN-D3** | Frame time | (a) one recorded tick per frame; each document's F relative to its own time origin, quantized to its grid without jitter, used by every timeline, rAF callback and off-main-thread sample; `performance.now() ≥ F`; dedicated workers derive F from the same tick; (b) jittered per realm (DM-D10 as written); (c) jittered per agent cluster with the offset in every record | **(a)** (6.3); amends DM-D10 | — | A published attack exploiting frame-time edges on the grid |
| **★ AN-D4** | Off the main thread | (a) classes C (compositor, `hg-interp` only) and G (content GPU process) on; G′ and U built behind a flag and shipped at AN-M3 if AN-S2 is positive; motion path on the main thread; group, hand-off-snapshot, one-time and bounded-record rules; recorded demotions; (b) transform and opacity only; (c) everything paint-only now | **(a)** (6.5) | Re-raster work in the content GPU process; GR-18 review of the record | AN-S2 negative: G′ and U stay off; AN-6 failure on a property: disable that property's class |
| **AN-D5** | Ready rule | (a) renderer releases pending tasks after the commit frame, ready time = its F; (b) compositor resolves the start time on first draw | **(a)** (6.3) | — | AN-33 worse than the best engine |
| **AN-D6** | SMIL ownership | (a) timing (intervals, fragment timelines, `SmilControl`, dependency graph) here; elements, attributes, value interpolation, overlay and geometry in the SVG spec (SV-D4, agreed); (b) SVG spec owns all of SMIL; (c) no SMIL | **(a)** (6.8) | 2–3† engineers here | — |
| **★ AN-D7** | View Transitions placement | (a) snapshots only in the content GPU process or compositor, never readable; compositor-held snapshots take only compositor effects, others skip; cross-document eligibility and capability re-grant in the broker; same isolation state required (Izcal choice); published limits; (b) eligibility in the renderer | **(a)** (6.7) | Broker work in `hg-navigation` | — |
| **AN-D8** | Scroll timelines | (a) the 2026 zero-range resolutions; spec stale rule plus resized view-timeline subjects, exactly one re-run; (b) the ED text as it stands | **(a)** (6.6); filed | — | The CSSWG resolves #14337 otherwise: follow it |
| **AN-D9** | Admission | As the table of 6.9 under ST-D18 | **As tabled** | — | An admitted spec changes incompatibly twice within a year: tighten |
| **★ AN-D10** | Motion controls | (a) one exposed value under R0; levels R (OS, with SV-D16 for SMIL), S (opt-in, discrete motion between keyframes with blur counted as motion), X (a UA pause flag, per page); flash protection on request; only the opt-in uniformity profile reports defaults; never remove an animation; (b) expose the media query only (every browser today); (c) (a) with S on by default under OS reduce-motion | **(a)** (6.10) | Shell UI; breakage corpus; patent search | AN-70 fails after fixes: S per-site only, never suggested |
| **AN-D11** | Energy | (a) frame demand with next-visible-change times, swept-bounds invisible skip under three conditions, Contain 2 for skipped content, listener-gated wake-ups that request a rendering opportunity, per-renderer votes, 30 Hz† on battery saver with input exempt, Hardened 60 Hz ceiling; (b) sample everything at the display rate | **(a)** (6.11) | — | AN-S4 shows perceptible regressions for Normal votes: raise Normal to the display rate |
| **AN-D12** | Observability | (a) decision records, A1–A8, recorded-session animation view, BiDi proposals, settled clause (agent parts behind `ai`); (b) live state only | **(a)** (6.13) | AN-75's budget | — |
| **AN-D13** | Limits and re-entrancy | (a) change list with re-validation, arena with generational handles, published limits including the compositor record caps, with deterministic fallbacks; (b) spec only, limits unpublished | **(a)** (6.14) | — | — |

### 9.5 Amendments this spec proposes to other documents (not applied)

| Document | Section | Amendment | Owner |
|---|---|---|---|
| DOM, bindings and event loop | DM-D10, 6.7 | One recorded tick per frame; each document's rAF argument and timeline time relative to its own time origin, quantized to its grid **without jitter**; `performance.now()` keeps its jitter but never returns less than the realm's latest frame time; dedicated workers receive the tick for `OffscreenCanvas` rAF | DOM owner |
| DOM | 6.7 step table, 6.17 | "Update animations and send events": `hg-anim::update` returns a change list; `hg-script` re-validates, settles promises, checkpoints, sorts and dispatches; `hg-anim-events` removed at AN-M1; "pending view-transition operations" owned by `hg-view-transition` with graphics; analytic wake-ups request a rendering opportunity | DOM owner |
| DOM | 6.7 register | Rows (or row text) for level X and its undo (control-thread command) and View Transition capture completion (IPC reply) | DOM owner |
| DOM | 6.15 | An always-on, constant-time summary per event type of whether a listener exists on a node's event path, including `Document` and `Window`; the settled signal's animation clause (6.13) | DOM owner |
| DOM | 6.9, G94 | `hg-navigation` gains cross-document View Transition eligibility (with the isolation-state rule) and snapshot capability re-grant (G136) | DOM owner |
| Graphics and rendering | GR-D13, 6.8, 6.28 | The bounded hand-off record of 6.5 replaces "keyframes, timing function, start time, timeline"; the sample-time rule of 6.3; per-frame reports (sample time, class S offset, end) go to tools and AN-6 only; `hg-interp` is owned by this spec from AN-M0 and split from `hg-interp-geom` | Graphics owner |
| Graphics | G61, G63, GR-18 | `hg-anim-time` joins G61's list; G63 states that the compositor links `hg-anim-time` and `hg-interp` only and samples only class C records, with no path data; the record's growth goes through GR-18's security review, with its IDL size published | Graphics owner |
| Graphics | GR 6.3, 6.4 | Static transform functions' trigonometry comes from `hg-interp`'s degree functions, so static and animated transforms paint identically | Graphics and style owners |
| Graphics | GR-D15, 6.9, 6.20 | The display runs at the highest needed rate; each renderer's BeginFrames are decimated to its own vote; 30 Hz† cap under low power with user scrolling and input exempt; the Hardened 60 Hz clamp as a ceiling; rAF cadence follows the renderer's rate | Graphics owner |
| Graphics | 6.28 View Transitions row, GR-D18 | Compositor-held snapshots take only opacity, transforms, rectangular and rounded clips and `plus-lighter` (if admitted in the compositor); other effects skip the transition | Graphics owner |
| Graphics | GR-D25 | The flash-protection mode becomes a motion setting this spec adopts; the compositor dims the damaged region over WCAG 2.3.1's thresholds while it is on | Graphics owner |
| Graphics | 9.3 | The compositor track is reduced by the `hg-interp` work (about 1†) that moves to this spec | Graphics owner |
| HTML and CSS parsing and style | 6.9, 6.10, 6.12, 6.15 | The `AnimationSource` trait (no `hg-style` dependency on `hg-anim`); the effect-stack snapshot read during the parallel pass; transition candidates written by workers; transition creation as a sequential post-pass after the merge; base value, compute against element, "`display` ignoring animation origins"; one style change event per flush, and a discard rebuild is not one; animation-type and motion-inducing columns in the property overlay; static transform trigonometry through `hg-interp`; the two inputs 6.10 left open are defined here (6.5, 6.6); before-change styles remain style's (ST 6.11, ST-30) | Style owner |
| Layout | 6.10, 6.19 | View-timeline subject geometry and scroll ranges per layout; the stale-timeline re-run as one more pass of the style and layout loop | Layout owner |
| JavaScript engine | 6.9 (`hg-math`) | Correctly rounded binary64 functions, degree variants of `sin`, `cos` and `atan2`, and the floating-point control-register assertion at frame start | JS owner |
| Accessibility | 6.11 | `hg-ax-prefs` carries Izcal's motion level per site (R, S, X), the Android duration scale (internal only) and the flash-protection setting; R0 holds in the Hardened profile and only the opt-in uniformity profile reports defaults; every UA motion behaviour reads the one exposed value; the 9.5 rows to graphics and the shell are aligned (author animation untouched at R; flash protection adopted) | Accessibility owner |
| Images and media | MD-D19, MD-D20 | Level X pauses animated images and muted loops; SV-D16's SVG-image rule under R; the media process links `hg-anim`'s scriptless core for SMIL in SVG images and receives ticks from the embedder; animated images use the frame-demand rules | Media owner |
| Resource footprint | FP 6.4, 6.6, 6.13, FP-1, FP-D15 | An "animation state" ledger category with 6.15's budgets, outside FP-11's per-node split; compositor animation-record bytes per renderer key in FP 6.6 and FP-1's compositor line; FP 6.6's "beyond the cap, changes re-raster in the content GPU process" corrected to demotion to the main thread; FP-D15's "display's lowest refresh" replaced by 30 Hz† with user input exempt (on ProMotion the lowest refresh is 10 Hz) | Footprint owner |
| SVG | 6.9 | SV-D4 agreed; this spec provides attribute effects, the overlay rule, internal iteration composite `accumulate`, paced offsets as inputs, `SmilControl`, fragment timelines, image timelines per (image, embedding document), observed-event timing under throttling and SV-D16; the SMIL composite position measured (Q-SV-7, Q-AN-1) before it is decided | SVG owner |
| DevTools | 6.1, IIP | An animation domain: decision records, A1–A8, the recorded-session animation view; CDP Animation domain in the compatibility profile; BiDi `animation` module proposals | DevTools owner |
| Shell UI design | Motion | Motion settings (level per site, X command and shortcut, rAF throttle offer, flash protection), and the uniformity-profile warning | Shell |
| Security scope | Fuzzing, threat model | `hg-anim-fuzz` and the SMIL fuzzer as top-tier targets; threat-model rows for the extended compositor record and the per-key compositor byte caps | Security owner |
| Stage 1 design | 2.1, 2.2, G9 | Crates and layers of 6.16; G130–G139 in the registry; `hg-anim`, `hg-anim-values` and `hg-view-transition` in G9's anchor list | Stage 1 owner |
| HGBA-3 | DET-1, PRIV-2, B10 | Frame ticks named among the recorded inputs; PRIV-2 rows for level S's discreteness, X's pause states, the rAF throttle and per-renderer cadence; no R0 normalization outside the opt-in uniformity profile | HGBA-3 editor |
| Reconciliation 2 | Q20, Section 4 | Owner rows for `AnimTime`, `hg-anim-time`, `hg-interp` and `hg-interp-geom`; the animation total added, with the graphics transfer subtracted | Reconciliation owner |
| Phase 3 ledger | G-13, A-FUT-4 | G-13 closed by this spec; A-FUT-4 fulfilled, noting the record is no longer "transform and opacity only" in content (it remains transform and opacity in the compositor) | Ledger editor |

## 10. Risks

| Risk | Likelihood | Effect | Mitigation |
|---|---|---|---|
| Exact time and fixed-point geometry cost more than f64 | Medium | Slower animation step | AN-S1 at AN-M0 on three ISAs, with (b) as a per-part fallback (AN-D2) |
| Deterministic boundary decisions differ from other engines where tests assume double rounding | Low | WPT expectations at exact boundaries | Q-AN-2 observations; file tests that assume rounding |
| Correctly rounded `hg-math` is not accepted, or slow | Medium | Identity still holds; checkability lost | Identity is the requirement; correct rounding is requested, not depended on |
| The Web Animations and Scroll-driven Animations EDs keep changing, and resolutions land before text | High | Rework | Pin the revision; implement resolutions explicitly and list them; anchors make changes findable ([SP 1.1][SP]) |
| Swept-bounds visibility is too conservative and rarely skips | Medium | Less energy saved | AN-30, AN-35 measure it; the bound can be refined per frame without changing semantics |
| Level S breaks pages | Medium | Users disable it | Opt-in, per site, measured (AN-70) |
| G′ and U stay off and authors expect colour animations to stay smooth under load | Medium | Jank on busy pages | Built behind a flag by AN-M2; AN-S2 decides; A3 explains the placement |
| View Transitions' cross-document path is subtle (bfcache, redirects, isolation) | Medium | Bugs at a security boundary | Broker-side eligibility (G136), probes (AN-53) |
| SMIL lifetime bugs | Medium | Crashes | Arena, no recursion, top-tier fuzzing ([SR 1.8][SR]) |
| Flash protection is patent-encumbered | Low | The method changes | Patent search at AN-M0 (MD-D3's precedent) |
| Hiring numerics and compositor specialists | Medium | Schedule | Shared with graphics and JS tracks; AN-M0 before GR-M1 |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-AN-1.** The SMIL composite class's position against CSS and script animations, per engine, black-box (with the SVG spec's Q-SV-7); decided after measurement.
- **Q-AN-2.** At an exact `steps()` boundary, and at exact iteration ends set through `currentTime`, which step and iteration does each engine show ([DR Q-DMR-4][DR])? What precision do engines expose for timeline and animation times ([DR Q-DMR-2][DR])? Does any engine's cubic-Bézier output depend on duration ([DR Q-DMR-1][DR])? Black-box, AN-M0.
- **Q-AN-3.** Which `steps()` level to anchor to, given that Easing 1 and Easing 2 disagree ([SP 5.5][SP]); file the difference.
- **Q-AN-4.** HTML has no step for stale scroll timelines; Izcal's subject-geometry addition ([SP 13.1][SP]). File with WHATWG and the CSSWG.
- **Q-AN-5.** The zero-range resolutions of 27 May and 3 June 2026 (#9256, #13817) are not yet in the Scroll-driven Animations text; file for the text ([FK][FK]).
- **Q-AN-6.** View Transition timeouts, maximum snapshot size and behaviour when the two documents' cross-origin isolation states differ, per engine, black-box; the circulating 4 s figure for Chrome is unverified ([SR 5.8][SR]).
- **Q-AN-7.** The 2024 reduced-motion cross-fade resolution is not yet in the View Transitions drafts ([MA 5.8–5.9][MA]).
- **Q-AN-8.** Engine limits (animations per document, keyframes, `linear()` stops) and events per frame for 1 ms iterations, black-box ([SR][SR] open questions 1–2).
- **Q-AN-9.** Which properties each engine keeps smooth with the main thread blocked, and whether Chrome ships composited `clip-path` and `background-color` today ([CF Q-9.1–9.2][CF]; [HP 6.7][HP]).
- **Q-AN-10.** Phase skew between main-thread and off-thread animations per engine ([CF Q-9.3][CF]); whether `getComputedStyle` equals the displayed value per engine ([HP][HP] open question 7).
- **Q-AN-11.** Start delay from `animate()` to the first changed frame per engine ([CF Q-9.7][CF]).
- **Q-AN-12.** rAF cadence on 120 Hz and 144 Hz panels and in each OS low-power mode, per engine ([CF Q-9.4][CF]).
- **Q-AN-13.** Whether engines keep waking for invisible animations, and spinner power per engine ([CF Q-9.5–9.6][CF]).
- **Q-AN-14.** How engines time style change events relative to script reads and rAF ([GO][GO] open question 6).
- **Q-AN-15.** How many users turn on reduced motion: no OS or survey figure was found ([GO 4.5][GO]; [MA 2][MA]).
- **Q-AN-16.** Whether the long-arc slerp and classical Gram–Schmidt in Transforms 2 should be raised with the CSSWG ([DR 6.3, 6.5][DR]).
- **Q-AN-17.** When each engine runs an author `then` getter on a ready promise relative to other animations' timeline updates (6.3); file Izcal's order with the CSSWG.
- **Q-AN-18.** The flash-protection patent and royalty search (AN-M0).

### 11.2 Claims that are single-source, unverified or conflicting

| Claim | Status |
|---|---|
| Power of CSS and SVG animations, 100–500 mW against about 25 mW idle | One lab blog, one machine ([MA 7.1][MA]; Medium) |
| Chrome's composited `background-color` trials | Two 2021 trials with conflicting outcomes, no figure published, later fate unconfirmed ([HP 6.6][HP]; [CF 1.7][CF]; [FK][FK]) |
| Safari caps page rendering near 60 fps on ProMotion by default | Third-party reports of a feature flag; no Apple statement ([CF 4.8][CF]; Low) |
| Chrome's composited `clip-path` "close to launching" (January 2026) | One engineer's comment; shipping unconfirmed ([CF 1.8][CF]; Medium) |
| SMIL deprecation suspended in 2016 | The blink-dev thread as served ends in September 2015 ([HP 2.10][HP]; Unverifiable here); the SVG spec's notes cite a www-svg message of 17 August 2016 for the suspension ([SV 6.9][SV]) |
| Animation Worklet's stall reasons | No published post-mortem ([HP 8.3][HP]; Low) |
| SMIL usage grew from 0.047% to 2.90% | Counting method changed around 2017–2018; ratio indicative only ([HP 2.12][HP]) |
| 2026 Chrome and Firefox animation CVEs | All 16 identifiers confirmed on NVD on 20 October 2026; counts are keyword tallies, lower bounds ([SR 1.1–1.5][SR]; [FK][FK]) |
| The 7.3% step-boundary experiment | Reproduces, but ranges from 6% to 14% with the durations chosen ([DR 4.8][DR]; [FK][FK]) |
| Epilepsy Foundation forum attack (2008), targeted GIF assault (2016) | Secondary sources ([SR 8.5–8.6][SR]; Low); the 2019 Twitter attack unverified ([SR 8.7][SR]) |
| 35% of US adults 40+ have vestibular dysfunction | A balance-test figure, not a measure of sensitivity to screen motion ([MA 1.3–1.4][MA]; Medium) |
| Kotcher et al., CCS 2013 | From recollection; the paper was not reachable ([SR 4.4][SR]; Medium) |
| Chrome Energy Saver's frame rate | Unpublished; the 30 fps figure is unverified ([MA 8.9][MA]) |
| `chrome.accessibilityFeatures.animationPolicy` on desktop | Unconfirmed ([GO 4.9][GO]) |

## Appendix A. Review disposition (revision 0.2)

Phase 8 was run by a fact-checker and a design critic who did not write this spec: [fact-check][FK] and [critique][CR].

### Fact-check: must-fix items

| # | Item | Disposition | Where |
|---|---|---|---|
| 1 | AN-D4's basis: "the one field trial found no smoothness gain" | Applied: both 2021 trials cited; G′ and U held because the evidence conflicts and no figure was published; built behind a flag | Summary, 2.1, 4.4, 5, 6.5, 6.18, 9.1, 11.2 |
| 2 | "Spec order is preserved" for promise settlement | Applied: the `then` getter runs inside the timeline update in the spec's order; Izcal's order is an Izcal choice, filed, with re-validation and fuzz cases | 6.3, 4.4, Q-AN-17 |
| 3 | "The same F for every document" | Applied: one tick, F per document from its own time origin and grid; AN-54 restated | 6.3, AN-D3, AN-54, 9.5 |
| 4 | Skipped `content-visibility` subtrees "unobservable" | Applied: they follow Contain 2; separated from the optimization | 6.11, 4.4 |
| 5 | Inactive scroll timelines follow obsolete text | Applied: the 27 May and 3 June 2026 resolutions implemented and filed for text | 6.6, 6.9, 7, Q-AN-5 |
| 6 | Easing error bound false for overshooting curves | Applied: curve-dependent step count, bound restated | 6.2 |
| 7 | Engineer-year arithmetic | Applied: about 105–175† | 9.3, summary |
| 8 | WCAG's blur exclusion amended by erratum | Applied: blur counted as motion-inducing under level S | 6.10 |

### Fact-check: recommended items

| Item | Disposition |
|---|---|
| 8× power | Applied: 100–500 mW against 25 mW, 4–20× |
| "Iterating more quickly" quote | Applied |
| WebKit default date is its main branch | Applied |
| `pageswap`/`pagereveal` activation | Applied: precise per event |
| Eligibility clauses (bfcache restore, UA navigation experience) | Applied |
| G137's quote | Applied: "can not be accessed by the Document" |
| "35 of 43 memory-class" wording | Applied: "35 of the 43 SMIL, CSS and Web Animations and View Transitions CVEs" |
| 11.2 CVE row | Applied: confirmed on NVD |
| WPT constants in 6.2 | Applied: removed; WPT used black-box |
| 7.3% range | Applied: 6–14% |
| Far-path exponents | Applied differently: time is i128 ns saturating with exact 256-bit products, so no floating-point path remains |
| Event-count citation | Applied: CSS Animations 2's event table |
| † on AN-30, AN-31, AN-32 windows | Applied |
| UAAG 2.0 is a Note | Applied |
| Note fixes (SR 5.1–5.2 fragments, SP 13.2 resolutions, HP 6.6 second trial, CF 1.8 date, DR 1.11, DR 9.3 table, SR 6.4 date, MA 3.3–3.4 blur, MA 7.1 range) | Applied in the notes |

### Design critique

| ID | Severity | Disposition | Where |
|---|---|---|---|
| C1 | High | Applied: `hg-interp` split from `hg-interp-geom`; compositor links no `hg-path`; motion path on the main thread; G63, GR-18 and A-FUT-4 amendments | 6.5, 6.16, G134, 9.5 |
| C2 | High | **Applied differently**: the SVG spec's draft 0.2 already adopted AN-D6 (it withdrew `hg-smil`); this spec takes its R10 requests (attribute effects and overlay, internal `accumulate`, paced offsets, `SmilControl`, fragment and image timelines, observed-event timing, SV-D16) and measures the composite position first | 6.1, 6.8, 6.9, 6.10, 9.5 |
| C3 | High | Applied: swept bounds, three conditions, Contain 2 separated, correctness cases in AN-30 | 6.11, AN-30 |
| C4 | High | Applied: effect-stack snapshot, transition candidates, sequential post-pass, `AnimationSource` trait; AN-24 | 6.4, G131, AN-24 |
| C5 | High | Applied: record and per-key caps, demotion, ledger | 6.5 rule 6, 6.14, 6.15, AN-42 |
| C6 | High | Applied: one tick, per-document F, `performance.now() ≥ F`, workers; per-cluster jitter weighed | 6.3, 6.18, AN-D3, AN-54 |
| C7 | High | Applied: R0 holds in the Hardened profile; only the opt-in uniformity profile reports defaults | 6.10, 6.14, AN-55, 9.5 |
| C8 | Medium | **Applied differently**: i128 ns saturating with exact products and quotients; no floating-point path; read-back invariant restated | 6.2 |
| C9 | Medium | Applied: curve-dependent step count; progress floored | 6.2 |
| C10 | Medium | Applied: Izcal choice, re-validation, fuzz cases, black-box question | 6.3, AN-5, Q-AN-17 |
| C11 | Medium | Applied: compositor-held snapshots take only compositor effects; others skip | 6.7, 9.5 |
| C12 | Medium | Applied: recorded-input rows added | 6.12, 9.5 |
| C13 | Medium | Applied: always-on listener summary asked of DOM; wake-ups request a rendering opportunity | 6.5, 6.11, 9.5 |
| C14 | Medium | Applied: X is a UA pause flag; View Transitions skipped under X | 6.10, AN-71 |
| C15 | Medium | Applied: one degree trigonometry for static and animated transforms; motion paths share the SVG spec's domain; AN-7 | 6.2, 6.5, 9.5 |
| C16 | Medium | Applied: AN-6 compares at the recorded (F, offset) | 6.5, 6.6, AN-6 |
| C17 | Medium | Applied: per-renderer decimation; Hardened ceiling; FP-D15 amended; input exempt; AN-37 | 6.11, 9.5 |
| C18 | Medium | Applied: G′ and U built behind a flag by AN-M2, shipped at AN-M3 if positive; class G budget set now | 6.5, 9.1, 9.2 |
| C19 | Medium | Applied: riscv64 forms of performance targets; AN-S1 on three ISAs | 8, 9.2 |
| C20 | Medium | Applied: "animation state" category; before-change styles stay style's; discard rebuild is not a style change event | 6.4, 6.15, 9.5 |
| C21 | Medium | Applied: engineer-years recomputed; one counting rule; graphics transfer subtracted | 9.3, 9.5 |
| C22 | Low | Applied: versions and section numbers updated | Header, 6.15 |
| C23 | Low | Applied: merge rule ("snap forward") and scroll axis defined | 6.10 |
| C24 | Low | Applied: MPFR oracle; exhaustive proof over the reduced domains | 6.2, AN-22 |
| C25 | Low | Applied: patent search; WCAG-only method; damaged region dimmed; AX and GR rows aligned | 6.10, 9.5, Q-AN-18 |
| C26 | Low | Applied: agent parts behind `ai`; production clause as a statement of fact | 6.13, 6.16 |
| C27 | Low | Applied: statistics stated | 8 |
| C28 | Low | Applied: isolation-state rule tagged, filed, observed | 6.7, Q-AN-6 |
| C29 | Low | Applied: licence lint (G139); fuzzing engines under the security scope's review | 6.16, 7 |
| E1 | — | Applied: WPT constants removed from the notes and the spec | 6.2, DR 4.3, DR 5.1 |
| E2 | — | Applied: Bugzilla 1208411 cited by comment prose and milestone; WebKit 250970's date is from its comments, as the fact-check confirmed | CF 3.6 |
| E3 | — | Applied: repository links marked as README prose | Notes |
| E4–E6 | — | No change needed | — |
| R1–R18 | — | Applied as 9.5 rows (graphics G63/GR-18/6.28/GR-D15/9.3, ledger, SVG, style, Stage 1, FP, DOM, AX, MD, HGBA-3, reconciliation 2, security scope) | 9.5 |

[HP]: ../research_notes/Izcal%20animation/history_postmortems.md
[SP]: ../research_notes/Izcal%20animation/specs_semantics.md
[CF]: ../research_notes/Izcal%20animation/compositor_frame_timing.md
[DR]: ../research_notes/Izcal%20animation/determinism_math_replay.md
[SR]: ../research_notes/Izcal%20animation/security_robustness.md
[GO]: ../research_notes/Izcal%20animation/gaps_opportunities.md
[MA]: ../research_notes/Izcal%20animation/motion_accessibility_energy.md
[FR]: ../research_notes/Izcal%20animation/frame.md
[FK]: ../research_notes/Izcal%20animation/factcheck.md
[CR]: ../research_notes/Izcal%20animation/critique.md
[H2]: HGBA%202%20browser%20architecture%20spec.md
[H3]: Future%20software%20browser%20engine.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[JS]: Izcal%20JavaScript%20engine.md
[FP]: Izcal%20resource%20footprint.md
[ST]: Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[LY]: Izcal%20layout.md
[MD]: Izcal%20images%20and%20media.md
[GR]: Izcal%20graphics%20and%20rendering.md
[DM]: Izcal%20DOM%2C%20bindings%20and%20event%20loop.md
[AX]: Izcal%20accessibility.md
[SV]: Izcal%20SVG.md
[DT]: Izcal%20DevTools%20spec.md
[RN2]: ../research_notes/Phase%202%20reconciliation/reconciliation_2.md
[LG]: ../research_notes/Phase%203%20amendments/ledger.md
