use std::fs;
use std::path::PathBuf;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::prelude::*;
use crate::ui::{render_current_page, Instruction};
use std::io::{Error, Result};
use std::string::String;
use image::{ImageBuffer, Rgb};
use img_parts::ImageEXIF;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::processor::*;
use img_parts::jpeg::Jpeg;
use img_parts::png::Png;

/// The list of pages in the application.
#[derive(Copy, Clone)]
pub enum Pages {
    Launching,
    SelectingImageSource,
    SelectingProcessingType,
    Preprocessing,
    Finished,
}



/// The application state container.
pub struct App {
    /// The current page.
    pub current_page: Pages,
    /// The source directory for input images.
    pub source_directory: PathBuf,
    /// The list of paths to images in the source directory.
    pub source_image_paths: Vec<PathBuf>,
    /// The output directory for edited images.
    pub output_directory: PathBuf,
    /// The current image selection used during selection.
    current_image_path_selection: usize,
    /// The selected image path.
    selected_image_path: Option<PathBuf>,
    /// The current processor selection used during selection.
    pub current_processor_selection: usize,
    /// The selected processor.
    pub selected_processor: Option<Box<dyn EditProcessor>>,
    /// The new image for editing.
    pub new_image: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
}
impl App {
    /// Returns a new application state container.
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

    /// Returns the name of the current page.
    pub fn current_page_name(&self) -> String {
        match self.current_page {
            Pages::Launching => "Launching".to_string(),
            Pages::SelectingImageSource => "Selecting Image Source".to_string(),
            Pages::SelectingProcessingType => "Selecting Processing Type".to_string(),
            Pages::Preprocessing => "Preprocessing".to_string(),
            Pages::Finished => "Finished".to_string(),
        }
    }

    /// Updates the selected image path based on the current selection.
    fn update_selected_image_path(&mut self) {
        if self.source_image_paths.is_empty() {
            self.selected_image_path = None;
        }
        else {
            self.selected_image_path = Some(self.source_image_paths[self.current_image_path_selection].clone());
        }
    }

    /// Selects the next image path in the source list.
    pub fn select_next_source_image_path(&mut self) {
        if self.current_image_path_selection >= self.source_image_paths.len() - 1 {
            self.current_image_path_selection = 0;
        }
        else {
            self.current_image_path_selection += 1;
        }
    }

    /// Selects the previous image path in the source list.
    pub fn select_previous_source_image_path(&mut self) {
        if self.current_image_path_selection == 0 {
            self.current_image_path_selection = self.source_image_paths.len() - 1;
        }
        else {
            self.current_image_path_selection -= 1;
        }
    }

    /// Returns the selected image filename.
    pub fn print_selected_image_filename(&self) -> String {
        if self.source_image_paths.is_empty() { return "Error: No images to edit".to_string() }

        self.source_image_paths[self.current_image_path_selection].clone().file_name().unwrap().to_string_lossy().to_string()
    }

    /// Selects the next processor in the list.
    pub fn select_next_processor(&mut self) {
        if self.current_processor_selection >= Processors::number_of_processors() - 1 {
            self.current_processor_selection = 0;
        } else {
            self.current_processor_selection += 1;
        }

        self.update_selected_image_path();
    }

    /// Selects the previous processor in the list.
    pub fn select_previous_processor(&mut self) {
        if self.current_processor_selection == 0 {
            self.current_processor_selection = Processors::number_of_processors() - 1;
        } else {
            self.current_processor_selection -= 1;
        }

        self.update_selected_image_path();
    }

    /// Resets the application to the launching page and resets the state.
    pub fn reset(&mut self) {
        self.current_page = Pages::SelectingImageSource;
        self.current_image_path_selection = 0;
        self.selected_image_path = None;
        self.current_processor_selection = 0;
        self.selected_processor = None;
    }



