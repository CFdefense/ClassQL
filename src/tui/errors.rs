use std::error::Error;
use std::fmt::{Display, Formatter};

impl Error for TUIError {}

#[derive(Debug)]
pub enum TUIError {
    TerminalError(String),
}

impl Display for TUIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TUIError::TerminalError(msg) => write!(f, "Terminal error: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AppError {
    NONE,
    SyntaxError(SyntaxError),
    UnrecognizedTokens(String, Vec<(usize, usize)>),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NONE => write!(f, "No error"),
            AppError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            AppError::UnrecognizedTokens(msg, _) => write!(f, "Unrecognized tokens: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    NOIMPL,
    UnexpectedToken(String),
    MissingToken(String),
    InvalidExpression(String),
    UnclosedParenthesis,
    InvalidOperator(String),
    EmptyQuery,
    InvalidIdentifier(String),
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxError::NOIMPL => write!(f, "Not implemented"),
            SyntaxError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            SyntaxError::MissingToken(token) => write!(f, "Missing token: {}", token),
            SyntaxError::InvalidExpression(expr) => write!(f, "Invalid expression: {}", expr),
            SyntaxError::UnclosedParenthesis => write!(f, "Unclosed parenthesis"),
            SyntaxError::InvalidOperator(op) => write!(f, "Invalid operator: {}", op),
            SyntaxError::EmptyQuery => write!(f, "Empty query"),
            SyntaxError::InvalidIdentifier(id) => write!(f, "Invalid identifier: {}", id),
        }
    }
}
