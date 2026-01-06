/// src/dsl/parser.rs
///
/// Parser for the DSL
///
/// Responsible for parsing the tokens into an AST
///
/// Contains:
/// --- ---
/// NodeType -> Node types for the AST
/// TreeNode -> Tree node struct
/// Ast -> AST struct
/// Parser -> Parser struct
///      Methods:
///      --- ---
///      new -> Create a new parser instance
///      get_completion_suggestions -> Get completion suggestions for the current input
///      parse -> Parse the tokens into an AST
///      --- ---
///--- ---
///
use crate::dsl::token::{Token, TokenType};
use crate::tui::errors::{make_user_friendly_for_completion, SyntaxError};
use std::vec;

/// Type alias for parser results
type ParseResult = Result<TreeNode, (SyntaxError, Vec<Token>)>;

/// Node types for the AST
///
/// Node types:
/// --- ---
/// ... (see below)
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for NodeType
/// Clone -> Clone trait for NodeType
/// PartialEq -> PartialEq trait for NodeType
/// Display -> Display trait for NodeType
/// --- ---
///
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

/// NodeType Display Trait Implementation
///
/// Parameters:
/// --- ---
/// self -> The NodeType to display
/// f -> The formatter to display the NodeType
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), std::fmt::Error> -> The result of the display
/// --- ---
///
impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::T(token_type) => write!(f, "{:?}", token_type),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// TreeNode for the AST
///
/// Fields:
/// --- ---
/// children -> The children of the TreeNode
/// node_type -> The type of the TreeNode
/// lexical_token -> The lexical token of the TreeNode
/// node_content -> The content of the TreeNode
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for TreeNode
/// Clone -> Clone trait for TreeNode
/// --- ---
///
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub children: Vec<TreeNode>,
    pub node_type: NodeType,
    pub lexical_token: Option<Token>,
    pub node_content: String,
}

/// TreeNode Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new TreeNode
/// --- ---
///
impl TreeNode {
    /// Create a new TreeNode
    ///
    /// Parameters:
    /// --- ---
    /// node_type -> The type of the TreeNode
    /// node_content -> The content of the TreeNode
    /// lexical_token -> The lexical token of the TreeNode
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// TreeNode -> The new TreeNode
    /// --- ---
    ///
    fn new(node_type: NodeType, node_content: String, lexical_token: Option<Token>) -> Self {
        TreeNode {
            children: Vec::new(),
            node_type,
            lexical_token,
            node_content,
        }
    }
}

/// AST for the DSL
///
/// Fields:
/// --- ---
/// head -> The head of the AST
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for Ast
/// Clone -> Clone trait for Ast
/// --- ---
///
#[derive(Debug, Clone)]
pub struct Ast {
    pub head: Option<TreeNode>,
}

/// Ast Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new Ast
/// --- ---
///
impl Ast {
    /// Create a new Ast
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Ast -> The new Ast
    /// --- ---
    ///
    fn new() -> Self {
        Ast { head: None }
    }
}

/// Parser for the DSL
///
/// Fields:
/// --- ---
/// input_string -> The input string to parse
/// ast -> The AST to parse
/// token_pointer -> The pointer to the current token
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// None -> No implemented traits
/// --- ---
///
pub struct Parser {
    input_string: String,
    ast: Ast,
    token_pointer: usize,
}

/// Parser Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new Parser
/// get_completion_suggestions -> Get completion suggestions for the current input
/// parse -> Parse the tokens into an AST
/// --- ---
///
impl Parser {
    /// Create a new Parser
    ///
    /// Parameters:
    /// --- ---
    /// input_string -> The input string to parse
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Parser -> The new Parser
    /// --- ---
    ///
    pub fn new(input_string: String) -> Self {
        Parser {
            input_string,
            ast: Ast::new(),
            token_pointer: 0,
        }
    }

