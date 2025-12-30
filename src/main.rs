#![allow(dead_code)]
use clap::Parser;
use classql::dsl::compiler::{
    Compiler,
    CompilerResult::{LexerError, ParserError, Success},
};
use classql::tui::render::Tui;
use classql::utils::visualizetree::ast_to_dot;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "QUERY_STRING")]
    query: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
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
