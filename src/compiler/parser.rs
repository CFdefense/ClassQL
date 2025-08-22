/*
    This module is responsible for parsing the tokens into an AST

    The AST is a tree of nodes that represent the query

    The AST can then be used by the semantic analyzer to check for semantic errors
*/

use crate::compiler::token::{Token, TokenType};
use crate::tui::errors::{make_user_friendly_for_completion, SyntaxError};
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

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::T(token_type) => write!(f, "{:?}", token_type),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub head: Option<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub children: Vec<TreeNode>,
    pub node_type: NodeType,
    pub lexical_token: Option<Token>,
    pub node_content: String,
}

impl TreeNode {
    fn new(node_type: NodeType, node_content: String, lexical_token: Option<Token>) -> Self {
        TreeNode {
            children: Vec::new(),
            node_type,
            lexical_token,
            node_content,
        }
    }
}

impl Ast {
    fn new() -> Self {
        Ast { head: None }
    }
}

pub struct Parser {
    input_string: String,
    ast: Ast,
    token_pointer: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

impl Parser {
    pub fn new(input_string: String) -> Self {
        Parser {
            input_string,
            ast: Ast::new(),
            token_pointer: 0,
        }
    }

    /// Get completion suggestions based on current parser state
    pub fn get_completion_suggestions(&mut self, tokens: &Vec<Token>) -> Vec<String> {
        // Reset parser state
        self.token_pointer = 0;
        self.ast = Ast::new();

        // Try to parse and see where it fails
        match self.parse(tokens) {
            Ok(_) => {
                // Parse succeeded - suggest logical operators or end query
                vec!["and".to_string(), "or".to_string()]
            }
            Err((error, _)) => {
                // Parse failed - extract suggestions from error
                match error {
                    SyntaxError::ExpectedAfter { expected, .. } => expected
                        .iter()
                        .map(|s| make_user_friendly_for_completion(s))
                        .collect(),
                    SyntaxError::InvalidContext { suggestions, .. } => suggestions
                        .iter()
                        .map(|s| make_user_friendly_for_completion(s))
                        .collect(),
                    SyntaxError::MissingToken(_) => {
                        // Provide generic suggestions based on current context
                        self.get_context_suggestions(tokens)
                    }
                    SyntaxError::EmptyQuery => {
                        // Empty query - suggest starting entities
                        vec![
                            "professor".to_string(),
                            "course".to_string(),
                            "subject".to_string(),
                            "title".to_string(),
                            "section".to_string(),
                            "number".to_string(),
                            "description".to_string(),
                            "credit".to_string(),
                            "prerequisites".to_string(),
                            "corequisites".to_string(),
                            "enrollment".to_string(),
                            "campus".to_string(),
                            "meeting".to_string(),
                        ]
                    }
                    _ => vec![],
                }
            }
        }
    }

    fn get_lexeme(&self, token: &Token) -> &str {
        &self.input_string[token.get_start()..token.get_end()]
    }

    /// Get context-aware suggestions when we have a missing token
    fn get_context_suggestions(&self, tokens: &[Token]) -> Vec<String> {
        if tokens.is_empty() {
            // Start of query
            vec![
                "professor".to_string(),
                "course".to_string(),
                "subject".to_string(),
                "title".to_string(),
                "section".to_string(),
            ]
        } else {
            let last_token = &tokens[tokens.len() - 1];
            match *last_token.get_token_type() {
                TokenType::Prof => vec![
                    "is".to_string(),
                    "equals".to_string(),
                    "contains".to_string(),
                ],
                TokenType::Course => vec![
                    "subject".to_string(),
                    "number".to_string(),
                    "title".to_string(),
                ],
                TokenType::Credit => vec!["hours".to_string()],
                TokenType::Meeting => vec!["type".to_string()],
                TokenType::Size | TokenType::Enrollment => {
                    vec!["=".to_string(), ">".to_string(), "<".to_string()]
                }
                TokenType::Start | TokenType::End => {
                    vec!["=".to_string(), ">".to_string(), "<".to_string()]
                }
                // String condition operators
                TokenType::Is | TokenType::Equals | TokenType::Contains => {
                    vec!["<value>".to_string()] // Placeholder for user input
                }
                // After values
                TokenType::Identifier | TokenType::String => {
                    vec!["and".to_string(), "or".to_string()]
                }
                _ => vec![],
            }
        }
    }

    fn is_valid_binop_token(token_type: &TokenType) -> bool {
        matches!(
            *token_type,
            TokenType::Equals
                | TokenType::NotEquals
                | TokenType::LessThan
                | TokenType::GreaterThan
                | TokenType::LessEqual
                | TokenType::GreaterEqual
                | TokenType::Equal
                | TokenType::EqualsWord
                | TokenType::Is
                | TokenType::Not
                | TokenType::Does
                | TokenType::Less
                | TokenType::Than
                | TokenType::Greater
                | TokenType::Least
                | TokenType::Most
                | TokenType::More
                | TokenType::Fewer
        )
    }

    fn next_token(&mut self, tokens: &[Token]) -> Result<Token, String> {
        if self.token_pointer < tokens.len() {
            let token = tokens[self.token_pointer];
            self.token_pointer += 1;
            Ok(token)
        } else {
            Err("Token stream empty, but attempted to take a token".into())
        }
    }

