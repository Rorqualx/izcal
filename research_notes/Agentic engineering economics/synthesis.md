# Agentic engineering economics: synthesis

Compiled 21 October 2026 from four research strands ([RCTs and academic evidence](rcts_academic.md), [industry data](industry_data.md), [token economics](token_economics.md), [incompressible remainder](incompressible_remainder.md)) and the specs' own staffing figures. Owner direction: this is a fully agentic coding project; translate the staffing and engineer-year figures into agentic time, tokens and cost, accurately, from evidence. Feeds the proposed owner decision **C-6R** (below); nothing here is applied to any spec until the owner accepts it.

## 1. What the evidence actually says (corrected anchors)

The strands corrected several commonly cited figures. The load-bearing rows:

| # | Finding | Numbers | Where |
|---|---|---|---|
| 1.1 | **Peng et al. 2023** RCT: Copilot made a standalone HTTP-server task faster | 55.8% faster (71.17 vs 160.89 min, p = 0.0017, 95% CI [21%, 89%]); ~35 completers per arm; quality not examined | [rcts 2.1] |
| 1.2 | **Cui et al. 2024** field RCT: Copilot raised completed tasks across three firms | +26.08% (SE 10.3%), 4,867 developers | [rcts 2.2] |
| 1.3 | **METR 2025** RCT, experienced developers in their own mature repos: completion **slowed**, and the developers believed the opposite | +19% completion time, 95% CI [+2%, +39%], 16 developers, 246 tasks | [rcts 2.3] |
| 1.4 | **Paradis et al.** (Google, ICSE-SEIP 2025): AI assistance speed on real engineering | 21% faster, but 95% CI [−0.51, 0.03] crosses zero, p = 0.086 — not significant | [rcts 2.4] |
| 1.5 | **Maier et al. 2026 meta-analysis** (23 studies): the setting, not the tool, decides the effect — lab 0.73, enterprise 0.19 (n.s.), open source 0.01 (n.s.), overall g = 0.33 [0.09, 0.58], I² = 99% | | [rcts 3.x] |
| 1.6 | **METR Feb 2026** follow-up (57 developers): point estimates 1.04x–1.22x speed, CIs include 1 — the authors call it unreliable | | [rcts 3.x] |
| 1.7 | **DORA 2024** (39,000+ respondents): +25% AI adoption came with delivery throughput **−1.5%** and stability **−7.2%**; self-reported productivity rose. DORA 2025: throughput turns positive, stability still negative | | [industry 2.x] |
| 1.8 | **Anthropic Economic Index**: in ~400K Claude Code sessions users make ~70% of planning decisions, Claude ~80% of execution decisions | | [industry 5.x] |
| 1.9 | Rework and review pressure is real: 45.1% of merged Claude Code PRs need changes (Watanabe, TOSEM); core developers review 6.5% more code and code 19% less (Xu); review time +91% and PR size +154% in one measured org; Veracode found AI-introduced flaws in 45% of tests | | [rcts 4.x], [industry 7.x] |
| 1.10 | Cost of tokens for fixed capability falls ~10x/year (Epoch 9–900x/year, a16z 10x/year), but **frontier list prices fall only 1.2–2.1x/year** and Gemini schedules increases | | [tokens 3.x] |
| 1.11 | One engineer-year in agent terms: **1,800 agent-hours ≈ 9B processed tokens ≈ $2.2k–$84.4k, centrally $5k–$25k** at October 2026 prices | | [tokens 4.x] |

**Reading:** agentic assistance measurably speeds *contained implementation tasks* and *slows or leaves unchanged mature-codebase engineering, review and verification-heavy work*. The strongest published support is 1.8: execution compresses, planning does not. Every claim below rests on this split.

## 2. The translation model

One flat multiplier is not supported by any study. The model is **activity-class based**, five classes:

