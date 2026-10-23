//! The run's trace: one line per scheduling decision, hashed as it is
//! written (CS-T5: same seed, same trace, byte for byte). Lines never
//! hold host addresses or host time, only simulated facts.

use core::fmt::Write as _;
use core::hash::Hasher as _;

use hg_types::ProcessHasher;

pub(crate) struct Trace {
    /// SipHash-2-4 with a fixed key, so the hash does not depend on the
    /// process hash key.
    hasher: ProcessHasher,
    line: String,
    text: Option<String>,
}

impl Trace {
    pub(crate) fn new(keep_text: bool) -> Self {
        Self {
            hasher: ProcessHasher::with_keys(0x4947_4441_5354_0001, 0x4947_4441_5354_0002),
            line: String::new(),
            text: keep_text.then(String::new),
        }
    }

    pub(crate) fn record(&mut self, args: core::fmt::Arguments<'_>) {
        self.line.clear();
        // Writing to a String cannot fail.
        let _ = self.line.write_fmt(args);
        self.line.push('\n');
        self.hasher.write(self.line.as_bytes());
        if let Some(text) = &mut self.text {
            text.push_str(&self.line);
        }
    }

    pub(crate) fn hash(&self) -> u64 {
        self.hasher.finish()
    }

    pub(crate) fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
}
