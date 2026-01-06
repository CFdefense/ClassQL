/// tests/codegen_tests.rs
///
/// Code generator tests
///
/// Responsible for testing the code generator using JSON-defined test cases,
/// similar to lexer, parser, and semantic tests.
///
/// Contains:
/// --- ---
/// CodegenTestCase -> Codegen test case struct
/// CodegenTestHelper -> Codegen test helper struct
///     Methods:
///     --- ---
///     new -> Create a new CodegenTestHelper
///     run_test -> Run a codegen test case
///     --- ---
/// Helper functions:
///     --- ---
///     load_test_file -> Load the test file
///     run_test_file -> Run the test file
///     --- ---
/// --- ---
///

use classql::dsl::codegen::generate_sql;
use classql::dsl::lexer::Lexer;
use classql::dsl::parser::Parser;
use classql::dsl::semantic::semantic_analysis;
use serde::{Deserialize, Serialize};
use std::fs;

/// Codegen test case struct
///
/// Fields:
/// --- ---
/// test_name -> The name of the test
/// description -> The description of the test
/// input -> The input query to generate SQL for
/// should_succeed -> Whether code generation should succeed
/// expected_fragments -> SQL fragments that should appear in the output (optional)
/// forbidden_fragments -> SQL fragments that should NOT appear in the output (optional)
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for CodegenTestCase
/// Deserialize -> Deserialize trait for CodegenTestCase
/// Serialize -> Serialize trait for CodegenTestCase
/// --- ---
///
#[derive(Debug, Deserialize, Serialize)]
struct CodegenTestCase {
    test_name: String,
    description: String,
    input: String,
    should_succeed: bool,
    #[serde(default)]
    expected_fragments: Vec<String>,
    #[serde(default)]
    forbidden_fragments: Vec<String>,
}

/// Codegen test helper struct
///
/// Fields:
/// --- ---
/// None
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Default -> Default trait for CodegenTestHelper
/// --- ---
///
#[derive(Default)]
struct CodegenTestHelper {}

/// Codegen test helper implementation
///
/// Methods:
/// --- ---
/// new -> Create a new CodegenTestHelper
/// run_test -> Run a codegen test case
/// --- ---
///
impl CodegenTestHelper {
    /// Create a new CodegenTestHelper
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// CodegenTestHelper -> The new CodegenTestHelper
    /// --- ---
    ///
    fn new() -> Self {
        Self {}
    }

    /// Run a codegen test case
    ///
    /// Parameters:
    /// --- ---
    /// self -> The CodegenTestHelper instance
    /// test_case -> The codegen test case to run
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn run_test(&mut self, test_case: &CodegenTestCase) {
        println!("Running codegen test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        println!("Input: '{}'", test_case.input);
        println!("Expected to succeed: {}", test_case.should_succeed);

        // Lexical analysis
        let mut lexer = Lexer::new(test_case.input.to_string());
        let tokens = match lexer.analyze() {
            Ok(tokens) => tokens,
            Err(e) => {
                if test_case.should_succeed {
                    panic!(
                        "Lexer failed in codegen test '{}' with error: {:?}",
                        test_case.test_name, e
                    );
                } else {
                    println!("Lexer failed as expected: {:?}\n", e);
                    return;
                }
            }
        };

        // Parsing
        let mut parser = Parser::new(test_case.input.to_string());
        let ast = match parser.parse(&tokens) {
            Ok(ast) => ast,
            Err((error, _remaining)) => {
                if test_case.should_succeed {
                    panic!(
                        "Parser failed in codegen test '{}' with error: {:?}",
                        test_case.test_name, error
                    );
                } else {
                    println!("Parser failed as expected: {:?}\n", error);
                    return;
                }
            }
        };

        // Semantic analysis
        if let Err((error, _positions)) = semantic_analysis(&ast) {
            if test_case.should_succeed {
                panic!(
                    "Semantic analysis failed in codegen test '{}' with error: {:?}",
                    test_case.test_name, error
                );
            } else {
                println!("Semantic analysis failed as expected: {:?}\n", error);
                return;
            }
        }

        // Code generation
        match generate_sql(&ast) {
            Ok(sql) => {
                if !test_case.should_succeed {
                    panic!(
                        "Code generation succeeded but was expected to fail in test '{}'\nGenerated SQL: {}",
                        test_case.test_name, sql
                    );
                }

                println!("Generated SQL:\n{}\n", sql);

                // Check expected fragments
                for fragment in &test_case.expected_fragments {
                    assert!(
                        sql.contains(fragment),
                        "Test '{}': Expected SQL to contain '{}' but it didn't.\nFull SQL: {}",
                        test_case.test_name,
                        fragment,
                        sql
                    );
                }

                // Check forbidden fragments
                for fragment in &test_case.forbidden_fragments {
                    assert!(
                        !sql.contains(fragment),
                        "Test '{}': SQL should NOT contain '{}' but it did.\nFull SQL: {}",
                        test_case.test_name,
                        fragment,
                        sql
                    );
                }

                println!("Code generation succeeded as expected\n");
            }
            Err(error) => {
                if test_case.should_succeed {
                    panic!(
                        "Code generation failed but was expected to succeed in test '{}': {:?}",
                        test_case.test_name, error
                    );
                } else {
                    println!("Code generation failed as expected with error: {:?}\n", error);
                }
            }
        }
    }
}

/// Load the codegen test file
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
    let path = format!("tests/codegen/{filename}");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read test file: {path}"))
}

/// Run the codegen test file
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
    let mut helper = CodegenTestHelper::new();
    let content = load_test_file(filename);
    let test_cases: Vec<CodegenTestCase> =
        serde_json::from_str(&content).expect("Failed to parse codegen JSON test file");

    for test_case in test_cases {
        helper.run_test(&test_case);
    }
}

#[test]
fn test_codegen_basic_queries() {
    run_test_file("basic_queries.json");
}

#[test]
fn test_codegen_string_conditions() {
    run_test_file("string_conditions.json");
}

#[test]
fn test_codegen_numeric_queries() {
    run_test_file("numeric_queries.json");
}

#[test]
fn test_codegen_time_queries() {
    run_test_file("time_queries.json");
}

#[test]
fn test_codegen_day_queries() {
    run_test_file("day_queries.json");
}

#[test]
fn test_codegen_logical_operators() {
    run_test_file("logical_operators.json");
}

#[test]
fn test_codegen_complex_queries() {
    run_test_file("complex_queries.json");
}

#[test]
fn test_codegen_keyword_variations() {
    run_test_file("keyword_variations.json");
}

#[test]
fn test_codegen_edge_cases() {
    run_test_file("edge_cases.json");
}