| Class | What it covers in Izcal | Compression range (human effort → agentic effort) | Evidence |
|---|---|---|---|
| **I — Implementation** | Writing `hg-*` crates from the accepted specs: parsers, layout, bindings, the JS engine, storage engine, inference kernels | **1.2x–2.0x faster** (central ~1.5x), *after* phase 8 corrections are applied | 1.1, 1.2, 1.4, 1.6 (lab/greenfield effects, enterprise field, 2026 agents); Izcal's specs are unusually detailed, the best documented case for agents |
| **V — Verification** | Phase-8 review, fact-checks, security review, conformance debugging, the rework loops of 1.9 | **1.0x or worse** (0.8x–1.0x speed; possibly slower) | 1.3, 1.7, 1.9 |
| **M — Measurement and machines** | FP-M* rig runs, AT lab, fuzz CPU-hours (SVG ≥1,000/week), WPT campaigns, riscv64 pool, GPU matrix | **1.0x** on machine time; harness *writing* is class I | [remainder 3–6]: machine time is set by hardware, not typing |
| **C — Craft, legal, governance** | Type design (150–400 designer-years), licence/patent/audit (9–17 EY once, 8–15 ongoing), the root program, owner decision bandwidth (51+38 decisions in batches 2–3 alone) | **1.0x**, legal possibly worse (hallucination rates 17–33%/58–88% in studies) | [remainder 2, 7, 8, 9] |
| **D — Design and planning** | The spec-writing itself, architecture decisions | **1.0x** — but note: the corpus's specs were already produced agentically; this translation applies to *future* derivation-derived figures | 1.8 |

A class-I component gains 1.2–2.0x on its implementation share; classes V/M/C/D do not compress. The net effect on a component is bounded by its implementation share.

## 3. The corpus's own numbers

The corpus maintains **people** totals (C-6: about 551–719 engineers† plus 2–4† non-engineers, with DOM 72–92†, accessibility, DevTools, security and infrastructure named beside it; batch 4 adds ML 14–20.5† and shell 14–20.25† engineers, phishing pending). It does **not** maintain an aggregate engineer-year figure — this synthesis reconstructs it, marking derived components:

| Spec | Engineer-years (stated †) | Derivation |
|---|---|---|
| Graphics and rendering | 420–780 | stated |
| Layout | 260–415 | stated |
| Images and media | 240–440 | stated |
| Style | 130–215 | stated |
| Fonts and text | 115–185 (+ 150–400 designer-years) | stated |
| Accessibility | 105–180 | stated across its revisions |
| Animation | 105–175 | stated |
| PDF | 55–106 | stated |
| SVG | 30–59 | stated |
| JavaScript engine | ~210–350 | derived (65–82 engineers × 39–51 months) |
| Networking | ~290–400 | derived (58–81 × 48–60 months) |
| DOM and bindings | ~290–460 | derived (72–92 × 48–60 months) |
| Editing (+ language data) | ~150–250 | derived (34–46 + 10–16 × 48–60 months) |
| Storage | 30–50 | stated in its author's record |
| ML service | ~28–62 | derived (14–20.5 × 24–36 months) |
| WebCrypto | ~8–19 | derived (3.5–5.5 × 27–42 months) |
| Shell | 14–20.25 | stated (168–243 engineer-months) |
| Extensions (Stage 2) | ~9–21 | derived (9–14 × 12–18 months) |
| Content blocking, footprint, DevTools | ~40–80 | small, derived |
| **Total (engineering)** | **~2,300–3,900 engineer-years†** | reconstructed; stated components sum to 1,460–2,545, derived add ~840–1,350 |
| Type design | 150–400 designer-years† | stated, class C |

## 4. The translated envelope

