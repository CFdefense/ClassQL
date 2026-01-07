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
    codegen::generate_sql,
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
/// None -> No fields
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// None -> No implemented traits
/// --- ---
///
pub struct Compiler {}

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
            // ..Default::default() // TODO: implement future functionality
        }
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

        // perform code generation
        let sql = match generate_sql(&ast) {
            Ok(sql) => sql,
            Err(e) => {
                return CompilerResult::CodeGenError {
                    message: e.to_string(),
                };
            }
        };

        // execute the SQL query against the database
        let db_path = get_default_db_path();
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
