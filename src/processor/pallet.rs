use std::cmp::min;
use std::collections::HashSet;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rayon::prelude::*;
use crate::ui::render_loading;

/// Gets the standard distance difference used to define whether two colors are in the same or different color regions.
fn color_region_differentiation() -> f32 { 8.0 }

/// Gets the standard multiplier to determine what is considered an accent color (applied to color_region_differentiation()).
fn accent_color_multiplier() -> f32 { 1.5 }


/// Gets the max size a pallet can be.
fn max_pallet_size() -> usize { 50000 }

/// Gets the standard bias for biased colorizing.
fn standard_bias() -> f32 { 1.5 }

/// Gets the standard step count required to catch all colors between any two different colors.
fn interpolation_steps() -> usize { 442 }

/// Returns a standard white color.
fn white() -> Rgb<u8> { Rgb([255, 255, 255]) }

/// Returns a standard black color.
fn black() -> Rgb<u8> { Rgb([0, 0, 0]) }

/// Checks if a given input is a valid HEX color code.
pub fn is_hex(code: &String) -> bool {
    let code = code.trim_start_matches('#');
    (code.len() == 3 || code.len() == 6 || code.len() == 8) && code.chars().all(|c| c.is_ascii_hexdigit())
}

/// Converts a HEX color code to an RGB color.
pub fn as_rgb(hex: &String) -> Option<Rgb<u8>> {
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

/// Reduces the pallet size to be used efficiently.
pub fn condense_color_pallet(pallet: &Vec<Rgb<u8>>) -> Vec<Rgb<u8>> {
    // checks if the pallet is already small enough
    let pallet = remove_duplicates_unordered(pallet.clone());
    if pallet.len() < max_pallet_size() { return pallet; }

    // sets up tracking variables
    let mut similar_color_threshold: u8 = 2;
    let mut current_reduction_iteration: usize = 0;
    // the condensed pallet being built
    let mut condensed_pallet = pallet.clone();
    // continues iterating until the pallet is small enough
    while condensed_pallet.len() > max_pallet_size() {
        // checks if the pallet is not being reduced fast enough in order to prevent infinite loops
        current_reduction_iteration += 1;
        if current_reduction_iteration > 50 { panic!("Failed to condense pallet in 1000 passes. Max size: {} Got: {}", max_pallet_size(), condensed_pallet.len()); }

        // increments the similar_color_threshold with each iteration
        similar_color_threshold += 1;
        // creates a new condensed pallet at the current threshold
        let mut new_condensed_pallet = Vec::new();
        for color in &pallet {
            new_condensed_pallet.push(Rgb([
                (color[0] / similar_color_threshold) * similar_color_threshold + (similar_color_threshold / 2),
                (color[1] / similar_color_threshold) * similar_color_threshold + (similar_color_threshold / 2),
                (color[2] / similar_color_threshold) * similar_color_threshold + (similar_color_threshold / 2)
            ]));
        }

        // updates the condensed pallet
        condensed_pallet = remove_duplicates_unordered(new_condensed_pallet);
    }

    // returns the condensed pallet
    condensed_pallet
}

/// Removes duplicate colors from a given list of colors while maintaining the original order.
fn remove_duplicates_ordered<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    data.into_iter().filter(|item| seen.insert(item.clone())).collect()
}

/// Removes duplicate colors from a given list of colors without maintaining the original order.
fn remove_duplicates_unordered<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
    let set: HashSet<_> = data.into_iter().collect();
    set.into_iter().collect()
}

/// Gets the distance between two colors.
/// Increasing the bias makes the two colors read as closer (in most use cases that means more likely)
fn get_distance(color_1: &Rgb<u8>, color_2: &Rgb<u8>, bias: &Option<f32>) -> f32 {
    let r = ((color_1[0] as f32 - color_2[0] as f32).abs() * 0.299) / bias.unwrap_or(1.0);
    let g = ((color_1[1] as f32 - color_2[1] as f32).abs() * 0.587) / bias.unwrap_or(1.0);
    let b = ((color_1[2] as f32 - color_2[2] as f32).abs() * 0.114) / bias.unwrap_or(1.0);

    (r.powi(2) + g.powi(2) + b.powi(2)).sqrt()
}

