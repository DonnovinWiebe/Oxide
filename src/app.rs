use std::path::{Path, PathBuf};
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::prelude::*;
use crate::processor;
use crate::ui::{render, Instruction};
use std::io::{Error, Result};
use std::string::String;
use image::{ImageBuffer, Rgba};
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
    pub source_image_paths: Vec<PathBuf>,
    pub output_directory: PathBuf,
    current_image_path_selection: usize, // by index
    selected_image_path: Option<PathBuf>,
    // processor selection
    pub current_processor_selection: usize,
    pub selected_processor: Option<Box<dyn EditProcessor>>,
    // new image
    pub new_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

impl App {
    pub fn new(source_directory: PathBuf, output_directory: PathBuf, source_image_paths: Vec<PathBuf>) -> App {
        let mut app = App {
            current_page: Pages::Launching,
            source_directory,
            source_image_paths,
            output_directory,
            current_image_path_selection: 0,
            selected_image_path: None,
            current_processor_selection: 0,
            selected_processor: None,
            new_image: None,
        };

        app.update_selected_image_path();
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

    fn update_selected_image_path(&mut self) {
        if self.source_image_paths.is_empty() {
            self.selected_image_path = None;
        }
        else {
            self.selected_image_path = Some(self.source_image_paths[self.current_image_path_selection].clone());
        }
    }

    pub fn select_next_source_image_path(&mut self) {
        if self.current_image_path_selection >= self.source_image_paths.len() - 1 {
            self.current_image_path_selection = 0;
        }
        else {
            self.current_image_path_selection += 1;
        }
    }

    pub fn select_previous_source_image_path(&mut self) {
        if self.current_image_path_selection == 0 {
            self.current_image_path_selection = self.source_image_paths.len() - 1;
        }
        else {
            self.current_image_path_selection -= 1;
        }
    }

    pub fn print_selected_image_filename(&self) -> String {
        self.selected_image_path.clone().unwrap().file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn select_next_processor(&mut self) {
        if self.current_processor_selection >= Processors::number_of_processors() - 1 {
            self.current_processor_selection = 0;
        } else {
            self.current_processor_selection += 1;
        }

        self.update_selected_image_path();
    }

    pub fn select_previous_processor(&mut self) {
        if self.current_processor_selection == 0 {
            self.current_processor_selection = Processors::number_of_processors() - 1;
        } else {
            self.current_processor_selection -= 1;
        }

        self.update_selected_image_path();
    }

    pub fn reset(&mut self) {
        self.current_page = Pages::Launching;
        self.current_image_path_selection = 0;
        self.selected_image_path = None;
        self.current_processor_selection = 0;
        self.selected_processor = None;
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
                            self.select_next_source_image_path();
                        }
                        if key.code == Instruction::select_previous().keybind {
                            self.select_previous_source_image_path();
                        }
                        if key.code == Instruction::confirm_instruction().keybind {
                            // cannot continue if there are no images to edit, and thus preventing downstream unwrap errors
                            // from here self.selected_image_path is guaranteed to be set
                            if self.source_image_paths.is_empty() { continue; }

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
                            // from here self.selected_processor is guaranteed to be set
                            let selected_processor = Processors::get_processor(self.current_processor_selection);
                            match selected_processor {
                                Processors::Monochromatic => {
                                    self.selected_processor = Some(Box::new(MonochromaticEdit::new(self.selected_image_path.clone().unwrap())));
                                }
                                Processors::Bichromatic => {
                                    self.selected_processor = Some(Box::new(BichromaticEdit::new(self.selected_image_path.clone().unwrap())));
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
                        // checks if processor is valid
                        if let Some(processor) = &mut self.selected_processor {
                            // trying to finish step
                            if key.code == Instruction::confirm_instruction().keybind {
                                processor.try_finish_current_step();
                                processor.try_populate();
                                self.new_image = processor.try_process();

                                if let Some(new_image) = self.new_image.as_ref() {
                                    let source_directory = self.selected_image_path.clone().unwrap();
                                    let output_directory = self.output_directory.clone();
                                    let filename = source_directory.file_name().unwrap();
                                    let output_path = output_directory.join(filename);
                                    new_image.save(&output_path).expect("Could not save image!");

                                    self.current_page = Pages::Finished;
                                }
                                else { continue; }
                            }

                            // updating the current guide step input
                            let new_input = term_tools::keypad(&processor.get_current_step_input(), key);
                            processor.update_current_step_input(new_input);

                            // trying to reset
                            if key.code == Instruction::reset_instruction().keybind {
                                self.reset();
                            }
                        }

                        // resets if processor is None
                        else { self.reset(); }
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