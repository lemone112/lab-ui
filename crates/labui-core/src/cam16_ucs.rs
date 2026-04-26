/**
 * CAM16-UCS color space implementation.
 *
 * CAM16 (CIECAM16) models human color appearance under specific
 * viewing conditions. UCS (Uniform Color Space) is a Euclidean
 * transform of CAM16 that makes J'M'h perceptually uniform.
 *
 * Formulas from Li et al. (2017):
 *   J' = 1.7 * J / (1 + 0.007 * J)
 *   M' = ln(1 + 0.0228 * M) / 0.0228
 *
 * This module uses palette crate for sRGB ↔ XYZ, then implements
 * CAM16 viewing conditions and transforms manually for precision.
 */
use crate::srgb::{D65_WHITE, hex_from_srgb, srgb_from_hex, srgb_to_xyz, xyz_to_srgb};

/// Standard sRGB viewing conditions for CAM16.
#[derive(Debug, Clone, Copy)]
pub struct ViewingConditions {
    /// Background luminance factor (Yb / Yw)
    pub n: f64,
    /// Achromatic response to white
    pub aw: f64,
    /// Chromatic induction factor
    pub nbb: f64,
    /// Chromatic induction factor (same as nbb in standard conditions)
    pub ncb: f64,
    /// Luminance-level adaptation factor
    pub fl: f64,
    /// Base exponential nonlinearity
    pub z: f64,
    /// Degree of chromatic adaptation
    pub c: f64,
    /// Chromatic induction factor (same as c in average surround)
    pub nc: f64,
    /// RGB discounting factors
    pub rgb_d: [f64; 3],
}

impl Default for ViewingConditions {
    fn default() -> Self {
        Self::srgb()
    }
}

