/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use crate::compiler::token::{Token, TokenType};
use crate::tui::errors::SyntaxError;
use std::vec;

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
    SectionQuery,
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
    TimeRange,
    DayQuery,
    Time,
    Condition,
    Binop,
    StringList,
    String,
    Integer,
    Identifier,
    EmailIdentifier,
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
        TreeNode {
            children: Vec::new(),
            node_type,
            node_content,
        }
    }
}

impl AST {
    fn new() -> Self {
        AST { head: None }
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            ast: AST::new(),
            token_pointer: 0,
        }
    }

    fn next_token(&mut self, tokens: &Vec<Token>) -> Result<Token, String> {
        if self.token_pointer < tokens.len() {
            let token = tokens[self.token_pointer].clone();
            self.token_pointer += 1;
            Ok(token)
        } else {
            return Err("Token stream empty, but attempted to take a token".into());
        }
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
        loop {
            let next_token = self.next_token(tokens).map_err(|_| {
                (
                    SyntaxError::MissingToken("Expected 'or' operator or end of query".into()),
                    vec![],
                )
            })?;

            if *next_token.get_token_type() != TokenType::Or {
                // Put the token back by decrementing the pointer
                self.token_pointer -= 1;
                break;
            }

            let next_term = self.parse_logical_term(tokens)?;

            let mut or_node = TreeNode::new(
                NodeType::T(TokenType::Or),
                NodeType::T(TokenType::Or).to_string(),
            );
            or_node.children.push(first_term);
            or_node.children.push(next_term);
            first_term = or_node;
        }

        // Add the final result to query node
        query_node.children.push(first_term);
        Ok(query_node)
    }

    // <logical_term> ::= <logical_factor> ("and" <logical_factor>)*
    fn parse_logical_term(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut logical_term_node =
            TreeNode::new(NodeType::LogicalTerm, NodeType::LogicalTerm.to_string());

        // parse the first logical factor
        let mut first_factor = self.parse_logical_factor(tokens)?;

        // continue parsing logical factors until we hit the end of the tokens or we hit a non-and token
        loop {
            let next_token = self.next_token(tokens).map_err(|_| {
                (
                    SyntaxError::MissingToken(
                        "Expected 'and' operator or end of logical term".into(),
                    ),
                    vec![],
                )
            })?;

            if *next_token.get_token_type() != TokenType::And {
                // Put the token back by decrementing the pointer
                self.token_pointer -= 1;
                break;
            }

            let next_factor = self.parse_logical_factor(tokens)?;

            let mut and_node = TreeNode::new(
                NodeType::T(TokenType::And),
                NodeType::T(TokenType::And).to_string(),
            );
            and_node.children.push(first_factor);
            and_node.children.push(next_factor);
            first_factor = and_node;
        }

        logical_term_node.children.push(first_factor);
        Ok(logical_term_node)
    }

    // <logical_factor> ::= <entity_query> | "(" <query> ")"
    fn parse_logical_factor(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut logical_factor_node =
            TreeNode::new(NodeType::LogicalFactor, NodeType::LogicalFactor.to_string());

        let first_query = self.parse_entity_query(tokens)?;
        logical_factor_node.children.push(first_query);

        // continue if more tokens available
        if self.token_pointer < tokens.len() {
            let next_token = self.next_token(tokens).map_err(|_| {
                (
                    SyntaxError::MissingToken(
                        "Expected opening parenthesis or end of logical factor".into(),
                    ),
                    vec![],
                )
            })?;

            if *next_token.get_token_type() == TokenType::LeftParen {
                let next_query = self.parse_query(tokens)?;

                // check that next token is closing paren or error
                let closing_token = self.next_token(tokens).map_err(|_| {
                    (
                        SyntaxError::MissingToken("Expected closing parenthesis".into()),
                        vec![],
                    )
                })?;

                if *closing_token.get_token_type() != TokenType::RightParen {
                    // Return error with the problematic tokens
                    let problematic_tokens = vec![next_token]; // the opening paren
                    return Err((SyntaxError::UnclosedParenthesis, problematic_tokens));
                }

                // Add the parenthesized query as a child
                logical_factor_node.children.push(next_query);
            } else {
                // Put the token back by decrementing the pointer
                self.token_pointer -= 1;
            }
        }

        Ok(logical_factor_node)
    }

    // <entity_query> ::= <professor_query> | <course_query> | <section_query> | <meeting_type_query> | <time_query> | <day_query>
    fn parse_entity_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut entity_query =
            TreeNode::new(NodeType::EntityQuery, NodeType::EntityQuery.to_string());

        // match next keyword
        let next_token = self.next_token(tokens).map_err(|e| {
            (
                SyntaxError::MissingToken("Expected entity query keyword".into()),
                vec![],
            )
        })?;

        let next_query = match *next_token.get_token_type() {
            TokenType::Prof => self.parse_professor_query(tokens)?,
            TokenType::Course => self.parse_course_query(tokens)?,
            TokenType::Subject => self.parse_subject_query(tokens)?,
            TokenType::Title => self.parse_title_query(tokens)?,
            TokenType::Section => self.parse_section_query(tokens)?,
            TokenType::Number => self.parse_number_query(tokens)?,
            TokenType::Description => self.parse_description_query(tokens)?,
            TokenType::Credit => self.parse_credit_hours_query(tokens)?,
            TokenType::Prereqs => self.parse_prereqs_query(tokens)?,
            TokenType::Corereqs => self.parse_coreqs_query(tokens)?,
            TokenType::Enrollment => self.parse_enrollment_query(tokens)?,
            TokenType::Method => self.parse_instruction_method_query(tokens)?,
            TokenType::Campus => self.parse_campus_query(tokens)?,
            TokenType::Meeting => self.parse_meeting_type_query(tokens)?,
            _ => {
                return Err((
                    SyntaxError::UnexpectedToken(next_token.get_token_type().to_string()),
                    vec![next_token],
                ))
            }
        };

        entity_query.children.push(next_query);
        Ok(entity_query)
    }

    // <professor_query> ::= "prof" <condition> <string>
    fn parse_professor_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut prof_node = TreeNode::new(
            NodeType::ProfessorQuery,
            NodeType::ProfessorQuery.to_string(),
        );

        let prof_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected 'prof' keyword".into()),
                vec![],
            )
        })?;

        if *prof_token.get_token_type() != TokenType::Prof {
            return Err((
                SyntaxError::UnexpectedToken(prof_token.get_token_type().to_string()),
                vec![prof_token],
            ));
        } else {
            let condition = self.parse_condition(tokens)?;
            let string = self.parse_string(tokens)?;

            prof_node.children.push(condition);
            prof_node.children.push(string);
        }

        return Ok(prof_node);
    }

    // <course_query> ::= "course" <subject_query> | <number_query> | <title_query> | <description_query> | <credit_hours_query> | <prereqs_query> | <coreqs_query>
    fn parse_course_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut course_node =
            TreeNode::new(NodeType::CourseQuery, NodeType::CourseQuery.to_string());

        let course_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected 'course' keyword".into()),
                vec![],
            )
        })?;

        if *course_token.get_token_type() != TokenType::Course {
            return Err((
                SyntaxError::UnexpectedToken(course_token.get_token_type().to_string()),
                vec![course_token],
            ));
        }

        let next_query = match *tokens[self.token_pointer].get_token_type() {
            TokenType::Subject => self.parse_subject_query(tokens)?,
            TokenType::Number => self.parse_number_query(tokens)?,
            TokenType::Title => self.parse_title_query(tokens)?,
            TokenType::Description => self.parse_description_query(tokens)?,
            TokenType::Credit => self.parse_credit_hours_query(tokens)?,
            TokenType::Prereqs => self.parse_prereqs_query(tokens)?,
            TokenType::Corereqs => self.parse_coreqs_query(tokens)?,
            _ => {
                return Err((
                    SyntaxError::UnexpectedToken(
                        tokens[self.token_pointer].get_token_type().to_string(),
                    ),
                    vec![tokens[self.token_pointer].clone()],
                ))
            }
        };

        course_node.children.push(next_query);
        Ok(course_node)
    }
    fn parse_subject_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::SubjectQuery,
            NodeType::SubjectQuery.to_string(),
        ))
    }
    fn parse_section_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::SectionQuery,
            NodeType::SectionQuery.to_string(),
        ))
    }
    fn parse_number_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::NumberQuery,
            NodeType::NumberQuery.to_string(),
        ))
    }
    fn parse_title_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::TitleQuery,
            NodeType::TitleQuery.to_string(),
        ))
    }
    fn parse_description_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::DescriptionQuery,
            NodeType::DescriptionQuery.to_string(),
        ))
    }
    fn parse_credit_hours_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::CreditHoursQuery,
            NodeType::CreditHoursQuery.to_string(),
        ))
    }
    fn parse_prereqs_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::PrereqsQuery,
            NodeType::PrereqsQuery.to_string(),
        ))
    }
    fn parse_coreqs_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::CoreqsQuery,
            NodeType::CoreqsQuery.to_string(),
        ))
    }
    fn parse_enrollment_cap_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::EnrollmentCapQuery,
            NodeType::EnrollmentCapQuery.to_string(),
        ))
    }
    fn parse_instruction_method_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::InstructionMethodQuery,
            NodeType::InstructionMethodQuery.to_string(),
        ))
    }
    fn parse_campus_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::CampusQuery,
            NodeType::CampusQuery.to_string(),
        ))
    }
    fn parse_enrollment_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::EnrollmentQuery,
            NodeType::EnrollmentQuery.to_string(),
        ))
    }
    fn parse_full_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::FullQuery,
            NodeType::FullQuery.to_string(),
        ))
    }
    fn parse_meeting_type_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::MeetingTypeQuery,
            NodeType::MeetingTypeQuery.to_string(),
        ))
    }
    fn parse_time_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::TimeQuery,
            NodeType::TimeQuery.to_string(),
        ))
    }
    fn parse_time_range(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::TimeRange,
            NodeType::TimeRange.to_string(),
        ))
    }
    fn parse_day_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::DayQuery,
            NodeType::DayQuery.to_string(),
        ))
    }
    fn parse_time(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(NodeType::Time, NodeType::Time.to_string()))
    }
    fn parse_condition(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::Condition,
            NodeType::Condition.to_string(),
        ))
    }
    fn parse_binop(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(NodeType::Binop, NodeType::Binop.to_string()))
    }
    fn parse_string_list(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::StringList,
            NodeType::StringList.to_string(),
        ))
    }
    fn parse_string(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::String,
            NodeType::String.to_string(),
        ))
    }
    fn parse_integer(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::Integer,
            NodeType::Integer.to_string(),
        ))
    }
    fn parse_identifier(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::Identifier,
            NodeType::Identifier.to_string(),
        ))
    }
    fn parse_email_identifier(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::EmailIdentifier,
            NodeType::EmailIdentifier.to_string(),
        ))
    }
}
