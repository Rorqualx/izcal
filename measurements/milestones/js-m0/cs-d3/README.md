# CS-D3 re-run: the five soundness criteria against Izcal's own heap

9 November 2026. JS-M0 plan Phase 2 item 2; heap design review, "The CS-D3 re-run"; coding standard CS-D3 and CS-TY-10; JS spec 9.5 and Q-JS-10.

CS-D3 chose option (a) — compile-time re-entrancy safety through tokens and rooted handles — on an October 2026 spike that was built against a V8 fork. Under HGBA-2 2.4.1 that spike's engine-internal findings are withdrawn and its measurements stand as the 0.2 record only. Its five soundness criteria were left to be **re-run against Izcal's own `Mutator`, `Root` and `ScriptToken`**, which is what this is. Nothing here cites or derives from any third-party source code (M-1 part 3).

Two criteria did not hold on the first look. Both are fixed in the same change as this write-up, and both were found by trying to produce the evidence rather than by reading the code.

## Verdicts

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | The token is derivable only from `&mut Mutator`, or from a callback prologue that consumes the scope | **PASS for the GC token as it stood; PASS for the script token after a fix** | 21 refusals and 5 controls in `crates/hg-heap/tests/ui/`; 3 refusals and 1 control in `crates/hg-js-runtime/tests/ui/`; `ScriptToken::for_job` now takes `&mut Mutator` and is disallowed outside the two job loops |
| 2 | Every entry point that can run script is reachable only through token-taking wrappers | **PASS, and mechanical** | `tools/token-audit.py`: 28 `runtime` declarations, 14 declaring `call_user`, each taking `&mut ScriptToken` and no other, plus the hand-written `RuntimeBase::tail_call` |
| 3 | Traced fields are never inside a cell the marker reads without the token | **PASS by construction, and the one unchecked obligation is now checked** | Every `NodeSpace` method takes `&self` and every slot is an atomic, so no borrow exists to straddle a GC point; and an external slot the space forgets to trace now panics on the first read after a cycle |
| 4 | A written multi-node mutation pattern, exercised on `insertBefore` with an observer and a reaction | **PASS, and stronger than the criterion asks** | `insertBefore` on the node space with an allocating callback mid-mutation, in all three collector modes; and script re-entry from a host function is unrepresentable, not merely guarded |
| 5 | Miri and loom tests of the cell API | **PASS for Miri; loom deferred to JS-M1, with a reason** | the `miri` CI job runs `hg-heap`'s whole suite with the moving test collector; loom has no interleaving to explore until concurrent marking exists (below) |

No criterion failed in a way that triggers CS-D3's fallback to option (b). The reversal trigger's other half — "the prototype costs > 2% on the M2 bindings benchmarks" — falls due at M2, as it says.

## Q-JS-10, settled: the tokens stay separate

**`ScriptToken` is a separate type from `Mutator`.** The JS spec recommended this and the code already did it; what was missing was the reason stated as something falsifiable, since "two tokens can drift" is a real cost and drift is what criterion 2 is about.

The deciding argument is Q-CS-7, which the owner already settled: **allocation does not take the script token.** If the two merged, `alloc` would take the script token, and every `TokenCell` borrow would therefore end at every allocation — not at every call that can run script, which is what the cell is for. A DOM operation that holds a borrow of a side table and allocates a JS string in the middle of it is ordinary, correct code; under a merged token it would not compile. So merging does not tighten the rule, it changes which rule is enforced, and to a stricter and wrong one.

The drift the question worried about is answered by making it checkable rather than by removing one token. The two tokens guard two different things — `Mutator` guards GC references, `ScriptToken` guards the re-entry — and `tools/token-audit.py` holds the second one's reach to exactly the operations whose descriptions declare `call_user`. Drift is now a failing check, not a latent inconsistency, which is what makes two tokens affordable.

## The two defects this found

**`ScriptToken::for_job` took no arguments.** Any crate could mint a script token out of nothing, and "only the agent's job loop calls this" was a comment on the function. Criterion 1 names `&mut Mutator` as the only derivation, so the signature now takes it, and `clippy.toml` disallows the call everywhere else: the two sites that mint one — the job loop and a FinalizationRegistry cleanup job — each carry an `#[expect]` naming the rule, so the list of exceptions is the list of minting sites. A third site was added to confirm clippy refuses it.

Nothing had gone wrong because of this: a forged token is useless without a `&mut Mutator<'h>`, and no path hands one out. That is why the defect is worth recording — it was invisible to every test, and it would have become load-bearing at M2, when the DOM gives the token something of its own to guard.

The token is deliberately **not** branded with `'h`. A brand sounds stronger and buys nothing here: the token holds no heap data, and every method that consumes it takes a `&mut Mutator<'h>` beside it, so a token that outlived its `enter` could not be used with the next one's mutator anyway.

**The external traced slot's tracing obligation was unchecked.** `heap-node-space-amendments.md` recorded it and routed it here: an `ExternalSlot` is traced because the space's `trace` visits it, so a space that forgets one leaves it un-updated, and a read after a moving cycle answers with the offset the object used to have. H-2 keeps that inside the cage — a wrong answer, not an invalid access — but a stale reference is what the rooting rules exist to make impossible.

