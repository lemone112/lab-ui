use wasm_bindgen::prelude::*;

/// Generate a 13-step neutral light-mode scale.
///
/// Returns a JSON array of hex strings.
#[wasm_bindgen]
pub fn create_neutral_light_scale(light: &str, base: &str, dark: &str) -> String {
    let scale = labui_core::neutral::create_neutral_light_scale(light, base, dark);
    serde_json::to_string(&scale).unwrap()
}
