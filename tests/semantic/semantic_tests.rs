/// tests/semantic_tests.rs
///
/// Semantic analyzer tests
///
/// Responsible for testing the semantic analyzer using JSON-defined test cases,
/// similar to lexer and parser tests.
///
/// Contains:
/// --- ---
/// SemanticTestCase -> Semantic test case struct
/// SemanticTestHelper -> Semantic test helper struct
///     Methods:
///     --- ---
///     new -> Create a new SemanticTestHelper
///     run_test -> Run a semantic test case
///     --- ---
/// Helper functions:
///     --- ---
///     load_test_file -> Load the test file
///     run_test_file -> Run the test file
///     --- ---
/// --- ---
///

use classql::dsl::lexer::Lexer;
use classql::dsl::parser::Parser;
use classql::dsl::semantic::semantic_analysis;
use classql::tui::errors::SemanticError;
use serde::{Deserialize, Serialize};
use crate::utils;

/// Semantic test case struct
///
/// Fields:
/// --- ---
/// test_name -> The name of the test
/// description -> The description of the test
/// input -> The input query to analyze
/// should_succeed -> Whether semantic analysis should succeed
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for SemanticTestCase
/// Deserialize -> Deserialize trait for SemanticTestCase
/// Serialize -> Serialize trait for SemanticTestCase
/// --- ---
///
#[derive(Debug, Deserialize, Serialize)]
struct SemanticTestCase {
    test_name: String,
    description: String,
    input: String,
    should_succeed: bool,
}

/// Semantic test helper struct
///
/// Fields:
/// --- ---
/// None
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Default -> Default trait for SemanticTestHelper
/// --- ---
///
#[derive(Default)]
struct SemanticTestHelper {}

/// Semantic test helper implementation
///
/// Methods:
/// --- ---
/// new -> Create a new SemanticTestHelper
/// run_test -> Run a semantic test case
/// --- ---
///
impl SemanticTestHelper {
    /// Create a new SemanticTestHelper
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// SemanticTestHelper -> The new SemanticTestHelper
    /// --- ---
    ///
    fn new() -> Self {
        Self {
            // TODO: ..Default::default()
        }
    }

    /// Run a semantic test case
    ///
    /// Parameters:
    /// --- ---
    /// self -> The SemanticTestHelper instance
    /// test_case -> The semantic test case to run
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn run_test(&mut self, test_case: &SemanticTestCase) {
        println!("Running semantic test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        println!("Input: '{}'", test_case.input);
        println!("Expected to succeed: {}", test_case.should_succeed);

        // lexical analysis
        let mut lexer = Lexer::new(test_case.input.to_string());
        let tokens = match lexer.analyze() {
            Ok(tokens) => tokens,
            Err(e) => {
                panic!(
                    "Lexer failed in semantic test '{}' with error: {:?}",
                    test_case.test_name, e
                );
            }
        };

        // parsing
        let mut parser = Parser::new(test_case.input.to_string());
        let ast = match parser.parse(&tokens) {
            Ok(ast) => ast,
            Err((error, remaining)) => {
                panic!(
                    "Parser failed in semantic test '{}' with error: {:?}, remaining tokens: {:?}",
                    test_case.test_name, error, remaining
                );
            }
        };

        // semantic analysis
        match semantic_analysis(&ast) {
            Ok(()) => {
                if !test_case.should_succeed {
                    panic!(
                        "Semantic analysis succeeded but was expected to fail in test '{}'",
                        test_case.test_name
                    );
                } else {
                    println!("Semantic analysis succeeded as expected\n");
                }
            }
            Err((error, positions)) => {
                if test_case.should_succeed {
                    panic!(
                        "Semantic analysis failed but was expected to succeed in test '{}': {:?}",
                        test_case.test_name, error
                    );
                } else {
                    // we currently expect all semantic failures to be InvalidContext.
                    match error {
                        SemanticError::InvalidContext { .. } => {
                            println!(
                                "Semantic analysis failed as expected with error: {:?}",
                                error
                            );
                        }
                    }

                    // basic sanity checks for reported positions: they must be within input bounds.
                    for (start, end) in positions {
                        assert!(
                            end > start,
                            "Semantic error position end should be greater than start"
                        );
                        assert!(
                            end <= test_case.input.len(),
                            "Semantic error position end should not exceed input length"
                        );
                    }

                    println!();
                }
            }
        }
    }
}

/// Run the semantic test file
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
    let mut helper = SemanticTestHelper::new();
    let content = utils::load_test_file("semantic", filename);
    let test_cases: Vec<SemanticTestCase> =
        serde_json::from_str(&content).expect("Failed to parse semantic JSON test file");

    for test_case in test_cases {
        helper.run_test(&test_case);
    }
}

#[test]
fn test_semantic_basic_valid_queries() {
    run_test_file("basic_valid_queries.json");
}

#[test]
fn test_semantic_basic_invalid_queries() {
    run_test_file("basic_invalid_queries.json");
}




