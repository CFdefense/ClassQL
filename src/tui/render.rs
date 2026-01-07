/// src/tui/render.rs
///
/// Render module for the TUI
///
/// Responsible for rendering the TUI interface and handling user interactions
///
/// Contains:
/// --- ---
/// Tui -> TUI struct that manages application state and event loop
///      Methods:
///      --- ---
///      new -> Create a new TUI instance
///      run -> Run the TUI event loop
///      terminate -> Terminate the TUI
///      clear_error_state -> Clear error state and toast messages
///      handle_tab_completion -> Handle tab completion logic
///      get_completion_hint -> Get helpful hint when no completions available
///      update_toast -> Update toast timer and auto-dismiss
///      show_toast -> Show a toast notification
///      --- ---
/// Helper functions:
///      --- ---
///      render_frame -> Main render function that orchestrates all rendering
///      render_logo -> Render the ASCII art logo
///      render_search_bar_with_data -> Render the search bar with input and cursor
///      render_search_helpers_with_data -> Render the help text at bottom of screen
///      render_query_results -> Render the query results in a 3-column grid
///      render_detail_view -> Render detailed class information overlay
///      render_syntax_highlighting -> Render syntax highlighting (placeholder)
///      render_toast_with_data -> Render error toast notifications
///      render_completion_dropdown -> Render tab completion suggestions dropdown
///      --- ---
/// --- ---
///
use crate::data::sql::Class;
use crate::dsl::compiler::{Compiler, CompilerResult};
use crate::tui::errors::TUIError;
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::ThemePalette;
use crate::tui::widgets::{
    render_completion_dropdown, render_detail_view, render_logo, render_main_menu,
    render_query_results, render_search_bar_with_data, render_search_helpers_with_data,
    render_settings, render_toast_with_data, MenuOption,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};

/// Tui struct
///
/// Main TUI struct that manages the application state and rendering
///
/// Tui fields:
/// --- ---
/// terminal -> The terminal instance for rendering
/// input -> The current user input string in the query box
/// user_query -> The last executed query string
/// toast_message -> Optional toast notification message
/// toast_start_time -> Timestamp when toast was shown (for auto-dismiss)
/// error_type -> Type of error if any (Lexer, Parser, Semantic)
/// problematic_positions -> Byte ranges of problematic tokens in input
/// compiler -> The DSL compiler instance
/// completions -> List of completion suggestions
/// completion_index -> Currently selected completion index
/// show_completions -> Whether completion dropdown is visible
/// partial_word -> The partial word being completed
/// query_results -> The list of Class results from the last query
/// results_scroll -> Scroll offset for results display
/// focus_mode -> Current UI focus (QueryInput, ResultsBrowse, DetailView)
/// selected_result -> Index of currently selected result in browse mode
/// cursor_visible -> Whether the input cursor is visible (for blinking)
/// last_cursor_blink -> Timestamp of last cursor blink toggle
/// max_items_that_fit -> Maximum number of items that fit on the screen
/// --- ---
///
pub struct Tui {
    terminal: DefaultTerminal,
    input: String,
    user_query: String,
    toast_message: Option<String>,
    toast_start_time: Option<Instant>,
    error_type: Option<ErrorType>,
    problematic_positions: Vec<(usize, usize)>,
    compiler: Compiler,
    completions: Vec<String>,
    completion_index: Option<usize>,
    show_completions: bool,
    partial_word: String,
    query_results: Vec<Class>,
    results_scroll: usize,
    focus_mode: FocusMode,
    selected_result: usize,
    cursor_visible: bool,
    last_cursor_blink: Instant,
    max_items_that_fit: usize,
    menu_index: usize,
    current_theme: ThemePalette,
    settings_index: usize,
}

