/// tests/lexer_tests.rs
/// 
/// Lexer tests
/// 
/// Responsible for testing the lexer
/// 
/// Contains:
/// --- ---
/// TestCase -> Test case struct
/// ExpectedToken -> Expected token struct
/// TestHelper -> Test helper struct
/// --- ---

use std::fs;

use classql::dsl::lexer::Lexer;
use serde::{Deserialize, Serialize};


/// Test case struct
/// 
/// Fields:
/// --- ---
/// test_name -> The name of the test
/// description -> The description of the test
/// code -> The code to test
/// should_succeed -> Whether the test should succeed
/// expected_error -> The expected error
/// result -> The expected result
/// --- ---
/// 
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for TestCase
/// Deserialize -> Deserialize trait for TestCase
/// Serialize -> Serialize trait for TestCase
/// --- ---
/// 
#[derive(Debug, Deserialize, Serialize)]
struct TestCase {
    test_name: String,
    description: String,
    code: String,
    #[serde(default)]
    should_succeed: Option<bool>,
    #[serde(default)]
    expected_error: Option<String>,
    result: Vec<ExpectedToken>,
}

/// Expected token struct
/// 
/// Fields:
/// --- ---
/// token_type -> The type of the token
/// content -> The content of the token
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
    token_type: String,
    content: String,
}

/// Test helper struct
/// 
/// Fields:
/// --- ---
/// None
/// --- ---
/// 
/// Implemented Traits:
/// --- ---
/// Default -> Default trait for TestHelper
/// --- ---
/// 
#[derive(Default)]
struct TestHelper {}

/// Test helper implementation
/// 
/// Methods:
/// --- ---
/// new -> Create a new TestHelper
/// parse_json_tests -> Parse the JSON tests
/// run_test -> Run a test
/// --- ---
/// 
impl TestHelper {
    /// Create a new TestHelper
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
    /// TestHelper -> The new TestHelper
    /// --- ---
    /// 
    fn new() -> Self {
        Self {
            // TODO: ..Default::default()
        }
    }

    /// Parse the JSON tests
    /// 
    /// Parameters:
    /// --- ---
    /// self -> The TestHelper instance
    /// json_content -> The JSON content to parse
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// Vec<TestCase> -> The parsed test cases
    /// --- ---
    /// 
    fn parse_json_tests(&self, json_content: &str) -> Vec<TestCase> {
        serde_json::from_str(json_content).expect("Failed to parse JSON test file")
    }

