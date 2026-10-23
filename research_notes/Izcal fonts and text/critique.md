# Izcal fonts and text: independent design critique (phase 8)

Written 19 October 2026 by a reviewer who did not write the spec. Subject: `reports/Izcal fonts and text.md`, draft 0.1 (line numbers below refer to that draft). Method: `.claude/skills/first-principles-spec/SKILL.md`, phase 8.

**Read against:** HGBA-2 2.3.0 (status line, M-1, I-1, I-2, I-6, D1, Section 4.8); Stage 1 design draft 0.11 (Section 1 process map, Section 2 G1–G9, 3.3, 5.6–5.8, 6 anti-fingerprinting row, 7.2, 7.5, 8.1–8.3, 11 D-S1, 13.2 rows "Text and input", "User agent", "Everyday features", "Threat model"); HGBA-3 (B1, B4, B5, B8–B10, DET-1, VER-1, PRIV-2); JS engine spec draft 0.3 (JS-D12, JS-D15, G6, G13, G14, G17, G18, 6.9, 6.13, 6.15); resource footprint spec draft 0.3 (FP-D11, FP-D13, FP-D14, FP-D19, 6.3, 6.4, 6.6, 6.12, 6.18.1, 8.2); DevTools spec 0.3; HTML/CSS parsing and style spec and networking spec on this branch (siblings written in parallel); shell UI design 4.10; security scope 0.3 (`../security-scope`, 6.3, 6.3a, 6.6 IPC-R7/R15/R17, 6.7, 6.10, ATK-2) and coding standard 0.3.1 (`../cs-q56`, tiers, CS-D2, CS-TY-12); the spec's seven research notes.

**Owner direction applied as binding:** engine work is all Izcal's (no HarfBuzz, rustybuzz, Fontations, FreeType, Parley, swash, cosmic-text, ICU4X segmenters, OTS); UCD and CLDR are data; other projects are ideas and evidence only; no shortcuts for time or complexity, staging only for correctness, security or measurement; font parsing is a top attack surface; B1 and JS-D15 determinism, FP-D13, FP-D14, FP-D19 and §8.2 budgets hold; cross-OS determinism of text matters for B1.

**Overall.** The spec is strong on its core idea (one description per format and script, generated readers with proofs, integer arithmetic, canonical fuel) and on making the hard choices explicit. Its weak points are at the edges it does not own: the OS (which fonts, which files, which OS text stacks still run in Izcal processes), the processes either side of the renderer (session image, GPU service, broker, shell), and sibling specs whose interfaces it assumes (style, graphics, footprint). Seven findings are High.

**Counts:** 37 findings: **7 High, 24 Medium, 6 Low.**

---

## High

### C1. High — OS base fonts "by manifest per OS version" reveal more than the UA, down to patch level

**Section:** 6.4 (lines 261, 265), TX-D1 (line 650), 6.13 (line 398), TX-20 (line 564).

**Problem.** Line 265 justifies OS base-font visibility because "base fonts reveal only the OS version, which the UA already reveals in coarser form". That sentence concedes the leak is *finer* than the UA. Stage 1 13.2 fixes "a Chrome-compatible UA string", and Chrome's reduced UA freezes the macOS version (`10_15_7`) and reports `Windows NT 10.0` for both Windows 10 and 11; the precise version is only a high-entropy client hint. A manifest **per OS version, keyed by file hash** (line 261) makes each macOS release and each Windows release distinguishable by probing named base fonts (presence and metrics), and because vendors update font files in point releases and cumulative updates (Segoe UI Emoji, Apple Color Emoji, CJK system fonts), the hash key also exposes patch level, or hides a font mid-month until Izcal publishes a new manifest. TX-20's success criterion "1 distinct result per (build, OS version)" codifies this leak instead of bounding it, and D-S1 requires discrete surfaces including font enumeration to be "reduced or normalized".

**Fix.** Decision change to TX-D1 (a): "The OS tier is defined **per OS family**, not per version: a face is visible by name only if a face with that family name exists in **every** OS release Izcal supports for that family; its identity is (family, style), not its hash. A file whose metrics differ between supported releases is visible only if TX-20 shows no metric difference on the probe set; otherwise it is dropped from the tier." Rewrite line 265 to state the residual entropy honestly. TX-20 becomes "1 distinct result per (build, OS family) with default settings; any per-release difference is a ship blocker for the responsible face". Manifests are delivered through the TUF update channel and their identity is a recorded B1 input.

### C2. High — The cross-OS identity claim excludes the font names most pages use, and precedence is unspecified

**Section:** 6.4 (lines 260, 263), 6.11 (line 353), TX-30 (line 571), TX-D1.

**Problem.** Line 263: "a page that uses generic families, web fonts or the Izcal set lays out and renders identically on macOS, Linux and Windows". But (i) `system-ui` is a generic family and line 263 maps it to the OS's UI font; (ii) the most common named families (Arial, Helvetica, Times New Roman, Courier New, Segoe UI, Roboto) are OS base fonts on macOS or Windows, while on Linux they resolve to Izcal's metric-compatible faces (line 260). Metric-compatible is not glyph-identical, so canvas text, coverage masks and any width that depends on kerning or ligatures differ by OS. The spec never says whether a page naming `Arial` on Windows gets the OS file or Izcal's substitute. TX-30 is measured "with the Izcal set and web fonts" only, so the B1 claim silently covers a minority of real pages.

