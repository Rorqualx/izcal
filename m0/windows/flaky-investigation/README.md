# The Windows idle-thread flake investigation, captured from rig253

Captured 15 November 2026 under the owner's DEL-1 answer. These twenty files sat
**uncommitted and untracked** in `C:\Users\Joe\izcal-flaky` on rig253, a
checkout pinned at `22364db` (31 October 2026).

They are the working material of the Windows flaky-test hunt whose conclusion is
recorded in PR #70 and PR #74: twelve hand-written PowerShell probes (`aff`,
`catch`, `catch2`, `ev`, `exp`, `hold`, `loop`, `mon`–`mon4`, `threads`), six run
logs, and `fix.diff`.

**The fix itself is not at risk and is not the reason for this capture.** The two
tracked modifications in that checkout — 84 insertions across
`crates/hg-substrate-os/src/ipc/windows/{sys.rs,tests.rs}` — **landed**: all 55
substantive added lines are in `main` today, with `STEP_MAX` appearing six times
and `pin_to_this_processor` four. Only the investigation scripts were unique, and
only `reuse.ps1` among them was already committed.

They are kept because a flake hunt's instruments are worth more than its
conclusion when the flake returns: these are what distinguished the DiagTrack ETW
enable from an idle-thread race, and rebuilding them would mean rediscovering the
question rather than the answer.

**The hueyj half of the same sweep is not here.** `~/hgba-m0`'s non-build
contents are V8-era evidence; the repo's `binary` check refuses archives and the
unpacked sources fail `rustfmt` and the SAFETY-comment rule, correctly, because
as-measured evidence must not be reformatted to Izcal's standard. They were left
in place on hueyj (only that directory's 19.98 GB of build output was removed)
with a verified second copy at `~/Documents/Cline/izcal-archive/`.
