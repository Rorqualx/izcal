# Izcal storage: independent fact-check (phase 8)

Checked 20–21 October 2026 by a fact-checker who did not write the spec or the research notes. Scope: the claims that carry the design in `reports/Izcal storage.md` (draft 0.1) — spec-semantics claims, cross-document quotes into `reports/`, historical dates/numbers/quotes, M-1 part 3 compliance, † discipline, and internal cross-references.

**Primary sources fetched this session** (normative text read directly):

- https://storage.spec.whatwg.org/ (quota rule, shed/shelf/bucket/bottle, endpoints table, "cleared in its entirety", session clearing, `persistent-storage`, storage-key note, pressure steps)
- https://www.w3.org/TR/IndexedDB/ (key ordering, silent index skip, durability tri-state, 2⁵³ generator, transaction modes, scheduling, commit trigger)
- https://w3c.github.io/ServiceWorker/ (HTTP-cache isolation, Cache.put reject conditions, opaque-response absence in put, registration persistence, 86 400 s staleness, byte-for-byte update comparison, cache lifetime note)
- https://fs.spec.whatwg.org/ (lock tri-state, sync-handle worker/bucket-FS restriction, write-on-close atomicity note, `getDirectory()`/bucket FS)
- https://www.w3.org/TR/webdatabase/ (WebSQL Note: abandonment date, independent-implementations sentence, SQLite 3.6.19 dialect)
- https://jakearchibald.com/2015/if-we-stand-still-we-go-backwards/ (AppCache postmortem quote)
- https://alistapart.com/article/application-cache-is-a-douchebag/ ("nothing updates, ever")
- https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-the-rise-of-first-party-sets/ (7-day deletion)
- https://www.usenix.org/conference/fast16/technical-sessions/presentation/lu (WiscKey abstract figures)
- ALFS PDF (research.cs.wisc.edu alice-osdi14.pdf, text extracted locally): 60 vulnerabilities, eleven systems, ALICE, OSDI 2014
- https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria (Firefox/Chrome/Safari quotas, page modified 2026-01-05)
- https://nolanlawson.com/2015/09/29/indexeddb-websql-localstorage-what-blocks-the-dom/ (jank numbers and the article's own correction)
- https://www.chromium.org/Home/chromium-security/site-isolation/ (renderer bug counts)
- https://github.com/xaitax/chrome-app-bound-encryption-decryption (ABE mechanism quotes, prose only)
- https://en.wikipedia.org/wiki/Gears_(software) (Gears dates and module list)

**Local sources read:** all six research notes in `research_notes/Izcal storage/`, and the quoted text located and compared in `reports/HGBA 2 browser architecture spec.md`, `reports/HGBA Stage 1 architecture design.md`, `reports/Izcal DOM, bindings and event loop.md`, `reports/Izcal resource footprint.md`, `reports/Izcal networking.md`, `reports/Izcal DevTools spec.md`, `reports/Izcal HTML and CSS parsing and style.md`, `reports/Izcal images and media.md`, `reports/Izcal content blocking.md`, `reports/Izcal extensions.md`, `research_notes/Phase 3 amendments/ledger.md`, `research_notes/Phase 2 reconciliation/reconciliation.md` and `reconciliation_2.md`.

Not re-checked this session (graded per the notes, none load-bearing enough to fetch here): the Brave 2 GiB vs ~149 GB DevTools disagreement, the Chrome DevTools localStorage doc's 2019-03-14 date, IndexedDB REC/2.0 dates and the 3.0 WD date of 13 Aug 2025, LevelDB impl.md figures, RocksDB write-stall triggers, evercookie's vector count, Firefox ~102 cookie-encryption claim (the spec does not rely on it), Safari's 1 GB/200 MB-step prompt figures.

---

## 1. Verdicts

### 1.1 Spec-semantics claims (checked against the normative text)

