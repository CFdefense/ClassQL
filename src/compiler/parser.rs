/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use crate::compiler::token::{Token, TokenType as NodeType};
use crate::tui::errors::AppError;

pub struct Parser {
    ast: AST,
    token_pointer: usize,
}

#[derive(Debug)]
pub struct AST {
    head: Option<TreeNode>,
}

#[derive(Debug)]
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

impl Parser  {
    pub fn new() -> Self{
        Parser { ast: AST::new(), token_pointer: 0 }
    }

    pub fn parse(&mut self, tokens: &Vec<Token>) -> Result<(),AppError> {
        // parse tokens into abstract syntax tree
        self.token_pointer = 0;
        self.ast = AST::new();
        self.parse_query();

        Ok(())
    }

    fn parse_query(&mut self) {}
    fn parse_entity_query(&mut self) {}
    fn parse_professor_query(&mut self) {}
    fn parse_course_query(&mut self) {}
    fn parse_subject_query(&mut self) {}
    fn parse_number_query(&mut self) {}
    fn parse_title_query(&mut self) {}
    fn parse_description_query(&mut self) {}
    fn parse_credit_hours_query(&mut self) {}
    fn parse_prereqs_query(&mut self) {}
    fn parse_coreqs_query(&mut self) {}
    fn parse_enrollment_cap_query(&mut self) {}
    fn parse_instruction_method_query(&mut self) {}
    fn parse_campus_query(&mut self) {}
    fn parse_enrollment_query(&mut self) {}
    fn parse_full_query(&mut self) {}
    fn parse_meeting_type_query(&mut self) {}
    fn parse_time_query(&mut self) {}
    fn parse_time_range(&mut self) {}
    fn parse_day_query(&mut self) {}
    fn parse_time(&mut self) {}
    fn parse_condition(&mut self) {}
    fn parse_binop(&mut self) {}
    fn parse_string_list(&mut self) {}
    fn parse_string(&mut self) {}
    fn parse_integer(&mut self) {}
    fn parse_identifier(&mut self) {}
    fn parse_email_identifier(&mut self) {}
}
