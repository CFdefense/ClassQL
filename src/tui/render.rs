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
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};

/// ErrorType enum
///
/// ErrorType types:
/// --- ---
/// Lexer -> Lexer error
/// Parser -> Parser error
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for ErrorType
/// Clone -> Clone trait for ErrorType
/// --- ---
///
#[derive(Debug, Clone)]
pub enum ErrorType {
    Lexer,
    Parser,
    Semantic,
}

/// FocusMode enum - tracks which element has keyboard focus
///
/// FocusMode types:
/// --- ---
/// QueryInput -> User is typing in the query box
/// ResultsBrowse -> User is browsing/selecting results
/// DetailView -> User is viewing detailed class info
/// --- ---
///
#[derive(Debug, Clone, PartialEq)]
pub enum FocusMode {
    QueryInput,
    ResultsBrowse,
    DetailView,
}

//// Tui struct
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
            focus_mode: FocusMode::QueryInput,
            selected_result: 0,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            max_items_that_fit: 0,
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
                );
                // update max_items_that_fit based on actual rendering
                self.max_items_that_fit = max_items;
            })?;

            // handle input events with timeout for cursor blinking
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
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
                        // exit the TUI if the user presses Ctrl+C or Esc
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                        KeyCode::Esc => break Ok(()),
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
) -> (usize, usize) {
    render_logo(frame);
    render_search_bar_with_data(frame, input, problematic_positions, focus_mode, cursor_visible);
    let (_items_rendered, max_items_that_fit) = render_query_results(frame, query_results, results_scroll, focus_mode, selected_result);
    render_search_helpers_with_data(frame, input, toast_message, query_results, focus_mode);
    render_syntax_highlighting(frame);
    render_toast_with_data(frame, toast_message, error_type);
    render_completion_dropdown(frame, completions, completion_index, show_completions);
    
    // render detail view overlay if in detail mode
    if *focus_mode == FocusMode::DetailView && selected_result < query_results.len() {
        render_detail_view(frame, &query_results[selected_result]);
    }
    
    (0, max_items_that_fit)
}

/// Render the logo
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_logo(frame: &mut Frame) {
    // logo ascii art
    let ascii_art = r#"            
    ██████╗  ██╗         █████╗    ███████╗   ███████╗   ██████╗   ██╗     
   ██╔════╝  ██║        ██╔══██╗   ██╔════╝   ██╔════╝  ██╔═══██╗  ██║     
   ██║       ██║        ███████║   ███████╗   ███████╗  ██║   ██║  ██║     
   ██║       ██║        ██╔══██║   ╚════██║   ╚════██║  ██║▄▄ ██║  ██║     
   ╚██████╗  ███████╗   ██║  ██║   ███████║   ███████║  ╚██████╔╝  ███████╗
    ╚═════╝  ╚══════╝   ╚═╝  ╚═╝   ╚══════╝   ╚══════╝   ╚══▀▀═╝   ╚══════╝
    "#;

    let lines: Vec<Line> = ascii_art
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default().fg(Color::Rgb(135, 206, 235)),
            ))
        })
        .collect();

    let logo_area = Rect {
        x: frame.area().width.saturating_sub(75) / 2,
        y: frame
            .area()
            .height
            .saturating_sub(ascii_art.len() as u16 + 1),
        width: 80,
        height: ascii_art.len() as u16,
    };

    let logo_paragraph = Paragraph::new(lines);
    frame.render_widget(logo_paragraph, logo_area);
}

