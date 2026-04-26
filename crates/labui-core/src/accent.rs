use crate::tint::perceptual_mix;

/// Accent colour with optional per-theme overrides.
///
/// When created with [`AccentConfig::from_hex`], all four theme variants
/// share the same base colour. Override individual themes via the
/// struct fields for Apple-style adaptive accents.
#[derive(Debug, Clone, PartialEq)]
pub struct AccentConfig {
    pub light: String,
    pub dark: String,
    pub ic_light: String,
    pub ic_dark: String,
}

impl AccentConfig {
    /// Create a theme-independent accent (same hex in all four themes).
    pub fn from_hex(hex: &str) -> Self {
        let h = hex.to_string();
        Self {
            light: h.clone(),
            dark: h.clone(),
            ic_light: h.clone(),
            ic_dark: h,
        }
    }
}

/// Resolve the accent base hex for a given theme.
pub fn resolve_accent_base(config: &AccentConfig, is_dark: bool, is_ic: bool) -> &str {
    match (is_dark, is_ic) {
        (false, false) => &config.light,
        (true, false) => &config.dark,
        (false, true) => &config.ic_light,
        (true, true) => &config.ic_dark,
    }
}

/// Generate the complete tint matrix for one accent on one neutral scale.
///
/// Returns `Vec<Vec<String>>` indexed as `[strength_index][bg_step]`.
/// `strengths` are percentages (e.g. `[2, 4, 8, 12, 20, 32, 52, 72]`).
pub fn create_accent_tint_matrix(
    accent_base: &str,
    neutral_scale: &[String],
    strengths: &[u8],
) -> Result<Vec<Vec<String>>, String> {
    let mut matrix = Vec::with_capacity(strengths.len());
    for strength in strengths {
        let s = f64::from(*strength) / 100.0;
        let mut row = Vec::with_capacity(neutral_scale.len());
        for bg in neutral_scale {
            row.push(perceptual_mix(accent_base, bg, s)?);
        }
        matrix.push(row);
    }
    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_clones_to_all_themes() {
        let cfg = AccentConfig::from_hex("#007AFF");
        assert_eq!(cfg.light, "#007AFF");
        assert_eq!(cfg.dark, "#007AFF");
        assert_eq!(cfg.ic_light, "#007AFF");
        assert_eq!(cfg.ic_dark, "#007AFF");
    }

    #[test]
    fn resolve_base_mapping() {
        let cfg = AccentConfig {
            light: "#007AFF".into(),
            dark: "#4A8FFF".into(),
            ic_light: "#0040DD".into(),
            ic_dark: "#409CFF".into(),
        };
        assert_eq!(resolve_accent_base(&cfg, false, false), "#007AFF");
        assert_eq!(resolve_accent_base(&cfg, true, false), "#4A8FFF");
        assert_eq!(resolve_accent_base(&cfg, false, true), "#0040DD");
        assert_eq!(resolve_accent_base(&cfg, true, true), "#409CFF");
    }

    #[test]
    fn tint_matrix_shape() {
        let neutral = vec!["#FFFFFF".into(), "#787880".into(), "#101012".into()];
        let strengths = &[20, 50];
        let matrix = create_accent_tint_matrix("#007AFF", &neutral, strengths).unwrap();
        assert_eq!(matrix.len(), 2);       // 2 strengths
        assert_eq!(matrix[0].len(), 3);    // 3 bg steps
        assert_eq!(matrix[1].len(), 3);
    }

    #[test]
    fn tint_matrix_anchors() {
        let neutral = vec!["#FFFFFF".into()];
        let strengths = &[0, 100];
        let matrix = create_accent_tint_matrix("#007AFF", &neutral, strengths).unwrap();
        // 0% strength → bg colour
        assert_eq!(matrix[0][0].to_ascii_uppercase(), "#FFFFFF");
        // 100% strength → accent colour
        assert_eq!(matrix[1][0].to_ascii_uppercase(), "#007AFF");
    }
}
