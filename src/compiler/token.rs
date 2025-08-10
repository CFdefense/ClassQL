#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
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
    
    // Days
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    
    // Operators
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    
    // Logical
    And,
    Or,
    Not,
    
    // Conditions
    Has,
    Is,
    Starts,
    With,
    Ends,
    Does,
    Equal,
    EqualsWord,
    
    // Binary operators
    Less,
    Than,
    Greater,
    At,
    Least,
    Most,
    More,
    Fewer,
    
    // Grouping
    LeftParen,
    RightParen,
    
    // Literals
    String,
    Integer,
    Time,
    Identifier,
    
    // Special
    Exclamation,
    Unrecognized,
}

impl TokenType {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    start: i32,
    end: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, start: i32, end: i32) -> Self {
        Token {
            token_type,
            lexeme,
            start,
            end,
        }
    }


    #[allow(dead_code)]
    pub fn get_lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn get_token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn get_start(&self) -> i32 {
        self.start
    }

    pub fn get_end(&self) -> i32 {
        self.end
    }
} 

impl TokenType {
    // Get all token patterns in lexing order (longest/most specific first)
    pub fn all_patterns() -> Vec<(TokenType, &'static str)> {
        vec![
            // Email-like identifiers
            (TokenType::Identifier, r"[a-zA-Z_][a-zA-Z0-9_]*@[a-zA-Z0-9_]*\.[a-zA-Z0-9_.]*"),
            
            // Multi-word operators
            (TokenType::EqualsWord, r"\bequals\b"),
            (TokenType::Starts, r"\bstarts\b"),
            (TokenType::With, r"\bwith\b"),
            (TokenType::Ends, r"\bends\b"),
            (TokenType::Does, r"\bdoes\b"),
            (TokenType::Equal, r"\bequal\b"),
            (TokenType::Less, r"\bless\b"),
            (TokenType::Than, r"\bthan\b"),
            (TokenType::Greater, r"\bgreater\b"),
            (TokenType::At, r"\bat\b"),
            (TokenType::Least, r"\bleast\b"),
            (TokenType::Most, r"\bmost\b"),
            (TokenType::More, r"\bmore\b"),
            (TokenType::Fewer, r"\bfewer\b"),
            
            // Keywords
            (TokenType::Contains, r"\bcontains\b"),
            (TokenType::Prereqs, r"\bprereqs\b"),
            (TokenType::Corereqs, r"\bcorereqs\b"),
            (TokenType::Subject, r"\b(subject|sub)\b"),
            (TokenType::Course, r"\bcourse\b"),
            (TokenType::Method, r"\bmethod\b"),
            (TokenType::Campus, r"\bcampus\b"),
            (TokenType::Credit, r"\bcredit\b"),
            (TokenType::Hours, r"\bhours\b"),
            (TokenType::Title, r"\btitle\b"),
            (TokenType::Term, r"\bterm\b"),
            (TokenType::Prof, r"\bprof\b"),
            
            // Days 
            (TokenType::Wednesday, r"\b(wednesday|wednesda|wednesd|wednes|wedne|wedn|wed|we|w)\b"),
            (TokenType::Thursday, r"\b(thursday|thursda|thurs|thur|thu|th)\b"),
            (TokenType::Saturday, r"\b(saturday|saturda|saturd|satur|satu|sat|sa)\b"),
            (TokenType::Tuesday, r"\b(tuesday|tuesda|tuesd|tues|tue|tu)\b"),
            (TokenType::Monday, r"\b(monday|monda|mond|mon|mo|m)\b"),
            (TokenType::Friday, r"\b(friday|frida|frid|fri|fr|f)\b"),
            (TokenType::Sunday, r"\b(sunday|sunda|sund|sun|su)\b"),
            
            // Operators
            (TokenType::NotEquals, r"!="),
            (TokenType::LessEqual, r"<="),
            (TokenType::GreaterEqual, r">="),
            (TokenType::Equals, r"="),
            (TokenType::LessThan, r"<"),
            (TokenType::GreaterThan, r">"),
            (TokenType::Exclamation, r"!"),
            (TokenType::LeftParen, r"\("),
            (TokenType::RightParen, r"\)"),
            
            // Logical
            (TokenType::And, r"\band\b"),
            (TokenType::Or, r"\bor\b"),
            (TokenType::Not, r"\bnot\b"),
            
            // Conditions
            (TokenType::Has, r"\bhas\b"),
            (TokenType::Is, r"\bis\b"),
            
            // Literals
            (TokenType::String, r#""[^"]*"?"#),
            (TokenType::Time, r"[0-9]+:[0-9]+\s(?:am|pm)|[0-9]+:[0-9]+(?:am|pm)|[0-9]+:[0-9]+|[0-9]+\s(?:am|pm)|[0-9]+(?:am|pm)"),
            (TokenType::Integer, r"[0-9]+"),
            (TokenType::Identifier, r"[a-zA-Z_][a-zA-Z0-9_]*"),
        ]
    }
} 