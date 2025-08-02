/*
Lexer for the query language.

The lexer is responsible for converting the input string into a stream of tokens.

The lexer is implemented as a state machine.

*/

use super::token::{Token, TokenType};
use regex::Regex;

pub struct Lexer {
    input: String,
    chars: Vec<char>,
    position: usize,
    current_char: char,
    unrecognized_chars: Vec<char>,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            input: String::new(),
            chars: Vec::new(),
            position: 0,
            current_char: ' ',
            unrecognized_chars: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.input = String::new();
        self.chars.clear();
        self.position = 0;
        self.current_char = ' ';
        self.unrecognized_chars.clear();
    }

    pub fn lexical_analysis(&mut self, input: String) -> Vec<Token> {
        // Store input and initialize position
        self.input = input;
        self.chars = self.input.chars().collect();
        self.position = 0;
        self.current_char = if self.chars.is_empty() { '\0' } else { self.chars[0] };
        
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
        
        while self.position < self.chars.len() {
            let remaining: String = self.chars[self.position..].iter().collect();
            
            // Skip whitespace
            if self.current_char.is_whitespace() {
                self.advance();
                continue;
            }
            
            let mut matched = false;
            for (token_type, regex) in &compiled_patterns {
                if let Some(mat) = regex.find(remaining.as_str()) {
                    if mat.start() == 0 { // Must match at beginning
                        let lexeme = mat.as_str().to_string();
                        let start_pos = self.position as i32;
                        let end_pos = (self.position + mat.len()) as i32;
                        let token = Token::new(token_type.clone(), lexeme, start_pos, end_pos);
                        tokens.push(token);
                        
                        // Advance position by match length
                        for _ in 0..mat.len() {
                            self.advance();
                        }
                        matched = true;
                        break;
                    }
                }
            }
            
            if !matched {
                // Handle unrecognized character - collect it for error reporting
                self.unrecognized_chars.push(self.current_char);
                self.advance();
            }
        }
        
        // If we found unrecognized characters, return them as error tokens instead
        if !self.unrecognized_chars.is_empty() {
            let mut error_tokens = Vec::new();
            for (i, &ch) in self.unrecognized_chars.iter().enumerate() {
                let token = Token::new(
                    TokenType::Unrecognized, 
                    ch.to_string(), 
                    i as i32, 
                    (i + 1) as i32
                );
                error_tokens.push(token);
            }
            return error_tokens;
        }
        
        tokens
    }

    // Helper method to advance position and update current_char
    fn advance(&mut self) {
        self.position += 1;
        if self.position < self.chars.len() {
            self.current_char = self.chars[self.position];
        } else {
            self.current_char = '\0'; // End of input
        }
    }
}