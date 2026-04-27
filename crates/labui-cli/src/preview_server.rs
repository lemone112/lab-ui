use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use serde_json::json;
use tiny_http::{Header, Request, Response, Server};

use crate::{generate, Config};

pub struct PreviewState {
    pub css: String,
    pub json: String,
    pub config: Config,
    pub config_mtime: Option<SystemTime>,
}

#[derive(Debug, Deserialize)]
struct ConfigUpdate {
    curve: Option<CurveUpdate>,
    accent_theming: Option<AccentThemingUpdate>,
}

#[derive(Debug, Deserialize)]
struct CurveUpdate {
    lightness_ease: Option<f64>,
    hue_ease: Option<f64>,
    chroma_peak: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AccentThemingUpdate {
    dark_factor: Option<f64>,
    ic_boost: Option<f64>,
}

const CANONICAL_ACCENTS: &[(&str, &str)] = &[
    ("Brand", "#007AFF"),
    ("Red", "#FF3B30"),
    ("Orange", "#FF9500"),
    ("Yellow", "#FFCC00"),
    ("Green", "#34C759"),
    ("Teal", "#30B0C7"),
    ("Mint", "#00C7BE"),
    ("Blue", "#007AFF"),
    ("Indigo", "#5856D6"),
    ("Purple", "#AF52DE"),
    ("Pink", "#FF2D55"),
];

fn preview_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = std::path::PathBuf::from(dir).join("preview");
        if path.exists() {
            return path;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let path = dir.join("preview");
            if path.exists() {
                return path;
            }
        }
    }
    std::path::PathBuf::from("preview")
}

fn read_config_with_mtime() -> Option<(Config, SystemTime)> {
    let path = Path::new("config.yaml");
    let meta = fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    let yaml = fs::read_to_string(path).ok()?;
    let config: Config = serde_yaml::from_str(&yaml).ok()?;
    Some((config, mtime))
}

fn regenerate(config: &Config) -> Result<(String, String), String> {
    generate(config).map_err(|e| e.to_string())
}

fn resolve_bg_anchors(config: &Config) -> (String, String, Option<String>, Option<String>) {
    if let Some(neutral) = config.primitives.get("neutral") {
        (
            neutral.light.clone(),
            neutral.dark.clone(),
            neutral.ic.as_ref().map(|ic| ic.light.clone()),
            neutral.ic.as_ref().map(|ic| ic.dark.clone()),
        )
    } else {
        (
            "#FFFFFF".into(),
            "#101012".into(),
            None,
            None,
        )
    }
}

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn compute_apca_lc(accent_hex: &str, bg_hex: &str) -> Result<f64, String> {
    let y_fg = labui_core::apca::srgb_hex_to_y(accent_hex)?;
    let y_bg = labui_core::apca::srgb_hex_to_y(bg_hex)?;
    Ok(labui_core::apca::apca_contrast(y_fg, y_bg))
}

fn build_accent_matrix(config: &Config) -> serde_json::Value {
    let (bg_light, bg_dark, bg_ic_light, bg_ic_dark) = resolve_bg_anchors(config);
    let params = labui_core::accent::AccentThemingParams {
        dark_factor: config.accent_theming.dark_factor,
        ic_boost: config.accent_theming.ic_boost,
    };

    let mut all_accents: Vec<(&str, &str)> = CANONICAL_ACCENTS.to_vec();
    for (name, hex) in &config.accents {
        all_accents.push((name.as_str(), hex.as_str()));
    }

    let mut result = Vec::new();

    for (name, hex) in &all_accents {
        let cfg = labui_core::accent::AccentConfig::from_hex(hex);
        let mut variants = Vec::new();

        let themes = [
            ("light", false, false, bg_light.as_str()),
            ("dark", true, false, bg_dark.as_str()),
            ("light-ic", false, true, bg_ic_light.as_deref().unwrap_or(&bg_light)),
            ("dark-ic", true, true, bg_ic_dark.as_deref().unwrap_or(&bg_dark)),
        ];

        for (theme_name, is_dark, is_ic, bg) in &themes {
            let variant_hex = labui_core::accent::resolve_accent_base(&cfg, *is_dark, *is_ic, bg, &params)
                .unwrap_or_else(|_| hex.to_string());
            let lc = compute_apca_lc(&variant_hex, bg).unwrap_or(0.0);
            variants.push(json!({
                "theme": theme_name,
                "hex": variant_hex,
                "lc": lc,
                "bg": bg,
            }));
        }

        result.push(json!({
            "name": name,
            "canonical": hex,
            "variants": variants,
        }));
    }

    serde_json::Value::Array(result)
}

fn json_response(data: serde_json::Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = data.to_string().into_bytes();
    let mut response = Response::from_data(body);
    response.add_header(Header::from_bytes(b"Content-Type", b"application/json").unwrap());
    response
}

fn text_response(text: String, content_type: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = text.into_bytes();
    let mut response = Response::from_data(body);
    response.add_header(Header::from_bytes(b"Content-Type", content_type.as_bytes()).unwrap());
    response
}

fn serve_static(path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let preview = preview_dir();
    let file_path = if path == "/" || path == "/index.html" {
        preview.join("index.html")
    } else {
        preview.join(path.trim_start_matches('/'))
    };

    if !file_path.starts_with(&preview) {
        return Response::from_string("Not found").with_status_code(404);
    }

    match fs::read(&file_path) {
        Ok(data) => {
            let mut response = Response::from_data(data);
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let ct = match ext {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "json" => "application/json",
                _ => "application/octet-stream",
            };
            response.add_header(Header::from_bytes(b"Content-Type", ct.as_bytes()).unwrap());
            response
        }
        Err(_) => Response::from_string("Not found").with_status_code(404),
    }
}

