use compiler::compiler::Compiler;
use tui::tui::Tui;

mod compiler;
mod sql;
mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut compiler = Compiler::new();
    let mut tui = Tui::new(&mut compiler)?;

    tui.run()?;
    tui.terminate()?;

    Ok(())
}
