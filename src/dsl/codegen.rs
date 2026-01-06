/// src/dsl/codegen.rs
///
/// Code generator for the DSL
///
/// Responsible for converting the AST into SQL queries
///
/// Contains:
/// --- ---
/// generate_sql -> Generate SQL from an AST
/// CodeGenError -> Error type for code generation
/// generate_node -> Generate SQL for a single AST node
/// generate_query -> Generate SQL for a Query node
/// generate_logical_term -> Generate SQL for a LogicalTerm node
/// generate_logical_factor -> Generate SQL for a LogicalFactor node
/// generate_entity_query -> Generate SQL for an EntityQuery node
/// generate_and -> Generate SQL for AND operation
/// generate_or -> Generate SQL for OR operation
/// generate_professor_query -> Generate SQL for ProfessorQuery node
/// generate_course_query -> Generate SQL for CourseQuery node
/// generate_subject_query -> Generate SQL for SubjectQuery node
/// generate_section_query -> Generate SQL for SectionQuery node
/// generate_number_query -> Generate SQL for NumberQuery node
/// generate_title_query -> Generate SQL for TitleQuery node
/// generate_description_query -> Generate SQL for DescriptionQuery node
/// generate_credit_hours_query -> Generate SQL for CreditHoursQuery node
/// generate_prereqs_query -> Generate SQL for PrereqsQuery node
/// generate_coreqs_query -> Generate SQL for CoreqsQuery node
/// generate_enrollment_cap_query -> Generate SQL for EnrollmentCapQuery node
/// generate_instruction_method_query -> Generate SQL for InstructionMethodQuery node
/// generate_campus_query -> Generate SQL for CampusQuery node
/// generate_enrollment_query -> Generate SQL for EnrollmentQuery node
/// generate_full_query -> Generate SQL for FullQuery node
/// generate_meeting_type_query -> Generate SQL for MeetingTypeQuery node
/// generate_time_query -> Generate SQL for TimeQuery node
/// generate_day_query -> Generate SQL for DayQuery node
/// --- ---
///
use crate::dsl::parser::{Ast, NodeType, TreeNode};
use crate::dsl::token::TokenType;

/// Type alias for code generation results
type CodeGenResult = Result<String, CodeGenError>;

/// Code generation error types
///
/// Errors:
/// --- ---
/// EmptyAst -> The AST has no root node
/// UnsupportedNode -> A node type is not supported for code generation
/// InvalidStructure -> The AST structure is invalid for the expected node type
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug
/// Clone
/// PartialEq
/// Display
/// --- ---
///
#[derive(Debug, Clone, PartialEq)]
pub enum CodeGenError {
    EmptyAst,
    UnsupportedNode { node_type: String },
    InvalidStructure { message: String },
}

impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeGenError::EmptyAst => write!(f, "Cannot generate SQL from an empty AST"),
            CodeGenError::UnsupportedNode { node_type } => {
                write!(f, "Unsupported node type for code generation: {}", node_type)
            }
            CodeGenError::InvalidStructure { message } => {
                write!(f, "Invalid AST structure: {}", message)
            }
        }
    }
}

