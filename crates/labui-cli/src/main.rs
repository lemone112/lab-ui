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
    accents: HashMap<String, AccentValue>,
    #[serde(default)]
    tint: TintConfig,
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

/// Accent input: either a single hex for all themes, or per-theme overrides.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum AccentValue {
    Simple(String),
    Themed {
        light: String,
        dark: String,
        #[serde(rename = "ic_light")]
        ic_light: String,
        #[serde(rename = "ic_dark")]
        ic_dark: String,
    },
}

impl From<AccentValue> for labui_core::accent::AccentConfig {
    fn from(v: AccentValue) -> Self {
        match v {
            AccentValue::Simple(hex) => Self::from_hex(&hex),
            AccentValue::Themed { light, dark, ic_light, ic_dark } => Self {
                light,
                dark,
                ic_light,
                ic_dark,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct TintConfig {
    #[serde(default = "default_strengths")]
    strengths: Vec<u8>,
}

fn default_strengths() -> Vec<u8> {
    vec![2, 4, 8, 12, 20, 32, 52, 72]
}

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

    // Store neutral scales for tint matrix generation.
    // Key: primitive name → (light, dark, ic_light, ic_dark)
    let mut scale_store: HashMap<String, (Vec<String>, Vec<String>, Option<Vec<String>>, Option<Vec<String>>)> = HashMap::new();

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

        scale_store.insert(name.clone(), (light.clone(), dark.clone(), ic_light.clone(), ic_dark.clone()));

        scss.push_str(":root {\n");
        for (i, hex) in light.iter().enumerate() {
            let var = format!("--{}-{}", name, i);
            scss.push_str(&format!("  {}: {};", var, hex.to_lowercase()));
            json_map.insert(format!("root-{}", var), hex.to_lowercase());
        }
        scss.push_str("}\n");

        scss.push_str(".dark {\n");
        for (i, hex) in dark.iter().enumerate() {
            let var = format!("--{}-{}", name, i);
            scss.push_str(&format!("  {}: {};", var, hex.to_lowercase()));
            json_map.insert(format!("dark-{}", var), hex.to_lowercase());
        }
        scss.push_str("}\n");

        if let Some(ref ic_l) = ic_light {
            scss.push_str(".ic {\n");
            for (i, hex) in ic_l.iter().enumerate() {
                let var = format!("--{}-{}", name, i);
                scss.push_str(&format!("  {}: {};", var, hex.to_lowercase()));
                json_map.insert(format!("ic-{}", var), hex.to_lowercase());
            }
            scss.push_str("}\n");
        }

        if let Some(ref ic_d) = ic_dark {
            scss.push_str(".dark.ic {\n");
            for (i, hex) in ic_d.iter().enumerate() {
                let var = format!("--{}-{}", name, i);
                scss.push_str(&format!("  {}: {};", var, hex.to_lowercase()));
                json_map.insert(format!("dark-ic-{}", var), hex.to_lowercase());
            }
            scss.push_str("}\n");
        }
    }

    // Generate accent tokens.
    if !config.accents.is_empty() {
        // Use the first primitive's neutral scale for tint backgrounds.
        let first_primitive = scale_store.keys().next()
            .expect("at least one primitive required when accents are defined");
        let (neutral_light, neutral_dark, neutral_ic_light, neutral_ic_dark) = scale_store.get(first_primitive).unwrap().clone();

        let themes = [
            (false, false, ":root", "root-", neutral_light),
            (true, false, ".dark", "dark-", neutral_dark),
        ];

        for (is_dark, is_ic, selector, json_prefix, neutral) in &themes {
            scss.push_str(&format!("{} {{", selector));
            for (name, val) in &config.accents {
                let cfg: labui_core::accent::AccentConfig = val.clone().into();
                let base = labui_core::accent::resolve_accent_base(&cfg, *is_dark, *is_ic);

                let var = format!("--accent-{}", name);
                scss.push_str(&format!("  {}: {};", var, base.to_lowercase()));
                json_map.insert(format!("{}{}", json_prefix, var), base.to_lowercase());

                let matrix = match labui_core::accent::create_accent_tint_matrix(base, neutral, &config.tint.strengths) {
                    Ok(m) => m,
                    Err(e) => { eprintln!("error generating tint matrix for '{}': {}", name, e); std::process::exit(1); }
                };

                for (si, strength) in config.tint.strengths.iter().enumerate() {
                    for step in 0..neutral.len() {
                        let var = format!("--tint-{}-{}-on-neutral-{}", name, strength, step);
                        let hex = matrix[si][step].to_lowercase();
                        scss.push_str(&format!("  {}: {};", var, hex));
                        json_map.insert(format!("{}{}", json_prefix, var), hex);
                    }
                }
            }
            scss.push_str("}\n");
        }

        // IC themes (if IC neutral scales exist)
        if let (Some(ic_light), Some(ic_dark)) = (neutral_ic_light, neutral_ic_dark) {
            let ic_themes = [
                (false, true, ".ic", "ic-", ic_light),
                (true, true, ".dark.ic", "dark-ic-", ic_dark),
            ];
            for (is_dark, is_ic, selector, json_prefix, neutral) in &ic_themes {
                scss.push_str(&format!("{} {{", selector));
                for (name, val) in &config.accents {
                    let cfg: labui_core::accent::AccentConfig = val.clone().into();
                    let base = labui_core::accent::resolve_accent_base(&cfg, *is_dark, *is_ic);

                    let var = format!("--accent-{}", name);
                    scss.push_str(&format!("  {}: {};", var, base.to_lowercase()));
                    json_map.insert(format!("{}{}", json_prefix, var), base.to_lowercase());

                    let matrix = match labui_core::accent::create_accent_tint_matrix(base, neutral, &config.tint.strengths) {
                        Ok(m) => m,
                        Err(e) => { eprintln!("error generating IC tint matrix for '{}': {}", name, e); std::process::exit(1); }
                    };

                    for (si, strength) in config.tint.strengths.iter().enumerate() {
                        for step in 0..neutral.len() {
                            let var = format!("--tint-{}-{}-on-neutral-{}", name, strength, step);
                            let hex = matrix[si][step].to_lowercase();
                            scss.push_str(&format!("  {}: {};", var, hex));
                            json_map.insert(format!("{}{}", json_prefix, var), hex);
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
