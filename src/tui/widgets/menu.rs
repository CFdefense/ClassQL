/// src/tui/widgets/menu.rs
///
/// Main menu widget rendering
///
/// Renders the main menu with options

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Menu option enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuOption {
    Search,
    Help,
    Quit,
}

impl MenuOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuOption::Search => "Search Classes",
            MenuOption::Help => "Help",
            MenuOption::Quit => "Quit",
        }
    }

    pub fn all() -> Vec<MenuOption> {
        vec![MenuOption::Search, MenuOption::Help, MenuOption::Quit]
    }
}

/// Render the main menu
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// selected_index -> The index of the currently selected menu option
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_main_menu(frame: &mut Frame, selected_index: usize) {
    let menu_options = MenuOption::all();
    let menu_width = 40_u16;
    let menu_height = (menu_options.len() as u16 + 4).min(10); // options + borders + title

    let menu_area = Rect {
        x: (frame.area().width.saturating_sub(menu_width)) / 2,
        y: (frame.area().height.saturating_sub(menu_height)) / 2,
        width: menu_width,
        height: menu_height,
    };

    let mut styled_lines = Vec::new();
    for (i, option) in menu_options.iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected {
            "> "
        } else {
            "  "
        };
        
        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
        };

        styled_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(option.as_str(), style),
        ]));
    }

    let menu_paragraph = Paragraph::new(styled_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Main Menu ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Cyan))
        );

    frame.render_widget(menu_paragraph, menu_area);
}

