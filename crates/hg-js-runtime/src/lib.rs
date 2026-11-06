//! `hg-js-runtime`: Izcal's JS-M0 runtime (JS spec 6.3). Layer L2e,
//! tier R.
//!
//! - [`Agent`]: one agent's realm, handles and jobs: [`Agent::run_job`]
//!   runs a unit as a job, [`Agent::clear_kept_objects`] is
//!   ClearKeptObjects, and [`Agent::run_cleanup_job`] runs a
//!   FinalizationRegistry cleanup job the host was asked to enqueue.
//! - [`JsRuntime`]: the generated [`Runtime`](hg_js_ops::Runtime): the
//!   conversions (ToPrimitive, ToNumber, ToString, ToPropertyKey), the
//!   generic operators, strings, dictionary properties and the
//!   finalization registry's cells.
//!
//! JS-M0's realm has no intrinsic prototypes (they arrive with JS-M1's
//! realm): a primitive base's wrapper prototype is treated as `null`, a
//! string's own `length` and index properties are its only properties,
//! and only ordinary dictionary objects hold properties. The callable
//! objects are host functions, called through [`HostFunctions`].
//!
//! Canonical units (R0-B10) the runtime charges: `string_units(n)` for
//! each string a conversion or concatenation makes (a one-code-unit
//! string from indexing is internal: any tier may cache it),
//! `property_units()` for each property added, `error_units()` for each
//! error it throws. Every charge goes through the agent's [`Pacer`], so
//! cycles run at decision points only (R0-B11).

#![forbid(unsafe_code)]

mod conv;
mod finreg;
mod props;
mod rt;

use hg_heap::{Any, Mutator, Root, Scope, Value};
use hg_js_bytecode::{ConstantRef, UnitView};
use hg_js_ops::heap::{self, ConstArray};
use hg_js_ops::support::Cx;
use hg_js_ops::support::{
    Completion, Consts, ErrorKind, Frame, Host, HostValue, OutOfMemory, Pacer, Realm, RuntimeBase,
    ScriptToken, Thrown,
};
use hg_substrate::{Entropy, EntropyPurpose, RandomStream};

hg_heap::object! {
    /// The agent's long-lived state, held by one root: the handles the
    /// host holds, the registries with pending cleanup jobs, and the
    /// `typeof` result strings.
    pub AgentRoots {
        traced HANDLES: Any,
        traced PENDING: Any,
        traced NAMES: Any,
        tail none,
    }
}

hg_heap::object! {
    /// A growable array: the length in use and the slots.
    pub Array {
        raw LEN: u32,
        tail slots,
    }
}

/// Host functions: the callable objects of JS-M0 (`%CreateHostFunction`).
pub trait HostFunctions {
    /// Calls function `id` with `this` and `args`.
    ///
    /// # Errors
    /// The value the function throws.
    fn call(
        &mut self,
        id: i32,
        this: &HostValue,
        args: &[HostValue],
    ) -> Result<HostValue, HostValue>;
}

/// A value the host holds across jobs: an index into the agent's handle
/// array (a root for the agent's life).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle(pub u32);

/// Why a job did not complete normally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobError {
    /// It threw this value (now a handle).
    Thrown(Handle),
    /// It threw a new error of this kind.
    Error(ErrorKind),
    /// The interrupt check stopped it.
    Interrupted,
    /// The unit has an inadmissible instruction at this offset.
    BadBytecode(u32),
    /// The cage filled.
    OutOfMemory,
    /// A handle the host passed does not exist.
    BadHandle,
}

impl From<OutOfMemory> for JobError {
    fn from(_: OutOfMemory) -> Self {
        Self::OutOfMemory
    }
}

/// The runtime the generated interpreter calls.
pub struct JsRuntime<'h, 'o> {
    realm: Realm<'h, 'o>,
    roots: Root<'h, 'o, AgentRoots>,
    pacer: Pacer,
    functions: Box<dyn HostFunctions>,
    /// Released handle slots and finished cleanup-job slots, reused before
    /// the arrays grow.
    free_handles: Vec<u32>,
    free_pending: Vec<u32>,
}

