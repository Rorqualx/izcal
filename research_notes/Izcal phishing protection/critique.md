# Design critique: Izcal phishing protection (draft 0.1)

**Role.** Independent design critic, phase 8 of the first-principles spec method. I judge the design from first principles and against the accepted corpus; I do not copy-edit.

**Spec under critique.** `/Users/joederas/Documents/Cline/code/izcal-wt/phase2c/reports/Izcal phishing protection.md`, draft 0.1, 21 October 2026.

**Independence.** I did not write this spec. I did not edit it, and I did not consult its author while forming this critique.

**Review limits.** I read the spec in full, its six research notes, and the accepted decisions it touches (security scope, content blocking, graphics, shell draft 0.1, networking, resource footprint, extensions, DevTools, editing, storage, HGBA-2, HGBA-3, the Phase 3 ledger and the licence audit). I did not re-verify external facts, dates or quotations; that is the fact-checker's job. I did not read the two quality-bar specs line by line. I did not open any third-party source code, header, commit, diff or code review. I did not audit every link. No email address appears in this file.

---

## 1. Findings

### 1.1 Blockers

#### C1. The MIT-only rule is treated as a code rule, and the feeds are neither MIT nor standards data (blocker)

**Where.** PH-D14; Section 7's licence audit; the Section 9.5 row for HGBA-2 2.3.2; Q-PH-7.

**Problem.** The binding brief says: "Everything Izcal ships, **code and data**, must be available under MIT (alone or as one choice of a dual licence); anything else Izcal builds itself". PH-D14's recommendation says "**the owner's MIT-only rule applies to code; the licence-audit precedent covers list data**", and Section 7 says "Under the licence-audit precedent for list data ([LA 2.5, OD-L2, OD-L6])". None of those precedents permits shipping these feeds:

- **OD-L5 / LA 2.5** decides that Izcal **builds its own** content-blocking lists; "users may still add third-party lists themselves (not shipped by Izcal)".
- **OD-L2** covers test-only inputs that "never ship".
- **OD-L6** covers standards data (PSL, CT log list, WHATWG, tz, IANA, OpenType) — the phishing feeds are not standards data.

The spec's own Section 9.5 row admits the point: "The selected feeds are **not MIT and are not standards data**. Record a named, owner-approved exception for security list data under the licence-audit precedent, or apply the crawl-only fallback and publish the coverage gap". Q-PH-7 also states the question is open. CERT Polska's "any use" is a permissive custom term, not MIT; Phishunt is CC0; URLAbuse is CC BY 4.0; URLhaus and the Tier 2 feed are commercial. The spec then treats SEC-D7 as "controlling" because it chose the sources, which confuses a source choice with an exception to the licence rule.

**Why it matters.** This is the owner's binding rule for everything Izcal ships. If the owner reads it as written, the Tier 1 feeds cannot ship, PH-D2/PH-D14 and the source table are wrong, PH-7's coverage target and the list operation's staffing change, and G-22/Q-CB-5 do not close. The spec's recommended option rests on a precedent that does not exist, so an implementer could ship a licence breach.

**Proposed change.** Make PH-D14 a starred owner decision with the two paths costed equally: (a) a **named, owner-approved exception** for security list data, recorded against the brief and HGBA-2 2.3.2, with the exact licences and the attribution obligations; or (b) the **crawl-only fallback**, designed as a first-class path (what the crawl can observe, what coverage it cannot reach, the revised PH-7 target and the published gap). Delete "the rule applies to code". State that no current rule permits the feeds until the owner decides, and make G-22/Q-CB-5 closure conditional on that decision. Publish a NOTICE with the bundle and its data licences (see C20).

#### C2. SEC-D7's "never by whole domain" is contradicted by host-suffix matching, and the amendment does not address it (blocker)

**Where.** Section 6.2's expression table and allowlist rules; the Section 9.5 row for SEC-D7.

**Problem.** SEC-D7, accepted at 0.3, says: "**Matching is by full URL, never by whole domain**, and the list build applies an allowlist of popular and shared-hosting domains." The spec's expression table says: "**Host suffix** | `example.com` | The host `example.com` and any subdomain, on any scheme, any port, any path | Domain feeds (CERT Polska)". That is whole-domain matching. The spec's "A URL feed entry is **never** widened to a host suffix" is the likely intended reading of SEC-D7, but SEC-D7's sentence is broader, and the Section 9.5 amendment only says "Mark the client, protocol and privacy analysis specified here ... the source list stays as decided; the M8 gate is unchanged". It does not amend the matching sentence.

**Why it matters.** SEC-D7 is an accepted owner decision and the only place the matching model is fixed. Leaving the contradiction unamended means the spec and the security scope disagree on the central privacy/over-blocking control; an implementer following SEC-D7 would refuse host-suffix entries and lose the CERT Polska domain feed, which is the primary source.

**Proposed change.** Amend SEC-D7 explicitly in Section 9.5: "A **URL-feed entry** is never widened to a host suffix; a **domain feed** may be expressed as a host suffix, subject to the allowlist and the build gate." Or change the expression model to URL-prefix-only and accept the coverage loss. Add the amended sentence to SEC-D7's text, not only to this spec's amendment table.

#### C3. The download path checks "before writing bytes", not before the request, and contradicts G193/PH-4/PH-5 (blocker)

**Where.** G193; Section 6.4.5; PH-4, PH-5, PH-16.

**Problem.** G193 says: "No network request depends on a checked URL: **the check precedes DNS resolution and connection; a listed URL is never contacted before the user proceeds**." PH-4 targets "**0** DNS/TCP/UDP/cache reads/reports for 1,000† listed URLs"; PH-5 targets "100% warned navigations 0 bytes to listed host". Section 6.4.5 says: "The download manager checks the **final URL and every redirect hop** before **writing bytes**." A download request to a listed URL has already resolved DNS, connected and received bytes by the time the final URL and hops are known; "before writing bytes" is after the contact. The spec also says "The download manager checks the final URL and every redirect hop", which cannot be done for a redirect hop before following it unless each hop is checked before the next request.