/// Returns the closest color from a given pallet to a given color.
pub fn get_closest_color(pallet: &Vec<Rgb<u8>>, color: &Rgb<u8>) -> Rgb<u8> {
    if pallet.is_empty() { return color.clone(); }

    let mut closest_color = pallet[0];
    let mut closest_distance = get_distance(color, &closest_color, &None);

    for &palette_color in &pallet[1..] {
        let distance = get_distance(color, &palette_color, &None);
        if distance < closest_distance {
            closest_distance = distance;
            closest_color = palette_color;
        }
    }

    closest_color
}

/// Returns the closest color from a given pallet to a given color.
pub fn get_closest_color_biased(biased_pallet: &Vec<Rgb<u8>>, standard_pallet: &Vec<Rgb<u8>>, color: &Rgb<u8>) -> Rgb<u8> {
    if biased_pallet.is_empty() || biased_pallet.is_empty() { return color.clone(); }

    let mut closest_color = biased_pallet[0];
    let mut closest_distance = get_distance(color, &closest_color, &Some(standard_bias()));

    for &biased_color in &biased_pallet[1..] {
        let distance = get_distance(color, &biased_color, &Some(standard_bias()));
        if distance < closest_distance {
            closest_distance = distance;
            closest_color = biased_color;
        }
    }
    for &standard_color in &standard_pallet[0..] {
        let distance = get_distance(color, &standard_color, &None);
        if distance < closest_distance {
            closest_distance = distance;
            closest_color = standard_color;
        }
    }

    closest_color
}

/// Gets all the colors between two other colors.
fn get_colors_between(color_1: &Rgb<u8>, color_2: &Rgb<u8>) -> Vec<Rgb<u8>> {
    // step information
    let r_difference = (color_2[0] as f64 - color_1[0] as f64) / interpolation_steps() as f64;
    let g_difference = (color_2[1] as f64 - color_1[1] as f64) / interpolation_steps() as f64;
    let b_difference = (color_2[2] as f64 - color_1[2] as f64) / interpolation_steps() as f64;
    let mut spectrum = vec![];

    // getting the spectrum
    let mut current_color = (color_1[0] as f64, color_1[1] as f64, color_1[2] as f64);
    for _ in 0..=interpolation_steps() {
        spectrum.push(Rgb([current_color.0.round() as u8, current_color.1.round() as u8, current_color.2.round() as u8]));
        current_color.0 = r_difference + current_color.0;
        current_color.1 = g_difference + current_color.1;
        current_color.2 = b_difference + current_color.2;
    }

    // removes duplicates from the spectrum
    spectrum = remove_duplicates_ordered(spectrum);

    // returns the spectrum
    spectrum
}

/// Gets the spectrum for a given color.
/// Each spectrum is a smooth gradient from white -> color -> black.
pub fn get_line_spectrum(color: &Rgb<u8>) -> Vec<Rgb<u8>> {
    // getting the spectrum
    let mut spectrum = vec![];
    spectrum.extend(get_colors_between(&white(), color));
    spectrum.extend(get_colors_between(color, &black()));

    // removes duplicates from the spectrum
    spectrum = remove_duplicates_ordered(spectrum);

    // returns the spectrum
    spectrum
}

/// Gets the 1d spectrums for all the colors in a given pallet and returns the results as a single pallet.
pub fn get_line_spectrums(pallet: &Vec<Rgb<u8>>) -> Vec<Vec<Rgb<u8>>> {
    let mut line_spectrums = Vec::new();
    pallet.iter().for_each(|color| {
        line_spectrums.push(get_line_spectrum(color));
    });

    line_spectrums
}