/// Tui Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new TUI instance
/// run -> Run the TUI event loop
/// terminate -> Terminate the TUI
/// --- ---
///
impl Tui {
    /// Create a new TUI instance
    ///
    /// Parameters:
    /// --- ---
    /// compiler -> The compiler instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<Tui, TUIError> -> The new TUI instance
    /// --- ---
    ///
    pub fn new(compiler: Compiler) -> Result<Self, TUIError> {
        // initialize the terminal instance
        let terminal = ratatui::init();

        // initialize and return our tui instance
        Ok(Tui {
            terminal,
            input: String::new(),
            user_query: String::new(),
            toast_message: None,
            toast_start_time: None,
            error_type: None,
            problematic_positions: Vec::new(),
            compiler,
            completions: Vec::new(),
            completion_index: None,
            show_completions: false,
            partial_word: String::new(),
            query_results: Vec::new(),
            results_scroll: 0,
            focus_mode: FocusMode::MainMenu,
            selected_result: 0,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            max_items_that_fit: 0,
            menu_index: 0,
            current_theme: ThemePalette::Default,
            settings_index: 0,
        })
    }

    /// Run the TUI event loop, handling user input and rendering the TUI
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<(), Box<dyn std::error::Error>> -> The result of the event loop
    /// --- ---
    ///
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // update toast timer
            self.update_toast();

            // update cursor blink (every 500ms)
            if self.last_cursor_blink.elapsed() > Duration::from_millis(500) {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_blink = Instant::now();
            }

            // extract render data to avoid borrow conflicts
            let input = self.input.clone();
            let problematic_positions = self.problematic_positions.clone();
            let toast_message = self.toast_message.clone();
            let error_type = self.error_type.clone();
            let completions = self.completions.clone();
            let completion_index = self.completion_index;
            let show_completions = self.show_completions;
            let query_results = self.query_results.clone();
            let results_scroll = self.results_scroll;
            let focus_mode = self.focus_mode.clone();
            let selected_result = self.selected_result;
            let cursor_visible = self.cursor_visible;
            let menu_index = self.menu_index;

            // draw the current state
            let terminal = &mut self.terminal;
            terminal.draw(|f| {
                let (_, max_items) = render_frame(
                    f,
                    &input,
                    &problematic_positions,
                    &toast_message,
                    &error_type,
                    &completions,
                    completion_index,
                    show_completions,
                    &query_results,
                    results_scroll,
                    &focus_mode,
                    selected_result,
                    cursor_visible,
                    menu_index,
                    self.current_theme,
                    self.settings_index,
                );
                // update max_items_that_fit based on actual rendering
                self.max_items_that_fit = max_items;
            })?;

