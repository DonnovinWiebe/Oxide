pub mod guide;
pub mod tooling;

use std::error::Error;
use image::{ImageBuffer, Rgba};
use crate::processor::guide::{ProcessingGuide, ProcessingStep, ProcessingStepTypes};



/// Defines an image processor.
pub trait EditProcessor {
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
    base_image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// The base color of the spectrum being used as a hex value.
    base_color_hex: String,
    /// The base color of the spectrum being used as an rgba color.
    base_color_rgba: Rgba<u8>,
    /// The steps used to create the processor.
    path: ProcessingGuide,
}
impl MonochromaticEdit {
    pub fn new(base_image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> MonochromaticEdit {
        MonochromaticEdit {base_image,
            base_color_rgba: Rgba([0, 0, 0, 255]),
            base_color_hex: "none".to_string(),
            path: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::ColorSmoothing, "Color Smoothing (whole number)".to_string()),
            ])
        }
    }
}
impl EditProcessor for MonochromaticEdit {
    fn get_current_step_label(&self) -> String {
        self.path.get_current_label()
    }

    fn get_current_step_input(&self) -> String {
        self.path.get_current_input()
    }

    fn update_current_step_input(&mut self, new_input: String) {
        self.path.update_current_input(new_input)
    }

    fn is_current_step_input_valid(&self) -> bool {
        self.path.is_current_input_valid()
    }

    fn finish_current_step(&mut self) -> bool {
        self.path.finish_step()
    }

    fn try_process(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        if self.is_current_step_input_valid() {
            todo!()
        }
        todo!()
    }
}



/// Processes an image into two color spectrums.
pub struct BichromaticEdit {
    /// The original image to be processed.
    pub base_image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// The first base color of the spectrum being used as a hex value.
    pub base_color_1_hex: String,
    /// The first base color of the spectrum being used as an rgba color.
    pub base_color_1_rgba: Rgba<u8>,
    /// The second base color of the spectrum being used as a hex value.
    pub base_color_2_hex: String,
    /// The second base color of the spectrum being used as an rgba color.
    pub base_color_2_rgba: Rgba<u8>,
    /// The steps used to create the processor.
    pub path: ProcessingGuide,
}
impl BichromaticEdit {
    pub fn new(base_image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> BichromaticEdit {
        BichromaticEdit {
            base_image,
            base_color_1_rgba: Rgba([0, 0, 0, 255]),
            base_color_1_hex: "none".to_string(),
            base_color_2_rgba: Rgba([0, 0, 0, 255]),
            base_color_2_hex: "none".to_string(),
            path: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 1 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 2 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::ColorSmoothing, "Color Smoothing (whole number)".to_string()),
            ])
        }
    }
}
impl EditProcessor for BichromaticEdit {
    fn get_current_step_label(&self) -> String {
        self.path.get_current_label()
    }

    fn get_current_step_input(&self) -> String {
        self.path.get_current_input()
    }

    fn update_current_step_input(&mut self, new_input: String) {
        self.path.update_current_input(new_input)
    }

    fn is_current_step_input_valid(&self) -> bool {
        self.path.is_current_input_valid()
    }

    fn finish_current_step(&mut self) -> bool {
        self.path.finish_step()
    }

    fn try_process(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
        if self.is_current_step_input_valid() {
            todo!()
        }
        todo!()
    }
}