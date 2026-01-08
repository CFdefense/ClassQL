/// src/tui/widgets/menu.rs
///
/// Main menu widget rendering
///
/// Renders the main menu with options
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Menu option enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuOption {
    Search,
    Help,
    Settings,
    Quit,
}

impl MenuOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuOption::Search => "Search Classes",
            MenuOption::Help => "Help",
            MenuOption::Settings => "Settings",
            MenuOption::Quit => "Quit",
        }
    }

    pub fn all() -> Vec<MenuOption> {
        vec![
            MenuOption::Search,
            MenuOption::Help,
            MenuOption::Settings,
            MenuOption::Quit,
        ]
    }
}

/// Render the main menu
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// selected_index -> The index of the currently selected menu option
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_main_menu(frame: &mut Frame, selected_index: usize, theme: &Theme) {
    let menu_options = MenuOption::all();
    let menu_width = 40_u16;
    let menu_height = (menu_options.len() as u16 + 4).min(10); // options + borders + title

    // Position menu below the logo
    // Logo is 7 lines tall and positioned near the top
    let logo_height = 7_u16;
    let spacing = 3_u16; // spacing between logo and menu
    let menu_y = logo_height + spacing;

    let frame_width = frame.area().width;
    let frame_height = frame.area().height;

    // clamp menu dimensions to fit within frame
    let menu_area = Rect {
        x: (frame_width.saturating_sub(menu_width.min(frame_width))) / 2,
        y: menu_y.min(frame_height.saturating_sub(menu_height.min(frame_height))),
        width: menu_width.min(frame_width),
        height: menu_height.min(frame_height),
    }.intersection(frame.area()); // ensure it's within frame bounds

    let mut styled_lines = Vec::new();
    for (i, option) in menu_options.iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected { "> " } else { "  " };

        let style = if is_selected {
            Style::default()
                .fg(theme.selected_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_color)
        };

        styled_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(option.as_str(), style),
        ]));
    }

    let menu_paragraph = Paragraph::new(styled_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Main Menu ")
            .title_style(
                Style::default()
                    .fg(theme.title_color)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(theme.border_color)),
    );

    frame.render_widget(menu_paragraph, menu_area);
}