    /// Runs the application.
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> where Error: From<<B as Backend>::Error> {
        // running
        loop {
            // rendering
            terminal.clear()?;
            terminal.draw(|frame| render_current_page(frame, self))?;



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

                            self.update_selected_image_path();
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
                                Processors::BichromaticBlend => {
                                    self.selected_processor = Some(Box::new(BichromaticBlendEdit::new(self.selected_image_path.clone().unwrap())));
                                }
                                Processors::Trichromatic => {
                                    self.selected_processor = Some(Box::new(TrichromaticEdit::new(self.selected_image_path.clone().unwrap())));
                                }
                                Processors::AutomaticMonochromatic => {
                                    self.selected_processor = Some(Box::new(AutomaticMonochromaticEdit::new(self.selected_image_path.clone().unwrap())));
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
                        // checks if the processor is valid
                        if let Some(processor) = &mut self.selected_processor {
                            // trying to finish the current step
                            if key.code == Instruction::confirm_instruction().keybind {
                                processor.try_finish_current_step();
                                processor.try_populate();



                                // Creates a temporary terminal with a concrete backend type
                                let mut concrete_terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
                                // processes the image and renders the progress
                                self.new_image = processor.try_process(&mut concrete_terminal);



                                // saves the new image if it is created by try_process()
                                if let Some(new_image) = self.new_image.as_ref() {
                                    let source_path = self.selected_image_path.clone().unwrap();
                                    let output_directory = self.output_directory.clone();

                                    let name = self.selected_image_path.clone().unwrap().file_stem().unwrap().to_string_lossy().to_string();
                                    let extension = self.selected_image_path.clone().unwrap().extension().unwrap().to_string_lossy().to_string();
                                    let filename = format!("{} {} {}.{}",
                                                           name,
                                                           Processors::get_processor(self.current_processor_selection).name(),
                                                           processor.get_color_set(),
                                                           extension
                                    );
                                    let output_path = output_directory.join(filename);



                                    // saving the new image and working with potential errors
                                    match new_image.save(&output_path) {
                                        // did save
                                        Ok(_) => {
                                            // getting the image type
                                            let image_type = output_path.extension()
                                            .and_then(|s| s.to_str())
                                            .map(|s| s.to_lowercase())
                                            .unwrap_or_default();

                                            // injecting the metadata from the source image
                                            match image_type.as_str() {
                                                "jpg" | "jpeg" => {
                                                    let source_image = Jpeg::from_bytes(fs::read(source_path.clone())?.into()).unwrap();
                                                    let mut new_image = Jpeg::from_bytes(fs::read(output_path.clone())?.into()).unwrap();
                                                    new_image.set_exif(source_image.exif().clone());
                                                    fs::write(output_path, new_image.encoder().bytes())?;
                                                }

                                                "png" => {
                                                    let source_image = Png::from_bytes(fs::read(source_path.clone())?.into()).unwrap();
                                                    let mut new_image = Png::from_bytes(fs::read(output_path.clone())?.into()).unwrap();
                                                    new_image.set_exif(source_image.exif().clone());
                                                    fs::write(output_path, new_image.encoder().bytes())?;
                                                }

                                                _ => {}
                                            }



                                            // finished
                                            self.current_page = Pages::Finished
                                        }

                                        // did not save
                                        Err(e) => {
                                            eprintln!("Save error: {:?}", e);
                                            eprintln!("Output path: {:?}", output_path);
                                        }
                                    }
                                }
                                // continues if no new image was created (not ready)
                                else { continue; }
                            }

                            // updating the current guide step input
                            let new_input = term_tools::keyboard(&processor.get_current_step_input(), key, true);
                            processor.update_current_step_input(new_input);

                            // trying to reset
                            if key.code == Instruction::reset_instruction().keybind {
                                self.reset();
                            }
                        }

                        // resets if the processor is None
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



/// Module containing terminal-related tools and utilities.
pub mod term_tools {
    use ratatui::crossterm::event;
    use ratatui::crossterm::event::{KeyCode, KeyEvent};

    /// Modifies a string-based number input field from a key event.
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

    /// Modifies a string-based text input field from a key event.
    pub fn keyboard(field: &str, input: KeyEvent, capitalize: bool) -> String {
        if input.kind == event::KeyEventKind::Release {}

        let mut field = field.to_string();
        match input.code {
            KeyCode::Backspace => {
                if field.is_empty() { return field; }
                field.remove(field.len() - 1);
            }
            KeyCode::Char(char) => { field.push(if capitalize { char.to_uppercase().next().unwrap() } else { char } ); }
            _ => {}
        }
        field
    }
}