/// src/dsl/token.rs
///
/// Token types for the DSL
///
/// Responsible for defining the token types for the DSL
///
/// Contains:
/// --- ---
/// TokenType -> Token type enum
/// Token -> Token struct
///      Methods:
///      --- ---
///      new -> Create a new token instance
///      get_token_type -> Get the token type
///      get_start -> Get the start position of the token
///      get_end -> Get the end position of the token
///      --- ---
/// --- ---
///

/// Token types for the DSL
///
/// Token types:
/// --- ---
/// ... (see below)
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for TokenType
/// Clone -> Clone trait for TokenType
/// PartialEq -> PartialEq trait for TokenType
/// Copy -> Copy trait for TokenType
/// Display -> Display trait for TokenType
/// --- ---
///
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenType {
    // keywords
    Term,
    Prof,
    Course,
    Subject,
    Contains,
    Title,
    Method,
    Campus,
    Credit,
    Hours,
    Prereqs,
    Corereqs,
    Email,

    // new tokens for parser support
    Number,
    Description,
    Enrollment,
    Cap,
    Size,
    Instruction,
    Meeting,
    Type,
    Full,
    Start,
    End,

    // days
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,

    // operators
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,

    // logical
    And,
    Or,
    Not,

    // conditions
    Has,
    Is,
    Starts,
    With,
    Ends,
    Does,
    Equal,
    EqualsWord,
    DoesNotEqual,
    DoesNotContain,

    // binary operators
    Less,
    Than,
    Greater,
    At,
    Least,
    Most,
    More,
    Fewer,
    To,

    // grouping
    LeftParen,
    RightParen,

    // literals
    String,
    Alphanumeric,
    Integer,
    Time,
    Identifier,

    // special
    Exclamation,
    Unrecognized,
    UnclosedString,
}

/// TokenType Display Trait Implementation
///
/// Parameters:
/// --- ---
/// self -> The TokenType to display
/// f -> The formatter to display the TokenType
/// --- ---
///
/// Returns:
/// --- ---
/// std::fmt::Result -> The result of the display
/// --- ---
///
impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T_{}", format!("{:?}", self).to_uppercase())
    }
}