**Fix.** Add to 6.4: "**Name resolution precedence:** web fonts → the OS tier → Izcal-set aliases for the core families → generics. A named family that resolves to the OS tier on one OS and to an alias on another is OS-dependent output by design; `system-ui` and `ui-*` are OS-dependent by design." Then either (a) state that TX-30's scope is pages whose resolved faces are all in the Izcal set or web fonts, and publish the corpus share it covers; or (b) offer a decision (new TX-D19): core named families resolve to the Izcal aliases on every OS by default (cross-OS identity for the common case), OS files only under the "use system fonts" setting. Recommend (b) if the corpus share under (a) is below a stated † threshold.

### C3. High — Mapped font files are not immutable: TOCTOU, nondeterministic acceptance and SIGBUS crashes

**Section:** 4.4 row 3 (line 129), 6.1 (line 210, 216), 6.4 (line 271), 6.16 (line 467), TX-D17.

**Problem.** Line 129 claims "a mapped file changing underneath ... cannot become a memory error" because accessors bound-check. Bounds checks do not cover (a) **truncation**: a read from a mapped page beyond the new end of file raises SIGBUS on Linux and macOS, which safe Rust cannot handle, so the renderer (or the shell renderer) dies; (b) **double reads**: validation and use read the same bytes at different times, so acceptance (which line 227 says "must be deterministic") and glyph output can change mid-session; (c) the session image's per-file hash (line 216) is checked once at first map, after which the content can change. OS base fonts live on protected system volumes, but per-site grants of user-installed fonts (TX-D17), Windows per-user fonts and `/Library/Fonts` are writable by the user or other software. This is the IPC-R7 double-fetch problem applied to files.

**Fix.** Add to 6.1: "Renderers map only **immutable** font bytes. A file on a read-only, integrity-protected system volume (macOS sealed system volume; Windows `%WINDIR%\Fonts` under TrustedInstaller) may be mapped directly after its hash is checked; every other file, including per-site granted user fonts, is copied once by the font service into a sealed shared region (sealed memfd, read-only section, read-only Mach memory entry) and only that region is granted. No renderer maps a user-writable file." Add a TX-13 regression: truncate and rewrite a granted font during layout; no crash, no output change.

### C4. High — The session image is a one-to-many attack artifact with no specified reader

**Section:** 6.0 diagram (lines 185–190), 6.1 (lines 209, 216), 6.4 (line 271).

**Problem.** One utility process parses attacker-reachable fonts (line 209 says installed fonts are attacker-reachable) and writes one image that **every** renderer and the shell renderer then read. A compromised font service therefore reaches every site's renderer and the shell renderer (which holds every site's titles, security scope 6.3) through whatever parses the image's own structures (fallback tables, coverage summaries, name lists). The mitigation in line 216 covers only font files (hash plus re-validation), not the image's structures. Security scope IPC-R17 says no process trusts another. The diagram also has the font service parse fonts *then* check the manifest (line 271), so a non-manifest file is parsed before it is rejected.

**Fix.** (1) "The session image format is described in `.hgfont`-style declarations and read only through a generated, Kani-proved reader; renderers treat it as untrusted input (IPC-R17)." (2) First-principles simplification: the Izcal set's part of the image is **built at build time** (tier T, signed, in the engine image inventory per FP 6.3), and the OS part is **derived from Izcal's manifests**, so per session the font service only **hashes** files and selects prebuilt entries; it parses no font bytes. A file whose hash is not in the manifest is never parsed. (3) Add TX-13 cases for a malformed session image.

### C5. High — Renderer-side caches are shared across partitions, against FP 6.4's rule

**Section:** 6.2 (line 235), 6.2 lazy validation (line 230), 6.9 strikes (line 337), 6.12 channel table (lines 387–394), TX-21.

**Problem.** FP 6.4: "every content-derived cache in a process is keyed by the **partition**". The spec partitions the shaping cache and parsed web fonts, but: the strike cache key is "(face instance, size ..., subpixel phase, hinting mode, darkening table)" with no partition (line 337); variation instances are shared by "every consumer of an instance" (line 235); and per-glyph lazy validation (line 230) records "validated on first use" process-wide for the Izcal set and OS fonts. In a renderer holding B under A and B under C, the time to draw a glyph reveals whether the other partition drew it. TX-21 probes only "the glyph atlas, shaping cache and parsed-font sharing", so it would pass with these channels open.

**Fix.** Key the strike cache, the instance outline cache and any lazily computed per-glyph state by partition, or make the per-glyph work constant-time and eager (validate whole tables at map time). Add these three to the 6.12 channel table as **Closed** and to TX-21's probe list.

### C6. High — Text input, IME composition and editing services lose their owner

**Section:** 9.5 row "Stage 1 5.6, 8.1, 8.3, 13.2" (line 678); 6.8 "Other services" (line 329); absent elsewhere.

**Problem.** Stage 1 13.2 "Text and input" rests on "Parley already provides ... IME composition", with platform IME integration (TSF, `NSTextInputClient`, Wayland text-input plus IBus or Fcitx) and composition events at M6. The spec's amendment keeps only "all scripts render in Stage 1". Nothing in the spec covers composition rendering, caret placement and movement by grapheme or by script rule (Indic backspace deletes a code point, not a cluster), word selection with SA dictionaries, `contenteditable` and form-field text services, or the spellcheck that 13.2 plans "through OS services" in Stage 2. On Windows, TSF loads third-party IME DLLs into the focused process, which is a placement question the text, shell and security specs must settle together. An M6 deliverable now has no spec.

