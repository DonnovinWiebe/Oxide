use std::path::PathBuf;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::prelude::*;
use crate::processor;
use crate::ui::{render, Instruction};
use std::io::{Error, Result};
use std::string::String;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::processor::{BichromaticEdit, EditProcessor, MonochromaticEdit, Processors};
use crate::processor::guide::{ProcessingGuide, ProcessingStep};
use crate::processor::Processors::Monochromatic;

#[derive(Copy, Clone)]
pub enum Pages {
    Launching,
    SelectingImageSource,
    SelectingProcessingType,
    Preprocessing,
    Finished,
}



pub struct App {
    // current page
    pub current_page: Pages,
    // image selection
    pub source_directory: PathBuf,
    pub source_images: Vec<PathBuf>,
    output_directory: PathBuf,
    current_image_selection: usize, // 0-sources.len() - 1 = single index; sources.len() = all sources
    selected_images: Vec<PathBuf>,
    // processor selection
    pub current_processor_selection: usize,
    pub processor: Box<dyn EditProcessor>,

}

impl App {
    pub fn new(source_directory: PathBuf, output_directory: PathBuf, source_images: Vec<PathBuf>) -> App {
        if source_images.is_empty() { panic!("No images provided!"); }
        
        let source_path = source_images[0].clone();
        
        let mut app = App {
            current_page: Pages::Launching,
            source_directory,
            source_images,
            output_directory,
            current_image_selection: 0,
            selected_images: Vec::new(),
            current_processor_selection: 0,
            processor: Box::new(MonochromaticEdit::new(source_path)),
        };

        app.update_selected_images();
        app
    }

    pub fn current_page_name(&self) -> String {
        match self.current_page {
            Pages::Launching => "Launching".to_string(),
            Pages::SelectingImageSource => "Selecting Image Source".to_string(),
            Pages::SelectingProcessingType => "Selecting Processing Type".to_string(),
            Pages::Preprocessing => "Preprocessing".to_string(),
            Pages::Finished => "Finished".to_string(),
        }
    }

    pub fn update_selected_images(&mut self) {
        // clears the selected images
        self.selected_images.clear();

        // adds the current image
        if self.current_image_selection < self.source_images.len() {
            self.selected_images.push(self.source_images[self.current_image_selection].clone());
        }

        // adds all images
        else {
            self.selected_images.extend(self.source_images.iter().cloned());
        }
        
        // updates the processor
        let base_image;
        if let Err(_) = image::open(self.selected_images[0].clone()) { base_image = None; }
        else { base_image = Some(image::open(self.selected_images[0].clone()).unwrap().to_rgba8()); }
        self.processor.update_base_image(base_image);
    }

    pub fn select_next_source_image(&mut self) {
        // resets if the source is empty
        if self.source_images.is_empty() {
            self.current_image_selection = 0;
            self.selected_images.clear();
            return;
        }

        // selects the next image (selects each image by index, then all at len(), then wraps to 0)
        if self.current_image_selection >= self.source_images.len() {
            self.current_image_selection = 0;
        } else {
            self.current_image_selection += 1;
        }

        // updates the selected images
        self.update_selected_images();
    }

    pub fn select_previous_source_image(&mut self) {
        // resets if the source is empty
        if self.source_images.is_empty() {
            self.current_image_selection = 0;
            self.selected_images.clear();
            return;
        }

        // selects the previous image (selects each image by index, then wraps to all at len(), then back through the indices backwards)
        if self.current_image_selection == 0 {
            self.current_image_selection = self.source_images.len();
        } else {
            self.current_image_selection -= 1;
        }

        // updates the selected images
        self.update_selected_images();
    }

    pub fn print_selected_images(&self) -> String {
        // returns none if no images are selected (no local images or error case)
        if self.selected_images.is_empty() {
            return "None".to_string();
        }

        // returns all if all images are selected
        if self.current_image_selection >= self.source_images.len() {
            return "All".to_string();
        }

        // returns error if more than one image is selected
        if self.selected_images.len() > 1 {
            return "Error selecting single image".to_string();
        }

        // returns the selected image
        self.selected_images[0].to_string_lossy().to_string()
    }

    pub fn select_next_processor(&mut self) {
        if self.current_processor_selection >= Processors::number_of_processors() - 1 {
            self.current_processor_selection = 0;
        } else {
            self.current_processor_selection += 1;
        }
    }

