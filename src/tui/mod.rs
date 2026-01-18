/// src/tui/mod.rs
///
/// Module for storing terminal user interface logic. Used for our TUI.
pub mod app;
pub mod errors;
pub mod save;
pub mod state;
pub mod themes;
pub mod widgets;

// re-export the main app
pub use app::TuiApp;
