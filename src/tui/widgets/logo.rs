/// src/tui/widgets/logo.rs
///
/// Logo widget rendering
///
/// Renders the ClassQL ASCII art logo
use crate::tui::state::FocusMode;
use crate::tui::themes::Theme;
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Logo widget for rendering the ClassQL ASCII art logo
///
/// Fields:
/// --- ---
/// (none) -> Stateless widget
/// --- ---
///
pub struct LogoWidget;

impl LogoWidget {
    /// Create a new LogoWidget
    ///
    /// Returns:
    /// --- ---
    /// LogoWidget -> The new LogoWidget
    /// --- ---
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Widget for LogoWidget {
    /// Render the logo widget
    ///
    /// Arguments:
    /// --- ---
    /// frame -> The frame to render to
    /// theme -> The theme to use for styling
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render(&self, frame: &mut Frame, theme: &Theme) {
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
            (frame_width.saturating_sub(clamped_width)) / 2 + 1 // shift 1 space to the right
        } else {
            1.min(frame_width.saturating_sub(1)) // still shift right if possible
        };
        // position logo a few rows down from the top
        let logo_y = 3.min(frame_height.saturating_sub(1));

        // ensure x + width doesn't exceed frame width (account for the 1-space shift)
        // clamp to ensure the area is within frame bounds
        let final_width = clamped_width.min(frame_width.saturating_sub(logo_x));
        let final_height = clamped_height.min(frame_height.saturating_sub(logo_y));

        // ensure logo_x + final_width doesn't exceed frame_width
        let logo_area = Rect {
            x: logo_x.min(frame_width.saturating_sub(1)),
            y: logo_y.min(frame_height.saturating_sub(1)),
            width: final_width
                .min(frame_width.saturating_sub(logo_x.min(frame_width.saturating_sub(1)))),
            height: final_height
                .min(frame_height.saturating_sub(logo_y.min(frame_height.saturating_sub(1)))),
        }
        .intersection(frame.area()); // Use intersection to ensure it's within frame

        let logo_paragraph = Paragraph::new(lines);
        frame.render_widget(logo_paragraph, logo_area);
    }

    /// Handle key event
    ///
    /// Arguments:
    /// --- ---
    /// key -> The key event to handle
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// KeyAction -> The action to take in response to the key
    /// --- ---
    ///
    fn handle_key(&mut self, _key: KeyEvent) -> KeyAction {
        KeyAction::Continue
    }

    /// Return the focus mode(s) this widget handles
    ///
    ///
    /// Returns:
    /// --- ---
    /// Vec<FocusMode> -> The focus modes this widget handles
    /// --- ---
    ///
    fn focus_modes(&self) -> Vec<FocusMode> {
        vec![]
    }
}
