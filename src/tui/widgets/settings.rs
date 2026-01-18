/// src/tui/widgets/settings_widget.rs
///
/// Settings widget with encapsulated state, input handling, and rendering
///
/// Handles theme selection, school/term pickers, and sync functionality
///
/// Contains:
/// --- ---
/// SettingsWidget -> Widget for settings functionality
/// SettingsAction -> Actions returned by settings widget
/// --- ---
use crate::data::sql::{School, Term};
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::{Theme, ThemePalette};
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Maximum visible items in picker dropdowns
const PICKER_MAX_VISIBLE: usize = 6;

/// Settings widget with encapsulated state
///
/// Manages application settings including theme selection, school/term pickers
/// with scrollable dropdowns, and database synchronization controls.
///
/// Fields:
/// --- ---
/// current_theme -> The current theme palette
/// selected_index -> Index of currently selected settings option (0=theme, 1=school, 2=term, 3=sync)
/// available_schools -> List of available schools from database
/// selected_school_index -> Index of currently selected school in picker
/// selected_school_id -> ID of the currently selected school
/// school_scroll_offset -> Scroll offset for school picker dropdown
/// available_terms -> List of available terms for selected school
/// selected_term_index -> Index of currently selected term in picker
/// selected_term_id -> ID of the currently selected term
/// term_scroll_offset -> Scroll offset for term picker dropdown
/// last_sync_time -> Timestamp string of last database sync
/// is_syncing -> Whether a sync operation is currently in progress
/// school_picker_open -> Whether school picker dropdown is open
/// term_picker_open -> Whether term picker dropdown is open
/// --- ---
///
pub struct SettingsWidget {
    pub current_theme: ThemePalette,
    pub selected_index: usize,
    pub available_schools: Vec<School>,
    pub selected_school_index: usize,
    pub selected_school_id: Option<String>,
    pub school_scroll_offset: usize,
    pub available_terms: Vec<Term>,
    pub selected_term_index: usize,
    pub selected_term_id: Option<String>,
    pub term_scroll_offset: usize,
    pub last_sync_time: Option<String>,
    pub is_syncing: bool,
    pub school_picker_open: bool,
    pub term_picker_open: bool,
}

/// Action returned by settings widget for app-level handling
///
/// Variants:
/// --- ---
/// None -> No action needed
/// SchoolSelected -> School was selected, caller should load terms
/// TermSelected -> Term was selected
/// SyncRequested -> Database sync was requested
/// ThemeChanged -> Theme palette was changed
/// --- ---
///
#[derive(Debug, Clone)]
pub enum SettingsAction {
    None,
    SchoolSelected {
        school_id: String,
        school_name: String,
    },
    TermSelected {
        term_id: String,
        term_name: String,
    },
    SyncRequested,
    ThemeChanged(ThemePalette),
}

impl SettingsWidget {
    /// Create a new SettingsWidget
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Self -> new SettingsWidget with default state
    /// --- ---
    ///
    pub fn new() -> Self {
        Self {
            current_theme: ThemePalette::Default,
            selected_index: 0,
            available_schools: Vec::new(),
            selected_school_index: 0,
            selected_school_id: None,
            school_scroll_offset: 0,
            available_terms: Vec::new(),
            selected_term_index: 0,
            selected_term_id: None,
            term_scroll_offset: 0,
            last_sync_time: None,
            is_syncing: false,
            school_picker_open: false,
            term_picker_open: false,
        }
    }

    /// Set available schools
    ///
    /// Arguments:
    /// --- ---
    /// schools -> list of available schools
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn set_schools(&mut self, schools: Vec<School>) {
        self.available_schools = schools;
        self.school_scroll_offset = 0;
        // try to find and select the currently selected school
        if let Some(ref id) = self.selected_school_id {
            self.selected_school_index = self
                .available_schools
                .iter()
                .position(|s| &s.id == id)
                .unwrap_or(0);
        }
    }

    /// Set available terms
    ///
    /// Arguments:
    /// --- ---
    /// terms -> list of available terms
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn set_terms(&mut self, terms: Vec<Term>) {
        self.available_terms = terms;
        self.selected_term_index = 0;
        self.term_scroll_offset = 0;
    }

    /// Set the last sync time
    ///
    /// Arguments:
    /// --- ---
    /// time -> optional timestamp string
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn set_last_sync_time(&mut self, time: Option<String>) {
        self.last_sync_time = time;
    }

