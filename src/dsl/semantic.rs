/// src/dsl/semantic.rs
///
/// Semantic analyzer for the DSL
///
/// Responsible for analyzing the AST and ensuring it is semantically correct.
///
/// Contains:
/// --- ---
/// semantic_analysis -> Run semantic analysis on a parsed AST
/// invalid_context -> Helper to build a `SemanticError`
/// analyze_node -> Analyze a node in the AST
/// --- ---
///
use crate::dsl::parser::{Ast, NodeType, TreeNode};
use crate::dsl::token::TokenType;
use crate::tui::errors::SemanticError;

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
pub fn semantic_analysis(ast: &Ast) -> Result<(), (SemanticError, Vec<(usize, usize)>)> {
    if let Some(root) = &ast.head {
        analyze_node(root)
    } else {
        // An empty AST is treated as a no‑op for semantics. The parser already
        // emits a SyntaxError::EmptyQuery for empty input.
        Ok(())
    }
}

/// Helper to build a semantic error.
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
/// SemanticError -> The semantic error
/// --- ---
fn invalid_context(token: String, context: &str, suggestions: &[&str]) -> SemanticError {
    SemanticError::InvalidContext {
        token,
        context: context.to_string(),
        suggestions: suggestions.iter().map(|s| (*s).to_string()).collect(),
    }
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
/// Result<(), (SemanticError, Vec<(usize, usize)>)> -> The result of the node analysis
/// --- ---
fn analyze_node(node: &TreeNode) -> Result<(), (SemanticError, Vec<(usize, usize)>)> {
    use NodeType::*;
    // Node‑local checks
    match node.node_type {
        // Queries over numeric fields should have shape: <Binop> <Integer>
        CreditHoursQuery | EnrollmentQuery | EnrollmentCapQuery => {
            if node.children.len() != 2 {
                let err = invalid_context(
                    node.node_content.clone(),
                    "numeric field query",
                    &["<comparison>", "<number>"],
                );
                let span = node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            if !matches!(node.children[0].node_type, Binop) {
                let child = &node.children[0];
                let err = invalid_context(
                    child.node_content.clone(),
                    "numeric comparison",
                    &["<comparison operator>"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            if !matches!(node.children[1].node_type, Integer) {
                let child = &node.children[1];
                let err = invalid_context(
                    child.node_content.clone(),
                    "numeric comparison",
                    &["<number>"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }
        }

        // Time queries:
        //   ("start" | "end") <binop> <time>
        //   ("start" | "end") <time_range>
        TimeQuery => {
            if node.children.is_empty() {
                let err = invalid_context(
                    node.node_content.clone(),
                    "time query",
                    &["start", "end"],
                );
                let span = node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // First child encodes whether this is "start" or "end"
            if !matches!(node.children[0].node_type, String) {
                let child = &node.children[0];
                let err = invalid_context(
                    child.node_content.clone(),
                    "time query",
                    &["start", "end"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            match node.children.len() {
                // ("start" | "end") <time_range>
                2 => {
                    if !matches!(node.children[1].node_type, TimeRange) {
                        let child = &node.children[1];
                        let err = invalid_context(
                            child.node_content.clone(),
                            "time range query",
                            &["<time> to <time>"],
                        );
                        let span = child
                            .lexical_token
                            .map(|t| vec![(t.get_start(), t.get_end())])
                            .unwrap_or_default();
                        return Err((err, span));
                    }
                }
                // ("start" | "end") <binop> <time>
                3 => {
                    if !matches!(node.children[1].node_type, Binop) {
                        let child = &node.children[1];
                        let err = invalid_context(
                            child.node_content.clone(),
                            "time comparison",
                            &["<comparison operator>"],
                        );
                        let span = child
                            .lexical_token
                            .map(|t| vec![(t.get_start(), t.get_end())])
                            .unwrap_or_default();
                        return Err((err, span));
                    }
                    if !matches!(node.children[2].node_type, Time) {
                        let child = &node.children[2];
                        let err = invalid_context(
                            child.node_content.clone(),
                            "time comparison",
                            &["<time value>"],
                        );
                        let span = child
                            .lexical_token
                            .map(|t| vec![(t.get_start(), t.get_end())])
                            .unwrap_or_default();
                        return Err((err, span));
                    }
                }
                _ => {
                    let err = invalid_context(
                        node.node_content.clone(),
                        "time query",
                        &["<comparison> <time>", "<time> to <time>"],
                    );
                    let span = node
                        .lexical_token
                        .map(|t| vec![(t.get_start(), t.get_end())])
                        .unwrap_or_default();
                    return Err((err, span));
                }
            }
        }

        // Time range must be exactly: <time>, <time>
        TimeRange => {
            if node.children.len() != 2 {
                let err = invalid_context(
                    node.node_content.clone(),
                    "time range",
                    &["<time> to <time>"],
                );
                let span = node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            if !matches!(node.children[0].node_type, Time)
                || !matches!(node.children[1].node_type, Time)
            {
                let err = invalid_context(
                    node.node_content.clone(),
                    "time range",
                    &["<time> to <time>"],
                );
                let span = node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
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
                let err = invalid_context(
                    node.node_content.clone(),
                    "day query",
                    &["monday <condition> <value>", "tuesday <condition> <value>"],
                );
                let span = node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            let day_node = &node.children[0];
            // We don’t depend on the concrete day token here, only on shape.
            if day_node.children.len() != 2 {
                let err = invalid_context(
                    day_node.node_content.clone(),
                    "day query",
                    &["<day> <condition> <value>"],
                );
                let span = day_node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // First child must be a string condition node.
            if !matches!(day_node.children[0].node_type, Condition) {
                let child = &day_node.children[0];
                let err = invalid_context(
                    child.node_content.clone(),
                    "day condition",
                    &["is", "equals", "contains", "has", "starts with", "ends with"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // Second child must be a string-like value; we allow identifiers and
            // generic string nodes so future lexer refinements keep working.
            if !matches!(
                day_node.children[1].node_type,
                Identifier | EmailIdentifier | String
            ) {
                let child = &day_node.children[1];
                let err = invalid_context(
                    child.node_content.clone(),
                    "day value",
                    &["true", "false", "<text value>"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // Additionally, reject obviously wrong literal categories such as
            // bare numbers or times in place of a boolean/textual day flag.
            if let Some(tok) = day_node.children[1].lexical_token {
                if matches!(
                    *tok.get_token_type(),
                    TokenType::Integer | TokenType::Time
                ) {
                    let err = invalid_context(
                        tok.get_token_type().to_string(),
                        "day value",
                        &["true", "false"],
                    );
                    let span = vec![(tok.get_start(), tok.get_end())];
                    return Err((err, span));
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
                let err = invalid_context(
                    node.node_content.clone(),
                    "string field query",
                    &["<condition> <value>"],
                );
                let span = node
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // First child must be a condition node (parser already enforces the
            // token category; semantics assert the AST shape).
            if !matches!(node.children[0].node_type, Condition) {
                let child = &node.children[0];
                let err = invalid_context(
                    child.node_content.clone(),
                    "string condition",
                    &["is", "equals", "contains", "has", "starts with", "ends with"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // Second child must be a string-like value node; we allow both
            // identifiers and email identifiers (once the parser distinguishes
            // them) as well as generic string nodes.
            if !matches!(
                node.children[1].node_type,
                Identifier | EmailIdentifier | String
            ) {
                let child = &node.children[1];
                let err = invalid_context(
                    child.node_content.clone(),
                    "string field value",
                    &["<text value>", "quoted string"],
                );
                let span = child
                    .lexical_token
                    .map(|t| vec![(t.get_start(), t.get_end())])
                    .unwrap_or_default();
                return Err((err, span));
            }

            // Also reject clearly wrong literal categories such as numeric or
            // time values in string fields (e.g. "prof equals 3").
            if let Some(tok) = node.children[1].lexical_token {
                if matches!(
                    *tok.get_token_type(),
                    TokenType::Integer | TokenType::Time
                ) {
                    let err = invalid_context(
                        tok.get_token_type().to_string(),
                        "string field value",
                        &["<text value>", "quoted string"],
                    );
                    let span = vec![(tok.get_start(), tok.get_end())];
                    return Err((err, span));
                }
            }
        }

        // Leaf typing checks – ensure lexer categories line up with node types.
        Integer => {
            if let Some(tok) = node.lexical_token {
                if *tok.get_token_type() != TokenType::Integer {
                    let err = invalid_context(
                        tok.get_token_type().to_string(),
                        "integer literal",
                        &["<number>"],
                    );
                    let span = vec![(tok.get_start(), tok.get_end())];
                    return Err((err, span));
                }
            }
        }

        Time => {
            if let Some(tok) = node.lexical_token {
                if *tok.get_token_type() != TokenType::Time {
                    let err = invalid_context(
                        tok.get_token_type().to_string(),
                        "time literal",
                        &["<time value>"],
                    );
                    let span = vec![(tok.get_start(), tok.get_end())];
                    return Err((err, span));
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