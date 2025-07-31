use tui::tui::Tui;

mod tui;
mod query;
mod compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize and run TUI
    let mut tui = Tui::new()?;
    tui.run()?;
    tui.terminate()?;
    
    Ok(())
}
