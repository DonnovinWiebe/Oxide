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
pub fn is_hex(code: String) -> bool {
    let code = code.trim_start_matches('#');
    (code.len() == 3 || code.len() == 6 || code.len() == 8) && code.chars().all(|c| c.is_ascii_hexdigit())
}

/// Converts a HEX color code to an RGB color.
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
fn get_distance(color_1: Rgb<u8>, color_2: Rgb<u8>, bias: Option<f32>) -> f32 {
    let r = ((color_1[0] as f32 - color_2[0] as f32).abs() * 0.299) / bias.unwrap_or(1.0);
    let g = ((color_1[1] as f32 - color_2[1] as f32).abs() * 0.587) / bias.unwrap_or(1.0);
    let b = ((color_1[2] as f32 - color_2[2] as f32).abs() * 0.114) / bias.unwrap_or(1.0);

    (r.powi(2) + g.powi(2) + b.powi(2)).sqrt()
}

/// Returns the closest color from a given pallet to a given color.
pub fn get_closest_color(pallet: &Vec<Rgb<u8>>, color: Rgb<u8>) -> Rgb<u8> {
    if pallet.is_empty() { return color; }

    let mut closest_color = pallet[0];
    let mut closest_distance = get_distance(color, closest_color, None);

    for &palette_color in &pallet[1..] {
        let distance = get_distance(color, palette_color, None);
        if distance < closest_distance {
            closest_distance = distance;
            closest_color = palette_color;
        }
    }

    closest_color
}

/// Returns the closest color from a given pallet to a given color.
pub fn get_closest_color_biased(biased_pallet: &Vec<Rgb<u8>>, standard_pallet: &Vec<Rgb<u8>>, color: Rgb<u8>) -> Rgb<u8> {
    if biased_pallet.is_empty() || biased_pallet.is_empty() { return color; }

    let mut closest_color = biased_pallet[0];
    let mut closest_distance = get_distance(color, closest_color, Some(standard_bias()));

    for &biased_color in &biased_pallet[1..] {
        let distance = get_distance(color, biased_color, Some(standard_bias()));
        if distance < closest_distance {
            closest_distance = distance;
            closest_color = biased_color;
        }
    }
    for &standard_color in &standard_pallet[0..] {
        let distance = get_distance(color, standard_color, None);
        if distance < closest_distance {
            closest_distance = distance;
            closest_color = standard_color;
        }
    }

    closest_color
}

/// Gets all the colors between two other colors.
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

/// Gets the spectrum for a given pair of colors.
/// Each spectrum is a region of 3d color space that envelopes white -> colors -> black in one or two connected planes.
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

/// Gets the spectrum for a given triplet of colors.
/// Each spectrum is a region of 3d color space that envelopes white -> colors -> black in a single region.
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
fn is_accent_color(color: Rgb<u8>) -> bool {
    let r_value = color[0] as f32 * 0.299;
    let g_value = color[1] as f32 * 0.587;
    let b_value = color[2] as f32 * 0.114;
    let brightness = r_value + g_value + b_value;
    let perceived_greyscale_color = Rgb([brightness as u8, brightness as u8, brightness as u8]);
    get_distance(color, perceived_greyscale_color, None) > color_region_differentiation() * accent_color_multiplier()
}

/// Returns whether two colors are different enough to be considered separate regions.
fn is_different_color_region(color_1: Rgb<u8>, color_2: Rgb<u8>) -> bool {
    get_distance(color_1, color_2, None) > color_region_differentiation()
}