**Class shares († first-principles, stated as parameters, reversal-triggered):** of the ~2,300–3,900 engineering EY, the strand evidence puts **C (legal/audit/governance) at ~25–50 EY**, **M at ~10–20%** (measurement-milestone-bound tracks in FP, GR, AX, JS), **V at ~25–40%** (every spec's review/conformance loops), leaving **I at ~45–60%**. The exact split per component comes from each spec's Section 9.x track tables at C-6R acceptance time; here it is applied as an envelope.

**Effort.** I compresses to 1/1.2–1/2.0 of itself; V, M, C stay. Net engineering effort: **~1,700–3,400 EY** (a 10–25% reduction, central ~15%) — i.e. agentic coding makes the *effort* somewhat cheaper, no more, on current evidence. Designer-years are untouched: 150–400.

**Agent-hours, tokens, dollars** (at 1.11's conversion, current prices):

| Quantity | Range | Central |
|---|---|---|
| Compressed EY | 1,700–3,400 | ~2,500 |
| Agent-hours | 3.1M–6.1M | ~4.5M |
| Processed tokens | 15.3T–30.6T | ~22.5T |
| Token cost (mid-tier $2/$10 → flagship $10/$50 mix) | **$6M–$350M** | ~$40M–$100M |
| Human staffing cost of the same EY at $200k–$400k loaded | $460M–$1.56B | — |
| Token cost as a share of human cost | **~1–15%** | ~5% |

The token bill is not the story. The **human headcount is the story**: full strength drops by roughly the implementation share's compression (I ≈ 45–60% at 1.2–2.0x ⇒ **~300–600 engineers at full strength** instead of ~551–719, once batch 4 folds in), and the humans who remain are concentrated in V, M and C.

**Wall-clock.** The evidence gives no licence to compress the calendar below the corpus's own non-typing constraints: Stage 1 exit is set by the layout/graphics milestone chain and the measurement calendar, not by implementation velocity; the audit gates, WPT campaigns, the type-design waves (5–10 years at ~45 designers) and the owner decision cadence are calendar-bound. Agentic parallelism compresses implementation months on the critical path, so the honest envelope is **Stage 1 exit ~40–60 months†** (48–60 as written; ~40 as the floor set by the measurement calendar and decision bandwidth, not by typing). Type design and the font waves still finish on their own 5–10 year schedule regardless.

## 5. Proposed owner decision C-6R — two-ledger staffing

**Proposal.** Restate C-6 and Stage 1 8.3 with **two ledgers**:

- **Ledger A (people and engineer-years):** the current figures, unchanged, as the human-derived planning basis (they remain the fallback and the audit basis).
- **Ledger B (agentic envelope):** the class-based translation of Section 4 — per component: implementation share, compressed agent-hours, token and dollar estimates at current prices, with the class shares marked †.

**Reversal trigger (falsifiability, as every other decision carries):** at each spec's first real milestones (FP-M0, GR-M0, JS-M1, the first phase-8 cycle on agentic output), record actual agent-hours, tokens, rework rate and wall-clock per activity class. If any two consecutive components measure **outside the Ledger B envelope on the same side** (faster or slower), C-6R is reopened and the class ranges are restated from the measurements, not from the literature.

**What accepting C-6R does not do:** it does not edit any spec's †-thresholds, milestones, budgets or decision records; it does not reduce the type-design, audit, lab or conformance commitments; it does not claim a schedule shorter than the measurement calendar allows. It adds one decision row and one dated revision line to C-6 and Stage 1 8.3.

**Open items before acceptance:** the per-component class shares should be extracted from each spec's Section 9.x track tables (a mechanical pass, one agent); phishing protection's staffing folds into the totals when its spec lands; Q-AE-1 (whether the two-ledger statement is the *published* staffing or an internal planning artifact) is the owner's call.

## 6. What is unknown (stated plainly)

- No published study measures a multi-year, spec-driven, greenfield systems project built agentically end to end; every effect size above comes from shorter tasks, and 1.3/1.5/1.7 disagree on direction.
- The corpus has no aggregate CI machine-year or fuzz CPU-year figure; M's share of effort is an envelope, not a sum.
- Token prices at the frontier are sticky (1.10); the dollar envelope assumes October 2026 prices and should be re-priced per year.
- Class I's 1.2–2.0x assumes the specs' †-thresholds keep binding: if agentic output raises rework (1.9), V absorbs the gain and the net moves toward the bottom of the envelope.
