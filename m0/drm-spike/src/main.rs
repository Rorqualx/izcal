//! HGBA spike DRM-1 (Windows, WebView2) / DRM-2 (macOS, WKWebView).
//!
//! A minimal "protected media mode" host: it opens one window with the OS's licensed web
//! engine via `wry`, unmodified. The user signs in to streaming services manually; this
//! program never sees, handles or stores credentials. No user-agent spoofing (only the
//! optional, honest macOS application-name suffix), no CDM changes, no key handling.
//!
//! Usage: drm-spike --container <name> [--url <url>] [--ua-mode default|appname]
//!                  [--results-dir <dir>] [--exit-after <secs>]

use std::{
    cell::RefCell,
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde_json::{json, Map, Value};
use tao::{
    dpi::LogicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{http::Response, WebViewBuilder};

const HOME_HTML: &str = include_str!("../assets/index.html");
const PROBE_HTML: &str = include_str!("../assets/probe.html");
const OBSERVER_JS: &str = include_str!("../assets/observer.js");

/// Tiny static block list used to demonstrate that HGBA content blocking can apply in
/// protected media mode. Matches the domain and all its subdomains.
const BLOCKED_DOMAINS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "google-analytics.com",
    "scorecardresearch.com",
    "adnxs.com",
];

#[cfg(target_os = "macos")]
const OS: &str = "macos";
#[cfg(target_os = "windows")]
const OS: &str = "windows";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const OS: &str = "other";

const APP_NAME_SUFFIX: &str = "HGBA-ProtectedMedia/0.1";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UaMode {
    Default,
    AppName,
}

struct Args {
    container: String,
    url: Option<String>,
    ua_mode: UaMode,
    results_dir: PathBuf,
    exit_after: Option<u64>,
}

fn usage() -> ! {
    eprintln!(
        "usage: drm-spike --container <name> [--url <url>] [--ua-mode default|appname]\n\
         \x20                [--results-dir <dir>] [--exit-after <secs>]\n\
         \n\
         --container   container name, [A-Za-z0-9_-]{{1,64}}; selects the persistent data store\n\
         --url         start URL (default: bundled home page; 'probe' = probe-only page)\n\
         --ua-mode     default = engine's own UA; appname = append \"{APP_NAME_SUFFIX}\" (macOS only)\n\
         --exit-after  close automatically after N seconds (smoke tests)"
    );
    std::process::exit(2)
}

fn parse_args() -> Args {
    let mut it = std::env::args().skip(1);
    let mut container = None;
    let mut url = None;
    let mut ua_mode = UaMode::Default;
    let mut results_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("results");
    let mut exit_after = None;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--container" => container = it.next(),
            "--url" => url = it.next(),
            "--ua-mode" => {
                ua_mode = match it.next().as_deref() {
                    Some("default") => UaMode::Default,
                    Some("appname") => UaMode::AppName,
                    _ => usage(),
                }
            }
            "--results-dir" => results_dir = PathBuf::from(it.next().unwrap_or_else(|| usage())),
            "--exit-after" => {
                exit_after = Some(it.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| usage()))
            }
            "-h" | "--help" => usage(),
            _ => usage(),
        }
    }
    let container = container.unwrap_or_else(|| usage());
    let valid = !container.is_empty()
        && container.len() <= 64
        && container.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        eprintln!("invalid container name: {container:?}");
        usage();
    }
    Args { container, url, ua_mode, results_dir, exit_after }
}

// ---------------------------------------------------------------------------------------
// Results log (JSON lines)

#[derive(Clone)]
struct Logger {
    file: Arc<Mutex<File>>,
    base: Map<String, Value>,
}

impl Logger {
    fn new(args: &Args) -> std::io::Result<(Self, PathBuf)> {
        fs::create_dir_all(&args.results_dir)?;
        let path = args.results_dir.join(format!("{OS}-{}.jsonl", args.container));
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let mut base = Map::new();
        base.insert("os".into(), json!(OS));
        base.insert("container".into(), json!(args.container));
        base.insert(
            "ua_mode".into(),
            json!(match args.ua_mode {
                UaMode::Default => "default",
                UaMode::AppName => "appname",
            }),
        );
        Ok((Self { file: Arc::new(Mutex::new(file)), base }, path))
    }

