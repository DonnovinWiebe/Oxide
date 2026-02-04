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
    BichromaticWithAccent,
    Trichromatic,
    VolcanicCrater,
    RedRocks,
    DeepestAfrica,
    ArcticWilderness,
    Iceland,
    EnglishOaks,
    WheatField,
    SouthAmericanJungle,
    EuropeanIslands,
    ColorfulIslands,
}
impl Processors {
    /// Returns the name of a given processor type.
    pub fn name(&self) -> String {
        match self {
            Processors::Monochromatic =>                    "Monochromatic".to_string(),
            Processors::AutomaticMonochromatic =>           "Automatic Monochromatic".to_string(),
            Processors::AutomaticMonochromaticWithAccent => "Automatic Monochromatic with Accent".to_string(),
            Processors::Bichromatic =>                      "Bichromatic".to_string(),
            Processors::BichromaticWithAccent =>            "Bichromatic with Accent".to_string(),
            Processors::Trichromatic =>                     "Trichromatic".to_string(),
            Processors::VolcanicCrater =>                   "Volcanic Crater".to_string(),
            Processors::RedRocks =>                         "Red Rocks".to_string(),
            Processors::DeepestAfrica =>                    "Deepest Africa".to_string(),
            Processors::ArcticWilderness =>                 "Arctic Wilderness".to_string(),
            Processors::Iceland =>                          "Iceland".to_string(),
            Processors::EnglishOaks =>                      "English Oaks".to_string(),
            Processors::WheatField =>                       "Wheat Field".to_string(),
            Processors::SouthAmericanJungle =>              "South American Jungle".to_string(),
            Processors::EuropeanIslands =>                  "European Islands".to_string(),
            Processors::ColorfulIslands =>                  "Colorful Islands".to_string(),

        }
    }

    /// Returns the number of available processors.
    pub fn number_of_processors() -> usize { 16 }

    /// Gets the processor type that corresponds to a given index.
    pub fn get_processor(selection: usize) -> Processors {
        match selection {
            0 => Processors::Monochromatic,
            1 => Processors::AutomaticMonochromatic,
            2 => Processors::AutomaticMonochromaticWithAccent,
            3 => Processors::Bichromatic,
            4 => Processors::BichromaticWithAccent,
            5 => Processors::Trichromatic,
            6 => Processors::VolcanicCrater,
            7 => Processors::RedRocks,
            8 => Processors::DeepestAfrica,
            9 => Processors::ArcticWilderness,
            10 => Processors::Iceland,
            11 => Processors::EnglishOaks,
            12 => Processors::WheatField,
            13 => Processors::SouthAmericanJungle,
            14 => Processors::EuropeanIslands,
            15 => Processors::ColorfulIslands,
            _ => panic!("Invalid processor selection: {}", selection),
        }
    }
}



/// Defines an image processor.
pub trait EditProcessor {
    /// Returns the set of colors used in editing the image in order to print them in the editing image filename
    fn get_descriptor(&self, name: String) -> String;

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
    fn get_descriptor(&self, name: String) -> String {
        format!("{} {}", name, self.base_color_hex.clone())
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
            self.base_color_rgb = as_rgb(&self.base_color_hex).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let mut spectrum = get_line_spectrum(&self.base_color_rgb);
            spectrum = condense_color_pallet(&spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image into a single color spectrum automatically.
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
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let mut spectrum = get_line_spectrum(&get_average_color_from_image(&source_image));
            spectrum = condense_color_pallet(&spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image into a single color spectrum with an accent color spectrum automatically.
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
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let mut base_spectrum = get_line_spectrum(&get_average_color_from_image(&source_image));
            base_spectrum = condense_color_pallet(&base_spectrum);
            let mut accent_spectrum = get_line_spectrum(&get_accent_color(&source_image));
            accent_spectrum = condense_color_pallet(&accent_spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_biased(source_image, base_spectrum, accent_spectrum))
        }

        None
    }
}



/// Processes an image into a two-color spectrum blend.
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
    fn get_descriptor(&self, name: String) -> String {
        format!("{} {}-{}", name, self.base_color_1_hex.clone(), self.base_color_2_hex.clone())
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
            self.base_color_1_rgb = as_rgb(&self.base_color_1_hex).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(&self.base_color_2_hex).unwrap();
        }
        else { return; }
        
        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let line_spectrum_1 = get_line_spectrum(&self.base_color_1_rgb);
            let line_spectrum_2 = get_line_spectrum(&self.base_color_2_rgb);
            let mut spectrum = get_plane_spectrum(&line_spectrum_1, &line_spectrum_2);
            spectrum = condense_color_pallet(&spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image into a two-color spectrum blend with an accent color spectrum.
pub struct BichromaticWithAccentEdit {
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
impl BichromaticWithAccentEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> BichromaticWithAccentEdit {
        BichromaticWithAccentEdit {
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
impl EditProcessor for BichromaticWithAccentEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{} {}-{}", name, self.base_color_1_hex.clone(), self.base_color_2_hex.clone())
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
            self.base_color_1_rgb = as_rgb(&self.base_color_1_hex).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(&self.base_color_2_hex).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let mut base_spectrum = get_plane_spectrum(&get_line_spectrum(&self.base_color_1_rgb), &get_line_spectrum(&self.base_color_2_rgb));
            base_spectrum = condense_color_pallet(&base_spectrum);
            let mut accent_spectrum = get_line_spectrum(&get_accent_color(&source_image));
            accent_spectrum = condense_color_pallet(&accent_spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_biased(source_image, base_spectrum, accent_spectrum))
        }

        None
    }
}



/// Processes an image into a three-color spectrum blend.
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
    fn get_descriptor(&self, name: String) -> String {
        format!("{} {}-{}-{}", name, self.base_color_1_hex.clone(), self.base_color_2_hex.clone(), self.base_color_3_hex.clone())
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
            self.base_color_1_rgb = as_rgb(&self.base_color_1_hex).unwrap();
        }
        else { return; }
        if base_color_hex_2_result.is_some() {
            self.base_color_2_hex = base_color_hex_2_result.unwrap();
            self.base_color_2_rgb = as_rgb(&self.base_color_2_hex).unwrap();
        }
        else { return; }
        if base_color_hex_3_result.is_some() {
            self.base_color_3_hex = base_color_hex_3_result.unwrap();
            self.base_color_3_rgb = as_rgb(&self.base_color_3_hex).unwrap();
        }
        else { return; }

        self.is_ready = true;
    }