/// Gets the spectrum for a given pair of colors.
/// Each spectrum is a region of 3d color space that envelopes white -> colors -> black in one or two connected planes.
pub fn get_plane_spectrum(line_spectrum_1: &Vec<Rgb<u8>>, line_spectrum_2: &Vec<Rgb<u8>>) -> Vec<Rgb<u8>> {
    let spectrum_steps = min(line_spectrum_1.len(), line_spectrum_2.len());

    let mut spectrum: Vec<Rgb<u8>> = (0..spectrum_steps).into_par_iter().flat_map(|i| {
        let mut colors_between = get_colors_between(&line_spectrum_1[i], &line_spectrum_2[i]);
        colors_between.extend(get_colors_between(&line_spectrum_1[line_spectrum_1.len() - 1 - i], &line_spectrum_2[line_spectrum_2.len() - 1 - i]));
        colors_between
    }).collect();

    spectrum = remove_duplicates_unordered(spectrum);

    spectrum
}

/// Combines all the plane spectrums between all line spectrums in a given list.
pub fn get_web_spectrum(line_spectrums: &Vec<Vec<Rgb<u8>>>) -> Vec<Rgb<u8>> {
    let mut spectrum = Vec::new();
    for x in 0..line_spectrums.len() {
        for y in 0..line_spectrums.len() {
            if x != y { spectrum.extend(get_plane_spectrum(&line_spectrums[x], &line_spectrums[y])); }
        }
    }

    remove_duplicates_unordered(spectrum)
}

/// Gets the average color from an image.
pub fn get_average_color_from_image(image: &DynamicImage) -> Rgb<u8> {
    let (width, height) = image.dimensions();
    let pixel_count = (width * height) as f32;
    let mut r: f32 = 0.0;
    let mut g: f32 = 0.0;
    let mut b: f32 = 0.0;

    for y in 0..height {
        for x in 0..width {
            r += image.get_pixel(x, y)[0] as f32;
            g += image.get_pixel(x, y)[1] as f32;
            b += image.get_pixel(x, y)[2] as f32;
        }
    }

    Rgb([(r / pixel_count).round() as u8, (g / pixel_count).round() as u8, (b / pixel_count).round() as u8])
}

/// Gets the average color from a list of pixels.
pub fn get_average_color_from_pixels(pixels: &Vec<Rgb<u8>>) -> Rgb<u8> {
    let mut r: f32 = 0.0;
    let mut g: f32 = 0.0;
    let mut b: f32 = 0.0;

    for pixel in pixels {
        r += pixel[0] as f32;
        g += pixel[1] as f32;
        b += pixel[2] as f32;
    }

    Rgb([(r / pixels.len() as f32).round() as u8, (g / pixels.len() as f32).round() as u8, (b / pixels.len() as f32).round() as u8])
}

/// Returns whether a color is considered an accent color.
fn is_accent_color(color: &Rgb<u8>) -> bool {
    let r_value = color[0] as f32 * 0.299;
    let g_value = color[1] as f32 * 0.587;
    let b_value = color[2] as f32 * 0.114;
    let brightness = r_value + g_value + b_value;
    let perceived_greyscale_color = Rgb([brightness as u8, brightness as u8, brightness as u8]);
    get_distance(color, &perceived_greyscale_color, &None) > color_region_differentiation() * accent_color_multiplier()
}

/// Returns whether two colors are different enough to be considered separate regions.
fn is_different_color_region(color_1: &Rgb<u8>, color_2: &Rgb<u8>) -> bool {
    get_distance(color_1, color_2, &None) > color_region_differentiation()
}

/// Gets the average color from an image.
pub fn get_accent_color(image: &DynamicImage) -> Rgb<u8> {
    let average_image_color = get_average_color_from_image(image);
    let mut accent_map = AccentMap::new();
    image.pixels().filter(|pixel| { is_accent_color(&pixel.2.to_rgb()) }).for_each(|pixel| {
        let color = pixel.2.to_rgb();
        accent_map.add_color(&color, get_distance(&average_image_color, &color, &None));
    });
    accent_map.accent()
}



pub mod pallets {
    use image::Rgb;
    pub fn volcanic_crater() -> Vec<Rgb<u8>> {
        vec![
            Rgb([47, 79, 79]),
            Rgb([139, 0, 0]),
            Rgb([255, 69, 0]),
            Rgb([255, 215, 0]),
        ]
    }

    pub fn red_rocks() -> Vec<Rgb<u8>> {
        vec![
            Rgb([139, 69, 19]),
            Rgb([184, 134, 11]),
            Rgb([233, 150, 122]),
            Rgb([188, 143, 143]),
        ]
    }