    /// Handle key and return any action that needs to be taken
    ///
    /// Arguments:
    /// --- ---
    /// key -> the key event to handle
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, SettingsAction) -> tuple of key action and settings action
    /// --- ---
    ///
    pub fn handle_key_with_action(&mut self, key: KeyEvent) -> (KeyAction, SettingsAction) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                (KeyAction::Exit, SettingsAction::None)
            }
            KeyCode::Esc => {
                if self.school_picker_open {
                    self.school_picker_open = false;
                    (KeyAction::Continue, SettingsAction::None)
                } else if self.term_picker_open {
                    self.term_picker_open = false;
                    (KeyAction::Continue, SettingsAction::None)
                } else {
                    (
                        KeyAction::Navigate(FocusMode::MainMenu),
                        SettingsAction::None,
                    )
                }
            }
            KeyCode::Up => {
                if self.school_picker_open {
                    if self.selected_school_index > 0 {
                        self.selected_school_index -= 1;
                        if self.selected_school_index < self.school_scroll_offset {
                            self.school_scroll_offset = self.selected_school_index;
                        }
                    }
                } else if self.term_picker_open {
                    if self.selected_term_index > 0 {
                        self.selected_term_index -= 1;
                        if self.selected_term_index < self.term_scroll_offset {
                            self.term_scroll_offset = self.selected_term_index;
                        }
                    }
                } else if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                (KeyAction::Continue, SettingsAction::None)
            }
            KeyCode::Down => {
                if self.school_picker_open {
                    let max = self.available_schools.len().saturating_sub(1);
                    if self.selected_school_index < max {
                        self.selected_school_index += 1;
                        if self.selected_school_index
                            >= self.school_scroll_offset + PICKER_MAX_VISIBLE
                        {
                            self.school_scroll_offset =
                                self.selected_school_index - PICKER_MAX_VISIBLE + 1;
                        }
                    }
                } else if self.term_picker_open {
                    let max = self.available_terms.len().saturating_sub(1);
                    if self.selected_term_index < max {
                        self.selected_term_index += 1;
                        if self.selected_term_index >= self.term_scroll_offset + PICKER_MAX_VISIBLE
                        {
                            self.term_scroll_offset =
                                self.selected_term_index - PICKER_MAX_VISIBLE + 1;
                        }
                    }
                } else {
                    let max_index = 3; // theme, school, term, sync
                    if self.selected_index < max_index {
                        self.selected_index += 1;
                    }
                }
                (KeyAction::Continue, SettingsAction::None)
            }
            KeyCode::Left | KeyCode::Right => {
                // change theme when on Theme option
                if self.selected_index == 0 {
                    let themes = ThemePalette::all();
                    let current_idx = themes
                        .iter()
                        .position(|&t| t == self.current_theme)
                        .unwrap_or(0);
                    let new_idx = if key.code == KeyCode::Left {
                        if current_idx > 0 {
                            current_idx - 1
                        } else {
                            themes.len() - 1
                        }
                    } else {
                        if current_idx < themes.len() - 1 {
                            current_idx + 1
                        } else {
                            0
                        }
                    };
                    self.current_theme = themes[new_idx];
                    (
                        KeyAction::Continue,
                        SettingsAction::ThemeChanged(self.current_theme),
                    )
                } else {
                    (KeyAction::Continue, SettingsAction::None)
                }
            }
            KeyCode::Enter => {
                match self.selected_index {
                    1 => {
                        // school selection
                        if self.school_picker_open {
                            let school_data = self
                                .available_schools
                                .get(self.selected_school_index)
                                .map(|s| (s.id.clone(), s.name.clone()));

                            if let Some((school_id, school_name)) = school_data {
                                self.selected_school_id = Some(school_id.clone());
                                // clear term selection when school changes
                                self.selected_term_id = None;
                                self.school_picker_open = false;
                                return (
                                    KeyAction::Continue,
                                    SettingsAction::SchoolSelected {
                                        school_id,
                                        school_name,
                                    },
                                );
                            }
                            self.school_picker_open = false;
                            (KeyAction::Continue, SettingsAction::None)
                        } else if !self.available_schools.is_empty() {
                            self.school_picker_open = true;
                            if let Some(ref id) = self.selected_school_id {
                                self.selected_school_index = self
                                    .available_schools
                                    .iter()
                                    .position(|s| &s.id == id)
                                    .unwrap_or(0);
                            }
                            (KeyAction::Continue, SettingsAction::None)
                        } else {
                            (
                                KeyAction::ShowToast {
                                    message: "No schools available. Sync first!".to_string(),
                                    error_type: ErrorType::Warning,
                                },
                                SettingsAction::None,
                            )
                        }
                    }
                    2 => {
                        // term selection
                        if self.term_picker_open {
                            if let Some(term) = self.available_terms.get(self.selected_term_index) {
                                let term_id = term.id.clone();
                                let term_name = term.name.clone();
                                self.selected_term_id = Some(term_id.clone());
                                self.term_picker_open = false;
                                return (
                                    KeyAction::Continue,
                                    SettingsAction::TermSelected { term_id, term_name },
                                );
                            }
                            self.term_picker_open = false;
                            (KeyAction::Continue, SettingsAction::None)
                        } else if self.selected_school_id.is_none() {
                            (
                                KeyAction::ShowToast {
                                    message: "Select a school first!".to_string(),
                                    error_type: ErrorType::Warning,
                                },
                                SettingsAction::None,
                            )
                        } else if !self.available_terms.is_empty() {
                            self.term_picker_open = true;
                            if let Some(ref id) = self.selected_term_id {
                                self.selected_term_index = self
                                    .available_terms
                                    .iter()
                                    .position(|t| &t.id == id)
                                    .unwrap_or(0);
                            }
                            (KeyAction::Continue, SettingsAction::None)
                        } else {
                            (
                                KeyAction::ShowToast {
                                    message: "No terms available for this school.".to_string(),
                                    error_type: ErrorType::Warning,
                                },
                                SettingsAction::None,
                            )
                        }
                    }
                    3 => {
                        // trigger sync
                        if !self.is_syncing {
                            self.is_syncing = true;
                            (KeyAction::Continue, SettingsAction::SyncRequested)
                        } else {
                            (KeyAction::Continue, SettingsAction::None)
                        }
                    }
                    _ => (KeyAction::Continue, SettingsAction::None),
                }
            }
            _ => (KeyAction::Continue, SettingsAction::None),
        }
    }

    /// Mark sync as complete
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    pub fn sync_complete(&mut self) {
        self.is_syncing = false;
    }

    /// Render the settings menu
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_settings(&self, frame: &mut Frame, theme: &Theme) {
        let settings_width = 60_u16;
        let base_height = 16_u16;

        // expand height if school or term picker is open
        let school_picker_items = self.available_schools.len().min(8);
        let term_picker_items = self.available_terms.len().min(8);
        let settings_height = if self.school_picker_open {
            base_height + school_picker_items as u16 + 2
        } else if self.term_picker_open {
            base_height + term_picker_items as u16 + 2
        } else {
            base_height
        };

        // position settings below the logo
        let logo_height = 7_u16;
        let spacing = 6_u16;
        let settings_y = logo_height + spacing;

        let frame_width = frame.area().width;
        let frame_height = frame.area().height;

        // clamp settings dimensions to fit within frame
        let settings_area = Rect {
            x: (frame_width.saturating_sub(settings_width.min(frame_width))) / 2,
            y: settings_y.min(frame_height.saturating_sub(settings_height.min(frame_height))),
            width: settings_width.min(frame_width),
            height: settings_height.min(frame_height),
        }
        .intersection(frame.area());

        let mut lines = Vec::new();

        // --- theme option ---
        let theme_prefix = if self.selected_index == 0 {
            "▸ "
        } else {
            "  "
        };
        let theme_style = if self.selected_index == 0 {
            Style::default()
                .fg(theme.selected_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_color)
        };
        lines.push(Line::from(vec![
            Span::styled(theme_prefix, theme_style),
            Span::styled("Theme: ", theme_style),
            Span::styled(
                self.current_theme.as_str(),
                Style::default().fg(theme.warning_color),
            ),
            Span::styled(" (← → to change)", Style::default().fg(theme.muted_color)),
        ]));
        lines.push(Line::from(""));

        // --- school selection option ---
        let school_prefix = if self.selected_index == 1 {
            "▸ "
        } else {
            "  "
        };
        let school_style = if self.selected_index == 1 {
            Style::default()
                .fg(theme.selected_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_color)
        };

        let school_name = if let Some(ref school_id) = self.selected_school_id {
            self.available_schools
                .iter()
                .find(|s| &s.id == school_id)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown")
        } else if self.available_schools.is_empty() {
            "No schools (sync first)"
        } else {
            "None selected"
        };

        let school_hint = if self.school_picker_open {
            " (↑↓ pick, Enter confirm)"
        } else {
            " (Enter to select)"
        };

        lines.push(Line::from(vec![
            Span::styled(school_prefix, school_style),
            Span::styled("School: ", school_style),
            Span::styled(school_name, Style::default().fg(theme.info_color)),
            Span::styled(school_hint, Style::default().fg(theme.muted_color)),
        ]));

        // show school picker dropdown if open
        if self.school_picker_open && !self.available_schools.is_empty() {
            lines.push(Line::from(""));
            let total = self.available_schools.len();
            let start = self.school_scroll_offset;
            let end = (start + PICKER_MAX_VISIBLE).min(total);

            // show scroll indicator at top if not at beginning
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!("     ↑ {} more above", start),
                    Style::default().fg(theme.muted_color),
                )));
            }

            for (i, school) in self
                .available_schools
                .iter()
                .enumerate()
                .skip(start)
                .take(end - start)
            {
                let is_selected = i == self.selected_school_index;
                let prefix = if is_selected { "   ● " } else { "   ○ " };
                let style = if is_selected {
                    Style::default()
                        .fg(theme.success_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_color)
                };
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(&school.name, style),
                ]));
            }

            // show scroll indicator at bottom if more items below
            if end < total {
                lines.push(Line::from(Span::styled(
                    format!("     ↓ {} more below", total - end),
                    Style::default().fg(theme.muted_color),
                )));
            }
        }
        lines.push(Line::from(""));

        // --- term selection option ---
        let term_prefix = if self.selected_index == 2 {
            "▸ "
        } else {
            "  "
        };
        let term_style = if self.selected_index == 2 {
            Style::default()
                .fg(theme.selected_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_color)
        };

        let term_name = if let Some(ref term_id) = self.selected_term_id {
            self.available_terms
                .iter()
                .find(|t| &t.id == term_id)
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown")
        } else if self.selected_school_id.is_none() {
            "Select school first"
        } else if self.available_terms.is_empty() {
            "No terms available"
        } else {
            "None selected"
        };

        let term_hint = if self.term_picker_open {
            " (↑↓ pick, Enter confirm)"
        } else {
            " (Enter to select)"
        };

        lines.push(Line::from(vec![
            Span::styled(term_prefix, term_style),
            Span::styled("Term: ", term_style),
            Span::styled(term_name, Style::default().fg(theme.info_color)),
            Span::styled(term_hint, Style::default().fg(theme.muted_color)),
        ]));

        // show term picker dropdown if open
        if self.term_picker_open && !self.available_terms.is_empty() {
            lines.push(Line::from(""));
            let total = self.available_terms.len();
            let start = self.term_scroll_offset;
            let end = (start + PICKER_MAX_VISIBLE).min(total);

            // show scroll indicator at top if not at beginning
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!("     ↑ {} more above", start),
                    Style::default().fg(theme.muted_color),
                )));
            }

            for (i, term) in self
                .available_terms
                .iter()
                .enumerate()
                .skip(start)
                .take(end - start)
            {
                let is_selected = i == self.selected_term_index;
                let prefix = if is_selected { "   ● " } else { "   ○ " };
                let style = if is_selected {
                    Style::default()
                        .fg(theme.success_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_color)
                };
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(&term.name, style),
                ]));
            }

            // show scroll indicator at bottom if more items below
            if end < total {
                lines.push(Line::from(Span::styled(
                    format!("     ↓ {} more below", total - end),
                    Style::default().fg(theme.muted_color),
                )));
            }
        }
        lines.push(Line::from(""));

        // --- sync option ---
        let sync_prefix = if self.selected_index == 3 {
            "▸ "
        } else {
            "  "
        };
        let sync_style = if self.selected_index == 3 {
            Style::default()
                .fg(theme.selected_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_color)
        };

        let sync_status = if self.is_syncing {
            Span::styled("Syncing...", Style::default().fg(theme.warning_color))
        } else {
            Span::styled(
                "[Press Enter to sync]",
                Style::default().fg(theme.success_color),
            )
        };

        lines.push(Line::from(vec![
            Span::styled(sync_prefix, sync_style),
            Span::styled("Sync Data: ", sync_style),
            sync_status,
        ]));
        lines.push(Line::from(""));

        // --- last sync time ---
        let sync_time_display = match &self.last_sync_time {
            Some(time) => time.to_string(),
            None => "Never".to_string(),
        };
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Last synced: ", Style::default().fg(theme.muted_color)),
            Span::styled(sync_time_display, Style::default().fg(theme.info_color)),
        ]));

        let settings_paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Settings ")
                .title_style(
                    Style::default()
                        .fg(theme.title_color)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(theme.border_color)),
        );

        frame.render_widget(settings_paragraph, settings_area);
    }
}

impl Widget for SettingsWidget {
    /// Render the settings menu
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render(&self, frame: &mut Frame, theme: &Theme) {
        self.render_settings(frame, theme);
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
    fn handle_key(&mut self, key: KeyEvent) -> KeyAction {
        let (action, _settings_action) = self.handle_key_with_action(key);
        action
    }

    /// Get the focus modes this widget handles
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Vec<FocusMode> -> The focus modes this widget handles
    /// --- ---
    ///
    fn focus_modes(&self) -> Vec<FocusMode> {
        vec![FocusMode::Settings]
    }
}
