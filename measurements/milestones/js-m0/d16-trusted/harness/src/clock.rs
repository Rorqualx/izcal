//! The clock, measured per cell rather than taken from a nominal figure.
//! hueyj runs the `powersave` governor and its clock moves under load, so
//! a single calibration would turn a frequency difference into an apparent
//! variant difference. No system setting is touched: the governor is read,
//! never written.

use std::hint::black_box;
use std::time::Instant;

/// A calibration: nanoseconds per dependent 64-bit add, the frequency
/// that implies at one cycle per add, and the kernel's own figure.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    /// Nanoseconds per add on the dependent chain.
    pub ns_per_add: f64,
    /// GHz: the kernel's own per-cell figure where there is one, else
    /// what the add chain implies at one cycle per add.
    pub ghz: f64,
    /// Which of those two the GHz above came from.
    pub ghz_source: &'static str,
    /// The lowest `cpu MHz` seen in `/proc/cpuinfo`, if it exists.
    pub mhz_min: f64,
    /// The highest.
    pub mhz_max: f64,
    /// The chain's result was the arithmetic it must have been.
    pub ok: bool,
}

impl Clock {
    /// `ns` as cycles at this clock.
    pub fn cycles(&self, ns: f64) -> f64 {
        ns * self.ghz
    }
}

/// A dependent add chain of `n` steps, checked against the value the
/// arithmetic must have produced. The addend goes through `black_box` on
/// every step, so the chain cannot be strength-reduced to one multiply —
/// the failure that produced a fast wrong answer in the JS-D1 spike.
pub fn calibrate(n: u64) -> Clock {
    const K: u64 = 0x9E37_79B9_7F4A_7C15;
    let (mhz_min_0, mhz_max_0) = cpu_mhz();
    let mut acc = 0u64;
    let t0 = Instant::now();
    for _ in 0..n {
        acc = acc.wrapping_add(black_box(K));
    }
    let ns = t0.elapsed().as_nanos() as f64;
    let got = black_box(acc);
    let (mhz_min_1, mhz_max_1) = cpu_mhz();
    let ns_per_add = ns / n as f64;
    let (mhz_min, mhz_max) = (mhz_min_0.min(mhz_min_1), mhz_max_0.max(mhz_max_1));
    // The kernel's figure is measured, and it is measured for this cell:
    // hueyj's governor moves the clock under load, so each cell reads it
    // again. The *highest* core is the clock to use, not the mean: on a
    // rig booked exclusive this thread is the busy core and the idle ones
    // sit at the governor's floor (800 MHz on hueyj), so a mean would
    // report less than half the real frequency — which is how the gate
    // first caught this. Both ends are recorded in every row's source
    // data. Where there is no such file the add chain stands in, and the
    // row says so.
    let (ghz, ghz_source) = if mhz_max > 0.0 {
        (mhz_max / 1000.0, "cpuinfo-max")
    } else if ns_per_add > 0.0 {
        (1.0 / ns_per_add, "add-chain")
    } else {
        (0.0, "none")
    };
    Clock {
        ns_per_add,
        ghz,
        ghz_source,
        mhz_min,
        mhz_max,
        ok: got == K.wrapping_mul(n),
    }
}

/// The lowest and highest `cpu MHz` in `/proc/cpuinfo`, or zeros where
/// there is no such file (macOS).
pub fn cpu_mhz() -> (f64, f64) {
    let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") else {
        return (0.0, 0.0);
    };
    let mut lo = f64::MAX;
    let mut hi: f64 = 0.0;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("cpu MHz") {
            if let Some(v) = v.split(':').nth(1) {
                if let Ok(x) = v.trim().parse::<f64>() {
                    lo = lo.min(x);
                    hi = hi.max(x);
                }
            }
        }
    }
    if hi == 0.0 { (0.0, 0.0) } else { (lo, hi) }
}

/// The CPU governor, for the record (read only).
pub fn governor() -> String {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