    fn try_process(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if !self.is_ready { return None; }

        let source_image_result = image::open(self.source_image_path.clone());
        if let Ok(source_image) = source_image_result {
            let _ = terminal.draw(|frame| render_loading(frame, "Loading colors...".to_string()));
            let line_spectrum_1 = get_line_spectrum(&self.base_color_1_rgb);
            let line_spectrum_2 = get_line_spectrum(&self.base_color_2_rgb);
            let line_spectrum_3 = get_line_spectrum(&self.base_color_3_rgb);
            let mut spectrum = get_web_spectrum(&vec![line_spectrum_1, line_spectrum_2, line_spectrum_3]);
            spectrum = condense_color_pallet(&spectrum);

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, spectrum))
        }

        None
    }
}



/// Processes an image with a volcanic crater themed pallet.
pub struct VolcanicCraterEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl VolcanicCraterEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> VolcanicCraterEdit {
        VolcanicCraterEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for VolcanicCraterEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::volcanic_crater()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with a red rocks themed pallet.
pub struct RedRocksEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl RedRocksEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> RedRocksEdit {
        RedRocksEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for RedRocksEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::red_rocks()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with a deepest africa themed pallet.
pub struct DeepestAfricaEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl DeepestAfricaEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> DeepestAfricaEdit {
        DeepestAfricaEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for DeepestAfricaEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::deepest_africa()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with an arctic wilderness themed pallet.
pub struct ArcticWildernessEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl ArcticWildernessEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> ArcticWildernessEdit {
        ArcticWildernessEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for ArcticWildernessEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::arctic_wilderness()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with an iceland themed pallet.
pub struct IcelandEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl IcelandEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> IcelandEdit {
        IcelandEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for IcelandEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::iceland()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with an english oaks themed pallet.
pub struct EnglishOaksEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl EnglishOaksEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> EnglishOaksEdit {
        EnglishOaksEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for EnglishOaksEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::english_oaks()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with a wheat field themed pallet.
pub struct WheatFieldEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl WheatFieldEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> WheatFieldEdit {
        WheatFieldEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for WheatFieldEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::wheat_field()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with a south american jungle themed pallet.
pub struct SouthAmericanJungleEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl SouthAmericanJungleEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> SouthAmericanJungleEdit {
        SouthAmericanJungleEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for SouthAmericanJungleEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::south_american_jungle()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with a european islands themed pallet.
pub struct EuropeanIslandsEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl EuropeanIslandsEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> EuropeanIslandsEdit {
        EuropeanIslandsEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for EuropeanIslandsEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::european_islands()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}



/// Processes an image with a colorful islands themed pallet.
pub struct ColorfulIslandsEdit {
    /// The path of the original image to be processed.
    source_image_path: PathBuf,
    /// The steps used to create the processor.
    guide: ProcessingGuide,
    /// Tracks if the processor is ready.
    is_ready: bool,
}
impl ColorfulIslandsEdit {
    /// Returns a new processor ready to be set up.
    pub fn new(source_image_path: PathBuf) -> ColorfulIslandsEdit {
        ColorfulIslandsEdit {
            source_image_path,
            guide: ProcessingGuide::new(vec![
                ProcessingStep::new(ProcessingStepTypes::NoInput, "Press Enter".to_string()),
            ]),
            is_ready: false,
        }
    }
}
impl EditProcessor for ColorfulIslandsEdit {
    fn get_descriptor(&self, name: String) -> String {
        format!("{}", name)
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
            let web_spectrum = get_web_spectrum(&get_line_spectrums(&pallets::colorful_islands()));

            let _ = terminal.draw(|frame| render_loading(frame, "Processing...".to_string()));
            return Some(process_evenly(source_image, web_spectrum));
        }

        None
    }
}