    /// Get completion suggestions based on current parser state
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to get completion suggestions for
    /// tokens -> The tokens to get completion suggestions for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Vec<String> -> Vector of strings of completion suggestions
    /// --- ---
    ///
    pub fn get_completion_suggestions(&mut self, tokens: &Vec<Token>) -> Vec<String> {
        // reset parser state
        self.token_pointer = 0;
        self.ast = Ast::new();

        // try to parse and see where it fails
        match self.parse(tokens) {
            Ok(_) => {
                // parse succeeded - suggest logical operators or end query
                vec!["and".to_string(), "or".to_string()]
            }
            Err((error, _)) => {
                // parse failed - extract suggestions from error
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
                        // provide generic suggestions based on current context
                        self.get_context_suggestions(tokens)
                    }
                    SyntaxError::EmptyQuery => {
                        // empty query - suggest starting entities
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

    /// Get the lexeme of a token
    ///
    /// Parameters:
    /// --- ---
    /// self -> The Parser to get the lexeme for
    /// token -> The token to get the lexeme for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// &str -> The lexeme of the token
    /// --- ---
    ///
    fn get_lexeme(&self, token: &Token) -> &str {
        &self.input_string[token.get_start()..token.get_end()]
    }

    /// Get context-aware suggestions when we have a missing token
    ///
    /// Parameters:
    /// --- ---
    /// self -> The Parser to get context suggestions for
    /// tokens -> The tokens to get context suggestions for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Vec<String> -> Vector of strings of context suggestions
    /// --- ---
    ///
    fn get_context_suggestions(&self, tokens: &[Token]) -> Vec<String> {
        // Full condition operators from grammar:
        // <condition> ::= "=" | "!=" | "contains" | "has" | "starts with" | "ends with" | "is" | "equals" | "not equals" | "does not equal"
        let string_conditions = vec![
            "is".to_string(),
            "equals".to_string(),
            "=".to_string(),
            "!=".to_string(),
            "contains".to_string(),
            "has".to_string(),
            "starts with".to_string(),
            "ends with".to_string(),
            "does not equal".to_string(),
        ];

        // Binary operators for numeric comparisons from grammar:
        // <binop> ::= "=" | "!=" | "<" | ">" | "<=" | ">=" | "equals" | "is" | "less than" | "greater than" | "at least" | "at most" | "more than" | "fewer than"
        let numeric_binops = vec![
            "=".to_string(),
            "!=".to_string(),
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
            "is".to_string(),
            "equals".to_string(),
            "less than".to_string(),
            "greater than".to_string(),
            "at least".to_string(),
            "at most".to_string(),
            "more than".to_string(),
            "fewer than".to_string(),
        ];

        if tokens.is_empty() {
            // start of query
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
        } else {
            let last_token = &tokens[tokens.len() - 1];
            match *last_token.get_token_type() {
                // Entities followed by <condition>
                TokenType::Prof
                | TokenType::Subject
                | TokenType::Title
                | TokenType::Description
                | TokenType::Number
                | TokenType::Campus
                | TokenType::Method
                | TokenType::Full
                | TokenType::Type => string_conditions,

                // Days are followed by <condition>
                TokenType::Monday
                | TokenType::Tuesday
                | TokenType::Wednesday
                | TokenType::Thursday
                | TokenType::Friday
                | TokenType::Saturday
                | TokenType::Sunday => string_conditions,

                // Prereqs/Corereqs followed by <condition>
                TokenType::Prereqs | TokenType::Corereqs => string_conditions,

                // Course can be followed by condition OR sub-queries
                TokenType::Course => {
                    let mut suggestions = vec![
                        "subject".to_string(),
                        "number".to_string(),
                        "title".to_string(),
                        "description".to_string(),
                        "credit".to_string(),
                        "prereqs".to_string(),
                        "corereqs".to_string(),
                    ];
                    suggestions.extend(string_conditions);
                    suggestions
                }

                // Section can be followed by sub-queries
                TokenType::Section => vec![
                    "subject".to_string(),
                    "number".to_string(),
                    "enrollment".to_string(),
                    "cap".to_string(),
                    "method".to_string(),
                    "campus".to_string(),
                    "full".to_string(),
                ],

                // Credit must be followed by "hours"
                TokenType::Credit => vec!["hours".to_string()],

                // Credit hours followed by <binop>
                TokenType::Hours => numeric_binops.clone(),

                // Meeting must be followed by "type"
                TokenType::Meeting => vec!["type".to_string()],

                // Numeric entities followed by <binop>
                TokenType::Size | TokenType::Enrollment | TokenType::Cap => numeric_binops.clone(),

                // Time entities followed by <binop> or time value
                TokenType::Start | TokenType::End => numeric_binops,

                // After values, suggest logical operators
                TokenType::Identifier | TokenType::Alphanumeric | TokenType::String | TokenType::Integer | TokenType::Time => {
                    vec!["and".to_string(), "or".to_string()]
                }

                // After logical operators, suggest entities
                TokenType::And | TokenType::Or => vec![
                    "professor".to_string(),
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

                _ => vec![],
            }
        }
    }

    /// Check if a token is a valid binary operator
    ///
    /// Parameters:
    /// --- ---
    /// token_type -> The token type to check
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// bool -> True if the token is a valid binary operator, false otherwise
    /// --- ---
    ///
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

    /// Get the next token
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to get the next token for
    /// tokens -> The tokens to get the next token for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<Token, String> -> The next token
    /// --- ---
    ///
    fn next_token(&mut self, tokens: &[Token]) -> Result<Token, String> {
        if self.token_pointer < tokens.len() {
            let token = tokens[self.token_pointer];
            self.token_pointer += 1;
            Ok(token)
        } else {
            Err("Token stream empty, but attempted to take a token".into())
        }
    }

    /// Parse the tokens into an AST
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the tokens for
    /// tokens -> The tokens to parse
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<Ast, (SyntaxError, Vec<Token>)>
    ///     Ok(Ast) -> Parsing succeeded, contains the AST
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    pub fn parse(&mut self, tokens: &Vec<Token>) -> Result<Ast, (SyntaxError, Vec<Token>)> {
        self.token_pointer = 0;
        self.ast = Ast::new();

        // check for empty query
        if tokens.is_empty() {
            return Err((SyntaxError::EmptyQuery, vec![]));
        }

        // create query node and set it as AST head
        let query_node = self.parse_query(tokens)?;
        self.ast.head = Some(query_node);

        // check if there are remaining unconsumed tokens
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

    /// Parse the query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <query> ::= <logical_term> ("or" <logical_term>)*
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the query for
    /// tokens -> The tokens to parse the query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_query(&mut self, tokens: &Vec<Token>) -> ParseResult {
        // create query node first
        let mut query_node = TreeNode::new(NodeType::Query, NodeType::Query.to_string(), None);

        // parse the first logical term
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
                // put the token back by decrementing the pointer
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

        // add the final result to query node
        query_node.children.push(first_term);
        Ok(query_node)
    }

    /// Parse the logical term into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <logical_term> ::= <logical_factor> ("and" <logical_factor>)*
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the logical term for
    /// tokens -> The tokens to parse the logical term for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    fn parse_logical_term(
        &mut self,
        tokens: &Vec<Token>,
    ) -> ParseResult {
        let mut logical_term_node = TreeNode::new(
            NodeType::LogicalTerm,
            NodeType::LogicalTerm.to_string(),
            None,
        );

        // parse the first logical factor
        let mut first_factor = self.parse_logical_factor(tokens)?;

        // continue parsing logical factors until we hit the end of the tokens or we hit a non-AND token
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
                // put the token back by decrementing the pointer
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

    /// Parse the logical factor into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <logical_factor> ::= <entity_query> | "(" <query> ")" | "not" <logical_factor>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the logical factor for
    /// tokens -> The tokens to parse the logical factor for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_logical_factor(
        &mut self,
        tokens: &Vec<Token>,
    ) -> ParseResult {
        let mut logical_factor_node = TreeNode::new(
            NodeType::LogicalFactor,
            NodeType::LogicalFactor.to_string(),
            None,
        );

        // check if the first token is "not"
        if self.token_pointer < tokens.len()
            && *tokens[self.token_pointer].get_token_type() == TokenType::Not
        {
            // parse "not" <logical_factor>
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
        // check if the first token is a left parenthesis
        else if self.token_pointer < tokens.len()
            && *tokens[self.token_pointer].get_token_type() == TokenType::LeftParen
        {
            // parse parenthesized query: "(" <query> ")"
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
            // parse entity query
            let entity_query = self.parse_entity_query(tokens)?;
            logical_factor_node.children.push(entity_query);
        }

        Ok(logical_factor_node)
    }

    /// Parse the entity query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <entity_query> ::= <professor_query> | <course_query> | <section_query> | <meeting_type_query> | <time_query> | <day_query>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the entity query for
    /// tokens -> The tokens to parse the entity query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_entity_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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

    /// Parse the professor query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <professor_query> ::= "prof" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the professor query for
    /// tokens -> The tokens to parse the professor query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_professor_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let prof_token = tokens[self.token_pointer - 1];
        let mut prof_node = TreeNode::new(
            NodeType::ProfessorQuery,
            NodeType::ProfessorQuery.to_string(),
            Some(prof_token),
        );

        let condition = self.parse_condition(tokens)?;

        // If there's nothing after the condition keyword, give a user-friendly hint
        // instead of the generic "identifier" terminology.
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("professor name or email".into()),
                vec![],
            ));
        }

        let string = self.parse_string(tokens)?;

        prof_node.children.push(condition);
        prof_node.children.push(string);

        Ok(prof_node)
    }