/// Generate SQL from an AST
///
/// Parameters:
/// --- ---
/// ast -> The AST to generate SQL from
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL query or an error
/// --- ---
///
pub fn generate_sql(ast: &Ast) -> CodeGenResult {
    let root = ast
        .head
        .as_ref()
        .ok_or(CodeGenError::EmptyAst)?;

    let where_clause = generate_node(root)?;

    // Build the full SQL query with joins
    let sql = format!(
        "SELECT DISTINCT \
            c.subject_code, \
            c.number AS course_number, \
            c.title, \
            c.description, \
            c.credit_hours, \
            c.prerequisites, \
            c.corequisites, \
            s.sequence AS section_sequence, \
            s.max_enrollment, \
            s.enrollment, \
            s.instruction_method, \
            s.campus, \
            p.name AS professor_name, \
            p.email_address AS professor_email, \
            mt.start_minutes, \
            mt.end_minutes, \
            mt.meeting_type, \
            mt.is_monday, \
            mt.is_tuesday, \
            mt.is_wednesday, \
            mt.is_thursday, \
            mt.is_friday, \
            mt.is_saturday, \
            mt.is_sunday \
        FROM sections s \
        JOIN courses c ON s.school_id = c.school_id \
            AND s.subject_code = c.subject_code \
            AND s.course_number = c.number \
        LEFT JOIN professors p ON s.primary_professor_id = p.id \
            AND s.school_id = p.school_id \
        LEFT JOIN meeting_times mt ON s.sequence = mt.section_sequence \
            AND s.term_collection_id = mt.term_collection_id \
            AND s.school_id = mt.school_id \
            AND s.subject_code = mt.subject_code \
            AND s.course_number = mt.course_number \
        WHERE {}",
        where_clause
    );

    Ok(sql)
}

/// Generate SQL for a single AST node
fn generate_node(node: &TreeNode) -> CodeGenResult {
    match &node.node_type {
        NodeType::Query => generate_query(node),
        NodeType::LogicalTerm => generate_logical_term(node),
        NodeType::LogicalFactor => generate_logical_factor(node),
        NodeType::EntityQuery => generate_entity_query(node),
        NodeType::T(TokenType::And) => generate_and(node),
        NodeType::T(TokenType::Or) => generate_or(node),
        NodeType::ProfessorQuery => generate_professor_query(node),
        NodeType::CourseQuery => generate_course_query(node),
        NodeType::SubjectQuery => generate_subject_query(node),
        NodeType::SectionQuery => generate_section_query(node),
        NodeType::NumberQuery => generate_number_query(node),
        NodeType::TitleQuery => generate_title_query(node),
        NodeType::DescriptionQuery => generate_description_query(node),
        NodeType::CreditHoursQuery => generate_credit_hours_query(node),
        NodeType::PrereqsQuery => generate_prereqs_query(node),
        NodeType::CoreqsQuery => generate_coreqs_query(node),
        NodeType::EnrollmentCapQuery => generate_enrollment_cap_query(node),
        NodeType::InstructionMethodQuery => generate_instruction_method_query(node),
        NodeType::CampusQuery => generate_campus_query(node),
        NodeType::EnrollmentQuery => generate_enrollment_query(node),
        NodeType::FullQuery => generate_full_query(node),
        NodeType::MeetingTypeQuery => generate_meeting_type_query(node),
        NodeType::TimeQuery => generate_time_query(node),
        NodeType::DayQuery => generate_day_query(node),
        _ => Err(CodeGenError::UnsupportedNode {
            node_type: format!("{:?}", node.node_type),
        }),
    }
}

/// Generate SQL for a Query node
fn generate_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "Query node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for a LogicalTerm node
fn generate_logical_term(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "LogicalTerm node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for a LogicalFactor node
fn generate_logical_factor(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "LogicalFactor node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for an EntityQuery node
fn generate_entity_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "EntityQuery node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for AND operation
fn generate_and(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "AND node must have exactly 2 children".to_string(),
        });
    }
    let left = generate_node(&node.children[0])?;
    let right = generate_node(&node.children[1])?;
    Ok(format!("({} AND {})", left, right))
}

/// Generate SQL for OR operation
fn generate_or(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "OR node must have exactly 2 children".to_string(),
        });
    }
    let left = generate_node(&node.children[0])?;
    let right = generate_node(&node.children[1])?;
    Ok(format!("({} OR {})", left, right))
}

/// Generate SQL for ProfessorQuery node
/// Structure: children[0] = Condition, children[1] = Identifier/String
fn generate_professor_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "ProfessorQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    // Search in professor name and email
    let sql_condition = build_string_condition("p.name", &condition, &value);
    let email_condition = build_string_condition("p.email_address", &condition, &value);
    
    Ok(format!("({} OR {})", sql_condition, email_condition))
}

