/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use crate::compiler::token::{Token, TokenType};
use crate::tui::errors::SyntaxError;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    T(TokenType),
    Query,
    LogicalTerm,
    LogicalFactor,
    EntityQuery,
    ProfessorQuery,
    CourseQuery,
    SubjectQuery,
    NumberQuery,
    TitleQuery,
    DescriptionQuery,
    CreditHoursQuery,
    PrereqsQuery,
    CoreqsQuery,
    EnrollmentCapQuery,
    InstructionMethodQuery,
    CampusQuery,
    EnrollmentQuery,
    FullQuery,
    MeetingTypeQuery,
    TimeQuery,
}

impl NodeType {
    fn to_string(&self) -> String {
        match self {
            NodeType::T(token_type) => format!("{:?}", token_type),
            _ => format!("{:?}", self),
        }
    }
}

pub struct Parser {
    ast: AST,
    token_pointer: usize,
}

#[derive(Debug)]
pub struct AST {
    head: Option<TreeNode>,
}

#[derive(Debug, Clone)]
struct TreeNode {
    children: Vec<TreeNode>,
    node_type: NodeType,
    node_content: String,
}

impl TreeNode {
    fn new(node_type: NodeType, node_content: String) -> Self {
        TreeNode { children: Vec::new(), node_type: node_type, node_content: node_content }
    }
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

    pub fn parse(&mut self, tokens: &Vec<Token>) -> Result<(), (SyntaxError, Vec<Token>)> {
        self.token_pointer = 0;
        self.ast = AST::new();
        
        // Create Query node and set it as AST head
        let query_node = self.parse_query(tokens)?;
        self.ast.head = Some(query_node);
        
        Ok(())
    }

    // <query> ::= <logical_term> ("or" <logical_term>)*
    fn parse_query(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // Create Query node first
        let mut query_node = TreeNode::new(NodeType::Query, NodeType::Query.to_string());
        
        // Parse the first logical term
        let mut first_term = self.parse_logical_term(tokens)?;
        
        // continue parsing logical terms until we hit the end of the tokens or we hit a non-or token
        while self.token_pointer < tokens.len() && tokens[self.token_pointer].token_type == TokenType::Or {
            self.token_pointer += 1;
            let next_term = self.parse_logical_term(tokens)?;
            
            let mut or_node = TreeNode::new(NodeType::T(TokenType::Or), NodeType::T(TokenType::Or).to_string());
            or_node.children.push(first_term);
            or_node.children.push(next_term);
            first_term = or_node;
        }
        
        // Add the final result to query node
        query_node.children.push(first_term);
        Ok(query_node)
    }
    
    // <logical_term> ::= <logical_factor> ("and" <logical_factor>)*
    fn parse_logical_term(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut logical_term_node = TreeNode::new(NodeType::LogicalTerm, NodeType::LogicalTerm.to_string());

        // parse the first logical factor
        let mut first_factor = self.parse_logical_factor(tokens)?;

        // continue parsing logical factors until we hit the end of the tokens or we hit a non-and token
        while self.token_pointer < tokens.len() && tokens[self.token_pointer].token_type == TokenType::And {
            self.token_pointer += 1;
            let next_factor = self.parse_logical_factor(tokens)?;

            let mut and_node = TreeNode::new(NodeType::T(TokenType::And), NodeType::T(TokenType::And).to_string());
            and_node.children.push(first_factor);
            and_node.children.push(next_factor);
            first_factor = and_node;
        }

        logical_term_node.children.push(first_factor);
        Ok(logical_term_node)
    }

    // <logical_factor> ::= <entity_query> | "(" <query> ")"
    fn parse_logical_factor(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut logical_factor_node = TreeNode::new(NodeType::LogicalFactor, NodeType::LogicalFactor.to_string());

        let first_query = self.parse_entity_query(tokens)?;

        // continue if more
        if self.token_pointer < tokens.len() && tokens[self.token_pointer].token_type == TokenType::LeftParen {
            self.token_pointer += 1;
            let next_query = self.parse_query(tokens)?;
            
            // check that next token is closing paren or error
            if self.token_pointer >= tokens.len() || tokens[self.token_pointer].token_type != TokenType::RightParen {
                // Return error with the problematic tokens
                let problematic_tokens = vec![tokens[self.token_pointer - 1].clone()]; // the opening paren
                return Err((SyntaxError::UnclosedParenthesis, problematic_tokens));
            }
            self.token_pointer += 1; // consume the closing paren
        }

        logical_factor_node.children.push(first_query);
        Ok(logical_factor_node)
    }

    // <entity_query> ::= <professor_query> | <course_query> | <section_query> | <meeting_type_query> | <time_query> | <day_query>
    fn parse_entity_query(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut entity_query = TreeNode::new(NodeType::EntityQuery, NodeType::EntityQuery.to_string());

        // match next keyword
        let next_query = match tokens[self.token_pointer].token_type {
            
            _ => return Err((SyntaxError::UnexpectedToken(tokens[self.token_pointer].token_type.to_string()), vec![tokens[self.token_pointer].clone()]))
        };

        entity_query.children.push(next_query);
        Ok(entity_query)
    }
    fn parse_professor_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_course_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_subject_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_number_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_title_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_description_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_credit_hours_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_prereqs_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_coreqs_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_enrollment_cap_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_instruction_method_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_campus_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_enrollment_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_full_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_meeting_type_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_time_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_time_range(&mut self, tokens: &Vec<Token>) {}
    fn parse_day_query(&mut self, tokens: &Vec<Token>) {}
    fn parse_time(&mut self, tokens: &Vec<Token>) {}
    fn parse_condition(&mut self, tokens: &Vec<Token>) {}
    fn parse_binop(&mut self, tokens: &Vec<Token>) {}
    fn parse_string_list(&mut self, tokens: &Vec<Token>) {}
    fn parse_string(&mut self, tokens: &Vec<Token>) {}
    fn parse_integer(&mut self, tokens: &Vec<Token>) {}
    fn parse_identifier(&mut self, tokens: &Vec<Token>) {}
    fn parse_email_identifier(&mut self, tokens: &Vec<Token>) {}
}
