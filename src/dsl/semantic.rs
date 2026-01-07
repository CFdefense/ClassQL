/// src/dsl/semantic.rs
///
/// Semantic analyzer for the DSL
///
/// Responsible for analyzing the AST and ensuring it is semantically correct.
///
/// Contains:
/// --- ---
/// SemanticResult -> Result type for semantic analysis
/// semantic_analysis -> Run semantic analysis on a parsed AST
/// invalid_context -> Helper to build a `SemanticError`
/// analyze_node -> Analyze a node in the AST (dispatches to specialized analyzers)
/// analyze_numeric_query -> Validate numeric field queries
/// analyze_time_query -> Validate time queries
/// analyze_time_range -> Validate time range nodes
/// analyze_day_query -> Validate day queries
/// analyze_string_field_query -> Validate string-based field queries
/// analyze_integer -> Validate integer literals
/// analyze_time -> Validate time literals
/// --- ---
///
use crate::dsl::parser::{Ast, NodeType, TreeNode};
use crate::dsl::token::TokenType;
use crate::tui::errors::SemanticError;

/// Type alias for semantic analysis results
type SemanticResult = Result<(), (SemanticError, Vec<(usize, usize)>)>;

/// Run semantic analysis on a parsed AST.
///
/// Responsible for walking the AST and validating basic typing and structural
/// invariants.
///
/// Checks:
/// --- ---
/// - Numeric queries (credit hours, enrollment, caps) use numeric operators and integer values
/// - Time queries either use a numeric comparison against a time value or a well‑formed time range
/// - Day queries and other leaf nodes are structurally consistent
/// --- ---
///
/// Notes:
/// --- ---
/// No symbol table or scoping is required because the DSL has no user‑defined
/// bindings – we only validate node shapes and token categories.
/// --- ---
///
/// Parameters:
/// --- ---
/// ast -> The AST to analyze
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), (SemanticError, Vec<(usize, usize)>)> -> The result of the semantic analysis
///     Ok(()) -> Semantic analysis succeeded
///     Err((SemanticError, Vec<(usize, usize)>)) -> Semantic analysis failed, contains
///         the `SemanticError` and byte‑range positions for the problematic input
/// --- ---
pub fn semantic_analysis(ast: &Ast) -> SemanticResult {
    if let Some(root) = &ast.head {
        analyze_node(root)
    } else {
        // An empty AST is treated as a no‑op for semantics. The parser already
        // emits a SyntaxError::EmptyQuery for empty input.
        Ok(())
    }
}

/// Helper to build a semantic error.
fn invalid_context(token: String, context: &str, suggestions: &[&str]) -> SemanticError {
    SemanticError::InvalidContext {
        token,
        context: context.to_string(),
        suggestions: suggestions.iter().map(|s| (*s).to_string()).collect(),
    }
}

/// Extract span from a node's lexical token
fn get_span(node: &TreeNode) -> Vec<(usize, usize)> {
    node.lexical_token
        .map(|t| vec![(t.get_start(), t.get_end())])
        .unwrap_or_default()
}

/// Analyze a node in the AST.
///
/// Dispatches to specialized analyzers based on node type, then recursively
/// analyzes all children.
fn analyze_node(node: &TreeNode) -> SemanticResult {
    use NodeType::*;

    match node.node_type {
        CreditHoursQuery | EnrollmentQuery | EnrollmentCapQuery => {
            analyze_numeric_query(node)?;
        }

        TimeQuery => {
            analyze_time_query(node)?;
        }

        TimeRange => {
            analyze_time_range(node)?;
        }

        DayQuery => {
            analyze_day_query(node)?;
        }

        ProfessorQuery
        | SubjectQuery
        | NumberQuery
        | TitleQuery
        | DescriptionQuery
        | PrereqsQuery
        | CoreqsQuery
        | InstructionMethodQuery
        | CampusQuery
        | FullQuery
        | MeetingTypeQuery => {
            analyze_string_field_query(node)?;
        }

        Integer => {
            analyze_integer(node)?;
        }

        Time => {
            analyze_time(node)?;
        }

        // Other node types have no extra semantic rules beyond what
        // the parser already guarantees
        _ => {}
    }

    // Recursively analyze children
    for child in &node.children {
        analyze_node(child)?;
    }

    Ok(())
}

