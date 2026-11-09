//! The clock, measured per cell rather than taken from a nominal figure
//! (hueyj runs the `powersave` governor and drifts under load; the JS-D1
//! spike found that one calibration for a whole sweep turns a frequency
//! difference into an apparent effect).
//!
//! Two readings, both kept, neither assumed:
//!
//! * `mhz_os_max` / `mhz_os_mean` — what the OS says the cores were
//!   running at while the cell ran, sampled before and after it. On Linux
//!   this is `/proc/cpuinfo`'s `cpu MHz`, which the kernel reads from the
//!   hardware. The max is the figure used for cycles: on a machine with
//!   more cores than the cell uses, the mean is diluted by idle cores.
//!   Both are 0 where the OS does not say (macOS has no such file).
//! * `ref_ns_per_add` — a speed reference with no cycle claim attached:
//!   a dependent chain of 64-bit adds whose addend goes through
//!   `black_box` each iteration, so the chain cannot be folded into a
//!   multiply, with the result checked against the value the arithmetic
//!   must have produced. **Its cost per add is not one cycle** — the
//!   `black_box` barrier costs several, measured at about 4.5 on an
//!   Apple M-series core and reported per cell here — so it is used only
//!   as a *relative* speed reference between cells of one run, and as
//!   the cross-check `ref_cycles_per_add`, which must hold steady across
//!   a sweep if the OS's frequency figure is to be believed.
//!
//! A cell whose `checked` is false, or whose reference drifts, is a cell
//! whose timing is not kept.

use std::hint::black_box;
use std::time::Instant;

/// One clock reading.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    /// The fastest core's MHz, as the OS reported it around the cell; 0
    /// if the OS does not say.
    pub mhz_os_max: f64,
    /// The mean MHz over the machine's cores; 0 if the OS does not say.
    pub mhz_os_mean: f64,
    /// Nanoseconds per add in the reference chain.
    pub ref_ns_per_add: f64,
    /// The reference chain's cycles per add at `mhz_os_max`; 0 if the OS
    /// does not say. Steady across a sweep means the frequency figure and
    /// the reference agree.
    pub ref_cycles_per_add: f64,
    /// Whether the reference chain's arithmetic came out as it must have.
    pub checked: bool,
}

impl Clock {
    /// Whether this reading can carry a cycle figure: the reference
    /// checked out and the OS gave a frequency.
    pub fn has_cycles(&self) -> bool {
        self.checked && self.mhz_os_max > 0.0
    }

    /// GHz to turn nanoseconds into cycles with, or 0.
    pub fn ghz(&self) -> f64 {
        self.mhz_os_max / 1000.0
    }

    /// Whether the cell's clock reading is sound enough to keep a
    /// measurement from it: the arithmetic checked out. A machine whose
    /// OS reports no frequency still gives valid times, only no cycles.
    pub fn sound(&self) -> bool {
        self.checked
    }
}

/// Measures the clock with `adds` dependent additions.
pub fn measure(adds: u64) -> Clock {
    let before = os_mhz();
    let mut x: u64 = black_box(0);
    let t = Instant::now();
    for _ in 0..adds {
        x = x.wrapping_add(black_box(1));
    }
    let secs = t.elapsed().as_secs_f64();
    let x = black_box(x);
    let after = os_mhz();
    let ref_ns_per_add = if adds == 0 {
        0.0
    } else {
        secs * 1e9 / adds as f64
    };
    let pick = |f: fn(&(f64, f64)) -> f64| -> f64 {
        match (&before, &after) {
            (Some(a), Some(b)) => (f(a) + f(b)) / 2.0,
            (Some(a), None) | (None, Some(a)) => f(a),
            (None, None) => 0.0,
        }
    };
    let mhz_os_max = pick(|p| p.0);
    let mhz_os_mean = pick(|p| p.1);
    Clock {
        mhz_os_max,
        mhz_os_mean,
        ref_ns_per_add,
        ref_cycles_per_add: ref_ns_per_add * mhz_os_max / 1000.0,
        checked: x == adds,
    }
}

/// `(max, mean)` of this machine's per-core MHz, if the OS says.
fn os_mhz() -> Option<(f64, f64)> {
    let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut n = 0u32;
    let mut sum = 0.0;
    let mut max = 0.0f64;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("cpu MHz")
            && let Some((_, v)) = rest.split_once(':')
            && let Ok(mhz) = v.trim().parse::<f64>()
        {
            sum += mhz;
            max = max.max(mhz);
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some((max, sum / f64::from(n)))
    }
}
