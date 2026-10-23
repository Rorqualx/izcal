# Agentic measurement: FP-M0 and the first substrate code, 22–23 October 2026

Second set of rows for C-6R's Ledger B. The 21 October record measured background agents only and listed **the owner-facing session's own usage** as the missing row. That session did nearly all of this work, so this record measures it directly. These are **measured** values, not † estimates.

## Method

- **Source:** the session transcript (`~/.claude/projects/…/c4776725-….jsonl`). Every model response there carries the API's usage fields: input tokens, output tokens, cache-read tokens and cache-write tokens. Responses are de-duplicated by message ID. Tool calls are counted from the responses' tool-use blocks.
- **Window:** 22 October 2026 21:43 UTC (the start of FP-M0's Linux leg) to 23 October 2026 ~22:00 UTC, as 34 owner requests.
- **Unit:** one owner request, from the request to the last response before the next request. "Active minutes" runs from the request to that last response, so it includes time spent waiting on rig runs (for example, 2-minute memory samples), not only model time.
- **Classes:** each request is assigned to one class by what it produced:
  - **I-measure:** FP-M0 measurement work.
  - **I-code:** engine code, `hg-substrate-os`.
  - **I-spec:** spec edits.
  - **V:** review, and fixes driven by review.
  - **O:** operations: pull requests, merges, CI, cleanup, the journey page.
- **Subagents:** one subagent ran in the window, the code review of PR #30: 6 tool calls, 5,684 output tokens, 52,671 input + cache-write tokens, 255,189 cache-read tokens, 1.1 minutes. It is reported separately below.
- **Two models served the session:** `deepseek-v4-pro` through the session's model routing, and `claude-opus-5-5`. Output tokens are split by model; prices differ, so this record gives tokens, not dollars.

## Rows (owner session)

| Class | Requests | Active min | Tool calls | Output tokens | Input + cache-write | Cache read | Output by model |
|---|---|---|---|---|---|---|---|
| I-measure | 5 | 278 | 301 | 185,027 | 151,458 | 57,346,448 | deepseek 163,372 · opus 21,655 |
| I-code | 8 | 116 | 334 | 284,253 | 788,392 | 163,414,653 | deepseek 181,831 · opus 102,422 |
| I-spec | 1 | 2 | 17 | 9,967 | 9,526 | 5,188,352 | deepseek 9,967 |
| V | 6 | 37 | 113 | 82,769 | 886,836 | 24,173,544 | opus 82,222 · deepseek 547 |
| O | 14 | 75 | 249 | 129,563 | 218,824 | 87,622,691 | deepseek 98,648 · opus 30,915 |
| **Total** | **34** | **510** | **1,014** | **691,579** | **2,055,036** | **337,745,688** | |

The **I-measure** row's last request also covered this record and the wakeup PR (#33). **V** includes the reconciliation PR (#29) that the commit review produced, and the fixes from PR #30's review.

## What it produced

- **FP-M0 half A:**
  - the incumbent matrix on Linux and Windows;
  - cache sizing, the PSI staircase, per-element memory, Chrome's soft process limit;
  - the Safari leg;
  - Q-17 closed item by item.
- **FP-M0 half B:**
  - `hg-substrate-os` with sandbox backends for macOS, Linux and Windows;
  - the empty-process floor and spawn distribution on all three OSes;
  - the Windows loader stall root-caused (STATUS_DLL_INIT_FAILED from console creation) and fixed;
  - the AppContainer tier that denies user-file reads;
  - idle wakeups measured and cut from 2/s to 0 on all three OSes.
- **Repository:** 14 component specs recovered onto main (#29), clippy on three OSes in CI, the Node 24 checkout, the branch cleanup, and seven archive tags pushed to GitHub.
- **Pull requests merged in the window:** #20–#32, 13 in total; #33 is open.

## First reading (provisional)

1. **Cache reads dominate the token volume:** 338M cache-read tokens against 2.1M input + cache-write and 0.7M output. The context is re-read on every response, so a long session's cost tracks context size × responses, not output. Ledger B's token envelope (15–31T) should be checked against this shape, not against output tokens alone.
2. **I-code cost more per request than I-measure** in tokens (284K vs 185K output over 8 vs 5 requests), but less active time (116 vs 278 min). Measurement time is rig-bound: samples, spawn runs, reboots.
3. **Review and operations are a large share:** V + O = 20 of 34 requests and 31% of output tokens. Merges, CI and cleanup are not free; the 21 October record had no O class, and later records should keep one.
4. These are **inputs to C-6R's reversal trigger, not a trigger**: one session, mixed models, and the measured milestones (FP-M0) are not yet complete.

## Next rows to collect

- The same measurement for the next session, so two consecutive data points exist per class.
- A per-model price table at the session's dates, to turn tokens into cost for Ledger B's dollar envelope.
