# Independent Browser Engine Landscape (as of October 2026) and Historical Lessons on Web-Compat Cost

Research date: 2026-10-14. All data dated. "Secondary" marks claims that are not from the project's own site.

## Ladybird (Ladybird Browser Initiative): team, funding, Rust migration, WPT, alpha, performance

### Takeaway
Ladybird is a donation-funded nonprofit. It started with a $1M gift in 2024 and had roughly 8 paid full-time engineers in 2026. It passes about 2.09M WPT subtests (Aug 2026) and is midway through moving LibWeb from C++ to Rust. The first alpha (Linux and macOS) is still promised for 2026. The work left before alpha is mostly product work such as crash reporting, signed builds and auto-update, not engine work.

### Cited Findings
- Launched as a nonprofit in July 2024 with a $1 million gift from GitHub co-founder Chris Wanstrath. At launch Andreas Kling said there were 4 full-time engineers, with 3 more coming — [heise](https://heise.de/-9789840); [Simon Willison](https://simonwillison.net/2024/Jul/1/the-ladybird-browser-initiative)
- Team size: a May 2026 blog says the project has "8 paid full-time engineers". The official FAQ only says "small full-time engineering team", stable for now. Another forum source says 6, date unclear — [urandom.io (May 2026)](https://urandom.io/blog/2026-05-09-the-browser-with-no-ad-sales-department); [ladybird.org](https://ladybird.org/)
- Funding model: the project is "entirely funded by the generous support of companies and individuals". Sponsorships are unrestricted and give no roadmap control. Tiers: Platinum $100k/yr, Gold $50k, Silver $10k, Bronze $5k, Copper $1k. The goal is to keep 18 months of runway — [ladybird.org](https://ladybird.org/); [Aug 2026 newsletter](https://ladybird.org/newsletter/2026-08-31/)
- New sponsor in Aug 2026: Arnav Guleria, $5,000 — [Aug 2026 newsletter](https://ladybird.org/newsletter/2026-08-31/)
- WPT subtests passed:
  - Apr 2026: 2,067,263 (secondary) — [byteiota](https://byteiota.com/ladybird-browser-nears-alpha-rust-engine-and-private-browsing/)
  - Jul 2026: 2,078,912 → 2,079,020 (+108, "no major movement… given the nature of most of this month's work", i.e. the Rust port) — [Jul 2026 newsletter](https://ladybird.org/newsletter/2026-07-31/)
  - Aug 2026: 2,079,020 → 2,088,677 (+9,657). About 2,500 came from referrer-policy, which now passes 99% — [Aug 2026 newsletter](https://ladybird.org/newsletter/2026-08-31/)
- Rust migration in 2026:
  - May: the HTML parser was fully migrated to Rust and runs about 10% faster than the C++ parser — [May 2026 newsletter](https://ladybird.org/newsletter/2026-05-31/)
  - July: much of LibWeb's style and layout pipeline moved to Rust. Parts still call back into C++, and CSS parsing, the DOM and painting had not moved — [Jul 2026](https://ladybird.org/newsletter/2026-07-31/)
  - August: CSS parsing moved, using a per-grammar-family A/B mode in which C++ was deleted only after zero output mismatches. Computed style storage and computation are now owned by Rust. The painting pipeline (display-list recording, hit testing) was ported and the C++ paintables deleted. Rust and C++ now share one string representation — [Aug 2026](https://ladybird.org/newsletter/2026-08-31/)
- Alpha: "We've said the first alpha starts in 2026, and that's still the plan." Remaining work: crash reporting, signed macOS and Linux builds, auto-updates, a first-run experience, and a download location — [Aug 2026](https://ladybird.org/newsletter/2026-08-31/)
- Performance during Aug 2026 (Linux):
  - Speedometer 2: about 47 → about 64
  - Speedometer 3: about 2.5 → about 3.9
  - StyleBench: about 3.5 → about 83. On an M3 MBP: Firefox 232.8, Safari 129.5, Ladybird 116.3, Chrome 103.8
  - Minecraft4k: 10.2 → 14.5 FPS
  - [Aug 2026](https://ladybird.org/newsletter/2026-08-31/)

### Inferences
- Speedometer 3 at about 3.9 is far below mainstream browsers, which score in the tens. JS and DOM throughput, not conformance, looks like Ladybird's biggest remaining gap. StyleBench is already competitive.
- The WPT count is now at about 2.09M subtests and growing slowly. Most subtests that are easy to pass are already passing, and further gains come in feature-specific bursts (e.g. referrer-policy).
- The A/B "zero mismatch" method used for the Rust port is a reusable pattern for incremental rewrites.

### Gaps
- No official total budget or donation figures for 2025–26 were found.
- No WPT time series before 2026 was gathered this session. Older newsletters are at [ladybird.org/news](https://ladybird.org/news/).
- Whether the alpha shipped in Sep–Oct 2026 is unconfirmed. The September 2026 newsletter was not retrieved.

## Servo (Linux Foundation Europe): funding, WPT, releases, embedding

### Takeaway
Servo nearly died after Mozilla's 2020 layoffs: merged PRs fell from 1,188 in 2018 to 65 in 2022. It recovered under Igalia from 2023. In 2025 it merged 3,183 PRs from 146 contributors, and its WPT score rose from 48.2% to 61.6% (93.4% of the subtests it runs). It now ships monthly releases and has been on crates.io since April 2026. Funding is a mix of Igalia's own investment, Sovereign Tech Fund, NLnet grants, and about $5.7k/month in donations (Nov 2025).

### Cited Findings
- Merged PRs and average monthly contributors, by year:

  | Year | PRs | Avg. contributors/month |
  |---|---|---|
  | 2018 | 1,188 | 27.33 |
  | 2019 | 986 | 27.17 |
  | 2020 | 669 | 14.75 |
  | 2021 | 118 | 4.92 |
  | 2022 | 65 | 2.83 |
  | 2023 | 776 | 11.33 |
  | 2024 | 1,771 | 26.33 |
  | 2025 | 3,183 | 42.42 |

  Source: [Igalia, Servo 2025 Stats (14 Jan 2026)](https://blogs.igalia.com/mrego/servo-2025-stats/)
- Contributors in 2025: 146 total, 43 with ≥10 PRs, 8 with ≥100 PRs. Project roles: 5 admins, 17 TSC members, 25 maintainers. About 35k GitHub stars — [same](https://blogs.igalia.com/mrego/servo-2025-stats/)
- WPT in 2025: the score went from 48.2% to 61.6%. Subtests passed went from 1,396,647/1,998,146 (69.9%) to 1,866,247/1,998,146 (93.4%). Some of the jump is artificial: 188,281 subtests were gained in October from new tests alone. Servo still does not run all WPT tests, so its numbers differ from wpt.fyi — [same](https://blogs.igalia.com/mrego/servo-2025-stats/)
- A draft from a third party says the WPT pass rate rose from 30% to 62% over 2.5 years — [webtransitions.org (draft, Mar 2026)](https://webtransitions.org/servo-readiness/)
- Releases:
  - First release, Servo 0.0.1, was announced 20 Oct 2025 — [Igalia](https://blogs.igalia.com/mrego/servo-2025-stats/)
  - "Servo is now available on crates.io" (2026-04-13) was an initial crates.io release plus an LTS version (0.1.0) — [servo.org/blog](https://servo.org/blog/)
  - Servo 0.4.0 shipped in late July 2026, with releases at least monthly (secondary) — [Habr](https://habr.com/ru/articles/1065628/)
- 2026 monthly posts mention multiple windows, proxy support, embedding-API work, FreeBSD support, a new Android UI, SharedWorker, "real world compat", and user scripts — [servo.org/blog](https://servo.org/blog/)
- On 2026-09-15 Servo published "Your Donations at Work: One Year of Sponsored Servo Development", marking the first donation-funded role — [servo.org/blog](https://servo.org/blog/)
- Funding sources:
  - Igalia took over stewardship in 2023 and the project moved to Linux Foundation Europe — [Igalia](https://www.igalia.com/2025/10/09/Igalia,-Servo,-and-the-Sovereign-Tech-Fund.html)
  - Sovereign Tech Fund commission (Oct 2025) covers accessibility, the WebView API, and sustainability — [Igalia](https://www.igalia.com/2025/10/09/Igalia,-Servo,-and-the-Sovereign-Tech-Fund.html)
  - NLnet NGI0 Commons grant for Writing Modes, starting June 2026 — [NLnet](https://nlnet.nl/project/Servo-writingmodes/)
  - Donations of about $5.7k/month (Nov 2025). Tiers run from $100 to $10k per month — [Phoronix](https://www.phoronix.com/news/Servo-Sponsorship-Tiers)
- Embedding: Verso, the Servo-based browser and Tauri webview funded by NLnet NGI Assure in 2024, was reportedly archived in Oct 2025. Its last prebuilt release was v0.0.9 (Sep 2025) (secondary, unverified) — [youngju.dev (Jul 2026)](https://www.youngju.dev/blog/2026-07-17-servo-embedding-crates-io-verso-archived.en); [NLnet Verso](https://nlnet.nl/project/Verso/)

### Inferences
- Servo shows that losing a corporate sponsor can collapse an engine within 2 years: activity fell by about 95%. It also shows that consultancy stewardship plus public grants can revive one.
- WPT percentages depend heavily on which tests are run and on test-suite churn, so cross-engine comparisons need care.

### Gaps
- No official 2026 WPT figure was found. Check [servo.org/wpt](https://servo.org/wpt/) and wpt.servo.org.
- Current donation totals and the content of the Sep 2026 donations post were not retrieved (the URL guessed for it returned 404).

## Blitz (Dioxus Labs), Flow (Ekioh), and others

### Takeaway
Blitz is the clearest example of a "compose existing Rust crates" engine: it uses Stylo, Taffy, Parley, and Vello or vello_cpu. It is still experimental. Ekioh's Flow is a commercial clean-room, multithreaded, GPU-rendered engine that started in set-top boxes. It uses SpiderMonkey for JS and was still in beta as of 2023. Lightpanda (written in Zig) handles only the DOM and JS, with no rendering, and is aimed at automation and AI agents.

### Cited Findings
- Blitz components:
  - Stylo (CSS), Taffy (box layout: flex, grid, block), Parley (text and inline layout), Vello via WGPU for rendering. The `blitz-dom` crate does style, layout, and events, but not parsing or painting.
  - `dioxus-native` renders a Dioxus VirtualDom. `blitz` renders HTML strings, but without interactivity.
  - Rendering is abstracted through AnyRender, now in a separate repo. The Python package uses vello_cpu.
  - Sources: [Blitz README](https://cdn.jsdelivr.net/gh/dioxuslabs/blitz@main/README.md); [blitz-py](https://pypi.org/project/blitz-py/)
- Blitz is described as an "experimental native renderer" — [Dioxus docs](https://dioxuslabs.com/learn/0.7/beyond/project_structure/llms.txt)
- crates.io lists 14 releases of blitz-html — [lib.rs](https://lib.rs/crates/blitz-html)
- A secondary claim says the Dioxus team and Blitz/Taffy maintainers joined Cognition (unverified) — [beri.net](https://www.beri.net/article/cognition-dioxus-acquihire-blitz-taffy-crate-owners-kitesurf-dependency)
- Flow (Ekioh):
  - Not a fork. Uses SpiderMonkey for JS. Origins in set-top boxes, where Ekioh's SVG engine ran on millions of devices.
  - Layout is multithreaded and rendering is on the GPU.
  - In 2020 it rendered and interacted with Gmail "quite well" but struggled with many other sites.
  - Sources: [The Register (Dec 2020)](https://theregister.com/2020/12/29/flow); [Ekioh](https://ekioh.com/company); [Ekioh news, "Google Docs in a clean-room browser"](https://www.ekioh.com/all-news/)
- Flow was still in beta in 2023. A Raspberry Pi preview is free for personal use, and commercial use needs a license — [AlternativeTo](https://alternativeto.net/software/flow-web-browser-/about)
- Lightpanda: built from scratch in Zig, not a Chromium fork, and has no graphical rendering. It covers DOM, JS via V8, networking, and a CDP server, and works with Puppeteer and Playwright. The vendor claims it is about 11x faster and uses about 9x less memory than Chrome — [linuxiac](https://linuxiac.com/?p=209081); [themenonlab](https://themenonlab.blog/blog/lightpanda-headless-browser-ai-agents)

### Inferences
- Blitz shows that reusing Stylo, Taffy, and Parley makes CSS cascade, flex/grid, and text layout largely solved problems for a Rust engine. The remaining hard parts are JS integration, the long tail of DOM APIs, and site compatibility.
- Flow shows that a niche such as TV/STB UI can fund a decade-long clean-room engine. Even so, it reached "full web" compatibility only for targeted sites like Gmail and Google Docs.

### Gaps
- Not researched this session (pointers only, unverified): Sciter, Ultralight, WebKit ports (WPE/WebKitGTK), Kosmonaut, Goanna/Pale Moon, NetSurf, Lexbor, and the browser.engineering textbook.
- No WPT numbers were found for Blitz, Flow, or Lightpanda.
- Flow's 2025–26 status was not verified.

## AI-built engines (2025–2026)

### Takeaway
Cursor's FastRender (January 2026) is the headline case. About 2,000 concurrent agents ran for roughly a week and produced over 1M lines of Rust. Some reports say 3M+, and the figures conflict. Sites like GitHub, Wikipedia, and CNN rendered, but JS did not work. It is a research demo, not a production engine. Even this one reused Skia, HarfBuzz, Taffy, and QuickJS.

### Cited Findings
- Origins and timeline:
  - Wilson Lin started FastRender as a side project in Nov 2025, and it became a Cursor research project.
  - At peak about 2,000 concurrent agents ran for about one week, producing nearly 30,000 commits.
  - Models used: Claude Opus 4.5, GPT-5.1, GPT-5.2.
  - Source: [Simon Willison (23 Jan 2026)](https://simonwillison.net/2026/Jan/23/fastrender/)
- Code size: over 1M lines of Rust ([Simon Willison](https://simonwillison.net/2026/Jan/23/fastrender/)). Other coverage cites over 3M lines and GPT-5.2-Codex ([Medium](https://medium.com/write-a-catalyst/ai-agents-built-a-web-browser-in-one-week-and-that-should-make-us-pause-1be4fab67d03)). The figures conflict, and the Medium piece is lower quality.
- Dependencies chosen by the agents: Skia, HarfBuzz, Taffy (vendored and modified), and QuickJS as a stopgap while a home-grown JS engine (ecma-rs) was built. Specs such as csswg-drafts, ecma262, and whatwg-html were vendored as submodules — [Simon Willison](https://simonwillison.net/2026/Jan/23/fastrender/)
- Quality: JS was disabled because the engine did not work. GitHub, Wikipedia, and CNN loaded slowly but were usable. Feedback loops were screenshot diffs against golden images plus compiler checks — [same](https://simonwillison.net/2026/Jan/23/fastrender/)
- A GitHub issue requests WPT coverage for FastRender, suggesting WPT conformance was not established — [GitHub issue #108](https://github.com/wilsonzlin/fastrender/issues/108)

### Inferences
- AI agents can quickly produce code that renders static pages. Their verification relied on screenshots and the compiler, not on conformance testing. A first-principles team in 2026 should use WPT as the agents' oracle.

### Gaps
- No WPT score was found for FastRender.
- Other AI-built engine claims (e.g. "Moli" on dev.to) were seen only in snippets and not verified.

## Historical lessons: Presto, EdgeHTML, Servo 2020, platform size, Interop, headcount

### Takeaway
Two well-funded vendors abandoned their engines, Opera in 2013 and Microsoft in 2018, and both said web compatibility and duplicated effort were the reason. The web platform is now about 2M WPT subtests. The browser vendors' yearly Interop effort shows that even the big three spend years converging on the same features.

### Cited Findings
- Opera announced in Feb 2013 that it would replace Presto with WebKit. CTO Håkon Wium Lie: "it makes sense for our engineers to focus on what's valuable rather than building something in parallel." — [TidBITS](https://tidbits.com/2013/05/30/opera-releases-chromed-preview-of-next-browser/); [css3.info](https://css3.info/?p=2026)
- In April 2013 Opera followed Google's Blink fork. Opera 15, the first Blink-based version, shipped 28 May 2013, and the last Presto release was 12.16 — [TNW](https://thenextweb.com/insider/2013/04/04/opera-confirms-it-will-follow-google-and-ditch-webkit-for-blink-as-part-of-its-commitment-to-chromium/); [itc.ua](https://itc.ua/articles/opera-software-pochemu-myi-pereshli-na-blink-i-chto-budet-dalshe/amp/)
- Microsoft (Dec 2018) moved Edge from EdgeHTML to Chromium. The stated goals were better web compatibility and "less fragmentation" for developers, because sites are optimized for Chrome. Mozilla responded with "Goodbye, EdgeHTML" — [9to5Google](https://9to5google.com/2018/12/06/microsoft-confirms-chromium-edge/); [Redmond Mag](https://redmondmag.com/articles/2018/12/06/microsoft-edge-shifting-to-chromium.aspx); [Gigazine](https://gigazine.net/gsc_news/en/20181207-goodbye-microsoft-edge)
- Servo after the 2020 Mozilla layoffs: 65 PRs and about 2.8 contributors/month in 2022 — [Igalia](https://blogs.igalia.com/mrego/servo-2025-stats/)
- Size of WPT: Servo's denominator was 1,998,146 subtests (end of 2025), and Ladybird passes 2,088,677 (Aug 2026). The full WPT suite is therefore more than 2M subtests — [Igalia](https://blogs.igalia.com/mrego/servo-2025-stats/); [Ladybird](https://ladybird.org/newsletter/2026-08-31/)
- Interop 2026 (announced 12 Feb 2026, the fifth year), run by Apple/WebKit, Google, Igalia, Microsoft, and Mozilla:
  - Scope: 19 focus areas, 3 cleanup areas, 4 investigation areas.
  - Features include advanced attr(), container style queries, scroll-driven animations, anchor positioning, view transitions, the Navigation API, scoped custom element registries, WebRTC, WebTransport, and JSPI.
  - A "web compat" area targets real-world breakage such as ESM loading and unprefixing `-webkit-user-select`.
  - Sources: [WebKit](https://webkit.org/blog/17818/announcing-interop-2026); [web.dev](https://web.dev/blog/interop-2026); [Igalia](https://igalia.com/news/interop-2026.html)
- Chromium headcount proxies:
  - In 2019, about 1,800–1,900 committers in one year, from emails at chromium.org or google.com (rough HN method) — [HN/bzbarsky thread](https://hn.svelte.dev/item/22059393)
  - In 2024, among non-Google contributors, Microsoft made 37.26% of commits, Igalia 17.94%, and Intel 12.41% — [Brian Kardell, Igalia](https://bkardell.com/blog/2024-Midseason.html)

### Inferences
- The lesson is about scale. Organizations with hundreds of engineers (Opera, Microsoft) concluded that compatibility parity was not worth it. Independent engines that survive either pick a niche (Flow on STB, Blitz for app UI, Lightpanda for headless) or reuse heavily (Stylo, Skia, HarfBuzz, SpiderMonkey/V8/QuickJS) and use WPT as the target.
- "-webkit-" prefix unprefixing and ESM-loading compat items show that the long tail goes beyond specs and into de facto Chromium/WebKit behavior.

### Gaps
- No credible current figures were found for Apple's WebKit team size, Google's Chromium headcount or spend, or Gecko market share in 2026.
- Acid test results for the new engines were not gathered.
- Interop 2026 mid-year scores were not retrieved. See [wpt.fyi/interop-2026](https://wpt.fyi/interop-2026).