/// Render the search bar with data
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The input string
/// problematic_positions -> The problematic positions (byte ranges)
/// focus_mode -> Current focus mode
/// cursor_visible -> Whether cursor should be visible (for blinking)
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_search_bar_with_data(
    frame: &mut Frame,
    input: &str,
    problematic_positions: &[(usize, usize)],
    focus_mode: &FocusMode,
    cursor_visible: bool,
) {
    let search_width = 50;
    let is_focused = *focus_mode == FocusMode::QueryInput;

    // position search bar directly below the logo
    let logo_height = 7; // Height of the ASCII art logo
    let search_y = logo_height + 2; // 2 lines below the logo

    let search_area = Rect {
        x: frame.area().width.saturating_sub(search_width) / 2,
        y: search_y,
        width: search_width,
        height: 3,
    };

    // calculate visible width (minus borders and "> " prefix and cursor)
    let visible_width = search_width.saturating_sub(5) as usize; // 2 for borders, 2 for "> ", 1 for cursor
    let input_len = input.chars().count();

    // calculate scroll offset to keep cursor (end of input) visible
    let scroll_offset = if input_len > visible_width {
        input_len - visible_width
    } else {
        0
    };

    // create styled spans for the input with highlighted problematic positions
    let mut styled_spans = Vec::new();

    // start with the "> " prefix (or "…" if scrolled)
    if scroll_offset > 0 {
        styled_spans.push(Span::styled("…", Style::default().fg(Color::DarkGray)));
    } else {
        styled_spans.push(Span::styled("> ", Style::default().fg(if is_focused { Color::Cyan } else { Color::DarkGray })));
    }

    // process only the visible portion of the input
    for (i, ch) in input.chars().enumerate().skip(scroll_offset) {
        // stop if we've filled the visible width
        if i - scroll_offset >= visible_width {
            break;
        }

        let is_problematic = problematic_positions.iter().any(|&(start, end)| {
            // positions are relative to the input string, so we need to match them correctly
            i >= start && i < end
        });

        let style = if is_problematic {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        styled_spans.push(Span::styled(ch.to_string(), style));
    }

    // add flashing cursor if focused
    if is_focused && cursor_visible {
        styled_spans.push(Span::styled("|", Style::default().fg(Color::Cyan)));
    } else if is_focused {
        styled_spans.push(Span::styled(" ", Style::default()));
    }

    let styled_line = Line::from(styled_spans);

    // border color depends on focus state
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::Rgb(60, 60, 70)
    };

    let title_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let search_paragraph = Paragraph::new(styled_line)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ClassQL Query")
                .title_style(title_style)
                .border_style(Style::default().fg(border_color)),
        );

    frame.render_widget(search_paragraph, search_area);
}

/// Render the search helpers with data
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The input string
/// toast_message -> The toast message
/// query_results -> The query results for showing navigation hints
/// focus_mode -> Current focus mode
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_search_helpers_with_data(
    frame: &mut Frame,
    input: &str,
    toast_message: &Option<String>,
    query_results: &[Class],
    focus_mode: &FocusMode,
) {
    // don't show help text if there's an active toast
    if toast_message.is_some() {
        return;
    }

    let help_text = match focus_mode {
        FocusMode::DetailView => "Press Esc or Enter to close detail view",
        FocusMode::ResultsBrowse => "←↑↓→ Navigate | Enter: Details | Esc: Quit | Type to Search",
        FocusMode::QueryInput => {
            if !query_results.is_empty() {
                "Enter: Search | Tab: Complete | ↓: Browse Results | Esc: Quit"
            } else if input.is_empty() {
                "Type a ClassQL query (e.g., 'prof is Brian') | Esc: Quit"
            } else {
                "Press Enter to Search, Tab for Completions | ↓: Browse Results | Esc: Quit"
            }
        }
    };

    let help_width = help_text.len() as u16;
    let help_area = Rect {
        x: frame.area().width.saturating_sub(help_width) / 2,
        y: frame.area().height.saturating_sub(3),
        width: help_width,
        height: 2,
    };

    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default());

    frame.render_widget(help_paragraph, help_area);
}