impl ViewingConditions {
    /// Standard sRGB viewing conditions (D65, 20% gray background, average surround).
    pub fn srgb() -> Self {
        // Match colorjs.io cam16.js exactly:
        // environment(white, (64 / Math.PI) * 0.2, 20, "average", false)
        let la = (64.0_f64 / std::f64::consts::PI) * 0.2; // ≈ 4.074
        let y_b = 20.0; // Background Y (% of white)
        let surround = 1.0; // Average surround

        // colorjs.io surroundMap.average = [1, 0.69, 1]
        let c = 0.69_f64;
        let nc = 1.0_f64;

        let k = 1.0_f64 / (5.0 * la + 1.0);
        let k4 = k * k * k * k;
        let fl = k4 * la + 0.1_f64 * (1.0 - k4).powi(2) * (5.0 * la).cbrt();

        let n = y_b / 100.0_f64;
        let nbb = 0.725_f64 * n.powf(-0.2);
        let z = 1.48_f64 + n.sqrt();

        // D65 white point (normalized Y=100)
        let xyz_w = [
            D65_WHITE[0] * 100.0,
            D65_WHITE[1] * 100.0,
            D65_WHITE[2] * 100.0,
        ];
        let rgb_w = xyz_to_cone(xyz_w);
        let f = surround; // surroundMap.average[0]
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

// MCAT02 matrix: XYZ → cone responses (LMS)
const XYZ_TO_CONE: [[f64; 3]; 3] = [
    [0.401288, 0.650173, -0.051461],
    [-0.250268, 1.204414, 0.045854],
    [-0.002079, 0.048952, 0.953127],
];

// Inverse MCAT02: cone responses → XYZ
const CONE_TO_XYZ: [[f64; 3]; 3] = [
    [1.86206786, -1.01125463, 0.14918677],
    [0.38752654, 0.62144744, -0.00897398],
    [-0.01584150, -0.03412294, 1.04996444],
];

fn xyz_to_cone(xyz: [f64; 3]) -> [f64; 3] {
    mat_vec_mul(XYZ_TO_CONE, xyz)
}

fn cone_to_xyz(lms: [f64; 3]) -> [f64; 3] {
    mat_vec_mul(CONE_TO_XYZ, lms)
}

fn mat_vec_mul(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Nonlinear adaptation function.
fn adapt(c: f64, fl: f64) -> f64 {
    let x = fl * c.abs() / 100.0;
    let y = x.powf(0.42);
    c.signum() * 400.0 * y / (y + 27.13)
}

/// Inverse nonlinear adaptation.
fn unadapt(a: f64, fl: f64) -> f64 {
    let x = a.abs();
    let y = (27.13 * x / (400.0 - x)).max(0.0);
    a.signum() * 100.0 * y.powf(1.0 / 0.42) / fl
}

/// CAM16-UCS color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cam16Ucs {
    /// Uniform lightness J'
    pub jp: f64,
    /// Uniform a' (red-green)
    pub ap: f64,
    /// Uniform b' (yellow-blue)
    pub bp: f64,
}

impl Cam16Ucs {
    /// Create from sRGB hex.
    pub fn from_hex(hex: &str) -> Self {
        let rgb = srgb_from_hex(hex);
        let xyz = srgb_to_xyz(rgb);
        cam16_ucs_from_xyz(xyz, &ViewingConditions::srgb())
    }

    /// Convert back to sRGB hex.
    pub fn to_hex(&self) -> String {
        let xyz = cam16_ucs_to_xyz(*self, &ViewingConditions::srgb());
        let rgb = xyz_to_srgb(xyz);
        hex_from_srgb(rgb)
    }
}

/// XYZ → CAM16-UCS.
fn cam16_ucs_from_xyz(xyz: [f64; 3], vc: &ViewingConditions) -> Cam16Ucs {
    // colorjs.io scales XYZ to 0..100 before CAM16
    let xyz = [xyz[0] * 100.0, xyz[1] * 100.0, xyz[2] * 100.0];
    let lms = xyz_to_cone(xyz);
    let lms_a = [
        lms[0] * vc.rgb_d[0],
        lms[1] * vc.rgb_d[1],
        lms[2] * vc.rgb_d[2],
    ];
    let lms_aa = [
        adapt(lms_a[0], vc.fl),
        adapt(lms_a[1], vc.fl),
        adapt(lms_a[2], vc.fl),
    ];

    let a = lms_aa[0] - 12.0 * lms_aa[1] / 11.0 + lms_aa[2] / 11.0;
    let b = (lms_aa[0] + lms_aa[1] - 2.0 * lms_aa[2]) / 9.0;
    let h = b.atan2(a).to_degrees().rem_euclid(360.0);

    let hr = h.to_radians();
    let e_hue = 0.25 * ((hr + 2.0).cos() + 3.8);
    let a_achrom = (2.0 * lms_aa[0] + lms_aa[1] + lms_aa[2] / 20.0) * vc.nbb;
    let j = 100.0 * (a_achrom / vc.aw).powf(vc.c * vc.z);

    let u = (a * a + b * b).sqrt();
    let t = (50000.0 / 13.0) * e_hue * vc.nc * vc.nbb * u
        / (lms_aa[0] + lms_aa[1] + 1.05 * lms_aa[2] + 0.305);
    let m = t.powf(0.9)
        * (j / 100.0).sqrt()
        * (1.64 - 0.29_f64.powf(vc.n)).powf(0.73)
        * vc.fl.powf(0.25);

    // UCS transform
    let jp = 1.7 * j / (1.0 + 0.007 * j);
    let mp = (1.0 + 0.0228 * m).ln() / 0.0228;

    Cam16Ucs {
        jp,
        ap: mp * hr.cos(),
        bp: mp * hr.sin(),
    }
}

/// CAM16-UCS → XYZ.
fn cam16_ucs_to_xyz(ucs: Cam16Ucs, vc: &ViewingConditions) -> [f64; 3] {
    let j = ucs.jp / (1.7 - 0.007 * ucs.jp);
    let mp = (ucs.ap * ucs.ap + ucs.bp * ucs.bp).sqrt();
    let m = (0.0228 * mp).exp_m1() / 0.0228;
    let h = ucs.bp.atan2(ucs.ap);

    let hr = h;
    let e_hue = 0.25 * ((hr + 2.0).cos() + 3.8);
    let t_inner = (1.64 - 0.29_f64.powf(vc.n)).powf(0.73);
    let t = (m / ((j / 100.0).sqrt() * t_inner * vc.fl.powf(0.25))).powf(1.0 / 0.9);

    let p1 = e_hue * (50000.0 / 13.0) * vc.nc * vc.nbb;
    let p2 = (vc.aw * (j / 100.0).powf(1.0 / (vc.c * vc.z))) / vc.nbb;
    let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * hr.cos() + 108.0 * t * hr.sin());

    let a = gamma * hr.cos();
    let b = gamma * hr.sin();

    let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
    let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
    let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;

    let r_c = unadapt(r_a, vc.fl);
    let g_c = unadapt(g_a, vc.fl);
    let b_c = unadapt(b_a, vc.fl);

    let lms = [r_c / vc.rgb_d[0], g_c / vc.rgb_d[1], b_c / vc.rgb_d[2]];

    let xyz = cone_to_xyz(lms);
    // colorjs.io scales back to 0..1
    [xyz[0] / 100.0, xyz[1] / 100.0, xyz[2] / 100.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_neutral_base() {
        let original = "#787880";
        let ucs = Cam16Ucs::from_hex(original);
        let back = ucs.to_hex();
        assert!(
            back.eq_ignore_ascii_case(original),
            "roundtrip drift: expected {}, got {}",
            original,
            back
        );
    }

    #[test]
    fn roundtrip_white() {
        let original = "#FFFFFF";
        let ucs = Cam16Ucs::from_hex(original);
        let back = ucs.to_hex();
        assert!(
            back.eq_ignore_ascii_case(original),
            "roundtrip drift: expected {}, got {}",
            original,
            back
        );
    }

    #[test]
    fn roundtrip_dark() {
        let original = "#101012";
        let ucs = Cam16Ucs::from_hex(original);
        let back = ucs.to_hex();
        assert!(
            back.eq_ignore_ascii_case(original),
            "roundtrip drift: expected {}, got {}",
            original,
            back
        );
    }
}