/// The `typeof` result names, by the descriptions' `TYPEOF_*` tags.
const TYPEOF_NAMES: [&str; 8] = [
    "undefined",
    "object",
    "boolean",
    "number",
    "string",
    "symbol",
    "bigint",
    "function",
];

/// One agent (ECMA-262 9.7): its realm and runtime, and the stream
/// `Math.random` draws from.
pub struct Agent<'h, 'o> {
    rt: JsRuntime<'h, 'o>,
    realm: Realm<'h, 'o>,
    random: Box<dyn RandomStream>,
}

impl<'h, 'o> Agent<'h, 'o> {
    /// A new agent whose long-lived roots are in `s`. Its heap must have
    /// been made with a trigger of `u64::MAX`; `trigger` is the
    /// canonical units between cycles, which run at decision points.
    ///
    /// # Errors
    /// [`OutOfMemory`] if the cage is full.
    pub fn new(
        m: &mut Mutator<'h>,
        s: &Scope<'h, 'o>,
        entropy: &dyn Entropy,
        functions: Box<dyn HostFunctions>,
        trigger: u64,
    ) -> Result<Self, OutOfMemory> {
        let realm = Realm::new(m, s)?;
        let roots = m.alloc::<AgentRoots>(s, 0)?;
        let handles = m.alloc::<Array>(s, 16)?;
        let pending = m.alloc::<Array>(s, 4)?;
        let names = m.alloc::<Array>(s, 8)?;
        {
            let r = roots.get(m);
            r.set_value(m, AgentRoots::HANDLES, Value::Ref(handles.get(m).erase()));
            r.set_value(m, AgentRoots::PENDING, Value::Ref(pending.get(m).erase()));
            r.set_value(m, AgentRoots::NAMES, Value::Ref(names.get(m).erase()));
        }
        for (i, n) in (0u32..).zip(TYPEOF_NAMES) {
            let units: Vec<u16> = n.encode_utf16().collect();
            let str_ = heap::alloc_flat(m, s, &units)?;
            let v = str_.get(m).value();
            names.get(m).set_tail(m, i, v);
        }
        names.get(m).set_raw(m, Array::LEN, 8);
        let rt = JsRuntime {
            realm,
            roots,
            pacer: Pacer::new(trigger),
            functions,
            free_handles: Vec::new(),
            free_pending: Vec::new(),
        };
        Ok(Self {
            rt,
            realm,
            random: entropy.stream(EntropyPurpose::Random),
        })
    }

    /// The agent's pacing clock (fuel, cycle count).
    pub fn pacer(&mut self) -> &mut Pacer {
        &mut self.rt.pacer
    }

    /// Holds `v` for the host.
    ///
    /// # Errors
    /// [`OutOfMemory`] if the cage is full.
    pub fn handle(&mut self, m: &mut Mutator<'h>, v: &HostValue) -> Result<Handle, OutOfMemory> {
        m.scope(|m, s| {
            let r = self.rt.value_from_host(m, s, v)?;
            self.rt.keep(m, s, r)
        })
    }

    /// The host's view of handle `h`.
    pub fn host_value(&self, m: &Mutator<'h>, h: Handle) -> Option<HostValue> {
        let v = self.rt.handle_value(m, h)?;
        Some(self.rt.host_view(m, v))
    }

    /// Releases handle `h`: its value is no longer held for the host, and
    /// the slot is reused. Releasing an unknown handle does nothing.
    pub fn release(&mut self, m: &Mutator<'h>, h: Handle) {
        let Some(a) = self.rt.array(m, AgentRoots::HANDLES) else {
            return;
        };
        if h.0 >= a.raw(m, Array::LEN) || self.rt.free_handles.contains(&h.0) {
            return;
        }
        a.set_tail(m, h.0, Value::Small(0));
        self.rt.free_handles.push(h.0);
    }

    /// Whether handles `a` and `b` hold the same value (SameValue by
    /// identity for objects).
    pub fn same(&self, m: &Mutator<'h>, a: Handle, b: Handle) -> bool {
        match (self.rt.handle_value(m, a), self.rt.handle_value(m, b)) {
            (Some(x), Some(y)) => heap::same(x, y),
            _ => false,
        }
    }

