/// src/tui/errors.rs
/// 
/// Errors for the TUI
///
/// Responsible for handling errors that occur in the TUI
///
/// Contains:
/// --- ---
/// TUIError -> TUI error enum
/// AppError -> Application error enum
/// SyntaxError -> Syntax error enum
/// Other helper functions:
///      --- ---
///      extract_user_text -> Extract the user text from the token
///      make_user_friendly_for_completion -> Make technical terms more user-friendly for completion
///      make_user_friendly -> Make technical terms more user-friendly
///      --- ---
/// --- ---

use std::error::Error;
use std::fmt::{Display, Formatter};

/// TUIError enum
///
/// TUIError types:
/// --- ---
/// TerminalError -> Terminal error
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for TUIError
/// Error -> Error trait for TUIError
/// Display -> Display trait for TUIError
/// --- ---
///
#[derive(Debug)]
pub enum TUIError {
    TerminalError(String),
}

/// TUIError Error Trait Implementation
/// 
/// Implements the Error trait for TUIError
/// 
impl Error for TUIError {}

/// TUIError Display Trait Implementation
/// 
/// Parameters:
/// --- ---
/// self -> The TUIError to display
/// f -> The formatter to display the TUIError
/// --- ---
///
/// Returns:
/// --- ---
/// std::fmt::Result -> The result of the display
/// --- ---
///
impl Display for TUIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TUIError::TerminalError(msg) => write!(f, "Terminal error: {msg}"),
        }
    }
}

/// AppError enum
///
/// AppError types:
/// --- ---
/// Empty -> Empty error
/// SyntaxError -> Syntax error
/// UnrecognizedTokens -> Unrecognized tokens
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for AppError
/// PartialEq -> PartialEq trait for AppError
/// Display -> Display trait for AppError
/// --- ---
///
#[derive(Debug, PartialEq)]
pub enum AppError {
    Empty,
    SyntaxError(SyntaxError),
    UnrecognizedTokens(String, Vec<(usize, usize)>),
}

/// AppError Display Trait Implementation
/// 
/// Parameters:
/// --- ---
/// self -> The AppError to display
/// f -> The formatter to display the AppError
/// --- ---
///
/// Returns:
/// --- ---
/// std::fmt::Result -> The result of the display
/// --- ---
///
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Empty => write!(f, "No error"),
            AppError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            AppError::UnrecognizedTokens(msg, _) => write!(f, "Unrecognized tokens: {}", msg),
        }
    }
}

/// SyntaxError enum
///
/// SyntaxError types:
/// --- ---
/// MissingToken -> Missing token
/// UnclosedParenthesis -> Unclosed parenthesis
/// EmptyQuery -> Empty query
/// ExpectedAfter -> Expected after
/// InvalidContext -> Invalid context
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for SyntaxError
/// PartialEq -> PartialEq trait for SyntaxError
/// Display -> Display trait for SyntaxError
/// --- ---
/// 
#[derive(Debug, PartialEq, Clone)]
pub enum SyntaxError {
    MissingToken(String),
    UnclosedParenthesis,
    EmptyQuery,
    ExpectedAfter {
        expected: Vec<String>,
        after: String,
        position: usize,
    },
    InvalidContext {
        token: String,
        context: String,
        suggestions: Vec<String>,
    },
}

