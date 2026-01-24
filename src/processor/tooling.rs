pub mod pallet {
    use std::cmp::min;
    use std::collections::HashSet;
    use image::{Pixel, Rgb};
    use rayon::prelude::*;

    fn interpolation_steps() -> usize { 442 } // this will catch every 8-bit color between any 2 8-bit colors
    fn white() -> Rgb<u8> { Rgb([255, 255, 255]) }
    fn black() -> Rgb<u8> { Rgb([0, 0, 0]) }


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

    pub fn remove_duplicates_ordered<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
        let mut seen = HashSet::new();
        data.into_iter().filter(|item| seen.insert(item.clone())).collect()
    }

    pub fn remove_duplicates_unordered<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
        let set: HashSet<_> = data.into_iter().collect();
        set.into_iter().collect()
    }

    fn get_distance_score(color_1: Rgb<u8>, color_2: Rgb<u8>) -> f32 {
        let r_score = ((color_1[0] as i32 - color_2[0] as i32) as f32).abs() / 0.299;
        let g_score = ((color_1[1] as i32 - color_2[1] as i32) as f32).abs() / 0.587;
        let b_score = ((color_1[2] as i32 - color_2[2] as i32) as f32).abs() / 0.114;

        r_score.powi(2) + g_score.powi(2) + b_score.powi(2)
    }

    pub fn get_closest_color(pallet: &Vec<Rgb<u8>>, color: Rgb<u8>) -> Rgb<u8> {
        if pallet.is_empty() { return color; }

        let mut closest_color = pallet[0];
        let mut closest_distance_score = get_distance_score(color, closest_color);

        for &palette_color in &pallet[1..] {
            let distance_score = get_distance_score(color, palette_color);
            if distance_score < closest_distance_score {
                closest_distance_score = distance_score;
                closest_color = palette_color;

                if distance_score == 0.0 { break; } // Early exit after exact match
            }
        }

        closest_color
    }

    fn get_colors_between(color_1: Rgb<u8>, color_2: Rgb<u8>) -> Vec<Rgb<u8>> {
        // step information
        let r_difference = (color_2[0] as f64 - color_1[0] as f64) / interpolation_steps() as f64;
        let g_difference = (color_2[1] as f64 - color_1[1] as f64) / interpolation_steps() as f64;
        let b_difference = (color_2[2] as f64 - color_1[2] as f64) / interpolation_steps() as f64;
        let mut spectrum = vec![];

        // getting the spectrum
        let mut current_color = (color_1[0] as f64, color_1[1] as f64, color_1[2] as f64);
        for _ in 0..=interpolation_steps() {
            spectrum.push(Rgb([current_color.0.round() as u8, current_color.1.round() as u8, current_color.2.round() as u8]));
            current_color.0 = (r_difference + current_color.0);
            current_color.1 = (g_difference + current_color.1);
            current_color.2 = (b_difference + current_color.2);
        }

        // removes duplicates from the spectrum
        spectrum = remove_duplicates_ordered(spectrum);

        // returns the spectrum
        spectrum
    }

    pub fn get_1d_spectrum(color: Rgb<u8>) -> Vec<Rgb<u8>> {
        // getting the spectrum
        let mut spectrum = vec![];
        spectrum.extend(get_colors_between(white(), color));
        spectrum.extend(get_colors_between(color, black()));

        // removes duplicates from the spectrum
        spectrum = remove_duplicates_ordered(spectrum);

        // returns the spectrum
        spectrum
    }

    pub fn get_2d_spectrum(color_1: Rgb<u8>, color_2: Rgb<u8>) -> Vec<Rgb<u8>> {
        let spectrum_1 = get_1d_spectrum(color_1);
        let spectrum_2 = get_1d_spectrum(color_2);
        let spectrum_steps = min(spectrum_1.len(), spectrum_2.len());

        let mut spectrum: Vec<Rgb<u8>> = (0..spectrum_steps).into_par_iter().flat_map(|i| {
            let mut colors_between = get_colors_between(spectrum_1[i], spectrum_2[i]);
            colors_between.extend(get_colors_between(spectrum_1[spectrum_1.len() - 1 - i], spectrum_2[spectrum_2.len() - 1 - i]));
            colors_between
        }).collect();

        spectrum = remove_duplicates_unordered(spectrum);

        spectrum
    }

    fn get_3d_spectrum(color_1: Rgb<u8>, color_2: Rgb<u8>, color_3: Rgb<u8>) -> Vec<Rgb<u8>> {
        // creating the spectrum
        let mut spectrum = vec![];
        spectrum.extend(get_2d_spectrum(color_1, color_2));
        spectrum.extend(get_2d_spectrum(color_2, color_3));
        spectrum.extend(get_2d_spectrum(color_3, color_1));
        spectrum = remove_duplicates_ordered(spectrum);

        for b in 0..=255 {
            let inside_colors = get_inside_colors_at_blue_value(&spectrum, b);
            spectrum.extend(inside_colors);
        }

        // removes duplicates from the spectrum
        spectrum = remove_duplicates_ordered(spectrum);

        // returns the spectrum
        spectrum
    }

    fn get_inside_colors_at_blue_value(all_edge_colors: &Vec<Rgb<u8>>, blue_value: u8) -> Vec<Rgb<u8>> {
        // I do not understand this code.
        // It was generated by ChatGPT and styled by me.
        // - Donnovin

        let mut inside_colors = vec![];

        let mut edge_colors = vec![];
        for color in all_edge_colors {
            if color[2] == blue_value { edge_colors.push(color); }
        }
        if edge_colors.is_empty() { return inside_colors; }

        let mut lowest_r = 255;
        let mut lowest_g = 255;
        let mut highest_r = 0;
        let mut highest_g = 0;
        for blue_match in &edge_colors {
            lowest_r = lowest_r.min(blue_match[0]);
            lowest_g = lowest_g.min(blue_match[1]);
            highest_r = highest_r.max(blue_match[0]);
            highest_g = highest_g.max(blue_match[1]);
        }

        for r in lowest_r..=highest_r {
            for g in lowest_g..=highest_g {
                let mut inside = true;

                for i in 0..edge_colors.len() {
                    let a = edge_colors[i];
                    let b = edge_colors[(i + 1) % edge_colors.len()];

                    let cross =
                        (r - a[0]) * (b[1] - a[1]) -
                            (g - a[1]) * (b[0] - a[0]);

                    if cross < 0 {
                        inside = false;
                        break;
                    }
                }

                if inside {
                    inside_colors.push(Rgb([r, g, blue_value]));
                }
            }
        }

        inside_colors
    }
}