/// Render the query results in a 3-column grid below the search bar
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// classes -> The classes to display
/// scroll -> The scroll offset (number of classes to skip)
/// focus_mode -> Current focus mode
/// selected_result -> Index of currently selected result
/// --- ---
///
/// Returns:
/// --- ---
/// (usize, usize) -> (number_of_items_rendered, max_items_that_fit)
/// --- ---
///
fn render_query_results(
    frame: &mut Frame,
    classes: &[Class],
    scroll: usize,
    focus_mode: &FocusMode,
    selected_result: usize,
) -> (usize, usize) {
    if classes.is_empty() {
        return (0, 0);
    }

    let is_browse_mode = *focus_mode == FocusMode::ResultsBrowse || *focus_mode == FocusMode::DetailView;

    // position the results grid below the search bar
    let logo_height = 7; // height of the ASCII art logo
    let search_y = logo_height + 2; // search bar position
    let search_height = 3; // search bar height
    let results_y = search_y + search_height + 1; // 1 line below search bar

    // calculate available space for results
    let available_height = frame.area().height.saturating_sub(results_y + 10); // leave room for help text and logo
    let cell_height = 7_u16; // height per class box
    let rows_to_show = (available_height / cell_height).max(1) as usize;

    // calculate grid dimensions
    let cell_width = 26_u16;
    let cols = 3_usize;
    let grid_width = cell_width * cols as u16 + (cols as u16 - 1) * 2; // cells + gaps
    let grid_x = frame.area().width.saturating_sub(grid_width) / 2;

    // calculate how many items can actually fit
    let max_items_that_fit = rows_to_show * cols;

    // apply scroll offset and get visible classes
    let visible_classes: Vec<(usize, &Class)> = classes
        .iter()
        .enumerate()
        .skip(scroll)
        .take(max_items_that_fit)
        .collect();
    
    let items_rendered = visible_classes.len();

    // render each class in a 3-column grid
    for (global_idx, class) in visible_classes.iter() {
        let idx = global_idx - scroll; // local index in visible area
        let row = idx / cols;
        let col = idx % cols;

        let cell_x = grid_x + (col as u16 * (cell_width + 2));
        let cell_y = results_y + (row as u16 * cell_height);

        let is_selected = is_browse_mode && *global_idx == selected_result;

        // create the class card
        let display_lines = class.format_for_display();

        // build styled lines for the card
        let mut styled_lines: Vec<Line> = Vec::new();

        // line 1: course code (bold cyan)
        if let Some(line) = display_lines.first() {
            let style = Style::default()
                    .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);
            styled_lines.push(Line::from(Span::styled(line.clone(), style)));
        }

        // line 2: title (white)
        if let Some(line) = display_lines.get(1) {
            let style = Style::default().fg(Color::White);
            styled_lines.push(Line::from(Span::styled(line.clone(), style)));
        }

        // line 3: professor (yellow)
        if let Some(line) = display_lines.get(2) {
            let style = Style::default().fg(Color::Yellow);
            styled_lines.push(Line::from(Span::styled(line.clone(), style)));
        }

        // line 4: days/time (green)
        if let Some(line) = display_lines.get(3) {
            let style = Style::default().fg(Color::Green);
            styled_lines.push(Line::from(Span::styled(line.clone(), style)));
        }

        // line 5: enrollment (gray)
        if let Some(line) = display_lines.get(4) {
            let style = Style::default().fg(Color::DarkGray);
            styled_lines.push(Line::from(Span::styled(line.clone(), style)));
        }

        let cell_area = Rect {
            x: cell_x,
            y: cell_y,
            width: cell_width,
            height: cell_height,
        };

        // border color depends on selection state
        let border_color = if is_selected {
            Color::Cyan
        } else {
            Color::Rgb(70, 70, 90)
        };

        let card = Paragraph::new(styled_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

        frame.render_widget(card, cell_area);
    }
    
    (items_rendered, max_items_that_fit)
}

