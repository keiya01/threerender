use threerender_color::rgb::{RGB, RGBA};

fn u8_to_color(v: u8) -> f32 {
    v as f32 / 255.
}

fn u8_to_color_64(v: u8) -> f64 {
    v as f64 / 255.
}

pub(super) fn rgb_to_array(rgb: &RGB) -> [f32; 3] {
    let RGB { r, g, b } = rgb;
    [u8_to_color(*r), u8_to_color(*g), u8_to_color(*b)]
}

pub(super) fn rgba_to_array(rgba: &RGBA) -> [f32; 4] {
    let RGBA { r, g, b, a } = rgba;
    [
        u8_to_color(*r),
        u8_to_color(*g),
        u8_to_color(*b),
        u8_to_color(*a),
    ]
}

pub(super) fn rgba_to_array_64(rgba: &RGBA) -> [f64; 4] {
    let RGBA { r, g, b, a } = rgba;
    [
        u8_to_color_64(*r),
        u8_to_color_64(*g),
        u8_to_color_64(*b),
        u8_to_color_64(*a),
    ]
}
