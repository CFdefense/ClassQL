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
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};
use text_to_ascii_art::to_art;
use crate::compiler::lexer::Lexer;
use std::time::{Duration, Instant};
use crate::compiler::token::TokenType;

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
            // Update toast timer
            self.appstate.update_toast();
            
            self.terminal.draw(|f| {
                self.appstate.render(f);
            })?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break Ok(()),
                    KeyCode::Enter => {
                        // Process the query here
                        self.appstate.user_query = self.appstate.input.clone();
                        
                        // Clear the lexer state before processing new input
                        self.appstate.lexer.clear();
                        
                        // Lex the input and check for unrecognized tokens
                        let tokens = self.appstate.lexer.lexical_analysis(self.appstate.input.clone());
                        
                        // Clear previous problematic tokens
                        self.appstate.problematic_tokens.clear();
                        
                        if let Some(AppError::UnrecognizedTokens(error_msg)) = Lexer::handle_unrecognized_tokens(&tokens) {
                            // Track problematic token positions for highlighting
                            for token in &tokens {
                                if matches!(token.get_token_type(), TokenType::Unrecognized) {
                                    let start = token.get_start() as usize;
                                    let end = token.get_end() as usize;
                                    self.appstate.problematic_tokens.push((start, end));
                                }
                            }
                            self.appstate.show_toast(error_msg);
                        }
                    
                    },
                    KeyCode::Backspace => {
                        self.appstate.input.pop();
                    },
                    KeyCode::Char(c) => {
                        // Clear any previous toasts and problematic tokens when user starts typing
                        self.appstate.toast_message = None;
                        self.appstate.toast_start_time = None;
                        self.appstate.problematic_tokens.clear();
                        self.appstate.input.push(c);
                    },
                    _ => {}
                }
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
    user_query: String,
    lexer: Lexer,
    toast_message: Option<String>,
    toast_start_time: Option<Instant>,
    problematic_tokens: Vec<(usize, usize)>,
}

impl AppState {
    fn new() -> Self {
        return AppState {
            input: String::new(),
            error: AppError::NONE,
            user_query: String::new(),
            lexer: Lexer::new(),
            toast_message: None,
            toast_start_time: None,
            problematic_tokens: Vec::new(),
        };
    }
    
    fn show_toast(&mut self, message: String) {
        self.toast_message = Some(message);
        self.toast_start_time = Some(Instant::now());
    }
    
    fn update_toast(&mut self) {
        if let Some(start_time) = self.toast_start_time {
            if start_time.elapsed() > Duration::from_secs(3) {
                self.toast_message = None;
                self.toast_start_time = None;
            }
        }
    }
    
    fn render_logo(&self, frame: &mut Frame) {
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
            x: frame.area().width.saturating_sub(85) /2,
            y: 1,
            width: 80,
            height: ascii_art.len() as u16,
        };
        
        let logo_paragraph = Paragraph::new(lines);
        frame.render_widget(logo_paragraph, logo_area);
    }

    // function to render the search bar 
    fn render_search_bar(&self, frame: &mut Frame) {
        let search_width = 50;
        let search_area = Rect {
            x: frame.area().width / 2 - 25,
            y: 10, // Below the logo with more space
            width: search_width,
            height: 3,
        };
        
        // Create styled spans for the input with highlighted problematic tokens
        let mut styled_spans = Vec::new();
        
        // Start with the "> " prefix
        styled_spans.push(Span::styled("> ", Style::default().fg(Color::White)));
        
        // Process the input character by character, highlighting problematic tokens
        for (i, ch) in self.input.chars().enumerate() {
            let is_problematic = self.problematic_tokens.iter().any(|&(start, end)| {
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
                    .border_style(Style::default().fg(Color::Gray))
            );
        
        frame.render_widget(search_paragraph, search_area);
    }

    // function to render the query results in the tui
    fn render_query_results(&self, frame: &mut Frame) {}

    // function to render the users search text in the tui
    fn render_search_helpers(&self, frame: &mut Frame) {
        let help_area = Rect {
            x: frame.area().width / 2 - 22,
            y: 14, // Below the search bar with proper spacing
            width: 50,
            height: 2,
        };
        
        let help_text = if self.input.is_empty() {
            "Type a ClassQL query (e.g., 'prof is Alan')"
        } else {
            "Press Enter to Search, Esc to Exit"
        };
        
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default());
            
        frame.render_widget(help_paragraph, help_area);
    }

    // function to render the syntax highlighting in the tui
    fn render_syntax_highlighting(&self, frame: &mut Frame) {}

    // function to render toast notifications
    fn render_toast(&self, frame: &mut Frame) {
        if let Some(message) = &self.toast_message {
            // Split message into lines
            let lines: Vec<String> = message.lines().map(|s| s.to_string()).collect();
            
            // Calculate toast position (bottom middle)
            let toast_width = 60;
            let toast_height = lines.len() as u16 + 2; // +2 for border
            
            let toast_area = Rect {
                x: (frame.area().width.saturating_sub(toast_width)) / 2,
                y: frame.area().height.saturating_sub(toast_height + 2),
                width: toast_width,
                height: toast_height,
            };
            
            // Create styled lines for the toast
            let styled_lines: Vec<Line> = lines
                .iter()
                .map(|line| {
                    Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::White),
                    ))
                })
                .collect();
            
            let toast_paragraph = Paragraph::new(styled_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Error")
                        .title_style(Style::default().fg(Color::Yellow))
                        .border_style(Style::default().fg(Color::Red))
                        .style(Style::default().bg(Color::DarkGray))
                );
            
            frame.render_widget(toast_paragraph, toast_area);
        }
    }

    // function to render the entire app state
    pub fn render(&self, frame: &mut Frame) {
        self.render_logo(frame);
        self.render_search_bar(frame);
        self.render_query_results(frame);
        self.render_search_helpers(frame);
        self.render_syntax_highlighting(frame);
        self.render_toast(frame);
    }
}
