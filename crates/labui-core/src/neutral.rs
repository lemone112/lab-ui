use crate::cam16_ucs::Cam16Ucs;

/// Lightness ease exponent.
const P_J: f64 = 1.7;

/// Hue ease exponent (fast jump from white hue to base hue).
const P_H: f64 = 0.6;

/// Sinusoid peak position (normalized 0..1).
const T_PEAK: f64 = 0.35;

/// Chroma overshoot above base (20 %).
const CHROMA_BOOST: f64 = 1.2;

/// Power ease-in (0 → 1).
fn ease_in(t: f64, p: f64) -> f64 {
    t.powf(p)
}

/// Power ease-out (1 → 0).
fn ease_out(t: f64, p: f64) -> f64 {
    1.0 - (1.0 - t).powf(p)
}

/// Sinusoidal envelope, peak at `t_peak`.
fn sine_env(t: f64, t_peak: f64) -> f64 {
    if t <= t_peak {
        ((std::f64::consts::PI * t) / (2.0 * t_peak)).sin()
    } else {
        ((std::f64::consts::PI * (1.0 - t)) / (2.0 * (1.0 - t_peak))).sin()
    }
}

/// Generate a 13-step neutral scale for light mode.
pub fn create_neutral_light_scale(light: &str, base: &str, dark: &str) -> Vec<String> {
    let a0 = Cam16Ucs::from_hex(light);
    let a6 = Cam16Ucs::from_hex(base);
    let a12 = Cam16Ucs::from_hex(dark);

    let c_base = (a6.ap * a6.ap + a6.bp * a6.bp).sqrt();
    let c_dark = (a12.ap * a12.ap + a12.bp * a12.bp).sqrt();
    let c_peak = c_base * CHROMA_BOOST;

    let base_hue = a6.bp.atan2(a6.ap);

    (0..=12)
        .map(|i| {
            // Pin anchors exactly to avoid roundtrip drift
            match i {
                0 => return light.to_string(),
                6 => return base.to_string(),
                12 => return dark.to_string(),
                _ => {}
            }

            let t = i as f64 / 12.0;

            // ----- J' (lightness) -----
            let jp = if i < 6 {
                let u = i as f64 / 6.0;
                a0.jp + (a6.jp - a0.jp) * ease_in(u, P_J)
            } else {
                let u = (i - 6) as f64 / 6.0;
                a6.jp + (a12.jp - a6.jp) * ease_out(u, P_J)
            };

            // ----- M' (chroma) -----
            let env = sine_env(t, T_PEAK);
            let mp = c_dark + (c_peak - c_dark) * env;

            // ----- h (hue) -----
            let hue = if i < 6 {
                let u = i as f64 / 6.0;
                let start_hue = a0.bp.atan2(a0.ap);
                start_hue + (base_hue - start_hue) * ease_in(u, P_H)
            } else {
                let u = (i - 6) as f64 / 6.0;
                base_hue + (a12.bp.atan2(a12.ap) - base_hue) * u
            };

            Cam16Ucs {
                jp,
                ap: mp * hue.cos(),
                bp: mp * hue.sin(),
            }
            .to_hex()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_light_scale_has_13_steps() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012");
        assert_eq!(scale.len(), 13);
    }

    #[test]
    fn anchors_are_exact() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012");
        assert_eq!(scale[0].to_uppercase(), "#FFFFFF");
        assert_eq!(scale[6].to_uppercase(), "#787880");
        assert_eq!(scale[12].to_uppercase(), "#101012");
    }

    /// Lightness must decrease monotonically from light to dark.
    #[test]
    fn lightness_is_monotonically_decreasing() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012");
        let mut prev_jp = f64::MAX;
        for hex in &scale {
            let ucs = Cam16Ucs::from_hex(hex);
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

    /// All 13 steps must be unique (for non-trivial anchors).
    #[test]
    fn all_steps_are_unique() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012");
        let mut seen = std::collections::HashSet::new();
        for hex in &scale {
            assert!(seen.insert(hex.to_uppercase()), "duplicate step: {}", hex);
        }
    }

    /// Lightness must be clamped between anchors (no overshoot).
    #[test]
    fn lightness_within_anchor_bounds() {
        let light = "#FFFFFF";
        let base = "#787880";
        let dark = "#101012";
        let scale = create_neutral_light_scale(light, base, dark);

        let j0 = Cam16Ucs::from_hex(light).jp;
        let j6 = Cam16Ucs::from_hex(base).jp;
        let j12 = Cam16Ucs::from_hex(dark).jp;

        assert!(j0 > j6, "light anchor should be lighter than base");
        assert!(j6 > j12, "base should be lighter than dark anchor");

        for (i, hex) in scale.iter().enumerate() {
            let j = Cam16Ucs::from_hex(hex).jp;
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
}
