# Fact-check: Izcal security scope, draft 0.1

*Independent fact-check of `reports/Izcal security scope.md` (draft 0.1, 18 October 2026), run 18 October 2026. Sources were fetched on that date. RustSec counts come from a shallow clone of `rustsec/advisory-db` taken that day; Deno and Wasmtime counts come from `gh api …/security-advisories`; the Project Zero tally comes from the sheet's per-year CSV export. Repository state was read from the working trees: the project on `main` at `781ac97`, and the fork at `m0/rusty_v8`, which is checked out on `ci/security-hardening` (PR #2, **open, not merged** into `hgba`; `izcal/hgba` is at `8ba5648`). No spec or other file was edited.*

## Summary

| Area | Checked | Confirmed | Partly right | Wrong | Unsupported by cited source | Source unreachable |
|---|---|---|---|---|---|---|
| 1. External facts | 90 | 54 | 30 | 2 | 4 | 0 |
| 2. Internal facts (Izcal documents and repositories) | 67 | 53 | 7 | 6 | 1 | — |
| 3. Arithmetic and consistency | 17 | 10 | 1 | 1 | — | — (5 inconsistencies) |

"Partly right" means the source supports the core of the claim but a number, quote, scope or attribution in the spec is off. No source was unreachable in the end, though several needed a mirror or an API: googlesource through its GitHub mirror, NVD through its API, Apple through its JSON endpoint, SquareX through a reader.

**The spec's overall direction stands.** The high-impact external numbers behind the targets are confirmed: io_uring's 60% in kCTF, Mozilla's 21-hour and 25-hour fixes, MiraclePtr's 57% of 168, the TAG 64%, App-Bound Encryption, immutable releases on 28 October 2025, SLSA L3 through reusable workflows, the TUF attack list, cargo-vet's criteria, the AI-found counts of 22 and 271, Deno's 40 advisories, Wasmtime's 51, the SDK 27 quote, and RustSec's 75/46 and 256. The largest problems are three things: a repository-state claim that describes an unmerged PR as fixed, one unsupported generalisation about logic bugs, and two places where the spec says other documents lack something they already contain.

---

## Required corrections, ordered by impact

1. **Fork "current state" describes PR #2, not `hgba`** (Section 6.16, the "Current state" table rows 1–3; Section 9.1, M0 "(done)"; Section 11.3 item 8). Three fork rows marked **Fixed** are true only on the unmerged branch `ci/security-hardening`. On `hgba`:
   - `izcal-v8-build.yml` has 8 tag-pinned `uses:` (`actions/checkout@v4`, `softprops/action-gh-release@v2` and others), and `izcal-checks.yml` has 2.
   - `sccache` and `actions/cache` run on tag builds.
   - `izcal-checks.yml` downloads gitleaks with no hash check and runs `bash <(curl …download-actionlint.bash)`.

   The project side (`checks.yml`, `tools/ci-install-tools.sh`, zizmor in `checks.sh`) **does** match `main`. Fix: say "fixed in the project (`main`); fixed for the fork in PR #2, not yet merged". In 9.1 M0, change SEC-13 (d) and (e) from "done" to "pending PR #2 merge". The open rows (archive hash, binding file, runners, unpinned `checks.sh`, `contents: write`) are true on both branches.