That document offered two directions, both structural: brand the rare-data tables per heap, or make the space declare its slots once so `hg-heap` holds the list. **This change takes a third.** Each slot now remembers the cycle it was last written or traced in: `set` stamps it with the cycles completed so far, the collector stamps it again as it traces it, and `get` asserts in debug builds that the stamp is not older than the last completed cycle. A forgotten slot therefore panics on the first read after a cycle, naming the fault.

Why the third way is the right one here, not a stopgap:

* it catches the omission **per slot**, where a declared-slot registry catches it per space — a space that registers its wrapper column and forgets its listener column passes the registry and fails this;
* it costs nothing in a release build (the stamp is a second `AtomicU32` present only under `debug_assertions`, so the slot stays four bytes) and runs under Miri, where the moving test collector moves every object on every allocation;
* the two structural options are decisions about `hg-dom`'s tables, which do not exist until M2; this one needs nothing from the embedder and does not foreclose either of them.

What it does not do is make the obligation a type error, and it does not touch the cross-heap half, which keeps the debug assertion it already had in `ExternalSlot::set`. Both remain open in the same terms the amendments document states.

## Method, and what keeps a green result from being empty

Criteria 1 and 4 rest on code the compiler must refuse, which no test can observe: the only evidence is the refusal. `tools/refusals.sh` compiles 24 `fail-*` fixtures that must not compile and 6 `control-*` fixtures that must, across two sets (the heap's brands, and the engine's script boundary), and `tools/checks.sh` runs it as `refusals` — in the required `checks` job and in the pre-commit hook, so a variance or signature change that quietly makes one of them legal fails CI instead of passing silently.

Four separate ways a green run could have meant nothing, each checked by breaking it:

1. **A refusal is not evidence by itself.** Every `fail-*` case names the error code it must be refused with. Without that, a stale rlib (`E0460`, `E0464`), a missing crate (`E0463`) or a typo would refuse everything and read as the whole discipline holding. This fired for real twice while the fixtures were being written.
2. **The code is not always enough.** The brand cases also name the note where rustc says which of the heap's types is invariant over `'h`. That check is not decoration: it caught `fail-cross-heap-store` passing while being refused because the outer mutator escaped the inner closure, with the brand never consulted. The four cross-heap cases were rewritten to take two separately-branded mutators as parameters, which isolates the brand; the nested shape became a case of its own that records both refusals.
3. **The controls must compile.** A run in which they do not is a broken harness whose other results prove nothing.
4. **The rlibs must be the ones the run built.** They come from cargo's own artifact messages, not from the newest `lib*-*.rlib` in `deps`, which also holds other feature sets and profiles.

The same discipline applies to the tests. The multi-node mutation test passed in a first version that proved nothing — its wrappers were still rooted in the scope, so they, not the node scope, were keeping the nodes alive — and the companion test that leaves the node scope out is what caught it. Both now fail if the node scope's roots are removed, which is how the pattern is shown to be load-bearing. The criterion-3 test has a control space beside the forgetful one, so a stamp that fired always would be caught too.

## Criterion 5: Miri yes, loom not yet, and why

Miri runs `hg-heap`'s whole suite, with the moving test collector that moves every object on every allocation, in CI's `miri` job on every pull request. Everything the five criteria rest on is in that suite.

**loom is deferred to JS-M1, deliberately.** loom explores thread interleavings, and at JS-M0 there are none to explore: `Mutator` is `!Send` and `!Sync` (two refusals show it), one agent heap has exactly one mutator, and the concurrent marker that would make the slots' atomics interesting is JS-M1 work — the heap design review's scope section says so, and the atomics exist today only so that the marker and its barrier can arrive without changing every field. A loom model written now would be a model of a protocol that does not exist, and would stand as evidence for a design nobody has written.

This is a deviation from the JS-M0 plan's Phase 0 item 5, which lists "a `loom` test target". The criterion itself is met by Miri at JS-M0; loom's half of it falls due with concurrent marking, and should be a condition on that work rather than on this.

It also means Izcal still has no `loom` dependency, which is worth saying plainly: the workspace's only external crates remain `libc`, `windows-link` and `windows-sys`.

## For the owner

* **Q-JS-10 is answered** — separate tokens — on the Q-CS-7 argument above. It needs confirming, not deciding again, unless the argument is wrong.
* **Criterion 5's loom half is deferred to JS-M1** with the reason above. If you want a loom model at JS-M0 anyway, it would be a model of the concurrent-marking protocol before that protocol is designed, and it would add the project's fourth external dependency.
* **Two openings stay open** in the terms `heap-node-space-amendments.md` states them: making the tracing obligation a type error, and the cross-heap half of the external slot. Both are decisions about `hg-dom`'s rare-data tables, so M2.
* **The 0.2 spike's third follow-up** — "the mechanical entry-point audit built into `hg-webidl-codegen` and the engine's conformance harness" — is met for the engine by `tools/token-audit.py`. The WebIDL half arrives with the bindings at M2.
