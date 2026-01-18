/// src/tui/app.rs
///
/// Refactored TUI application using widget pattern
///
/// This demonstrates how to use the new widget structs for a cleaner architecture.
/// Widgets encapsulate their own state and key handling.
use crate::data::sql::Class;
use crate::data::sql::{fetch_schools, fetch_terms, get_last_sync_time, School};
use crate::data::sync::get_synced_db_path;
use crate::dsl::compiler::Compiler;
use crate::tui::errors::TUIError;
use crate::tui::save::{self, SavedSchedule};
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::widgets::{
    DetailViewWidget, HelpBarWidget, KeyAction, LogoWidget, MainMenuWidget, QueryGuideWidget,
    ScheduleAction, ScheduleWidget, SearchWidget, SettingsAction, SettingsWidget, ToastWidget,
    Widget,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::DefaultTerminal;
use std::time::{Duration, Instant};

/// Refactored TUI application using widget pattern
///
/// Fields:
/// --- ---
/// terminal -> The terminal instance for rendering
/// compiler -> The DSL compiler instance
/// focus_mode -> Current UI focus mode
///
/// Widgets:
/// main_menu -> Main menu widget
/// search -> Search widget (query input + results)
/// settings -> Settings widget
/// schedule -> Schedule widget
/// guide -> Query guide widget
///
/// Shared state:
/// toast_message -> Optional toast notification message
/// toast_start_time -> Timestamp when toast was shown
/// error_type -> Type of error if any
/// saved_schedules -> List of saved schedules
/// selected_saved_schedule_index -> Index of selected saved schedule
/// save_name_input -> Current save name input
/// save_name_cursor_visible -> Whether save name cursor is visible
/// save_name_last_blink -> Timestamp of last save name cursor blink
/// selected_class_for_details -> Class selected for detail view
/// detail_return_focus -> Focus mode to return to after detail view
/// --- ---
///
pub struct TuiApp {
    pub main_menu: MainMenuWidget,
    pub search: SearchWidget,
    pub settings: SettingsWidget,
    pub schedule: ScheduleWidget,
    pub guide: QueryGuideWidget,
    pub logo: LogoWidget,
    pub help_bar: HelpBarWidget,
    pub toast: ToastWidget,
    pub detail_view: DetailViewWidget,
    terminal: DefaultTerminal,
    compiler: Compiler,
    focus_mode: FocusMode,
    toast_message: Option<String>,
    toast_start_time: Option<Instant>,
    error_type: Option<ErrorType>,
    saved_schedules: Vec<SavedSchedule>,
    selected_saved_schedule_index: usize,
    save_name_input: String,
    save_name_cursor_visible: bool,
    save_name_last_blink: Instant,
    selected_class_for_details: Option<Class>,
    detail_return_focus: FocusMode,
}

impl TuiApp {
    /// Create a new TuiApp instance
    ///
    /// Arguments:
    /// --- ---
    /// compiler -> The DSL compiler instance to use
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<Self, TUIError> -> The new TuiApp instance or an error
    /// --- ---
    ///
    pub fn new(compiler: Compiler) -> Result<Self, TUIError> {
        let terminal = ratatui::init();

        Ok(TuiApp {
            terminal,
            compiler,
            focus_mode: FocusMode::MainMenu,

            // initialize widgets
            main_menu: MainMenuWidget::new(),
            search: SearchWidget::new(),
            settings: SettingsWidget::new(),
            schedule: ScheduleWidget::new(),
            guide: QueryGuideWidget::new(),
            logo: LogoWidget::new(),
            help_bar: HelpBarWidget::new(),
            toast: ToastWidget::new(),
            detail_view: DetailViewWidget::new(),

            // shared state
            toast_message: None,
            toast_start_time: None,
            error_type: None,
            saved_schedules: Vec::new(),
            selected_saved_schedule_index: 0,
            save_name_input: String::new(),
            save_name_cursor_visible: true,
            save_name_last_blink: Instant::now(),
            selected_class_for_details: None,
            detail_return_focus: FocusMode::ResultsBrowse,
        })
    }

    /// Run the TUI event loop
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Result<(), Box<dyn std::error::Error>> -> Ok on success, error on failure
    /// --- ---
    ///
    /// Runs the main event loop, handling input and rendering until exit
    ///
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // update timers
            self.update_toast();
            self.search.update_cursor_blink();
            self.update_save_name_cursor();

            // sync widget state
            self.main_menu.set_cart_empty(self.schedule.is_cart_empty());

            // draw the current state
            self.draw()?;

            // handle input events
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.handle_key(key) {
                        KeyAction::Exit => break Ok(()),
                        KeyAction::Continue => {}
                        KeyAction::Navigate(mode) => self.navigate_to(mode),
                        KeyAction::ShowToast {
                            message,
                            error_type,
                        } => {
                            self.show_toast(message, error_type);
                        }
                    }
                }
            }
        }
    }

    /// Handle a key event based on current focus mode
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
        match self.focus_mode {
            FocusMode::MainMenu => self.main_menu.handle_key(key),

            FocusMode::QueryInput | FocusMode::ResultsBrowse => {
                // handle special keys that need app-level coordination
                // but only intercept Enter if completions are NOT showing
                if key.code == KeyCode::Enter
                    && self.search.is_query_input()
                    && !self.search.completion.show_completions
                {
                    // check if school/term are selected first
                    if self.settings.selected_school_id.is_none() {
                        return KeyAction::ShowToast {
                            message: "Please select a school first (Settings → School)".to_string(),
                            error_type: ErrorType::Warning,
                        };
                    }
                    if self.settings.selected_term_id.is_none()
                        && self.settings.selected_school_id != Some("_test".to_string())
                    {
                        return KeyAction::ShowToast {
                            message: "Please select a term first (Settings → Term)".to_string(),
                            error_type: ErrorType::Warning,
                        };
                    }
                    // show "Searching..." indicator before executing query
                    self.search.is_searching = true;

                    // draw the search view with "Searching..." indicator
                    let _ = self.draw();

                    // execute query
                    let result = self.search.execute_query(&mut self.compiler);
                    self.search.is_searching = false;

                    if let Some(action) = result {
                        return action;
                    }
                    return KeyAction::Continue;
                }

                if key.code == KeyCode::Tab && !self.search.completion.show_completions {
                    // handle tab completion
                    if let Some(hint) = self.search.handle_tab_completion(&mut self.compiler) {
                        return KeyAction::ShowToast {
                            message: hint,
                            error_type: ErrorType::Info,
                        };
                    }
                    return KeyAction::Continue;
                }

                let action = self.search.handle_key(key);

                // sync the app's focus_mode with the search widget's internal focus
                self.focus_mode = self.search.current_focus_mode();

                // handle navigation to detail view
                if matches!(&action, KeyAction::Navigate(FocusMode::DetailView)) {
                    if let Some(class) = self.search.selected_class() {
                        self.selected_class_for_details = Some(class.clone());
                        self.detail_return_focus = self.search.current_focus_mode();
                    }
                }

                action
            }

            FocusMode::Settings => {
                let (action, settings_action) = self.settings.handle_key_with_action(key);

                // handle settings-specific actions
                match settings_action {
                    SettingsAction::SchoolSelected {
                        school_id,
                        school_name,
                    } => {
                        self.compiler.set_school_id(Some(school_id.clone()));
                        self.load_terms(&school_id);
                        self.schedule.clear();
                        self.search.query_results.clear();
                        self.show_toast(format!("Selected: {}", school_name), ErrorType::Success);
                    }
                    SettingsAction::TermSelected { term_id, term_name } => {
                        self.compiler.set_term_id(Some(term_id));
                        self.schedule.clear();
                        self.search.query_results.clear();
                        self.show_toast(format!("Selected: {}", term_name), ErrorType::Success);
                    }
                    SettingsAction::ThemeChanged(_theme) => {
                        // theme is stored in settings widget
                    }
                    SettingsAction::SyncRequested => {
                        self.show_toast("Starting sync...".to_string(), ErrorType::Info);
                        self.perform_sync();
                    }
                    SettingsAction::None => {}
                }

                action
            }

            FocusMode::ScheduleCreation => {
                let (action, schedule_action) = self.schedule.handle_key_with_action(key);

                match schedule_action {
                    ScheduleAction::OpenDetailView(class) => {
                        self.selected_class_for_details = Some(class);
                        self.detail_return_focus = FocusMode::ScheduleCreation;
                    }
                    ScheduleAction::SaveSchedule => {
                        // will navigate to SaveNameInput
                        self.save_name_input.clear();
                    }
                    _ => {}
                }

                action
            }

            FocusMode::QueryGuide => self.guide.handle_key(key),

            FocusMode::DetailView => self.handle_detail_view_key(key),

            FocusMode::MySchedules => self.handle_my_schedules_key(key),

            FocusMode::SaveNameInput => self.handle_save_name_key(key),

            FocusMode::Help => {
                // help is handled by QueryGuide
                self.guide.handle_key(key)
            }
        }
    }

    /// Navigate to a new focus mode with necessary setup
    ///
    /// Arguments:
    /// --- ---
    /// mode -> The focus mode to navigate to
    /// --- ---
    ///
    /// Returns: None
    ///
    fn navigate_to(&mut self, mode: FocusMode) {
        match mode {
            FocusMode::Settings => {
                self.load_school_data();
            }
            FocusMode::ScheduleCreation => {
                // only enter creation mode if NOT coming from DetailView, SaveNameInput, or MySchedules
                // to preserve state when returning from overlays or viewing saved schedules
                if self.focus_mode != FocusMode::DetailView
                    && self.focus_mode != FocusMode::SaveNameInput
                    && self.focus_mode != FocusMode::MySchedules
                    && !self.schedule.is_cart_empty()
                {
                    self.schedule.enter_creation_mode();
                }
            }
            FocusMode::MySchedules => {
                if let Ok(schedules) = save::load_all_schedules() {
                    self.saved_schedules = schedules;
                    self.selected_saved_schedule_index = 0;
                }
            }
            FocusMode::QueryGuide => {
                self.guide.open(self.focus_mode.clone());
            }
            FocusMode::QueryInput => {
                self.search.set_focus(FocusMode::QueryInput);
            }
            FocusMode::ResultsBrowse => {
                self.search.set_focus(FocusMode::ResultsBrowse);
            }
            _ => {}
        }
        self.focus_mode = mode;
    }

    /// Draw the current frame
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Result<(), Box<dyn std::error::Error>> -> Ok on success, error on failure
    /// --- ---
    ///
    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // extract all values needed for rendering to avoid borrow conflicts
        let theme = self.settings.current_theme.to_theme();
        let focus_mode = self.focus_mode.clone();
        let detail_return_focus = self.detail_return_focus.clone();
        let toast_message = self.toast_message.clone();
        let error_type = self.error_type.clone();

        // track values to update after rendering
        let mut new_guide_max_scroll = self.guide.max_scroll;

        let terminal = &mut self.terminal;
        terminal.draw(|frame| {
            // clear with background color
            let buffer = frame.buffer_mut();
            for y in 0..buffer.area.height {
                for x in 0..buffer.area.width {
                    let cell = &mut buffer[(x, y)];
                    cell.set_bg(theme.background_color);
                }
            }

            // always render logo
            self.logo.render(frame, &theme);

            // render based on focus mode
            match focus_mode {
                FocusMode::MainMenu => {
                    self.main_menu.render(frame, &theme);
                }
                FocusMode::QueryInput | FocusMode::ResultsBrowse => {
                    self.search.render(frame, &theme);
                }
                FocusMode::Settings => {
                    self.settings.render(frame, &theme);
                }
                FocusMode::ScheduleCreation => {
                    self.schedule.render(frame, &theme);
                }
                FocusMode::QueryGuide | FocusMode::Help => {
                    let (_total_lines, max_scroll) = self.guide.render_guide(frame, &theme);
                    new_guide_max_scroll = max_scroll;
                }
                FocusMode::DetailView => {
                    // render background based on return focus
                    match detail_return_focus {
                        FocusMode::ScheduleCreation | FocusMode::MySchedules => {
                            self.schedule.render(frame, &theme);
                        }
                        _ => {
                            self.search.render(frame, &theme);
                        }
                    }
                    // render detail view overlay
                    if let Some(ref class) = self.selected_class_for_details {
                        let in_cart = self.schedule.cart_classes.contains_key(&class.unique_id());
                        let show_cart_option = detail_return_focus != FocusMode::ScheduleCreation;
                        self.detail_view.class = Some(class.clone());
                        self.detail_view.is_in_cart = in_cart;
                        self.detail_view.show_cart_option = show_cart_option;
                        self.detail_view.render(frame, &theme);
                    }
                }
                FocusMode::MySchedules => {
                    let width = 50_u16.min(frame.area().width.saturating_sub(4));
                    let height = 15_u16.min(frame.area().height.saturating_sub(20));
                    let x = (frame.area().width.saturating_sub(width)) / 2;
                    let y = 13_u16;

                    let area = Rect {
                        x,
                        y,
                        width,
                        height,
                    };

                    let mut lines = Vec::new();
                    if self.saved_schedules.is_empty() {
                        lines.push(Line::from(Span::styled(
                            "No saved schedules yet.",
                            Style::default().fg(theme.muted_color),
                        )));
                    } else {
                        for (i, schedule) in self.saved_schedules.iter().enumerate() {
                            let is_selected = i == self.selected_saved_schedule_index;
                            let prefix = if is_selected { "▸ " } else { "  " };
                            let style = if is_selected {
                                Style::default()
                                    .fg(theme.selected_color)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(theme.text_color)
                            };
                            lines.push(Line::from(vec![
                                Span::styled(prefix, style),
                                Span::styled(&schedule.name, style),
                            ]));
                        }
                    }

                    let para = Paragraph::new(lines).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" My Schedules ")
                            .title_style(
                                Style::default()
                                    .fg(theme.title_color)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .border_style(Style::default().fg(theme.border_color)),
                    );
                    frame.render_widget(para, area);
                }
                FocusMode::SaveNameInput => {
                    self.schedule.render(frame, &theme);

                    let width = 40_u16;
                    let height = 5_u16;
                    let x = (frame.area().width.saturating_sub(width)) / 2;
                    let y = (frame.area().height.saturating_sub(height)) / 2;
                    let area = Rect {
                        x,
                        y,
                        width,
                        height,
                    };

                    frame.render_widget(Clear, area);

                    let cursor = if self.save_name_cursor_visible {
                        "│"
                    } else {
                        " "
                    };
                    let input_line = Line::from(vec![
                        Span::styled(&self.save_name_input, Style::default().fg(theme.text_color)),
                        Span::styled(cursor, Style::default().fg(theme.selected_color)),
                    ]);

                    let para = Paragraph::new(vec![Line::from(""), input_line])
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Save Schedule ")
                                .title_style(
                                    Style::default()
                                        .fg(theme.title_color)
                                        .add_modifier(Modifier::BOLD),
                                )
                                .border_style(Style::default().fg(theme.border_color))
                                .style(Style::default().bg(theme.background_color)),
                        );
                    frame.render_widget(para, area);
                }
            }

            // render helpers and toast
            self.help_bar.toast_message = toast_message.clone();
            self.help_bar.focus_mode = focus_mode.clone();
            self.help_bar.schedule_selection_mode = Some(self.schedule.schedule_selection_mode);
            self.help_bar.render(frame, &theme);

            self.toast.toast_message = toast_message.clone();
            self.toast.error_type = error_type.clone();
            self.toast.render(frame, &theme);
        })?;

        // update values after render
        self.guide.max_scroll = new_guide_max_scroll;

        Ok(())
    }

    /// Handle detail view key events
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
    fn handle_detail_view_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter => {
                let return_to = if self.detail_return_focus == FocusMode::MySchedules {
                    FocusMode::ScheduleCreation
                } else {
                    self.detail_return_focus.clone()
                };
                KeyAction::Navigate(return_to)
            }
            KeyCode::Char('g') | KeyCode::Char('G')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.guide.open(FocusMode::DetailView);
                KeyAction::Navigate(FocusMode::QueryGuide)
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.detail_return_focus != FocusMode::ScheduleCreation {
                    if let Some(ref class) = self.selected_class_for_details {
                        self.schedule.toggle_cart(class);
                    }
                }
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    /// Handle my schedules view key events
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
    fn handle_my_schedules_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Exit,
            KeyCode::Esc => KeyAction::Navigate(FocusMode::MainMenu),
            KeyCode::Up => {
                if self.selected_saved_schedule_index > 0 {
                    self.selected_saved_schedule_index -= 1;
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                if self.selected_saved_schedule_index < self.saved_schedules.len().saturating_sub(1)
                {
                    self.selected_saved_schedule_index += 1;
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                if self.selected_saved_schedule_index < self.saved_schedules.len() {
                    // load all saved schedules so PageUp/PageDown can cycle through them
                    let all_schedules: Vec<Vec<crate::data::sql::Class>> = self
                        .saved_schedules
                        .iter()
                        .map(|s| s.classes.clone())
                        .collect();
                    let all_names: Vec<String> = self
                        .saved_schedules
                        .iter()
                        .map(|s| s.name.clone())
                        .collect();
                    self.schedule.load_saved_schedules(
                        all_schedules,
                        all_names,
                        self.selected_saved_schedule_index,
                    );
                    KeyAction::Navigate(FocusMode::ScheduleCreation)
                } else {
                    KeyAction::Continue
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if self.selected_saved_schedule_index < self.saved_schedules.len() {
                    let saved = &self.saved_schedules[self.selected_saved_schedule_index];
                    let name = saved.name.clone();
                    let timestamp = saved.timestamp;
                    if save::delete_schedule(timestamp).is_ok() {
                        self.saved_schedules
                            .remove(self.selected_saved_schedule_index);
                        if self.selected_saved_schedule_index >= self.saved_schedules.len()
                            && !self.saved_schedules.is_empty()
                        {
                            self.selected_saved_schedule_index = self.saved_schedules.len() - 1;
                        }
                        return KeyAction::ShowToast {
                            message: format!("Schedule '{}' deleted", name),
                            error_type: ErrorType::Success,
                        };
                    }
                }
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    /// Handle save name input key events
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
    fn handle_save_name_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Exit,
            KeyCode::Esc => {
                self.save_name_input.clear();
                KeyAction::Navigate(FocusMode::ScheduleCreation)
            }
            KeyCode::Enter => {
                if self.save_name_input.trim().is_empty() {
                    return KeyAction::ShowToast {
                        message: "Schedule name cannot be empty!".to_string(),
                        error_type: ErrorType::Semantic,
                    };
                }
                if let Some(schedule) = self.schedule.current_schedule() {
                    match save::save_schedule(
                        self.save_name_input.trim(),
                        self.settings.selected_school_id.as_deref(),
                        self.settings.selected_term_id.as_deref(),
                        schedule,
                    ) {
                        Ok(_) => {
                            let msg = format!("Schedule '{}' saved!", self.save_name_input.trim());
                            self.save_name_input.clear();
                            self.focus_mode = FocusMode::ScheduleCreation;
                            return KeyAction::ShowToast {
                                message: msg,
                                error_type: ErrorType::Success,
                            };
                        }
                        Err(e) => {
                            return KeyAction::ShowToast {
                                message: format!("Failed to save schedule: {}", e),
                                error_type: ErrorType::Semantic,
                            };
                        }
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Backspace => {
                self.save_name_input.pop();
                KeyAction::Continue
            }
            KeyCode::Char(c) => {
                self.save_name_input.push(c);
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    // helper methods

    /// Update toast message state based on elapsed time
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    /// Automatically clears toast messages after 3 seconds
    ///
    fn update_toast(&mut self) {
        if let Some(start_time) = self.toast_start_time {
            if start_time.elapsed() > Duration::from_secs(3) {
                self.toast_message = None;
                self.toast_start_time = None;
                self.error_type = None;
            }
        }
    }

    /// Update save name input cursor blink state
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    /// Toggles cursor visibility every 500ms when in SaveNameInput focus mode
    ///
    fn update_save_name_cursor(&mut self) {
        if self.focus_mode == FocusMode::SaveNameInput {
            if self.save_name_last_blink.elapsed() > Duration::from_millis(500) {
                self.save_name_cursor_visible = !self.save_name_cursor_visible;
                self.save_name_last_blink = Instant::now();
            }
        }
    }

    /// Show a toast notification message
    ///
    /// Arguments:
    /// --- ---
    /// message -> The message to display
    /// error_type -> The type of notification (error, warning, info, success)
    /// --- ---
    ///
    /// Returns: None
    ///
    fn show_toast(&mut self, message: String, error_type: ErrorType) {
        self.toast_message = Some(message);
        self.toast_start_time = Some(Instant::now());
        self.error_type = Some(error_type);
    }

    /// Load school data from the database
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    /// Loads schools from both test database (if available) and synced database,
    /// then loads terms for the currently selected school
    ///
    fn load_school_data(&mut self) {
        let db_path = get_synced_db_path();
        let mut schools = Vec::new();

        let test_db = std::path::PathBuf::from("classy/test.db");
        if test_db.exists() {
            schools.push(School {
                id: "_test".to_string(),
                name: "Test Database (Marist 2025)".to_string(),
            });
        }

        if db_path.exists() {
            if let Ok(synced_schools) = fetch_schools(&db_path) {
                schools.extend(synced_schools);
            }
            self.settings
                .set_last_sync_time(get_last_sync_time(&db_path));
        }

        self.settings.set_schools(schools);

        // clone to avoid borrow issue
        if let Some(school_id) = self.settings.selected_school_id.clone() {
            self.load_terms(&school_id);
        }
    }

    /// Load term data for a specific school
    ///
    /// Arguments:
    /// --- ---
    /// school_id -> The ID of the school to load terms for
    /// --- ---
    ///
    /// Returns: None
    ///
    /// Loads terms from the synced database for the given school.
    /// Skips loading if school_id is "_test"
    ///
    fn load_terms(&mut self, school_id: &str) {
        if school_id == "_test" {
            self.settings.set_terms(Vec::new());
            return;
        }

        let db_path = get_synced_db_path();
        if db_path.exists() {
            if let Ok(terms) = fetch_terms(&db_path, school_id) {
                self.settings.set_terms(terms);
            }
        }
    }

    /// Perform database synchronization
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    /// Syncs data from remote sources using configuration from environment variables.
    /// Shows toast notifications for success or failure, and reloads school data on success
    ///
    fn perform_sync(&mut self) {
        match crate::data::sync::SyncConfig::from_env() {
            Ok(config) => match crate::data::sync::sync_all(&config) {
                Ok(_) => {
                    self.show_toast(
                        "Sync completed successfully!".to_string(),
                        ErrorType::Success,
                    );
                    self.load_school_data();
                }
                Err(e) => {
                    self.show_toast(format!("Sync failed: {}", e), ErrorType::Warning);
                }
            },
            Err(e) => {
                self.show_toast(format!("Config error: {}", e), ErrorType::Warning);
            }
        }
        self.settings.sync_complete();
    }

    /// Terminate the TUI gracefully
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Result<(), TUIError> -> Ok on success, error on failure
    /// --- ---
    ///
    pub fn terminate(&self) -> Result<(), TUIError> {
        ratatui::restore();
        Ok(())
    }
}
