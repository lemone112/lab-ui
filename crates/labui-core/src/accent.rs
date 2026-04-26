use crate::apca::{apca_contrast, apca_inverse, srgb_hex_to_y};
use crate::color::ucs::Cam16Ucs;

/// Dark-theme contrast factor, empirically derived from Apple iOS System
/// Color patterns: dark-mode accents need ~70 % of the light-mode contrast
/// magnitude to avoid excessive brightness.
const DARK_FACTOR: f64 = 0.7;

/// Increased-contrast boost (APCA Lc).  IC variants shift by ±15 Lc.
const IC_BOOST: f64 = 15.0;

/// Accent configuration: a single canonical hex.
///
/// Four theme variants are generated algorithmically via APCA inverse.
#[derive(Debug, Clone, PartialEq)]
pub struct AccentConfig {
    pub canonical: String,
}

impl AccentConfig {
    pub fn from_hex(hex: &str) -> Self {
        Self {
            canonical: hex.to_string(),
        }
    }
}

/// Resolve the accent base hex for a given theme.
///
/// * `is_dark` — dark mode (light text on dark background).
/// * `is_ic`  — increased contrast.
/// * `bg_hex` — reference background for this theme.
pub fn resolve_accent_base(
    config: &AccentConfig,
    is_dark: bool,
    is_ic: bool,
    bg_hex: &str,
) -> Result<String, String> {
    let y_canonical = srgb_hex_to_y(&config.canonical)?;
    let y_white = srgb_hex_to_y("#FFFFFF")?;
    let lc_on_white = apca_contrast(y_canonical, y_white);

    let y_bg = srgb_hex_to_y(bg_hex)?;

    if !is_dark && !is_ic {
        return Ok(config.canonical.clone());
    }

    let target_lc = match (is_dark, is_ic) {
        (true, false) => -lc_on_white.abs() * DARK_FACTOR,
        (false, true) => lc_on_white.abs() + IC_BOOST,
        (true, true) => -(lc_on_white.abs() + IC_BOOST) * DARK_FACTOR,
        (false, false) => unreachable!(),
    };

    let canonical_ucs = Cam16Ucs::from_hex(&config.canonical)?;
    match apca_inverse(y_bg, target_lc, &canonical_ucs) {
        Some(ucs) => Ok(ucs.to_hex()),
        None => Err(format!(
            "apca_inverse failed for {} on bg {} with target Lc {}",
            config.canonical, bg_hex, target_lc
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_stores_canonical() {
        let cfg = AccentConfig::from_hex("#007AFF");
        assert_eq!(cfg.canonical, "#007AFF");
    }

    #[test]
    fn light_theme_returns_canonical() {
        let cfg = AccentConfig::from_hex("#007AFF");
        let got = resolve_accent_base(&cfg, false, false, "#FFFFFF").unwrap();
        assert_eq!(got.to_ascii_uppercase(), "#007AFF");
    }

    #[test]
    fn dark_theme_returns_lighter() {
        let cfg = AccentConfig::from_hex("#007AFF");
        let got = resolve_accent_base(&cfg, true, false, "#101012").unwrap();
        let y_original = srgb_hex_to_y("#007AFF").unwrap();
        let y_derived = srgb_hex_to_y(&got).unwrap();
        assert!(
            y_derived > y_original,
            "dark theme accent should be lighter, got {} (Y={}) vs #007AFF (Y={})",
            got, y_derived, y_original
        );
    }

    #[test]
    fn ic_light_returns_darker() {
        let cfg = AccentConfig::from_hex("#007AFF");
        let got = resolve_accent_base(&cfg, false, true, "#FFFFFF").unwrap();
        let y_original = srgb_hex_to_y("#007AFF").unwrap();
        let y_derived = srgb_hex_to_y(&got).unwrap();
        assert!(
            y_derived < y_original,
            "IC light accent should be darker, got {} (Y={}) vs #007AFF (Y={})",
            got, y_derived, y_original
        );
    }
}
