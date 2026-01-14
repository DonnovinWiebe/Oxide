pub mod guide;
pub mod tooling;

use std::error::Error;
use std::path::PathBuf;
use image::{GenericImageView, ImageBuffer, Rgba};
use crate::processor::guide::{ProcessingGuide, ProcessingStep, ProcessingStepTypes};



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
    fn update_base_image(&mut self, new_base_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>);

    /// Returns the label of the current step.
    fn get_current_step_label(&self) -> String;

    /// Returns the input of the current step.
    fn get_current_step_input(&self) -> String;

    /// Updates the input of the current step.
    fn update_current_step_input(&mut self, new_input: String);

    /// Checks if the input is valid for the current step.
    fn is_current_step_input_valid(&self) -> bool;

    /// Advances the guide to the next step (if the input is valid) and returns whether or not the guide is finished.
    fn finish_current_step(&mut self) -> bool;

    /// Processes the image and returns the new image.
    fn try_process(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>>;
}



/// Processes an image into a single color spectrum.
pub struct MonochromaticEdit {
    /// The original image to be processed.
    base_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    /// The base color of the spectrum being used as a hex value.
    base_color_hex: String,
    /// The base color of the spectrum being used as an rgba color.
    base_color_rgba: Rgba<u8>,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
}
impl MonochromaticEdit {
    pub fn new(source_path: PathBuf) -> MonochromaticEdit {
        let base_image;
        if let Err(_) = image::open(source_path.clone()) { base_image = None; }
        else { base_image = Some(image::open(source_path.clone()).unwrap().to_rgba8()); }
        MonochromaticEdit {base_image,
            base_color_rgba: Rgba([0, 0, 0, 255]),
            base_color_hex: "none".to_string(),
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::ColorSmoothing, "Color Smoothing (whole number)".to_string()),
            ])
        }
    }
}
impl EditProcessor for MonochromaticEdit {
    fn update_base_image(&mut self, new_base_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>) {
        self.base_image = new_base_image;
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

    fn finish_current_step(&mut self) -> bool {
        self.guide.finish_step()
    }

    fn try_process(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        if let Some(base_image) = &self.base_image {
            let (width, height) = base_image.dimensions();

            let mut new_image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

            // Process pixel by pixel using coordinates
            for y in 0..height {
                for x in 0..width {
                    let pixel = base_image.get_pixel(x, y);

                    // Example: invert colors
                    let new_pixel = Rgba([
                        255 - pixel[0],
                        255 - pixel[1],
                        255 - pixel[2],
                        pixel[3],
                    ]);

                    new_image.put_pixel(x, y, new_pixel);
                }
            }

            return Ok(new_image)
        }
        else {
            return Err("No image provided!".into());
        }
    }
}



/// Processes an image into two color spectrums.
pub struct BichromaticEdit {
    /// The original image to be processed.
    pub base_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    /// The first base color of the spectrum being used as a hex value.
    pub base_color_1_hex: String,
    /// The first base color of the spectrum being used as an rgba color.
    pub base_color_1_rgba: Rgba<u8>,
    /// The second base color of the spectrum being used as a hex value.
    pub base_color_2_hex: String,
    /// The second base color of the spectrum being used as an rgba color.
    pub base_color_2_rgba: Rgba<u8>,
    /// The steps used to create the processor.
    pub guide: ProcessingGuide,
}
impl BichromaticEdit {
    pub fn new(source_path: PathBuf) -> BichromaticEdit {
        let base_image;
        if let Err(_) = image::open(source_path.clone()) { base_image = None; }
        else { base_image = Some(image::open(source_path.clone()).unwrap().to_rgba8()); }
        BichromaticEdit {
            base_image,
            base_color_1_rgba: Rgba([0, 0, 0, 255]),
            base_color_1_hex: "none".to_string(),
            base_color_2_rgba: Rgba([0, 0, 0, 255]),
            base_color_2_hex: "none".to_string(),
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 1 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 2 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::ColorSmoothing, "Color Smoothing (whole number)".to_string()),
            ])
        }
    }
}
impl EditProcessor for BichromaticEdit {
    fn update_base_image(&mut self, new_base_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>) {
        self.base_image = new_base_image;
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

    fn finish_current_step(&mut self) -> bool {
        self.guide.finish_step()
    }

    fn try_process(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        if let Some(base_image) = &self.base_image {
            let (width, height) = base_image.dimensions();

            let mut new_image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

            // Process pixel by pixel using coordinates
            for y in 0..height {
                for x in 0..width {
                    let pixel = base_image.get_pixel(x, y);

                    // Example: invert colors
                    let new_pixel = Rgba([
                        255 - pixel[0],
                        255 - pixel[1],
                        255 - pixel[2],
                        pixel[3],
                    ]);

                    new_image.put_pixel(x, y, new_pixel);
                }
            }

            return Ok(new_image)
        }
        else {
            return Err("No image provided!".into());
        }
    }
}