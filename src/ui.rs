use ratatui::crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Pages};
use crate::processor::guide::ProcessingStepTypes;
use crate::processor::Processors;

/// Renders the current page of the application.
pub fn render_current_page(frame: &mut Frame, app: &App) {
    // header
    let header_block = Block::new().borders(Borders::ALL);
    let header = Paragraph::new(vec![
        Line::raw("Oxide"),
        Line::raw(app.current_page_name()),
    ]).block(header_block);

    // footer
    let footer_height = Instruction::get_instructions_for(&app.current_page).len() as u16 + 2;
    let footer_block = Block::new().borders(Borders::ALL);
    let instructions = Instruction::get_instructions_for(&app.current_page);
    let footer = Paragraph::new(instructions).block(footer_block);

    // The sections of the screen.
    let leaflets = Layout::new(Direction::Vertical, [
        Constraint::Length(4), // header
        Constraint::Fill(1), // body
        Constraint::Length(footer_height), // footer
    ]).split(frame.area());

    // rendering the header and footer
    frame.render_widget(header, leaflets[0]);
    frame.render_widget(footer, leaflets[2]);


    // body
    match app.current_page {
        Pages::Launching => {
            let body = Paragraph::new("Press any button to continue...");
            frame.render_widget(body, leaflets[1]);
        }

        Pages::SelectingImageSource => {
            let body = Paragraph::new(vec![
                Line::raw(format!("Found {} images", app.source_image_paths.len())),
                Line::raw(format!("In: {}", app.source_directory.to_string_lossy())),
                Line::raw(format!("Selected image: {}", app.print_selected_image_filename())),
            ]);
            frame.render_widget(body, leaflets[1]);
        }

        Pages::SelectingProcessingType => {
            let body = Paragraph::new(format!("Selected processor: {}", Processors::get_processor(app.current_processor_selection).name()));
            frame.render_widget(body, leaflets[1]);
        }

        Pages::Preprocessing => {
            if let Some(processor) = &app.selected_processor {
                let body = Paragraph::new(vec![
                    Line::raw(format!("Step: {}", processor.get_current_step_label())),
                    Line::raw(if processor.get_current_step_type() == ProcessingStepTypes::Color {
                        format!("Color: #{}", processor.get_current_step_input())
                    }
                    else {
                        format!("Input: {}", processor.get_current_step_input())
                    }),
                ]);
                frame.render_widget(body, leaflets[1]);
            }
            else {
                let body = Paragraph::new("Error: No processor");
                frame.render_widget(body, leaflets[1]);
            }
        }

        Pages::Finished => {
            let body = Paragraph::new(vec![
                Line::raw("Saved"),
                Line::raw(format!("Processed in {:.3} seconds", app.processing_time.as_millis() as f64 / 1000.0)),
            ]);
            frame.render_widget(body, leaflets[1]);
        }
    }
}

/// Renders a loading screen during processing.
pub fn render_loading(frame: &mut Frame, info: String) {
    // header
    let header_block = Block::new().borders(Borders::ALL);
    let header = Paragraph::new(vec![
        Line::raw("Oxide"),
        Line::raw("Processing..."),
    ]).block(header_block);

    // The sections of the screen.
    let leaflets = Layout::new(Direction::Vertical, [
        Constraint::Length(4), // header
        Constraint::Fill(1), // body
    ]).split(frame.area());

    // rendering the header
    frame.render_widget(header, leaflets[0]);

    // rendering the progress
    let body = Paragraph::new(vec![
        Line::raw(""),
        Line::raw(""),
        Line::raw(""),
        Line::raw(info),
    ]);
    frame.render_widget(body, leaflets[1]);
}