/// Validate numeric field queries (credit hours, enrollment, caps).
///
/// Expected shape: <Binop> <Integer>
fn analyze_numeric_query(node: &TreeNode) -> SemanticResult {
    if node.children.len() != 2 {
        let err = invalid_context(
            node.node_content.clone(),
            "numeric field query",
            &["<comparison>", "<number>"],
        );
        return Err((err, get_span(node)));
    }

    if !matches!(node.children[0].node_type, NodeType::Binop) {
        let child = &node.children[0];
        let err = invalid_context(
            child.node_content.clone(),
            "numeric comparison",
            &["<comparison operator>"],
        );
        return Err((err, get_span(child)));
    }

    if !matches!(node.children[1].node_type, NodeType::Integer) {
        let child = &node.children[1];
        let err = invalid_context(
            child.node_content.clone(),
            "numeric comparison",
            &["<number>"],
        );
        return Err((err, get_span(child)));
    }

    Ok(())
}

/// Validate time queries.
///
/// Expected shapes:
/// - ("start" | "end") <binop> <time>
/// - ("start" | "end") <time_range>
fn analyze_time_query(node: &TreeNode) -> SemanticResult {
    if node.children.is_empty() {
        let err = invalid_context(node.node_content.clone(), "time query", &["start", "end"]);
        return Err((err, get_span(node)));
    }

    // First child encodes whether this is "start" or "end"
    if !matches!(node.children[0].node_type, NodeType::String) {
        let child = &node.children[0];
        let err = invalid_context(child.node_content.clone(), "time query", &["start", "end"]);
        return Err((err, get_span(child)));
    }

    match node.children.len() {
        // ("start" | "end") <time_range>
        2 => {
            if !matches!(node.children[1].node_type, NodeType::TimeRange) {
                let child = &node.children[1];
                let err = invalid_context(
                    child.node_content.clone(),
                    "time range query",
                    &["<time> to <time>"],
                );
                return Err((err, get_span(child)));
            }
        }
        // ("start" | "end") <binop> <time>
        3 => {
            if !matches!(node.children[1].node_type, NodeType::Binop) {
                let child = &node.children[1];
                let err = invalid_context(
                    child.node_content.clone(),
                    "time comparison",
                    &["<comparison operator>"],
                );
                return Err((err, get_span(child)));
            }
            if !matches!(node.children[2].node_type, NodeType::Time) {
                let child = &node.children[2];
                let err = invalid_context(
                    child.node_content.clone(),
                    "time comparison",
                    &["<time value>"],
                );
                return Err((err, get_span(child)));
            }
        }
        _ => {
            let err = invalid_context(
                node.node_content.clone(),
                "time query",
                &["<comparison> <time>", "<time> to <time>"],
            );
            return Err((err, get_span(node)));
        }
    }

    Ok(())
}

/// Validate time range nodes.
///
/// Expected shape: <time> to <time>
fn analyze_time_range(node: &TreeNode) -> SemanticResult {
    if node.children.len() != 2 {
        let err = invalid_context(
            node.node_content.clone(),
            "time range",
            &["<time> to <time>"],
        );
        return Err((err, get_span(node)));
    }

    if !matches!(node.children[0].node_type, NodeType::Time)
        || !matches!(node.children[1].node_type, NodeType::Time)
    {
        let err = invalid_context(
            node.node_content.clone(),
            "time range",
            &["<time> to <time>"],
        );
        return Err((err, get_span(node)));
    }

    Ok(())
}

