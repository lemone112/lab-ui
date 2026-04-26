use crate::Cam16Ucs;

/// Perceptually-uniform mix of two sRGB hex colours in CAM16-UCS.
///
/// `strength` is in the range `0.0..=1.0` where:
/// - `0.0` → exactly `bg_hex`
/// - `1.0` → exactly `fg_hex`
/// - `0.5` → perceptual midpoint
///
/// The interpolation is done linearly in CAM16-UCS (`J'`, `a'`, `b'`),
/// which preserves hue and lightness relationships far better than
/// naive sRGB alpha compositing.
pub fn perceptual_mix(fg_hex: &str, bg_hex: &str, strength: f64) -> Result<String, String> {
    if !(0.0..=1.0).contains(&strength) {
        return Err(format!(
            "mix strength must be in 0.0..=1.0, got {}",
            strength
        ));
    }

    let fg = Cam16Ucs::from_hex(fg_hex)?;
    let bg = Cam16Ucs::from_hex(bg_hex)?;

    let mixed = Cam16Ucs {
        jp: bg.jp + (fg.jp - bg.jp) * strength,
        ap: bg.ap + (fg.ap - bg.ap) * strength,
        bp: bg.bp + (fg.bp - bg.bp) * strength,
    };

    Ok(mixed.to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_zero_is_bg() {
        let got = perceptual_mix("#FF0000", "#FFFFFF", 0.0).unwrap();
        assert_eq!(got.to_ascii_uppercase(), "#FFFFFF");
    }

    #[test]
    fn mix_one_is_fg() {
        let got = perceptual_mix("#FF0000", "#FFFFFF", 1.0).unwrap();
        assert_eq!(got.to_ascii_uppercase(), "#FF0000");
    }

    #[test]
    fn mix_midpoint_not_srgb() {
        let got = perceptual_mix("#FF0000", "#0000FF", 0.5).unwrap();
        // CAM16-UCS midpoint is NOT #800080 (the sRGB midpoint)
        assert_ne!(got.to_ascii_uppercase(), "#800080");
    }

    #[test]
    fn mix_rejects_oob() {
        assert!(perceptual_mix("#FF0000", "#FFFFFF", 1.5).is_err());
        assert!(perceptual_mix("#FF0000", "#FFFFFF", -0.1).is_err());
    }
}