**Fix.** Add Section 6.8a "Text input and editing services" (or name the owning spec and add it to 9.5): composition string and attribute rendering via `hg-text`; caret and selection movement rules per UAX 29 plus script tailoring (backspace by code point where the script requires it); IME process placement (which process hosts TSF and `NSTextInputClient`, and that no IME DLL loads into a renderer); spellcheck data flow (OS service in which process, results as ranges). Amend the 13.2 row to say who owns IME after Parley's removal.

### C7. High — OS text stacks still run in Izcal processes; TX-10 is unachievable as written

**Section:** G-TX2 (line 457), TX-10 (line 553), 6.12 (line 380), Section 1 Telugu row (line 18).

**Problem.** TX-10: "**0** OS font or text libraries in any binary's loaded-library closure". The broker owns native windows and OS integration (Stage 1 Section 1; security scope 6.3), and on macOS AppKit's closure includes Core Text; on Windows, USER32/DWM title bars, menus and TSF use DirectWrite or GDI. More importantly, page-controlled strings (titles in native window titles, Dock and taskbar menus, jump lists, OS notifications) reach **OS shapers inside the trusted broker**, which is exactly the class the spec cites (CVE-2018-4124, a Telugu string crashing Core Text, line 18). The spec's threat table has no row for it.

**Fix.** Restate G-TX2/TX-10: "No Izcal crate calls an OS font or text API; OS frameworks linked by the broker for native UI are inventoried, and **no page-controlled string reaches an OS text API in the broker**: native window titles, menus and jump lists show broker-generated strings (for example the site's registrable domain), or text pre-rendered by the shell renderer." Add a 6.12 row "Hostile strings to OS shapers in the broker" closed by that rule, with a T1 test that fuzzes titles through every native surface.

---

## Medium

### C8. Medium — The GPU service does receive outlines, and mask transfer ignores IPC-R7

**Section:** 6.1 (lines 213–214), 6.9 (lines 335, 343), 6.10 (line 347), TX-15.

**Problem.** Line 213 says the GPU service "never sees font bytes or outlines from web content", but line 214 and 6.9 send large and transformed glyphs and large COLRv1 glyphs "as paths" or "vector content" to it: those are web-font outlines. The path pipeline's complexity bounds (segments, nesting, coordinate range) are left to an unwritten graphics spec, in a tier-P process every site shares. Masks "arrive over shared memory with checked dimensions" (line 347); IPC-R7 requires copy-then-validate for anything parsed from shared memory, and exempts only buffers whose size and format "the receiver fixed at creation". No per-partition upload rate or byte cap is stated.

**Fix.** Line 213: "receives coverage masks and, for glyphs above the threshold, paths in the graphics spec's bounded path format". State: mask width, height and format travel in the IPC message (copied); the bytes are in a region the GPU service allocated at that size; the GPU service never reads dimensions from shared memory. Add a per-partition upload budget per frame, and path bounds as a 9.5 item for the graphics spec.

### C9. Medium — The glyph-program side-channel analysis relies on placement, which does not stop the attack

**Section:** 6.3 (line 251), TX-D2.

**Problem.** Line 251: "where it runs, it runs in the page's own renderer, which holds only that site's data". Trampert and Schwarz describe cache-contention and Prime+Probe attacks (RC 2.8, graded Medium, abstract only); those measure **other** processes through shared microarchitectural state, so process placement is not a mitigation. Removing the VM by default helps, but GSUB/GPOS chaining (Fontemon runs in GSUB), AAT state machines and COLRv1 graphs remain scriptless compute, bounded only per code point or per glyph, never per document, and they run even with JavaScript disabled or in the Hardened profile.

**Fix.** Replace the sentence with: "Placement does not mitigate cross-process microarchitectural attacks. Mitigations: the TrueType VM off by default (TX-D5); canonical fuel per document per frame across all glyph programs (new †); no glyph program can observe time." Read the paper in full and regrade RC 2.8 before it supports a decision.

### C10. Medium — Font-file page-cache residency is "Bounded" with no bound

**Section:** 6.12 channel table (line 392), TX-21 (line 565).

**Problem.** A page can time a layout that touches a particular glyph range of a shared CJK font and learn whether another site's process recently drew characters on that page of the file, which leaks text content across sites. FP 6.18.1 reduces the analogous engine-image channel by pre-populating rarely used sections; that does not scale to 20–50 MiB CJK fonts. TX-21 says only "capacity measured and published": no threshold, no failure action.

**Fix.** Add a decision with options: (a) accept, bounded at ≤ N† bits/s measured in the device lab, failure action option (b); (b) pre-fault the whole file when any renderer first maps it (residency then reveals only "some process used CJK"); (c) per-renderer private copies of touched pages for large fonts (memory cost). Give TX-21 a threshold and a failure action.

### C11. Medium — TX-D5: a valid measurement gate, but its fallback rules are FreeType constructs with no anchor

**Section:** 6.9 (line 339), TX-D5 (line 654), 9.1 (line 602).

**Problem.** Building the VM and shipping it only if measured legibility justifies its known attack surface is a **measurement and security gate, not a schedule shortcut**: it is acceptable under the owner's rule. Three parts are not first-principles: (i) "y-only mode" is not defined by OpenType; it is FreeType v40's invention, so implementing it means reproducing another engine's behaviour (M-1 part 3); (ii) "a font whose outlines are built by instructions" needs a detection rule, and the known one is FreeType's "tricky fonts" list (names and checksums), which is another project's data; (iii) "the OS manifests exclude such fonts" (for example Windows CJK fonts such as MingLiU, if Q-TX-5 confirms them) contradicts the "OS base fonts by name" rationale, and a web font of that kind would render wrong, not fall back. The decision rule weighs only legibility, not the attack surface.

