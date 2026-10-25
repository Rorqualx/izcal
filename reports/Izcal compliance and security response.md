# Izcal compliance and security response

*First-principles specification of Izcal's legal duties as a software manufacturer and of the security-response operation that carries most of them: the EU Cyber Resilience Act (role, conformity route, technical documentation, declaration and CE marking, user information, support period, security updates, SBOM, vulnerability handling, corrective action and the Article 14 reporting runbook), the vulnerability record and its projections (advisories, CVE records, CRA notifications, release notes), intake, severity, embargoes and CVE Numbering Authority status, the accessibility conformance report, data protection for the services Izcal runs, the DMA as it applies to a non-gatekeeper, age signals, terms, privacy notices, licence notices, trademark policy and export control. Date: 24 October 2026. Status: draft 0.4, owner decisions recorded: CO-D1 to CO-D21 accepted on 25 October 2026. Research notes: `research_notes/Izcal compliance and security response/` (`history_postmortems.md`, `cra_eu_product_law.md`, `security_response_operations.md`, `data_access_platform_law.md`; phase 8: `factcheck.md`, `critique.md`). Relates to: the feature inventory audit [FI-1, FI-2, FI-3, FI-11, FI-12, FI-33, FI-35, FI-D2, FI-D10–FI-D15, Q-FI-1, Q-FI-2, Section 8][FI]; the security scope [SEC-1, SEC-2, SEC-13–SEC-17, SEC-24, SEC-25, SEC-27, SEC-35, SEC-D2, SEC-D4, SEC-D10, SEC-D14, UPD-R1–UPD-R5, SH-R4, SH-R5, Sections 6.8, 6.16, 6.17][SEC]; the Stage 1 design [Sections 1, 2.2, 6, 7.2, 7.7, 7.8, 8.1 (M1), 8.3, 13.1a, 13.2; D-S4, D-S7, D-S8, D-S9, D-S10, D-S12][S1]; the shell [6.4, 6.9, 6.11, 6.12, 6.14, 6.16; SH-3, SH-28; SH-D11, SH-D20, SH-D23, SH-D24, SH-D29][SH]; the shell UI design [UI-1][UI]; networking [NW-D17, NW-36, Q-NW-6][NW]; graphics [GR-D21][GR]; storage [SG-11][SG]; HGBA-2 [M-1, 2.3.2][H2].*

