# Agentic engineering economics: frame

Compiled 21 October 2026. Owner direction: this is a fully agentic coding project; translate the staffing and engineer-year figures into agentic time and token use, accurately, from evidence — feeding a staffing-methodology amendment to ledger C-6 and Stage 1 design 8.3.

## The problem and who has it

Izcal's staffing figures (C-6: about 551–719 engineers† full strength, roughly 2,000–3,400+ engineer-years†, Stage 1 exit 48–60 months) were derived from first principles against historical comparables: real engine team sizes and schedules from the Firefox, WebKit and Chromium record, which predate usable agentic development. If development is done agentically, those figures overstate calendar time and headcount, but by how much is not established anywhere in the corpus. The owner, who will staff and fund this, needs the numbers restated in the units the work will actually consume: agent hours, tokens and cost.

## What the existing specs decide or constrain

- **C-6** (ledger) and **Stage 1 8.3** carry the staffing statement; the corpus's convention is that restating them is an owner decision recorded with a dated revision line, not an editor's rewrite.
- **†-discipline**: thresholds marked † are set from first principles; a translation is therefore another first-principles derivation, marked † until the first real agentic milestones measure it (the specs' existing measurement milestones are where it becomes measured).
- **The specs' own activity classes**: each spec's Section 9.x breaks staffing into tracks (e.g. parsers, runtime, lab, language data, review, directory operations). Any accurate translation must be per activity class, not a flat multiplier, because agentic coding compresses implementation work and does not compress measurement, verification or craft work.
- **Phase 8**: independent fact-check and design critique are part of the method and are themselves partly agentic, partly human review.
- **Falsifiability and reversal**: every decision in the corpus carries a reversal trigger. The translation must carry one too (e.g. "if the first N agentic milestones measure outside the translated envelope, restate").

## Success, measurably

1. Every translation factor rests on a dated, sourced evidence row with a confidence grade (High/Medium/Low), in the research-note format.
2. The model is activity-class based with stated ranges and error bars, and says which classes are incompressible (lab measurement, conformance runs, fuzzing CPU-years, licence and legal review, type-design craft, owner decision bandwidth).
3. Output: per-component agentic time (agent-hours and wall-clock), token use and dollar cost at current and projected prices, plus the aggregate totals.
4. A proposed owner decision (new ID, e.g. C-6 restatement) with a dated record and a reversal trigger, so the corpus stays falsifiable rather than silently optimistic.
5. No third-party source code is read for any of this (M-1 part 3, HGBA-2 2.3.3); published papers, studies and vendor pricing pages are prose evidence.

## Strands

1. **RCTs and academic evidence** — controlled measurements of developer speedup (effect sizes, confidence, task types, experience levels), and the quality/rework counter-evidence.
2. **Industry data** — DORA, GitClear, Stack Overflow, enterprise deployments, time-to-merge and rework analyses.
3. **Token economics** — tokens per task and per agent-hour, price-per-1M-token trajectory 2023–2026, inference cost curves.
4. **The incompressible remainder** — evidence for what agentic coding does not compress in Izcal specifically: WPT/conformance compute, fuzzing CPU-years, the AT lab, licence and legal review, type-design craft, independent review, owner decision bandwidth, hardware time.