**Why it matters.** The spec's central claim is that a listed URL is never contacted before the user decides. A direct download link to a listed URL, or a redirect chain that reaches one, breaks that claim and would fail PH-4/PH-5 as written. The privacy property is the reason for the local-list design (PH-D1).

**Proposed change.** Define the download handoff like the navigation handoff: the download manager asks the verdict for the **initial download URL before the network service connects**, and for **each redirect target before the next request**; bytes are written only after `allow` (or after the user proceeds). Update G193, Section 6.4.5, PH-4, PH-5 and PH-16 to say "before the request/connection", not "before writing bytes", and add the download path to PH-4's canary.

#### C4. The warning surface has two accepted mechanisms and the spec does not say which is authoritative (blocker)

**Where.** PH-D6, G195, Section 6.5.1; GR-D28; shell draft 0.1 Section 6.5.

**Problem.** GR-D28 (accepted, graphics 0.4) says prompt surfaces "bypass the shared compositor: the prompt renderer's tiles are rastered by the shell's own content GPU process and handed to the broker, which places them in **their own OS layer above the compositor's layers** (a sibling CALayer, a DirectComposition visual above the content visual, a Wayland subsurface placed above, an Android child SurfaceControl)." The shell draft (0.1) says: "A prompt is drawn in a **separate owned OS window** whose content is produced by the prompt renderer and whose placement, size, lifetime and input acceptance are decided by the broker. This is a stronger boundary than drawing above a line". A sibling layer in the page window and a separate owned OS window are different mechanisms. The spec cites both and says "the shell spec's separate owned prompt window **refines the same property**", without saying whether it supersedes GR-D28, and PH-D6/G195 name only "GR-D28's OS layer".

**Why it matters.** The unspoofable-warning property, the broker's unoccluded hit test and the 500 ms rule all depend on the surface's mechanism. Two specs describing two surfaces for one prompt class is exactly the kind of ambiguity a compromised-compositor attacker exploits, and the shell draft is not yet accepted. The security claim (PH-12, GR-19) cannot be tested against an undefined surface.

**Proposed change.** Pick one mechanism and make the other spec conform. If the shell's separate owned window is the intended refinement, say that the window's own layer **is** the GR-D28 OS layer, amend GR-D28's "(a sibling CALayer...)" wording and PH-D6/G195 to name the window, and record that GR-D28 is refined by the accepted shell decision. If GR-D28's layer stands, remove the "separate owned OS window" claim from this spec and state the dependency on the shell draft. Record the relationship in Section 9.5 for both documents.

### 1.2 Major

#### C5. CB-D16's per-entry two-person review is dropped by a "named exception", while Section 6.6.4 says CB-D16 applies "unchanged"; the sampling cannot enforce its own threshold (major)

**Where.** Section 6.6.3, Section 6.6.4, Section 9.5's CB-D16/CB-21 row; CB 6.8 and CB-D16.

**Problem.** CB 6.8 says: "**Every new entry, every change and every exception is reviewed by two people** (a curator and a second reviewer) before publication." CB-D16's reversal trigger says: "Measured review throughput too low for NW-54's coverage: **staff up, never drop the second review for blocks**." The spec auto-accepts feed entries and samples 1%; Section 9.5 calls it "a **named, published exception** to CB-D16's per-entry two-person review and CB-21's 12-week revalidation"; but Section 6.6.4 says "The content-blocking list operation's rules (CB-D16) **apply unchanged**". Both cannot be true. The sampling rule is also underpowered: "1%† of auto-accepted entries per week (at least 50†) ... If the sample's false-positive rate exceeds 0.1%†, that source's auto-accept is suspended". One false positive in 50 is 2%, so any single FP suspends the source; if the true rate is 0.1%, the chance of seeing at least one FP in 50 is about 4.9%, so the source is suspended in roughly one week in twenty for noise. To observe a 0.1% rate with 95% confidence and zero observed FPs needs about 3,000 samples (rule of three), or about 60 weeks at 50 samples/week.

**Why it matters.** The exception is defensible — a 124,500-domain feed rebuilt every five minutes cannot be reviewed per entry — but an accepted decision is being changed by a sentence, and the audit that replaces the review cannot detect the rate it claims to enforce. The result is either frequent false suspensions or an unmeasured false-positive rate, and the security list's governance precedent (no payment, evidence, two reviews) is weakened without owner acceptance.

**Proposed change.** Make the exception explicit and owner-accepted in the CB-D16 amendment (source-level vetting plus entry-level sampling, with the two-person review kept for allowlist, human, exception and top-10k entries); delete "apply unchanged". State the sampling's power and use a plan that matches the threshold: a sequential or stratified sample with a stated false-suspension rate, a stated confidence bound, or a larger sample; make suspension a review trigger, not an automatic verdict on one FP. Publish the audit method and the confidence with the transparency report. Draw the sample randomly (or publish a commitment and reveal the seed after the week), not "deterministically", so a source cannot place entries.

#### C6. PH-16 and PH-M4 conflict when the malware licence is absent (major)

**Where.** PH-16; PH-M4; Section 6.4.5's malware-gap paragraph.

**Problem.** Section 6.4.5 says: "Until that licence is signed, the `malware` class has **no shipped source**: the phishing class works, downloads get SH-R7's type rules, and the malware-URL gap is documented at M8". PH-16 requires "**100%** of a URLhaus test set (test-only until licensed) blocked or warned at the final URL or a redirect hop". PH-M4 requires "**Every PH target met**; the malware licence signed **or the gap documented**". If the licence is absent, the shipped bundle has no malware entries, so a test against the shipped bundle cannot block or warn the URLhaus set; the target is either unachievable or silently tests a test-only bundle, not the shipped path.

**Why it matters.** A milestone that says "every target met" while a target cannot be met is not a gate; it invites a waiver or a test that does not test the shipped product. The malware gap is a known, accepted limitation, and the target should say so.

