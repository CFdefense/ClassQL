use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;
use classql::compiler::lexer::Lexer;

#[derive(Debug, Deserialize, Serialize)]
struct TestCase {
    test_name: String,
    description: String,
    code: String,
    result: Vec<ExpectedToken>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExpectedToken {
    #[serde(rename = "type")]
    token_type: String,
    content: String,
}

struct TestHelper {
    lexer: Lexer,
}

impl TestHelper {
    fn new() -> Self {
        Self {
            lexer: Lexer::new(),
        }
    }

    fn parse_json_tests(&self, json_content: &str) -> Vec<TestCase> {
        serde_json::from_str(json_content).expect("Failed to parse JSON test file")
    }

    fn run_test(&mut self, test_case: &TestCase) {
        println!("Running test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        println!("Input: '{}'", test_case.code);
        
        // Clear lexer state before each test
        self.lexer.clear();
        
        // Tokenize the input
        let tokens = match self.lexer.lexical_analysis(test_case.code.clone()) {
            Ok(tokens) => tokens,
            Err(e) => panic!("Lexer error in test '{}': {:?}", test_case.test_name, e),
        };
        
        // Print actual tokens
        println!("Expected {} tokens, got {} tokens", test_case.result.len(), tokens.len());
        
        if tokens.len() != test_case.result.len() {
            println!("\n=== TOKEN COUNT MISMATCH ===");
            println!("Expected tokens:");
            for (i, expected) in test_case.result.iter().enumerate() {
                println!("  [{}] {} = '{}'", i, expected.token_type, expected.content);
            }
            println!("Actual tokens:");
            for (i, actual) in tokens.iter().enumerate() {
                println!("  [{}] {} = '{}'", i, actual.get_token_type().to_string(), actual.get_lexeme());
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
        for (i, (actual, expected)) in tokens.iter().zip(test_case.result.iter()).enumerate() {
            let type_match = actual.get_token_type().to_string() == expected.token_type;
            let content_match = actual.get_lexeme() == expected.content;
            
            if !type_match || !content_match {
                if !has_diff {
                    println!("\n=== TOKEN DIFFERENCES ===");
                    has_diff = true;
                }
                println!("Position [{}]:", i);
                if !type_match {
                    println!("  Type:    Expected '{}' but got '{}'", expected.token_type, actual.get_token_type().to_string());
                }
                if !content_match {
                    println!("  Content: Expected '{}' but got '{}'", expected.content, actual.get_lexeme());
                }
            }
            
            assert_eq!(
                actual.get_token_type().to_string(),
                expected.token_type,
                "Token type mismatch at position {} in test '{}'. Expected: {}, Got: {}",
                i,
                test_case.test_name,
                expected.token_type,
                actual.get_token_type().to_string()
            );
            
            assert_eq!(
                actual.get_lexeme(),
                expected.content,
                "Token content mismatch at position {} in test '{}'. Expected: {}, Got: {}",
                i,
                test_case.test_name,
                expected.content,
                actual.get_lexeme()
            );
        }
        
        if has_diff {
            println!("========================\n");
        } else {
            println!("All tokens match!\n");
        }
    }
}

fn load_test_file(filename: &str) -> String {
    let path = format!("tests/lexer/{}", filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path))
}

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