/// Validate day queries.
///
/// Expected shape: DayQuery -> [ day_node ]
/// where day_node has children [ <Condition>, <value> ]
fn analyze_day_query(node: &TreeNode) -> SemanticResult {
    if node.children.len() != 1 {
        let err = invalid_context(
            node.node_content.clone(),
            "day query",
            &["monday <condition> <value>", "tuesday <condition> <value>"],
        );
        return Err((err, get_span(node)));
    }

    let day_node = &node.children[0];

    // Validate day node has exactly 2 children
    if day_node.children.len() != 2 {
        let err = invalid_context(
            day_node.node_content.clone(),
            "day query",
            &["<day> <condition> <value>"],
        );
        return Err((err, get_span(day_node)));
    }

    // First child must be a condition node
    if !matches!(day_node.children[0].node_type, NodeType::Condition) {
        let child = &day_node.children[0];
        let err = invalid_context(
            child.node_content.clone(),
            "day condition",
            &[
                "is",
                "is not",
                "equals",
                "does not equal",
                "contains",
                "does not contain",
                "has",
                "starts with",
                "ends with",
            ],
        );
        return Err((err, get_span(child)));
    }

    // Second child must be a string-like value
    let value_node = &day_node.children[1];
    if !matches!(
        value_node.node_type,
        NodeType::Identifier | NodeType::EmailIdentifier | NodeType::String
    ) {
        let err = invalid_context(
            value_node.node_content.clone(),
            "day value",
            &["true", "false", "<text value>"],
        );
        return Err((err, get_span(value_node)));
    }

    // Reject numeric or time values
    if let Some(tok) = value_node.lexical_token {
        if matches!(*tok.get_token_type(), TokenType::Integer | TokenType::Time) {
            let err = invalid_context(
                tok.get_token_type().to_string(),
                "day value",
                &["true", "false"],
            );
            return Err((err, vec![(tok.get_start(), tok.get_end())]));
        }
    }

    // Day predicates should be boolean: "true" or "false"
    let value_text = value_node.node_content.to_lowercase();
    if value_text != "true" && value_text != "false" {
        let err = invalid_context(
            value_node.node_content.clone(),
            "day value",
            &["true", "false"],
        );
        return Err((err, get_span(value_node)));
    }

    Ok(())
}

/// Validate string-based field queries.
///
/// Expected shape: [ <Condition>, <Identifier-or-email> ]
fn analyze_string_field_query(node: &TreeNode) -> SemanticResult {
    if node.children.len() != 2 {
        let err = invalid_context(
            node.node_content.clone(),
            "string field query",
            &["<condition> <value>"],
        );
        return Err((err, get_span(node)));
    }

    // First child must be a condition node
    if !matches!(node.children[0].node_type, NodeType::Condition) {
        let child = &node.children[0];
        let err = invalid_context(
            child.node_content.clone(),
            "string condition",
            &[
                "is",
                "is not",
                "equals",
                "does not equal",
                "contains",
                "does not contain",
                "has",
                "starts with",
                "ends with",
            ],
        );
        return Err((err, get_span(child)));
    }

    // Second child must be a string-like value node
    let value_node = &node.children[1];
    if !matches!(
        value_node.node_type,
        NodeType::Identifier | NodeType::EmailIdentifier | NodeType::String
    ) {
        let err = invalid_context(
            value_node.node_content.clone(),
            "string field value",
            &["<text value>", "quoted string"],
        );
        return Err((err, get_span(value_node)));
    }

    // Reject numeric or time values in string fields
    if let Some(tok) = value_node.lexical_token {
        if matches!(*tok.get_token_type(), TokenType::Integer | TokenType::Time) {
            let err = invalid_context(
                tok.get_token_type().to_string(),
                "string field value",
                &["<text value>", "quoted string"],
            );
            return Err((err, vec![(tok.get_start(), tok.get_end())]));
        }
    }

    Ok(())
}

/// Validate integer literals.
fn analyze_integer(node: &TreeNode) -> SemanticResult {
    if let Some(tok) = node.lexical_token {
        if *tok.get_token_type() != TokenType::Integer {
            let err = invalid_context(
                tok.get_token_type().to_string(),
                "integer literal",
                &["<number>"],
            );
            return Err((err, vec![(tok.get_start(), tok.get_end())]));
        }
    }
    Ok(())
}

/// Validate time literals.
///
/// Ensures the time token is correct and includes am/pm suffix.
fn analyze_time(node: &TreeNode) -> SemanticResult {
    if let Some(tok) = node.lexical_token {
        if *tok.get_token_type() != TokenType::Time {
            let err = invalid_context(
                tok.get_token_type().to_string(),
                "time literal (must include am/pm)",
                &["6:00am", "6:00pm", "9:30am", "2:15pm"],
            );
            return Err((err, vec![(tok.get_start(), tok.get_end())]));
        }
    }

    // Validate that time includes am/pm suffix for clarity
    let time_str = node.node_content.to_lowercase();
    if !time_str.contains("am") && !time_str.contains("pm") {
        let err = invalid_context(
            node.node_content.clone(),
            "time literal (must include am/pm)",
            &["6:00am", "6:00pm", "9:30am", "2:15pm"],
        );
        return Err((err, get_span(node)));
    }

    Ok(())
}