**Proposed change.** Scope PH-16 to the phishing class plus the SH-R7 type rules when the licence is absent, or state that the URLhaus test runs against a **test-only bundle** and tests the download path's matching, not the shipped coverage. Make PH-M4's "every PH target met" conditional on the licence, or explicitly list the targets that are waived when the malware class is a documented gap.

#### C7. The local-to-prefix reversal trigger is not measurable (major)

**Where.** PH-D1's reversal trigger; Section 4.4's reversal paragraph.

**Problem.** PH-D1 says: "A licensed feed over the bundle budget, or a **measured freshness gain a 15-min poll cannot use**: prefix confirmation through a non-colluding OHTTP relay". Section 4.4 says "a measured source freshness that a 15-minute poll cannot use". There is no metric, threshold or measurement method. CERT Polska rebuilds every five minutes, so a 15-minute poll already discards some rebuilds; the spec does not say what "cannot use" means — a source detection lag below 15 minutes? A poll lag that is a large share of the pre-victim window? A coverage gain of X points? Without a number, the trigger cannot fire, and the design will stay local by default even if the evidence changes.

**Why it matters.** The reversal trigger is the only stated way the design changes its central privacy decision. An unfalsifiable trigger makes the decision unfalsifiable, which is the opposite of the method's requirement for falsifiable targets and reversal evidence.

**Proposed change.** Define the metric and threshold: for example, the source's median time from first report to listing (measured on a held-out, test-only set), the poll lag's share of the campaign's pre-victim window, and a coverage or accuracy gain (the same held-out set) at which prefix confirmation is adopted; pair it with PH-22's bundle-size trigger and a recorded privacy analysis. State the measurement cadence and the owner decision that would follow.

#### C8. The alternatives table omits allowlist-only, ecosystem aggregation and a distinct v4-prefix row (major)

**Where.** Section 6.12.

**Problem.** The table has six rows: cloud v5 real-time, prefix confirmation, on-device classifier, list-free heuristics, daily list, per-site ignore. It does not present **allowlist-only** (warn only on a small set of high-value brands), **ecosystem aggregation** (combine many feeds, as Phishing Army does), or a **v4-prefix** design as a separate option. The spec rejects aggregators in Section 6.6.1 ("Phishing Army ... adds delay rather than detection") and mentions v4 in the history, but neither appears as an alternative with a cost and a reversal trigger. The v4/v5 prefix privacy trade-off is also the same as prefix confirmation's, and the spec should say so rather than leave a reader to infer it.

**Why it matters.** The method requires real alternatives with a recommendation, cost and reversal evidence. Allowlist-only and aggregation are the two designs a sceptical owner or a competing browser would propose; leaving them out makes the local-full-list decision look like the only option. The v4 omission matters because v4 is still the mental model many readers have of Safe Browsing.

**Proposed change.** Add rows: allowlist-only (reject: it cannot cover the long-tail phish that the source feeds exist for; cost is the measured coverage gap; reversal is a measured brand-only threat model); ecosystem aggregation (reject: the measured Phishing Army delay and the overlap data; cost is the added latency and the upstream-rights problem); v4 prefix (reject or treat as identical to prefix confirmation's privacy property; state that v4's update API is superseded and its 4-byte prefix has the same re-identification issue). Keep the current rows.

#### C9. "A separate TUF target with its own schedule" is not how TUF repositories work, and the shared timestamp is not reconciled (major)

**Where.** Section 6.7's "One bundle, every client" and "TUF's part and Izcal's part"; Section 9.5's security-scope and CB-D17 rows.

**Problem.** Section 6.7 says "The phishing bundle is a **separate TUF target** from the content-blocking bundle, with its own version, metadata and schedule." In TUF, `timestamp` and `snapshot` metadata are repository-level and cover all targets; a target has its own version, but it does not get an independent schedule unless it lives in a separate repository (with its own root, keys and witnesses). The spec does not say whether the phishing target is in the content-blocking repository or a new one. If it is shared, the 15-minute poll requires the shared timestamp to be re-signed every 15 minutes, and the shared timestamp's expiry (24 hours in this spec) applies to content blocking too; CB-D17's "one daily† slot for every client" and SEC-17's "timestamp expiry 3 days†" are affected. The spec says "SEC-17's 3-day† timestamp expiry is a floor; this target's is 24 h†"; a 3-day expiry is a **ceiling** (maximum age), so a 24-hour expiry is stricter, not below a floor — the wording is backwards, and the amendment must say that a shorter per-channel expiry is allowed.

**Why it matters.** The update channel is the list's security boundary. An ambiguous repository structure means the signer load, the expiry semantics and the freeze behaviour are undefined, and CB-D17's daily clients may see a timestamp that changes 96 times a day.

**Proposed change.** Specify either a **separate TUF repository** for the phishing bundle (own root, keys, timestamp/snapshot, witnesses; state the signer and on-call) or a **shared repository** with the phishing target and a documented timestamp/snapshot cadence that serves both the 15-minute poll and CB's daily slot. State the expiry rule as "24 h is stricter than SEC-17's 3-day ceiling; SEC-17 is amended to allow a per-channel shorter expiry". Add the timestamp/snapshot cadence and the signer load to the delivery section.

#### C10. The footprint targets are not reconciled with FP-28, and the total mapping and matcher state are unstated (major)

**Where.** PH-22; Section 9.5 (no FP-28/§8.2 row); FP-28.

**Problem.** PH-22 sets "p99 check ≤ **100 µs†** on the reference device; mapped bundle ≤ **16 MiB†** clean and shared". FP-28 sets "the **compiled default bundle ≤ 8 MiB†**, a sealed file mapped **clean and shared** ... **charged matcher state ≤ 256 KiB† per partition service** ... p99 ≤ 20 µs† per request". The phishing bundle is a second bundle; the spec does not state the total clean/shared mapping (8 + 16 = 24 MiB on the same device), the phishing matcher's charged state per partition service, or how the 100 µs check and FP-28's 20 µs request target compose in the same partition service. PH-22's "the check adds **0** wakeups" covers the check, not the 15-minute poll timer, and the spec does not say whether the poll counts against FP's wakeup budgets. There is no FP-28/§8.2 amendment in Section 9.5.

