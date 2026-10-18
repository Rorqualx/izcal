# AI-Native and Agentic Browsing (state as of October 14, 2026) and what it demands from a browser engine

Status labels: **[shipped]** = generally available to users; **[prototype]** = flag, origin trial, early access, or beta; **[research]** = paper or proposal only. Dates are publication or event dates. Many 2026 secondary sources (comparison and review blogs) contradict each other. They are flagged where used. Primary vendor sources are preferred.

## 1. Products: who ships agentic browsing, on what engine, and with what traction

### Takeaway
Agentic browsing has moved away from standalone "AI browsers" and into incumbent browsers and assistant apps. OpenAI shut down ChatGPT Atlas on Aug 9, 2026, and Edge Copilot Mode was reportedly folded back into Edge in May 2026. Chrome's Gemini auto browse, the Brave/Comet-style Chromium forks, and extensions are now the main delivery channels. Every notable product is built on Chromium. None ships a new engine.

### Cited Findings
- **ChatGPT Atlas (OpenAI)** [shipped, then retired]. It was a Chromium-based browser. OpenAI's engineering post describes "OWL" (OpenAI's Web Layer), which runs Chromium's browser process outside the main Atlas app process. The app is the OWL client and Chromium is the OWL host. — [OpenAI, Building ChatGPT Atlas](https://openai.com/index/building-chatgpt-atlas)
  - Per a third-party write-up, the client and host talk over Mojo (Chromium IPC), with custom Swift/TypeScript bindings. The Swift client exposes Session/Profile/WebView abstractions. — [ByteByteGo](https://blog.bytebytego.com/p/the-architecture-behind-atlas-openais)
  - OpenAI's stated reason was that Chromium "can take hours to check out and build," and it wanted engineers shipping on day one. — [OpenAI](https://openai.com/index/building-chatgpt-atlas)
  - On July 9, 2026, OpenAI said it is "deprecating Atlas as we bring browser-based agentic capabilities into ChatGPT and Codex." Atlas was scheduled to stop working Aug 9, 2026, and browser data does not migrate. — [OpenAI ChatGPT release notes](https://help.openai.com/en/articles/6825453-chatgpt-release-notes); [OpenAI community notice](https://community.openai.com/t/atlas-is-scheduled-to-stop-working-on-august-9-2026/1389473); [TechCrunch, 2026-07-09](https://techcrunch.com/2026/07/09/openai-is-shutting-down-atlas-but-its-ai-browser-ambitions-are-still-growing/); [MacRumors, 2026-07-10](https://www.macrumors.com/2026/07/10/openais-chatgpt-atlas-browser-shutting-down/)
  - The replacements are a built-in browser in the ChatGPT desktop app (it can visit sites, log in, and download files), a cloud browser running on OpenAI servers, and a Chrome extension. — [TheNextWeb](https://thenextweb.com/news/openai-chatgpt-atlas-browser-shutdown-superapp); [TechCrunch](https://techcrunch.com/2026/07/09/openai-is-shutting-down-atlas-but-its-ai-browser-ambitions-are-still-growing/)
  - TechCrunch links the shutdown to an internal push to cut "side quests" (Sora was also shut down). — [TechCrunch](https://techcrunch.com/2026/07/09/openai-is-shutting-down-atlas-but-its-ai-browser-ambitions-are-still-growing/)
  - Atlas launched on macOS in October 2025 (Wikipedia summary; I did not open it to confirm the exact date). — [Wikipedia: ChatGPT Atlas](https://en.wikipedia.org/wiki/ChatGPT_Atlas)
- **Chrome + Gemini "auto browse" (Google)** [shipped, paid tiers]. It launched Jan 28, 2026 on desktop (macOS, Windows, Chromebook Plus), built on Gemini 3, for US Google AI Pro/Ultra subscribers.
  - Gemini scrolls, clicks, and types. The tab glows and a side panel shows each step. Users can take over at any time.
  - It can use Google Password Manager only after the user authorizes it, and it pauses before purchases and social posts. — [blog.google (Parisa Tabriz)](https://blog.google/products-and-platforms/products/chrome/gemini-3-auto-browse/); [9to5Google, 2026-01-28](https://9to5google.com/2026/01/28/chrome-gemini-auto-browse/); [MobileSyrup](https://mobilesyrup.com/2026/01/28/new-gemini-in-chrome-brings-auto-browse-and-side-panel/)
  - At I/O (May 19, 2026), Google announced auto browse on Android for "late June," Gemini in Chrome for Android (devices with 4GB+ RAM, US English), and "Skills in Chrome" (saved multi-step prompts). — [Chrome for Developers, I/O 2026](https://developer.chrome.com/blog/chrome-at-io26)
  - One secondary guide claims a full US Android rollout on Aug 18, 2026, capped at 20 tasks/day (Pro) and 200/day (Ultra). This is unverified. — [nohacks.co](https://nohacks.co/blog/agentic-browser-landscape-2026)
  - Project Mariner (the research prototype behind auto browse) was reportedly discontinued in May 2026 and folded into Gemini Agent and auto browse. This comes from a single secondary source and is unverified. — [searchviu](https://www.searchviu.com/en/ai-browsers-2026-compared/)
- **Perplexity Comet** [shipped]. It is a Chromium-based browser.
  - Secondary sources say it has been free since October 2025, with the autonomous "Background Assistants" behind the $200/mo Max tier, and that it reached iOS in March 2026. These claims are unverified. — [searchviu](https://www.searchviu.com/en/ai-browsers-2026-compared/)
  - **Legal:** On March 9, 2026, Judge Maxine Chesney (N.D. Cal.) granted Amazon a preliminary injunction. She found Amazon likely to succeed under the CFAA and California computer-fraud law. The order bars Comet agents from Amazon's protected systems even through user accounts, and requires deletion of collected data.
    - The court separated *user permission* from *platform authorization*.
    - Amazon alleged that Comet disguised its agent as an ordinary Chrome session and pushed an update within 24 hours of being blocked in August 2025.
    - Perplexity appealed to the 9th Circuit on March 10, 2026. — [GeekWire, 2026-03-10](https://www.geekwire.com/2026/03/10/judge-blocks-perplexitys-ai-bot-from-shopping-on-amazon-in-early-test-of-agentic-commerce/); [Decrypt](https://decrypt.co/360629/amazon-perplexity-comet-court-order-agentic-commerce); [PaymentExpert, 2026-03-19](https://paymentexpert.com/2026/03/19/amazon-secures-injunction-perplexity/)
- **Microsoft Edge Copilot Mode** [shipped 2025, reportedly retired as a separate mode in May 2026]. Secondary sources say "Copilot Mode" was retired as a product in May 2026 and its features (sidepane, Journeys, actions with user approval) were folded into Edge, free of charge. One review rates Edge's task completion as "Limited." I found no Microsoft primary source. — [nohacks.co](https://nohacks.co/blog/agentic-browser-landscape-2026); [windowsforum](https://windowsforum.com/threads/best-ai-browsers-in-2026-edge-chrome-brave-opera-arc-dia-and-comet.421079/)
- **Dia (The Browser Company, now Atlassian)** [shipped, limited platforms].
  - Atlassian agreed in September 2025 to buy The Browser Company for $610M in cash. TBC stays a separate company focused on Dia, and Arc development has ended. CEO Josh Miller and CTO Hursh Agrawal stay on. The plan is to connect Dia with Jira and Trello. — [heise](https://heise.de/-10632710); [TechCentral.ie](https://www.techcentral.ie/atlassian-buys-the-browser-company-for-610m/); [AlternativeTo](https://alternativeto.net/news/2025/9/atlassian-acquires-the-browser-company-creator-of-the-arc-and-dia-ai-powered-browsers)
  - Reported closing dates conflict (end of 2025 vs Q1 2026), and I found no closing announcement.
  - Secondary reviews conflict on platform (macOS only vs Windows expanding) and on agent depth. — [toolchase](https://toolchase.com/blog/best-ai-browsers-2026/); [commercepundit](https://www.commercepundit.com/blog/best-ai-browsers/)
- **Opera Neon** [shipped, paid agent layer]. Sources conflict.
  - One says $19.90/mo for the agent layer.
  - Another says a free plan arrived in August 2026 that lets *external agents drive it*, while Opera's own AI stays paid. If accurate, that is notable: the browser is positioned as an agent runtime.
  - All of these are unverified secondary sources. — [sigmabrowser](https://www.sigmabrowser.com/blog/what-is-an-agentic-browser-best-agentic-browsers-in-2026); [clawnewbie](https://clawnewbie.com/reviews/best-ai-browser-agents-2026); [firecrawl](https://www.firecrawl.dev/blog/best-browser-agents)
- **Brave Leo agentic "AI browsing"** [prototype, early testing in all channels].
  - It launched in Nightly in December 2025. Brave's blog was updated May 5, 2026: "Leo Agentic browsing has migrated from Brave Nightly, and is now available for early testing in all release channels." — [Brave blog](https://brave.com/blog/ai-browsing/); [Privacy Guides, 2025-12-12](https://www.privacyguides.org/news/2025/12/12/brave-adds-experimental-agentic-ai-browsing-feature/)
  - The agent runs in a separate browser profile, with no shared cookies, logins, caches, or site data.
  - A second model reviews planned actions against the user's request. The feature must be manually invoked.
  - The agent is blocked from non-HTTPS pages, Safe-Browsing-flagged sites, extension pages, and brave:// pages. The AI cannot delete session logs.
  - Brave says these safeguards "do not eliminate risks such as prompt injection." — [Brave support](https://support.brave.app/hc/en-us/articles/41240379376909); [CyberInsider](https://cyberinsider.com/brave-begins-early-testing-of-privacy-focused-ai-browsing-mode/)
- **Claude for Chrome (Anthropic extension)** [shipped]. One secondary source, citing the Chrome Web Store, claims about 40K installs in December 2025 and more than 10M by June 2026. This is a single source and unverified. — [searchviu](https://www.searchviu.com/en/ai-browsers-2026-compared/)
- **Safari:** One secondary source says Apple showed its first agentic Safari feature at WWDC 2026. This is unverified and has no details. — [searchviu](https://www.searchviu.com/en/ai-browsers-2026-compared/)

### Inferences
- Every product above is either a Chromium fork (Atlas, Comet, Dia, Neon, Brave) or Chrome/Edge itself. Atlas is the only one with a published architecture change, and that change is process isolation of Chromium from the app (OWL), not engine changes. A first-principles engine has no direct competitor at the engine layer. Its pitch has to be capabilities Chromium forks can't easily add.
- The market is consolidating toward agentic features inside the incumbents (Chrome, Edge) and assistant apps with an embedded or cloud browser (ChatGPT desktop, Claude extension). The standalone "AI browser" is shrinking as a category. Being an *agent runtime* (embeddable, headless, cloud-hostable, and drivable by third-party agents, as Opera Neon reportedly is) may matter more than being a consumer browser.
- The Amazon v. Perplexity injunction shows that sites want to tell agent traffic from human traffic, and that disguising agents as normal Chrome is legally risky. An engine should support honest agent identification natively, for example a distinct UA token or signed agent requests. That is my inference, not a sourced standard.

### Gaps
- No primary-source adoption numbers (MAU/DAU) for Comet, Dia, Neon, or auto browse. Atlas usage figures before shutdown were not found.
- No Microsoft primary source on Copilot Mode's 2026 status. Edge's agent architecture is undocumented.
- No primary confirmation of the Mariner discontinuation, Comet's iOS release, Opera Neon pricing, or Safari's WWDC agent feature.
- Whether the Atlassian/TBC deal closed was not confirmed.

## 2. Agent interfaces: how agents perceive and act on pages, and how well it works

### Takeaway
Production agents mix screenshots (vision models), DOM and accessibility-tree snapshots, and synthesized input events through CDP/Playwright-style automation. This is slow, token-heavy, and brittle. The industry answer is site-declared tools: WebMCP, now in a Chrome 149 origin trial. Benchmarks show large gains (WebArena about 14% to about 74%, OSWorld about 12% to about 83% from 2024 to 2026), but results swing 20–30 points depending on the scaffold. That makes the observation/action interface a first-order variable.

### Cited Findings
- **WebMCP** [prototype, origin trial].
  - The spec draft lives in the W3C Web Machine Learning CG repo, first published Aug 13, 2025. Original authors are from Microsoft (Walderman, Lee, Nolan) and Google (Bokan, Sagar, Van Opstal), and Dominic Farolino drove later drafts. — [github.com/webmachinelearning/webmcp](https://github.com/webmachinelearning/webmcp)
  - **API:** `document.modelContext.registerTool({name, description, inputSchema, execute}, {signal})`. Aborting the signal unregisters the tool.
    - `exposedTo` shares tools with listed origins. `getTools({fromOrigins})` and `executeTool()` support cross-origin discovery and execution, mediated by the browser.
    - A `toolchange` event fires when tools are added or removed.
    - A declarative `<form>`-based tool synthesis is only exploratory and not in the spec. — [webmcp repo](https://github.com/webmachinelearning/webmcp)
  - **Security model:** WebMCP is on by default in top-level and same-origin frames. Cross-origin iframes need `allow="tools"`, otherwise `NotAllowedError`. By default, tools are exposed to the owner, same-origin documents, and built-in agents. Open questions include user confirmation for sensitive tools and whether built-in agents get tools by default. — [webmcp repo](https://github.com/webmachinelearning/webmcp)
  - **Relation to MCP:** It borrows MCP vocabulary (tools, JSON schemas) but does not adopt the MCP spec, because MCP lacks origins, permissions, and DOM integration. It complements backend MCP. — [webmcp repo](https://github.com/webmachinelearning/webmcp)
  - **Rationale:** Screenshot, DOM-snapshot, and simulated-click agents are "slow and brittle," and agents fall back to generic automation when no tool exists. — [webmcp repo](https://github.com/webmachinelearning/webmcp)
  - **Chrome status:** At I/O on May 19, 2026, Google said the experimental WebMCP origin trial starts in Chrome 149 and "Gemini in Chrome will soon support WebMCP APIs." — [Chrome for Developers, I/O 2026](https://developer.chrome.com/blog/chrome-at-io26)
    - Secondary sources say the trial runs Chrome 149–156, ending about Nov 17, 2026. They conflict on the start date (June 2 vs June 9, 2026). — [ppc.land](https://ppc.land/chrome-149-origin-trial-puts-webmcp-in-developers-hands-at-last/); [InfoQ, 2026-06](https://www.infoq.com/news/2026/06/webmcp-web-agent-standard-chrome/)
    - Unverified claims: the getter moved from `navigator.modelContext` to `document.modelContext`, with the old name deprecated in Chrome 150, and Shopify and Cloudflare enabled WebMCP by default in August 2026. The repo's current use of `document.modelContext` is consistent with the first claim. — [openhermit blog](https://www.openhermit.com/blog/webmcp-chrome-149-origin-trial-production-2026)
- **Chrome DevTools for agents** [shipped]. It gives coding agents access to console logs, network traffic, and **accessibility trees**, and is "available today" for Antigravity and 20+ coding agents. This shows Chrome exposing the a11y tree as an agent-facing interface. — [Chrome for Developers, I/O 2026](https://developer.chrome.com/blog/chrome-at-io26)
- **Observation cost:** BrowserGym exposes the accessibility tree, raw HTML, and screenshots together. A single observation can exceed 20,000 tokens, which motivates "signal-driven" observation pruning for long-horizon agents. — [arXiv 2606.06708](https://arxiv.org/pdf/2606.06708)
- **WALT (Web Agents that Learn Tools)** [research]. It reverse-engineers site functionality into reusable tools instead of acting step by step on raw UI. This is an academic parallel to WebMCP. — [arXiv 2510.01524](https://arxiv.org/pdf/2510.01524)
- **Benchmarks (as compiled in May 2026):**
  - OSWorld rose from 12.2% to 82.6% (top: Holo3-35B-A3B), at or above the 72.4% human baseline.
  - WebArena rose from 14.4% to 74.3% (top: DeepSeek v3.2 + WebTactix).
  - These figures come from a single survey's table. — [AgentAtlas, arXiv 2605.20530](https://arxiv.org/html/2605.20530v1)
- **Harness sensitivity:** WebArena results vary 20–30 points depending on the scaffold, and OSWorld vendor self-reports are not comparable to public submissions. — [benchmarkingagents.com](https://benchmarkingagents.com/best-benchmarks-for-browser-agents/); [benchmarkingagents.com/osworld](https://benchmarkingagents.com/osworld/)
- **Modality split:** Rough benchmark mapping: WebArena for DOM/a11y agents, VisualWebArena for screenshot agents, OSWorld for full desktop. — [benchmarkingagents.com](https://benchmarkingagents.com/best-benchmarks-for-browser-agents/)
- **OSWorld 2.0** [research, June 2026]: 108 long-horizon workflows. Each takes humans a median of about 1.6 hours and needs about 318 tool calls with Claude Opus 4.7. Saturated short-task benchmarks are giving way to long-horizon ones. — [arXiv 2606.29537](https://arxiv.org/html/2606.29537v1)
- **Evaluation in the wild:** Emergence WebVoyager argues for consistent, transparent evaluation on live sites, because live-web results are poorly reproducible. — [arXiv 2603.29020](https://arxiv.org/pdf/2603.29020)

### Inferences
- There are three tiers of agent interface, in increasing reliability and decreasing coverage:
  1. Pixels plus synthetic input (works anywhere, costs the most, and is vulnerable to visual injection).
  2. Semantic tree (DOM/a11y) plus element-targeted actions (cheaper, but depends on authoring quality).
  3. Site-declared tools (WebMCP; reliable, but needs site opt-in).

  A new engine should treat all three as first-class, with a unified action model and fallbacks between tiers.
- Token cost per observation (often over 20K tokens) and the 20–30 point scaffold variance mean an engine can measurably improve agent success by producing compact, stable, diffable semantic snapshots. That kind of improvement is hard to bolt onto Chromium via CDP.
- WebMCP's registry is per-document, mediated by the browser, and origin-scoped. That puts it squarely in engine/runtime territory: tool registry, permissions-policy `tools`, cross-origin mediation, and the `toolchange` lifecycle.

### Gaps
- I found no matched-conditions study comparing a11y-tree and screenshot accuracy in 2026.
- No reliable per-task cost or latency numbers for production agents (auto browse, Comet, ChatGPT agent).
- I did not research these in this pass: llms.txt adoption, WebDriver BiDi status for agent use, Playwright MCP's a11y-snapshot approach, and BrowseComp and Mind2Web 2026 numbers.

## 3. Security: indirect prompt injection and the mitigations vendors propose

### Takeaway
Indirect prompt injection is accepted as unsolved. OpenAI frames it as an ongoing arms race, and Brave calls it "a systemic challenge facing the entire category." Mitigations converge on five architectural moves:
- a privileged critic/planner split, where the trusted model never sees raw web content
- per-task origin allowlists that extend site isolation
- isolated agent profiles
- deterministic confirmation gates for sensitive sites, credentials, and payments
- parallel injection classifiers

### Cited Findings
- **Brave on Comet:** Brave disclosed a Comet injection in August 2025. In October 2025 it showed "unseeable" injections: faint text (light blue on yellow) in screenshots is OCR-extracted and passed to the LLM as if it were part of the user's query. Brave found the issue Oct 1, reported it to Perplexity Oct 2, and published Oct 21, 2025.
  - Brave called it "a systemic challenge facing the entire category of AI-powered browsers" and withheld a vulnerability in another browser at the vendor's request.
  - Brave recommends isolating agentic from regular browsing and starting agent actions only on explicit user invocation. — [Brave blog](https://brave.com/blog/unseeable-prompt-injections/); [Simon Willison](https://simonwillison.net/2025/Oct/21/unseeable-prompt-injections/)
- **OpenAI, Atlas hardening (Dec 22, 2025):** Automated red-teaming powered by reinforcement learning found a new class of injections. OpenAI shipped an adversarially trained model with stronger safeguards, and compared injection to ever-evolving online scams that need continuous defense. — [OpenAI](https://openai.com/index/hardening-atlas-against-prompt-injection)
  - CyberScoop headlined that injection may never be "solved" for browser agents. I could not verify the exact OpenAI sentence. — [CyberScoop](https://cyberscoop.com/?p=87276)
- **Google, Chrome agent security architecture (Dec 8, 2025, Nathan Parker):** — [blog.google](https://blog.google/security/architecting-security-for-agentic/); [The Hacker News](https://thehackernews.com/2025/12/google-adds-layered-defenses-to-chrome.html)
  - **Threat model:** Injection can come from malicious sites, third-party iframes, or UGC, and can lead to unwanted transactions or data exfiltration.
  - **User Alignment Critic:** A separate Gemini model checks each action the planner proposes. It **sees only metadata about the action, not raw web content**, so the content can't poison it. It vetoes and sends feedback, and repeated failures return control to the user. It is inspired by the dual-LLM pattern and DeepMind's CaMeL.
  - **Agent Origin Sets:** Google states that an unrestricted agent would effectively be a **Site Isolation bypass**. A trusted gating function decides which origins are relevant to the task, and the planner can't add origins without its approval.
    - Origins are split into **read-only** and **read-write** sets. Content from origins outside the set is not shown to the model.
    - Data may flow from read-only to writable origins only.
    - New navigations are checked by a Critic variant. Model-generated URLs are checked deterministically against known public URLs.
    - At initial rollout, only the read-writable set is enforced.
  - **Confirmations:** Deterministic confirmations are required before navigating to sensitive sites (banking, medical), before signing in via Password Manager (the model never sees passwords), and before purchases, payments, or sending messages. A work log lets the user pause, take over, or stop.
  - **Classifier and red-teaming:** A prompt-injection classifier runs on every page in parallel with planner inference. Automated red-teaming generates malicious sandboxed sites, prioritizing UGC and ads. The VRP pays up to $20,000 for agent security-boundary breaches.
- **Brave's own agent design:** isolated profile, a second model that reviews actions, explicit invocation only, blocked sensitive and internal pages, and logs the AI can't delete. Brave still warns the risk remains. — [Brave support](https://support.brave.app/hc/en-us/articles/41240379376909); [Brave blog](https://brave.com/blog/ai-browsing/)
- **Credentials and payments in Chrome auto browse:** Password Manager use requires user authorization, and the agent pauses before purchases and social posts. — [9to5Google](https://9to5google.com/2026/01/28/chrome-gemini-auto-browse/); [MobileSyrup](https://mobilesyrup.com/2026/01/28/new-gemini-in-chrome-brings-auto-browse-and-side-panel/)
- **Authorization and legal exposure:** The Amazon v. Perplexity preliminary injunction found that user consent does not equal site authorization under the CFAA. — [Decrypt](https://decrypt.co/360629/amazon-perplexity-comet-court-order-agentic-commerce)
- **WebMCP's own security posture:** permission-policy gating for cross-origin frames, explicit `exposedTo`, and secure-context-only discovery. Confirmation for sensitive tools is an open issue. — [webmcp repo](https://github.com/webmachinelearning/webmcp)

### Inferences
- The same-origin policy protects sites from each other's *code*, but an agent is a single principal that reads every origin and acts on every origin. That is why Google calls an unconstrained agent a Site Isolation bypass. A new engine should treat the agent as a distinct principal and give it its own capability set: per-task origin read/write sets with information-flow rules. It should not be a privileged extension of the user.
- Each mitigation needs engine primitives:
  - Origin-tagged content provenance, so the engine knows which origin produced each a11y/DOM node or pixel region given to the model. This is needed to enforce "content from non-set origins is not shown to the model" and to defend against iframe/ad injection.
  - Isolated agent storage partitions (profiles).
  - Trusted UI for confirmations that the page and the model can't spoof.
  - Deterministic sensitive-action detection: payment forms, credential fields, navigations to sensitive categories.
  - Tamper-proof action logs.
- Brave's attack shows that text invisible to humans still reaches the model. Engines could compute "human-visible" flags (contrast, size, occlusion, off-screen) per text node in the semantic snapshot so agents can ignore or flag low-visibility text. This is a proposal, not shipped anywhere I found.

### Gaps
- No quantitative attack-success-rate figures were found from Google or OpenAI for their production agents in this pass.
- Perplexity's own mitigation architecture for Comet was not found in primary sources.
- I did not cover academic benchmarks for agent injection (for example AgentDojo or WASP-style evaluations).

## 4. Engine implications: capabilities a first-principles engine should provide for agents

### Takeaway
Chrome is already turning agent-facing outputs into product features: the a11y tree in DevTools-for-agents, the WebMCP tool registry, a stable Prompt API for on-device models, and agent origin sets built on Site Isolation. A new engine can make these core rather than bolted on:
- a high-quality, provenance-tagged, deterministic semantic snapshot
- a native tool registry
- an agent principal with per-task origin capabilities
- headless/parallel and cloud-hostable sessions
- rendering pipelines that can skip pixels when no one is looking

### Cited Findings
- **On-device AI in Chrome:** The Prompt API is **stable in Chrome 148** (multimodal input, structured output, reliable JSON). Google announced "Gemma 197M," a small expert model for task-specific APIs like summarization. Translator and Language Detector APIs are available, and models are shared across sites to avoid per-site download costs. The I/O 2026 post does not mention Gemini Nano by name. — [Chrome for Developers, I/O 2026](https://developer.chrome.com/blog/chrome-at-io26)
- **Agent-facing introspection:** Chrome DevTools for agents exposes a11y trees, console, and network to coding agents [shipped]. — [Chrome for Developers, I/O 2026](https://developer.chrome.com/blog/chrome-at-io26)
- **Tool exposure as an engine API:** the WebMCP `document.modelContext` registry, the `tools` permissions-policy feature, and browser-mediated cross-origin execution [prototype]. — [webmcp repo](https://github.com/webmachinelearning/webmcp)
- **Cloud/remote browsers as an agent substrate:** OpenAI's Atlas replacement includes a cloud browser on OpenAI servers where agents complete tasks remotely. — [TheNextWeb](https://thenextweb.com/news/openai-chatgpt-atlas-browser-shutdown-superapp)
- **Embedding architecture:** Atlas's OWL decoupled the app from Chromium via an out-of-process host with Mojo bindings, exposing Session/Profile/WebView abstractions. This suggests agent products want a clean embedding API, multiple profiles, and fast startup. — [OpenAI](https://openai.com/index/building-chatgpt-atlas); [ByteByteGo](https://blog.bytebytego.com/p/the-architecture-behind-atlas-openais)
- **Observation size problem:** Combined a11y, HTML, and screenshot observations exceed 20K tokens, and research focuses on pruning observations. — [arXiv 2606.06708](https://arxiv.org/pdf/2606.06708)
- **Credential mediation:** Chrome's "Immediate UI mode" unifies passwords and passkeys into one browser-managed sign-in flow, and Chrome's agent design keeps passwords away from the model. Browser-mediated credentials let agents sign in without seeing secrets. — [Chrome for Developers, I/O 2026](https://developer.chrome.com/blog/chrome-at-io26); [blog.google](https://blog.google/security/architecting-security-for-agentic/)
- **Isolated agent profile as a product pattern** (Brave): separate cookies, cache, and logins. — [Brave blog](https://brave.com/blog/ai-browsing/)

### Inferences (design recommendations for a new engine, not sourced facts)
1. **Semantic tree as a first-class output.** Produce an accessibility/semantic tree in the core pipeline, alongside layout. It should carry:
   - stable node IDs across mutations
   - the origin and frame provenance of every node
   - visibility and contrast flags (as an injection defense)
   - actionable affordances (role, name, state, bounding box)

   Provide incremental diffs, so the agent pays only for changes instead of a 20K-token re-snapshot.
2. **Deterministic snapshots and replay.** Snapshot at a quiescent point (network idle, no pending layout or animations), with a virtual clock and seeded randomness so agent steps are reproducible for evaluation and auditing. This answers the benchmark non-reproducibility and scaffold variance noted above.
3. **Pixel-optional rendering.** Agents on tiers 2 and 3 don't need rasterization. The engine should run style and layout (for geometry and visibility) while skipping paint and compositing for unobserved agent tabs, and rasterize on demand only when a screenshot is requested. This lowers the cost of parallel tabs.
4. **Headless/parallel and cloud-native sessions.** Many concurrent isolated contexts (one per agent task) with cheap fork/clone of session state, and a stable automation protocol (WebDriver BiDi-like) as a native interface, not a debug port.
5. **Agent principal and capability model.** Build Google's Agent Origin Sets into the security core:
   - per-task read and read-write origin sets
   - information-flow tagging on data handed to the model
   - trusted, non-spoofable confirmation UI
   - deterministic detection of sensitive actions (payment, credential, PII forms)
   - a mandatory, tamper-proof action log
6. **Native WebMCP-style tool registry**, with permissions-policy gating, cross-origin mediation, and a declarative `<form>`-to-tool fallback. When a page registers tools, prefer them, and fall back to the semantic tree, then to pixels.
7. **Honest agent identification** to sites (a distinct UA token or signed requests), given the Amazon v. Perplexity legal exposure.
8. **On-device model hosting.** A Prompt-API-compatible local model runtime, shared across origins, and a WebNN/WebGPU path. Useful for cheap on-device injection classifiers, and as a planner for privacy-sensitive tasks.

### Gaps
- I did not verify WebNN's 2026 standardization or shipping status, or Gemini Nano's naming and status in 2026. The I/O post mentions only the Prompt API and Gemma 197M.
- No public data quantifies how much rendering cost agents waste on unneeded pixels.
- No primary source documents how Chrome's auto browse actually perceives pages (screenshot vs a11y vs DOM). Google's posts describe actions (click, scroll, type) but not the observation format.
