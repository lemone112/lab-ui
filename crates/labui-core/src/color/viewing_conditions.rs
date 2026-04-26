use crate::srgb::D65_WHITE;

use super::{cam16::adapt, cat16::xyz_to_cone};

/// Viewing conditions for the CIECAM16 colour appearance model.
///
/// Defaults match the sRGB standard (D65, 20 % grey background,
/// average surround, no discounting).
#[derive(Debug, Clone, Copy)]
pub struct ViewingConditions {
    /// Background luminance factor (Yb / Yw).
    pub n: f64,
    /// Achromatic response to the reference white.
    pub aw: f64,
    /// Chromatic induction factor.
    pub nbb: f64,
    pub ncb: f64,
    /// Luminance-level adaptation factor.
    pub fl: f64,
    /// Base exponential nonlinearity.
    pub z: f64,
    /// Degree of chromatic adaptation.
    pub c: f64,
    /// Chromatic induction factor.
    pub nc: f64,
    /// RGB discounting factors.
    pub rgb_d: [f64; 3],
}

impl Default for ViewingConditions {
    fn default() -> Self {
        Self::srgb()
    }
}

impl ViewingConditions {
    /// Standard sRGB viewing conditions.
    ///
    /// Matches the defaults used by colorjs.io:
    /// `environment(white, (64/π)*0.2, 20, "average", false)`.
    pub fn srgb() -> Self {
        let la = (64.0_f64 / std::f64::consts::PI) * 0.2; // ≈ 4.074 cd/m²
        let y_b = 20.0;
        let surround = 1.0; // average

        // colour-science / colorjs.io surroundMap["average"] = [1.0, 0.69, 1.0]
        let c = 0.69_f64;
        let nc = 1.0_f64;

        let k = 1.0_f64 / (5.0 * la + 1.0);
        let k4 = k * k * k * k;
        let fl = k4 * la + 0.1_f64 * (1.0 - k4).powi(2) * (5.0 * la).cbrt();

        let n = y_b / 100.0_f64;
        let nbb = 0.725_f64 * n.powf(-0.2);
        let z = 1.48_f64 + n.sqrt();

        let xyz_w = [
            D65_WHITE[0] * 100.0,
            D65_WHITE[1] * 100.0,
            D65_WHITE[2] * 100.0,
        ];
        let rgb_w = xyz_to_cone(xyz_w);
        let f = surround;
        let d = (f * (1.0 - (1.0 / 3.6) * ((-la - 42.0) / 92.0).exp()))
            .max(0.0)
            .min(1.0);
        let rgb_d = [
            d * (100.0 / rgb_w[0]) + 1.0 - d,
            d * (100.0 / rgb_w[1]) + 1.0 - d,
            d * (100.0 / rgb_w[2]) + 1.0 - d,
        ];

        let rgb_w_adapted = [
            rgb_w[0] * rgb_d[0],
            rgb_w[1] * rgb_d[1],
            rgb_w[2] * rgb_d[2],
        ];
        let rgb_aw = [
            adapt(rgb_w_adapted[0], fl),
            adapt(rgb_w_adapted[1], fl),
            adapt(rgb_w_adapted[2], fl),
        ];
        let aw = (2.0 * rgb_aw[0] + rgb_aw[1] + rgb_aw[2] / 20.0) * nbb;

        Self {
            n,
            aw,
            nbb,
            ncb: nbb,
            fl,
            z,
            c,
            nc,
            rgb_d,
        }
    }
}
