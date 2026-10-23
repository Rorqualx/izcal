# HGBA shell UI design

*Companion to the Stage 1 architecture design, HGBA-2.1 and HGBA-3.1. Written against design draft 0.8, HGBA-2.1.3 and HGBA-3.1.3; the revisions then current were design draft 0.10, HGBA-2 2.2.0 and HGBA-3 3.2.0 (now 0.14.5, HGBA-2 2.4.4 and HGBA-3 3.3.3). Date: 14 October 2026. Status: draft 0.5, complete for review (0.5, 21 October 2026: phase 3 amendments A-UI-1 to A-UI-6 and A-UI-8 applied from the accepted resource-footprint, fonts and text, images and media, graphics, layout and content-blocking specs and the SVG, animation and editing specs, Section 10; 0.4, 21 October 2026: batch-4 amendments applied from the accepted shell spec 0.3 and phishing protection spec 0.3 — prompts are separate broker-owned OS windows, the trusted-region property is restated as an own-layer/no-overlap property, the site panel gains Phishing protection and trust-store rows, the phishing/malware interstitial is the phishing spec's surface, UI-9 cites PH-11 and UI-10 covers the interstitial, notifications move to Stage 2; 0.3, 17 October 2026: keyboard model anchored to WAI-ARIA APG and UAAG 2.0 under HGBA-2 M-1). Evidence notes: `research_notes/Shell UI design/`.*

This document consolidates every UI decision scattered across the HGBA documents, then designs the shell the same way the engine was designed. Each surface traces to a requirement, each claim about user behaviour carries a source, each element has a maturity label, and each target can fail. It changes no engine target. Thresholds marked † are set by this document.

---

## 1. Principles for the shell

These apply the engine's principles (HGBA-3.1 P1–P7) to the UI. When two conflict, the higher one wins.

| ID | Principle | What it means for the UI | Derived from |
|---|---|---|---|
| UP1 | **Honest over reassuring** | A label says what is protected and what is not. No icon implies more safety than the engine provides | P1; the review's Tor-route label; D-S7 protected-media label |
| UP2 | **Defaults do the work** | Every feature must be right for a user who never opens settings. Settings exist for exceptions | Before Firefox made tracking protection the default, about 20% of users had it on; Spool's "fewer than 5% change settings" holds as an informal finding, though a 1996 CHI study found 92% of WordPerfect users customized something (`tabs_onboarding_a11y.md`) |
| UP3 | **Nothing appears that the user did not ask for** | No AI surfaces when AI is off, no promotions, no nags to change a choice already made, no re-prompts after a "no" | HGBA-3.1 B10; Section 13 (default browser) |
| UP4 | **The shell is a page** | The shell is web content in a sandboxed renderer. It inherits the engine's accessibility, is tested like a page, and replays under the deterministic substrate | D-S2; HGBA-3.1 B1 |
| UP5 | **No chrome as differentiation** | Tab-strip conveniences are table stakes, built cheaply. Effort goes to surfaces that protect users | Composable-platform review; Arc's 2025 letter |
| UP6 | **Security UI is small and measured** | Few indicators, each tested for comprehension; warnings are tested for adherence | 89% of users misunderstood the padlock (Google, n=1,880, 2022); none of 14 URL/EV formats improved identity judgements (USENIX Security 2019); a redesigned warning raised heeding from 37% to 62% (CHI 2015), and attention drops sharply from the second exposure (`security_permissions.md`) |

---

## 2. Consolidated decision register

Every UI-bearing decision already made, with its source. This table is the shell's requirements list.

| # | Decision | Source |
|---|---|---|
| R1 | Shell is HTML/CSS rendered by the engine, in its own sandboxed **shell renderer** with zero ambient authority; it reaches browser state only through capability endpoints granted by the broker | Stage 1 design D-S2, Section 1 |
| R2 | User theming is **CSS-only**; scripted customization goes through W-1 components | D-S2 |
| R3 | A minimal **native recovery window** opens when the HTML shell fails to start twice in a row; it offers **five actions and nothing else**: reset the profile, start without hardware acceleration, start with themes and extensions off, send the crash report, and quit | D-S2, as amended by SH-D22, Section 6 |
| R4 | **AI off by default**: no AI entry points, chips, suggestions or promotions anywhere in default UI; settings is the only place AI appears; the choice survives updates with no re-prompt | HGBA-3.1 B10, AG-0 |
| R5 | **Accessibility model setting** is separate from the AI switch and off by default | HGBA-3.1 B10 (R4 class) |
| R6 | **Repair inspector**: "What the browser changed on this page" lists every accessibility repair; one toggle reverts them all | HGBA-3.1.1 B8; Section 5.8 |
| R7 | **Containers**: a few named defaults (personal, work, banking, shopping), editable; never a blank policy editor; each carries storage, block lists, JavaScript mode, agent limits and a network route | Section 6 |
| R8 | **Tor route label**: "hides your IP address, not your browser" | Section 6 |
| R9 | **JavaScript modes** per site, inherited from the container: off, same-site only, full; the same-site caveat (a taken-over subdomain counts as same-site) is stated in the mode's description | Section 6 |
| R10 | **Protected media mode** label: "Playing protected video using Windows' / macOS's media engine"; one-click switch back; remembered per container; per-site "never switch" | Section 13.1a |
| R11 | **Permission prompts** require user activation; quiet UI; auto-revocation of unused grants; per-site and per-container grants | Section 6, 13.2 |
| R12 | **Notification permission is not offered in Stage 1 at all** (amended by SH-D17); Stage 2 adds delivery and controls: mute and snooze from the notification, opt-in per-site digest, never delays by default | Section 6; SH-D17 |
| R13 | **Per-site resource meter** (Stage 2): CPU, memory, network and energy shown on request; visibility only | Section 6 |
| R14 | **Crash reports** opt-out with a per-crash prompt, "send" pre-selected, viewable before sending; **telemetry** opt-in | D-S4 |
| R15 | **Onboarding** explains AI-off, containers, import and the DRM position | Section 13.2 |
| R16 | **Import** from Chrome and Firefox (bookmarks, history, passwords) and a **basic password manager** in Stage 1 | Section 13.2 |
| R17 | Stage 1 everyday features: find in page, zoom, fullscreen, clipboard, drag and drop, downloads manager with checks | Section 13.2 |
| R18 | Stage 2: vertical tabs, split view, tab groups as table stakes; save for offline with reader view; PDF editing | Section 6 |
| R19 | Default-browser handling uses OS-sanctioned APIs only and never re-prompts after "no" | Section 6 |
| R20 | The shell passes the same accessibility checks as a web page; the UI is English-only in Stage 1 with Fluent strings | Section 13.2 |
| R21 | The honest-UA, GPC, cookie partitioning and anti-fingerprinting defaults are engine behaviour; the shell's job is to explain them, not to expose knobs by default | Sections 6, 13.2; D-S1 |

---

## 3. Shell architecture

```
 BROWSER PROCESS (broker)                          SHELL RENDERER (sandboxed web content)
 +---------------------------------+   capability   +-----------------------------------+
 | tabs & navigation service       |<-- endpoints ->| window chrome: tab strip, address  |
 | permission store                |   (generated   | bar, toolbar, panels, prompts,     |
 | container & policy service      |    IPC, B2)    | settings, onboarding, inspector   |
 | downloads, profile, import      |                 | (HTML/CSS + project JS)           |
 | protected-media switch          |                 | user theme: CSS only               |
 +---------------------------------+                 +-----------------------------------+
        | launches only on demand                          ^ composited with page content
        v                                                  | by the GPU service
 NATIVE RECOVERY WINDOW (R3)          content renderers, protected-media host, ...
```

- **Each shell service is a capability endpoint** with its own IDL file (Section 4 of the Stage 1 design). The shell renderer gets exactly these endpoints and no others, so a compromised shell cannot open files or sockets. This is the shell-side version of HGBA-2.0 I-2.
- **Prompts that must not be spoofable** (permissions, protected-media switch, credential confirmation for agents, and the phishing/malware interstitial) are **separate broker-owned OS windows** with their own OS layer, not a region drawn inside the page window (SH-D2, SH-D3). The prompt renderer software-rasterises them in a jitless, project-JS-only process; the content GPU is never on the prompt pixel path. Page-rendered content never contains a prompt the browser issued. Where an OS requires a window to carry a prompt, that window's own layer is the trusted layer; the property is tested, not the window class (SH-11).
- **The shell runs under the deterministic substrate.** A shell bug found in simulation replays from its seed like any engine bug, because the shell is a page.

---

## 4. Information architecture and wireframes

Maturity labels follow HGBA-3.1 P7: **shipped** (in a browser today), **shipped elsewhere** (in another product category), **new** (no shipped precedent found).

### 4.1 Main window

```
+--------------------------------------------------------------------------------+
| [<] [>] [reload]  [@ work]  example.com/path/page              [site] [dl] [=] |  <- toolbar + address bar
+--------------------------------------------------------------------------------+
| [x Mail  ] [x Docs  ] [x example.com ] [+]                         [tab search] |  <- tab strip (horizontal default)
+================================================================================+  <- "line of death": below is page
|                                                                                |
|                               page content                                    |
|                                                                                |
+--------------------------------------------------------------------------------+
```

- **Tab strip:** horizontal by default. Most browsing happens with one tab open, plus a long tail of heavy users (Dubroy, CHI 2010; Huang and White, 2010). Vertical tabs are a **one-click, persistent toggle** in Stage 2, not a forced layout (UP5). *Shipped.*
- **Fear of losing tabs is the problem to solve,** not tab count. 55% of surveyed users "can't let go" of tabs, and about 25% have crashed from too many (Chang et al., CHI 2021). Two answers that cost little:
  - every closed tab is recoverable through a searchable "recently closed" list;
  - cold tabs are discarded under the shared memory policy (Stage 1 design, Section 6) instead of the user closing them.

  *Shipped* (Chrome tab search, Firefox recently closed), combined here.
- **Container chip** before the address: the container's **name and colour together**. No study of container-colour comprehension exists, so colour alone is not trusted; target UI-8 measures it. *Shipped* (Firefox Containers), with name added.
- **No padlock and no trust icon.** A neutral **site-controls** button opens the site panel (4.3), following Chrome 117's change: 89% misread the padlock, and in testing users opened site controls more often without confusion. Plain-HTTP pages show "Not secure". HTTPS-first is the default; Chrome's equivalent warned on fewer than 3% of page loads. *Shipped.*
- **URL display:** the full URL with the registrable domain emphasized. Chrome's domain-only trial "didn't move relevant security metrics" and none of 14 formats improved identity judgements, so no new format is invented. Lookalike-domain detection relies on IDN homograph rules plus a lookalike warning; users identified the real site for only 40% of lookalike URLs (CHI 2020). *Shipped.*
- **Trusted layer, not a trusted strip inside the page:** browser prompts (permissions, the protected-media switch, credential confirmation, the phishing/malware interstitial) are drawn in **their own broker-owned OS window above the content layer**, so page content cannot paint over or cover them (UI-10, SH-D2, SH-D3). The earlier "line of death" region inside the page window is withdrawn: the property is *unoccludable by the page, broker-hit-tested*, and a separate owned window is how Izcal implements it on every OS (SH-11). *Shipped mechanism* (separate OS prompt windows); the property claim is tested, not assumed.

### 4.2 Address bar states

```
 normal (HTTPS)      [@ personal]  example.com/page                         [site]
 plain HTTP          [@ personal]  (!) Not secure  example.com/page         [site]
 protected media     [@ personal]  netflix.com/watch   [>> Windows media engine | back]
 Tor route           [@ research]  (o) Tor: hides your IP, not your browser  site.org
 JS same-site mode   [@ banking ]  bank.example  [JS: same-site]            [site]
 permission pending  [@ work    ]  meet.example  [cam? Allow once | Block]   [site]
```

- **Protected media** (R10): the label states which engine is rendering, and "back" returns the tab to HGBA's engine. *New* as a browser feature; the closest precedent is Edge's IE mode indicator (*shipped*).
- **Tor route** (R8): the honest label is part of the address bar, not a tooltip. *New wording*, shipped mechanism (Brave Tor windows).
- **Wording rule:** every label states a capability or a limit in plain terms. Vague promises are banned: a study of 13 browsers' private-browsing explanations found most misconceptions uncorrected and some made worse, with "your secrets are safe"-style wording singled out (WWW 2018). Target UI-3 tests each label.

### 4.3 Site panel (opened from [site])

```
+-------------------------------------------------+
| example.com                         [@ personal]|
| Connection: secure (TLS 1.3, hybrid PQ)         |
|-------------------------------------------------|
| Permissions        camera: ask  location: off   |
| Blocked here       9 parties, 4 orgs    [list]  |
| Phishing protect.  protected · list 12 min old   |
| JavaScript         full          [change]       |
| Fingerprinting     protected (noise on readback)|
| Accessibility      2 repairs applied [inspect]  |  <- only if repairs are enabled (R6)
| Resources          CPU 3%  net 1.2 MB  [why]    |  <- Stage 2 (R13): causal view, not a meter
| Protected media    not in use                   |
| Certificates       checked by your OS store     |
|-------------------------------------------------|
| [Clear data for this site]   [Site settings]    |
+-------------------------------------------------+
```

**Phishing protection row** (added 21 October 2026, phishing spec PH-D6): the row states whether this (site, container) is protected, the **bundle age** and the **last check**, with a report link; it has **no off switch** (unlike the Blocking row's per-site off). It shows the `no-bundle` state as "protection data unavailable", and after a per-navigation proceed, "You chose to visit a listed site — Leave". Paste-and-go runs the check before the request and never previews speculatively. The row reads the durable prompt-ledger summary linked by `check_id`, not the 24 h† check-log detail.

**Password manager and form fill:** before filling, the manager consults the check record and **does not autofill on a `listed` verdict or a `no-bundle` state** without a fresh user action. Lookalike-domain detection remains the shell's warning class (Q-PH-16, open); the reputation list supplies the URL formatter, never a lookalike claim.

**Certificates row:** carries the trust-store disclosure ("Certificates are checked by your operating system's store until Izcal's root programme ships") on every site while the OS store is in use; `about:build` records the mode (networking NW-D24).

One panel holds every per-site control, replacing a scatter of icons. Brave and Firefox have no published comprehension data for their shield icons (gap), so the panel uses text rows rather than an icon state. *Shipped* in parts (Chrome site controls, Brave Shields); the combined panel is *new*.

### 4.4 Permission prompts

```
                [@ work] meet.example wants to use your camera
                ( Allow this time )   ( Allow always )   ( Block )
```

- **Shown only after a user gesture** on the page (R11). Notification prompts are the bulk of the problem: 74% of Chrome's permission prompts, granted only 12% of the time on desktop and 23% on Android. A quieter prompt cut unnecessary prompt actions by up to 30% (USENIX Security 2021). In Firefox about 99% went unaccepted; requiring a gesture raised first-time accepts to 52–56%. *Shipped.*
- **Notifications use the quiet chip only**: a small address-bar chip, never a modal. *Shipped* (Chrome quiet UI).
- **"Allow this time" is the first option** for camera, microphone and location. *Shipped* (Chrome one-time permissions; uptake was never published).
- **Auto-revocation** of grants a site does not use, with a notice in the site panel. *Shipped* (Chrome); the effect sizes were never published.
- **Page-embedded permission elements** such as `<geolocation>` (Chrome 144) are supported once standardized. The only published results are partner figures (for example Zoom: 46.9% fewer camera and microphone errors). *Shipped* in one browser.

### 4.5 Warnings (interstitials)

- **The safe action is the prominent button, and "proceed" sits behind "details".** This design raised heeding from 37% to 62% in Chrome's field study (CHI 2015), though it did not improve understanding.
- Attention drops sharply from the second exposure to a repeated warning, so warning surfaces are few, and their appearance varies with the threat class (certificate, phishing, malware, lookalike) rather than reusing one template.
- **The phishing and malware interstitials are the phishing protection spec's surfaces** (PH-D6, PH-12; class `security-interstitial`), drawn by the prompt renderer in the broker-placed OS layer: broker-formatted target URL (scheme, host, port, path) and Izcal-authored list evidence, never page strings; the safe action is prominent and a proceed is per navigation, never persisted. The shell owns accessibility, timing, placement and the ledger row; the phishing spec owns the decision, lists, copy and report decision. There is **no second warning UI**.
- **Downloads** use two levels, "suspicious" and "dangerous", each with a different action set. *Shipped* (Chrome, which published no numbers). The verdict feeding them is the phishing spec's URL check (initial URL and every redirect hop) plus SH-R7's type rules; the malware class is a documented gap until URLhaus/Smarthaus is licensed (PH-D7, Q-PH-2).

### 4.6 Onboarding (first run)

```
 1  Welcome          "Your browser, with AI off and tracking blocked by default."   [Next]
 2  Bring your stuff Import from Chrome / Firefox: bookmarks, history, passwords      [Import] [Later]
 3  Containers       personal · work · banking · shopping  (rename or skip)          [Next]
 4  Things to know   - AI features are off; turn them on in Settings > AI if you want them.
                     - Streaming video uses your system's media engine when needed (D-S7).
                     - On Linux, DRM-protected streaming is not available yet.
                     [Make default browser]  [Not now]                               [Done]
```

- **Four screens, all skippable.** No published data links onboarding length to completion (gap), so UI-4 measures it.
- **Import is offered, never forced, and runs in the background.** In Mozilla's study of import at first run (CHI 2018), most users skipped it and slow imports lowered later usage, so import stays fast and stays available from Settings.
- **The default-browser request appears once**, through the OS-sanctioned API. "Not now" is final unless the user returns to Settings (R19). When the moment of asking changed, the share of people picking the preinstalled browser nearly doubled (11% to 19%) in Mozilla's 12,000-person experiment, so the prompt avoids pressure design.
- **Nothing about AI beyond one factual line** (R4).

### 4.7 Settings

- **Organized by task, not a catch-all "General" page**, following Firefox's June 2026 redesign, which was based on card sorting and usability tests:
  1. Privacy and security
  2. Containers
  3. Permissions and site settings
  4. Appearance
  5. Passwords and autofill
  6. Downloads
  7. Accessibility
  8. AI (the only AI surface, R4)
  9. Media and DRM
  10. Updates and recovery
  11. About
- **A search field at the top.** No vendor has published how often settings search is used (gap).
- **Accessibility settings group:** R0–R4 repair classes (HGBA-3.1.1 B10), forced colors following the OS, zoom, and keyboard options.
- **No setting is removed without measuring its use first.** Firefox removed compact density in 2021 on an unmeasured assumption, then restored it in 2026. Usage telemetry is opt-in (D-S4), so measurement comes from opt-in data plus lab studies.

### 4.8 Recovery window (native, R3)

```
+------------------------------------------------------------------+
|  [Browser name] couldn't start normally (2 attempts).             |
|                                                                   |
|  ( Start without hardware acceleration )                          |
|  ( Start with themes and add-ons turned off )                     |
|  ( Reset profile settings: keeps bookmarks, history, passwords )  |
|  ( Send the crash report )        [view what is sent]             |
|  ( Quit )                                                         |
+------------------------------------------------------------------+
```

The name is **"Troubleshoot"**, and every option says exactly what it disables. Firefox renamed Safe Mode to Troubleshoot Mode in 2021, and an earlier bug showed the old wording misstated what was disabled. Chrome has no safe mode. *Shipped* (Firefox).

### 4.9 Accessibility of the shell

- **Standards:** WCAG 2.2 AA applied to software through **WCAG2ICT (2024)**, plus **UAAG 2.0**, the browser-specific guideline.
- **Keyboard model:** anchored to the WAI-ARIA Authoring Practices toolbar, tabs and landmark patterns and to UAAG 2.0 keyboard guidance. Chromium's published shortcut model is the compatibility reference for users' habits (tested by UI-2), not the source. F6 cycles between the toolbar, tab strip, page and panels. Toolbar buttons are reached with arrow keys inside the toolbar, not by tabbing through each one. Every action has a keyboard path.
- **Screen readers:** the shell is a page (UP4), so it exposes its tree through the same AccessKit path as content. Firefox's HTML UI and its 2023 accessibility rewrite (screen readers 2–20× faster) show this design can perform. 52% of screen-reader users mainly use Chrome (WebAIM 2024), so their Chrome keyboard habits are the reference.
- **Forced colors and contrast** follow the OS (R0). Dark mode follows the OS; no representative usage data exists (gap).

### 4.10 Visual language

- **System UI fonts:** the OS tier's UI face on macOS and Windows and Izcal's sans alias face on Linux, with the interim tier for scripts that face does not cover until their Izcal faces ship; native-surface strings follow the fonts and text spec's TX-16 rule (fonts and text spec 9.5, R23).
- Two densities, normal and compact. Compact is kept because removing it was a measured-later mistake elsewhere.
- Light and dark themes following the OS.
- No brand colour on any security state.
- Container colours from a palette checked for colour-blind distinguishability, always paired with the container name.

---

## 5. Targets

Same form as the engine targets. "Lab study" means a moderated, task-based study with participants recruited outside the project; sizes are set by power analysis at pre-registration, as for A11Y-3.

| ID | Target | Threshold / method | If missed |
|---|---|---|---|
| UI-1 | Shell accessibility | Zero WCAG 2.2 AA (WCAG2ICT) failures from automated checks on every shell page in CI; a manual audit with NVDA, JAWS, VoiceOver and Orca each release; a published UAAG 2.0 conformance statement | Ship blocker |
| UI-2 | Keyboard completeness | 100% of shell actions reachable and operable by keyboard; F6 region cycling works; verified by a scripted keyboard-only walkthrough run under the deterministic substrate | Ship blocker |
| UI-3 | Honest labels are understood | For each label (protected media, Tor route, container chip, same-site JavaScript caveat, AI off, fingerprinting row), ≥ **80%†** of lab participants answer correctly what the label does and does not protect. Method follows the WWW 2018 private-browsing comprehension study | Reword and retest; a label that fails twice is redesigned |
| UI-4 | Onboarding completes | ≥ **90%†** of lab participants finish onboarding in ≤ **90 s†**; import from Chrome finishes in the background without blocking first use | Cut screens, starting with Containers |
| UI-5 | Prompts respect gestures | 0 permission prompts shown without user activation (engine-enforced, property test); notification grant and deny rates published from opt-in telemetry | Ship blocker for the first part |
| UI-6 | Nothing unrequested appears | Shell audit against the AG-0 UI-surface list: 0 AI surfaces, 0 promotions, 0 re-prompts after a "no", in fresh-install and upgrade scenarios | Ship blocker |
| UI-7 | Shell is responsive | p95 input-to-paint ≤ **16 ms†** for typing in the address bar and switching tabs, on the reference devices; shell first paint counted inside RES-1 | Profile the shell like any page; no new shell features until fixed |
| UI-8 | Containers are recognized | ≥ **90%†** of lab participants correctly name the current tab's container from the chip (name plus colour), including participants with colour-vision deficiency | Change chip design; never rely on colour alone |
| UI-9 | Warnings are heeded | Lab adherence to certificate interstitials ≥ the adherence measured for current stable Chrome's interstitials in the same study; for the phishing and malware interstitial the method and threshold are **PH-11** in the phishing protection spec, and the two studies do not double-count participants | Redesign the interstitial |
| UI-10 | Browser prompts cannot be spoofed | Property test: every browser-issued prompt, **including the phishing/malware interstitial**, is a broker-placed layer above the content layer, unoccludable by the page and broker-hit-tested; a red-team page suite cannot cover, imitate or relabel one; a separate owned prompt window conforms where the OS requires one | Ship blocker |
| UI-11 | Usability does not regress | System Usability Scale (SUS) on core tasks each stage. No published browser SUS benchmark exists, so the Stage 1 measurement is the baseline; later stages may not drop more than **5 points†** | Investigate the regressed tasks before shipping the stage |

---

## 6. Staging

| Stage | Surfaces |
|---|---|
| **1 (M4–M8)** | Main window, horizontal tabs, tab search and recently closed, address bar states (normal, Not secure, container chip, permission chip, protected media once M7 lands), site panel without the resource meter (with the Phishing protection and Certificates rows), permission prompts **except notifications**, interstitials, downloads with two-level warnings, onboarding, settings (all groups, with the AI group showing the off switch only), recovery window (five actions, R3), find in page, zoom, fullscreen, import, basic password manager, keyboard model, screen-reader support |
| **2** | Vertical tabs, split view, tab groups; resource meter; **notification permission, delivery, mute, snooze and digests** (R12); R2 repair inspector; JavaScript modes; container routes including the Tor label; save for offline and reader view; PDF editing |
| **3** | Agent surfaces (only with the AI switch on); embedding-API settings |

---

## 7. Open questions

- **Q-UI-1:** how do omnibox navigations split between typed, searched and suggested? No vendor publishes it, and it decides how much the address bar should favour search suggestions. Stays open; measured in the address-bar study at SH-M4 (shell spec §9.5).
- **Q-UI-2:** what is the right default for container names and count? **Resolved for Stage 1**: four named defaults — personal, work, banking, shopping — editable and skippable; UI-8 and UI-3 remain the first comprehension data.
- **Q-UI-3:** how should the protected-media label appear when a page mixes protected and unprotected media? **Deferred to Stage 2** as Q-SH-12; Stage 1 shows the per-origin protected state and the switch.
- **Q-UI-4:** what is the project name? The name appears in the UA token, the recovery window and onboarding (Section 13.2, owner decision). Unchanged.
- **Q-UI-5 (Q-PH-16):** who owns the lookalike-domain warning class? The recommendation is the shell (UI-10/SH-11), with the phishing spec supplying the URL formatter and no reputation claim; left open so the shell's phase 8 settles it.

## 8. Non-goals

The shell does not compete on tab-strip layout (UP5). It does not show trust badges, padlocks or brand colours on security states. It does not promote features, including AI. It does not re-prompt after a "no". It does not hide what a setting disables. It does not rely on colour alone for any state. It does not draw the phishing/malware interstitial itself: that surface belongs to the phishing protection spec (4.5).

## 9. Batch-4 amendments (21 October 2026)

The owner accepted the shell spec's decisions (SH-D1–SH-D30) and the phishing protection spec's decisions (PH-D1–PH-D15) on 21 October 2026. This design is owner-approved direction, not a spec, so the table records what changed here and what the shell spec now owns.

**Adopted unchanged into the shell spec:** UI R1 → SH-D4/D6/D7; R2 → SH-D5; R4 → SH-D16/SH-8; R5 (setting exposed only if the model ships); R6 (repair overlay and toggle stay with the engine, AX spec); R7 → SH-D26; R8 verbatim in the site panel's route row; R9 with the caveat stated; R10 with exact OS wording (Q-SH-12 for the mixed case); R11 → SH-D12 plus the broker's hit-test and timing; R13 as Stage 2 visibility only; R14 → SH-D29; R15 → SH-D23/SH-6; R16 → SH-D19; R17 as surfaces in shell 6.8/6.10/6.12; R18 → SH-D27 (PDF editing is PD's); R19 → SH-D21; R20 → SH-D24; R21 (the site panel explains engine behaviour; settings expose registry rows only).

**Amended here, with the shell spec's text governing:**
- **Prompts are separate owned OS windows** (SH-D2, SH-D3, 6.5). The trusted-region-inside-the-page model is withdrawn; UI-10 is the own-layer/no-overlap/broker-hit-test property (3, 4.1, 4.5, 5).
- **UI R3 recovery** becomes five actions (SH-D22): add "start with themes and extensions off" and "quit" (2, 6).
- **UI R12 notifications** are not offered in Stage 1 at all (SH-D17); Stage 2 adds delivery and controls (2, 6).
- **Site panel:** the padlock and shield language is replaced by the neutral site-controls button, and the panel gains the Phishing protection and Certificates rows (4.3).
- **Staging** follows shell 9.1: tab groups, palette, notifications and the resource meter are Stage 2; Stage 1 keeps the surfaces listed in 6.
- **UI-9** delegates the phishing study to PH-11. The §5 targets map to the shell's SH targets in order: UI-1 → SH-3; UI-2 → SH-4; UI-3 → SH-9/SH-13; UI-4 → SH-6; UI-5 → SH-7; UI-6 → SH-8; UI-7 → SH-2; UI-8 → SH-9; UI-9 → SH-10; UI-10 → SH-11; UI-11 → SH-12. Where thresholds differ, the stricter spec target governs (for example SH-9's 95%† over UI-8's 90%†).

**Open after batch 4:** Q-UI-1 (omnibox query mix, measured at SH-M4); Q-UI-3 (mixed protected media, Q-SH-12); Q-UI-4 (project name, owner); Q-UI-5/Q-PH-16 (lookalike class ownership); AX-D3/D4/D9/D11 placement and AT approval (pending, shell spec 9.5).

## 10. Phase 3 amendments (21 October 2026)

The owner accepted the component specs' Section 9.4/9.5 recommendations on 20–21 October 2026. These rows were queued for this design. This section records what changed here; each component spec's text governs the engine behaviour.

- **Resources (A-UI-1; R13).** The per-site resource meter becomes the **causal view** of the resource-footprint spec's 6.14: it names the top cause — heavy third-party script, a background timer loop, a runaway animation, a spinning worker, a retainer — from the same attribution data, and shows numbers rather than asking for them; the whole-browser view uses the same causes. The site panel gains the two outcome controls of FP-D15: **"keep this site running in the background"** (a declared reason) and **"limit this site on battery"** (hidden CPU at the floor, visible animation capped at 30 Hz† with user input exempt), the battery control shipping only after FP-30 shows the attribution it acts on is right. There are no numeric MB or CPU budgets for users to set.
- **System UI fonts (A-UI-2; 4.10).** As 4.10 now states: the OS tier's UI face on macOS and Windows, Izcal's sans alias face on Linux, the interim tier for scripts that face does not cover, and TX-16's rule for native-surface strings (fonts and text spec 9.5, R23).
- **Media (A-UI-3; R17, MD-D32).** The shell owns the native controls' shell parts and the fullscreen exit affordance with its security message; Picture-in-Picture and Document PiP are shell-owned windows with their own exit affordances; protected playback carries MD-D32's affordance; per-site autoplay and animation settings sit in the site panel; caption preferences (size, colour, opacity, edge) are in Settings > Media and DRM, with the OS style as the default for properties the page has not styled; a media-process crash is labelled a media crash, not a page crash; per-site media energy appears in the resource view.
- **Site panel and prompts (A-UI-4; GR-D11, GR-D28, GR-S11).** Suspended GPU contexts and unavailable WebGL or WebGPU get named states with a reason, never a blank canvas, and the WebGPU permission is a prompt. The prompt surface is a separate broker-owned OS layer (already 0.4, SH-D2/SH-D3). Reduce-motion, reduce-transparency and flash protection are shell settings that also enable the compositor's interventions.
- **Settings (A-UI-5; LY 9.5).** Settings > Appearance gains an **overlay-or-classic scrollbar** preference defaulting to the OS accessibility setting; **page-zoom levels are recorded environment inputs**, so a replay reproduces them; form-control pickers (select, date, colour) are positioned from `hg-layout-query` rectangles, never guessed.
- **Site panel (A-UI-6; CB-D13, CB-D19).** "Blocked here" counts **parties and organisations**, never requests. The controls are graduated, narrowest first: allow one party here; allow sign-in with a provider for this tab; hiding off here; protection off here. Turning protection off or reporting a break opens the report flow (a narrow fix and a consented report), and when a fix ships the panel offers to restore protection. User-added lists carry the **"recognizability"** label, because a unique list set distinguishes the user. The **Security interstitial** is the security list's warning surface, distinct from the phishing and malware interstitial (PH-D6, PH-12).
- **SVG, motion and editing surfaces (A-UI-8; SVG R14, AN-D10, editing 6.6–6.12).** **SVG:** the SV-D9 scriptless-document bar and its "run with script" action; play controls for SVG images and inline SMIL paused under reduce-motion, with the per-site animation setting to resume. **Motion:** Settings > Accessibility gains the motion level per site (R, S, X), the command and shortcut for level X, the offer to throttle rAF, and flash protection; the opt-in uniformity profile shows the warning that reduced motion stops working and that levels S and X are detectable. **Editing:** the Paste popup (a separate OS window at the pointer or caret, enabled after 1 s†, no default action); the autofill preview (a separate non-interactive OS window over the field); the UI-10 amendment allowing such in-page-area OS windows when page content cannot draw over them, tested with the red-team suite; the clipboard panel (memory only, nothing in private windows) and its notices; the command-text warning; find options (case, diacritics, whole word, regex); the spelling menu with "Add to dictionary"; the autofill chooser listing fields; a setting to stop caret blinking.

**No target changed.** Section 5's targets stand, and UI-1 to UI-11 map to the shell spec's SH targets as Section 9 records. A-UI-7 was applied in 0.4.
