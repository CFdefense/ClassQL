/// src/tui/widgets/schedule.rs
///
/// Schedule widget with encapsulated state, input handling, and rendering
///
/// Handles schedule creation, cart management, schedule viewing, and generation
///
/// Contains:
/// --- ---
/// ScheduleWidget -> Widget for schedule functionality
/// ScheduleAction -> Actions returned by schedule widget
/// --- ---
use crate::data::sql::Class;
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::Theme;
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::collections::{HashMap, HashSet};

/// Schedule widget with encapsulated state
///
/// Manages the schedule creation workflow including cart management,
/// schedule generation, and viewing of both generated and saved schedules.
///
/// Fields:
/// --- ---
/// cart_classes -> Map of all classes in the cart (ID -> Class)
/// selected_for_schedule -> Set of class IDs selected for schedule generation
/// generated_schedules -> All generated non-conflicting schedules
/// current_schedule_index -> Index of currently displayed schedule
/// schedule_cart_focus -> Whether the cart is focused
/// selected_cart_index -> Index of currently selected cart item
/// schedule_selection_mode -> Whether in class selection mode (true) or viewing mode (false)
/// selected_time_block_day -> Index of currently selected day in schedule viewing mode
/// selected_time_block_slot -> Index of currently selected time slot
/// current_saved_schedule_name -> Name of currently viewed saved schedule (if any)
/// saved_schedule_names -> All saved schedule names (for viewing saved schedules)
/// viewing_saved_schedules -> Whether viewing saved schedules (vs generated schedules)
/// detail_return_focus -> Focus mode to return to after detail view
/// --- ---
///
pub struct ScheduleWidget {
    pub cart_classes: HashMap<String, Class>,
    pub selected_for_schedule: HashSet<String>,
    pub generated_schedules: Vec<Vec<Class>>,
    pub current_schedule_index: usize,
    pub schedule_cart_focus: bool,
    pub selected_cart_index: usize,
    pub schedule_selection_mode: bool,
    pub selected_time_block_day: usize,
    pub selected_time_block_slot: usize,
    pub current_saved_schedule_name: Option<String>,
    pub saved_schedule_names: Vec<String>,
    pub viewing_saved_schedules: bool,
    pub detail_return_focus: FocusMode,
}

/// Action returned by schedule widget for app-level handling
///
/// Variants:
/// --- ---
/// None -> No action needed
/// OpenDetailView -> Open detail view for a class
/// SaveSchedule -> Request to save current schedule
/// RefreshSavedSchedules -> Need to refresh saved schedules from MySchedules navigation
/// --- ---
///
#[derive(Debug, Clone)]
pub enum ScheduleAction {
    None,
    OpenDetailView(Class),
    SaveSchedule,
    RefreshSavedSchedules,
}

impl ScheduleWidget {
    /// Create a new ScheduleWidget
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Self -> new ScheduleWidget with default state
    /// --- ---
    ///
    pub fn new() -> Self {
        Self {
            cart_classes: HashMap::new(),
            selected_for_schedule: HashSet::new(),
            generated_schedules: Vec::new(),
            current_schedule_index: 0,
            schedule_cart_focus: true,
            selected_cart_index: 0,
            schedule_selection_mode: true,
            selected_time_block_day: 0,
            selected_time_block_slot: 0,
            current_saved_schedule_name: None,
            saved_schedule_names: Vec::new(),
            viewing_saved_schedules: false,
            detail_return_focus: FocusMode::ScheduleCreation,
        }
    }

    /// Check if cart is empty
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// bool -> true if cart has no classes
    /// --- ---
    ///
    pub fn is_cart_empty(&self) -> bool {
        self.cart_classes.is_empty()
    }

    /// Add a class to the cart
    ///
    /// Arguments:
    /// --- ---
    /// class -> Class to add
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn add_to_cart(&mut self, class: Class) {
        let id = class.unique_id();
        self.cart_classes.insert(id, class);
    }

    /// Remove a class from the cart
    ///
    /// Arguments:
    /// --- ---
    /// class_id -> unique ID of class to remove
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn remove_from_cart(&mut self, class_id: &str) {
        self.cart_classes.remove(class_id);
        self.selected_for_schedule.remove(class_id);
    }

    /// Toggle cart status for a class
    ///
    /// Arguments:
    /// --- ---
    /// class -> Class to toggle (add if missing, remove if present)
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn toggle_cart(&mut self, class: &Class) {
        let id = class.unique_id();
        if self.cart_classes.contains_key(&id) {
            self.cart_classes.remove(&id);
            self.selected_for_schedule.remove(&id);
        } else {
            self.cart_classes.insert(id, class.clone());
        }
    }

    /// Clear cart and related data (when switching schools/terms)
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    pub fn clear(&mut self) {
        self.cart_classes.clear();
        self.selected_for_schedule.clear();
        self.generated_schedules.clear();
        self.current_schedule_index = 0;
        self.selected_cart_index = 0;
    }

    /// Enter schedule creation mode from main menu
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    pub fn enter_creation_mode(&mut self) {
        // initialize selected_for_schedule with all cart items if empty
        if self.selected_for_schedule.is_empty() {
            self.selected_for_schedule = self.cart_classes.keys().cloned().collect();
        }
        self.schedule_selection_mode = true;
        self.current_schedule_index = 0;
        self.schedule_cart_focus = true;
        self.selected_cart_index = 0;
        self.generated_schedules.clear();
        self.current_saved_schedule_name = None;
        self.saved_schedule_names.clear();
        self.viewing_saved_schedules = false;
        self.detail_return_focus = FocusMode::ScheduleCreation;
    }

