# Izcal resource footprint: independent design critique (phase 8)

Reviewed 19 October 2026. Subject: `reports/Izcal resource footprint.md`, draft 0.1 (the "FP spec"). The reviewer did not write it.
Checked against: `reports/Izcal JavaScript engine.md` draft 0.1 (the "JS spec", written after the FP spec), HGBA-2 2.3.0 (P1–P6, M-1, I-1, I-6, 4.6, S-3, X-4), HGBA-3 (B1, B4, B7, B9, B10, DET-1, DET-2, INC-3, D8), the Stage 1 design (Section 1, Section 6, 8.1, 13.3, BF-1), the DevTools spec 0.3, the security scope (worktree `security-scope`, revision 0.3: 6.3, 6.3a, 6.6 IPC-R1/R7/R15, 6.7, 6.9, 6.10, 6.13, 6.15), and the coding standard (worktree `cs-amendments`, 0.3.1: CS-PF-1 to CS-PF-8, CS-T12).
Owner direction applied as binding: no engine libraries (M-1 part 1); the best design from first principles, with no shortcuts for time or complexity; staging only where a first-principles argument favours it, never for schedule alone; security (P1, I-1) is never traded for footprint.

Severity: **High** means the spec contradicts a binding rule or another spec, or a target cannot pass as written. **Medium** means a gap, an unsupported claim or a missing alternative that changes a decision. **Low** means a local fix.

## Findings

### High