    pub fn select_previous_processor(&mut self) {
        if self.current_processor_selection == 0 {
            self.current_processor_selection = Processors::number_of_processors() - 1;
        } else {
            self.current_processor_selection -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.current_page = Pages::SelectingImageSource;
        self.source_images = Vec::new();
        self.current_image_selection = 0;
        self.selected_images = Vec::new();
        self.current_processor_selection = 0;
    }



    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> where Error: From<<B as Backend>::Error> {
        // running
        loop {
            // pre-render
            let footer_height = Instruction::get_instructions_for(&self.current_page).len() as u16 + 2;
            let header_height = 4;
            let page_height = terminal.size()?.height - footer_height - header_height; // todo check if necessary



            // rendering
            terminal.draw(|frame| render(frame, self))?;



            // getting input
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release { continue; }

                match self.current_page {
                    Pages::Launching => {
                        self.current_page = Pages::SelectingImageSource;
                    }

                    Pages::SelectingImageSource => {
                        if key.code == Instruction::select_next().keybind {
                            self.select_next_source_image();
                        }
                        if key.code == Instruction::select_previous().keybind {
                            self.select_previous_source_image();
                        }
                        if key.code == Instruction::confirm_instruction().keybind {
                            self.current_page = Pages::SelectingProcessingType;
                        }
                        if key.code == Instruction::reset_instruction().keybind {
                            self.reset();
                        }
                        if key.code == Instruction::quit_instruction().keybind {
                            break;
                        }
                    }

                    Pages::SelectingProcessingType => {
                        if key.code == Instruction::select_next().keybind {
                            self.select_next_processor();
                        }
                        if key.code == Instruction::select_previous().keybind {
                            self.select_previous_processor();
                        }
                        if key.code == Instruction::confirm_instruction().keybind {
                            let selected_processor = Processors::get_processor(self.current_processor_selection);
                            match selected_processor {
                                Processors::Monochromatic => {
                                    self.processor = Box::new(MonochromaticEdit::new(self.selected_images[0].clone()));
                                }
                                Processors::Bichromatic => {
                                    self.processor = Box::new(BichromaticEdit::new(self.selected_images[0].clone()));
                                }
                            }
                            
                            self.current_page = Pages::Preprocessing;
                        }
                        if key.code == Instruction::reset_instruction().keybind {
                            self.reset();
                        }
                        if key.code == Instruction::quit_instruction().keybind {
                            break;
                        }
                    }

                    Pages::Preprocessing => {
                        if key.code == Instruction::confirm_instruction().keybind {
                            let finished = self.processor.finish_current_step();
                            if finished {
                                let result = self.processor.try_process();
                                if result.is_ok() {
                                    let source_directory = self.source_images[0].clone();
                                    let output_directory = self.output_directory.clone();
                                    let filename = source_directory.file_name().unwrap();
                                    let output_path = output_directory.join(filename);
                                    result.unwrap().save(&output_path).expect("Could not save image!");
                                    
                                    self.current_page = Pages::Finished;
                                }
                                else { continue; }
                            }
                            else { continue; }
                        }
                        
                        if key.code == Instruction::reset_instruction().keybind {
                            self.reset();
                        }
                        
                        self.processor.update_current_step_input(term_tools::keypad(&self.processor.get_current_step_input(), key));
                    }

                    Pages::Finished => {
                        if key.code == Instruction::run_again_instruction().keybind {
                            self.reset();
                            continue;
                        }
                        if key.code == Instruction::quit_instruction().keybind {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}



pub mod term_tools {
    use ratatui::crossterm::event;
    use ratatui::crossterm::event::{KeyCode, KeyEvent};

    pub fn numpad(field: &str, input: KeyEvent) -> String {
        if input.kind == event::KeyEventKind::Release { return field.to_string(); }

        let mut field = field.to_string();
        match input.code {
            KeyCode::Backspace => {
                if field.is_empty() { return field; }
                field.remove(field.len() - 1);
            }
            KeyCode::Char(char) => {
                match char {
                    '0'..='9' => field.push(char),
                    '.' => { if !field.contains('.') { field.push(char); } }
                    _ => {}
                }
            }
            _ => {}
        }

        field
    }

    pub fn keypad(field: &str, input: KeyEvent) -> String {
        if input.kind == event::KeyEventKind::Release {}

        let mut field = field.to_string();
        match input.code {
            KeyCode::Backspace => {
                if field.is_empty() { return field; }
                field.remove(field.len() - 1);
            }
            KeyCode::Char(char) => { field.push(char); }
            _ => {}
        }
        field
    }
}