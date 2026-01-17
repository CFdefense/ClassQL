use crate::data::sql::{execute_query, get_default_db_path, Class};
/// src/dsl/compiler.rs
///
/// Compiler for the DSL
///
/// Responsible for compiling the DSL into a SQL query
///
/// Contains:
/// --- ---
/// CompilerResult -> Result types for the compiler
/// Compiler -> Compiler struct
///      Methods:
///      --- ---
///      new -> Create a new compiler instance
///      run -> Compile the DSL into a SQL query
///      get_tab_completion -> Get tab completion suggestions for the current input
///      --- ---
/// --- ---
///
use crate::dsl::{
    codegen::generate_sql_with_filters,
    lexer::Lexer,
    parser::{Ast, Parser},
    semantic::semantic_analysis,
};
use crate::tui::errors::AppError;

/// Result Types for the Compiler
///
/// Results:
/// --- ---
/// Sucess -> Compilation was successful, contains message, generated SQL, positions and AST
/// LexerError -> Lexical analysis failed, contains message and problematic positions
/// ParserError -> Parsing failed, contains message and problematic positions
/// SemanticError -> Semantic analysis failed, contains message and problematic positions
/// CodeGenError -> Code generation failed, contains message
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for CompilerResult
/// --- ---
///
///
#[derive(Debug)]
pub enum CompilerResult {
    Success {
        message: String,
        sql: String,
        classes: Vec<Class>,
        ast: Ast,
    },
    LexerError {
        message: String,
        problematic_positions: Vec<(usize, usize)>,
    },
    ParserError {
        message: String,
        problematic_positions: Vec<(usize, usize)>,
    },
    SemanticError {
        message: String,
        problematic_positions: Vec<(usize, usize)>,
    },
    CodeGenError {
        message: String,
    },
}

/// Compiler for the DSL
///
/// Responsible for compiling the DSL into a SQL query
///
/// Fields:
/// --- ---
/// school_id -> Optional school ID to filter results
/// term_id -> Optional term ID to filter results
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// None -> No implemented traits
/// --- ---
///
pub struct Compiler {
    school_id: Option<String>,
    term_id: Option<String>,
}

/// Compiler Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new compiler instance
/// run -> Compile the DSL into a SQL query
/// get_tab_completion -> Get tab completion suggestions for the current input
/// --- ---
///
impl Compiler {
    /// Create a new compiler instance
    ///
    /// TODO: implement future functionality for cleaner state refresh
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Compiler -> The new compiler instance
    /// --- ---
    ///
    pub fn new() -> Self {
        Compiler {
            school_id: None,
            term_id: None,
        }
    }
    
    /// Set the school ID for filtering results
    ///
    /// Parameters:
    /// --- ---
    /// school_id -> The school ID to filter by, or None to clear
    /// --- ---
    ///
    pub fn set_school_id(&mut self, school_id: Option<String>) {
        self.school_id = school_id;
    }
    
    /// Set the term ID for filtering results
    ///
    /// Parameters:
    /// --- ---
    /// term_id -> The term ID to filter by, or None to clear
    /// --- ---
    ///
    pub fn set_term_id(&mut self, term_id: Option<String>) {
        self.term_id = term_id;
    }

    /// Compile the DSL into a SQL query
    ///
    /// Parameters:
    /// --- ---
    /// input -> The input string to compile
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// CompilerResult:
    ///     Success -> Compilation was successful, contains message and AST
    ///     LexerError -> Lexical analysis failed, contains message and problematic tokens
    ///     ParserError -> Parsing failed, contains message and problematic tokens
    /// --- ---
    ///
    pub fn run(&mut self, input: &str) -> CompilerResult {
        // refresh lexer state
        let mut lexer = Lexer::new(input.to_string());

        // perform lexical analysis
        let tokens = match lexer.analyze() {
            Ok(tokens) => tokens,
            Err(AppError::UnrecognizedTokens(error_msg, problematic_positions)) => {
                return CompilerResult::LexerError {
                    message: error_msg,
                    problematic_positions,
                };
            }
            Err(_) => {
                return CompilerResult::LexerError {
                    message: "Unknown lexer error".to_string(),
                    problematic_positions: Vec::new(),
                };
            }
        };

        // perform parsing
        let mut parser = Parser::new(input.to_string());

        // try to parse the tokens
        let ast = match parser.parse(&tokens) {
            Ok(ast) => ast,
            Err(error_tuple) => {
                let (e, problematic_tokens) = error_tuple;
                let problematic_positions: Vec<(usize, usize)> = problematic_tokens
                    .iter()
                    .map(|token| (token.get_start(), token.get_end()))
                    .collect();
                return CompilerResult::ParserError {
                    message: e.to_string(),
                    problematic_positions,
                };
            }
        };

        // perform semantic analysis
        match semantic_analysis(&ast) {
            Ok(()) => {}
            Err((e, problematic_positions)) => {
                return CompilerResult::SemanticError {
                    message: e.to_string(),
                    problematic_positions,
                };
            }
        }

        // check if using test database (special "_test" school ID)
        let use_test_db = self.school_id.as_deref() == Some("_test");
        
        // perform code generation with optional school and term filters
        // skip filters if using test database
        let (school_filter, term_filter) = if use_test_db {
            (None, None)
        } else {
            (self.school_id.as_deref(), self.term_id.as_deref())
        };
        
        let sql = match generate_sql_with_filters(&ast, school_filter, term_filter) {
            Ok(sql) => sql,
            Err(e) => {
                return CompilerResult::CodeGenError {
                    message: e.to_string(),
                };
            }
        };

        // execute the SQL query against the database
        let db_path = if use_test_db {
            std::path::PathBuf::from("classy/test.db")
        } else {
            get_default_db_path()
        };
        let classes = match execute_query(&sql, &db_path) {
            Ok(classes) => classes,
            Err(e) => {
                return CompilerResult::CodeGenError {
                    message: format!("Database query error: {}", e),
                };
            }
        };

        // return success if all operations were successful
        CompilerResult::Success {
            message: "Success".to_string(),
            sql,
            classes,
            ast,
        }
    }

    /// Get tab completion suggestions for the current input
    ///
    /// Partial Compilation Method:
    /// --- ---
    /// Will ONLY preform lexical analysis on the input and then a special parser method to get completion suggestions
    /// --- ---
    ///
    /// Parameters:
    /// --- ---
    /// input -> The input string to get completion suggestions for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Vec<String> -> Vector of strings of completion suggestions
    /// --- ---
    ///
    pub fn get_tab_completion(&mut self, input: String) -> Vec<String> {
        // refresh lexer state
        let mut lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(input.to_string());

        // try to analyze the input
        match lexer.analyze() {
            Ok(tokens) => {
                // lexical analysis succeeded, now try to get completion suggestions from parser
                parser.get_completion_suggestions(&tokens)
            }
            Err(_) => {
                // lexical analysis failed, provide basic suggestions
                if input.trim().is_empty() {
                    vec![
                        "professor".to_string(),
                        "course".to_string(),
                        "subject".to_string(),
                        "title".to_string(),
                    ]
                } else {
                    vec![] // can't provide suggestions for invalid tokens
                }
            }
        }
    }
}
