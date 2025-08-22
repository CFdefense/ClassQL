#![allow(dead_code)]
use compiler::driver::Compiler;
use tui::render::Tui;

mod compiler;
mod data;
mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();
    let mut tui = Tui::new(compiler)?;

    tui.run()?;
    tui.terminate()?;

    Ok(())
}
