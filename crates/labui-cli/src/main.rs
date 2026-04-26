#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    primitives: HashMap<String, ScaleConfig>,
    #[serde(default)]
    accents: HashMap<String, String>,
    #[serde(default)]
    output: OutputConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScaleConfig {
    light: String,
    base: String,
    dark: String,
    #[serde(default)]
    ic: Option<IcAnchors>,
    #[serde(default)]
    curve: CurveConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct IcAnchors {
    light: String,
    base: String,
    dark: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CurveConfig {
    #[serde(default = "default_lightness_ease")]
    lightness_ease: f64,
    #[serde(default = "default_hue_ease")]
    hue_ease: f64,
    #[serde(default = "default_chroma_peak")]
    chroma_peak: f64,
}

impl Default for CurveConfig {
    fn default() -> Self {
        Self {
            lightness_ease: default_lightness_ease(),
            hue_ease: default_hue_ease(),
            chroma_peak: default_chroma_peak(),
        }
    }
}

fn default_lightness_ease() -> f64 { 1.7 }
fn default_hue_ease() -> f64 { 0.6 }
fn default_chroma_peak() -> f64 { 0.35 }

#[derive(Debug, Deserialize, Serialize, Default)]
struct OutputConfig {
    #[serde(default = "default_scss")]
    scss: String,
    #[serde(default = "default_json")]
    json: String,
}

fn default_scss() -> String { "dist/tokens.scss".into() }
fn default_json() -> String { "dist/tokens.json".into() }

fn main() {
    let config_path = Path::new("config.yaml");
    if !config_path.exists() {
        eprintln!("error: config.yaml not found");
        std::process::exit(1);
    }

    let yaml = fs::read_to_string(config_path).expect("failed to read config.yaml");
    let config: Config = serde_yaml::from_str(&yaml).expect("failed to parse config.yaml");

    // Resolve background anchors for accent generation.
    let (bg_light, bg_dark, bg_ic_light, bg_ic_dark) =
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
        };

    let mut scss = String::new();
    let mut json_map: HashMap<String, String> = HashMap::new();

    // ------------------------------------------------------------------
    // 1. Generate neutral primitive blocks.
    // ------------------------------------------------------------------
    for (name, scale_cfg) in &config.primitives {
        let params = labui_core::neutral::CurveParams {
            lightness_ease: scale_cfg.curve.lightness_ease,
            hue_ease: scale_cfg.curve.hue_ease,
            chroma_peak: scale_cfg.curve.chroma_peak,
        };

        let light = match labui_core::neutral::create_neutral_light_scale(
            &scale_cfg.light, &scale_cfg.base, &scale_cfg.dark, &params,
        ) {
            Ok(s) => s,
            Err(e) => { eprintln!("error generating light scale '{}': {}", name, e); std::process::exit(1); }
        };

        let dark = match labui_core::neutral::create_neutral_dark_scale(
            &scale_cfg.light, &scale_cfg.base, &scale_cfg.dark, &params,
        ) {
            Ok(s) => s,
            Err(e) => { eprintln!("error generating dark scale '{}': {}", name, e); std::process::exit(1); }
        };

        let (ic_light, ic_dark) = if let Some(ref ic) = scale_cfg.ic {
            let ic_l = match labui_core::neutral::create_neutral_light_scale(
                &ic.light, &ic.base, &ic.dark, &params,
            ) {
                Ok(s) => s,
                Err(e) => { eprintln!("error generating IC light scale '{}': {}", name, e); std::process::exit(1); }
            };
            let ic_d = match labui_core::neutral::create_neutral_dark_scale(
                &ic.light, &ic.base, &ic.dark, &params,
            ) {
                Ok(s) => s,
                Err(e) => { eprintln!("error generating IC dark scale '{}': {}", name, e); std::process::exit(1); }
            };
            (Some(ic_l), Some(ic_d))
        } else {
            (None, None)
        };

        let mut emit_selector = |selector: &str, json_prefix: &str, scale: &[String]| {
            scss.push_str(&format!("{} {{\n", selector));
            for (i, hex) in scale.iter().enumerate() {
                let var = format!("--{}-{}", name, i);
                scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
                json_map.insert(format!("{}{}", json_prefix, var), hex.to_lowercase());
            }
            scss.push_str("}\n");
        };

        emit_selector(":root", "root-", &light);
        emit_selector(".dark", "dark-", &dark);
        if let Some(ref ic_l) = ic_light {
            emit_selector(".ic", "ic-", ic_l);
        }
        if let Some(ref ic_d) = ic_dark {
            emit_selector(".dark.ic", "dark-ic-", ic_d);
        }
    }

    // ------------------------------------------------------------------
    // 2. Generate accent blocks (once per theme selector).
    // ------------------------------------------------------------------
    if !config.accents.is_empty() {
        let themes: Vec<(&str, &str, &str, bool, bool)> = vec![
            (":root", "root-", &bg_light, false, false),
            (".dark", "dark-", &bg_dark, true, false),
        ];

        for (selector, json_prefix, bg, is_dark, is_ic) in &themes {
            scss.push_str(&format!("{} {{\n", selector));
            for (accent_name, accent_hex) in &config.accents {
                let cfg = labui_core::accent::AccentConfig::from_hex(accent_hex);
                match labui_core::accent::resolve_accent_base(&cfg, *is_dark, *is_ic, bg) {
                    Ok(hex) => {
                        let var = format!("--accent-{}", accent_name);
                        scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
                        json_map.insert(format!("{}{}", json_prefix, var), hex.to_lowercase());
                    }
                    Err(e) => {
                        eprintln!("error resolving accent '{}': {}", accent_name, e);
                        std::process::exit(1);
                    }
                }
            }
            scss.push_str("}\n");
        }

        // IC themes (if IC neutral scales exist)
        if let (Some(ic_light), Some(ic_dark)) = (&bg_ic_light, &bg_ic_dark) {
            let ic_themes: Vec<(&str, &str, &str, bool, bool)> = vec![
                (".ic", "ic-", ic_light, false, true),
                (".dark.ic", "dark-ic-", ic_dark, true, true),
            ];
            for (selector, json_prefix, bg, is_dark, is_ic) in &ic_themes {
                scss.push_str(&format!("{} {{\n", selector));
                for (accent_name, accent_hex) in &config.accents {
                    let cfg = labui_core::accent::AccentConfig::from_hex(accent_hex);
                    match labui_core::accent::resolve_accent_base(&cfg, *is_dark, *is_ic, bg) {
                        Ok(hex) => {
                            let var = format!("--accent-{}", accent_name);
                            scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
                            json_map.insert(format!("{}{}", json_prefix, var), hex.to_lowercase());
                        }
                        Err(e) => {
                            eprintln!("error resolving IC accent '{}': {}", accent_name, e);
                            std::process::exit(1);
                        }
                    }
                }
                scss.push_str("}\n");
            }
        }
    }

    fs::create_dir_all(Path::new(&config.output.scss).parent().unwrap_or(Path::new(".")))
        .expect("failed to create output directory");

    fs::write(&config.output.scss, scss).expect("failed to write scss");
    fs::write(&config.output.json, serde_json::to_string_pretty(&json_map).unwrap())
        .expect("failed to write json");

    println!("Generated:");
    println!("  {}", config.output.scss);
    println!("  {}", config.output.json);
}