/// Render detailed view of a selected class as an overlay
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// class -> The class to display in detail
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_detail_view(frame: &mut Frame, class: &Class) {
    let detail_width = 60_u16;
    
    // calculate description lines needed (before building content)
    let content_width = (detail_width.saturating_sub(4)) as usize; // -4 for borders and padding
    let desc_lines = if let Some(desc) = &class.description {
        if !desc.trim().is_empty() {
            // calculate how many lines the description will take
            let mut remaining = desc.as_str();
            let mut lines_count = 0;
            let max_desc_lines = 8; // maximum description lines
            
            while !remaining.is_empty() && lines_count < max_desc_lines {
                if remaining.len() <= content_width {
                    lines_count += 1;
                    break;
                } else {
                    let mut break_point = content_width;
                    if let Some(space_pos) = remaining[..content_width.min(remaining.len())].rfind(' ') {
                        break_point = space_pos;
                    } else if let Some(comma_pos) = remaining[..content_width.min(remaining.len())].rfind(',') {
                        break_point = comma_pos + 1;
                    } else if let Some(period_pos) = remaining[..content_width.min(remaining.len())].rfind('.') {
                        break_point = period_pos + 1;
                    }
                    remaining = remaining[break_point..].trim_start();
                    lines_count += 1;
                }
            }
            lines_count
        } else {
            1 // "(No description available)" line
        }
    } else {
        1 // "(No description available)" line
    };
    
    // calculate base content lines (without description)
    let mut base_lines = 2; // course code + title
    base_lines += 1; // blank line
    base_lines += 1; // professor
    if class.professor_email.is_some() {
        base_lines += 1; // email
    }
    base_lines += 1; // blank line
    base_lines += 1; // "Schedule:" label
    // count schedule lines
    if let Some(meeting_times_str) = &class.meeting_times {
        if !meeting_times_str.is_empty() {
            base_lines += meeting_times_str.split('|').filter(|mt| !mt.is_empty()).count();
        } else {
            base_lines += 1; // "TBD"
        }
    } else {
        base_lines += 1; // "TBD"
    }
    if class.meeting_type.is_some() {
        base_lines += 1; // type
    }
    if class.campus.is_some() {
        base_lines += 1; // campus
    }
    base_lines += 1; // method
    base_lines += 1; // blank line
    base_lines += 1; // enrollment
    base_lines += 1; // credits
    base_lines += 2; // blank line + "Description:" label
    
    // total content lines = base + description lines
    let total_content_lines = base_lines + desc_lines;
    
    // calculate height: content + borders (2) + title (1)
    let min_height = 20_u16; // minimum height when no description
    let max_height = 35_u16; // maximum height
    let calculated_height = (total_content_lines as u16 + 3).min(max_height).max(min_height);
    let detail_height = calculated_height;

    let detail_area = Rect {
        x: (frame.area().width.saturating_sub(detail_width)) / 2,
        y: (frame.area().height.saturating_sub(detail_height)) / 2,
        width: detail_width,
        height: detail_height,
    };

    // build detailed content
    let mut lines: Vec<Line> = Vec::new();

    // course code and title
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} {} - {}", class.subject_code, class.course_number, class.section_sequence),
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        class.title.clone(),
        Style::default().fg(Color::Rgb(0, 0, 0)).add_modifier(Modifier::BOLD), // Black text on white
    )));
    lines.push(Line::from("")); // blank line

    // professor
    lines.push(Line::from(vec![
        Span::styled("Professor: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            class.professor_name.as_deref().unwrap_or("TBA"),
            Style::default().fg(Color::Rgb(0, 0, 0)), // Black text on white
        ),
    ]));

    // email
    if let Some(email) = &class.professor_email {
        lines.push(Line::from(vec![
            Span::styled("Email: ", Style::default().fg(Color::Yellow)),
            Span::styled(email, Style::default().fg(Color::Rgb(0, 0, 0))), // Black text on white
        ]));
    }

    lines.push(Line::from("")); // blank line

    // schedule
    lines.push(Line::from(vec![
        Span::styled("Schedule:", Style::default().fg(Color::Green)),
    ]));
    
    // helper function to format time
    let format_time = |time: &str| -> String {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() >= 2 {
            let hours: i32 = parts[0].parse().unwrap_or(0);
            let minutes: i32 = parts[1].parse().unwrap_or(0);
            
            let (display_hour, period) = if hours == 0 {
                (12, "am")
            } else if hours < 12 {
                (hours, "am")
            } else if hours == 12 {
                (12, "pm")
            } else {
                (hours - 12, "pm")
            };
            
            format!("{}:{:02}{}", display_hour, minutes, period)
        } else {
            time.to_string()
        }
    };
    
    // parse meeting_times if available, otherwise fall back to old format
    if let Some(meeting_times_str) = &class.meeting_times {
        if !meeting_times_str.is_empty() {
            // parse meeting times: "M:08:00:00-10:45:00|R:08:00:00-09:15:00"
            for mt in meeting_times_str.split('|') {
                if mt.is_empty() {
                    continue;
                }
                if let Some(colon_pos) = mt.find(':') {
                    let days_part = &mt[..colon_pos];
                    let time_part = &mt[colon_pos + 1..];
                    if let Some(dash_pos) = time_part.find('-') {
                        let start = format_time(&time_part[..dash_pos]);
                        let end = format_time(&time_part[dash_pos + 1..]);
                        if !days_part.is_empty() && !start.is_empty() && !end.is_empty() {
    lines.push(Line::from(vec![
                                Span::styled("    ", Style::default().fg(Color::Rgb(0, 0, 0))), // 4 spaces for indentation
                                Span::styled(format!("{} {}-{}", days_part, start, end), Style::default().fg(Color::Rgb(0, 0, 0))),
                            ]));
                        }
                    }
                }
            }
        } else {
            // empty meeting_times
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default().fg(Color::Rgb(0, 0, 0))), // 4 spaces for indentation
                Span::styled("TBD", Style::default().fg(Color::Rgb(0, 0, 0))),
            ]));
        }
    } else {
        // no meeting_times available
        lines.push(Line::from(vec![
            Span::styled("    ", Style::default().fg(Color::Rgb(0, 0, 0))), // 4 spaces for indentation
            Span::styled("TBD", Style::default().fg(Color::Rgb(0, 0, 0))),
    ]));
    }

    // meeting type
    if let Some(meeting_type) = &class.meeting_type {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Green)),
            Span::styled(meeting_type, Style::default().fg(Color::Rgb(0, 0, 0))), // Black text on white
        ]));
    }

    // location/campus
    if let Some(campus) = &class.campus {
        lines.push(Line::from(vec![
            Span::styled("Campus: ", Style::default().fg(Color::Green)),
            Span::styled(campus, Style::default().fg(Color::Rgb(0, 0, 0))), // Black text on white
        ]));
    }

    // instruction method
        lines.push(Line::from(vec![
            Span::styled("Method: ", Style::default().fg(Color::Green)),
        Span::styled(
            class.instruction_method.as_deref().unwrap_or("N/A"),
            Style::default().fg(Color::Rgb(0, 0, 0)), // Black text on white
        ),
        ]));

    lines.push(Line::from("")); // blank line

    // enrollment
    let enrollment_str = match (class.enrollment, class.max_enrollment) {
        (Some(e), Some(m)) => format!("{} / {} ({:.0}% full)", e, m, (e as f64 / m as f64) * 100.0),
        _ => "Unknown".to_string(),
    };
    lines.push(Line::from(vec![
        Span::styled("Enrollment: ", Style::default().fg(Color::Magenta)),
        Span::styled(enrollment_str, Style::default().fg(Color::Rgb(0, 0, 0))), // Black text on white
    ]));

    // credit hours
    lines.push(Line::from(vec![
        Span::styled("Credits: ", Style::default().fg(Color::Magenta)),
        Span::styled(format!("{}", class.credit_hours), Style::default().fg(Color::Rgb(0, 0, 0))), // Black text on white
    ]));

    // description
        lines.push(Line::from("")); // blank line
    lines.push(Line::from(vec![
        Span::styled("Description: ", Style::default().fg(Color::Green)),
    ]));
    
    if let Some(desc) = &class.description {
        if !desc.trim().is_empty() {
            // wrap description to fit within detail width (account for borders and padding)
            let content_width = (detail_width.saturating_sub(4)) as usize; // -4 for borders and padding
            let mut remaining = desc.as_str();
            let mut desc_lines_added = 0;
            let max_desc_lines = 8; // maximum description lines to show
            
            while !remaining.is_empty() && desc_lines_added < max_desc_lines {
                if remaining.len() <= content_width {
                    lines.push(Line::from(Span::styled(remaining.to_string(), Style::default().fg(Color::Rgb(60, 60, 60)))));
                    break;
        } else {
                    // find a good break point (space, comma, period, etc.)
                    let mut break_point = content_width;
                    if let Some(space_pos) = remaining[..content_width.min(remaining.len())].rfind(' ') {
                        break_point = space_pos;
                    } else if let Some(comma_pos) = remaining[..content_width.min(remaining.len())].rfind(',') {
                        break_point = comma_pos + 1;
                    } else if let Some(period_pos) = remaining[..content_width.min(remaining.len())].rfind('.') {
                        break_point = period_pos + 1;
                    }
                    
                    let line_text = if desc_lines_added == max_desc_lines - 1 {
                        // last line, truncate if needed
                        if remaining.len() > content_width {
                            format!("{}...", &remaining[..content_width.saturating_sub(3)])
                        } else {
                            remaining.to_string()
                        }
                    } else {
                        remaining[..break_point].to_string()
                    };
                    
                    lines.push(Line::from(Span::styled(line_text, Style::default().fg(Color::Rgb(60, 60, 60)))));
                    remaining = remaining[break_point..].trim_start();
                    desc_lines_added += 1;
                }
            }
        } else {
            // description exists but is empty/whitespace
            lines.push(Line::from(Span::styled("(No description available)", Style::default().fg(Color::Rgb(120, 120, 120)))));
        }
    } else {
        // description is None
        lines.push(Line::from(Span::styled("(No description available)", Style::default().fg(Color::Rgb(120, 120, 120)))));
    }

    // first, clear the area to cover results below with solid background
    frame.render_widget(Clear, detail_area);

    let white_bg = Color::Rgb(255, 255, 255); // True white

    let detail_paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Class Details ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(white_bg)),
    );

    frame.render_widget(detail_paragraph, detail_area);
    
    // force white background on empty/border cells, preserve styled text cells
    let buffer = frame.buffer_mut();
    for y in detail_area.top()..detail_area.bottom() {
        for x in detail_area.left()..detail_area.right() {
            let cell = &mut buffer[(x, y)];
            // Only set white background if cell is empty or a border character
            // This preserves the text colors and backgrounds set by the paragraph
            if cell.symbol() == " " || cell.symbol() == "│" || cell.symbol() == "─" || 
               cell.symbol() == "┌" || cell.symbol() == "┐" || cell.symbol() == "└" || 
               cell.symbol() == "┘" || cell.symbol() == "├" || cell.symbol() == "┤" ||
               cell.symbol() == "┬" || cell.symbol() == "┴" {
                cell.set_bg(white_bg);
            }
        }
    }
}

