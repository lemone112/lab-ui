use crate::color::ucs::Cam16Ucs;
use crate::color::viewing_conditions::ViewingConditions;
use crate::srgb::{srgb_from_hex, xyz_to_srgb};

// ------------------------------------------------------------------
// APCA constants (0.98G-4g)
// ------------------------------------------------------------------
const NORM_BG: f64 = 0.56;
const NORM_TXT: f64 = 0.57;
const REV_TXT: f64 = 0.62;
const REV_BG: f64 = 0.65;
const BLK_THRS: f64 = 0.022;
const BLK_CLMP: f64 = 1.414;
const SCALE_BOW: f64 = 1.14;
const SCALE_WOB: f64 = 1.14;
const LO_BOW_THRESH: f64 = 0.035_991;
const LO_BOW_FACTOR: f64 = 27.784_723_958_767_5;
const LO_BOW_OFFSET: f64 = 0.027;
const LO_CLIP: f64 = 0.001;
const DELTA_Y_MIN: f64 = 0.000_5;

/// Convert an sRGB hex string to relative luminance `Y`.
///
/// Uses the official sRGB → linear transfer with a 2.4 exponent,
/// matching the APCA reference implementation.
pub fn srgb_hex_to_y(hex: &str) -> Result<f64, String> {
    let rgb = srgb_from_hex(hex)?;
    Ok(0.212_672_9 * rgb[0] + 0.715_152_2 * rgb[1] + 0.072_175_0 * rgb[2])
}

/// APCA forward contrast: return `Lc` for a text/background pair.
///
/// `txt_y` and `bg_y` are relative luminances in `[0, 1]`.
/// Positive result  → black text on white-ish background (BoW).
/// Negative result  → white text on dark background (WoB).
pub fn apca_contrast(txt_y: f64, bg_y: f64) -> f64 {
    let txt_y = if txt_y > BLK_THRS {
        txt_y
    } else {
        txt_y + (BLK_THRS - txt_y).powf(BLK_CLMP)
    };
    let bg_y = if bg_y > BLK_THRS {
        bg_y
    } else {
        bg_y + (BLK_THRS - bg_y).powf(BLK_CLMP)
    };

    if (bg_y - txt_y).abs() < DELTA_Y_MIN {
        return 0.0;
    }

    let sapc: f64;
    let output: f64;

    if bg_y > txt_y {
        // Black on White
        sapc = (bg_y.powf(NORM_BG) - txt_y.powf(NORM_TXT)) * SCALE_BOW;
        output = if sapc < LO_CLIP {
            0.0
        } else if sapc < LO_BOW_THRESH {
            sapc - sapc * LO_BOW_FACTOR * LO_BOW_OFFSET
        } else {
            sapc - LO_BOW_OFFSET
        };
    } else {
        // White on Black
        sapc = (bg_y.powf(REV_BG) - txt_y.powf(REV_TXT)) * SCALE_WOB;
        output = if sapc > -LO_CLIP {
            0.0
        } else if sapc > -LO_BOW_THRESH {
            sapc - sapc * LO_BOW_FACTOR * LO_BOW_OFFSET
        } else {
            sapc + LO_BOW_OFFSET
        };
    }

    output * 100.0
}

