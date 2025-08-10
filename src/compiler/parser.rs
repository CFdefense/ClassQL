/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use crate::compiler::token::{Token, TokenType as NodeType};

pub struct Parser {
    ast: AST,
    tokens: Vec<Token>,
}

pub struct AST {
    head: Option<TreeNode>,
}

struct TreeNode {
    children: Vec<TreeNode>,
    node_type: NodeType,
    node_content: String,
}

impl AST {
    fn new() -> Self {
        AST { head: None}
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self{
        Parser { ast: AST::new(), tokens: tokens }
    }

    fn next_token(&mut self) -> Token {
        return self.tokens.remove(0);
    }

    pub fn parse(&mut self) {
        // parse tokens into abstract syntax tree
        

    }

    fn parse_query() {}
    fn parse_entity_query() {}
    fn parse_professor_query() {}
    fn parse_course_query() {}
    fn parse_subject_query() {}
    fn parse_number_query() {}
    fn parse_title_query() {}
    fn parse_description_query() {}
    fn parse_credit_hours_query() {}
    fn parse_prereqs_query() {}
    fn parse_coreqs_query() {}
    fn parse_enrollment_cap_query() {}
    fn parse_instruction_method_query() {}
    fn parse_campus_query() {}
    fn parse_enrollment_query() {}
    fn parse_full_query() {}
    fn parse_meeting_type_query() {}
    fn parse_time_query() {}
    fn parse_time_range() {}
    fn parse_day_query() {}
    fn parse_time() {}
    fn parse_condition() {}
    fn parse_binop() {}
    fn parse_string_list() {}
    fn parse_string() {}
    fn parse_integer() {}
    fn parse_identifier() {}
    fn parse_email_identifier() {}
}
