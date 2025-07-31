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

use crate::tui::errors::{AppError, TUIError};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct Tui {
    terminal: DefaultTerminal,
    appstate: AppState,
}

impl Tui {
    pub fn new() -> Result<Self, TUIError> {
        // allows for direct terminal input access
        enable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO ENABLE RAW MODE")))?;

        // initialize the terminal instance
        let terminal = ratatui::init();

        // initialize and return our tui instance
        return Ok(Tui {
            terminal,
            appstate: AppState::new(),
        });
    }

    // function to run the tui event loop
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.terminal.draw(|f| {
                self.appstate.render(f);
            })?;
            if matches!(event::read()?, Event::Key(key) if key.code == KeyCode::Esc) {
                break Ok(());
            }
        }
    }

    // function to terminate the tui gracefully
    pub fn terminate(&self) -> Result<(), TUIError> {
        disable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO DISABLE RAW MODE")))?;
        Ok(ratatui::restore())
    }
}

// state of the application
struct AppState {
    input: String,
    error: AppError,
    user_query: String, // other fields... like query results, search text, syntax highlighting, etc.
}

impl AppState {
    fn new() -> Self {
        return AppState {
            input: String::new(),
            error: AppError::NONE,
            user_query: String::new(),
        };
    }

    // function to render the border of the tui
    fn render_tui_border(&self, frame: &mut Frame) {
        let search_paragraph = Paragraph::new(self.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Search"));
        frame.render_widget(search_paragraph, frame.area());
    }

    // function to render the query results in the tui
    fn render_query_results(&self, frame: &mut Frame) {}

    // function to render the users search text in the tui
    fn render_search_text(&self, frame: &mut Frame) {}

    // function to render the syntax highlighting in the tui
    fn render_syntax_highlighting(&self, frame: &mut Frame) {}

    // function to render the app error in the tui
    fn render_app_error(&self, frame: &mut Frame) {}

    // function to render the entire app state
    pub fn render(&self, frame: &mut Frame) {
        self.render_tui_border(frame);
        self.render_query_results(frame);
        self.render_search_text(frame);
        self.render_syntax_highlighting(frame);
        self.render_app_error(frame);
    }
}
