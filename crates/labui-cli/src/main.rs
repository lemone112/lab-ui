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
    semantic: HashMap<String, String>,
    #[serde(default)]
    output: OutputConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScaleConfig {
    light: String,
    base: String,
    dark: String,
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
    scss.push_str(":root {\n");

    let mut json_map: HashMap<String, String> = HashMap::new();

    for (name, scale_cfg) in &config.primitives {
        let scale = labui_core::neutral::create_neutral_light_scale(
            &scale_cfg.light,
            &scale_cfg.base,
            &scale_cfg.dark,
        );

        for (i, hex) in scale.iter().enumerate() {
            let var_name = format!("--{}-{}", name, i);
            scss.push_str(&format!("  {}: {};\n", var_name, hex.to_lowercase()));
            json_map.insert(var_name.clone(), hex.to_lowercase());
        }
    }

    for (semantic_name, primitive_ref) in &config.semantic {
        let var_name = format!("--{}", semantic_name.replace('_', "-"));
        let target = format!("--{}", primitive_ref);
        scss.push_str(&format!("  {}: var({});\n", var_name, target));
        json_map.insert(var_name.clone(), format!("var({})", target));
    }

    scss.push_str("}\n");

    fs::create_dir_all(Path::new(&config.output.scss).parent().unwrap_or(Path::new(".")))
        .expect("failed to create output directory");

    fs::write(&config.output.scss, scss).expect("failed to write scss");
    fs::write(&config.output.json, serde_json::to_string_pretty(&json_map).unwrap())
        .expect("failed to write json");

    println!("Generated:",);
    println!("  {}", config.output.scss);
    println!("  {}", config.output.json);
}
