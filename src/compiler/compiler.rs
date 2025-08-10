use super::lexer::Lexer;
use super::token::Token;
use super::parser::Parser;
use crate::tui::errors::AppError;

pub struct Compiler {
    lexer: Lexer,
    parser: Option<Parser>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            lexer: Lexer::new(),
            parser: None,
        }
    }

    pub fn run(&mut self, input: &str) -> CompilerResult {
        // Clear previous state
        self.lexer.clear();
        
        // Perform lexical analysis
        let tokens = self.lexer.lexical_analysis(input.to_string());
        
        // Check for unrecognized tokens
        if let Some(AppError::UnrecognizedTokens(error_msg)) = Lexer::handle_unrecognized_tokens(&tokens) {
            // Collect problematic token positions for highlighting
            let mut problematic_positions = Vec::new();
            for token in &tokens {
                if matches!(token.get_token_type(), super::token::TokenType::Unrecognized) {
                    let start = token.get_start() as usize;
                    let end = token.get_end() as usize;
                    problematic_positions.push((start, end));
                }
            }
            
            return CompilerResult::Error {
                message: error_msg,
                problematic_tokens: problematic_positions,
            };
        }

        // TODO: Parse tokens here
        // let parser = Parser::new(tokens);
        
        CompilerResult::Success { tokens }
    }

    // TODO: Add parse method here
    // fn parse(&self, tokens: Vec<Token>) -> AST {
    //     // Parsing logic will go here
    // }
}

#[derive(Debug)]
pub enum CompilerResult {
    Success {
        tokens: Vec<Token>,
        // ast: AST, // Will be added when parsing is implemented
    },
    Error {
        message: String,
        problematic_tokens: Vec<(usize, usize)>,
    },
} 