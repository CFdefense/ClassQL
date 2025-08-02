/*
Lexer for the query language.

The lexer is responsible for converting the input string into a stream of tokens.

The lexer is implemented as a state machine.

*/

use super::token::Token;

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

    pub fn begin_lexing(&mut self, input: String) -> Vec<Token> {
        Vec::new()
    }
}