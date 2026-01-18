/// src/tui/widgets/traits.rs
///
/// Widget trait definition for TUI components
///
/// Provides a unified interface for widgets that can render and handle input
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::Theme;
use crossterm::event::KeyEvent;
use ratatui::Frame;

/// Result of handling a key event in a widget
///
/// Variants:
/// --- ---
/// Continue -> Stay in the current widget, no state change
/// Exit -> Exit the application
/// Navigate -> Navigate to a different focus mode
/// ShowToast -> Show a toast notification
/// --- ---
///
#[derive(Debug, Clone)]
pub enum KeyAction {
    /// Stay in the current widget
    Continue,
    /// Exit the application
    Exit,
    /// Navigate to a different focus mode
    Navigate(FocusMode),
    /// Show a toast notification
    ShowToast {
        message: String,
        error_type: ErrorType,
    },
}

/// Trait for TUI widgets that can render and handle input
///
/// Methods:
/// --- ---
/// render -> Render the widget to the frame
/// handle_key -> Handle a key event and return an action
/// focus_mode -> Return the focus mode(s) this widget handles
/// --- ---
///
pub trait Widget {
    /// Render the widget to the frame
    ///
    /// Parameters:
    /// --- ---
    /// frame -> The frame to render to
    /// theme -> The current theme
    /// --- ---
    ///
    fn render(&self, frame: &mut Frame, theme: &Theme);

    /// Handle a key event and return an action
    ///
    /// Parameters:
    /// --- ---
    /// key -> The key event to handle
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// KeyAction -> The action to take in response to the key
    /// --- ---
    ///
    fn handle_key(&mut self, key: KeyEvent) -> KeyAction;

    /// Return the focus mode(s) this widget handles
    ///
    /// Returns:
    /// --- ---
    /// Vec<FocusMode> -> The focus modes this widget handles
    /// --- ---
    ///
    fn focus_modes(&self) -> Vec<FocusMode>;
}
