/// src/tui/widgets/completion.rs
///
/// Completion dropdown widget rendering
///
/// Renders tab completion suggestions dropdown
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Render the completion dropdown
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// completions -> The completions
/// completion_index -> The completion index
/// show_completions -> Whether to show completions
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_completion_dropdown(
    frame: &mut Frame,
    completions: &[String],
    completion_index: Option<usize>,
    show_completions: bool,
    theme: &Theme,
) {
    if !show_completions {
        return;
    }

    let dropdown_width = 50;

    // position below the search bar
    let logo_height = 7; // height of the ASCII art logo
    let search_y = logo_height + 2; // search bar position
    let search_height = 3; // search bar height
    let dropdown_y = search_y + search_height + 1; // 1 line below search bar

    // calculate max available height (leave some space at bottom)
    let max_available_height = frame.area().height.saturating_sub(dropdown_y + 2);

    // height = number of completions + 2 for borders, capped by available space
    let dropdown_height = (completions.len() as u16 + 2).min(max_available_height);

    let dropdown_area = Rect {
        x: frame.area().width.saturating_sub(dropdown_width) / 2,
        y: dropdown_y,
        width: dropdown_width,
        height: dropdown_height,
    }.intersection(frame.area()); // ensure it's within frame bounds

    let mut styled_lines = Vec::new();
    for (i, completion) in completions.iter().enumerate() {
        let style = if Some(i) == completion_index {
            Style::default()
                .fg(theme.background_color)
                .bg(theme.selected_color)
        } else {
            Style::default()
                .fg(theme.text_color)
                .bg(theme.background_color)
        };
        styled_lines.push(Line::from(Span::styled(completion, style)));
    }

    // first, clear the area to cover results below with solid background
    frame.render_widget(Clear, dropdown_area);

    let dropdown_paragraph = Paragraph::new(styled_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Suggestions (↑↓ to navigate, Enter to select)")
            .title_style(Style::default().fg(theme.warning_color))
            .border_style(Style::default().fg(theme.warning_color))
            .style(Style::default().bg(theme.background_color)),
    );

    frame.render_widget(dropdown_paragraph, dropdown_area);

    // force background color on empty/border cells, preserve styled text cells
    let buffer = frame.buffer_mut();
    let buffer_width = buffer.area.width;
    let buffer_height = buffer.area.height;

    // ensure we don't access out of bounds - right() and bottom() are exclusive
    // so we need to clamp them to buffer dimensions
    let start_y = dropdown_area.top();
    let start_x = dropdown_area.left();
    // right() and bottom() return exclusive coordinates, so clamp to buffer bounds
    let end_y = dropdown_area.bottom().min(buffer_height);
    let end_x = dropdown_area.right().min(buffer_width);

    // only iterate if area is valid and within bounds
    if start_y < buffer_height && start_x < buffer_width && end_y > start_y && end_x > start_x {
        for y in start_y..end_y {
            for x in start_x..end_x.min(buffer_width) {
                // final bounds check before access
                if x < buffer_width && y < buffer_height {
                    let cell = &mut buffer[(x, y)];

                    // only set background if cell is empty or a border character
                    // this preserves the text colors and backgrounds set by the paragraph
                    if cell.symbol() == " "
                        || cell.symbol() == "│"
                        || cell.symbol() == "─"
                        || cell.symbol() == "┌"
                        || cell.symbol() == "┐"
                        || cell.symbol() == "└"
                        || cell.symbol() == "┘"
                        || cell.symbol() == "├"
                        || cell.symbol() == "┤"
                        || cell.symbol() == "┬"
                        || cell.symbol() == "┴"
                    {
                        cell.set_bg(theme.background_color);
                    }
                }
            }
        }
    }
}