/// Generate SQL for CourseQuery node
fn generate_course_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "CourseQuery has no children".to_string(),
        });
    }
    // Course query can contain sub-queries or be a direct condition
    if node.children.len() == 2 {
        // Direct condition: course <condition> <value>
        let condition = extract_condition(&node.children[0])?;
        let value = extract_string_value(&node.children[1])?;
        
        // Search in title and subject code combined
        let title_cond = build_string_condition("c.title", &condition, &value);
        let subject_cond = build_string_condition("c.subject_code", &condition, &value);
        
        Ok(format!("({} OR {})", title_cond, subject_cond))
    } else {
        // Sub-query
        generate_node(&node.children[0])
    }
}

/// Generate SQL for SubjectQuery node
fn generate_subject_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "SubjectQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("c.subject_code", &condition, &value))
}

/// Generate SQL for SectionQuery node
fn generate_section_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "SectionQuery has no children".to_string(),
        });
    }
    // Section query typically contains sub-queries
    generate_node(&node.children[0])
}

/// Generate SQL for NumberQuery node
fn generate_number_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "NumberQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("c.number", &condition, &value))
}

/// Generate SQL for TitleQuery node
fn generate_title_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "TitleQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("c.title", &condition, &value))
}

/// Generate SQL for DescriptionQuery node
fn generate_description_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "DescriptionQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("c.description", &condition, &value))
}

/// Generate SQL for CreditHoursQuery node
/// Structure: children[0] = Binop, children[1] = Integer
fn generate_credit_hours_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "CreditHoursQuery must have operator and value".to_string(),
        });
    }
    let operator = extract_binop(&node.children[0])?;
    let value = extract_integer_value(&node.children[1])?;
    
    Ok(format!("c.credit_hours {} {}", operator, value))
}

/// Generate SQL for PrereqsQuery node
/// Structure: children[0] = Condition, children[1] = Identifier/String
fn generate_prereqs_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "PrereqsQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("c.prerequisites", &condition, &value))
}

/// Generate SQL for CoreqsQuery node
/// Structure: children[0] = Condition, children[1] = Identifier/String
fn generate_coreqs_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "CoreqsQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("c.corequisites", &condition, &value))
}

/// Generate SQL for EnrollmentCapQuery node
fn generate_enrollment_cap_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "EnrollmentCapQuery must have operator and value".to_string(),
        });
    }
    let operator = extract_binop(&node.children[0])?;
    let value = extract_integer_value(&node.children[1])?;
    
    Ok(format!("s.max_enrollment {} {}", operator, value))
}

/// Generate SQL for InstructionMethodQuery node
fn generate_instruction_method_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "InstructionMethodQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("s.instruction_method", &condition, &value))
}

/// Generate SQL for CampusQuery node
fn generate_campus_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "CampusQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("s.campus", &condition, &value))
}

/// Generate SQL for EnrollmentQuery node
fn generate_enrollment_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "EnrollmentQuery must have operator and value".to_string(),
        });
    }
    let operator = extract_binop(&node.children[0])?;
    let value = extract_integer_value(&node.children[1])?;
    
    Ok(format!("s.enrollment {} {}", operator, value))
}

/// Generate SQL for FullQuery node
/// "full equals true" means enrollment >= max_enrollment
fn generate_full_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "FullQuery must have condition and value".to_string(),
        });
    }
    let value = extract_string_value(&node.children[1])?;
    let is_full = value.to_lowercase() == "true";
    
    if is_full {
        Ok("s.enrollment >= s.max_enrollment".to_string())
    } else {
        Ok("s.enrollment < s.max_enrollment".to_string())
    }
}

/// Generate SQL for MeetingTypeQuery node
fn generate_meeting_type_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "MeetingTypeQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    Ok(build_string_condition("mt.meeting_type", &condition, &value))
}

