/// src/dsl/semantic.rs
///
/// Semantic analyzer for the DSL
///
/// Responsible for analyzing the AST and ensuring it is semantically correct.
///
/// Contains:
/// --- ---
/// semantic_analysis -> Run semantic analysis on a parsed AST
/// invalid_context -> Helper to build a semantic error wrapped as an `AppError`
/// analyze_node -> Analyze a node in the AST
/// --- ---
///
use crate::dsl::parser::{Ast, NodeType, TreeNode};
use crate::dsl::token::TokenType;
use crate::tui::errors::{AppError, SemanticError};

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
/// Result<(), AppError> -> The result of the semantic analysis
/// --- ---
pub fn semantic_analysis(ast: &Ast) -> Result<(), AppError> {
    if let Some(root) = &ast.head {
        analyze_node(root)
    } else {
        // An empty AST is treated as a no‑op for semantics. The parser already
        // emits a SyntaxError::EmptyQuery for empty input.
        Ok(())
    }
}

/// Helper to build a semantic error wrapped as an `AppError`.
///
/// Parameters:
/// --- ---
/// token -> The token or node description associated with the error
/// context -> A short description of where the token is invalid
/// suggestions -> Suggested replacements or valid alternatives
/// --- ---
///
/// Returns:
/// --- ---
/// AppError -> The wrapped semantic error
/// --- ---
fn invalid_context(token: String, context: &str, suggestions: &[&str]) -> AppError {
    AppError::SemanticError(SemanticError::InvalidContext {
        token,
        context: context.to_string(),
        suggestions: suggestions.iter().map(|s| (*s).to_string()).collect(),
    })
}

