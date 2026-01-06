# ClassQL Test Suite

This directory contains comprehensive integration tests for the ClassQL DSL compiler, organized by component.

## Test Structure

Tests are organized into subdirectories, each corresponding to a component of the compiler:

```
tests/
├── lexer/          # Lexer (tokenization) tests
├── parser/         # Parser (AST construction) tests
├── semantic/       # Semantic analysis tests
├── codegen/        # Code generation (SQL) tests
└── utils/          # Shared test utilities
```

Each test module contains:
- `*_tests.rs` - Test implementation file
- `mod.rs` - Module declaration
- `tests/` - Directory containing JSON test case files

## Running Tests

Run all tests:
```bash
cargo test --test mod
```

Run tests for a specific component:
```bash
cargo test --test mod lexer
cargo test --test mod parser
cargo test --test mod semantic
cargo test --test mod codegen
```

## Test Suites

### Lexer Tests (`tests/lexer/`)

Tests the tokenization phase of the compiler. Verifies that input strings are correctly broken down into tokens.

**Test Files:**
- `basic_keywords.json` - Core keyword recognition (prof, course, subject, etc.)
- `operators.json` - Operator tokenization (=, !=, <, >, etc.)
- `literals.json` - String, integer, and time literal parsing
- `days.json` - Day name tokenization (monday, tuesday, etc.)
- `complex_queries.json` - Multi-token query parsing
- `whitespace.json` - Whitespace handling
- `edge_cases.json` - Boundary conditions and edge cases
- `stress_tests.json` - Large/complex input handling
- `malformed_tests.json` - Invalid input handling
- `boundary_tests.json` - Token boundary detection
- `unrecognized_tests.json` - Unknown token handling
- `parentheses_grouping_tests.json` - Parentheses tokenization
- `core_tokens.json` - Core token type validation
- `core_tokens_edge_cases.json` - Edge cases for core tokens
- `core_tokens_stress_tests.json` - Stress tests for core tokens
- `time_queries.json` - Time format parsing
- `time_edge_cases.json` - Time parsing edge cases

**What it tests:**
- Correct token type identification
- Token content extraction
- Token position tracking
- Error handling for invalid input

### Parser Tests (`tests/parser/`)

Tests the parsing phase that builds an Abstract Syntax Tree (AST) from tokens.

**Test Files:**
- `basic_valid_queries.json` - Simple valid query parsing
- `complex_valid_queries.json` - Complex query structures
- `invalid_syntax_queries.json` - Syntax error detection
- `malformed_operators.json` - Operator syntax errors
- `empty_and_whitespace.json` - Empty/whitespace-only input
- `nested_expressions.json` - Nested logical expressions
- `time_and_day_queries.json` - Time and day query parsing
- `time_queries.json` - Time range and comparison queries
- `enrollment_queries.json` - Enrollment-related queries
- `size_queries.json` - Class size queries
- `string_literals.json` - String literal handling
- `numeric_queries.json` - Numeric value queries
- `identifier_queries.json` - Identifier parsing
- `error_recovery.json` - Parser error recovery
- `token_position_tracking.json` - Position tracking in AST
- `ast_structure.json` - AST structure validation
- `edge_cases.json` - Edge cases and boundary conditions
- `advanced_logical_expressions.json` - Complex AND/OR expressions
- `all_keyword_variations.json` - All keyword synonym variations
- `comprehensive_grammar_tests.json` - Complete grammar coverage

**What it tests:**
- AST construction correctness
- Syntax error detection and reporting
- Problematic token position tracking
- Error type classification (Lexer vs Parser errors)
- Complex expression parsing

### Semantic Tests (`tests/semantic/`)

Tests the semantic analysis phase that validates query semantics.

**Test Files:**
- `basic_valid_queries.json` - Semantically valid queries
- `basic_invalid_queries.json` - Semantically invalid queries

**What it tests:**
- Semantic error detection
- Query validity checking
- Type and constraint validation

### Codegen Tests (`tests/codegen/`)

Tests the SQL code generation phase that converts AST to SQL queries.

**Test Files:**
- `basic_queries.json` - Basic query SQL generation
- `string_conditions.json` - String condition SQL (contains, equals, starts with, etc.)
- `numeric_queries.json` - Numeric comparison SQL (=, <, >, etc.)
- `time_queries.json` - Time-based query SQL generation
- `day_queries.json` - Day-based query SQL generation
- `logical_operators.json` - AND/OR operator SQL generation
- `complex_queries.json` - Complex multi-condition queries
- `keyword_variations.json` - SQL generation for keyword synonyms
- `edge_cases.json` - Edge cases in SQL generation

**What it tests:**
- Correct SQL query generation
- SQL fragment presence/absence validation
- Query structure correctness
- Aggregation and JOIN handling

## Test File Format

Test files are JSON arrays containing test case objects. Each test case typically includes:

- `test_name` - Unique identifier for the test
- `description` - Human-readable description
- `input` - The ClassQL query string to test
- `should_succeed` - Whether the test should pass or fail
- Additional fields specific to each test type (expected tokens, SQL fragments, etc.)

## Shared Utilities

The `utils/` module provides shared functionality:
- `load_test_file()` - Loads JSON test files from module test directories
- `run_test_file()` - Generic test file runner with custom processor

All test modules use these utilities to reduce code duplication.

