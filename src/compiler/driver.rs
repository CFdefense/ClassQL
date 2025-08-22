use super::lexer::Lexer;
use super::parser::{Ast, Parser};
use crate::tui::errors::AppError;

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            // ..Default::default()
        }
    }

    pub fn run(&mut self, input: &str) -> CompilerResult {
        // refresh state
        let mut lexer = Lexer::new(input.to_string());

        // Perform lexical analysis
        let tokens = match lexer.analyze() {
            Ok(tokens) => tokens,
            Err(AppError::UnrecognizedTokens(error_msg, problematic_tokens)) => {
                return CompilerResult::LexerError {
                    message: error_msg,
                    problematic_tokens,
                };
            }
            Err(_) => {
                return CompilerResult::LexerError {
                    message: "Unknown lexer error".to_string(),
                    problematic_tokens: Vec::new(),
                };
            }
        };

        // Parse the tokens
        let mut parser = Parser::new(input.to_string());

        let ast = match parser.parse(&tokens) {
            Ok(ast) => ast,
            Err(error_tuple) => {
                let (e, problematic_tokens) = error_tuple;
                let problematic_positions: Vec<(usize, usize)> = problematic_tokens
                    .iter()
                    .map(|token| (token.get_start(), token.get_end()))
                    .collect();
                return CompilerResult::ParserError {
                    message: e.to_string(),
                    problematic_tokens: problematic_positions,
                };
            }
        };
        CompilerResult::Success {
            message: "Success".to_string(),
            ast,
        }
    }

    /// Get tab completion suggestions for the current input
    pub fn get_tab_completion(&mut self, input: String) -> Vec<String> {
        // refresh state
        let mut lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(input.to_string());

        // First, try to lex the input
        match lexer.analyze() {
            Ok(tokens) => {
                // Lexing succeeded, now try to get completion suggestions from parser
                parser.get_completion_suggestions(&tokens)
            }
            Err(_) => {
                // Lexing failed, provide basic suggestions
                if input.trim().is_empty() {
                    vec![
                        "professor".to_string(),
                        "course".to_string(),
                        "subject".to_string(),
                        "title".to_string(),
                        "section".to_string(),
                    ]
                } else {
                    vec![] // Can't provide suggestions for invalid tokens
                }
            }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum CompilerResult {
    Success {
        message: String,
        ast: Ast,
    },
    LexerError {
        message: String,
        problematic_tokens: Vec<(usize, usize)>,
    },
    ParserError {
        message: String,
        problematic_tokens: Vec<(usize, usize)>,
    },
}
