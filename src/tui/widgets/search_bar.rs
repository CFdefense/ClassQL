/// src/tui/widgets/search_bar.rs
///
/// Search bar widget rendering
///
/// Renders the query input search bar with syntax highlighting
use crate::tui::state::FocusMode;
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the search bar with data
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The input string
/// problematic_positions -> The problematic positions (byte ranges)
/// focus_mode -> Current focus mode
/// cursor_visible -> Whether cursor should be visible (for blinking)
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_search_bar_with_data(
    frame: &mut Frame,
    input: &str,
    problematic_positions: &[(usize, usize)],
    focus_mode: &FocusMode,
    cursor_visible: bool,
    theme: &Theme,
) {
    let search_width = 50;
    let is_focused = *focus_mode == FocusMode::QueryInput;

    // position search bar below the logo with same vertical gap as menus
    let logo_height = 7; // Height of the ASCII art logo
    let search_y = logo_height + 6; // 6 lines below the logo

    let frame_width = frame.area().width;
    let frame_height = frame.area().height;
    let search_height = 3_u16;

    // clamp search bar dimensions to fit within frame
    let search_area = Rect {
        x: (frame_width.saturating_sub(search_width.min(frame_width) as u16)) / 2,
        y: search_y.min(frame_height.saturating_sub(search_height)),
        width: (search_width as u16).min(frame_width),
        height: search_height.min(frame_height),
    }.intersection(frame.area()); // ensure it's within frame bounds

    // calculate visible width (minus borders and "> " prefix and cursor)
    let visible_width = search_width.saturating_sub(5) as usize; // 2 for borders, 2 for "> ", 1 for cursor
    let input_len = input.chars().count();

    // calculate scroll offset to keep cursor (end of input) visible
    let scroll_offset = if input_len > visible_width {
        input_len - visible_width
    } else {
        0
    };

    // create styled spans for the input with highlighted problematic positions
    let mut styled_spans = Vec::new();

    // start with the "> " prefix (or "…" if scrolled)
    if scroll_offset > 0 {
        styled_spans.push(Span::styled("…", Style::default().fg(theme.muted_color)));
    } else {
        styled_spans.push(Span::styled(
            "> ",
            Style::default().fg(if is_focused {
                theme.selected_color
            } else {
                theme.muted_color
            }),
        ));
    }

    // process only the visible portion of the input
    for (i, ch) in input.chars().enumerate().skip(scroll_offset) {
        // stop if we've filled the visible width
        if i - scroll_offset >= visible_width {
            break;
        }

        let is_problematic = problematic_positions.iter().any(|&(start, end)| {
            // positions are relative to the input string, so we need to match them correctly
            i >= start && i < end
        });

        let style = if is_problematic {
            Style::default().fg(theme.error_color)
        } else {
            Style::default().fg(theme.text_color)
        };

        styled_spans.push(Span::styled(ch.to_string(), style));
    }

    // add flashing cursor if focused
    if is_focused && cursor_visible {
        styled_spans.push(Span::styled("|", Style::default().fg(theme.selected_color)));
    } else if is_focused {
        styled_spans.push(Span::styled(" ", Style::default()));
    }

    let styled_line = Line::from(styled_spans);

    // border color depends on focus state
    let border_color = if is_focused {
        theme.border_color
    } else {
        theme.muted_color
    };

    let title_style = if is_focused {
        Style::default()
            .fg(theme.title_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted_color)
    };

    let search_paragraph = Paragraph::new(styled_line)
        .style(Style::default().fg(theme.text_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ClassQL Query")
                .title_style(title_style)
                .border_style(Style::default().fg(border_color)),
        );

    frame.render_widget(search_paragraph, search_area);
}
