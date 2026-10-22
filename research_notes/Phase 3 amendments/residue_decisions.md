# Owner decision record: the phase 3 residue

Decided 21 October 2026. The owner settled every item the phase 3 wave left open:

- **C-11 (Jitless ceiling).** S-4a takes the JS spec's text plus the security scope's CET/BTI and ACG wording; the default tier stays JS-D13's — Full by default, Jitless for Izcal-only renderers and the banking container — and C-11 is withdrawn. The Jitless ceiling and its enforcement are identical either way, so no security claim changed; the security scope's "jitless by default" rule is re-decided to match at its rebase. Applied: HGBA-2 2.4.1 (9.2, S-4a, D2), Stage 1 0.14.2 (11.1), ledger.
- **C-19 (DevTools replay unit).** The ledger's recommendation is adopted: recordings are per process and joined by IDs (P20's rule); a replay host runs one process kind; the graphics spec's multi-process unit replays as a group of hosts with the compositor inputs recorded at the broker boundary. DevTools owns the surface and records the definitions. Applied: HGBA-2 2.4.1 (9.2), Stage 1 0.14.2 (11.1), DevTools 0.3.5, ledger.
- **C-21 (PDF pagination).** The PDF spec (accepted, batch 1) is the named owner of the writer, viewer and process; the pagination contract sits between layout's paged layout and `hg-pdf-write`, resolved between the two owners under the Stage 1 13.2 row. G-14 closes with it. Applied: Stage 1 0.14.2 (11.1), ledger.
- **G-4 (riscv64).** X-1 stays; riscv64 is covered by the JS, bindings and DOM track per JS-D22 — the 5–7† riscv64 engineers live inside the JavaScript engine spec's own 65–82† — so no separate 8.3 row exists and Stage 1's riscv64 references are no longer conditional. Applied: Stage 1 0.14.2 (8.3), ledger.
- **A-SH-39 (Windows always-on memory).** The ~74 MiB† always-on ceiling on Windows is accepted, measured at SH-M0 with the stated trade-off (the UI host is where TSF/IME and AT code lives); FP-1's ≤ 64 MiB† stands for every non-Windows platform, and the Windows line is FP-1's dated amendment. Applied: shell 0.3.2, footprint spec FP-1, ledger.
- **A-LA-1/2/3 (licence audit).** Confirmed as recommended: the table rule (no table taken from RFC code components; per-publisher confirmation of codec constant tables), the Vulkan loader/SPIR-V/ash dispositions, and the content-blocking dataset dispositions. Applied: licence audit dated note, ledger.

Nothing else changes: these resolutions edit no thresholds, milestones or budgets beyond the A-SH-39 ceiling the owner just accepted.

**Appended 21 October 2026 (the last three):**
- **A-SH-10.** The accessibility baseline is AX-D4's validated broker mirror, read by the UI host through a read-only mirror client that is the single seam if AX-M0 chooses otherwise. Applied: accessibility 0.2.4, shell 0.3.3.
- **A-SH-11.** AX-D9's UI-host AT-adapter placement and FP row are confirmed provisional until AX-M0 measures them. Applied: accessibility 0.2.4, shell 0.3.3.
- **A-SH-12.** The security owner adopts AX-D11's proposal: AT-invoked approval of high-risk prompt classes requires person-present evidence, and until it is implemented a high-risk AT approval is not accepted and the AT action moves focus only. Applied: accessibility 0.2.4, security scope 0.3.6.

**Appended 21 October 2026 (the final item):**
- **Q-SH-3 (history retention).** Decided as recommended: **90 days† of full visits and 12 months† of the aggregate record, user-configurable**. Applied: shell 0.3.6, ledger.