**Why it matters.** The footprint budgets are absolute gates in the footprint spec. A second matcher with a larger bundle and a 5× per-check target can break FP-1/FP-28 and the startup budget, and the owner will see the conflict only at FP-M3.

**Proposed change.** Add an FP §8.2 row for the phishing bundle and matcher: the clean/shared mapping, the charged state per partition service, the total with the content-blocking bundle, and the per-check cost against FP-28's 20 µs. State the poll's scheduling and battery behaviour and whether its timer is a counted wakeup. If the 16 MiB/100 µs figures are caps with measured 1–3 MB/20 µs actuals, say so and make the gate the measured figure.

#### C11. No on-call, incident runbook or bad-bundle withdrawal path (major)

**Where.** Section 9.2's staffing table and "What is not counted here"; PH-6's failure action; Section 10's risk table.

**Problem.** PH-6's failure action is "**Page the list on-call**; fix the poll or the build", but Section 9.2 has no on-call row. It defers "the automated TUF signer and its on-call" to "networking, NW-D17, CB-D17", but content blocking's staffing table has no on-call row; the networking spec's on-call row is a security-response rotation, not a list build/signer rotation. The risk table has no row for "a bad bundle ships" or "a source is compromised": the mitigations are sampling, the build gate and appeals, but there is no procedure to **withdraw** a bad bundle. TUF's rollback protection means a withdrawn version cannot be re-served; the fix must be a new, higher version, built and published under the emergency path, with a public notice.

**Why it matters.** A 15-minute build, a public list and a false-positive event on a popular domain are operational facts; without an on-call and a withdrawal path, the first bad bundle becomes either a long outage or a silent over-block. The staffing total (6–10.5) is not credible without the pager.

