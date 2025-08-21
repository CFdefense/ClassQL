/*
    TUI: Terminal User Interface

    Using: ratatui crate

    Functions:
    1. Render a search bar to type in
    2. Send typed string to query
    3. Show Nicely Formatted Query Results
    4. Allow users to scroll Query Results
    5. Provide Syntax Highlighting
    6. Look Aesthetic
*/

use crate::compiler::driver::{Compiler, CompilerResult};
use crate::tui::errors::TUIError;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum ErrorType {
    Lexer,
    Parser,
}

pub struct Tui<'a> {
    terminal: DefaultTerminal,
    input: String,
    user_query: String,
    toast_message: Option<String>,
    toast_start_time: Option<Instant>,
    error_type: Option<ErrorType>,
    problematic_tokens: Vec<(usize, usize)>,
    compiler: &'a mut Compiler,
    // Tab completion state
    completions: Vec<String>,
    completion_index: Option<usize>,
    show_completions: bool,
}

impl<'a> Tui<'a> {
    pub fn new(compiler: &'a mut Compiler) -> Result<Self, TUIError> {
        // allows for direct terminal input access
        enable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO ENABLE RAW MODE")))?;

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
            problematic_tokens: Vec::new(),
            compiler,
            completions: Vec::new(),
            completion_index: None,
            show_completions: false,
        })
    }

    // function to run the tui event loop
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Update toast timer
            self.update_toast();

            // Extract render data to avoid borrow conflicts
            let input = self.input.clone();
            let problematic_tokens = self.problematic_tokens.clone();
            let toast_message = self.toast_message.clone();
            let error_type = self.error_type.clone();
            let completions = self.completions.clone();
            let completion_index = self.completion_index;
            let show_completions = self.show_completions;

            // Draw the current state
            let terminal = &mut self.terminal;
            terminal.draw(|f| {
                render_frame(
                    f,
                    &input,
                    &problematic_tokens,
                    &toast_message,
                    &error_type,
                    &completions,
                    completion_index,
                    show_completions,
                );
            })?;

            // Handle input events
            if let Event::Key(key) = event::read()? {
                // Handle completion navigation first if completions are showing
                if self.show_completions {
                    match key.code {
                        KeyCode::Esc => {
                            self.show_completions = false;
                            self.completion_index = None;
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
                                }
                            }
                        }
                        KeyCode::Enter => {
                            // Insert selected completion
                            if let Some(index) = self.completion_index {
                                if index < self.completions.len() {
                                    let completion = &self.completions[index];
                                    // Don't add placeholders like <value>
                                    if !completion.starts_with('<') {
                                        if !self.input.is_empty() && !self.input.ends_with(' ') {
                                            self.input.push(' ');
                                        }
                                        self.input.push_str(completion);
                                        if !completion.starts_with('"') {
                                            // Don't add space after quoted strings
                                            self.input.push(' ');
                                        }
                                    }
                                }
                            }
                            self.show_completions = false;
                            self.completion_index = None;
                        }
                        KeyCode::Tab => {
                            // Refresh completions on tab
                            self.handle_tab_completion();
                        }
                        _ => {
                            // Any other key hides completions
                            self.show_completions = false;
                            self.completion_index = None;
                        }
                    }
                    continue; // Skip normal key handling when completions are shown
                }

                match key.code {
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Enter => {
                        // Process the query here
                        self.user_query = self.input.clone();

                        // Run the compiler and handle the result
                        match self.compiler.run(&self.input) {
                            CompilerResult::Success { message, ast } => {
                                // Clear any error state
                                self.toast_message = None;
                                self.toast_start_time = None;
                                self.error_type = None;
                                self.problematic_tokens.clear();
                                // Show a brief success message
                                self.toast_message = Some(message);
                                self.toast_start_time = Some(Instant::now());
                                // TODO: Process successful AST for semantic analysis or query execution
                                println!("Parsed AST: {:?}", ast); // Debug output for now
                            }
                            CompilerResult::LexerError {
                                message,
                                problematic_tokens,
                            } => {
                                // Show error and highlight problematic tokens
                                self.show_toast(message, ErrorType::Lexer);
                                self.problematic_tokens = problematic_tokens;
                            }
                            CompilerResult::ParserError {
                                message,
                                problematic_tokens,
                            } => {
                                // Show error and highlight problematic tokens
                                self.show_toast(message, ErrorType::Parser);
                                self.problematic_tokens = problematic_tokens;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        // Clear any previous toasts and problematic tokens when user backspaces
                        self.clear_error_state();
                        self.input.pop();
                    }
                    KeyCode::Tab => {
                        // Handle tab completion
                        self.handle_tab_completion();
                    }
                    KeyCode::Char(c) => {
                        // Clear any previous toasts and problematic tokens when user starts typing
                        self.clear_error_state();
                        self.input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }

    fn clear_error_state(&mut self) {
        self.toast_message = None;
        self.toast_start_time = None;
        self.error_type = None;
        self.problematic_tokens.clear();
    }

    fn handle_tab_completion(&mut self) {
        // Get completion suggestions from compiler
        self.completions = self.compiler.get_tab_completion(self.input.clone());

        if !self.completions.is_empty() {
            self.show_completions = true;
            self.completion_index = Some(0);
        }
    }

    // function to terminate the tui gracefully
    pub fn terminate(&self) -> Result<(), TUIError> {
        disable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO DISABLE RAW MODE")))?;
        ratatui::restore();
        Ok(())
    }

    fn update_toast(&mut self) {
        if let Some(start_time) = self.toast_start_time {
            if start_time.elapsed() > Duration::from_secs(3) {
                self.toast_message = None;
                self.toast_start_time = None;
                self.error_type = None;
            }
        }
    }

    fn show_toast(&mut self, message: String, error_type: ErrorType) {
        self.toast_message = Some(message);
        self.toast_start_time = Some(Instant::now());
        self.error_type = Some(error_type);
    }
}

// Standalone render function to avoid borrow checker conflicts
#[allow(clippy::too_many_arguments)]
fn render_frame(
    frame: &mut Frame,
    input: &str,
    problematic_tokens: &[(usize, usize)],
    toast_message: &Option<String>,
    error_type: &Option<ErrorType>,
    completions: &[String],
    completion_index: Option<usize>,
    show_completions: bool,
) {
    render_logo(frame);
    render_search_bar_with_data(frame, input, problematic_tokens);
    render_query_results(frame);
    render_search_helpers_with_data(frame, input, toast_message);
    render_syntax_highlighting(frame);
    render_toast_with_data(frame, toast_message, error_type);
    render_completion_dropdown(frame, completions, completion_index, show_completions);
}

fn render_logo(frame: &mut Frame) {
    // Generate ASCII art using the crate (thanks thomas)
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

fn render_search_bar_with_data(
    frame: &mut Frame,
    input: &str,
    problematic_tokens: &[(usize, usize)],
) {
    let search_width = 50;

    // Position search bar directly below the logo
    let logo_height = 7; // Height of the ASCII art logo
    let search_y = logo_height + 2; // 2 lines below the logo

    let search_area = Rect {
        x: frame.area().width.saturating_sub(search_width) / 2,
        y: search_y,
        width: search_width,
        height: 3,
    };

    // Create styled spans for the input with highlighted problematic tokens
    let mut styled_spans = Vec::new();

    // Start with the "> " prefix
    styled_spans.push(Span::styled("> ", Style::default().fg(Color::White)));

    // Process the input character by character, highlighting problematic tokens
    for (i, ch) in input.chars().enumerate() {
        let is_problematic = problematic_tokens.iter().any(|&(start, end)| {
            // Token positions are relative to the input string, so we need to match them correctly
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

fn render_search_helpers_with_data(frame: &mut Frame, input: &str, toast_message: &Option<String>) {
    // Don't show help text if there's an active toast
    if toast_message.is_some() {
        return;
    }

    let help_text = if input.is_empty() {
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

fn render_query_results(frame: &mut Frame) {
    let _ = frame;
    todo!()
}

fn render_syntax_highlighting(frame: &mut Frame) {
    let _ = frame;
    todo!()
}

fn render_toast_with_data(
    frame: &mut Frame,
    toast_message: &Option<String>,
    error_type: &Option<ErrorType>,
) {
    if let Some(message) = toast_message {
        // Use the passed error type to determine toast dimensions
        let is_parser_error = matches!(error_type, Some(ErrorType::Parser));

        // Calculate toast dimensions based on error type
        let (toast_width, max_toast_height) = if is_parser_error {
            // Parser errors need more space for context and suggestions
            (80_u16, 15)
        } else {
            // Lexer errors are typically shorter
            (60, 8)
        };

        // Wrap text to fit within the toast width (account for borders and padding)
        let content_width = toast_width.saturating_sub(4) as usize; // -4 for borders and padding
        let mut wrapped_lines = Vec::new();

        for line in message.lines() {
            if line.len() <= content_width {
                wrapped_lines.push(line.to_string());
            } else {
                // Split long lines into multiple lines
                let mut remaining = line;
                while !remaining.is_empty() {
                    if remaining.len() <= content_width {
                        wrapped_lines.push(remaining.to_string());
                        break;
                    } else {
                        // Find a good break point (space, comma, etc.)
                        let mut break_point = content_width;
                        if let Some(space_pos) = remaining[..content_width].rfind(' ') {
                            break_point = space_pos;
                        } else if let Some(comma_pos) = remaining[..content_width].rfind(',') {
                            break_point = comma_pos + 1; // Include the comma
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

        // Create styled lines for the toast
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
    let dropdown_height = (completions.len() as u16).min(8) + 2; // Dynamic height based on completions, max 8 items + borders

    // Position below the search bar
    let logo_height = 7; // Height of the ASCII art logo
    let search_y = logo_height + 2; // Search bar position
    let search_height = 3; // Search bar height
    let dropdown_y = search_y + search_height + 1; // 1 line below search bar

    let dropdown_area = Rect {
        x: frame.area().width.saturating_sub(dropdown_width) / 2,
        y: dropdown_y,
        width: dropdown_width,
        height: dropdown_height,
    };

    let mut styled_lines = Vec::new();
    for (i, completion) in completions.iter().enumerate() {
        let style = if Some(i) == completion_index {
            Style::default().fg(Color::Black).bg(Color::White) // Better contrast for selected item
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
