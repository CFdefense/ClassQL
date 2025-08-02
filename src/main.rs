use tui::tui::Tui;

mod compiler;
mod query;
mod tui;

use compiler::lexer::Lexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize and run TUI
    let mut tui = Tui::new()?;
    tui.run()?;

    // tui to give some output here and we compile it
    let dummy_output = String::from("course CMPT section 101");
    let mut lexer = Lexer::new();
    let tokens = lexer.lexical_analysis(dummy_output);


    tui.terminate()?;

    Ok(())
}