2. **"The logic bugs were all sandbox escapes" is unsupported** (Section 3.1, bullet 3; it feeds the intro and Section 3.2's framing). The Project Zero sheet labels only CVE-2025-2783 a sandbox escape. It lists CVE-2022-3075 as "Insufficient data validation in Mojo" and CVE-2022-2856 as "Insufficient validation of untrusted input in Intents", with no escape analysis. Related:
   - "21 of 37 (57%) V8" holds only if CVE-2024-0519, listed simply as "Out-of-bounds memory access", is counted as V8. State that.
   - Section 4.4 row 1, "Every 2025 escape crossed the broker (Section 3.2)", contradicts Section 3.2 itself. GTIG says the 2025 escapes went through ART and the GPU user-mode drivers, so they did not cross the browser broker. Only CVE-2025-2783 did.
   - Section 3.2's GTIG list omits CVE-2025-2783 (GTIG files it as an OS-based Windows escape). Metal (CVE-2025-14174) belongs to an Apple chain, not "2025's Android chains" (Section 6.9).
3. **SEC-12 / SEC-D3 overstate the gap in Stage 1** (Sections 3.5 "C codecs in Stage 1", 6.4 S-5 row, 6.10, 9.3 SEC-D3). The Stage 1 design already puts C codecs and PDF in sandboxed utility processes:
   - Section 1's process map: Utility does "Media and image decode (RLBox-style for C libraries)… PDF parsing and rendering".
   - 13.2 Codecs: "libvpx in the library sandbox".
   - Section 6 PDF viewer: "a Rust PDF renderer, or a C/C++ one in the RLBox-style library sandbox, running in a utility process".

   What is missing is a **target and gate** (S-5 is a Stage 2 *target*). The design is already there. Reword SEC-D3 as "make Stage 1's existing library-sandbox design a Stage 1 target", and drop "Stage 1 ships libvpx, dav1d and OpenH264 before S-5 applies" as stated.
4. **"No measurement, no failure action" for V8 patch latency is wrong** (Section 3.5 row 1, Section 6.8 para 1, Section 11.3 item 1). Stage 1 7.7 already defines a lag metric, "Days behind Chrome stable V8 security fixes", with an escalation at ">2 weeks lag behind upstream". HGBA-2 S-4e is a ship-blocker target with a defined fallback: the backport rule when the rebase misses the window. The real gap is narrower. There is no **latency target by severity**, no per-advisory publication, and no failure action when the *fix itself* is late. Restate it that way.
5. **"Jitless costs 1.5–10×" is wrong** (Section 6.18, Windows renderer JIT row). On v8.dev/blog/sandbox, 1.5–10× is the cost of "using a memory-safe language… and removing JIT compilers". The jitless post measures about 40% slower on Speedometer 2 and about 80% on the Web Tooling Benchmark. "About half of exploited V8 bugs [were in compilers]" is confirmed. Also, "not yet" (Section 2.1, Section 6.8) is not the post's wording. It says there are "a number of issues to resolve before it becomes a strong security boundary". Paraphrase it, or quote that.
6. **Rust advisory tallies** (Section 3.3):
   - quinn-proto has **10** advisories, of which 8 are from 2026. "8 advisories" is right only for 2026.
   - rustls had **5** 2026 advisories, not 3. The three named are real, plus RUSTSEC-2026-0285 and -0346, none of them memory corruption.
   - The RustSec yearly baseline is **73–81** (2023: 81, 2024: 78, 2025: 73), not 73–82. 2026 is 256 ✓.
   - Wasmtime's "4 a year" had an exception: 2022 had 8.
7. **Cookie encryption at rest has a broken reference and no decision** (Section 6.5, Storage service "Open": "Encryption at rest with an OS-bound key for cookies (Section 6.14)"). Section 6.14 says nothing about it, and no other section decides it. The spec stresses ATK-7 and the App-Bound lesson, and it gives *recordings* an OS-bound key (SEC-21 (c)) but leaves Izcal's own cookie store open. Add a target or a SEC-D decision, or fix the reference.
8. **Retention is both decided and pending** (Section 6.13 "Retention", Section 6.15 "Crash reports", SEC-21 (c), SEC-D9, Section 8 closing line). SEC-21 (c) already makes 30 days† for saved recordings a target, and 6.15 states 30 days† for dumps as fact. SEC-D9 lists both as an owner decision, and Section 8 says the crash-report target waits for SEC-D9. Pick one. Also, Section 6.5 says every threat-model file is created "at M1", but Section 9.1 M1 creates only broker, renderer and network service.
9. **Attributions to other Izcal documents that are off:**
   - IPC-R13, "nested sync is forbidden (Stage 1 4.1)": Stage 1 4.1 has the reviewed sync allowlist but no nested-sync rule. Mark the rule as new.
   - Section 6.7, "the fork's rule (changes written against V8's public interfaces, S-4e)": that text is in HGBA-2 M-1 (Section 2), Stage 1 D-S10 and `CONTRIBUTING.md`, not in S-4e.
   - Section 6.4, D-S12 described as "IIP with adapters": D-S12 is **compatibility oracles** (black-box observations for `hg-cdp` and HAR). IIP with adapters is the 18 October update of D-S11.
   - Section 6.4, the "`security` check" row lists gitleaks, but gitleaks is the separate `secrets` check.
   - G12 (Section 6.5, Replay host) is a *proposed* DevTools-spec rule (DevTools 9.5, "Add Section 6.1's crates and rules G10–G12"), not yet in Stage 1.
10. **Citations that do not carry the claim, and quotes that need tightening:**
    - Storm-0558: cite the CSRB PDF, not the landing page.
    - "TOTP was phished in chalk/debug": the Endor Labs page does not say this; find another source.
    - tj-actions "moved tags": the CISA alert does not mention tags.
    - libwebp "four platforms": Isosceles does not say it; the TAG 2023 PDF does (Chrome, Safari, Android, Firefox), so cite that.
    - CVE-2018-5158 needs its own source.
    - WebView2 "keeps its loaded runtime version": this is on the Distribution page, not the enterprise page.
    - Symantec distrust: not in the Google 2024 post.
    - Pwn2Own "no team entered Chrome": cite ZDI's full-schedule page.
    - "Mozilla found the same pattern within two days": the MFSA does not give the timing.
    - arXiv 2510.09023: ">90% for most" of the 12, not all.
    - LayerX: "up to 90% more exposed than traditional browsers"; "to phishing" is not inside the quote.
    - Unit 42: "malicious extensions with basic permissions", not "a basic extension".
    - Apple MIE: Apple shows "six of the real-world exploit chains that we evaluated", so "6 of 6 tested" overstates it.
    - Edge SDSM: names neither W^X nor "vendors ship JIT-off modes".
    - Site isolation's 10–13% is the desktop figure (Android is about 3–5%); on Android below 2 GB it is "not yet supported" rather than switched off.
    - CET's Intel 11th Gen and Zen 3 are examples, and the "off in the renderer" state is Chrome 90 in 2021; date it.
    - Mozilla's IPC post says the *lack of a cheap reset* breaks determinism; the spec reverses this.
    - arrayref: the payload was in `proc-macro1`, which the three republished crates depended on.
    - EverParse emits Rust only through EverCDDL (3D emits C).
    - Fuchsia's rights page says rights "can be reduced", not that they "can only shrink".
    - GitHub Security Lab never mentions Rust, so "Rust removes the lifetime bugs" is the spec's own inference.

    Also worth folding in, though the spec is not wrong here: on the OpenSSH list, the SDK 27 "these profiles" are the **built-in named `sandbox_init` profiles** (kSBXProfile\*, including PureComputation). That supports Q-SEC-2's framing: custom profiles through `sandbox_init_with_parameters` are not obviously covered. It also bears on the Section 6.7 table's macOS "Pure-computation profile" for utility processes, which is exactly one of the killed profiles.

---

## 1. External facts

Verdicts: **C** confirmed, **P** partly right, **W** wrong, **U** unsupported by the cited source.

| # | Claim (short) | Spec location | Verdict | Evidence or correct fact | Source |
|---|---|---|---|---|---|
| A1 | CSVs behind 64% of browser and mobile 0-days in 2023 | §1 | C | PDF: "64% of all mobile and browser vulnerabilities exploited (24 of 37)"; the blog says "over 60 percent" | TAG 2023 review + PDF |
| A2 | libwebp and libvpx each hit four platforms | §6.10 | P | Counts right (4863: Chrome, Safari, Android, Firefox; 5217: Chrome, Firefox, iOS, Android); the PDF never says "libwebp" | TAG 2023 PDF |
| A3 | 67.4% of bugs and 70.4% of code-execution bugs in the rendering engine | §2.1 | P | Numbers right (87/129, 38/54), but they are IE, Firefox and Safari bugs that "would have occurred in the rendering engine" of Chromium | Barth et al. 2008 |
| A4 | Site isolation 10–13% memory; off below 2 GB on Android | §2.1 | P | 10–13% is desktop with all sites isolated; Android partial is about 3–5%; "not yet supported" below 2 GB | Google 2018; chromium.org |
| A5 | 70% of Chrome's severe bugs are memory safety | §2.1 | C | "around 70% of our serious security bugs" | chromium.org |
| A6 | MiraclePtr mitigated 57% of 168 UAFs, not in renderers; memory-cost conflict | §2.1, §11.2 | C | Both overheads confirmed: 2022 +4.5–6.5% Windows; 2024 +5.5–8% desktop browser process | Google 2024/01, 2022/09 |
| A7 | V8 sandbox "not yet" a boundary; "had to get past"; jitless 1.5–10× and half of exploited bugs | §2.1, §6.8, §6.18 | **W** | "Not yet" not verbatim; "about half" ✓; **1.5–10× is for a memory-safe language plus no JIT**; jitless is about 40% (Speedometer 2) and 80% (Web Tooling) slower | v8.dev/blog/sandbox; /jitless |
| A8 | CET off in Chrome's renderer; needs 20H1 and Intel 11th Gen or Zen 3 | §2.1, §6.7 | C | Confirmed for Chrome 90 (2021); the CPUs are examples ("such as") | Google 2021/05 |
| A9 | Fontations 39 fuzzer bugs, none security-critical | §2.1, §6.10 | C | Verbatim | developer.chrome.com |
| A10 | Android memory safety below 20% in 2025 | §2.1 | C | "falling below 20%… for the first time" | blog.google, Nov 2025 |
| A11 | Chrome agent origin sets and critic; only the writable set enforced at rollout | §2.1, §5 | C | "just tracks the read-writeable set" | Google 2025/12 |
| A12 | Browser 0-days 17 → 11 → <10% of 90; GTIG undecided on cause | §3.1 | C | All confirmed; both explanations offered | GTIG 2024, 2025 |
| A13 | 2025 escapes not generic; via ART, Adreno/Mali, Metal through ANGLE; ANGLE CVEs | §3.2, §6.1, §6.9 | P | "none were generic" ✓; omits CVE-2025-2783 (Windows ipcz); Metal 14174 is Apple; Adreno 27038 not tied to ANGLE; 6558 is Mali, fixed in ANGLE | GTIG 2025 |
| A14 | 25% (2020), over 40% (2022) variants | §3.1, §6.17 | C | 2022: 17 of 41 | P0 2021; TAG 2022 |
| A15 | Infostealers use Chrome remote debugging; Chrome 136 ignores flags on default profile | §3.4, §6.12 | C | Chrome 136: the default *data directory* | Elastic; developer.chrome.com |
| A16 | DPAPI readable by any same-user app, so App-Bound | §6.13, §6.14 | C | Also: 2024 post says passwords are *planned* for App-Bound (spec already flags unverified) | Google 2024/07 |
| A17 | kCTF 60% io_uring; "safe only for use by trusted components" | §5, §6.7 | C | Verbatim | Google 2023/06 |
| A18 | Leaky.page 1 kB/s | §6.1 | C | Chrome 88, Skylake | Google 2021/03 |
| A19 | V8 sandbox flat $5,000 (Oct 2026); Chrome cut then restructured rewards for AI volume | §6.8, §6.17 | P | $5,000 ✓ (Help Net, 14 Oct 2026); SecurityWeek covers only the May cut ($500 base); restructuring is in Help Net | Help Net; SecurityWeek |
| A20 | Windows "no practical way…"; "any leaked handle can be abused" | §4.2, IPC-R1 | C | Verbatim | sandbox.md (GitHub mirror) |
| A21 | Mojo: use the origin the browser knows; no `CHECK` on untrusted content | IPC-R2, IPC-R5 | C | Paraphrase faithful | mojo.md (GitHub mirror) |
| A22 | 37 Chrome ITW 0-days 2022–Sep 2026; 21 V8 (14 type confusion); graphics 6; media 4; 3 logic, **all sandbox escapes** | Intro, §3.1 | **U** | 37 / 14 / 6 / 4 / 3 reproduced; V8 = 21 only if CVE-2024-0519 counted as V8; **only CVE-2025-2783 is labelled a sandbox escape** | P0 sheet CSV |
| A23 | Google allowed fingerprinting from Feb 2025 | §6.1 | C | From 16 February 2025 | ICO |
| A24 | CORB withheld cross-site responses | §6.11 | C | Verbatim | Google 2018/07 |
| A25 | Staged CT-evidenced distrust for Symantec and Entrust | §6.11 | P | Entrust ✓ (SCT after 11 Nov 2024); Symantec not in this post | Google 2024/06 |
| B1 | Firefox 48 e10s to about 1%; 8 of top 20 extensions | §2.1 | C | "fewer than one in every 100 users"; "just 8 of the top 20" | Computerworld |
| B2 | RLBox in 95; Win32k in 100; retrofit about four years | §2.1, §5 | P | Four years is the wider security-boundary re-architecture | Mozilla Hacks ×2 |
| B3 | Pwn2Own fixed under 21 h; ITW in 25 h | §3.1 | C | Verbatim | Mozilla 2024-04, 2024-10 |
| B4 | CVE-2025-2857 same pattern "within two days" | §3.2 | P | Pattern ✓; timing not in the advisory | MFSA 2025-19 |
| B5 | CVE-2022-1529 `__proto__` origin; CVE-2019-11708 renderer-sent URL | §3.2 | C | Both confirmed | ZDI; Mozilla A&D |
| B6 | Firefox WebGPU: 2 sandbox escapes in one release | §5, §6.9 | C | CVE-2025-13023, -13026 (Firefox 145); the advisory does not name wgpu | MFSA 2025-87 |
| B7 | Mozilla IPC fuzzing: restart too slow; resets destroyed determinism; hard to tell when done | §5, §6.6 | P | "too slow" ✓; "done" ✓; determinism reversed (lack of a cheap reset breaks it) | Mozilla A&D 2024-06 |
| B8 | Stylo reintroduced a fixed history leak | §5 | C | Bug 641731 regressed as 1420001 | Mozilla Hacks 2019 |
| B9 | Opus 4.6: 22 vulns in two weeks; Firefox 150: 271 with Mythos Preview; quote | §3.3, §6.17 | C | All verbatim | Anthropic; Mozilla |
| B10 | Firefox 145 roughly halved unique users | §6.15 | C | "by almost half"; "normalization" is the spec's word | Mozilla |
| B11 | Mozilla treats dumps as "highly sensitive private data" | §6.15 | P | A Mozilla engineer's quote in a comment (from bug 1138399), not policy | Bugzilla 1177121 |
| B12 | MFSA 2013-83 supports verify-then-apply on a locked file | §6.16 | C | "does not lock MAR file after signature verification" | MFSA 2013-83 |
| B13 | arrayref: payload in three crates, 86–107 min, ran at `cargo build` | §2.1, §3.3, §6.16 | P | Payload in `proc-macro1`; three crates republished to depend on it; 86–107 ✓ | Rust blog 2026-08-20 |
| B14 | rustls 2026: downgrade, ECH leak, 1 GiB pre-auth; none memory corruption; RUSTSEC-2026-0349 | §3.3, §6.1 | P | All three exist and the ID is right, but there were **5** 2026 advisories (also -0285, -0346) | advisory-db |
| B15 | quinn-proto 8 advisories, mostly OOM and panics | §3.3 | **W** | **10** in total (8 in 2026), all DoS | advisory-db |
| B16 | 75 malicious, 46 since 2024; 256 in 2026 vs 73–82 a year | §3.3 | P | 75 / 46 / 256 ✓; baseline **73–81** | advisory-db |
| B17 | Deno 40 advisories, about 35 logic; GHSA IDs and mechanisms | §3.3, §5, §6.8 | P | 40 ✓; both GHSA/CVE mappings and mechanisms ✓; "35 logic" holds only with a broad definition | gh api deno |
| B18 | Wasmtime 51 in 2026 vs 4 a year; CVE-2026-34971 Cranelift aarch64 escape | §3.3, §6.14 | P | 51 ✓; CVE ✓ ("This is a sandbox escape"); 2022 had 8 | gh api wasmtime; NVD |
| B19 | Immutable releases GA 2025-10-28 | §6.16 | C | Verbatim | GitHub changelog |
| B20 | Attestations plus reusable workflows achieve SLSA L3 | §6.16 | P | The docs say "help you achieve SLSA v1.0 Build Level 3" | GitHub docs |
| B21 | SLSA v1.2 exists | §7 | C | Approved | slsa.dev |
| B22 | cargo-vet safe-to-deploy and safe-to-run, imports, exemptions | §6.16 | C | Confirmed | cargo-vet book |
| B23 | TUF names the 10 listed attacks | SEC-17 | P | All present; two names differ ("malicious mirrors preventing updates"; key compromise is a resilience goal) | TUF spec §1.5.2 |
| B24 | CISA: moved tags; PyPI: cache; Clinejection: AI triager with shell | §2.2, §3.3 | P | CISA says "compromised" and "malicious commit", not moved tags; PyPI ✓ (Actions cache); Clinejection ✓ | CISA; PyPI; Khan |
| B25 | Storm-0558: key rotation switched off | §5 | U | True, but only in the CSRB PDF ("stopped the rotation entirely in 2021"), not the cited landing page | CISA CSRB page + PDF |
| B26 | Notepad++ unsigned metadata and redirection; Zen CVE-2026-41431 verification removed | §2.2, §5 | P | Redirection ✓; "unsigned" implied (now XMLDSig); Zen ✓ via OSV and Debian mirrors | Notepad++; search |
| B27 | TOTP phished in chalk/debug | §6.16 | U | Endor says only "phishing attack" | Endor Labs |
| B28 | s1ngularity used local AI CLIs | §6.1 | C | Claude, Gemini, Q | Wiz |
| B29 | SUNSPOT altered the build | §4.2 | C | Replaced InventoryManager.cs during build | CrowdStrike |
| C1 | Apple DTS: SBPL "not considered API"; why `<sandbox.h>` is deprecated | §2.2, §6.7 | C | Verbatim | Apple forums 777509 |
| C2 | SDK 27 `sandbox_init(3)` quote | §6.7 | C | Verbatim (continues "…killed on attempt to do so"); **"these profiles" = built-in named profiles (kSBXProfile\*, incl. PureComputation)** | openssh-unix-dev |
| C3 | Enhanced Security extensions "in a sandbox that restricts…"; entitlement name | §6.7 | C | Verbatim; Enhanced Security also needs `…enhanced-security-version-string` | Apple docs (JSON) |
| C4 | MIE broke 6 of 6 tested chains | §2.1, §11.2 | P | "six of the real-world exploit chains that we evaluated"; none could be rebuilt | Apple Security Research |
| C5 | Edge 2017 moved JIT out of process for ACG | §2.1 | C | Confirmed | Edge blog |
| C6 | SDSM: JIT blocks ACG, W^X, CET; vendors ship JIT-off modes | §2.2 | P | ACG and CET ✓; W^X and "vendors ship" not in post | Edge VR |
| C7 | Edge ActiveX post 2015 | §2.1 | C | 6 May 2015 | Edge blog |
| C8 | WebView2 guidance; running app keeps runtime; process groups per data folder | §6.14 | P | (a) ✓; (b) on the **Distribution** page, not the cited enterprise page; (c) ✓ | Microsoft Learn |
| C9 | WebView2 host can export every cookie | §3.4 | C | Paraphrase of "export the site's cookies" | BleepingComputer |
| C10 | Lumma 394,000 machines in two months | §6.1 | C | "over 394,000", 16 Mar to 16 May 2025 | Microsoft |
| C11 | Okta HAR: 5 customers' sessions hijacked | §3.4 | C | Verbatim | BleepingComputer |
| C12 | CometJacking URL param; Atlas CSRF memory; LayerX "up to 90%… to phishing" | §3.4, §6.14 | P | First two ✓; LayerX quote does not include "to phishing" | The Hacker News ×2 |
| C13 | PleaseFix `view-source:file://` and `#.pdf` | §3.4, §6.12 | C | Bypassed "twice" ✓ | Zenity |
| C14 | CVE-2026-0628: "a basic extension" took the Gemini panel | §3.4, §6.12 | P | Source says "malicious extensions with basic permissions" | Unit 42 |
| C15 | Adaptive attacks broke 12 defenses at >90% | §3.4, §6.12 | P | ">90% for most" | arXiv 2510.09023 |
| C16 | Anthropic 1% with 100 attempts; "meaningful risk" | §3.4 | C | Verbatim | Anthropic |
| C17 | MCPTox 36.5%; higher for capable models | §6.1, §6.12 | C | 36.5% in §4.2; abstract says "often more susceptible" | arXiv 2508.14925 |
| C18 | 10 of 11 password managers leaked | §5, §6.14 | C | Credentials 10/11 | The Hacker News |
| C19 | Meta Rule of Two | §6.12 | C | Confirmed | Willison |
| C20 | Pwn2Own Berlin 2026: Edge four logic bugs; no team entered Chrome | §3.2 | P | Edge ✓; Day One post doesn't mention Chrome (the full-schedule page has no Chrome entry) | ZDI |
| C21 | Securelist: `-2` pseudo-handle quote | §3.2 | C | Verbatim | Securelist |
| C22 | FORCEDENTRY "minimal, enumerable and reviewable"; FileWriter lifetime | §5 | C | Verbatim | Project Zero ×2 |
| C23 | Canonical cites 44%; Qualys three bypasses | §6.7 | C | Verbatim | Ubuntu; Qualys |
| C24 | DarthShader 39 bugs, 15 CVEs | §6.9 | C | Confirmed | arXiv 2409.01824 |
| C25 | WebGPU keystroke timing F1 82–98% | §6.13 | C | Confirmed | Giner et al. |
| C26 | PDF.js CVE-2024-4367 and CVE-2018-5158 both executed document data | §6.10 | P | 4367 ✓; 5158 only named | Codean Labs |
| C27 | CVE-2025-40943 crafted trace file ran script | §6.13 | C | Confirmed | NVD API |
| C28 | CA AG Healthline $1.55M partly over GPC | §6.15 | C | Confirmed | oag.ca.gov |
| C29 | "System call filtering isn't a sandbox"; Landlock TCP ABI 4, IOCTL_DEV ABI 5, UDP ABI 10 | §4.2, §6.7 | C | All confirmed | kernel docs |
| C30 | Zircon rights can only shrink | §4.2 | P | Page says rights "can be reduced"; cite `zx_handle_duplicate` docs | fuchsia.dev |
| C31 | Kani/Firecracker; Vest round trip and Rust; EverParse3D double-fetch; EverParse emits Rust | IPC-R6/R7, §6.6, §6.18 | P | First three ✓ (Vest round trip in the paper PDF); EverParse emits Rust only through EverCDDL | Four URLs |
| C32 | Rust removes the lifetime bugs that dominated Chromium IPC | §6.6 | P | Source: "many of these issues are object lifetime management issues"; no Rust, no "dominated" | GitHub Security Lab |
| C33 | libwebp/libvpx four platforms (Isosceles) | §5 | U | Isosceles does not say four platforms (the TAG PDF does) | Isosceles |
| C34 | GPU stack "only weakly sandboxed if at all" | §6.5 GPU | C | About the shader compilation pipeline specifically | Thinkst |
| C35 | PyTorch self-hosted runner attack | §6.16 | C | Confirmed | SecurityWeek |
| C36 | Extensions can intercept WebAuthn | §6.14 | C | Confirmed | SquareX |

Not checked (lower impact): Capsicum "about 100 lines", McIlroy et al. quote, IEBlog, NPAPI and PNaCl posts, CT how-it-works, DigiNotar and Symantec 2017 posts, Snyder et al., DRAWNAPART, Chrowned, arXiv 2406.10279, Hive Pro, Red Canary, Tor and Chromium deterministic-build docs, the Rule of 2 doc, MojoLPM, the "OpenAI via press" quote, and SecurityWeek's Gray Swan figures (the spec already marks these secondary).

## 2. Internal facts about Izcal

| # | Claim | Spec location | Verdict | Evidence |
|---|---|---|---|---|
| 1 | P1 = security first (HGBA-2 §2) | §2.1, §6.4 | C | HGBA-2 §2 table |
| 2 | M-1 (HGBA-2 §2) and D-S10 (Stage 1 13.1) forbid copying; build our own | §1, §4.3, §6.4 | C | HGBA-2 l.68–70; Stage 1 13.1 D-S10 |
| 3 | I-1 to I-6 meanings | §4.2, §4.3, §6.4 | C | HGBA-2 §4.1 |
| 4 | I-1(b) already partitions connection pools and DNS | §6.4, §6.15 | C | "connection and socket pools, DNS cache…" |
| 5 | I-2 note: protected-media host outside boundary; agent tabs never use it | §6.4, §6.5 | C | HGBA-2 l.124 |
| 6 | HGBA-2 §3 "Mach ports under Seatbelt" | §3.5 | C | HGBA-2 l.79 |
| 7 | 4.8: unsafe inventory, tainted FFI, Rule of 2, FFI fuzzing | §6.4 | C | HGBA-2 §4.8 |
| 8 | DrumBrake 23 RCE claim already in HGBA-2 4.2; D2 rule on ITW exploit | §6.8 | C | HGBA-2 l.140, l.394 |
| 9 | S-1 to S-8 meanings (S-2 incl. io_uring, ptrace, pkey_mprotect; S-4a "CET/BTI on every platform"; S-5 and S-6 Stage 2) | §3.5, §6.4 | C | HGBA-2 §6 rows 333–356 |
| 10 | S-4e holds the rebase procedure, backport rule and one-cycle window | §3.5, §6.4, §6.8 | C | S-4e row (titled "cppgc API exposure stays valid") |
| 11 | "Changes written against V8's public interfaces" is S-4e's rule | §6.7 | **W** | It is M-1 (HGBA-2 l.70), D-S10, CONTRIBUTING |
| 12 | D2: "ship V8 security fixes within one Chrome stable cycle" | §3.5, §6.8, §11.3 | C | HGBA-2 l.392 verbatim |
| 13 | Patch latency has no measurement and no failure action | §3.5, §6.8, §11.3 #1 | **W** | Stage 1 7.7 lag metric "Days behind Chrome stable V8 security fixes", escalation at >2 weeks; S-4e is a ship blocker with a backport fallback |
| 14 | S-4a has a per-site exception mechanism | §10 | C | S-4a row |
| 15 | B1, B2, B8, B9, B10 meanings | §1, §4.3, §6.4 | C | HGBA-3 §B1–B10 |
| 16 | VER-1 = proven or Kani-checked; option (a) "Kani per message (VER-1 as is)" | §6.4, §6.18 | P | VER-1 already allows EverParse-class proofs; Stage 1 4.1 picks Kani with EverParse as a Stage A stretch |
| 17 | DET-1, DST-1, AG-0, AG-3 (model defenses off), AG-5, PRIV-1, PRIV-2 | §6.4 | C | HGBA-3 targets l.398–432 |
| 18 | B10 security-classifier carve-out; ML inventory empty | §3.5, §11.3 #4 | C | HGBA-3 l.261 ("empty at 3.1.0") |
| 19 | Fifteen process kinds = Stage 1 §1 (10) + DevTools 6.1 (4) + crash handler (Stage 1 §6) | §6.3, §6.5 | C | Counted |
| 20 | 13.2 threat-model row lists 8; omits protected-media host, ML service and crash handler | §6.5, §11.3 #3 | C | 13.2 "Threat model" row |
| 21 | G1–G9; G4–G7 meanings | §6.4 | C | Stage 1 §2.2 |
| 22 | G12 restricts replay hosts to `hg-substrate-sim` | §6.5 | P | Defined in DevTools 6.1, a proposed amendment (DevTools 9.5) |
| 23 | Stage 1 §4: `.hgidl`, endpoints, sequence affinity, sync allowlist reviewed by security owner, Kani per message, `hg-ipc-codegen` | §6.4, §6.6 | C | Stage 1 4.1 (Stage 1 itself also says `hg-ipc-gen` in §2) |
| 24 | IPC-R13: nested sync forbidden per Stage 1 4.1 | §6.6 | U | No nested-sync rule in Stage 1 |
| 25 | 7.1: mirror, `sccache` generally, vendored release deps, M0 17.5–46 min release builds | §3.5, §6.8, §6.16 | C | Stage 1 7.1 table |
| 26 | 7.4 shared bug database; 7.5 Fuzzilli REPRL, cargo-fuzz per IPC message | §6.6, §6.8, SEC-5 | C | Stage 1 7.4, 7.5 |
| 27 | 7.7 V8 bot within 24 h†; security owner's SLA one cycle; 7.8 signed updates | §6.8, §3.5 | C | Stage 1 7.7, 7.8 |
| 28 | 8.3 sums to 23–30.5; Network and security track 2 covering network service, sandboxing, VER-1 | §9.2 | C | Rows sum 23–30.5 |
| 29 | Security owner referenced (4.1, 7.7) but not staffed | §9.2, §11.3 #2 | C | No role in 8.3 |
| 30 | D-S1, D-S2 (sandboxed shell, CSS-only), D-S4, D-S7 (13.1a), D-S8, D-S11 | §6.4 | C | Stage 1 §11, 13.1 |
| 31 | D-S12 = "IIP with adapters" | §6.4 | **W** | D-S12 = compatibility oracles (black-box observations); IIP is D-S11 |
| 32 | 13.2 rows: codecs list, TLS/certs (platform verifier, CT, CRLs), importer and OS-keychain password manager, download reputation, TUF updates, Flatpak Stage 2, disclosure policy | §3.5, §6.10, §6.11, §6.14 | C | Stage 1 13.2 |
| 33 | Stage 1 ships C codecs before library sandboxing applies | §3.5, §6.10, SEC-D3 | P | Target is Stage 2, but Stage 1 §1, 13.2 ("libvpx in the library sandbox") and §6 PDF already put them in RLBox-style or utility sandboxes |
| 34 | Stage 1 §6: GPC three states; "no custom credential store"; seed (site, container, session); separate crash handler; banking container | §6.13, §6.15, §11.3, SEC-D5 | C | Stage 1 §6 |
| 35 | Stage 1 3.4: recording user-initiated per tab, replays in simulator | §11.3 #11 | C | Stage 1 3.4 |
| 36 | DevTools 6.1 kinds and workspace metadata; 6.5 rules 3, 7, 8, 9, 10 | §6.3, §6.5, §6.12, §6.13 | C | DevTools 6.1, 6.5 |
| 37 | DT-10, DT-11, DT-13, DT-D2, Q-DT-3 | §3.5, §6.5, §6.13 | C | DevTools §8, l.214, l.492 |
| 38 | DevTools 9.2: 8 engineer-months in Network and security for `hg-net-observe`, qlog, four threat models | §9.2 | C | DevTools 9.2 table |
| 39 | Shell UI 4.5 phishing interstitials; UI-5, UI-9, UI-10 | §3.5, §6.4, §11.3 #4 | C | Shell UI 4.5, §5 |
| 40 | CONTRIBUTING: self-merge allowed; "commit their `SHA256SUMS`" | §1, §3.5, §11.3 #5 | C | "Code review"; "Large files and artifacts" |
| 41 | Release tags client-side (`pre-push`); S-4e step 4 lifts force-push protection; no signing | §6.16, §11.3 #6 | C | "Pushing", S-4e steps; no signing rule anywhere |
| 42 | `security` check includes gitleaks | §6.4 | P | gitleaks is the `secrets` check; `security` = SAFETY, risky patterns, zizmor, cargo-deny advisories |
| 43 | AI review: read-only tools, structured verdict, fails open | §6.16 | C | `checks.sh` l.344–395 (`--tools "Read,Grep,Glob"`, JSON schema, warns on failure) |
| 44 | `ci-install-tools.sh` pins and SHA-checks gitleaks, actionlint, zizmor, cargo-deny | §6.16 | C | Script |
| 45 | `checks.yml` SHA-pinned; event fields through `env:` | §6.16 | C | On `main` |
| 46 | `checks.yml` runs the PR's own `checks.sh` | §6.16 | C | `pull_request` checkout |
| 47 | Fork workflows' actions pinned to SHAs: "Fixed" | §6.16, §9.1 | **W** (for `hgba`) | Only in open PR #2; `hgba` uses `@v4` and similar tags |
| 48 | CI tool downloads verified, incl. fork | §6.16 | **W** (for `hgba`) | `hgba` `izcal-checks.yml` downloads unverified and pipes a script into bash; PR #2 fixes it |
| 49 | Caches skipped on `refs/tags/`: "Fixed" | §6.16, §9.1, §11.3 #8 | **W** (for `hgba`) | Only in PR #2 |
| 50 | `build.rs` checks the archive only if `RUSTY_V8_ARCHIVE_SHA256` is set; nothing sets it | §3.5, §6.16 | C | `pinned_archive_sha256()`; only logs mention the variable (both branches) |
| 51 | Binding file never hash-checked | §6.16 | C | `print_prebuilt_src_binding_path()` calls `try_download_artifact(…, None)` |
| 52 | `SHA256SUMS` made and published by the same pipeline | §6.16 | C | Build jobs hash; publish job concatenates and releases |
| 53 | `softprops/action-gh-release` with `contents: write` | §6.16 | C | Publish job (SHA-pinned only in PR #2) |
| 54 | Fork fetches `checks.sh` from unpinned default branch | §6.16 | C | `izcal-checks.yml` sparse checkout with no `ref` |
| 55 | `IZCAL_RUNNER_*` can route `pull_request` jobs to self-hosted runners | §6.16 | C | Matrix `vars.IZCAL_RUNNER_*`; `pull_request` trigger |
| 56 | M0 measured runtime determinism, not archive equality | §6.16 | C | `m0/determinism_crossplatform.md` |
| 57 | §11.3 #1 patch latency not a target | §11.3 | P | See #13: a metric and escalation exist; a severity target does not |
| 58 | §11.3 #2 security owner unstaffed | §11.3 | C | — |
| 59 | §11.3 #3 threat-model list incomplete | §11.3 | C | — |
| 60 | §11.3 #4 interstitials without a phishing source | §11.3 | C | No phishing or Safe Browsing source in HGBA-2, Stage 1, HGBA-3 (classifier carve-out only) |
| 61 | §11.3 #5 `SHA256SUMS` committed but unverified | §11.3 | C | — |
| 62 | §11.3 #6 protection server-side yet lifted for S-4e; tags client-side | §11.3 | C | — |
| 63 | §11.3 #7 S-4a "CET/BTI on every platform" vs hardware | §11.3 | P | The CPU and OS gap is real; "Apple Silicon uses PAC and BTI, not CET" does not contradict "CET/BTI" |
| 64 | §11.3 #8 `sccache` generally vs fork skipping on tags | §11.3 | P | Stage 1 7.1 is as stated; the fork's skip is only in PR #2 |
| 65 | §11.3 #9 GPC three states vs about twelve | §11.3 | C | Stage 1 §6 says three (the external count is Low confidence) |
| 66 | §11.3 #10 "no custom credential store" vs 13.2 password manager | §11.3 | C | Both texts as quoted |
| 67 | §11.3 #11 recording start vs DevTools pending amendment | §11.3 | C | DevTools 9.5 row for Stage 1 3.4 |

Note: Section 11.3 lists **11** contradictions, not 10. Of these, 8 are real as stated (#2–#6, #9–#11) and 3 are partly right (#1, #7, #8).

## 3. Arithmetic and internal consistency

| # | Check | Spec location | Verdict | Detail |
|---|---|---|---|---|
| 1 | 8.3 rows sum to 23–30.5; +2–3 = 25–33.5 | Intro, §9.2 | OK | 2+5+3+2+2+2+1+4+1+1 = 23; 3+6+4+2+2+3+2+5+1.5+2 = 30.5 |
| 2 | Engineer-years 46–92 → 50–100 | §9.2 | OK | 23×2 = 46, 30.5×3 = 91.5; 25×2 = 50, 33.5×3 = 100.5 |
| 3 | 83 engineer-months; 2.3–3.5 engineers; track 2–3† | §9.2 | Partly | Sum = 83 ✓; 83/36 = 2.3, 83/24 = 3.46 ✓; the top end (3.5) exceeds the 2–3† track and relies on contracting "part of the audit work" without a number |
| 4 | Target IDs referenced but undefined; duplicates | Whole spec | OK | SEC-1–28, ATK-1–9, TB-1–12, IPC-R1–15, Q-SEC-1–12 all defined once and only those referenced |
| 5 | Decisions referenced but undefined | Whole spec | OK | SEC-D1–D10 defined and only those referenced; SEC-D7/D9 deferrals of targets are explicit |
| 6 | Internal section references resolve | Whole spec | **Wrong** (1) | §6.5 Storage "Open" → "Section 6.14" has no cookie-encryption content; other references resolve (4.8, 7.x, 8.3, 13.2 point to other documents and are labelled) |
| 7 | Threat-model files at M1 | §6.5 vs §9.1 | Inconsistent | §6.5: each model a file "at M1"; §9.1 M1: only broker, renderer, network service |
| 8 | Retention decided vs pending | §6.13, §6.15, SEC-21 (c), SEC-D9, §8 | Inconsistent | SEC-21 (c) already targets 30-day recording retention while SEC-D9 leaves it open |
| 9 | "Every 2025 escape crossed the broker" | §4.4 row 1 vs §3.2 | Inconsistent | §3.2 (GTIG) says they went through ART and GPU drivers |
| 10 | † on thresholds the spec sets | §8, §6.16, §6.18, §9.3 | Inconsistent | Unmarked: "two misses in a rolling six months" (SEC-1), "two quarters" (SEC-5 (b), SEC-D1, §6.18), "under one engineer" (SEC-D2), certificates rotate "at least yearly" (§6.16), checklist "monthly" and "medium severity or above" (SEC-27), "each quarter" (SEC-2) |
| 11 | M0 "(done)" items | §9.1 | Inconsistent with repository | SEC-13 (d), (e) done only in the project and in fork PR #2 (open) |
| 12 | Markdown link syntax | Whole spec | OK | No malformed links; the one relative link (`../CONTRIBUTING.md`) resolves |
| 13 | Fifteen process kinds in §6.3 and §6.5 | §6.3, §6.5 | OK | 15 rows; 15 first-pass models |
| 14 | Milestone placement of SEC targets | §8 vs §9.1 | OK | SEC-5 (a) M6, SEC-8 macOS M1, SEC-12 M7, SEC-26 M6, SEC-10 (a) work M3–M4 / due M8 consistent |
| 15 | SEC-D1 vs §6.18 V8 row | §6.18, §9.3 | OK | Same options and trigger |
| 16 | "Six documents" in intro | Intro | OK | HGBA-2, HGBA-3, Stage 1, DevTools, shell UI, CONTRIBUTING |
| 17 | "About forty IDs" in intro | Intro | OK | §6.4 maps about 40 existing IDs |