/// SyntaxError Display Trait Implementation
/// 
/// Parameters:
/// --- ---
/// self -> The SyntaxError to display
/// f -> The formatter to display the SyntaxError
/// --- ---
///
/// Returns:
/// --- ---
/// std::fmt::Result -> The result of the display
/// --- ---
///
impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxError::MissingToken(token) => write!(f, "Missing: {}", token),
            SyntaxError::UnclosedParenthesis => write!(f, "Missing closing parenthesis ')'"),
            SyntaxError::EmptyQuery => write!(f, "Please enter a query to search"),
            SyntaxError::ExpectedAfter {
                expected,
                after,
                position: _,
            } => {
                let user_friendly_expected: Vec<String> = expected
                    .iter()
                    .map(|s| format!("'{}'", make_user_friendly(s)))
                    .collect();

                let user_friendly_after = make_user_friendly(after);

                // special case for start of query - use "Please start with" instead of "After..."
                if after == "start of query" {
                    if user_friendly_expected.len() == 1 {
                        write!(f, "Please start with: {}", user_friendly_expected[0])
                    } else {
                        write!(
                            f,
                            "Please start with one of: {}",
                            user_friendly_expected.join(", ")
                        )
                    }
                } else if user_friendly_expected.len() == 1 {
                    write!(
                        f,
                        "After '{}', please add: {}",
                        user_friendly_after, user_friendly_expected[0]
                    )
                } else {
                    write!(
                        f,
                        "After '{}', please add one of: {}",
                        user_friendly_after,
                        user_friendly_expected.join(", ")
                    )
                }
            }
            SyntaxError::InvalidContext {
                token,
                context,
                suggestions,
            } => {
                let clean_token = extract_user_text(token);
                let user_friendly_context = make_user_friendly(context);

                if suggestions.is_empty() {
                    write!(
                        f,
                        "'{}' is not valid here ({})",
                        clean_token, user_friendly_context
                    )
                } else {
                    let user_friendly_suggestions: Vec<String> = suggestions
                        .iter()
                        .map(|s| format!("'{}'", make_user_friendly(s)))
                        .collect();
                    write!(
                        f,
                        "'{}' is not valid here. Try: {}",
                        clean_token,
                        user_friendly_suggestions.join(", ")
                    )
                }
            }
        }
    }
}

/// Helper function to extract the actual user text from technical token descriptions
/// 
/// Parameters:
/// --- ---
/// token -> The token to extract the user text from
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The user text from the token
/// --- ---
///
fn extract_user_text(token: &str) -> String {
    // handle patterns like "T_IDENTIFIER ('man')" -> "man"
    if let Some(start) = token.find("('") {
        if let Some(end) = token.rfind("')") {
            return token[start + 2..end].to_string();
        }
    }

    // handle patterns like "T_CONTAINS" -> "contains"
    if let Some(token_name) = token.strip_prefix("T_") {
        // extract the token name and convert to user-friendly format
        return token_name.to_lowercase().replace("_", " ");
    }

    // otherwise return the token as-is
    token.to_string()
}

/// Helper function to make technical terms more user-friendly for completion
/// 
/// Parameters:
/// --- ---
/// term -> The term to make more user-friendly
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The user-friendly term
/// --- ---
///
pub fn make_user_friendly_for_completion(term: &str) -> String {
    match term {
        "prof" => "professor".to_string(),
        "corereqs" => "corequisites".to_string(),
        "prereqs" => "prerequisites".to_string(),
        "starts" => "starts with".to_string(),
        "ends" => "ends with".to_string(),
        "remove extra text" => "remove extra words".to_string(),
        "text value" => "<value>".to_string(),
        "quoted string" => "\"text\"".to_string(),
        "identifier" => "<name>".to_string(),
        _ => term.replace("_", " ").to_lowercase(),
    }
}

/// Helper function to make technical terms more user-friendly
/// 
/// Parameters:
/// --- ---
/// term -> The term to make more user-friendly
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The user-friendly term
/// --- ---
///
fn make_user_friendly(term: &str) -> String {
    match term {
        "entity keyword" => "search field".to_string(),
        "string condition" => "when searching".to_string(),
        "after string condition" => "when searching for text".to_string(),
        "numeric comparison" => "when comparing numbers".to_string(),
        "query start" => "at the beginning".to_string(),
        "end of query" => "at the end".to_string(),
        "start of query" => "the beginning".to_string(),
        "prof" => "professor".to_string(),
        "corereqs" => "corequisites".to_string(),
        "prereqs" => "prerequisites".to_string(),
        "is" => "is".to_string(),
        "equals" => "equals".to_string(),
        "contains" => "contains".to_string(),
        "has" => "has".to_string(),
        "starts" => "starts with".to_string(),
        "ends" => "ends with".to_string(),
        "and" => "and".to_string(),
        "or" => "or".to_string(),
        "remove extra text" => "remove extra words".to_string(),
        "text value" => "some text".to_string(),
        "quoted string" => "text in quotes".to_string(),
        "identifier" => "a name or value".to_string(),
        "remove duplicate operator" => "remove the repeated word".to_string(),
        _ => term.replace("_", " ").to_lowercase(),
    }
}
