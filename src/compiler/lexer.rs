/*
Lexer for the query language.

The lexer is responsible for converting the input string into a stream of tokens.

The lexer is implemented as a state machine.

*/

use super::token::{Token, TokenType};
use regex::Regex;

pub struct Lexer {
    input: String,
    position: usize,
    current_char: char,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            input: String::new(),
            position: 0,
            current_char: ' ',
        }
    }

    pub fn clear(&mut self) {
        self.input = String::new();
        self.position = 0;
        self.current_char = ' ';
    }

    pub fn lexical_analysis(&mut self, input: String) -> Vec<Token> {
        // Example of how to use the regex patterns:
        
        // Get all patterns in lexing order (longest/most specific first)
        let patterns = TokenType::all_patterns();
        
        // Compile patterns once for efficiency
        let compiled_patterns: Vec<(TokenType, Regex)> = patterns
            .into_iter()
            .map(|(token_type, pattern)| {
                (token_type, Regex::new(pattern).unwrap())
            })
            .collect();
        
        let mut tokens = Vec::new();
        let mut position = 0;
        
        while position < input.len() {
            let remaining = &input[position..];
            
            // Skip whitespace
            if remaining.chars().next().unwrap().is_whitespace() {
                position += 1;
                continue;
            }
            
            let mut matched = false;
            for (token_type, regex) in &compiled_patterns {
                if let Some(mat) = regex.find(remaining) {
                    if mat.start() == 0 { // Must match at beginning
                        let lexeme = mat.as_str().to_string();
                        let token = Token::new(token_type.clone(), lexeme, position as i32, (position + mat.len()) as i32);
                        tokens.push(token);
                        position += mat.len();
                        matched = true;
                        break;
                    }
                }
            }
            
            if !matched {
                // Handle unrecognized character
                position += 1;
            }
        }
        
        tokens
    }

    // Helper method to get individual pattern
    pub fn get_pattern_for_token(token_type: &TokenType) -> &'static str {
        token_type.regex_pattern()
    }
}