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
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    start: i32,
    end: i32,
}

impl Token {
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
        }
    }

    pub fn get_lexeme(&self) -> &str {
        return &self.lexeme
    }
} 