/// Gets the average color from an image.
pub fn get_accent_color(image: &DynamicImage) -> Rgb<u8> {
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
    let overlapping_accent_regions: Vec<Vec<Rgb<u8>>> = chunks.into_par_iter().flat_map(|chunk| {
        let mut accent_regions: Vec<Vec<Rgb<u8>>> = Vec::new();
        chunk.into_iter().filter(|pixel| is_accent_color(*pixel)).for_each(|pixel| {
            // checks to see if the current pixel fits into an existing region
            let mut is_new_region = true;
            for region in &mut accent_regions {
                // adds the pixel to the region if it fits
                if !is_different_color_region(get_average_color_from_pixels(region), pixel) {
                    is_new_region = false;
                    region.push(pixel);
                    break;
                }
            }

            // if no existing region fits, a new region for the pixel is created
            if is_new_region { accent_regions.push(vec![pixel]); }
        });
        accent_regions
    }).collect();

    // the list of merged accent regions
    let mut merged_accent_regions: Vec<Vec<Rgb<u8>>> = Vec::new();

    // iterates over every overlapping accent region to find where it fits in the merged accent regions
    // if there is no fitting merged region, a new merged region is created
    for overlapping_region in &overlapping_accent_regions {
        let average_overlapping_region_color = get_average_color_from_pixels(&overlapping_region);

        // iterates over every merged region to find where the overlapping region fits in the merged regions
        let mut merged = false;
        for merged_region in &mut merged_accent_regions {
            let average_merged_region_color = get_average_color_from_pixels(merged_region);
            // merges the overlapping region into the merged region if it fits
            if !is_different_color_region(average_overlapping_region_color, average_merged_region_color) {
                merged_region.extend(overlapping_region);
                merged = true;
                break;
            }
        }
        // if the overlapping region does not fit in any merged region, a new merged region is created
        if !merged { merged_accent_regions.push(overlapping_region.clone()); }
    }

    // returns the average color of the image if there are no accent color regions
    if merged_accent_regions.is_empty() { return get_average_color_from_image(image); }

    // gets the region with the most pixels and returns its average color
    let largest_region = merged_accent_regions.iter().max_by_key(|region| region.len()).unwrap();
    get_average_color_from_pixels(largest_region)
}

/// Evenly colorizes an image using only the colors in a given pallet.
pub fn palletize_evenly(source_image: DynamicImage, pallet: Vec<Rgb<u8>>, render_progress: impl Fn(String) + Sync) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // information
    let (width, height) = source_image.dimensions();
    let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // pixel information
    render_progress("Editing pixels...".to_string());
    let pixel_count = width * height;
    let pixels_edited = AtomicUsize::new(0);

    // editing pixel by pixel and collecting the new pixels
    let rows: Vec<(u32, u32, Rgb<u8>)> = (0..height).into_par_iter().flat_map(|y| {
        let mut row = vec![];
        for x in 0..width {
            let pixel = source_image.get_pixel(x, y).to_rgb();
            let new_pixel = get_closest_color(&pallet, pixel);

            row.push((x, y, new_pixel));

            let pixels_edited_internal = pixels_edited.fetch_add(1, Ordering::Relaxed);
            if pixels_edited_internal % 20000 == 0 {
                let percentage = (pixels_edited_internal as f64 / pixel_count as f64) * 100.0;
                render_progress(format!("Editing pixels... {}% complete", percentage.round() as usize));
            }
        }
        row
    }).collect();

    // filling the new image with the new pixels
    render_progress("Filling new image...".to_string());
    for (x, y, pixel) in rows {
        new_image.put_pixel(x, y, pixel);
    }

    // returns the new image
    new_image
}

/// Colorizes an image with two pallets with one being preferred.
pub fn palletize_biased(source_image: DynamicImage, biased_pallet: Vec<Rgb<u8>>, standard_pallet: Vec<Rgb<u8>>, render_progress: impl Fn(String) + Sync) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // information
    let (width, height) = source_image.dimensions();
    let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // pixel information
    render_progress("Editing pixels...".to_string());
    let pixel_count = width * height;
    let pixels_edited = AtomicUsize::new(0);

    // editing pixel by pixel and collecting the new pixels
    let rows: Vec<(u32, u32, Rgb<u8>)> = (0..height).into_par_iter().flat_map(|y| {
        let mut row = vec![];
        for x in 0..width {
            let pixel = source_image.get_pixel(x, y).to_rgb();
            let new_pixel = get_closest_color_biased(&biased_pallet, &standard_pallet, pixel);

            row.push((x, y, new_pixel));

            let pixels_edited_internal = pixels_edited.fetch_add(1, Ordering::Relaxed);
            if pixels_edited_internal % 20000 == 0 {
                let percentage = (pixels_edited_internal as f64 / pixel_count as f64) * 100.0;
                render_progress(format!("Editing pixels... {}% complete", percentage.round() as usize));
            }
        }
        row
    }).collect();

    // filling the new image with the new pixels
    render_progress("Filling new image...".to_string());
    for (x, y, pixel) in rows {
        new_image.put_pixel(x, y, pixel);
    }

    // returns the new image
    new_image
}