| # | Spec claim (section) | Verdict | Evidence |
|---|---|---|---|
| V1 | Quota "must not be a function of the available storage space on the device" (§2, §4, §6.5, SG-D4) | **Correct** | storage.spec.whatwg.org §6: "It must not be a function of the available storage space on the device", with the fingerprinting warning as quoted |
| V2 | Shed/shelf/bucket/bottle hierarchy, shed = map of storage keys to shelves, bottle per endpoint (§6.4) | **Correct** | Storage §4.3–4.7 definitions match ("a map of storage keys to storage shelves"; bottle "carved out for a single storage endpoint") |
| V3 | Endpoints `caches`/`indexedDB`/`localStorage`/`serviceWorkerRegistrations` (local) and `sessionStorage` (session); 5 MiB recommended quota for localStorage and sessionStorage, null for the rest (§6.4, §6.3) | **Correct** | Storage §4.1 table matches exactly |
| V4 | Whole-bucket eviction: "cleared in its entirety" (§2, §6.5, SG-D5) | **Correct** | Storage §7: "it must be cleared in its entirety" |
| V5 | Session storage cleared as traversable navigables close (SG-D12, §6.3) | **Correct** (semantics), citation wrong (see S4) | Storage §7.1: "Session storage buckets must be cleared as traversable navigables are closed" |
| V6 | `persistent-storage` permission name; note encouraging not asking twice (§6.11, SG-D6) | **Correct** | Storage §5: "`persistent-storage`"; note on not letting the user answer twice present |
| V7 | Storage key is origin-only, "expected to change" (§6.8, Q-SG-3, SG-D4) | **Correct** | Storage §4.2 note: "This is expected to change; see Client-Side Storage Partitioning" |
| V8 | Pressure sequence: clear best-effort, then inform the user and offer clearing (§6.5) | **Correct** | Storage §7: "should clear network state and local storage buckets whose mode is 'best-effort'"; then "should inform the user and offer a way to clear" |
| V9 | IndexedDB key order array > binary > string > date > number (§4) | **Correct** | IndexedDB §2.4: "Number keys are less than date keys. Date keys are less than string keys. String keys are less than binary keys. Binary keys are less than array keys." |
| V10 | Silent index skip: record stored without an index entry when key-path evaluation fails, exception not rethrown (§6.2) | **Correct** | IndexedDB §6.1: "If index key is an exception, or invalid, or failure, take no further actions for index, and continue these steps for the next index." + note "An exception thrown in this step is not rethrown." |
| V11 | Durability `default`/`strict`/`relaxed` per transaction; strict = verified write to persistent medium; relaxed = written to OS without verification; failed write aborts (§4, §6.6, SG-D3) | **Correct** | IndexedDB §2.7 durability text and §5.4 commit algorithm match, including the abort-on-write-failure path |
| V12 | Generator 2⁵³ ceiling, `ConstraintError` on exhaustion (§4) | **Correct** | IndexedDB §2.11: "If key is greater than 2⁵³ (9007199254740992), then return failure"; `ConstraintError` on exhaustion |
| V13 | Modes exactly `readonly`/`readwrite`/`versionchange`; read-only overlap concurrency; read/write waits for earlier overlapping transactions (§4, §6.2) | **Correct** | IndexedDB §2.7.2 scheduling text matches |
| V14 | Commit when all requests completed and handled, none new, not aborted (§6.13) | **Correct** | IndexedDB §2.7.1: "must attempt to commit an inactive transaction when all requests placed against the transaction have completed and their returned results handled, no new requests have been placed…" |
| V15 | Cache API "completely isolated from the browser's HTTP cache"; author-managed; no automatic updates (§2, §6.3) | **Correct** | Service Workers §5: "they are completely isolated from the browser's HTTP cache"; §5.2: "do not get updated unless authors explicitly request them to be"; "do not disappear just because the service worker script is updated" |
| V16 | Cache.put rejects: non-GET/non-http(s) scheme, status 206, `Vary: *`, disturbed/locked body; no opaque-type check in put (§2 header, §6.3's opaque handling) | **Correct** | Service Workers §5.4.5 steps 4, 6, 7, 8 as quoted; no opaque-type condition exists in the put steps (the CORP check is in `matchAll`) |
| V17 | Registrations must be persisted; keyed (storage key, scope); stale at 86 400 s; byte-for-byte script-resource-map comparison (§6.3, §6.4) | **Correct** | Service Workers: "A user agent must persistently keep a list of registered service worker registrations…"; staleness > 86 400 s; Update step: "newestWorker's script resource map[url]'s body is not byte-for-byte identical with response's body" |
| V18 | Lock tri-state ("open"/"taken-exclusive"/"taken-shared"); writable stream = shared lock, sync handle = exclusive (§6.7, §6.14) | **Correct** | File System §3 lock model, verbatim |
| V19 | Sync access handles: DedicatedWorker + SecureContext, bucket-FS-only (§6.7) | **Correct** | File System: "this method will only succeed when the fileHandle is in a bucket file system"; `[Exposed=DedicatedWorker, SecureContext]` |
| V20 | Write-on-close atomicity (§6.7) | **Correct** | File System createwritable note (temporary file, replace at close) and close algorithm: "atomically updates the contents of the file on disk" |
| V21 | OPFS = "bucket file system" in the standard; `getDirectory()` without prompting; path's first item empty string (§6.14) | **Correct** | File System §9 and bucket-FS detection text as quoted |
| V22 | `StructuredSerializeForStorage` exists in HTML for storage serialization (§7) | **Correct** (per note 8.5, High; section number §8.3 not independently re-verified) | specs_semantics 8.5 |

### 1.2 Cross-document claims (quotes compared against `reports/`)

| # | Spec claim | Verdict | Evidence |
|---|---|---|---|
| V23 | HGBA-2 4.6 sentence to strike: "Persistent storage uses proven crash-consistent embedded stores such as SQLite in WAL mode ([SQLite WAL](https://www.sqlite.org/wal.html)), never custom persistence layers." (§9.5) | **Correct** — verbatim match | HGBA-2 line 190 |
| V24 | HGBA-2 Section 3 diagram contains "storage (SQLite)," (§9.5) | **Correct** | HGBA-2 line 113 (inside the UTILITY SERVICES box) |
| V25 | Stage 1 Section 1 Storage service row "Owns": "SQLite (WAL) for IndexedDB, Cache API, localStorage, cookies at rest" (§9.5) | **Correct** | Stage 1 line 26 |
| V26 | Stage 1 13.2 Storage row text as quoted, including "The design is written before M5" (§9.5, §9.2) | **Correct** | Stage 1 line 536 |
| V27 | Stage 1 13.2 Everyday features row does not name a storage holder for imported history/bookmarks (§9.5) | **Correct** | Stage 1 line 537: "import from Chrome and Firefox (bookmarks, history and passwords…)", no holder named |
| V28 | Stage 1 13.2 threat model row names the storage process ("broker, renderer, GPU, network, storage, utility, shell renderer, agent host") (§9.5) | **Correct** | Stage 1 line 538 |
| V29 | Ledger G-19 text as quoted (header, §9.2) | **Correct** | ledger line 106 (the ledger additionally carries an in-progress update row the spec's quote need not repeat) |
| V30 | DM 6.13: "The storage service, IndexedDB's engine, the Cache API's store and quotas belong to the storage spec (ledger G-19)" and the ownership split (header, §6.1) | **Correct** | DM line 588, verbatim |
| V31 | DM 6.18 both tables hand IndexedDB/Cache/Buckets/storage service to the storage spec (§9.5's amendment target) | **Correct** | DM lines 701 and 724 |
| V32 | DM-D11 snapshot protocol service side (read-only snapshot, overlay, last-writer-wins in sequence order, install at task boundary as recorded input, quota enforced service-side, snapshot requested at navigation commit) (§6.7) | **Correct** | DM line 590 matches every element the spec restates |
| V33 | DM 9.5 removes `localStorage` reads from the sync allowlist; NW 9.5 removes `document.cookie` (§6.1) | **Correct** (quotes exist), **Partly** (see S5) | DM line 926 and NW line 906; both are pending applications ("once … cases pass"), not applied text |
| V34 | DM-D12: IndexedDB values stored by the storage service as opaque bytes; decoder reads every published version forever (§6.1, G152) | **Correct** | DM line 546, verbatim |
| V35 | G99: partition keys from broker-minted capability, never computed in the renderer (§6.1, G155) | **Correct** | DM line 688, verbatim |
| V36 | FP-D16 fallback: "leaves the class and moves to a crash-consistent embedded store" (§2, §9.5) | **Correct** | FP line 1057, verbatim |
| V37 | FP-D18 (a): "Held in the storage service, 24 h† detail and 7 days† aggregates, cleared with history and site data, per container, memory-only in private windows, shell sees only displayed rows" (§6.10, §6.3) | **Correct** | FP line 1059, verbatim |
| V38 | FP-37 text and "Storage tests; shell renderer inspection in DST" (§6.3) | **Correct** | FP line 982, verbatim |
| V39 | FP-1 charges the storage service ≤ 4 MiB† idle ("storage ≤ 4" in the 8.2 baseline row) (§6.1, SG-4) | **Correct** | FP line 862, verbatim |
| V40 | FP 9.4's amendment to HGBA-2 4.6 does not strike the SQLite sentence, so the conflict FP left open is real (§9.5's rationale) | **Correct** | FP 9.4 HGBA-2 4.6 row replaces V8 mechanisms and adds the regenerable-cache class; the SQLite sentence remains in HGBA-2 line 190 |
| V41 | FP 6.14: attribution history in the storage service, shell renderer receives rows on screen only (§6.1, §6.3) | **Correct** | FP line 635, verbatim |
| V42 | FP 6.9: suspend file "stored in the partition's storage", SEC-34/REC-R2 key rule (§6.3) | **Correct** | FP line 549, verbatim |
| V43 | NW 6.8: "Service-worker registrations, Cache Storage — Storage key (T, C, X) + O"; (T, C, X, n) is the renderer-cache projection only (§6.8) | **Correct** | NW line 426, verbatim; the (T, C, X, n) renderer-cache projection is elsewhere in NW 6.8 as the interfaces note quotes |
| V44 | NW 6.9: jars "persist through the storage service (SQLite WAL), encrypted under per-container keys…" and the diagram line "Storage service: unpartitioned cookies at rest (SQLite WAL, SEC-34)" (§9.5, §6.3) | **Correct** | NW lines 450 and 215, verbatim |
| V45 | NW 6.16: reads from long-lived state recorded as the values read; writes re-executed in replay (§6.12, SG-D8) | **Correct** | NW line 537, verbatim |
| V46 | DT 6.5 rule 1: sensitive classes "(cookies and storage, bodies, form values, TLS secrets)"; rule 8: replay host has "no network, storage service or GPU device"; rule 10: saved bundles encrypted under the profile key, stored in the container's partition, deleted with it (§6.10, §6.12, §6.3) | **Correct** | DT lines 297, 187, 306, verbatim |
| V47 | DT Stage 2–3 views list "storage keys" (§9.5's DevTools amendment target) | **Correct** | DT lines 95 and 245 |
| V48 | ST 6.16/ST-D17: the sheet's source, the only copy, lives in the storage service; compiled file joins FP-D16 (§6.3) | **Correct** | ST line 549, verbatim |
| V49 | MD 6.1/MD-31: "the Cache API stores it in the storage service"; opaque bytes never map into a renderer (§6.1, SG-8) | **Correct** | MD line 228, verbatim |
| V50 | CB 6.5: interaction record lives in the storage service per container, 45 days, cleared with history and site data, memory-only in private windows (§6.3) | **Correct** | CB line 314, verbatim |
| V51 | Prefix check: SG- collides with nothing in `reports/`; G150–G159 free (header) | **Correct** | No SG- prefix found in any report; AX uses G100–G109, PDF G110–G119, SVG G120–G129, animation G130–G139, editing G140–G149; the extensions spec's ledger row explicitly records "G150–G159 as unassigned" and registers G170–G179 to itself — consistent with this spec's claim |
| V52 | "not counted anywhere" listing in **both** reconciliation notes (header, §9.2) | **Partly correct** | RN2 line 269 carries the literal "Not counted anywhere: … storage …"; RN1 line 236 states the same fact in other words ("The total does not include … storage …"). Substance holds; the quoted phrase exists in one of the two notes |
| V53 | Combined staffing "about 437–560 people†" (§9.2) | **Correct** | Matches DM 9.3's combined-staffing sentence as recorded in izcal_interfaces 4.7 and RN2's arithmetic |

### 1.3 Historical claims

| # | Spec claim (section) | Verdict | Evidence |
|---|---|---|---|
| V54 | Cookies re-specified "four times (2109, 2965, 6265, bis)" (§2) | **Correct** | history_postmortems 1.2/1.5 (RFC 6265 names 2109 and 2965; 6265bis-22 at the RFC Editor); not independently re-fetched, High per note |
| V55 | Gears: deprecated Feb 2010 when HTML5 absorbed the ideas; "its three modules are the ancestors of Workers, AppCache and WebSQL" (§2) | **Partly correct** | Deprecation month correct (development stopped announced 19 Feb 2010; Chrome removed Gears June 2011 — Wikipedia Gears page). But Gears shipped **five** API components (Database, LocalServer, WorkerPool, Desktop, Geolocation); the three named are the storage/offline ones. "Its three modules" overstates; see S2 |
| V56 | Web Storage sync jank "10k writes: 527 ms Chrome" (§2) | **Partly correct** | Lawson 2015's table does say 527 ms (Chrome 47), 177 ms (Firefox); but the article's own correction note says the LocalStorage numbers were "inaccurate" because of a test-suite bug (values stored as `"[object Object]"`), and that after the fix "LocalStorage performs more poorly" — i.e. the real cost is worse, with no corrected figure published. The direction of the claim survives; the number needs the caveat. See S1 |
| V57 | WebSQL: April 2009, abandoned after 19 months (Nov 2010); W3C Note "all interested implementors have used the same SQL backend (Sqlite), but we need multiple independent implementations"; spec mandated "the SQL dialect supported by Sqlite 3.6.19" (§2, §5) | **Correct** | W3C Web SQL Note: Working Group Note of 18 November 2010; both quoted sentences verbatim on the page |
| V58 | WebSQL "was *faster* than IndexedDB when it died" (213 vs 572 ms Chrome; 173 vs 10 658 ms Safari 9) (§2) | **Correct** | Lawson 2015 table: WebSQL Chrome 213 ms vs IndexedDB 572 ms; Safari 9 WebSQL 173 ms. (The 10 658 ms Safari IndexedDB figure is in the article's IndexedDB column per the note; not re-read this session — note-graded High) |
| V59 | IndexedDB "REC 2015; 2.0 in 2018; 3.0 still a Working Draft in 2025 — fourteen years" (§2, §5) | **Partly correct** | The dates are right (2015-01-08 REC; 2018-01-30 2.0; 3.0 WD, note-graded High at 2025-08-13). The arithmetic is not: 2009 (proposal) to 2025 is **sixteen** years; "fourteen" counts only from the first public IndexedDB draft (2011). See FC-M4 |
| V60 | AppCache: "nothing updates, ever" documented (§2) | **Correct** | A List Apart, Gotcha 4: "…the user's cached version still points at the old manifest, which is obviously byte-for-byte the same so nothing updates, ever." |
| V61 | AppCache postmortem quote: "we didn't have enough evidence, and created a high-level solution that couldn't be fixed" (§2, §5) | **Wrong** | Archibald, "If we stand still, we go backwards" (3 Aug 2015) actually reads: "we didn't have enough evidence, and created a high-level solution that **couldn't be used the way developers wanted**." The spec's quotation alters the quote. See FC-M1 |
| V62 | Service Worker + Cache API 2014–2018 as AppCache's replacement; strictly isolated caches (§2) | **Correct** | V15/V16 above; the 2014 replacement framing is Offline Cookbook per note 6.3 (not re-fetched, High per note) |
| V63 | Safari deletes all script-writable storage after 7 days of use without interaction, home-screen web apps exempt (§2, §4, SG-D5) | **Correct** | WebKit blog 24 Mar 2020: "deleting all of a website's script-writable storage after seven days of Safari use without user interaction on the site"; home-screen web apps keep their own usage counter |
| V64 | LevelDB-era numbers: 26 MB read+write worst-case compaction; fillsync 0.4 MB/s vs fillseq 62.7 MB/s (§2 cites [H11][HP]; numbers live in the note) | **Not re-checked this session** | Note-graded High against leveldb impl.md; the spec's own text does not quote the numbers, only the qualitative costs, which the note supports |
| V65 | WiscKey: 2.5×–111× faster loading than LevelDB (§2's KV-separation result, §6.2's value-log rationale) | **Correct** | FAST '16 landing page abstract: "WiscKey is 2.5x–111x faster than LevelDB for loading a database and 1.6x–14x faster for random lookups" |
| V66 | ALFS/ALICE: 60 crash-consistency vulnerabilities across 11 applications, including LevelDB and SQLite (§5, SG-1, G157) | **Correct** | Paper abstract (PDF text extracted): "we analyze eleven widely-used systems … and find a total of 60 vulnerabilities"; LevelDB and SQLite in the study set; OSDI 2014 (the security note's correction of the brief's 2017 is right) |
| V67 | Chrome ABE as the documented overpromise: path-based trust check defeated by injection into the trusted process (§6.9, SG-D7) | **Correct** | Research-tool README prose: "As the call originates from a trusted process path, all of the server's security checks are passed"; "…satisfy its path-validation security checks." Used within the note's confidence grade (mechanism High, timeline Medium) |
| V68 | "5–15 exploitable renderer bugs per milestone is the published historical rate" (§6.9) | **Correct** | Chromium Site Isolation page: M69 = 10, M70 = 5, M71 = 13, M72 = 13, M73 = 15; "We assume that determined attackers will be able to find a way to compromise a renderer process." |
| V69 | Storage Buckets: Draft Community Group Report 19 Dec 2023, single engine (§2, §6.4) | **Correct** per note (SSB 4.1, HP 10.4, both High; not re-fetched — the draft's date and Chromium-122 shipping are cross-consistent between two notes) | specs_semantics 4.1; history_postmortems 10.4 |
| V70 | Chrome 60%/80%, Firefox min(10 %, 10 GiB group) vs web.dev's 50 %-of-free/2 GB conflict (§3, Q-SG-2, §11) | **Correct, and Q-SG-2 can now be settled** | MDN quotas page (modified 2026-01-05): Firefox best-effort = smaller of 10 % of total disk or the 10 GiB group limit; persistent 50 % capped at 8 TiB; Chromium origin cap 60 % of total disk, browser 80 %. The **MDN figure is current**; web.dev's 2024 figures (50 % of free disk, 2 GB eTLD+1) are superseded |
| V71 | Safari ~60 %/15 %/1⁄10-of-parent quota figures (§11's flagged-claims context) | **Correct** | MDN quotas page: ~60 % of total disk for browser apps (macOS 14+/iOS 17+), ~15 % for embedded web content, cross-origin frames ~1/10 of the parent |
| V72 | `estimate()` documented as padded ("only returns the estimated usage value, not the actual value") (§3, §6.5) | **Correct** | MDN quotas page, sentence verbatim; browsers pad "to avoid fingerprinting" |

### 1.4 M-1 part 3 compliance

| # | Check | Verdict |
|---|---|---|
| V73 | The spec cites no third-party source code anywhere. Its external links are spec pages (storage.spec.whatwg.org, w3c.github.io/ServiceWorker, wicg.github.io/storage-buckets) and the SQLite WAL page quoted only inside the text being struck. The research notes state no code was read; what they cite from repositories is documentation prose (LevelDB README/impl.md/index.md, redb design doc, sled README, LMDB docs, RocksDB wiki, Btrfs kernel docs, SQLite documentation) and papers, which M-1 part 3 permits | **Correct** |
| V74 | No value in the spec is derived from source: every SG threshold carries †; the one source-stated constant in the notes (redb's savepoint allocator size) is not used as a spec value; Q-SG-5 explicitly plans padding magnitudes as black-box measurement | **Correct** |
| V75 | SQLite licence claim: "public domain, not MIT — fails on licence before M-1 even reaches it" (§4, SG-D1, §6.2) | **Correct as to the rule** — HGBA-2 2.3.2 requires everything shipped to be "available under MIT, alone or as one choice of a dual licence", which public domain is not, so the claim stands. But the engine_design licence register marks SQLite "MIT-compatible? **Yes** (no licence obligations)" — the note contradicts the spec and the rule. See S3 |

### 1.5 † discipline and internal cross-references

| # | Check | Verdict |
|---|---|---|
| V76 | No † threshold is presented as measured; SG-5's comparator is a black-box median of shipping engines, not a claimed measurement; Section 11 states "none measured on Izcal yet" | **Correct** |
| V77 | "5 MiB† recommended bottle quota" (§6.3) | **Wrong marking** — 5 MiB is the Storage standard's recommended per-bottle quota (Storage §4.1), not a first-principles threshold this document sets. The † violates the method's own marking rule and Section 11 correctly does not list it among "this document's own first-principles settings". See FC-M2 |
| V78 | Section 11's enumeration of the document's own † numbers omits three † marks the spec makes: 128 KiB† (§6.5 usage padding), ≤ 64 KiB† (SG-5), and the eviction score's "endpoint class weight†" (§6.5) | **Partly correct** — the statement "each marked †" is true; the enumeration is incomplete. See S6 |
| V79 | SG-1…SG-12, SG-D1…SG-D12, SG-M0…SG-M4, G150…G159 all defined and internally referenced consistently (§6.15's list matches the 9.3 table; Q-SG-6/Q-SG-9 referenced where used and defined in Section 11) | **Correct except one pointer** |
| V80 | §6.15: "Each load-bearing choice appears in a decision table in Section 9.4" | **Wrong** — the decision table is **Section 9.3**; Section 9.4 is the crate rules (G150–G159). See FC-M3 |
| V81 | §9.5 amendment rows: every quoted target text exists verbatim in the amended document (V23–V28, V40, V43–V44, V47), and the rows do not conflict with each other | **Correct** |
| V82 | §6.3 asset table citation "[SP 3.5][SPN]" for session clearing, and SG-D12's "[SP 3.3, 3.5][SPN]" | **Partly correct** — the semantics are right (V5) but note row 3.5 is the persistence-permission row; the session-clearing finding is note row **1.5**. See S4 |
| V83 | Header "Relates to" versions: FP 0.4.1, NW 0.3.4, HGBA-2 2.3.3 correct; DM 0.2.1, DT 0.3, ST 0.3.3, MD 0.4, CB 0.3 do not match the current files | **Partly correct** — current files are DM 0.2.2, DT 0.3.1, ST 0.3.5, MD 0.4.1, CB 0.3.1. See FC-M5 |
| V84 | M7 mapping: "storage service" is an M7 exit item (§9.1 SG-M2) | **Correct** | Stage 1 line 402 |
| V85 | SG-2's "hg-wpt sharded runs per Stage 1 7.2" | **Correct** | Stage 1 7.2 T1 row includes "Full WPT sharded" |

---

## 2. MUST-FIX before 0.2

- **FC-M1 — AppCache postmortem quote is altered.** Section 2 (AppCache row) and Section 5 (AppCache lesson) quote: "we didn't have enough evidence, and created a high-level solution that couldn't be fixed". The source (Jake Archibald, "If we stand still, we go backwards", 3 August 2015) says: "we didn't have enough evidence, and created a high-level solution that **couldn't be used the way developers wanted**." Replace both occurrences with the verbatim text (or paraphrase without quotation marks). `history_postmortems.md` 6.2 carries the same misquote and should be corrected in the same pass.

- **FC-M2 — "5 MiB†" misuses the † mark.** Section 6.3, localStorage row: "areas ≤ the 5 MiB† recommended bottle quota". The 5 MiB is the Storage standard's recommended quota (storage.spec.whatwg.org §4.1), not a first-principles threshold this document sets. Remove the † and cite the standard: "areas ≤ the Storage standard's recommended 5 MiB bottle quota ([SP 3.1][SPN])". Leave Section 11's flagged-claims list unchanged (it already excludes 5 MiB).

- **FC-M3 — wrong internal pointer in §6.15.** "Each load-bearing choice appears in a decision table in Section 9.4" → the decision table is **Section 9.3** (SG-D1–SG-D12); Section 9.4 is the crate rules. Change "9.4" to "9.3".

- **FC-M4 — "fourteen years" arithmetic.** Section 2 (IndexedDB row): "3.0 still a Working Draft in 2025 — fourteen years" and Section 5: "fourteen-year gestation". From the 2009 proposal the interval is **sixteen** years; fourteen counts only from the first public IndexedDB draft (2011). Either write "sixteen years from the 2009 proposal" or "fourteen years from the first draft", and align `history_postmortems.md` 5.2 ("fourteen years from proposal to (still pending) third edition"), which has the same defect.

- **FC-M5 — header version pins are stale.** The header cites "DOM, bindings and event loop spec 0.2.1", "DevTools spec 0.3", "HTML and CSS parsing and style spec 0.3.3", "images and media spec 0.4", "content blocking spec 0.3". The current files are DM **0.2.2**, DT **0.3.1**, ST **0.3.5**, MD **0.4.1**, CB **0.3.1** (all quotes verified against these versions). Update the header pins.

---

## 3. SHOULD-FIX / notes

- **S1 — LocalStorage 527 ms needs the source's own caveat.** Section 2 presents "10k writes: 527 ms Chrome" as a measurement. The article's correction note says the LocalStorage figures were inaccurate (test-suite bug storing `"[object Object]"`) and that corrected LocalStorage "performs more poorly" — so the defensible claim is "≥ 527 ms (the source's figures were later corrected as understating the cost)". Add the caveat in the spec's sentence or rely on the qualitative claim; fix `history_postmortems.md` 3.2 likewise.

- **S2 — Gears module count.** Section 2: "its three modules are the ancestors of Workers, AppCache and WebSQL". Gears shipped five API components (Database, LocalServer, WorkerPool, Desktop, Geolocation). Write "three of its modules" or "its Database, LocalServer and WorkerPool modules", which is also more precise about the ancestry claim.

- **S3 — SQLite licence register contradiction.** The spec's licence reasoning (V75) is correct under HGBA-2 2.3.2, but `engine_design.md`'s licence register records SQLite as "MIT-compatible? Yes (no licence obligations)". Reconcile: either the register's column means "no licence obligations impose on a user" (a different question) or it should say "No — not available under MIT", per 2.3.2's "alone or as one choice of a dual licence". As written, a reader of the note could conclude SQLite passes the MIT-only rule and the spec's premise fails.

- **S4 — research-note row citations for session clearing.** §6.3 cites "[SP 3.5][SPN]" and SG-D12 "[SP 3.3, 3.5][SPN]" for the traversable-close clearing rule; that rule is note row **1.5** (row 3.5 is the `persistent-storage` permission). Point the clearing claim at [SP 1.5] (or cite Storage §7.1 directly).

- **S5 — "is gone by DM-D11" overstates application status.** §6.1 says the sync-IPC allowlist "that once named `localStorage` reads is gone by DM-D11, `document.cookie` by NW-D10". DM 9.5's and NW 9.5's rows are **proposed** amendments ("once DM-D11's cases pass" / "once NW-2's ordering cases pass"); Stage 1 4.1 still lists both. The decisions are accepted but the allowlist text is not yet amended. Phrase as "is removed by DM-D11's accepted amendment (pending application)".

- **S6 — Section 11's † enumeration is incomplete.** It lists the spec's own † settings but omits 128 KiB† (usage padding, §6.5), ≤ 64 KiB† (SG-5), and the eviction score's endpoint-class weight† (§6.5). Either add them or replace the enumeration with "every † in this document".

- **S7 — "both reconciliation notes" wording.** The literal phrase "not counted anywhere" appears only in RN2 (line 269); RN1 (line 236) states the same exclusion in other words. Substantively correct; optionally reword to "the uncounted listing in both reconciliation notes".

- **S8 — "Browsers adopted log-structured (LSM) embedded stores" (§2, LevelDB era row).** The prose evidence in the notes documents LevelDB for Chromium's localStorage (the Edge 127 corruption reports, the plyvel discussion); per-engine adoption for IndexedDB-class state was explicitly left unverified (`history_postmortems.md` open question 2). Consider "Chromium-class browsers adopted…" or mark the row Medium in the note it cites.

- **S9 — Q-SG-2 can be closed.** The fact-check settled it: MDN (page modified 5 Jan 2026) is current — Firefox best-effort = min(10 % of total disk, 10 GiB group limit); persistent = 50 % of total disk capped at 8 TiB; web.dev's 2024 figures (50 % of free disk, 2 GB per eTLD+1) are superseded. Record the resolution in 0.2 and quote MDN with its date if a comparative figure is ever used.

- **S10 — Inherited † values re-marked.** §6.3 re-marks values that are other documents' first-principles settings (FP-D18's 24 h†/7 days†, FP's 7-day† suspend TTL, NW's 7-day† backstop). Each is faithful to its source, but consider citing the owning document's value without a fresh † (the † preamble convention is per-document), so a reader does not take them as this spec's independent settings.

- **S11 — Not re-checked, carried at note grade.** The Brave 2 GiB/~149 GB disagreement, the DevTools localStorage doc's 2019-03-14 date, evercookie's vector count, LevelDB impl.md and RocksDB wiki figures, and IndexedDB's REC/2.0/WD dates were not re-verified this session; all are used in the spec only at or below their note grade, and the two spec-level exceptions (Lawson's LocalStorage figures, FC-M4's arithmetic) are handled above.