    /// Load saved schedules for viewing
    ///
    /// Arguments:
    /// --- ---
    /// all_schedules -> all saved schedules (classes for each)
    /// all_names -> names of all saved schedules
    /// selected_index -> index of the schedule to display initially
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn load_saved_schedules(
        &mut self,
        all_schedules: Vec<Vec<Class>>,
        all_names: Vec<String>,
        selected_index: usize,
    ) {
        self.generated_schedules = all_schedules;
        self.saved_schedule_names = all_names;
        self.current_schedule_index = selected_index;
        self.schedule_selection_mode = false;
        self.viewing_saved_schedules = true;
        self.selected_time_block_day = 0;
        self.selected_time_block_slot = 0;
        self.current_saved_schedule_name = self.saved_schedule_names.get(selected_index).cloned();
        self.detail_return_focus = FocusMode::MySchedules;
    }

    /// Get sorted cart class IDs (for consistent ordering)
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Vec<String> -> sorted list of cart class IDs
    /// --- ---
    ///
    fn sorted_cart_ids(&self) -> Vec<String> {
        let mut cart_classes_vec: Vec<&Class> = self.cart_classes.values().collect();
        cart_classes_vec.sort_by_key(|class| class.unique_id());
        cart_classes_vec
            .iter()
            .map(|class| class.unique_id())
            .collect()
    }

