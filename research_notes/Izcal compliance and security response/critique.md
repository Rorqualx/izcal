# Design critique: Izcal compliance and security response (draft 0.1)

**Role.** Independent design critic, phase 8 of the first-principles spec method. I judge the design from first principles and against the accepted corpus; I do not copy-edit.

**Spec under critique.** `/Users/joederas/Documents/Cline/code/izcal-wt/spec-compliance/reports/Izcal compliance and security response.md`, draft 0.1, 24 October 2026.

**Independence.** I did not write this spec. I did not edit it, and I did not consult its author while forming this critique.

**Review limits.** I read the spec in full, its four research notes (fully for the CRA and data-protection strands, by section for the other two), and the parts of the corpus it touches: the feature inventory audit (FI-1, FI-2, FI-3, FI-11, FI-33, FI-35, Sections 5–9, FI-D1 to FI-D17 and the 24 October 2026 outcomes, and the PO-row definitions in its research notes); the security scope (6.8, 6.13, 6.14, 6.15, 6.16 with UPD-R1 to R5, 6.17, Section 8's SEC targets, SEC-D1 to SEC-D14, the 9.2 budget, 11.1); the Stage 1 design (Section 1's process map, 2.2 and G9, 3.4, 7.1–7.8, 8.1, 8.3, Section 6's crash-report row, 11 D-S1 to D-S6, 13.1, 13.1a, 13.2); the shell (6.4 ledger retention, 6.7, 6.9–6.14, SH-3 to SH-5, SH-22, SH-24, SH-27, SH-28, SH-D11, SH-D15, SH-D20, SH-D22 to SH-D24, SH-D29); the shell UI design (UI-1); networking (NW-D17, NW-36, Q-NW-6, 6.13); graphics (GR-D21 and its Windows 10 note); HGBA-2 (M-1 revisions 2.3.1–2.3.3, 2.3.2); accessibility (AX-1, AX-2, AX-7, AX-42); storage (G150–G159, SG-11); the phishing, content-blocking and extensions specs where they define services Izcal runs. I read the generated `BuildId` in `crates/hg-ipc-gen/src/generated/process.rs` (Izcal's own code). I did not re-verify external facts, dates or quotations; that is the fact-checker's job, and where a design choice rests on a claim that looks shaky I say so. I did not read the coding standard or the ML service spec line by line. I did not open any third-party source code, header, commit, diff or code review. I did not audit every link. No email address appears in this file.

**Note on the file under review.** The spec changed on disk while I was reading it (one line in Section 3). My findings are against the text as of 24 October 2026; if a later edit already fixes something, the finding can be closed.

---

## 1. Findings

### 1.1 Blockers

#### C1. The CRA clock has no defined start, and the "witnessed clocks" witness entries, not events (blocker)

**Where.** 6.4 "The exploited test" and the record's clock fields; 6.7's step table; CO-1, CO-7; Section 4's claim that "no clock can be backdated"; the 9.5 amendment to SEC-1.

