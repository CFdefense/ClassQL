/*
    TUI: Terminal User Interface

    Using: ratatui crate

    Functions:
    1. Render a search bar to type in
    2. Send typed string to query
    3. Show Nicely Formatted Query Results
    4. Allow users to scross Query Results
    5. Provide Syntax Highlighting
    6. Look Aesthetic
*/

use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::tui::errors::{TUIError, AppError};

pub struct Tui {
    terminal: DefaultTerminal,
    appstate: AppState,
}

impl Tui {
    pub fn new() -> Result<Self, TUIError> {
        // allows for direct terminal input access
        enable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO ENABLE RAW MODE")))?;

        let terminal = ratatui::init();
        return Ok(Tui {
            terminal,
            appstate: AppState::new(),
        });
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.terminal.draw(|f| {
                self.appstate.render_searchbar(f);
            })?;
            if matches!(event::read()?, Event::Key(_)) {
                break Ok(());
            }
        }
    }

    pub fn terminate(&self) -> Result<(), TUIError> {
        // disable raw mode before closing program
        disable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO DISABLE RAW MODE")))?;
        Ok(ratatui::restore())
    }
}

// state of the tui application
struct AppState {
    input: String,
    error: AppError,
    // other fields...
}

impl AppState {
    fn new() -> Self {
        return AppState {
            input: String::new(),
            error: AppError::NONE,
        };
    }

    fn render_searchbar(&self, frame: &mut Frame) {
        let search_paragraph = Paragraph::new(self.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Search"));
        frame.render_widget(search_paragraph, frame.area());
    }
}