/// APCA inverse: find a CAM16-UCS colour with the same hue/chroma as
/// `canonical` that yields `target_lc` against `bg_y`.
///
/// Searches via binary search on `J'` (lightness).  If the exact
/// colour falls outside the sRGB gamut, chroma is reduced with
/// [`find_closest_in_gamut`].
pub fn apca_inverse(bg_y: f64, target_lc: f64, canonical: &Cam16Ucs) -> Option<Cam16Ucs> {
    let mp = (canonical.ap * canonical.ap + canonical.bp * canonical.bp).sqrt();
    let hr = canonical.bp.atan2(canonical.ap);

    let compute_lc = |jp: f64| -> f64 {
        let candidate = Cam16Ucs {
            jp,
            ap: mp * hr.cos(),
            bp: mp * hr.sin(),
        };
        let y = candidate.to_xyz(&ViewingConditions::srgb())[1];
        apca_contrast(y, bg_y)
    };

    let lc_lo = compute_lc(0.0);
    let lc_hi = compute_lc(100.0);

    // Light theme: target positive, max at J'=0.
    // Dark theme: target negative, min at J'=100.
    if target_lc > 0.0 && lc_lo < target_lc {
        return None;
    }
    if target_lc < 0.0 && lc_hi > target_lc {
        return None;
    }

    let mut lo = 0.0;
    let mut hi = 100.0;
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        let lc = compute_lc(mid);
        if lc > target_lc {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let result = Cam16Ucs {
        jp: (lo + hi) / 2.0,
        ap: mp * hr.cos(),
        bp: mp * hr.sin(),
    };

    Some(find_closest_in_gamut(result))
}

/// Return `true` if the UCS colour maps inside the sRGB cube.
fn is_in_gamut(ucs: &Cam16Ucs) -> bool {
    let rgb = xyz_to_srgb(ucs.to_xyz(&ViewingConditions::srgb()));
    rgb.iter().all(|&c| (0.0..=1.0).contains(&c))
}

/// If `ucs` is outside the sRGB gamut, reduce chroma (`M'`) while
/// preserving hue and lightness until it fits.
///
/// Uses binary search on `M'` for f64 precision.
pub fn find_closest_in_gamut(ucs: Cam16Ucs) -> Cam16Ucs {
    if is_in_gamut(&ucs) {
        return ucs;
    }

    let mp = (ucs.ap * ucs.ap + ucs.bp * ucs.bp).sqrt();
    let hr = ucs.bp.atan2(ucs.ap);

    let mut lo = 0.0; // in-gamut (achromatic is always safe)
    let mut hi = mp;  // out-of-gamut by definition here

    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        let candidate = Cam16Ucs {
            jp: ucs.jp,
            ap: mid * hr.cos(),
            bp: mid * hr.sin(),
        };
        if is_in_gamut(&candidate) {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    Cam16Ucs {
        jp: ucs.jp,
        ap: lo * hr.cos(),
        bp: lo * hr.sin(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_on_black_is_most_negative() {
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();
        let y_black = srgb_hex_to_y("#000000").unwrap();
        let lc = apca_contrast(y_white, y_black);
        assert!(lc < -100.0, "white on black should be < -100, got {}", lc);
    }

    #[test]
    fn black_on_white_is_most_positive() {
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();
        let y_black = srgb_hex_to_y("#000000").unwrap();
        let lc = apca_contrast(y_black, y_white);
        assert!(lc > 100.0, "black on white should be > 100, got {}", lc);
    }

    #[test]
    fn same_color_is_zero() {
        let y = srgb_hex_to_y("#787880").unwrap();
        assert_eq!(apca_contrast(y, y), 0.0);
    }

    #[test]
    fn brand_on_white_known_value() {
        let y_brand = srgb_hex_to_y("#007AFF").unwrap();
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();
        let lc = apca_contrast(y_brand, y_white);
        // From manual calculation: ~64
        assert!((lc - 64.0).abs() < 2.0, "Brand on white Lc expected ~64, got {}", lc);
    }

    #[test]
    fn inverse_roundtrip_on_white() {
        let canonical = Cam16Ucs::from_hex("#007AFF").unwrap();
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();
        let y_canonical = srgb_hex_to_y("#007AFF").unwrap();
        let target = apca_contrast(y_canonical, y_white);

        let result = apca_inverse(y_white, target, &canonical).unwrap();
        let y_result = result.to_xyz(&ViewingConditions::srgb())[1];
        let lc_result = apca_contrast(y_result, y_white);

        assert!((lc_result - target).abs() < 0.1, "inverse roundtrip failed: {} vs {}", lc_result, target);
    }

    #[test]
    fn inverse_finds_darker_for_positive_lc() {
        let canonical = Cam16Ucs::from_hex("#007AFF").unwrap();
        let y_white = srgb_hex_to_y("#FFFFFF").unwrap();

        let result = apca_inverse(y_white, 75.0, &canonical).unwrap();
        let y_result = result.to_xyz(&ViewingConditions::srgb())[1];
        let lc_result = apca_contrast(y_result, y_white);

        assert!(lc_result > 70.0, "expected Lc > 70, got {}", lc_result);
        // Darker than canonical
        let y_canonical = srgb_hex_to_y("#007AFF").unwrap();
        assert!(y_result < y_canonical, "inverse should be darker, Y {} vs {}", y_result, y_canonical);
    }

    #[test]
    fn inverse_finds_lighter_for_negative_lc() {
        let canonical = Cam16Ucs::from_hex("#007AFF").unwrap();
        let y_black = srgb_hex_to_y("#000000").unwrap();

        let result = apca_inverse(y_black, -60.0, &canonical).unwrap();
        let y_result = result.to_xyz(&ViewingConditions::srgb())[1];
        let lc_result = apca_contrast(y_result, y_black);

        assert!(lc_result < -55.0, "expected Lc < -55, got {}", lc_result);
        // Lighter than canonical
        let y_canonical = srgb_hex_to_y("#007AFF").unwrap();
        assert!(y_result > y_canonical, "inverse should be lighter, Y {} vs {}", y_result, y_canonical);
    }
}