**Problem.** Three faults compound.
1. **"Awareness" is never mapped to a field.** CRA Art. 14 runs from "becoming aware". The record has "report received", "detection in the wild" and "exploitation evidence assessed", and the runbook says "Assess: ≤ 24 h† after any exploitation claim" and "Early warning: ≤ 24 h after awareness". If awareness is read as the moment of the positive assessment, the design allows up to **48 hours** from the claim to the early warning; if it is read as the claim, the assessment consumes the whole 24-hour window. The spec says "delaying the assessment cannot delay the clock", but no sentence says which timestamp is the Art. 14 time zero. Q-CO-1 to Q-CO-24 do not ask counsel when awareness begins.
2. **Witnessing proves when an entry existed, not when the event happened.** A daily checkpoint proves that "report received = T" was in the chain by the next checkpoint. It cannot prove T is true. CO-7 allows "time-zero fields entered ≤ 24 h† after the event", so an officer can type any T inside that day before the checkpoint. SEC-1 says time zero is taken from the record "never entered after the fact"; CO-7 permits exactly that. The independent witness of a report's arrival is the intake system and the reporter, not a log of what officers typed.
3. **The checkpoint is a side channel.** Publishing the chain head daily to the shared witnessed log (UPD-R1's) reveals on which days the private record changed. During an embargo, an observer learns that security work is active.

**Why it matters.** Missing the 72-hour notification or the final report carries the CRA's top fine tier, which the spec itself cites (Art. 64(2)). The spec's headline novelty ("clocks witnessed, so they cannot be backdated") does not hold as designed, and the most consequential clock in the document is undefined.

**Proposed fix.** (a) Define Art. 14 time zero as a field: the earliest of the receipt of a claim that carries evidence of exploitation in the wild and the officers' own detection, recommended default "receipt of the claim", and add a legal question (when does a manufacturer "become aware"?) with that conservative default and a reversal trigger. (b) Make receipt clocks machine-stamped: the intake form and the role mailbox sign a receipt (time, digest of the report) and send it to the reporter; the record imports the stamp and an officer cannot edit it. Only clocks with no machine source (detection by the officers themselves) are typed, and they are flagged as such in the record and the quarterly report. (c) Publish the checkpoint on a fixed schedule whether or not anything changed, as a commitment that reveals no change count, or batch commitments weekly; state which. (d) Align CO-7 with SEC-1's "never entered after the fact" or amend SEC-1 explicitly.

#### C2. The spec's own infrastructure contradicts its EU-only hosting decision and its "no IP at ingestion" targets (blocker)

**Where.** CO-D17 ("EU only", no reversal trigger); 6.17 ("held in the EU"); 6.0's record row ("EU-hosted"); 6.3 and 6.4 (the private security repository mirrors release branches and runs "the same required checks … (SEC-27)"); 6.5 (GitHub private vulnerability reporting as a secondary channel); 6.12 and CO-18 (CDN logs ≤ 24 h, "0 client IPs seen by crash and telemetry ingestion").

**Problem.**
- The vulnerability record "is … version-controlled in the private security repository", which "mirrors the release branches" and runs SEC-27's checks. Izcal's repositories and CI are on GitHub (SEC 6.16, SEC-27), a US service. The record holds reporter identities and reproducers. That is the most sensitive personal data in the design, outside the EU, against CO-D17 and 6.17.
- GitHub private vulnerability reporting is "disclosed in the privacy notice because GitHub is a US processor". CO-D17 has no exception.
- SEC 6.16 decides that update metadata and binaries are "served from storage Izcal does not trust (GitHub or a CDN), with TUF as the only integrity root". A global CDN processes users' IP addresses at edges worldwide, and GitHub Releases gives Izcal neither raw-log deletion within 24 hours nor aggregate counts. 6.12's "Raw logs ≤ 24 h†, then aggregate counts only" and CO-18's "Raw CDN and update logs with IP addresses deleted ≤ 24 h†" are not achievable on the hosting SEC 6.16 chose, and the edges are not in the EU.
- "Uploaded through the OHTTP relay (NW-D17), so ingestion never sees the IP" assumes an independent relay. NW-D17's own condition is "A relay operator contract (Q-NW-6)", which is open, and its fallback is "(b) direct to Izcal's servers with a published logging policy". Under the fallback, CO-18's "0 client IPs seen by crash and telemetry ingestion" fails by construction. An independent relay is also a processor that sees IP addresses, very likely outside the EU.

**Why it matters.** An implementer cannot satisfy CO-D17, CO-18, 6.17 and SEC 6.16 together. The privacy promises that justify the design (no identity, EU-only, raw logs gone in a day) rest on hosting the corpus has not chosen, or has chosen differently.

**Proposed fix.** Add a hosting table to 6.12: for each data flow (security repository and record, intake, PVR, CDN edge, update origin, OHTTP relay, crash and telemetry ingestion, list and model fetches), name the operator class, the region, who sees IP addresses, and the log-retention control Izcal actually holds. Then either (a) move the security repository and record off GitHub to EU-hosted infrastructure, with SEC-27's controls restated for it, and drop or reclassify PVR; or (b) restate CO-D17 as "personal data Izcal stores at rest stays in the EU; transit processors (CDN, relay, GitHub) are listed with their transfer basis", with a reversal trigger tied to C-703/25 P. Make CO-18 conditional on Q-NW-6's answer and give it a fallback threshold. Amend SEC 6.16's hosting row if the CDN must be one whose logs Izcal controls.

#### C3. The recommended OS-drop default commits Izcal to many five-year legacy branches, and the cheaper reading is missing as an option (blocker)

**Where.** 6.2 "Dropping an OS (CO-D5)"; CO-D5; 6.18's OS-drops row; 10's risk row; 9.5's GR-D21 amendment.

**Problem.** CO-D5's default (a) keeps "a security-only legacy branch for the last release that ran on the dropped OS until the support periods of the releases it placed on the market for that OS have ended", priced as "an extra backport target per dropped OS for up to five years". GR-D21 drops an OS version whenever its vendor stops consumer security updates. macOS versions leave Apple's patch window roughly once a year, so in steady state the default carries about five concurrent macOS branches, plus Windows 10 (consumer ESU to 12 October 2027, GR's own note) and each Linux distribution release GR-D21 supports. Each branch needs Izcal's own engine fixes backported to an engine up to five years old, built reproducibly (SEC-15), signed, CSAF-published and drilled. Staffing for this is 2 officers inside the security track and 0.5–1 compliance engineer (9.2); the cost appears nowhere in Section 9 or in S1 8.3. The risk table rates it "otherwise low", which holds only for Windows 10.

The option the text most directly supports is missing: Art. 13(8) names "the availability of the operating environment" as a factor in setting the period, so Izcal can **state the support period per platform at the placing on the market**, as min(five years, the OS vendor's announced end of security support), with the reason recorded in the technical documentation. That is a statement made before sale, not a drop during the period, so the Art. 13(10) "additional costs" question does not arise for it.

**Why it matters.** It is the most expensive commitment in the spec, it is recommended by default, its cost is understated by about an order of magnitude, and it binds FI-2, GR-D21 and the release pipeline before counsel answers Q-CO-7.

**Proposed fix.** Add option (c), a per-platform support statement fixed at release from the OS vendor's published end date, with download-page and About wording, and cost all three options in branches × years × platforms against 9.2's staffing. Recommend (c) unless counsel rejects it, with (a) as the reversal. Add the result to S1 8.3 and FI-2's inputs.

### 1.2 Major

#### C4. The release-gate targets contradict the release vehicles, and CO-9 would block emergency releases (major)

**Where.** CO-9 and 6.10's (2)(a) row; CO-3; 6.4's severity table.

**Problem.** CO-9: "**0** open critical, high or exploited vulnerabilities in shipped components at publish … or a recorded risk decision per item. If it fails: Release blocked." An emergency release for exploited bug X, published while high bug Y is still inside SEC-25's 30-day bound, is blocked by CO-9. The security scope expects "a high early advisory rate" (SEC 6.17), so most feature releases would also be blocked or waved through by routine risk decisions, which makes the gate hollow. Annex I Part I(2)(a) says "known exploitable", which the spec equates with "open critical or high" without saying why. Separately, CO-3 requires "**100%** of security fixes for Release users ship in a security-only, emergency or mitigation release", but the severity table sends Moderate fixes to "Next release" and Low fixes to "Next feature release". Every feature release carrying a Low fix fails CO-3, and CO-3's failure action ("re-cut without the feature changes") would then fire.

**Why it matters.** Two ship-blocking targets cannot be met by the vehicles the same spec prescribes, and one of them can block the path SEC-1 and UPD-R4 exist to protect.

**Proposed fix.** Exempt emergency and security-only releases from CO-9 when they reduce the open set. Define "known exploitable" (for example: a fix exists and is not shipped, or the issue is exploited, or a public proof of concept exists). State who signs a risk decision, its criteria, and publish the count per quarter, with a ceiling† whose breach blocks the release. Restrict CO-3 to the severities whose vehicle is security-only (critical, high, exploited), or change the table.

#### C5. Mapping the new scale onto SEC-1 and SEC-25 gives the worst class the slower bound (major)

**Where.** 6.4's severity table ("Bound: SEC-1 (engine) or SEC-25"); SEC-1; SEC-25.

**Problem.** SEC-1 bounds engine bugs at ≤ 7 days† for critical or high. SEC-25 bounds "critical issues in Izcal's own code within 14 days† and high within 30 days†". The spec defines **critical** as a sandbox escape, an IPC or consent bypass or a forged capability, which lives in broker, IPC or service code, so it takes SEC-25's 14 days. **High** includes "code execution inside a sandboxed renderer", which is engine code, so it takes SEC-1's 7 days. A sandbox escape gets twice the time of the renderer bug it would be chained with.

**Why it matters.** The scale is meant to "express browser process boundaries". Inherited targets keyed by crate, not by impact, invert it. Chains need both halves fixed; the slower half sets the exposure.

**Proposed fix.** Give the Izcal scale one bound table independent of component (for example critical ≤ 7 days†, high ≤ 14 days†, with exploited at 72 h†), and amend SEC-1 and SEC-25 in 9.5 to cite it. Say which existing bound each row replaces.

#### C6. CO-24's installed-fix uptake cannot be measured from the fetches the design produces (major)

**Where.** Section 3's "Installed-fix uptake without telemetry"; 6.12's CDN row; 6.17; CO-24.

**Problem.** CO-24's threshold is "≥ **90%†** of the previous week's fetching installations on the fixed version within 7 days". With no per-install identifier, there is no count of installations. A TUF client polls the same timestamp metadata whatever version it runs, so its fetches do not reveal its version; only payload downloads of the new version are distinguishable, which counts updates, not the installed base. Enterprise caching proxies and Linux mirrors (FI-D10's APT and YUM repositories) serve many installations from one fetch, and delta updates (FI-2, PO-3.3) change what is fetched. To measure the share on a version, the client would have to reveal its version in a request (for example a from→to delta URL), which is a change to the update protocol and to NW-36's endpoint traffic that the spec does not state. Published counts per (day, version, channel, platform, country) can also produce cells of one or two.

**Why it matters.** CO-24 is the measurable form of a claimed novelty and the evidence behind Art. 14(2)(a)'s Member-State list and Art. 14(7)(d)'s CSIRT rule. As written it is not falsifiable, and its failure action ("goes to FI-2's owner") does not bite.

**Proposed fix.** Either specify the signal (what the client reveals, in which request, with the privacy analysis and an NW-D17/NW-36 amendment) or downgrade CO-24 to what can be counted (payload downloads of the fixed version per day, against the previous version's), with no 90% threshold until a measured denominator exists. Add a minimum cell size† before publication and say how proxies and mirrors bias the counts.

#### C7. "One record, many projections" has no disclosure classes, so a generator can leak embargoed or personal fields (major)

**Where.** 6.4's field table and projections table; 6.17; Section 3 ("so they cannot disagree").

**Problem.** One record feeds projections with different audiences and times: the CRA notification (confidential, before publication), the CSIRT delay request, the CVE record (reserved, then public), CSAF and OSV (public at release), release notes, enterprise notes. The record also holds the reproducer, reporter identity, embargo parties and the CSIRT reference. The spec gives no per-field disclosure class (public at release; to CSIRT only; internal; personal data) and no test that a public projection contains no restricted field. The claim that projections "cannot disagree" also holds only at generation: CSAF revision histories, CVE record updates and EUVD copies move independently afterwards, and third-party rescoring continues regardless (the spec's own Section 2.2).

**Why it matters.** A single source of truth is also a single point of leakage. A bug in one generator publishes an embargoed fix's details or a reporter's identity to every scanner.

**Proposed fix.** Add a disclosure class to every field and a publish-time check in `hg-advisory` (a target: 0 restricted fields in any public projection, tested with planted canaries). Specify revision handling: every projection carries the record version it was generated from, and a correction regenerates and republishes all public projections together. Reword "cannot disagree" to "are generated from one version of the record".

#### C8. Reporters' reproducers can become public regression tests, and the retention rules disagree (major)

**Where.** 6.4's Evidence group; 6.5 "Reproductions"; 6.12's intake row; 6.17; SEC-24; REC-R3.

**Problem.** SEC-24's regression suite "carries a test for every fixed security bug", and Izcal's repository is public, with fixes merged at release (6.3). The natural test is the reporter's reproducer or a reduction of their recording, which can carry page content, credentials-adjacent strings or a third party's data (REC-R notes recordings hold "every response, timer and input event"). Nothing says the public test must be a minimised, synthetic reduction. Retention also disagrees across the spec: recordings are "deleted when the record closes" (6.5); reproducers are "deleted 90 days† after release" (6.17); intake data is kept "record lifetime plus 2 years†" (6.12), while 6.17 keeps "reporter contact … for 2 years†"; REC-R3 expires saved bundles after 30 days† unless pinned. The record itself may live for the ten-year documentation period (Art. 13(13)), so "record lifetime plus 2 years" is effectively unbounded.

**Why it matters.** The security-response design is itself a privacy risk, and the one place a reporter's material most likely ends up public has no rule.

**Proposed fix.** Rule: a regression test committed to the public repository is a minimised reduction written or reviewed by an officer, contains no reporter-supplied page content, and is checked for strings from the original reproducer before merge. Give one retention table for reproducers, recordings, reporter identity and contact, each with a start event and a period†, and reconcile it with REC-R3.

#### C9. The broker's security event log duplicates the prompt ledger and escapes private-window and clearing rules (major)

**Where.** 6.0's "Local security event log" row; 6.10's (2)(l) row; the 9.5 security-scope "new rule" row.

**Problem.** The log records "permission grants and revocations (the prompt ledger, SH-D11), extension installs and capability changes, policy changes, update events, UPD-R5 switches, certificate-error overrides, protected-media switches, DevTools and automation attachments", kept 90 days†, "user-switchable", and so on by default, as Annex I Part I(2)(l)'s opt-out implies. SH-D11 already makes the ledger "the canonical account of consent", stored in the storage service under each container's key with one documented cross-container index exception, cleared by "Clear browsing data" and never written by private windows (shell 6.4, 6.7). The new log names sites (certificate overrides, protected-media switches, DevTools attachments), so it is browsing-history-equivalent data. The spec does not say under which key it is stored, whether private windows write to it, whether clearing a site or "Clear browsing data" clears it, or how it relates to the ledger. A second cross-container store needs its own storage-spec exception, which the 9.5 storage row does not list.

**Why it matters.** A default-on, 90-day log of sites a person visited, outside the clearing rules, contradicts the shell's privacy model, and two consent logs will disagree.

**Proposed fix.** Extend the ledger (SH-D11) with the new event kinds instead of creating a broker log: same keys, same clearing, private windows write nothing, export only on request. If a separate log is kept, specify its key, its private-window rule, its clearing rule and the storage exception, and add them to the shell and storage amendment rows.

#### C10. The shared header has a hash cycle, redundant fields, and an identifier that does leave the device (major)

**Where.** 9.4 and the 9.1 "M1 exit" row; 9.5's S1 8.1 row.

**Problem.**
- **SBOM digest.** The SBOM must carry "SHA-512 of the deployable file and the executable" (6.8, BSI TR-03183-2). If the executable embeds the SBOM's digest, the SBOM must hash a file that contains the SBOM's hash: a cycle.
- **Redundancy.** `BuildId` is a `u64` "from the release process" (`process.hgidl`, generated in `crates/hg-ipc-gen/src/generated/process.rs`). Product version, release line, channel, platform, ISA and the SBOM are all functions of the build. Repeating them in every header invites the disagreement the spec's "one record" idea exists to prevent. Only OS version, process kind and sandbox tier, and the policy digest are per-run facts.
- **Profile identity.** "An internal, per-installation random value that never leaves the device" is in the recording header, but 6.5 invites reporters to attach recordings, and DevTools export bundles. Exported recordings carry it off the device.
- **Policy digest.** A digest of an organisation's enterprise policy set is close to unique per organisation, so in a crash report it is a quasi-identifier, against "no personal data in any field" and SH-28's "0 identifiers".
- **Status column.** "Exists" for build ID, data versions and the decision-point log means "named in M1's exit criterion". No recording-header struct exists in the code yet; the column should say so. "The record's schema version reserved" has no reason to be an M1 exit item.

**Why it matters.** M1's exit is the cheapest time to fix the format (FI 8, item 1). Fixing it with a cycle and a leaking identifier would cost a version bump later.

**Proposed fix.** Header = build ID (keying a published build manifest that maps to version, line, channel, platform, ISA and SBOM), OS version, process kind and tier, decision-point log reference, data versions, schema version. Drop the SBOM digest. Strip the profile identity on every export and upload, or derive a per-export random ID. Replace the policy digest in crash reports with a coarse "policy present / which risk-relevant policies are set" vector, or keep the digest in recordings only. Relabel "Exists" as "In M1's exit criterion". Move the record's schema version out of M1.

#### C11. The deletion-token design links telemetry, can strand server data, and leaves access unserved (major)

**Where.** 6.12 "Rights without identity"; CO-19; SH-28; SH-D29; 6.10's (2)(m) row.

**Problem.** "Telemetry uses the same pattern per reporting period": one token per period links every telemetry record in that period on the server. SH-28 requires "0 identifiers … in telemetry" and SH-D29 "collects no identifiers". "Delete all Izcal data on this device" deletes the local tokens, after which server-held crash reports can never be deleted by the person; the spec does not order the two. The design serves erasure but not access (Art. 15) or portability for server-held crash data, which PO-15.3 asks for. The deletion endpoint is a new user-initiated request to Izcal's servers, and the NW-D17 amendment row names crash uploads and telemetry but not it.

**Why it matters.** A privacy mechanism that creates a linkable identifier contradicts an accepted shell target, and a wipe that strands server data defeats the right it serves.

**Proposed fix.** Use a token per record for telemetry too, or state why linking a period is acceptable and amend SH-28/SH-D29. Make "delete everything" first offer, and by default perform, server deletion with the held tokens. Add access: `about:crashes` keeps the exact bytes sent (or their digest and a "fetch what the server holds" action through the same token). Add the deletion endpoint to NW-D17's list.

#### C12. CO-5's "all three platforms" ship blocker contradicts FI-D10 for Linux packages (major)

**Where.** 6.3 "Automatic by default"; CO-5; Q-CO-8; FI-D10.

**Problem.** FI-D10 (decided 24 October 2026): `.deb` and `.rpm` from Izcal's repositories are "updated by the system package manager … the per-user TUF updater stays off for them". On common distributions, unattended upgrades cover only the distribution's own security archives by default, so an Izcal package does not update automatically unless the user configures it. CO-5 requires "On all three platforms: on by default … **100%** in the update harness … Ship blocker". The spec defers the conflict to Q-CO-8, labelled "Legal; FI-2", while CO-5 already blocks.

**Why it matters.** Either the Linux packages cannot ship, or CO-5 is not enforced; an engineering answer exists and is not designed.

**Proposed fix.** Present options with a default: (a) the package installs a drop-in that adds Izcal's origin to the system's automatic-update service where one exists, with the Annex II page saying so and how to turn it off; (b) Linux packages are documented as administrator-managed, with the opt-in printed at install; (c) Linux users get the tarball with the TUF updater as the EU default. Make CO-5's Linux row follow the chosen option, and keep the legal question for whether (b) satisfies Annex I Part I(2)(c).

#### C13. "Closed cryptography is already ruled out by D-S7" misreads D-S7 (major)

**Where.** 6.16 Export control; 6.1 and CO-D2 (Art. 32(5)); 6.8 SBOM; Section 7.

**Problem.** D-S7 excludes **unlicensed** DRM; it does not exclude licensed closed modules. Its licensing track runs "a Widevine license request to Google; a castlabs third-party-licensee quote (the only Linux route)" (S1 13.1a), and S1 13.2 already warns that "a release that bundles a closed DRM module is typically classified as mass-market encryption, so legal review comes before the first public binary". If that track succeeds, Izcal ships a closed cryptographic component. The spec does not ask (i) whether a release containing it still qualifies as "free and open-source software" for the Art. 32(5) route, (ii) how a closed binary appears in the SBOM and VEX, or (iii) how its vulnerabilities enter the record when Izcal cannot fix them. The S1 13.2 export amendment ("none is planned") would then be wrong.

**Why it matters.** CO-D2's whole conformity route, and the export conclusion, depend on every shipped component being free software.

**Proposed fix.** Replace the D-S7 sentence with the actual condition: "no closed cryptographic component ships unless D-S7's licensing track succeeds; if it does, export classification, Art. 32(5) eligibility and the SBOM treatment are re-decided before that release." Add those as questions for counsel and as a reversal trigger on CO-D2.

#### C14. Vulnerabilities in OS components Izcal runs but does not own have no path (major)

**Where.** 6.4 (severity, scope, projections); 6.7; 6.5 "Dependencies"; SEC 6.10a, 6.14 SH-R4; the security scope's summary.

**Problem.** The security scope's central finding is that "the risk moves to … code Izcal runs but does not own: GPU drivers, OS media decoders, and the OS code and vendor CDM in the protected-media path". SEC-25 already includes "vendor binaries such as the PlayReady CDM where Izcal ships the fix path". This spec's severity scale, scope fields, CVE assignment rule, Art. 14 runbook and advisories cover Izcal's crates and "shipped dependencies" only. It does not say what happens when an exploited WebView2 or WKWebView flaw reaches users through protected-media mode, or an OS decoder flaw through Izcal's media path: whether Izcal reports under Art. 14, how the advisory states it, which UPD-R5 switch applies (only "one named codec off" exists), and whether the in-product notice fires.

**Why it matters.** It is the class the corpus says will dominate, and the users most exposed are the ones in protected-media mode.

**Proposed fix.** Add an "OS-owned component" class to the record (scope: OS component and version; fix owner: vendor; Izcal mitigation: switch or refusal of protected-media mode below a version, SH-R4 (d)). Add a legal question on Art. 14 for vulnerabilities in OS components a product relies on, with the conservative default of reporting when Izcal's product exposes the flaw. Consider adding "protected-media mode off" to UPD-R5's closed list as a proposed security-scope amendment.

#### C15. Several manufacturer duties are left unowned, including one the CRA strand handed over (major)

**Where.** Section 6 as a whole; the CRA note's Section 5 and its design item **CRA-13** ("Recall and kill-switch procedure for non-conforming builds | Art. 13(21)").

**Problem.** The spec owns none of these, and the strings "13(21)", "13(22)", "13(23)" and "recall" do not occur in it:
- **Art. 13(21):** corrective action, withdrawal or recall of a non-conforming product. CRA-13 was handed to the spec and dropped.
- **Art. 13(22):** information and cooperation on a reasoned request from a market surveillance authority. There is no runbook, contact or time bound.
- **Art. 13(23):** informing authorities and users before ceasing operations. There is no end-of-project plan, though D-S8 is deferred and the project has one maintainer.
- **Art. 13(5):** due diligence on integrated third-party components is implied by cargo-vet and the SBOM, but not stated as the CRA duty with its evidence.

**Why it matters.** The success criterion is that every CRA duty has an owner before the first EU distribution. A recall path in particular needs design (what a withdrawn build does), not only text.

**Proposed fix.** Add a short section: the withdrawal mechanism (a TUF-delivered "withdrawn build" state that the shell shows, distinct from UPD-R5's switches, plus archive removal), the authority-request runbook (who answers, within what time, from the published documentation), the cessation plan (notice period, successor hand-over under MIT, what happens to the signing keys and the CNA), and the Art. 13(5) evidence row in the traceability table. Add targets for the first two.

#### C16. The data inventory and the amendment table miss services the corpus already plans (major)

**Where.** 6.12's inventory; 6.13 (OSA and DSA "do not reach it"); 9.5.

**Problem.** Services Izcal will run that collect personal data and are absent from 6.12:
- **Content-blocking breakage reports** (CB-D18; NW 6.15): scheme and host, an optional path, the settings vector and "optionally … a redacted recording", kept ≤ 30 days†.
- **Phishing reports and site-operator appeals** (phishing 6.6): "reviewed user reports" and a per-entry contest object, so site owners' contact data.
- **The extension and app directory and signing service** (EX-D10, the app host's A23; Stage 2 in S1 8.1): developer accounts, uploads, reviews. A directory that stores third parties' packages and makes them available to the public is plausibly a hosting service, and possibly an online platform under the DSA, which contradicts the unqualified "the DSA's Art. 28 (online platforms) do not reach it". Its CRA role toward third-party extensions is also unexamined.
- **The root program's** communications with CAs (NW-D5) and **Web Push** beyond its retention line.
9.5 has no rows for the content-blocking, phishing, extensions or app-host specs, FI-3 (enterprise policy that turns updates off or forces the security log), or FI-12, the planned privacy spec that will own the "short browser notice".

**Why it matters.** PO-15.1's privacy notices, Art. 30's record of processing and the DPIA scope are built from this inventory. Missing rows become missing notices.

**Proposed fix.** Add the rows with personal data, legal basis (to confirm), retention and host. Qualify 6.13: "the browser itself is not an online platform; the Stage 2 directory is assessed before it opens", and add a legal question. Add 9.5 rows for CB (breakage reports' notice and retention), PH (appeals data), EX/AH (directory: DSA, GDPR, CRA role), FI-3 (policy keys for update opt-out and the security log) and FI-12 (notice ownership).

#### C17. The staffing does not fit the duties, and S1 8.3 is not amended (major)

**Where.** 9.2; 10's holiday risk; SEC 9.2 and SEC-D2; S1 8.3; 9.5.

**Problem.** The two officers sit "**inside** the security track's 2.6–3.8†". That figure is SEC 9.2's in-house share of 101 engineer-months for audits, threat models, fuzzing and the on-call, computed before this spec added CNA duties, CSAF and OSV publication, embargo coordination, twice-yearly CRA drills, AI-scale intake with a human reply to every report, the weekly security slot, and C3's legacy branches. Two people sharing a 24-hour CRA clock means each is on call half of every week indefinitely. The same two people are also SEC-D4's release-authority pair, the TUF `releases` key holders, the CNA's administrative contacts and the EU Login holders, so one departure removes half of every control at once. The 0.5–1† compliance engineer is not added to S1 8.3's table, and 9.5 has no S1 8.3 row.

**Why it matters.** The brief asks whether this is feasible for a one-to-two-person security team. As written, it is not shown to be, and the ledger that funds Stage 1 does not know the spec exists.

**Proposed fix.** Estimate the security-response workload in engineer-months (intake at an assumed report rate, triage, coordination, publication, drills, branches) and state whether SEC 9.2's budget absorbs it or the track grows. Name a third on-call person or an external retainer for weekends and holidays before the first EU distribution. Separate at least one of the concentrated roles. Add the compliance engineer and any growth to S1 8.3 through a 9.5 row.

#### C18. SEC-D4's fallback ships without automatic updates, which the CRA forbids (major)

**Where.** SEC-D4 (decided 0.3); 9.1's "before the first public binary" row; CO-5; 9.5's SEC-D4 row.

**Problem.** SEC-D4's reversal reads: "If D-S8 cannot fund one, the first public binary waits or ships labelled as a preview without auto-update." In the EU, a Release without automatic security updates fails Annex I Part I(2)(c) and CO-5; a "preview" is either an Art. 4(3) testing build, with CO-15's notice and expiry, or a non-conforming product. The 9.5 SEC-D4 row only adds roles to the second person and does not touch the fallback.

**Why it matters.** An accepted decision prescribes a release form this spec makes unlawful in the EU, and nothing reconciles them.

**Proposed fix.** Amend SEC-D4's fallback: "waits, or ships outside the EU only, or ships in the EU only as an Art. 4(3) testing build under CO-15". Add it to 9.5.

#### C19. Q-CO-20 misstates D-S12 and is too narrow (major)

**Where.** Section 7's flag; Q-CO-20.

**Problem.** The spec says testing "black-box against the receiving service's acceptance" is what "D-S12's narrow ruling allows only for compatibility oracles". D-S12 permits recorded black-box observations for adapters "with no normative specification", and its 20 October 2026 clarification (DT-D12) says a repository's protocol JSON files "are neither an input nor an oracle". The CVE record format is the opposite case: it has a normative definition published by the format's owner. The real question is how that definition is classified: as a specification artifact of the format owner, like WebIDL or the standards-body data HGBA-2 2.3.2 already admits, or as a third-party repository file under DT-D12's precedent. The same question applies to every schema-defined format the spec generates (CSAF 2.0, CSAF VEX, OSV, CycloneDX/ECMA-424, SPDX 3.0.1) and to the third-party validators it runs ("CSAF provider checker", schema validation in CO-8). Q-CO-20 asks only about CVE and gives no recommended default.

**Why it matters.** Every projection and the SBOM depend on the answer. Framed wrongly, the owner may decide by analogy to DT-D12 and block the generators, or allow a repository file without recording why it differs.

**Proposed fix.** Reframe Q-CO-20 as an owner decision covering all six formats and the validators: (a) a schema published by the format owner as part of the normative specification is specification text, cited by version, never copied into shipped code; (b) only the published prose plus black-box acceptance tests. Recommend (a) for schemas the standard itself designates normative (CSAF's are part of the OASIS Standard), and (b) where the schema exists only as a repository file. Record the difference from DT-D12 in one sentence. Say whether running a third-party validator as a black-box tool is permitted under the toolchain exception.

### 1.3 Minor

| ID | Where | Problem | Why it matters | Proposed fix |
|---|---|---|---|---|
| C20 | CO-4's failure action; the 9.5 SEC-1/SEC-25 row | "The SEC-1 clock for that issue restarts from the leak." SEC-1's time zero is the earliest event, so a later leak changes nothing; "restarts" reads as a reset that lengthens the bound. Adding "fix publicly visible" as a time-zero candidate has effect only for upstream fixes | The failure action does not bite, and the wording invites a weaker reading | Failure action: a leaked fix is treated as public disclosure, so the issue moves to the exploited-risk path (emergency release or UPD-R5) at once |
| C21 | 6.1 ("Opt-in-only telemetry … keeps the last sign closed"; "Izcal does not need it to self-assess"); 6.2 ("fits this exactly"); 6.13 ("the cost was one OS call"; OSA/DSA "do not reach it") | Readings of law stated as conclusions while the same points are open questions (Q-CO-2, Q-CO-9, Q-CO-16) or unexamined (the Stage 2 directory, C16). Requesting an age signal is not legally free: AB 1043 deems the developer to have "actual knowledge … across all platforms", which Q-CO-16 itself asks about | The brief requires open legal questions to stay questions | Reword each as "the text appears to …; Q-CO-n", or move it to 11.1 |
| C22 | Section 1's success criterion | "Every one of the 14 FI-1 rows has an owner and a target." PO-15.1 (privacy notices, a ship blocker), PO-15.5, PO-15.10, PO-15.11 (trademark policy), PO-16.6 (scale published) and PO-16.10 (enterprise notes) have no CO target | The success claim is false as written, and a ship blocker has only a milestone | Add targets (for example: a notice per 6.12 row published before M7; the scale and enterprise notes per release, checked by `hg-advisory`), or say which rows close by decision only |
| C23 | 9.5's shell and S1 rows | Missing amendments: **SH-D23** (four skippable screens) against a testing-build first-run dialog "that cannot be skipped without reading it", which is also untestable; **SH-D29** ("crash reports are opt-out per crash"); S1 Section 6's crash-report row ("a prompt at each crash with 'send' pre-selected") and Section 1's crash-handler row ("the D-S4 opt-out"); the shell 6.9 "Update channel" row, which lets a Release user move to Beta without a download, against 6.1's "installed only by an explicit download"; the opt-out row placed under "About Izcal", where the shell's group is "Updates and recovery". Shell 6.4 already says "no control is pre-selected", which supports CO-D6 and should be cited | Accepted texts will contradict the design after it is accepted | Add the rows; make the testing notice a dismissible screen whose dismissal is recorded; route a channel change through the notice |
| C24 | 6.5 "Bounty"; 9.5 | The spec adds a condition to SEC-D10's bounty trigger ("triage is staffed for AI-scale volume first", EU-hosted platform) without a SEC-D10 amendment row | Two documents will state different triggers | Add a SEC-D10 row |
| C25 | 6.0 ("Release gates … T2 release checks"); 6.0's crate rules | S1 7.2's T2 is the nightly tier; release checks are the "Release gate" tier. The crate rules ("none … links into a shipped browser binary", input limits for `hg-sbom`) have no G numbers or stated mechanical check | A gate in the wrong tier does not run per release; an unnumbered rule is not enforced by `hg-graph-lint` | Use the Release gate tier; register G230+ with one rule per row and its check |
| C26 | 6.10's (2)(f) and (2)(m) rows; 9.5's storage row | Integrity and corruption reporting already exist (G153's AEAD on every page; SG-11 "preserved or explicitly reported"); the amendment re-specifies them. "Delete all Izcal data on this device" does not list what "all" covers: the OS keychain entries, protected-media mode's WebView2 and WKWebsiteDataStore folders, model files, local dumps, recordings, OS-bound keys. Per-user installs (SEC-D14) also mean "this device" covers only the current OS user | Duplicated rules drift; a deletion that misses OS-held stores fails (2)(m) | Cite SG-11 and specify only the user-facing report; publish the deletion inventory; rename to "on this device for this user" or explain |
| C27 | CO-12; 6.2 "Updates stay available" | Ten-year verifiability is tested on a one-year-old build. Code-signing certificates expire or are revoked, and verifying build-provenance attestations years later depends on GitHub's attestation service continuing | The test does not test the claim | Require timestamped signatures; publish an archive manifest signed by an archival key and checkpointed in the witnessed log; test the oldest archived build each quarter |
| C28 | 6.1 "each release's declaration"; 6.9; CO-10 | A generated declaration of conformity per release, signed by the entity, with a weekly security slot, means 50+ legal signatures a year. Generated documents still need a responsible human sign-off, which the pipeline does not include | The legal statement must be made by a person, not a build job | Offer (a) one declaration per release line listing the versions it covers, updated per release, against (b) one per release; add a named signatory step to the release gate |
| C29 | 6.8 "Published, signed and attested" | The chain "Annex VII lists the SBOM … and Art. 32(5) makes that documentation public, so Izcal publishes it" skips the CRA note's row 4.9: Annex VII(8) includes the SBOM "further to a reasoned request". Producing both CycloneDX and SPDX doubles maintenance for no recipient that requires both | The conclusion (publish) may be right, but its stated basis looks shaky; the dual format is copied practice | State publication as Izcal's choice; pick one primary format and generate the other only when a recipient asks, until an implementing act decides |
| C30 | 6.5 "Downstream notice" | "Notice 72 hours before release" cannot hold on the emergency path (≤ 72 h from time zero in total), and under FI-D10 Izcal builds its own Linux packages, so for them pre-notice is internal | The rule will be broken on exactly the releases that matter | State the emergency-path rule (notice when the fix is final, embargo ≤ 24 h†) and limit the list to third-party packagers |
| C31 | 6.17 "AI off" | AI triage of untrusted reports "is an operational choice". SEC-27 forbids any AI agent "with a shell, write token, secrets or cache write" on untrusted input in CI; reports and reproducers are untrusted input and can carry prompt injection | The intake path is an injection surface into the most sensitive repository | Apply SEC-27's rule to triage tooling: read-only, no network, no secrets, no repository write |
| C32 | Section 7 | No "dependencies used unmodified" list, which the template requires: OpenPGP signing for CSAF and the mailbox, schema validators, the CSAF checker, the RustSec database and OSV service as gate inputs, GitHub, the CDN, ENISA's platform. The licence and M-1 standing of each tool is unstated | The M-1 section cannot be checked | List each with its role (shipped, tool, service, data input), licence class and why it is allowed |
| C33 | 6.3 path 1; CO-D18 | The weekly slot cites Chrome's reason (public fixes create a patch gap), which this spec removes by keeping fixes private until release (CO-4). The remaining reason (planning) is cheap compared with weekly reproducible, signed, CSAF-published releases on every supported line | The cadence is copied without the constraint that drove it | Justify the weekly slot from Izcal's own constraints, or default to on-demand security releases bounded by SEC-1/SEC-25, and keep weekly as the alternative |
| C34 | Section 8 and 9.3 | Most † values have no rationale (24 h† assessment, 4 h† drill, 7 days† triage, 90%† uptake, 90 days† log, weekly† notice, 24 h† rollout, 2 years† contact); only the 30-day† Nightly expiry has one. CO-D9, CO-D17 and the SBOM-depth row have no reversal trigger | The house style gives each † a rationale and each decision a trigger | Add one-line rationales and triggers |
| C35 | 6.4 "In-product notice"; 6.2; 6.3 | The data source for "an installed build has an exploited vulnerability" and for a platform drop is unstated (presumably a field in TUF targets metadata, so no new fetch under NW-36). Whether UPD-R5 switches still apply after the user opts out of automatic security updates is unstated | An implementer could add a fetch, or silently ignore the opt-out | Specify the metadata field and an UPD-R1 amendment; state the UPD-R5 rule under opt-out |
| C36 | 6.13; CO-21 | Requesting the signal "at launch" may show an OS consent sheet on each launch, depending on the platform API's behaviour, which the notes do not establish (macOS returns no value per a forum report; the Windows API is Low). Under-18 suppression covers crash upload and telemetry but not the feedback form. CO-21 is "Ship blocker … from 1 January 2027", years before any binary | Possible nag against SH-D23's spirit; a child's free text is still collected; a dated ship blocker with no binary is not a target | Make a black-box measurement of each OS API's UI a precondition; extend suppression to feedback; key CO-21 to the first binary on each OS |
| C37 | 6.11; CO-16 | CO-16's failure action publishes unbacked criteria as "Not Evaluated". My recollection is that the VPAT 2.x template allows "Not Evaluated" only for WCAG AAA criteria; if so, the failure action is not allowed for A and AA (flag for the fact-checker) | The failure action may be invalid under the template the spec adopts | Fact-check the template's rule; if it holds, use "Partially Supports" with remarks citing the missing test |
| C38 | 11.1's Owner column | "Compliance" owns most questions, but no role of that name exists in 9.2 (there is a compliance engineer, an engineer, and external counsel) | Legal questions need an owner who can commission counsel | Name the owner (for example the data-protection lead or the owner) per question |

---

## 2. Contradictions with accepted decisions

| # | Accepted decision (quote) | This spec (quote) | Finding |
|---|---|---|---|
| 1 | SEC-1: time zero "taken from the report, disclosure or detection record, **never entered after the fact**" | CO-7: "time-zero fields entered ≤ 24 h† after the event" | C1 |
| 2 | SEC 6.16: update metadata and binaries "served from storage Izcal does not trust (**GitHub or a CDN**)" | 6.12/CO-18: "Raw CDN and update logs with IP addresses deleted ≤ 24 h†"; CO-D17: "EU only" | C2 |
| 3 | NW-D17: OHTTP needs "A relay operator contract (Q-NW-6)"; fallback "(b) direct to Izcal's servers" | 6.12: "Uploaded through the OHTTP relay (NW-D17), so ingestion never sees the IP"; CO-18: "**0** client IPs seen by crash and telemetry ingestion" | C2 |
| 4 | GR-D21: "Izcal supports an OS version while its vendor ships security updates to consumers" | CO-D5 (a): a legacy branch per dropped OS for up to five years, costed as "an extra backport target" | C3 |
| 5 | UPD-R4/SEC-1: emergency release ≤ 72 h† for exploited bugs | CO-9: "**0** open critical, high or exploited vulnerabilities … Release blocked" | C4 |
| 6 | SEC-25: "critical issues in Izcal's own code within 14 days† and high within 30 days†"; SEC-1: engine critical or high ≤ 7 days† | 6.4: critical = sandbox escape (bound SEC-25); high = renderer code execution (bound SEC-1) | C5 |
| 7 | SH-D11: the ledger is "the canonical account of consent"; shell 6.7: "Private windows write nothing" | 6.10: a second, broker-side log of grants, overrides and attachments, 90 days†, with no key, clearing or private-window rule | C9 |
| 8 | SH-28: "**0** identifiers … in telemetry"; SH-D29: "collects no identifiers" | 6.12: "Telemetry uses the same pattern **per reporting period**" | C11 |
| 9 | FI-D10: Linux packages are "updated by the system package manager … the per-user TUF updater stays off for them" | CO-5: "On all three platforms: on by default … Ship blocker" | C12 |
| 10 | S1 13.1a (D-S7): "a Widevine license request to Google; a castlabs third-party-licensee quote" | 6.16: "shipping closed cryptography, is already ruled out by D-S7" | C13 |
| 11 | SEC-D4 reversal: "the first public binary waits or ships labelled as a preview **without auto-update**" | Annex I Part I(2)(c) via 6.3 and CO-5: automatic security updates on by default | C18 |
| 12 | D-S12 / DT-D12: repository JSON files are "neither an input nor an oracle" for adapters with no normative specification | Section 7: black-box testing is what "D-S12's narrow ruling allows" for the CVE format | C19 |
| 13 | SH-D23: "Onboarding is **four skippable screens**" | 6.1: a first-run dialog "that cannot be skipped without reading it" | C23 |
| 14 | SEC-27: "No AI agent with a shell, write token, secrets or cache write runs on untrusted input" | 6.17: AI triage of reports "is an operational choice" | C31 |

No contradiction found with D-S10, HGBA-2 2.3.2's rule for shipped code (the four `hg-*` crates are tools and FI-D13 covers servers), FI-D12 (CDN aggregation is server-side and needs no client change unless C6's signal is added), NW-36 (the age-signal call and the security log add no browser traffic; the deletion endpoint must join NW-D17's list, C11), AI-off B10, or the process map's placement of a broker-written store in the storage service (the problem is the store's rules, C9, not its process).

---

## 3. M-1 and owner-rule compliance

| Rule | Status | Evidence and required change |
|---|---|---|
| **M-1 part 3: no third-party source code as evidence** | **Compliant.** | The four notes carry explicit M-1 statements; the operations note avoided "JSON schemas inside code repositories". The only code-host domains cited are `docs.github.com` (prose documentation) and `ossf.github.io/osv-schema/` (the rendered specification site, the outside copy OD-7 asks for). Chrome's severity guidelines were deliberately not read. No value is derived from code. |
| **M-1: framing of schema-defined formats** | **Not settled.** | Q-CO-20 misstates D-S12 and covers only CVE; reframe as an owner decision over CSAF, VEX, OSV, CVE, CycloneDX and SPDX, plus validators (C19). |
| **MIT for what ships (HGBA-2 2.3.2)** | **Compliant today, conditional later.** | Nothing in the spec ships in the browser. A licensed closed CDM under D-S7's track would change export, Art. 32(5) and SBOM treatment (C13). |
| **Dependencies listed (template Section 7)** | **Missing.** | No list of the tools, services and data inputs the pipelines use, or their standing (C32). |
| **Build our own** | **Compliant.** | The generators, record and ACR are Izcal's own, built from published specifications. |
| **Privacy by construction** | **Strong in intent, with gaps.** | No per-install ID, local age signal, deletion tokens, OHTTP and the affirmative crash consent are right. Gaps: hosting and relay (C2), linkable telemetry tokens (C11), the header's identifiers (C10), the security log (C9), reproducers (C8), projections' disclosure classes (C7). |
| **No email address anywhere** | **Compliant.** | Neither the spec nor this critique contains one. |

---

## 4. Verdict

**Is the design sound?** In direction, yes. Treating compliance as two pipelines that run every release is the right first-principles move for a project that ships every few weeks and already keeps its specs, threat models and test results as data. The decisions most likely to survive are good: plan as a manufacturer; take the Art. 32(5) route and tie it openly to D-S8; run Nightly and Beta as Art. 4(3) testing builds with per-build expiry and Art. 14 reporting until counsel says otherwise; make crash consent an affirmative act; become a CNA under ENISA's root and publish machine-readable advisories under Izcal's own IDs; require a reproducer and defer any paid bounty; generate the ACR from the tests and say "Not Evaluated" rather than "Supports" without evidence. The legal questions are, with the exceptions in C21, kept as questions with conservative defaults and reversals, and the history section draws real lessons rather than templates.

**Is it implementation-ready?** No. Three blockers and sixteen major findings remain. The draft is a good 0.1; it is not a 0.2.

**What must change before draft 0.2.**
1. Resolve C1: define the Art. 14 time zero as a record field, machine-stamp receipt clocks, and fix the checkpoint side channel.
2. Resolve C2: a hosting table for every data flow, and either EU hosting for the security repository or a restated CO-D17; make CO-18 conditional on Q-NW-6.
3. Resolve C3: add the per-platform support statement as an option, cost all three in branches and people, and recommend from the cost.
4. Fix the targets that cannot be met as written: C4 (CO-9 and CO-3), C5 (the bound inversion), C6 (CO-24).
5. Fix the privacy gaps in the response design itself: C7, C8, C9, C10, C11.
6. Close the missed constraints: C12 (Linux auto-update), C13 (D-S7's licensed CDM), C14 (OS components), C15 (Art. 13(21)–(23)), C16 (services and amendment rows), C17 (staffing and S1 8.3), C18 (SEC-D4's fallback), C19 (Q-CO-20).
7. Fix the minors C20–C38, especially the shell and S1 amendment rows (C23), the deletion inventory (C26) and the VPAT check (C37).

**The five findings that most affect the design.**
1. **C1**: the CRA clock. It is the duty with the highest fine and the spec's headline claim about witnessed clocks.
2. **C2**: hosting. It decides whether the privacy promises of 6.12 and CO-18 are true.
3. **C3**: the OS-drop default. It is the largest cost in the document and it is unpriced.
4. **C4/C5**: the release gate and the severity bounds. They decide whether the emergency path works and whether the scale means what it says.
5. **C17**: staffing. Every operational target assumes two officers who are also every other control's second person.

---

## 5. Summary

| ID | Severity | One line |
|---|---|---|
| C1 | Blocker | Art. 14 time zero undefined; witnessed log witnesses entries, not events; manual entry contradicts SEC-1; checkpoint side channel |
| C2 | Blocker | Security repo on GitHub, GitHub PVR, global CDN and an unchosen OHTTP relay contradict CO-D17, 6.17 and CO-18 |
| C3 | Blocker | CO-D5's legacy-branch default implies many five-year branches; the per-platform support statement is missing as an option |
| C4 | Major | CO-9 blocks emergency releases and most releases; CO-3 contradicts the moderate/low vehicles |
| C5 | Major | Critical (sandbox escape) gets 14 days, high (renderer RCE) 7 days |
| C6 | Major | CO-24's uptake share has no measurable denominator; small cells published |
| C7 | Major | Projections lack field disclosure classes; "cannot disagree" holds only at generation |
| C8 | Major | Reproducers may become public regression tests; retention rules disagree |
| C9 | Major | Broker security log duplicates the ledger and escapes private-window and clearing rules |
| C10 | Major | Header: SBOM-digest hash cycle, fields redundant with BuildId, profile ID leaves via exports, policy digest identifies |
| C11 | Major | Per-period telemetry token contradicts SH-28; wipe strands server data; no Art. 15 access |
| C12 | Major | CO-5's Linux ship blocker vs FI-D10's package-manager updates |
| C13 | Major | D-S7 does not rule out a licensed closed CDM; Art. 32(5), export and SBOM consequences unhandled |
| C14 | Major | Vulnerabilities in OS components (protected-media host, OS decoders, drivers) have no path |
| C15 | Major | Art. 13(21) recall (the note's CRA-13), 13(22), 13(23) and 13(5) unowned |
| C16 | Major | Data inventory and 9.5 miss CB breakage reports, phishing appeals, the extension directory (DSA), FI-3, FI-12 |
| C17 | Major | Officers placed inside a pre-existing budget; 2-person 24 h on-call; role concentration; no S1 8.3 row |
| C18 | Major | SEC-D4's "preview without auto-update" fallback is unlawful in the EU and unamended |
| C19 | Major | Q-CO-20 misstates D-S12, ignores DT-D12 and covers only one of six schema formats |
| C20 | Minor | CO-4's "clock restarts from the leak" does not bite |
| C21 | Minor | Legal readings stated as conclusions |
| C22 | Minor | Six FI-1 rows have no target despite the success claim |
| C23 | Minor | Missing SH-D23, SH-D29, S1 §1 and §6 crash rows, shell channel-row amendments |
| C24 | Minor | SEC-D10 bounty trigger changed without an amendment row |
| C25 | Minor | Release gates placed in nightly T2; crate rules lack G IDs and checks |
| C26 | Minor | Storage amendment duplicates SG-11; deletion inventory and "this device" scope unstated |
| C27 | Minor | Ten-year verifiability tested on a one-year-old build |
| C28 | Minor | Per-release declarations need a human signatory; per-line option missing |
| C29 | Minor | SBOM publication reasoning skips Annex VII(8); dual formats copied |
| C30 | Minor | 72 h distro pre-notice impossible on the emergency path |
| C31 | Minor | AI triage of untrusted reports must follow SEC-27 |
| C32 | Minor | Section 7 lacks the dependencies-used list |
| C33 | Minor | Weekly slot copies Chrome's reason that private fixes remove |
| C34 | Minor | Most † lack rationales; three decisions lack reversal triggers |
| C35 | Minor | Data source for in-product notices and UPD-R5 under opt-out unstated |
| C36 | Minor | Age-signal API UI unmeasured; feedback not suppressed; CO-21 dated before any binary |
| C37 | Minor | VPAT "Not Evaluated" may be AAA-only (flag); CO-16's failure action at risk |
| C38 | Minor | "Compliance" owner role undefined |

**Counts: 3 blockers, 16 major, 19 minor (38 findings).**
