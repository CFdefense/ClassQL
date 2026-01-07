/// src/tui/widgets/completion.rs
///
/// Completion dropdown widget rendering
///
/// Renders tab completion suggestions dropdown

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
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
    };

    let white_bg = Color::Rgb(255, 255, 255);
    let mut styled_lines = Vec::new();
    for (i, completion) in completions.iter().enumerate() {
        let style = if Some(i) == completion_index {
            Style::default().fg(Color::Black).bg(Color::Cyan) // selected: black text on cyan
        } else {
            Style::default().fg(Color::Rgb(0, 0, 0)).bg(white_bg) // unselected: pure black text on white for maximum contrast
        };
        styled_lines.push(Line::from(Span::styled(completion, style)));
    }

    // first, clear the area to cover results below with solid background
    frame.render_widget(Clear, dropdown_area);

    let white_bg = Color::Rgb(255, 255, 255); // True white

    let dropdown_paragraph = Paragraph::new(styled_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Suggestions (↑↓ to navigate, Enter to select)")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(white_bg)),
        );

    frame.render_widget(dropdown_paragraph, dropdown_area);
    
    // force white background on empty/border cells, preserve styled text cells
    let buffer = frame.buffer_mut();
    for y in dropdown_area.top()..dropdown_area.bottom() {
        for x in dropdown_area.left()..dropdown_area.right() {
            let cell = &mut buffer[(x, y)];
            // only set white background if cell is empty or a border character
            // this preserves the text colors and backgrounds set by the paragraph
            if cell.symbol() == " " || cell.symbol() == "│" || cell.symbol() == "─" || 
               cell.symbol() == "┌" || cell.symbol() == "┐" || cell.symbol() == "└" || 
               cell.symbol() == "┘" || cell.symbol() == "├" || cell.symbol() == "┤" ||
               cell.symbol() == "┬" || cell.symbol() == "┴" {
                cell.set_bg(white_bg);
            }
        }
    }
}

