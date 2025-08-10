use super::lexer::Lexer;
use super::parser::Parser;
use crate::tui::errors::AppError;

pub struct Compiler {
    lexer: Lexer,
    parser: Parser,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            lexer: Lexer::new(),
            parser: Parser::new(),
        }
    }

    pub fn run(&mut self, input: &str) -> CompilerResult {
        // Clear previous state
        self.lexer.clear();
        
        // Perform lexical analysis
        let tokens = match self.lexer.lexical_analysis(input.to_string()) {
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
        let _ = match self.parser.parse(&tokens) {
            Ok(()) => (),
            Err(error_tuple) => {
                let (e, problematic_tokens) = error_tuple;
                let problematic_positions: Vec<(usize, usize)> = problematic_tokens
                    .iter()
                    .map(|token| (token.get_start() as usize, token.get_end() as usize))
                    .collect();
                return CompilerResult::ParserError {
                    message: e.to_string(),
                    problematic_tokens: problematic_positions,
                };
            }
        };
        CompilerResult::Success { message: "Success".to_string() }
    }
}

#[derive(Debug)]
pub enum CompilerResult {
    Success {
        message: String,
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