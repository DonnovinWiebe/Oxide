use std::cmp::min;
use image::{Pixel, Rgb};



pub fn is_hex(code: String) -> bool {
    let code = code.trim_start_matches('#');
    (code.len() == 3 || code.len() == 6 || code.len() == 8) && code.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn as_rgb(hex: String) -> Option<Rgb<u8>> {
    let hex = hex.trim_start_matches('#');

    let (r, g, b, a) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            (r, g, b, 255)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };

    Some(Rgb([r, g, b]))
}

pub fn get_spectrum(color: Rgb<u8>, steps: usize) -> Vec<Rgb<u8>> {
    // the interpolation information
    let mut base_color = (color[0] as f64, color[1] as f64, color[2] as f64);
    let (r_c, g_c, b_c) = &mut base_color;

    let white = (255.0, 255.0, 255.0);
    let (r_w, g_w, b_w) = &white;

    let black = (0.0, 0.0, 0.0);
    let (r_b, g_b, b_b) = &black;

    let white_increment = (r_w - *r_c, g_w - *g_c, b_w - *b_c);
    let (r_wi, g_wi, b_wi) = &white_increment;

    let black_increment = (r_b - *r_c, g_b - *g_c, b_b - *b_c);
    let (r_bi, g_bi, b_bi) = &black_increment;


    // the interpolated spectrum
    let mut interpolated_spectrum = Vec::new();
    interpolated_spectrum.push((r_c.clone(), g_c.clone(), b_c.clone()));


    // builds the interpolation spectrum from the base color to white
    let (mut r, mut g, mut b) = base_color.clone();
    for _ in 0..steps {
        r += r_wi;
        g += g_wi;
        b += b_wi;
        interpolated_spectrum.push((r.clone(), g.clone(), b.clone()));
    }


    // builds the interpolation spectrum from the base color to black
    let (mut r, mut g, mut b) = base_color.clone();
    for _ in 0..steps {
        r += r_bi;
        g += g_bi;
        b += b_bi;
        interpolated_spectrum.push((r.clone(), g.clone(), b.clone()));
    }


    // turns the interpolation spectrum to an Rgb spectrum
    let mut spectrum = Vec::new();
    for interpolated_color in interpolated_spectrum.iter_mut() {
        let (r_i, g_i, b_i) = interpolated_color;
        let new_color = Rgb([r_i.round() as u8, g_i.round() as u8, b_i.round() as u8]);
        spectrum.push(new_color);
    }


    // returns the spectrum
    spectrum
}

pub fn get_closest_color(pallet: &Vec<Rgb<u8>>, color: Rgb<u8>) -> Rgb<u8> {
    if pallet.is_empty() { return color; }

    let mut closest_color_index = 0;
    let mut closest_color_distance = f32::MAX;

    for i in 0..pallet.len() {
        let distance = get_distance(color, pallet[i]);
        if distance < closest_color_distance {
            closest_color_index = i;
            closest_color_distance = distance;
        }
    }

    pallet[closest_color_index]
}

pub fn get_distance(color_1: Rgb<u8>, color_2: Rgb<u8>) -> f32 {
    let r_dist = (color_1[0] as f32 - color_2[0] as f32).abs();
    let g_dist = (color_1[1] as f32 - color_2[1] as f32).abs();
    let b_dist = (color_1[2] as f32 - color_2[2] as f32).abs();

    (r_dist.powi(2) + g_dist.powi(2) + b_dist.powi(2)).sqrt()
}