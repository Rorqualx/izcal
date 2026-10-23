---
name: first-principles-spec
description: Izcal's method for designing anything new: research the history of the area, analyze what came before and why it was built that way, its limits, technology mismatches and failures, then design from first principles and write a falsifiable spec. Use this whenever you plan, scope, design or spec a new Izcal component, tool or feature, and especially for web developer tools (DevTools, inspectors, debuggers, profilers, network or protocol tools, agent and MCP integrations, editor integrations). Also use it for any new planning document in reports/, even if the user only says "scope", "plan", "design doc", "spec it out" or "what should we build for X".
---

# First-principles spec

Izcal is meant to be a completely new browser, not a better copy of an existing one. Most design work in this field goes wrong in one of two ways: it copies the shape of what already exists (the side-panel inspector, the tab-per-tool layout, the protocol designed for one engine's internals) and inherits its limits, or it ignores history and repeats failures that other projects already paid for. This method avoids both. **Learn everything from the past, take no code and no template from it, and design for what the platform and the user actually need now.**

Every spec this method produces must satisfy HGBA-2's method constraint **M-1** (original, first-principles, spec-anchored implementation) and Stage 1 design decision **D-S10** (no code from another browser). If a component would otherwise come from another browser, **Izcal builds its own, following its specs**. That is the standing answer, not a question to ask.

## The workflow

Work through the phases in order. Each one produces a written artifact, because the next phase, the fact-checkers and future readers all need the evidence and not only the conclusion.

### 1. Frame

Write down, in a few lines:
- the problem and who has it (which users, doing which task);
- what Izcal's existing specs already decide or constrain. Read the relevant parts of `reports/HGBA 2 browser architecture spec.md`, `reports/Future software browser engine.md` (HGBA-3) and `reports/HGBA Stage 1 architecture design.md`, and note the IDs that apply (targets such as S-4c, decisions such as D-S11, building blocks such as B1 or B9);
- what would make the result a success, in terms someone could later measure.

### 2. History: what came before, and why

Build a chronology of the area from its origins to today. For each significant project, protocol or product, record:
- **what it did**, and the problem it solved at the time;
- **why it was built that way**: the constraints of its era (hardware, network, browser architecture, the standards that existed, business model, team size). A design that looks wrong today was usually right for its constraints, and knowing which constraint drove it tells you whether it still applies;
- **its limits**: what users could not do with it, and what it got wrong;
- **technology mismatches**: assumptions baked into it that are no longer true (single process, synchronous APIs, one engine's internals, desktop-only, a human always at the keyboard);
- **its fate**, and **why**: if it failed, was abandoned, merged or lost users, find the actual reasons from postmortems, maintainers' statements, issue trackers or usage data. Do not guess them. "It lost to X" is not a reason; why it lost is;
- **what survived** into later tools, and why it survived.

### 3. Today: the landscape and its gaps

Survey current tools and what developers struggle with, with evidence (surveys, issue trackers, forums, published studies). Then ask the questions that matter most for a new browser:
- **What is invisible today?** What do current tools fail to show, or show only in fragments?
- **What could Izcal expose that no one does?** Look at what Izcal has that others do not (see `references/izcal-capabilities.md`).
- **Which interactions are new?** New ways to work with the protocols, the data and the user's own tools, not only new views.

For web developer tools, also work through `references/devtools-lens.md`.

### 4. First principles

Set the existing tools aside and derive the design from:
- the users' real tasks (what they are trying to find out or change, not which panel they open);
- the platform's actual primitives: the specifications, the protocols and the engine's real data;
- Izcal's principles (HGBA-2 P1–P6, M-1) and its existing architecture.

List the assumptions inherited from existing tools, and decide for each whether to keep it or break it, and why. Breaking an assumption needs a reason; keeping one does too.

### 5. Lessons learned

Make a table with columns **project → what happened → the lesson → how Izcal applies it**. Lessons are ideas and evidence. They are never code, and never a template to reproduce (M-1).

### 6. Design

Describe the architecture and the user experience. Where there are real alternatives, present them with a recommendation, what it costs, and what evidence would reverse it. Integrate with Izcal's existing design: crate layers and rules G1–G9, process model, capability IPC, security and privacy principles, AI-off by default (HGBA-3 B10) and the agent principal (B9). Name new crates `hg-*`.

Check every design claim against what Izcal has **measured**, not only what its specs intend. The M0 results (`m0/M0_SUMMARY.md`, `m0/determinism_crossplatform.md`) and the process model are where designs most often break. For example, V8 is deterministic only under the fork's predictable platform, renderer processes are per site and not per tab, and isolation (HGBA-2 P1, I-1) forbids putting two sites' data in one process. When a design needs something that has not been measured, mark it as a risk and make measuring it a milestone.

### 7. The spec

Write it with the structure in `references/spec-template.md`. The parts that make it a spec rather than an essay:
- **falsifiable targets**, each with an ID, a metric, a threshold, how it is measured and what happens if it fails. Mark thresholds you set from first principles with †, and never invent a number and present it as measured;
- **inline links to primary sources** for every claim a decision or target depends on. The research notes are the evidence trail, but a reader should be able to check a key claim without opening a note;
- an **M-1 compliance** section: which dependencies are used, unmodified; what is built new; which specifications or protocol definitions each part is anchored to;
- **open questions** and **flagged claims**: anything single-source, unverified or conflicting, marked as such.

### 8. Verify

Before the spec is done:
- an independent **fact-check** of the claims that carry the design: dates, numbers, quotes, "X was abandoned because Y";
- an independent **design critique** against the existing specs, looking for contradictions, missed constraints and M-1 violations;
- every link resolves, and every `#fragment` exists.
- no link points to any third-party project's source code, and no value is derived from it (M-1 part 3).

Fix what they find, then record the result in the spec's header.

## How to run it with agents

The research is too broad for one context. Split phases 2 and 3 into **strands** (for example: history and postmortems, protocols and standards, invisible data and gaps, visualization and UX, integrations), and give each strand to its own researcher in parallel. Each researcher writes one file under `research_notes/<Report title>/<strand>.md`, using the format below. A synthesis author then reads every note plus the existing specs, and writes phases 4–7 into `reports/<Report title>.md`. A fact-checker and a design critic, who did not write the spec, then do phase 8 independently.

Brief every agent with this skill's path, so the method travels with the work. **Researchers never read or cite any third-party project's source code** (browser engines, libraries, tools, assistive technology, OS components: code files, headers, commits, diffs, code reviews), even for a single constant (M-1 part 3). Prose documents hosted in code repositories are fine only under OD-7's three conditions: no code file, header, diff or review is opened; an outside copy is cited where one exists; and no constant is taken that appears only beside code. Otherwise they use published prose and black-box tests; where only the source states a value, they record it as a question for a black-box measurement. Researchers share a limited web-search budget, so tell them to research their most important questions first. When the budget runs out, they read known primary pages directly and grade anything left unchecked as Medium or Low.

## Research note format

The existing notes in `research_notes/` follow this format; keep it:

```markdown
# <Report title>: <strand>

Compiled <date>. Primary sources were preferred (papers, specifications, maintainers' posts, release notes, issue trackers).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

## 1. <Topic>

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | ... | ... | [title](url) | yyyy-mm-dd | High |

**Implications:** what this means for Izcal, one or two sentences.
```

## Writing style

Match the existing reports. Use short declarative sentences, with tables for comparisons and targets. Bold the key terms, put sources inline as links, and use absolute dates. Say plainly what is unknown.

## Reference files

- `references/devtools-lens.md`: the question bank for web developer tools (read it for any developer-tool work).
- `references/izcal-capabilities.md`: what Izcal's architecture makes possible that other browsers cannot do (read it in phase 3 and phase 6).
- `references/spec-template.md`: the structure for the spec (read it in phase 7).
