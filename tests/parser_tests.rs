/// tests/parser_tests.rs
/// 
/// Parser tests
/// 
/// Responsible for testing the parser
/// 
/// Contains:
/// --- ---
/// ParserTestCase -> Parser test case struct
/// ExpectedToken -> Expected token struct
/// ParserTestHelper -> Parser test helper struct
///     Methods:
///     --- ---
///     new -> Create a new ParserTestHelper
///     create_tokens -> Create tokens from input
///     test_parse -> Test the parser
///     validate_problematic_tokens -> Validate problematic tokens
///     validate_error_type -> Validate error type
///     validate_error_message -> Validate error message
///     --- ---
/// Helper functions:
///     --- ---
///     load_test_file -> Load the test file
///     run_test_file -> Run the test file
///     --- ---
/// --- ---

use classql::dsl::lexer::Lexer;
use classql::dsl::parser::Parser;
use classql::dsl::token::Token;
use classql::tui::errors::SyntaxError;
use serde::{Deserialize, Serialize};
use std::fs;

/// Parser test case struct
/// 
/// Fields:
/// --- ---
/// test_name -> The name of the test
/// description -> The description of the test
/// input -> The input to parse
/// should_succeed -> Whether the test should succeed
/// expected_error_type -> The expected error type
/// expected_error_message -> The expected error message
/// expected_problematic_tokens -> The expected problematic tokens
/// --- ---
/// 
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for ParserTestCase
/// Deserialize -> Deserialize trait for ParserTestCase
/// Serialize -> Serialize trait for ParserTestCase
/// --- ---
/// 
#[derive(Debug, Deserialize, Serialize)]
struct ParserTestCase {
    test_name: String,
    description: String,
    input: String,
    should_succeed: bool,
    expected_error_type: Option<String>,
    expected_error_message: Option<String>,
    expected_problematic_tokens: Option<Vec<ExpectedToken>>,
}

/// Expected token struct
/// 
/// Fields:
/// --- ---
/// lexeme -> The lexeme of the token
/// start -> The start position of the token
/// end -> The end position of the token
/// --- ---
/// 
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for ExpectedToken
/// Deserialize -> Deserialize trait for ExpectedToken
/// Serialize -> Serialize trait for ExpectedToken
/// --- ---
/// 
#[derive(Debug, Deserialize, Serialize)]
struct ExpectedToken {
    lexeme: String,
    start: usize,
    end: usize,
}

/// Parser test helper struct
/// 
/// Fields:
/// --- ---
/// None
/// --- ---
/// 
/// Implemented Traits:
/// --- ---
/// Default -> Default trait for ParserTestHelper
/// --- ---
/// 
#[derive(Default)]
struct ParserTestHelper {}

/// Parser test helper implementation
/// 
/// Methods:
/// --- ---
/// new -> Create a new ParserTestHelper
/// create_tokens -> Create tokens from input
/// test_parse -> Test the parser
/// validate_problematic_tokens -> Validate problematic tokens
/// validate_error_type -> Validate error type
/// validate_error_message -> Validate error message
/// --- ---
/// 
impl ParserTestHelper {
    /// Create a new ParserTestHelper
    /// 
    /// TODO: This should be implemented further
    /// 
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// ParserTestHelper -> The new ParserTestHelper
    /// --- ---
    /// 
    fn new() -> Self {
        Self {
            // TODO: ..Default::default()
        }
    }

