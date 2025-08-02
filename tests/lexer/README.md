# ClassQL Lexer Test Suite

This directory contains comprehensive test cases for the ClassQL lexer, organized by category.

## Test Files

- **`basic_keywords.json`** - Tests for basic ClassQL keywords (term, prof, course, subject, etc.)
- **`operators.json`** - Tests for comparison operators, condition operators, and logical operators
- **`literals.json`** - Tests for string literals, identifiers, numbers, and time formats
- **`days.json`** - Tests for all day abbreviations with progressive combinations
- **`complex_queries.json`** - Tests for real-world ClassQL query patterns
- **`whitespace.json`** - Tests for proper whitespace handling (spaces, tabs, newlines)
- **`edge_cases.json`** - Tests for edge cases and boundary conditions
- **`stress_tests.json`** - Tests for challenging inputs (no whitespace, smashed tokens)
- **`malformed_tests.json`** - Tests for invalid/malformed inputs and error recovery
- **`boundary_tests.json`** - Tests for extreme cases and boundary conditions
- **`unrecognized_tests.json`** - Tests for unrecognized character error handling

## Test Format

Each test file follows this JSON structure:

```json
[
    {
        "test_name": "Descriptive Test Name",
        "description": "What this test is checking",
        "code": "ClassQL code to tokenize",
        "result": [
            {"type": "TOKEN_TYPE", "content": "token_content"}
        ]
    }
]
```

## Running Tests

To run the lexer tests:

```bash
cargo run -- --debug lexer
```

## Token Types

The test suite expects the following token types based on the ClassQL grammar:

### Keywords
- `T_TERM`, `T_PROF`, `T_COURSE`, `T_SUBJECT`
- `T_CONTAINS`, `T_TITLE`, `T_METHOD`, `T_CAMPUS`
- `T_CREDIT`, `T_HOURS`, `T_PREREQS`, `T_COREREQS`
- Day keywords: `T_MONDAY`, `T_TUESDAY`, `T_WEDNESDAY`, `T_THURSDAY`, `T_FRIDAY`, `T_SATURDAY`, `T_SUNDAY`

### Operators
- Comparison: `T_EQUALS`, `T_NOT_EQUALS`, `T_LESS_THAN`, `T_GREATER_THAN`, etc.
- Logical: `T_AND`, `T_OR`, `T_NOT`
- Condition: `T_HAS`, `T_IS`, `T_CONTAINS`, `T_STARTS`, `T_WITH`, etc.

### Literals
- `T_STRING` - Quoted string literals
- `T_INTEGER` - Integer numbers
- `T_TIME` - Time format patterns
- `T_IDENTIFIER` - Unquoted alphanumeric identifiers

## Test Coverage

The test suite covers:

- All ClassQL keywords from the grammar specification  
-  All operator types (comparison, logical, condition)  
-  String and numeric literals  
-  Time format patterns  
-  Progressive day abbreviations (m, mo, mon, mond, monda, monday)  
-  Whitespace handling  
-  Complex real-world query patterns  
-  Edge cases and boundary conditions  
-  **No-whitespace stress tests**  
-  **Invalid character handling**  
-  **Malformed input recovery**  
-  **Extreme boundary conditions**  
-  **Unrecognized character error reporting**  
-  Case sensitivity  
-  Adjacent operators  
-  Empty inputs