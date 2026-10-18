//! Turn results/*.jsonl into the DRM-1/DRM-2 results tables (Markdown on stdout).
//!
//! Usage: cargo run --release --bin summarize [-- <results-dir>]
//!
//! Optional manual input: <results-dir>/manual.csv with lines
//!   service,os,ua_mode,plays,max_resolution,isolation_held,notes
//! Non-empty manual fields override/augment the automatic columns (the stats-overlay
//! resolution and the two-container isolation check are observed by the user).

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use serde_json::Value;

const SERVICES: &[(&str, &[&str])] = &[
    ("Netflix", &["netflix.com", "nflxvideo.net", "nflxext.com"]),
    ("Disney+", &["disneyplus.com", "disney-plus.net", "bamgrid.com", "dssott.com"]),
    ("Prime Video", &["primevideo.com", "amazon.com", "amazon.co.uk", "amazon.de", "media-amazon.com"]),
    ("Spotify", &["spotify.com", "scdn.co"]),
    ("Max", &["max.com", "hbomax.com"]),
];

fn service_of(origin: &str) -> Option<&'static str> {
    let host = origin.split("://").nth(1).unwrap_or(origin);
    let host = host.split(':').next().unwrap_or(host);
    SERVICES.iter().find_map(|(name, doms)| {
        doms.iter().any(|d| host == *d || host.ends_with(&format!(".{d}"))).then_some(*name)
    })
}

#[derive(Default)]
struct Row {
    containers: BTreeSet<String>,
    key_systems: BTreeSet<String>,
    max_px: u64,
    max_res: Option<(u64, u64)>,
    encrypted_video: bool,
    license_requests: u64,
    usable_keys: bool,
    restricted: BTreeSet<String>,
    mksa_denied: BTreeSet<String>,
    switch_triggered: bool,
    uas: BTreeSet<String>,
}

