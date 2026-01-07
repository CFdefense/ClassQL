/// tests/utils/mod.rs
///
/// Shared utilities for test modules
///
/// Contains:
/// --- ---
/// load_test_file -> Generic function to load test JSON files
/// run_test_file -> Generic function to run test files with a custom processor
/// --- ---
///
use std::fs;

/// Load a test file from a module's tests directory
///
/// Parameters:
/// --- ---
/// module_name -> The name of the module (e.g., "lexer", "parser", "codegen", "semantic")
/// filename -> The name of the test file to load
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The contents of the test file
/// --- ---
///
pub fn load_test_file(module_name: &str, filename: &str) -> String {
    let path = format!("tests/{}/tests/{}", module_name, filename);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read test file: {}", path))
}