**Fix.** Define "y-only execution" in Izcal's own words from the instruction set's semantics, with each deviation from the OpenType instruction text recorded as a black-box observation. Define detection from a measurable property (for example, unhinted outline self-intersection or degenerate contours at a probe ppem), not a name list. State the result for web fonts of that kind (run the VM under fuel, or reject at load as a font load failure). Add to TX-D5's trigger: "and the security review accepts the VM's fuzz and proof record (TX-11, TX-12)".

### C12. Medium — The shell renderer's text path is held to a weaker bar than the threat model needs

**Section:** TX-12 (line 555), Section 1 (line 18), Q-TX-8.

**Problem.** Security scope ATK-2 names "a page title that triggers a shell-renderer bug". TX-12 allows open crashes for "14 days†" and proves panic-freedom only "with the Izcal font set", while the shell uses "System UI fonts" (shell UI design 4.10) and so OS base fonts, AAT `trak`/`morx`, and maybe the VM. A shell renderer crash takes down every tab's UI.

**Fix.** Add TX-12a: "Strings that reach the shell renderer (titles, URLs, notification text, download names) are capped at a length†; the shaping, segmentation, bidi and matching path used by the shell has **0** open fuzz crashes at release (no 14-day window), over every face the shell can use, including the OS UI font." Answer Q-TX-8 here rather than leaving it open.

### C13. Medium — Shaping and strike caches are classed (b), which the footprint spec defines differently

**Section:** 6.8 (line 331), 6.9 (line 337), 6.16 (lines 469–470), 6.14 Footprint row (line 428).

**Problem.** In FP 6.4, class (b) is "heap- or trusted-region-resident" engine data, flushed "only at deterministic points ... never under pressure"; class (a) is own-arena data released under pressure. The spec calls the shaping cache class (b) **and** "released under pressure". FP's own table already lists "Glyph rasters (GPU service)" as class (a). Separately, cache insertion under parallel layout makes the bytes these caches hold depend on interleaving, so FP's canonical-byte accounting (FP-12, simulation counts canonical bytes) and pressure-ladder decisions become nondeterministic unless defined.

**Fix.** Class (a) for the shaping cache, strike cache and parsed-face working state, released by the safepoint handshake. State that cache insertions from parallel workers are merged in a deterministic order at the phase boundary, or that cache bytes are excluded from canonical bytes and the ledger reports them separately.

### C14. Medium — The font set's size has no bill of materials, and the footprint conflict is waved through

**Section:** TX-D1 cost (line 650), 6.16 (line 472), TX-51 (line 583), 9.5 (line 683).

**Problem.** "Estimated 200–350 MiB†" comes from Noto CJK sizes alone. The set must cover: every generic family (`serif`, `sans-serif`, `monospace`, `cursive`, `fantasy`, `system-ui`, `ui-serif`, `ui-sans-serif`, `ui-monospace`, `ui-rounded`, `math`, `emoji`, `fangsong`) for every script; Han regional variants (SC, TC, HK, JP, KR) in both sans and serif; metric-compatible core families; and a last-resort face. That may exceed the range. FP-25 caps the installer at 50 MiB† plus the data set's compressed size, and FP's data-set placeholder is 24 MiB†: the spec multiplies it by about 10 and says only "rises accordingly". TX-51's threshold is "the TX-D1 figure the owner accepts", which is circular.

**Fix.** Add Q-TX-1's answer as a TX-M0 deliverable: a table of faces per (generic, script or region) with measured sizes. Give TX-D1 an explicit owner option: (a) full set in the installer; (b) a separate signed data package installed **with** every installation (still FP-D13-compliant: never fetched because a page asked). Set TX-51 as "installed ≤ X MiB†, compressed ≤ Y MiB†", with the failure action "owner decision between a smaller generic matrix and (b)".

### C15. Medium — Session-image construction is on the first-text critical path, with no budget

**Section:** 6.1 (line 209), 6.4 (line 271), FP 6.3.

**Problem.** Every renderer's first text waits for the session image. Building it means parsing every bundled face (hundreds of MiB) and hashing every OS manifest font, every session, cold. Firefox's font-list stalls (GO 6.1–6.2: 0.13 s to over 1 s) are the spec's own evidence. No target covers it.

**Fix.** Adopt C4's build-time image. Cache OS file hashes keyed by (file ID, size, modification time) in the profile. Add TX-43: "session image ready ≤ N ms† after browser start, cold, on the slowest reference device; first text never waits on it beyond that".

### C16. Medium — Colour glyphs, not only CJK, break the 2 MiB atlas

**Section:** 6.10 (lines 347–349), TX-52 (line 584).

**Problem.** The RGBA pool shares FP-14's 2 MiB cap. A 64 device-px emoji at DPR 2 costs about 16 KiB, so the whole atlas holds about 128 such emoji, fewer than one screen of a chat or social page. The spec treats CJK as "the open number" and measures only CJK at DPR 2.

**Fix.** Extend TX-52 to an emoji-dense part of the corpus, and the 9.5 FP amendment to size the RGBA pool separately. State that colour glyphs above the threshold are drawn as vector content and never enter the atlas.

### C17. Medium — TX-20 conflicts with D-S1's canvas noise and ignores locale and settings

**Section:** TX-20 (line 564), 6.13 (line 398), 6.4 (line 263).

