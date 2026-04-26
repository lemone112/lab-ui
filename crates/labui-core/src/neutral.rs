use crate::spaces::{ease_in, ease_out, OklabColor};

/// Lightness ease exponent.
const P_L: f32 = 1.53;

/// Chroma/a/b ease exponent for light half.
const P_C: f32 = 1.0;

/// Generate a 13-step neutral scale for light mode.
pub fn create_neutral_light_scale(light: &str, base: &str, dark: &str) -> Vec<String> {
    let a0 = OklabColor::from_hex(light);
    let a6 = OklabColor::from_hex(base);
    let a12 = OklabColor::from_hex(dark);

    (0..=12)
        .map(|i| {
            let t = i as f32 / 12.0;

            let l = if i < 6 {
                let u = i as f32 / 6.0;
                a0.l + (a6.l - a0.l) * ease_in(u, P_L)
            } else if i == 6 {
                a6.l
            } else {
                let u = (i - 6) as f32 / 6.0;
                a6.l + (a12.l - a6.l) * ease_out(u, P_L)
            };

            let a = if i < 6 {
                let u = i as f32 / 6.0;
                a0.a + (a6.a - a0.a) * ease_in(u, P_C)
            } else if i == 6 {
                a6.a
            } else {
                let u = (i - 6) as f32 / 6.0;
                a6.a + (a12.a - a6.a) * u
            };

            let b = if i < 6 {
                let u = i as f32 / 6.0;
                a0.b + (a6.b - a0.b) * ease_in(u, P_C)
            } else if i == 6 {
                a6.b
            } else {
                let u = (i - 6) as f32 / 6.0;
                a6.b + (a12.b - a6.b) * u
            };

            OklabColor { l, a, b }.to_hex()
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
}
