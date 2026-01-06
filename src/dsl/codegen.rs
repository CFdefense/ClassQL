/// src/dsl/codegen.rs
///
/// Code generator module for the DSL
///
/// Responsible for converting the AST into SQL queries
///
/// Contains:
/// --- ---
/// generate_sql -> Main function to generate SQL from an AST
/// CodeGenError -> Error type for code generation
/// generate_node -> Generate SQL for a single AST node (dispatcher)
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
/// extract_condition -> Extract condition type from Condition node
/// extract_binop -> Extract binary operator from Binop node
/// extract_string_value -> Extract string value from Identifier/String node
/// extract_integer_value -> Extract integer value from Integer node
/// extract_time_value -> Extract time value from Time node
/// token_to_sql_operator -> Convert token type string to SQL operator
/// normalize_time -> Normalize time string to HH:MM:SS format
/// build_string_condition -> Build SQL string condition based on condition type
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

    // generate WHERE clause - day queries use the joined mt table directly
    let where_clause = generate_node(root)?;

    // build the full SQL query with joins and aggregation
    let sql = format!(
        "SELECT \
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
            GROUP_CONCAT( \
                (CASE WHEN mt.is_monday = 1 THEN 'M' ELSE '' END || \
                 CASE WHEN mt.is_tuesday = 1 THEN 'T' ELSE '' END || \
                 CASE WHEN mt.is_wednesday = 1 THEN 'W' ELSE '' END || \
                 CASE WHEN mt.is_thursday = 1 THEN 'TH' ELSE '' END || \
                 CASE WHEN mt.is_friday = 1 THEN 'F' ELSE '' END || \
                 CASE WHEN mt.is_saturday = 1 THEN 'S' ELSE '' END || \
                 CASE WHEN mt.is_sunday = 1 THEN 'U' ELSE '' END) || \
                ':' || mt.start_minutes || '-' || mt.end_minutes, \
                '|' \
            ) AS meeting_times, \
            GROUP_CONCAT(DISTINCT mt.meeting_type) AS meeting_type, \
            MAX(mt.is_monday) AS is_monday, \
            MAX(mt.is_tuesday) AS is_tuesday, \
            MAX(mt.is_wednesday) AS is_wednesday, \
            MAX(mt.is_thursday) AS is_thursday, \
            MAX(mt.is_friday) AS is_friday, \
            MAX(mt.is_saturday) AS is_saturday, \
            MAX(mt.is_sunday) AS is_sunday \
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
        WHERE {} \
        GROUP BY \
            c.subject_code, \
            c.number, \
            c.title, \
            c.description, \
            c.credit_hours, \
            c.prerequisites, \
            c.corequisites, \
            s.sequence, \
            s.term_collection_id, \
            s.school_id, \
            s.max_enrollment, \
            s.enrollment, \
            s.instruction_method, \
            s.campus, \
            p.name, \
            p.email_address",
        where_clause
    );

    Ok(sql)
}

/// Generate SQL for a single AST node
///
/// This is the main dispatcher function that routes to the appropriate generator
/// based on the node type.
///
/// Parameters:
/// --- ---
/// node -> The AST node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The Query node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "Query node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for a LogicalTerm node
///
/// Parameters:
/// --- ---
/// node -> The LogicalTerm node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_logical_term(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "LogicalTerm node has no children".to_string(),
        });
    }
    
    // collect all conditions from the AND chain
    let mut conditions = Vec::new();
    let mut current = &node.children[0];
    
    // traverse the AND chain to collect all conditions
    loop {
        match &current.node_type {
            NodeType::T(TokenType::And) => {
                if current.children.len() >= 2 {
                    conditions.push(generate_node(&current.children[0])?);
                    current = &current.children[1];
                } else {
                    break;
                }
            }
            _ => {
                conditions.push(generate_node(current)?);
                break;
            }
        }
    }
    
    // sort: non-EXISTS conditions first, then EXISTS subqueries
    // this ensures more selective filters (like professor) are evaluated before
    // expensive EXISTS subqueries (like day queries)
    conditions.sort_by(|a, b| {
        let a_is_exists = a.starts_with("EXISTS");
        let b_is_exists = b.starts_with("EXISTS");
        match (a_is_exists, b_is_exists) {
            (true, false) => std::cmp::Ordering::Greater,  // exists goes after
            (false, true) => std::cmp::Ordering::Less,     // non-exists goes first
            _ => std::cmp::Ordering::Equal,                // keep original order
        }
    });
    
    Ok(conditions.join(" AND "))
}

