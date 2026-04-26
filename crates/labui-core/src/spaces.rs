use palette::{FromColor, IntoColor, Oklab, Srgb};

/// Perceptual color in Oklab space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklabColor {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl OklabColor {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        let srgb = Srgb::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        );
        let oklab: Oklab = srgb.into_color();
        Self {
            l: oklab.l,
            a: oklab.a,
            b: oklab.b,
        }
    }

    pub fn to_hex(&self) -> String {
        let oklab = Oklab::new(self.l, self.a, self.b);
        let srgb: Srgb = oklab.into_color();
        format!(
            "#{:02X}{:02X}{:02X}",
            (srgb.red * 255.0).round() as u8,
            (srgb.green * 255.0).round() as u8,
            (srgb.blue * 255.0).round() as u8,
        )
    }
}

/// Power ease-in (0 → 1).
pub fn ease_in(t: f32, p: f32) -> f32 {
    t.powf(p)
}

/// Power ease-out (1 → 0).
pub fn ease_out(t: f32, p: f32) -> f32 {
    1.0 - (1.0 - t).powf(p)
}
