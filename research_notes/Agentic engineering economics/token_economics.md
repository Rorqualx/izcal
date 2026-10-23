# Agentic engineering economics: Token economics

Compiled 21 October 2026. Primary sources were preferred (vendor pricing pages, vendor telemetry documentation, published analyses, user reports). The session's shared web-search budget was exhausted before this strand ran, so known primary pages were read directly; anything that could not be read is marked **Low** and excluded from the derived model.
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, an illustrative example rather than a population measurement, or a vendor claim without data; **Low** = secondary source only.

**Scope.** This note supplies the conversion constants the synthesis needs: tokens per real coding session or task, price per million tokens 2023–2026, inference cost curves, agent-hour token rates, and the resulting dollars. It does not decide productivity multipliers (strand 1) or the incompressible remainder (strand 4). Derived figures are marked †, per the corpus's †-discipline.

## 1. Tokens per real coding session and task

### 1.1 Platform-published and documented sessions

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **Claude Code enterprise average spend.** Anthropic's own deployment average, not a study. | ~**$13 per developer per active day** and **$150–250 per developer per month**; below **$30 per active day for 90% of users** | [Manage costs effectively](https://code.claude.com/docs/en/costs) | 2026-10-21 (accessed) | Medium — vendor aggregate with no published dataset |
| 1.2 | **Documented Claude Code session** (`/usage` example in the docs). The only vendor-published per-session token breakdown found. | **1.2k input + 5.3k output + 940.0k cache read + 50.0k cache write = 996.5k tokens processed**; **$0.55** on Sonnet 4.6; **14 requests**; **91% of input tokens from cache**; **6m 20s API duration vs 6h 33m wall** | [Manage costs effectively](https://code.claude.com/docs/en/costs) | 2026-10-21 (accessed) | Medium — documentation example, not a population sample |
| 1.3 | **Documented user-day** (Claude Code Analytics API schema example). | **5 sessions**; **100k input + 35k output + 10k cache read + 5k cache creation = 150k tokens**; **$1.13** (113 cents) on Opus 5.5; **76 tool actions** (68 accepted, 8 rejected), i.e. ~**15 tool actions per session** | [Claude Code Analytics API](https://platform.claude.com/docs/en/build-with-claude/claude-code-analytics-api) | 2026-10-21 (accessed) | Medium — illustrative response example |
| 1.4 | **Managed Agents worked example.** A one-hour coding session used as a billing illustration. | **50k input + 15k output = 65k tokens**; **$0.58** including **$0.08 session runtime** on Opus 5.5 | [Pricing](https://platform.claude.com/docs/en/about-claude/pricing) | 2026-10-21 (accessed) | Medium — illustrative, far below a real tool-loop session |
| 1.5 | **Wall-clock is mostly idle.** Claude Code bills token use and reports API duration separately from wall duration. | Same session: **6m 20s API-active vs 6h 33m wall** (6.2% active) | [Manage costs effectively](https://code.claude.com/docs/en/costs) | 2026-10-21 (accessed) | High — documented output |
| 1.6 | **Agent teams multiply tokens.** Anthropic's warning for multi-instance sessions. | **~7x more tokens than standard sessions** when teammates run in plan mode | [Manage costs effectively](https://code.claude.com/docs/en/costs) | 2026-10-21 (accessed) | Medium — vendor claim, no dataset |
| 1.7 | **Capacity guidance per active developer.** Throughput ceiling, not measured average. | **200k–300k TPM per user** for 1–5 users; **10k–15k TPM** at 500+ users | [Manage costs effectively](https://code.claude.com/docs/en/costs) | 2026-10-21 (accessed) | Medium — provisioning guidance |
| 1.8 | **Tokenizer drift across generations.** Same text is not the same token count on newer Claude models. | Claude 4.7+ produces **~30% more tokens for the same text** than earlier models | [Pricing](https://platform.claude.com/docs/en/about-claude/pricing) | 2026-10-21 (accessed) | High — affects every cross-generation token comparison |

### 1.2 Project-level reports and platform billing units

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.9 | **A real release built mostly agentically.** Well-documented single-developer report with per-transcript costs. | sqlite-utils 4.0rc2: **37 prompts, 34 commits, +1,321/−190 lines, 30 files**; **$149.25 total** (main session **$141.02** on Fable 5; four review agents **$7.91**; one Opus 4.8 agent **$0.32**); **$4.39 per commit**, **$0.113 per added line** | [Simon Willison, 5 Jul 2026](https://simonwillison.net/2026/Jul/5/sqlite-utils-fable/) | 2026-07-05 | Medium — single-user report, no token breakdown |
| 1.10 | **A long autonomous run.** Duration only; no cost or token figures published. | A web agent ran **>12 hours** on one project | [Simon Willison, 16 Oct 2026](https://simonwillison.net/2026/Oct/16/sqlite-sparql-rust/) | 2026-10-16 | Medium — user report, duration only |
| 1.11 | **GitHub Copilot hides tokens.** Billing is in abstract credits. | **1 AI credit = $0.01**; Pro **$10/mo = $15 of credits (1,500)**, Pro+ **$39 = $70 (7,000)**, Max **$100 = $200 (20,000)**; consumption "depends on the model you choose and the complexity of the task"; **no token counts published**; completions use no credits | [Copilot plans](https://github.com/features/copilot/plans) | 2026-10-21 (accessed) | High — for the absence of token data |
| 1.12 | **Cursor hides tokens.** Plan tiers with a shared pool. | Free/$8/$30/$100/$200 per month; Teams **$30+/user/mo**; "shared AI usage pool" sized **1x/4x/15x/50x**; **no token quotas or per-token prices published** | [Cursor pricing](https://cursor.com/pricing) | 2026-10-21 (accessed) | High — for the absence of token data |
| 1.13 | **Devin could not be verified.** Public pricing URLs are gone. | `https://cognition.ai/pricing` redirects to `https://cognition.com/pricing`, which returns **HTTP 404**; the docs index states only that "individual and Teams plans are available"; **no ACU price, minutes per ACU, or token accounting found** | [Cognition](https://cognition.com/), [Devin docs](https://docs.devin.ai/) | 2026-10-21 (accessed) | Low — unverified, excluded from the model |
| 1.14 | **Local session accounting exists.** Tooling can extract per-session tokens from Claude Code's local JSONL. | ccusage reports **input, output, cache creation, cache read and USD cost** by session, day, month and **5-hour billing block**; **no published population aggregate** | [ccusage](https://github.com/ryoppippi/ccusage) | 2026-10-21 (accessed) | High — tool capability; no numbers |
| 1.15 | **Academic agent task cost.** Cost per solved issue is reported, but the basis is ambiguous. | Agentless on SWE-bench Lite: **32.00% (96 fixes)** at "low cost (**$0.70**)"; the abstract does not say whether $0.70 is per issue or total | [arXiv:2407.01489](https://arxiv.org/abs/2407.01489) | 2024-07 | Medium — abstract only |
| 1.16 | **No published tokens-per-commit or tokens-per-PR aggregate** was found. The API to compute it exists. | Analytics API reports `tokens.input/output`, `tokens.cache_read/cache_creation`, `commits_by_claude_code` and `pull_requests_by_claude_code` per user-day, so token/commit is computable per organisation; **no aggregate is published** | [Claude Code Analytics API](https://platform.claude.com/docs/en/build-with-claude/claude-code-analytics-api) | 2026-10-21 (accessed) | High — for the absence |
| 1.17 | **Human+AI time, not token time.** Controlled measurement of experienced developers. | **19% slower** with AI access; **16 developers, 246 tasks** | [METR RCT, arXiv:2507.09089](https://arxiv.org/abs/2507.09089) | 2025-07-12 | High — primary abstract read |

**Implications:** Documented per-session token counts span **65k (billing example) to ~1.0M (cache-heavy tool session)**, a 15x range, and the enterprise average is **$13 per active day**. Only Anthropic publishes token and cache detail; Copilot, Cursor and Devin bill in opaque units, so any tokens-per-task factor for the translation must be labelled as derived from Anthropic's documented examples, not as an industry measurement. The **91–94% cache-read share** is the single most important number for cost: it cuts the effective input price by roughly an order of magnitude. Pricing the **$149.25 project** at Fable 5's list prices implies **16–133M processed tokens** depending on cache mix, or **0.5–3.9M tokens per commit** and **12k–101k tokens per added line** (†, derived).

## 2. Price per 1M tokens, 2023–2026

All prices are first-party API list prices in USD per million tokens (MTok). Cache and long-context terms are listed where the vendor publishes them.

### 2.1 OpenAI

Source for every price in this table: [OpenAI API pricing](https://developers.openai.com/api/docs/pricing), read 2026-10-21. The page lists model IDs and prices but not release dates; dates below come from model-ID dates or public launch dates and are graded accordingly.

| # | Model | Input | Cached input | Output | Long context | Date | Conf. |
|---|---|---|---|---|---|---|---|
| 2.1 | GPT-4 (`gpt-4-0613`) | $30.00 | — | $60.00 | — | 2023-03 launch; 2023-06 snapshot ID | High |
| 2.2 | GPT-4 Turbo (`gpt-4-turbo-2024-04-09`) | $10.00 | — | $30.00 | — | 2024-04 snapshot | High |
| 2.3 | GPT-4o (`gpt-4o-2024-05-13`) | $5.00 | — | $15.00 | — | 2024-05-13 | High |
| 2.4 | GPT-4o (current list) | $2.50 | $1.25 | $10.00 | — | by 2026-10-21 | High |
| 2.5 | o1 | $15.00 | $7.50 | $60.00 | — | 2024-12 (launch) | Medium — date not on page |
| 2.6 | GPT-4.1 | $2.00 | $0.50 | $8.00 | — | 2025-04 (launch) | Medium — date not on page |
| 2.7 | GPT-5 / GPT-5.1 | $1.25 | $0.125 | $10.00 | — | 2025-08 / 2025-11 | Medium — dates not on page |
| 2.8 | GPT-5.4 | $2.50 | $0.25 | $15.00 | >272k: $5.00 / $0.50 / $22.50 | by 2026-10-21 | High |
| 2.9 | GPT-5.5 | $5.00 | $0.50 | $30.00 | >272k: $10.00 / $1.00 / $45.00 | by 2026-10-21 | High |
| 2.10 | GPT-5.6 Sol | $4.00 | $0.40 | $20.00 | >272k: $8.00 / $0.80 / $30.00 | by 2026-10-21 | High |
| 2.11 | GPT-6.1 Sol | $2.00 | $0.10 | $10.00 | >272k: $4.00 / $0.20 / $15.00 | by 2026-10-21 | High |
| 2.12 | GPT-6 Astra (flagship) | $10.00 | $1.00 | $50.00 | >272k: $20.00 / $2.00 / $75.00 | by 2026-10-21 | High |

### 2.2 Anthropic

| # | Model | Input | Cache write (5m / 1h) | Cache read | Output | Date | Conf. |
|---|---|---|---|---|---|---|---|
| 2.13 | Claude 3 Opus / Sonnet / Haiku | $15.00 / $3.00 / $0.25 | — | — | $75.00 / $15.00 / $1.25 | 2024-03-04 | High |
| 2.14 | Claude 4 Opus / Sonnet | $15.00 / $3.00 | — | — | $75.00 / $15.00 | 2025-05-22 | High |
| 2.15 | Claude Opus 4.5 | $5.00 | $6.25 / $10.00 | $0.50 | $25.00 | 2025-11-24 | High |
| 2.16 | Claude Opus 5.5 | $4.00 | $5.00 / $8.00 | $0.20 | $20.00 | 2026-09-22 | High |
| 2.17 | Claude Sonnet 5.5 | $2.00 | $2.50 / $4.00 | $0.10 | $10.00 | 2026-09-28 | High |
| 2.18 | Claude Haiku 5.5 (≤100k prompt) | $0.10 | $0.125 / $0.20 | $0.01 | $0.50 | by 2026-10-21 | Medium — launch date not checked |
| 2.19 | Claude Haiku 5.5 (>100k prompt) | $0.50 | $0.625 / $1.00 | $0.05 | $2.50 | by 2026-10-21 | Medium |
| 2.20 | Claude Fable 5 / Fable 5.1 (flagship) | $10.00 | $12.50 / $20.00 | $0.25 (Fable 5.1; $1.00 on Fable 5) | $50.00 | by 2026-10-21 | Medium — launch date not checked |
| 2.21 | Prior generations (for the trajectory) | Opus 4.1/4 **$15.00**; Sonnet 4.6 **$3.00**; Haiku 4.5 **$1.00** | — | Opus **$1.50**; Sonnet **$0.30**; Haiku **$0.10** | Opus **$75.00**; Sonnet **$15.00**; Haiku **$5.00** | 2025–2026 | High — listed on the live page |

Source for 2.13–2.14: [Claude 3 family](https://www.anthropic.com/news/claude-3-family), [Claude 4](https://www.anthropic.com/news/claude-4). Source for 2.15: [Claude Opus 4.5](https://www.anthropic.com/news/claude-opus-4-5). Source for 2.16–2.21: [Anthropic pricing](https://platform.claude.com/docs/en/about-claude/pricing), [Opus 5.5](https://www.anthropic.com/news/claude-opus-5-5), [Sonnet 5.5](https://www.anthropic.com/news/claude-sonnet-5-5). Anthropic's cache-read multiplier is **0.1x** base input generally, **0.05x** on Opus 5.5 and Sonnet 5.5, and **0.025x** on Fable 5.1; cache writes are **1.25x (5m)** and **2x (1h)**. Claude 4.6 and later include the full 1M context at standard rates; Haiku 5.5 charges the higher tier above 100k prompt tokens.

### 2.3 Google Gemini

Source for prices: [Gemini API pricing](https://ai.google.dev/gemini-api/docs/pricing), read 2026-10-21. The page gives current models and scheduled changes, not launch dates.

| # | Model | Input | Cached input | Output | Long context | Date | Conf. |
|---|---|---|---|---|---|---|---|
| 2.22 | Gemini 2.5 Pro | $1.25 | $0.125 | $10.00 | >200k: $2.50 / $0.25 / $15.00 | announced 2025-03-25; price by 2026-10-21 | High — [launch date](https://blog.google/technology/google-deepmind/gemini-model-thinking-updates-march-2025/) |
| 2.23 | Gemini 2.5 Flash | $0.30 | $0.03 | $2.50 | — | by 2026-10-21 | High |
| 2.24 | Gemini 2.5 Flash-Lite | $0.10 | $0.01 | $0.40 | — | by 2026-10-21 | High |
| 2.25 | Gemini 3.1 Pro Preview | $2.00 | $0.20 | $12.00 | >200k: $4.00 / $0.40 / $18.00 | by 2026-10-21 | High |
| 2.26 | Gemini 3.8 Flash | $0.75 | $0.075 | $3.75 | — | by 2026-10-21; **doubles to $1.50 / $0.15 / $7.50 on 2027-01-01** | High |
| 2.27 | Gemini 4 Argon (flagship) | $2.00 | $0.10 | $10.00 | — | by 2026-10-21; **doubles to $4.00 / $0.20 / $20.00 on 2026-12-01** | High |
| 2.28 | Gemini 1.5 Pro (2024 era) | reported **$3.50** | — | reported **$10.50** | reported $7.00 / $21.00 above 128k | 2024-02-15 | Low — [launch post](https://blog.google/technology/ai/google-gemini-next-generation-model-february-2024/) confirms the date but **not the price**; the model is delisted from the live pricing page. Excluded from calculations. |

### 2.4 The trajectory in one table

Representative frontier and mid-tier list prices; this is the series the conversion should be priced against.

| Date | Model | Input $/MTok | Output $/MTok | Cache read $/MTok |
|---|---|---|---|---|
| 2023-03 | GPT-4 | 30.00 | 60.00 | — |
| 2024-03 | Claude 3 Opus | 15.00 | 75.00 | — |
| 2024-05 | GPT-4o (launch) | 5.00 | 15.00 | — |
| 2025-05 | Claude 4 Opus | 15.00 | 75.00 | — |
| 2025-08 | GPT-5 | 1.25 | 10.00 | 0.125 |
| 2025-11 | Claude Opus 4.5 | 5.00 | 25.00 | 0.50 |
| 2026-09 | Claude Opus 5.5 | 4.00 | 20.00 | 0.20 |
| 2026-09 | Claude Sonnet 5.5 | 2.00 | 10.00 | 0.10 |
| 2026-10 | GPT-6.1 Sol | 2.00 | 10.00 | 0.10 |
| 2026-10 | Gemini 4 Argon | 2.00 | 10.00 | 0.10 |
| 2026-10 | Claude Fable 5 (flagship) | 10.00 | 50.00 | 0.25 |
| 2026-10 | GPT-6 Astra (flagship) | 10.00 | 50.00 | 1.00 |

Sources: the vendor pricing pages cited in §2.1–2.3; dates from §2.1–2.3 and the linked launch posts.

**Implications:** In October 2026 the **capable mid-tier converges on $2 input / $10 output** (Sonnet 5.5, GPT-6.1 Sol, Gemini 4 Argon) and the **flagship tier on $10 / $50** (Fable 5, GPT-6 Astra). That mid-tier is **15x cheaper on input and 6x cheaper on output than GPT-4 in March 2023**, but only **1.5x cheaper than Claude 3 Sonnet in March 2024**. Cache reads at **0.05–0.1x** input are worth more to a coding agent than the headline price cut. Two vendors have **scheduled price increases** at the frontier (Gemini 4 Argon on 2026-12-01, Gemini 3.8 Flash on 2027-01-01), so the trajectory is not monotonically down.

## 3. Inference cost curves

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Epoch AI's fixed-capability price decline.** Lowest price needed to match a fixed benchmark score, six benchmarks. | **9x to 900x per year** across thresholds; **40x per year** for GPT-4-level performance on PhD-level science questions; declines were fastest in the prior year, and Epoch warns they may not persist | [Epoch AI, "LLM inference prices have fallen rapidly but unequally across tasks"](https://epoch.ai/blog/llm-inference-price-trends) | 2025-03-12 | High — analysis read |
| 3.2 | **a16z's ~10x/year claim.** Same methodology, capability-normalised. | "For an LLM of equivalent performance, the cost is decreasing by **10x every year**"; MMLU-42 went from **$60/MTok (GPT-3, 2021-11)** to **$0.06/MTok (Llama 3.2 3B, 2024-11)**, a **1,000x fall in 3 years**; MMLU-83 fell **~62x** after GPT-4; o1 launched at the **same $60 output price as GPT-3** | [a16z, "LLMflation"](https://a16z.com/llmflation-llm-inference-cost/) | 2024-11-12 | High — article read |
| 3.3 | **First-principles check against list prices.** GPT-4-level capability is now available on small models at a fraction of GPT-4's price. | GPT-4 output **$60** (2023-03) → GPT-5 nano output **$0.40** (2025-08): **150x in 2.48 years ≈ 8x/year**. Input **$30 → $0.05**: **600x ≈ 13x/year**. Because GPT-5 nano is stronger than GPT-4, these are **lower bounds** on the fixed-capability decline. | Prices: [OpenAI pricing](https://developers.openai.com/api/docs/pricing); calculation this note | 2026-10-21 | Medium — prices High, calculation † |
| 3.4 | **Frontier list prices fall far more slowly than fixed-capability prices.** | Anthropic Opus output **$75 (2024-03) → $20 (2026-09)**: **3.75x in 2.55 years ≈ 1.7x/year**; Sonnet **$15 → $10**: **1.5x ≈ 1.2x/year**. OpenAI GPT-4 output **$60 → GPT-4o $15** (1.17 years): **4x ≈ 3.3x/year**; **→ GPT-5 $10** (2.4 years): **6x ≈ 2.1x/year**. | Vendor pricing pages above; calculation this note | 2026-10-21 | Medium — prices High, calculation † |
| 3.5 | **2026 has scheduled frontier increases, not 10x declines.** | Gemini 4 Argon **$2/$10 → $4/$20 on 2026-12-01**; Gemini 3.8 Flash **$0.75/$3.75 → $1.50/$7.50 on 2027-01-01** | [Gemini API pricing](https://ai.google.dev/gemini-api/docs/pricing) | 2026-10-21 (accessed) | High |
| 3.6 | **No 2026 Epoch projection was found.** The data-insights index lists the March 2025 analysis as the latest inference-price entry. | No 2026-dated inference-price insight on the index; the 2025 caveat that fast declines may not persist still stands | [Epoch AI data insights](https://epoch.ai/data-insights) | 2026-10-21 (accessed) | High — for the absence |

**Implications:** The **~10x/year claim is supported for fixed capability** (Epoch 9–900x; a16z 10x; this note's 8–13x list-price check) but **not for frontier list prices**, which fall at roughly **1.2–2.1x/year** and in 2026 are scheduled to rise at Gemini's frontier. For the translation, price work at **current list prices**; apply a fixed-capability decline only if the work can be moved to a smaller model at equal quality, and treat a 10x/year decline in the budget as an upside case, not the base case.

## 4. Human-equivalent conversions

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **No published engineer-year → token conversion was found.** | No analysis converting engineer-years (or headcount) into tokens or agent-hours was verified in this session; the conversion below is assembled from components and marked † | — | 2026-10-21 | Low — absence finding |
| 4.2 | **METR task-horizon measurement.** How long a task an agent completes reliably; the relevant unit for agent-hours per task. | Claude 3.7 Sonnet: **~1 hour at 50% reliability**; horizon doubling **~every 7 months**; **almost 100% success below 4 minutes**, **<10% above ~4 hours**; week-long tasks projected in **2–4 years** | [METR](https://metr.org/blog/2025-03-19-measuring-ai-ability-to-complete-long-tasks/) | 2025-03-19 | High — blog read |
| 4.3 | **Human+AI is not automatically faster.** Controlled RCT, experienced open-source developers on their own repositories. | **19% slower** with AI; **16 developers, 246 tasks** | [arXiv:2507.09089](https://arxiv.org/abs/2507.09089) | 2025-07-12 | High — abstract read |
| 4.4 | **A priced real project.** One release, one developer, agentic. | **$149.25** for **34 commits / 1,321 added lines / 37 prompts**; **$4.39 per commit**, **$0.113 per added line** | [Simon Willison](https://simonwillison.net/2026/Jul/5/sqlite-utils-fable/) | 2026-07-05 | Medium — single report |
| 4.5 | **Anthropic's operating average.** The closest thing to a published cost per developer-year of agent use. | **$13/active day**; at ~200 active days/year ≈ **$2,600 per developer-year** of agent use (derived), consistent with the stated **$150–250/month** | [Claude Code costs](https://code.claude.com/docs/en/costs) | 2026-10-21 (accessed) | Medium — vendor aggregate; annualisation † |
| 4.6 | **Unit conversion.** Standard productive hours per engineer-year. | **1 engineer-year = 1,800 productive hours** (assumption †; 2,080 paid hours less leave, training and non-project time) | Assumption, marked † | — | Low — assumption, not a measurement |
| 4.7 | **Engineer-year → agent-hours → tokens.** The conversion the synthesis needs. | At 1x productivity: **1,800 agent-hours** per engineer-year; at **5M tokens per active agent-hour**, **9 billion tokens per engineer-year**; range **3.6–18 billion** at 2–10M tokens/hour | Derived from 1.1–1.8 and 5.1–5.5; calculation this note | 2026-10-21 | Medium — component numbers High/Medium; product † |
| 4.8 | **Continuous single-agent operation.** A unit conversion, not a schedule claim. | **8,760 agent-hours per calendar year** at 24/7; one engineer-year of output at 1x takes **~1,800 agent-hours ≈ 2.5 months** of continuous single-agent runtime before parallelism | Calculation this note | 2026-10-21 | Medium — † |
| 4.9 | **Token cost per engineer-year equivalent.** Depends mostly on model and cache mix. | **$2.2k–$84.4k per engineer-year** at 1x and 5M tokens/hour: Sonnet 5.5 **$2.4k (cache-heavy) to $17.1k (fresh-heavy)**; Opus 5.5 **$4.9k–$34.2k**; Fable 5 **$10.1k–$84.4k**; GPT-5.5 **$7.9k–$47.3k**; GPT-6.1 Sol **$2.2k–$16.7k**. Add **$144/engineer-year** for Managed Agents runtime at $0.08/hour. | Derived; blend assumptions in §5.2; calculation this note | 2026-10-21 | Medium — † |
| 4.10 | **Token cost is small against human staffing.** At 2,000–3,400 engineer-years. | Token-only totals: **$4.4M–$168.8M at 2,000 EY** and **$7.5M–$286.9M at 3,400 EY** (1x productivity, 5M tokens/hour). Against **$200k–$300k fully loaded per engineer-year** (**$400M–$1.02B** for 2,000–3,400 EY), tokens are roughly **1–13% in the central cases** and **0.4–70% across the full grid**. | Derived; salary assumption † | 2026-10-21 | Medium — †, salary band is an assumption |

**Implications:** There is **no published engineer-year-to-token conversion**; the note supplies one in components. At 1x productivity, **one engineer-year ≈ 1,800 agent-hours ≈ 9B tokens ≈ $2k–$84k**, with a central estimate of **$5k–$25k**. Against human staffing, token spend is **small — likely 1–13% in the base cases** — so the binding constraints for Izcal are agent-hours, review capacity and the incompressible remainder, not token dollars. The 19% slowdown RCT is a warning against assuming 1x without measurement.

## 5. What an agent-hour costs

An **agent-hour** here means one hour of **active** agent runtime (model turns plus tool loops), not wall-clock with a human idle. Claude Code's own example shows why: **6m 20s active vs 6h 33m wall**.

### 5.1 Tokens per active agent-hour

| # | Method | Result | Source | Conf. |
|---|---|---|---|---|
| 5.1 | Documented session: 996.5k tokens over 6m 20s active | **~9.4M tokens/hour** | [Claude Code costs](https://code.claude.com/docs/en/costs) | Medium — one documented example |
| 5.2 | Enterprise average: $13/active day at cache-heavy blended prices | **~24M (Opus 5.5) to ~48M (Sonnet 5.5) tokens/day**; **3–12M/hour** over 4–8 active hours | Derived from row 1.1 and the blend table below | Medium — † |
| 5.3 | Rate-limit guidance: 200k–300k TPM per user for small teams | **12–18M tokens/hour** maximum sustained | [Claude Code costs](https://code.claude.com/docs/en/costs) | Medium — ceiling, not average |
| 5.4 | Managed Agents billing example: 65k tokens in one hour | **0.065M tokens/hour** | [Pricing](https://platform.claude.com/docs/en/about-claude/pricing) | Medium — illustrative lower bound |
| 5.5 | **Central estimate for a tool-loop coding agent** | **5M tokens per active agent-hour**, with a working range of **2–10M** | Triangulation of 5.1–5.4; marked † | Medium — † |

### 5.2 Blended price per million processed tokens

Blend assumptions by token share. "Cache-heavy" matches the documented Claude Code session; "balanced" is an output-heavy tool loop; "fresh-heavy" is a worst case with little cache reuse. Prices are from §2.1–2.3; the token-share mixes are this note's assumptions (†).

| Model (input / cache write / cache read / output $/MTok) | Cache-heavy: 0.1% / 5% / 94.4% / 0.5% | Balanced: 10% / 5% / 75% / 10% | Fresh-heavy: 30% / 10% / 50% / 10% |
|---|---|---|---|
| Claude Sonnet 5.5 (2.00 / 2.50 / 0.10 / 10.00) | **$0.271** | **$1.400** | **$1.900** |
| Claude Opus 5.5 (4.00 / 5.00 / 0.20 / 20.00) | **$0.543** | **$2.800** | **$3.800** |
| Claude Fable 5 (10.00 / 12.50 / 0.25 / 50.00) | **$1.121** | **$6.813** | **$9.375** |
| GPT-5.5 (5.00 / — / 0.50 / 30.00) | **$0.877** | **$4.125** | **$5.250** |
| GPT-6.1 Sol (2.00 / — / 0.10 / 10.00) | **$0.246** | **$1.375** | **$1.850** |
| Gemini 4 Argon (2.00 / — / 0.10 / 10.00) | **$0.246** | **$1.375** | **$1.850** |

### 5.3 Dollars per active agent-hour

At the central **5M tokens/hour** (range 2–10M; multiply the table by 0.4 or 2.0).

| Model | Cache-heavy | Balanced | Fresh-heavy |
|---|---|---|---|
| Claude Sonnet 5.5 | **$1.36** | **$7.00** | **$9.50** |
| Claude Opus 5.5 | **$2.71** | **$14.00** | **$19.00** |
| Claude Fable 5 | **$5.61** | **$34.06** | **$46.88** |
| GPT-5.5 | **$4.39** | **$20.63** | **$26.25** |
| GPT-6.1 Sol | **$1.23** | **$6.88** | **$9.25** |
| Gemini 4 Argon | **$1.23** | **$6.88** | **$9.25** |

| # | Cost component | Numbers | Source | Conf. |
|---|---|---|---|---|
| 5.6 | **Managed Agents runtime** (infrastructure, not tokens) | **$0.08 per session-hour**, metered only while running; idle, rescheduling and terminated time are not billed | [Pricing](https://platform.claude.com/docs/en/about-claude/pricing) | High |
| 5.7 | **Idle/background overhead** | Typically **under $0.04 per session** even without active interaction | [Claude Code costs](https://code.claude.com/docs/en/costs) | High |
| 5.8 | **Multi-agent multiplier** | Agent teams use **~7x** the tokens of a standard session; budget scales with teammate count | [Claude Code costs](https://code.claude.com/docs/en/costs) | Medium — vendor claim |
| 5.9 | **Working cost of an agent-hour** | **$1.2–$14/hour** for cache-heavy to balanced sessions on Sonnet/Opus/GPT-6.1-class models; **$34–$47/hour** for an output-heavy Fable session; **7x** for agent teams | Derived from 5.3 and 5.8 | Medium — † |

**Implications:** A reasonable base case is **5M tokens and $1–$14 per active agent-hour** for current mid-tier and Opus-class models; a worst case on the flagship with little cache reuse is **~$47/hour**. Anthropic's own enterprise average (**$13/active day**) is consistent with a few active hours per day at these rates. Multi-agent fan-out is the largest swing factor (**~7x**), not the choice between Sonnet and Opus (**~2x**).

## 6. Conversion recipe for the synthesis

Use this as the arithmetic spine; replace the † assumptions with measured values at the first milestones.

```
agent_hours = engineer_years × 1,800 / productivity_multiplier
tokens      = agent_hours × 5,000,000          (range 2M–10M per active hour)
cost_usd    = tokens / 1e6 × blended_price     (table 5.2)
            + agent_hours × 0.08               (Managed Agents runtime, optional)
```

Per **100 engineer-years** at 1x productivity and 5M tokens/hour (**180,000 agent-hours; 900B tokens**); all values derived and marked †:

| Model and mix | Tokens | Token cost | Runtime | Total |
|---|---|---|---|---|
| Sonnet 5.5, cache-heavy | 900B | **$244k** | $14.4k | **$258k** |
| Opus 5.5, cache-heavy | 900B | **$489k** | $14.4k | **$503k** |
| Opus 5.5, balanced | 900B | **$2.52M** | $14.4k | **$2.53M** |
| Fable 5, fresh-heavy | 900B | **$8.44M** | $14.4k | **$8.45M** |

Scaling rules, all †:

- **Productivity multiplier:** 0.5x doubles agent-hours and cost; 2x halves them; 4x quarters them.
- **Token rate:** 2M/hour is 40% of the table; 10M/hour is 200%.
- **Cache mix:** the cache-heavy-to-fresh-heavy spread is **~7x** for Sonnet/Opus and **~8x** for Fable — larger than the model spread at a fixed mix.
- **Concurrency:** cost per calendar month scales with the number of parallel agents, not with wall-clock alone. A 10-agent fan-out is 10x the token burn of one agent at the same per-agent rate; agent teams add another **~7x** on top of a standard session.

**Proposed reversal trigger for the translation:** if the first three agentic milestones measure outside **2–10M tokens per active agent-hour**, or a blended cost outside **$0.25–$9.40 per million processed tokens**, or cache-read share below **50%**, restate the token and dollar translation. These bounds are † set from the documented examples and price list, not measured on Izcal.

## 7. What is not known

1. **No platform-published token telemetry** from GitHub Copilot, Cursor or Devin. Copilot bills in **$0.01 AI credits**; Cursor bills in plan tiers and a shared pool; Devin's public pricing/ACU pages return **404** as of 2026-10-21. Any cross-platform tokens-per-task claim is therefore an Anthropic-derived estimate.
2. **No published engineer-year → token or engineer-year → agent-hour conversion** was found. Sections 4 and 6 construct one; it is † until measured.
3. **No 2026 Epoch AI projection** on inference prices appears on their data-insights index; the latest verified analysis is **2025-03-12**. The ~10x/year figure is a fixed-capability historical trend, not a forecast.
4. **Anthropic's $13/active-day average has no published methodology** (model mix, active-hour definition, cache assumptions). It is used only as a cross-check.
5. **Gemini 1.5 Pro's 2024 price is unverified** (delisted model; the launch post gives no price). It is excluded from the calculations and the trend.
6. **Per-commit and per-PR token counts are not published.** The only proxy found is one project's **$4.39/commit** and **$0.113/added line**.
7. **Tokenizer changes (+30% tokens on Claude 4.7+)** make token counts non-comparable across generations; cost comparisons must be made in dollars or in capability-normalised units, not raw tokens.
8. **The agent-hour is not yet a measured unit for Izcal.** It is a derived unit (active runtime × token rate); the first milestone should log API-active duration and tokens per session so the 5M/hour central estimate is replaced with Izcal's own number.
