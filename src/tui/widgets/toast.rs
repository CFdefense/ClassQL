/// src/tui/widgets/toast.rs
///
/// Toast widget rendering
///
/// Renders error toast notifications
use crate::tui::state::ErrorType;
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the toast with data
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// toast_message -> The toast message
/// error_type -> The error type
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_toast_with_data(
    frame: &mut Frame,
    toast_message: &Option<String>,
    error_type: &Option<ErrorType>,
    theme: &Theme,
) {
    if let Some(message) = toast_message {
        // use the passed error type to determine toast dimensions
        let is_parser_error = matches!(error_type, Some(ErrorType::Parser));

        // calculate toast dimensions based on error type
        let (toast_width, max_toast_height) = if is_parser_error {
            // parser errors need more space for context and suggestions
            (80_u16, 15)
        } else {
            // lexer errors are typically shorter
            (60, 8)
        };

        // wrap text to fit within the toast width (account for borders and padding)
        let content_width = toast_width.saturating_sub(4) as usize; // -4 for borders and padding
        let mut wrapped_lines = Vec::new();

        for line in message.lines() {
            if line.len() <= content_width {
                wrapped_lines.push(line.to_string());
            } else {
                // split long lines into multiple lines
                let mut remaining = line;
                while !remaining.is_empty() {
                    if remaining.len() <= content_width {
                        wrapped_lines.push(remaining.to_string());
                        break;
                    } else {
                        // find a good break point (space, comma, etc.)
                        let mut break_point = content_width;
                        if let Some(space_pos) = remaining[..content_width].rfind(' ') {
                            break_point = space_pos;
                        } else if let Some(comma_pos) = remaining[..content_width].rfind(',') {
                            break_point = comma_pos + 1; // include the comma
                        }

                        wrapped_lines.push(remaining[..break_point].to_string());
                        remaining = remaining[break_point..].trim_start();
                    }
                }
            }
        }

        let toast_height = (wrapped_lines.len() as u16 + 2).min(max_toast_height);

        let toast_area = Rect {
            x: (frame.area().width.saturating_sub(toast_width)) / 2,
            y: frame.area().height.saturating_sub(toast_height + 1),
            width: toast_width,
            height: toast_height,
        };

        // create styled lines for the toast
        let styled_lines: Vec<Line> = wrapped_lines
            .iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(theme.text_color))))
            .collect();

        let toast_paragraph = Paragraph::new(styled_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .title_style(Style::default().fg(theme.warning_color))
                .border_style(Style::default().fg(theme.error_color))
                .style(Style::default().bg(theme.muted_color)),
        );

        frame.render_widget(toast_paragraph, toast_area);
    }
}
