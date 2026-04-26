#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    primitives: BTreeMap<String, ScaleConfig>,
    #[serde(default)]
    accents: BTreeMap<String, String>,
    #[serde(default)]
    accent_theming: AccentThemingConfig,
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

#[derive(Debug, Deserialize, Serialize)]
struct AccentThemingConfig {
    #[serde(default = "default_dark_factor")]
    dark_factor: f64,
    #[serde(default = "default_ic_boost")]
    ic_boost: f64,
}

impl Default for AccentThemingConfig {
    fn default() -> Self {
        Self {
            dark_factor: default_dark_factor(),
            ic_boost: default_ic_boost(),
        }
    }
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
fn default_lightness_ease() -> f64 { 1.7 }
fn default_hue_ease() -> f64 { 0.6 }
fn default_chroma_peak() -> f64 { 0.35 }
fn default_dark_factor() -> f64 { 0.7 }
fn default_ic_boost() -> f64 { 15.0 }

// ------------------------------------------------------------------
//  Generation pipeline
// ------------------------------------------------------------------

#[derive(Debug)]
enum GenerateError {
    NeutralScale { name: String, source: String },
    AccentResolve { name: String, source: String },
    JsonSerialize(serde_json::Error),
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerateError::NeutralScale { name, source } => {
                write!(f, "error generating scale '{}': {}", name, source)
            }
            GenerateError::AccentResolve { name, source } => {
                write!(f, "error resolving accent '{}': {}", name, source)
            }
            GenerateError::JsonSerialize(e) => {
                write!(f, "JSON serialization error: {}", e)
            }
        }
    }
}

impl std::error::Error for GenerateError {}

struct SelectorBlock {
    json_prefix: String,
    neutral_vars: Vec<(String, String)>,
    accent_vars: Vec<(String, String)>,
}

/// Resolve theme background anchors from the "neutral" primitive (or fallback).
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

/// Generate neutral scales and collect variables into per-selector blocks.
fn collect_neutral_blocks(
    config: &Config,
) -> Result<BTreeMap<String, SelectorBlock>, GenerateError> {
    let mut blocks: BTreeMap<String, SelectorBlock> = BTreeMap::new();

    for (name, scale_cfg) in &config.primitives {
        let params = labui_core::neutral::CurveParams {
            lightness_ease: scale_cfg.curve.lightness_ease,
            hue_ease: scale_cfg.curve.hue_ease,
            chroma_peak: scale_cfg.curve.chroma_peak,
        };

        let light = labui_core::neutral::create_neutral_light_scale(
            &scale_cfg.light, &scale_cfg.base, &scale_cfg.dark, &params,
        ).map_err(|e| GenerateError::NeutralScale { name: name.clone(), source: e })?;

        let dark = labui_core::neutral::create_neutral_dark_scale(
            &scale_cfg.light, &scale_cfg.base, &scale_cfg.dark, &params,
        ).map_err(|e| GenerateError::NeutralScale { name: name.clone(), source: e })?;

        let (ic_light, ic_dark) = if let Some(ref ic) = scale_cfg.ic {
            let ic_l = labui_core::neutral::create_neutral_light_scale(
                &ic.light, &ic.base, &ic.dark, &params,
            ).map_err(|e| GenerateError::NeutralScale { name: format!("{} IC light", name), source: e })?;
            let ic_d = labui_core::neutral::create_neutral_dark_scale(
                &ic.light, &ic.base, &ic.dark, &params,
            ).map_err(|e| GenerateError::NeutralScale { name: format!("{} IC dark", name), source: e })?;
            (Some(ic_l), Some(ic_d))
        } else {
            (None, None)
        };

        push_neutral_vars(&mut blocks, ":root", "root-", name, &light);
        push_neutral_vars(&mut blocks, ".dark", "dark-", name, &dark);
        if let Some(ref ic_l) = ic_light {
            push_neutral_vars(&mut blocks, ".ic", "ic-", name, ic_l);
        }
        if let Some(ref ic_d) = ic_dark {
            push_neutral_vars(&mut blocks, ".dark.ic", "dark-ic-", name, ic_d);
        }
    }

    Ok(blocks)
}

fn push_neutral_vars(
    blocks: &mut BTreeMap<String, SelectorBlock>,
    selector: &str,
    json_prefix: &str,
    primitive_name: &str,
    scale: &[String],
) {
    let block = blocks.entry(selector.to_string()).or_insert_with(|| SelectorBlock {
        json_prefix: json_prefix.to_string(),
        neutral_vars: Vec::new(),
        accent_vars: Vec::new(),
    });
    for (i, hex) in scale.iter().enumerate() {
        let var = format!("--{}-{}", primitive_name, i);
        block.neutral_vars.push((var, hex.to_lowercase()));
    }
}