*Revision 0.4 (25 October 2026): owner decisions. **The owner accepted every recommendation in Section 9.3 (CO-D2 to CO-D21) on 25 October 2026**, after CO-D1 earlier the same day; CO-D2 (self-certification under Art. 32(5) while every edition is free and open-source software, with a closed edition or module taking a notified-body or harmonised-standard route whose cost D-S8 weighs), CO-D3 (Art. 4(3) testing builds), CO-D5 (option (c), the per-platform support statement at release, reviewed together with the distribution spec's DU-D15), CO-D6 (nothing pre-selected, which amends D-S4) and CO-D20 (normative schemas are allowed as test oracles) were reviewed explicitly. Each row of 9.3 is marked accepted; the D-S4 amendment in 9.5 is now owner-approved. Crate rules G260–G262 unchanged. No decision text, target or threshold changed.*

*Revision 0.3 (25 October 2026): owner decisions recorded. **CO-D1 accepted**: an EU-established legal entity before the first EU distribution. **With the distribution spec's DU-D10, signed public Nightlies start at least 3 months before the first Stable, as Art. 4(3) testing builds (CO-D3)**, so the first public binary is the first public Nightly: the CNA, security-response, disclosure, intake and external security-audit gates move to the first public Nightly (9.1, CO-25), while the conformity set, declaration, CE marking and Annex II page stay with the first Stable. The distribution spec's cadence (a 4-week Stable, later 2 weeks, with security-only point releases) is adopted as the release lines this spec's per-release targets run on (6.3). **The recording and crash-report header has one definition, owned by FI-11**: 9.4 now lists this spec's fields as a contribution to it, beside the enterprise-policy spec's 32-byte `PolicyDigest` and the OS-integration spec's recorded OS inputs. Section 6.10a (the accessibility conformance report), dropped by mistake in the 0.2 rewrite, is restored with its 0.1 content and 0.2's corrections. No target, threshold or other decision changes.*

*Revision 0.2 (24 October 2026): phase 8 applied. **Fact-check** (`factcheck.md`): the official CRA and PLD texts were read from the Publications Office's rendering of the Official Journal (EUR-Lex served bot challenges); 53 CRA provisions checked, **Art. 32(5) confirmed verbatim**; 0 blockers, 5 major, 23 minor, all applied: Art. 69(2) covers the whole Regulation, not only Annex I, so the testing-build notice before 11 December 2027 is preparation, not a duty (F1); CNA two contacts is a recommendation, human response a requirement, no assignment-policy rule (F2, F3); CSAF trusted-provider roles restated with the options Izcal uses (F4); a misattributed quote replaced (F5); paragraph numbers, the Art. 4(3), 13(10), Part II(4), Part I(2)(c) and Art. 64(10) wording, the PLD row, the security.txt `CSAF` field's source, SEC-D2's track size, UPD-R4's scope, citations and one link corrected (F6–F27). **Design critique** (`critique.md`): 3 blockers, 16 major, 19 minor, all applied, three in a different form from the one proposed (Appendix A): the Art. 14 clock now starts at a defined, machine-stamped field, and checkpoints are published on a fixed schedule (C1); a hosting table, EU hosting for the security repository and record, GitHub's private reporting dropped, and CO-18 conditional on the relay (C2); a per-platform support statement fixed at release replaces five-year legacy branches as the OS-drop default, with all options costed (C3); release gates, one severity-bound table, an uptake estimate that needs no client signal, disclosure classes for record fields, one retention table, the prompt ledger extended instead of a second log, the M1 header redesigned, per-record deletion tokens, a Linux automatic-update design, D-S7's licensed-CDM trigger, an OS-component class, corrective action and cessation (new 6.11), missing services, staffing with an S1 8.3 row, SEC-D4's fallback, and the schema-format question as owner decision CO-D20 (C4–C19); minors C20–C38 applied.*

*Revision 0.1 (24 October 2026): first draft, written by the synthesis author from the four research notes and the specs named above. No other report was edited.*

Izcal has no public binary yet, but the law it will meet already applies. The EU **Cyber Resilience Act** (CRA) lists "standalone and embedded browsers" as Class I important products. Its reporting duty for actively exploited vulnerabilities has applied **since 11 September 2026**, and the revised **Product Liability Directive** treats software as a product for products placed on the market after **9 December 2026**. Both reach Izcal through the same test, **commercial activity**, which depends on the funding model the owner has deferred (D-S8). This spec plans for the stricter reading, as FI-D11 decided: **Izcal is a manufacturer**. Its central idea is that compliance is not a document set written once but **two pipelines that run on every release**. The first is a **security-response pipeline** around one **vulnerability record**, whose receipt clocks are stamped by machines and committed to a witnessed log, and from which every advisory, CVE record, CRA notification, release note and quarterly report is generated with field-level disclosure rules. The second is a **conformity pipeline** that generates the technical documentation, SBOM, declaration of conformity, user security information and accessibility conformance report from the specs and the test results of that release. The decisions that matter most are: (1) the **conformity route**: Art. 32(5) lets a free and open-source manufacturer self-assess a Class I product without a harmonised standard if it **publishes its technical documentation**, which holds only while every Izcal edition is free software (CO-D2, tied to D-S8); (2) **Nightly and Beta** run as the CRA's "unfinished software" for testing, with a visible notice, opt-in only and an expiry per build, until legal advice settles whether a standing channel qualifies (CO-D3); (3) a **support period of at least five years**, stated per platform at release so that an OS leaving its vendor's support does not create five-year legacy branches (CO-D4, CO-D5); (4) **crash-report consent** changes so that nothing is pre-selected, because EDPB guidance puts crash uploads inside ePrivacy Art. 5(3) (CO-D6, amends D-S4); and (5) **Izcal becomes a CVE Numbering Authority** under ENISA's Root and publishes its own machine-readable advisories, so it depends on no single registry (CO-D7, CO-D8).

## 1. The problem and who has it

**The problem.** A browser vendor is a regulated manufacturer and a security-response operation as much as an engineering project. The audit's work item FI-1 found 14 rows with no owner: the CRA (PO-15.9, a ship blocker), privacy notice and terms (PO-15.1, a ship blocker), the browser UI's accessibility conformance report (PO-14.6, a ship blocker for public-sector buyers), GDPR access and deletion for server-held data (PO-15.3, 15.4), DMA choice screens and data portability (PO-15.5, 15.6), age assurance (PO-15.7), EAA exposure (PO-15.10), trademark rules (PO-15.11), end-of-support handling (PO-3.15), CVE assignment (PO-16.3), a severity scale (PO-16.6) and enterprise security notes (PO-16.10) ([FI 4.1][FI]). The corpus specifies a security model in depth, but S1 13.2 gives security response two lines ("a published vulnerability-disclosure policy, security advisories, an embargo process … and a bug bounty once funding allows") and privacy and legal four ([S1 13.2][S1]).

**Who has it.**

| Who | Their task | What fails today without this spec |
|---|---|---|
| **A user** | Learn that a fix exists, get it without effort, know how long the build is supported, remove their data | No support date, no end-of-support notice, no opt-out or postpone rule for automatic updates, no "delete everything" that covers server data |
| **A security researcher** | Report a bug, know when they will hear back and when it will be public, get credit | No intake channel, no published times, no deadline, no safe harbour |
| **A CSIRT and ENISA** | Receive the 24-hour early warning, the 72-hour notification and the final report | No runbook, no accounts on ENISA's platform, no rule for which CSIRT |
| **A market surveillance authority** | Read the technical documentation, the declaration and the SBOM; order a corrective action | None exists; no withdrawal mechanism |
| **An enterprise or public-sector buyer** | Read an accessibility conformance report and enterprise security notes; track advisories by machine | None exists; no CVE status, no CSAF feed |
| **A Linux packager or downstream distributor** | Get embargoed notice before a fix ships; know what the trademark allows | No notice list, no trademark policy |
| **A data subject** | Exercise access and erasure for crash reports, telemetry, sync data | No path, because the services carry no identity |
| **The owner** | Know the legal exposure of each funding option before choosing one | The CRA, PLD and GDPR consequences of D-S8 are unstated |

**What the existing specs already decide.** HGBA-2 **M-1** (original implementation) and **2.3.2** (everything Izcal ships, code and data, is MIT; anything else Izcal builds) ([H2][H2]); **D-S4** (usage telemetry opt-in; crash reports opt-out with a first-run disclosure and a per-crash prompt with "send" pre-selected, minidumps only) ([S1 6, 11][S1]); **D-S7** (licensed DRM only: the protected-media mode uses the OS engine's, native PlayReady uses the module Windows ships, and a licensing track pursues Widevine and a castlabs licence) ([S1 13.1a][S1]); **D-S8** (funding deferred); **D-S9** (the name Izcal, with a trademark clearance search before the first public binary); the security scope's **SEC-1** (time zero is the earliest of report, disclosure or detection, "never entered after the fact"; fixed release ≤ 72 h† for exploited, ≤ 7 days† for critical or high in the engine), **SEC-2** (quarterly emergency-release drill), **SEC-25** (acknowledge ≤ 3 days†; critical ≤ 14 days†, high ≤ 30 days† in Izcal's own code), **SEC-35** (dumps deleted after 30 days†), **SEC-D2** (a security track of 3–4†), **SEC-D4** (a second person with release authority before the first public binary), **SEC-D10** (an external audit before the first public binary; a bounty when D-S8 allows), **SEC-D14** (per-user installs in Stage 1) and **UPD-R1 to UPD-R5** (TUF channels, the emergency release class and the emergency mitigation channel) ([SEC 6.8, 6.16, 6.17, 8, 9.3][SEC]); the shell's **SH-D11** (the prompt ledger is the canonical account of consent), **SH-D24** (WCAG 2.2 AA via WCAG2ICT, UAAG 2.0), **SH-28** and **SH-D29** (no identifiers in telemetry) and **UI-1** ("a published UAAG 2.0 conformance statement") ([SH][SH]; [UI][UI]); **NW-D17** (Izcal's own traffic goes to a published endpoint list; user-specific requests go through an Oblivious HTTP relay whose operator is open, Q-NW-6) and **NW-36** (no other traffic) ([NW 6.13][NW]); **GR-D21** (an OS version is supported while its vendor ships security updates) ([GR][GR]); and the owner's decisions of 24 October 2026: **FI-D10** Linux packages are updated by the system package manager, **FI-D11** plan as a CRA manufacturer and get legal advice before the first EU distribution, **FI-D12** opt-in usage telemetry only and no server-side experiments, **FI-D13** the MIT rule covers what Izcal ships, not servers it runs, **FI-D14** open protocols first, **FI-D15** rely on the OS for family controls and honour its age signal ([FI 7.1][FI]).

**Success, in terms someone can measure.** Each of the 14 FI-1 rows is closed here, by a target or, where nothing recurs, by a decision and a milestone (Appendix B maps them). Before the first EU distribution, the CRA reporting runbook has been drilled, the corrective-action mechanism works, and every legal question this spec cannot settle from primary text has an answer from counsel (Section 11). Before the first EU Release, the technical documentation for it is generated and published. After that, the targets of Section 8 are measured from the vulnerability record and the release pipeline, not asserted.

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project or event | What it did | Why that way | Fate, and why | What survived |
|---|---|---|---|---|---|
| 1988 | **CERT/CC** | A neutral incident-response and coordination centre ([SEI](https://www.sei.cmu.edu/divisions/cert/)) | The Internet had no one to call when a worm spread | Still running | The neutral coordinator for multi-vendor cases |
| 1993–2021 | **Bugtraq** full disclosure | Public posting of details, often with no vendor notice | Vendors ignored private reports; publishing was the only lever | Posting stopped in 2020; the January 2021 shutdown notice said resources "have not been prioritized" ([seclists](https://seclists.org/bugtraq/2021/Jan/0)) | The deadline as the researcher's lever |
| 2004–2005 | **Mozilla bounty and MFSA advisories** | USD 500 bounties; numbered advisories from MFSA 2005-01 with a four-level impact key ([Mozilla](https://www.mozilla.org/en-US/security/advisories/)) | A small non-profit wanted outside eyes; distributions needed one list to track | Running | Numbered per-release advisories and a short browser-specific severity scale |
| 2010 | **"Coordinated" vulnerability disclosure** | Microsoft renamed responsible disclosure; "In essence, coordinated vulnerability disclosure would work much like responsible disclosure" ([The Register](https://www.theregister.com/2010/07/22/microsoft_coordinated_disclosure/)) | The responsible-versus-full argument blocked cooperation | The term won; ISO, FIRST and the CRA use it | Coordination as the norm, with the vendor still setting the pace |
| 2014, 2018–2019 | **ISO/IEC 29147 and 30111** | Standards for receiving reports and handling them ([ISO 72311](https://www.iso.org/standard/72311.html); [ISO 69725](https://www.iso.org/standard/69725.html)) | Buyers and governments needed something to cite | Current editions 2018 and 2019, both under revision | The vocabulary the CRA and the UK PSTI rules use |
| 2014 | **Heartbleed** | One OpenSSL bug in most TLS servers; OpenSSL notified several Linux distributions on 7 April 2014, the day of release, and missed others ([Durumeric et al.](https://nebelwelt.net/files/14IMC.pdf)) | An unfunded project with no notice list | New funding for core projects | Plan downstream notice before it is needed |
| 2014–2021 | **Project Zero deadlines** | 90 days; then 90+30 and 7+30 for exploited bugs, after the 2020 "Full 90" trial "didn't observe a significant shift in patch development timelines" ([Project Zero](https://projectzero.google/2021/04/policy-and-disclosure-2021-edition.html)) | Vendors' timelines had no limit, and users install patches late | Running; a reporting-transparency trial from 29 July 2025 ([Project Zero](https://projectzero.google/reporting-transparency.html)) | A fix deadline plus a patch-adoption window |
| 2018 | **Meltdown and Spectre** | Months of vendor-by-vendor embargo; many distributions and the BSDs were "left out in the cold" ([LWN](https://lwn.net/Articles/743363)) | Hardware flaws needed every OS and compiler | The embargo broke early; the kernel wrote down a process for embargoed hardware issues ([kernel docs](https://docs.kernel.org/process/embargoed-hardware-issues.html)) | A small coordination team with a documented contact process |
| 2019–2023 | **The Chrome patch gap** | Fixes in public source shipped weeks later; Google cut the gap from 35 days to about 15 with biweekly updates, then went weekly from Chrome 116 on 8 August 2023 ([Google](https://security.googleblog.com/2023/08/an-update-on-chrome-security-updates.html)) | An open repository makes every fix visible before it ships | Weekly security updates continue alongside two-week milestones from Chrome 153, 8 September 2026 ([Chrome](https://developer.chrome.com/blog/chrome-two-week-release)) | Shipping cadence as a security control |
| 1999–2026 | **CVE and NVD** | One shared ID per vulnerability; an expanding set of CNAs from 2016, and federated Roots since ([cve.org](https://www.cve.org/About/History)) | Every database used its own names | MITRE's contract nearly lapsed on 16 April 2025 ([Nextgov](https://www.nextgov.com/cybersecurity/2025/04/cisa-extends-mitre-backed-cve-contract-hours-its-lapse/404601)); NIST enriches only priority CVEs from 15 April 2026 ([NIST](https://www.nist.gov/itl/nvd)) | The ID; and the lesson not to depend on one registry |
| 2021 | **Log4Shell** | A library flaw everywhere, with no inventory of where; the CSRB called it "endemic" ([CSRB](https://www.cisa.gov/sites/default/files/publications/CSRB-Report-on-Log4-July-11-2022_508.pdf)) | Volunteer maintainers; no component inventories | Long tail | SBOMs and machine-readable advisories as baseline duties |
| 2023–2026 | **Apple RSR → Background Security Improvements** | Out-of-band Safari and WebKit fixes; the newer mechanism installs automatically when the setting is on ([Apple](https://support.apple.com/en-us/102657)) | Full OS updates were slow | RSR was rarely used; BSI carried its first WebKit fix on 17 March 2026 (secondary) | A mitigation channel between releases |
| 2024 | **xz-utils backdoor** | A maintainer account inserted a backdoor into release tarballs; disclosure went to Debian and the distros list, not upstream ([oss-security](https://www.openwall.com/lists/oss-security/2024/03/29/4)) | One tired maintainer; a multi-year takeover | Caught by chance | Release integrity is part of security response; a second release signer |
| 2024 | **Linux kernel CNA** | The kernel issues its own CVEs ([LWN](https://lwn.net/Articles/978711)) | It wanted control of its records | Thousands of CVEs; disputed volume | Publish the assignment rule |
| 2022–2024 | **The CRA** | Manufacturer duties for every product with digital elements | Software had no EU baseline | The 2023 open-source letter warned of a "chilling effect" ([Eclipse](https://newsroom.eclipse.org/news/announcements/open-letter-european-commission-cyber-resilience-act)); the final text added the open-source steward and the commercial-activity tests; reporting from 11 September 2026 ([EUR-Lex][CRA-EL]) | Commercial status turns on price, monetisation, data and donations |
| 2025–2026 | **AI-scale reports** | curl's confirmed-report share fell from over 15% to under 5%; it ended its bounty on 31 January 2026 ([Stenberg](https://daniel.haxx.se/blog/2026/01/26/the-end-of-the-curl-bug-bounty/)); Chrome cut and then partly restored rewards in 2026 (secondary, [SecurityWeek](https://www.securityweek.com/google-partially-reverses-chrome-bug-bounty-cuts/)) | Report volume grew faster than triage | Bounties now ask for a reproducer | Intake must filter at scale before money is offered |
| 2001–2025 | **VPAT and browser ACRs** | Self-attested conformance reports; ITI "does not review or approve VPATs" ([ITI](https://www.itic.org/policy/accessibility/vpat)) | US procurement under Section 508 needed a vendor statement | Chrome's latest is Chrome 134 (April 2025, one platform per report); Firefox's is Firefox 124 on Windows (April 2024); Apple's macOS report never names Safari ([HIST 5][HIST]) | The template; not its staleness |

### 2.2 Technology mismatches: assumptions that no longer hold

| Old assumption | Why it held | Why it no longer holds |
|---|---|---|
| Disclosure is between one researcher and one vendor | No regulator was involved | The CRA puts a CSIRT and ENISA in the loop within 24 hours of awareness of exploitation (Art. 14) |
| The patch gap is measured from the report | Closed-source fixes were invisible until release | Izcal's source is public; a fix is visible the moment it lands |
| A human-readable advisory page is enough | Readers were people | Scanners, SBOM tools and Art. 14(8) expect machine-readable advisories |
| CVE and NVD will enrich and score records | NVD did so | NVD now enriches only priority CVEs; third-party rescoring overrides the vendor ([HIST 3.4, 3.9][HIST]) |
| A bounty brings signal | Reports cost effort | AI tools make reports cheap; money attracts noise |
| Open source is outside product law | No EU product law named software | CRA and PLD reach free software supplied in a commercial activity |
| An accessibility report is a yearly PDF | Releases were yearly | Browsers ship every two weeks; a report for one old version on one platform is stale on publication |
| Compliance is a document set written once | Products changed rarely | A feature release may be a substantial modification (CRA recital 39); conformity must be re-established per release |

## 3. Today: the landscape and its gaps

**The duties, by date.** Izcal's first public binary is years away (S1 8.1 puts the Stage 1 exit at 48–60 months†), so by the time a Release ships, the CRA will apply in full.

| Date | What applies | Reaches Izcal? | Source |
|---|---|---|---|
| 28 June 2025 | European Accessibility Act | Indirectly: the Directive never names browsers | [FI factcheck 5a][FIFC] |
| 11 June 2026 | CRA Chapter IV (notified bodies) | Only on a notified-body route | [CRA Art. 71][CRA-EL] |
| 11 September 2026 | CRA Art. 14 reporting, including products placed on the market before 11 December 2027 (Art. 69(3)) | From the first EU distribution | [CRA Arts. 14, 69, 71][CRA-EL] |
| 11 September 2026 | ENISA's single reporting platform live; EU Login with MFA; English only; no API | Yes | [ENISA SRP FAQ](https://www.enisa.europa.eu/topics/product-security/single-reporting-platform-srp/frequently-asked-questions) |
| 9 December 2026 | Revised PLD, for products placed on the market after this date: software is a product, and the defence for later-arising defects is lost where the defect is due to "a lack of software updates or upgrades necessary to maintain safety" within the manufacturer's control (Art. 11(2)(c)); free software outside a commercial activity is excluded (Art. 2(2)) | If supplied in a commercial activity | [PLD 2024/2853](https://eur-lex.europa.eu/eli/dir/2024/2853/oj/eng) |
| 1 January 2027 | California Digital Age Assurance Act (AB 1043): developers request an OS age signal "when the application is downloaded and launched" | Possibly; an amendment (AB 1856) may exempt openly licensed software, signing unverified | [DATA 1][DATA] |
| 11 December 2027 | The CRA in full: Annex I, conformity, CE marking, SBOM, support period. Products placed before this date are subject to the Regulation's requirements only if substantially modified from it (Art. 69(2)) | Yes, for every release placed on the market | [CRA Arts. 69, 71][CRA-EL] |

**The operations landscape.** CNA Operational Rules **4.2.0** have applied since 25 August 2026: a CNA must publish a CVE Record within 72 hours of disclosure (24 hours is the recommendation), must keep a public distribution point for its advisories (login should not be required and URLs should stay available), and must give administrative contacts (email and phone, not public) who respond in a timely manner, where "automated responses do not qualify"; at least two contacts are recommended; CVSS is not required ([CNA Rules](https://www.cve.org/ResourcesSupport/AllResources/CNARules)). **ENISA has been a CVE Root since November 2025** ([ENISA](https://enisa.europa.eu/news/stepping-up-our-role-in-vulnerability-management-enisa-becomes-cve-root)) and lists open-source projects and foundations among potential CNAs ([ENISA](https://www.enisa.europa.eu/topics/vulnerability-services/join-the-global-cve-program-under-the-enisa-root)); its **EUVD** has run since 13 May 2025 and lists CVE IDs as aliases ([ENISA](https://www.enisa.europa.eu/news/consult-the-european-vulnerability-database-to-enhance-your-digital-security)). **CSAF 2.0** has been an OASIS Standard since 18 November 2022 and is ISO/IEC 20153:2025; CSAF 2.1 is still a draft ([OASIS](https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html)). The **OSV** schema is at 1.9.2 ([OSV](https://ossf.github.io/osv-schema/)). **security.txt** is RFC 9116 ([RFC 9116](https://www.rfc-editor.org/rfc/rfc9116.html)). The most concrete CRA-aligned SBOM rule is Germany's **BSI TR-03183-2 v2.1.0** (20 August 2025): CycloneDX ≥ 1.6 or SPDX ≥ 3.0.1, recursive to the first component outside the delivery, a SHA-512 hash per deployable component, one SBOM per version and no vulnerability data in it ([BSI][BSI]). The browser standard **EN 304 617** reached final draft V1.0.0 on 5 August 2026 and is in ETSI's public enquiry (a third-party guide dates the notice to 13 August 2026; a national listing gives 26 August to 27 October 2026); no CRA harmonised standard was cited in the Official Journal as of 8 September 2026 (Medium; [factcheck F25][FC]). **EN 301 549 v4.1.1** was published in September 2026, with Official Journal citation expected around 30 November 2026 (Medium; [DATA 3.1][DATA]); the reporting template is **VPAT 2.5Rev** of 24 April 2025 ([ITI](https://www.itic.org/policy/accessibility/vpat)).

**What is invisible today.**
- **Support status.** No major browser tells a user, in the product, the month and year its security support ends for their OS. The CRA now requires both the date at the time of purchase and an end-of-support notice where technically feasible (Art. 13(19)).
- **Whether a fix was installed.** Vendors publish when a fix shipped, not how many users have it; Project Zero's 2021 reasoning is that adoption is what matters. Measuring it usually means telemetry, which FI-D12 forbids by default.
- **Which advisory is right.** A vendor's page, its CVE record, NVD's enrichment and a scanner's score routinely disagree, and the vendor cannot stop it ([HIST 3.9][HIST]).
- **Accessibility at the version a buyer uses.** Browser ACRs describe one old version on one platform.

**What Izcal can expose that no one does.**
- **One record, many projections, with disclosure rules per field.** Every advisory, CVE record, CRA notification and release note is generated from one record, so a correction regenerates all of them together, and each field's disclosure class decides where it may appear.
- **Clocks stamped by machines, not typed.** Receipt times come from the intake system and are sent back to the reporter as a signed receipt; the record imports them and no one can edit them.
- **Installed-fix uptake without telemetry.** Update payloads are public, cacheable fetches with no per-user identity (NW-D17). Aggregate counts, kept as counts and never as logs, estimate how fast a fix reaches users and which Member States it reached, which Art. 14(2)(a) asks for.
- **Advisories anchored to the specifications.** Izcal's algorithms carry spec anchors (S1 G9), so an advisory can name the specification step whose implementation was wrong and its regression test (SEC-24).
- **Compliance evidence generated per release.** The specs, threat models and CI results already exist as text and data, so the technical documentation, the Annex I traceability table and the ACR are generated from them per release and per platform.

**Gaps in Izcal's own specs that this spec closes**, beyond the 14 FI-1 rows: Annex I Part I's reset to original state, update opt-out and postpone, user-switchable security logging, and full data removal (Section 6.10); Art. 13(9)'s ten-year update archive; Art. 13(19)'s support date and notice; Art. 13(21)'s corrective action and 13(23)'s cessation notice (Section 6.11); Art. 14(7)'s choice of CSIRT; D-S4's pre-selected crash prompt against ePrivacy Art. 5(3) (Section 6.13); SEC-D4's fallback of a release without automatic updates; S1 13.2's export-control wording; and GR-D21's OS-drop rule against the support period (Section 6.2).

## 4. First principles

### 4.1 The real tasks

1. **A reporter reaches a person**, learns when they will hear back, and is told when the issue becomes public.
2. **The vendor decides quickly whether the bug is exploited**, because that one fact starts the CRA's legal clock and Izcal's fastest internal clock (SEC-1).
3. **Every user gets the fix without a feature change and without effort**, and can see that they have it.
4. **Every party learns the same facts**, each only the facts it may see: users, downstream packagers, scanners, the CSIRT, the CVE program.
5. **An authority or buyer can check a claim** (conformity, accessibility, support period) against evidence for the exact version in use, and can order a correction that reaches users.
6. **A person can find, export and delete what Izcal holds about them**, even when the service was designed to hold no identity.

### 4.2 The primitives

- **Law, read from its text:** CRA Arts. 3, 4, 13, 14, 16, 17, 24, 27–32, 64, 69, 71 and Annexes I, II, V, VII ([CRA-EL]); PLD Arts. 2, 4, 7, 11, 17; GDPR Arts. 3, 6, 7, 13, 15, 17, 20, 25, 27, 28, 30, 33, 35, 37; ePrivacy Art. 5(3) and EDPB Guidelines 2/2023; DMA Art. 6; EAR 15 CFR 734.3, 742.15 and 772.1 ([DATA][DATA]).
- **Open formats, implemented from their published specifications:** CSAF 2.0, OSV, the CVE Record Format, RFC 9116, CycloneDX (ECMA-424) and SPDX 3.0.1, TUF, VPAT 2.5Rev, EN 301 549, WCAG2ICT.
- **Izcal's own mechanisms:** the TUF updater and its channels (UPD-R1 to R5), the emergency release class and mitigation channel, SEC-1's clock, the shared witnessed log UPD-R1 uses, the published endpoint list and OHTTP relay (NW-D17), the shell's settings registry, About pages, prompt ledger and export format (SH 6.9, 6.12, 6.14, SH-D11), the accessibility test suites (SH-3 to SH-5, AX targets), spec anchors (G9) and the deterministic recordings (REC-R rules).
- **Izcal's principles:** M-1 and D-S10 (Izcal builds its own tools from these specifications), HGBA-2 2.3.2 (MIT for what ships), FI-D12 (no telemetry unless the user opts in), FI-D14 (open protocols first), AI off by default (HGBA-3 B10).

### 4.3 Assumptions kept and broken

| Assumption from existing practice | Kept or broken | Why |
|---|---|---|
| A published disclosure policy with deadlines | **Kept** | Every durable norm since 2000 has an acknowledgement deadline, a fix deadline and an adoption window ([HIST 1][HIST]); Apple's EU engine rules and the CRA both require a policy |
| Advisory pages written by hand per release | **Broken** | They disagree with CVE records and scanners; one generated record removes the disagreement |
| CVSS as the vendor's severity | **Broken** | A one-dimensional score does not express browser process boundaries; Izcal keeps its own scale and publishes a CVSS 4.0 vector as a secondary field |
| The patch-gap clock starts at the report | **Broken** | Public source starts it when the fix lands; Izcal's fixes stay private until the release that ships them (CO-4) |
| A fixed weekly security release day | **Broken** | Chrome's reason was public fixes waiting to ship; Izcal keeps fixes private, so security releases go out on demand within each bound (CO-D18) |
| A paid bounty from launch | **Broken** | AI-generated reports make an unstaffed bounty a cost; SEC-D10 already defers it |
| Rely on CVE and NVD for enrichment | **Broken** | Izcal is the source of truth for its products and maps to CVE, EUVD and GCVE |
| An ACR is written once a year | **Broken** | It is generated per release and platform from tests, and audited by people at a fixed cadence |
| Telemetry is needed to know who has a fix | **Broken** | Aggregate counts of public fetches estimate it without any identifier |
| Crash reports default to "send" | **Broken** | ePrivacy consent cannot be pre-ticked; Section 6.13 |
| A browser needs terms of use | **Broken** | The MIT licence governs the software; only services with accounts need terms |
| A separate legal track from engineering | **Broken** | Most CRA duties are build outputs and release gates; legal advice answers the questions the text does not settle |

## 5. Lessons learned

Lessons are ideas and evidence, never a template to reproduce (M-1).

| Project | What happened | The lesson | How Izcal applies it |
|---|---|---|---|
| Bugtraq, RFPolicy | Researchers published because vendors were silent | A vendor that does not acknowledge loses the timeline | Published acknowledgement and update times, measured (CO-13) |
| Project Zero 2020–2021 | A strict 90 days did not shorten fixes; adoption mattered | Measure installed fixes, not shipped ones | Aggregate uptake estimate per security release (CO-24) |
| Meltdown and Spectre | Siloed notice; parties could not talk | Coordination needs a written process and a shared channel | A written embargo procedure with a named coordinator (6.5) |
| Heartbleed | Pre-notice on the day of release missed distributions | Keep the notice list before it is needed | A third-party packager notice list maintained with each release (6.5) |
| xz-utils | A maintainer takeover; disclosure bypassed upstream | Release integrity is a security-response matter | Severe-incident runbook for signing and CI (6.7); SEC-D4's second person |
| Chrome patch gap | Public fixes became n-day exploits | Cadence is a control, and public source starts the clock | Fixes land privately until release (CO-4); security releases separate from features (CO-3) |
| Apple RSR → BSI | A mechanism that needed user action was rarely used | Out-of-band fixes must apply by default | Automatic security updates on by default (CO-5); UPD-R5 for mitigations |
| curl, Chrome VRP 2026 | AI-generated report floods | Money draws noise; intake must filter first | Reproducer required; no paid bounty until triage is staffed (CO-D10) |
| CVE and NVD 2025–2026 | Funding and enrichment failed | Do not depend on one registry | Become a CNA; publish CSAF and OSV; aliases to CVE, EUVD, GCVE (CO-D7, CO-D8) |
| Linux kernel CNA | Volume dispute | Publish the assignment rule | A written CVE assignment rule (6.6) |
| curl and CVSS | Third parties rescored curl's issues | Own the severity statement | Izcal's scale plus a published CVSS mapping (6.4) |
| Firefox Terms of Use, February 2025 | A broad content-licence clause was revised within two days after backlash ([Mozilla](https://blog.mozilla.org/en/products/firefox/update-on-terms-of-use/)) | Terms for a browser must not claim rights over what users do | No terms for the browser binary; service terms only where accounts exist (CO-D14) |
| Debian Iceweasel, 2006–2016 | Trademark terms forced a rename for a decade (Low; [DATA 6.5][DATA]) | A trademark policy can split a project from its packagers | Packagers may ship patched builds under the name with disclosure (CO-D15) |
| VPATs | Self-attested, stale, often inaccurate (19.6% of items in one study, abstract only) | A report must be tied to tests of the version it describes | ACR generated per release from tests, audited annually (CO-16) |
| *Planet49* (CJEU C-673/17) | A pre-ticked box is not consent | Defaults decide legality | Crash prompt with nothing pre-selected (CO-D6) |

## 6. Design

### 6.0 Architecture at a glance

Nothing in this spec adds a process kind to the browser. It adds **build-time tools**, **release gates**, **server-side services**, a few **shell surfaces** and new event classes in the **prompt ledger**.

| Part | What it is | Where it runs | Crate or owner |
|---|---|---|---|
| **Vulnerability record and security-response log** | One record per vulnerability and per incident, with field disclosure classes; machine-stamped receipt clocks; changes signed and hash-chained; commitments published on a fixed schedule | EU-hosted security infrastructure, not GitHub (6.12's hosting table); access limited to the security officers | `hg-advisory` (record schema, signing, generators) |
| **Projections** | CSAF 2.0 security advisory and VEX, OSV, CVE record, CRA notification text, release-note security section, enterprise security notes, in-product notice metadata, quarterly report | Generated by `hg-advisory` at release | `hg-advisory` |
| **SBOM** | A CycloneDX document per release artifact; SPDX on request | Release pipeline | `hg-sbom` |
| **Conformity set** | Technical documentation, Annex I traceability table, declaration of conformity, user security information, support statement | Release pipeline | `hg-conformity` |
| **Accessibility conformance report** | VPAT 2.5Rev INT per release line and platform, from test results and audit records | Release pipeline | `hg-acr` |
| **Release gates** | No fixable known-exploitable vulnerability in a feature release; SBOM complete; conformity set generated; testing-build notice present; the human sign-off | S1 7.2's **Release gate** tier | `hg-conformity` |
| **Shell surfaces** | Support date and end-of-support notice, testing-build notice, automatic-update opt-out and postpone, security and withdrawal notices, deletion actions, `about:crashes`, simplified declaration | Shell renderer, from broker state | Shell (amendments, 9.5) |
| **Security event classes in the prompt ledger** | Annex I Part I(2)(l): security-relevant events beside consent decisions, user-switchable, never uploaded | The ledger (SH-D11), under its keys, retention, clearing and private-window rules | Shell (amendment, 9.5) |
| **Services Izcal runs** | Intake, update and data channels, crash ingestion, opt-in telemetry, feedback, breakage reports, phishing reports and appeals; Stage 2 push, sync and the extension directory | Izcal's servers, hosting per 6.12 | FI-2, FI-11, FI-35 and the component specs build them; this spec sets their data-protection rules (6.13) |

**Crate rules G260–G262**, in the range G260–G269 assigned to this spec in S1 2.2's registry (G263–G269 reserved), each with its mechanical check:

| Rule | Check |
|---|---|
| **G260.** `hg-advisory`, `hg-sbom`, `hg-conformity` and `hg-acr` are tools: no shipped `hg-*-main` binary depends on them | `hg-graph-lint` on `cargo metadata` |
| **G261.** `hg-sbom` reads only build inputs: the lockfile, vendored sources, the data ledger (ML-20) and the image inventory (SEC-16 (e)) | A sandboxed build step with only those paths mounted |
| **G262.** `hg-advisory` never writes a field of class `private` or `restricted` into a public projection | CO-26's canary suite in CI |

### 6.1 Role, scope and the conformity route

**Role.** Izcal plans as a **manufacturer** (FI-D11). A manufacturer is whoever markets a product "under its name or trademark, whether for payment, monetisation or free of charge" (Art. 3(13)). Recital 15's signs of commercial activity include support priced above cost, monetising "other services" through the software, donations "exceeding the costs", and personal data required "as a condition for use" for reasons other than security, compatibility or interoperability ([CRA 1.2, 1.5][CRA]). Opt-in-only telemetry (FI-D12) appears to keep the last sign closed; that and every other reading of recital 15 is for counsel (Q-CO-2). The open-source steward route (Art. 24) still requires Art. 14 reporting and forbids CE marking (recital 19), so it saves little and costs enterprise buyers their CE-marked product; this spec does not plan for it.

**What each funding option does, as the text reads.** No legal conclusion is drawn (Q-CO-2):

| D-S8 option | What the CRA text says | Effect on this spec |
|---|---|---|
| Donations, sponsorship, grants | Not commercial by themselves, unless donations exceed costs (recitals 15, 18) | Izcal may be outside the CRA; the plan does not change |
| A not-for-profit with all earnings after costs used for its objectives | Not commercial (recital 18) | As above |
| Search-engine revenue share | Close to "monetises other services" (recital 15) | Manufacturer |
| Paid support above cost | Commercial (recital 15) | Manufacturer |
| Paid enterprise or agent-runtime edition | Commercial; if that edition is not under a free licence, it is not "free and open-source software" (Art. 3(48)) | Manufacturer; **that edition loses the Art. 32(5) route** |

**The conformity route (CO-D2, accepted by the owner, 25 October 2026).** Class I products may use internal control (module A) only with a harmonised standard, common specification or certification scheme applied in full; otherwise a notified body is involved (Art. 32(2)). But **Art. 32(5)**, confirmed verbatim against the Official Journal text, says manufacturers of products "qualifying as free and open-source software, which fall under the categories set out in Annex III, shall be able to demonstrate conformity … by using one of the procedures referred to in paragraph 1 …, provided that the technical documentation referred to in Article 31 is made available to the public at the time of the placing on the market" ([factcheck][FC]). Izcal is MIT and already publishes its specs, threat models and CI configuration, so publishing the technical documentation costs little. **The route therefore depends on D-S8**: it holds only while every Izcal edition placed on the market is free and open-source software. It may also depend on D-S7: if the licensing track ever leads Izcal to ship a closed DRM module inside its own build, whether the product still "qualif[ies] as free and open-source software" is a question for counsel (Q-CO-25), and the route, the export classification (6.17) and the SBOM's treatment of that component are re-decided. EN 304 617 still matters: once its reference is in the Official Journal, applying it in full gives a presumption of conformity (Art. 27(1)) and simplifies the evidence, so `hg-conformity` maps Izcal's rules to its clauses as soon as the text is public (Q-CO-5). Q-FI-2 is answered for planning: as of 24 October 2026 the standard is a final draft in public enquiry, not cited, and **on the text of Art. 32(5) Izcal can self-assess without it**, subject to counsel's reading (Q-CO-9).

**What is placed on the market.** The first EU-available signed Release binary is a placing on the market. Recital 39 suggests that a feature release that adds functions may be a substantial modification, which would make every feature release a new placing (Q-CO-6). Izcal does not depend on the answer: the conformity pipeline runs on **every release** (CO-10), and each release's documentation and support date are its own. Art. 69(2) subjects products placed on the market before 11 December 2027 to the Regulation's requirements only if they are substantially modified from that date, and Art. 69(3) keeps Art. 14 reporting for them ([factcheck F1][FC]). Izcal plans no Release before then, so the full requirements apply to its first one.

**Testing builds: Nightly and Beta (CO-D3, accepted by the owner, 25 October 2026).** Under the distribution spec's DU-D10, **signed public Nightlies start at least 3 months before the first Stable**, so Izcal's first public binary, and its first EU distribution, is a Nightly. Everything this spec requires "before the first public binary" is therefore due before the first public Nightly. Art. 4(3) says Member States "shall not prevent the making available on the market of unfinished software which does not comply with this Regulation, provided that the software is made available only for a limited period required for testing purposes with a visible sign clearly indicating that it does not comply with this Regulation and that it will not be available on the market for purposes other than testing". Recital 37 names "alpha versions, beta versions or release candidates", asks for a risk assessment, compliance "to the extent possible", vulnerability handling "to the extent possible", and that manufacturers "should not force users to upgrade to versions only released for testing purposes" ([CRA 1.11, 1.12][CRA]). "Nightly" appears nowhere, and whether a channel that exists indefinitely is "a limited period" is open (Q-CO-3). Until 11 December 2027, Art. 69(2) means a testing build distributed before that date owes only Art. 14 reporting; the notice and expiry below are therefore **preparation** that makes the same channel lawful afterwards, not a present duty. The recommended treatment:
- **The notice.** Every testing build shows a notice on its download page, as a dismissible first-run screen whose dismissal is recorded (added for testing builds only, in front of SH-D23's four screens), and permanently in About: "Testing build. This build does not comply with Regulation (EU) 2024/2847 and is made available only for testing." The text is one generated string, so the three places cannot drift.
- **Opt-in only.** A testing build is installed by an explicit download, or by a channel change in Settings that shows the same notice and needs a confirmation. The updater never moves a Release installation to Beta or Nightly by itself, and a testing channel moves to Release only when the user chooses.
- **A limited period per build.** Each Nightly build is updated or, failing that, stops opening new tabs after **30 days†**, with a notice that explains why; each Beta build lives for its cycle plus 30 days†. The limit is per build because each build, not the channel, is what is "made available". Rationale for 30 days†: a nightly that has not updated in a month is no longer being tested, and a stale testing build is the riskiest software Izcal distributes.
- **Security to the extent possible.** Testing builds go through the same pipeline, fuzzing and advisory process. **An actively exploited vulnerability found in a testing build is reported under Art. 14 as if it were in Release**, until counsel says otherwise (Q-CO-4). Over-reporting costs a form; under-reporting risks the top fine tier.
- **Reversal.** If counsel says a standing channel does not qualify, Beta becomes a conforming product (it is weeks from Release, so its documentation is nearly the same) and Nightly becomes a time-boxed programme with sign-up, or a source-only build that users compile themselves.

**The legal entity (CO-D1).** The declaration needs a signatory (Art. 28), the product needs a postal address (Art. 13(16)), the CSIRT follows the main establishment (Art. 14(7)), and the GDPR needs a controller and possibly a representative (GDPR Art. 27). With no EU establishment and no authorised representative, importer or distributor, reports go to the CSIRT of the Member State with the most users, which Izcal can only estimate (Art. 14(7)(d)). **The owner decided on 25 October 2026 (CO-D1): an entity established in an EU Member State before the first EU distribution**, with its legal form set by D-S8 and counsel. It makes the CSIRT, the language of the user information, the supervisory authority and the representative question all predictable at once.

### 6.2 Support period and end of support

**The period (CO-D4).** Art. 13(8) sets "at least five years" unless expected use is shorter, determined from "reasonable user expectations, the nature of the product", other manufacturers' periods and "the availability of the operating environment", with the reasons recorded in the technical documentation. Recital 60 lists operating systems among products expected to need longer. A browser is closer to an OS than to an app in how long people keep using it, but it is also replaced continuously by its own updates. Each release states its support end as a month and year, at download and in About (Art. 13(19)).

**Delivery as the latest release.** Art. 13(10) applies "where a manufacturer has placed subsequent substantially modified versions of a software product on the market": it lets the manufacturer meet Annex I Part II(2) "only for the version that it has last placed on the market", provided users of earlier versions can get it "free of charge" and "do not incur additional costs to adjust the hardware and software environment" ([factcheck F9][FC]). If feature releases are substantial modifications, Izcal's rolling model fits 13(10) while every user can run the latest release on the OS and hardware they have. If they are not, Izcal is one product continuously updated, and the support period simply runs for it. Both readings lead to the same engineering; which applies is Q-CO-6.

**Platforms leave their vendors' support (CO-D5).** GR-D21 supports an OS version while its vendor ships consumer security updates ([GR-D21][GR]). macOS versions leave Apple's patch window roughly once a year, Windows 10's consumer updates end on 12 October 2027, and each supported Linux distribution release has its own end. A user still on such an OS would need a new OS or new hardware to keep receiving fixes. Three options, costed:

| Option | What it means | Branches in steady state | Cost† |
|---|---|---|---|
| (a) Legacy branches | Keep a security-only branch for the last release that ran on a dropped OS until five years after the last release placed on that OS | About 5 macOS versions plus Windows 10 plus each dropped distribution release, each with Izcal's own engine fixes backported up to five years | 2–3.5 engineers† of backporting, building, signing and publishing, growing with platforms; not in any budget |
| (b) OS end bounds the period | Read Art. 13(8)'s "availability of the operating environment" as ending the period when the OS vendor's support ends, even after a five-year statement | None | None, but it contradicts a stated date, which Art. 13(19) and consumers rely on |
| (c) **Per-platform statement at release** | The support end stated for a release on each platform is the earlier of five years and that OS version's vendor end of security support. Where the vendor publishes no end date (macOS), Izcal states a conservative estimate of its own, and if the OS loses support earlier, Izcal keeps a security-only branch only until the stated date | At most one short branch per platform, usually none | 0.2–0.5 engineer† for the occasional short branch |

**Decided by the owner, 25 October 2026: (c)** (CO-D5, reviewed with the distribution spec's DU-D15), recorded with its reason in the technical documentation, because it is a statement made before supply, so no promise is broken later, and it uses the factor Art. 13(8) names. Whether a support period shorter than five years on an old OS is lawful is for counsel (Q-CO-7); the reversal is (a), with its cost added to S1 8.3. GR-D21's drop decision records the support statements it ends. The download page for an OS whose vendor support ends within five years shows both dates.

**Updates stay available (Art. 13(9)).** Each security update stays downloadable for **at least ten years** or the rest of the support period, whichever is longer. Izcal's TUF metadata expires within days by design (SEC-17), and code-signing certificates expire, so archived builds are verified another way: release signatures carry **trusted timestamps**, and an **archive manifest** listing every archived artifact's hash is signed by a dedicated archival key and committed to the witnessed log, so an archived build verifies years after its certificate expires. The archive page carries Art. 13(11)'s warning about unsupported software. CO-12 tests the oldest archived build each quarter.

**End of support in the product (Art. 13(19), PO-3.15).** About shows the support end date. After it, or when the platform's stated date passes, the shell shows a non-modal, dismissible notice at most once a week†, never a modal, with what the user can do. The data source is a field in the TUF targets metadata the updater already fetches, so no new request is added (NW-36). The download page shows the month and year before download.

### 6.3 Security updates: automatic, separate from features, free

**Automatic by default (Annex I Part I(2)(c)).** The CRA requires security updates "including, where applicable, through automatic security updates that are installed within an appropriate timeframe enabled as a default setting, with a clear and easy-to-use opt-out mechanism, through the notification of available updates to users, and the option to temporarily postpone them" ([factcheck F12][FC]). Izcal's per-user TUF updater already installs by default (SEC-D14, UPD-R2), and the shell's update row already notifies and lets the user choose "Later" for 7 days† ([SH 6.11][SH]). Additions:
- **Opt-out.** Settings > Updates and recovery > "Install security updates automatically", on by default. Turning it off shows what the user gives up and how to update by hand, which is also Annex II(8)(e)'s instruction. A separate row, "Apply emergency security switches", stays on unless the user turns it off too, because a UPD-R5 switch changes behaviour without installing software; both states are shown on `izcal://sandbox`.
- **Postpone.** A security update can be postponed for up to 7 days†, the period the shell already uses for relaunch; an emergency release (UPD-R4) can be postponed until the next start, not longer. Rationale: the exploited tier's own bound is 72 hours†, so a longer postponement would undo it.
- **Enterprise policy** can turn automatic updates off (recital 56, FI-3).
- **Linux packages (CO-D19).** Under FI-D10, packages from Izcal's repositories are updated by the system package manager, which does not by default update third-party repositories automatically. Recommended: Izcal's package installs a **drop-in configuration for the distribution's own unattended-update mechanism, scoped to Izcal's repository**, enabled by default with a documented opt-out; and the browser, which already learns of new versions from TUF metadata, shows the update notification and the package manager's command. The tarball keeps the TUF updater. Whether this meets Part I(2)(c)'s "where applicable" for a package-managed install is Q-CO-8; FI-2 owns the packaging.

**Separate from features (Annex I Part II(2)).** "Where technically feasible, new security updates shall be provided separately from functionality updates." Three paths:
The release lines are the distribution spec's: a **4-week Stable** (moving to 2 weeks later), with **security-only point releases** between them; every per-release target here runs on that cadence.
1. **Security-only release.** A point release on the current stable line, built from the line's branch with only security fixes and their tests, published **on demand** within each issue's bound (6.4), batching fixes whose bounds allow it. No fixed weekday is promised: Chrome went weekly to close a patch gap created by public fixes, and Izcal keeps fixes private until release (CO-4). Staged rollout completes within **24 h†**, so a normal security release is everywhere well inside the 7-day critical bound.
2. **Emergency release** (UPD-R4) for the exploited tier. UPD-R4 lets it publish on a verified provenance attestation while the reproducibility comparison runs; **this spec adds**: it goes to every user at once, with no staged rollout.
3. **Emergency mitigation** (UPD-R5) while a fix is built.

Every critical, high or exploited fix for Release users takes one of these paths (CO-3). Moderate and low fixes may ride the next feature release, as the severity table says. Where a backport of a critical or high fix is technically infeasible (it depends on a refactor in the next feature release), the release records why in the technical documentation, since Part II(2) requires separation only "where technically feasible".

**Fixes stay private until release (CO-4).** Izcal's repository is public, so a security fix pushed to `main` publishes the bug before users have the fix. Security fixes are developed in a **private security repository** on EU-hosted infrastructure (6.12), mirroring the release branches, under SEC-27's controls (hardware keys, server-side rules, required checks from the protected base branch, no self-hosted runner on untrusted input), and merged publicly **at the moment the release that ships them is published**. A fix already public upstream (a dependency's advisory) is the exception, and then its public date is a candidate time zero.

**Free and with advisories (Annex I Part II(8)).** Security updates are free in every edition, and every security release carries an advisory link in its notes and in the update row.

### 6.4 The vulnerability record

**One record per vulnerability or incident.** The record is the single source of truth: a typed document in the private security repository. Every change is signed by a security officer's key and hash-chained to the previous one.

**Clocks that cannot be typed after the fact.** SEC-1 requires time zero "never entered after the fact". Three rules make that true:
- **Machine stamps.** The intake form and the role mailbox stamp every incoming report (time, digest of the content), sign the stamp with the intake service's key and send it to the reporter as a receipt. The record imports the stamp; no officer can edit it. The same applies to CSIRT notices and upstream advisories received through intake.
- **Typed clocks are flagged.** Only events with no machine source (detection by the officers themselves, a phone call) are typed. They are entered when they are known, flagged as typed in the record, and counted in the quarterly report.
- **Commitments on a fixed schedule.** Every day† at a fixed time, the security infrastructure publishes to the shared witnessed log a commitment to the chain head, salted so that a day with no change is indistinguishable from a day with changes. An observer learns nothing about when security work is active, and any edit before a published commitment is detectable afterwards.

**Fields, each with a disclosure class.** `public` fields appear in public projections at release; `restricted` fields appear only after an embargo lifts or to named coordination parties; `private` fields never leave the record except to a CSIRT or authority under the runbook.

| Group | Fields | Class |
|---|---|---|
| Identity | `IZCAL-<year>-<n>` (Izcal's own stable ID); aliases: CVE, EUVD, GCVE, GHSA | public |
| Clocks | report received (stamped); public disclosure; detection in the wild; **Art. 14 awareness** (below); exploitation assessment; fix landed privately; fix released per line; advisory published; CVE record published; Art. 14 early warning, notification, intermediate and final report sent | release and advisory times public; the rest private |
| Classification | Izcal severity; the **exploited** flag; CVSS 4.0 CVSS-BT vector (secondary); CWE; affected component (`hg-*` crate, shipped dependency or OS-owned component); process kind and sandbox tier | public at release |
| Scope | affected versions per line, channel, platform and ISA; fixed versions; the SBOM component reference; testing builds only | public at release |
| Evidence | reproducer (encrypted); exploitation evidence and its source; regression test ID (SEC-24); variant-analysis result (SEC 6.17); spec anchor of the faulty step | private, except the test ID and anchor, public at release |
| Coordination | reporter identity, contact and credit consent; embargo state and parties; upstream reports sent (Art. 13(6)); CSIRT and SRP reference; delay request (Art. 16(2)) | private; credit public with consent |

A correction to any field regenerates **every** projection in one step and publishes a revision of each (CSAF's revision history carries it), so the projections cannot drift after generation. CO-26 seeds canary values into `private` and `restricted` fields and fails the build if any appears in a public output.

**The exploited test.** One written test drives SEC-1's 72-hour tier, the CRA Art. 14 clock and the advisory's exploited flag. It is the CRA's own definition, "reliable evidence that a malicious actor has exploited it in a system without permission of the system owner" (Art. 3(42)), close to CISA's KEV criterion, which excludes proofs of concept and research ([CISA KEV](https://www.cisa.gov/known-exploited-vulnerabilities)). A public proof of concept alone is not exploitation.

**Art. 14 awareness, defined (CO-D21).** Art. 14 runs from the manufacturer "becoming aware". The record's **awareness** field is set, by default, to the **stamped receipt time of the first report, CSIRT notice or own detection that claims exploitation in the wild with any evidence**, not to the time the officers finish assessing it. The assessment is done inside the same 24 hours. If it is negative, the early warning is not sent and the record says why; if it is positive or undecided at hour 20†, the early warning is sent, marked as unconfirmed where it is. This is the conservative reading; when a manufacturer "becomes aware" is for counsel (Q-CO-26), and the reversal is the positive assessment as awareness. Rationale for hour 20†: four hours of slack for the manual SRP submission (CO-2).

**OS-owned components.** The security scope's central finding is that risk moves to code Izcal runs but does not own: GPU drivers, OS media decoders, and the OS engine and vendor CDM in protected-media mode ([SEC summary][SEC]). The record has an **OS-owned component** class: scope is the OS component and version; the fix owner is the vendor; Izcal's action is a mitigation (a UPD-R5 switch, or refusing protected-media mode below a runtime version, SH-R4 (d)) and an advisory that names the vendor's fix. Whether Art. 14 applies to a vulnerability in an OS component a product relies on is for counsel (Q-CO-27); the default is to report when Izcal's product exposes the flaw and exploitation through Izcal is evidenced. This spec proposes adding **"protected-media mode off"** to UPD-R5's closed switch list (amendment, 9.5).

**Severity and bounds (PO-16.6).** Izcal's own four levels, defined by its process model, with exploitation as a separate flag because the law and SEC-1 treat it separately. **One bound table** replaces the split between SEC-1 and SEC-25, which gave a sandbox escape a slower bound than a renderer bug:

| Level | Definition | Release vehicle | Bound from time zero |
|---|---|---|---|
| **Exploited** (any level) | The exploited test is met | Emergency release; the narrowest UPD-R5 switch meanwhile; Art. 14 reporting | ≤ 72 h† (SEC-1's exploited tier) |
| **Critical** | Code execution or data access with broker, network-service, GPU-process or updater authority from web content with no interaction beyond browsing (a sandbox escape, an IPC or consent logic bypass, a capability forged across processes); compromise of an update or data channel | Security-only release | ≤ 7 days† everywhere |
| **High** | Code execution inside a sandboxed renderer from normal browsing; a cross-site or cross-container data leak (an I-1 bypass, UXSS); a spoofable trusted surface (SH-11); a TLS or certificate-validation bypass | Security-only release | ≤ 7 days† in the engine (SEC-1); ≤ 30 days† elsewhere (SEC-25) |
| **Moderate** | As high or critical, but needing an uncommon configuration, an unusual user action or an open developer tool | Next feature release, or a security-only release | Next release |
| **Low** | Minor information leaks, defence-in-depth gaps | Next feature release | — |

The critical row tightens SEC-25's 14 days† for Izcal's own code to 7 days† and relaxes nothing (amendment, 9.5). Rationale: a critical bug gives an attacker more than a high one, so it must never wait longer. The published scale states its mapping to CVSS 4.0 qualitative bands, so scanners can map it, and states that Izcal's level is authoritative.

**Projections, generated by `hg-advisory`.**

| Projection | Format and rule | Audience |
|---|---|---|
| **Security advisory** | CSAF 2.0 security-advisory profile, published in the **trusted provider** role (CSAF 2.0 §7.2.3): requirements 1–7; requirements 8 (the `CSAF` field in security.txt) and 9 (`/.well-known/csaf/provider-metadata.json`); the directory distribution, 11–14; and 18–20 (a hash file and an OpenPGP signature per document, and the public key) ([OASIS §7][OASIS]); one advisory per release listing every fixed vulnerability | Scanners, enterprises, CSIRTs; Art. 14(8) |
| **VEX** | CSAF 2.0 VEX profile per release: fixed, known affected, known not affected (with the impact statement), under investigation | SBOM consumers |
| **OSV** | OSV schema JSON per vulnerability, aliases filled | The open-source tooling ecosystem |
| **CVE record** | CVE Record Format, submitted as CNA (6.6) | The CVE program |
| **CRA notification text** | English text in the field order of Art. 14(2) and (4), for manual entry in ENISA's platform, which has no API | Coordinating CSIRT, ENISA |
| **Release notes** | A security section linking each advisory | Users |
| **Enterprise security notes** (PO-16.10) | Per release: fixed IDs, severities, affected policies, mitigations | Administrators |
| **In-product notice metadata** | A field in the TUF targets metadata naming builds with an exploited vulnerability, or withdrawn (6.11); the shell shows the notice while an update is pending | Users; Art. 14(8) |
| **Quarterly report** | Bound compliance per level, CO-1 times, typed-clock count, uptake (CO-24) | Public |

Annex I Part II(4) allows a delay of public disclosure in duly justified cases where publication's risks outweigh its benefits, until users can apply the patch; Izcal's policy uses it only where an embargo with other vendors requires it.

### 6.5 Intake, disclosure policy and coordination

**Channels.** A **security.txt** under RFC 9116 on Izcal's website, with `Contact` and `Expires` (both required; less than a year ahead is RFC 9116's recommendation, which CO-14 makes a hard target), `Policy`, `Canonical`, `Encryption`, `Preferred-Languages`, and the `CSAF` field that CSAF 2.0 requirement 8 adds ([RFC 9116](https://www.rfc-editor.org/rfc/rfc9116.html); [OASIS §7.1.8][OASIS]). The channel is a **self-hosted HTTPS form and an OpenPGP-capable role mailbox**, hosted in the EU, which stamp every report (6.4). GitHub's private vulnerability reporting is **not** an intake channel: it would put reporter data on a US service against CO-D17, and curl found it lacking for triage ([HIST 2.12][HIST]); reports filed there are answered with a pointer to the form. Reports may also arrive anonymously through a CSIRT (CRA recital 76). The same role contact is the CRA's **single point of contact** (Art. 13(17)), which must not "limit such means to automated tools": every report gets a reply from a person.

**The published policy.** It cites ISO/IEC 29147 and 30111 and commits to:
- **acknowledgement ≤ 3 days†** (SEC-25), aiming at 24 hours;
- a **triage verdict ≤ 7 days†** (valid or not, severity, expected vehicle); rationale: the critical bound is 7 days†, so a reporter knows the verdict no later than a critical fix would ship;
- **status updates at least every 7 days†** until release, for the same reason;
- a default **disclosure deadline of 90 days** for issues reported to Izcal, which researchers expect from Project Zero and Pwn2Own, with the fix bounds of 6.4 far shorter;
- credit with the reporter's consent;
- **safe harbour**: Izcal will not pursue good-faith research that follows the policy, defined in Izcal's own words around access solely to find and report a flaw, avoidance of harm and no use beyond reporting;
- what a report must contain: a **reproducer on a named build** (a page, a script or a test case), the observed and expected behaviour, and the build ID. Reports without a reproducer, or machine-generated reports nobody has verified, receive one templated reply and are closed; repeated ones are blocked. This is the filter curl lacked and Chrome now applies ([OPS 5.1, 5.6][OPS]).

**Bounty.** No paid bounty at launch; a public hall of fame. This spec adds two conditions to SEC-D10's trigger ("bounty when D-S8 allows"): triage is staffed for AI-scale volume first, and a paid programme uses Izcal's own intake or an EU-hosted platform (CO-D10; amendment, 9.5).

**Triage tooling.** Reports and reproducers are untrusted input that can carry prompt injection. Any automated or AI-assisted triage tool follows SEC-27's rule: read-only, no network, no secrets, no shell, no write access to any repository. Reports are never sent to a third-party AI service.

**Embargoes and coordination.**
- **Downstream notice.** Izcal builds its own Linux packages (FI-D10), so pre-notice matters only for **third-party packagers** (distributions that package Izcal themselves). They get notice through the **linux-distros** list: at most 14 days of embargo, preferably under 7 ([Openwall](https://oss-security.openwall.org/wiki/mailing-lists/distros)). For a security-only release Izcal sends notice **72 hours** before release, as the Rust project does ([Rust](https://www.rust-lang.org/policies/security)); on the emergency path, notice goes out when the fix is final, with an embargo of at most 24 hours†.
- **Shared specifications and protocols.** A flaw in a web specification goes to the W3C's standards disclosure process ([W3C](https://www.w3.org/TR/security-disclosure/)) or the WHATWG editors; a protocol flaw (TLS, QUIC, HTTP) goes through CERT/CC or the national CSIRT for multi-vendor coordination. Each vulnerable implementation gets its own CVE when the specification can be used securely, and one CVE covers the flaw when it cannot (CNA Rules 4.2.14) ([CNA Rules][CNA]).
- **Dependencies (Art. 13(6)).** A vulnerability Izcal finds in a dependency (the RustCrypto pins, the generic crates of S1 7.7) is reported to its maintainer within **7 days†**, with Izcal's fix where one exists; where the upstream supplier is a CNA, Izcal leaves the CVE assignment to it (CNA Rules 4.2.16.6). Dependency advisories arrive through the RustSec advisory database that SEC-14's cargo-deny check reads, and their OSV-format records feed `hg-advisory`.
- **Reproductions.** A reporter using Izcal may attach a deterministic recording. Recordings carry page content, so they follow the REC-R rules and 6.18's retention table, and are never attached to a public advisory. A public regression test (SEC-24) is a **minimised reduction written by Izcal**, reviewed to contain nothing from the reporter's content, never the reporter's reproducer itself.

### 6.6 CVE numbering

**Izcal becomes a supplier CNA (CO-D7)** under the **ENISA Root**, before the first public binary, which is the first public Nightly (6.1). ENISA's scope includes open-source projects, and an EU Root fits a vendor whose legal reporting goes to ENISA anyway. Until then, CVE IDs are requested from GitHub's CNA, which reviews requests in about 72 hours ([GitHub](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/about-repository-security-advisories)). The rules fit the design: records within 72 hours of disclosure (Izcal targets 24 hours†, CO-6), a public distribution point (the CSAF provider), and administrative contacts who respond as people. Two contacts are the CNA rules' recommendation and **Izcal's own rule**; the second is held by a different person from the first (9.2).

**The assignment rule**, published as Izcal's own choice (the CNA rules require a published scope, not an assignment policy): every vulnerability fixed in a build that reached the Release or Beta channel gets a CVE ID; issues only ever present in Nightly are listed in the advisory as "testing builds only" without one; defence-in-depth hardening gets no ID; one ID per root cause. The rule exists so that a volume dispute like the kernel's cannot arise.

**Aliases.** Izcal agrees by default to ENISA adding fixed vulnerabilities to the EUVD (Art. 17(5)). GCVE registration, which a CNA can add cheaply, is deferred until there is demand (Q-CO-11).

### 6.7 The CRA Article 14 runbook

**Who.** The **security officers** (6.19's staffing), each with an **EU Login account with MFA**, created before the first EU distribution. ENISA advises registering on the platform only when a notification is due, so the registration steps are rehearsed up to that point ([ENISA SRP FAQ][SRP]).

**Which CSIRT.** The CSIRT of the Member State of the main establishment (Art. 14(7)). Under CO-D1 that is known in advance. Without an EU establishment, rule (d) applies: the Member State with the most users, estimated from aggregate counts (6.13).

**The steps for an actively exploited vulnerability.** Time zero for the CRA steps is the record's **awareness** field (6.4).

| Step | When | What | Source in the record |
|---|---|---|---|
| Assess | Within the first 20 h† | Exploited test applied; result signed into the record | Clocks, evidence |
| **Early warning** | ≤ 24 h after awareness | Product, and the Member States where Izcal knows it is available (Art. 14(2)(a)); marked unconfirmed if the assessment is undecided | Scope; aggregate counts |
| **Notification** | ≤ 72 h after awareness | General information on the product, the general nature of the exploit and vulnerability, corrective measures taken and those users can take, sensitivity (Art. 14(2)(b)) | Classification, mitigation (UPD-R5 switch) |
| Delay request | With the notification, if an embargo runs | Ask the CSIRT to delay dissemination (Art. 16(2); Delegated Regulation (EU) 2026/881) | Coordination |
| Intermediate report | On the CSIRT's request | Status (Art. 14(6)) | — |
| Users informed | As soon as a mitigation or fix exists | In-product notice and CSAF advisory (Art. 14(8)) | Projections |
| **Final report** | ≤ 14 days after the fix or mitigation is available | Description with severity and impact, the malicious actor where known, the update or measure (Art. 14(2)(c)) | Whole record |

**Severe incidents** (Art. 14(3)–(5)) include anything "capable of leading to the introduction or execution of malicious code" in the product or users' systems; the recital's example is malicious code introduced into the release channel ([CRA 6.4][CRA]). For Izcal that means compromise or suspected compromise of a signing key (release, archival, intake or record keys), the TUF repository, the release CI, the update CDN's integrity, a data channel (phishing bundle, content-blocking lists, models) or the security repository. They follow the same runbook, with a final report within one month of the notification. The **GDPR breach clock** (72 hours to the supervisory authority, Art. 33) runs in the same incident process when personal data is involved: one incident, one record, several clocks.

**Testing builds.** Reported as for Release until counsel says otherwise (6.1).

**Penalties.** Missing Art. 13 or 14 duties sits in the top tier of fines (up to EUR 15 million or 2.5% of worldwide turnover, whichever is higher, Art. 64(2)). Art. 64(10) exempts micro and small enterprises from fines for missing the 24-hour early warnings, but its derogation refers to paragraphs 3 to 9 while Art. 14 fines sit in paragraph 2; counsel confirms the exemption's reach (Q-CO-1's brief) ([factcheck F11][FC]).

**Drill (CO-2).** Twice a year†, combined with SEC-2's emergency-release drill: a planted exploited vulnerability is taken from report to the generated early-warning, notification and final-report texts, with the CSIRT and SRP steps rehearsed short of submission. The drill measures the time from record to early-warning text against **≤ 4 h†**. Rationale: the window is 24 hours, and the 4 hours are what remain after the 20-hour assessment allowance.

### 6.8 SBOM and VEX

**What (CO-D11).** For every release artifact (per platform, ISA, channel and tier ceiling), `hg-sbom` writes a **CycloneDX 1.6 or later JSON** SBOM meeting **BSI TR-03183-2 v2.1.0**'s required fields: SBOM creator and timestamp; per component the creator, name, version, filename, dependencies with completeness stated, distribution licence, **SHA-512** of the deployable file and the executable, archive and structured properties; and, where they exist, source URI, purl and the security.txt URL ([BSI §5][BSI]). An **SPDX 3.0.1** rendering is generated when a recipient asks for one; a second format is not maintained by default until an implementing act (Art. 13(24)) or a buyer requires it. Depth is **recursive to the first component outside the delivery**, which for Izcal is small: its own `hg-*` crates, the few pinned external crates, the engine images (SEC-16 (e)), the shipped data sets from the data ledger (UCD and CLDR data, the Public Suffix List, fonts, lists, model weights), and the OS interfaces, which are listed as external components and not expanded. The CRA's minimum is top-level dependencies (Annex I Part II(1)); Izcal exceeds it because full depth is cheap for a project that owns almost all its code. A closed component shipped under D-S7's licensing track would appear as an opaque component with its licence and hash (6.1).

**No vulnerability data in the SBOM.** Status goes in the CSAF VEX document per release (6.4), as the BSI rule requires.

**Published, signed and attested, by choice.** Recital 77 says manufacturers "should not be obliged to make the SBOM public", and Annex VII(8) puts it in the documentation "further to a reasoned request" while Annex VII(2)(b) lists it among vulnerability-handling information; whether Art. 32(5)'s public documentation must include it is Q-CO-9. Izcal publishes it with each release anyway, signed and covered by the release's provenance attestation (SEC-13 (b)), because an MIT project loses nothing by it. It also drives `about:licences` and `about:credits`, so the licence notices cannot drift from what shipped (CO-22).

### 6.9 Technical documentation, declaration, CE marking and user information

**Technical documentation (Art. 31, Annex VII).** `hg-conformity` generates one set per release line from sources that already exist:

| Annex VII item | Generated from |
|---|---|
| (1) General description, intended purpose, versions affecting compliance, user information | Release metadata; the Annex II page below |
| (2) Design, development and production: system architecture; vulnerability handling (SBOM, CVD policy, contact address, update distribution); production and monitoring | S1 Sections 1–4 and 7, the security scope 6.16, this spec 6.3–6.8, the CI tier definitions and the reproducibility results (SEC-15) |
| (3) The cybersecurity risk assessment | The security scope and its thirty-nine threat-model files, plus the **traceability table**: each item of Annex I Part I(2)(a)–(m) and Part II(1)–(8), and Art. 13(5)'s due diligence on third-party components (cargo-vet and cargo-deny records, SEC-14), mapped to the Izcal rule and target IDs that implement it and to the test evidence of this release, with a justification wherever an item is not applicable (Art. 13(3)–(4)) |
| (4) Support-period rationale | Section 6.2 and the per-platform statements |
| (5) Standards applied | EN 304 617 clause mapping once cited; until then "solutions adopted", pointing to the traceability table |
| (6) Test reports | Tier results, fuzzing campaigns, SEC-24 audit reports, the security regression suite |
| (7) Declaration of conformity | Below |
| (8) SBOM | 6.8 |

The traceability table's rows are anchors: each Izcal rule that implements a CRA item carries an anchor to the item, in the same way algorithms carry spec anchors (G9), so `hg-anchor` can generate and check the table instead of a person maintaining it (amendment, 9.5). The set is published at the placing on the market (Art. 32(5)), kept for **ten years** or the support period (Art. 13(13)), and updated as the support period runs (Art. 31(2)). What may be withheld is open (Q-CO-9); the recommendation is to publish everything except live secrets and personal data, which the set does not contain by construction (CO-D12).

**Declaration and CE marking.** One EU declaration of conformity **per release line**, in Annex V's structure, listing the versions it covers and updated with each release, rather than one per point release; the Release gate requires a **named human signatory** of the entity (CO-D1) to approve each update of the declaration, because a legal statement is made by a person, not a build job. A **simplified declaration** in About gives the exact address of the full one (Art. 13(20)). For software, the CE marking goes "either to the EU declaration of conformity … or on the website accompanying the software product", in a section "easily and directly accessible to consumers" (Art. 30(1)); Izcal puts it on the download page and the declaration.

**User security information (Annex II).** One "Security information" page per release line, linked from About and the download page and kept online for ten years (Art. 13(18)), containing: the manufacturer's name, postal address, digital contact and website; the single point of contact and the CVD policy; product identification (version and build ID, Art. 13(15)); intended purpose and security properties; known risky circumstances; the declaration's address; the type of support and **the end date per platform**; how to use the product securely, how updates install and how to turn automatic updates off; how to remove all data; the UPD-R5 switch list; and where the SBOM is. The shell is English-only in Stage 1 ([S1 13.2][S1]), but Annex II information must be in a language users and authorities can easily understand; the page is published in English and the official language of the establishment Member State from the first EU distribution, and in more languages with FI-6 (Q-CO-10).

### 6.10 Product properties the CRA adds

Most of Annex I Part I is already Izcal's security design. The traceability table (6.9) records the mapping; these items are new design:

| Annex I Part I | Izcal design |
|---|---|
| (2)(a) No known exploitable vulnerabilities at release | Release gate for **feature releases**: 0 open critical, high or exploited vulnerabilities **for which a fix exists**, in Izcal's own code or matched against the SBOM through OSV and RustSec. Security-only and emergency releases are exempt: they ship what they fix even while other issues are open, and each open item has a risk decision published, without exploit detail, in the advisory (CO-9) |
| (2)(b) Secure by default; reset to original state | Defaults are recorded in the conformity set from the settings registry (SH-D15); the recovery window's "Reset profile settings" (SH 6.14) is the reset, and Settings gains the same action |
| (2)(c) Automatic security updates, opt-out, notification, postpone | 6.3 |
| (2)(d) Report possible unauthorised access | Stage 2 sync: a notice on every new device sign-in (FI-35). Stage 1 has no account; OS account security governs the local profile |
| (2)(f) Integrity, and report corruptions | Already designed: SG-11 preserves user data or explicitly reports it on corruption ([SG][SG]), and update and channel verification failures show the "updates are stale" state (SEC 6.16). This spec adds only the user-facing wording of the corruption report |
| (2)(l) Record and monitor relevant internal activity, with an opt-out | The **prompt ledger** (SH-D11) gains security event classes beside consent decisions: extension installs and capability changes, policy changes, update events, UPD-R5 switches, certificate-error overrides, protected-media switches, DevTools and automation attachments. They take the ledger's retention (90 days†), keys, clearing and **private-window rule (private windows write nothing)**, a switch of their own, and are never uploaded |
| (2)(m) Remove all data and settings permanently | "Delete everything in this profile" and "Delete all Izcal data for this user on this device". Under per-user installs (SEC-D14), "this device" means this OS user. The second action has a published **deletion inventory**: profiles, the OS keychain entries Izcal created, protected-media mode's WebView2 and WKWebView data folders, model files, local crash dumps, recordings and OS-bound keys. Both first perform **server deletion** of crash reports and telemetry with the held tokens (6.13), then local deletion; in Stage 2 the first also deletes server-held sync data (FI-35). The transfer path is `izcal-export` (SH 6.12) |

### 6.10a Accessibility conformance report

**What (PO-14.6).** An **ACR in the VPAT 2.5Rev INT edition** (WCAG 2.2 A and AA, Revised Section 508, EN 301 549) per release line and per platform (Windows, macOS, Linux; Android in Stage 2). EN 301 549 v3.2.1 is the cited version until v4.1.1's Official Journal citation and an ITI template for it exist; then the report moves to v4.1.1 (Q-CO-12). The scope is the one browser vendors use, the browser UI and the exposure of web content to platform accessibility APIs, **without their exclusions**: Izcal's ACR covers the prompt and recovery windows, the PDF viewer and the DevTools frontend, which Firefox and Chrome leave out of theirs ([HIST 5.7, 5.8][HIST]). Documentation and support are covered under EN 301 549 chapter 12.

**How.** `hg-acr` generates each criterion's conformance level from test results that already exist: SH-3 (WCAG 2.2 AA, automated and manual), SH-4 (keyboard), SH-5 (names and roles), SH-27 (zoom and reflow), AX-1, AX-2 and AX-7 (content exposure and real screen readers), AX-42 (low vision and motion), and the AT-lab audit that UI-1 already requires each release. Every "Supports" cell links to the test IDs and results of that release; a criterion with no test evidence is never reported as "Supports" (CO-16's failure action; Q-CO-12 asks which levels the template allows). The UAAG 2.0 statement UI-1 requires is published beside it. The report says it is **self-attested** unless that release was audited; an **external audit** runs before the first Stable and then once a year† (CO-16). Rationale for yearly†: the generated report carries the per-release evidence, so the audit checks the method, which changes slowly.

**EAA.** The Directive never names browsers; Izcal's exposure is indirect, through sites that must comply and depend on the browser exposing their content to assistive technology ([FI factcheck 5a][FIFC]). No duty is recorded.

### 6.11 Corrective action, authorities and cessation

**Withdrawal and recall (Art. 13(21)).** When a build is found not in conformity, the manufacturer must "immediately take the corrective measures …, or … withdraw or recall the product". Izcal's mechanism is a **withdrawn-build state** in the TUF targets metadata, distinct from UPD-R5's switches: a withdrawn build's updater moves to the corrected build at once (as an emergency release), the shell shows a notice naming the reason, the download page and archive stop offering the build to users (it stays in the archive for authorities), and the advisory feed records the withdrawal. A withdrawal is a severe-incident-class event in the record (6.7) when it stems from a security cause. CO-27 measures it.

**Requests from authorities (Art. 13(22)).** A reasoned request from a market surveillance authority is answered by the security officer on duty, from the published documentation where it suffices, within **10 working days†** (CO-28), and logged in the record. Rationale: the documentation is generated and published per release, so most requests need only a pointer; 10 working days leaves time for counsel on the rest.

**Cessation (Art. 13(23)).** The project's end-of-operations plan, published with the CVD policy: authorities and users are told at least **6 months†** before operations stop (a manufacturer must inform them "before the cessation of operations takes effect"); the source is already public under MIT, so a successor can take it over; the TUF root and release keys are either transferred to a named successor under a recorded ceremony or used once to publish a final "no further updates" state; the CNA scope is handed to the successor or returned to the Root; and the archive and documentation stay online for their ten years. Rationale for 6 months†: it covers one Annex II update and gives enterprises a procurement cycle to move.

### 6.12 Hosting and where personal data flows

Every data flow, its operator class, region, who sees IP addresses and the retention control Izcal holds:

| Flow | Operator | Region | Sees IP addresses | Retention control Izcal holds |
|---|---|---|---|---|
| Security repository and vulnerability record | Izcal, on EU-hosted infrastructure (not GitHub) | EU | Officers only | Full |
| Intake form and role mailbox | Izcal, EU-hosted | EU | Intake service | Full; 6.18's table |
| Public source repository and CI | GitHub (SEC 6.16, SEC-27) | US | GitHub | None; holds no personal data from users |
| Update, trust-data, list and model payloads | A CDN chosen under CO-D17's condition: per-request logs Izcal can switch off or delete within 24 h†, and aggregate counts per (day, version, channel, platform, country) | Global edges | CDN | Raw logs ≤ 24 h†, then counts only. If SEC 6.16's hosting (GitHub or a CDN without that control) stays, the counts are unavailable and CO-18 (a) is recorded as not met for that channel |
| OHTTP relay (NW-D17) | An independent operator, open (Q-NW-6) | Per contract | Relay | By contract. If Q-NW-6's fallback is chosen (direct to Izcal), ingestion sees IPs and CO-18 (c) is restated as "IP addresses dropped at ingestion before storage" |
| Crash and telemetry ingestion, feedback, breakage reports, phishing reports and appeals | Izcal, EU-hosted | EU | Only under the relay fallback | Full; 6.13's inventory |
| ENISA's single reporting platform | ENISA | EU | ENISA | Not Izcal's |
| Stage 2: push, sync, extension directory | Izcal, EU-hosted | EU | The service | Full |

**CO-D17** therefore reads: **personal data Izcal stores at rest stays in the EU**; transit processors (the CDN edges and the relay) are listed in the privacy notice with their transfer basis; the public source repository holds no personal data from users. This removes the dependency on the EU–US Data Privacy Framework for stored data while its appeal (C-703/25 P) is pending ([DATA 2.16][DATA]).

### 6.13 Data protection for the services Izcal runs

**The inventory.**

| Service | Personal data | Legal basis (to confirm, Q-CO-13) | Retention | Notes |
|---|---|---|---|---|
| Update, trust-data, list and model fetches | IP address at the CDN | Necessary for the security updates the CRA requires | Raw logs ≤ 24 h†, then counts | No per-install ID (NW-D17) |
| Crash reports (D-S4) | Minidumps may hold page strings and URLs | Consent (CO-D6) | 30 days† (SEC-35), reviewed in the DPIA | Uploaded through the OHTTP relay; a deletion token per report |
| Opt-in telemetry (FI-D12) | Pseudonymous metrics | Consent | Set by FI-11 | OHTTP; a deletion token per record, never per period; published schema (SH-D29) |
| Feedback form (FI-11) | What the user writes | Consent | 90 days† | Suppressed for under-18 signals (6.14) |
| Content-blocking breakage reports (CB-D18) | Scheme and host, optional path, settings vector, optionally a redacted recording | Consent per report | ≤ 30 days† (CB's own) | OHTTP |
| Phishing reports and site-operator appeals (phishing 6.6) | Reporter input; site operators' contact data in appeals | Legitimate interest (Q-CO-13) | Per the phishing spec | — |
| Security intake | Reporter identity and the report | Legitimate interest; the reporter's request | 6.18's table | EU-hosted |
| Root program correspondence (NW-D5) | CA contacts | Legitimate interest | Per NW | — |
| Stage 2 push | Push endpoint URLs | Contract (the user's subscription) | While subscribed | Q-SEC-12 |
| Stage 2 sync (FI-35) | Account email, device list, timestamps; payloads end-to-end encrypted | Contract | While the account exists | Arts. 15, 17, 20 apply to the account data |
| Stage 2 extension directory (EX-D10) | Developer accounts, uploads, reviews | Contract | Per EX | Assessed before it opens: a directory that stores third parties' packages and makes them public may be a hosting service or an online platform under the DSA, and its CRA role towards third-party extensions is open (Q-CO-28) |

**Rights without identity.** Crash reports and telemetry carry no account, so a person cannot prove which reports are theirs. Each uploaded crash report and each telemetry record gets its own random **deletion token** that only the client holds; tokens are never shared across records, so they link nothing (SH-28, SH-D29). `about:crashes` lists sent reports with their IDs and keeps the exact bytes that were sent until the server copy expires, which serves access (Art. 15) without a server query; "delete from server" presents the token through the OHTTP relay, and ingestion deletes the record and confirms. The deletion endpoint joins NW-D17's published list (amendment, 9.5). The deletion actions of 6.10 perform server deletion first, so a wipe cannot strand server copies. Access and portability for local data are `izcal-export` (SH 6.12); for Stage 2 sync, account export joins the same format and account deletion removes server data. Requests are completed within **7 days†**, inside the GDPR's one month; rationale: deletion by token is automatic, so a week covers only failures (CO-19).

**Crash-report consent (CO-D6, amends D-S4; accepted by the owner, 25 October 2026).** EDPB Guidelines 2/2023 say ePrivacy Art. 5(3) applies when "the accessing entity distributes software on the terminal equipment of the user that is stored and will then proactively call an Application Programming Interface ('API') endpoint over the network" (para. 33) ([EDPB](https://www.edpb.europa.eu/our-work-tools/our-documents/guidelines/guidelines-22023-technical-scope-art-53-eprivacy-directive_en)). That describes a crash uploader. The guidelines do not analyse the consent exemptions (para. 56), and in *Planet49* the Court of Justice held that a pre-ticked box is not valid consent ([DATA 2.4–2.6][DATA]). D-S4's per-crash prompt pre-selects "send" ([S1 6][S1]). The recommendation keeps D-S4's prompt and its first-run disclosure and makes sending an **affirmative act**: the prompt has "Send report" and "Don't send" with neither pre-selected, which is already the shell's rule for prompts ("no control is pre-selected", SH 6.4), and a Settings row "Send crash reports automatically", off by default, that can be turned on from the prompt. The cost is a smaller, less representative crash sample, which D-S4's own rationale warned about. The reversal is counsel confirming that the "strictly necessary" exemption covers crash upload (Q-CO-14). Update checks raise the same Art. 5(3) question, but the CRA requires automatic security updates by default, so the question for counsel is how the two fit, not whether to keep update checks (Q-CO-14).

**Other duties.** A **record of processing** (Art. 30; the small-entity exemption does not cover non-occasional processing); a **DPIA for crash ingestion** before the first public binary (minidumps holding arbitrary page strings, at scale, Art. 35); a **DPO assessment** (Art. 37); an **EU representative** only if the entity is not EU-established (Art. 27, moot under CO-D1); processor contracts with hosts, the CDN and the relay (Art. 28); a breach process inside the incident runbook (6.7). UK GDPR mirrors these for UK users. Whether Izcal is a CCPA "business" depends on its legal form and revenue (Q-CO-15); it sells and shares no personal data either way.

**Privacy notices (PO-15.1).** One notice per service in the inventory, with GDPR Art. 13's contents, plus a short browser notice that lists what the browser sends with default settings (NW-36's endpoint list) and what each opt-in adds. S1 13.2 already requires the privacy policy before M7; the planned privacy spec (FI-12) owns the browser notice's wording (CO-29).

### 6.14 Age assurance and age signals

**The position (FI-D15).** On the text, the UK Online Safety Act (s. 3, user-to-user and search services) and the DSA's Art. 28 (online platforms) appear not to reach a browser that runs neither, and Utah's app-store law covers mobile devices only ([DATA 1.15, 1.22][DATA]); the Stage 2 extension directory is assessed separately (6.13). The one text found that could reach Izcal's desktop build is **California AB 1043**, operative 1 January 2027: a developer of an application on "a computer, a mobile device, or any other general purpose computing device" must "request a signal … when the application is downloaded and launched", and a developer that receives one is deemed to have "actual knowledge" of the user's age range "across all platforms of the application". A 2026 amendment, AB 1856, reportedly exempts software distributed under licence terms that allow copying, modification and redistribution, but its signing could not be confirmed (Q-CO-16). Requesting a signal therefore has legal effect of its own (actual knowledge), which counsel weighs before the default is enabled.

**The design (CO-D13).** Izcal never asks for an age and never shows an age gate. Where the OS offers an age-signal API (Apple's Declared Age Range where it returns a value; a Windows API when it ships; Google Play Age Signals in Stage 2), the broker requests the bracket at launch, keeps it in memory only, never exposes it to web content, extensions or agents, and never transmits it. Its only effect: **for any bracket under 18, crash-report upload, telemetry and the feedback form are off and their prompts are not shown**. Two preconditions: each OS API's user interface is measured black-box first, so a request never shows the user an OS consent sheet on every launch; and counsel's answer on what "actual knowledge" would oblige (Q-CO-16). With both satisfied, the behaviour is enabled on every OS that offers an API, from that OS's first public binary or 1 January 2027, whichever is later. OS family controls are honoured where the OS exposes them to third-party browsers; FI-4 (OS integration) specifies which. Stage 2's Play distribution meets the Texas and Utah app-store laws through the mobile shell spec (FI-33).

### 6.15 The DMA as a non-gatekeeper

Izcal owes nothing under the DMA. Only designated gatekeeper browsers owe the choice screen ([FI factcheck 4][FIFC]).

| What Izcal must do | What Izcal may use | Decision |
|---|---|---|
| Nothing | Google's Data Portability API (Art. 6(9)) to import Chrome history, bookmarks and autofill from a Google account | **Refused in Stages 1 and 2 (CO-D16).** The history, bookmark and autofill scopes are "restricted" ([scopes](https://developers.google.com/data-portability/user-guide/scopes)); apps must complete an app verification, including a security assessment where required, re-verify annually, cannot serve users under 18, and must check availability by location ([overview](https://developers.google.com/data-portability/user-guide/overview)). The local-profile importer (SH-R2) is the open route FI-D14 prefers. Reversal: Chrome's local profile data stops being importable from disk |
| Nothing | The Android EEA browser choice screen | Stage 2 eligibility: Play distribution, at least 5,000 installs, support for a new Android version within 90 days ([Android](https://android.com/choicescreen/dma/browser)); an input to FI-33 and FI-2 |
| Nothing | Apple's EU browser choice screen | Not applicable: iOS is refused (S1 13.2) |

### 6.16 Terms, licence notices and trademark

**Terms (CO-D14).** The browser is software under the MIT licence; it needs no terms of use and shows no acceptance screen (onboarding has none, SH-D23). Services with accounts (Stage 2 sync, the extension directory) get service terms that grant Izcal only "what is needed to do what you ask", in plain words, which is where Mozilla landed after its 2025 revision ([Mozilla](https://blog.mozilla.org/en/products/firefox/update-on-terms-of-use/)). What EU consumer law requires beyond the MIT disclaimer for the binary is for counsel (Q-CO-17).

**Licence notices (PO-15.2, already owned by SH 6.14).** `about:licences` and `about:credits` are generated from the release's SBOM (6.8).

**Trademark policy (CO-D15, PO-15.11).** Written before the first public binary, after D-S9's clearance search and filings (Nice classes 9 and 42, in the EU and the US). It allows redistribution of unmodified builds under the name; it allows distributions to package and patch Izcal under the name if the changes are disclosed, which avoids a split like Debian's decade of Iceweasel; it requires forks that change behaviour to rebrand; and it forbids presenting a build as official. A distribution that modifies the browser and ships it under the Izcal name may itself become a manufacturer under the CRA's substantial-modification rule (Arts. 21–22), which the policy notes without settling (Q-CO-18).

### 6.17 Export control

Izcal's own cryptography is in its published MIT source and uses published algorithms: TLS 1.3, X25519, AES-GCM, ChaCha20-Poly1305 and the ML-KEM hybrid (FIPS 203). Under the EAR, publicly available encryption source code is not subject to the EAR (15 CFR 742.15(b)), nor is object code built from it (the note to 734.3(b)), and the notification to BIS is owed only for "non-standard cryptography", meaning proprietary or unpublished functionality (772.1), a narrowing made on 29 March 2021 ([DATA 5][DATA]). So no notification is planned. D-S7 does **not** rule out closed cryptography for ever: its licensing track could lead Izcal to ship a licensed closed CDM in its own build, and that would need a mass-market classification (License Exception ENC) and a re-decision of Art. 32(5) and the SBOM treatment (6.1, 6.8). The EU dual-use regulation's public-domain note works the same way for the published source. Counsel confirms the ML-KEM point and whether US sanctions limit free downloads (Q-CO-19). S1 13.2's "with notification where still required" becomes "notification only for non-standard cryptography; none is planned; re-decided if a closed module ships" (amendment, 9.5).

### 6.18 Security, privacy and retention in the response operation

- **The record is sensitive.** Reproducers, recordings and reporter identities are encrypted at rest, held in the EU, readable by the security officers only.
- **One retention table** for the response operation:

| Item | Retention† | Rationale |
|---|---|---|
| Reporter's reproducer and attached recordings | Deleted 90 days after the fixing release | Long enough for variant analysis and a regression reduction, no longer |
| Exploitation evidence | 10 years, in the record | Art. 14 reports and Art. 13(13)'s documentation period |
| Reporter contact | 2 years after the record closes, unless credit or follow-up is ongoing | A reporter often returns with a variant within a release cycle or two |
| The record without the items above | 10 years | Art. 13(13) and the PLD's ten-year period |

- **The security repository is a crown jewel.** It holds unreleased fixes, so it carries SEC-27's controls and its compromise is a severe incident (6.7).
- **Aggregate counts never become logs.** Aggregation runs at the CDN within 24 hours†; counts below **1,000†** per cell are merged upward before publication, so no small community or country is singled out. Rationale for 1,000†: at that size a count cannot be tied to a small identifiable group.
- **AI off.** Nothing here adds an AI surface to the browser (B10). Triage tooling follows SEC-27 (6.5).

### 6.19 Staffing and the response workload

**The workload.** Estimates†, assuming 30–100 reports a month at AI-scale volume, under one in five of them valid:

| Activity | Engineers† |
|---|---|
| Intake, triage and human replies (1–2 h per report) | 0.3–1.2 |
| Coordination, embargoes, upstream reports | 0.2 |
| Advisory, CVE and CSAF publication, quarterly report | 0.2 |
| CRA drills and reporting | 0.1 |
| Security-only releases and the occasional short branch (CO-D5 (c)) | 0.2–0.5 |
| **Total security response** | **1.0–2.2** |

The security track (SEC-D2: 3–4†; about 2.6–3.8 engineers of in-house effort in SEC 9.2) already counts the on-call rotation, about 0.5†, so the **net addition is 0.5–1.7 engineers†**, plus the **compliance engineer at 0.5–1†** (`hg-advisory`, `hg-sbom`, `hg-conformity`, `hg-acr`, the release gates and traceability anchors). Together: **1–2.7 engineers†**, an S1 8.3 row (amendment, 9.5). If CO-D5 (a) is chosen instead, add 2–3.5†.

**People and roles.** At least **two security officers** share on-call, with a **retained external incident-response provider** covering weekends and holidays from the first EU distribution, so neither officer is on call half of every week indefinitely. The concentrated roles are split: the TUF `releases` key holders (SEC-D4's pair) are not the only CNA contacts and EU Login holders; the compliance engineer holds the second CNA contact and an EU Login account. External counsel (CRA, GDPR, consumer law, trademark, export), an external accessibility auditor (yearly†) and a data-protection lead (0.2 person†, until a DPO is required) are owner-budget items in D-S8's ledger beside the media programme's legal costs.

### 6.20 Alternatives

| Question | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|
| CRA role | (a) Manufacturer; (b) steward; (c) assume outside the CRA | **(a)** (FI-D11) | Full manufacturer duties | Counsel finds Izcal outside the CRA under the chosen D-S8 option: keep the pipelines, drop the declaration and CE marking |
| Conformity route | (a) Art. 32(5) module A with public documentation; (b) wait for EN 304 617 and use module A; (c) notified body | **(a)**, applying EN 304 617 in full once cited | Publishing the documentation | D-S8 adds a non-FOSS edition, or D-S7 ships a closed module: that edition or build takes (b) or (c) |
| Testing builds | (a) Art. 4(3) unfinished software with notice, opt-in and per-build expiry; (b) testing builds as conforming products; (c) no testing builds in the EU | **(a)** | A notice, an expiry rule | Counsel says a standing channel does not qualify |
| Platform support ends | (a) Legacy branches; (b) OS end bounds the period; (c) per-platform statement at release | **(c)** | 0.2–0.5† | Counsel rejects a per-platform period shorter than five years: (a) |
| Crash consent | (a) Keep pre-selected send; (b) nothing pre-selected; (c) opt-in setting only | **(b)** | Smaller sample | Counsel confirms the strictly-necessary exemption |
| Advisory format | (a) CSAF 2.0 and OSV; (b) OSV only; (c) prose only | **(a)** | Two generators | CSAF 2.1 becomes a Standard: move to it |
| CVE | (a) CNA under ENISA; (b) CNA under MITRE; (c) request IDs from GitHub's CNA | **(a)**, (c) until then | Contacts, rule compliance | ENISA's Root closes to new CNAs: (b) |
| SBOM | (a) CycloneDX at BSI depth, SPDX on request; (b) both always; (c) CRA top-level minimum | **(a)** | Negligible for Izcal | A recipient or implementing act requires SPDX: generate it every release |
| Intake | (a) Self-hosted EU form and mailbox; (b) a commercial EU platform; (c) GitHub private reporting | **(a)** | Running a form and mailbox | Volume exceeds the officers: (b) |
| Security release timing | (a) On demand within bounds; (b) a fixed weekly slot | **(a)** | None | Packagers ask for predictability two quarters† running: (b) |

## 7. M-1 compliance

- **No third-party code.** No component in this spec is taken from another project's source. The research notes and the fact-check record that no source file, header, commit, diff, code review or repository data file was opened, including standards material hosted in code repositories ([CRA M-1 statement][CRA]; [OPS M-1 statement][OPS]; [factcheck][FC]).
- **Built new by Izcal:** `hg-advisory`, `hg-sbom`, `hg-conformity` and `hg-acr`, the shell surfaces and the ledger's security event classes. The four crates are build tools; none ships in the browser binary (6.0's crate rules). They are Izcal's own under the standing build-our-own answer. Server software Izcal runs is outside HGBA-2 2.3.2's rule for what ships (FI-D13).
- **Used unmodified, and why each is allowed:**

| Item | Role | Standing |
|---|---|---|
| OpenPGP signing for CSAF documents and the role mailbox | Tool | An implementation of a published standard (RFC 9580); chosen under the toolchain rules of the coding standard; nothing ships |
| cargo-deny and cargo-vet | Tool | Already decided (SEC-14) |
| RustSec advisory database and its OSV records | Data input to a release gate | Advisory data, not code; not shipped |
| GitHub (public repository and CI), the CDN, the OHTTP relay, ENISA's platform, CVE Services | Services | Not shipped; listed in 6.12 with their data roles |
| Schema validators | Tool | Izcal's own, written from the specifications (CO-D20); third-party validators are not used |

- **Anchored to:** for the record and its projections, the OASIS CSAF 2.0 Standard, the OSV schema specification site, the CVE Program's record-format documentation and CNA Rules 4.2.0, RFC 9116 and ISO/IEC 29147 and 30111 (cited, not reproduced); for the SBOM, ECMA-424 (CycloneDX), the SPDX 3.0.1 specification and BSI TR-03183-2 v2.1.0; for conformity, Regulation (EU) 2024/2847's articles and annexes; for the ACR, VPAT 2.5Rev, EN 301 549, WCAG 2.2 and WCAG2ICT; for data protection, the GDPR, the ePrivacy Directive and EDPB Guidelines 2/2023.
- **Schema-defined formats (CO-D20, accepted by the owner, 25 October 2026: normative schemas may be used as test oracles).** Six formats the generators write have machine-readable definitions: CSAF 2.0 and its VEX profile, OSV, the CVE record, CycloneDX and SPDX. This is not D-S12's case: D-S12 and its DT-D12 clarification concern adapters to protocols with **no normative specification**, where a repository's JSON files "are neither an input nor an oracle". Here the format owner publishes a normative definition. The owner decides how to classify it (CO-D20).
- **No value derived from code.** No threshold here comes from another project's code. Chrome's severity guidelines live in its source repository and were not read; Izcal's scale is its own ([OPS 3.7][OPS]).

## 8. Targets

Thresholds marked † are set by this document from first principles; rationales are in the sections cited or in the table below. "Record" means the vulnerability record and its witnessed commitments (6.4).

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CO-1** | **CRA reporting is on time** | From the awareness field (6.4): exploitation assessment ≤ 20 h†; early warning ≤ 24 h; notification ≤ 72 h; final report ≤ 14 days after the fix (severe incidents: ≤ 1 month after the notification); **100%** | Record clocks against the witnessed commitments | Postmortem within 7 days†; a missed 72-hour notification or final report blocks the next feature release until a drill passes |
| **CO-2** | **The runbook is rehearsed** | Twice a year†; from record to early-warning text ≤ **4 h†**; every officer's EU Login account with MFA valid; the CSIRT selection current | Drill log, with SEC-2 | No EU distribution until a drill passes |
| **CO-3** | **Serious fixes never need a feature update** | **100%** of critical, high and exploited fixes for Release users ship in a security-only, emergency or mitigation release; every exception has a written technical-infeasibility reason in the documentation | Release diff classification | The release is re-cut without the feature changes, or the reason is written before publish |
| **CO-4** | **No fix is public before its release** | **0** security fixes visible in the public repository before the release that ships them, excluding fixes already public upstream | Public merge times against release times, per record | The leak is treated as public disclosure: the issue moves at once to the exploited-risk path (emergency release or UPD-R5 switch), and a postmortem follows |
| **CO-5** | **Automatic security updates are the default, with opt-out and postpone** | For the per-user installs on all three platforms and for Linux packages under CO-D19: on by default; opt-out in ≤ 2 actions†; postpone ≤ 7 days† (emergency: next start); notification shown; **100%** in the update harness | Release-gate update harness (SH-24's) | Ship blocker for the affected install form |
| **CO-6** | **Every fixed vulnerability is published, in every projection** | **100%** of fixed vulnerabilities in released builds in a valid CSAF 2.0 advisory at release, published in the trusted-provider role with the options of 6.4; OSV at release; CVE record ≤ **24 h†** after the advisory (CNA rule: 72 h); scale and enterprise notes in every security release | `hg-advisory` validator; provider check; CVE publication times | Release blocked for a missing advisory; a late CVE record is a postmortem |
| **CO-7** | **Clocks cannot be backdated** | **100%** of receipt clocks machine-stamped, with the receipt sent to the reporter; **100%** of chain checks pass against the daily† commitments; typed clocks flagged and counted | Chain verifier in CI; commitment audit | Security-response log frozen and audited before the next advisory |
| **CO-8** | **Every artifact has a complete SBOM** | **100%** of release artifacts have a CycloneDX ≥ 1.6 document with BSI TR-03183-2 v2.1.0's required fields; **0** components in the build inputs missing from the SBOM; signed and attested | `hg-sbom` against the lockfile, data ledger and image inventory; Izcal's validator | Release blocked |
| **CO-9** | **No fixable known-exploitable vulnerability ships in a feature release** | Feature releases: **0** open critical, high or exploited vulnerabilities with a fix available, in Izcal's code or SBOM matches in OSV and RustSec. Security-only and emergency releases: every open item carries a published risk decision | Release gate | Feature release blocked; a missing risk decision blocks the security release until written |
| **CO-10** | **Conformity is re-established every release** | **100%** of releases have the generated conformity set; the traceability table covers **100%** of Annex I Part I(2)(a)–(m), Part II(1)–(8) and Art. 13(5) with a rule ID and either test evidence from this release or a justification; published at release; the declaration update approved by the named signatory | `hg-conformity` in the Release gate; publication check | Release blocked |
| **CO-11** | **Users know their support date** | **100%** of builds show the per-platform support end month and year in About and on the download page; the end-of-support notice appears within one start after the date in a clock-shifted harness | Release-gate harness | Ship blocker |
| **CO-12** | **Old security updates stay available and verifiable** | **100%** of security updates downloadable for ≥ **10 years** or the support period; the **oldest** archived build verifies (timestamped signature, archive manifest, witnessed commitment) each quarter† | Archive check | Restore the archive before the next release |
| **CO-13** | **Reporters hear from a person** | Acknowledgement ≤ 3 days† for **100%** (SEC-25), median ≤ 24 h†; triage verdict ≤ 7 days†; status every ≤ 7 days†; **0** acknowledgements by automation alone | Record clocks | Postmortem; repeated misses go to SEC-25's staffing review |
| **CO-14** | **The contact surface is valid** | security.txt valid under RFC 9116 with `Expires` ≤ 1 year ahead and the `CSAF` field; CSAF provider metadata valid; checked weekly† | CI job against the live site | Fixed within 24 h† |
| **CO-15** | **Testing builds are labelled and contained** | **100%** of Nightly and Beta builds show the Art. 4(3) notice on the download page, the first-run screen and About; **0** moves from Release to a testing channel without the notice and a confirmation; Nightly builds expire at 30 days†, Beta at cycle plus 30 days† | Release-gate harness; updater test | Testing-build distribution paused (after 11 December 2027, or earlier by choice) |
| **CO-16** | **The accessibility report describes the shipping version** | An ACR per release line and platform published ≤ **14 days†** after release; **100%** of "Supports" cells link to that release's tests or audit records; external audit before the first Stable and yearly† | `hg-acr` output check | Unbacked criteria are reported with the template's lower conformance level and a remark naming the missing test; the release notes say so |
| **CO-17** | **No crash report leaves without an affirmative act** | **0** crash uploads without a click on "Send report" or a prior opt-in to automatic sending (under CO-D6); **0** with an under-18 age signal | T1 test; ingestion audit | Crash upload disabled until fixed |
| **CO-18** | **Izcal's servers hold the minimum** | (a) Raw CDN and update logs with IP addresses deleted ≤ **24 h†**; (b) **0** per-install identifiers in update and list requests; (c) **0** client IPs stored by crash and telemetry ingestion (through the relay, or dropped at ingestion under Q-NW-6's fallback) | NW-36 capture; configuration audit; relay test | The service stops logging until fixed; on a CDN without the control, (a) is recorded as unmet for that channel |
| **CO-19** | **Rights work without identity** | **100%** of token deletions completed ≤ **7 days†**; `about:crashes` holds the bytes sent for every unexpired report; Stage 2 account export and deletion ≤ 7 days† | Ingestion audit; T1 test | Postmortem; the GDPR's one month is the hard limit |
| **CO-20** | **One incident, every clock** | In each CO-2 drill a severe-incident scenario produces the CRA 24 h and 72 h texts and, with personal data, the GDPR 72 h notification | Drill log | As CO-2 |
| **CO-21** | **Age signals stay on the device** | On every OS with an API, from its first public binary: the signal requested at launch with no repeated OS sheet; **0** bytes of it on the network or to content, extensions or agents; under-18 brackets disable crash upload, telemetry and feedback | T1 tests per OS; NW-36 capture | Ship blocker for that OS |
| **CO-22** | **Licence notices match what shipped** | `about:licences` equals the release SBOM's licence set, **100%** | Release-gate comparison | Release blocked |
| **CO-23** | **Upstream maintainers are told** | **100%** of vulnerabilities found in dependencies reported to the maintainer ≤ **7 days†** | Record clocks | Postmortem |
| **CO-24** | **Fixes reach users** | Published per security release: estimated share of installations on the fixed version at 7 days, from aggregate counts (fixed-version payload downloads over the mean daily count of TUF timestamp fetches, which every installation makes once a day); cells < 1,000† merged; for an exploited fix, ≥ **90%†** within 7 days | Aggregate counts | Investigate the updater and rollout; a second miss goes to FI-2's owner |
| **CO-25** | **Izcal is a CNA before it ships** | CNA status (ENISA Root) and two administrative contacts held by different people† before the first public Nightly | Partner list | First public binary waits, or ships with GitHub's CNA as a recorded interim |
| **CO-26** | **Projections leak nothing** | **0** canary values from `private` or `restricted` fields in any public projection; a correction regenerates **100%** of projections in one step | CI canary suite | Publication blocked |
| **CO-27** | **A withdrawal reaches users** | A withdrawn build's installations move to the corrected build, and its download links close, within **72 h†** of the decision, in the update harness and in each CO-2 drill | Harness; drill log | Treated as a severe incident in the drill's postmortem |
| **CO-28** | **Authorities get answers** | **100%** of reasoned requests answered ≤ **10 working days†** | Record | Postmortem |
| **CO-29** | **Every service has a notice** | A privacy notice for **100%** of the services in 6.13's inventory published before the service opens; the browser notice before M7 | Publication check | The service does not open |

**Rationales for thresholds not explained in place.** 20 h† assessment and 4 h† drill: Section 6.4 and 6.7. 7 days† triage, status and deletion: aligned with the critical bound. 24 h† CVE record: the CNA rules' recommended time, which Izcal can meet because the record exists before the advisory. 24 h† rollout and CDN logs: a day covers the staged rollout and is the shortest window in which aggregation runs reliably. ≤ 2 actions† to opt out: "clear and easy-to-use" means a setting, not a procedure. 14 days† ACR: the release-cycle length the incumbents now use. 90%† in 7 days: with automatic updates on and a 7-day postpone limit, the installations that miss it are the ones not running. 72 h† withdrawal: the emergency path's bound. 10 working days†: Section 6.11. Weekly† security.txt check: shorter than any plausible expiry gap. Two contacts held by different people†: one departure must not remove the CNA's contacts.

## 9. Milestones and staffing

### 9.1 Milestones

| When | What must exist |
|---|---|
| **Now** | This spec; the legal-advice brief (Section 11.1) sent before the first EU distribution, as FI-D11 requires |
| **M1 exit** | This spec's fields in **FI-11's shared recording and crash-report header** (Section 9.4) |
| **Before M5** | Inputs to FI-2 (support statements, channels, security-only line, archive, rollout limits, Linux auto-update) and to FI-11 (OHTTP upload, per-record deletion tokens, retention); GR-D21's drop rule amended (CO-D5) |
| **M7** | Privacy notices per service (S1 13.2's deadline; CO-29); the ACR pipeline running with SH-M6's AT lab; DPIA for crash ingestion drafted |
| **Before the first public Nightly** (the first public binary and the first EU distribution; at least 3 months before the first Stable, DU-D10) | Disclosure policy, security.txt, intake with machine stamps, the record and `hg-advisory`, the security repository on EU hosting, CNA status (CO-25), `hg-sbom`, the trademark policy and filings, the export-control check, SEC-D10's and SEC-24's external security audit, SEC-D4's second person, the age-signal API measurements per OS; legal advice on Section 11.1; the entity (CO-D1); EU Login accounts with MFA; the runbook drilled (CO-2); the withdrawal mechanism tested (CO-27); the incident-response retainer; the Art. 4(3) notice and per-build expiry (CO-15; preparation until 11 December 2027, Art. 69(2)) |
| **Before the first Stable** (the first Release placed on the market) | The conformity set, declaration with its signatory, CE marking and Annex II page (CO-10); the first ACR and its external accessibility audit (CO-16); privacy notices for every service it uses (CO-29) |
| **9 December 2026** | The PLD applies to products placed after this date, if commercial; no Izcal product is on the market |
| **1 January 2027** | AB 1043 operative; the age-signal behaviour (CO-21) applies from each OS's first public Nightly |
| **11 December 2027** | The CRA in full |

### 9.2 Staffing

Section 6.19 gives the workload and roles: a net **1–2.7 engineers†** (security response plus the compliance engineer), at least two security officers inside the security track with an external incident-response retainer, split key and contact roles, and owner-budget items for counsel, audits and data protection. The S1 8.3 row is proposed in 9.5.

### 9.3 Decisions for the owner

**All twenty-one were accepted by the owner on 25 October 2026 as recommended** (CO-D1 first, then CO-D2 to CO-D21). Build-versus-reuse is not on this list: the standing answer is that Izcal builds its own (M-1, D-S10).

| ID | Decision | Options | Recommendation | Reversal trigger |
|---|---|---|---|---|
| **CO-D1** | Legal entity and establishment | (a) An entity in an EU Member State before the first EU distribution; (b) a non-EU entity with an EU authorised representative; (c) no entity until a Release | **Accepted by the owner, 25 October 2026:** **(a)**, legal form per D-S8 and counsel | Counsel or D-S8 makes (b) materially cheaper |
| **CO-D2** | **Conformity route, and its dependence on D-S8** | (a) Art. 32(5) module A with the technical documentation public, which holds only while every edition is free and open-source software; (b) module A on EN 304 617 once cited; (c) a notified body | **Accepted by the owner, 25 October 2026:** **(a)**, applying EN 304 617 in full once cited. **If D-S8 chooses a paid edition that is not free software, or D-S7's licensing track puts a closed module in Izcal's own build, that edition or build needs (b) or (c); the owner should weigh that cost in D-S8** | Counsel reads Art. 32(5) differently, or D-S8 or D-S7 changes the product's FOSS status |
| **CO-D3** | **Nightly and Beta under the CRA** | (a) Art. 4(3) unfinished software: notice, opt-in, per-build expiry, full security pipeline, exploited issues reported; (b) testing builds as conforming products; (c) no testing builds offered in the EU | **Accepted by the owner, 25 October 2026:** **(a)** until counsel answers Q-CO-3; until 11 December 2027 it is preparation, since Art. 69(2) applies only Art. 14 to products placed before then | Counsel says a standing channel is not "a limited period": Beta takes (b); Nightly becomes a time-boxed programme or source-only |
| **CO-D4** | Support period | (a) At least five years per release, stated per platform (CO-D5), delivered as the latest release; (b) longer, citing recital 60 | **Accepted by the owner, 25 October 2026:** **(a)**, reviewed at the Stage 1 exit | Counsel or Commission guidance expects longer for browsers |
| **CO-D5** | **When a platform leaves its vendor's support** | (a) Legacy security branches for five years after the last release on that OS; (b) the OS's end bounds a stated period; (c) a per-platform statement fixed at release: the earlier of five years and the OS version's vendor end (or Izcal's conservative estimate where none is published), with a short branch only if the OS ends before the stated date | **Accepted by the owner, 25 October 2026:** **(c)**: 0.2–0.5 engineer† against 2–3.5† for (a); reviewed together with the distribution spec's DU-D15 | Counsel rejects a per-platform period shorter than five years: (a), with its cost added to S1 8.3 |
| **CO-D6** | Crash-report consent (amends D-S4) | (a) Keep "send" pre-selected; (b) nothing pre-selected, with an opt-in "send automatically"; (c) opt-in setting only | **Accepted by the owner, 25 October 2026:** **(b)** | Counsel confirms the strictly-necessary exemption: (a) may return |
| **CO-D7** | CVE numbering | (a) Supplier CNA under ENISA's Root before the first public binary; (b) under MITRE; (c) GitHub's CNA only | **Accepted by the owner, 25 October 2026:** **(a)**, with (c) until then | ENISA's Root closes to new CNAs |
| **CO-D8** | Advisory formats and IDs | (a) Izcal IDs with aliases; CSAF 2.0 trusted provider and OSV; (b) CVE IDs only, prose advisories | **Accepted by the owner, 25 October 2026:** **(a)** | CSAF 2.1 becomes a Standard: adopt it |
| **CO-D9** | Severity scale and bounds | (a) Izcal's four levels plus an exploited flag, one bound table (critical ≤ 7 days† everywhere), CVSS 4.0 secondary; (b) CVSS as primary, SEC-1 and SEC-25 as they are | **Accepted by the owner, 25 October 2026:** **(a)** | Two quarters† of drills show the critical bound cannot be met outside the engine: restate it in the decision log with the measured time |
| **CO-D10** | Intake and bounty | (a) Self-hosted EU form and mailbox with machine stamps, reproducer required, no paid bounty until triage is staffed (SEC-D10 plus two conditions); (b) a paid platform from launch | **Accepted by the owner, 25 October 2026:** **(a)** | Funding and staffed officers: a paid programme on Izcal's intake or an EU-hosted platform |
| **CO-D11** | SBOM | (a) CycloneDX ≥ 1.6 at BSI depth, published, SPDX on request; (b) both formats every release; (c) CRA minimum, unpublished | **Accepted by the owner, 25 October 2026:** **(a)** | An implementing act or a recipient requires SPDX: generate it every release |
| **CO-D12** | What the public technical documentation may withhold | (a) Nothing but secrets and personal data, which it does not contain; (b) withhold the risk assessment's sensitive parts | **Accepted by the owner, 25 October 2026:** **(a)** until counsel answers Q-CO-9 | Counsel requires less, or a regulator objects |
| **CO-D13** | Age signals | (a) Request the OS signal at launch on every OS with an API, after the API's UI is measured and counsel answers Q-CO-16; local only; under-18 disables crash upload, telemetry and feedback; (b) only where the law is confirmed to apply; (c) nothing | **Accepted by the owner, 25 October 2026:** **(a)** | An OS signal proves unreliable enough to disable reporting for adults, or counsel finds "actual knowledge" costly: (b) |
| **CO-D14** | Terms of use | (a) None for the browser; service terms only for account services; a privacy notice per service; (b) terms for the binary with acceptance at first run | **Accepted by the owner, 25 October 2026:** **(a)** | Counsel finds EU consumer law needs terms for the binary |
| **CO-D15** | Trademark policy | (a) Unmodified and disclosed-patch packaging allowed under the name; behaviour-changing forks rebrand; (b) every modified build rebrands | **Accepted by the owner, 25 October 2026:** **(a)** | A distribution's patched build causes a security incident attributed to Izcal |
| **CO-D16** | Google Data Portability API | (a) Refuse in Stages 1 and 2 (local import is the open route, FI-D14); (b) build it as a fallback importer | **Accepted by the owner, 25 October 2026:** **(a)** | Chrome's local profile data stops being importable from disk |
| **CO-D17** | Where personal data lives | (a) Stored at rest in the EU, transit processors listed (6.12); (b) wherever is cheapest under the DPF | **Accepted by the owner, 25 October 2026:** **(a)** | The CJEU upholds the DPF in C-703/25 P and a non-EU host is materially better: revisit with counsel |
| **CO-D18** | Security release timing | (a) On demand within each bound, fixes private until release, emergency and mitigation paths; (b) a fixed weekly slot | **Accepted by the owner, 25 October 2026:** **(a)** | Packagers ask for predictability two quarters† running: (b) |
| **CO-D19** | Automatic updates for Linux packages | (a) A drop-in for the distribution's unattended-update mechanism, scoped to Izcal's repository, on by default with an opt-out, plus the in-browser notification; (b) notification only | **Accepted by the owner, 25 October 2026:** **(a)**, with FI-2 owning the packaging | Distributions object, or counsel finds (b) enough under "where applicable" (Q-CO-8) |
| **CO-D20** | **Schema-defined formats under M-1** | (a) A schema the format owner designates as part of its normative specification (CSAF's are part of the OASIS Standard) is specification text: cited by version, used as a test oracle, never copied into shipped code; a schema that exists only as a repository file is used only through the prose and black-box acceptance tests; (b) prose plus black-box acceptance only, for every format | **Accepted by the owner, 25 October 2026:** **(a)**, with Izcal's own validators | The owner applies DT-D12's rule to normative schemas too: (b) |
| **CO-D21** | Art. 14 awareness | (a) The stamped receipt of the first claim of exploitation with any evidence; (b) the positive assessment | **Accepted by the owner, 25 October 2026:** **(a)** until counsel answers Q-CO-26 | Counsel confirms (b) |

### 9.4 Input to M1's exit: this spec's fields for the shared header

**One header, one owner.** Recordings and crash reports share **one header definition, owned by FI-11** (crash, telemetry, feedback and experiment operations), fixed at M1's exit ([FI 8, item 1][FI]). This section is not a second definition: it lists the fields CRA reporting and the vulnerability record need, as this spec's contribution to FI-11's header, beside the enterprise-policy spec's **32-byte `PolicyDigest`** and the OS-integration spec's **recorded OS inputs**. The need is the same for all three: to identify exactly which product, configuration and environment a crash, a recording or a report came from, so a record's affected scope is computed rather than guessed.

**What this spec asks of the header.**

| Field | Why this spec needs it | Relation to the other contributions |
|---|---|---|
| Engine build ID | Keys the build manifest below; already `process.hgidl`'s `BuildId` | In M1's exit criterion |
| Data versions | Affected scope for data-channel issues | In M1's exit criterion |
| Decision-point log reference | Reproduction | In M1's exit criterion |
| **OS version** | Affected-platform scope; the per-platform support statements (CO-D5) | Taken from the OS-integration spec's recorded OS inputs if they carry it; otherwise added |
| **Process kind and sandbox tier** | Severity classification (6.4) | The audit's "process kind" |
| **Policy state** | Enterprise policy changes the attack surface | The enterprise-policy spec's `PolicyDigest` in recordings. **Request to FI-11:** crash reports carry only coarse flags for which risk-relevant policies are set, not the digest, because a digest of an organisation's policy set is close to unique to that organisation (SH-28's "0 identifiers") |
| **Header schema version** | Later fields need no format break | Owned by FI-11 |

**Kept out of the header.** Everything that is a function of the build is looked up in a **published build manifest** keyed by the build ID (signed with the release): product version and release line, channel, platform and ISA, tier ceiling, and the SBOM's location and digest. The SBOM digest lives in the manifest, never in the executable, because the SBOM hashes the executable. **The profile identity** the audit lists stays inside recordings only and is replaced by a fresh random value on every export or upload, so it never leaves the device. None of this changes the IPC wire format or the substrate traits.

### 9.5 Amendments this spec proposes to other documents (not applied)

This document edits no other file. The spec owners apply these with revision lines as `CONTRIBUTING.md` requires.

| Document | Proposed amendment |
|---|---|
| **Security scope** 6.17 | Point to this spec for the disclosure policy, intake, advisories, CNA, embargoes and CRA reporting; keep pre-release audits, variant analysis and the regression suite |
| Security scope SEC-1, SEC-25 | One bound table (6.4): SEC-25's critical bound for Izcal's own code tightens from 14 to **7 days†**; the exploited tier uses the single exploited test (Art. 3(42)); "fix publicly visible" joins the time-zero candidates; SEC-25's log is the vulnerability record of 6.4 with machine-stamped receipts |
| Security scope 6.16, SEC-27 | Add the private security repository on EU hosting under SEC-27's controls; its compromise, and any signing-key (including archival, intake and record keys), TUF, CI or channel compromise, is a CRA severe incident (Art. 14(5)) with SEC-16's and SEC-17's failure actions. The update hosting row adds CO-D17's CDN condition (log control and aggregate counts) |
| Security scope UPD-R1, UPD-R4, UPD-R5 | UPD-R1: the targets metadata carries the support end, exploited-build and withdrawn-build fields (6.2, 6.4, 6.11). UPD-R4: emergency releases go to every user at once with no staged rollout. UPD-R5: add **"protected-media mode off"** to the closed switch list; switches apply unless the user turns off "Apply emergency security switches"; the list is described on the Annex II page |
| Security scope SH-R4 | Refusing protected-media mode below a runtime version is also the mitigation for OS-owned-component advisories (6.4) |
| Security scope SH-R5, SEC-35 | The 30-day retention† is reviewed in the crash-ingestion DPIA; crash uploads go through the OHTTP relay with a per-report deletion token (6.13) |
| Security scope SEC-D4 | The fallback becomes: "the first public binary waits, or ships outside the EU only, or ships in the EU only as an Art. 4(3) testing build under CO-15", replacing "a preview without auto-update", which Annex I Part I(2)(c) does not allow for a Release. The second person is not also the only second CNA contact or EU Login holder (6.19) |
| Security scope SEC-D10 | The bounty trigger adds: triage staffed for AI-scale volume first, and a paid programme on Izcal's intake or an EU-hosted platform |
| Security scope Q-SEC-12 | Push retention is set in this spec's inventory (6.13) for Stage 2; the threat model stays the security scope's |
| **Stage 1 design** §1 crash-handler row and §6 crash-reports row | Owner-approved, 25 October 2026 (CO-D6): the per-crash prompt pre-selects nothing; "send automatically" is an opt-in setting |
| Stage 1 2.2 and G9 | Register G260–G269 for this spec (rules G260–G262 in 6.0; G263–G269 reserved); `hg-anchor` gains a legal-requirement anchor kind for the traceability table (6.9) |
| Stage 1 7.7 | Add: a vulnerability found in a dependency is reported upstream within 7 days† (Art. 13(6)); dependency advisories come through the RustSec database |
| Stage 1 7.8 | Nightly and Beta are Art. 4(3) testing builds (notice, opt-in, per-build expiry); security-only releases on the stable line, on demand; one CycloneDX SBOM per artifact (6.8); the conformity set and ACR generated per release; security updates archived for ten years with timestamped signatures and an archive manifest |
| Stage 1 8.1 (M1) | One shared header owned by FI-11, with this spec's fields and the build manifest of 9.4 |
| Stage 1 8.3 | A row for security response and compliance: **1–2.7 engineers†** (6.19); plus 2–3.5† if CO-D5 (a) is chosen |
| Stage 1 11, D-S4 | **Owner-approved, 25 October 2026 (CO-D6):** the per-crash prompt pre-selects nothing; "send automatically" is an opt-in setting, off by default. This amends D-S4, an owner decision, by the owner's own decision |
| Stage 1 13.2 "Security response" | Replace with a pointer to this spec |
| Stage 1 13.2 "Privacy and legal" | Privacy notice per service; DPIA for crash ingestion; CO-D17's hosting rule; no terms for the browser binary (CO-D14); export control: "notification only for non-standard cryptography; none is planned; re-decided if a closed module ships" |
| Stage 1 13.2 "Updates" | Automatic security updates on by default with an opt-out and a bounded postpone (Annex I Part I(2)(c)); Linux packages per CO-D19 |
| **Shell** 6.4 | Cited as CO-D6's precedent ("no control is pre-selected"); no change |
| Shell 6.9 | Under "Updates and recovery": "Install security updates automatically", "Apply emergency security switches", "Send crash reports automatically" (off); a change of update channel shows the Art. 4(3) notice and needs a confirmation; the ledger's security-event switch; "Delete everything in this profile" and "Delete all Izcal data for this user on this device" |
| Shell 6.11 | The opt-out with Annex II's instructions; postpone limits (7 days†; emergency: next start); the security notice for exploited builds; the withdrawn-build notice |
| Shell 6.14 | About shows the per-platform support end date, the simplified declaration and the security information link; testing builds show the Art. 4(3) notice in About and as a first-run screen; `about:licences` and `about:credits` are generated from the SBOM; `about:crashes` lists sent reports with their bytes and "delete from server" |
| Shell SH-D11 and 6.4's ledger | The ledger gains the security event classes of 6.10 under its existing retention, keys, clearing and private-window rules |
| Shell SH-D23 | Testing builds add one dismissible notice screen before the four onboarding screens |
| Shell SH-D29 | Owner-approved, 25 October 2026 (CO-D6): "Crash reports are opt-out per crash" becomes: crash reports are sent on an affirmative act or an opt-in setting (CO-D6) |
| Shell (new) | The end-of-support notice (6.2) |
| **Graphics** GR-D21 | A platform drop records the support statements it ends (CO-D5 (c)) |
| **Networking** NW-D17 | Crash uploads, telemetry and the deletion endpoint are named on the published endpoint list as user-specific requests that use the OHTTP relay; update-payload fetches are aggregated to counts at the CDN with raw logs deleted ≤ 24 h†; Q-NW-6's fallback restates CO-18 (c) |
| **Storage** | No rule change (SG-11 already covers corruption); the deletion inventory of 6.10 lists what the storage service removes |
| **Content blocking** CB-D18 | Breakage reports appear in this spec's inventory and privacy notices |
| **Phishing protection** 6.6 | Reports and appeals data appear in this spec's inventory and privacy notices |
| **Extensions** EX-D10 and the **app host** | The Stage 2 directory is assessed for the DSA, the GDPR and its CRA role before it opens (Q-CO-28) |
| **Shell UI design** UI-1 | The UAAG 2.0 statement is published with the generated ACR (6.10a) |
| **Feature inventory audit** FI-1 | Its path becomes this file (the audit named `reports/Izcal compliance.md`) |
| FI-2 (distribution, being written) | Inputs: per-platform support statements, testing-build rules, the security-only line, on-demand timing, the ten-year archive, 24 h† staged rollout, CO-D19 |
| FI-3 (enterprise policy) | Policy keys for turning automatic updates off and for the ledger's security events; the coarse policy flags of 9.4 |
| FI-11 (crash and telemetry) | Owns the one recording and crash-report header, with 9.4's fields and the coarse policy flags for crash reports; OHTTP upload, per-record deletion tokens, CO-D6, under-18 suppression (CO-21), retention, DPIA |
| FI-12 (privacy spec) | Owns the browser privacy notice's wording (CO-29) |
| FI-35 (sync) | Arts. 15, 17, 20 for account data; new-device notice (Annex I Part I(2)(d)); EU hosting; breach process |
| FI-33 (Android) | Play age signals; Texas and Utah app-store duties; the Android EEA choice-screen criteria |

## 10. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Counsel reads Art. 4(3) narrowly, so Nightly cannot be distributed in the EU as planned | Medium | Testing loses its widest audience | CO-D3's reversal: Beta conforms; Nightly time-boxed or source-only |
| A paid edition under D-S8, or a closed module under D-S7, loses Art. 32(5) | Depends on D-S8 and D-S7 | A notified body for that edition or build | CO-D2 surfaces the cost before those decisions |
| Counsel rejects a per-platform support period shorter than five years | Medium | 2–3.5 engineers† of legacy branches | CO-D5's reversal is costed; S1 8.3 grows |
| The officers cannot meet 24-hour clocks over holidays | Medium | A late notification carries the top fine tier | The incident-response retainer; CO-2's 4-hour† bound; split roles (6.19) |
| AI-generated report volume swamps intake | High | Real reports wait | Reproducer rule; templated closure; no bounty until staffed |
| The CVE program's governance changes again | Medium | IDs delayed | Izcal IDs are primary; EUVD and GCVE aliases |
| ENISA's platform changes format or adds an API | Medium | Templates drift | Templates generated from the record, versioned with it |
| The crash sample shrinks under CO-D6 | High | Fewer crash signatures | Opt-in automatic sending; the recovery window's send action (SH 6.14); counsel may restore the default |
| No CDN offers the log control CO-D17 needs at acceptable cost | Medium | No uptake counts; IPs held by the CDN under its own terms | Recorded per channel under CO-18 (a); an EU CDN or Izcal-run origin for payloads |
| The witnessed log or the security repository is compromised | Low | False clocks; leaked unreleased fixes | Machine stamps with receipts held by reporters; scheduled commitments; SEC-27 controls; severe-incident runbook |
| Annex II translation outruns Stage 1's English-only UI | Medium | Distribution limited to languages covered | English and the establishment's language first; FI-6 |

## 11. Open questions and flagged claims

### 11.1 Open questions

The questions marked **legal** go in one brief to counsel before the first EU distribution (FI-D11). None is guessed above: where the design needs an answer first, it takes the conservative option and names the reversal. "Compliance lead" is the compliance engineer of 6.19, who prepares the brief; the owner commissions counsel.

| ID | Question | Kind | Owner | Answered by |
|---|---|---|---|---|
| **Q-CO-1** | Where is the Izcal entity established, and does a non-EU manufacturer that distributes only by download need an EU authorised representative (CRA Art. 14(7); Regulation (EU) 2019/1020 as amended by CRA Art. 66)? Does Art. 64(10)'s small-enterprise exemption, which refers to paragraphs 3 to 9, reach Art. 14 fines under paragraph 2? | Legal; owner | Owner | Before the first EU distribution |
| **Q-CO-2** | For the chosen D-S8 option, is Izcal's supply a "commercial activity" (recitals 15, 18)? Is a default-search revenue share "monetising other services"? Does a paid edition make the free edition commercial? How are costs measured for donations? Could a not-for-profit entity be a steward rather than a manufacturer? Does opt-in-only telemetry keep the personal-data sign closed? (Q-FI-1) | Legal | Owner | With D-S8 |
| **Q-CO-3** | Is a permanent Nightly or Beta channel "made available only for a limited period required for testing purposes" (Art. 4(3)) if each build expires? (Q-FI-1) | Legal | Compliance lead | Before 11 December 2027 |
| **Q-CO-4** | Does Art. 14 apply to a vulnerability present only in a testing build? | Legal | Compliance lead | Before the first EU testing build |
| **Q-CO-5** | EN 304 617: when is it adopted and cited, and which clauses does Izcal's evidence not yet cover? (Q-FI-2's remainder) | Black-box | Compliance lead | On publication |
| **Q-CO-6** | For a browser released every few weeks, what is "the product" whose support period runs, and is each feature release a substantial modification (recital 39)? Does Art. 13(10) apply if feature releases are not substantial modifications? What does Commission guidance C(2026) 5252 say? | Legal | Compliance lead | Before the first EU Release |
| **Q-CO-7** | May the support period stated at release for an OS version be shorter than five years when that OS's vendor support ends sooner (Art. 13(8)'s "availability of the operating environment")? If not, must Izcal keep security updates for the last compatible release (Art. 13(10))? | Legal | Compliance lead | Before the first EU Release |
| **Q-CO-8** | For Linux packages updated by the system package manager (FI-D10), is CO-D19 enough for Annex I Part I(2)(c)'s "where applicable"? | Legal; FI-2 | FI-2 owner | With FI-2 |
| **Q-CO-9** | Under Art. 32(5), must the full risk assessment, test reports and SBOM be public (Annex VII(2)(b) and (8), recital 77), or may sensitive parts be withheld? Does a module A self-assessment under 32(5) need any notified body involvement? | Legal | Compliance lead | Before the first EU Release |
| **Q-CO-10** | Which languages must Annex II information and the declaration use (Arts. 13(18), 28(2))? | Legal | Compliance lead | With CO-D1 |
| **Q-CO-11** | Should Izcal register as a GCVE numbering authority, and when? | Operations | Security officers | After CNA status |
| **Q-CO-12** | When will ITI publish a VPAT edition for EN 301 549 v4.1.1, and when is v4.1.1 cited in the Official Journal? Which conformance levels does VPAT 2.5Rev allow for untested A and AA criteria? | Black-box | Compliance lead | On publication |
| **Q-CO-13** | Confirm the legal basis for each service in 6.13, including legitimate interest for security intake and phishing appeals | Legal | Data-protection lead | Before the first public binary |
| **Q-CO-14** | Does ePrivacy Art. 5(3)'s "strictly necessary" exemption cover crash upload and update checks? How does the CRA's automatic-update default fit Art. 5(3)? Is the 30-day minidump retention defensible, and does crash ingestion need a DPIA? | Legal | Data-protection lead | Before the first EU binary |
| **Q-CO-15** | Is the Izcal entity a CCPA "business"? Does the UK Children's Code reach Izcal's update or crash services? | Legal | Data-protection lead | With CO-D1 |
| **Q-CO-16** | Was California AB 1856 signed, and does its open-licence exemption cover an MIT browser whose official binaries carry a trademark? If AB 1043 applies, is a request at each launch required? What does "actual knowledge" oblige, and does an age signal make COPPA apply to crash reports? | Legal | Compliance lead | Before 1 January 2027 |
| **Q-CO-17** | What does EU consumer law require of a free MIT browser binary beyond the licence's disclaimer? | Legal | Compliance lead | Before the first public binary |
| **Q-CO-18** | Does a distribution that patches Izcal and ships it under the name become a manufacturer (Arts. 21–22), and what should the trademark policy say? Is the name Izcal clear in classes 9 and 42? | Legal | Owner | With D-S9's clearance search |
| **Q-CO-19** | Is the ML-KEM hybrid "standard cryptography" under 15 CFR 772.1, and do US sanctions limit free downloads to sanctioned countries? | Legal | Compliance lead | Before the first public binary |
| **Q-CO-20** | Which of the six formats' schemas are designated normative by their owners (input to CO-D20)? | Black-box | Compliance lead | Before the generators are built |
| **Q-CO-21** | Does the UK PSTI regime reach a downloaded browser (Schedule 3 excepts desktops and laptops; software is not mentioned)? | Legal | Compliance lead | Before the first UK distribution |
| **Q-CO-22** | Does the PLD's free-software exclusion (Art. 2(2)) cover an MIT browser that accepts donations, and does Art. 11(2)(c) reach releases past their support period? | Legal | Compliance lead | With Q-CO-2 |
| **Q-CO-23** | Do update servers, crash ingestion, telemetry, push and sync count as the product's "remote data processing" (Art. 3(2)), and may aggregate counts be used for Art. 14's Member-State list? | Legal | Compliance lead | Before the first EU distribution |
| **Q-CO-24** | At what size would sync or an Izcal-run DNS resolver bring Izcal under NIS2? | Legal | Compliance lead | Stage 2 |
| **Q-CO-25** | If Izcal's own build ever includes a licensed closed module (D-S7), does the product still "qualify as free and open-source software" for Art. 32(5)? | Legal | Owner | Before such a build |
| **Q-CO-26** | When does a manufacturer "become aware" under Art. 14: at receipt of a claim with evidence, or at its own positive assessment? | Legal | Compliance lead | Before the first EU distribution |
| **Q-CO-27** | Does Art. 14 apply to a vulnerability in an OS component a product relies on (the protected-media engine, an OS decoder, a driver) when it is exploited through the product? | Legal | Compliance lead | Before the first EU distribution |
| **Q-CO-28** | Is the Stage 2 extension directory a hosting service or an online platform under the DSA, and what is Izcal's CRA role towards the third-party extensions it distributes? | Legal | Owner | Before the directory opens |

### 11.2 Claims that are single-source, unverified or conflicting

- **CRA text.** The CRA strand read a reproduction; the fact-check then read the official Official Journal text through the Publications Office (EUR-Lex itself served bot challenges) and confirmed every provision this spec relies on, with the corrections applied ([factcheck][FC]).
- **EN 304 617's status** rests on search snippets of ETSI document titles and secondary listings that give different enquiry dates; the final draft needs a login (Medium).
- **EN 301 549 v4.1.1's publication** in September 2026 and its expected citation date rest on secondary sources; the day is unconfirmed (Medium).
- **California AB 1856's final status** is unverified (Low); its content comes from press and advocacy reports.
- **The Windows age-signal API** is reported by a secondary source only (Low); Apple's API reportedly returns no value on macOS (Medium, developer forum).
- **Chrome's 2026 bounty changes** and **Firefox's two-week cadence** rest on news coverage (Medium to Low).
- **ENISA's Root date** (November 2025) is from ENISA's announcement and a repost dated 26 November 2025 (Medium on the day).
- **Firefox's ACR**: the history strand read a Firefox 124 report of April 2024 on Mozilla's wiki (High); the data strand did not find one. The history strand is used.
- **Commission guidance C(2026) 5252**: only its publication is confirmed; its annex was not read (Low on content).
- **The Debian Iceweasel history** is from memory (Low) and is used only as an illustration.
- ***Planet49*** was not re-read; its holding is well known (Medium).
- **The workload estimate of 6.19** rests on an assumed report rate; the first two quarters of intake replace it with measured numbers.

## Appendix A. Phase 8 disposition

### A.1 Fact-check (`factcheck.md`): 0 blockers, 5 major, 23 minor

| ID | Disposition |
|---|---|
| F1 Art. 69(2) | Applied: Section 3 table, 6.1, CO-D3, CO-15, 9.1, 11.2 |
| F2 CNA contacts | Applied: Section 3, 6.6, CO-25 (two contacts as Izcal's own rule, †) |
| F3 CNA advisory location, no assignment-policy rule | Applied: Section 3, 6.6 (the assignment rule is Izcal's choice) |
| F4 CSAF trusted provider | Applied: 6.4 (requirements 1–7, 8, 9, 11–14, 18–20), CO-6 |
| F5 Misattributed quote | Applied: 2.1 |
| F6 Art. 30(1) | Applied: 6.9 |
| F7 CNA Rules 4.2.14, 4.2.16.6 | Applied: 6.5 |
| F8 Art. 4(3) "shall not prevent" | Applied: 6.1 |
| F9 Art. 13(10) condition | Applied: 6.2, Q-CO-6 |
| F10 Part II(4) | Applied: 6.4 |
| F11 Art. 64(10) | Applied: 6.7, Q-CO-1 |
| F12 Part I(2)(c) qualifiers | Applied: 6.3, Q-CO-8 |
| F13 SBOM publication | Applied: 6.8 (published by choice), Q-CO-9 |
| F14 Art. 13(4) hook | Applied: 6.3 |
| F15 PLD row | Applied: Section 3 table |
| F16 security.txt `CSAF` field | Applied: 6.5 |
| F17 CVE federation wording | Applied: 2.1 |
| F18 ENISA Root source | Applied: Section 3, 11.2 |
| F19 Data Portability citations | Applied: 6.15 |
| F20 D-S4 citation | Applied: Section 1, 6.13 |
| F21 SEC-D2 size | Applied: Section 1, 6.19 |
| F22 UPD-R4 scope | Applied: 6.3, 9.5 |
| F23 DATA row numbers | Applied: 6.14 |
| F24 cargo-deny and OSV | Applied: 6.5 |
| F25 EN 304 617 dates | Applied: Section 3, 11.2 |
| F26 EN 301 549 day | Applied: Section 3, 11.2 |
| F27 Heartbleed link | Applied: 2.1 |
| F28 Research note row 7.6 | No spec change; the note's row stands corrected by this record |

### A.2 Design critique (`critique.md`): 3 blockers, 16 major, 19 minor

| ID | Disposition |
|---|---|
| C1 Art. 14 clock and witnessing | Applied: 6.4 (awareness field, machine stamps with receipts, typed clocks flagged, salted commitments on a fixed schedule), CO-D21, Q-CO-26, CO-1, CO-7 |
| C2 Hosting | Applied: new 6.12 hosting table; the security repository and record on EU hosting; GitHub private reporting dropped as an intake channel; CO-D17 restated for stored data; CO-18 conditional on Q-NW-6 and the CDN; SEC 6.16 amendment |
| C3 OS-drop cost | Applied: 6.2 costs three options; CO-D5 recommends (c), with (a) as the reversal and its cost in 6.19 and the S1 8.3 row |
| C4 Release gates | Applied: CO-9 exempts security-only and emergency releases and counts only fixable issues; CO-3 narrowed to critical, high and exploited |
| C5 Bound inversion | Applied: one bound table in 6.4; SEC-25's critical bound tightened to 7 days† (amendment) |
| C6 Uptake measurement | Applied in a different form: no client signal is added; CO-24 estimates uptake from fixed-version downloads over daily TUF timestamp fetches, with a 1,000† minimum cell |
| C7 Disclosure classes | Applied: field classes in 6.4, joint regeneration, CO-26 canary suite, a crate rule |
| C8 Reproducers and retention | Applied: public regression tests are Izcal-written reductions (6.5); one retention table (6.18) |
| C9 Security log | Applied: the prompt ledger gains security event classes under its rules, instead of a broker log (6.10) |
| C10 M1 header | Applied: 9.4 redesigned (in 0.3, restated as a contribution to FI-11's single header) around a published build manifest; no SBOM digest in the executable; profile ID replaced on export; coarse policy flags in crash reports; status column relabelled; record schema version removed from M1 |
| C11 Deletion tokens | Applied: a token per record; server deletion first in the deletion actions; `about:crashes` keeps the bytes sent for access; the endpoint joins NW-D17 |
| C12 Linux automatic updates | Applied: CO-D19 and CO-5's scope |
| C13 D-S7 | Applied: 6.1, 6.8, 6.17, CO-D2, Q-CO-25 |
| C14 OS-owned components | Applied: record class in 6.4, Q-CO-27, UPD-R5 "protected-media mode off" amendment |
| C15 Unowned duties | Applied: new 6.11 (withdrawal, authority requests, cessation); Art. 13(5) row in the traceability table; CO-27, CO-28 |
| C16 Missing services | Applied: 6.13 inventory rows (breakage reports, phishing appeals, root program, extension directory); 6.14 qualified; Q-CO-28; 9.5 rows for CB, phishing, EX and AH, FI-3, FI-12 |
| C17 Staffing | Applied: 6.19 workload estimate, retainer, split roles, S1 8.3 row |
| C18 SEC-D4 fallback | Applied: 9.5 amendment |
| C19 Q-CO-20 | Applied as owner decision CO-D20 over all six formats, with the distinction from D-S12 and DT-D12 recorded in Section 7; Q-CO-20 now asks only which schemas are normative |
| C20 CO-4 failure action | Applied: a leak is treated as public disclosure and moves to the exploited-risk path |
| C21 Legal readings as conclusions | Applied: 6.1, 6.2, 6.14 reworded as readings with their questions |
| C22 FI-1 rows without targets | Applied: CO-29 (privacy notices), CO-6 extended (scale and enterprise notes); Appendix B maps every row |
| C23 Missing shell and S1 rows | Applied: 9.5 rows for SH-D23, SH-D29, shell 6.4 and 6.9, S1 §1 and §6 crash rows; the testing notice is a dismissible, recorded screen; channel changes show the notice |
| C24 SEC-D10 | Applied: 9.5 row |
| C25 Gate tier and crate rules | Applied: Release gate tier; crate rules G260–G262 with checks (6.0), in the range the coordinator assigned |
| C26 Storage duplication and deletion inventory | Applied: SG-11 cited; deletion inventory and "for this user" (6.10) |
| C27 Archive verifiability | Applied: timestamped signatures, archive manifest under an archival key, oldest build tested (6.2, CO-12) |
| C28 Declarations | Applied: one declaration per release line with a named signatory step (6.9, CO-10) |
| C29 SBOM publication and formats | Applied: published by choice; CycloneDX only, SPDX on request (6.8, CO-D11) |
| C30 Distro notice on the emergency path | Applied: 6.5 |
| C31 AI triage | Applied: 6.5 follows SEC-27 |
| C32 Dependencies list | Applied: Section 7 table |
| C33 Weekly slot | Applied in a different form: security releases are on demand within bounds (CO-D18); a weekly slot is the reversal |
| C34 Rationales and triggers | Applied: rationales after Section 8; reversal triggers for CO-D9, CO-D17 and the SBOM row |
| C35 Notice data source, UPD-R5 under opt-out | Applied: TUF targets fields (6.2, 6.4, UPD-R1 amendment); a separate emergency-switch setting (6.3) |
| C36 Age-signal preconditions | Applied: OS UI measured first, feedback suppressed, CO-21 keyed to each OS's first binary (6.14) |
| C37 VPAT "Not Evaluated" | Applied in a different form: CO-16's failure action uses the template's lower level with a remark, and Q-CO-12 asks which levels the template allows, since neither reviewer verified the template's rule |
| C38 Question owners | Applied: owners named in 11.1 |

## Appendix B. How each FI-1 row closes

| Row | Closed by |
|---|---|
| PO-3.15 End-of-support handling | 6.2; CO-11; CO-D4, CO-D5 |
| PO-14.6 Accessibility conformance report | 6.10a (ACR); CO-16 |
| PO-15.1 Privacy notice and terms of use | 6.13, 6.16; CO-29; CO-D14 |
| PO-15.3 GDPR access and export | 6.13 (`about:crashes`, `izcal-export`); CO-19 |
| PO-15.4 Data deletion | 6.10, 6.13; CO-19 |
| PO-15.5 DMA choice screens | 6.15: no duty; Android eligibility handed to FI-33 and FI-2 (decision) |
| PO-15.6 DMA data portability | 6.15; CO-D16 (decision) |
| PO-15.7 Age assurance and age signals | 6.14; CO-21; CO-D13 |
| PO-15.9 EU Cyber Resilience Act | 6.1–6.11; CO-1 to CO-12, CO-15, CO-26 to CO-28 |
| PO-15.10 EAA exposure | 6.10a: indirect, no duty (record) |
| PO-15.11 Trademark and brand rules | 6.16; CO-D15; milestone before the first public binary |
| PO-16.3 CVE assignment | 6.6; CO-6, CO-25; CO-D7 |
| PO-16.6 Severity scale | 6.4; CO-6; CO-D9 |
| PO-16.10 Enterprise security notes | 6.4's projections; CO-6 |

[FI]: Izcal%20feature%20inventory%20audit.md
[FIFC]: ../research_notes/Izcal%20feature%20inventory%20audit/factcheck.md
[SEC]: Izcal%20security%20scope.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[SH]: Izcal%20shell.md
[UI]: HGBA%20shell%20UI%20design.md
[NW]: Izcal%20networking.md
[GR]: Izcal%20graphics%20and%20rendering.md
[SG]: Izcal%20storage.md
[H2]: HGBA%202%20browser%20architecture%20spec.md
[CRA]: ../research_notes/Izcal%20compliance%20and%20security%20response/cra_eu_product_law.md
[OPS]: ../research_notes/Izcal%20compliance%20and%20security%20response/security_response_operations.md
[HIST]: ../research_notes/Izcal%20compliance%20and%20security%20response/history_postmortems.md
[DATA]: ../research_notes/Izcal%20compliance%20and%20security%20response/data_access_platform_law.md
[FC]: ../research_notes/Izcal%20compliance%20and%20security%20response/factcheck.md
[CRA-EL]: https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng
[BSI]: https://www.bsi.bund.de/SharedDocs/Downloads/EN/BSI/Publications/TechGuidelines/TR03183/BSI-TR-03183-2_v2_1_0.pdf?__blob=publicationFile
[OASIS]: https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html
[CNA]: https://www.cve.org/ResourcesSupport/AllResources/CNARules
[SRP]: https://www.enisa.europa.eu/topics/product-security/single-reporting-platform-srp/frequently-asked-questions