/// Generate SQL for a LogicalFactor node
///
/// Parameters:
/// --- ---
/// node -> The LogicalFactor node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_logical_factor(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "LogicalFactor node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for an EntityQuery node
///
/// Parameters:
/// --- ---
/// node -> The EntityQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_entity_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "EntityQuery node has no children".to_string(),
        });
    }
    generate_node(&node.children[0])
}

/// Generate SQL for AND operation
///
/// Parameters:
/// --- ---
/// node -> The AND node to generate SQL for (must have 2 children)
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment with AND condition or an error
/// --- ---
///
fn generate_and(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "AND node must have exactly 2 children".to_string(),
        });
    }
    let left = generate_node(&node.children[0])?;
    let right = generate_node(&node.children[1])?;
    
    // put non-EXISTS conditions first so they filter rows before EXISTS subqueries
    let (first, second) = if right.starts_with("EXISTS") && !left.starts_with("EXISTS") {
        (left, right)  // non-exists first
    } else {
        (left, right)  // keep original order if both are exists or left is exists
    };
    
    Ok(format!("({} AND {})", first, second))
}

/// Generate SQL for OR operation
///
/// Parameters:
/// --- ---
/// node -> The OR node to generate SQL for (must have 2 children)
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment with OR condition or an error
/// --- ---
///
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
///
/// Structure: children[0] = Condition, children[1] = Identifier/String
/// Searches in both professor name and email address.
///
/// Parameters:
/// --- ---
/// node -> The ProfessorQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment (OR condition for name/email) or an error
/// --- ---
///
fn generate_professor_query(node: &TreeNode) -> CodeGenResult {
    if node.children.len() != 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "ProfessorQuery must have condition and value".to_string(),
        });
    }
    let condition = extract_condition(&node.children[0])?;
    let value = extract_string_value(&node.children[1])?;
    
    // search in professor name and email
    let sql_condition = build_string_condition("p.name", &condition, &value);
    let email_condition = build_string_condition("p.email_address", &condition, &value);
    
    Ok(format!("({} OR {})", sql_condition, email_condition))
}

/// Generate SQL for CourseQuery node
///
/// Can handle direct conditions or sub-queries.
/// Searches in both course title and subject code.
///
/// Parameters:
/// --- ---
/// node -> The CourseQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_course_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "CourseQuery has no children".to_string(),
        });
    }

    // course query can contain sub-queries or be a direct condition
    if node.children.len() == 2 {
        // direct condition: course <condition> <value>
        let condition = extract_condition(&node.children[0])?;
        let value = extract_string_value(&node.children[1])?;
        
        // search in title and subject code combined
        let title_cond = build_string_condition("c.title", &condition, &value);
        let subject_cond = build_string_condition("c.subject_code", &condition, &value);
        
        Ok(format!("({} OR {})", title_cond, subject_cond))
    } else {
        // sub-query
        generate_node(&node.children[0])
    }
}