    /// ClearKeptObjects (ECMA-262 9.10.2): the host calls it when a
    /// synchronous sequence of jobs completes.
    pub fn clear_kept_objects(&mut self, m: &mut Mutator<'h>) {
        m.clear_kept_objects();
    }

    /// A decision point outside a unit (a job's entry or end): a
    /// requested cycle runs, and at an entry the progress counter advances.
    fn decision_point(&mut self, m: &mut Mutator<'h>, host: &mut dyn Host<'h>, entry: bool) {
        // R0-B11: the progress counter counts LoopBacks and logical
        // function and job entries; a job's end is a decision point but
        // does not count.
        if entry {
            m.advance(1);
        }
        if self.rt.pacer.decide(m) {
            self.rt.on_canonical_cycle(m, host);
        }
    }

    /// Runs `unit` as a job with receiver `this` and arguments `args`, and
    /// returns its completion value as a handle.
    ///
    /// # Errors
    /// [`JobError`] for an abrupt completion.
    pub fn run_job(
        &mut self,
        m: &mut Mutator<'h>,
        host: &mut dyn Host<'h>,
        unit: &UnitView<'_>,
        this: Option<Handle>,
        args: &[Handle],
    ) -> Result<Handle, JobError> {
        self.decision_point(m, host, true);
        let r = m.scope(|m, s| -> Result<Handle, JobError> {
            let realm = self.realm.reroot(m, s);
            let consts = self.rt.materialize(m, s, unit)?;
            let this = match this {
                Some(h) => self.rt.handle_value(m, h).ok_or(JobError::BadHandle)?,
                None => realm.undefined(m),
            };
            let this = s.root_value(m, this);
            let mut roots = Vec::new();
            for a in args {
                let v = self.rt.handle_value(m, *a).ok_or(JobError::BadHandle)?;
                roots.push(s.root_value(m, v));
            }
            let frame = Frame::new(m, s, &realm, unit.params(), unit.registers(), this, &roots)?;
            let mut token = ScriptToken::for_job();
            let mut cx = Cx {
                t: &mut token,
                rt: &mut self.rt,
                host: &mut *host,
                random: &mut *self.random,
                realm: &realm,
                frame,
                consts,
                strict: unit.strict(),
            };
            let c = hg_js_ops::run(m, &mut cx, unit);
            match c {
                Completion::Normal => {
                    let acc = s.root_value(m, frame.acc(m));
                    Ok(self.rt.keep(m, s, acc)?)
                }
                Completion::Throw => match frame.thrown(m) {
                    Thrown::Value(v) => {
                        let v = s.root_value(m, v);
                        Err(JobError::Thrown(self.rt.keep(m, s, v)?))
                    }
                    Thrown::Error(k) => Err(JobError::Error(k)),
                    Thrown::OutOfMemory => Err(JobError::OutOfMemory),
                },
                Completion::Interrupted => Err(JobError::Interrupted),
                Completion::BadBytecode(pc) => Err(JobError::BadBytecode(pc)),
            }
        });
        self.decision_point(m, host, false);
        r
    }

    /// Runs cleanup job `job` (CleanupFinalizationRegistry, ECMA-262
    /// 9.13): each cell whose target a cycle cleared is removed and the
    /// registry's callback is called with its held value. Returns the
    /// number of callbacks made.
    ///
    /// # Errors
    /// What a callback throws, or [`JobError::BadHandle`] for an unknown
    /// job.
    pub fn run_cleanup_job(
        &mut self,
        m: &mut Mutator<'h>,
        host: &mut dyn Host<'h>,
        job: u64,
    ) -> Result<u32, JobError> {
        self.decision_point(m, host, true);
        let r = m.scope(|m, s| self.rt.cleanup(m, s, job));
        self.decision_point(m, host, false);
        r
    }
}