            // handle input events with timeout for cursor blinking
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // handle main menu mode
                    if self.focus_mode == FocusMode::MainMenu {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                            KeyCode::Esc => break Ok(()),
                            KeyCode::Up => {
                                if self.menu_index > 0 {
                                    self.menu_index -= 1;
                                } else {
                                    self.menu_index = MenuOption::all().len() - 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.menu_index < MenuOption::all().len() - 1 {
                                    self.menu_index += 1;
                                } else {
                                    self.menu_index = 0;
                                }
                            }
                            KeyCode::Enter => {
                                let options = MenuOption::all();
                                if self.menu_index < options.len() {
                                    match options[self.menu_index] {
                                        MenuOption::Search => {
                                            self.focus_mode = FocusMode::QueryInput;
                                        }
                                        MenuOption::Help => {
                                            // TODO: Show help screen
                                            self.show_toast("Help feature coming soon!".to_string(), ErrorType::Semantic);
                                        }
                                        MenuOption::Settings => {
                                            self.focus_mode = FocusMode::Settings;
                                        }
                                        MenuOption::Quit => break Ok(()),
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle completion navigation first if completions are showing
                    if self.show_completions {
                        match key.code {
                            KeyCode::Esc => {
                                self.show_completions = false;
                                self.completion_index = None;
                                self.partial_word.clear();
                            }
                            KeyCode::Up => {
                                if let Some(index) = self.completion_index {
                                    if index > 0 {
                                        self.completion_index = Some(index - 1);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if let Some(index) = self.completion_index {
                                    if index < self.completions.len() - 1 {
                                        self.completion_index = Some(index + 1);
                                    } else {
                                        self.completion_index = Some(0);
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // insert selected completion
                                if let Some(index) = self.completion_index {
                                    if index < self.completions.len() {
                                        let completion = &self.completions[index];
                                        // don't add placeholders like <value>
                                        if !completion.starts_with('<') {
                                            // only replace if there's a partial word that matches
                                            if !self.partial_word.is_empty()
                                                && completion.to_lowercase().starts_with(&self.partial_word)
                                            {
                                                // remove the partial word from input
                                                let trim_len = self.partial_word.len();
                                                let new_len = self.input.len().saturating_sub(trim_len);
                                                self.input.truncate(new_len);
                                            } else {
                                                // no partial word - just append with space
                                                if !self.input.is_empty() && !self.input.ends_with(' ') {
                                                    self.input.push(' ');
                                                }
                                            }
                                            self.input.push_str(completion);
                                            if !completion.starts_with('"') {
                                                // add space after completion for next word
                                                self.input.push(' ');
                                            }
                                        }
                                    }
                                }
                                self.show_completions = false;
                                self.completion_index = None;
                                self.partial_word.clear();
                            }
                            KeyCode::Tab => {
                                // tab moves down through completions (same as Down arrow)
                                if let Some(index) = self.completion_index {
                                    if index < self.completions.len() - 1 {
                                        self.completion_index = Some(index + 1);
                                    } else {
                                        self.completion_index = Some(0);
                                    }
                                }
                            }
                            _ => {
                                // any other key hides completions
                                self.show_completions = false;
                                self.completion_index = None;
                                self.partial_word.clear();
                            }
                        }
                        continue; // skip normal key handling when completions are shown
                    }

                    // handle settings mode
                    if self.focus_mode == FocusMode::Settings {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                            KeyCode::Esc => {
                                // exit settings, go back to main menu
                                self.focus_mode = FocusMode::MainMenu;
                            }
                            KeyCode::Up => {
                                if self.settings_index > 0 {
                                    self.settings_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                let max_index = 3;
                                if self.settings_index < max_index {
                                    self.settings_index += 1;
                                }
                            }
                            KeyCode::Left | KeyCode::Right => {
                                // Change theme when on Theme option
                                if self.settings_index == 0 {
                                    let themes = ThemePalette::all();
                                    let current_idx = themes.iter().position(|&t| t == self.current_theme).unwrap_or(0);
                                    if key.code == KeyCode::Left {
                                        let new_idx = if current_idx > 0 { current_idx - 1 } else { themes.len() - 1 };
                                        self.current_theme = themes[new_idx];
                                    } else {
                                        let new_idx = if current_idx < themes.len() - 1 { current_idx + 1 } else { 0 };
                                        self.current_theme = themes[new_idx];
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle detail view mode
                    if self.focus_mode == FocusMode::DetailView {
                        match key.code {
                            KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace => {
                                // exit detail view, go back to results browse
                                self.focus_mode = FocusMode::ResultsBrowse;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle results browse mode
                    if self.focus_mode == FocusMode::ResultsBrowse {
                        match key.code {
                            // exit the TUI if the user presses Ctrl+C or Esc
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                            KeyCode::Esc => break Ok(()),
                            KeyCode::Up => {
                                // if at top of results, go back to query input
                                if self.selected_result == 0 {
                                    self.focus_mode = FocusMode::QueryInput;
                                } else {
                                // move selection up (3 items per row)
                                let cols = 3;
                                if self.selected_result >= cols {
                                    self.selected_result -= cols;
                                    // adjust scroll if needed
                                    if self.selected_result < self.results_scroll {
                                        // align scroll to row boundaries (multiples of cols) to preserve columns
                                        let target_row = self.selected_result / cols;
                                        self.results_scroll = target_row * cols;
                                    }
                                } else {
                                    // can't go up more, go to query input
                                    self.focus_mode = FocusMode::QueryInput;
                                }
                            }
                            }
                            KeyCode::Down => {
                                // move selection down (3 items per row)
                                let cols = 3;
                                if self.selected_result + cols < self.query_results.len() {
                                    self.selected_result += cols;
                                    // only scroll if result is not already visible
                                    let total_results = self.query_results.len();
                                    let max_visible = self.max_items_that_fit;
                                    // if all results fit on screen, don't scroll
                                    if total_results <= max_visible {
                                        // all results visible, don't scroll
                                        self.results_scroll = 0; // ensure scroll is reset
                                    } else if self.selected_result >= self.results_scroll + max_visible {
                                        // result is beyond visible window, scroll to show it
                                        // align scroll to row boundaries (multiples of cols) to preserve columns
                                        let rows_visible = max_visible / cols;
                                        let current_row = self.selected_result / cols;
                                        let scroll_row = current_row.saturating_sub(rows_visible - 1);
                                        self.results_scroll = (scroll_row * cols).max(0);
                                    }
                                }
                            }
                            KeyCode::Left => {
                                // move selection left
                                if self.selected_result > 0 {
                                    self.selected_result -= 1;
                                    // only scroll if result is not already visible
                                    if self.selected_result < self.results_scroll {
                                        self.results_scroll = self.selected_result;
                                    }
                                }
                            }
                            KeyCode::Right => {
                                // move selection right
                                if self.selected_result + 1 < self.query_results.len() {
                                    self.selected_result += 1;
                                    // only scroll if result is not already visible
                                    let total_results = self.query_results.len();
                                    let max_visible = self.max_items_that_fit;
                                    // if all results fit on screen, don't scroll
                                    if total_results <= max_visible {
                                        // all results visible, don't scroll
                                        self.results_scroll = 0; // ensure scroll is reset
                                    } else if self.selected_result >= self.results_scroll + max_visible {
                                        // result is beyond visible window, scroll to show it
                                        self.results_scroll = self.selected_result.saturating_sub(max_visible - 1);
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // open detail view for selected result
                                if self.selected_result < self.query_results.len() {
                                    self.focus_mode = FocusMode::DetailView;
                                }
                            }
                            KeyCode::Char(_) | KeyCode::Backspace => {
                                // typing goes back to query input
                                self.focus_mode = FocusMode::QueryInput;
                                if let KeyCode::Char(c) = key.code {
                                    self.clear_error_state();
                                    self.input.push(c);
                                } else {
                                    self.clear_error_state();
                                    self.input.pop();
                                }
                            }
                            KeyCode::Tab => {
                                // go back to query input for tab completion
                                self.focus_mode = FocusMode::QueryInput;
                                self.handle_tab_completion();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // queryInput mode (default)
                    match key.code {
                        // exit the TUI if the user presses Ctrl+C
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                        // Esc goes back to main menu
                        KeyCode::Esc => {
                            self.focus_mode = FocusMode::MainMenu;
                        }
                        KeyCode::Down => {
                            // if we have results, move to top left result
                            if !self.query_results.is_empty() {
                                self.focus_mode = FocusMode::ResultsBrowse;
                                self.selected_result = 0;
                                // only change scroll if result 0 is not already visible
                                // (i.e., if scroll > 0, or if all results don't fit and we're scrolled)
                                // For simplicity: only reset scroll if it's not 0 and result 0 isn't visible
                                if self.results_scroll > 0 {
                                    self.results_scroll = 0;
                                }
                                // If scroll is already 0, don't change it (all results already visible)
                            }
                        }

                        // use compiler to process the query
                        KeyCode::Enter => {
                            // process the query here
                            self.user_query = self.input.clone();

                            // run the compiler and handle the result
                            match self.compiler.run(&self.input) {
                                CompilerResult::Success { classes, .. } => {
                                    // clear any error state
                                    self.toast_message = None;
                                    self.toast_start_time = None;
                                    self.error_type = None;
                                    self.problematic_positions.clear();

                                    // store the query results
                                    self.query_results = classes;
                                    self.results_scroll = 0;
                                    self.selected_result = 0;
                                }
                                CompilerResult::LexerError {
                                    message,
                                    problematic_positions,
                                } => {
                                    // show error and highlight problematic tokens
                                    self.show_toast(message, ErrorType::Lexer);
                                    self.problematic_positions = problematic_positions;
                                }
                                CompilerResult::ParserError {
                                    message,
                                    problematic_positions,
                                } => {
                                    // show error and highlight problematic tokens
                                    self.show_toast(message, ErrorType::Parser);
                                    self.problematic_positions = problematic_positions;
                                }
                                CompilerResult::SemanticError {
                                    message,
                                    problematic_positions,
                                } => {
                                    // show error and highlight problematic tokens
                                    self.show_toast(message, ErrorType::Semantic);
                                    self.problematic_positions = problematic_positions;
                                }
                                CompilerResult::CodeGenError { message } => {
                                    // show code generation error
                                    self.show_toast(message, ErrorType::Semantic);
                                    self.problematic_positions.clear();
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            // clear any previous toasts and problematic tokens when user backspaces
                            self.clear_error_state();
                            self.input.pop();
                        }
                        KeyCode::Tab => {
                            // handle tab completion
                            self.handle_tab_completion();
                        }
                        KeyCode::Char(c) => {
                            // clear any previous toasts and problematic tokens when user starts typing
                            self.clear_error_state();
                            self.input.push(c);
                        }
                        KeyCode::PageUp => {
                            // scroll results up (show earlier classes)
                            if self.results_scroll >= 3 {
                                self.results_scroll -= 3;
                            } else {
                                self.results_scroll = 0;
                            }
                        }
                        KeyCode::PageDown => {
                            // scroll results down (show later classes)
                            let max_scroll = self.query_results.len().saturating_sub(3);
                            if self.results_scroll + 3 < max_scroll {
                                self.results_scroll += 3;
                            } else {
                                self.results_scroll = max_scroll;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Clear the error state
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn clear_error_state(&mut self) {
        self.toast_message = None;
        self.toast_start_time = None;
        self.error_type = None;
        self.problematic_positions.clear();
    }

    /// Handle tab completion
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The TUI instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn handle_tab_completion(&mut self) {
        // check if input ends with space (no partial word to complete)
        let has_partial = !self.input.is_empty() && !self.input.ends_with(' ');

        // extract the potential partial word (last word after space)
        let potential_partial = if has_partial {
            self.input
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_lowercase()
        } else {
            String::new()
        };

        // get completion suggestions from compiler
        let suggestions = self.compiler.get_tab_completion(self.input.clone());

        // if there's a potential partial word, check if any suggestions match it
        // if matches exist, filter to those; otherwise it's a complete value, show all
        if !potential_partial.is_empty() {
            let matching: Vec<String> = suggestions
                .iter()
                .filter(|s| s.to_lowercase().starts_with(&potential_partial))
                .cloned()
                .collect();

            if !matching.is_empty() {
                // partial word matches some suggestions - filter to those
                self.partial_word = potential_partial;
                self.completions = matching;
            } else {
                // no matches - the "partial" is actually a complete value
                // show all suggestions without replacement
                self.partial_word = String::new();
                self.completions = suggestions;
            }
        } else {
            self.partial_word = String::new();
            self.completions = suggestions;
        };

        if !self.completions.is_empty() {
            self.show_completions = true;
            self.completion_index = Some(0);
        } else if !self.input.trim().is_empty() {
            // no completions available - show helpful hint based on context
            let hint = self.get_completion_hint();
            if !hint.is_empty() {
                self.toast_message = Some(hint);
                self.toast_start_time = Some(Instant::now());
                self.error_type = None; // not an error, just a hint
            }
        }
    }

    /// Get a helpful hint when no completions are available
    fn get_completion_hint(&self) -> String {
        let last_word = self.input.split_whitespace().last().unwrap_or("");
        let last_word_lower = last_word.to_lowercase();

        // check if last word is a condition operator that expects a value
        match last_word_lower.as_str() {
            "contains" | "is" | "equals" | "has" => {
                "Enter a value in quotes, e.g. \"Computer Science\"".to_string()
            }
            "=" | "!=" => {
                "Enter a value, e.g. \"CS\" or 101".to_string()
            }
            "<" | ">" | "<=" | ">=" => {
                "Enter a number, e.g. 3 or 100".to_string()
            }
            "with" => {
                // "starts with" or "ends with"
                "Enter a value in quotes, e.g. \"Intro\"".to_string()
            }
            "hours" => {
                "Enter an operator (=, <, >, etc.) then a number".to_string()
            }
            "type" => {
                "Enter a condition (is, equals, contains) then a value".to_string()
            }
            _ => String::new(),
        }
    }

    /// Terminate the TUI gracefully
    ///
    /// Parameters:
    /// --- ---
    /// self -> The TUI instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    pub fn terminate(&self) -> Result<(), TUIError> {
        ratatui::restore();
        Ok(())
    }

    /// Update the toast
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The TUI instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn update_toast(&mut self) {
        if let Some(start_time) = self.toast_start_time {
            if start_time.elapsed() > Duration::from_secs(3) {
                self.toast_message = None;
                self.toast_start_time = None;
                self.error_type = None;
            }
        }
    }

    /// Show the toast
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The TUI instance
    /// message -> The message to show
    /// error_type -> The error type
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn show_toast(&mut self, message: String, error_type: ErrorType) {
        self.toast_message = Some(message);
        self.toast_start_time = Some(Instant::now());
        self.error_type = Some(error_type);
    }
}

/// Main render function that orchestrates all rendering
///
/// This is a standalone function to avoid borrow checker conflicts with the Tui struct.
/// It coordinates all the individual render functions in the correct order.
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The current user input string
/// problematic_positions -> Byte ranges of problematic tokens for error highlighting
/// toast_message -> Optional toast notification message
/// error_type -> Type of error if any (Lexer, Parser, Semantic)
/// completions -> List of completion suggestions
/// completion_index -> Currently selected completion index
/// show_completions -> Whether completion dropdown is visible
/// query_results -> The query results (classes) to display
/// results_scroll -> The scroll offset for results display
/// focus_mode -> Current UI focus (QueryInput, ResultsBrowse, DetailView)
/// selected_result -> Index of currently selected result in browse mode
/// cursor_visible -> Whether the input cursor is visible (for blinking)
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
#[allow(clippy::too_many_arguments)]
fn render_frame(
    frame: &mut Frame,
    input: &str,
    problematic_positions: &[(usize, usize)],
    toast_message: &Option<String>,
    error_type: &Option<ErrorType>,
    completions: &[String],
    completion_index: Option<usize>,
    show_completions: bool,
    query_results: &[Class],
    results_scroll: usize,
    focus_mode: &FocusMode,
    selected_result: usize,
    cursor_visible: bool,
    menu_index: usize,
    current_theme: ThemePalette,
    settings_index: usize,
) -> (usize, usize) {
    let theme = current_theme.to_theme();
    
    // check window size 
    let buffer_height = frame.area().height;
    let buffer_width = frame.area().width;
    
    // check minimum window size requirements
    let min_height = 25_u16;
    let min_width = 80_u16;
    
    if buffer_height < min_height || buffer_width < min_width {
        // display error message in center of screen
        let error_msg = format!(
            "Window too small! Minimum size: {}x{} (Current: {}x{})",
            min_width, min_height, buffer_width, buffer_height
        );
        // ensure error message fits on screen and handle width constraints
        let error_width = error_msg.len() as u16;
        let error_x = if buffer_width >= error_width {
            (buffer_width.saturating_sub(error_width)) / 2
        } else {
            0 // if window is too narrow, start at left edge
        };
        let error_area = Rect {
            x: error_x,
            y: buffer_height.saturating_sub(1) / 2, // center vertically, ensure in bounds
            width: error_width.min(buffer_width.saturating_sub(error_x)), // don't exceed available width
            height: 1,
        };
        let error_paragraph = ratatui::widgets::Paragraph::new(error_msg)
            .style(ratatui::style::Style::default().fg(theme.error_color));
        frame.render_widget(error_paragraph, error_area);
        return (0, 0);
    }
    
    // clear the entire frame with the theme background color
    // use saturating_sub to ensure we don't go out of bounds
    let buffer = frame.buffer_mut();
    // use buffer's actual dimensions to prevent index errors
    let actual_width = buffer.area.width;
    let actual_height = buffer.area.height;
    
    // double-check dimensions match and are valid
    if actual_width < min_width || actual_height < min_height {
        // buffer dimensions don't meet minimum - show error and return early
        let error_msg = format!(
            "Window too small! Minimum size: {}x{} (Current: {}x{})",
            min_width, min_height, actual_width, actual_height
        );
        let error_width = error_msg.len() as u16;
        let error_x = if actual_width >= error_width {
            (actual_width.saturating_sub(error_width)) / 2
        } else {
            0
        };
        let error_y = actual_height.saturating_sub(1) / 2;
        let error_area = Rect {
            x: error_x,
            y: error_y.min(actual_height.saturating_sub(1)),
            width: error_width.min(actual_width.saturating_sub(error_x)),
            height: 1,
        };
        let error_paragraph = ratatui::widgets::Paragraph::new(error_msg)
            .style(ratatui::style::Style::default().fg(theme.error_color));
        frame.render_widget(error_paragraph, error_area);
        return (0, 0);
    }
    
    // use exclusive range: 0..actual_width gives 0 to actual_width-1
    for y in 0..actual_height {
        for x in 0..actual_width {
            let cell = &mut buffer[(x, y)];
            cell.set_bg(theme.background_color);
        }
    }
    
    // only render widgets if window is large enough
    render_logo(frame, &theme);
    
    // render main menu if in menu mode
    if *focus_mode == FocusMode::MainMenu {
        render_main_menu(frame, menu_index, &theme);
        render_search_helpers_with_data(frame, input, toast_message, query_results, focus_mode, &theme);
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }
    
    // render settings if in settings mode
    if *focus_mode == FocusMode::Settings {
        render_settings(frame, current_theme, &theme, settings_index);
        render_search_helpers_with_data(frame, input, toast_message, query_results, focus_mode, &theme);
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }
    
    // render query interface
    render_search_bar_with_data(frame, input, problematic_positions, focus_mode, cursor_visible, &theme);
    let (_items_rendered, max_items_that_fit) = render_query_results(frame, query_results, results_scroll, focus_mode, selected_result, &theme);
    render_search_helpers_with_data(frame, input, toast_message, query_results, focus_mode, &theme);
    render_toast_with_data(frame, toast_message, error_type, &theme);
    render_completion_dropdown(frame, completions, completion_index, show_completions, &theme);
    
    // render detail view overlay if in detail mode
    if *focus_mode == FocusMode::DetailView && selected_result < query_results.len() {
        render_detail_view(frame, &query_results[selected_result], &theme);
    }
    
    (0, max_items_that_fit)
}

// Widget functions have been moved to src/tui/widgets/ modules            
