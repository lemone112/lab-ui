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

    /// Full parity test against TypeScript golden master (neutral.ts).
    #[test]
    fn light_scale_matches_ts_reference() {
        let scale = create_neutral_light_scale("#FFFFFF", "#787880", "#101012");
        let expected = [
            "#FFFFFF", "#F6F8FA", "#E4E7ED", "#CDD0D9", "#B3B5BF", "#9698A2", "#787880", "#5B5C64",
            "#44444B", "#303136", "#212125", "#151518", "#101012",
        ];
        for (i, (got, want)) in scale.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got.to_uppercase(),
                *want,
                "step {} mismatch: got {} expected {}",
                i,
                got,
                want
            );
        }
    }

    #[test]
    fn light_scale_matches_ts_reference_alternate_anchors() {
        let scale = create_neutral_light_scale("#F2F2F5", "#73737C", "#141416");
        let expected = [
            "#F2F2F5", "#EBEBF0", "#DADBE3", "#C4C6D1", "#ABADB9", "#90919C", "#73737C", "#595963",
            "#43444C", "#313238", "#232328", "#19191C", "#141416",
        ];
        for (i, (got, want)) in scale.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got.to_uppercase(),
                *want,
                "step {} mismatch: got {} expected {}",
                i,
                got,
                want
            );
        }
    }
}
