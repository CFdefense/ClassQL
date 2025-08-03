use tui::tui::Tui;

mod compiler;
mod query;
mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize and run TUI
    let mut tui = match Tui::new() {
        Ok(tui) => tui,
        Err(e) => {
            eprintln!("Failed to initialize TUI: {}", e);
            return Err(Box::new(e));
        }
    };
    
    // Run the TUI event loop
    if let Err(e) = tui.run() {
        eprintln!("TUI error: {}", e);
        // Make sure to terminate even if there's an error
        let _ = tui.terminate();
        return Err(e);
    }
    
    // Clean up and terminate
    if let Err(e) = tui.terminate() {
        eprintln!("Failed to terminate TUI: {}", e);
        return Err(Box::new(e));
    }

    Ok(())
}