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

    fn clear(&mut self) {
        self.lexer.clear();
    }

    fn parse_json_tests(&self, json_content: &str) -> Vec<TestCase> {
        serde_json::from_str(json_content).expect("Failed to parse JSON test file")
    }

    fn run_test(&mut self, test_case: &TestCase) {
        println!("Running test: {}", test_case.test_name);
        println!("Description: {}", test_case.description);
        
        // Clear lexer state before each test
        self.clear();
        
        // Tokenize the input
        let tokens = self.lexer.begin_lexing(test_case.code.clone());
        
        // Compare results
        assert_eq!(
            tokens.len(),
            test_case.result.len(),
            "Token count mismatch in test '{}'. Expected: {}, Got: {}",
            test_case.test_name,
            test_case.result.len(),
            tokens.len()
        );

        for (i, (actual, expected)) in tokens.iter().zip(test_case.result.iter()).enumerate() {
            assert_eq!(
                actual.token_type_as_string(),
                expected.token_type,
                "Token type mismatch at position {} in test '{}'. Expected: {}, Got: {}",
                i,
                test_case.test_name,
                expected.token_type,
                actual.token_type_as_string()
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