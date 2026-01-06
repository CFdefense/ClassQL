/// src/tui/render.rs
///
/// Render for the TUI
///
/// Responsible for rendering the TUI
///
/// Contains:
/// --- ---
/// Tui -> TUI struct
///      Methods:
///      --- ---
///      new -> Create a new TUI instance
///      run -> Run the TUI event loop
///      terminate -> Terminate the TUI
///      --- ---
/// Helper functions:
///      --- ---
///      render_logo -> Render the logo
///      render_search_bar_with_data -> Render the search bar with data
///      render_search_helpers_with_data -> Render the search helpers with data
///      render_query_results -> Render the query results
///      render_syntax_highlighting -> Render the syntax highlighting
///      render_toast_with_data -> Render the toast with data
///      render_completion_dropdown -> Render the completion dropdown
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
use ratatui::widgets::{Block, Borders, Paragraph};
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

/// Tui struct
///
/// Tui fields:
/// --- ---
/// terminal -> The terminal instance
/// input -> The input string
/// user_query -> The user query string
/// toast_message -> The toast message
/// toast_start_time -> The toast start time
/// error_type -> The error type
/// problematic_positions -> The problematic positions (byte ranges)
/// compiler -> The compiler instance
/// completions -> The completions
/// completion_index -> The completion index
/// show_completions -> Whether to show completions
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for Tui
/// Clone -> Clone trait for Tui
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
    /// The partial word being completed (for replacement on selection)
    partial_word: String,
    /// The query results (classes) to display
    query_results: Vec<Class>,
    /// Scroll offset for results (how many rows to skip)
    results_scroll: usize,
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

            // draw the current state
            let terminal = &mut self.terminal;
            terminal.draw(|f| {
                render_frame(
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
                );
            })?;

            // handle input events
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
                            // Tab moves down through completions (same as Down arrow)
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

                match key.code {
                    // exit the TUI if the user presses Ctrl+C or Esc
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                    KeyCode::Esc => break Ok(()),

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
                                let count = classes.len();
                                self.query_results = classes;
                                self.results_scroll = 0;

                                // show success message with count
                                if count == 0 {
                                    self.toast_message = Some("No results found".to_string());
                                } else {
                                    self.toast_message = Some(format!("Found {} class{}", count, if count == 1 { "" } else { "es" }));
                                }
                                self.toast_start_time = Some(Instant::now());
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
                    KeyCode::Up | KeyCode::PageUp => {
                        // scroll results up (show earlier classes)
                        if self.results_scroll >= 3 {
                            self.results_scroll -= 3;
                        } else {
                            self.results_scroll = 0;
                        }
                    }
                    KeyCode::Down | KeyCode::PageDown => {
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

/// Standalone render function to avoid borrow checker conflicts
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The input string
/// problematic_positions -> The problematic positions (byte ranges)
/// toast_message -> The toast message
/// error_type -> The error type
/// completions -> The completions
/// completion_index -> The completion index
/// show_completions -> Whether to show completions
/// query_results -> The query results (classes) to display
/// results_scroll -> The scroll offset for results
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
) {
    render_logo(frame);
    render_search_bar_with_data(frame, input, problematic_positions);
    render_query_results(frame, query_results, results_scroll);
    render_search_helpers_with_data(frame, input, toast_message, query_results);
    render_syntax_highlighting(frame);
    render_toast_with_data(frame, toast_message, error_type);
    render_completion_dropdown(frame, completions, completion_index, show_completions);
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
) {
    let search_width = 50;

    // position search bar directly below the logo
    let logo_height = 7; // Height of the ASCII art logo
    let search_y = logo_height + 2; // 2 lines below the logo

    let search_area = Rect {
        x: frame.area().width.saturating_sub(search_width) / 2,
        y: search_y,
        width: search_width,
        height: 3,
    };

    // calculate visible width (minus borders and "> " prefix)
    let visible_width = search_width.saturating_sub(4) as usize; // 2 for borders, 2 for "> "
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
        styled_spans.push(Span::styled("> ", Style::default().fg(Color::White)));
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

    let styled_line = Line::from(styled_spans);
    let search_paragraph = Paragraph::new(styled_line)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ClassQL Query")
                .title_style(Style::default().fg(Color::Cyan))
                .border_style(Style::default().fg(Color::Gray)),
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
) {
    // don't show help text if there's an active toast
    if toast_message.is_some() {
        return;
    }

    let help_text = if !query_results.is_empty() {
        "↑↓ Scroll Results | Enter to Search | Esc to Exit"
    } else if input.is_empty() {
        "Type a ClassQL query (e.g., 'prof is Brian')"
    } else {
        "Press Enter to Search, Esc to Exit"
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
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_query_results(frame: &mut Frame, classes: &[Class], scroll: usize) {
    if classes.is_empty() {
        return;
    }

    // Position the results grid below the search bar
    let logo_height = 7; // Height of the ASCII art logo
    let search_y = logo_height + 2; // search bar position
    let search_height = 3; // search bar height
    let results_y = search_y + search_height + 1; // 1 line below search bar

    // Calculate available space for results
    let available_height = frame.area().height.saturating_sub(results_y + 10); // leave room for help text and logo
    let cell_height = 7_u16; // height per class box
    let rows_to_show = (available_height / cell_height).max(1) as usize;

    // Calculate grid dimensions
    let cell_width = 26_u16;
    let cols = 3_usize;
    let grid_width = cell_width * cols as u16 + (cols as u16 - 1) * 2; // cells + gaps
    let grid_x = frame.area().width.saturating_sub(grid_width) / 2;

    // Apply scroll offset and get visible classes
    let visible_classes: Vec<&Class> = classes.iter().skip(scroll).take(rows_to_show * cols).collect();

    // Render each class in a 3-column grid
    for (idx, class) in visible_classes.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;

        let cell_x = grid_x + (col as u16 * (cell_width + 2));
        let cell_y = results_y + (row as u16 * cell_height);

        // Create the class card
        let display_lines = class.format_for_display();

        // Build styled lines for the card
        let mut styled_lines: Vec<Line> = Vec::new();

        // Line 1: Course code (bold cyan)
        if let Some(line) = display_lines.first() {
            styled_lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        // Line 2: Title (white)
        if let Some(line) = display_lines.get(1) {
            styled_lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::White),
            )));
        }

        // Line 3: Professor (yellow)
        if let Some(line) = display_lines.get(2) {
            styled_lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Line 4: Days/Time (green)
        if let Some(line) = display_lines.get(3) {
            styled_lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::Green),
            )));
        }

        // Line 5: Enrollment (gray)
        if let Some(line) = display_lines.get(4) {
            styled_lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let cell_area = Rect {
            x: cell_x,
            y: cell_y,
            width: cell_width,
            height: cell_height,
        };

        let card = Paragraph::new(styled_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(70, 70, 90))),
        );

        frame.render_widget(card, cell_area);
    }
}

/// TODO:Render the syntax highlighting
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

    let mut styled_lines = Vec::new();
    for (i, completion) in completions.iter().enumerate() {
        let style = if Some(i) == completion_index {
            Style::default().fg(Color::Black).bg(Color::White) // better contrast for selected item
        } else {
            Style::default().fg(Color::Gray)
        };
        styled_lines.push(Line::from(Span::styled(completion, style)));
    }

    let dropdown_paragraph = Paragraph::new(styled_lines)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Suggestions (↑↓ to navigate, Enter to select)")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(dropdown_paragraph, dropdown_area);
}
