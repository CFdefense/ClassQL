/// src/tui/widgets/logo.rs
///
/// Logo widget rendering
///
/// Renders the ClassQL ASCII art logo

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Render the logo
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_logo(frame: &mut Frame) {
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
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default().fg(Color::Rgb(135, 206, 235)),
            ))
        })
        .collect();

    let logo_area = Rect {
        x: frame.area().width.saturating_sub(75) / 2,
        y: frame
            .area()
            .height
            .saturating_sub(ascii_art.len() as u16 + 1),
        width: 80,
        height: ascii_art.len() as u16,
    };

    let logo_paragraph = Paragraph::new(lines);
    frame.render_widget(logo_paragraph, logo_area);
}

