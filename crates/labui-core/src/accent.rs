use crate::apca::{apca_contrast, apca_inverse, apca_to_hex, srgb_hex_to_y};
use crate::color::ucs::Cam16Ucs;

/// Parameters that control how a canonical accent is adapted across
/// themes.  These are empirically derived from Apple iOS System Color
/// patterns but exposed so users can tune them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentThemingParams {
    /// Dark-mode contrast as a fraction of the light-mode contrast
    /// magnitude.  Value ~0.7 prevents dark accents from becoming
    /// excessively bright.
    pub dark_factor: f64,
    /// Increased-contrast boost (APCA Lc).  Added to the absolute
    /// light-mode contrast for IC variants.
    pub ic_boost: f64,
}

impl Default for AccentThemingParams {
    fn default() -> Self {
        Self {
            dark_factor: 0.7,
            ic_boost: 15.0,
        }
    }
}

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
/// * `params` — theming parameters (`dark_factor`, `ic_boost`).
///
/// # IC Dark formula note
///
/// The IC Dark target is `-(lc_on_white + ic_boost) * dark_factor`.
/// For Brand this yields ~-55 Lc against Figma's ~-47.  The 8 Lc gap
/// is acceptable because the design system does not copy Figma 1:1;
/// it algorithmically derives values.  Users who need exact Figma
/// matching can lower `dark_factor` or `ic_boost` via
/// [`AccentThemingParams`].
pub fn resolve_accent_base(
    config: &AccentConfig,
    is_dark: bool,
    is_ic: bool,
    bg_hex: &str,
    params: &AccentThemingParams,
) -> Result<String, String> {
    let y_canonical = srgb_hex_to_y(&config.canonical)?;
    let y_white = srgb_hex_to_y("#FFFFFF")?;
    let lc_on_white = apca_contrast(y_canonical, y_white);

    let y_bg = srgb_hex_to_y(bg_hex)?;

    if !is_dark && !is_ic {
        return Ok(config.canonical.clone());
    }

    let target_lc = match (is_dark, is_ic) {
        (true, false) => -lc_on_white.abs() * params.dark_factor,
        (false, true) => lc_on_white.abs() + params.ic_boost,
        (true, true) => -(lc_on_white.abs() + params.ic_boost) * params.dark_factor,
        (false, false) => unreachable!(),
    };

    let canonical_ucs = Cam16Ucs::from_hex(&config.canonical)?;
    match apca_inverse(y_bg, target_lc, &canonical_ucs) {
        Some(ucs) => Ok(apca_to_hex(&ucs)),
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
        let got = resolve_accent_base(&cfg, false, false, "#FFFFFF", &AccentThemingParams::default()).unwrap();
        assert_eq!(got.to_ascii_uppercase(), "#007AFF");
    }

    #[test]
    fn dark_theme_returns_lighter() {
        let cfg = AccentConfig::from_hex("#007AFF");
        let got = resolve_accent_base(&cfg, true, false, "#101012", &AccentThemingParams::default()).unwrap();
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
        let got = resolve_accent_base(&cfg, false, true, "#FFFFFF", &AccentThemingParams::default()).unwrap();
        let y_original = srgb_hex_to_y("#007AFF").unwrap();
        let y_derived = srgb_hex_to_y(&got).unwrap();
        assert!(
            y_derived < y_original,
            "IC light accent should be darker, got {} (Y={}) vs #007AFF (Y={})",
            got, y_derived, y_original
        );
    }

    #[test]
    fn dark_theme_apca_contract() {
        let cfg = AccentConfig::from_hex("#007AFF");
        let bg_dark = "#101012";
        let got = resolve_accent_base(&cfg, true, false, bg_dark, &AccentThemingParams::default()).unwrap();

        let y_got = srgb_hex_to_y(&got).unwrap();
        let y_bg = srgb_hex_to_y(bg_dark).unwrap();
        let lc_got = apca_contrast(y_got, y_bg);

        let y_canonical = srgb_hex_to_y("#007AFF").unwrap();
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();
        let lc_canonical = apca_contrast(y_canonical, y_white);
        let expected_target = -(lc_canonical.abs() * AccentThemingParams::default().dark_factor);

        assert!(
            (lc_got - expected_target).abs() < 1.0,
            "dark accent Lc {} should be close to target {} (within 1.0)",
            lc_got, expected_target
        );
    }

    #[test]
    fn ic_light_apca_contract() {
        let cfg = AccentConfig::from_hex("#007AFF");
        let bg = "#FFFFFF";
        let got = resolve_accent_base(&cfg, false, true, bg, &AccentThemingParams::default()).unwrap();

        let y_got = srgb_hex_to_y(&got).unwrap();
        let y_bg = srgb_hex_to_y(bg).unwrap();
        let lc_got = apca_contrast(y_got, y_bg);

        let y_canonical = srgb_hex_to_y("#007AFF").unwrap();
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();
        let lc_canonical = apca_contrast(y_canonical, y_white);
        let expected_target = lc_canonical.abs() + AccentThemingParams::default().ic_boost;

        assert!(
            (lc_got - expected_target).abs() < 1.0,
            "IC light accent Lc {} should be close to target {} (within 1.0)",
            lc_got, expected_target
        );
    }
}