**Problem.** TX-20 expects "canvas text hashes ... **1** distinct result per (build, OS version)" with default settings, but D-S1 still seeds canvas readback noise per (site, container, session) by default, so hashes differ by design. Han fallback for unlabelled text depends on the user's language preferences (FP 6.3 puts "the user's locale preferences" in the session image), so metrics vary by locale. The "use system fonts" setting and per-site grants also change results, and TX-20 measures none of them. The spec never says how unlabelled Han text picks SC, TC, JP or KR forms.

**Fix.** TX-20: compare canvas **pre-noise** text rasters in a test build, plus measured metrics; report results per (build, OS family, UI locale); add PRIV-2 residual-entropy measurement for non-default settings. Add a matching rule to 6.4: "unlabelled Han text takes its region from the document language, then the user's first matching language preference, then a fixed default", recorded as a B1 input.

### C18. Medium — `@font-face src: local()` is unspecified

**Section:** 6.4, 6.5, TX-D1, TX-D17.

**Problem.** `local()` is the classic way to probe for installed fonts without a font-enumeration API, and it matches on full and PostScript names, not family names. The spec never mentions it. TX-D1 says user-installed fonts are never visible, but nothing says what `local()` resolves against or how it matches.

**Fix.** Add to 6.4: "`local()` resolves only against the page's visible set (the Izcal set by its own names, the OS tier, per-site granted fonts), matching full and PostScript names per CSS Fonts 4 §4.3; aliases never match `local()`." Add a `local()` probe to TX-20.

### C19. Medium — Quantizing font size to 1/64 CSS px breaks scaled SVG and canvas text

**Section:** 6.8 numeric contract (line 320), TX-D4 (line 653).

**Problem.** Quantization is "1/64 CSS px", but SVG text sizes are in user units under `viewBox` scaling, and canvas `font` sizes are in canvas units under `ctx.scale()`. A 0.1-unit font scaled 100× quantizes to 6/64 = 0.09375, a 6% error that is plainly visible. The rounding mode for the quantization, and for converting CSS numbers (fractional weights, `font-variation-settings`, `font-stretch` percentages) to Fixed and F2Dot14, is not stated. TX-D4's reversal trigger ("failures ... on the corpus") will not see SVG-heavy or canvas-heavy cases.

**Fix.** Quantize in the **local coordinate space of the text's drawing context** with a relative floor (for example 1/64 of a unit or 2⁻¹⁶ em, whichever is finer), with the rounding mode specified (round half to even) for size and every CSS-to-fixed conversion. Add WPT `svg/text` and scaled-canvas cases to TX-2.

### C20. Medium — The metrics query that the style spec needs is missing

**Section:** 6.14 Integrations (line 423).

**Problem.** The sibling style spec (font-relative units) says: "the fonts and text spec provides a metrics query for (font list, size, style, language) whose results are B1 inputs, and a web-font load that changes metrics is a mutation-log entry". `ex`, `ch`, `ic`, `cap`, `lh` and `font-size-adjust` need the first available font's metrics during style computation, before layout. The integration table lists only "Computed font properties ... `hg-font-match` consumes them".

**Fix.** Add the style row: "`hg-font-match`/`hg-text` → `hg-style`: first-available-font metrics query (CSS Values 4 font-relative lengths, `font-size-adjust`); font install emits mutation-log entries for dependent elements (B4)". Fix the contract at TX-M0 together with the layout contract.

### C21. Medium — Case mapping, case folding and text search are absent from the Unicode table

**Section:** 6.6 algorithm table (lines 281–291), 6.8 pipeline (line 317).

**Problem.** `text-transform` (uppercase, lowercase, capitalize, `full-width`, `full-size-kana`) needs UCD SpecialCasing and CSS Text's language-sensitive rules (Turkish dotted i, Greek accent removal, Dutch IJ). Find in page (Stage 1 13.2 "Everyday features") needs case-insensitive and normalization-insensitive matching. JS's `toUpperCase`/`toLocaleUpperCase` need the same data. None is listed, and ownership between `hg-unicode` and `hg-intl` is not stated. (UTS 10 collation is not needed by the text stack; say so.)

**Fix.** Add rows: "Case mapping and folding | UCD SpecialCasing, CaseFolding; CSS Text 3 §2.1 | Generated tables, shared with ECMA-262 and ECMA-402 in L2u | UCD-derived property tests; WPT `css/css-text/text-transform`" and "Text search for find in page | Default caseless matching plus canonical equivalence | WPT and corpus". Add one line: "Collation (UTS 10) stays in `hg-intl`; the text stack does not use it."

### C22. Medium — The coverage-to-pixel step depends on an unwritten graphics spec and may not be deterministic

**Section:** 6.9 (lines 335, 341, 343), 6.11 table (lines 362–363).

