use crate::Cam16Ucs;

/// Perceptual curve parameters for neutral-scale interpolation.
///
/// Defaults are tuned to the sRGB average-surround neutral scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveParams {
    /// Lightness ease exponent (power ease-in / ease-out).
    pub lightness_ease: f64,
    /// Hue ease exponent (how fast white's intrinsic hue snaps to base).
    pub hue_ease: f64,
    /// Sinusoid peak position, normalised 0..1.
    pub chroma_peak: f64,
}

impl Default for CurveParams {
    fn default() -> Self {
        Self {
            lightness_ease: 1.7,
            hue_ease: 0.6,
            chroma_peak: 0.35,
        }
    }
}

/// Generate a 13-step neutral scale for light mode.
pub fn create_neutral_light_scale(
    light: &str,
    base: &str,
    dark: &str,
    params: &CurveParams,
) -> Result<Vec<String>, String> {
    let a0 = Cam16Ucs::from_hex(light)?;
    let a6 = Cam16Ucs::from_hex(base)?;
    let a12 = Cam16Ucs::from_hex(dark)?;

    let c_base = (a6.ap * a6.ap + a6.bp * a6.bp).sqrt();
    let c_dark = (a12.ap * a12.ap + a12.bp * a12.bp).sqrt();

    let base_hue = a6.bp.atan2(a6.ap);

    let mut out = Vec::with_capacity(13);
    for i in 0..=12 {
        // Pin anchors exactly to avoid roundtrip drift.
        match i {
            0 => { out.push(light.to_string()); continue; }
            6 => { out.push(base.to_string()); continue; }
            12 => { out.push(dark.to_string()); continue; }
            _ => {}
        }

        let t = i as f64 / 12.0;

        // ----- J' (lightness) -----
        let jp = if i < 6 {
            let u = i as f64 / 6.0;
            a0.jp + (a6.jp - a0.jp) * ease_in(u, params.lightness_ease)
        } else {
            let u = (i - 6) as f64 / 6.0;
            a6.jp + (a12.jp - a6.jp) * ease_out(u, params.lightness_ease)
        };

        // ----- M' (chroma) -----
        let env = sine_env(t, params.chroma_peak);
        let mp = c_dark + (c_base - c_dark) * env;

        // ----- h (hue) -----
        let hue = if i < 6 {
            let u = i as f64 / 6.0;
            let start_hue = a0.bp.atan2(a0.ap);
            start_hue + (base_hue - start_hue) * ease_in(u, params.hue_ease)
        } else {
            let u = (i - 6) as f64 / 6.0;
            base_hue + (a12.bp.atan2(a12.ap) - base_hue) * u
        };

        out.push(
            Cam16Ucs {
                jp,
                ap: mp * hue.cos(),
                bp: mp * hue.sin(),
            }
            .to_hex(),
        );
    }
    Ok(out)
}

fn ease_in(t: f64, p: f64) -> f64 {
    t.powf(p)
}

fn ease_out(t: f64, p: f64) -> f64 {
    1.0 - (1.0 - t).powf(p)
}

fn sine_env(t: f64, t_peak: f64) -> f64 {
    if t <= t_peak {
        ((std::f64::consts::PI * t) / (2.0 * t_peak)).sin()
    } else {
        ((std::f64::consts::PI * (1.0 - t)) / (2.0 * (1.0 - t_peak))).sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_light_scale_has_13_steps() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012", &CurveParams::default()).unwrap();
        assert_eq!(scale.len(), 13);
    }

    #[test]
    fn anchors_are_exact() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012", &CurveParams::default()).unwrap();
        assert_eq!(scale[0].to_uppercase(), "#FFFFFF");
        assert_eq!(scale[6].to_uppercase(), "#787880");
        assert_eq!(scale[12].to_uppercase(), "#101012");
    }

    #[test]
    fn lightness_is_monotonically_decreasing() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012", &CurveParams::default()).unwrap();
        let mut prev_jp = f64::MAX;
        for hex in &scale {
            let ucs = Cam16Ucs::from_hex(hex).unwrap();
            assert!(
                ucs.jp <= prev_jp + 1e-9,
                "lightness increased at {}: jp={} prev={}",
                hex,
                ucs.jp,
                prev_jp
            );
            prev_jp = ucs.jp;
        }
    }

    #[test]
    fn all_steps_are_unique() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012", &CurveParams::default()).unwrap();
        let mut seen = std::collections::HashSet::new();
        for hex in &scale {
            assert!(seen.insert(hex.to_uppercase()), "duplicate step: {}", hex);
        }
    }

    #[test]
    fn lightness_within_anchor_bounds() {
        let light = "#FFFFFF";
        let base = "#787880";
        let dark = "#101012";
        let scale = create_neutral_light_scale(light, base, dark, &CurveParams::default()).unwrap();

        let j0 = Cam16Ucs::from_hex(light).unwrap().jp;
        let j6 = Cam16Ucs::from_hex(base).unwrap().jp;
        let j12 = Cam16Ucs::from_hex(dark).unwrap().jp;

        assert!(j0 > j6, "light anchor should be lighter than base");
        assert!(j6 > j12, "base should be lighter than dark anchor");

        for (i, hex) in scale.iter().enumerate() {
            let j = Cam16Ucs::from_hex(hex).unwrap().jp;
            assert!(
                j <= j0 + 1e-9 && j >= j12 - 1e-9,
                "step {} ({}) out of bounds: jp={} not in [{}, {}]",
                i,
                hex,
                j,
                j12,
                j0
            );
        }
    }

    #[test]
    fn rejects_malformed_hex() {
        let err = create_neutral_light_scale("#GGGGGG", "#787880", "#101012", &CurveParams::default());
        assert!(err.is_err());
    }
}
