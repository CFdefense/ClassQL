/// src/tui/widgets/logo.rs
///
/// Logo widget rendering
///
/// Renders the ClassQL ASCII art logo
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Render the logo
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_logo(frame: &mut Frame, theme: &Theme) {
    // logo ascii art
    let ascii_art = r#"            
    ██████╗  ██╗         █████╗    ███████╗   ███████╗   ██████╗   ██╗     
   ██╔════╝  ██║        ██╔══██╗   ██╔════╝   ██╔════╝  ██╔═══██╗  ██║     
   ██║       ██║        ███████║   ███████╗   ███████╗  ██║   ██║  ██║     
   ██║       ██║        ██╔══██║   ╚════██║   ╚════██║  ██║▄▄ ██║  ██║     
   ╚██████╗  ███████╗   ██║  ██║   ███████║   ███████║  ╚██████╔╝  ███████╗
    ╚═════╝  ╚══════╝   ╚═╝  ╚═╝   ╚══════╝   ╚══════╝   ╚══▀▀═╝   ╚══════╝
    "#;

    let lines: Vec<Line> = ascii_art
        .lines()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(theme.logo_color))))
        .collect();

    let frame_width = frame.area().width;
    let frame_height = frame.area().height;
    let logo_width = 80_u16;
    let logo_height = ascii_art.len() as u16;

    // clamp logo dimensions to fit within frame
    let clamped_width = logo_width.min(frame_width);
    let clamped_height = logo_height.min(frame_height);
    let logo_x = if frame_width >= clamped_width {
        (frame_width.saturating_sub(clamped_width)) / 2 + 3 // shift 3 spaces to the right
    } else {
        3.min(frame_width.saturating_sub(1)) // still shift right if possible
    };
    let logo_y = if frame_height >= clamped_height {
        frame_height.saturating_sub(clamped_height + 1)
    } else {
        0
    };

    // ensure x + width doesn't exceed frame width (account for the 3-space shift)
    // clamp to ensure the area is within frame bounds
    let final_width = clamped_width.min(frame_width.saturating_sub(logo_x));
    let final_height = clamped_height.min(frame_height.saturating_sub(logo_y));
    
    // ensure logo_x + final_width doesn't exceed frame_width
    let logo_area = Rect {
        x: logo_x.min(frame_width.saturating_sub(1)),
        y: logo_y.min(frame_height.saturating_sub(1)),
        width: final_width.min(frame_width.saturating_sub(logo_x.min(frame_width.saturating_sub(1)))),
        height: final_height.min(frame_height.saturating_sub(logo_y.min(frame_height.saturating_sub(1)))),
    }.intersection(frame.area()); // Use intersection to ensure it's within frame

    let logo_paragraph = Paragraph::new(lines);
    frame.render_widget(logo_paragraph, logo_area);
}
