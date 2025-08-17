/*
Lexer for the query language.

The lexer is responsible for converting the input string into a stream of tokens.

Will return a vector of tokens.

If there is an unrecognized character, it will return a vector of error tokens.

These error tokens should be used to tell the user that there is some unrecognized character in the input.
*/

use super::token::{Token, TokenType};
use crate::tui::errors::AppError;
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

    pub fn lexical_analysis(&mut self, input: String) -> Result<Vec<Token>, AppError> {
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
        
        let mut all_tokens = Vec::new();
        let mut byte_pos = 0;
        
        // First pass: parse the entire input and collect all tokens
        while byte_pos < self.input.len() {
            let remaining = &self.input[byte_pos..];
            
            // Skip whitespace
            if remaining.starts_with(char::is_whitespace) {
                let next_char = remaining.chars().next().unwrap();
                byte_pos += next_char.len_utf8();
                continue;
            }
            
            let mut matched = false;
            for (token_type, regex) in &compiled_patterns {
                if let Some(mat) = regex.find(remaining) {
                    if mat.start() == 0 { // Must match at beginning
                        let lexeme = mat.as_str().to_string();
                        let start_pos = byte_pos as i32;
                        let end_pos = (byte_pos + mat.len()) as i32;
                        let token = Token::new(token_type.clone(), lexeme.clone(), start_pos, end_pos);
                        all_tokens.push(token);
                        
                        // Advance byte position by match length
                        byte_pos += mat.len();
                        matched = true;
                        break;
                    }
                }
            }
            
            if !matched {
                // Found unrecognized character - collect it
                let next_char = remaining.chars().next().unwrap();
                let token = Token::new(
                    TokenType::Unrecognized,
                    next_char.to_string(),
                    byte_pos as i32,
                    (byte_pos + next_char.len_utf8()) as i32
                );
                all_tokens.push(token);
                byte_pos += next_char.len_utf8();
            }
        }
        
        // Check if we found any unrecognized tokens
        let unrecognized_tokens: Vec<Token> = all_tokens
            .iter()
            .filter(|token| matches!(token.get_token_type(), TokenType::Unrecognized))
            .cloned()
            .collect();
        
        // If we found any unrecognized characters, return an error
        if !unrecognized_tokens.is_empty() {
            let problematic_positions: Vec<(usize, usize)> = unrecognized_tokens
                .iter()
                .map(|token| (token.get_start() as usize, token.get_end() as usize))
                .collect();
            
            let unrecognized_chars: Vec<String> = unrecognized_tokens
                .iter()
                .map(|token| format!("'{}'", token.get_lexeme()))
                .collect();
            
            let message = format!(
                "Unrecognized character{}: {}",
                if unrecognized_chars.len() > 1 { "s" } else { "" },
                unrecognized_chars.join(", ")
            );
            
            return Err(AppError::UnrecognizedTokens(message, problematic_positions));
        }
        
        // Otherwise return all valid tokens
        Ok(all_tokens)
    }
}