    /// Parse the course query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <course_query> ::= "course" (<condition> <string> | <subject_query> | <number_query> | <title_query> | <description_query> | <credit_hours_query> | <prereqs_query> | <coreqs_query>)
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the course query for
    /// tokens -> The tokens to parse the course query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_course_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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
            // direct condition (like "course contains CS")
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

    /// Parse the subject query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <subject_query> ::= ("subject" | "sub") <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the subject query for
    /// tokens -> The tokens to parse the subject query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_subject_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let subject_token = tokens[self.token_pointer - 1];
        let mut subject_node = TreeNode::new(
            NodeType::SubjectQuery,
            NodeType::SubjectQuery.to_string(),
            Some(subject_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("subject code (e.g., 'CS', 'MATH', 'PHYS')".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        subject_node.children.push(condition_query);
        subject_node.children.push(string_query);

        Ok(subject_node)
    }

    /// Parse the section query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <section_query> ::= "section" <subject_query> | <course_query> | <enrollment_cap_query> | <instruction_method_query> | <campus_query> | <enrollment_query> | <full_query>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the section query for
    /// tokens -> The tokens to parse the section query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_section_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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

    /// Parse the number query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <number_query> ::= "number" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the number query for
    /// tokens -> The tokens to parse the number query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_number_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let number_token = tokens[self.token_pointer - 1];
        let mut number_node = TreeNode::new(
            NodeType::NumberQuery,
            NodeType::NumberQuery.to_string(),
            Some(number_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("course number (e.g., '101', '3500')".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        number_node.children.push(condition_query);
        number_node.children.push(string_query);

        Ok(number_node)
    }

    /// Parse the title query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <title_query> ::= "title" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the title query for
    /// tokens -> The tokens to parse the title query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_title_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let title_token = tokens[self.token_pointer - 1];
        let mut title_node = TreeNode::new(
            NodeType::TitleQuery,
            NodeType::TitleQuery.to_string(),
            Some(title_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("course title (e.g., 'Calculus', 'Introduction')".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        title_node.children.push(condition_query);
        title_node.children.push(string_query);

        Ok(title_node)
    }

    /// Parse the description query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <description_query> ::= "description" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the description query for
    /// tokens -> The tokens to parse the description query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_description_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let description_token = tokens[self.token_pointer - 1];
        let mut description_node = TreeNode::new(
            NodeType::DescriptionQuery,
            NodeType::DescriptionQuery.to_string(),
            Some(description_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("description text to search for".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        description_node.children.push(condition_query);
        description_node.children.push(string_query);

        Ok(description_node)
    }

    /// Parse the credit hours query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <credit_hours_query> ::= "credit hours" <binop> <integer>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the credit hours query for
    /// tokens -> The tokens to parse the credit hours query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_credit_hours_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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

        // Provide a user-friendly error message when number is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("number of credit hours (e.g., 3, 4)".into()),
                vec![],
            ));
        }

        let integer_query = self.parse_integer(tokens)?;

        credit_node.children.push(binop_query);
        credit_node.children.push(integer_query);

        Ok(credit_node)
    }

    /// Parse the prereqs query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <prereqs_query> ::= "prereqs" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the prereqs query for
    /// tokens -> The tokens to parse the prereqs query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_prereqs_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let prereqs_token = tokens[self.token_pointer - 1];
        let mut prereqs_node = TreeNode::new(
            NodeType::PrereqsQuery,
            NodeType::PrereqsQuery.to_string(),
            Some(prereqs_token),
        );

        let condition = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when prerequisite course is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("prerequisite course (e.g., 'CS101', 'MATH200')".into()),
                vec![],
            ));
        }

        let string = self.parse_string(tokens)?;

        prereqs_node.children.push(condition);
        prereqs_node.children.push(string);

        Ok(prereqs_node)
    }