    /// Append one record. `origin` is the sender page's origin (never the full URL).
    fn log(&self, origin: &str, rec: Value) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let mut out = Map::new();
        out.insert("ts".into(), json!(rfc3339(now)));
        out.insert("ts_ms".into(), json!(now.as_millis() as u64));
        for (k, v) in &self.base {
            out.insert(k.clone(), v.clone());
        }
        out.insert("origin".into(), json!(origin));
        match rec {
            Value::Object(m) => out.extend(m),
            other => {
                out.insert("data".into(), other);
            }
        }
        let line = Value::Object(out).to_string();
        if let Ok(mut f) = self.file.lock() {
            let _ = writeln!(f, "{line}");
            let _ = f.flush();
        }
    }
}

fn rfc3339(d: Duration) -> String {
    let secs = d.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil-from-days (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        (rem / 60) % 60,
        rem % 60,
        d.subsec_millis()
    )
}

/// scheme://host[:port] of a URL, without path/query (which may carry tokens).
fn origin_of(url: &str) -> String {
    match url.find("://") {
        Some(i) => {
            let rest = &url[i + 3..];
            let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
            format!("{}{}", &url[..i + 3], &rest[..end])
        }
        None => url.split(['?', '#']).next().unwrap_or("").to_string(),
    }
}

fn host_of(url: &str) -> Option<&str> {
    let rest = &url[url.find("://")? + 3..];
    let auth = &rest[..rest.find(['/', '?', '#']).unwrap_or(rest.len())];
    let auth = auth.rsplit('@').next().unwrap_or(auth);
    Some(auth.split(':').next().unwrap_or(auth))
}

#[cfg_attr(not(windows), allow(dead_code))]
fn blocked_domain(url: &str) -> Option<&'static str> {
    let host = host_of(url)?.to_ascii_lowercase();
    BLOCKED_DOMAINS
        .iter()
        .copied()
        .find(|d| host == *d || host.ends_with(&format!(".{d}")))
}

/// Deterministic 16-byte identifier for a container (used as the macOS
/// WKWebsiteDataStore identifier). Two FNV-1a passes with different seeds, then
/// UUID version-4/variant bits so it is a well-formed UUID.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn container_uuid(name: &str) -> [u8; 16] {
    let h = |seed: u64| {
        let mut x = seed;
        for b in "hgba-drm-spike:".bytes().chain(name.bytes()) {
            x ^= b as u64;
            x = x.wrapping_mul(0x100_0000_01b3);
        }
        x
    };
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h(0xcbf2_9ce4_8422_2325).to_be_bytes());
    out[8..].copy_from_slice(&h(0x84222325_cbf29ce4).to_be_bytes());
    out[6] = (out[6] & 0x0f) | 0x40;
    out[8] = (out[8] & 0x3f) | 0x80;
    out
}

