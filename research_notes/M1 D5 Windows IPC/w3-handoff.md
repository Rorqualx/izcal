# D5 W3, a fresh pipe per move: handoff (26 Oct 2026)

Branch `ipc/d5-windows`, worktree `izcal-wt/m1d5`. Never push. Book rigs under session `m1-substrate`. The design is in the M1 plan (`~/.claude/plans/m1-finish.md`, section "D5 W3: a fresh pipe per move"), with the decision recorded below it: **option 2, the broker keeps the old pipes** (coordinator, 26 Oct). This note holds the state and the exact next steps for the worker who builds it.

## Decided
- **The rule.** After every move of a channel end, the moved end and its peer are on a new pipe. The broker creates it, with every channel's flags; children never choose flags.
- **Who switches.** The peer switches before the moving end is pulled. The broker peeks the moving end (no close-source), looks up the other end's holder, sends it `RebindValue` or `RebindId` with the new end already given into its table, and waits for `Rebound` (deadline 5 s; past it the relay is refused and the channel ends in `Closed`). Only then does it pull with close-source and check with `CompareObjectHandles` that the pulled object is the one it peeked.
- **The settle loop.** The broker re-checks where the peer end is after each `Rebound`, because both ends may be moving at once from different processes:
  - held by a child: rebind it;
  - held by the broker: switch it locally;
  - in a ticket: swap the handle in `held`;
  - gone: no new pipe;
  - in the same relay: give the new holder both new ends.
- **Old pipes (option 2).** The broker keeps the old pipe's server end in a per-channel FIFO and does not hand it to the new holder.
  - On `listen`, a receiver that may have old pipes sends one `TakeDrains{rx}`. The broker peeks `rx` on the child's worker, checks the caller holds the receiver, and gives the queued server ends in order. The receiver reads each until `ERROR_BROKEN_PIPE`, then its current pipe, and reads nothing while the request is out.
  - A receiver that is already listening when its sender moves keeps its old pipe locally as its own drain.