impl<'h, 'o> JsRuntime<'h, 'o> {
    fn array<'m>(
        &self,
        m: &'m Mutator<'h>,
        f: hg_heap::Traced<AgentRoots, Any>,
    ) -> Option<hg_heap::Gc<'h, 'm, Array>> {
        match self.roots.get(m).get_value(m, f) {
            Value::Ref(g) => g.cast::<Array>(m),
            Value::Small(_) => None,
        }
    }

    fn free_list(&mut self, f: hg_heap::Traced<AgentRoots, Any>) -> Option<&mut Vec<u32>> {
        if f.index() == AgentRoots::HANDLES.index() {
            Some(&mut self.free_handles)
        } else if f.index() == AgentRoots::PENDING.index() {
            Some(&mut self.free_pending)
        } else {
            None
        }
    }

    /// Stores `v` in the array in field `f`: in a released slot if there
    /// is one, else appended (growing the array); returns its index.
    fn push(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, '_>,
        f: hg_heap::Traced<AgentRoots, Any>,
        v: Root<'h, '_, Any>,
    ) -> Result<u32, OutOfMemory> {
        if let Some(i) = self.free_list(f).and_then(Vec::pop) {
            let a = self.array(m, f).ok_or(OutOfMemory)?;
            a.set_tail(m, i, v.get_value(m));
            return Ok(i);
        }
        let (len, cap) = match self.array(m, f) {
            Some(a) => (a.raw(m, Array::LEN), a.len(m)),
            None => return Err(OutOfMemory),
        };
        if len >= cap {
            let new_cap = cap.checked_mul(2).ok_or(OutOfMemory)?.max(4);
            let bigger = m.alloc::<Array>(s, new_cap)?;
            let old = self.array(m, f).ok_or(OutOfMemory)?;
            let b = bigger.get(m);
            for i in 0..len {
                if let Some(x) = old.tail(m, i) {
                    b.set_tail(m, i, x);
                }
            }
            b.set_raw(m, Array::LEN, len);
            self.roots.get(m).set_value(m, f, Value::Ref(b.erase()));
        }
        let a = self.array(m, f).ok_or(OutOfMemory)?;
        a.set_tail(m, len, v.get_value(m));
        a.set_raw(m, Array::LEN, len + 1);
        Ok(len)
    }

    fn keep(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, '_>,
        v: Root<'h, '_, Any>,
    ) -> Result<Handle, OutOfMemory> {
        self.push(m, s, AgentRoots::HANDLES, v).map(Handle)
    }

    fn handle_value<'m>(&self, m: &'m Mutator<'h>, h: Handle) -> Option<Value<'h, 'm>> {
        let a = self.array(m, AgentRoots::HANDLES)?;
        if h.0 >= a.raw(m, Array::LEN) || self.free_handles.contains(&h.0) {
            return None;
        }
        a.tail(m, h.0)
    }

    /// The unit's constants as values of this agent (bytecode note 4.5).
    fn materialize<'s>(
        &mut self,
        m: &mut Mutator<'h>,
        s: &Scope<'h, 's>,
        unit: &UnitView<'_>,
    ) -> Result<Consts<'h, 's>, OutOfMemory> {
        let n = unit.constant_count();
        let arr = m.alloc::<ConstArray>(s, n)?;
        for i in 0..n {
            let v = match unit.constant(i) {
                Some(ConstantRef::Number(x)) => heap::number(m, s, x)?,
                Some(ConstantRef::String(u) | ConstantRef::Key(u)) => {
                    let units: Vec<u16> = u.iter().collect();
                    heap::alloc_flat(m, s, &units)?.erase()
                }
                None => s.root_value(m, Value::Small(0)),
            };
            let x = v.get_value(m);
            arr.get(m).set_tail(m, i, x);
        }
        let g = arr.get(m).erase();
        Ok(Consts::new(s.root(m, g)))
    }
}

/// Re-rooting the realm in a job's scope, so the frame, the constants
/// and the realm share one scope lifetime.
trait Reroot<'h> {
    fn reroot<'s>(&self, m: &Mutator<'h>, s: &Scope<'h, 's>) -> Realm<'h, 's>;
}

impl<'h> Reroot<'h> for Realm<'h, '_> {
    fn reroot<'s>(&self, m: &Mutator<'h>, s: &Scope<'h, 's>) -> Realm<'h, 's> {
        Realm::from_values(
            s.root_value(m, self.undefined(m)),
            s.root_value(m, self.null(m)),
            s.root_value(m, self.true_value(m)),
            s.root_value(m, self.false_value(m)),
            s.root_value(m, self.empty(m)),
        )
    }
}