/// Render the syntax highlighting
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
/// Note: Currently a placeholder - syntax highlighting is handled inline in render_search_bar_with_data
///
fn render_syntax_highlighting(frame: &mut Frame) {
    let _ = frame;
}

/// Render the toast with data
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// toast_message -> The toast message
/// error_type -> The error type
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_toast_with_data(
    frame: &mut Frame,
    toast_message: &Option<String>,
    error_type: &Option<ErrorType>,
) {
    if let Some(message) = toast_message {
        // use the passed error type to determine toast dimensions
        let is_parser_error = matches!(error_type, Some(ErrorType::Parser));

        // calculate toast dimensions based on error type
        let (toast_width, max_toast_height) = if is_parser_error {
            // parser errors need more space for context and suggestions
            (80_u16, 15)
        } else {
            // lexer errors are typically shorter
            (60, 8)
        };

        // wrap text to fit within the toast width (account for borders and padding)
        let content_width = toast_width.saturating_sub(4) as usize; // -4 for borders and padding
        let mut wrapped_lines = Vec::new();

        for line in message.lines() {
            if line.len() <= content_width {
                wrapped_lines.push(line.to_string());
            } else {
                // split long lines into multiple lines
                let mut remaining = line;
                while !remaining.is_empty() {
                    if remaining.len() <= content_width {
                        wrapped_lines.push(remaining.to_string());
                        break;
                    } else {
                        // find a good break point (space, comma, etc.)
                        let mut break_point = content_width;
                        if let Some(space_pos) = remaining[..content_width].rfind(' ') {
                            break_point = space_pos;
                        } else if let Some(comma_pos) = remaining[..content_width].rfind(',') {
                            break_point = comma_pos + 1; // include the comma
                        }

                        wrapped_lines.push(remaining[..break_point].to_string());
                        remaining = remaining[break_point..].trim_start();
                    }
                }
            }
        }

        let toast_height = (wrapped_lines.len() as u16 + 2).min(max_toast_height);

        let toast_area = Rect {
            x: (frame.area().width.saturating_sub(toast_width)) / 2,
            y: frame.area().height.saturating_sub(toast_height + 1),
            width: toast_width,
            height: toast_height,
        };

        // create styled lines for the toast
        let styled_lines: Vec<Line> = wrapped_lines
            .iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::White))))
            .collect();

        let toast_paragraph = Paragraph::new(styled_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::DarkGray)),
        );

        frame.render_widget(toast_paragraph, toast_area);
    }
}

