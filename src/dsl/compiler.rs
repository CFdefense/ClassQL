use super::lexer::Lexer;
use super::parser::{Ast, Parser};
use crate::tui::errors::AppError;

/// Result Types for the Compiler
/// 
/// Results:
/// --- ---
/// Sucess -> Compilation was successful, contains message and AST
/// LexerError -> Lexical analysis failed, contains message and problematic tokens
/// ParserError -> Parsing failed, contains message and problematic tokens
/// --- ---
/// 
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

/// Compiler for the DSL
///
/// Responsible for compiling the DSL into a SQL query
///
pub struct Compiler {}

/// Compiler methods
/// 
/// Methods:
/// --- ---
/// new -> Create a new compiler instance
/// run -> Compile the DSL into a SQL query
/// get_tab_completion -> Get tab completion suggestions for the current input
/// --- ---
/// 
impl Compiler {
    /// Create a new compiler instance
    ///
    /// TODO: implement future functionality for cleaner state refresh
    ///
    pub fn new() -> Self {
        Compiler {
            // ..Default::default() // TODO: implement future functionality
        }
    }

    /// Compile the DSL into a SQL query
    ///
    /// Will return a CompilerResult
    ///
    /// Will return a CompilerResult::Success if the compilation is successful
    ///
    /// Will return a CompilerResult::LexerError if the lexical analysis fails
    ///
    pub fn run(&mut self, input: &str) -> CompilerResult {
        // refresh lexer state
        let mut lexer = Lexer::new(input.to_string());

        // perform lexical analysis
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

        // parse the tokens
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
    /// 
    /// Partial Compilation Method
    ///
    /// Will preform lexical analysis on the input and then a special parser method to get completion suggestions
    /// 
    /// Will return a vector of strings of completion suggestions
    ///
    pub fn get_tab_completion(&mut self, input: String) -> Vec<String> {
        // refresh lexer state
        let mut lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(input.to_string());

        // try to analyze the input
        match lexer.analyze() {
            Ok(tokens) => {
                // lexical analysis succeeded, now try to get completion suggestions from parser
                parser.get_completion_suggestions(&tokens)
            }
            Err(_) => {
                // lexical analysis failed, provide basic suggestions
                if input.trim().is_empty() {
                    vec![
                        "professor".to_string(),
                        "course".to_string(),
                        "subject".to_string(),
                        "title".to_string(),
                        "section".to_string(),
                    ]
                } else {
                    vec![] // can't provide suggestions for invalid tokens
                }
            }
        }
    }
}