/// Generate SQL for SubjectQuery node
///
/// Parameters:
/// --- ---
/// node -> The SubjectQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Section query typically contains sub-queries.
///
/// Parameters:
/// --- ---
/// node -> The SectionQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The NumberQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The TitleQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The DescriptionQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Structure: children[0] = Binop, children[1] = Integer
///
/// Parameters:
/// --- ---
/// node -> The CreditHoursQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Structure: children[0] = Condition, children[1] = Identifier/String
///
/// Parameters:
/// --- ---
/// node -> The PrereqsQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Structure: children[0] = Condition, children[1] = Identifier/String
///
/// Parameters:
/// --- ---
/// node -> The CoreqsQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The EnrollmentCapQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The InstructionMethodQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The CampusQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The EnrollmentQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// "full equals true" means enrollment >= max_enrollment
///
/// Parameters:
/// --- ---
/// node -> The FullQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The MeetingTypeQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
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
///
/// Structure: children[0] = String ("start"/"end")
///            children[1] = TimeRange or Binop
///            children[2] = Time (if Binop)
///
/// Parameters:
/// --- ---
/// node -> The TimeQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_time_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "TimeQuery has no children".to_string(),
        });
    }

    // determine if this is start or end time
    let time_type = &node.children[0].node_content;
    let column = if time_type.to_lowercase().contains("start") {
        "mt.start_minutes"
    } else {
        "mt.end_minutes"
    };

    if node.children.len() == 2 {
        // time range: start 9:00 to 17:00
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
        // comparison: start >= 9:00
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
///
/// Structure: children[0] = String node (day name) with children[0] = Condition, children[1] = value
///
/// Parameters:
/// --- ---
/// node -> The DayQuery node to generate SQL for
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The generated SQL fragment or an error
/// --- ---
///
fn generate_day_query(node: &TreeNode) -> CodeGenResult {
    if node.children.is_empty() {
        return Err(CodeGenError::InvalidStructure {
            message: "DayQuery has no children".to_string(),
        });
    }

    let day_node = &node.children[0];
    let day_name = day_node.node_content.to_lowercase();
    
    // map day names to column names
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

    if day_node.children.len() < 2 {
        return Err(CodeGenError::InvalidStructure {
            message: "Day node missing condition and value".to_string(),
        });
    }
    
    let value = extract_string_value(&day_node.children[1])?;
    let is_true = value.to_lowercase() == "true";
    let day_value = if is_true { 1 } else { 0 };
    
    // use the joined mt table directly in WHERE clause
    // this is efficient because we're already joining meeting_times
    Ok(format!("{} = {}", column, day_value))
}

/// Extract the condition type from a Condition node
///
/// Parameters:
/// --- ---
/// node -> The Condition node to extract from
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The condition string or an error
/// --- ---
///
fn extract_condition(node: &TreeNode) -> CodeGenResult {
    if node.node_type != NodeType::Condition {
        return Err(CodeGenError::InvalidStructure {
            message: format!("Expected Condition node, got {:?}", node.node_type),
        });
    }
    
    // for "is not", the condition node's node_content is set to "is not"
    // otherwise, the condition type is stored in the first child's node_content
    if !node.node_content.is_empty() && node.node_content == "is not" {
        Ok("is not".to_string())
    } else if let Some(child) = node.children.first() {
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
///
/// Parameters:
/// --- ---
/// node -> The Binop node to extract from
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The SQL operator string or an error
/// --- ---
///
fn extract_binop(node: &TreeNode) -> CodeGenResult {
    if node.node_type != NodeType::Binop {
        return Err(CodeGenError::InvalidStructure {
            message: format!("Expected Binop node, got {:?}", node.node_type),
        });
    }
    
    // get the operator from the child node's content or lexical token
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
///
/// Parameters:
/// --- ---
/// token -> The token type string to convert
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The corresponding SQL operator
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The Identifier or String node to extract from
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The extracted string value or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The Integer node to extract from
/// --- ---
///
/// Returns:
/// --- ---
/// Result<i64, CodeGenError> -> The extracted integer value or an error
/// --- ---
///
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
///
/// Parameters:
/// --- ---
/// node -> The Time node to extract from
/// --- ---
///
/// Returns:
/// --- ---
/// CodeGenResult -> The normalized time string (HH:MM:SS) or an error
/// --- ---
///
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
///
/// Handles various time formats including am/pm notation and converts to 24-hour format.
///
/// Parameters:
/// --- ---
/// time -> The time string to normalize
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The normalized time string in HH:MM:SS format
/// --- ---
///
fn normalize_time(time: &str) -> String {
    let time_lower = time.to_lowercase();
    let is_pm = time_lower.contains("pm");
    let is_am = time_lower.contains("am");
    
    // remove am/pm suffix
    let clean = time_lower
        .replace("am", "")
        .replace("pm", "")
        .trim()
        .to_string();
    
    // parse hours and minutes
    let parts: Vec<&str> = clean.split(':').collect();
    let hours: i32 = parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let minutes: i32 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    
    // convert to 24-hour format
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
///
/// Supports various string conditions: equals, contains, starts with, ends with, etc.
///
/// Parameters:
/// --- ---
/// column -> The SQL column name
/// condition -> The condition type (e.g., "contains", "equals", "starts with")
/// value -> The value to match against
/// --- ---
///
/// Returns:
/// --- ---
/// String -> The generated SQL condition string
/// --- ---
///
fn build_string_condition(column: &str, condition: &str, value: &str) -> String {
    let escaped_value = value.replace('\'', "''");
    let upper = condition.to_uppercase();
    
    match upper.as_str() {
        s if s == "IS NOT" || s.contains("IS NOT") => {
            format!("LOWER({}) != LOWER('{}')", column, escaped_value)
        }
        // check "DOES NOT CONTAIN" / "DOESN'T CONTAIN" before "CONTAINS" since it contains "CONTAINS"
        s if s.contains("DOES NOT CONTAIN") || s.contains("DOESN'T CONTAIN") || s.contains("DOESNT CONTAIN") => {
            format!("{} NOT LIKE '%{}%' COLLATE NOCASE", column, escaped_value)
        }
        s if s.contains("NOTEQUALS") || (s.contains("NOT") && !s.contains("IS NOT") && !s.contains("DOES NOT")) => {
            format!("LOWER({}) != LOWER('{}')", column, escaped_value)
        }
        s if s.contains("EQUALS") || s.contains("IS") || s.contains("EQUAL") => {
            format!("LOWER({}) = LOWER('{}')", column, escaped_value)
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