    /// Run a test
    /// 
    /// Parameters:
    /// --- ---
    /// self -> The TestHelper instance
    /// test_case -> The test case to run
    /// --- ---
    /// 
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    /// 
    fn run_test(&mut self, test_case: &TestCase) {
        println!("Running test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        println!("Input: '{}'", test_case.code);

        // Clear lexer state before each test
        let mut lexer = Lexer::new(test_case.code.clone());

        // Tokenize the input
        match lexer.analyze() {
            Ok(tokens) => {
                // Default to expecting success unless explicitly marked as false
                if test_case.should_succeed == Some(false) {
                    panic!(
                        "Test '{}' expected to fail but succeeded with {} tokens",
                        test_case.test_name,
                        tokens.len()
                    );
                }

                // Print actual tokens
                println!(
                    "Expected {} tokens, got {} tokens",
                    test_case.result.len(),
                    tokens.len()
                );

                if tokens.len() != test_case.result.len() {
                    println!("\n=== TOKEN COUNT MISMATCH ===");
                    println!("Expected tokens:");
                    for (i, expected) in test_case.result.iter().enumerate() {
                        println!("  [{}] {} = '{}'", i, expected.token_type, expected.content);
                    }
                    println!("Actual tokens:");
                    for (i, actual) in tokens.iter().enumerate() {
                        println!(
                            "  [{}] {} = '{}'",
                            i,
                            actual.get_token_type(),
                            &test_case.code[actual.get_start()..actual.get_end()]
                        );
                    }
                    println!("========================\n");
                }

                // Compare results
                assert_eq!(
                    tokens.len(),
                    test_case.result.len(),
                    "Token count mismatch in test '{}'. Expected: {}, Got: {}",
                    test_case.test_name,
                    test_case.result.len(),
                    tokens.len()
                );

                let mut has_diff = false;
                for (i, (actual, expected)) in
                    tokens.iter().zip(test_case.result.iter()).enumerate()
                {
                    let type_match = actual.get_token_type().to_string() == expected.token_type;
                    let content_match =
                        test_case.code[actual.get_start()..actual.get_end()] == expected.content;

                    if !type_match || !content_match {
                        if !has_diff {
                            println!("\n=== TOKEN DIFFERENCES ===");
                            has_diff = true;
                        }
                        println!("Position [{}]:", i);
                        if !type_match {
                            println!(
                                "  Type:    Expected '{}' but got '{}'",
                                expected.token_type,
                                actual.get_token_type()
                            );
                        }
                        if !content_match {
                            println!(
                                "  Content: Expected '{}' but got '{}'",
                                expected.content,
                                &test_case.code[actual.get_start()..actual.get_end()]
                            );
                        }
                    }

                    assert_eq!(
                        actual.get_token_type().to_string(),
                        expected.token_type,
                        "Token type mismatch at position {} in test '{}'. Expected: {}, Got: {}",
                        i,
                        test_case.test_name,
                        expected.token_type,
                        actual.get_token_type()
                    );

                    assert_eq!(
                        &test_case.code[actual.get_start()..actual.get_end()],
                        expected.content,
                        "Token content mismatch at position {} in test '{}'. Expected: {}, Got: {}",
                        i,
                        test_case.test_name,
                        expected.content,
                        &test_case.code[actual.get_start()..actual.get_end()]
                    );
                }

                if has_diff {
                    println!("========================\n");
                } else {
                    println!("All tokens match!\n");
                }
            }
            Err(e) => {
                // Default to expecting success unless explicitly marked as false
                if test_case.should_succeed == Some(false) {
                    // Check if we expected a specific error
                    if let Some(expected_error) = &test_case.expected_error {
                        let error_str = format!("{:?}", e);
                        if !error_str.contains(expected_error) {
                            panic!(
                                "Test '{}' expected error containing '{}' but got: {:?}",
                                test_case.test_name, expected_error, e
                            );
                        }
                    }

                    println!("Test failed as expected with error: {:?}\n", e);
                } else {
                    panic!(
                        "Test '{}' expected to succeed but failed with error: {:?}",
                        test_case.test_name, e
                    );
                }
            }
        }
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
    let path = format!("tests/lexer/{}", filename);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read test file: {}", path))
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
    let mut helper = TestHelper::new();
    let content = load_test_file(filename);
    let test_cases = helper.parse_json_tests(&content);

    for test_case in test_cases {
        helper.run_test(&test_case);
    }
}

#[test]
fn test_basic_keywords() {
    run_test_file("basic_keywords.json");
}

#[test]
fn test_operators() {
    run_test_file("operators.json");
}

#[test]
fn test_literals() {
    run_test_file("literals.json");
}

#[test]
fn test_days() {
    run_test_file("days.json");
}

#[test]
fn test_complex_queries() {
    run_test_file("complex_queries.json");
}

#[test]
fn test_whitespace() {
    run_test_file("whitespace.json");
}

#[test]
fn test_edge_cases() {
    run_test_file("edge_cases.json");
}

#[test]
fn test_stress_tests() {
    run_test_file("stress_tests.json");
}

#[test]
fn test_malformed_tests() {
    run_test_file("malformed_tests.json");
}

#[test]
fn test_boundary_tests() {
    run_test_file("boundary_tests.json");
}

#[test]
fn test_unrecognized_tests() {
    run_test_file("unrecognized_tests.json");
}

#[test]
fn test_parentheses_grouping() {
    run_test_file("parentheses_grouping_tests.json");
}

#[test]
fn test_core_tokens() {
    run_test_file("core_tokens.json");
}

#[test]
fn test_core_tokens_edge_cases() {
    run_test_file("core_tokens_edge_cases.json");
}

#[test]
fn test_core_tokens_stress_tests() {
    run_test_file("core_tokens_stress_tests.json");
}

#[test]
fn test_time_queries() {
    run_test_file("time_queries.json");
}

#[test]
fn test_time_edge_cases() {
    run_test_file("time_edge_cases.json");
}
