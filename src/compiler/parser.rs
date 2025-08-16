/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use ratatui::widgets::block::title;
use serde::de::IntoDeserializer;

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
        };

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
        };

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

    // <subject_query> ::= ("subject" | "sub") <condition> <string>
    fn parse_subject_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut subject_node =
            TreeNode::new(NodeType::SubjectQuery, NodeType::SubjectQuery.to_string());

        let subject_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected subject token".into()),
                vec![],
            )
        })?;

        if *subject_token.get_token_type() != TokenType::Subject {
            return Err((
                SyntaxError::UnexpectedToken(subject_token.get_token_type().to_string()),
                vec![subject_token],
            ));
        };

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        subject_node.children.push(condition_query);
        subject_node.children.push(string_query);

        Ok(subject_node)
    }

    // <section_query> ::= "section" <subject_query> | <course_query> | <enrollment_cap_query> | <instruction_method_query> | <campus_query> | <enrollment_query> | <full_query>
    fn parse_section_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut section_node =
            TreeNode::new(NodeType::SectionQuery, NodeType::SectionQuery.to_string());

        let section_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected section token".into()),
                vec![],
            )
        })?;

        if *section_token.get_token_type() != TokenType::Section {
            return Err((
                SyntaxError::UnexpectedToken(section_token.get_token_type().to_string()),
                vec![section_token],
            ));
        };

        let next_query = match *tokens[self.token_pointer].get_token_type() {
            TokenType::Subject => self.parse_subject_query(tokens)?,
            TokenType::Course => self.parse_course_query(tokens)?,
            TokenType::Enrollment => self.parse_enrollment_query(tokens)?,
            TokenType::Instruction => self.parse_instruction_method_query(tokens)?,
            TokenType::Campus => self.parse_campus_query(tokens)?,
            TokenType::Not => self.parse_enrollment_cap_query(tokens)?, // ! add SIZE HERE
            TokenType::Full => self.parse_full_query(tokens)?,
            _ => {
                return Err((
                    SyntaxError::UnexpectedToken(
                        tokens[self.token_pointer].get_token_type().to_string(),
                    ),
                    vec![tokens[self.token_pointer].clone()],
                ))
            }
        };

        section_node.children.push(next_query);
        Ok(section_node)
    }

    // <number_query> ::= "number" <condition> <string>
    fn parse_number_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut number_node =
            TreeNode::new(NodeType::NumberQuery, NodeType::NumberQuery.to_string());

        let number_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected number token".into()),
                vec![],
            )
        })?;

        if *number_token.get_token_type() == TokenType::Number {
            return Err((
                SyntaxError::UnexpectedToken(number_token.get_token_type().to_string()),
                vec![number_token],
            ));
        };

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        number_node.children.push(condition_query);
        number_node.children.push(string_query);

        Ok(number_node)
    }

    // <title_query> ::= "title" <condition> <string>
    fn parse_title_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut title_node = TreeNode::new(NodeType::TitleQuery, NodeType::TitleQuery.to_string());

        let title_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected title token".into()),
                vec![],
            )
        })?;

        if *title_token.get_token_type() != TokenType::Title {
            return Err((
                SyntaxError::UnexpectedToken(title_token.get_token_type().to_string()),
                vec![title_token],
            ));
        }

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        title_node.children.push(condition_query);
        title_node.children.push(string_query);

        Ok(title_node)
    }

    // <description_query> ::= "description" <condition> <string>
    fn parse_description_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut description_node = TreeNode::new(
            NodeType::DescriptionQuery,
            NodeType::DescriptionQuery.to_string(),
        );

        let description_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected description token".into()),
                vec![],
            )
        })?;

        if *description_token.get_token_type() != TokenType::Description {
            return Err((
                SyntaxError::UnexpectedToken(description_token.get_token_type().to_string()),
                vec![description_token],
            ));
        }

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        description_node.children.push(condition_query);
        description_node.children.push(string_query);

        Ok(description_node)
    }

    // <credit_hours_query> ::= "credit hours" <binop> <integer>
    fn parse_credit_hours_query(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut credit_node = TreeNode::new(
            NodeType::CreditHoursQuery,
            NodeType::CreditHoursQuery.to_string(),
        );

        let credit_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected credit hours token".into()),
                vec![],
            )
        })?;

        if *credit_token.get_token_type() != TokenType::Credit {
            return Err((
                SyntaxError::UnexpectedToken(credit_token.get_token_type().to_string()),
                vec![credit_token],
            ));
        }

        let binop_query = self.parse_binop(tokens)?;
        let integer_query = self.parse_integer(tokens)?;

        credit_node.children.push(binop_query);
        credit_node.children.push(integer_query);

        Ok(credit_node)
    }

    // <prereqs_query> ::= "prereqs" <string> | "prereqs" <string_list>
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

    // <coreqs_query> ::= "corereqs" <string> | "corereqs" <string_list>
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

    // <enrollment_cap_query> ::= "cap" <binop> <integer>
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

    // <instruction_method_query> ::= "method" <condition> <string>
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

    // <campus_query> ::= "campus" <condition> <string>
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

    // <enrollment_query> ::= "size" <binop> <integer>
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

    // <full_query> ::= <condition> "full"
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

    // <meeting_type_query> ::= ("meeting type" | "type") <condition> <string>
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

    // <time_query> ::= ("start" | "end") ((<binop> <time>) | (("from" | "in") | ("not in") <time_range>))
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

    // <time_range> ::= <time> to <time>
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

    // <day_query> ::= <monday_query> | <tuesday_query> | <wednesday_query> | <thursday_query> | <friday_query> | <saturday_query> | <sunday_query>
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

    // <time> ::= [0-9]+:[0-9]+\s(?:am|pm)|[0-9]+:[0-9]+(?:am|pm)|[0-9]+:[0-9]+|[0-9]+\s(?:am|pm)|[0-9]+(?:am|pm)
    fn parse_time(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(NodeType::Time, NodeType::Time.to_string()))
    }

    // <condition> ::= "=" | "!=" | "contains" | "has" | "starts with" | "ends with" | "is" | "equals" | "not equals" | "does not equal"
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

    // <binop> ::= "=" | "!=" | "<" | ">" | "<=" | ">=" | "equals" | "is" | "not equals" | "not" | "does not equal" | "less than" | "greater than" | "less than or equal to" | "greater than or equal to" | "at least" | "at most" | "more than" | "fewer than"
    fn parse_binop(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(NodeType::Binop, NodeType::Binop.to_string()))
    }

    // <string_list> ::= <string> | <string_list> "," <string>
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

    // <string> ::= "[^"]*"?
    fn parse_string(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // TODO: Implement actual parsing logic
        Ok(TreeNode::new(
            NodeType::String,
            NodeType::String.to_string(),
        ))
    }

    // <integer> ::= [0-9]+
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

    // <identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*
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

    // <email_identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_]*\.[a-zA-Z0-9_.]*
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
