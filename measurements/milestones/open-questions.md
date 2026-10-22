# The open-question tracker

Started 22 October 2026. Every component spec carries its own Q- list; each
question's gate (which milestone measures it, which owner decision settles
it, or which later stage admits it) is stated next to the question in its
spec. This tracker is the index: per-document counts, the resolution paths,
and the questions that are startable now. A question closes in the spec's
own Q- section with a dated line when its gate resolves it — the decision
log then records it, per the footprint spec's item-by-item rule.

## Inventory (open Q- IDs per document, 22 October 2026)

| Document | Open Q- IDs | Primary gates |
|---|---|---|
| Graphics and rendering | 25 | GR-M0–M3 matrix, GR-D11's timer measurement |
| JavaScript engine | 24 | JS-M0–M3, the riscv64 CI, WPT velocity |
| Layout | 23 | LY-M0–M2, the corpus measurements |
| Accessibility | 22 | AX-M0 (the AT lab), AX-M1 study |
| DOM, bindings and event loop | 21 | DM-M0–M2, WPT, the JS engine's M0 |
| SVG | 21 | SV-M0–M2, the corpus |
| HTML and CSS parsing and style | 20 | ST-M0–M2, WPT |
| Phishing protection | 20 | PH-M0–M3, the feed licences (owner) |
| Animation / Fonts / Networking | 19 each | AN-M0–M2; TX-M0–M2, the type-design waves; NW-M0–M5, the root program |
| Editing / Images and media | 18 each | ED-M0–M2, IME matrix; MD-M0–M2, the patent decision (owner) |
| Shell / Devices | 17 each | SH-M0–M5; DV-M0–M5, DV-E exceptions (owner, proposed) |
| Content blocking | 16 | CB-M0–M3, list licences |
| ML service | 15 | ML-M0–M3, the corpus budget (owner) |
| PDF | 14 | PD-M0–M2 |
| DevTools | 13 | DT-M0–M4 |
| App host | 10 | AH-M0–M5, the AH-D8 exceptions (owner) |
| Storage | 10 | SG-M0–M4 |
| Shell UI design / Extensions / WebCrypto | 8 each | UI rows; EX-M0–M2; WC-M0–M4, Q-WC-1 registry timing |
| Resource footprint | 2 | FP-M0 (question 17) and FP-M3 (question 19) |
| HGBA-3 / Stage 1 | 2 each | Direction-doc rows, G-4-era questions |

## Resolution paths

1. **Measurement milestones** (most questions): the question names its
   milestone in its spec; the milestone folder in `measurements/milestones/`
   owns the run and the decision log records the close.
2. **Owner decisions** (a minority): licence and patent calls (the MD codec
   tables, the phishing feeds, Q-ML-12's corpus budget), the DV-E1–E6
   exceptions and the AH-D8 exceptions — these stay listed here until the
   owner decides; they cannot be measured into existence.
3. **Later stages**: questions whose feature ships in Stage 2/3 resolve
   when the stage opens (extensions' Stage 2 items, sync, WebMCP).

## Startable now (FP-M0's own list, runnable on the current rigs)

- **FP open question 17**: which OS counters each incumbent's reported
  footprint follows (macOS counter mapping — extend the matrix), macOS
  accounting after `MADV_FREE_REUSABLE` (a small standalone probe — in
  this folder), HTTP-cache sizing (black-box observation), Linux PSI
  reactions (the .185 rig).
- **The cross-site tab mix** (the matrix's stated caveat): 30 tabs across
  distinct domains, Chrome and Firefox.
- **The Linux and Windows matrix legs** (booked rigs), **Safari** (a
  dedicated session).

The tracker updates whenever a Q- closes or a new Q- is filed; the specs
remain the source of truth for each question's wording and gate.