fn uuid_string(b: &[u8; 16]) -> String {
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

// ---------------------------------------------------------------------------------------
// Auto-trigger evaluation (plan, section 1): log once per top-level origin whether the
// page's EME activity would switch an HGBA tab into protected media mode.

fn evaluate_trigger(seen: &RefCell<HashSet<String>>, logger: &Logger, origin: &str, rec: &Value) {
    let kind = rec.get("kind").and_then(Value::as_str).unwrap_or("");
    let reason = match kind {
        "encrypted" => "encrypted event on a media element",
        "mediakeys_created" => "MediaKeys created",
        "license_request" => "license request generated",
        _ => return,
    };
    let top = rec.get("topOrigin").and_then(Value::as_str).unwrap_or(origin).to_string();
    let frame = rec.get("frameOrigin").and_then(Value::as_str).unwrap_or(origin);
    let cross_origin_frame = rec.get("isTop") == Some(&Value::Bool(false)) && frame != top;
    let key = format!("{top}|{reason}|{cross_origin_frame}");
    if !seen.borrow_mut().insert(key) {
        return;
    }
    logger.log(
        origin,
        json!({
            "kind": "trigger_evaluation",
            "would_switch": !cross_origin_frame,
            "reason": if cross_origin_frame {
                format!("{reason} in cross-origin iframe {frame}: cannot switch top-level page")
            } else {
                reason.to_string()
            },
            "top_origin": top,
            "keySystem": rec.get("keySystem").cloned().unwrap_or(Value::Null),
        }),
    );
}

// ---------------------------------------------------------------------------------------

enum UserEvent {
    Title(String),
}

fn main() {
    let args = parse_args();
    let (logger, log_path) = Logger::new(&args).expect("cannot open results file");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let label = if cfg!(windows) {
        "Playing protected video using Windows' media engine (WebView2)"
    } else {
        "Playing protected video using macOS's media engine (WebKit)"
    };
    let window = WindowBuilder::new()
        .with_title(format!("[{}] {label}", args.container))
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)
        .expect("window");

    // Bundled pages are served from a custom protocol. Windows maps hgba://localhost/ to
    // https://hgba.localhost/ (with_https_scheme) so the page is a secure context, which
    // EME requires.
    let home_base = if cfg!(windows) { "https://hgba.localhost/" } else { "hgba://localhost/" };
    let start_url = match args.url.as_deref() {
        None | Some("home") => format!("{home_base}index.html"),
        Some("probe") => format!("{home_base}probe.html"),
        Some(u) => u.to_string(),
    };

    // Per-container persistent storage.
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("profiles").join(&args.container);
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut web_context = wry::WebContext::new(if cfg!(windows) { Some(data_dir.clone()) } else { None });
    let uuid = container_uuid(&args.container);

    let ua_effective = match (args.ua_mode, cfg!(target_os = "macos")) {
        (UaMode::AppName, true) => "appname",
        (UaMode::AppName, false) => {
            eprintln!("note: --ua-mode appname is only implemented on macOS (WKWebViewConfiguration.applicationNameForUserAgent); using the engine default");
            "default (appname unsupported on this OS)"
        }
        _ => "default",
    };

    logger.log(
        "host",
        json!({
            "kind": "session_start",
            "start_origin": origin_of(&start_url),
            "ua_mode_effective": ua_effective,
            "storage": if cfg!(windows) {
                json!({"type": "webview2_user_data_folder", "path": data_dir.display().to_string()})
            } else {
                json!({"type": "wkwebsitedatastore_identifier", "uuid": uuid_string(&uuid)})
            },
            "blocklist": BLOCKED_DOMAINS,
            "blocking_mechanism": if cfg!(windows) { "WebView2 WebResourceRequested (counted)" } else { "WKContentRuleList (not countable)" },
            "wry": "0.57",
        }),
    );
    eprintln!("drm-spike: container={} results={}", args.container, log_path.display());

    let seen = RefCell::new(HashSet::new());
    let ipc_logger = logger.clone();
    let title_proxy = proxy.clone();

    let builder = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_initialization_script_for_main_only(OBSERVER_JS, false)
        .with_custom_protocol("hgba".into(), |_id, req| {
            let path = req.uri().path();
            let (body, ctype) = match path {
                "/" | "/index.html" => (HOME_HTML, "text/html; charset=utf-8"),
                "/probe.html" => (PROBE_HTML, "text/html; charset=utf-8"),
                _ => ("not found", "text/plain"),
            };
            Response::builder()
                .status(if ctype == "text/plain" { 404 } else { 200 })
                .header("Content-Type", ctype)
                .body(std::borrow::Cow::Borrowed(body.as_bytes()))
                .unwrap()
        })
        .with_ipc_handler(move |req| {
            let origin = origin_of(&req.uri().to_string());
            let rec: Value = serde_json::from_str(req.body())
                .unwrap_or_else(|_| json!({"kind": "unparsed", "raw": req.body().chars().take(500).collect::<String>()}));
            evaluate_trigger(&seen, &ipc_logger, &origin, &rec);
            ipc_logger.log(&origin, rec);
        })
        .with_document_title_changed_handler(move |t| {
            let _ = title_proxy.send_event(UserEvent::Title(t));
        })
        // Popups (sign-in flows) open in an engine-default window that shares this container's
        // storage (same WebView2 environment / same WKWebViewConfiguration). Popups do not get
        // the observer script or (on Windows) the request-interception hook.
        .with_new_window_req_handler(|_url, _features| wry::NewWindowResponse::Allow)
        .with_devtools(true);

    #[cfg(windows)]
    let builder = {
        use wry::WebViewBuilderExtWindows;
        builder.with_https_scheme(true)
    };

    #[cfg(target_os = "macos")]
    let builder = {
        use wry::{WebViewBuilderExtDarwin, WebViewBuilderExtMacos};
        let config = macos::configuration(&uuid, args.ua_mode == UaMode::AppName);
        // The data store identifier is also set on our configuration (wry uses the
        // configuration's store when one is supplied); keep the builder option for clarity.
        builder.with_data_store_identifier(uuid).with_webview_configuration(config)
    };

    // On macOS the URL is loaded after the content rule list compiles (async).
    #[cfg(not(target_os = "macos"))]
    let builder = builder.with_url(&start_url);

    let webview = builder.build(&window).expect("webview");

    #[cfg(windows)]
    if let Err(e) = windows_block::install(&webview, logger.clone()) {
        logger.log("host", json!({"kind": "blocking_unavailable", "error": format!("{e:?}")}));
    }

    #[cfg(target_os = "macos")]
    let webview = {
        let wv = std::rc::Rc::new(webview);
        macos::install_rules_then_load(wv.clone(), start_url.clone(), logger.clone());
        wv
    };

    let started = Instant::now();
    let exit_after = args.exit_after.map(Duration::from_secs);
    let container = args.container.clone();
    let exit_logger = logger.clone();
    let mut webview = Some(webview);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = match exit_after {
            Some(d) => ControlFlow::WaitUntil(started + d),
            None => ControlFlow::Wait,
        };
        let mut quit = false;
        match event {
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => quit = true,
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => quit = true,
            Event::UserEvent(UserEvent::Title(t)) => {
                window.set_title(&format!("[{container}] {t} - {label}"));
            }
            _ => {}
        }
        if quit {
            exit_logger.log("host", json!({"kind": "session_end", "seconds": started.elapsed().as_secs()}));
            webview.take();
            *control_flow = ControlFlow::Exit;
        }
    });
}