#[derive(Default)]
struct Probe {
    results: BTreeMap<(String, String), bool>,
    secure: Option<bool>,
    block_self_test: Option<String>,
    triggered: bool,
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("results"));
    let mut rows: BTreeMap<(String, String, String), Row> = BTreeMap::new();
    let mut probes: BTreeMap<(String, String), Probe> = BTreeMap::new();
    let mut blocked: BTreeMap<String, u64> = BTreeMap::new();
    let mut blocked_by_container: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut blocking_mech: BTreeMap<String, String> = BTreeMap::new();

    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    files.sort();

    for f in &files {
        let text = fs::read_to_string(f).unwrap_or_default();
        for line in text.lines() {
            let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
            let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            let (os, container, ua_mode, kind) = (s("os"), s("container"), s("ua_mode"), s("kind"));
            let top = v.get("topOrigin").and_then(Value::as_str).map(str::to_string)
                .or_else(|| v.get("top_origin").and_then(Value::as_str).map(str::to_string))
                .unwrap_or_else(|| s("origin"));

            match kind.as_str() {
                "blocked" => {
                    *blocked.entry(os.clone()).or_default() += 1;
                    *blocked_by_container.entry((os.clone(), container.clone())).or_default() += 1;
                }
                "session_start" => {
                    if let Some(m) = v.get("blocking_mechanism").and_then(Value::as_str) {
                        blocking_mech.insert(os.clone(), m.to_string());
                    }
                }
                _ => {}
            }

            let is_probe = top.ends_with("localhost") && (top.starts_with("hgba://") || top.contains("hgba.localhost"));
            if is_probe {
                let p = probes.entry((os.clone(), container.clone())).or_default();
                if kind == "probe_summary" {
                    p.secure = v.get("secureContext").and_then(Value::as_bool);
                    p.block_self_test = v.get("blockSelfTest").and_then(Value::as_str).map(str::to_string);
                    for r in v.get("results").and_then(Value::as_array).into_iter().flatten() {
                        let ks = r.get("keySystem").and_then(Value::as_str).unwrap_or("").to_string();
                        let var = r.get("variant").and_then(Value::as_str).unwrap_or("").to_string();
                        let ok = r.get("supported").and_then(Value::as_bool).unwrap_or(false);
                        p.results.insert((ks, var), ok);
                    }
                }
                if kind == "trigger_evaluation" && v.get("would_switch") == Some(&Value::Bool(true)) {
                    p.triggered = true;
                }
                continue;
            }

            let Some(service) = service_of(&top) else { continue };
            let row = rows.entry((service.to_string(), os.clone(), ua_mode.clone())).or_default();
            row.containers.insert(container.clone());
            let ks = v.get("keySystem").and_then(Value::as_str).filter(|k| !k.is_empty() && *k != "unknown");
            match kind.as_str() {
                "page" => { row.uas.insert(s("ua")); }
                "mksa_result" if v.get("granted") == Some(&Value::Bool(false)) => {
                    if let Some(k) = ks { row.mksa_denied.insert(k.to_string()); }
                }
                "mediakeys_created" | "mediakeys_attached" => {
                    if let Some(k) = ks { row.key_systems.insert(k.to_string()); }
                }
                "license_request" => {
                    row.license_requests += 1;
                    if let Some(k) = ks { row.key_systems.insert(k.to_string()); }
                }
                "key_statuses" => {
                    for st in v.get("statuses").and_then(Value::as_array).into_iter().flatten() {
                        match st.as_str() {
                            Some("usable") => row.usable_keys = true,
                            Some(other) => { row.restricted.insert(other.to_string()); }
                            None => {}
                        }
                    }
                }
                "video" => {
                    if v.get("encrypted") == Some(&Value::Bool(true)) {
                        row.encrypted_video = true;
                        let w = v.get("width").and_then(Value::as_u64).unwrap_or(0);
                        let h = v.get("height").and_then(Value::as_u64).unwrap_or(0);
                        if w * h > row.max_px { row.max_px = w * h; row.max_res = Some((w, h)); }
                        if let Some(k) = ks { row.key_systems.insert(k.to_string()); }
                    }
                }
                "trigger_evaluation" if v.get("would_switch") == Some(&Value::Bool(true)) => row.switch_triggered = true,
                _ => {}
            }
        }
    }

    // Manual overrides.
    let mut manual: BTreeMap<(String, String, String), Vec<String>> = BTreeMap::new();
    if let Ok(text) = fs::read_to_string(dir.join("manual.csv")) {
        for line in text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')) {
            let mut cols: Vec<String> = line.splitn(7, ',').map(|c| c.trim().to_string()).collect();
            cols.resize(7, String::new());
            manual.insert((cols[0].clone(), cols[1].clone(), cols[2].clone()), cols[3..].to_vec());
            rows.entry((cols[0].clone(), cols[1].clone(), cols[2].clone())).or_default();
        }
    }

    println!("## Service results (from {} file(s) in {})\n", files.len(), dir.display());
    println!("| Service | OS | UA mode | Plays? | Max resolution | Key system | Isolation held? | Notes |");
    println!("|---|---|---|---|---|---|---|---|");
    for (key, r) in &rows {
        let (service, os, ua) = key;
        let m = manual.get(key);
        let mf = |i: usize| m.map(|m| m[i].clone()).filter(|s| !s.is_empty());
        let plays = mf(0).unwrap_or_else(|| {
            if r.encrypted_video { "yes (encrypted video decoded)".into() }
            else if r.usable_keys { "license ok (keys usable)".into() }
            else if r.license_requests > 0 { "no? (license requested, keys never usable)".into() }
            else if !r.mksa_denied.is_empty() { "no (key system denied)".into() }
            else { "no EME activity seen".into() }
        });
        let res = mf(1).unwrap_or_else(|| r.max_res.map(|(w, h)| format!("{w}x{h} (decoded)")).unwrap_or("-".into()));
        let ks = if r.key_systems.is_empty() { "-".into() } else { r.key_systems.iter().cloned().collect::<Vec<_>>().join(", ") };
        let iso = mf(2).unwrap_or_else(|| format!("(check manually; containers: {})", r.containers.iter().cloned().collect::<Vec<_>>().join(", ")));
        let mut notes = Vec::new();
        if let Some(n) = mf(3) { notes.push(n); }
        if !r.restricted.is_empty() { notes.push(format!("key statuses: {}", r.restricted.iter().cloned().collect::<Vec<_>>().join("/"))); }
        if !r.mksa_denied.is_empty() { notes.push(format!("denied: {}", r.mksa_denied.iter().cloned().collect::<Vec<_>>().join("/"))); }
        notes.push(format!("auto-trigger: {}", if r.switch_triggered { "would switch" } else { "no" }));
        println!("| {service} | {os} | {ua} | {plays} | {res} | {ks} | {iso} | {} |", notes.join("; "));
    }
    if rows.is_empty() {
        println!("| (no service sessions recorded yet) | | | | | | | |");
    }

    println!("\n## Probe-only page: key systems reported as supported\n");
    let variants: BTreeSet<_> = probes.values().flat_map(|p| p.results.keys().cloned()).collect();
    if probes.is_empty() {
        println!("(no probe runs recorded)");
    } else {
        let cols: Vec<_> = probes.keys().collect();
        print!("| Key system | Variant |");
        for (os, c) in &cols { print!(" {os} ({c}) |"); }
        println!();
        print!("|---|---|");
        for _ in &cols { print!("---|"); }
        println!();
        for (ks, var) in &variants {
            print!("| {ks} | {var} |");
            for k in &cols {
                let v = probes[*k].results.get(&(ks.clone(), var.clone()));
                print!(" {} |", match v { Some(true) => "yes", Some(false) => "no", None => "-" });
            }
            println!();
        }
        print!("| secure context | |");
        for k in &cols { print!(" {} |", probes[*k].secure.map_or("-".into(), |b| b.to_string())); }
        println!();
        print!("| auto-trigger fired on probe page (must be no) | |");
        for k in &cols { print!(" {} |", if probes[*k].triggered { "YES (rule violated)" } else { "no" }); }
        println!();
        print!("| blocking self-test (doubleclick.net) | |");
        for k in &cols {
            let page = probes[*k].block_self_test.clone().unwrap_or("-".into());
            match blocked_by_container.get(*k) {
                Some(n) => print!(" blocked (host counted {n}; page saw {page}) |"),
                None => print!(" {page} |"),
            }
        }
        println!();
    }

    println!("\n## Content blocking\n");
    let oses: BTreeSet<_> = blocking_mech.keys().chain(blocked.keys()).cloned().collect();
    for os in oses {
        println!(
            "- {os}: {} - blocked requests counted: {}",
            blocking_mech.get(&os).cloned().unwrap_or_default(),
            blocked.get(&os).copied().unwrap_or(0)
        );
    }
}
