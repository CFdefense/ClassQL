/// src/tui/widgets/helpers.rs
///
/// Helper widgets rendering
///
/// Renders help text and other helper UI elements

use crate::data::sql::Class;
use crate::tui::state::FocusMode;
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

/// Render the search helpers with data
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The input string
/// toast_message -> The toast message
/// query_results -> The query results for showing navigation hints
/// focus_mode -> Current focus mode
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_search_helpers_with_data(
    frame: &mut Frame,
    input: &str,
    toast_message: &Option<String>,
    query_results: &[Class],
    focus_mode: &FocusMode,
    theme: &Theme,
) {
    // don't show help text if there's an active toast
    if toast_message.is_some() {
        return;
    }

    let help_text = match focus_mode {
        FocusMode::MainMenu => "↑↓ Navigate | Enter: Select | Esc: Quit",
        FocusMode::Settings => "Esc: Back to Main Menu | Ctrl+C: Quit",
        FocusMode::DetailView => "Press Esc or Enter to close detail view",
        FocusMode::ResultsBrowse => "←↑↓→ Navigate | Enter: Details | Esc: Quit | Type to Search",
        FocusMode::QueryInput => {
            if !query_results.is_empty() {
                "Enter: Search | Tab: Complete | ↓: Browse Results | Esc: Main Menu"
            } else if input.is_empty() {
                "Type a ClassQL query (e.g., 'prof is Brian') | Esc: Main Menu"
            } else {
                "Press Enter to Search, Tab for Completions | ↓: Browse Results | Esc: Main Menu"
            }
        }
    };

    let help_width = help_text.len() as u16;
    let help_area = Rect {
        x: frame.area().width.saturating_sub(help_width) / 2,
        y: frame.area().height.saturating_sub(3),
        width: help_width,
        height: 2,
    };

    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(theme.muted_color))
        .block(Block::default());

    frame.render_widget(help_paragraph, help_area);
}

