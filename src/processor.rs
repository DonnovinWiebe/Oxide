pub mod guide;
pub mod pallet;
mod compute;

use std::cell::RefCell;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use rayon::prelude::*;
use crate::processor::compute::*;
use crate::processor::guide::*;
use crate::processor::pallet::*;
use crate::ui::*;

/// The list of available processors.
pub enum Processors {
    Monochromatic,
    AutomaticMonochromatic,
    AutomaticMonochromaticWithAccent,
    Bichromatic,
    BichromaticBlend,
    BichromaticBlendWithAccent,
    Trichromatic,
}
impl Processors {
    /// Returns the name of a given processor type.
    pub fn name(&self) -> String {
        match self {
            Processors::Monochromatic => "Monochromatic".to_string(),
            Processors::AutomaticMonochromatic => "Automatic Monochromatic".to_string(),
            Processors::AutomaticMonochromaticWithAccent => "Automatic Monochromatic with Accent".to_string(),
            Processors::Bichromatic => "Bichromatic".to_string(),
            Processors::BichromaticBlend => "Bichromatic Blend".to_string(),
            Processors::BichromaticBlendWithAccent => "Bichromatic Blend with Accent".to_string(),
            Processors::Trichromatic => "Trichromatic".to_string(),
        }
    }

    /// Returns the number of available processors.
    pub fn number_of_processors() -> usize { 7 }

    /// Gets the processor type that corresponds to a given index.
    pub fn get_processor(selection: usize) -> Processors {
        match selection {
            0 => Processors::Monochromatic,
            1 => Processors::AutomaticMonochromatic,
            2 => Processors::AutomaticMonochromaticWithAccent,
            3 => Processors::Bichromatic,
            4 => Processors::BichromaticBlend,
            5 => Processors::BichromaticBlendWithAccent,
            6 => Processors::Trichromatic,
            _ => panic!("Invalid processor selection: {}", selection),
        }
    }
}



/// Defines an image processor.
pub trait EditProcessor {
    /// Returns the set of colors used in editing the image in order to print them in the editing image filename
    fn get_color_set(&self) -> String;

    /// Returns the input type of the current step.
    fn get_current_step_type(&self) -> ProcessingStepTypes;

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
    /// Returns a new processor ready to be set up.
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

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
            self.base_color_rgb = as_rgb(self.base_color_hex.clone()).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let spectrum = get_1d_spectrum(self.base_color_rgb);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image into a single color spectrum.
pub struct AutomaticMonochromaticWithAccentEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl AutomaticMonochromaticWithAccentEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> AutomaticMonochromaticWithAccentEdit {
        AutomaticMonochromaticWithAccentEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for AutomaticMonochromaticWithAccentEdit {
    fn get_color_set(&self) -> String { // todo get the automatic color selection
        "Automatic with Accent".to_string()
    }

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let base_spectrum = get_1d_spectrum(get_average_color_from_image(&source_image));
            let accent_spectrum = get_1d_spectrum(get_accent_color(&source_image));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_biased(source_image, base_spectrum, accent_spectrum))
        }

        None
    }
}



/// Processes an image into a single color spectrum.
pub struct AutomaticMonochromaticEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl AutomaticMonochromaticEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> AutomaticMonochromaticEdit {
        AutomaticMonochromaticEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for AutomaticMonochromaticEdit {
    fn get_color_set(&self) -> String { // todo get the automatic color selection
        "Automatic".to_string()
    }

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let spectrum = get_1d_spectrum(get_average_color_from_image(&source_image));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
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
    /// Returns a new processor ready to be set up.
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

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
            self.base_color_1_rgb = as_rgb(self.base_color_1_hex.clone()).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(self.base_color_2_hex.clone()).unwrap();
        }
        else { return; }
        
        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let mut spectrum = get_1d_spectrum(self.base_color_1_rgb);
            spectrum.extend(get_1d_spectrum(self.base_color_2_rgb));
            spectrum = remove_duplicates_unordered(spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image into a two-color spectrum blend.
pub struct BichromaticBlendEdit {
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
impl BichromaticBlendEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> BichromaticBlendEdit {
        BichromaticBlendEdit {
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
impl EditProcessor for BichromaticBlendEdit {
    fn get_color_set(&self) -> String {
        format!("{}-{} Blend", self.base_color_1_hex.clone(), self.base_color_2_hex.clone())
    }

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
            self.base_color_1_rgb = as_rgb(self.base_color_1_hex.clone()).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(self.base_color_2_hex.clone()).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let spectrum = get_2d_spectrum(self.base_color_1_rgb, self.base_color_2_rgb);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image into a two-color spectrum blend with an accent color spectrum.
pub struct BichromaticBlendWithAccentEdit {
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
impl BichromaticBlendWithAccentEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> BichromaticBlendWithAccentEdit {
        BichromaticBlendWithAccentEdit {
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
impl EditProcessor for BichromaticBlendWithAccentEdit {
    fn get_color_set(&self) -> String {
        format!("{}-{} Blend with", self.base_color_1_hex.clone(), self.base_color_2_hex.clone())
    }

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
            self.base_color_1_rgb = as_rgb(self.base_color_1_hex.clone()).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(self.base_color_2_hex.clone()).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let base_spectrum = get_2d_spectrum(self.base_color_1_rgb, self.base_color_2_rgb);
            let accent_spectrum = get_1d_spectrum(get_accent_color(&source_image));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_biased(source_image, base_spectrum, accent_spectrum))
        }

        None
    }
}



/// Processes an image into three color spectrums.
pub struct TrichromaticEdit {
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
    /// The third base color of the spectrum being used as a hex value.
    pub base_color_3_hex: String,
    /// The third base color of the spectrum being used as an rgb color.
    pub base_color_3_rgb: Rgb<u8>,
    /// The steps used to create the processor.
    pub guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl TrichromaticEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> TrichromaticEdit {
        TrichromaticEdit {
            source_image_path,
            base_color_1_rgb: Rgb([0, 0, 0]),
            base_color_1_hex: "none".to_string(),
            base_color_2_rgb: Rgb([0, 0, 0]),
            base_color_2_hex: "none".to_string(),
            base_color_3_rgb: Rgb([0, 0, 0]),
            base_color_3_hex: "none".to_string(),
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 1 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 2 (HEX)".to_string()),
                ProcessingStep::new(ProcessingStepTypes::Color, "Base Color 3 (HEX)".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for TrichromaticEdit {
    fn get_color_set(&self) -> String {
        format!("{}-{}-{}", self.base_color_1_hex.clone(), self.base_color_2_hex.clone(), self.base_color_3_hex.clone())
    }

    fn get_current_step_type(&self) -> ProcessingStepTypes {
        self.guide.get_current_step_type()
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
        let base_color_hex_3_result = self.guide.steps[2].as_hex();
        if base_color_hex_1_result.is_some() {
            self.base_color_1_hex = base_color_hex_1_result.unwrap();
            self.base_color_1_rgb = as_rgb(self.base_color_1_hex.clone()).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(self.base_color_2_hex.clone()).unwrap();
        }
        else { return; }
        if base_color_hex_3_result.is_some() {
            self.base_color_3_hex = base_color_hex_3_result.unwrap();
            self.base_color_3_rgb = as_rgb(self.base_color_3_hex.clone()).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let mut spectrum = get_1d_spectrum(self.base_color_1_rgb);
            spectrum.extend(get_1d_spectrum(self.base_color_2_rgb));
            spectrum.extend(get_1d_spectrum(self.base_color_3_rgb));
            spectrum = remove_duplicates_unordered(spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}