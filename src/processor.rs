use std::error::Error;
use image::{ImageBuffer, Rgba};

pub enum StepTypes {
    Color,
}



pub enum EditTypes {
    Monochromatic,
    Bichromatic,
}
impl EditTypes {
    pub fn get_steps(&self) -> Vec<EditStep> {
        match self {
            EditTypes::Monochromatic => vec![
                EditStep::new(StepTypes::Color, "Base Color (HEX)".to_string()),
            ],

            EditTypes::Bichromatic => vec![
                EditStep::new(StepTypes::Color, "Base Color 1 (HEX)".to_string()),
                EditStep::new(StepTypes::Color, "Base Color 2 (HEX)".to_string()),
            ],
        }
    }
}



struct EditPath {
    pub edit_type: EditTypes,
    pub steps: Vec<EditStep>,
    pub current_step: usize,
}
impl EditPath {
    fn new(edit_type: EditTypes) -> EditPath {
        EditPath { steps: edit_type.get_steps(), edit_type, current_step: 0 }
    }

    fn get_input(&self) -> String {
        self.steps[self.current_step].get_input()
    }

    fn update_input(&mut self, new_input: String) {
        self.steps[self.current_step].update_input(new_input)
    }

    fn is_step_valid(&self) -> bool {
        match self.steps[self.current_step].step_type {
            StepTypes::Color => oxidation::is_hex(self.get_input()),
        }
    }

    fn finish_step(&mut self) -> bool {
        // not done
        if self.current_step >= self.steps.len() - 2 {
            self.current_step += 1;
            return false;
        }
        // done
        true
    }
}



struct EditStep {
    pub step_type: StepTypes,
    pub label: String,
    pub input: String,
}
impl EditStep {
    fn new(step_type: StepTypes, label: String) -> EditStep {
        EditStep { step_type, label, input: "".to_string() }
    }

    fn get_input(&self) -> String {
        self.input.clone()
    }

    fn update_input(&mut self, new_input: String) {
        self.input = new_input;
    }
}



pub trait EditProcessor {
    fn get_step_input(&self) -> String;
    fn update_step_input(&mut self, new_input: String, path: &mut EditPath);
    fn finish_step(&mut self, base_image: &ImageBuffer<Rgba<u8>, Vec<u8>>, path: &mut EditPath) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        // checks if the path is now complete
        let is_complete = path.finish_step();

        // returns a new image if the path is complete
        if is_complete {
            let result = self.process();
            if let Ok(new_image) = result {
                return Some(new_image);
            }
            // only an error case for invalid processing todo: make a proper error case
            else {
                return None;
            }
        }

        // returns None if the path is not complete
        None
    }

    fn process(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>>;
}



pub struct MonochromaticEdit {
    pub edit_type: EditTypes,
    pub base_image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub base_color: Rgba<u8>,
    pub path: EditPath,
}
impl MonochromaticEdit {
    pub fn new(edit_type: EditTypes, base_image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> MonochromaticEdit {
        MonochromaticEdit { edit_type, base_image, base_color: Rgba([0, 0, 0, 255]), path: EditPath::new(EditTypes::Monochromatic) }
    }
}
impl EditProcessor for MonochromaticEdit {
    fn get_step_input(&self) -> String {
        self.path.get_input()
    }

    fn update_step_input(&mut self, new_input: String, path: &mut EditPath) {
        self.path.update_input(new_input);
    }

    fn process(&self) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>), Box<dyn Error>> {
        let mut new_img = self.base_image.clone();

        // todo implement

        Ok(new_img)
    }
}



pub struct BichromaticEdit {
    pub edit_type: EditTypes,
    pub base_image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub base_color_1: Rgba<u8>,
    pub base_color_2: Rgba<u8>,
    pub path: EditPath,
}
impl BichromaticEdit {
    pub fn new(edit_type: EditTypes, base_image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> BichromaticEdit {
        BichromaticEdit { edit_type, base_image, base_color_1: Rgba([0, 0, 0, 255]), base_color_2: Rgba([0, 0, 0, 255]), path: EditPath::new(EditTypes::Bichromatic) }
    }
}
impl EditProcessor for BichromaticEdit {
    fn get_step_input(&self) -> String {
        self.path.get_input()
    }

    fn update_step_input(&mut self, new_input: String, path: &mut EditPath) {
        self.path.update_input(new_input);
    }

    fn process(&self) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>), Box<dyn Error>> {
        let mut new_img = self.base_image.clone();

        // todo implement

        Ok(new_img)
    }
}

pub mod oxidation {
    use image::Rgba;

    pub fn is_hex(code: String) -> bool {
        let code = code.trim_start_matches('#');
        (code.len() == 3 || code.len() == 6 || code.len() == 8) && code.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn as_rgba(hex: String) -> Option<Rgba<u8>> {
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

        Some(Rgba([r, g, b, a]))
    }

    pub fn get_spectrum(color: Rgba<u8>, steps: usize) -> Vec<Rgba<u8>> {
        // the interpolation information
        let mut base_color = (color[0] as f64, color[1] as f64, color[2] as f64, color[3] as f64);
        let (r_c, g_c, b_c, a_c) = &mut base_color;

        let white = (255.0, 255.0, 255.0, 255.0);
        let (r_w, g_w, b_w, a_w) = &white;

        let black = (0.0, 0.0, 0.0, 255.0);
        let (r_b, g_b, b_b, a_b) = &black;

        let white_increment = (r_w - *r_c, g_w - *g_c, b_w - *b_c, a_w - *a_c);
        let (r_wi, g_wi, b_wi, a_wi) = &white_increment;

        let black_increment = (r_b - *r_c, g_b - *g_c, b_b - *b_c, a_b - *a_c);
        let (r_bi, g_bi, b_bi, a_bi) = &black_increment;


        // the interpolated spectrum
        let mut interpolated_spectrum = Vec::new();
        interpolated_spectrum.push((r_c.clone(), g_c.clone(), b_c.clone(), a_c.clone()));


        // builds the interpolation spectrum from the base color to white
        let (mut r, mut g, mut b, mut a) = base_color.clone();
        for _ in 0..steps {
            r += r_wi;
            g += g_wi;
            b += b_wi;
            a += a_wi;
            interpolated_spectrum.push((r.clone(), g.clone(), b.clone(), a.clone()));
        }


        // builds the interpolation spectrum from the base color to black
        let (mut r, mut g, mut b, mut a) = base_color.clone();
        for _ in 0..steps {
            r += r_bi;
            g += g_bi;
            b += b_bi;
            a += a_bi;
            interpolated_spectrum.push((r.clone(), g.clone(), b.clone(), a.clone()));
        }


        // turns the interpolation spectrum to an Rgba spectrum
        let mut spectrum = Vec::new();
        for interpolated_color in interpolated_spectrum.iter_mut() {
            let (r_i, g_i, b_i, a_i) = interpolated_color;
            let new_color = Rgba([r_i.round() as u8, g_i.round() as u8, b_i.round() as u8, a_i.round() as u8]);
            spectrum.push(new_color);
        }


        // returns the spectrum
        spectrum
    }
}