    /// Create tokens from input
    /// 
    /// Parameters:
    /// --- ---
    /// self -> The ParserTestHelper instance
    /// input -> The input to create tokens from
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// Vec<Token> -> The created tokens
    /// --- ---
    /// 
    fn create_tokens(&mut self, input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input.to_string());
        match lexer.analyze() {
            Ok(tokens) => tokens,
            Err(_) => panic!("Failed to tokenize input: {}", input),
        }
    }

    /// Test the parser
    /// 
    /// Parameters:
    /// --- ---
    /// self -> The ParserTestHelper instance
    /// test_case -> The test case to test
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    /// 
    fn test_parse(&mut self, test_case: &ParserTestCase) {
        println!("Running test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        println!("Input: '{}'", test_case.input);
        println!("Expected to succeed: {}", test_case.should_succeed);

        let tokens = self.create_tokens(&test_case.input);
        println!("Generated {} tokens", tokens.len());

        let mut parser = Parser::new(test_case.input.to_string());
        let result = parser.parse(&tokens);

        match result {
            Ok(_) => {
                if test_case.should_succeed {
                    println!("Parse succeeded as expected");
                    // TODO: Add AST validation here once AST structure is implemented
                } else {
                    panic!("Parse succeeded but was expected to fail");
                }
            }
            Err((error, problematic_tokens)) => {
                if test_case.should_succeed {
                    panic!("Parse failed but was expected to succeed: {:?}", error);
                } else {
                    println!("Parse failed as expected with error: {:?}", error);
                    println!("Problematic tokens: {:?}", problematic_tokens);

                    // Validate that problematic tokens have valid positions
                    for token in &problematic_tokens {
                        assert!(
                            token.get_end() > token.get_start(),
                            "Token end should be greater than start"
                        );
                        assert!(
                            token.get_end() <= test_case.input.len(),
                            "Token end should not exceed input length"
                        );
                    }

                    // Validate specific error type if expected
                    if let Some(expected_error_type) = &test_case.expected_error_type {
                        self.validate_error_type(&error, expected_error_type);
                    }

                    // Validate specific error message if expected
                    if let Some(expected_error_message) = &test_case.expected_error_message {
                        self.validate_error_message(&error, expected_error_message);
                    }

                    // If we have expected problematic tokens, validate them
                    if let Some(expected_tokens) = &test_case.expected_problematic_tokens {
                        self.validate_problematic_tokens(
                            &problematic_tokens,
                            expected_tokens,
                            &test_case.input,
                        );
                    }
                }
            }
        }
        println!();
    }

    /// 
    /// Parameters:
    /// --- ---
    /// self -> The ParserTestHelper instance
    /// actual -> The actual tokens
    /// expected -> The expected tokens
    /// input -> The input to validate the tokens from
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    /// 
    fn validate_problematic_tokens(
        &self,
        actual: &[Token],
        expected: &[ExpectedToken],
        input: &str,
    ) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "Expected {} problematic tokens, but got {}",
            expected.len(),
            actual.len()
        );

        for (i, (actual_token, expected_token)) in actual.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                &input[actual_token.get_start()..actual_token.get_end()],
                expected_token.lexeme,
                "Token {} lexeme mismatch: expected '{}', got '{}'",
                i,
                expected_token.lexeme,
                &input[actual_token.get_start()..actual_token.get_end()]
            );

            assert_eq!(
                actual_token.get_start(),
                expected_token.start,
                "Token {} start position mismatch: expected {}, got {}",
                i,
                expected_token.start,
                actual_token.get_start()
            );

            assert_eq!(
                actual_token.get_end(),
                expected_token.end,
                "Token {} end position mismatch: expected {}, got {}",
                i,
                expected_token.end,
                actual_token.get_end()
            );

            // verify the token content matches the input at those positions
            let token_content = &input[expected_token.start..expected_token.end];
            assert_eq!(
                token_content, expected_token.lexeme,
                "Token content mismatch for token {}",
                i
            );
        }
    }

    /// Validate error type
    /// 
    /// Parameters:
    /// --- ---
    /// self -> The ParserTestHelper instance
    /// actual_error -> The actual error
    /// expected_error_type -> The expected error type
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    /// 
    fn validate_error_type(&self, actual_error: &SyntaxError, expected_error_type: &str) {
        let actual_error_type = match actual_error {
            SyntaxError::MissingToken(_) => "MissingToken",
            SyntaxError::UnclosedParenthesis => "UnclosedParenthesis",
            SyntaxError::EmptyQuery => "EmptyQuery",
            SyntaxError::ExpectedAfter { .. } => "ExpectedAfter",
            SyntaxError::InvalidContext { .. } => "InvalidContext",
        };

        assert_eq!(
            actual_error_type, expected_error_type,
            "Expected error type '{}', but got '{}'",
            expected_error_type, actual_error_type
        );
    }

    /// Validate error message
    /// 
    /// Parameters:
    /// --- ---
    /// self -> The ParserTestHelper instance
    /// actual_error -> The actual error
    /// expected_message -> The expected error message
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    /// 
    fn validate_error_message(&self, actual_error: &SyntaxError, expected_message: &str) {
        let actual_message = actual_error.to_string();
        assert_eq!(
            actual_message, expected_message,
            "Expected error message '{}', but got '{}'",
            expected_message, actual_message
        );
    }
}

/// Load the test file
/// 
/// Parameters:
/// --- ---
/// filename -> The filename to load
/// --- ---
/// 
/// Returns:
/// --- ---
/// String -> The content of the test file
/// --- ---
/// 
fn load_test_file(filename: &str) -> String {
    let path = format!("tests/parser/{filename}",);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read test file: {path}"))
}

/// Run the test file
/// 
/// Parameters:
/// --- ---
/// filename -> The filename to run
/// --- ---
/// 
/// Returns:
/// --- ---
/// None
/// --- ---
/// 
fn run_test_file(filename: &str) {
    let mut helper = ParserTestHelper::new();
    let content = load_test_file(filename);
    let test_cases: Vec<ParserTestCase> =
        serde_json::from_str(&content).expect("Failed to parse JSON test file");

    for test_case in test_cases {
        helper.test_parse(&test_case);
    }
}

#[test]
fn test_basic_valid_queries() {
    run_test_file("basic_valid_queries.json");
}

#[test]
fn test_complex_valid_queries() {
    run_test_file("complex_valid_queries.json");
}

#[test]
fn test_invalid_syntax_queries() {
    run_test_file("invalid_syntax_queries.json");
}

#[test]
fn test_malformed_operators() {
    run_test_file("malformed_operators.json");
}

#[test]
fn test_empty_and_whitespace() {
    run_test_file("empty_and_whitespace.json");
}

#[test]
fn test_nested_expressions() {
    run_test_file("nested_expressions.json");
}

#[test]
fn test_time_and_day_queries() {
    run_test_file("time_and_day_queries.json");
}

#[test]
fn test_time_queries() {
    run_test_file("time_queries.json");
}

#[test]
fn test_enrollment_queries() {
    run_test_file("enrollment_queries.json");
}

#[test]
fn test_size_queries() {
    run_test_file("size_queries.json");
}

#[test]
fn test_string_literals() {
    run_test_file("string_literals.json");
}

#[test]
fn test_numeric_queries() {
    run_test_file("numeric_queries.json");
}

#[test]
fn test_identifier_queries() {
    run_test_file("identifier_queries.json");
}

#[test]
fn test_error_recovery() {
    run_test_file("error_recovery.json");
}

#[test]
fn test_token_position_tracking() {
    run_test_file("token_position_tracking.json");
}

#[test]
fn test_ast_structure() {
    run_test_file("ast_structure.json");
}

#[test]
fn test_edge_cases() {
    run_test_file("edge_cases.json");
}