    pub fn deepest_africa() -> Vec<Rgb<u8>> {
        vec![
            Rgb([85, 107, 47]),
            Rgb([139, 69, 19]),
            Rgb([184, 134, 11]),
            Rgb([222, 184, 135]),
            Rgb([255, 99, 71]),
        ]
    }

    pub fn arctic_wilderness() -> Vec<Rgb<u8>> {
        vec![
            Rgb([47, 79, 79]),
            Rgb([70, 130, 180]),
            Rgb([95, 158, 160]),
            Rgb([173, 216, 230]),
        ]
    }

    pub fn iceland() -> Vec<Rgb<u8>> {
        vec![
            Rgb([47, 79, 79]),
            Rgb([85, 107, 47]),
            Rgb([112, 128, 144]),
            Rgb([143, 188, 143]),
            Rgb([176, 196, 222]),
            Rgb([210, 180, 140]),
        ]
    }

    pub fn english_oaks() -> Vec<Rgb<u8>> {
        vec![
            Rgb([47, 79, 47]),
            Rgb([107, 142, 35]),
            Rgb([128, 128, 0]),
            Rgb([139, 69, 19]),
            Rgb([189, 183, 107]),
        ]
    }

    pub fn wheat_field() -> Vec<Rgb<u8>> {
        vec![
            Rgb([184, 134, 11]),
            Rgb([218, 165, 32]),
            Rgb([240, 230, 140]),
            Rgb([255, 248, 220]),
        ]
    }

    pub fn south_american_jungle() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 100, 0]),
            Rgb([46, 139, 87]),
            Rgb([85, 107, 47]),
            Rgb([128, 128, 0]),
            Rgb([139, 69, 19]),
            Rgb([173, 255, 47]),
            Rgb([255, 69, 0]),
            Rgb([255, 215, 0]),
        ]
    }

    pub fn european_islands() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 206, 209]),
            Rgb([60, 179, 113]),
            Rgb([70, 130, 180]),
            Rgb([112, 128, 144]),
            Rgb([143, 188, 143]),
            Rgb([210, 180, 140]),
        ]
    }

    pub fn colorful_islands() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 100, 0]),
            Rgb([0, 139, 139]),
            Rgb([30, 144, 255]),
            Rgb([64, 224, 208]),
            Rgb([245, 222, 179]),
            Rgb([255, 99, 71]),
            Rgb([255, 215, 0]),
        ]
    }
}



/// Maps out which accent colors are the most prominent in list of colors.
struct AccentMap { // based on a color_region_differentiation() being 8.0
    /// The map of possible accents.
    map: Vec<Vec<Vec<f32>>>,
    /// Tracks which accent score is the greatest as the map is being built.
    greatest_accent_score: f32,
    /// Tracks the index of the current greatest accent score.
    greatest_accent_index: (usize, usize, usize),
}
impl AccentMap {
    /// Creates a new accent map.
    pub fn new() -> AccentMap {
        AccentMap {
            map: vec![vec![vec![0f32; 32]; 32]; 32],
            greatest_accent_score: 0.0,
            greatest_accent_index: (0, 0, 0),
        }
    }

    /// Adds a color to the accent map and updates which accent is the greatest.
    pub fn add_color(&mut self, new_color: &Rgb<u8>, distance_from_average: f32) {
        let x = (new_color[0] / 8) as usize;
        let y = (new_color[1] / 8) as usize;
        let z = (new_color[2] / 8) as usize;
        self.map[x][y][z] += distance_from_average;

        if self.map[x][y][z] > self.greatest_accent_score {
            self.greatest_accent_score = self.map[x][y][z];
            self.greatest_accent_index = (x, y, z);
        }
    }

    /// Returns the current greatest accent.
    pub fn accent(&self) -> Rgb<u8> {
        let r = self.greatest_accent_index.0 * 8 + 4;
        let g = self.greatest_accent_index.1 * 8 + 4;
        let b = self.greatest_accent_index.2 * 8 + 4;
        Rgb([r as u8, g as u8, b as u8])
    }
}