/// Generate SQL for TimeQuery node
/// Structure: children[0] = String ("start"/"end")
///            children[1] = TimeRange or Binop
///            children[2] = Time (if Binop)
fn generate_time_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "TimeQuery has no children".to_string(),
        });
    }

    // Determine if this is start or end time
    let time_type = &node.children[0].node_content;
    let column = if time_type.to_lowercase().contains("start") {
        "mt.start_minutes"
    } else {
        "mt.end_minutes"
    };

    if node.children.len() == 2 {
        // Time range: start 9:00 to 17:00
        let time_range = &node.children[1];
        if time_range.node_type == NodeType::TimeRange {
            let start_time = extract_time_value(&time_range.children[0])?;
            let end_time = extract_time_value(&time_range.children[1])?;
            Ok(format!("({} >= '{}' AND {} <= '{}')", column, start_time, column, end_time))
        } else {
            Err(CodeGenError::InvalidStructure {
                message: "Expected TimeRange node".to_string(),
            })
        }
    } else if node.children.len() == 3 {
        // Comparison: start >= 9:00
        let operator = extract_binop(&node.children[1])?;
        let time_value = extract_time_value(&node.children[2])?;
        Ok(format!("{} {} '{}'", column, operator, time_value))
    } else {
        Err(CodeGenError::InvalidStructure {
            message: "TimeQuery has unexpected number of children".to_string(),
        })
    }
}

/// Generate SQL for DayQuery node
/// Structure: children[0] = String node (day name) with children[0] = Condition, children[1] = value
fn generate_day_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "DayQuery has no children".to_string(),
        });
    }

    let day_node = &node.children[0];
    let day_name = day_node.node_content.to_lowercase();
    
    // Map day names to column names
    let column = match day_name.as_str() {
        "monday" => "mt.is_monday",
        "tuesday" => "mt.is_tuesday",
        "wednesday" => "mt.is_wednesday",
        "thursday" => "mt.is_thursday",
        "friday" => "mt.is_friday",
        "saturday" => "mt.is_saturday",
        "sunday" => "mt.is_sunday",
        _ => {
            return Err(CodeGenError::InvalidStructure {
                message: format!("Unknown day: {}", day_name),
            });
        }
    };

    // Get the boolean value
    if day_node.children.len() < 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "Day node missing condition and value".to_string(),
        });
    }
    
    let value = extract_string_value(&day_node.children[1])?;
    let is_true = value.to_lowercase() == "true";
    
    // SQLite uses 0/1 for booleans
    Ok(format!("{} = {}", column, if is_true { 1 } else { 0 }))
}

/// Extract the condition type from a Condition node
fn extract_condition(node: &TreeNode) -> CodeGenResult {
    if node.node_type != NodeType::Condition {
        return Err(CodeGenError::InvalidStructure {
            message: format!("Expected Condition node, got {:?}", node.node_type),
        });
    }
    
    // The condition type is stored in the first child's node_content
    if let Some(child) = node.children.first() {
        let token_str = &child.node_content;
        Ok(token_str.clone())
    } else if let Some(token) = node.lexical_token {
        Ok(format!("{:?}", token.get_token_type()))
    } else {
        Err(CodeGenError::InvalidStructure {
            message: "Condition node has no content".to_string(),
        })
    }
}

/// Extract the binary operator from a Binop node
fn extract_binop(node: &TreeNode) -> CodeGenResult {
    if node.node_type != NodeType::Binop {
        return Err(CodeGenError::InvalidStructure {
            message: format!("Expected Binop node, got {:?}", node.node_type),
        });
    }
    
    // Get the operator from the child node's content or lexical token
    if let Some(child) = node.children.first() {
        let token_str = &child.node_content;
        Ok(token_to_sql_operator(token_str))
    } else if let Some(token) = node.lexical_token {
        Ok(token_to_sql_operator(&format!("{:?}", token.get_token_type())))
    } else {
        Err(CodeGenError::InvalidStructure {
            message: "Binop node has no content".to_string(),
        })
    }
}