    /// Handle key with action return
    ///
    /// Arguments:
    /// --- ---
    /// key -> the key event to handle
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> tuple of key action and schedule action
    /// --- ---
    ///
    pub fn handle_key_with_action(&mut self, key: KeyEvent) -> (KeyAction, ScheduleAction) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                (KeyAction::Exit, ScheduleAction::None)
            }
            KeyCode::Esc => self.handle_esc(),
            KeyCode::Up => self.handle_up(),
            KeyCode::Down => self.handle_down(),
            KeyCode::Left => self.handle_left(),
            KeyCode::Right => self.handle_right(),
            KeyCode::PageUp => self.handle_page_up(),
            KeyCode::PageDown => self.handle_page_down(),
            KeyCode::Enter => self.handle_enter(),
            KeyCode::Char('s') | KeyCode::Char('S') => self.handle_save(),
            KeyCode::Char(' ') => self.handle_space(),
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('c') | KeyCode::Char('C') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    (KeyAction::Exit, ScheduleAction::None)
                } else {
                    self.handle_delete()
                }
            }
            KeyCode::Tab => self.handle_tab(),
            _ => (KeyAction::Continue, ScheduleAction::None),
        }
    }

    /// Handle Escape key - exit creation mode or return to previous view
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> navigation or continue action
    /// --- ---
    ///
    fn handle_esc(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // exit schedule creation, go back to main menu
            (
                KeyAction::Navigate(FocusMode::MainMenu),
                ScheduleAction::None,
            )
        } else {
            // check if we came from MySchedules
            if self.detail_return_focus == FocusMode::MySchedules {
                // go back to MySchedules view
                self.current_saved_schedule_name = None;
                (
                    KeyAction::Navigate(FocusMode::MySchedules),
                    ScheduleAction::None,
                )
            } else {
                // go back to class selection mode
                self.schedule_selection_mode = true;
                self.schedule_cart_focus = true;
                self.generated_schedules.clear();
                (KeyAction::Continue, ScheduleAction::None)
            }
        }
    }

    /// Handle Up key - navigate cart items or time slots
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_up(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // navigate cart items up
            if !self.cart_classes.is_empty() && self.selected_cart_index > 0 {
                self.selected_cart_index -= 1;
            }
        } else {
            // navigate time blocks: up = previous time slot
            if self.selected_time_block_slot > 0 {
                self.selected_time_block_slot -= 1;
            } else {
                // wrap to last time slot
                self.selected_time_block_slot = 28; // 29 time slots (0-28) for 8am-10:30pm
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle Down key - navigate cart items or time slots
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_down(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // navigate cart items down
            let cart_ids = self.sorted_cart_ids();
            if !cart_ids.is_empty() && self.selected_cart_index < cart_ids.len() - 1 {
                self.selected_cart_index += 1;
            }
        } else {
            // navigate time blocks: down = next time slot
            if self.selected_time_block_slot < 28 {
                self.selected_time_block_slot += 1;
            } else {
                // wrap to first time slot
                self.selected_time_block_slot = 0;
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle Left key - navigate days in schedule view
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_left(&mut self) -> (KeyAction, ScheduleAction) {
        if !self.schedule_selection_mode {
            // navigate time blocks: left = previous day
            if self.selected_time_block_day > 0 {
                self.selected_time_block_day -= 1;
            } else {
                // wrap to Sunday
                self.selected_time_block_day = 6;
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle Right key - navigate days in schedule view
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_right(&mut self) -> (KeyAction, ScheduleAction) {
        if !self.schedule_selection_mode {
            // navigate time blocks: right = next day
            if self.selected_time_block_day < 6 {
                self.selected_time_block_day += 1;
            } else {
                // wrap to Monday
                self.selected_time_block_day = 0;
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle PageUp key - previous schedule
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_page_up(&mut self) -> (KeyAction, ScheduleAction) {
        if !self.schedule_selection_mode && !self.generated_schedules.is_empty() {
            if self.current_schedule_index > 0 {
                self.current_schedule_index -= 1;
            } else {
                self.current_schedule_index = self.generated_schedules.len() - 1;
            }
            // update current saved schedule name when viewing saved schedules
            if self.viewing_saved_schedules {
                self.current_saved_schedule_name = self
                    .saved_schedule_names
                    .get(self.current_schedule_index)
                    .cloned();
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle PageDown key - next schedule
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_page_down(&mut self) -> (KeyAction, ScheduleAction) {
        if !self.schedule_selection_mode && !self.generated_schedules.is_empty() {
            if self.current_schedule_index < self.generated_schedules.len() - 1 {
                self.current_schedule_index += 1;
            } else {
                self.current_schedule_index = 0;
            }
            // update current saved schedule name when viewing saved schedules
            if self.viewing_saved_schedules {
                self.current_saved_schedule_name = self
                    .saved_schedule_names
                    .get(self.current_schedule_index)
                    .cloned();
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle Enter key - generate schedules or view class details
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> navigation or toast action
    /// --- ---
    ///
    fn handle_enter(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // generate schedules and switch to viewing mode
            if self.selected_for_schedule.is_empty() {
                return (
                    KeyAction::ShowToast {
                        message: "No classes selected! Select classes first.".to_string(),
                        error_type: ErrorType::Semantic,
                    },
                    ScheduleAction::None,
                );
            }

            // generate valid (non-conflicting) schedules
            self.generated_schedules =
                generate_schedules(&self.cart_classes, &self.selected_for_schedule, false);

            if self.generated_schedules.is_empty() {
                // no valid schedules found - show which classes conflict
                let selected_classes: Vec<Class> = self
                    .selected_for_schedule
                    .iter()
                    .filter_map(|class_id| self.cart_classes.get(class_id))
                    .cloned()
                    .collect();
                let conflicts = find_conflicting_classes(&selected_classes);
                let conflict_msg = if conflicts.len() == 1 {
                    format!(
                        "No valid schedules. Classes conflict: {} and {}",
                        conflicts[0].0, conflicts[0].1
                    )
                } else {
                    let mut msg = "No valid schedules. Classes conflict: ".to_string();
                    for (i, (class1, class2)) in conflicts.iter().enumerate() {
                        if i > 0 {
                            msg.push_str(", ");
                        }
                        msg.push_str(&format!("{} & {}", class1, class2));
                    }
                    msg
                };
                return (
                    KeyAction::ShowToast {
                        message: conflict_msg,
                        error_type: ErrorType::Semantic,
                    },
                    ScheduleAction::None,
                );
            }

            // valid schedules found - proceed to viewing mode
            self.schedule_selection_mode = false;
            self.current_schedule_index = 0;
            self.selected_time_block_day = 0;
            self.selected_time_block_slot = 0;
            (KeyAction::Continue, ScheduleAction::None)
        } else {
            // show class details in detail view
            if !self.generated_schedules.is_empty()
                && self.current_schedule_index < self.generated_schedules.len()
            {
                let schedule = &self.generated_schedules[self.current_schedule_index];
                if let Some(class) = find_class_at_time_block(
                    schedule,
                    self.selected_time_block_day,
                    self.selected_time_block_slot,
                ) {
                    return (
                        KeyAction::Navigate(FocusMode::DetailView),
                        ScheduleAction::OpenDetailView(class.clone()),
                    );
                }
            }
            (KeyAction::Continue, ScheduleAction::None)
        }
    }

    /// Handle Save key - save current schedule
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> navigation to save input or continue
    /// --- ---
    ///
    fn handle_save(&mut self) -> (KeyAction, ScheduleAction) {
        // don't allow saving already-saved schedules
        if self.viewing_saved_schedules {
            return (KeyAction::Continue, ScheduleAction::None);
        }
        if !self.schedule_selection_mode && !self.generated_schedules.is_empty() {
            // save current schedule - enter name input mode
            (
                KeyAction::Navigate(FocusMode::SaveNameInput),
                ScheduleAction::SaveSchedule,
            )
        } else {
            (KeyAction::Continue, ScheduleAction::None)
        }
    }

    /// Handle Space key - toggle class selection
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_space(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // toggle selected cart item for schedule generation
            let cart_ids = self.sorted_cart_ids();
            if self.selected_cart_index < cart_ids.len() {
                let class_id = &cart_ids[self.selected_cart_index];
                if self.selected_for_schedule.contains(class_id) {
                    self.selected_for_schedule.remove(class_id);
                } else {
                    self.selected_for_schedule.insert(class_id.clone());
                }
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle Delete key - remove class from cart
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> continue action
    /// --- ---
    ///
    fn handle_delete(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // remove selected cart item from cart
            let cart_ids = self.sorted_cart_ids();
            if self.selected_cart_index < cart_ids.len() {
                let class_id = cart_ids[self.selected_cart_index].clone();
                self.cart_classes.remove(&class_id);
                self.selected_for_schedule.remove(&class_id);

                // adjust selected index if needed
                if self.selected_cart_index >= self.cart_classes.len()
                    && !self.cart_classes.is_empty()
                {
                    self.selected_cart_index = self.cart_classes.len() - 1;
                } else if self.cart_classes.is_empty() {
                    self.selected_cart_index = 0;
                }
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Handle Tab key - open detail view for selected class
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// (KeyAction, ScheduleAction) -> navigation to detail view or continue
    /// --- ---
    ///
    fn handle_tab(&mut self) -> (KeyAction, ScheduleAction) {
        if self.schedule_selection_mode {
            // open detail view for selected class
            let cart_ids = self.sorted_cart_ids();
            if self.selected_cart_index < cart_ids.len() {
                let class_id = &cart_ids[self.selected_cart_index];
                if let Some(class) = self.cart_classes.get(class_id) {
                    self.detail_return_focus = FocusMode::ScheduleCreation;
                    return (
                        KeyAction::Navigate(FocusMode::DetailView),
                        ScheduleAction::OpenDetailView(class.clone()),
                    );
                }
            }
        }
        (KeyAction::Continue, ScheduleAction::None)
    }

    /// Get current schedule for saving
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Option<&Vec<Class>> -> reference to current schedule or None
    /// --- ---
    ///
    pub fn current_schedule(&self) -> Option<&Vec<Class>> {
        if !self.generated_schedules.is_empty()
            && self.current_schedule_index < self.generated_schedules.len()
        {
            Some(&self.generated_schedules[self.current_schedule_index])
        } else {
            None
        }
    }

    /// Render the schedule creation interface
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_schedule(&self, frame: &mut Frame, theme: &Theme) {
        let frame_width = frame.area().width;
        let frame_height = frame.area().height;

        // position below logo at top (logo is 7 lines tall, add spacing)
        let logo_height = 7_u16;
        let spacing = 6_u16;
        let start_y = logo_height + spacing;

        // calculate size - use full available height for schedule viewing
        let max_width = 90_u16.min(frame_width.saturating_sub(4)); // leave margins, max 90 chars wide
        let max_height = if self.schedule_selection_mode {
            // in selection mode, limit height for cart
            (frame_height.saturating_sub(start_y + 3)).min(20)
        } else {
            // in viewing mode, use full available height for calendar
            // only reserve minimal space for help text (1 line) and gap/counter (2 lines)
            frame_height.saturating_sub(start_y + 1 + 2) // start_y + help text + gap/counter
        };
        let time_col_width = 7_u16;
        let logo_shift = 1_u16; // logo is shifted 1 space to the right
        let schedule_x =
            (frame_width.saturating_sub(max_width)) / 2 + time_col_width / 2 + logo_shift;

        let area = Rect {
            x: schedule_x,
            y: start_y,
            width: max_width,
            height: max_height,
        };

        if self.schedule_selection_mode {
            // in selection mode, show only cart (narrower width)
            let cart_width = 35_u16.min(frame_width.saturating_sub(4));
            let message_width = 50_u16.min(frame_width.saturating_sub(4)); // wider for messages
            let cart_x = (frame_width.saturating_sub(cart_width)) / 2;
            let message_x = (frame_width.saturating_sub(message_width)) / 2;
            // calculate cart height (leave room for messages below)
            let cart_height = (max_height.saturating_sub(4)).min(15); // leave 4 lines for messages
            let cart_area = Rect {
                x: cart_x,
                y: start_y,
                width: cart_width,
                height: cart_height,
            };

            // position messages below cart
            let message_y = start_y + cart_height + 1;
            let message_area = Rect {
                x: message_x,
                y: message_y,
                width: message_width,
                height: 3, // 3 lines for messages
            };
            self.render_cart_section(frame, cart_area, message_area, theme);
        } else {
            // in viewing mode, show time-block calendar
            // if schedule name is provided, render it above the schedule with a gap
            let schedule_area = if let Some(ref name) = self.current_saved_schedule_name {
                // render schedule name above the schedule
                let name_height = 1;
                let gap_height = 2; // nice gap between name and schedule
                let name_y = start_y;
                let schedule_y = name_y + name_height + gap_height;

                // adjust schedule area to account for name and gap
                let adjusted_height = max_height.saturating_sub(name_height + gap_height);
                let name_area = Rect {
                    x: schedule_x,
                    y: name_y,
                    width: max_width,
                    height: name_height,
                };

                // render the schedule name
                let name_para = Paragraph::new(name.as_str())
                    .style(
                        Style::default()
                            .fg(theme.title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .alignment(Alignment::Center);
                frame.render_widget(name_para, name_area);

                // return adjusted area for schedule
                Rect {
                    x: schedule_x,
                    y: schedule_y,
                    width: max_width,
                    height: adjusted_height,
                }
            } else {
                // no schedule name, use original area
                area
            };

            // get saved schedule index and total when viewing saved schedules
            let (saved_idx, total_saved) =
                if self.viewing_saved_schedules && !self.saved_schedule_names.is_empty() {
                    (
                        Some(self.current_schedule_index),
                        Some(self.saved_schedule_names.len()),
                    )
                } else {
                    (None, None)
                };

            if !self.generated_schedules.is_empty()
                && self.current_schedule_index < self.generated_schedules.len()
            {
                self.render_time_block_calendar(
                    frame,
                    schedule_area,
                    &self.generated_schedules[self.current_schedule_index],
                    self.current_schedule_index,
                    self.generated_schedules.len(),
                    self.selected_time_block_day,
                    self.selected_time_block_slot,
                    saved_idx,
                    total_saved,
                    theme,
                );
            } else {
                self.render_empty_schedule_section(frame, schedule_area, true, theme);
            }
        }
    }

    /// Render cart section
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// cart_area -> the area to render the cart in
    /// message_area -> the area to render messages below the cart
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_cart_section(
        &self,
        frame: &mut Frame,
        cart_area: Rect,
        message_area: Rect,
        theme: &Theme,
    ) {
        let cart_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(cart_area);

        let message_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(message_area);

        let border_color = if self.schedule_cart_focus {
            theme.selected_color
        } else {
            theme.border_color
        };

        // cart items - get classes from cart_classes map, sorted by ID for consistent ordering
        let mut cart_classes_vec: Vec<&Class> = self.cart_classes.values().collect();
        // sort by unique_id for consistent ordering
        cart_classes_vec.sort_by_key(|class| class.unique_id());

        let cart_text = if cart_classes_vec.is_empty() {
            vec![Line::from(Span::styled(
                "Cart is empty",
                Style::default().fg(theme.muted_color),
            ))]
        } else {
            cart_classes_vec
                .iter()
                .enumerate()
                .map(|(idx, class)| {
                    let is_selected = self.schedule_cart_focus && idx == self.selected_cart_index;
                    let class_id = class.unique_id();
                    let checkbox = if self.selected_for_schedule.contains(&class_id) {
                        "☑ "
                    } else {
                        "☐ "
                    };
                    let prefix = if is_selected { "> " } else { "  " };
                    let base_style = if is_selected {
                        Style::default()
                            .fg(theme.selected_color)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(theme.text_color)
                            .add_modifier(Modifier::BOLD)
                    };
                    Line::from(vec![
                        Span::styled(prefix, base_style),
                        Span::styled(checkbox, base_style),
                        Span::styled(
                            format!(
                                "{} {}-{}",
                                class.subject_code, class.course_number, class.section_sequence
                            ),
                            base_style,
                        ),
                    ])
                })
                .collect()
        };

        // add tiny gap at top, then classes from top down
        let mut padded_text: Vec<Line> = vec![Line::from("")]; // tiny gap
        padded_text.extend(cart_text);

        let cart_widget = Paragraph::new(padded_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Cart ")
                    .title_style(
                        Style::default()
                            .fg(theme.title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().bg(theme.background_color))
            .alignment(Alignment::Center);
        frame.render_widget(cart_widget, cart_chunks[0]);

        // messages below cart (using message_area for proper width)
        let message1 = Paragraph::new("Select desired classes to build a schedule")
            .style(Style::default().fg(theme.muted_color))
            .alignment(Alignment::Center);
        frame.render_widget(message1, message_chunks[0]);

        // empty line for gap
        let empty_line = Paragraph::new("").style(Style::default().fg(theme.background_color));
        frame.render_widget(empty_line, message_chunks[1]);

        // message to press enter to continue
        let message2 = Paragraph::new("Press Enter to continue")
            .style(Style::default().fg(theme.info_color))
            .alignment(Alignment::Center);
        frame.render_widget(message2, message_chunks[2]);
    }

    /// Render time-block calendar view
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// area -> the area to render the calendar in
    /// schedule -> the schedule classes to display
    /// current_index -> index of currently displayed schedule
    /// total_schedules -> total number of schedules available
    /// selected_day -> selected day index (0-6 for Mon-Sun)
    /// selected_slot -> selected time slot index
    /// saved_schedule_index -> optional index for saved schedules
    /// total_saved_schedules -> optional total saved schedules count
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_time_block_calendar(
        &self,
        frame: &mut Frame,
        area: Rect,
        schedule: &[Class],
        current_index: usize,
        total_schedules: usize,
        selected_day: usize,
        selected_slot: usize,
        saved_schedule_index: Option<usize>,
        total_saved_schedules: Option<usize>,
        theme: &Theme,
    ) {
        // use the full area for the calendar, we'll position the counter manually
        let calendar_area = area;

        // time slots: 8am to 10:30pm, 30-minute intervals = 30 slots
        let time_slots: Vec<(i32, String)> = (16..46) // 8:00 am - 10:30pm
            .map(|half_hour| {
                let hours = half_hour / 2;
                let minutes = (half_hour % 2) * 30;
                let (display_hour, period) = if hours == 0 {
                    (12, "am")
                } else if hours < 12 {
                    (hours, "am")
                } else if hours == 12 {
                    (12, "pm")
                } else {
                    (hours - 12, "pm")
                };
                // format with leading zero for single-digit hours (1-9) to make all times 5 digits
                let time_str = format!("{:02}:{:02}{}", display_hour, minutes, period);
                (half_hour * 30, time_str) // minutes since midnight
            })
            .collect();

        // day names
        let day_names = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let day_codes = vec!["M", "T", "W", "TH", "F", "S", "SU"];

        // build time block grid: map (day, slot) -> class
        let mut time_blocks: HashMap<(usize, usize), &Class> = HashMap::new();

        for class in schedule {
            if let Some(meeting_times_str) = &class.meeting_times {
                if !meeting_times_str.is_empty() {
                    let meetings = parse_meeting_times(meeting_times_str);
                    for (days, start_minutes, end_minutes) in meetings {
                        for day_code in &days {
                            // find day index
                            if let Some(day_idx) = day_codes.iter().position(|&d| d == day_code) {
                                // find time slots that overlap with this meeting
                                for (slot_idx, (slot_start, _)) in time_slots.iter().enumerate() {
                                    let slot_end = *slot_start + 30; // 30-minute slots
                                                                     // check if meeting overlaps with this time slot
                                    if *slot_start < end_minutes && slot_end > start_minutes {
                                        time_blocks.insert((day_idx, slot_idx), class);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // calculate column widths
        // find maximum time string width to ensure "am"/"pm" is never cut off
        let time_col_width = time_slots
            .iter()
            .map(|(_, time_str)| time_str.len() as u16)
            .max()
            .unwrap_or(7) // default to 7 if empty (covers "08:00am", "10:00am", "12:00pm")
            .max(7); // ensure at least 7 to cover all formatted times (all are 7 chars: "08:00am")
        let day_col_width = (calendar_area.width.saturating_sub(time_col_width + 2)) / 7; // 7 days

        // create header row with day names
        let header_y = calendar_area.y;
        for (idx, day_name) in day_names.iter().enumerate() {
            // day headers are never highlighted, only time slots are highlighted
            let style = Style::default()
                .fg(theme.title_color)
                .add_modifier(Modifier::BOLD);
            // render day header
            let day_x = calendar_area.x + time_col_width + (idx as u16 * day_col_width);
            let day_area = Rect {
                x: day_x,
                y: header_y,
                width: day_col_width,
                height: 1,
            };
            let day_para = Paragraph::new(day_name.to_string())
                .style(style)
                .alignment(Alignment::Center);
            frame.render_widget(day_para, day_area);
        }

        // render time slots and track the last rendered row
        let mut last_rendered_y = header_y;
        for (slot_idx, (_, time_str)) in time_slots.iter().enumerate() {
            let slot_y = header_y + 1 + slot_idx as u16;
            if slot_y >= calendar_area.y + calendar_area.height {
                break;
            }
            last_rendered_y = slot_y;

            // render time label
            let time_area = Rect {
                x: calendar_area.x,
                y: slot_y,
                width: time_col_width,
                height: 1,
            };
            let time_para =
                Paragraph::new(time_str.clone()).style(Style::default().fg(theme.muted_color));
            frame.render_widget(time_para, time_area);

            // render day columns
            for (day_idx, _) in day_names.iter().enumerate() {
                let day_x = calendar_area.x + time_col_width + (day_idx as u16 * day_col_width);
                let block_area = Rect {
                    x: day_x,
                    y: slot_y,
                    width: day_col_width,
                    height: 1,
                };

                let is_selected = day_idx == selected_day && slot_idx == selected_slot;
                let has_class = time_blocks.contains_key(&(day_idx, slot_idx));

                if has_class {
                    let class = time_blocks[&(day_idx, slot_idx)];
                    let class_code = format!("{}{}", class.subject_code, class.course_number);
                    let display_text = if class_code.len() <= day_col_width as usize {
                        class_code
                    } else {
                        class_code[..day_col_width as usize].to_string()
                    };

                    let style = if is_selected {
                        Style::default()
                            .fg(theme.selected_color)
                            .bg(theme.background_color)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default()
                            .fg(theme.info_color)
                            .bg(theme.background_color)
                            .add_modifier(Modifier::BOLD)
                    };

                    let block_para = Paragraph::new(display_text.as_str())
                        .style(style)
                        .alignment(Alignment::Center);
                    frame.render_widget(block_para, block_area);
                } else if is_selected {
                    // show selection indicator for empty blocks
                    let style = Style::default()
                        .fg(theme.selected_color)
                        .add_modifier(Modifier::REVERSED);
                    let block_para = Paragraph::new(" ").style(style);
                    frame.render_widget(block_para, block_area);
                }
            }
        }

        // render schedule counter right after the last time slot (with 1 line gap)
        let counter_y = last_rendered_y + 2;
        if counter_y < frame.area().height {
            let counter_area = Rect {
                x: calendar_area.x,
                y: counter_y,
                width: calendar_area.width,
                height: 1,
            };

            // if viewing from saved schedules, show saved schedule index instead
            let counter_text = if let (Some(saved_idx), Some(total_saved)) =
                (saved_schedule_index, total_saved_schedules)
            {
                format!("Schedule {} of {}", saved_idx + 1, total_saved)
            } else {
                format!("Schedule {} of {}", current_index + 1, total_schedules)
            };
            let counter_para = Paragraph::new(counter_text)
                .style(Style::default().fg(theme.info_color))
                .alignment(Alignment::Center);
            frame.render_widget(counter_para, counter_area);
        }
    }

    /// Render empty schedule section
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// area -> the area to render in
    /// focused -> whether the section is focused
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_empty_schedule_section(
        &self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        theme: &Theme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        let border_color = if focused {
            theme.selected_color
        } else {
            theme.border_color
        };
        let title = Paragraph::new("No Schedules")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Generated Schedule ")
                    .title_style(
                        Style::default()
                            .fg(theme.title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(border_color)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        let empty_text = vec![Line::from(Span::styled(
            "No valid schedules found",
            Style::default().fg(theme.muted_color),
        ))];

        let empty_widget = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().bg(theme.background_color));
        frame.render_widget(empty_widget, chunks[1]);
    }
}

impl Widget for ScheduleWidget {
    fn render(&self, frame: &mut Frame, theme: &Theme) {
        self.render_schedule(frame, theme);
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyAction {
        let (action, _schedule_action) = self.handle_key_with_action(key);
        action
    }

    fn focus_modes(&self) -> Vec<FocusMode> {
        vec![FocusMode::ScheduleCreation]
    }
}

// ============================================================================
// Schedule generation and conflict detection logic
// ============================================================================

/// Find class at a specific time block
///
/// Arguments:
/// --- ---
/// schedule -> the schedule classes
/// day -> day index (0-6 for Mon-Sun)
/// slot -> time slot index (0-23 for 8am-8pm in 30-min intervals)
/// --- ---
///
/// Returns:
/// --- ---
/// Option<&Class> -> the class at that time block, if any
/// --- ---
///
pub fn find_class_at_time_block(schedule: &[Class], day: usize, slot: usize) -> Option<&Class> {
    let day_codes = vec!["M", "T", "W", "TH", "F", "S", "SU"];
    let day_code = day_codes.get(day)?;

    // time slot: 0-28 represents 8am-10:30pm in 30-minute intervals
    // slot 0 = 8:00am = 16 half-hours = 480 minutes
    let slot_start_minutes = ((16 + slot) * 30) as i32;
    let slot_end_minutes = slot_start_minutes + 30;

    for class in schedule {
        if let Some(meeting_times_str) = &class.meeting_times {
            if !meeting_times_str.is_empty() {
                let meetings = parse_meeting_times(meeting_times_str);
                for (days, start_minutes, end_minutes) in meetings {
                    if days.contains(&day_code.to_string()) {
                        // check if meeting overlaps with this time slot
                        if slot_start_minutes < end_minutes && slot_end_minutes > start_minutes {
                            return Some(class);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Generate all possible non-conflicting schedules from classes in the cart
///
/// Arguments:
/// --- ---
/// cart_classes -> map of all classes in the cart (ID -> Class)
/// selected_for_schedule -> set of class IDs selected for schedule generation
/// allow_conflicts -> whether to allow conflicting schedules
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> all valid schedule combinations
/// --- ---
///
pub fn generate_schedules(
    cart_classes: &HashMap<String, Class>,
    selected_for_schedule: &HashSet<String>,
    allow_conflicts: bool,
) -> Vec<Vec<Class>> {
    // get all classes from selected_for_schedule
    let selected_classes: Vec<Class> = selected_for_schedule
        .iter()
        .filter_map(|class_id| cart_classes.get(class_id))
        .cloned()
        .collect();

    if selected_classes.is_empty() {
        return Vec::new();
    }

    if allow_conflicts {
        // generate all possible combinations including conflicts
        generate_all_schedules(&selected_classes)
    } else {
        // generate all possible combinations and filter out conflicts
        find_valid_schedules(&selected_classes)
    }
}

/// Find all conflicting class pairs
///
/// Arguments:
/// --- ---
/// classes -> list of classes to check
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<(String, String)> -> list of (class1_id, class2_id) pairs that conflict
/// --- ---
///
pub fn find_conflicting_classes(classes: &[Class]) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();
    for i in 0..classes.len() {
        for j in (i + 1)..classes.len() {
            if classes_conflict(&classes[i], &classes[j]) {
                let class1_id = format!(
                    "{} {}-{}",
                    classes[i].subject_code, classes[i].course_number, classes[i].section_sequence
                );
                let class2_id = format!(
                    "{} {}-{}",
                    classes[j].subject_code, classes[j].course_number, classes[j].section_sequence
                );
                conflicts.push((class1_id, class2_id));
            }
        }
    }
    conflicts
}

/// Generate all possible schedules from classes (including conflicting ones)
///
/// Arguments:
/// --- ---
/// classes -> list of classes to generate schedules from
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> all schedule combinations (including conflicts)
/// --- ---
///
fn generate_all_schedules(classes: &[Class]) -> Vec<Vec<Class>> {
    let mut all_schedules = Vec::new();

    // use backtracking to generate all combinations (without conflict checking)
    fn backtrack(
        classes: &[Class],
        current_schedule: &mut Vec<Class>,
        index: usize,
        all_schedules: &mut Vec<Vec<Class>>,
    ) {
        if index >= classes.len() {
            // we've considered all classes
            if !current_schedule.is_empty() {
                all_schedules.push(current_schedule.clone());
            }
            return;
        }

        // try adding current class (no conflict check)
        current_schedule.push(classes[index].clone());
        backtrack(classes, current_schedule, index + 1, all_schedules);
        current_schedule.pop();

        // try without adding current class
        backtrack(classes, current_schedule, index + 1, all_schedules);
    }

    let mut current = Vec::new();
    backtrack(classes, &mut current, 0, &mut all_schedules);

    all_schedules
}

/// Find all valid (non-conflicting) schedules from a list of classes
///
/// Arguments:
/// --- ---
/// classes -> list of classes to generate schedules from
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> all valid schedule combinations
/// --- ---
///
fn find_valid_schedules(classes: &[Class]) -> Vec<Vec<Class>> {
    let mut all_valid_schedules = Vec::new();

    // use backtracking to generate all valid combinations
    fn backtrack(
        classes: &[Class],
        current_schedule: &mut Vec<Class>,
        index: usize,
        valid_schedules: &mut Vec<Vec<Class>>,
    ) {
        if index >= classes.len() {
            // we've considered all classes
            if !current_schedule.is_empty() {
                valid_schedules.push(current_schedule.clone());
            }
            return;
        }

        // try adding current class
        let current_class = &classes[index];
        let mut can_add = true;

        // check for conflicts with existing classes in schedule
        for existing_class in current_schedule.iter() {
            if classes_conflict(current_class, existing_class) {
                can_add = false;
                break;
            }
        }

        if can_add {
            current_schedule.push(current_class.clone());
            backtrack(classes, current_schedule, index + 1, valid_schedules);
            current_schedule.pop();
        }

        // try without adding current class
        backtrack(classes, current_schedule, index + 1, valid_schedules);
    }

    let mut current = Vec::new();
    backtrack(classes, &mut current, 0, &mut all_valid_schedules);

    // filter to keep only maximal schedules (schedules that are not subsets of other schedules)
    filter_maximal_schedules(&all_valid_schedules)
}

/// Filter schedules to keep only maximal ones (remove schedules that are subsets of others)
///
/// Arguments:
/// --- ---
/// schedules -> all valid schedules
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> only maximal schedules
/// --- ---
///
fn filter_maximal_schedules(schedules: &[Vec<Class>]) -> Vec<Vec<Class>> {
    let mut maximal_schedules = Vec::new();

    for schedule in schedules {
        let schedule_ids: HashSet<String> = schedule.iter().map(|c| c.unique_id()).collect();

        // check if this schedule is a subset of any other schedule
        let is_subset = schedules.iter().any(|other_schedule| {
            if other_schedule.len() <= schedule.len() {
                return false; // can't be a subset if other is same size or smaller
            }
            let other_ids: HashSet<String> = other_schedule.iter().map(|c| c.unique_id()).collect();
            // this schedule is a subset if all its classes are in the other schedule
            schedule_ids.is_subset(&other_ids)
        });

        // only keep if it's not a subset (i.e., it's maximal)
        if !is_subset {
            maximal_schedules.push(schedule.clone());
        }
    }

    maximal_schedules
}

/// Check if two classes conflict (overlap in time)
///
/// Arguments:
/// --- ---
/// class1 -> first class
/// class2 -> second class
/// --- ---
///
/// Returns:
/// --- ---
/// bool -> true if classes conflict, false otherwise
/// --- ---
///
fn classes_conflict(class1: &Class, class2: &Class) -> bool {
    // if either class has no meeting times, they don't conflict
    let times1 = match &class1.meeting_times {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };
    let times2 = match &class2.meeting_times {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };

    // parse meeting times for both classes
    let meetings1 = parse_meeting_times(times1);
    let meetings2 = parse_meeting_times(times2);

    // check for any overlap
    for m1 in &meetings1 {
        for m2 in &meetings2 {
            if meetings_overlap(m1, m2) {
                return true;
            }
        }
    }

    false
}

/// Parse meeting times string into structured format
///
/// Arguments:
/// --- ---
/// times_str -> meeting times string (e.g., "M:08:00:00-10:45:00|TH:08:00:00-09:15:00")
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<(Vec<String>, i32, i32)> -> list of (days, start_minutes, end_minutes)
/// --- ---
///
fn parse_meeting_times(times_str: &str) -> Vec<(Vec<String>, i32, i32)> {
    let mut meetings = Vec::new();

    for mt in times_str.split('|') {
        if mt.is_empty() {
            continue;
        }

        if let Some(colon_pos) = mt.find(':') {
            let days_part = &mt[..colon_pos];
            let time_part = &mt[colon_pos + 1..];

            if let Some(dash_pos) = time_part.find('-') {
                let start_str = &time_part[..dash_pos];
                let end_str = &time_part[dash_pos + 1..];

                let start_minutes = time_to_minutes(start_str);
                let end_minutes = time_to_minutes(end_str);

                // parse days (handle "MW", "TTH", etc.)
                let days = parse_days(days_part);

                if !days.is_empty() && start_minutes > 0 && end_minutes > start_minutes {
                    meetings.push((days, start_minutes, end_minutes));
                }
            }
        }
    }

    meetings
}

/// Parse day codes into individual days
///
/// Arguments:
/// --- ---
/// days_str -> day string (e.g., "MW", "TTH")
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<String> -> list of day codes
/// --- ---
///
fn parse_days(days_str: &str) -> Vec<String> {
    let mut days = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = days_str.chars().collect();

    while i < chars.len() {
        if i + 1 < chars.len() {
            let two_char = format!("{}{}", chars[i], chars[i + 1]);
            match two_char.as_str() {
                "TH" => {
                    days.push("TH".to_string());
                    i += 2;
                    continue;
                }
                "SU" => {
                    days.push("SU".to_string());
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        match chars[i] {
            'M' => days.push("M".to_string()),
            'T' => days.push("T".to_string()),
            'W' => days.push("W".to_string()),
            'F' => days.push("F".to_string()),
            'S' => days.push("S".to_string()),
            _ => {}
        }
        i += 1;
    }

    days
}

/// Convert time string (HH:MM:SS) to minutes since midnight
///
/// Arguments:
/// --- ---
/// time_str -> time string
/// --- ---
///
/// Returns:
/// --- ---
/// i32 -> minutes since midnight
/// --- ---
///
fn time_to_minutes(time_str: &str) -> i32 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() >= 2 {
        let hours: i32 = parts[0].parse().unwrap_or(0);
        let minutes: i32 = parts[1].parse().unwrap_or(0);
        hours * 60 + minutes
    } else {
        0
    }
}

/// Check if two meetings overlap
///
/// Arguments:
/// --- ---
/// m1 -> first meeting (days, start, end)
/// m2 -> second meeting (days, start, end)
/// --- ---
///
/// Returns:
/// --- ---
/// bool -> true if meetings overlap, false otherwise
/// --- ---
///
fn meetings_overlap(m1: &(Vec<String>, i32, i32), m2: &(Vec<String>, i32, i32)) -> bool {
    // check if they share any day
    let days_overlap = m1.0.iter().any(|d| m2.0.contains(d));
    if !days_overlap {
        return false;
    }

    // check if time ranges overlap
    let (_, start1, end1) = m1;
    let (_, start2, end2) = m2;

    // overlap if: start1 < end2 && start2 < end1
    start1 < end2 && start2 < end1
}
