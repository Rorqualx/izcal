# Agentic measurement: spec authoring, 21 October 2026

First real data rows for C-6R's Ledger B (the agentic envelope, adopted 21 October 2026 as an internal planning artifact). These are **measured** values — the reported token, tool-call and wall-clock usage of the background agents that wrote and reviewed the corpus's specs — not † estimates. Scope caveat stated plainly: each number is the usage reported in the agent's completion notification for its whole task tree (its own calls plus its spawned subagents); the session-level total (this conversation's full token use) is not instrumented here, so these rows undercount the orchestration overhead of the owner-facing session itself.

## Activity class I: implementation (spec authoring)

| Spec | Tokens (tree) | Tool calls | Wall time | Spec's own † effort claim |
|---|---|---|---|---|
| Extensions (draft 0.2) | 302,462 | 74 | 56 min | 9–14† engineers, 12–18 months |
| Storage (draft 0.2) | 345,048 | 94 | 97 min | 7–10† engineers, ~30–50 EY |
| WebCrypto (draft 0.2) | 347,268 | 80 | 47 min | 3.5–5.5† engineers |
| Shell (draft 0.2) | 23,382* | 467 | 56 min | 14–20.25† engineers, 168–243 EM |
| ML service (draft 0.2) | 68,892* | 607 | 44 min | 14–20.5† engineers, 24–36 months |
| Phishing protection (draft 0.2) | 142,321 | 574 | 71 min | 5–8† engineers + 1.5–3 |
| Devices (draft 0.2) | 65,378* | 517 | 49 min | 23–31† peak, 70–110 EY |
| App host (draft 0.2) | 165,435 | 586 | 50 min | 16–39† incremental EY, ~21 months |

\* = the notification's usage field appears to cover the author's own calls only, with the parallel research strands reported separately (see below); the unstarred rows include their strand subtrees. The numbers are therefore a lower bound for the starred rows.

## Activity class I: research strands

| Strand | Tokens | Tool calls | Wall time |
|---|---|---|---|
| WebCrypto security/side-channels | 132,411 | 20 | 27 min |
| Extensions history | 120,568 | 60 | 33 min |
| Incompressible remainder (agentic economics) | 148,732 | 113 | ~7 min |
| Industry data (agentic economics) | 119,486 | 91 | 12 min |
| Token economics (agentic economics) | 158,231 | 66 | 10 min |
| RCTs and academic evidence (agentic economics) | 153,804 | 119 | 16 min |

## Activity class V: independent phase-8 review and corpus application

| Agent | Tokens | Tool calls | Wall time |
|---|---|---|---|
| Batch-3 decision application (corpus-wide) | 28,615 | 399 | 18 min |
| Batch-4 decision application | 49,696 | 533 | 22 min |
| Devices decision application | 83,474 | 435 | 21 min |
| App host decision application | 80,959 | 315 | 13 min |
| Phase-5 security/coding rebases | 161,180 | 253 | — |
| Phase-3 sweep (the residue) | 134,743 | 846 | 43 min |
| HGBA-2 2.4.0 rewrite | 114,325 | 256 | 17 min |
| HGBA-3 3.3.0 | 77,324 | 177 | 10 min |
| Stage 1 redesign (0.14) | 136,506 | 278 | 20 min |

## What these rows say (first, provisional reading)

1. **A complete, phase-8-reviewed component spec costs on the order of 10⁵–10⁶ tree tokens and well under two agent-hours of wall time** — against the specs' own † estimates of tens of engineer-years for the same component's full engineering life. The gap is exactly what Ledger B models: the spec document is the compressible part; the uncompressed remainder (measurement, conformance, fuzzing compute, review, craft) has not started, so no reading about the remainder follows from these rows.
2. **Class V is not free**: the independent fact-check and critique (class V) run as separate agent trees whose cost is comparable to the authoring cost, matching the literature's review-pressure finding (industry_data.md 7.x).
3. The rows are **inputs to C-6R's reversal trigger**, not evidence for restating the class ranges yet: the trigger fires when two consecutive components measure outside the envelope on the same side at their first real milestones, and these are authoring-time data points, not milestone measurements.

## Next rows to collect

- Per-agent token counts for the fact-check and critique trees specifically (class V split from class I).
- The owner-session orchestration overhead (this conversation's own usage), when instrumented.
- The first implementation measurements (FP-M0 etc.), which belong in `../milestones/`, not here.
