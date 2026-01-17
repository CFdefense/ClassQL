/// src/tui/widgets/settings.rs
///
/// Settings widget rendering
///
/// Renders the settings menu with theme, school, term selection, and sync options
use crate::data::sql::{School, Term};
use crate::tui::themes::{Theme, ThemePalette};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Maximum visible items in picker dropdowns
const PICKER_MAX_VISIBLE: usize = 6;

/// Settings state for rendering
/// 
/// Fields:
/// --- ---
/// current_theme -> The current theme
/// selected_index -> The index of the currently selected option
/// available_schools -> The available schools
/// selected_school_index -> The index of the currently selected school
/// selected_school_id -> The ID of the currently selected school
/// school_scroll_offset -> Scroll offset for school picker
/// available_terms -> The available terms for the selected school
/// selected_term_index -> The index of the currently selected term
/// selected_term_id -> The ID of the currently selected term
/// term_scroll_offset -> Scroll offset for term picker
/// last_sync_time -> The last sync time
/// is_syncing -> Whether the data is currently syncing
/// school_picker_open -> Whether the school picker is open
/// term_picker_open -> Whether the term picker is open
/// --- ---
///
pub struct SettingsState<'a> {
    pub current_theme: ThemePalette,
    pub selected_index: usize,
    pub available_schools: &'a [School],
    pub selected_school_index: usize,
    pub selected_school_id: Option<&'a str>,
    pub school_scroll_offset: usize,
    pub available_terms: &'a [Term],
    pub selected_term_index: usize,
    pub selected_term_id: Option<&'a str>,
    pub term_scroll_offset: usize,
    pub last_sync_time: Option<&'a str>,
    pub is_syncing: bool,
    pub school_picker_open: bool,
    pub term_picker_open: bool,
}

/// Render the settings menu
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// theme -> The current theme
/// state -> The settings state
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_settings(frame: &mut Frame, theme: &Theme, state: &SettingsState) {
    let settings_width = 60_u16;
    let base_height = 16_u16;
    
    // expand height if school or term picker is open
    let school_picker_items = state.available_schools.len().min(8);
    let term_picker_items = state.available_terms.len().min(8);
    let settings_height = if state.school_picker_open {
        base_height + school_picker_items as u16 + 2
    } else if state.term_picker_open {
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
    let theme_prefix = if state.selected_index == 0 { "▸ " } else { "  " };
    let theme_style = if state.selected_index == 0 {
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
            state.current_theme.as_str(),
            Style::default().fg(theme.warning_color),
        ),
        Span::styled(" (← → to change)", Style::default().fg(theme.muted_color)),
    ]));
    lines.push(Line::from(""));

    // --- school selection option ---
    let school_prefix = if state.selected_index == 1 { "▸ " } else { "  " };
    let school_style = if state.selected_index == 1 {
        Style::default()
            .fg(theme.selected_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_color)
    };

    let school_name = if let Some(school_id) = state.selected_school_id {
        state
            .available_schools
            .iter()
            .find(|s| s.id == school_id)
            .map(|s| s.name.as_str())
            .unwrap_or("Unknown")
    } else if state.available_schools.is_empty() {
        "No schools (sync first)"
    } else {
        "None selected"
    };

    let school_hint = if state.school_picker_open {
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
    if state.school_picker_open && !state.available_schools.is_empty() {
        lines.push(Line::from(""));
        let total = state.available_schools.len();
        let start = state.school_scroll_offset;
        let end = (start + PICKER_MAX_VISIBLE).min(total);
        
        // show scroll indicator at top if not at beginning
        if start > 0 {
            lines.push(Line::from(Span::styled(
                format!("     ↑ {} more above", start),
                Style::default().fg(theme.muted_color),
            )));
        }
        
        for (i, school) in state.available_schools.iter().enumerate().skip(start).take(end - start) {
            let is_selected = i == state.selected_school_index;
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
    let term_prefix = if state.selected_index == 2 { "▸ " } else { "  " };
    let term_style = if state.selected_index == 2 {
        Style::default()
            .fg(theme.selected_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_color)
    };

    let term_name = if let Some(term_id) = state.selected_term_id {
        state
            .available_terms
            .iter()
            .find(|t| t.id == term_id)
            .map(|t| t.name.as_str())
            .unwrap_or("Unknown")
    } else if state.selected_school_id.is_none() {
        "Select school first"
    } else if state.available_terms.is_empty() {
        "No terms available"
    } else {
        "None selected"
    };

    let term_hint = if state.term_picker_open {
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
    if state.term_picker_open && !state.available_terms.is_empty() {
        lines.push(Line::from(""));
        let total = state.available_terms.len();
        let start = state.term_scroll_offset;
        let end = (start + PICKER_MAX_VISIBLE).min(total);
        
        // show scroll indicator at top if not at beginning
        if start > 0 {
            lines.push(Line::from(Span::styled(
                format!("     ↑ {} more above", start),
                Style::default().fg(theme.muted_color),
            )));
        }
        
        for (i, term) in state.available_terms.iter().enumerate().skip(start).take(end - start) {
            let is_selected = i == state.selected_term_index;
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
    let sync_prefix = if state.selected_index == 3 { "▸ " } else { "  " };
    let sync_style = if state.selected_index == 3 {
        Style::default()
            .fg(theme.selected_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_color)
    };

    let sync_status = if state.is_syncing {
        Span::styled("Syncing...", Style::default().fg(theme.warning_color))
    } else {
        Span::styled("[Press Enter to sync]", Style::default().fg(theme.success_color))
    };

    lines.push(Line::from(vec![
        Span::styled(sync_prefix, sync_style),
        Span::styled("Sync Data: ", sync_style),
        sync_status,
    ]));
    lines.push(Line::from(""));

    // --- last sync time ---
    let sync_time_display = match state.last_sync_time {
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