// ---------------------------------------------------------------------------------------
// Windows: content blocking through WebView2's WebResourceRequested event.

#[cfg(windows)]
mod windows_block {
    use super::{blocked_domain, json, origin_of, Logger};
    use std::collections::HashMap;
    use webview2_com::{
        take_pwstr, Microsoft::Web::WebView2::Win32::*, WebResourceRequestedEventHandler,
    };
    use windows::core::{w, PWSTR};
    use wry::WebViewExtWindows;

    pub fn install(webview: &wry::WebView, logger: Logger) -> windows::core::Result<()> {
        let core = webview.webview();
        let env = webview.environment();
        let mut counts: HashMap<&'static str, u64> = HashMap::new();
        let mut total: u64 = 0;
        unsafe {
            core.AddWebResourceRequestedFilter(w!("*"), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)?;
            let mut token = Default::default();
            core.add_WebResourceRequested(
                &WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let req = args.Request()?;
                    let mut uri = PWSTR::null();
                    req.Uri(&mut uri)?;
                    let uri = take_pwstr(uri);
                    if let Some(domain) = blocked_domain(&uri) {
                        let resp = env.CreateWebResourceResponse(
                            None,
                            403,
                            w!("Blocked by HGBA"),
                            w!(""),
                        )?;
                        args.SetResponse(&resp)?;
                        total += 1;
                        let c = counts.entry(domain).or_insert(0);
                        *c += 1;
                        logger.log(
                            "host",
                            json!({"kind": "blocked", "domain": domain, "request_origin": origin_of(&uri),
                                   "count_for_domain": *c, "total_blocked": total}),
                        );
                    }
                    Ok(())
                })),
                &mut token,
            )?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------