/// Convert a token type string to SQL operator
fn token_to_sql_operator(token: &str) -> String {
    let upper = token.to_uppercase();
    match upper.as_str() {
        "T_EQUALS" | "T_EQUALSWORD" | "T_IS" | "T_EQUAL" => "=".to_string(),
        "T_NOTEQUALS" => "!=".to_string(),
        "T_LESSTHAN" | "T_LESS" => "<".to_string(),
        "T_GREATERTHAN" | "T_GREATER" => ">".to_string(),
        "T_LESSEQUAL" => "<=".to_string(),
        "T_GREATEREQUAL" => ">=".to_string(),
        "T_LEAST" => ">=".to_string(), // "at least" means >=
        "T_MOST" => "<=".to_string(),  // "at most" means <=
        "T_MORE" => ">".to_string(),   // "more than"
        "T_FEWER" => "<".to_string(),  // "fewer than"
        _ => "=".to_string(),
    }
}

/// Extract string value from an Identifier or String node
fn extract_string_value(node: &TreeNode) -> CodeGenResult {
    match &node.node_type {
        NodeType::Identifier | NodeType::EmailIdentifier | NodeType::String => {
            let value = node.node_content.clone();
            Ok(value.trim_matches('"').to_string())
        }
        _ => Err(CodeGenError::InvalidStructure {
            message: format!("Expected string-like node, got {:?}", node.node_type),
        }),
    }
}

/// Extract integer value from an Integer node
fn extract_integer_value(node: &TreeNode) -> Result<i64, CodeGenError> {
    if node.node_type != NodeType::Integer {
        return Err(CodeGenError::InvalidStructure {
            message: format!("Expected Integer node, got {:?}", node.node_type),
        });
    }
    
    node.node_content
        .parse()
        .map_err(|_| CodeGenError::InvalidStructure {
            message: format!("Cannot parse '{}' as integer", node.node_content),
        })
}

/// Extract time value from a Time node
fn extract_time_value(node: &TreeNode) -> CodeGenResult {
    if node.node_type != NodeType::Time {
        return Err(CodeGenError::InvalidStructure {
            message: format!("Expected Time node, got {:?}", node.node_type),
        });
    }
    
    let time_str = &node.node_content;
    Ok(normalize_time(time_str))
}

/// Normalize time string to HH:MM:SS format
fn normalize_time(time: &str) -> String {
    let time_lower = time.to_lowercase();
    let is_pm = time_lower.contains("pm");
    let is_am = time_lower.contains("am");
    
    // Remove am/pm suffix
    let clean = time_lower
        .replace("am", "")
        .replace("pm", "")
        .trim()
        .to_string();
    
    // Parse hours and minutes
    let parts: Vec<&str> = clean.split(':').collect();
    let hours: i32 = parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let minutes: i32 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    
    // Convert to 24-hour format
    let hours_24 = if is_pm && hours != 12 {
        hours + 12
    } else if is_am && hours == 12 {
        0
    } else {
        hours
    };
    
    format!("{:02}:{:02}:00", hours_24, minutes)
}

/// Build a SQL string condition based on the condition type
fn build_string_condition(column: &str, condition: &str, value: &str) -> String {
    let escaped_value = value.replace('\'', "''");
    let upper = condition.to_uppercase();
    
    match upper.as_str() {
        s if s.contains("EQUALS") || s.contains("IS") || s.contains("EQUAL") => {
            format!("LOWER({}) = LOWER('{}')", column, escaped_value)
        }
        s if s.contains("NOTEQUALS") || s.contains("NOT") => {
            format!("LOWER({}) != LOWER('{}')", column, escaped_value)
        }
        s if s.contains("CONTAINS") || s.contains("HAS") => {
            format!("{} LIKE '%{}%' COLLATE NOCASE", column, escaped_value)
        }
        s if s.contains("STARTS") => {
            format!("{} LIKE '{}%' COLLATE NOCASE", column, escaped_value)
        }
        s if s.contains("ENDS") => {
            format!("{} LIKE '%{}' COLLATE NOCASE", column, escaped_value)
        }
        _ => {
            format!("LOWER({}) = LOWER('{}')", column, escaped_value)
        }
    }
}
