use std::io::Stdout;
use std::sync::{Arc, Mutex};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crate::processor::pallet::{palletize_biased, palletize_evenly};
use crate::ui::render_loading;

/// Evenly processes and image using only the colors in a given pallet.
pub fn process_evenly(source_image: DynamicImage, pallet: Vec<Rgb<u8>>, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // clearing for processing render loop
    let _ = terminal.clear();

    // information
    let _ = terminal.draw(|frame| render_loading(frame, "Loading image information...".to_string()));
    let (width, height) = source_image.dimensions();

    let terminal_mutex = Arc::new(Mutex::new(terminal));
    let new_image = palletize_evenly(
        source_image,
        pallet,
        {
            let term = Arc::clone(&terminal_mutex);
            move |message| {
                let _ = term.lock().unwrap().draw(|frame| render_loading(frame, message));
            }
        }
    );

    new_image
}

/// Processes an image with two pallets with one being preferred.
pub fn process_biased(source_image: DynamicImage, biased_pallet: Vec<Rgb<u8>>, standard_pallet: Vec<Rgb<u8>>, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // clearing for processing render loop
    let _ = terminal.clear();

    // information
    let _ = terminal.draw(|frame| render_loading(frame, "Loading image information...".to_string()));
    let (width, height) = source_image.dimensions();

    let terminal_mutex = Arc::new(Mutex::new(terminal));
    let new_image = palletize_biased(
        source_image,
        biased_pallet,
        standard_pallet,
        {
            let term = Arc::clone(&terminal_mutex);
            move |message| {
                let _ = term.lock().unwrap().draw(|frame| render_loading(frame, message));
            }
        }
    );

    new_image
}