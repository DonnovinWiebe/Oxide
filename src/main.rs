mod processor;

use std::error::Error;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let working_directory = std::env::current_dir()?;
    let output_directory = working_directory.join("output");
    fs::create_dir_all(&output_directory)?;

    let image_paths: Vec<PathBuf> = fs::read_dir(&working_directory)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|s| matches!(s.to_lowercase().as_str(), "png" | "jpg" | "jpeg"))
                .unwrap_or(false)
        })
        .collect();

    for path in image_paths {
        // todo implement
    }

    Ok(())
}