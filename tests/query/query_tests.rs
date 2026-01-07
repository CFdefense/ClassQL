/// tests/query_tests.rs
///
/// End-to-end query tests
///
/// Responsible for testing the full compiler pipeline (lexer -> parser -> semantic -> codegen -> database execution)
/// with hardcoded expected results based on actual database queries.
///
/// Contains:
/// --- ---
/// QueryTestCase -> Query test case struct
/// QueryTestHelper -> Query test helper struct
///     Methods:
///     --- ---
///     new -> Create a new QueryTestHelper
///     run_test -> Run a query test case
///     --- ---
/// Helper functions:
///     --- ---
///     load_test_file -> Load the test file
///     run_test_file -> Run the test file
///     --- ---
/// --- ---
///

use classql::dsl::compiler::Compiler;
use serde::{Deserialize, Serialize};
use crate::utils;

/// Query test case struct
///
/// Fields:
/// --- ---
/// test_name -> The name of the test
/// description -> The description of the test
/// input -> The input query to test
/// should_succeed -> Whether the query should succeed
/// expected_count -> Expected number of results (optional)
/// expected_classes -> Hardcoded expected class results (optional)
/// min_count -> Minimum number of results (optional)
/// max_count -> Maximum number of results (optional)
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for QueryTestCase
/// Deserialize -> Deserialize trait for QueryTestCase
/// Serialize -> Serialize trait for QueryTestCase
/// --- ---
///
#[derive(Debug, Deserialize, Serialize)]
struct QueryTestCase {
    test_name: String,
    description: String,
    input: String,
    should_succeed: bool,
    #[serde(default)]
    expected_count: Option<usize>,
    #[serde(default)]
    expected_classes: Vec<ExpectedClass>,
    #[serde(default)]
    min_count: Option<usize>,
    #[serde(default)]
    max_count: Option<usize>,
}

/// Expected class result struct
///
/// Fields:
/// --- ---
/// subject_code -> Expected subject code
/// course_number -> Expected course number
/// title -> Expected course title (optional, can be partial match)
/// professor_name -> Expected professor name (optional)
/// --- ---
///
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ExpectedClass {
    subject_code: String,
    course_number: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    professor_name: Option<String>,
    #[serde(default)]
    section_sequence: Option<String>,
}

/// Query test helper struct
///
/// Fields:
/// --- ---
/// db_path -> Path to the test database
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// None
/// --- ---
///
struct QueryTestHelper {}

/// Query test helper implementation
///
/// Methods:
/// --- ---
/// new -> Create a new QueryTestHelper
/// run_test -> Run a query test case
/// --- ---
///
impl QueryTestHelper {
    /// Create a new QueryTestHelper
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// QueryTestHelper -> The new QueryTestHelper
    /// --- ---
    ///
    fn new() -> Self {
        QueryTestHelper {}
    }

    /// Run a query test case
    ///
    /// Parameters:
    /// --- ---
    /// self -> The QueryTestHelper instance
    /// test_case -> The query test case to run
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn run_test(&mut self, test_case: &QueryTestCase) {
        println!("Running query test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        println!("Input: '{}'", test_case.input);
        println!("Expected to succeed: {}", test_case.should_succeed);

        // Create compiler with test database
        let mut compiler = Compiler::new();
        
        // Override database path for testing
        // Note: This requires modifying the compiler to accept a db_path parameter
        // For now, we'll use the default path and assume test_db.db is in the right place
        
        // Run the compiler
        let result = compiler.run(&test_case.input);

        match result {
            classql::dsl::compiler::CompilerResult::Success { classes, .. } => {
                if !test_case.should_succeed {
                    panic!(
                        "Query test '{}' succeeded but was expected to fail. Got {} results.",
                        test_case.test_name,
                        classes.len()
                    );
                }

                println!("Query succeeded. Got {} results.", classes.len());

                // Check count constraints
                if let Some(expected_count) = test_case.expected_count {
                    assert_eq!(
                        classes.len(),
                        expected_count,
                        "Query test '{}': Expected {} results, got {}",
                        test_case.test_name,
                        expected_count,
                        classes.len()
                    );
                }

                if let Some(min_count) = test_case.min_count {
                    assert!(
                        classes.len() >= min_count,
                        "Query test '{}': Expected at least {} results, got {}",
                        test_case.test_name,
                        min_count,
                        classes.len()
                    );
                }

                if let Some(max_count) = test_case.max_count {
                    assert!(
                        classes.len() <= max_count,
                        "Query test '{}': Expected at most {} results, got {}",
                        test_case.test_name,
                        max_count,
                        classes.len()
                    );
                }

                // Check expected classes if provided
                if !test_case.expected_classes.is_empty() {
                    for expected in &test_case.expected_classes {
                        let found = classes.iter().any(|class| {
                            class.subject_code == expected.subject_code
                                && class.course_number == expected.course_number
                                && expected.title.as_ref().map_or(true, |t| {
                                    class.title.contains(t)
                                })
                                && expected.professor_name.as_ref().map_or(true, |p| {
                                    class.professor_name.as_ref().map_or(false, |name| name.contains(p))
                                })
                                && expected.section_sequence.as_ref().map_or(true, |s| {
                                    class.section_sequence == *s
                                })
                        });

                        assert!(
                            found,
                            "Query test '{}': Expected class {}-{} not found in results",
                            test_case.test_name,
                            expected.subject_code,
                            expected.course_number
                        );
                    }
                }

                println!("Query test '{}' passed.\n", test_case.test_name);
            }
            _ => {
                if test_case.should_succeed {
                    panic!(
                        "Query test '{}' failed but was expected to succeed. Error: {:?}",
                        test_case.test_name, result
                    );
                } else {
                    println!("Query failed as expected: {:?}\n", result);
                }
            }
        }
    }
}

/// Run a test file
///
/// Parameters:
/// --- ---
/// filename -> The name of the test file to run
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn run_test_file(filename: &str) {
    let mut helper = QueryTestHelper::new();
    let content = utils::load_test_file("query_tests", filename);
    let test_cases: Vec<QueryTestCase> =
        serde_json::from_str(&content).expect("Failed to parse JSON test file");

    for test_case in test_cases {
        helper.run_test(&test_case);
    }
}

#[test]
fn test_basic_queries() {
    run_test_file("basic_queries.json");
}

#[test]
fn test_professor_queries() {
    run_test_file("professor_queries.json");
}

#[test]
fn test_subject_queries() {
    run_test_file("subject_queries.json");
}

#[test]
fn test_course_queries() {
    run_test_file("course_queries.json");
}

#[test]
fn test_string_conditions() {
    run_test_file("string_conditions.json");
}

#[test]
fn test_numeric_queries() {
    run_test_file("numeric_queries.json");
}

#[test]
fn test_time_queries() {
    run_test_file("time_queries.json");
}

#[test]
fn test_day_queries() {
    run_test_file("day_queries.json");
}

#[test]
fn test_logical_operators() {
    run_test_file("logical_operators.json");
}

#[test]
fn test_complex_queries() {
    run_test_file("complex_queries.json");
}

#[test]
fn test_all_conditions() {
    run_test_file("all_conditions.json");
}

#[test]
fn test_method_campus_type_queries() {
    run_test_file("method_campus_type_queries.json");
}

#[test]
fn test_day_conditions() {
    run_test_file("day_conditions.json");
}

#[test]
fn test_email_queries() {
    run_test_file("email_queries.json");
}


