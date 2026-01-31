use std::cmp::min;
use std::collections::HashSet;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rayon::prelude::*;
use crate::ui::render_loading;

/// Gets the standard distance difference used to define whether two colors are in the same or different color regions.
fn color_region_differentiation() -> f32 { 8.0 }

/// Gets the standard multiplier to determine what is considered an accent color (applied to color_region_differentiation()).
fn accent_color_multiplier() -> f32 { 2.0 }

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

/// Removes duplicate colors from a given list of colors while maintaining the original order.
pub fn remove_duplicates_ordered<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    data.into_iter().filter(|item| seen.insert(item.clone())).collect()
}

/// Removes duplicate colors from a given list of colors without maintaining the original order.
pub fn remove_duplicates_unordered<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
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
pub fn get_1d_spectrum(color: &Rgb<u8>) -> Vec<Rgb<u8>> {
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
pub fn get_1d_spectrums_for_pallet(pallet: &Vec<Rgb<u8>>) -> Vec<Rgb<u8>> {
    let sum_pallet = pallet.par_iter().flat_map(|color| {
        get_1d_spectrum(color)
    }).collect();

    remove_duplicates_ordered(sum_pallet)
}

/// Gets the spectrum for a given pair of colors.
/// Each spectrum is a region of 3d color space that envelopes white -> colors -> black in one or two connected planes.
pub fn get_2d_spectrum(color_1: &Rgb<u8>, color_2: &Rgb<u8>) -> Vec<Rgb<u8>> {
    let spectrum_1 = get_1d_spectrum(color_1);
    let spectrum_2 = get_1d_spectrum(color_2);
    let spectrum_steps = min(spectrum_1.len(), spectrum_2.len());

    let mut spectrum: Vec<Rgb<u8>> = (0..spectrum_steps).into_par_iter().flat_map(|i| {
        let mut colors_between = get_colors_between(&spectrum_1[i], &spectrum_2[i]);
        colors_between.extend(get_colors_between(&spectrum_1[spectrum_1.len() - 1 - i], &spectrum_2[spectrum_2.len() - 1 - i]));
        colors_between
    }).collect();

    spectrum = remove_duplicates_unordered(spectrum);

    spectrum
}

/// Gets the spectrum for a given triplet of colors.
/// Each spectrum is a region of 3d color space that envelopes white -> colors -> black in a single region.
fn get_3d_spectrum(color_1: &Rgb<u8>, color_2: &Rgb<u8>, color_3: &Rgb<u8>) -> Vec<Rgb<u8>> {
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

/// Returns the colors in 3d color space that are inside the listed color points at the given blue coordinate plane.
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
    // gets the average color of the image for later
    let average_image_color = get_average_color_from_image(image);
    // gets the number of chunks to split the image into to process it in parallel
    let chunk_count = rayon::current_num_threads();

    // splits the image into chunks to process in parallel
    let pixels: Vec<Rgb<u8>> = image.pixels().map(|p| {
        let rgba = p.2;
        Rgb([rgba[0], rgba[1], rgba[2]])
    }).collect();
    let pixel_count = pixels.len();
    let pixels_per_chunk = pixel_count as u32 / chunk_count as u32;
    let chunks: Vec<Vec<Rgb<u8>>> = pixels.chunks(pixels_per_chunk as usize).map(|chunk| chunk.to_vec()).collect();

    // collects all the accent regions from different threads
    let overlapping_accent_regions: Vec<Region> = chunks.into_par_iter().flat_map(|chunk| {
        let mut accent_regions: Vec<Region> = Vec::new();
        chunk.into_iter().filter(|pixel| is_accent_color(pixel)).for_each(|pixel| {
            // checks to see if the current pixel fits into an existing region
            let mut is_new_region = true;
            for region in &mut accent_regions {
                // adds the pixel to the region if it fits
                if !is_different_color_region(&region.average_color, &pixel) {
                    is_new_region = false;
                    region.add_color(pixel);
                    break;
                }
            }

            // if no existing region fits, a new region for the pixel is created
            if is_new_region {
                let mut new_region = Region::new();
                new_region.add_color(pixel);
                accent_regions.push(new_region);
            }
        });
        accent_regions
    }).collect();

    // the list of merged accent regions
    let mut merged_accent_regions: Vec<Region> = Vec::new();

    // iterates over every overlapping accent region to find where it fits in the merged accent regions
    // if there is no fitting merged region, a new merged region is created
    for overlapping_region in &overlapping_accent_regions {
        // iterates over every merged region to find where the overlapping region fits in the merged regions
        let mut merged = false;
        for merged_region in &mut merged_accent_regions {
            // merges the overlapping region into the merged region if it fits
            if !is_different_color_region(&overlapping_region.average_color, &merged_region.average_color) {
                merged_region.add_colors(&overlapping_region.colors);
                merged = true;
                break;
            }
        }
        // if the overlapping region does not fit in any merged region, a new merged region is created
        if !merged { merged_accent_regions.push(overlapping_region.clone()); }
    }

    // returns the average color of the image if there are no accent color regions
    if merged_accent_regions.is_empty() { return average_image_color; }

    // gets the region with the greatest score and returns its average color
    let mut greatest_accent_region_score: f32 = 0.0;
    let mut greatest_accent_region_index = 0;
    for i in 0..merged_accent_regions.len() {
        let accent_score = merged_accent_regions[i].accent_score(&average_image_color);
        if accent_score > greatest_accent_region_score {
            greatest_accent_region_score = accent_score;
            greatest_accent_region_index = i;
        }
    }
    merged_accent_regions[greatest_accent_region_index].average_color
}



pub mod pallets {
    use image::Rgb;

    pub fn volcanic_crater() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 0, 0]),
            Rgb([28, 28, 28]),
            Rgb([47, 79, 79]),
            Rgb([105, 105, 105]),
            Rgb([139, 0, 0]),
            Rgb([165, 42, 42]),
            Rgb([178, 34, 34]),
            Rgb([220, 20, 60]),
            Rgb([255, 69, 0]),
            Rgb([255, 99, 71]),
            Rgb([255, 140, 0]),
            Rgb([255, 165, 0]),
            Rgb([255, 215, 0]),
        ]
    }

    pub fn red_rocks() -> Vec<Rgb<u8>> {
        vec![
            Rgb([139, 69, 19]),
            Rgb([160, 82, 45]),
            Rgb([205, 133, 63]),
            Rgb([210, 105, 30]),
            Rgb([184, 134, 11]),
            Rgb([218, 165, 32]),
            Rgb([233, 150, 122]),
            Rgb([244, 164, 96]),
            Rgb([188, 143, 143]),
            Rgb([193, 154, 107]),
        ]
    }

    pub fn deepest_africa() -> Vec<Rgb<u8>> {
        vec![
            Rgb([139, 69, 19]),
            Rgb([160, 82, 45]),
            Rgb([205, 133, 63]),
            Rgb([210, 105, 30]),
            Rgb([222, 184, 135]),
            Rgb([244, 164, 96]),
            Rgb([85, 107, 47]),
            Rgb([107, 142, 35]),
            Rgb([139, 115, 85]),
            Rgb([184, 134, 11]),
            Rgb([255, 99, 71]),
            Rgb([255, 140, 0]),
        ]
    }

    pub fn arctic_wilderness() -> Vec<Rgb<u8>> {
        vec![
            Rgb([224, 255, 255]),
            Rgb([240, 248, 255]),
            Rgb([240, 255, 255]),
            Rgb([175, 238, 238]),
            Rgb([176, 224, 230]),
            Rgb([173, 216, 230]),
            Rgb([135, 206, 235]),
            Rgb([135, 206, 250]),
            Rgb([70, 130, 180]),
            Rgb([95, 158, 160]),
            Rgb([112, 128, 144]),
            Rgb([47, 79, 79]),
        ]
    }

    pub fn iceland() -> Vec<Rgb<u8>> {
        vec![
            Rgb([47, 79, 79]),
            Rgb([85, 107, 47]),
            Rgb([112, 128, 144]),
            Rgb([119, 136, 153]),
            Rgb([143, 188, 143]),
            Rgb([159, 182, 205]),
            Rgb([176, 196, 222]),
            Rgb([192, 192, 192]),
            Rgb([210, 180, 140]),
            Rgb([224, 238, 238]),
            Rgb([240, 248, 255]),
            Rgb([67, 67, 67]),
        ]
    }

    pub fn english_oaks() -> Vec<Rgb<u8>> {
        vec![
            Rgb([47, 79, 47]),
            Rgb([59, 83, 35]),
            Rgb([85, 107, 47]),
            Rgb([107, 142, 35]),
            Rgb([128, 128, 0]),
            Rgb([139, 69, 19]),
            Rgb([143, 188, 143]),
            Rgb([160, 82, 45]),
            Rgb([189, 183, 107]),
            Rgb([210, 105, 30]),
            Rgb([222, 184, 135]),
        ]
    }

    pub fn wheat_field() -> Vec<Rgb<u8>> {
        vec![
            Rgb([184, 134, 11]),
            Rgb([218, 165, 32]),
            Rgb([210, 180, 140]),
            Rgb([222, 184, 135]),
            Rgb([240, 230, 140]),
            Rgb([245, 222, 179]),
            Rgb([255, 228, 181]),
            Rgb([255, 239, 213]),
            Rgb([255, 248, 220]),
            Rgb([255, 250, 205]),
        ]
    }

    pub fn south_american_jungle() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 100, 0]),
            Rgb([34, 139, 34]),
            Rgb([46, 139, 87]),
            Rgb([60, 179, 113]),
            Rgb([85, 107, 47]),
            Rgb([107, 142, 35]),
            Rgb([128, 128, 0]),
            Rgb([139, 69, 19]),
            Rgb([154, 205, 50]),
            Rgb([173, 255, 47]),
            Rgb([255, 69, 0]),
            Rgb([255, 215, 0]),
            Rgb([28, 28, 28]),
        ]
    }

    pub fn european_islands() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 206, 209]),
            Rgb([32, 178, 170]),
            Rgb([60, 179, 113]),
            Rgb([70, 130, 180]),
            Rgb([102, 205, 170]),
            Rgb([112, 128, 144]),
            Rgb([135, 206, 235]),
            Rgb([143, 188, 143]),
            Rgb([176, 196, 222]),
            Rgb([210, 180, 140]),
            Rgb([222, 184, 135]),
            Rgb([245, 245, 220]),
        ]
    }

    pub fn colorful_islands() -> Vec<Rgb<u8>> {
        vec![
            Rgb([0, 100, 0]),
            Rgb([0, 139, 139]),
            Rgb([0, 206, 209]),
            Rgb([30, 144, 255]),
            Rgb([32, 178, 170]),
            Rgb([34, 139, 34]),
            Rgb([64, 224, 208]),
            Rgb([72, 209, 204]),
            Rgb([255, 215, 0]),
            Rgb([245, 222, 179]),
            Rgb([250, 250, 210]),
            Rgb([255, 99, 71]),
            Rgb([255, 127, 80]),
        ]
    }
}



/// A region of colors.
#[derive(Clone)]
struct Region {
    /// All the colors in the region
    pub colors: Vec<Rgb<u8>>,
    /// The average color of the region
    pub average_color: Rgb<u8>,
}
impl Region {
    /// Returns a new region.
    pub fn new() -> Region {
        Region {
            colors: Vec::new(),
            average_color: Rgb([0, 0, 0])
        }
    }

    /// Adds a color to the region and updates the average color.
    pub fn add_color(&mut self, color: Rgb<u8>) {
        self.colors.push(color);
        self.update_average_color();
    }

    /// Adds a list of colors to the region and updates the average color.
    pub fn add_colors(&mut self, colors: &Vec<Rgb<u8>>) {
        self.colors.extend(colors);
        self.update_average_color();
    }

    /// Updates the average color of the region.
    fn update_average_color(&mut self) {
        self.average_color = get_average_color_from_pixels(&self.colors);
    }

    /// Gets the accent score based on the amount of colors and the distance from the images average color.
    pub fn accent_score(&self, average_image_color: &Rgb<u8>) -> f32 {
        let base_score = self.colors.len() as f32;
        let distance_multiplier = get_distance(&self.average_color, average_image_color, &None) / 10.0;
        base_score * distance_multiplier
    }
}