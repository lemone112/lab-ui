/**
 * Exact sRGB <-> XYZ(D65) transforms matching colorjs.io matrices.
 *
 * Matrices from https://github.com/w3c/csswg-drafts/issues/5922
 * as used in colorjs.io src/spaces/srgb-linear.js
 */

/// D65 white point (normalized Y=1).
pub const D65_WHITE: [f64; 3] = [0.9504559270516716, 1.0000000000000000, 1.0890577507598784];

// linear sRGB -> XYZ (colorjs.io exact)
const TO_XYZ_M: [[f64; 3]; 3] = [
    [0.41239079926595934, 0.357584339383878, 0.1804807884018343],
    [0.21263900587151027, 0.715168678767756, 0.07219231536073371],
    [0.01933081871559182, 0.11919477979462598, 0.9505321522496607],
];

// XYZ -> linear sRGB (colorjs.io exact)
const FROM_XYZ_M: [[f64; 3]; 3] = [
    [3.2409699419045226, -1.537383177570094, -0.4986107602930034],
    [-0.9692436362808796, 1.8759675015077202, 0.04155505740717559],
    [
        0.05563007969699366,
        -0.20397695888897652,
        1.0569715142428786,
    ],
];

fn mat_vec_mul(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Decode sRGB gamma -> linear light.
fn srgb_gamma_inv(v: f64) -> f64 {
    let sign = if v < 0.0 { -1.0 } else { 1.0 };
    let abs = v * sign;
    if abs <= 0.04045 {
        v / 12.92
    } else {
        sign * ((abs + 0.055) / 1.055).powf(2.4)
    }
}

/// Encode linear light -> sRGB gamma.
fn srgb_gamma(v: f64) -> f64 {
    let sign = if v < 0.0 { -1.0 } else { 1.0 };
    let abs = v * sign;
    if abs > 0.0031308 {
        sign * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
    } else {
        12.92 * v
    }
}

/// Parse #RRGGBB -> linear sRGB [0..1].
pub fn srgb_from_hex(hex: &str) -> [f64; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f64 / 255.0;
    [srgb_gamma_inv(r), srgb_gamma_inv(g), srgb_gamma_inv(b)]
}

/// Linear sRGB [0..1] -> #RRGGBB (clamped & rounded).
pub fn hex_from_srgb(rgb: [f64; 3]) -> String {
    let r = (srgb_gamma(rgb[0]).clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (srgb_gamma(rgb[1]).clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (srgb_gamma(rgb[2]).clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// linear sRGB -> XYZ(D65).
pub fn srgb_to_xyz(rgb: [f64; 3]) -> [f64; 3] {
    mat_vec_mul(TO_XYZ_M, rgb)
}

/// XYZ(D65) -> linear sRGB.
pub fn xyz_to_srgb(xyz: [f64; 3]) -> [f64; 3] {
    mat_vec_mul(FROM_XYZ_M, xyz)
}
