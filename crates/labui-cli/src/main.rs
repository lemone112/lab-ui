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

    let mut scss = String::new();
    let mut json_map: HashMap<String, String> = HashMap::new();

    for (name, scale_cfg) in &config.primitives {
        let params = labui_core::neutral::CurveParams {
            lightness_ease: scale_cfg.curve.lightness_ease,
            hue_ease: scale_cfg.curve.hue_ease,
            chroma_peak: scale_cfg.curve.chroma_peak,
        };

        // Normal light scale → :root
        let light = match labui_core::neutral::create_neutral_light_scale(
            &scale_cfg.light, &scale_cfg.base, &scale_cfg.dark, &params,
        ) {
            Ok(s) => s,
            Err(e) => { eprintln!("error generating light scale '{}': {}", name, e); std::process::exit(1); }
        };

        // Normal dark scale → .dark
        let dark = match labui_core::neutral::create_neutral_dark_scale(
            &scale_cfg.light, &scale_cfg.base, &scale_cfg.dark, &params,
        ) {
            Ok(s) => s,
            Err(e) => { eprintln!("error generating dark scale '{}': {}", name, e); std::process::exit(1); }
        };

        scss.push_str(":root {\n");
        for (i, hex) in light.iter().enumerate() {
            let var = format!("--{}-{}", name, i);
            scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
            json_map.insert(format!("root-{}", var), hex.to_lowercase());
        }
        scss.push_str("}\n");

        scss.push_str(".dark {\n");
        for (i, hex) in dark.iter().enumerate() {
            let var = format!("--{}-{}", name, i);
            scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
            json_map.insert(format!("dark-{}", var), hex.to_lowercase());
        }
        scss.push_str("}\n");

        // IC scales (only if IC anchors provided)
        if let Some(ref ic) = scale_cfg.ic {
            let ic_light = match labui_core::neutral::create_neutral_light_scale(
                &ic.light, &ic.base, &ic.dark, &params,
            ) {
                Ok(s) => s,
                Err(e) => { eprintln!("error generating IC light scale '{}': {}", name, e); std::process::exit(1); }
            };

            let ic_dark = match labui_core::neutral::create_neutral_dark_scale(
                &ic.light, &ic.base, &ic.dark, &params,
            ) {
                Ok(s) => s,
                Err(e) => { eprintln!("error generating IC dark scale '{}': {}", name, e); std::process::exit(1); }
            };

            scss.push_str(".ic {\n");
            for (i, hex) in ic_light.iter().enumerate() {
                let var = format!("--{}-{}", name, i);
                scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
                json_map.insert(format!("ic-{}", var), hex.to_lowercase());
            }
            scss.push_str("}\n");

            scss.push_str(".dark.ic {\n");
            for (i, hex) in ic_dark.iter().enumerate() {
                let var = format!("--{}-{}", name, i);
                scss.push_str(&format!("  {}: {};\n", var, hex.to_lowercase()));
                json_map.insert(format!("dark-ic-{}", var), hex.to_lowercase());
            }
            scss.push_str("}\n");
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
