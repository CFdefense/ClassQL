#[derive(Debug, Clone)]
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
    
    // Literals
    String,
    Integer,
    Time,
    Identifier,
    
    // Special
    Exclamation,
    Unrecognized,
}

#[allow(dead_code)]
pub struct Token {
    token_type: TokenType,
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
    pub fn token_type_as_string(&self) -> String {
        match self.token_type {
            TokenType::Term => "T_TERM".to_string(),
            TokenType::Prof => "T_PROF".to_string(),
            TokenType::Course => "T_COURSE".to_string(),
            TokenType::Subject => "T_SUBJECT".to_string(),
            TokenType::Contains => "T_CONTAINS".to_string(),
            TokenType::Title => "T_TITLE".to_string(),
            TokenType::Method => "T_METHOD".to_string(),
            TokenType::Campus => "T_CAMPUS".to_string(),
            TokenType::Credit => "T_CREDIT".to_string(),
            TokenType::Hours => "T_HOURS".to_string(),
            TokenType::Prereqs => "T_PREREQS".to_string(),
            TokenType::Corereqs => "T_COREREQS".to_string(),
            TokenType::Monday => "T_MONDAY".to_string(),
            TokenType::Tuesday => "T_TUESDAY".to_string(),
            TokenType::Wednesday => "T_WEDNESDAY".to_string(),
            TokenType::Thursday => "T_THURSDAY".to_string(),
            TokenType::Friday => "T_FRIDAY".to_string(),
            TokenType::Saturday => "T_SATURDAY".to_string(),
            TokenType::Sunday => "T_SUNDAY".to_string(),
            TokenType::Equals => "T_EQUALS".to_string(),
            TokenType::NotEquals => "T_NOT_EQUALS".to_string(),
            TokenType::LessThan => "T_LESS_THAN".to_string(),
            TokenType::GreaterThan => "T_GREATER_THAN".to_string(),
            TokenType::LessEqual => "T_LESS_EQUAL".to_string(),
            TokenType::GreaterEqual => "T_GREATER_EQUAL".to_string(),
            TokenType::And => "T_AND".to_string(),
            TokenType::Or => "T_OR".to_string(),
            TokenType::Not => "T_NOT".to_string(),
            TokenType::Has => "T_HAS".to_string(),
            TokenType::Is => "T_IS".to_string(),
            TokenType::Starts => "T_STARTS".to_string(),
            TokenType::With => "T_WITH".to_string(),
            TokenType::Ends => "T_ENDS".to_string(),
            TokenType::Does => "T_DOES".to_string(),
            TokenType::Equal => "T_EQUAL".to_string(),
            TokenType::EqualsWord => "T_EQUALS_WORD".to_string(),
            TokenType::Less => "T_LESS".to_string(),
            TokenType::Than => "T_THAN".to_string(),
            TokenType::Greater => "T_GREATER".to_string(),
            TokenType::At => "T_AT".to_string(),
            TokenType::Least => "T_LEAST".to_string(),
            TokenType::Most => "T_MOST".to_string(),
            TokenType::More => "T_MORE".to_string(),
            TokenType::Fewer => "T_FEWER".to_string(),
            TokenType::String => "T_STRING".to_string(),
            TokenType::Integer => "T_INTEGER".to_string(),
            TokenType::Time => "T_TIME".to_string(),
            TokenType::Identifier => "T_IDENTIFIER".to_string(),
            TokenType::Exclamation => "T_EXCLAMATION".to_string(),
            TokenType::Unrecognized => "T_UNRECOGNIZED".to_string(),
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