**H1. One copy of the DOM: FP-D2 and the JS spec's JS-D4 are two incompatible designs, and each one rejects the other.**
- *Section:* FP 6.4 "One copy of the DOM", 6.19 FP-D2, 8.3 FP-11, 9.4 (HGBA-3 B4 amendment).
- *Problem:* FP-D2 stores node structure in **columnar arenas indexed by a 32-bit slot**, with the **JS wrapper created lazily** on first script access, and retires the data-plane mirror. The JS spec decides the opposite on both points. Under JS-D4 the node *is* the JS heap object, with tree links as traced fields and a shape word in every node, and "a lazily created wrapper per node" is JS-D4's **rejected** option (b). The JS spec also keeps the mirror: "Style, layout and the semantic plane never read the heap; they read the data plane (B4)", "B4's data plane unchanged" (JS 6.5, 6.16), and its S-4c amendment says "phase workers never read `hg-heap` memory". FP-D2, by contrast, has workers read the DOM columns during a flush, and those columns are "traced by the one collector as part of the node". So under the JS spec the DOM still exists twice (links in the heap node, plus links, tag, state, attributes and style reference in the data plane). FP-11's 48 B and 512 B per node, and the HGBA-3 B4 and INC-3 amendments, assume a design the JS spec does not adopt. Each spec's reversal trigger also measures the other spec's design: FP-D2 waits on "the JS engine spec shows that tracing columnar node storage costs more", and JS-D4 waits on "per-node bytes > 10%† above the wrapper design".
- *Evidence:* FP 6.4 and FP-D2; JS 6.2 "DOM nodes are JS objects (JS-D4)", 6.5 "One heap per agent", 9.5 S-4c row, 6.17 JS-D4 row; HGBA-3 B4 lists three reasons for the mirror that do not need GC ownership.
- *Required change:* make **one** decision record with one ID, owned by the JS spec, because that spec owns the heap. Measure it with one spike (JS-M0's per-node spike) whose metric counts **every** copy of node structure, the data plane included. The JS spec must state whether the B4 data plane survives JS-D4, and if it does, justify the second copy of the links from first principles. The FP spec turns FP-D2 into requirements on that record (total bytes per node across all copies, one copy of the links, workers never trace or write) and stops choosing the wrapper strategy itself.

**H2. The FP spec budgets one heap per process. The JS spec has one heap per agent.**
- *Section:* FP 6.1 (F_fixed), 6.3 line items, FP-4, the 6.4 sharing table, 6.10 FP-D3.
- *Problem:* In the JS spec, each agent (each window event loop, dedicated worker and worklet) owns its own agent heap, `Mutator` and 16 GB† cage, and "heaps do not mix" (JS 6.0, 6.5 step 6, 6.12). Same-site tabs in separate browsing context groups are separate agents. Four FP claims follow from the wrong model:
  - **FP-D3's benefit.** Same-site tabs do not "pay F_fixed once". The 1 MiB† "JS engine runtime and one realm" line is paid per agent heap: nursery, root scopes, per-heap stub cache, release list and the startup-image mapping at the start of each cage.
  - **FP-4.** The 128 KiB† per extra same-site frame holds only for frames in the **same** agent (iframes, opener popups). No budget covers a new agent heap in an existing process, which is what every extra same-site tab costs.
  - **The sharing table.** "Bytecode and compiled code: same-site realms in the process" cannot hold. Bytecode and IC programs are GC objects in one agent's cage, and no cross-heap reference exists.
  - **The split rule.** FP-D3 splits a (site, container) into a second process on a budget, but the JS spec puts a `SharedArrayBuffer` agent cluster in **one** process (JS 6.12). The split rule does not say it may never divide an agent cluster.
- *Evidence:* JS 6.0 ("Each agent … owns one agent heap and one `Mutator`"), 6.5, 6.12; FP 6.3, 6.4, 6.10.
- *Required change:* the FP spec adds a target for an **additional agent heap in an existing process**, restates F_fixed as the process part plus a per-agent part, and limits the sharing table to what one heap can share. It adds the missing alternative of process granularity per (site, container, agent cluster), and states that splits happen only at agent-cluster boundaries. The JS spec decides whether a per-process, read-only, same-site code space exists. I-6 permits same-site sharing, so that would restore the bytecode sharing FP counts on.

**H3. The reclaim ladder drops engine data that lives in the GC heap, and it unmaps stores that the running script thread may be reading.**
- *Section:* FP 6.4 "Discardable, rebuildable derived data", 6.7 ("never needs memory"), FP-12, FP-15.
- *Problem:* FP says that each derived store "lives in its own arena, so dropping it is unmapping its regions: no traversal, no allocation, no GC", and its list includes "bytecode, JIT code, inline-cache and feedback data". In the JS spec, bytecode, IC programs, shapes and closures are **GC objects in the cage** (JS 6.1 regions table), mixed with other objects in Immix blocks. Flushing them takes a GC, and that GC must also run at a deterministic safepoint (JS 6.9). That breaks FP's "no GC" rule on the reclaim path. JIT code sits in a code region reached through the code table, and its removal must be installed at a safepoint. Separately, the reclaim path runs on the **control thread** "while script runs a long task" (FP 6.3 threads table), and rung (4) drops derived stores of **visible** pages. Unmapping computed-style or fragment arenas that the script thread can be reading at that moment (`getComputedStyle`, layout queries) is a use-after-free. In Rust that is an `unsafe` hole that no type rule covers.
- *Evidence:* JS 6.1 regions table, 6.4 (deopt and invalidation at safepoints), 6.9 rows "Tier-up" and "GC completion"; FP 6.3 threads table and 6.7 rung (4).
- *Required change:* split derived data into two classes:
  - (a) stores in their own arenas that are never reachable from a running mutator. Off-thread unmapping is allowed only for frozen or parked pages, or after a bounded safepoint handshake with the script thread, and the spec gives that handshake a latency bound;
  - (b) heap-resident engine data (bytecode, feedback, IC programs, code). This is flushed by a GC at a deterministic point **before** pressure (the freeze trim, navigation away), never on the Critical path.

  The JS spec adds bytecode and feedback flushing, which it does not mention today (FP 9.4 asks for it). FP-15's failing-allocator test adds a script thread that is running during reclaim.

**H4. Downloading locales, hyphenation dictionaries and codecs when a page needs them creates a cross-site channel, a network leak and a determinism break.**
- *Section:* FP 6.12 ("locales beyond the user's downloaded on demand"; "Optional components: hyphenation dictionaries, extra locales, rarely used codecs: on-demand components").
- *Problem:* If a page can trigger the download of shared installation state, then:
  - (1) **site B can probe whether site A caused it**. A first `Intl` or hyphenation call is slow or falls back when the data is absent and fast when it is present, which leaks the user's history across sites. This is the HTTP-cache partitioning problem again in a cache that I-1(b) does not partition;
  - (2) the update server and network observers learn which locales and codecs the pages used;
  - (3) the set of installed components becomes a fingerprint;
  - (4) web-visible output (`Intl` formatting, `hyphens: auto` layout, `canPlayType`) depends on the installation state. That breaks P2 and DET-1, and contradicts JS 6.9's rule that "CLDR, UCD and tz versions are part of the engine build and recorded".
- *Evidence:* FP 6.12; HGBA-2 I-1(b) partitioned-state list; JS 6.9 "Intl data" row and 6.13 Intl; HGBA-3 PRIV-2.
- *Required change:* no web-observable data is ever fetched because a page asked for it. Either ship it, or fetch it from the user's settings (UI languages) independently of page behaviour, and record the installed set as part of the engine configuration. Decide JS Q-JS-6 jointly under this rule. FP-25's installer budget follows from that decision, not the other way round.

**H5. "Thin launchers over one shared library" defeats the jitless executable.**
- *Section:* FP 6.12 Code row ("The JIT and jitless executables are thin launchers over one shared library, so code is on disk once").
- *Problem:* Security scope 6.7 makes the JIT tiers **separate executables**, and the JS spec's **G14** requires that `hg-renderer-jitless-main` "do not link" `hg-codegen`, `hg-jit-baseline`, `hg-jit-opt` or compiled regex, "checked on the built binaries". If both launchers load one shared library, the jitless process maps the code generator. The security property that "the jitless tier is enforced by what the executable contains" (JS 6.8, 6.15) is lost, and the Windows ACG and CET argument for jitless renderers loses its basis. This trades P1 for an install-size gain.
- *Evidence:* JS 6.8 ceiling table, 6.15 G14; security scope 6.7 "JIT tiers are separate executables".
- *Required change:* put the code generator in a library (or the executable) that only the JIT renderer loads, and keep everything else shared. Have G14's binary check run on the loaded-library closure, not only on the executable. Restate FP-25's 120 MiB† with that layout.

**H6. The broker parses fonts and filter lists, against IPC-R15.**
- *Section:* FP 6.3 rule 5 (session image: "System font list and fallback tables … The broker builds them once per session"), 6.11 ("compiled at list-update time, in the broker or a utility process").
- *Problem:* Font fallback tables need the coverage (`cmap`) and name tables of every installed font. User-installed fonts are attacker-reachable input (security scope 6.5 lists fonts as utility-process entry points). Filter lists are downloaded third-party text. IPC-R15 says "No parsing of web formats in the broker, even though it is Rust", and 6.3 states the broker's design goal as "no web-format parsing". The research note recommended the broker as the builder (`compact_engine_data.md` 9, Implications), and the spec carried that over.
- *Evidence:* security scope 6.3, 6.6 IPC-R15, 6.10; FP 6.3 rule 5, 6.11.
- *Required change:* build the session image and the compiled filter automaton in a utility process at the strictest tier. The broker only passes sealed, read-only handles (IPC-R1). Add both jobs to the utility row of 6.3a and to the utility threat model.

**H7. One glyph atlas shared by all sites breaks the spec's own "no cross-site content-derived sharing" rule.**
- *Section:* FP 6.6 "Shared atlases: One glyph atlas per device, shared by all tabs; its contents are glyph rasters of fonts, not page content".
- *Problem:* Web fonts are bytes the page supplied, and the page's text decides which glyphs are rasterized. A shared atlas therefore deduplicates content-derived data across sites, which FP 6.4 forbids ("no deduplication of anything a page supplied") and which P1 and I-6 exist to prevent. Atlas occupancy and eviction are also a cache-timing channel across sites, from frame timing or GPU timing (GPU-R7 coarsens only WebGPU timestamps). The design is WebRender's (`compact_engine_data.md` 7.4, 9.2: "which WebRender already does"), so it is also a template (M-1 part 3).
- *Evidence:* FP 6.4 sharing rule, 6.6; HGBA-2 I-6; security scope 6.9 GPU-R7, 6.15.
- *Required change:* partition glyph atlases per (site, container), at least for web fonts and preferably for everything. Budget the atlas per site, not per device, and add the partition to the GPU service threat model. Footprint does not justify a P1 exception.

**H8. FP-24 promises bit-identical results across macOS, Linux and Windows. The reference fleet spans two ISAs, and the JS spec guarantees identity only per ISA.**
- *Section:* FP 6.2 "Canonical bytes", 6.16, FP-24.
- *Problem:* JS-50 promises identity "across macOS, Linux and Windows **on one ISA**", and JS-D12 excludes NaN bit patterns and relaxed-SIMD results across ISAs. The reference devices are Apple silicon (AArch64) and x86-64 Linux and Windows machines (FP 8.1). Canonical bytes also include machine-code commits, whose sizes differ per ISA, and JIT code exists only under some tier ceilings. As written, FP-24 cannot pass, and it is the PR gate (+0.5%†).
- *Evidence:* JS 6.9 last rows, JS-50, JS-D12; FP-24, FP 8.1.
- *Required change:* state FP-24 per ISA, and per tier ceiling or with code measured in canonical units. Either define canonical bytes so they exclude ISA-dependent quantities, or run the gate per ISA. Record the excluded classes the JS spec already names.

**H9. Staging decided by schedule, which the owner direction forbids, and a staging conflict with the JS spec.**
- *Section:* FP 9.1 "Suspend-to-disk spike (FP-S1) … if that is not ready by FP-M5, the spike moves to Stage 2"; 6.14 and FP-D7 "user budgets in Stage 2".
- *Problem:* (a) FP-S1's fallback is pure schedule: "not ready by" a milestone. (b) FP-D7 puts per-site user budgets in Stage 2 with no first-principles argument. The Stage 1 design (Section 6, "Per-site resource meter") says interventions "are engine defaults, not user budgets", so whether user budgets should exist at all is undecided, and FP-D7 settles it by deferring it. (c) The JS spec stages live tab restore from heap images in **Stage 3, "after measurement"**, in `hg-js-snapshot` (JS 6.11), while the FP spec runs a Stage 1 spike in a separate crate, `hg-suspend`. The same capability has two owners and two stages.
- *Evidence:* owner direction; JS 6.11 use table, 9.1 "Live tab restore … follow measurement"; Stage 1 design Section 6.
- *Required change:* give one owner (the `hg-js-snapshot` format, with `hg-suspend` as its lifecycle client) and one stage with an argument. The argument that holds is measurement: the spike *is* the measurement JS 9.1 asks for, so it belongs before the JS spec's restore stage, not after it. Replace the schedule fallback with FP-31's kill criteria alone. For FP-D7, either argue from first principles why user budgets wait (for example, that their UX needs FP-30's attribution accuracy first, which makes it a measurement gate), or remove the staging.

### Medium

**M1. The HGBA-2 4.6 "custom persistence layers" ruling is asked only for suspend files, but the JS spec adds three more on-disk formats.**
- *Section:* FP 6.9 "Persistence class", 9.4 HGBA-2 4.6 row, 11.2 conflict.
- *Problem:* The JS spec's code cache (Stage 1), B1 checkpoints and DevTools heap export are custom on-disk formats too (JS 6.11). FP also adds its own code cache budget (6.12). Read narrowly, "cache, not storage" exempts all of them, but the class is not defined.
- *Required change:* one amendment to HGBA-2 4.6 that defines a **regenerable-cache class**: authenticated, versioned by engine build, discarded and never repaired on any failure, never the only copy of user data, partitioned per (site, container). It names every member (suspend files, code cache, heap images, compiled filter automata). The owner rules once.

**M2. "No escalation" is wrong. A compromised renderer can persist through a suspend file or code cache.**
- *Section:* FP 6.9 Security ("can only attack a later process of the same site, which is no escalation"), 6.12.
- *Problem:* A one-time renderer compromise that writes a crafted suspend file or code-cache entry survives process exit, browser restart and the closing of the exploit page. The attacker gains persistence for that site until the file is deleted. AEAD under an OS-bound key does not help, because the renderer that writes the file is the attacker. The JS spec treats every image as hostile input (JS 6.11), which addresses parsing but not persistence.
- *Required change:* state persistence in the renderer threat model and bound it. Images are tied to the engine build (the JS spec already discards across builds). Discard every image written by a process that later crashed, failed a check or was killed for a bad message. Set a TTL. Restoring an image rewards nothing a fresh load would not.

**M3. The read-only image clashes with the JS object model at three points: mutable shape transitions, mark state, and hash seeds.**
- *Section:* FP 6.3 rules 2–3; JS 6.1 "Static roots", 6.2 "Shapes", 6.9 hash-seed row, 6.11.
- *Problem:* JS shapes are "immutable heap object[s] holding … a transition table", but adding a property to an object whose shape lives in the image writes that table. The JS spec also says nothing about how image objects are marked, about validity cells registered on image prototypes, or about IC state for image objects, all of which FP rule 2 moves to per-process side tables. Any hash table baked into the image at build time carries a build-time seed that is the same for every user. JS 5 itself cites the Node CVE-2017-11499 lesson ("Restored state must re-derive its entropy"), and the JS spec keys hashes per process (CS-AS-2).
- *Required change:* the JS spec adopts FP rule 2 explicitly: transition tables, validity cells, IC state and mark bits for image objects live in per-heap side tables, and image objects are immortal and never traced into. Image hash structures are perfect hashes over fixed key sets only, and any table that grows at run time is a per-process table under the per-process key.

**M4. `oom` crash reports to sites leak device pressure, which FP 6.15 itself forbids.**
- *Section:* FP 6.7 "Runaway pages" and 6.15 table rows "Crash reports with `oom` reason" and "Device, battery or pressure state: Never".
- *Problem:* A site killed at Emergency rung (7) because **other** tabs used memory learns, on its next load, that the device was under pressure. That is cross-site information and a device-state signal.
- *Required change:* report `oom` only when the page's own process crossed the runaway threshold. Kills by the ladder report nothing, or something indistinguishable from navigation.

**M5. Shared budgets and shared pages are cross-site channels, and 6.18 does not analyse them.**
- *Section:* FP 6.3 rules 4–5, 6.10 bfcache, 6.12 suspend quota, 6.7 ladder, 6.3 broker worker cap, 6.18.
- *Problem:* (a) The bfcache budget is global, and eviction is visible to a page through `pageshow.persisted`, so one site's footprint affects whether another site's entry survives. (b) The suspend quota (≤ 1 GiB†, ≤ 2%† of free disk) is global too. (c) The ladder's global decisions change one page's rebuild latency according to other sites' memory use. (d) The broker caps runnable workers across processes. (e) Image and session-image pages are shared clean file pages. Major-fault versus minor-fault timing tells whether **another** process recently touched a rarely used page, such as one locale slice or the descriptor of one WebIDL interface (the page-cache attacks of Gruss et al., CCS 2019, were not in the notes). FP 4.3 treats read-only sharing as safe without this analysis.
- *Required change:* add a "shared-resource channels" subsection to 6.18 and to the renderer threat model. Per-(site, container) sub-budgets for bfcache and suspend, quantized ladder effects, and prefaulting or populating small image sections at install or start are candidate mitigations. Each channel is either closed, or bounded and recorded. None is traded away for footprint.

**M6. Attribution and the intervention log are browsing history, and they sit in the shell renderer.**
- *Section:* FP 6.14.
- *Problem:* Per-site CPU, energy and memory over "the last minute, hour and day", per-script third-party origins, and a timestamped intervention log ("frozen at 14:02") together reconstruct browsing history. The spec sets no retention, says nothing about clearing with history, private windows or per-container separation, and does not name a holder. The views are shell UI. The shell renderer is "untrusted; cross-site by design" (security scope 6.3), and it currently holds tab titles, not per-site script inventories and activity history.
- *Required change:* define retention, clearing with history and site data, per-container scoping and private-window behaviour. Keep the history in the broker or storage service and send the shell only what is on screen. Add the asset to the shell renderer's row in security scope 6.3.

**M7. The heavy-frame intervention copies Chrome's shape, its target is circular, and its firing cannot be replayed.**
- *Section:* FP 6.14, FP-29, FP-D7.
- *Problem:* (a) It acts per frame, as Chrome's heavy-ad intervention does. The JS spec notes that Izcal can attribute cost to individual **scripts**, "unlike Chrome's frame-level heavy-ad intervention" (JS 6.8). "Like nothing before" points to a script-level design, which the alternatives do not consider. (b) The thresholds are set at "the 99.9th percentile of ad frames on the corpus", and FP-29 then requires firing on "≤ 0.1%† of ad frames on the corpus". On the same corpus that holds by construction. (c) The thresholds use real CPU seconds, which are not deterministic, so whether the intervention fires is not reproducible under B1 unless the decision is recorded as an input. "Seeded noise (replayable under B1)" covers only the noise.
- *Required change:* add the script-level alternative with its own reversal trigger. Measure FP-29 on a held-out corpus or in the lab against interaction ground truth. Record every intervention decision as a B1 input, as the JS spec already does for pressure signals.

**M8. Constants and shapes taken from other browsers (M-1 part 3).**
- *Section:* FP 6.2 (Linux charged = `RssAnon + VmSwap`, which this critique matched to Chromium's private memory footprint from an engine-source reading that is no longer evidence, M-1 part 3), 6.10 (3 entries per tab = Firefox's cap; 10 min TTL = Chrome's 600 s; the spare process and its low-memory skip = Chrome; one process per site = Firefox `webIsolated`), 6.7 (a free-disk floor "as Chromium does", a value later removed at 0.3.1 because it came from engine source; the kill ranking "Android exposes the same idea through Chrome's ranking"), 6.8 ("The reasons mirror Chrome's exemptions").
- *Problem:* Each is labelled as evidence, but none is derived. Where Izcal's number equals the incumbent's, the † does not show that a first-principles derivation exists. The owner asked for a design "like nothing before".
- *Required change:* for each one, add a one-line derivation from Izcal's own model, or name the FP-M milestone that will set it from measurement (with the incumbent's value as the prior, stated as such). Derive declared reasons from user-visible activity classes rather than from Chrome's list. That would also catch M10's gaps.

**M9. The FP-D8 reversal trigger trades isolation for footprint, and FP-8 cannot be tested as stated.**
- *Section:* FP-D8, FP-8.
- *Problem:* FP-D8 reverses origin keying (a P1 property, HGBA-2 I-1(c)) if "FP-7 fails with origin keying on". That turns isolation off because of a memory target. FP-8 promises reduced isolation is "never needed on devices ≥ 2 GB†", but the lab's smallest device is 4 GB (FP-D12).
- *Required change:* the FP-D8 trigger becomes "reduce F_fixed and content duplication". Isolation is never the lever. Either add a 2 GB device or emulate one (accepting FP-D12's caveat), or move FP-8's claim to 4 GB.

**M10. Freeze by default would freeze agents, automation and DevTools recordings.**
- *Section:* FP 6.8 declared reasons, 6.17 Agents row, 9.4 HGBA-3 B7 amendment.
- *Problem:* The list leaves out an agent task in progress (B9: agent tabs are hidden and active), a WebDriver or automation session (WPT runs in background windows), a DevTools recording or attached session, muted Picture-in-Picture video, and a SharedWorker or BroadcastChannel serving a visible same-site page. The B7 amendment "frozen subsumes pixel-optional" is wrong for agent tabs, which must keep running unpainted.
- *Required change:* add these reasons, each with a budget. Keep B7's pixel-optional mode for hidden **running** pages.

**M11. FP-D1 says `posix_spawn`, which cannot set up the Linux sandbox that security scope 6.7 requires.**
- *Section:* FP 6.3 "No fork server", FP-27, FP-M0.
- *Problem:* Security scope 6.7 gives the renderer new user, PID, network and mount namespaces. `posix_spawn` creates none, and a PID namespace for the child must come from the parent's `clone`/`clone3`. The cited advantage (no copy of the broker's page tables, through `CLONE_VM | CLONE_VFORK`) has to be re-shown for the real spawn path. On macOS and Windows, spawn-to-ready also includes `sandbox_init`, the AppContainer and the lockdown token.
- *Required change:* specify the spawn mechanism per OS with the full sandbox. FP-M0 measures FP-27 on that path, not on a bare spawn.

**M12. The FP-2 ship blocker comes before the floor it depends on, and two line items do not match the JS spec.**
- *Section:* FP 6.3 line-item table, FP-2 "If it fails".
- *Problem:* (a) "Ship blocker … if over 6 MiB†" is fixed before FP-M0 measures the OS and sandbox floor, which Izcal does not control. The best published figure is 11.5 MB (Firefox on Windows), and the JS spec's own evidence puts an isolate at about 3 MB (JS 4.4, GO 10.1). (b) The 1 MiB† "JS engine runtime" line does not budget the JS spec's copying nursery. (c) The script thread's "8 MiB reserve … deep JS recursion" contradicts JS 6.6: all JS and Wasm run on **engine-managed stack segments** with a canonical frame budget, not on the OS thread stack.
- *Required change:* set the blocker as the FP-M0 measured floor plus Izcal's line items, and add an explicit nursery line (committed size at idle, decommitted when empty). Replace the stack row with the engine segment's initial commit, plus a small OS stack for the Rust-only frames.

**M13. Region-granularity ledger tags cannot attribute JS heap bytes per script.**
- *Section:* FP 6.2 ledger, 6.14 ("Which frames and which third parties, by script origin"), FP-30, risk table ("Tags at region granularity").
- *Problem:* One agent heap's Immix blocks mix objects from every realm and script, so a per-region tag cannot say which script owns a byte. The JS spec offers allocation-site provenance only as a heap query in a **replay host** (JS 6.10).
- *Required change:* specify the live mechanism jointly with the JS spec: per-allocation-site counters in canonical units, sampled allocation tracking, or per-realm block segregation with its fragmentation cost. Say what live attribution can and cannot claim. FP-30's 95%† depends on that answer.

**M14. Three targets cannot be measured as written.**
- *Section:* FP-33, FP-21, FP-22.
- *Problem:* (a) FP-33 predicts charged memory within ±10%† from **commit** granules. On Linux and macOS, charged memory counts touched or dirty pages, not commits (FP's own 6.2 definitions), so the two diverge by design wherever commit runs ahead of touch (nursery, stacks, reservations). (b) FP-21's 1.02× idle power is below the noise of the named meters, and no run count or confidence interval is given. (c) FP-22 and JS-26 say "non-increasing release over release" without a noise band.
- *Required change:* define canonical bytes as canonical **touched** granules, or predict Windows commit only. Give FP-21 a protocol (runs, interval, meter resolution). Give the energy targets a noise band, as the JS spec's Section 8 preamble does for its own metrics.

**M15. DOM text and JS strings are two copies under the JS spec. FP requires one.**
- *Section:* FP 6.4 "Text"; JS 6.2 (DOM "text … in side tables outside the cage"), 6.13 (strings in the cage).
- *Required change:* a joint rule: either DOM text lives in the cage as engine strings that workers read through an immutable published-string view (JS 6.13 says published strings never change representation), or the binding layer shares the buffer without copying. Budget the copy rate as FP proposes.

**M16. A missing alternative: let the OS page out frozen processes. The charged metric would not count it.**
- *Section:* FP 6.7, 6.9, 6.19, FP-6.
- *Problem:* HGBA-2 4.6 names `MADV_PAGEOUT` and "SSD-backed swap and serialized state" as tiers. Paging out a frozen process restores without reload and needs no new file format, serializer or attack surface. It is not among the alternatives to FP-D6. Because charged memory on Linux includes `VmSwap` and Windows commit includes paged-out memory, FP-6 could never credit it, so the metric chooses the design.
- *Required change:* add the alternative with its cost and reversal evidence. Report resident RAM next to charged memory for frozen tabs, so the comparison with FP-S1 is fair.

**M17. Changes to other documents that Section 9.4 does not list.**
- *Problem:* (a) FP-D11 decodes images in the site process. The Stage 1 design places images in utility processes (8.1 M7: "Images, fonts, media and PDF via utility processes"), and Q-SEC-13 asks **which** decoders, not where they run. (b) Security scope 6.3 and 6.3a need rows for the spare process, the filter-list and session-image utility jobs (H6), the measurement-only co-locating build (L6), and the shell renderer's new attribution asset (M6). (c) IPC-R7 exempts opaque buffers only when "the **receiver** fixed their size and format at creation". FP 6.5 says the region is "broker-allocated", so it must say that the GPU service fixes the format.
- *Required change:* add each one to 9.4. Decide FP-D11 jointly with the security owner.

**M18. Numbers the JS spec left to the FP spec are not supplied.**
- *Section:* JS-30 ("thresholds come from the resource-footprint spec"), JS 6.5 pacing ("bounded by the budgets of the forthcoming resource-footprint spec"), JS 6.8 ("The same grant carries the heap limit and a CPU budget per renderer … numbers come from the forthcoming resource-footprint spec"), Q-JS-6 (locale data budget).
- *Problem:* The FP spec gives process-level budgets (FP-5) and runaway thresholds, but no engine heap threshold per page, no heap-growth knob, no heap-limit or CPU-budget capability, no behaviour when an agent heap reaches its limit (P3: degrade the least important page), and no locale budget.
- *Required change:* add a section that answers each request with an ID.

**M19. The node-size evidence contradicts itself.**
- *Section:* FP 6.4 "One copy of the DOM".
- *Problem:* The section cites the spike's measured data plane of **150–175 B per node** and, a paragraph later, "Izcal's current two-copy estimate of **95–115 B**". The second number is a researcher's estimate from member lists (`compact_engine_data.md` 1, Implications), and it is lower than the measured data plane alone. HGBA-3's measured toy DOM was about 430–440 B per node, plus 150–175 B for the data plane.
- *Required change:* use the measured figures, label the estimate as an estimate, and recompute FP-11's comparison.

### Low

**L1.** FP-D9 and FP-D12 have no reversal trigger ("—"). Every decision needs one: for FP-D9, for example, "a privacy-preserving aggregation scheme passes PRIV-review"; for FP-D12, "emulation reproduces pressure behaviour within X% of the real device".

**L2.** The scenario name **S-30** uses HGBA-2's `S-` target prefix (S-1 to S-8, S-3). Rename it, for example to FP-S30, so the claim in 8 that "FP- does not collide" covers every new ID.

**L3.** FP 6.15 says other origins' frames are "shown as totals only, as DevTools rules already require (DevTools spec Section 3)". DevTools 6.5 rule 1 makes out-of-set frames **opaque** for scoped principals, and Section 3 states no totals rule. Cite correctly, and say whether totals leak for non-`devtools-user` principals.

**L4.** "Empty spans returned … by a substrate timer" (6.3) and "exits after 10 s† idle" (threads table) must be one-shot timers armed by activity. Periodic timers would contradict FP-17, FP-18 and 6.8's "no periodic timers of its own".

**L5.** "Opt-in aggregates" (FP-D9) has no aggregation or privacy mechanism. Specify it, or point to a privacy spec.

**L6.** The measurement-only co-locating build (6.10, FP-7) breaks I-1 by design. Exclude it with a build-graph rule checked on release binaries, as G14 does, not by convention.

**L7.** CS-PF-5 and CS-PF-6 cite RES-3 and RES-*, which FP replaces. CS-PF-6 also says the global allocator is chosen "in the perf lab … and recorded as decisions", while FP-D5 chooses before any measurement. Add both to the 9.4 coding-standard row, and make FP-D5's first lab comparison its confirmation.

**L8.** HGBA-2 4.6 classifies tabs as hot, warm, cold and frozen. FP introduces visible-active, visible-idle, hidden, frozen, suspended and discarded. The 9.4 amendment for 4.6 should map one set onto the other.

**L9.** The global wake grid (6.8) must be a fixed function of time and of the process's own timers, never derived from other processes' demand. State this so the grid cannot become a cross-process signal.

**L10.** The session image's font list must not expose more fonts than the PRIV-2 font normalization allows a page. State that the per-page view filters it.

## Reconciliation with the JS engine spec

Each row names the conflict and the document that should change. "Both" means a joint decision, recorded once.

| # | Topic | FP spec says | JS spec says | Which document changes |
|---|---|---|---|---|
| R1 | DOM representation (H1) | Columnar arenas, lazy wrappers, mirror retired (FP-D2) | Node is the heap object, no wrapper (JS-D4); B4 data plane kept; workers never read `hg-heap` | **Both**: one decision record owned by the JS spec; FP-D2 becomes requirements; the JS spec states and justifies whether the data plane survives |
| R2 | Heap granularity (H2) | One heap, one realm and one F_fixed per process; bytecode shared across same-site realms | One heap and one cage per **agent**; heaps never mix | **FP** restates F_fixed, FP-4, the sharing table and the FP-D3 split rule; **JS** decides whether a per-process same-site read-only code space exists |
| R3 | Startup image identity and placement | One `hg-image` file (Unicode, name tables, UA sheet, JS builtins, WebIDL descriptors), mapped once per process at a random address | `hg-js-snapshot` startup image mapped at the start of **every cage**, including Izcal UI AOT bytecode; CLDR in a separate `hg-cldr-build` format | **Both**: one image inventory (files, owners, per-process-kind contents). The shell's UI bytecode should not go into site renderers; the size budget covers all files |
| R4 | Mutable state for image objects (M3) | Rule 2: mark bits, IC state and refcounts in per-process side tables | Shapes hold transition tables; static roots; no statement on marking or validity cells for image objects; per-process hash key | **JS** adopts rule 2 and the immortal-image rule; image hashes perfect or re-keyed |
| R5 | Substrate memory traits | `Memory`: reserve, commit, decommit, purge, protect, `map_image`, `map_shared`, each with a ledger tag | `VirtualMemory` (reserve, commit, protect, decommit) plus `CodeMemory` | **Both**: one trait set in one Stage 1 3.1 amendment, with ledger tags and `purge`; `CodeMemory` kept as the W^X window |
| R6 | Canonical accounting | Canonical **bytes** = commits rounded to 16 KiB granules | Canonical **allocation units** per operation, fixed by its description, for GC triggers | **Both**: define how they relate; engine-internal derived allocations (lazy compile, recompiles after discard, IC programs) never count toward GC debt, otherwise H3's discard and rebuild shifts GC timing and `WeakRef` clearing |
| R7 | Determinism scope (H8) | Bit-identical across macOS, Linux and Windows (FP-24) | Identical per ISA; NaN and relaxed SIMD excluded across ISAs (JS-50, JS-D12) | **FP** restates FP-24 per ISA and per tier ceiling |
| R8 | Discardable engine data (H3) | Bytecode, JIT code, IC and feedback are derived stores in their own arenas, unmappable without GC | Bytecode, IC programs and shapes are cage objects; code is installed at safepoints; no bytecode flushing specified | **Both**: two classes of derived data; the JS spec adds flushing at deterministic points; the FP ladder never flushes heap data on the Critical path |
| R9 | Memory-return hooks | Freeze trim, FP-9 "returned within 10 s", ≤ 12 MiB† frozen page, no periodic timers | GC pacing by allocation debt only; idle work in frame gaps; no hook for shrinking on freeze or GC on navigation away | **JS** adds shrink-on-freeze (final GC, nursery decommit, empty-block release) and a navigation-away hook; **FP** schedules them outside pressure |
| R10 | Threads | Two threads at idle; one shared worker pool (style, layout, decode, GC helpers, JIT) of min(cores − 1, 4)†; script thread with an 8 MiB OS stack | Background compilation, concurrent marking helpers, off-thread parse; a compiler thread holds the W^X write window; `[[CanBlock]]` agents get their own thread; JS runs on engine stack segments | **Both**: the pool serves engine work, but the code-writer role is a dedicated thread identity (`MAP_JIT` toggling is per thread), never one that also runs image decoders; FP-32 counts agent threads; FP's stack row follows JS 6.6 (M12) |
| R11 | Engine heap limits and CPU budgets (M18) | Process-level runaway and backstop thresholds only | Broker-granted heap limit and CPU budget per renderer; JS-30 thresholds and the growth knob are left to FP | **FP** supplies the numbers, the behaviour at the limit and the locale budget (Q-JS-6) |
| R12 | Live-page snapshot (H9) | `hg-suspend` spike in Stage 1 (FP-M4 to FP-M5); falls back to Stage 2 on schedule | Live tab restore in `hg-js-snapshot`, Stage 3 "after measurement" | **Both**: one owner (`hg-js-snapshot` format) and one stage gated by FP-31's measurement; the schedule fallback is dropped; FP's suspend design adds the JS spec's rule that entropy is re-seeded on restore |
| R13 | Persistence class (M1, M2) | Suspend files read as a cache | Code cache (Stage 1), checkpoints and heap export are on-disk images; images hostile, encrypted and partitioned | **Both** feed one HGBA-2 4.6 amendment that defines the regenerable-cache class and covers persistence of a compromise |
| R14 | Tier ceilings (H5) | One shared library for JIT and jitless; budgets do not mention ceilings | G14: the jitless binary does not link the code generator; ceilings per site, container and emergency switch | **FP** fixes the library layout; budgets FP-2, FP-5 and FP-22 are stated per ceiling (the banking container and Izcal-only renderers are jitless; an emergency switch to jitless changes energy) |
| R15 | Text (M15) | One text buffer, no copy at the binding | DOM text in side tables outside the cage; JS strings in the cage | **Both**: one rule; the JS spec owns the representation |
| R16 | Attribution and time accounting (M13, M7) | Thread CPU clock read per task; ledger tags at region granularity; frame-level interventions; FP-23 overhead ≤ 0.5%† | `hg-js-observe` time accounting per script, realm and origin (JS-23, ≥ 99%†); script-level caps; JS-60 overhead ≤ 0.5%†; no engine code reads an OS clock | **Both**: one accounting mechanism and one overhead budget; CPU clocks read through the substrate; intervention decisions recorded as B1 inputs; FP adds the script-level alternative |
| R17 | Energy and relative thresholds | FP-22 absolute, non-increasing; 8.4: other browsers are "never the threshold" | JS-26 "≤ 1.0×† the best shipping engine's at Stage 3"; JS-20, JS-21 and JS-24 relative | **Both**: one policy on relative thresholds; if relative targets stay in the JS spec, FP 8.4's sentence is limited to footprint |
| R18 | RES targets | RES-1 to RES-5 replaced (8.4) | JS-22 counts engine start inside RES-1; 9.5 says RES-1 to RES-3 are "reviewed against" the FP spec | **JS** cites FP-27 and FP-1 to FP-6 instead |
| R19 | Origin keying | FP-D8: on for every desktop class | 6.7: below the threshold, subdomains share a renderer and the engine relies on the same-origin policy | **JS** follows FP-D8 once it is decided (with M9's corrected trigger) |
| R20 | Reservations | Page-table line of 0.5 MiB† from "one heap reservation" | A 16 GB† cage per agent heap, plus a guarded reservation per Wasm memory; JS risk row on reservations | **FP** counts reservations and mappings per agent in the page-table line and lists the per-OS limits (mapping count, `RLIMIT_AS` if the sandbox sets it) |

**Points where the two specs agree** (no change needed): 32-bit references inside a process (FP 6.4 matches JS-D1); no periodic engine timers, with idle GC in frame gaps (FP 6.8 matches JS 6.5); OS pressure signals as recorded B1 inputs (FP 6.7 matches JS 6.9); site images treated as hostile, encrypted and partitioned per (site, container) (FP 6.9 matches JS 6.11); a separate `hg-alloc` for Rust and `hg-heap` for GC objects on one substrate (FP 6.3 matches JS 6.15 G13).

## Summary

| Severity | Count | IDs |
|---|---|---|
| High | 9 | H1–H9 |
| Medium | 19 | M1–M19 |
| Low | 10 | L1–L10 |
| **Total** | **38** | |
| Reconciliation items with the JS spec | 20 | R1–R20 (FP only 4: R7, R11, R14, R20; JS only 3: R4, R18, R19; both 13) |

**What the spec does well.** It makes the cost of a process the central lever for isolation. Its read-only, position-independent image is safer than a fork server. "Reclaim never needs memory" is tested with failing allocators. Every target names its denominator. It replaces relative RES targets with absolute ones and gates them deterministically. It also identifies its own conflicts with HGBA-2 4.6 and the Stage 1 "site instance" wording.

**What must change before the owner reviews it.**
- The two engine-facing designs, the one-copy DOM (H1) and the per-process heap model (H2), conflict with the JS spec and are decided twice.
- The reclaim ladder assumes heap data can be unmapped without a GC (H3).
- Four mechanisms weaken P1: page-triggered component downloads (H4), the shared library under the jitless executable (H5), parsing in the broker (H6) and the shared glyph atlas (H7).
- FP-24 cannot pass across ISAs (H8).
- Two items are staged by schedule (H9).

None of these needs a lesser design. Each is fixed by moving a mechanism to the right owner or process.
