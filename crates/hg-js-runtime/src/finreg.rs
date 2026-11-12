//! FinalizationRegistry cells (ECMA-262 26.2, 9.13): registering and
//! unregistering, what a canonical cycle's cleared cells start, and the
//! cleanup job.

use hg_heap::{Any, Gc, Mutator, Root, Scope, Value};
use hg_js_ops::heap::{Cell, FinRegObj, HostFnObj};
use hg_js_ops::support::{Host, OutOfMemory, ScriptToken, Throw};

use crate::{AgentRoots, Array, JobError, JsRuntime};

/// Cell states.
const LIVE: u32 = 0;
const CLEARED: u32 = 1;
const REMOVED: u32 = 2;

fn reg<'h, 'm>(m: &'m Mutator<'h>, v: Value<'h, 'm>) -> Option<Gc<'h, 'm, FinRegObj>> {
    match v {
        Value::Ref(g) => g.cast::<FinRegObj>(m),
        Value::Small(_) => None,
    }
}

fn cell<'h, 'm>(m: &'m Mutator<'h>, v: Value<'h, 'm>) -> Option<Gc<'h, 'm, Cell>> {
    match v {
        Value::Ref(g) => g.cast::<Cell>(m),
        Value::Small(_) => None,
    }
}

impl<'h> JsRuntime<'h, '_> {
    /// FinalizationRegistry.prototype.register step 7: a new cell at the
    /// head of the registry's list. Its units were charged by the
    /// description (`cell_units`).
    #[allow(clippy::too_many_arguments)] // the description's register step: registry, target, held, token
    pub(crate) fn register<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        r: Root<'h, 's, Any>,
        target: Root<'h, 's, Any>,
        held: Root<'h, 's, Any>,
        token: Root<'h, 's, Any>,
        has_token: bool,
    ) -> Result<bool, OutOfMemory> {
        let c = m.alloc::<Cell>(s, 0)?;
        let g = c.get(m);
        let Some(registry) = reg(m, r.get_value(m)) else {
            return Ok(false);
        };
        g.set_value(m, Cell::HELD, held.get_value(m));
        g.set_value(m, Cell::REGISTRY, r.get_value(m));
        g.set_value(m, Cell::NEXT, registry.get_value(m, FinRegObj::CELLS));
        if let Value::Ref(t) = target.get_value(m) {
            g.set_weak(m, Cell::TARGET, Some(t));
        }
        if has_token && let Value::Ref(t) = token.get_value(m) {
            g.set_weak(m, Cell::TOKEN, Some(t));
        }
        g.set_raw(m, Cell::STATE, LIVE);
        registry.set_value(m, FinRegObj::CELLS, Value::Ref(g.erase()));
        Ok(true)
    }

    /// FinalizationRegistry.prototype.unregister step 5: removes every
    /// cell whose token is `token`; whether any was removed.
    pub(crate) fn unregister<'m>(
        &mut self,
        m: &'m Mutator<'h>,
        r: Value<'h, 'm>,
        token: Value<'h, 'm>,
    ) -> bool {
        let Some(registry) = reg(m, r) else {
            return false;
        };
        let Value::Ref(tok) = token else { return false };
        let mut removed = false;
        // Unlink matching cells: `prev` is the last kept cell.
        let mut prev: Option<Gc<'h, 'm, Cell>> = None;
        let mut cur = registry.get_value(m, FinRegObj::CELLS);
        let undefined = self.realm.undefined(m);
        while let Some(c) = cell(m, cur) {
            let next = c.get_value(m, Cell::NEXT);
            if c.get_weak(m, Cell::TOKEN)
                .is_some_and(|t| t.slot() == tok.slot())
            {
                c.set_raw(m, Cell::STATE, REMOVED);
                c.set_value(m, Cell::NEXT, undefined);
                match prev {
                    Some(p) => p.set_value(m, Cell::NEXT, next),
                    None => registry.set_value(m, FinRegObj::CELLS, next),
                }
                removed = true;
            } else {
                prev = Some(c);
            }
            cur = next;
        }
        removed
    }

    /// After a canonical cycle: each live cell whose target the cycle
    /// cleared is marked, and its registry, if no cleanup job is pending
    /// for it, gets one (HostEnqueueFinalizationRegistryCleanupJob).
    pub(crate) fn after_cycle(&mut self, m: &mut Mutator<'h>, host: &mut dyn Host<'h>) {
        m.scope(|m, s| {
            for h in m.take_cleared(s) {
                let Some(c) = cell(m, h.get_value(m)) else {
                    continue;
                };
                if c.raw(m, Cell::STATE) != LIVE || c.get_weak(m, Cell::TARGET).is_some() {
                    continue;
                }
                c.set_raw(m, Cell::STATE, CLEARED);
                let r = c.get_value(m, Cell::REGISTRY);
                let Some(registry) = reg(m, r) else { continue };
                if registry.raw(m, FinRegObj::PENDING) != 0 {
                    continue;
                }
                let r = s.root_value(m, r);
                // A full cage ends the agent (JS spec 6.9): the cell stays
                // marked and no job is enqueued.
                if let Ok(job) = self.push(m, s, AgentRoots::PENDING, r)
                    && let Some(registry) = reg(m, r.get_value(m))
                {
                    registry.set_raw(m, FinRegObj::PENDING, job + 1);
                    host.enqueue_cleanup_job(u64::from(job));
                }
            }
        });
    }

    /// CleanupFinalizationRegistry (ECMA-262 9.13) for cleanup job `job`.
    pub(crate) fn cleanup<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        job: u64,
    ) -> Result<u32, JobError> {
        let index = u32::try_from(job).map_err(|_| JobError::BadHandle)?;
        let r = {
            let a = self
                .array(m, AgentRoots::PENDING)
                .ok_or(JobError::BadHandle)?;
            if index >= a.raw(m, Array::LEN) {
                return Err(JobError::BadHandle);
            }
            let v = a.tail(m, index).ok_or(JobError::BadHandle)?;
            if self.free_pending.contains(&index) {
                return Err(JobError::BadHandle);
            }
            let u = self.realm.undefined(m);
            a.set_tail(m, index, u);
            v
        };
        self.free_pending.push(index);
        let r = s.root_value(m, r);
        let Some(registry) = reg(m, r.get_value(m)) else {
            return Ok(0);
        };
        registry.set_raw(m, FinRegObj::PENDING, 0);
        let mut calls = 0;
        // CS-D3 criterion 1: a cleanup job is a job, so this is the
        // second and last place allowed to mint a script token.
        #[expect(
            clippy::disallowed_methods,
            reason = "CS-D3 criterion 1: a cleanup job mints its own token"
        )]
        let mut token = ScriptToken::for_job(m);
        loop {
            // Find and unlink the first cleared cell (step 3.a-b).
            let found = {
                let Some(registry) = reg(m, r.get_value(m)) else {
                    break;
                };
                let mut prev: Option<Gc<'h, '_, Cell>> = None;
                let mut cur = registry.get_value(m, FinRegObj::CELLS);
                let mut found = None;
                while let Some(c) = cell(m, cur) {
                    let next = c.get_value(m, Cell::NEXT);
                    if c.raw(m, Cell::STATE) == CLEARED {
                        match prev {
                            Some(p) => p.set_value(m, Cell::NEXT, next),
                            None => registry.set_value(m, FinRegObj::CELLS, next),
                        }
                        c.set_raw(m, Cell::STATE, REMOVED);
                        found = Some(c.get_value(m, Cell::HELD));
                        break;
                    }
                    prev = Some(c);
                    cur = next;
                }
                found
            };
            let Some(held) = found else { break };
            let held = s.root_value(m, held);
            // Step 3.c: HostCallJobCallback(callback, undefined, « held »).
            let callback = match reg(m, r.get_value(m)) {
                Some(g) => g.get_value(m, FinRegObj::CLEANUP),
                None => break,
            };
            let callback = s.root_value(m, callback);
            let this = {
                let u = self.realm.undefined(m);
                s.root_value(m, u)
            };
            let is_host_fn =
                matches!(callback.get_value(m), Value::Ref(g) if g.cast::<HostFnObj>(m).is_some());
            if !is_host_fn {
                break;
            }
            calls += 1;
            if let Err(e) = self.call(m, &mut token, s, callback, this, &[held]) {
                return Err(match e {
                    Throw::Value(v) => JobError::Thrown(self.keep(m, s, v)?),
                    Throw::Error(k) => JobError::Error(k),
                    Throw::OutOfMemory => JobError::OutOfMemory,
                    Throw::BadBytecode => JobError::BadBytecode(0),
                });
            }
        }
        Ok(calls)
    }
}
