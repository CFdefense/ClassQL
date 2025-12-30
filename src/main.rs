/// src/main.rs
///
/// Main entry point for the ClassQL application
///
/// Responsible for parsing CLI arguments and running the appropriate mode:
/// --- ---
/// - If a query is provided, compile it and visualize the AST
/// - If no query is provided, run the TUI
/// --- ---
///
/// Contains:
/// --- ---
/// Args -> CLI arguments struct
/// main -> Main function
/// --- ---
use clap::Parser;

use classql::dsl::compiler::{
    Compiler,
    CompilerResult::{LexerError, ParserError, Success},
};
use classql::tui::render::Tui;
use classql::utils::visualizetree::ast_to_dot;

/// Args struct
///
/// Fields:
/// --- ---
/// query -> The query string to compile and visualize the AST
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Parser -> Parser trait for Args
/// Debug -> Debug trait for Args
/// --- ---
///
/// Attributes:
/// --- ---
/// author -> The author of the application (Clap attribute)
/// version -> The version of the application (Clap attribute)
/// about -> The about of the application (Clap attribute)
/// long_about -> The long about of the application (Clap attribute)
/// --- ---
///
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "QUERY_STRING")]
    query: Option<String>,
}

/// Main function
///
/// Parameters:
/// --- ---
/// None
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), Box<dyn std::error::Error>> -> The result of the main function
/// --- ---
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse the cli arguments
    let args = Args::parse();

    // if a query is provided, compile it and visualize the AST
    if let Some(query) = args.query {
        let mut compiler = Compiler::new();
        match compiler.run(&query) {
            Success { ast, .. } => {
                println!("{}", ast_to_dot(query, &ast))
            }
            LexerError { message, .. } => {
                println!("{}", message);
                std::process::exit(1);
            }
            ParserError { message, .. } => {
                println!("{}", message);
                std::process::exit(1);
            }
        }
    } else {
        let compiler = Compiler::new();
        let mut tui = Tui::new(compiler)?;

        tui.run()?;
        tui.terminate()?;
    }

    Ok(())
}
