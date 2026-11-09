//! What the OS says about this process's address space, read from the
//! OS's own view rather than from the substrate's ledger, so the two can
//! be compared: the substrate counts one reservation per process, and the
//! kernel counts the mappings that reservation has actually become.

/// The OS's view of one moment.
#[derive(Debug, Clone, Default)]
pub struct OsView {
    /// Mappings the process has in all (Linux: lines of
    /// `/proc/self/maps`; macOS: regions `vmmap` lists).
    pub mappings: u64,
    /// Mappings that start inside the range asked about.
    pub mappings_in_range: u64,
    /// Virtual size in bytes, as the OS reports it (`VmSize`, or
    /// `vmmap`'s total virtual size); 0 when unknown.
    pub virtual_bytes: u64,
    /// Resident size in bytes (`VmRSS`, or `ps rss`); 0 when unknown.
    pub resident_bytes: u64,
    /// Peak virtual size (`VmPeak`); 0 when unknown.
    pub peak_virtual_bytes: u64,
    /// The kernel's cap on mappings, if it has one
    /// (`vm.max_map_count`); 0 when unknown.
    pub max_map_count: u64,
}

/// The OS's view now. `range` is `(base, len)`: mappings starting inside
/// it are counted separately, which is how the cage's own fragmentation
/// is measured.
pub fn read(range: Option<(usize, usize)>) -> OsView {
    #[cfg(target_os = "linux")]
    {
        linux(range)
    }
    #[cfg(target_os = "macos")]
    {
        macos(range)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = range;
        OsView::default()
    }
}

/// Whether this build can read the OS's own view at all.
pub fn available() -> bool {
    cfg!(any(target_os = "linux", target_os = "macos"))
}

#[cfg(target_os = "linux")]
fn linux(range: Option<(usize, usize)>) -> OsView {
    let mut v = OsView::default();
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            v.mappings += 1;
            let Some((start, _)) = line.split_once('-') else {
                continue;
            };
            let Ok(addr) = usize::from_str_radix(start, 16) else {
                continue;
            };
            if let Some((base, len)) = range
                && addr >= base
                && addr < base.saturating_add(len)
            {
                v.mappings_in_range += 1;
            }
        }
    }
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            // After the prefix is stripped the rest is " 1234 kB", so the
            // number is the first field, not the second.
            let kb = |l: &str| -> u64 {
                l.split_whitespace()
                    .next()
                    .and_then(|n| n.parse::<u64>().ok())
                    .unwrap_or(0)
                    .saturating_mul(1024)
            };
            if let Some(rest) = line.strip_prefix("VmSize:") {
                v.virtual_bytes = kb(rest);
            } else if let Some(rest) = line.strip_prefix("VmRSS:") {
                v.resident_bytes = kb(rest);
            } else if let Some(rest) = line.strip_prefix("VmPeak:") {
                v.peak_virtual_bytes = kb(rest);
            }
        }
    }
    v.max_map_count = std::fs::read_to_string("/proc/sys/vm/max_map_count")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    v
}

/// macOS has no `/proc`, so this asks `vmmap`, which lists one line per
/// region with its address range. Its "VIRTUAL" and "RESIDENT" summary
/// figures are read from `ps` instead, which needs no entitlement.
#[cfg(target_os = "macos")]
fn macos(range: Option<(usize, usize)>) -> OsView {
    let mut v = OsView::default();
    let pid = std::process::id().to_string();
    if let Ok(out) = std::process::Command::new("/usr/bin/vmmap")
        .arg("-w")
        .arg(&pid)
        .output()
        && out.status.success()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            // Region lines carry "  <hex>-<hex>  " in their address column.
            let Some(addr) = region_start(line) else {
                continue;
            };
            v.mappings += 1;
            if let Some((base, len)) = range
                && addr >= base
                && addr < base.saturating_add(len)
            {
                v.mappings_in_range += 1;
            }
        }
    }
    if let Ok(out) = std::process::Command::new("/bin/ps")
        .args(["-o", "rss=,vsz=", "-p", &pid])
        .output()
        && out.status.success()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut it = text.split_whitespace();
        v.resident_bytes = it
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_mul(1024);
        v.virtual_bytes = it
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_mul(1024);
    }
    v
}

/// The start address of a `vmmap` region line, if it is one: a field of
/// the form `<hex>-<hex>` with at least 9 hex digits on each side.
#[cfg(target_os = "macos")]
fn region_start(line: &str) -> Option<usize> {
    for field in line.split_whitespace() {
        let Some((a, b)) = field.split_once('-') else {
            continue;
        };
        if a.len() >= 9
            && b.len() >= 9
            && a.bytes().all(|c| c.is_ascii_hexdigit())
            && b.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return usize::from_str_radix(a, 16).ok();
        }
    }
    None
}
