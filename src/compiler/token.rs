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
    Email,
    
    // New tokens for parser support
    Section,
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
    To,
    
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
        format!("T_{}", format!("{:?}", self).to_uppercase())
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
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
            // Multi-word operators
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
            
            // Days 
            (TokenType::Wednesday, r"(?i)\b(wednesday|wednesda|wednesd|wednes|wedne|wedn|wed|we|w)\b"),
            (TokenType::Thursday, r"(?i)\b(thursday|thursda|thurs|thur|thu|th)\b"),
            (TokenType::Saturday, r"(?i)\b(saturday|saturda|saturd|satur|satu|sat|sa)\b"),
            (TokenType::Tuesday, r"(?i)\b(tuesday|tuesda|tuesd|tues|tue|tu)\b"),
            (TokenType::Monday, r"(?i)\b(monday|monda|mond|mon|mo|m)\b"),
            (TokenType::Friday, r"(?i)\b(friday|frida|frid|fri|fr|f)\b"),
            (TokenType::Sunday, r"(?i)\b(sunday|sunda|sund|sun|su)\b"),
            
            // Keywords - these must come before the general identifier pattern
            (TokenType::Contains, r"(?i)\bcontains\b"),
            (TokenType::Prereqs, r"(?i)\bprereqs\b"),
            (TokenType::Corereqs, r"(?i)\bcorereqs\b"),
            (TokenType::Subject, r"(?i)\b(subject|sub)\b"),
            (TokenType::Course, r"(?i)\bcourse\b"),
            (TokenType::Method, r"(?i)\bmethod\b"),
            (TokenType::Campus, r"(?i)\bcampus\b"),
            (TokenType::Credit, r"(?i)\bcredit\b"),
            (TokenType::Hours, r"(?i)\bhours\b"),
            (TokenType::Title, r"(?i)\btitle\b"),
            (TokenType::Term, r"(?i)\bterm\b"),
            (TokenType::Prof, r"(?i)\bprof\b"),
            (TokenType::Section, r"(?i)\bsection\b"),
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
            
            // Logical
            (TokenType::And, r"(?i)\band\b"),
            (TokenType::Or, r"(?i)\bor\b"),
            (TokenType::Not, r"(?i)\bnot\b"),
            
            // Conditions
            (TokenType::Has, r"(?i)\bhas\b"),
            (TokenType::Is, r"(?i)\bis\b"),
            
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
            
            // Literals
            (TokenType::String, r#""[^"]*"?"#),
            (TokenType::Time, r"[0-9]+:[0-9]+\s(?:am|pm)|[0-9]+:[0-9]+(?:am|pm)|[0-9]+:[0-9]+|[0-9]+\s(?:am|pm)|[0-9]+(?:am|pm)"),
            (TokenType::Integer, r"[0-9]+"),
            
            // General identifier pattern - must come last
            (TokenType::Identifier, r"[a-zA-Z_][a-zA-Z0-9_]*"),
            
            // Unrecognized characters - must come last to catch anything else
            (TokenType::Unrecognized, r"[^\s]"),
        ]
    }
} 