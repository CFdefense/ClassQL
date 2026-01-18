/// src/tui/widgets/menu.rs
///
/// Main menu widget with encapsulated state and input handling
///
/// Contains:
/// --- ---
/// MainMenuWidget -> Widget for the main menu with navigation
/// MenuOption -> Enum for menu options
/// --- ---
/// 
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::Theme;
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Menu option enum representing available main menu choices
///
/// Variants:
/// --- ---
/// Search -> Navigate to class search view
/// ScheduleCreation -> Navigate to schedule creation (requires cart items)
/// MySchedules -> View saved schedules
/// Help -> View the query guide/help
/// Settings -> Navigate to settings (theme, school, term, sync)
/// Quit -> Exit the application
/// --- ---
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuOption {
    Search,
    ScheduleCreation,
    MySchedules,
    Help,
    Settings,
    Quit,
}

impl MenuOption {
    /// Convert menu option to a string
    ///
    /// Returns:
    /// --- ---
    /// &'static str -> The string representation of the menu option
    /// --- ---
    ///
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuOption::Search => "Search Classes",
            MenuOption::ScheduleCreation => "Create Schedule",
            MenuOption::MySchedules => "My Schedules",
            MenuOption::Help => "Help",
            MenuOption::Settings => "Settings",
            MenuOption::Quit => "Quit",
        }
    }

    /// Get all menu options
    ///
    /// Returns:
    /// --- ---
    /// Vec<MenuOption> -> All menu options
    /// --- ---
    ///
    pub fn all() -> Vec<MenuOption> {
        vec![
            MenuOption::Search,
            MenuOption::ScheduleCreation,
            MenuOption::MySchedules,
            MenuOption::Help,
            MenuOption::Settings,
            MenuOption::Quit,
        ]
    }

    /// Convert menu option to the corresponding focus mode (if applicable)
    ///
    /// Returns:
    /// --- ---
    /// Option<FocusMode> -> The focus mode corresponding to the menu option
    /// --- ---
    ///
    pub fn to_focus_mode(&self) -> Option<FocusMode> {
        match self {
            MenuOption::Search => Some(FocusMode::QueryInput),
            MenuOption::ScheduleCreation => Some(FocusMode::ScheduleCreation),
            MenuOption::MySchedules => Some(FocusMode::MySchedules),
            MenuOption::Help => Some(FocusMode::QueryGuide),
            MenuOption::Settings => Some(FocusMode::Settings),
            MenuOption::Quit => None, // Quit exits the app
        }
    }
}

/// Main menu widget with encapsulated state
///
/// Handles navigation between the main application views and validates
/// that required conditions are met before navigation (e.g., cart not empty).
///
/// Fields:
/// --- ---
/// selected_index -> Index of currently selected menu option
/// cart_empty -> Whether the cart is empty (for schedule creation validation)
/// --- ---
///
pub struct MainMenuWidget {
    pub selected_index: usize,
    pub cart_empty: bool,
}

impl MainMenuWidget {
    /// Create a new MainMenuWidget
    ///
    /// Returns:
    /// --- ---
    /// MainMenuWidget -> The new MainMenuWidget
    /// --- ---
    ///
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            cart_empty: true,
        }
    }

    /// Update cart status
    /// 
    /// Arguments:
    /// --- ---
    /// empty -> Whether the cart is empty
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn set_cart_empty(&mut self, empty: bool) {
        self.cart_empty = empty;
    }

    /// Get the currently selected menu option
    ///
    /// Returns:
    /// --- ---
    /// MenuOption -> The currently selected menu option
    /// --- ---
    ///
    pub fn selected_option(&self) -> MenuOption {
        let options = MenuOption::all();
        options[self.selected_index.min(options.len() - 1)]
    }
}

impl Widget for MainMenuWidget {
    /// Render the main menu
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
        let menu_options = MenuOption::all();
        let menu_width = 40_u16;
        let menu_height = (menu_options.len() as u16 + 4).min(10);

        // position menu below the logo
        let logo_height = 7_u16;
        let spacing = 6_u16;
        let menu_y = logo_height + spacing;

        let frame_width = frame.area().width;
        let frame_height = frame.area().height;

        let menu_area = Rect {
            x: (frame_width.saturating_sub(menu_width.min(frame_width))) / 2,
            y: menu_y.min(frame_height.saturating_sub(menu_height.min(frame_height))),
            width: menu_width.min(frame_width),
            height: menu_height.min(frame_height),
        }
        .intersection(frame.area());

        let mut styled_lines = Vec::new();
        for (i, option) in menu_options.iter().enumerate() {
            let is_selected = i == self.selected_index;
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

    /// Handle a key event and return an action
    ///
    /// Arguments:
    /// --- ---
    /// key -> The key event to handle
    /// --- ---
    ///
    /// Returns: KeyAction -> The action to take in response to the key
    ///
    fn handle_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Exit,
            KeyCode::Esc => KeyAction::Exit,
            KeyCode::Up => {
                let options_len = MenuOption::all().len();
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.selected_index = options_len - 1;
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                let options_len = MenuOption::all().len();
                if self.selected_index < options_len - 1 {
                    self.selected_index += 1;
                } else {
                    self.selected_index = 0;
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                let option = self.selected_option();
                match option {
                    MenuOption::Quit => KeyAction::Exit,
                    MenuOption::ScheduleCreation => {
                        if self.cart_empty {
                            KeyAction::ShowToast {
                                message: "Cart is empty! Add classes to cart first.".to_string(),
                                error_type: ErrorType::Semantic,
                            }
                        } else {
                            KeyAction::Navigate(FocusMode::ScheduleCreation)
                        }
                    }
                    _ => {
                        if let Some(mode) = option.to_focus_mode() {
                            KeyAction::Navigate(mode)
                        } else {
                            KeyAction::Continue
                        }
                    }
                }
            }
            _ => KeyAction::Continue,
        }
    }

    /// Return the focus mode(s) this widget handles
    ///
    /// Returns:
    /// --- ---
    /// Vec<FocusMode> -> The focus modes this widget handles
    /// --- ---
    ///
    fn focus_modes(&self) -> Vec<FocusMode> {
        vec![FocusMode::MainMenu]
    }
}