**Problem.** "The compositor blends in linear light with a pinned contrast curve" places gamma, contrast and darkening application in the GPU service, where float evaluation is not bit-exact across vendors (the spec's own WGSL §15.7 point). Canvas text needs bit-exact pixels before D-S1 noise, but the spec never says who composites glyph masks into a canvas bitmap. The GPU path's "stated tolerance" is never stated. Linear-light blending also departs from how other engines render text by default, a web-visible change of look that no CSS specification asks for.

**Fix.** State: darkening and contrast are applied in integer arithmetic in the renderer as part of the mask (pinned tables); compositing color space is owned by the graphics spec (9.5 item); canvas 2D text is composited on the CPU path in the renderer. Give the GPU outline path a tolerance† (maximum per-pixel coverage difference and pixel count) and a test.

### C23. Medium — PDF viewing and printing need fonts the spec excludes or ignores

**Section:** TX-D7 (line 656), 6.2 "Not supported" (line 233).

**Problem.** Stage 1 ships a PDF viewer in a utility process (Section 1, Section 6); Stage 2 ships printing and print-to-PDF (13.2). PDFs embed Type 1, bare CFF, CID-keyed CFF, Type 3 and hinted TrueType fonts (the Trampert paper also tracks PDF pages via hinting). TX-D7 excludes Type 1 without considering PDF. Print-to-PDF needs subsetting and must honour `OS/2.fsType` embedding permissions. Neither is mentioned, and it is not said whether the PDF renderer uses this stack.

**Fix.** Add a scope line: "The PDF viewer uses `hg-fontfmt`, `hg-font-vm` and `hg-glyph-raster` in its utility process; Type 1 and Type 3 are supported **for PDF only**, under fuel, in that process (TX-D7 amended); print-to-PDF embeds subsets per `fsType`." Or name the spec that owns it in 9.5.

### C24. Medium — M-1 part 3: the output contract reuses HarfBuzz's names and API shape

**Section:** 6.7 output contract (line 309), 7 (line 496), 6.7 engines (line 301).

**Problem.** Line 496 claims "Izcal's own names are used". The per-glyph flags are "unsafe-to-break", "unsafe-to-concat" and "safe-to-insert-kashida", HarfBuzz's glyph flags in name and meaning, and the per-glyph record (glyph ID, advance, offset, cluster, flags) is HarfBuzz's buffer shape. "Strike" is Skia's and FreeType's term. The engine list includes a separate "Thai and Lao" engine, whose known behaviours (for example sara am decomposition and legacy PUA fallback) are documented by other shapers, not by a normative or Microsoft text that the spec cites.

**Fix.** Derive the contract from CSS Text's requirement (where a break or a join requires reshaping) and name it in Izcal's terms (for example `reshape_on_break`, `join_boundary`, `kashida_opportunity`), recording the convergence as evidence in 7. For every engine, cite its normative or Microsoft anchor; where none exists (check Thai/Lao and Hangul), mark its rules as black-box observations in `compat/observations/` (Stage 1 7.3).

### C25. Medium — Holding `word-break: auto-phrase` is staging that the SA decision itself contradicts

**Section:** 6.6 (line 293), 9.1 (line 605).

**Problem.** The reason given is "'UA-dependent' behaviour has no oracle". But SA-script word breaking is also UA-dependent with no normative oracle, and the spec builds it with dictionaries and native-reader review (TX-D12). The same method applies to Japanese phrase breaking. Holding one and not the other is a scope cut by convenience, which the owner's rule does not allow.

**Fix.** "`auto-phrase` is built with an integer-scored phrase dictionary under TX-D12's method, gated by native-reader review (the TX-3 method) and WPT; it ships when its gate passes."

### C26. Medium — Decisions against LCD and Graphite misstate the evidence and have triggers that cannot fire

**Section:** TX-D6 (line 655), TX-D7 (line 656), 6.17 (lines 486–487).

**Problem.** TX-D6 lists the cost as "Nothing", but the spec's own source (HP 4.4) records the ex-Apple engineer conceding that LCD AA "looks better on 1× displays", and RC 3.5 says most desktop displays are 1×. "Microsoft recommends grayscale" (line 44) is a Store-app recommendation (HP 4.1). The trigger, "a low-DPI reading study", has no design. For Graphite, the trigger "> 0.1%† of pages" is a page share, which structurally never fires for minority scripts (some SIL fonts' complex behaviour is in Graphite; verify which). U1 promises "every script ... as its users expect".

**Fix.** TX-D6 cost: "legibility at DPR 1 (unmeasured); LCD would need opaque layers and per-display geometry". Trigger: TX-D5's legibility study also compares an LCD prototype on opaque layers at DPR 1. TX-D7 trigger: "a script or language whose readers' main font needs Graphite for correct rendering, per the native-reader review" (per script, not per page).

### C27. Medium — Feasibility: text is on Stage 1's critical path and the estimate is not derived

**Section:** 9.2 (line 612), 9.3 (lines 624–642).

**Problem.** Line 612 says text "is not on the engine's critical path if TX-M1 precedes M4", but TX-M2 (27–33 months) must precede Stage 1 exit because 13.2 requires all scripts, and Stage 1's own placeholder is 24–36 months. TX-M0 at 9 months asks 12–15 newly hired specialists to deliver `.hgfont`, proofs, Unicode at 100%, the shaping-spec model, three engine drafts, the font set and its licence review, and manifests. The 28–37† total exceeds the whole Stage 1 staffing (23–30.5†) and is a list of track ranges, not derived from a size model. The JS Intl track (4–5) and the security scope's security track (3–4) overlap with "Unicode for text" and "Verification and security".