/// Analyze a node in the AST.
///
/// Responsible for applying node‑local semantic checks and recursively
/// analyzing all descendant nodes.
///
/// Parameters:
/// --- ---
/// node -> The node to analyze
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), AppError> -> The result of the node analysis
/// --- ---
fn analyze_node(node: &TreeNode) -> Result<(), AppError> {
    use NodeType::*;
    // Node‑local checks
    match node.node_type {
        // Queries over numeric fields should have shape: <Binop> <Integer>
        CreditHoursQuery | EnrollmentQuery | EnrollmentCapQuery => {
            if node.children.len() != 2 {
                return Err(invalid_context(
                    node.node_content.clone(),
                    "numeric field query",
                    &["<comparison>", "<number>"],
                ));
            }

            if !matches!(node.children[0].node_type, Binop) {
                return Err(invalid_context(
                    node.children[0].node_content.clone(),
                    "numeric comparison",
                    &["<comparison operator>"],
                ));
            }

            if !matches!(node.children[1].node_type, Integer) {
                return Err(invalid_context(
                    node.children[1].node_content.clone(),
                    "numeric comparison",
                    &["<number>"],
                ));
            }
        }

        // Time queries:
        //   ("start" | "end") <binop> <time>
        //   ("start" | "end") <time_range>
        TimeQuery => {
            if node.children.is_empty() {
                return Err(invalid_context(
                    node.node_content.clone(),
                    "time query",
                    &["start", "end"],
                ));
            }

            // First child encodes whether this is "start" or "end"
            if !matches!(node.children[0].node_type, String) {
                return Err(invalid_context(
                    node.children[0].node_content.clone(),
                    "time query",
                    &["start", "end"],
                ));
            }

            match node.children.len() {
                // ("start" | "end") <time_range>
                2 => {
                    if !matches!(node.children[1].node_type, TimeRange) {
                        return Err(invalid_context(
                            node.children[1].node_content.clone(),
                            "time range query",
                            &["<time> to <time>"],
                        ));
                    }
                }
                // ("start" | "end") <binop> <time>
                3 => {
                    if !matches!(node.children[1].node_type, Binop) {
                        return Err(invalid_context(
                            node.children[1].node_content.clone(),
                            "time comparison",
                            &["<comparison operator>"],
                        ));
                    }
                    if !matches!(node.children[2].node_type, Time) {
                        return Err(invalid_context(
                            node.children[2].node_content.clone(),
                            "time comparison",
                            &["<time value>"],
                        ));
                    }
                }
                _ => {
                    return Err(invalid_context(
                        node.node_content.clone(),
                        "time query",
                        &["<comparison> <time>", "<time> to <time>"],
                    ));
                }
            }
        }

        // Time range must be exactly: <time>, <time>
        TimeRange => {
            if node.children.len() != 2 {
                return Err(invalid_context(
                    node.node_content.clone(),
                    "time range",
                    &["<time> to <time>"],
                ));
            }

            if !matches!(node.children[0].node_type, Time)
                || !matches!(node.children[1].node_type, Time)
            {
                return Err(invalid_context(
                    node.node_content.clone(),
                    "time range",
                    &["<time> to <time>"],
                ));
            }
        }

        // Day queries wrap a specific day node; the parser currently encodes
        // each day as a `String` node with children describing the condition
        // and value. Here we just assert there is exactly one child so the
        // downstream SQL builder can rely on that shape.
        DayQuery => {
            // The parser builds day queries as:
            //   DayQuery -> [ day_node ]
            // where day_node is a `String` node whose children are
            //   [ <Condition>, <Identifier-or-email> ]
            if node.children.len() != 1 {
                return Err(invalid_context(
                    node.node_content.clone(),
                    "day query",
                    &["monday <condition> <value>", "tuesday <condition> <value>"],
                ));
            }

            let day_node = &node.children[0];
            // We don’t depend on the concrete day token here, only on shape.
            if day_node.children.len() != 2 {
                return Err(invalid_context(
                    day_node.node_content.clone(),
                    "day query",
                    &["<day> <condition> <value>"],
                ));
            }

            // First child must be a string condition node.
            if !matches!(day_node.children[0].node_type, Condition) {
                return Err(invalid_context(
                    day_node.children[0].node_content.clone(),
                    "day condition",
                    &["is", "equals", "contains", "has", "starts with", "ends with"],
                ));
            }

            // Second child must be a string-like value; we allow identifiers and
            // generic string nodes so future lexer refinements keep working.
            if !matches!(
                day_node.children[1].node_type,
                Identifier | EmailIdentifier | String
            ) {
                return Err(invalid_context(
                    day_node.children[1].node_content.clone(),
                    "day value",
                    &["true", "false", "<text value>"],
                ));
            }

            // Additionally, reject obviously wrong literal categories such as
            // bare numbers or times in place of a boolean/textual day flag.
            if let Some(tok) = day_node.children[1].lexical_token {
                if matches!(
                    *tok.get_token_type(),
                    TokenType::Integer | TokenType::Time
                ) {
                    return Err(invalid_context(
                        tok.get_token_type().to_string(),
                        "day value",
                        &["true", "false"],
                    ));
                }
            }
        }

        // String-based field queries (professor, subject, number, title,
        // description, method, campus, full, meeting type) should all have a
        // consistent shape:
        //   <EntityQuery> -> [ <Condition>, <Identifier-or-email> ]
        ProfessorQuery
        | SubjectQuery
        | NumberQuery
        | TitleQuery
        | DescriptionQuery
        | InstructionMethodQuery
        | CampusQuery
        | FullQuery
        | MeetingTypeQuery => {
            if node.children.len() != 2 {
                return Err(invalid_context(
                    node.node_content.clone(),
                    "string field query",
                    &["<condition> <value>"],
                ));
            }

            // First child must be a condition node (parser already enforces the
            // token category; semantics assert the AST shape).
            if !matches!(node.children[0].node_type, Condition) {
                return Err(invalid_context(
                    node.children[0].node_content.clone(),
                    "string condition",
                    &["is", "equals", "contains", "has", "starts with", "ends with"],
                ));
            }

            // Second child must be a string-like value node; we allow both
            // identifiers and email identifiers (once the parser distinguishes
            // them) as well as generic string nodes.
            if !matches!(
                node.children[1].node_type,
                Identifier | EmailIdentifier | String
            ) {
                return Err(invalid_context(
                    node.children[1].node_content.clone(),
                    "string field value",
                    &["<text value>", "quoted string"],
                ));
            }

            // Also reject clearly wrong literal categories such as numeric or
            // time values in string fields (e.g. "prof equals 3").
            if let Some(tok) = node.children[1].lexical_token {
                if matches!(
                    *tok.get_token_type(),
                    TokenType::Integer | TokenType::Time
                ) {
                    return Err(invalid_context(
                        tok.get_token_type().to_string(),
                        "string field value",
                        &["<text value>", "quoted string"],
                    ));
                }
            }
        }

        // Leaf typing checks – ensure lexer categories line up with node types.
        Integer => {
            if let Some(tok) = node.lexical_token {
                if *tok.get_token_type() != TokenType::Integer {
                    return Err(invalid_context(
                        tok.get_token_type().to_string(),
                        "integer literal",
                        &["<number>"],
                    ));
                }
            }
        }

        Time => {
            if let Some(tok) = node.lexical_token {
                if *tok.get_token_type() != TokenType::Time {
                    return Err(invalid_context(
                        tok.get_token_type().to_string(),
                        "time literal",
                        &["<time value>"],
                    ));
                }
            }
        }

        // Other node types currently have no extra semantic rules beyond what
        // the parser already guarantees; they are still traversed recursively
        // below.
        _ => {}
    }

    // Recursively analyze children
    for child in &node.children {
        analyze_node(child)?;
    }

    Ok(())
}