/// Resolve accent colours per theme and push into the selector blocks.
fn collect_accent_vars(
    config: &Config,
    blocks: &mut BTreeMap<String, SelectorBlock>,
) -> Result<(), GenerateError> {
    if config.accents.is_empty() {
        return Ok(());
    }

    let (bg_light, bg_dark, bg_ic_light, bg_ic_dark) = resolve_bg_anchors(config);

    let theming_params = labui_core::accent::AccentThemingParams {
        dark_factor: config.accent_theming.dark_factor,
        ic_boost: config.accent_theming.ic_boost,
    };

    for (accent_name, accent_hex) in &config.accents {
        let cfg = labui_core::accent::AccentConfig::from_hex(accent_hex);
        for (selector, block) in blocks.iter_mut() {
            let (is_dark, is_ic, bg_hex) = match selector.as_str() {
                ":root" => (false, false, &bg_light),
                ".dark" => (true, false, &bg_dark),
                ".ic" => (false, true, bg_ic_light.as_ref().unwrap_or(&bg_light)),
                ".dark.ic" => (true, true, bg_ic_dark.as_ref().unwrap_or(&bg_dark)),
                _ => continue,
            };
            let hex = labui_core::accent::resolve_accent_base(&cfg, is_dark, is_ic, bg_hex, &theming_params)
                .map_err(|e| GenerateError::AccentResolve { name: accent_name.clone(), source: e })?;
            let var = format!("--accent-{}", accent_name);
            block.accent_vars.push((var, hex.to_lowercase()));
        }
    }

    Ok(())
}

/// Emit SCSS and JSON from the collected blocks in logical selector order.
fn emit_css_and_json(
    blocks: BTreeMap<String, SelectorBlock>,
) -> Result<(String, String), GenerateError> {
    let mut scss = String::new();
    let mut json_map: BTreeMap<String, String> = BTreeMap::new();

    let selector_order = [":root", ".dark", ".ic", ".dark.ic"];
    for selector in &selector_order {
        let Some(block) = blocks.get(*selector) else { continue };
        scss.push_str(&format!("{} {{\n", selector));
        for (var, hex) in &block.neutral_vars {
            scss.push_str(&format!("  {}: {};\n", var, hex));
        }
        for (var, hex) in &block.accent_vars {
            scss.push_str(&format!("  {}: {};\n", var, hex));
        }
        scss.push_str("}\n");

        for (var, hex) in &block.neutral_vars {
            json_map.insert(format!("{}{}", block.json_prefix, var), hex.clone());
        }
        for (var, hex) in &block.accent_vars {
            json_map.insert(format!("{}{}", block.json_prefix, var), hex.clone());
        }
    }

    let json = serde_json::to_string_pretty(&json_map)
        .map_err(GenerateError::JsonSerialize)?;

    Ok((scss, json))
}

fn generate(config: &Config) -> Result<(String, String), GenerateError> {
    let mut blocks = collect_neutral_blocks(config)?;
    collect_accent_vars(config, &mut blocks)?;
    emit_css_and_json(blocks)
}

// ------------------------------------------------------------------
//  Main entrypoint
// ------------------------------------------------------------------

fn main() {
    let config_path = Path::new("config.yaml");
    if !config_path.exists() {
        eprintln!("error: config.yaml not found");
        std::process::exit(1);
    }

    let yaml = fs::read_to_string(config_path).expect("failed to read config.yaml");
    let config: Config = serde_yaml::from_str(&yaml).expect("failed to parse config.yaml");

    // Warn about incomplete IC anchors (outside generate() to keep tests quiet).
    for (name, scale_cfg) in &config.primitives {
        if let Some(ref ic) = scale_cfg.ic {
            if ic.light.is_empty() || ic.base.is_empty() || ic.dark.is_empty() {
                eprintln!("warning: primitive '{}' has incomplete IC anchors", name);
            }
        }
    }

    let (scss, json) = match generate(&config) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    fs::create_dir_all(Path::new(&config.output.scss).parent().unwrap_or(Path::new(".")))
        .expect("failed to create output directory");

    fs::write(&config.output.scss, scss).expect("failed to write scss");
    fs::write(&config.output.json, json).expect("failed to write json");

    println!("Generated:");
    println!("  {}", config.output.scss);
    println!("  {}", config.output.json);
}
