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

pub enum AppError {
    NONE,
    SyntaxError(SyntaxError),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NONE => write!(f, "No error"),
            AppError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
        }
    }
}

pub enum SyntaxError {
    NONE,
    InvalidSyntax(String),
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxError::NONE => write!(f, "No error"),
            SyntaxError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
        }
    }
}