    /// Parse the coreqs query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <coreqs_query> ::= "corereqs" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the coreqs query for
    /// tokens -> The tokens to parse the coreqs query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_coreqs_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let coreqs_token = tokens[self.token_pointer - 1];
        let mut coreqs_node = TreeNode::new(
            NodeType::CoreqsQuery,
            NodeType::CoreqsQuery.to_string(),
            Some(coreqs_token),
        );

        let condition = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when corequisite course is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("corequisite course (e.g., 'CS101L', 'PHYS201')".into()),
                vec![],
            ));
        }

        let string = self.parse_string(tokens)?;

        coreqs_node.children.push(condition);
        coreqs_node.children.push(string);

        Ok(coreqs_node)
    }

    /// Parse the enrollment cap query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <enrollment_cap_query> ::= "cap" <binop> <integer> | "enrollment cap" <binop> <integer>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the enrollment cap query for
    /// tokens -> The tokens to parse the enrollment cap query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_enrollment_cap_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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

        // Provide a user-friendly error message when number is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("maximum enrollment number (e.g., 30, 100)".into()),
                vec![],
            ));
        }

        let integer_query = self.parse_integer(tokens)?;

        cap_node.children.push(binop_query);
        cap_node.children.push(integer_query);

        Ok(cap_node)
    }

    /// Parse the instruction method query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <instruction_method_query> ::= "method" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the instruction method query for
    /// tokens -> The tokens to parse the instruction method query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_instruction_method_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let method_token = tokens[self.token_pointer - 1];
        let mut method_node = TreeNode::new(
            NodeType::InstructionMethodQuery,
            NodeType::InstructionMethodQuery.to_string(),
            Some(method_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("instruction method (e.g., 'online', 'in-person', 'hybrid')".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        method_node.children.push(condition_query);
        method_node.children.push(string_query);

        Ok(method_node)
    }

    /// Parse the campus query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <campus_query> ::= "campus" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the campus query for
    /// tokens -> The tokens to parse the campus query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_campus_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let campus_token = tokens[self.token_pointer - 1];
        let mut campus_node = TreeNode::new(
            NodeType::CampusQuery,
            NodeType::CampusQuery.to_string(),
            Some(campus_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("campus name (e.g., 'Main', 'Downtown')".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        campus_node.children.push(condition_query);
        campus_node.children.push(string_query);

        Ok(campus_node)
    }

    /// Parse the enrollment query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <enrollment_query> ::= "size" <binop> <integer> | "enrollment" <binop> <integer>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the enrollment query for
    /// tokens -> The tokens to parse the enrollment query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_enrollment_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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
                    "comparison like '>', '<', '=' followed by a number".into(),
                ),
                vec![],
            ));
        }

        let next_token = &tokens[self.token_pointer];
        if !Self::is_valid_binop_token(next_token.get_token_type()) {
            return Err((
                SyntaxError::MissingToken(
                    "comparison like '>', '<', '=' followed by a number".into(),
                ),
                vec![],
            ));
        }

        let binop_query = self.parse_binop(tokens)?;

        // Provide a user-friendly error message when number is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("enrollment count (e.g., 20, 50)".into()),
                vec![],
            ));
        }

        let integer_query = self.parse_integer(tokens)?;

        enrollment_node.children.push(binop_query);
        enrollment_node.children.push(integer_query);

        Ok(enrollment_node)
    }

    /// Parse the full query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <full_query> ::= "full" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the full query for
    /// tokens -> The tokens to parse the full query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_full_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let full_token = tokens[self.token_pointer - 1];
        let mut full_node = TreeNode::new(
            NodeType::FullQuery,
            NodeType::FullQuery.to_string(),
            Some(full_token),
        );

        let condition_query = self.parse_condition(tokens)?;

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("'true' or 'false'".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        full_node.children.push(condition_query);
        full_node.children.push(string_query);

        Ok(full_node)
    }

    /// Parse the meeting type query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <meeting_type_query> ::= ("meeting type" | "type") <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the meeting type query for
    /// tokens -> The tokens to parse the meeting type query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_meeting_type_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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

        // Provide a user-friendly error message when value is missing
        if self.token_pointer >= tokens.len() {
            return Err((
                SyntaxError::MissingToken("meeting type (e.g., 'lecture', 'lab', 'recitation')".into()),
                vec![],
            ));
        }

        let string_query = self.parse_string(tokens)?;

        meeting_node.children.push(condition_query);
        meeting_node.children.push(string_query);

        Ok(meeting_node)
    }

    /// Parse the time query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <time_query> ::= ("start" | "end") (<binop> <time> | <time_range>)
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the time query for
    /// tokens -> The tokens to parse the time query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_time_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
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
                            "comparison (like '>', '<', '=') and a time (e.g., '9:00am')".into(),
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
                SyntaxError::MissingToken("a time (e.g., '9:00am to 5:00pm' or '> 10:00am')".into()),
                vec![],
            ));
        }

        Ok(time_node)
    }

    /// Parse the time range into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <time_range> ::= <time> "to" <time>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the time range for
    /// tokens -> The tokens to parse the time range for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_time_range(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let start_time = self.parse_time(tokens)?;

        let to_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("'to' followed by end time (e.g., '9:00am to 5:00pm')".into()),
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

    /// Parse the day query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <day_query> ::= <monday_query> | <tuesday_query> | <wednesday_query> | <thursday_query> | <friday_query> | <saturday_query> | <sunday_query>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the day query for
    /// tokens -> The tokens to parse the day query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_day_query(&mut self, tokens: &[Token]) -> ParseResult {
        // the day token was already consumed in parse_entity_query, check which one it was
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

    /// Parse a day query with optional condition (defaults to "= true" if condition is missing)
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser
    /// tokens -> The tokens to parse
    /// day_name -> The name of the day (e.g., "monday")
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult -> The parsed day query node
    /// --- ---
    ///
    fn parse_day_query_helper(
        &mut self,
        tokens: &[Token],
        day_name: &str,
    ) -> ParseResult {
        let day_token = tokens[self.token_pointer - 1];
        let mut day_node = TreeNode::new(NodeType::String, day_name.to_string(), Some(day_token));

        // check if next token is a logical operator (and/or) or end of input
        // if so, default to "= true" for convenience
        let condition_query = if self.token_pointer < tokens.len() {
            let next_token = &tokens[self.token_pointer];
            match *next_token.get_token_type() {
                TokenType::And | TokenType::Or => {
                    // default to "= true" when followed by logical operator
                    let equals_token = Token::new(TokenType::Equals, 0, 0);
                    TreeNode::new(NodeType::Condition, "=".to_string(), Some(equals_token))
                }
                _ => self.parse_condition(tokens)?,
            }
        } else {
            // end of input, default to "= true"
            let equals_token = Token::new(TokenType::Equals, 0, 0);
            TreeNode::new(NodeType::Condition, "=".to_string(), Some(equals_token))
        };

        // for day queries, we typically expect a boolean-style value (like true/false)
        // after the condition. If we defaulted to "=", also default the value to "true"
        let string_query = if self.token_pointer < tokens.len() {
            let next_token = &tokens[self.token_pointer];
            match *next_token.get_token_type() {
                TokenType::And | TokenType::Or => {
                    // Default to "true" when followed by logical operator
                    let true_token = Token::new(TokenType::Identifier, 0, 0);
                    TreeNode::new(NodeType::Identifier, "true".to_string(), Some(true_token))
                }
                _ => self.parse_string(tokens)?,
            }
        } else {
            // end of input, default to "true"
            let true_token = Token::new(TokenType::Identifier, 0, 0);
            TreeNode::new(NodeType::Identifier, "true".to_string(), Some(true_token))
        };

        day_node.children.push(condition_query);
        day_node.children.push(string_query);

        Ok(day_node)
    }

    /// Parse the monday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <monday_query> ::= "monday" [<condition> <string>]
    ///                     If condition is omitted, defaults to "= true"
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the monday query for
    /// tokens -> The tokens to parse the monday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_monday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "monday")
    }

    /// Parse the tuesday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <tuesday_query> ::= "tuesday" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the tuesday query for
    /// tokens -> The tokens to parse the tuesday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_tuesday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "tuesday")
    }

    /// Parse the wednesday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <wednesday_query> ::= "wednesday" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the wednesday query for
    /// tokens -> The tokens to parse the wednesday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_wednesday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "wednesday")
    }

    /// Parse the thursday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <thursday_query> ::= "thursday" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the thursday query for
    /// tokens -> The tokens to parse the thursday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_thursday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "thursday")
    }

    /// Parse the friday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <friday_query> ::= "friday" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the friday query for
    /// tokens -> The tokens to parse the friday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_friday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "friday")
    }

    /// Parse the saturday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <saturday_query> ::= "saturday" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the saturday query for
    /// tokens -> The tokens to parse the saturday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_saturday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "saturday")
    }

    /// Parse the sunday query into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <sunday_query> ::= "sunday" <condition> <string>
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the sunday query for
    /// tokens -> The tokens to parse the sunday query for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_sunday_query(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        self.parse_day_query_helper(tokens, "sunday")
    }

    /// Parse the time into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <time> ::= [0-9]+:[0-9]+\s(?:am|pm)|[0-9]+:[0-9]+(?:am|pm)|[0-9]+:[0-9]+|[0-9]+\s(?:am|pm)|[0-9]+(?:am|pm)
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the time for
    /// tokens -> The tokens to parse the time for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_time(&mut self, tokens: &[Token]) -> ParseResult {
        let time_token = self
            .next_token(tokens)
            .map_err(|_| (SyntaxError::MissingToken("time (e.g., '9:00am', '2:30pm')".into()), vec![]))?;
        // Store the actual lexeme for better semantic checks and error messages
        let lexeme = self.get_lexeme(&time_token).to_string();
        let mut time_node = TreeNode::new(NodeType::Time, lexeme, Some(time_token));

        // for now, we'll assume any token can be a time
        // in a real implementation, you'd validate it matches the time regex pattern
        time_node.children.push(TreeNode::new(
            NodeType::String,
            time_token.get_token_type().to_string(),
            Some(time_token),
        ));

        Ok(time_node)
    }

    /// Parse the condition into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <condition> ::= "=" | "!=" | "contains" | "has" | "starts with" | "ends with" | "is" | "equals" | "not equals" | "does not equal"
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the condition for
    /// tokens -> The tokens to parse the condition for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_condition(&mut self, tokens: &[Token]) -> ParseResult {
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

        // check if it's a valid condition token
        match *condition_token.get_token_type() {
            TokenType::Equals
            | TokenType::NotEquals
            | TokenType::Contains
            | TokenType::Has
            | TokenType::Is
            | TokenType::Equal
            | TokenType::EqualsWord
            | TokenType::Does => {
                // valid standalone condition
            }
            TokenType::Starts => {
                // "STARTS" must be followed by "WITH"
                if self.token_pointer >= tokens.len()
                    || *tokens[self.token_pointer].get_token_type() != TokenType::With
                {
                    return Err((
                        SyntaxError::MissingToken("Expected 'with' after 'starts'".into()),
                        vec![],
                    ));
                }
                // consume the "with" token
                self.next_token(tokens).map_err(|_| {
                    (
                        SyntaxError::MissingToken("Expected 'with' after 'starts'".into()),
                        vec![],
                    )
                })?;
            }
            TokenType::Ends => {
                // "ENDS" must be followed by "WITH"
                if self.token_pointer >= tokens.len()
                    || *tokens[self.token_pointer].get_token_type() != TokenType::With
                {
                    return Err((
                        SyntaxError::MissingToken("Expected 'with' after 'ends'".into()),
                        vec![],
                    ));
                }
                // consume the "WITH" token
                self.next_token(tokens).map_err(|_| {
                    (
                        SyntaxError::MissingToken("Expected 'with' after 'ends'".into()),
                        vec![],
                    )
                })?;
            }
            TokenType::With => {
                // "WITH" by itself is not valid
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
                // check if it's a binary operator (invalid in condition context)
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

    /// Parse the binop into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <binop> ::= "=" | "!=" | "<" | ">" | "<=" | ">=" | "equals" | "is" | "not equals" | "not" | "does not equal" | "less than" | "greater than" | "less than or equal to" | "greater than or equal to" | "at least" | "at most" | "more than" | "fewer than"
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the binop for
    /// tokens -> The tokens to parse the binop for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_binop(&mut self, tokens: &[Token]) -> ParseResult {
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

        // check if it's a valid binary operator
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
                // valid operator
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

    /// Parse the string into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <string> ::= "[^"]*"?
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the string for
    /// tokens -> The tokens to parse the string for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_string(&mut self, tokens: &[Token]) -> ParseResult {
        // try to parse as email first, fallback to regular identifier
        if self.token_pointer < tokens.len() {
            let next_token = &tokens[self.token_pointer];
            if next_token.get_token_type().to_string().contains("@") {
                self.parse_email_identifier(tokens)
            } else {
                // Alphanumeric tokens (like "424N") should be parsed as identifiers
                self.parse_identifier(tokens)
            }
        } else {
            self.parse_identifier(tokens)
        }
    }

    /// Parse the integer into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <integer> ::= [0-9]+
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the integer for
    /// tokens -> The tokens to parse the integer for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_integer(&mut self, tokens: &[Token]) -> ParseResult {
        let digit_token = self
            .next_token(tokens)
            .map_err(|_| (SyntaxError::MissingToken("a number".into()), vec![]))?;
        // Store the actual lexeme for better semantic checks and error messages
        let lexeme = self.get_lexeme(&digit_token).to_string();
        let mut integer_node =
            TreeNode::new(NodeType::Integer, lexeme, Some(digit_token));

        // for now, we'll assume any token can be an integer
        // in a real implementation, you'd validate it's actually numeric
        integer_node.children.push(TreeNode::new(
            NodeType::String,
            digit_token.get_token_type().to_string(),
            Some(digit_token),
        ));

        Ok(integer_node)
    }

    /// Parse the identifier into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the identifier for
    /// tokens -> The tokens to parse the identifier for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_identifier(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let id_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("a value to search for".into()),
                vec![],
            )
        })?;
        // Store the actual identifier text so semantics can reason about values
        let lexeme = self.get_lexeme(&id_token).to_string();
        let mut identifier_node =
            TreeNode::new(NodeType::Identifier, lexeme, Some(id_token));

        // check if this is an operator token that shouldn't be here
        match *id_token.get_token_type() {
            // binary operators that are invalid in string context
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
            // string condition operators that are invalid here (double operators)
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
            // accept other tokens as valid identifiers
            _ => {
                // valid identifier
            }
        }

        identifier_node.children.push(TreeNode::new(
            NodeType::String,
            id_token.get_token_type().to_string(),
            Some(id_token),
        ));

        Ok(identifier_node)
    }

    /// Parse the email identifier into a TreeNode
    ///
    /// Syntax:
    /// --- ---
    /// <email_identifier> ::= [a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_]*\.[a-zA-Z0-9_.]*
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The Parser to parse the email identifier for
    /// tokens -> The tokens to parse the email identifier for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// ParseResult
    ///     Ok(TreeNode) -> Parsing succeeded, contains the TreeNode
    ///     Err((SyntaxError, Vec<Token>)) -> Parsing failed, contains the SyntaxError and the remaining tokens
    /// --- ---
    ///
    fn parse_email_identifier(
        &mut self,
        tokens: &[Token],
    ) -> ParseResult {
        let email_token = self.next_token(tokens).map_err(|_| {
            (
                SyntaxError::MissingToken("Expected email identifier".into()),
                vec![],
            )
        })?;
        // Store the actual email text for downstream consumers
        let lexeme = self.get_lexeme(&email_token).to_string();
        let mut email_node =
            TreeNode::new(NodeType::EmailIdentifier, lexeme, Some(email_token));

        // for now, we'll assume any token can be an email identifier
        // in a real implementation, you'd validate it matches the email pattern
        email_node.children.push(TreeNode::new(
            NodeType::String,
            email_token.get_token_type().to_string(),
            Some(email_token),
        ));

        Ok(email_node)
    }
}
