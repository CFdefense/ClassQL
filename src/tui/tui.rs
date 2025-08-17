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

use crate::compiler::compiler::{Compiler, CompilerResult};
use crate::tui::errors::TUIError;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use std::time::{Duration, Instant};

pub struct Tui<'a> {
    terminal: DefaultTerminal,
    input: String,
    user_query: String,
    toast_message: Option<String>,
    toast_start_time: Option<Instant>,
    problematic_tokens: Vec<(usize, usize)>,
    compiler: &'a mut Compiler,
}

impl<'a> Tui<'a> {
    pub fn new(compiler: &'a mut Compiler) -> Result<Self, TUIError> {
        // allows for direct terminal input access
        enable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO ENABLE RAW MODE")))?;

        // initialize the terminal instance
        let terminal = ratatui::init();

        // initialize and return our tui instance
        return Ok(Tui {
            terminal,
            input: String::new(),
            user_query: String::new(),
            toast_message: None,
            toast_start_time: None,
            problematic_tokens: Vec::new(),
            compiler,
        });
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

            // Draw the current state
            let terminal = &mut self.terminal;
            terminal.draw(|f| {
                render_frame(f, &input, &problematic_tokens, &toast_message);
            })?;

            // Handle input events
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Enter => {
                        // Process the query here
                        self.user_query = self.input.clone();
                        
                        // Run the compiler and handle the result
                        match self.compiler.run(&self.input) {
                            CompilerResult::Success { message } => {
                                // Clear any error state
                                self.toast_message = None;
                                self.show_toast(message);
                                self.toast_start_time = None;
                                self.problematic_tokens.clear();
                                // TODO: Process successful tokens
                            }
                            CompilerResult::LexerError { message, problematic_tokens } => {
                                // Show error and highlight problematic tokens
                                self.show_toast(message);
                                self.problematic_tokens = problematic_tokens;
                            }
                            CompilerResult::ParserError { message, problematic_tokens } => {
                                // Show error and highlight problematic tokens
                                self.show_toast(message);
                                self.problematic_tokens = problematic_tokens;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        // Clear any previous toasts and problematic tokens when user backspaces
                        self.clear_error_state();
                        self.input.pop();
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
        self.problematic_tokens.clear();
    }

    // function to terminate the tui gracefully
    pub fn terminate(&self) -> Result<(), TUIError> {
        disable_raw_mode()
            .map_err(|_| TUIError::TerminalError(String::from("FAILED TO DISABLE RAW MODE")))?;
        Ok(ratatui::restore())
    }

    fn update_toast(&mut self) {
        if let Some(start_time) = self.toast_start_time {
            if start_time.elapsed() > Duration::from_secs(3) {
                self.toast_message = None;
                self.toast_start_time = None;
            }
        }
    }

    fn show_toast(&mut self, message: String) {
        self.toast_message = Some(message);
        self.toast_start_time = Some(Instant::now());
    }
}

// Standalone render function to avoid borrow checker conflicts
fn render_frame(frame: &mut Frame, input: &str, problematic_tokens: &[(usize, usize)], toast_message: &Option<String>) {
    render_logo(frame);
    render_search_bar_with_data(frame, input, problematic_tokens);
    render_query_results(frame);
    render_search_helpers_with_data(frame, input, toast_message);
    render_syntax_highlighting(frame);
    render_toast_with_data(frame, toast_message);
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
        y: frame.area().height.saturating_sub(ascii_art.len() as u16 + 1),
        width: 80,
        height: ascii_art.len() as u16,
    };

    let logo_paragraph = Paragraph::new(lines);
    frame.render_widget(logo_paragraph, logo_area);
}

fn render_search_bar_with_data(frame: &mut Frame, input: &str, problematic_tokens: &[(usize, usize)]) {
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
        "Type a ClassQL query (e.g., 'prof is Alan')"
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

fn render_query_results(frame: &mut Frame) {}

fn render_syntax_highlighting(frame: &mut Frame) {}

fn render_toast_with_data(frame: &mut Frame, toast_message: &Option<String>) {
    if let Some(message) = toast_message {
        // Split message into lines
        let lines: Vec<String> = message.lines().map(|s| s.to_string()).collect();

        // Calculate toast position (bottom middle)
        let toast_width = 60;
        let max_toast_height = 10;
        let toast_height = (lines.len() as u16 + 2).min(max_toast_height);

        let toast_area = Rect {
            x: (frame.area().width.saturating_sub(toast_width)) / 2,
            y: frame.area().height.saturating_sub(toast_height + 1),
            width: toast_width,
            height: toast_height,
        };

        // Create styled lines for the toast
        let styled_lines: Vec<Line> = lines
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