fn apply_config_update(config: &mut Config, update: &ConfigUpdate) {
    if let Some(ref curve) = update.curve {
        for scale in config.primitives.values_mut() {
            if let Some(v) = curve.lightness_ease {
                scale.curve.lightness_ease = v;
            }
            if let Some(v) = curve.hue_ease {
                scale.curve.hue_ease = v;
            }
            if let Some(v) = curve.chroma_peak {
                scale.curve.chroma_peak = v;
            }
        }
    }
    if let Some(ref theming) = update.accent_theming {
        if let Some(v) = theming.dark_factor {
            config.accent_theming.dark_factor = v;
        }
        if let Some(v) = theming.ic_boost {
            config.accent_theming.ic_boost = v;
        }
    }
}

fn handle_request(mut request: Request, state: Arc<Mutex<PreviewState>>) {
    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or("").to_string();
    let method = request.method().clone();

    match (method.as_str(), path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => {
            request.respond(serve_static("/")).ok();
            return;
        }
        ("GET", p) if !p.starts_with("/api/") => {
            request.respond(serve_static(p)).ok();
            return;
        }
        _ => {}
    }

    match (method.as_str(), path.as_str()) {
        ("GET", "/api/tokens") => {
            let state = state.lock().unwrap();
            let json_value: serde_json::Value = serde_json::from_str(&state.json).unwrap_or_else(|_| json!({}));
            request.respond(json_response(json_value)).ok();
        }
        ("GET", "/api/css") => {
            let state = state.lock().unwrap();
            request.respond(text_response(state.css.clone(), "text/css")).ok();
        }
        ("GET", "/api/config") => {
            let state = state.lock().unwrap();
            match serde_json::to_value(&state.config) {
                Ok(v) => { request.respond(json_response(v)).ok(); }
                Err(_) => { request.respond(Response::from_string("Error").with_status_code(500)).ok(); }
            }
        }
        ("POST", "/api/config") => {
            let mut body = String::new();
            if request.as_reader().read_to_string(&mut body).is_ok() {
                if let Ok(update) = serde_json::from_str::<ConfigUpdate>(&body) {
                    let mut state = state.lock().unwrap();
                    apply_config_update(&mut state.config, &update);

                    if let Ok(yaml) = serde_yaml::to_string(&state.config) {
                        fs::write("config.yaml", yaml).ok();
                    }

                    if let Ok((css, json)) = regenerate(&state.config) {
                        state.css = css;
                        state.json = json;
                    }

                    request.respond(json_response(json!({"ok": true}))).ok();
                } else {
                    request.respond(Response::from_string("Bad request").with_status_code(400)).ok();
                }
            } else {
                request.respond(Response::from_string("Bad request").with_status_code(400)).ok();
            }
        }
        ("GET", "/api/perceptual-mix") => {
            let query = url.split('?').nth(1).unwrap_or("");
            let params: BTreeMap<_, _> = query.split('&').filter_map(|p| {
                let mut parts = p.splitn(2, '=');
                let k = parts.next()?;
                let v = parts.next()?;
                Some((k, v))
            }).collect();

            let fg = percent_decode(params.get("fg").copied().unwrap_or("#FFFFFF"));
            let bg = percent_decode(params.get("bg").copied().unwrap_or("#000000"));
            let strength = params.get("strength").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.5);

            match labui_core::tint::perceptual_mix(&fg, &bg, strength) {
                Ok(hex) => { request.respond(json_response(json!({"hex": hex}))).ok(); }
                Err(e) => { request.respond(json_response(json!({"error": e})).with_status_code(400)).ok(); }
            }
        }
        ("GET", "/api/accent-matrix") => {
            let state = state.lock().unwrap();
            let matrix = build_accent_matrix(&state.config);
            request.respond(json_response(matrix)).ok();
        }
        _ => {
            request.respond(Response::from_string("Not found").with_status_code(404)).ok();
        }
    }
}

fn init_state() -> PreviewState {
    let (config, mtime) = read_config_with_mtime().expect("config.yaml not found or invalid");
    let (css, json) = regenerate(&config).expect("failed to generate tokens");
    PreviewState {
        css,
        json,
        config,
        config_mtime: Some(mtime),
    }
}

pub fn run() {
    let state = Arc::new(Mutex::new(init_state()));
    let watcher_state = Arc::clone(&state);

    thread::spawn(move || {
        let config_path = Path::new("config.yaml");
        loop {
            thread::sleep(Duration::from_millis(300));
            if let Ok(meta) = fs::metadata(config_path) {
                if let Ok(mtime) = meta.modified() {
                    let mut locked = watcher_state.lock().unwrap();
                    let should_update = locked.config_mtime.map(|last| last != mtime).unwrap_or(true);
                    if should_update {
                        locked.config_mtime = Some(mtime);
                        if let Some((config, _)) = read_config_with_mtime() {
                            locked.config = config;
                            if let Ok((css, json)) = regenerate(&locked.config) {
                                locked.css = css;
                                locked.json = json;
                            }
                        }
                    }
                }
            }
        }
    });

    let server = Server::http("127.0.0.1:3000").expect("failed to bind to 127.0.0.1:3000");
    println!("Preview server running at http://127.0.0.1:3000");

    for request in server.incoming_requests() {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            handle_request(request, state);
        });
    }
}
