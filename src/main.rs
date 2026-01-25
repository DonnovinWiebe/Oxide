mod processor;
mod app;
mod ui;

use std::io::Result;
use std::{fs, io};
use std::path::PathBuf;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use crate::app::App;

fn main() -> Result<()> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // image source setup
    let working_directory = std::env::current_dir()?; // the binary/run location
    let source_directory = working_directory.join("source"); // where the source images are
    let output_directory = working_directory.join("output"); // where the output images are
    fs::create_dir_all(&source_directory)?;
    fs::create_dir_all(&output_directory)?;

    let mut source_image_paths: Vec<PathBuf> = fs::read_dir(&source_directory)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|s| matches!(s.to_lowercase().as_str(), "png" | "jpg" | "jpeg"))
                .unwrap_or(false)
        })
        .collect();
    source_image_paths.sort();

    // app setup
    let mut app = App::new(source_directory, output_directory, source_image_paths);

    // running
    let result = app.run(&mut terminal);

    // restoring the terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // returning result
    result
}
