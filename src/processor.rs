pub mod guide;
pub mod tooling;

use std::error::Error;
use std::io::{Stdout, Write};
use std::path::PathBuf;
use image::{GenericImageView, ImageBuffer, ImageResult, Pixel, Rgb};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use crate::processor::guide::{ProcessingGuide, ProcessingStep, ProcessingStepTypes};
use crate::processor::tooling::{get_closest_color, get_spectrum};
use crate::ui::{render_current_page, render_progress};

pub enum Processors {
    Monochromatic,
    Bichromatic,
}
impl Processors {
    pub fn name(&self) -> String {
        match self {
            Processors::Monochromatic => "Monochromatic".to_string(),
            Processors::Bichromatic => "Bichromatic".to_string(),
        }
    }

    pub fn number_of_processors() -> usize { 2 }

    pub fn get_processor(selection: usize) -> Processors {
        match selection {
            0 => Processors::Monochromatic,
            1 => Processors::Bichromatic,
            _ => panic!("Invalid processor selection: {}", selection),
        }
    }
}



/// Defines an image processor.
pub trait EditProcessor {
    /// Returns the set of colors used in editing the image in order to print them in the editing image filename
    fn get_color_set(&self) -> String;

    /// Returns the label of the current step.
    fn get_current_step_label(&self) -> String;

    /// Returns the input of the current step.
    fn get_current_step_input(&self) -> String;

    /// Updates the input of the current step.
    fn update_current_step_input(&mut self, new_input: String);

    /// Checks if the input is valid for the current step.
    fn is_current_step_input_valid(&self) -> bool;

    /// Advances the guide to the next step if the input is valid.
    fn try_finish_current_step(&mut self);

    /// Populates the processor steps from the guide if the guide is ready.
    fn try_populate(&mut self);

    /// Returns if the processor is ready to process the image.
    fn is_ready(&self) -> bool;

    /// Processes the image and returns the new image.
    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) ->  Option<ImageBuffer<Rgb<u8>, Vec<u8>>>;
}



