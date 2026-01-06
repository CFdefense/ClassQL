/// src/dsl/lexer.rs
///
/// Lexer for the query language.
///
/// Responsible for converting the input string into a stream of tokens.
///
/// Contains:
/// --- ---
/// Lexer -> Lexer struct
///      Methods:
///      --- ---
///      new -> Create a new lexer instance
///      get_lexeme -> Get the lexeme of a token
///      analyze -> Convert the input string into a stream of tokens
///      --- ---
/// --- ---
///
use super::token::{Token, TokenType};
use crate::tui::errors::AppError;
use regex::Regex;

/// Lexer for the query language.
///
/// Responsible for converting the input string into a stream of tokens.
///
/// Fields:
/// --- ---
/// input_string -> The input string to convert into tokens
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// None -> No implemented traits
/// --- ---
///
pub struct Lexer {
    input_string: String,
}

/// Lexer Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new lexer instance
/// get_lexeme -> Get the lexeme of a token
/// analyze -> Convert the input string into a stream of tokens
/// --- ---
///
impl Lexer {
    /// Create a new lexer instance
    ///
    /// Parameters:
    /// --- ---
    /// input_string -> The input string to convert into tokens
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Lexer -> The new lexer instance
    /// --- ---
    ///
    pub fn new(input_string: String) -> Self {
        Lexer { input_string }
    }

    /// Get the lexeme of a token
    ///
    /// Parameters:
    /// --- ---
    /// token -> The token to get the lexeme of
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// &str -> The lexeme of the token
    /// --- ---
    ///
    fn get_lexeme(&self, token: &Token) -> &str {
        &self.input_string[token.get_start()..token.get_end()]
    }

    /// Convert the input string into a stream of tokens
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<Vec<Token>, AppError>:
    ///     Ok -> The stream of tokens
    ///     Err -> An error occurred, contains message and problematic tokens
    /// --- ---
    ///
    pub fn analyze(&mut self) -> Result<Vec<Token>, AppError> {
        // Get all patterns in lexing order (longest/most specific first)
        let patterns = TokenType::all_patterns();

        // Compile patterns once for efficiency
        let compiled_patterns: Vec<(TokenType, Regex)> = patterns
            .into_iter()
            .map(|(token_type, pattern)| (token_type, Regex::new(pattern).unwrap()))
            .collect();

        let mut all_tokens = Vec::new();
        let mut byte_pos = 0;

        // First pass: parse the entire input and collect all tokens
        while byte_pos < self.input_string.len() {
            let remaining = &self.input_string[byte_pos..];

            // Skip whitespace
            if remaining.starts_with(char::is_whitespace) {
                let next_char = remaining.chars().next().unwrap();
                byte_pos += next_char.len_utf8();
                continue;
            }

            let mut matched = false;
            for (token_type, regex) in &compiled_patterns {
                if let Some(mat) = regex.find(remaining) {
                    if mat.start() == 0 {
                        // Must match at beginning
                        let start_pos = byte_pos;
                        let end_pos = byte_pos + mat.len();
                        let token = Token::new(*token_type, start_pos, end_pos);
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
                    byte_pos,
                    byte_pos + next_char.len_utf8(),
                );
                all_tokens.push(token);
                byte_pos += next_char.len_utf8();
            }
        }

        // Check if we found any unclosed string tokens
        let unclosed_strings: Vec<Token> = all_tokens
            .iter()
            .filter(|token| matches!(token.get_token_type(), TokenType::UnclosedString))
            .cloned()
            .collect();

        // If we found any unclosed strings, return an error
        if !unclosed_strings.is_empty() {
            let problematic_positions: Vec<(usize, usize)> = unclosed_strings
                .iter()
                .map(|token| (token.get_start(), token.get_end()))
                .collect();

            return Err(AppError::UnrecognizedTokens(
                "Unclosed string - add closing quote \"".to_string(),
                problematic_positions,
            ));
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
                .map(|token| (token.get_start(), token.get_end()))
                .collect();

            let unrecognized_chars: Vec<String> = unrecognized_tokens
                .iter()
                .map(|token| format!("'{}'", self.get_lexeme(token)))
                .collect();

            let message = format!(
                "Unrecognized character{}: {}",
                if unrecognized_chars.len() > 1 {
                    "s"
                } else {
                    ""
                },
                unrecognized_chars.join(", ")
            );

            return Err(AppError::UnrecognizedTokens(message, problematic_positions));
        }

        // Otherwise return all valid tokens
        Ok(all_tokens)
    }
}
