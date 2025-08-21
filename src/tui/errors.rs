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
            TUIError::TerminalError(msg) => write!(f, "Terminal error: {msg}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AppError {
    Empty,
    SyntaxError(SyntaxError),
    UnrecognizedTokens(String, Vec<(usize, usize)>),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Empty => write!(f, "No error"),
            AppError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            AppError::UnrecognizedTokens(msg, _) => write!(f, "Unrecognized tokens: {}", msg),
        }
    }
}

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

                // Special case for start of query - use "Please start with" instead of "After..."
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
                } else {
                    if user_friendly_expected.len() == 1 {
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

// Helper function to extract the actual user text from technical token descriptions
fn extract_user_text(token: &str) -> String {
    // Handle patterns like "T_IDENTIFIER ('man')" -> "man"
    if let Some(start) = token.find("('") {
        if let Some(end) = token.rfind("')") {
            return token[start + 2..end].to_string();
        }
    }

    // Handle patterns like "T_CONTAINS" -> "contains"
    if token.starts_with("T_") {
        // Extract the token name and convert to user-friendly format
        let token_name = &token[2..]; // Remove "T_" prefix
        return token_name.to_lowercase().replace("_", " ");
    }

    // Otherwise return the token as-is
    token.to_string()
}

// Helper function to make technical terms more user-friendly for completion
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

// Helper function to make technical terms more user-friendly
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
