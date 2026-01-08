/// src/tui/widgets/settings.rs
///
/// Settings widget rendering
///
/// Renders the settings menu
use crate::tui::themes::{Theme, ThemePalette};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the settings menu
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// current_theme -> The currently selected theme palette
/// theme -> The current theme
/// selected_index -> The currently selected settings option
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_settings(
    frame: &mut Frame,
    current_theme: ThemePalette,
    theme: &Theme,
    selected_index: usize,
) {
    let settings_width = 50_u16;
    let settings_height = 15_u16;

    // Position settings below the logo
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
    }.intersection(frame.area()); // ensure it's within frame bounds

    let mut lines = Vec::new();

    // Settings options
    let theme_prefix = if selected_index == 0 { "> " } else { "  " };
    let theme_style = if selected_index == 0 {
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
            current_theme.as_str(),
            Style::default().fg(theme.warning_color),
        ),
        Span::styled(" (← → to change)", Style::default().fg(theme.muted_color)),
    ]));
    lines.push(Line::from(""));

    let autocomplete_prefix = if selected_index == 1 { "> " } else { "  " };
    let autocomplete_style = if selected_index == 1 {
        Style::default()
            .fg(theme.selected_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_color)
    };
    lines.push(Line::from(vec![
        Span::styled(autocomplete_prefix, autocomplete_style),
        Span::styled("Auto-complete: ", autocomplete_style),
        Span::styled("TBD", Style::default().fg(theme.muted_color)),
    ]));
    lines.push(Line::from(""));

    let school_prefix = if selected_index == 2 { "> " } else { "  " };
    let school_style = if selected_index == 2 {
        Style::default()
            .fg(theme.selected_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_color)
    };
    lines.push(Line::from(vec![
        Span::styled(school_prefix, school_style),
        Span::styled("Selected school: ", school_style),
        Span::styled("None", Style::default().fg(theme.warning_color)),
    ]));
    lines.push(Line::from(""));

    let sync_prefix = if selected_index == 3 { "> " } else { "  " };
    let sync_style = if selected_index == 3 {
        Style::default()
            .fg(theme.selected_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_color)
    };
    lines.push(Line::from(vec![
        Span::styled(sync_prefix, sync_style),
        Span::styled("Sync: ", sync_style),
        Span::styled("N/A", Style::default().fg(theme.muted_color)),
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