- **The limit.** At most **16† queued old pipes per channel** († = set from first principles, not measured). A move that would make it 17 is refused, the channel ends in `Closed`, and the reason is recorded (a `Refusal` code, plus a ledger/transport reason in the docs). This needs a test.
- **Quotas.** Queued old pipes count against the per-child quota (the channel's creator) and the per-channel quota. They are released when the channel dies, and when its creator or the holder dies (`child_gone`, `sweep`).
- **Ledger.** One key per channel: the first pipe's `PipeKey`. Each new pipe's key (creator = broker, from its name) is an alias of it, and `key_of` resolves aliases. Aliases are dropped when that pipe's server end is handed out and later closed, or when the channel is swept. Bindings, bound senders, tickets' carriers and quota counts stay on the channel key.
- **Credits.** Credits are cumulative (`on_credit` accepts an equal count), so credits stranded in an old pipe are superseded. A receiver writes its current delivered count as its first record on a new pipe, and when it listens after a move.
- **Order.** Records keep their `seq`, and the reader reads each old pipe to its end before the next, so `check_record` still catches a gap. The `after` rule is unchanged: item 4 drains a carried sender's writes before the relay.

## Done
- **Probes, round 4** (53834ff, `windows_facts.md`):
  - W10: pending reads complete in issue order across processes.
  - W10b: with records queued and the writer closed, the former holder's pending read takes record 1, and the new holder reads the rest, then 109.
  - W11: the writer closing an empty pipe fails a pending read with 109.
  - W12: a broker-made pipe works in an AppContainer child.
- **Option C** recorded in the m1-d5 README as a candidate for later (0953fb8).

## Work in progress, not committed
- **`w3-wip-frame.patch`** (this directory; apply with `git apply`) adds:
  - control records `RebindValue{req,end,new}` (kind 12, 40 bytes), `RebindId{req,id,new}` (13, 40 bytes) and `Rebound{req,switched}` (14, 24 bytes);
  - a drain flag in each item's first reserved byte, allowed only on a `Receiver` in `Redeemed`;
  - frame tests (they pass on the Mac);
  - a corrected comment on which lengths the round-trip proofs cover (Closed is 24 bytes, and records of 3 to 16 items have no harness of their own).
- **For option 2:** the drain flag in `Redeemed` can carry the old pipes in the `TakeDrains` reply, which still needs its own record kinds (`TakeDrains{req,rx}` and a `Drains{req,items}` reply). `windows.rs` must be updated for the new variants and for `Item.drain` before anything builds on Windows. That is why the patch is not committed.

## Open: measure, don't assume (coordinator)
A former holder may have left reads pending on an old pipe. W10 shows the first such read takes the next record. One process can post **several** overlapped reads before it moves the end, so the bound may be "every record queued at the move", not "one record".

Probe **W13** answers this: one child posts N pending reads on a server end, then the writer writes M records. Count how many the child takes, and whether the broker can stop it before handing the pipe over:
- the broker's `CancelIoEx` fails for another process's requests (W8b, 1168);
- the broker reading the old pipe itself would only queue behind the stale reads;
- the broker cannot close the old pipe's other end early, because that is the sender's pre-switch stream.

Whatever the probe shows goes into `windows_facts.md`. Then document the exact bound in the module docs and in the README's "what Windows doesn't attest" list: a compromised former holder can take only records sent while it held the end, never one sent after the switch, and the new holder sees a `seq` gap reported as `Violation(Sequence)`. If the probe shows a way to close even that, implement it instead.

## Next steps, in order
1. **W13 on rig253** (light booking). Record the result in `windows_facts.md`, and set the bound text in the plan.
2. **Ledger:** aliases (`add_alias`, `canonical`, drop on sweep), a per-channel old-pipe FIFO with the 16† cap, and quota charging and release. Unit tests on the Mac. The ledger model proofs do not cover aliases or the FIFO; note that, or add a model step.
3. **Frame:** apply `w3-wip-frame.patch`, then add `TakeDrains` and `Drains`.
4. **Transport, peer side:**
   - `End` gains an old handle and flags for an operation still in flight on it.
   - A sender's switch: new writes go to the new pipe, a pending write on the old one completes, then the old one is closed. Its credit read on the old pipe is cancelled, and that 995 is ignored.
   - A receiver's switch: the current pipe becomes a local drain, read until 109. The first credit goes on the new pipe.
   - Handling `Rebind*`, with `Rebound{switched:false}` for an end no longer held or whose relay is already out.
   - Channels this process makes keep their pipe id (`RebindId`).
   - `listen` sends `TakeDrains` when the end came by a move.
5. **Broker relay:** peek first, the settle loop (`Rebound` answered inline on the IO thread, never on the relayer's worker, which waits on it), `sys` helpers to create the new pipe and compare objects, pull and verify, ticket handles per end, give-map and alias updates, the FIFO push, and `TakeDrains` handling on the child's worker.
6. **Tests:**
   - `an_old_holders_pending_read_gets_nothing_after_a_move` (must fail on today's code);
   - order across a move with records queued;
   - a sender move with a stranded credit (the new holder is not stuck `Full`);
   - the 17th queued old pipe closes the channel, with its reason;
   - quota release when the channel, the creator or the holder dies;
   - the rebind deadline;
   - both ends moving at once.
7. **Measure** under an exclusive rig253 booking, with `rig check` before each run: the handle-carrying round trip before and after, and the `TakeDrains` round trip at `listen`.
8. **Docs:** module docs (W3 section, threads and limits), the m1-d5 README ("what Windows doesn't attest", the 16† limit, measurements), and the plan.
9. **End checks:**
   - clippy on 3 targets;
   - the Windows suite 10 times in a row;
   - workspace tests on rig253 and hueyj;
   - the affected Kani (frame, and ledger if the model changes) on hueyj, one harness at a time, `prlimit --as=10737418240`, 15-minute limit, the exact-name idle check `pgrep -x cbmc|cargo-kani|kani-driver`;
   - the dst hash `f06e8ea9…911b` on this branch.

## Ledger proofs (separate, finishing in this session)
The three ledger Kani harnesses now run on a fixed-array model of the ledger, with a differential test that holds the model to the real ledger (`the_model_agrees_with_the_ledger`). The why is measured in `ledger.rs`'s `model` doc. They verify in the 13:22 hueyj slot (b-076556); this session commits them and updates the README's Kani table.