/// Renders a progress indicator during processing.
pub fn render_progress(frame: &mut Frame, percent_complete: f64) {
    // header
    let header_block = Block::new().borders(Borders::ALL);
    let header = Paragraph::new(vec![
        Line::raw("Oxide"),
        Line::raw("Processing..."),
    ]).block(header_block);

    // The sections of the screen.
    let leaflets = Layout::new(Direction::Vertical, [
        Constraint::Length(4), // header
        Constraint::Fill(1), // body
    ]).split(frame.area());

    // rendering the header
    frame.render_widget(header, leaflets[0]);

    // rendering the progress
    let body = Paragraph::new(vec![
        Line::raw(""),
        Line::raw(""),
        Line::raw(""),
        Line::raw(format!("Progress: {:.1}%", percent_complete)),
    ]);
    frame.render_widget(body, leaflets[1]);
}



/// Defines a keybind instruction for the user.
pub struct Instruction {
    /// The keybind label.
    key: String,
    /// A concise description of what the keybind does.
    label: String,
    /// The keybind code to be compared to a given input.
    pub keybind: KeyCode
}
impl Instruction {
    /// Returns a new instruction.
    pub fn new(key: String, label: String, keybind: KeyCode) -> Instruction { Instruction {key, label, keybind } }

    /// Returns a formatted string representation of the instruction.
    fn printed(&mut self) -> String {
        let mut print = "".to_string();
        print += &format!("[{}] {}", &self.key, &self.label);
        print
    }

    /// Groups a list of instructions into a list of lines.
    pub fn in_groups(instructions: Vec<Instruction>, group_limit: usize) -> Vec<Line<'static>> {
        // the lines of instructions to be returned
        let mut lines = Vec::new();

        // the current line of instructions being assembled
        let mut current_group: String = "".to_string();
        let mut amount_in_group: usize = 0;

        // adds the current line to the list of lines and creates a new blank line in its place if it reaches the group limit
        for mut instruction in instructions {
            if amount_in_group >= group_limit {
                lines.push(Line::from(current_group));
                current_group = "".to_string();
                amount_in_group = 0;
            }

            // adds the current instruction to the current line
            amount_in_group += 1;
            if current_group != "" { current_group += " | "; }
            current_group += instruction.printed().as_str();
        }

        // adds the last line to the list of lines if it isn't empty
        if current_group != "" { lines.push(Line::from(current_group)); }

        // returns the list of lines
        lines
    }



    // instructions
    pub fn select_next() -> Instruction { Instruction::new(">".to_string(), "next page".to_string(), KeyCode::Right) }
    pub fn select_previous() -> Instruction { Instruction::new("<".to_string(), "previous page".to_string(), KeyCode::Left) }
    pub fn confirm_instruction() -> Instruction { Instruction::new("ENTER".to_string(), "confirm".to_string(), KeyCode::Enter) }
    pub fn reset_instruction() -> Instruction { Instruction::new("ESC".to_string(), "reset".to_string(), KeyCode::Esc) }
    pub fn run_again_instruction() -> Instruction { Instruction::new("R".to_string(), "run again".to_string(), KeyCode::Char('r')) }
    pub fn quit_instruction() -> Instruction { Instruction::new("Q".to_string(), "quit".to_string(), KeyCode::Char('q')) }

    /// Gets the instructions for a given page.
    pub fn get_instructions_for(page: &Pages) -> Vec<Line> {
        match page {
            Pages::Launching => {
                Instruction::in_groups(vec![], 4)
            }
            Pages::SelectingImageSource => {
                Instruction::in_groups(vec![
                    Instruction::select_next(),
                    Instruction::select_previous(),
                    Instruction::confirm_instruction(),
                    Instruction::reset_instruction(),
                    Instruction::quit_instruction(),
                ], 4)
            }
            Pages::SelectingProcessingType => {
                Instruction::in_groups(vec![
                    Instruction::select_next(),
                    Instruction::select_previous(),
                    Instruction::confirm_instruction(),
                    Instruction::reset_instruction(),
                    Instruction::quit_instruction(),
                ], 4)
            }
            Pages::Preprocessing => {
                Instruction::in_groups(vec![
                    Instruction::confirm_instruction(),
                    Instruction::reset_instruction(),
                ], 4)
            }
            Pages::Finished => {
                Instruction::in_groups(vec![
                    Instruction::run_again_instruction(),
                    Instruction::quit_instruction(),
                ], 4)
            }
        }
    }
}