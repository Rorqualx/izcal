# Spec template

Use this structure for a spec written with the first-principles method. Omit a section only when it truly does not apply, and say so in one line rather than deleting it silently.

```markdown
# <Title>

*<What this document is> for Izcal. Date: <date>. Status: draft 0.1. Research notes: `research_notes/<Title>/`. Relates to: <spec IDs it extends or amends>.*

*Revision 0.1 (<date>): first draft.*

<Summary paragraph: the problem, the central idea, and the three or four decisions that matter most.>

## 1. The problem and who has it
## 2. History: what came before and why it was built that way
   - Chronology table: year | project | what it did | why that way | fate and why
   - Technology mismatches: assumptions that no longer hold
## 3. Today: the landscape and its gaps
   - What current tools cannot see or do, with evidence
## 4. First principles
   - Users' real tasks; platform primitives; Izcal's principles
   - Assumptions kept and broken, each with its reason
## 5. Lessons learned
   - Table: project | what happened | lesson | how Izcal applies it (ideas only, M-1)
## 6. Design
   - Architecture: crates (`hg-*`), processes, protocols, data sources
   - User experience
   - Integrations
   - Security, privacy and consent
   - Alternatives, each with a recommendation and its reversal trigger
## 7. M-1 compliance
   - Dependencies used unmodified; what is built new; what each part is anchored to (specifications, protocol definitions)
## 8. Targets
   - Table: ID | target | metric and threshold (with † where set by this document) | how measured | if it fails
## 9. Milestones and staffing
   - How the work fits Stage 1 design Section 8 (milestones, tracks)
## 10. Risks
## 11. Open questions and flagged claims
   - Open questions
   - Claims that are single-source, unverified or conflicting
```

**Target IDs:** use a short prefix unique to the spec (for example `DT-` for developer tools), and check that it does not collide with prefixes already in use (search `reports/` for the prefix first).
