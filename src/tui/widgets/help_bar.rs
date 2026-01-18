/// src/tui/widgets/help_bar.rs
///
/// Help bar widget rendering
///
/// Renders context-sensitive help text at the bottom of the screen
use crate::tui::state::FocusMode;
use crate::tui::themes::Theme;
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

/// Help bar widget for rendering context-sensitive help text
///
/// Fields:
/// --- ---
/// toast_message -> optional toast message (help hidden when present)
/// focus_mode -> current focus mode to determine help text
/// schedule_selection_mode -> optional schedule mode for context
/// --- ---
///
pub struct HelpBarWidget {
    pub toast_message: Option<String>,
    pub focus_mode: FocusMode,
    pub schedule_selection_mode: Option<bool>,
}

impl HelpBarWidget {
    /// Create a new HelpBarWidget
    ///
    /// Returns:
    /// --- ---
    /// HelpBarWidget -> The new HelpBarWidget
    /// --- ---
    ///
    pub fn new() -> Self {
        Self {
            toast_message: None,
            focus_mode: FocusMode::MainMenu,
            schedule_selection_mode: None,
        }
    }
}

impl Widget for HelpBarWidget {
    /// Render the help bar widget
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
        // don't show help text if there's an active toast
        if self.toast_message.is_some() {
            return;
        }

        let help_text = match self.focus_mode {
            FocusMode::MainMenu => "↑↓ Navigate | Enter: Select | Esc: Quit",
            FocusMode::Settings => "Esc: Back to Main Menu | Ctrl+C: Quit",
            FocusMode::DetailView => "Press Esc or Enter to close detail view | C: Toggle Cart",
            FocusMode::ResultsBrowse => {
                "←↑↓→ Navigate | Enter: Details | Esc: Main Menu | Type to Search | Alt+G: Guide"
            }
            FocusMode::QueryInput => {
                "Enter: Search | Tab: Completions | ↓: Browse Results | Esc: Main Menu | Alt+G: Guide"
            }
            FocusMode::QueryGuide => "↑↓ Scroll | Page Up/Down | Home/End | Alt+G or Esc: Close",
            FocusMode::Help => "↑↓ Scroll | Page Up/Down | Home/End | Esc: Close",
            FocusMode::ScheduleCreation => {
                // show different help text based on whether we're in selection mode or viewing mode
                if self.schedule_selection_mode == Some(true) {
                    "↑↓ Navigate | Space: Toggle | Tab: Details | Enter: Continue | d: Delete | Esc: Back"
                } else {
                    "←→ Days | ↑↓ Time | Enter: Details | Page Up/Down: Schedules | s: Save | Esc: Back"
                }
            }
            FocusMode::MySchedules => "↑↓ Navigate | Enter: View | d: Delete | Esc: Back",
            FocusMode::SaveNameInput => "Enter: Save | Esc: Cancel",
        };

        let help_width = help_text.len() as u16;

        // position navigation controls at the bottom of the screen
        let help_y = frame.area().height.saturating_sub(2);

        let help_area = Rect {
            x: frame.area().width.saturating_sub(help_width) / 2,
            y: help_y,
            width: help_width,
            height: 2,
        }
        .intersection(frame.area());

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(theme.muted_color))
            .block(Block::default());

        frame.render_widget(help_paragraph, help_area);
    }

    /// Handle a key event and return an action
    ///
    /// Arguments:
    /// --- ---
    /// key -> The key event to handle
    /// --- ---
    ///
    /// Returns: KeyAction -> The action to take in response to the key
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