    pub fn parse(&mut self, tokens: &Vec<Token>) -> Result<Ast, (SyntaxError, Vec<Token>)> {
        self.token_pointer = 0;
        self.ast = Ast::new();

        // Check for empty query
        if tokens.is_empty() {
            return Err((SyntaxError::EmptyQuery, vec![]));
        }

        // Create Query node and set it as AST head
        let query_node = self.parse_query(tokens)?;
        self.ast.head = Some(query_node);

        // Check if there are remaining unconsumed tokens
        if self.token_pointer < tokens.len() {
            return Err((
                SyntaxError::InvalidContext {
                    token: format!(
                        "{} ('{}')",
                        tokens[self.token_pointer].get_token_type(),
                        self.get_lexeme(&tokens[self.token_pointer])
                    ),
                    context: "end of query".to_string(),
                    suggestions: vec![
                        "and".to_string(),
                        "or".to_string(),
                        "remove extra text".to_string(),
                    ],
                },
                vec![tokens[self.token_pointer]],
            ));
        }

        Ok(Ast {
            head: self.ast.head.take(),
        })
    }

    // <query> ::= <logical_term> ("or" <logical_term>)*
    fn parse_query(&mut self, tokens: &Vec<Token>) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // Create Query node first
        let mut query_node = TreeNode::new(NodeType::Query, NodeType::Query.to_string(), None);

        // Parse the first logical term
        let mut first_term = self.parse_logical_term(tokens)?;

        // continue parsing logical terms until we hit the end of the tokens or we hit a non-or token
        while self.token_pointer < tokens.len() {
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
                Some(next_token),
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
        let mut logical_term_node = TreeNode::new(
            NodeType::LogicalTerm,
            NodeType::LogicalTerm.to_string(),
            None,
        );

        // parse the first logical factor
        let mut first_factor = self.parse_logical_factor(tokens)?;

        // continue parsing logical factors until we hit the end of the tokens or we hit a non-and token
        while self.token_pointer < tokens.len() {
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
                Some(next_token),
            );
            and_node.children.push(first_factor);
            and_node.children.push(next_factor);
            first_factor = and_node;
        }

        logical_term_node.children.push(first_factor);
        Ok(logical_term_node)
    }

    // <logical_factor> ::= <entity_query> | "(" <query> ")" | "not" <logical_factor>
    fn parse_logical_factor(
        &mut self,
        tokens: &Vec<Token>,
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut logical_factor_node = TreeNode::new(
            NodeType::LogicalFactor,
            NodeType::LogicalFactor.to_string(),
            None,
        );

        // Check if the first token is "not"
        if self.token_pointer < tokens.len()
            && *tokens[self.token_pointer].get_token_type() == TokenType::Not
        {
            // Parse "not" <logical_factor>
            let not_token = self.next_token(tokens).map_err(|_| {
                (
                    SyntaxError::MissingToken("Expected 'not' token".into()),
                    vec![],
                )
            })?;

            let factor = self.parse_logical_factor(tokens)?;

            let mut not_node = TreeNode::new(
                NodeType::T(TokenType::Not),
                NodeType::T(TokenType::Not).to_string(),
                Some(not_token),
            );
            not_node.children.push(factor);
            logical_factor_node.children.push(not_node);
        }
        // Check if the first token is a left parenthesis
        else if self.token_pointer < tokens.len()
            && *tokens[self.token_pointer].get_token_type() == TokenType::LeftParen
        {
            // Parse parenthesized query: "(" <query> ")"
            let left_paren = self.next_token(tokens).map_err(|_| {
                (
                    SyntaxError::MissingToken("Expected opening parenthesis".into()),
                    vec![],
                )
            })?;

            let query = self.parse_query(tokens)?;

            let right_paren = self.next_token(tokens).map_err(|_| {
                (
                    SyntaxError::MissingToken("Expected closing parenthesis".into()),
                    vec![],
                )
            })?;

            if *right_paren.get_token_type() != TokenType::RightParen {
                return Err((SyntaxError::UnclosedParenthesis, vec![left_paren]));
            }

            logical_factor_node.children.push(query);
        } else {
            // Parse entity query
            let entity_query = self.parse_entity_query(tokens)?;
            logical_factor_node.children.push(entity_query);
        }

        Ok(logical_factor_node)
    }

    // <entity_query> ::= <professor_query> | <course_query> | <section_query> | <meeting_type_query> | <time_query> | <day_query>
    fn parse_entity_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut entity_query = TreeNode::new(
            NodeType::EntityQuery,
            NodeType::EntityQuery.to_string(),
            None,
        );