**Fix.** State: "Text is on Stage 1's critical path at M4 (TX-M1) and M8 (TX-M2)." Derive track sizes from component inventories (formats × tables, engines × scripts, algorithms) and reference sizes (rustybuzz 17 KLOC; Fontations' team and time), with the double counting with the JS and security tracks removed explicitly. Give TX-M0 a measured-velocity review at month 6.

### C28. Medium — Omitted text features are not stated as omitted

**Section:** whole spec; template rule "say so in one line rather than deleting silently".

**Problem.** Missing without comment: `text-decoration` (underline position and thickness from `post` and `OS/2`, `text-decoration-skip-ink` needs outline intersection), ruby, `text-emphasis`, `text-combine-upright`, CSS Text 4 `text-spacing-trim` and `text-autospace` (font features `halt`, `chws`), `initial-letter`, `font-variant-emoji` (a matching input), the `font-palette` property (only `@font-palette-values` is named), palettes and colour fonts in canvas, `text-overflow: ellipsis` shaping, `text-wrap: balance`/`pretty` algorithms, `hanging-punctuation`, `-webkit-text-stroke` (outlines to paint), highlight painting (`::selection`, `::highlight`, `::spelling-error`), forced colors with colour fonts, localized family names in matching, and a user minimum font size.

**Fix.** Add Section 6.18 "Coverage of CSS text features": a table with each feature, its owner (this spec, layout or graphics) and what this stack supplies (metrics, outlines, features), or "not in scope, because ...".

### C29. Medium — Several targets and triggers cannot be falsified

**Section:** 8, 9.4.

**Problem.** TX-21 "capacity measured and published" has no threshold. TX-51 is circular (C14). TX-D5's "reading study ... statistically significant" has no design, sample size or test. TX-D1 and TX-D6 triggers cite unspecified reading studies. The GPU path tolerance is unstated (C22). TX-3's "unexplained" divergences are judged by the team that writes the specification. TX-60's "100% of text on a page" has no defined denominator.

**Fix.** Give each a number† and a method: TX-21 ≤ N† bits/s; the reading study as a pre-registered protocol (participants, task, primary metric, α); TX-3 "unexplained" means "no Izcal-spec rule ID and no recorded observation is attached"; TX-60 denominator "every glyph in the display list".

### C30. Medium — Tier and lint conflicts with the coding standard

**Section:** 6.15 (lines 435–436), G-TX3 (line 458).

**Problem.** `hg-ucd` is "R (also linked by tier P where the shell's URL display needs it)", and URL display needs UAX 9 and UTS 39 from `hg-unicode`. The coding standard's `hg-graph-lint` computes a crate's tier from the strictest binary that links it and fails when the declared tier is laxer, so the broker linking either crate makes it tier P, which forbids tier-R panics on any input. G-TX3's "checked or explicitly widened operations" duplicates the JS spec's G18, and in tier R release builds wrap on overflow (CS-D2 (b)) unless a rule says otherwise.

**Fix.** Declare `hg-ucd` and the UAX 9 and UTS 39 parts of `hg-unicode` tier P (no panics on any input), or keep URL display out of the broker (it is drawn by the shell renderer). Fold G-TX3 into G18 by extending G18's scope to "every value derived from font bytes", which carries G18's "no wrapping in release" clause.

### C31. Medium — Offering to embed OS font files in exported recordings invites licence violations

**Section:** 6.11 TX-D18 (line 370), TX-D18 (line 668).

**Problem.** "The export dialog offers to embed the OS font files the recording used, with a notice". Apple's system font licences forbid redistribution (GO 5.4 for San Francisco), and Windows fonts' EULAs are similar. A notice does not make Izcal's feature lawful, and it puts proprietary files in shared artifacts.

**Fix.** Do not embed OS font files. A shared recording records each OS font by manifest entry; replay without it fails loudly (as TX-D18 (a) already says), or replays with the OS faces replaced by their Izcal aliases in a mode marked "not exact".

---

## Low

### C32. Low — Q-TX-11 is already answered by the networking spec

**Section:** 6.2 (line 233), Q-TX-11 (line 719), 7 (line 498).

**Problem.** The sibling networking spec decides NW-D6: Izcal's own `hg-compress` (L2, tier P) decodes deflate and Brotli, "shared with the text stack (WOFF2)".

**Fix.** Cite NW-D6 and `hg-compress`, delete Q-TX-11, and state the WOFF2 decompression budget (declared size, ratio cap) using `hg-compress`'s budget interface.

### C33. Low — avar2 held for a document purchase

**Section:** 9.1 (line 604), Q-TX-2.

**Problem.** "Built when the ISO 5th edition text is obtained" holds a feature on procurement, which is neither correctness, security nor measurement.

**Fix.** "The ISO/IEC 14496-22:2026 text is bought at TX-M0; avar2 is built with variations."

### C34. Low — Linux system-font handling is garbled and touches the broker

**Section:** 6.4 (line 271), Q-TX-8.

**Problem.** "reads fontconfig's aliases and rejects as data where they apply to the shell's own UI" does not parse as a sentence. Resolving fontconfig `<include>` and `<dir>` elements to find the files to open requires parsing XML in the broker, against IPC-R15.

**Fix.** "On Linux the broker opens `/etc/fonts/fonts.conf` and passes the handle; the font service parses it and returns the list of further paths it needs, which the broker opens without parsing. Aliases are used only for the shell's UI font (Q-TX-8)."

### C35. Low — Variation arithmetic is underspecified where layout can observe it

**Section:** 6.2 (line 235).

**Problem.** "interpolation of untouched points (IUP) in 26.6" applies a device-space format to font-unit deltas, and the rounding order (per tuple, or after summing) is unstated. Advances that come from `gvar` phantom points, when `HVAR` is absent, are layout-observable.

**Fix.** Specify deltas as font units in 16.16 accumulated across tuples and rounded once (half to even) before scaling, with IUP in the same space, and record differences from other engines as observations.

### C36. Low — Some options are strawmen and some decisions are missing

**Section:** 9.4.

**Problem.** TX-D11 (b) "separate implementations", TX-D16 (b) "plain word cache" and TX-D8 (b) are not credible alternatives. Missing decisions include: OS-tier granularity (C1), name precedence (C2), the Han default (C17), the `local()` policy (C18), build-time versus per-session image (C4), the emoji face for the `emoji` generic (the Izcal COLRv1 face versus the OS emoji font by default), and IME placement (C6).

**Fix.** Replace each strawman with a real alternative (for example TX-D16 (b): no cross-run cache, with incremental reshaping only), and add the missing rows.

### C37. Low — Accessibility output is thinner than the semantic plane needs

**Section:** 6.14 (line 416).

**Problem.** AccessKit-style text APIs need per-character bounds and word boundaries for each text run, which come from the cluster map. The last-resort hex glyph must still expose the code point to assistive technology, and forced colors must override COLR and `CPAL` colours.

**Fix.** Add these to the semantic-plane bullet and to TX-60's facts.

---

## Reconciliation list (amendments to other documents that Section 9.5 misses)

| # | Document | Amendment |
|---|---|---|
| R1 | Stage 1 13.2 "Text and input" | Name the owner of IME composition, caret and editing services and IME process placement now that Parley is gone (C6) |
| R2 | Stage 1 Section 1 and Section 6 (PDF viewer) | The PDF utility process uses this stack's reader, VM and rasterizer, with Type 1 and Type 3 for PDF only (C23) |
| R3 | Stage 1 3.3 determinism table | Add text rows: shaping-cache insertions under phase purity, font installs as recorded decision points (JS-D15), session-image identity and the OS manifest identity as recorded inputs |
| R4 | Stage 1 2.2 G9 | Add the L2t and L2u crates and the generators `hg-fontgen` and `hg-scriptgen` to G9's anchored list |
| R5 | Stage 1 7.2 | T0 runs Kani harnesses for changed `.hgfont` accessors; T1 runs split-and-compare and caches-off (TX-6, TX-31), with their CI budget |
| R6 | Stage 1 13.2 "Everyday features" | Find in page uses `hg-unicode` case folding and normalization; print-to-PDF uses font subsetting with `fsType`; spellcheck's process placement (C21, C23, C6) |
| R7 | Footprint 6.4 class table and "Sharing" rule | Shaping cache, strike cache and face working state are class (a); the cross-process sharing rule ("only what Izcal built itself") gains an explicit exception for OS font files on integrity-protected volumes, with C3's immutability condition |
| R8 | Footprint 6.3 | Session image: owner (`hg-image` versus `hg-font-service`), format, the build-time Izcal-set part in the engine image inventory (signed, SEC-16), and its start-up budget (C4, C15) |
| R9 | Footprint 6.18.1 and FP-36 | Add the strike cache, instance outline cache and lazy-validation state as closed channels (C5); give the font-file residency channel a bound (C10) |
| R10 | Footprint 6.6 and FP-14 | Separate RGBA sizing for emoji-dense pages (C16) |
| R11 | Footprint FP-25 and 6.12 | Installer and installed-size lines for the font set, or the separate signed data package option (C14) |
| R12 | HTML/CSS parsing and style spec 9.5 | The first-available-font metrics query and font-install mutation-log entries (C20); `text-transform` case mapping in `hg-unicode` (C21); font-size quantization stated as a used-value rule, not a computed-value one (C19) |
| R13 | Networking spec (NW-D6) | `hg-compress` serves WOFF2 in the renderer with a decompression budget; IFT patch fetches are partitioned subresources of destination `font` (C32) |
| R14 | Security scope IPC-R7, IPC-R17, GPU-service threat model | Mask transfer with receiver-fixed size; bounded path format for large glyphs; the session image treated as untrusted input (C4, C8) |
| R15 | Security scope broker threat model and 6.7 | No page-controlled string reaches an OS text API in the broker; inventory OS frameworks linked by the broker (C7) |
| R16 | Security scope shell-renderer threat model (6.6a, ATK-2) | Title length caps and the shell text path's fuzz bar (C12) |
| R17 | Security scope utility row (6.3) and TB-6 entry points | The font service parses no font bytes per session if C4 (2) is adopted; "outputs normalized (glyph outlines)" no longer describes fonts |
| R18 | Coding standard tier table and G18 | Tier for `hg-ucd` and `hg-unicode` if the broker links them; G18's scope covers font-derived values (C30) |
| R19 | Stage 1 D-S1 and HGBA-3 PRIV-2 | Pre-noise canvas text comparison; locale and setting variants in residual entropy; `local()` probes (C17, C18); OS-family rather than OS-version granularity (C1) |
| R20 | HGBA-3 DET-1 scope | State that DET-1's text scope is the Izcal set plus web fonts, and that GPU-path glyphs are outside it (C2, C22) |
| R21 | Future graphics spec (replacing HGBA-2 4.4) | Compositing color space for text, the bounded path format, the GPU outline tolerance, and CPU canvas text compositing (C22) |
| R22 | Future layout spec | The 1/64 local-unit numeric contract and the `reshape_on_break` contract (C19, C24) |
| R23 | Shell UI design 4.10 | "System UI fonts" means the OS tier's UI face on macOS and Windows and the Izcal UI face on Linux (answers Q-TX-8); shell strings follow C7's rule |
| R24 | Stage 1 5.8 semantic plane | Per-character bounds and word boundaries from `hg-text` for AccessKit (C37) |
| R25 | Stage 1 8.3 and the security scope's staffing | Remove double counting between the text tracks, the JS Intl and Unicode track and the security track (C27) |
