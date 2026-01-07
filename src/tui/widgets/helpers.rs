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
    _input: &str,
    toast_message: &Option<String>,
    _query_results: &[Class],
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
        FocusMode::ResultsBrowse => {
            "←↑↓→ Navigate | Enter: Details | Esc: Main Menu | Type to Search"
        }
        FocusMode::QueryInput => {
            "Enter: Search | Tab: Completions | ↓: Browse Results | Esc: Main Menu"
        }
    };

    let help_width = help_text.len() as u16;

    // position navigation controls at the bottom of the screen
    let help_y = frame.area().height.saturating_sub(2);

    let help_area = Rect {
        x: frame.area().width.saturating_sub(help_width) / 2,
        y: help_y,
        width: help_width,
        height: 2,
    };

    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(theme.muted_color))
        .block(Block::default());

    frame.render_widget(help_paragraph, help_area);
}

/// Get day order for sorting (Monday = 0, Sunday = 6)
///
/// Parameters:
/// --- ---
/// day_code -> Day code string (M, T, W, TH, F, S, SU)
/// --- ---
///
/// Returns:
/// --- ---
/// u8 -> Day order (0-6 for valid days, 99 for unknown)
/// --- ---
///
pub fn get_day_order(day_code: &str) -> u8 {
    match day_code {
        "M" => 0,  // monday
        "T" => 1,  // Tuesday
        "W" => 2,  // wednesday
        "TH" => 3, // thursday
        "F" => 4,  // friday
        "S" => 5,  // saturday
        "SU" => 6, // sunday
        _ => 99,   // unknown days go last
    }
}

/// Format day code for display (add space after single-letter codes)
///
/// Parameters:
/// --- ---
/// day_code -> Day code string (M, T, W, TH, F, S, SU)
/// --- ---
///
/// Returns:
/// --- ---
/// String -> Formatted day code with space padding for alignment
/// --- ---
///
pub fn format_day_for_display(day_code: &str) -> String {
    // check if it's a single letter (not TH, SU, etc.)
    if day_code.len() == 1 {
        format!("{} ", day_code) // add space after single letter
    } else {
        day_code.to_string() // keep multi-letter codes as-is
    }
}
