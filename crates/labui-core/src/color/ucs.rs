use crate::srgb::{hex_from_srgb, srgb_from_hex, srgb_to_xyz, xyz_to_srgb};

use super::{cam16, viewing_conditions::ViewingConditions};

/// CAM16-UCS perceptually-uniform colour.
///
/// Coordinates:
/// - `jp` — uniform lightness  J'
/// - `ap` — uniform red–green  a'
/// - `bp` — uniform yellow–blue b'
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cam16Ucs {
    pub jp: f64,
    pub ap: f64,
    pub bp: f64,
}

impl Cam16Ucs {
    /// Create from an sRGB hex string (`#RRGGBB`).
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let rgb = srgb_from_hex(hex)?;
        let xyz = srgb_to_xyz(rgb);
        Ok(Self::from_xyz(xyz, &ViewingConditions::srgb()))
    }

    /// Convert back to an sRGB hex string.
    pub fn to_hex(&self) -> String {
        let xyz = self.to_xyz(&ViewingConditions::srgb());
        let rgb = xyz_to_srgb(xyz);
        hex_from_srgb(rgb)
    }

    /// CIE XYZ(D65) → CAM16-UCS.
    pub(crate) fn from_xyz(xyz: [f64; 3], vc: &ViewingConditions) -> Self {
        // CAM16 operates on XYZ scaled to Yw = 100.
        let xyz = [xyz[0] * 100.0, xyz[1] * 100.0, xyz[2] * 100.0];

        let lms = super::cat16::xyz_to_cone(xyz);
        let lms_a = [
            lms[0] * vc.rgb_d[0],
            lms[1] * vc.rgb_d[1],
            lms[2] * vc.rgb_d[2],
        ];
        let lms_aa = [
            cam16::adapt(lms_a[0], vc.fl),
            cam16::adapt(lms_a[1], vc.fl),
            cam16::adapt(lms_a[2], vc.fl),
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

        // UCS nonlinearity (Li et al. 2017).
        let jp = 1.7 * j / (1.0 + 0.007 * j);
        let mp = (1.0 + 0.0228 * m).ln() / 0.0228;

        Self {
            jp,
            ap: mp * hr.cos(),
            bp: mp * hr.sin(),
        }
    }

    /// CAM16-UCS → CIE XYZ(D65).
    pub(crate) fn to_xyz(&self, vc: &ViewingConditions) -> [f64; 3] {
        let j = self.jp / (1.7 - 0.007 * self.jp);
        let mp = (self.ap * self.ap + self.bp * self.bp).sqrt();
        let m = (0.0228 * mp).exp_m1() / 0.0228;
        let h = self.bp.atan2(self.ap);
        let hr = h;

        let e_hue = 0.25 * ((hr + 2.0).cos() + 3.8);
        let t_inner = (1.64 - 0.29_f64.powf(vc.n)).powf(0.73);
        let t = (m / ((j / 100.0).sqrt() * t_inner * vc.fl.powf(0.25))).powf(1.0 / 0.9);

        let p1 = e_hue * (50000.0 / 13.0) * vc.nc * vc.nbb;
        let p2 = (vc.aw * (j / 100.0).powf(1.0 / (vc.c * vc.z))) / vc.nbb;
        let gamma = 23.0 * (p2 + 0.305) * t
            / (23.0 * p1 + 11.0 * t * hr.cos() + 108.0 * t * hr.sin());

        let a = gamma * hr.cos();
        let b = gamma * hr.sin();

        let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
        let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
        let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;

        let r_c = cam16::unadapt(r_a, vc.fl);
        let g_c = cam16::unadapt(g_a, vc.fl);
        let b_c = cam16::unadapt(b_a, vc.fl);

        let lms = [r_c / vc.rgb_d[0], g_c / vc.rgb_d[1], b_c / vc.rgb_d[2]];
        let xyz = super::cat16::cone_to_xyz(lms);

        // Scale back to Yw = 1.0.
        [xyz[0] / 100.0, xyz[1] / 100.0, xyz[2] / 100.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_neutral_base() {
        let original = "#787880";
        let ucs = Cam16Ucs::from_hex(original).unwrap();
        let back = ucs.to_hex();
        assert!(
            back.eq_ignore_ascii_case(original),
            "roundtrip drift: expected {original}, got {back}"
        );
    }

    #[test]
    fn roundtrip_white() {
        let original = "#FFFFFF";
        let ucs = Cam16Ucs::from_hex(original).unwrap();
        let back = ucs.to_hex();
        assert!(
            back.eq_ignore_ascii_case(original),
            "roundtrip drift: expected {original}, got {back}"
        );
    }

    #[test]
    fn roundtrip_dark() {
        let original = "#101012";
        let ucs = Cam16Ucs::from_hex(original).unwrap();
        let back = ucs.to_hex();
        assert!(
            back.eq_ignore_ascii_case(original),
            "roundtrip drift: expected {original}, got {back}"
        );
    }

    #[test]
    fn from_hex_rejects_short_string() {
        assert!(Cam16Ucs::from_hex("#fff").is_err());
    }
}