/// Processes an image into a single color spectrum.
pub struct MonochromaticEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The base color of the spectrum being used as a hex value.
    base_color_hex: String,
    /// The base color of the spectrum being used as an rgb color.
    base_color_rgb: Rgb<u8>,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl MonochromaticEdit {
    pub fn new(source_image_path: PathBuf) -> MonochromaticEdit {
        MonochromaticEdit {
            source_image_path,
            base_color_rgb: Rgb([0, 0, 0]),
            base_color_hex: "none".to_string(),
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color (HEX)".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for MonochromaticEdit {
    fn get_color_set(&self) -> String {
        format!("{}", self.base_color_hex.clone())
    }

    fn get_current_step_label(&self) -> String {
        self.guide.get_current_label()
    }

    fn get_current_step_input(&self) -> String {
        self.guide.get_current_input()
    }

    fn update_current_step_input(&mut self, new_input: String) {
        self.guide.update_current_input(new_input)
    }

    fn is_current_step_input_valid(&self) -> bool {
        self.guide.is_current_input_valid()
    }

    fn try_finish_current_step(&mut self) {
        if self.is_current_step_input_valid() { self.guide.try_finish_current_step(); }
    }

    fn try_populate(&mut self) {
        if !self.guide.is_ready() { return; }
        
        let base_color_hex_result = self.guide.steps[0].as_hex();
        if base_color_hex_result.is_some() {
            self.base_color_hex = base_color_hex_result.unwrap();
            self.base_color_rgb = tooling::as_rgb(self.base_color_hex.clone()).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }
        
        let source_image_result = image::open(self.source_image_path.clone());
        if let ImageResult::Ok(source_image) = source_image_result {
            // information
            let (width, height) = source_image.dimensions();
            let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
            let spectrum = get_spectrum(self.base_color_rgb);

            // Process pixel by pixel using coordinates
            let pixel_count = width * height;
            let mut pixels_edited = 0;
            for y in 0..height {
                for x in 0..width {
                    let pixel = source_image.get_pixel(x, y).to_rgb();
                    let new_pixel = get_closest_color(&spectrum, pixel);

                    new_image.put_pixel(x, y, new_pixel);

                    pixels_edited += 1;
                    if pixels_edited % 300 == 0 {
                        let percentage = (pixels_edited as f64 / pixel_count as f64) * 100.0;
                        let _ = terminal.draw(|frame| render_progress(frame, percentage));
                    }
                }
            }

            // returns the new image
            return Some(new_image)
        }
        
        None
    }
}



/// Processes an image into two color spectrums.
pub struct BichromaticEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The first base color of the spectrum being used as a hex value.
    pub base_color_1_hex: String,
    /// The first base color of the spectrum being used as an rgb color.
    pub base_color_1_rgb: Rgb<u8>,
    /// The second base color of the spectrum being used as a hex value.
    pub base_color_2_hex: String,
    /// The second base color of the spectrum being used as an rgb color.
    pub base_color_2_rgb: Rgb<u8>,
    /// The steps used to create the processor.
    pub guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl BichromaticEdit {
    pub fn new(source_image_path: PathBuf) -> BichromaticEdit {
        BichromaticEdit {
            source_image_path,
            base_color_1_rgb: Rgb([0, 0, 0]),
            base_color_1_hex: "none".to_string(),
            base_color_2_rgb: Rgb([0, 0, 0]),
            base_color_2_hex: "none".to_string(),
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 1 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 2 (HEX)".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for BichromaticEdit {
    fn get_color_set(&self) -> String {
        format!("{}-{}", self.base_color_1_hex.clone(), self.base_color_2_hex.clone())
    }
    fn get_current_step_label(&self) -> String {
        self.guide.get_current_label()
    }

    fn get_current_step_input(&self) -> String {
        self.guide.get_current_input()
    }

    fn update_current_step_input(&mut self, new_input: String) {
        self.guide.update_current_input(new_input)
    }

    fn is_current_step_input_valid(&self) -> bool {
        self.guide.is_current_input_valid()
    }

    fn try_finish_current_step(&mut self) {
        if self.is_current_step_input_valid() { self.guide.try_finish_current_step(); }
    }

    fn try_populate(&mut self) {
        if !self.guide.is_ready() { return; }

        let base_color_hex_1_result = self.guide.steps[0].as_hex();
        let base_color_hex_2_result = self.guide.steps[1].as_hex();
        if base_color_hex_1_result.is_some() {
            self.base_color_1_hex = base_color_hex_1_result.unwrap();
            self.base_color_1_rgb = tooling::as_rgb(self.base_color_1_hex.clone()).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = tooling::as_rgb(self.base_color_2_hex.clone()).unwrap();
        }
        else { return; }
        
        self.is_ready = true;
    }

    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let ImageResult::Ok(source_image) = source_image_result {
            // information
            let (width, height) = source_image.dimensions();
            let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
            let mut spectrum = get_spectrum(self.base_color_1_rgb);
            spectrum.extend(get_spectrum(self.base_color_2_rgb));

            // Process pixel by pixel using coordinates
            let pixel_count = width * height;
            let mut pixels_edited = 0;
            for y in 0..height {
                for x in 0..width {
                    let pixel = source_image.get_pixel(x, y).to_rgb();
                    let new_pixel = get_closest_color(&spectrum, pixel);

                    new_image.put_pixel(x, y, new_pixel);

                    pixels_edited += 1;
                    if pixels_edited % 300 == 0 {
                        let percentage = (pixels_edited as f64 / pixel_count as f64) * 100.0;
                        let _ = terminal.draw(|frame| render_progress(frame, percentage));
                    }
                }
            }

            // returns the new image
            return Some(new_image)
        }

        None
    }
}