/// TokenType Implementation
///
/// Methods:
/// --- ---
/// all_patterns -> Get all token patterns in lexing order (longest/most specific first)
/// --- ---
///
impl TokenType {
    /// Get all token patterns in lexing order (longest/most specific first)
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Vec<(TokenType, &'static str)> -> All token patterns in lexing order (longest/most specific first)
    /// --- ---
    ///
    pub fn all_patterns() -> Vec<(TokenType, &'static str)> {
        vec![
            // multi-word operators - must come before individual words
            (
                TokenType::DoesNotEqual,
                r"(?i)\bdoes\s+not\s+equal\b|\bdoesn't\s+equal\b|\bdoesnt\s+equal\b",
            ),
            (
                TokenType::DoesNotContain,
                r"(?i)\bdoes\s+not\s+contain\b|\bdoesn't\s+contain\b|\bdoesnt\s+contain\b",
            ),
            (TokenType::EqualsWord, r"(?i)\bequals\b"),
            (TokenType::Starts, r"(?i)\bstarts\b"),
            (TokenType::With, r"(?i)\bwith\b"),
            (TokenType::Ends, r"(?i)\bends\b"),
            (TokenType::Does, r"(?i)\bdoes\b"),
            (TokenType::Equal, r"(?i)\bequal\b"),
            (TokenType::Less, r"(?i)\bless\b"),
            (TokenType::Than, r"(?i)\bthan\b"),
            (TokenType::Greater, r"(?i)\bgreater\b"),
            (TokenType::At, r"(?i)\bat\b"),
            (TokenType::Least, r"(?i)\bleast\b"),
            (TokenType::Most, r"(?i)\bmost\b"),
            (TokenType::More, r"(?i)\bmore\b"),
            (TokenType::Fewer, r"(?i)\bfewer\b"),
            (TokenType::To, r"(?i)\bto\b"),
            // days
            (
                TokenType::Wednesday,
                r"(?i)\b(wednesday|wednesda|wednesd|wednes|wedne|wedn|wed|we|w)\b",
            ),
            (
                TokenType::Thursday,
                r"(?i)\b(thursday|thursda|thurs|thur|thu|th)\b",
            ),
            (
                TokenType::Saturday,
                r"(?i)\b(saturday|saturda|saturd|satur|satu|sat|sa)\b",
            ),
            (
                TokenType::Tuesday,
                r"(?i)\b(tuesday|tuesda|tuesd|tues|tue|tu)\b",
            ),
            (TokenType::Monday, r"(?i)\b(monday|monda|mond|mon|mo|m)\b"),
            (TokenType::Friday, r"(?i)\b(friday|frida|frid|fri|fr|f)\b"),
            (TokenType::Sunday, r"(?i)\b(sunday|sunda|sund|sun|su)\b"),
            // keywords - these must come before the general identifier pattern
            (TokenType::Contains, r"(?i)\bcontains\b"),
            (TokenType::Prereqs, r"(?i)\b(?:prerequisites|prereqs)\b"),
            (TokenType::Corereqs, r"(?i)\b(?:corequisites|corereqs)\b"),
            (TokenType::Subject, r"(?i)\b(subject|sub)\b"),
            (TokenType::Course, r"(?i)\bcourse\b"),
            (TokenType::Method, r"(?i)\bmethod\b"),
            (TokenType::Campus, r"(?i)\bcampus\b"),
            (TokenType::Credit, r"(?i)\bcredit\b"),
            (TokenType::Hours, r"(?i)\bhours\b"),
            (TokenType::Title, r"(?i)\btitle\b"),
            (TokenType::Term, r"(?i)\bterm\b"),
            (TokenType::Prof, r"(?i)\b(?:prof|professor)\b"),
            (TokenType::Number, r"(?i)\bnumber\b"),
            (TokenType::Description, r"(?i)\bdescription\b"),
            (TokenType::Enrollment, r"(?i)\benrollment\b"),
            (TokenType::Cap, r"(?i)\bcap\b"),
            (TokenType::Size, r"(?i)\bsize\b"),
            (TokenType::Instruction, r"(?i)\binstruction\b"),
            (TokenType::Meeting, r"(?i)\bmeeting\b"),
            (TokenType::Type, r"(?i)\btype\b"),
            (TokenType::Full, r"(?i)\bfull\b"),
            (TokenType::Start, r"(?i)\bstart\b"),
            (TokenType::End, r"(?i)\bend\b"),
            (TokenType::Email, r"(?i)\bemail\b"),
            // logical
            (TokenType::And, r"(?i)\band\b"),
            (TokenType::Or, r"(?i)\bor\b"),
            (TokenType::Not, r"(?i)\bn't\b"),
            (TokenType::Not, r"(?i)\bnot\b"),
            // conditions
            (TokenType::Has, r"(?i)\bhas\b"),
            (TokenType::Is, r"(?i)\bis\b"),
            // operators
            (TokenType::NotEquals, r"!="),
            (TokenType::LessEqual, r"<="),
            (TokenType::GreaterEqual, r">="),
            (TokenType::Equals, r"="),
            (TokenType::LessThan, r"<"),
            (TokenType::GreaterThan, r">"),
            (TokenType::Exclamation, r"!"),
            (TokenType::LeftParen, r"\("),
            (TokenType::RightParen, r"\)"),
            // literals
            (TokenType::String, r#""[^"]*""#),
            (TokenType::UnclosedString, r#""[^"]*$"#),
            (
                TokenType::Time,
                r"[0-9]+:[0-9]+\s(?:am|pm)|[0-9]+:[0-9]+(?:am|pm)|[0-9]+:[0-9]+|[0-9]+\s(?:am|pm)|[0-9]+(?:am|pm)",
            ),
            // Alphanumeric course numbers (e.g., "424N", "101L") - must come before Integer
            (TokenType::Alphanumeric, r"[0-9]+[A-Za-z]+"),
            (TokenType::Integer, r"[0-9]+"),
            // general identifier pattern - must come last
            (TokenType::Identifier, r"[a-zA-Z_][a-zA-Z0-9_]*"),
            // unrecognized characters - must come last to catch anything else
            (TokenType::Unrecognized, r"[^\s]"),
        ]
    }
}

/// Token for the DSL
///
/// Fields:
/// --- ---
/// token_type -> The type of the token
/// start -> The start position of the token
/// end -> The end position of the token
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for Token
/// Clone -> Clone trait for Token
/// Copy -> Copy trait for Token
/// --- ---
///
#[derive(Debug, Clone, Copy)]
pub struct Token {
    token_type: TokenType,
    start: usize,
    end: usize,
}

/// Token Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new token instance
/// get_token_type -> Get the token type
/// get_start -> Get the start position of the token
/// get_end -> Get the end position of the token
/// --- ---
///
impl Token {
    /// Create a new token instance
    ///
    /// Parameters:
    /// --- ---
    /// token_type -> The type of the token
    /// start -> The start position of the token
    /// end -> The end position of the token
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Token -> The new token instance
    /// --- ---
    ///
    pub fn new(token_type: TokenType, start: usize, end: usize) -> Self {
        Token {
            token_type,
            start,
            end,
        }
    }

    /// Get the token type
    ///
    /// Parameters:
    /// --- ---
    /// self -> The Token to get the token type for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// &TokenType -> The token type
    /// --- ---
    ///
    pub fn get_token_type(&self) -> &TokenType {
        &self.token_type
    }

    /// Get the start position of the token
    ///
    /// Parameters:
    /// --- ---
    /// self -> The Token to get the start position for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// usize -> The start position of the token
    /// --- ---
    ///
    pub fn get_start(&self) -> usize {
        self.start
    }

    /// Get the end position of the token
    ///
    /// Parameters:
    /// --- ---
    /// self -> The Token to get the end position for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// usize -> The end position of the token
    /// --- ---
    ///
    pub fn get_end(&self) -> usize {
        self.end
    }
}