// macOS: per-container WKWebsiteDataStore, honest UA suffix, WKContentRuleList blocking.

#[cfg(target_os = "macos")]
mod macos {
    use super::{json, Logger, APP_NAME_SUFFIX, BLOCKED_DOMAINS};
    use block2::RcBlock;
    use objc2::{rc::Retained, MainThreadMarker};
    use objc2_foundation::{NSError, NSString, NSUUID};
    use objc2_web_kit::{
        WKContentRuleList, WKContentRuleListStore, WKWebViewConfiguration, WKWebsiteDataStore,
    };
    use std::rc::Rc;
    use wry::WebViewExtMacOS;

    pub fn configuration(uuid: &[u8; 16], app_name: bool) -> Retained<WKWebViewConfiguration> {
        let mtm = MainThreadMarker::new().expect("main thread");
        unsafe {
            let config = WKWebViewConfiguration::new(mtm);
            // WKWebsiteDataStore(forIdentifier:) is macOS 14+; this Mac and the plan assume 14+.
            let store = WKWebsiteDataStore::dataStoreForIdentifier(&NSUUID::from_bytes(*uuid), mtm);
            config.setWebsiteDataStore(&store);
            if app_name {
                // Appends to WebKit's own UA; does not impersonate Safari or Chrome.
                config.setApplicationNameForUserAgent(Some(&NSString::from_str(APP_NAME_SUFFIX)));
            }
            // Lets players use element fullscreen (macOS 12.3+).
            config.preferences().setElementFullscreenEnabled(true);
            config
        }
    }

    fn rules_json() -> String {
        let rules: Vec<_> = BLOCKED_DOMAINS
            .iter()
            .map(|d| {
                json!({
                    "trigger": {"url-filter": format!("^[^:]+://+([^:/]+\\.)?{}[:/]", d.replace('.', "\\."))},
                    "action": {"type": "block"}
                })
            })
            .collect();
        serde_json::Value::Array(rules).to_string()
    }

    /// Compile the rule list, attach it to the webview's content controller, then load
    /// the start URL so the very first page load is already covered.
    pub fn install_rules_then_load(webview: Rc<wry::WebView>, start_url: String, logger: Logger) {
        let mtm = MainThreadMarker::new().expect("main thread");
        let store = unsafe { WKContentRuleListStore::defaultStore(mtm) };
        let Some(store) = store else {
            logger.log("host", json!({"kind": "blocking_unavailable", "error": "no WKContentRuleListStore"}));
            let _ = webview.load_url(&start_url);
            return;
        };
        let wv = webview.clone();
        let block = RcBlock::new(move |list: *mut WKContentRuleList, err: *mut NSError| {
            if let Some(list) = unsafe { list.as_ref() } {
                unsafe { wv.manager().addContentRuleList(list) };
                logger.log(
                    "host",
                    json!({"kind": "blocking_installed", "mechanism": "WKContentRuleList",
                           "rules": BLOCKED_DOMAINS.len(),
                           "note": "WebKit gives no per-request callback for content rule lists, so blocked requests cannot be counted"}),
                );
            } else {
                let msg = unsafe { err.as_ref() }.map(|e| e.localizedDescription().to_string());
                logger.log("host", json!({"kind": "blocking_unavailable", "error": msg}));
            }
            let _ = wv.load_url(&start_url);
        });
        unsafe {
            store.compileContentRuleListForIdentifier_encodedContentRuleList_completionHandler(
                Some(&NSString::from_str("hgba-drm-spike-blocklist")),
                Some(&NSString::from_str(&rules_json())),
                Some(&block),
            );
        }
    }
}