/// Render the completion dropdown
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// completions -> The completions
/// completion_index -> The completion index
/// show_completions -> Whether to show completions
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_completion_dropdown(
    frame: &mut Frame,
    completions: &[String],
    completion_index: Option<usize>,
    show_completions: bool,
) {
    if !show_completions {
        return;
    }

    let dropdown_width = 50;

    // position below the search bar
    let logo_height = 7; // height of the ASCII art logo
    let search_y = logo_height + 2; // search bar position
    let search_height = 3; // search bar height
    let dropdown_y = search_y + search_height + 1; // 1 line below search bar

    // calculate max available height (leave some space at bottom)
    let max_available_height = frame.area().height.saturating_sub(dropdown_y + 2);
    
    // height = number of completions + 2 for borders, capped by available space
    let dropdown_height = (completions.len() as u16 + 2).min(max_available_height);

    let dropdown_area = Rect {
        x: frame.area().width.saturating_sub(dropdown_width) / 2,
        y: dropdown_y,
        width: dropdown_width,
        height: dropdown_height,
    };

    let white_bg = Color::Rgb(255, 255, 255);
    let mut styled_lines = Vec::new();
    for (i, completion) in completions.iter().enumerate() {
        let style = if Some(i) == completion_index {
            Style::default().fg(Color::Black).bg(Color::Cyan) // selected: black text on cyan
        } else {
            Style::default().fg(Color::Rgb(0, 0, 0)).bg(white_bg) // unselected: pure black text on white for maximum contrast
        };
        styled_lines.push(Line::from(Span::styled(completion, style)));
    }

    // first, clear the area to cover results below with solid background
    frame.render_widget(Clear, dropdown_area);

    let white_bg = Color::Rgb(255, 255, 255); // True white

    let dropdown_paragraph = Paragraph::new(styled_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Suggestions (↑↓ to navigate, Enter to select)")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(white_bg)),
        );

    frame.render_widget(dropdown_paragraph, dropdown_area);
    
    // force white background on empty/border cells, preserve styled text cells
    let buffer = frame.buffer_mut();
    for y in dropdown_area.top()..dropdown_area.bottom() {
        for x in dropdown_area.left()..dropdown_area.right() {
            let cell = &mut buffer[(x, y)];
            // only set white background if cell is empty or a border character
            // this preserves the text colors and backgrounds set by the paragraph
            if cell.symbol() == " " || cell.symbol() == "│" || cell.symbol() == "─" || 
               cell.symbol() == "┌" || cell.symbol() == "┐" || cell.symbol() == "└" || 
               cell.symbol() == "┘" || cell.symbol() == "├" || cell.symbol() == "┤" ||
               cell.symbol() == "┬" || cell.symbol() == "┴" {
                cell.set_bg(white_bg);
            }
        }
    }
}