        // match next keyword
        let next_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::ExpectedAfter {
                    expected: vec![
                        "prof".to_string(),
                        "course".to_string(),
                        "subject".to_string(),
                        "title".to_string(),
                        "section".to_string(),
                    ],
                    after: "start of query".to_string(),
                    position: self.token_pointer,
                },
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
            TokenType::Enrollment => {
                // Look ahead to see if this is "enrollment cap"
                if self.token_pointer < tokens.len()
                    && *tokens[self.token_pointer].get_token_type() == TokenType::Cap
                {
                    self.parse_enrollment_cap_query(tokens)?
                } else {
                    self.parse_enrollment_query(tokens)?
                }
            }
            TokenType::Size => self.parse_enrollment_query(tokens)?,
            TokenType::Cap => self.parse_enrollment_cap_query(tokens)?,
            TokenType::Full => self.parse_full_query(tokens)?,
            TokenType::Method => self.parse_instruction_method_query(tokens)?,
            TokenType::Campus => self.parse_campus_query(tokens)?,
            TokenType::Meeting => {
                // Check if next token is "type" for "meeting type" compound
                if self.token_pointer < tokens.len()
                    && *tokens[self.token_pointer].get_token_type() == TokenType::Type
                {
                    // Consume the "type" token
                    self.next_token(tokens).map_err(|_| {
                        (
                            SyntaxError::ExpectedAfter {
                                expected: vec!["type".to_string()],
                                after: "meeting".to_string(),
                                position: self.token_pointer,
                            },
                            vec![],
                        )
                    })?;
                    self.parse_meeting_type_query(tokens)?
                } else {
                    // "meeting" must be followed by "type" according to grammar
                    return Err((
                        SyntaxError::ExpectedAfter {
                            expected: vec!["type".to_string()],
                            after: "meeting".to_string(),
                            position: self.token_pointer,
                        },
                        vec![],
                    ));
                }
            }
            TokenType::Type => self.parse_meeting_type_query(tokens)?,
            TokenType::Time => self.parse_time_query(tokens)?,
            TokenType::Start | TokenType::End => self.parse_time_query(tokens)?,
            TokenType::Monday
            | TokenType::Tuesday
            | TokenType::Wednesday
            | TokenType::Thursday
            | TokenType::Friday
            | TokenType::Saturday
            | TokenType::Sunday => self.parse_day_query(tokens)?,
            _ => {
                return Err((
                    SyntaxError::InvalidContext {
                        token: format!(
                            "{} ('{}')",
                            next_token.get_token_type(),
                            self.get_lexeme(&next_token)
                        ),
                        context: "query start".to_string(),
                        suggestions: vec![
                            "prof".to_string(),
                            "course".to_string(),
                            "subject".to_string(),
                            "title".to_string(),
                            "section".to_string(),
                            "number".to_string(),
                            "description".to_string(),
                            "credit".to_string(),
                            "prereqs".to_string(),
                            "corereqs".to_string(),
                            "enrollment".to_string(),
                            "campus".to_string(),
                            "meeting".to_string(),
                        ],
                    },
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
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let prof_token = tokens[self.token_pointer - 1];
        let mut prof_node = TreeNode::new(
            NodeType::ProfessorQuery,
            NodeType::ProfessorQuery.to_string(),
            Some(prof_token),
        );

        let condition = self.parse_condition(tokens)?;
        let string = self.parse_string(tokens)?;

        prof_node.children.push(condition);
        prof_node.children.push(string);

        Ok(prof_node)
    }

    // <course_query> ::= "course" (<condition> <string> | <subject_query> | <number_query> | <title_query> | <description_query> | <credit_hours_query> | <prereqs_query> | <coreqs_query>)
    fn parse_course_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let course_token = tokens[self.token_pointer - 1];
        let mut course_node = TreeNode::new(
            NodeType::CourseQuery,
            NodeType::CourseQuery.to_string(),
            Some(course_token),
        );

        // Check what the next token is to decide between direct condition or sub-query
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::ExpectedAfter {
                    expected: vec![
                        "subject".to_string(),
                        "number".to_string(),
                        "title".to_string(),
                        "description".to_string(),
                        "credit".to_string(),
                        "is".to_string(),
                        "contains".to_string(),
                        "equals".to_string(),
                    ],
                    after: "course".to_string(),
                    position: self.token_pointer,
                },
                vec![],
            ));
        }

        let next_token = &tokens[self.token_pointer];
        let next_query = match *next_token.get_token_type() {
            // Sub-queries
            TokenType::Subject => {
                self.token_pointer += 1; // consume the subject token
                self.parse_subject_query(tokens)?
            }
            TokenType::Number => {
                self.token_pointer += 1; // consume the number token
                self.parse_number_query(tokens)?
            }
            TokenType::Title => {
                self.token_pointer += 1; // consume the title token
                self.parse_title_query(tokens)?
            }
            TokenType::Description => {
                self.token_pointer += 1; // consume the description token
                self.parse_description_query(tokens)?
            }
            TokenType::Credit => {
                self.token_pointer += 1; // consume the credit token
                self.parse_credit_hours_query(tokens)?
            }
            TokenType::Prereqs => {
                self.token_pointer += 1; // consume the prereqs token
                self.parse_prereqs_query(tokens)?
            }
            TokenType::Corereqs => {
                self.token_pointer += 1; // consume the corereqs token
                self.parse_coreqs_query(tokens)?
            }
            // Direct condition (like "course contains CS")
            TokenType::Equals
            | TokenType::NotEquals
            | TokenType::Contains
            | TokenType::Has
            | TokenType::Starts
            | TokenType::With
            | TokenType::Ends
            | TokenType::Is
            | TokenType::Equal
            | TokenType::EqualsWord
            | TokenType::Does => {
                // Parse as direct condition + string for course number/code
                let condition = self.parse_condition(tokens)?;
                let string = self.parse_string(tokens)?;

                let mut number_node = TreeNode::new(
                    NodeType::NumberQuery,
                    NodeType::NumberQuery.to_string(),
                    None,
                );
                number_node.children.push(condition);
                number_node.children.push(string);
                number_node
            }
            _ => {
                // Check if it's a binary operator (invalid for course conditions)
                if Self::is_valid_binop_token(next_token.get_token_type()) {
                    return Err((
                        SyntaxError::InvalidContext {
                            token: self.get_lexeme(next_token).to_string(),
                            context: "after 'course'".to_string(),
                            suggestions: vec![
                                "subject".to_string(),
                                "number".to_string(),
                                "title".to_string(),
                                "description".to_string(),
                                "credit".to_string(),
                                "is".to_string(),
                                "contains".to_string(),
                                "equals".to_string(),
                            ],
                        },
                        vec![*next_token],
                    ));
                } else {
                    return Err((
                        SyntaxError::InvalidContext {
                            token: format!(
                                "{} ('{}')",
                                next_token.get_token_type(),
                                self.get_lexeme(next_token)
                            ),
                            context: "after 'course'".to_string(),
                            suggestions: vec![
                                "subject".to_string(),
                                "number".to_string(),
                                "title".to_string(),
                                "description".to_string(),
                                "credit".to_string(),
                                "is".to_string(),
                                "contains".to_string(),
                                "equals".to_string(),
                            ],
                        },
                        vec![*next_token],
                    ));
                }
            }
        };

        course_node.children.push(next_query);
        Ok(course_node)
    }

    // <subject_query> ::= ("subject" | "sub") <condition> <string>
    fn parse_subject_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let subject_token = tokens[self.token_pointer - 1];
        let mut subject_node = TreeNode::new(
            NodeType::SubjectQuery,
            NodeType::SubjectQuery.to_string(),
            Some(subject_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        subject_node.children.push(condition_query);
        subject_node.children.push(string_query);

        Ok(subject_node)
    }

    // <section_query> ::= "section" <subject_query> | <course_query> | <enrollment_cap_query> | <instruction_method_query> | <campus_query> | <enrollment_query> | <full_query>
    fn parse_section_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let section_token = tokens[self.token_pointer - 1];
        let mut section_node = TreeNode::new(
            NodeType::SectionQuery,
            NodeType::SectionQuery.to_string(),
            Some(section_token),
        );

        let next_query = match *tokens[self.token_pointer].get_token_type() {
            TokenType::Subject => self.parse_subject_query(tokens)?,
            TokenType::Course => self.parse_course_query(tokens)?,
            TokenType::Enrollment => self.parse_enrollment_query(tokens)?,
            TokenType::Instruction => self.parse_instruction_method_query(tokens)?,
            TokenType::Campus => self.parse_campus_query(tokens)?,
            TokenType::Size => self.parse_enrollment_query(tokens)?,
            TokenType::Cap => self.parse_enrollment_cap_query(tokens)?,
            TokenType::Full => self.parse_full_query(tokens)?,
            _ => {
                return Err((
                    SyntaxError::InvalidContext {
                        token: format!(
                            "{} ('{}')",
                            tokens[self.token_pointer].get_token_type(),
                            self.get_lexeme(&tokens[self.token_pointer])
                        ),
                        context: "after 'section'".to_string(),
                        suggestions: vec![
                            "subject".to_string(),
                            "course".to_string(),
                            "enrollment".to_string(),
                            "instruction".to_string(),
                            "campus".to_string(),
                            "size".to_string(),
                            "cap".to_string(),
                            "full".to_string(),
                        ],
                    },
                    vec![tokens[self.token_pointer]],
                ))
            }
        };

        section_node.children.push(next_query);
        Ok(section_node)
    }

    // <number_query> ::= "number" <condition> <string>
    fn parse_number_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let number_token = tokens[self.token_pointer - 1];
        let mut number_node = TreeNode::new(
            NodeType::NumberQuery,
            NodeType::NumberQuery.to_string(),
            Some(number_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        number_node.children.push(condition_query);
        number_node.children.push(string_query);

        Ok(number_node)
    }

    // <title_query> ::= "title" <condition> <string>
    fn parse_title_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let title_token = tokens[self.token_pointer - 1];
        let mut title_node = TreeNode::new(
            NodeType::TitleQuery,
            NodeType::TitleQuery.to_string(),
            Some(title_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        title_node.children.push(condition_query);
        title_node.children.push(string_query);

        Ok(title_node)
    }

    // <description_query> ::= "description" <condition> <string>
    fn parse_description_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let description_token = tokens[self.token_pointer - 1];
        let mut description_node = TreeNode::new(
            NodeType::DescriptionQuery,
            NodeType::DescriptionQuery.to_string(),
            Some(description_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        description_node.children.push(condition_query);
        description_node.children.push(string_query);

        Ok(description_node)
    }

    // <credit_hours_query> ::= "credit hours" <binop> <integer>
    fn parse_credit_hours_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let credit_token = tokens[self.token_pointer - 1];
        let mut credit_node = TreeNode::new(
            NodeType::CreditHoursQuery,
            NodeType::CreditHoursQuery.to_string(),
            Some(credit_token),
        );

        // The "credit" token was already consumed, now consume "hours"
        let hours_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::ExpectedAfter {
                    expected: vec!["hours".to_string()],
                    after: "credit".to_string(),
                    position: self.token_pointer,
                },
                vec![],
            )
        })?;

        if *hours_token.get_token_type() != TokenType::Hours {
            return Err((
                SyntaxError::ExpectedAfter {
                    expected: vec!["hours".to_string()],
                    after: "credit".to_string(),
                    position: self.token_pointer,
                },
                vec![hours_token],
            ));
        }

        let binop_query = self.parse_binop(tokens)?;
        let integer_query = self.parse_integer(tokens)?;

        credit_node.children.push(binop_query);
        credit_node.children.push(integer_query);

        Ok(credit_node)
    }

    // <prereqs_query> ::= "prereqs" <string_list>
    fn parse_prereqs_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let prereqs_token = tokens[self.token_pointer - 1];
        let mut prereqs_node = TreeNode::new(
            NodeType::PrereqsQuery,
            NodeType::PrereqsQuery.to_string(),
            Some(prereqs_token),
        );

        let string_list_query = self.parse_string_list(tokens)?;
        prereqs_node.children.push(string_list_query);

        Ok(prereqs_node)
    }

    // <coreqs_query> ::= "corereqs" <string_list>
    fn parse_coreqs_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let coreqs_token = tokens[self.token_pointer - 1];
        let mut coreqs_node = TreeNode::new(
            NodeType::CoreqsQuery,
            NodeType::CoreqsQuery.to_string(),
            Some(coreqs_token),
        );

        let string_list_query = self.parse_string_list(tokens)?;
        coreqs_node.children.push(string_list_query);

        Ok(coreqs_node)
    }

    // <enrollment_cap_query> ::= "cap" <binop> <integer> | "enrollment cap" <binop> <integer>
    fn parse_enrollment_cap_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let cap_token = tokens[self.token_pointer - 1];
        let mut cap_node = TreeNode::new(
            NodeType::EnrollmentCapQuery,
            NodeType::EnrollmentCapQuery.to_string(),
            Some(cap_token),
        );

        // Check if we need to consume a "cap" token (for cases like "enrollment cap" or standalone "cap")
        if self.token_pointer < tokens.len() {
            let next_token = &tokens[self.token_pointer];
            if *next_token.get_token_type() == TokenType::Cap {
                // Consume the "cap" token
                self.next_token(tokens).map_err(|_| {
                    (
                        SyntaxError::MissingToken("Expected cap token".into()),
                        vec![],
                    )
                })?;
            }
        }

        let binop_query = self.parse_binop(tokens)?;
        let integer_query = self.parse_integer(tokens)?;

        cap_node.children.push(binop_query);
        cap_node.children.push(integer_query);

        Ok(cap_node)
    }

    // <instruction_method_query> ::= "method" <condition> <string>
    fn parse_instruction_method_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let method_token = tokens[self.token_pointer - 1];
        let mut method_node = TreeNode::new(
            NodeType::InstructionMethodQuery,
            NodeType::InstructionMethodQuery.to_string(),
            Some(method_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        method_node.children.push(condition_query);
        method_node.children.push(string_query);

        Ok(method_node)
    }

    // <campus_query> ::= "campus" <condition> <string>
    fn parse_campus_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let campus_token = tokens[self.token_pointer - 1];
        let mut campus_node = TreeNode::new(
            NodeType::CampusQuery,
            NodeType::CampusQuery.to_string(),
            Some(campus_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        campus_node.children.push(condition_query);
        campus_node.children.push(string_query);

        Ok(campus_node)
    }

    // <enrollment_query> ::= "size" <binop> <integer> | "enrollment" <binop> <integer>
    fn parse_enrollment_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let enrollment_token = tokens[self.token_pointer - 1];
        let mut enrollment_node = TreeNode::new(
            NodeType::EnrollmentQuery,
            NodeType::EnrollmentQuery.to_string(),
            Some(enrollment_token),
        );

        // Check if next token is a valid binary operator
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken(
                    "Expected comparison operator after size/enrollment".into(),
                ),
                vec![],
            ));
        }

        let next_token = &tokens[self.token_pointer];
        if !Self::is_valid_binop_token(next_token.get_token_type()) {
            return Err((
                SyntaxError::MissingToken(
                    "Expected comparison operator after size/enrollment".into(),
                ),
                vec![],
            ));
        }

        let binop_query = self.parse_binop(tokens)?;
        let integer_query = self.parse_integer(tokens)?;

        enrollment_node.children.push(binop_query);
        enrollment_node.children.push(integer_query);

        Ok(enrollment_node)
    }

    // <full_query> ::= "full" <condition> <string>
    fn parse_full_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let full_token = tokens[self.token_pointer - 1];
        let mut full_node = TreeNode::new(
            NodeType::FullQuery,
            NodeType::FullQuery.to_string(),
            Some(full_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        full_node.children.push(condition_query);
        full_node.children.push(string_query);

        Ok(full_node)
    }

    // <meeting_type_query> ::= ("meeting type" | "type") <condition> <string>
    fn parse_meeting_type_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let main_token = if self.token_pointer > 1
            && *tokens[self.token_pointer - 2].get_token_type() == TokenType::Meeting
        {
            tokens[self.token_pointer - 2]
        } else {
            tokens[self.token_pointer - 1]
        };
        let mut meeting_node = TreeNode::new(
            NodeType::MeetingTypeQuery,
            NodeType::MeetingTypeQuery.to_string(),
            Some(main_token),
        );

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        meeting_node.children.push(condition_query);
        meeting_node.children.push(string_query);

        Ok(meeting_node)
    }

    // <time_query> ::= ("start" | "end") (<binop> <time> | <time_range>)
    fn parse_time_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let time_type_token = &tokens[self.token_pointer - 1];
        let mut time_node = TreeNode::new(
            NodeType::TimeQuery,
            NodeType::TimeQuery.to_string(),
            Some(*time_type_token),
        );

        // The start/end token was already consumed in parse_entity_query, but we need to check which one it was
        // let time_type_token = &tokens[self.token_pointer - 1]; // Get the already consumed token

        // Validate it's a start or end token
        if *time_type_token.get_token_type() != TokenType::Start
            && *time_type_token.get_token_type() != TokenType::End
        {
            return Err((
                SyntaxError::InvalidContext {
                    token: format!(
                        "{} ('{}')",
                        time_type_token.get_token_type(),
                        self.get_lexeme(time_type_token)
                    ),
                    context: "time query".to_string(),
                    suggestions: vec!["start".to_string(), "end".to_string()],
                },
                vec![*time_type_token],
            ));
        }

        // Add time type to node
        time_node.children.push(TreeNode::new(
            NodeType::String,
            time_type_token.get_token_type().to_string(),
            Some(*time_type_token),
        ));

        // Check if this is a time range by looking ahead
        if self.token_pointer < tokens.len() {
            // If next token looks like a time, check if the one after that is "to"
            if self.token_pointer + 1 < tokens.len()
                && *tokens[self.token_pointer + 1].get_token_type() == TokenType::To
            {
                // Parse: <time_range> (start 9:00 to 17:00)
                let time_range_spec = self.parse_time_range(tokens)?;
                time_node.children.push(time_range_spec);
            } else {
                // Parse: <binop> <time> (start > 9:00)
                // Check if next token is a valid binary operator
                let next_token = &tokens[self.token_pointer];
                if !Self::is_valid_binop_token(next_token.get_token_type()) {
                    return Err((
                        SyntaxError::MissingToken(
                            "Expected comparison operator after start/end".into(),
                        ),
                        vec![],
                    ));
                }
                let binop_spec = self.parse_binop(tokens)?;
                let time_spec = self.parse_time(tokens)?;
                time_node.children.push(binop_spec);
                time_node.children.push(time_spec);
            }
        } else {
            return Err((
                SyntaxError::MissingToken("Expected operator or time after start/end".into()),
                vec![],
            ));
        }

        Ok(time_node)
    }

    // <time_range> ::= <time> "to" <time>
    fn parse_time_range(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let start_time = self.parse_time(tokens)?;

        let to_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected 'to' separator".into()),
                vec![],
            )
        })?;

        if *to_token.get_token_type() != TokenType::To {
            return Err((
                SyntaxError::ExpectedAfter {
                    expected: vec!["to".to_string()],
                    after: "start time".to_string(),
                    position: self.token_pointer,
                },
                vec![to_token],
            ));
        }

        let mut time_range_node = TreeNode::new(
            NodeType::TimeRange,
            NodeType::TimeRange.to_string(),
            Some(to_token),
        );

        let end_time = self.parse_time(tokens)?;

        time_range_node.children.push(start_time);
        time_range_node.children.push(end_time);

        Ok(time_range_node)
    }

    // <day_query> ::= <monday_query> | <tuesday_query> | <wednesday_query> | <thursday_query> | <friday_query> | <saturday_query> | <sunday_query>
    fn parse_day_query(&mut self, tokens: &[Token]) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // The day token was already consumed in parse_entity_query, check which one it was
        let day_token = &tokens[self.token_pointer - 1];
        let mut day_node = TreeNode::new(
            NodeType::DayQuery,
            NodeType::DayQuery.to_string(),
            Some(*day_token),
        );

        let day_query = match *day_token.get_token_type() {
            TokenType::Monday => self.parse_monday_query(tokens)?,
            TokenType::Tuesday => self.parse_tuesday_query(tokens)?,
            TokenType::Wednesday => self.parse_wednesday_query(tokens)?,
            TokenType::Thursday => self.parse_thursday_query(tokens)?,
            TokenType::Friday => self.parse_friday_query(tokens)?,
            TokenType::Saturday => self.parse_saturday_query(tokens)?,
            TokenType::Sunday => self.parse_sunday_query(tokens)?,
            _ => {
                return Err((
                    SyntaxError::InvalidContext {
                        token: format!(
                            "{} ('{}')",
                            day_token.get_token_type(),
                            self.get_lexeme(day_token)
                        ),
                        context: "day name".to_string(),
                        suggestions: vec![
                            "monday".to_string(),
                            "tuesday".to_string(),
                            "wednesday".to_string(),
                            "thursday".to_string(),
                            "friday".to_string(),
                            "saturday".to_string(),
                            "sunday".to_string(),
                        ],
                    },
                    vec![*day_token],
                ));
            }
        };

        day_node.children.push(day_query);
        Ok(day_node)
    }

    // Helper functions for individual day queries
    fn parse_monday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut monday_node =
            TreeNode::new(NodeType::String, "monday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        monday_node.children.push(condition_query);
        monday_node.children.push(string_query);

        Ok(monday_node)
    }

    fn parse_tuesday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut tuesday_node =
            TreeNode::new(NodeType::String, "tuesday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        tuesday_node.children.push(condition_query);
        tuesday_node.children.push(string_query);

        Ok(tuesday_node)
    }

    fn parse_wednesday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut wednesday_node =
            TreeNode::new(NodeType::String, "wednesday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        wednesday_node.children.push(condition_query);
        wednesday_node.children.push(string_query);

        Ok(wednesday_node)
    }

    fn parse_thursday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut thursday_node =
            TreeNode::new(NodeType::String, "thursday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        thursday_node.children.push(condition_query);
        thursday_node.children.push(string_query);

        Ok(thursday_node)
    }

    fn parse_friday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut friday_node =
            TreeNode::new(NodeType::String, "friday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        friday_node.children.push(condition_query);
        friday_node.children.push(string_query);

        Ok(friday_node)
    }

    fn parse_saturday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut saturday_node =
            TreeNode::new(NodeType::String, "saturday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        saturday_node.children.push(condition_query);
        saturday_node.children.push(string_query);

        Ok(saturday_node)
    }

    fn parse_sunday_query(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let day_token = tokens[self.token_pointer - 1];
        let mut sunday_node =
            TreeNode::new(NodeType::String, "sunday".to_string(), Some(day_token));

        let condition_query = self.parse_condition(tokens)?;
        let string_query = self.parse_string(tokens)?;

        sunday_node.children.push(condition_query);
        sunday_node.children.push(string_query);

        Ok(sunday_node)
    }

    // <time> ::= [0-9]+:[0-9]+\s(?:am|pm)|[0-9]+:[0-9]+(?:am|pm)|[0-9]+:[0-9]+|[0-9]+\s(?:am|pm)|[0-9]+(?:am|pm)
    fn parse_time(&mut self, tokens: &[Token]) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let time_token = self
            .next_token(tokens)
            .map_err(|_| (SyntaxError::MissingToken("Expected time".into()), vec![]))?;
        let mut time_node =
            TreeNode::new(NodeType::Time, NodeType::Time.to_string(), Some(time_token));

        // For now, we'll assume any token can be a time
        // In a real implementation, you'd validate it matches the time regex pattern
        time_node.children.push(TreeNode::new(
            NodeType::String,
            time_token.get_token_type().to_string(),
            Some(time_token),
        ));

        Ok(time_node)
    }

    // <condition> ::= "=" | "!=" | "contains" | "has" | "starts with" | "ends with" | "is" | "equals" | "not equals" | "does not equal"
    fn parse_condition(&mut self, tokens: &[Token]) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let condition_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::ExpectedAfter {
                    expected: vec![
                        "is".to_string(),
                        "equals".to_string(),
                        "contains".to_string(),
                        "has".to_string(),
                        "starts".to_string(),
                        "ends".to_string(),
                        "=".to_string(),
                        "!=".to_string(),
                    ],
                    after: "entity keyword".to_string(),
                    position: self.token_pointer,
                },
                vec![],
            )
        })?;
        let mut condition_node = TreeNode::new(
            NodeType::Condition,
            NodeType::Condition.to_string(),
            Some(condition_token),
        );

        // Check if it's a valid condition token
        match *condition_token.get_token_type() {
            TokenType::Equals
            | TokenType::NotEquals
            | TokenType::Contains
            | TokenType::Has
            | TokenType::Is
            | TokenType::Equal
            | TokenType::EqualsWord
            | TokenType::Does => {
                // Valid standalone condition
            }
            TokenType::Starts => {
                // "starts" must be followed by "with"
                if self.token_pointer >= tokens.len()
                    || *tokens[self.token_pointer].get_token_type() != TokenType::With
                {
                    return Err((
                        SyntaxError::MissingToken("Expected 'with' after 'starts'".into()),
                        vec![],
                    ));
                }
                // Consume the "with" token
                self.next_token(tokens).map_err(|_| {
                    (
                        SyntaxError::MissingToken("Expected 'with' after 'starts'".into()),
                        vec![],
                    )
                })?;
            }
            TokenType::Ends => {
                // "ends" must be followed by "with"
                if self.token_pointer >= tokens.len()
                    || *tokens[self.token_pointer].get_token_type() != TokenType::With
                {
                    return Err((
                        SyntaxError::MissingToken("Expected 'with' after 'ends'".into()),
                        vec![],
                    ));
                }
                // Consume the "with" token
                self.next_token(tokens).map_err(|_| {
                    (
                        SyntaxError::MissingToken("Expected 'with' after 'ends'".into()),
                        vec![],
                    )
                })?;
            }
            TokenType::With => {
                // "with" by itself is not valid
                return Err((
                    SyntaxError::InvalidContext {
                        token: self.get_lexeme(&condition_token).to_string(),
                        context: "string condition".to_string(),
                        suggestions: vec![
                            "is".to_string(),
                            "equals".to_string(),
                            "contains".to_string(),
                            "has".to_string(),
                            "starts with".to_string(),
                            "ends with".to_string(),
                        ],
                    },
                    vec![condition_token],
                ));
            }
            _ => {
                // Check if it's a binary operator (invalid in condition context)
                if Self::is_valid_binop_token(condition_token.get_token_type()) {
                    return Err((
                        SyntaxError::InvalidContext {
                            token: self.get_lexeme(&condition_token).to_string(),
                            context: "string condition".to_string(),
                            suggestions: vec![
                                "is".to_string(),
                                "equals".to_string(),
                                "contains".to_string(),
                                "has".to_string(),
                                "starts".to_string(),
                                "ends".to_string(),
                            ],
                        },
                        vec![condition_token],
                    ));
                } else {
                    return Err((
                        SyntaxError::InvalidContext {
                            token: format!(
                                "{} ('{}')",
                                condition_token.get_token_type(),
                                self.get_lexeme(&condition_token)
                            ),
                            context: "string condition".to_string(),
                            suggestions: vec![
                                "is".to_string(),
                                "equals".to_string(),
                                "contains".to_string(),
                                "has".to_string(),
                                "starts".to_string(),
                                "ends".to_string(),
                            ],
                        },
                        vec![condition_token],
                    ));
                }
            }
        }

        condition_node.children.push(TreeNode::new(
            NodeType::String,
            condition_token.get_token_type().to_string(),
            Some(condition_token),
        ));

        Ok(condition_node)
    }

    // <binop> ::= "=" | "!=" | "<" | ">" | "<=" | ">=" | "equals" | "is" | "not equals" | "not" | "does not equal" | "less than" | "greater than" | "less than or equal to" | "greater than or equal to" | "at least" | "at most" | "more than" | "fewer than"
    fn parse_binop(&mut self, tokens: &[Token]) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let operator_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::ExpectedAfter {
                    expected: vec![
                        "=".to_string(),
                        "!=".to_string(),
                        "<".to_string(),
                        ">".to_string(),
                        "<=".to_string(),
                        ">=".to_string(),
                        "equals".to_string(),
                        "is".to_string(),
                        "less than".to_string(),
                        "greater than".to_string(),
                    ],
                    after: "numeric field".to_string(),
                    position: self.token_pointer,
                },
                vec![],
            )
        })?;
        let mut binop_node = TreeNode::new(
            NodeType::Binop,
            NodeType::Binop.to_string(),
            Some(operator_token),
        );

        // Check if it's a valid binary operator
        match *operator_token.get_token_type() {
            TokenType::Equals
            | TokenType::NotEquals
            | TokenType::LessThan
            | TokenType::GreaterThan
            | TokenType::LessEqual
            | TokenType::GreaterEqual
            | TokenType::Equal
            | TokenType::EqualsWord
            | TokenType::Is
            | TokenType::Not
            | TokenType::Does
            | TokenType::Less
            | TokenType::Than
            | TokenType::Greater
            | TokenType::Least
            | TokenType::Most
            | TokenType::More
            | TokenType::Fewer => {
                // Valid operator
            }
            _ => {
                return Err((
                    SyntaxError::InvalidContext {
                        token: operator_token.get_token_type().to_string(),
                        context: "numeric comparison".to_string(),
                        suggestions: vec![
                            "=".to_string(),
                            "!=".to_string(),
                            "<".to_string(),
                            ">".to_string(),
                            "<=".to_string(),
                            ">=".to_string(),
                            "equals".to_string(),
                            "is".to_string(),
                            "less than".to_string(),
                            "greater than".to_string(),
                        ],
                    },
                    vec![operator_token],
                ));
            }
        }

        binop_node.children.push(TreeNode::new(
            NodeType::String,
            operator_token.get_token_type().to_string(),
            Some(operator_token),
        ));

        Ok(binop_node)
    }

    // <string_list> ::= <string> | <string_list> "," <string>
    fn parse_string_list(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let mut string_list_node =
            TreeNode::new(NodeType::StringList, NodeType::StringList.to_string(), None);

        // Parse first string
        let first_string = self.parse_string(tokens)?;
        string_list_node.children.push(first_string);

        // Check if there are more strings separated by commas
        while self.token_pointer < tokens.len() {
            let next_token = &tokens[self.token_pointer];

            // If we hit a comma, parse the next string
            if next_token.get_token_type().to_string() == "comma"
                || next_token.get_token_type().to_string().contains(",")
            {
                self.token_pointer += 1; // consume comma
                let next_string = self.parse_string(tokens)?;
                string_list_node.children.push(next_string);
            } else {
                // No more strings in the list
                break;
            }
        }

        Ok(string_list_node)
    }

    // <string> ::= "[^"]*"?
    fn parse_string(&mut self, tokens: &[Token]) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        // Try to parse as email first, fallback to regular identifier
        if self.token_pointer < tokens.len() {
            let next_token = &tokens[self.token_pointer];
            if next_token.get_token_type().to_string().contains("@") {
                self.parse_email_identifier(tokens)
            } else {
                self.parse_identifier(tokens)
            }
        } else {
            self.parse_identifier(tokens)
        }
    }

    // <integer> ::= [0-9]+
    fn parse_integer(&mut self, tokens: &[Token]) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let digit_token = self
            .next_token(tokens)
            .map_err(|_| (SyntaxError::MissingToken("Expected number".into()), vec![]))?;
        let mut integer_node = TreeNode::new(
            NodeType::Integer,
            NodeType::Integer.to_string(),
            Some(digit_token),
        );

        // For now, we'll assume any token can be an integer
        // In a real implementation, you'd validate it's actually numeric
        integer_node.children.push(TreeNode::new(
            NodeType::String,
            digit_token.get_token_type().to_string(),
            Some(digit_token),
        ));

        Ok(integer_node)
    }

    // <identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*
    fn parse_identifier(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let id_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected identifier".into()),
                vec![],
            )
        })?;
        let mut identifier_node = TreeNode::new(
            NodeType::Identifier,
            NodeType::Identifier.to_string(),
            Some(id_token),
        );

        // Check if this is an operator token that shouldn't be here
        match *id_token.get_token_type() {
            // Binary operators that are invalid in string context
            TokenType::Equals
            | TokenType::NotEquals
            | TokenType::LessThan
            | TokenType::GreaterThan
            | TokenType::LessEqual
            | TokenType::GreaterEqual => {
                return Err((
                    SyntaxError::InvalidContext {
                        token: self.get_lexeme(&id_token).to_string(),
                        context: "after string condition".to_string(),
                        suggestions: vec![
                            "text value".to_string(),
                            "quoted string".to_string(),
                            "identifier".to_string(),
                        ],
                    },
                    vec![id_token],
                ));
            }
            // String condition operators that are invalid here (double operators)
            TokenType::Contains
            | TokenType::Has
            | TokenType::Starts
            | TokenType::Ends
            | TokenType::Is
            | TokenType::Equal
            | TokenType::EqualsWord
            | TokenType::Does => {
                return Err((
                    SyntaxError::InvalidContext {
                        token: self.get_lexeme(&id_token).to_string(),
                        context: "after string condition".to_string(),
                        suggestions: vec![
                            "text value".to_string(),
                            "remove duplicate operator".to_string(),
                        ],
                    },
                    vec![id_token],
                ));
            }
            // Accept other tokens as valid identifiers
            _ => {
                // Valid identifier
            }
        }

        identifier_node.children.push(TreeNode::new(
            NodeType::String,
            id_token.get_token_type().to_string(),
            Some(id_token),
        ));

        Ok(identifier_node)
    }

    // <email_identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_]*\.[a-zA-Z0-9_.]*
    fn parse_email_identifier(
        &mut self,
        tokens: &[Token],
    ) -> Result<TreeNode, (SyntaxError, Vec<Token>)> {
        let email_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected email identifier".into()),
                vec![],
            )
        })?;
        let mut email_node = TreeNode::new(
            NodeType::EmailIdentifier,
            NodeType::EmailIdentifier.to_string(),
            Some(email_token),
        );

        // For now, we'll assume any token can be an email identifier
        // In a real implementation, you'd validate it matches the email pattern
        email_node.children.push(TreeNode::new(
            NodeType::String,
            email_token.get_token_type().to_string(),
            Some(email_token),
        ));

        Ok(email_node)
    }
}