**Proposed change.** Add an on-call/incident row to Section 9.2 (the 15-minute build, the signer, false-positive surges, source compromise, the emergency addition) and cost it. Add a runbook: source quarantine (remove that source's entries, rebuild, publish), bad-bundle withdrawal (new higher version, two-person review, public notice, status-page entry), and the client's handling. State who can authorise an emergency rebuild and how the two-person review works at 03:00.

#### C12. A root-path URL prefix is a host-level block that evades the allowlist and PH-8 gate (major)

**Where.** Section 6.2's URL-prefix rule and allowlist rules; PH-8.

**Problem.** A URL prefix matches on a path-segment boundary, so an expression like `example.com/` matches every path on the exact host — a host-level block. The allowlist rule says a URL-prefix entry on an allowlisted domain is "kept only as a full path"; a root path is a full path, so it survives. PH-8 says "**0** shipped host-suffix entries for a top-10k crawl-ranked registrable domain (every top-10k match is a reviewed full-path entry)"; a root-path entry is counted as a reviewed full-path entry while matching the whole host. A feed entry with a URL of `https://www.example.com/` is a root path.

**Why it matters.** This is a hole in the two controls that keep the list from blocking popular sites. A compromised or sloppy feed can list a popular apex's root URL as a "URL prefix" and pass the host-suffix gate.

**Proposed change.** Treat an empty/root path as a host-level expression for the allowlist/build gate and for PH-8: refuse it for a top-10k domain, or review it as a host-level entry with the evidence a host-suffix entry needs. Add the root-path case to the canonicalization/build corpus and to PH-8's check.

#### C13. Port handling is inconsistent between expression kinds and drops the port for named hosts (major)

**Where.** Section 6.2, canonicalization step 1 and the expression table.

**Problem.** Step 1 says "remove the default port for the scheme; **keep a non-default port as part of the host key only for IP-literal hosts**". For a named host, a non-default port is dropped, so `example.com:8443` and `example.com:443` canonicalize to the same host key. The host-suffix row says "any port"; the URL-prefix row says "the exact host (no subdomains)" and does not state the port rule. A URL feed that lists `https://example.com:8443/login` may match `https://example.com/login`, and vice versa.

**Why it matters.** Scheme- and port-insensitive matching can widen an entry to a different service on the same host (a shared host, a CDN, a dev server), producing false positives or a bypass; the spec itself says one canonicalization disagreement is a bypass.

**Proposed change.** Keep the port for every non-default port (for host-suffix and URL-prefix alike), or state port-insensitivity as deliberate policy per expression kind and justify it. Add the named-host port cases to the golden corpus (PH-3).

#### C14. The query rule refers to a "full-URL entry" kind that does not exist, and ignores the query for non-allowlisted URL prefixes (major)

**Where.** Section 6.2, canonicalization step 3 and the expression table.

**Problem.** Step 3 says "ignore the query for a host-suffix expression, and keep it only when the entry is a **full-URL entry** whose source URL had a query and the host is on the allowlist (6.2.3)". The bundle stores only two expression kinds, "Host suffix" and "URL prefix"; there is no "full-URL" kind. For a URL prefix on a non-allowlisted host, the query is ignored, so `evil.example/login?campaign=a` matches `evil.example/login?campaign=b` — a deliberate widening that the spec does not state. For an allowlisted host, keeping the query only when the source URL had one means a source entry with a query matches only that query, while the same path without a query matches any query.

**Why it matters.** Query handling is a canonicalization decision with false-positive and bypass consequences, and the undefined "full-URL" kind means the build and the client can disagree — a silent bypass or a false positive, which the spec calls a ship blocker.

**Proposed change.** Define a third expression kind (a query-bearing URL entry) or drop the exception and state that the query is ignored for every URL prefix. Define the matching for query-bearing URLs in the table and add the cases (query present/absent, allowlisted/non-allowlisted) to the golden corpus.

#### C15. The PSL version and the URL-parser snapshot are not recorded inputs, so PH-18's reproducibility claim is incomplete (major)

**Where.** Section 6.2's allowlist; Section 6.10.2's build manifest; PH-18; `canon:<n>`.

**Problem.** The allowlist is built from "the top 10,000† registrable domains by Izcal's own crawl ranking", and registrable domains come from the Public Suffix List (MPL-2.0, OD-L6). The build manifest records "the input source snapshots' hashes and timestamps ..., the allowlist version, the compiler version", but not the PSL version. `canon:<n>` versions "the rules" but not the `hg-url`/WHATWG URL parser snapshot; a WHATWG change can alter a canonical URL without a `canon` bump. PH-18 requires "1,000 of 1,000 rebuilds from the same archived input hashes, allowlist and compiler version give byte-identical bundles", which cannot hold if the PSL or the parser changes between builds.

**Why it matters.** The spec's determinism claim ("a replay years later reproduces the same bytes") depends on every input that can change the output being recorded and archived. The PSL is a standards-data exception, not an unversioned constant.

**Proposed change.** Record the PSL version and the `hg-url`/WHATWG snapshot in the build manifest and in `canon:<n>`; archive them with the input hashes; bump `canon` on a parser change; add the PSL version to PH-18's inputs and to the replay record.

#### C16. The check record's 24-hour life conflicts with the site panel's example and the shell's durable prompt ledger, and the no-initiator record has no owner (major)

**Where.** Section 6.10.1; Section 6.5.3's example; Section 6.3's check-only service; shell SH-D11.

**Problem.** Section 6.10.1 keeps the record "locally for **24 hours†** in a bounded log (FP-D18's attribution window)". Section 6.5.3's example shows "Last warning: **18 Oct, 14:02**" on 21 October — three days old. The shell's SH-D11 says: "**Every prompt creates a ledger entry**, whether it is shown, expired, denied, allowed, revoked or failed. The ledger is the durable half of consent". The warning is a prompt, so the durable ledger should carry it; the spec never mentions SH-D11 or says how the 24-hour check log relates to the durable ledger. Separately, Section 6.3 says a no-initiator check runs in a check-only service that "carries the record; if the user proceeds it becomes the page's service, and if not it is discarded without state. A refused navigation leaves no destination state." If the service is discarded, who keeps the 24-hour record? Section 6.9 says the broker or partition service writes it, but the spec does not say for this case.

**Why it matters.** Two records of the same event with different lifetimes is a privacy and a consistency problem: the user may see a warning in the durable ledger that the check log cannot explain, or the site panel may show a date it cannot substantiate. The no-initiator case is the common address-bar navigation, so the ownership gap is not an edge case.

**Proposed change.** Make the check record the payload of the shell's prompt-ledger entry (linked by `check_id`): the durable ledger carries the fact, entry and decision; the 24-hour bounded log carries the detailed canonical URL and is the only part that expires. Define the broker or storage service that writes the record for a no-initiator refused navigation before the check-only service is discarded, and state its lifetime. Change the site panel's example or its rule to match the 24-hour log (or show the ledger's durable entry instead). Separate the match result (`clean`/`listed`) from the bundle state (`fresh`/`stale`/`no-bundle`) so a listed match on a stale bundle is representable.

#### C17. "A listed URL is never contacted" is stated absolutely but scoped only to checked surfaces (major)

**Where.** Section 1; G193; PH-4/PH-5; Section 4.5 and Section 6.8's extension-fetch rule.

**Problem.** Section 1 says "a listed URL is never contacted, resolved or hashed towards anyone before the user decides (PH-4)"; G193 says "a listed URL is never contacted before the user proceeds". The design deliberately does not check subresources ("request-level blocking of malicious subresources (content blocking's Security list)"), extension `fetch` ("An extension's `fetch` to a listed URL is **not** a phishing check; it is content blocking's request domain"), or speculative connections such as preconnect/DNS-prefetch (not mentioned). A page the user is on can therefore contact a listed host as a subresource or a preconnect, and an extension can fetch a listed URL, without any user decision to visit it.

**Why it matters.** The absolute claim is the spec's headline privacy promise. As written it is false for the surfaces the spec itself excludes; a reader or a red team will find the gap, and PH-4/PH-5 cannot be measured against an unscoped "listed URL".

**Proposed change.** State the boundary precisely: the claim covers **checked surfaces** (top-level navigation, redirect hop, popup, download, deeplink, paste-and-go, agent navigation), not subresources, extension fetches or speculative connections. Scope G193, Section 1 and PH-4/PH-5 to those surfaces. Decide whether to gate preconnect/DNS-prefetch to listed hosts (and if not, record it as a residual with the content-blocking owner); add the subresource/extension cases to the surface inventory so the exclusion is visible.

#### C18. A same-document `pushState` can display a listed URL without a navigation or a check, and PH-14 claims every surface (major)

**Where.** PH-14; Section 6.4's surface table.

**Problem.** A page on a shared host (say `pages.dev/user`) can call `history.pushState` to a listed URL-prefix path on the same origin (say `pages.dev/other/login`) without a navigation; the address bar changes, the page content is the attacker's, and no check runs. The top-level host is allowlisted (it is a shared host), so the host-suffix check would not catch it; the URL-prefix entry is the only control. PH-14 says "**100%** of the surface suite warns/blocks ... **0** surfaces missing" but its enumerated list omits same-document URL changes. The service-worker navigation fallback is text-covered but not listed; `blob:`/`data:` downloads are a stated residual.

**Why it matters.** The URL-prefix model exists for shared hosts, and `pushState` is the standard way to change the displayed path without a navigation. A listed path can be shown and trusted without the warning the spec promises. "Every surface" is an overclaim that hides the gap.

**Proposed change.** Re-run the check on a same-document URL change at the next task boundary (or show an unverified/held state in the address bar and site panel), and add it to PH-14 and the surface table. Add the service-worker navigation fallback to the enumerated suite. Narrow PH-14's "every surface" to the enumerated suite plus the stated residuals.

#### C19. Lookalike domains and password-manager autofill have no owner (major)

**Where.** Section 4.5's out-of-scope paragraph; shell draft 0.1's prompt table.

**Problem.** The spec says it does not own "lookalike/homograph detection (**not designed; the shell's warning taxonomy may add it later**); password-manager or form-fill warnings (**the shell**)". The shell draft's prompt table already lists "Certificate error, phishing, malware, **lookalike domain**" as a security interstitial class. That is the same shape as the original G-22 problem: a UI class with no detector and no owner. The password manager is the surface where the user's task ("Tell me before I type my password into a fake site") actually fails; a page that becomes listed after it loads, or a lookalike that no list covers, can still receive autofill.

**Why it matters.** A stated residual is acceptable; an unowned surface is not. The ledger gap G-22 was created for exactly this, and the shell's taxonomy makes the lookalike class a near-term commitment.

**Proposed change.** Name the owner and the trigger for lookalikes (the shell, with a ledger row and a decision at shell M-phase; or this spec with a new class and a measured source). Require the password manager to consult the check record before filling on a navigation whose verdict is `listed`/`stale`/`no-bundle` (or record the residual with the shell owner and a target). Add both to the ownership table.

#### C20. The public bundle needs a NOTICE and data-licence metadata, not only an in-app attribution line (major)

**Where.** Section 6.6.4's source-attribution sentence; Section 7's licence audit; the public bundle.

**Problem.** URLAbuse is CC BY 4.0, which requires attribution; CERT Polska permits any use; Phishunt is CC0. The spec says "the source page in settings names every source and its licence; URLAbuse's CC BY 4.0 attribution is shown there". The bundle is public and may be redistributed (the spec makes its hash public, publishes it through TUF, and wants third parties to verify it). An in-app settings page does not travel with a downloaded bundle, and the bundle's data licence is not the code's MIT.

**Why it matters.** A downstream redistributor of the bundle needs the attribution and the licence terms with the data. An in-app-only notice is thin for CC BY 4.0 and does not state the bundle's mixed data licence at all.

**Proposed change.** Publish a NOTICE (or a LICENSE-DATA file) with every bundle, containing each source, its licence and the attribution text, and put the same metadata in the bundle header. State the data licence separately from the code licence. Keep the in-app source page.

#### C21. riscv64 is absent from the reproducibility and performance targets (major)

**Where.** PH-18; PH-22.

**Problem.** The brief: "**riscv64 is a target from the start** (owner decision, 19 October 2026), alongside x86-64 and arm64: any ISA-specific work (code generation, SIMD, cross-ISA identity tests, CI) covers all three." PH-18 requires "1,000 of 1,000 rebuilds ... on macOS, Linux and Windows" but does not mention the ISA; PH-22 measures "the reference device" without saying which. The matcher is pure Rust with no SIMD, so the design is plausibly ISA-independent, but the reproducibility test and the p99 target are exactly the cross-ISA identity and performance checks the owner direction requires.

**Why it matters.** A binding owner direction is missed. The bundle should be byte-identical across ISAs (it is a pure function), and the 100 µs p99 must be known on the slower riscv64 reference device, not assumed from x86-64/arm64.

**Proposed change.** Add riscv64 to PH-18: byte-identical bundles across x86-64, AArch64 and riscv64, with the Linux-only cross-OS comparison on riscv64 stated as the footprint spec does. Measure PH-22 on the riscv64 reference device (and mark the target per ISA if the core is slower, as FP does). State that the matcher has no ISA-specific code.

### 1.3 Minor

| ID | Where | Problem | Why it matters | Proposed change |
|---|---|---|---|---|
| C22 | Section 7's "Dependencies used unmodified" | It names `hg-url`, `hg-update`, the PSL, the witness machinery, the OHTTP implementation and the crypto stack, but states a licence only for the PSL (MPL-2.0, OD-L6). It does not say "Izcal's own, MIT" for the `hg-*` crates, the witness machinery, OHTTP or crypto, and it mixes Izcal's own crates with external data under "dependencies used unmodified". | The brief requires the licence of every crate or data set named. An incomplete audit is the kind of omission that becomes a shipping breach. | Split the list into "built new (Izcal, MIT)", "Izcal's own crates used unmodified (MIT)", and "external data: PSL (MPL-2.0, OD-L6)"; state each. Add the bundle's data licences (C1/C20). |
| C23 | Section 7 / HGBA-2 2.3.3 | No royalty-free statement. | The brief requires royalty-free for anything patent-encumbered (MD-D3 precedent). The spec uses TUF, OHTTP (RFC 9458), WHATWG URL and UTS #46; none is patent-encumbered to my knowledge, but the spec should say so. | Add one sentence: the design uses no patent-encumbered technique; the specifications it follows (TUF, RFC 9458, WHATWG, UTS #46) are royalty-free. |
| C24 | Section 9.5's CB-D17 row; Section 6.7's emergency additions | CB-D17 says "**Security additions** that must reach users faster ride the update system's emergency channel (UPD-R5), **never a new poll**". The spec says an emergency phishing entry "does **not** use UPD-R5 ... The next 15-minute manifest poll picks it up". Section 9.5 amends CB-D17's diff and signer but not this sentence. | A reader implementing CB-D17 would route a phishing entry through UPD-R5, contradicting this spec and the "data, not a switch" argument. | Amend CB-D17's sentence explicitly: a phishing entry is a new bundle version, not a UPD-R5 security switch; or state why it is not a "Security addition" in CB's sense. |
| C25 | PH-M0 | The exit is a checklist (protocol design, privacy analysis recorded, licences recorded, prototypes, seed plan, threat model, corpus started, agent schema, charter) with no measurable gate. | A milestone with no measurable exit lets a weak foundation pass into M1. | Make the exit measurable: the privacy analysis and licence audit are published; the canonicalization corpus has N cases with the client/build differential green; the seed bundle verifies; the threat model and the surface inventory are reviewed; the agent error schema is agreed in writing. |
| C26 | PH-8 / PH-11; shell SH-34 | The shell's SH-34 sets "≤ 3%† of navigations show a security interstitial and ≤ 1%† of page loads show a dangerous-download warning"; the phishing spec has no warning-volume target and does not cite SH-34. | Two specs measure warning volume differently, and the phishing interstitial is the largest contributor to SH-34. | Cite SH-34 in PH-8/PH-11 and align the measurement corpus; or state that SH-34 is the shell's target and this spec supplies the phishing class's share. |
| C27 | Section 6.5.2 | "'Proceed anyway' sits behind 'Details' ... and needs a **second confirmation**"; the second confirmation is undefined (a second prompt? a checkbox? typed text?), and "Details" is also the shell's label for opening the prompt ledger. | The second confirmation is the last control before a user visits a listed site; if it is not broker-owned and hit-tested, it can be spoofed or clickjacked. The label collision confuses the proceed path with the ledger. | Define the second confirmation as a control on the same broker-owned prompt (or a second prompt) under CON-R1/R5, recorded in the ledger; distinguish "Details" (why/evidence) from the shell's "Why did I see this?" ledger link. |
| C28 | Section 10's legal row; Section 6.11 | The risk table uses "§230(c)(2) good faith" as a mitigation; Section 6.11 cites "§230(c)(2)/EU Q-PH-4". Q-PH-4 covers the EU/German opinion; the US safe-harbor claim is not flagged. | A legal safe harbor is not an engineering control; presenting it as a mitigation without counsel's opinion overstates the protection. | Flag the §230(c)(2) claim as unverified/legal, to be confirmed with counsel at M8, like Q-PH-4. |
| C29 | Section 6.7 vs Section 9.2 | Section 6.7 says CDN egress is "roughly **$0.3k–35k/month**" ($3.6k–420k/year); Section 9.2 says "$0.05–0.42M a year". The low end differs by about 14×. | The owner uses the run cost to judge the governance commitment; an order-of-magnitude low-end error matters. | Recompute from one assumption set (bundle size, diff size, clients, egress price) and state the assumptions; make both sections use the same range. |
| C30 | `gaps_opportunities.md` lines 34 and 94 | The note cites a `chromium.googlesource.com` URL of the form `/+/<40-hex>/docs/bug-triage-labels.md` — a prose document pinned to a commit — while the notes say no commit links were used. The file is prose, so this is not an M-1 part 3 violation in substance. | The M-1 claim should be exact; a commit-pinned URL invites a reader to think a commit was used as evidence. | Replace it with an un-pinned prose URL or cite the document by title and date; state in the note that the doc is prose. |

---

## 2. Contradictions with accepted decisions

| # | Accepted decision (quote) | This spec (quote) | Finding |
|---|---|---|---|
| 1 | Binding brief: "Everything Izcal ships, **code and data**, must be available under MIT ... anything else Izcal builds itself." | PH-D14: "the owner's MIT-only rule **applies to code**; the licence-audit precedent covers list data". Section 9.5's HGBA-2 row: "The selected feeds are **not MIT and are not standards data**. Record a named, owner-approved exception ... or apply the crawl-only fallback." | C1 |
| 2 | SEC-D7: "**Matching is by full URL, never by whole domain**, and the list build applies an allowlist of popular and shared-hosting domains." | Section 6.2: "**Host suffix** ... The host `example.com` and any subdomain, on any scheme, any port, any path | Domain feeds (CERT Polska)". The Section 9.5 SEC-D7 amendment does not change the matching sentence. | C2 |
| 3 | G193: "No network request depends on a checked URL: **the check precedes DNS resolution and connection; a listed URL is never contacted before the user proceeds**." | Section 6.4.5: "The download manager checks the **final URL and every redirect hop** before **writing bytes**." | C3 |
| 4 | GR-D28: the broker "places them in **their own OS layer above the compositor's layers** (a sibling CALayer, a DirectComposition visual above the content visual, a Wayland subsurface placed above, an Android child SurfaceControl)". | Shell draft 0.1 Section 6.5: "A prompt is drawn in a **separate owned OS window** ... This is a stronger boundary than drawing above a line." Spec Section 6.5.1: "the shell spec's separate owned prompt window **refines the same property**". | C4 |
| 5 | CB 6.8: "**Every new entry, every change and every exception is reviewed by two people** (a curator and a second reviewer) before publication." CB-D16: "**staff up, never drop the second review for blocks**." | Section 6.6.3: feed entries are auto-accepted with a 1% sample. Section 9.5: "a **named, published exception** to CB-D16's per-entry two-person review and CB-21's 12-week revalidation". Section 6.6.4: "the rules (CB-D16) **apply unchanged**." | C5 |
| 6 | CB-D17: "**Security additions** that must reach users faster ride the update system's emergency channel (UPD-R5), **never a new poll**." | Section 6.7: "**Emergency additions** ... It does **not** use UPD-R5 ... The next 15-minute manifest poll picks it up." | C24 |
| 7 | Shell SH-D11: "**Every prompt creates a ledger entry** ... The ledger is the durable half of consent." | Section 6.10.1: the record is "kept locally for **24 hours†**". Section 6.5.3: "Last warning: **18 Oct, 14:02**" (three days before the spec's date). No SH-D11 amendment. | C16 |
| 8 | FP-28: "the **compiled default bundle ≤ 8 MiB†** ... **charged matcher state ≤ 256 KiB† per partition service** ... p99 ≤ **20 µs†** per request." | PH-22: "p99 check ≤ **100 µs†** ... mapped bundle ≤ **16 MiB†** clean and shared." No FP-28/§8.2 amendment. | C10 |
| 9 | SEC-17: "timestamp expiry **3 days†**." | Section 6.7: "timestamp metadata expires in **24 hours†**"; Section 9.5: "SEC-17's 3-day† timestamp expiry is a **floor**; this target's is 24 h†." A 3-day expiry is a ceiling, not a floor; 24 h is stricter. | C9 |

The spec does handle several other accepted decisions correctly: it amends SEC-D7's privacy analysis, CB-D23's scope, SEC-29's string rule, SH-R6's origin mark and DT-D2's recording start. Those are not contradictions.

---

## 3. M-1 and owner-rule compliance

| Rule | Status | Evidence and required change |
|---|---|---|
| **M-1 part 3: no third-party source code as evidence** | **Compliant in the spec; one hygiene item in the notes.** | The spec cites published prose and specifications (Safe Browsing v5 documentation, TUF, RFC 9458, WHATWG URL, UTS #46) and says "no code is read". The notes' Chromium references are prose documents. The one commit-pinned prose URL in `gaps_opportunities.md` should be un-pinned (C30). No engine source, header, commit, diff or code review was used as evidence. |
| **MIT only, code and data** | **Not compliant as written.** | The feeds are not MIT and not standards data; the cited precedents (OD-L2 test-only, OD-L6 standards data, LA 2.5/OD-L5 build-own) do not permit shipping them. The spec's own Section 9.5 row admits this. An owner-approved named exception or the crawl-only fallback is required before G-22 closes (C1). The dependency licence audit is also incomplete (C22). |
| **No shortcuts for time or complexity** | **Compliant.** | The design keeps the full local list, the pure pinned verdict, the prompt surface and the agent refusal; the staging reasons (bundle size, the malware licence, mobile) are correctness/measurement/licence facts, stated. The 1% sampling is a governance design, not a schedule cut, but its power must be fixed (C5). |
| **Royalty-free** | **Not stated.** | Add the royalty-free statement for TUF, OHTTP, WHATWG URL and UTS #46 (C23). |
| **riscv64 a target from the start** | **Missed.** | PH-18 and PH-22 do not mention riscv64; add the cross-ISA bundle identity and the riscv64 performance measurement (C21). |
| **Determinism and isolation** | **Strong, with two gaps.** | The pure verdict, the bundle pinning and the one record are right. The PSL/parser versions are missing from the recorded inputs (C15), and the record's owner/lifetime for refused no-initiator navigations is ambiguous (C16). |
| **Privacy by construction** | **Strong, with a scope gap.** | No URL leaves the device for the check, no hit telemetry, no per-user variant, reports are user-initiated and OHTTP. The absolute "never contacted" claim must be scoped to checked surfaces (C17), and the 15-minute poll's metadata/battery cost belongs in the privacy and footprint analysis (C10). |
| **Footprint budgets** | **Not reconciled.** | PH-22's 16 MiB/100 µs vs FP-28's 8 MiB/20 µs; no FP §8.2 row (C10). |
| **No email address anywhere** | **Compliant.** | The spec contains no address; the Comet incident's "extract the user's email" is a generic description, not an address. |

---

## 4. Verdict

**Is the design sound?** Yes, in direction. The central decision — a full local list with the check before DNS, no URL leaving the device, a pure pinned verdict and a broker-owned warning — is well grounded in the measured evidence: CERT Polska rebuilds every five minutes with about 21 new domains an hour, URLhaus every few minutes, a campaign lasts about 21 hours and detection lags about 9 hours. The spec is honest about the costs (staleness, the zero-day gap, the free-builder and shared-host residuals, the malware-class licence gap) and fair about the alternatives it does present. The warning UX is the strongest part: broker-owned, unspoofable, no countdown, evidence-bearing, accessible, with a per-navigation decision and no permanent bypass. The agent refusal with a typed error, the pure verdict with a pinned bundle, the no-telemetry analysis and the content-blocking governance reuse are all right. The spec does not read third-party source code, and it states its flagged claims and residuals.

**Is it implementation-ready?** No. Four blockers and a set of major design decisions are unresolved. The draft is a good 0.1; it is not a 0.2.

**What must change before draft 0.2.**

1. Resolve C1: the owner-approved MIT exception or the crawl-only fallback, with both paths costed; stop saying the rule applies only to code; make G-22/Q-CB-5 closure conditional.
2. Resolve C2: amend SEC-D7 to distinguish "never widen a URL-feed entry to a host" from "a domain feed may be a host suffix", or change the model.
3. Resolve C3: check the download URL before the request and each redirect target before the next request; update G193, PH-4, PH-5, PH-16.
4. Resolve C4: one authoritative prompt mechanism for the phishing interstitial, with GR-D28 and shell draft 0.1 reconciled.
5. Resolve C5: make the CB-D16/CB-21 exception explicit and owner-accepted, delete "apply unchanged", and fix the sampling's power.
6. Fix C7 (measurable reversal trigger), C9 (TUF repository and timestamp), C10 (FP-28/§8.2), C11 (on-call and bad-bundle withdrawal), C15 (PSL/parser versions), C16 (record vs prompt ledger), C17 (scope of "never contacted"), C18 (`pushState`), C21 (riscv64).
7. Fix the minors: C22–C30, especially the dependency licence audit, the CB-D17 emergency-channel amendment and the run-cost arithmetic.

**The five findings that most affect the design.**

1. **C1** — the MIT-only data conflict. It decides whether the list, its sources and the G-22 closure exist in the proposed form.
2. **C2** — SEC-D7's "never by whole domain" vs host-suffix matching. It decides the matching model and the primary domain feed.
3. **C3** — the download path's pre-request check. It decides whether the spec's headline privacy promise is true.
4. **C4** — the warning surface's authority (GR-D28's OS layer vs the shell's separate window). It decides the unspoofability property.
5. **C5** — the CB-D16/CB-21 governance exception and the sampling's power. It decides whether the list's review and false-positive controls are credible.

C9, C10 and C11 are close behind: the TUF structure, the footprint and the on-call are the operational facts that turn the design into a service.
