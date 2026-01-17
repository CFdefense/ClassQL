/// src/tui/render.rs
///
/// Render module for the TUI
///
/// Responsible for rendering the TUI interface and handling user interactions
///
/// Contains:
/// --- ---
/// Tui -> TUI struct that manages application state and event loop
///      Methods:
///      --- ---
///      new -> Create a new TUI instance
///      run -> Run the TUI event loop
///      terminate -> Terminate the TUI
///      clear_error_state -> Clear error state and toast messages
///      handle_tab_completion -> Handle tab completion logic
///      get_completion_hint -> Get helpful hint when no completions available
///      update_toast -> Update toast timer and auto-dismiss
///      show_toast -> Show a toast notification
///      --- ---
/// Helper functions:
///      --- ---
///      render_frame -> Main render function that orchestrates all rendering
///      render_logo -> Render the ASCII art logo
///      render_search_bar_with_data -> Render the search bar with input and cursor
///      render_search_helpers_with_data -> Render the help text at bottom of screen
///      render_query_results -> Render the query results in a 3-column grid
///      render_detail_view -> Render detailed class information overlay
///      render_syntax_highlighting -> Render syntax highlighting (placeholder)
///      render_toast_with_data -> Render error toast notifications
///      render_completion_dropdown -> Render tab completion suggestions dropdown
///      --- ---
/// --- ---
///
use crate::data::sql::Class;
use crate::dsl::compiler::{Compiler, CompilerResult};
use crate::tui::errors::TUIError;
use crate::tui::save::{self, SavedSchedule};
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::ThemePalette;
use crate::data::sync::get_synced_db_path;
use crate::data::sql::{School, Term, fetch_schools, fetch_terms, get_last_sync_time};
use crate::tui::widgets::{
    render_completion_dropdown, render_detail_view, render_logo, render_main_menu,
    render_query_guide, render_query_results, render_search_bar_with_data,
    render_search_helpers_with_data, render_settings, render_toast_with_data, MenuOption,
    SettingsState,
};
use crate::tui::widgets::schedule::{find_class_at_time_block, find_conflicting_classes, generate_schedules};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};

/// Tui struct
///
/// Main TUI struct that manages the application state and rendering
///
/// Tui fields:
/// --- ---
/// terminal -> The terminal instance for rendering
/// input -> The current user input string in the query box
/// user_query -> The last executed query string
/// toast_message -> Optional toast notification message
/// toast_start_time -> Timestamp when toast was shown (for auto-dismiss)
/// error_type -> Type of error if any (Lexer, Parser, Semantic)
/// problematic_positions -> Byte ranges of problematic tokens in input
/// compiler -> The DSL compiler instance
/// completions -> List of completion suggestions
/// completion_index -> Currently selected completion index
/// show_completions -> Whether completion dropdown is visible
/// partial_word -> The partial word being completed
/// query_results -> The list of Class results from the last query
/// results_scroll -> Scroll offset for results display
/// focus_mode -> Current UI focus (QueryInput, ResultsBrowse, DetailView)
/// selected_result -> Index of currently selected result in browse mode
/// cursor_visible -> Whether the input cursor is visible (for blinking)
/// last_cursor_blink -> Timestamp of last cursor blink toggle
/// max_items_that_fit -> Maximum number of items that fit on the screen
/// menu_index -> Index of currently selected menu option
/// current_theme -> Current theme
/// settings_index -> Index of currently selected settings option
/// detail_return_focus -> Where to return when closing detail view
/// cart_classes -> Map of all classes in the cart (ID -> Class)
/// generated_schedules -> All generated non-conflicting schedules
/// current_schedule_index -> Index of currently displayed schedule
/// schedule_cart_focus -> Whether the cart is focused
/// selected_cart_index -> Index of currently selected cart item
/// schedule_selection_mode -> Whether the schedule selection mode is active
/// selected_time_block_day -> Index of currently selected day in schedule viewing mode
/// selected_time_block_slot -> Index of currently selected time slot in schedule viewing mode
/// --- ---
///
pub struct Tui {
    terminal: DefaultTerminal,
    input: String,
    user_query: String,
    toast_message: Option<String>,
    toast_start_time: Option<Instant>,
    error_type: Option<ErrorType>,
    problematic_positions: Vec<(usize, usize)>,
    compiler: Compiler,
    completions: Vec<String>,
    completion_index: Option<usize>,
    show_completions: bool,
    partial_word: String,
    query_results: Vec<Class>,
    results_scroll: usize,
    focus_mode: FocusMode,
    selected_result: usize,
    cursor_visible: bool,
    last_cursor_blink: Instant,
    max_items_that_fit: usize,
    menu_index: usize,
    current_theme: ThemePalette,
    settings_index: usize,
    guide_scroll: usize,
    guide_max_scroll: usize,
    guide_return_focus: FocusMode,
    detail_return_focus: FocusMode,
    cart_classes: std::collections::HashMap<String, Class>,
    selected_for_schedule: std::collections::HashSet<String>,
    generated_schedules: Vec<Vec<Class>>,
    current_schedule_index: usize,
    schedule_cart_focus: bool,
    selected_cart_index: usize,
    schedule_selection_mode: bool,
    selected_time_block_day: usize,
    selected_time_block_slot: usize,
    saved_schedules: Vec<SavedSchedule>,
    selected_saved_schedule_index: usize,
    current_saved_schedule_name: Option<String>,
    save_name_input: String,
    save_name_cursor_visible: bool,
    save_name_last_blink: Instant,
    selected_class_for_details: Option<Class>,
    available_schools: Vec<School>,
    selected_school_index: usize,
    selected_school_id: Option<String>,
    school_scroll_offset: usize,
    available_terms: Vec<Term>,
    selected_term_index: usize,
    selected_term_id: Option<String>,
    term_scroll_offset: usize,
    last_sync_time: Option<String>,
    is_syncing: bool,
    school_picker_open: bool,
    term_picker_open: bool,
}

/// Tui Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new TUI instance
/// run -> Run the TUI event loop
/// terminate -> Terminate the TUI
/// --- ---
///
impl Tui {
    /// Create a new TUI instance
    ///
    /// Parameters:
    /// --- ---
    /// compiler -> The compiler instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<Tui, TUIError> -> The new TUI instance
    /// --- ---
    ///
    pub fn new(compiler: Compiler) -> Result<Self, TUIError> {
        // initialize the terminal instance
        let terminal = ratatui::init();

        // initialize and return our tui instance
        Ok(Tui {
            terminal,
            input: String::new(),
            user_query: String::new(),
            toast_message: None,
            toast_start_time: None,
            error_type: None,
            problematic_positions: Vec::new(),
            compiler,
            completions: Vec::new(),
            completion_index: None,
            show_completions: false,
            partial_word: String::new(),
            query_results: Vec::new(),
            results_scroll: 0,
            focus_mode: FocusMode::MainMenu,
            selected_result: 0,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            max_items_that_fit: 0,
            menu_index: 0,
            current_theme: ThemePalette::Default,
            settings_index: 0,
            guide_scroll: 0,
            guide_max_scroll: 0,
            guide_return_focus: FocusMode::QueryInput,
            detail_return_focus: FocusMode::ResultsBrowse,
            cart_classes: std::collections::HashMap::new(),
            selected_for_schedule: std::collections::HashSet::new(),
            generated_schedules: Vec::new(),
            current_schedule_index: 0,
            schedule_cart_focus: true, // start focused on cart
            selected_cart_index: 0,
            schedule_selection_mode: true, // start in class selection mode
            selected_time_block_day: 0, // Monday
            selected_time_block_slot: 0, // first time slot
            saved_schedules: Vec::new(),
            selected_saved_schedule_index: 0,
            current_saved_schedule_name: None,
            save_name_input: String::new(),
            save_name_cursor_visible: true,
            save_name_last_blink: Instant::now(),
            selected_class_for_details: None,
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
        })
    }
    
    /// Load available schools and sync info from the database
    /// 
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    fn load_school_data(&mut self) {
        let db_path = get_synced_db_path();
        let mut schools = Vec::new();
        
        // add test database option if it exists
        let test_db = std::path::PathBuf::from("classy/test.db");
        if test_db.exists() {
            schools.push(School {
                id: "_test".to_string(),
                name: "Test Database (Local)".to_string(),
            });
        }
        
        // add synced schools
        if db_path.exists() {
            if let Ok(synced_schools) = fetch_schools(&db_path) {
                schools.extend(synced_schools);
            }
            self.last_sync_time = get_last_sync_time(&db_path);
        }
        
        self.available_schools = schools;
        
        // reload terms if school is selected
        if let Some(ref school_id) = self.selected_school_id {
            self.load_terms(school_id.clone());
        }
    }
    
    /// Load available terms for the selected school
    /// 
    /// Parameters:
    /// --- ---
    /// school_id -> The school ID to load terms for
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    fn load_terms(&mut self, school_id: String) {
        // test database doesn't use terms
        if school_id == "_test" {
            self.available_terms = Vec::new();
            self.selected_term_index = 0;
            self.term_scroll_offset = 0;
            return;
        }
        
        let db_path = get_synced_db_path();
        if db_path.exists() {
            if let Ok(terms) = fetch_terms(&db_path, &school_id) {
                self.available_terms = terms;
                self.selected_term_index = 0;
                self.term_scroll_offset = 0;
            }
        }
    }

    /// Run the TUI event loop, handling user input and rendering the TUI
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Result<(), Box<dyn std::error::Error>> -> The result of the event loop
    /// --- ---
    ///
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // update toast timer
            self.update_toast();

            // update cursor blink (every 500ms)
            if self.last_cursor_blink.elapsed() > Duration::from_millis(500) {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_blink = Instant::now();
            }
            
            // update save name cursor blink
            if self.focus_mode == FocusMode::SaveNameInput {
                if self.save_name_last_blink.elapsed() > Duration::from_millis(500) {
                    self.save_name_cursor_visible = !self.save_name_cursor_visible;
                    self.save_name_last_blink = Instant::now();
                }
            }

            // extract render data to avoid borrow conflicts
            let input = self.input.clone();
            let problematic_positions = self.problematic_positions.clone();
            let toast_message = self.toast_message.clone();
            let error_type = self.error_type.clone();
            let completions = self.completions.clone();
            let completion_index = self.completion_index;
            let show_completions = self.show_completions;
            let query_results = self.query_results.clone();
            let results_scroll = self.results_scroll;
            let focus_mode = self.focus_mode.clone();
            let selected_result = self.selected_result;
            let cursor_visible = self.cursor_visible;
            let menu_index = self.menu_index;
            let cart_classes = self.cart_classes.clone();
            let selected_for_schedule = self.selected_for_schedule.clone();
            let generated_schedules = self.generated_schedules.clone();
            let current_schedule_index = self.current_schedule_index;
            let schedule_cart_focus = self.schedule_cart_focus;
            let selected_cart_index = self.selected_cart_index;
            let schedule_selection_mode = self.schedule_selection_mode;
            let selected_time_block_day = self.selected_time_block_day;
            let selected_time_block_slot = self.selected_time_block_slot;
            let detail_return_focus = self.detail_return_focus.clone();
            let saved_schedules = self.saved_schedules.clone();
            let selected_saved_schedule_index = self.selected_saved_schedule_index;
            let current_saved_schedule_name = self.current_saved_schedule_name.clone();
            let save_name_input = self.save_name_input.clone();
            let save_name_cursor_visible = self.save_name_cursor_visible;
            let selected_class_for_details = self.selected_class_for_details.clone();

            // draw the current state
            let terminal = &mut self.terminal;
            let mut new_guide_max_scroll = self.guide_max_scroll;
            terminal.draw(|f| {
                let (_, max_items) = render_frame(
                    f,
                    &input,
                    &problematic_positions,
                    &toast_message,
                    &error_type,
                    &completions,
                    completion_index,
                    show_completions,
                    &query_results,
                    results_scroll,
                    &focus_mode,
                    selected_result,
                    cursor_visible,
                    menu_index,
                    self.current_theme,
                    self.settings_index,
                    self.guide_scroll,
                    &mut new_guide_max_scroll,
                    &self.user_query,
                    &cart_classes,
                    &selected_for_schedule,
                    &generated_schedules,
                    current_schedule_index,
                    schedule_cart_focus,
                    selected_cart_index,
                    schedule_selection_mode,
                    selected_time_block_day,
                    selected_time_block_slot,
                    detail_return_focus,
                    &saved_schedules,
                    selected_saved_schedule_index,
                    current_saved_schedule_name.as_deref(),
                    selected_class_for_details.as_ref(),
                    &save_name_input,
                    save_name_cursor_visible,
                    &self.available_schools,
                    self.selected_school_index,
                    self.selected_school_id.as_deref(),
                    self.school_scroll_offset,
                    &self.available_terms,
                    self.selected_term_index,
                    self.selected_term_id.as_deref(),
                    self.term_scroll_offset,
                    self.last_sync_time.as_deref(),
                    self.is_syncing,
                    self.school_picker_open,
                    self.term_picker_open,
                );
                // update max_items_that_fit based on actual rendering
                self.max_items_that_fit = max_items;
            })?;
            // update guide_max_scroll after render
            self.guide_max_scroll = new_guide_max_scroll;
            if focus_mode == FocusMode::QueryGuide {
                // clamp guide_scroll to valid range
                if self.guide_max_scroll > 0 {
                    self.guide_scroll = self.guide_scroll.min(self.guide_max_scroll);
                } else {
                    self.guide_scroll = 0;
                }
            }

            // handle input events with timeout for cursor blinking
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // handle main menu mode
                    if self.focus_mode == FocusMode::MainMenu {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(())
                            }
                            KeyCode::Esc => break Ok(()),
                            KeyCode::Up => {
                                if self.menu_index > 0 {
                                    self.menu_index -= 1;
                                } else {
                                    self.menu_index = MenuOption::all().len() - 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.menu_index < MenuOption::all().len() - 1 {
                                    self.menu_index += 1;
                                } else {
                                    self.menu_index = 0;
                                }
                            }
                            KeyCode::Enter => {
                                let options = MenuOption::all();
                                if self.menu_index < options.len() {
                                    match options[self.menu_index] {
                                        MenuOption::Search => {
                                            self.focus_mode = FocusMode::QueryInput;
                                        }
                                        MenuOption::ScheduleCreation => {
                                            // check if cart is empty
                                            if self.cart_classes.is_empty() {
                                                self.show_toast(
                                                    "Cart is empty! Add classes to cart first.".to_string(),
                                                    ErrorType::Semantic,
                                                );
                                            } else {
                                                // Initialize selected_for_schedule with all cart items if empty
                                                if self.selected_for_schedule.is_empty() {
                                                    self.selected_for_schedule = self.cart_classes.keys().cloned().collect();
                                                }
                                                // enter class selection mode (don't generate schedules yet)
                                                self.focus_mode = FocusMode::ScheduleCreation;
                                                self.schedule_selection_mode = true;
                                                self.current_schedule_index = 0;
                                                self.schedule_cart_focus = true;
                                                self.selected_cart_index = 0;
                                                self.generated_schedules.clear(); // clear any previous schedules
                                            }
                                        }
                                        MenuOption::MySchedules => {
                                            // load saved schedules and enter MySchedules view
                                            match save::load_all_schedules() {
                                                Ok(schedules) => {
                                                    self.saved_schedules = schedules;
                                                    self.selected_saved_schedule_index = 0;
                                                    self.focus_mode = FocusMode::MySchedules;
                                                }
                                                Err(e) => {
                                                    self.show_toast(
                                                        format!("Failed to load schedules: {}", e),
                                                        ErrorType::Semantic,
                                                    );
                                                }
                                            }
                                        }
                                        MenuOption::Help => {
                                            // open the query guide as the help page from main menu
                                            self.guide_return_focus = FocusMode::MainMenu;
                                            self.focus_mode = FocusMode::QueryGuide;
                                            self.guide_scroll = 0;
                                        }
                                        MenuOption::Settings => {
                                            self.load_school_data();
                                            self.focus_mode = FocusMode::Settings;
                                        }
                                        MenuOption::Quit => break Ok(()),
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle completion navigation first if completions are showing
                    if self.show_completions {
                        match key.code {
                            KeyCode::Esc => {
                                self.show_completions = false;
                                self.completion_index = None;
                                self.partial_word.clear();
                            }
                            KeyCode::Up => {
                                if let Some(index) = self.completion_index {
                                    if index > 0 {
                                        self.completion_index = Some(index - 1);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if let Some(index) = self.completion_index {
                                    if index < self.completions.len() - 1 {
                                        self.completion_index = Some(index + 1);
                                    } else {
                                        self.completion_index = Some(0);
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // insert selected completion
                                if let Some(index) = self.completion_index {
                                    if index < self.completions.len() {
                                        let completion = &self.completions[index];
                                        // don't add placeholders like <value>
                                        if !completion.starts_with('<') {
                                            // only replace if there's a partial word that matches
                                            if !self.partial_word.is_empty()
                                                && completion
                                                    .to_lowercase()
                                                    .starts_with(&self.partial_word)
                                            {
                                                // remove the partial word from input
                                                let trim_len = self.partial_word.len();
                                                let new_len =
                                                    self.input.len().saturating_sub(trim_len);
                                                self.input.truncate(new_len);
                                            } else {
                                                // no partial word - just append with space
                                                if !self.input.is_empty()
                                                    && !self.input.ends_with(' ')
                                                {
                                                    self.input.push(' ');
                                                }
                                            }
                                            self.input.push_str(completion);
                                            if !completion.starts_with('"') {
                                                // add space after completion for next word
                                                self.input.push(' ');
                                            }
                                        }
                                    }
                                }
                                self.show_completions = false;
                                self.completion_index = None;
                                self.partial_word.clear();
                            }
                            KeyCode::Tab => {
                                // tab moves down through completions (same as Down arrow)
                                if let Some(index) = self.completion_index {
                                    if index < self.completions.len() - 1 {
                                        self.completion_index = Some(index + 1);
                                    } else {
                                        self.completion_index = Some(0);
                                    }
                                }
                            }
                            _ => {
                                // any other key hides completions
                                self.show_completions = false;
                                self.completion_index = None;
                                self.partial_word.clear();
                            }
                        }
                        continue; // skip normal key handling when completions are shown
                    }

                    // handle schedule creation mode
                    if self.focus_mode == FocusMode::ScheduleCreation {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(())
                            }
                            KeyCode::Esc => {
                                if self.schedule_selection_mode {
                                    // exit schedule creation, go back to main menu
                                    self.focus_mode = FocusMode::MainMenu;
                                } else {
                                    // if class details are showing, close detail view first
                                    if self.focus_mode == FocusMode::DetailView {
                                        self.selected_class_for_details = None;
                                        self.focus_mode = FocusMode::ScheduleCreation;
                                    } else {
                                        // check if we came from MySchedules
                                        if self.detail_return_focus == FocusMode::MySchedules {
                                            // go back to MySchedules view
                                            self.focus_mode = FocusMode::MySchedules;
                                            self.current_saved_schedule_name = None;
                                        } else {
                                            // go back to class selection mode
                                            self.schedule_selection_mode = true;
                                            self.schedule_cart_focus = true;
                                            self.generated_schedules.clear();
                                        }
                                    }
                                }
                            }
                            KeyCode::Up => {
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
                            }
                            KeyCode::Down => {
                                if self.schedule_selection_mode {
                                    // navigate cart items down
                                    // get classes in the same sorted order as rendering
                                    let mut cart_classes_vec: Vec<&Class> = self.cart_classes.values().collect();
                                    cart_classes_vec.sort_by_key(|class| class.unique_id());
                                    let cart_class_ids: Vec<String> = cart_classes_vec
                                        .iter()
                                        .map(|class| class.unique_id())
                                        .collect();
                                    if !cart_class_ids.is_empty() 
                                        && self.selected_cart_index < cart_class_ids.len() - 1 {
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
                            }
                            KeyCode::Left => {
                                if !self.schedule_selection_mode {
                                    // navigate time blocks: left = previous day
                                    if self.selected_time_block_day > 0 {
                                        self.selected_time_block_day -= 1;
                                    } else {
                                        // wrap to Sunday
                                        self.selected_time_block_day = 6;
                                    }
                                }
                            }
                            KeyCode::Right => {
                                if !self.schedule_selection_mode {
                                    // navigate time blocks: right = next day
                                    if self.selected_time_block_day < 6 {
                                        self.selected_time_block_day += 1;
                                    } else {
                                        // wrap to Monday
                                        self.selected_time_block_day = 0;
                                    }
                                }
                            }
                            KeyCode::PageUp => {
                                if !self.schedule_selection_mode {
                                    // if viewing from MySchedules, navigate to previous saved schedule
                                    if self.detail_return_focus == FocusMode::MySchedules {
                                        if self.selected_saved_schedule_index > 0 {
                                            self.selected_saved_schedule_index -= 1;
                                            let saved = &self.saved_schedules[self.selected_saved_schedule_index];
                                            self.generated_schedules = vec![saved.classes.clone()];
                                            self.current_saved_schedule_name = Some(saved.name.clone());
                                            self.current_schedule_index = 0;
                                        } else {
                                            // wrap to last saved schedule
                                            self.selected_saved_schedule_index = self.saved_schedules.len().saturating_sub(1);
                                            let saved = &self.saved_schedules[self.selected_saved_schedule_index];
                                            self.generated_schedules = vec![saved.classes.clone()];
                                            self.current_saved_schedule_name = Some(saved.name.clone());
                                            self.current_schedule_index = 0;
                                        }
                                    } else {
                                        // navigate to previous generated schedule
                                        if !self.generated_schedules.is_empty() {
                                            if self.current_schedule_index > 0 {
                                                self.current_schedule_index -= 1;
                                            } else {
                                                self.current_schedule_index = self.generated_schedules.len() - 1;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::PageDown => {
                                if !self.schedule_selection_mode {
                                    // if viewing from MySchedules, navigate to next saved schedule
                                    if self.detail_return_focus == FocusMode::MySchedules {
                                        if self.selected_saved_schedule_index < self.saved_schedules.len().saturating_sub(1) {
                                            self.selected_saved_schedule_index += 1;
                                            let saved = &self.saved_schedules[self.selected_saved_schedule_index];
                                            self.generated_schedules = vec![saved.classes.clone()];
                                            self.current_saved_schedule_name = Some(saved.name.clone());
                                            self.current_schedule_index = 0;
                                        } else {
                                            // wrap to first saved schedule
                                            self.selected_saved_schedule_index = 0;
                                            let saved = &self.saved_schedules[self.selected_saved_schedule_index];
                                            self.generated_schedules = vec![saved.classes.clone()];
                                            self.current_saved_schedule_name = Some(saved.name.clone());
                                            self.current_schedule_index = 0;
                                        }
                                    } else {
                                        // navigate to next generated schedule
                                        if !self.generated_schedules.is_empty() {
                                            if self.current_schedule_index < self.generated_schedules.len() - 1 {
                                                self.current_schedule_index += 1;
                                            } else {
                                                self.current_schedule_index = 0;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if self.schedule_selection_mode {
                                    // generate schedules and switch to viewing mode
                                    if self.selected_for_schedule.is_empty() {
                                        self.show_toast(
                                            "No classes selected! Select classes first.".to_string(),
                                            ErrorType::Semantic,
                                        );
                                    } else {
                                        // generate valid (non-conflicting) schedules
                                        self.generated_schedules = generate_schedules(&self.cart_classes, &self.selected_for_schedule, false);
                                        if self.generated_schedules.is_empty() {
                                            // no valid schedules found - show which classes conflict
                                            let selected_classes: Vec<Class> = self.selected_for_schedule
                                                .iter()
                                                .filter_map(|class_id| self.cart_classes.get(class_id))
                                                .cloned()
                                                .collect();
                                            let conflicts = find_conflicting_classes(&selected_classes);
                                            let conflict_msg = if conflicts.len() == 1 {
                                                format!("No valid schedules. Classes conflict: {} and {}", conflicts[0].0, conflicts[0].1)
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
                                            self.show_toast(conflict_msg, ErrorType::Semantic);
                                        } else {
                                            // valid schedules found - proceed to viewing mode
                                            self.schedule_selection_mode = false;
                                            self.current_schedule_index = 0;
                                            self.selected_time_block_day = 0;
                                            self.selected_time_block_slot = 0;
                                        }
                                    }
                                } else {
                                    // show class details in detail view
                                    if !self.generated_schedules.is_empty() 
                                        && self.current_schedule_index < self.generated_schedules.len() {
                                        let schedule = &self.generated_schedules[self.current_schedule_index];
                                        // find class at selected time block
                                        if let Some(class) = find_class_at_time_block(
                                            schedule,
                                            self.selected_time_block_day,
                                            self.selected_time_block_slot,
                                        ) {
                                            // set selected class and open detail view
                                            self.selected_class_for_details = Some(class.clone());
                                            // ensure detail_return_focus is set correctly
                                            if self.detail_return_focus != FocusMode::MySchedules {
                                                self.detail_return_focus = FocusMode::ScheduleCreation;
                                            }
                                            self.focus_mode = FocusMode::DetailView;
                                        } else {
                                            // no class at this time block, clear selection
                                            self.selected_class_for_details = None;
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                if !self.schedule_selection_mode && !self.generated_schedules.is_empty() {
                                    // save current schedule - enter name input mode
                                    self.save_name_input.clear();
                                    self.focus_mode = FocusMode::SaveNameInput;
                                }
                            }
                            KeyCode::Char(' ') => {
                                if self.schedule_selection_mode {
                                    // toggle selected cart item for schedule generation
                                    // get classes in the same sorted order as rendering
                                    let mut cart_classes_vec: Vec<&Class> = self.cart_classes.values().collect();
                                    cart_classes_vec.sort_by_key(|class| class.unique_id());
                                    let cart_class_ids: Vec<String> = cart_classes_vec
                                        .iter()
                                        .map(|class| class.unique_id())
                                        .collect();
                                    if self.selected_cart_index < cart_class_ids.len() {
                                        let class_id = &cart_class_ids[self.selected_cart_index];

                                        // toggle: add/remove from selected_for_schedule (not cart)
                                        if self.selected_for_schedule.contains(class_id) {
                                            self.selected_for_schedule.remove(class_id);
                                        } else {
                                            self.selected_for_schedule.insert(class_id.clone());
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('c') | KeyCode::Char('C') => {
                                if self.schedule_selection_mode {
                                    // remove selected cart item from cart
                                    // get classes in the same sorted order as rendering
                                    let mut cart_classes_vec: Vec<&Class> = self.cart_classes.values().collect();
                                    cart_classes_vec.sort_by_key(|class| class.unique_id());
                                    let cart_class_ids: Vec<String> = cart_classes_vec
                                        .iter()
                                        .map(|class| class.unique_id())
                                        .collect();
                                    if self.selected_cart_index < cart_class_ids.len() {
                                        let class_id = &cart_class_ids[self.selected_cart_index];

                                        // remove from cart and selected_for_schedule
                                        self.cart_classes.remove(class_id);
                                        self.selected_for_schedule.remove(class_id);
                                        
                                        // adjust selected index if needed
                                        if self.selected_cart_index >= self.cart_classes.len() && !self.cart_classes.is_empty() {
                                            self.selected_cart_index = self.cart_classes.len() - 1;
                                        } else if self.cart_classes.is_empty() {
                                            self.selected_cart_index = 0;
                                        }
                                    }
                                }
                            }
                            KeyCode::Tab => {
                                if self.schedule_selection_mode {
                                    // open detail view for selected class
                                    // get classes in the same sorted order as rendering
                                    let mut cart_classes_vec: Vec<&Class> = self.cart_classes.values().collect();
                                    cart_classes_vec.sort_by_key(|class| class.unique_id());
                                    let cart_class_ids: Vec<String> = cart_classes_vec
                                        .iter()
                                        .map(|class| class.unique_id())
                                        .collect();
                                    if self.selected_cart_index < cart_class_ids.len() {
                                        let class_id = &cart_class_ids[self.selected_cart_index];
                                        if let Some(class_idx) = self.query_results.iter().position(|c| c.unique_id() == *class_id) {
                                            self.selected_result = class_idx;
                                            self.detail_return_focus = FocusMode::ScheduleCreation;
                                            self.focus_mode = FocusMode::DetailView;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle settings mode
                    if self.focus_mode == FocusMode::Settings {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(())
                            }
                            KeyCode::Esc => {
                                if self.school_picker_open {
                                    self.school_picker_open = false;
                                } else if self.term_picker_open {
                                    self.term_picker_open = false;
                                } else {
                                    self.focus_mode = FocusMode::MainMenu;
                                }
                            }
                            KeyCode::Up => {
                                if self.school_picker_open {
                                    if self.selected_school_index > 0 {
                                        self.selected_school_index -= 1;
                                        // scroll up if needed
                                        if self.selected_school_index < self.school_scroll_offset {
                                            self.school_scroll_offset = self.selected_school_index;
                                        }
                                    }
                                } else if self.term_picker_open {
                                    if self.selected_term_index > 0 {
                                        self.selected_term_index -= 1;
                                        // scroll up if needed
                                        if self.selected_term_index < self.term_scroll_offset {
                                            self.term_scroll_offset = self.selected_term_index;
                                        }
                                    }
                                } else if self.settings_index > 0 {
                                    self.settings_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                const PICKER_MAX_VISIBLE: usize = 6;
                                if self.school_picker_open {
                                    let max = self.available_schools.len().saturating_sub(1);
                                    if self.selected_school_index < max {
                                        self.selected_school_index += 1;
                                        // scroll down if needed
                                        if self.selected_school_index >= self.school_scroll_offset + PICKER_MAX_VISIBLE {
                                            self.school_scroll_offset = self.selected_school_index - PICKER_MAX_VISIBLE + 1;
                                        }
                                    }
                                } else if self.term_picker_open {
                                    let max = self.available_terms.len().saturating_sub(1);
                                    if self.selected_term_index < max {
                                        self.selected_term_index += 1;
                                        // scroll down if needed
                                        if self.selected_term_index >= self.term_scroll_offset + PICKER_MAX_VISIBLE {
                                            self.term_scroll_offset = self.selected_term_index - PICKER_MAX_VISIBLE + 1;
                                        }
                                    }
                                } else {
                                    let max_index = 3; // theme, school, term, sync
                                    if self.settings_index < max_index {
                                        self.settings_index += 1;
                                    }
                                }
                            }
                            KeyCode::Left | KeyCode::Right => {
                                // change theme when on Theme option
                                if self.settings_index == 0 {
                                    let themes = ThemePalette::all();
                                    let current_idx = themes
                                        .iter()
                                        .position(|&t| t == self.current_theme)
                                        .unwrap_or(0);
                                    if key.code == KeyCode::Left {
                                        let new_idx = if current_idx > 0 {
                                            current_idx - 1
                                        } else {
                                            themes.len() - 1
                                        };
                                        self.current_theme = themes[new_idx];
                                    } else {
                                        let new_idx = if current_idx < themes.len() - 1 {
                                            current_idx + 1
                                        } else {
                                            0
                                        };
                                        self.current_theme = themes[new_idx];
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                match self.settings_index {
                                    1 => {
                                        // school selection
                                        if self.school_picker_open {
                                            // clone data first to avoid borrow issues
                                            let school_data = self.available_schools
                                                .get(self.selected_school_index)
                                                .map(|s| (s.id.clone(), s.name.clone()));
                                            
                                            if let Some((school_id, school_name)) = school_data {
                                                self.selected_school_id = Some(school_id.clone());
                                                self.compiler.set_school_id(Some(school_id.clone()));
                                                // load terms for selected school
                                                self.load_terms(school_id);
                                                // clear term selection when school changes
                                                self.selected_term_id = None;
                                                self.compiler.set_term_id(None);
                                                self.show_toast(format!("Selected: {}", school_name), ErrorType::Success);
                                            }
                                            self.school_picker_open = false;
                                        } else if !self.available_schools.is_empty() {
                                            self.school_picker_open = true;
                                            if let Some(ref id) = self.selected_school_id {
                                                self.selected_school_index = self.available_schools
                                                    .iter()
                                                    .position(|s| &s.id == id)
                                                    .unwrap_or(0);
                                            }
                                        } else {
                                            self.show_toast("No schools available. Sync first!".to_string(), ErrorType::Warning);
                                        }
                                    }
                                    2 => {
                                        // term selection
                                        if self.term_picker_open {
                                            if let Some(term) = self.available_terms.get(self.selected_term_index) {
                                                self.selected_term_id = Some(term.id.clone());
                                                self.compiler.set_term_id(Some(term.id.clone()));
                                                self.show_toast(format!("Selected: {}", term.name), ErrorType::Success);
                                            }
                                            self.term_picker_open = false;
                                        } else if self.selected_school_id.is_none() {
                                            self.show_toast("Select a school first!".to_string(), ErrorType::Warning);
                                        } else if !self.available_terms.is_empty() {
                                            self.term_picker_open = true;
                                            if let Some(ref id) = self.selected_term_id {
                                                self.selected_term_index = self.available_terms
                                                    .iter()
                                                    .position(|t| &t.id == id)
                                                    .unwrap_or(0);
                                            }
                                        } else {
                                            self.show_toast("No terms available for this school.".to_string(), ErrorType::Warning);
                                        }
                                    }
                                    3 => {
                                        // trigger sync
                                        if !self.is_syncing {
                                            self.is_syncing = true;
                                            self.show_toast("Starting sync...".to_string(), ErrorType::Info);
                                            
                                            match crate::data::sync::SyncConfig::from_env() {
                                                Ok(config) => {
                                                    match crate::data::sync::sync_all(&config) {
                                                        Ok(_db_path) => {
                                                            self.show_toast("Sync completed successfully!".to_string(), ErrorType::Success);
                                                            self.load_school_data();
                                                        }
                                                        Err(e) => {
                                                            self.show_toast(format!("Sync failed: {}", e), ErrorType::Warning);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    self.show_toast(format!("Config error: {}", e), ErrorType::Warning);
                                                }
                                            }
                                            self.is_syncing = false;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle query guide mode
                    if self.focus_mode == FocusMode::QueryGuide {
                        match key.code {
                            KeyCode::Esc => {
                                // exit query guide, go back to the saved previous focus mode
                                self.focus_mode = self.guide_return_focus.clone();
                                self.guide_scroll = 0; // reset scroll when closing
                            }
                            KeyCode::Char('g') | KeyCode::Char('G')
                                if key.modifiers.contains(KeyModifiers::ALT) =>
                            {
                                // exit query guide, go back to the saved previous focus mode
                                self.focus_mode = self.guide_return_focus.clone();
                                self.guide_scroll = 0; // reset scroll when closing
                            }
                            KeyCode::Up => {
                                // scroll up
                                if self.guide_scroll > 0 {
                                    self.guide_scroll -= 1;
                                }
                            }
                            KeyCode::Down => {
                                // scroll down - clamp to max_scroll
                                if self.guide_max_scroll > 0 {
                                    self.guide_scroll = (self.guide_scroll + 1).min(self.guide_max_scroll);
                                } else {
                                    // max_scroll not calculated yet, allow incrementing
                                    self.guide_scroll += 1;
                                }
                            }
                            KeyCode::PageUp => {
                                // scroll up by page (10 lines)
                                if self.guide_scroll >= 10 {
                                    self.guide_scroll -= 10;
                                } else {
                                    self.guide_scroll = 0;
                                }
                            }
                            KeyCode::PageDown => {
                                // scroll down by page (10 lines) - clamp to max_scroll
                                if self.guide_max_scroll > 0 {
                                    self.guide_scroll = (self.guide_scroll + 10).min(self.guide_max_scroll);
                                } else {
                                    self.guide_scroll += 10;
                                }
                            }
                            KeyCode::Home => {
                                // scroll to top
                                self.guide_scroll = 0;
                            }
                            KeyCode::End => {
                                // scroll to bottom
                                if self.guide_max_scroll > 0 {
                                    self.guide_scroll = self.guide_max_scroll;
                                } else {
                                    // max_scroll not calculated yet, set large value
                                    self.guide_scroll = 10000;
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle save name input mode
                    if self.focus_mode == FocusMode::SaveNameInput {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(())
                            }
                            KeyCode::Esc => {
                                // cancel saving, go back to schedule view
                                self.focus_mode = FocusMode::ScheduleCreation;
                                self.save_name_input.clear();
                            }
                            KeyCode::Enter => {
                                // save the schedule
                                if self.save_name_input.trim().is_empty() {
                                    self.show_toast(
                                        "Schedule name cannot be empty!".to_string(),
                                        ErrorType::Semantic,
                                    );
                                } else if !self.generated_schedules.is_empty() 
                                    && self.current_schedule_index < self.generated_schedules.len() {
                                    let schedule = &self.generated_schedules[self.current_schedule_index];
                                    match save::save_schedule(&self.save_name_input.trim(), schedule) {
                                        Ok(_) => {
                                            self.show_toast(
                                                format!("Schedule '{}' saved!", self.save_name_input.trim()),
                                                ErrorType::Semantic,
                                            );
                                            self.focus_mode = FocusMode::ScheduleCreation;
                                            self.save_name_input.clear();
                                        }
                                        Err(e) => {
                                            self.show_toast(
                                                format!("Failed to save schedule: {}", e),
                                                ErrorType::Semantic,
                                            );
                                        }
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                self.save_name_input.pop();
                            }
                            KeyCode::Char(c) => {
                                self.save_name_input.push(c);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle my schedules mode
                    if self.focus_mode == FocusMode::MySchedules {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(())
                            }
                            KeyCode::Esc => {
                                self.focus_mode = FocusMode::MainMenu;
                            }
                            KeyCode::Up => {
                                if self.selected_saved_schedule_index > 0 {
                                    self.selected_saved_schedule_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.selected_saved_schedule_index < self.saved_schedules.len().saturating_sub(1) {
                                    self.selected_saved_schedule_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                // view schedule in detail
                                if self.selected_saved_schedule_index < self.saved_schedules.len() {
                                    let saved = &self.saved_schedules[self.selected_saved_schedule_index];
                                    // set generated_schedules to show this saved schedule
                                    self.generated_schedules = vec![saved.classes.clone()];
                                    self.current_schedule_index = 0;
                                    self.schedule_selection_mode = false;
                                    self.selected_time_block_day = 0;
                                    self.selected_time_block_slot = 0;
                                    self.current_saved_schedule_name = Some(saved.name.clone());
                                    self.detail_return_focus = FocusMode::MySchedules;
                                    self.focus_mode = FocusMode::ScheduleCreation;
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                // delete selected schedule
                                if self.selected_saved_schedule_index < self.saved_schedules.len() {
                                    let saved_name = self.saved_schedules[self.selected_saved_schedule_index].name.clone();
                                    let saved_timestamp = self.saved_schedules[self.selected_saved_schedule_index].timestamp;
                                    match save::delete_schedule(saved_timestamp) {
                                        Ok(_) => {
                                            self.saved_schedules.remove(self.selected_saved_schedule_index);
                                            if self.selected_saved_schedule_index >= self.saved_schedules.len() 
                                                && !self.saved_schedules.is_empty() {
                                                self.selected_saved_schedule_index = self.saved_schedules.len() - 1;
                                            } else if self.saved_schedules.is_empty() {
                                                self.selected_saved_schedule_index = 0;
                                            }
                                            self.show_toast(
                                                format!("Schedule '{}' deleted", saved_name),
                                                ErrorType::Semantic,
                                            );
                                        }
                                        Err(e) => {
                                            self.show_toast(
                                                format!("Failed to delete schedule: {}", e),
                                                ErrorType::Semantic,
                                            );
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle detail view mode
                    if self.focus_mode == FocusMode::DetailView {
                        match key.code {
                            KeyCode::Esc | KeyCode::Backspace => {
                                // exit detail view, go back to where we came from
                                // if we came from MySchedules, go back to ScheduleCreation first
                                if self.detail_return_focus == FocusMode::MySchedules {
                                    self.focus_mode = FocusMode::ScheduleCreation;
                                } else {
                                    self.focus_mode = self.detail_return_focus.clone();
                                }
                            }
                            KeyCode::Enter => {
                                // exit detail view, go back to where we came from
                                // if we came from MySchedules, go back to ScheduleCreation first
                                if self.detail_return_focus == FocusMode::MySchedules {
                                    self.focus_mode = FocusMode::ScheduleCreation;
                                } else {
                                    self.focus_mode = self.detail_return_focus.clone();
                                }
                            }
                            KeyCode::Char('g') | KeyCode::Char('G')
                                if key.modifiers.contains(KeyModifiers::ALT) =>
                            {
                                // open query guide from detail view
                                self.guide_return_focus = FocusMode::DetailView;
                                self.focus_mode = FocusMode::QueryGuide;
                                self.guide_scroll = 0;
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                // toggle cart for current class (only if not from schedule mode)
                                if self.detail_return_focus != FocusMode::ScheduleCreation {
                                    if self.selected_result < self.query_results.len() {
                                        let class = &self.query_results[self.selected_result];
                                        let class_id = class.unique_id();
                                        if self.cart_classes.contains_key(&class_id) {
                                            self.cart_classes.remove(&class_id);
                                        } else {
                                            self.cart_classes.insert(class_id, class.clone());
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // handle results browse mode
                    if self.focus_mode == FocusMode::ResultsBrowse {
                        match key.code {
                            // exit the TUI if the user presses Ctrl+C
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break Ok(())
                            }
                            // esc goes back to main menu
                            KeyCode::Esc => {
                                self.focus_mode = FocusMode::MainMenu;
                            }
                            KeyCode::Char('g') | KeyCode::Char('G')
                                if key.modifiers.contains(KeyModifiers::ALT) =>
                            {
                                // open query guide from results browse
                                self.guide_return_focus = FocusMode::ResultsBrowse;
                                self.focus_mode = FocusMode::QueryGuide;
                                self.guide_scroll = 0;
                            }
                            KeyCode::Up => {
                                // if at top of results, go back to query input
                                if self.selected_result == 0 {
                                    self.focus_mode = FocusMode::QueryInput;
                                } else {
                                    // move selection up (3 items per row)
                                    let cols = 3;
                                    if self.selected_result >= cols {
                                        self.selected_result -= cols;
                                        // adjust scroll if needed
                                        if self.selected_result < self.results_scroll {
                                            // align scroll to row boundaries (multiples of cols) to preserve columns
                                            let target_row = self.selected_result / cols;
                                            self.results_scroll = target_row * cols;
                                        }
                                    } else {
                                        // can't go up more, go to query input
                                        self.focus_mode = FocusMode::QueryInput;
                                    }
                                }
                            }
                            KeyCode::Down => {
                                // move selection down (3 items per row)
                                let cols = 3;
                                if self.selected_result + cols < self.query_results.len() {
                                    self.selected_result += cols;
                                    // only scroll if result is not already visible
                                    let total_results = self.query_results.len();
                                    let max_visible = self.max_items_that_fit;
                                    // if all results fit on screen, don't scroll
                                    if total_results <= max_visible {
                                        // all results visible, don't scroll
                                        self.results_scroll = 0; // ensure scroll is reset
                                    } else if self.selected_result
                                        >= self.results_scroll + max_visible
                                    {
                                        // result is beyond visible window, scroll to show it
                                        // align scroll to row boundaries (multiples of cols) to preserve columns
                                        let rows_visible = max_visible / cols;
                                        let current_row = self.selected_result / cols;
                                        let scroll_row =
                                            current_row.saturating_sub(rows_visible - 1);
                                        self.results_scroll = (scroll_row * cols).max(0);
                                    }
                                }
                            }
                            KeyCode::Left => {
                                // move selection left
                                if self.selected_result > 0 {
                                    self.selected_result -= 1;
                                    // only scroll if result is not already visible
                                    if self.selected_result < self.results_scroll {
                                        self.results_scroll = self.selected_result;
                                    }
                                }
                            }
                            KeyCode::Right => {
                                // move selection right
                                if self.selected_result + 1 < self.query_results.len() {
                                    self.selected_result += 1;
                                    // only scroll if result is not already visible
                                    let total_results = self.query_results.len();
                                    let max_visible = self.max_items_that_fit;
                                    // if all results fit on screen, don't scroll
                                    if total_results <= max_visible {
                                        // all results visible, don't scroll
                                        self.results_scroll = 0; // ensure scroll is reset
                                    } else if self.selected_result
                                        >= self.results_scroll + max_visible
                                    {
                                        // result is beyond visible window, scroll to show it
                                        self.results_scroll =
                                            self.selected_result.saturating_sub(max_visible - 1);
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // open detail view for selected result
                                if self.selected_result < self.query_results.len() {
                                    self.detail_return_focus = FocusMode::ResultsBrowse;
                                    self.focus_mode = FocusMode::DetailView;
                                }
                            }
                            KeyCode::Char(_) | KeyCode::Backspace => {
                                // typing goes back to query input
                                self.focus_mode = FocusMode::QueryInput;
                                if let KeyCode::Char(c) = key.code {
                                    self.clear_error_state();
                                    self.input.push(c);
                                } else {
                                    self.clear_error_state();
                                    self.input.pop();
                                }
                            }
                            KeyCode::Tab => {
                                // go back to query input for tab completion
                                self.focus_mode = FocusMode::QueryInput;
                                self.handle_tab_completion();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // queryInput mode (default)
                    match key.code {
                        // exit the TUI if the user presses Ctrl+C
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break Ok(())
                        }
                        // esc goes back to main menu
                        KeyCode::Esc => {
                            self.focus_mode = FocusMode::MainMenu;
                        }
                        // alt+g opens query guide
                        KeyCode::Char('g') | KeyCode::Char('G')
                            if key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            // open query guide from query input
                            self.guide_return_focus = FocusMode::QueryInput;
                            self.focus_mode = FocusMode::QueryGuide;
                            self.guide_scroll = 0;
                        }
                        KeyCode::Down => {
                            // if we have results, move to top left result
                            if !self.query_results.is_empty() {
                                self.focus_mode = FocusMode::ResultsBrowse;
                                self.selected_result = 0;
                                // only change scroll if result 0 is not already visible
                                // (i.e., if scroll > 0, or if all results don't fit and we're scrolled)
                                // For simplicity: only reset scroll if it's not 0 and result 0 isn't visible
                                if self.results_scroll > 0 {
                                    self.results_scroll = 0;
                                }
                                // If scroll is already 0, don't change it (all results already visible)
                            }
                        }

                        // use compiler to process the query
                        KeyCode::Enter => {
                            // process the query here
                            self.user_query = self.input.clone();

                            // run the compiler and handle the result
                            match self.compiler.run(&self.input) {
                                CompilerResult::Success { classes, .. } => {
                                    // clear any error state
                                    self.toast_message = None;
                                    self.toast_start_time = None;
                                    self.error_type = None;
                                    self.problematic_positions.clear();

                                    // store the query results
                                    self.query_results = classes;
                                    self.results_scroll = 0;
                                    self.selected_result = 0;
                                }
                                CompilerResult::LexerError {
                                    message,
                                    problematic_positions,
                                } => {
                                    // show error and highlight problematic tokens
                                    self.show_toast(message, ErrorType::Lexer);
                                    self.problematic_positions = problematic_positions;
                                }
                                CompilerResult::ParserError {
                                    message,
                                    problematic_positions,
                                } => {
                                    // show error and highlight problematic tokens
                                    self.show_toast(message, ErrorType::Parser);
                                    self.problematic_positions = problematic_positions;
                                }
                                CompilerResult::SemanticError {
                                    message,
                                    problematic_positions,
                                } => {
                                    // show error and highlight problematic tokens
                                    self.show_toast(message, ErrorType::Semantic);
                                    self.problematic_positions = problematic_positions;
                                }
                                CompilerResult::CodeGenError { message } => {
                                    // show code generation error
                                    self.show_toast(message, ErrorType::Semantic);
                                    self.problematic_positions.clear();
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            // clear any previous toasts and problematic tokens when user backspaces
                            self.clear_error_state();
                            self.input.pop();
                        }
                        KeyCode::Tab => {
                            // handle tab completion
                            self.handle_tab_completion();
                        }
                        KeyCode::Char(c) => {
                            // clear any previous toasts and problematic tokens when user starts typing
                            self.clear_error_state();
                            self.input.push(c);
                        }
                        KeyCode::PageUp => {
                            // scroll results up (show earlier classes)
                            if self.results_scroll >= 3 {
                                self.results_scroll -= 3;
                            } else {
                                self.results_scroll = 0;
                            }
                        }
                        KeyCode::PageDown => {
                            // scroll results down (show later classes)
                            let max_scroll = self.query_results.len().saturating_sub(3);
                            if self.results_scroll + 3 < max_scroll {
                                self.results_scroll += 3;
                            } else {
                                self.results_scroll = max_scroll;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Clear the error state
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn clear_error_state(&mut self) {
        self.toast_message = None;
        self.toast_start_time = None;
        self.error_type = None;
        self.problematic_positions.clear();
    }

    /// Handle tab completion
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The TUI instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn handle_tab_completion(&mut self) {
        // check if input ends with space (no partial word to complete)
        let has_partial = !self.input.is_empty() && !self.input.ends_with(' ');

        // extract the potential partial word (last word after space)
        let potential_partial = if has_partial {
            self.input
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_lowercase()
        } else {
            String::new()
        };

        // get completion suggestions from compiler
        let suggestions = self.compiler.get_tab_completion(self.input.clone());

        // if there's a potential partial word, check if any suggestions match it
        // if matches exist, filter to those; otherwise it's a complete value, show all
        if !potential_partial.is_empty() {
            let matching: Vec<String> = suggestions
                .iter()
                .filter(|s| s.to_lowercase().starts_with(&potential_partial))
                .cloned()
                .collect();

            if !matching.is_empty() {
                // partial word matches some suggestions - filter to those
                self.partial_word = potential_partial;
                self.completions = matching;
            } else {
                // no matches - the "partial" is actually a complete value
                // show all suggestions without replacement
                self.partial_word = String::new();
                self.completions = suggestions;
            }
        } else {
            self.partial_word = String::new();
            self.completions = suggestions;
        };

        if !self.completions.is_empty() {
            self.show_completions = true;
            self.completion_index = Some(0);
        } else if !self.input.trim().is_empty() {
            // no completions available - show helpful hint based on context
            let hint = self.get_completion_hint();
            if !hint.is_empty() {
                self.toast_message = Some(hint);
                self.toast_start_time = Some(Instant::now());
                self.error_type = None; // not an error, just a hint
            }
        }
    }

    /// Get a helpful hint when no completions are available
    fn get_completion_hint(&self) -> String {
        let last_word = self.input.split_whitespace().last().unwrap_or("");
        let last_word_lower = last_word.to_lowercase();

        // check if last word is a condition operator that expects a value
        match last_word_lower.as_str() {
            "contains" | "is" | "equals" | "has" => {
                "Enter a value in quotes, e.g. \"Computer Science\"".to_string()
            }
            "=" | "!=" => "Enter a value, e.g. \"CS\" or 101".to_string(),
            "<" | ">" | "<=" | ">=" => "Enter a number, e.g. 3 or 100".to_string(),
            "with" => {
                // "starts with" or "ends with"
                "Enter a value in quotes, e.g. \"Intro\"".to_string()
            }
            "hours" => "Enter an operator (=, <, >, etc.) then a number".to_string(),
            "type" => "Enter a condition (is, equals, contains) then a value".to_string(),
            _ => String::new(),
        }
    }

    /// Terminate the TUI gracefully
    ///
    /// Parameters:
    /// --- ---
    /// self -> The TUI instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    pub fn terminate(&self) -> Result<(), TUIError> {
        ratatui::restore();
        Ok(())
    }

    /// Update the toast
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The TUI instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
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

    /// Show the toast
    ///
    /// Parameters:
    /// --- ---
    /// mut self -> The TUI instance
    /// message -> The message to show
    /// error_type -> The error type
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// None
    /// --- ---
    ///
    fn show_toast(&mut self, message: String, error_type: ErrorType) {
        self.toast_message = Some(message);
        self.toast_start_time = Some(Instant::now());
        self.error_type = Some(error_type);
    }

}

/// Main render function that orchestrates all rendering
///
/// This is a standalone function to avoid borrow checker conflicts with the Tui struct.
/// It coordinates all the individual render functions in the correct order.
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The current user input string
/// problematic_positions -> Byte ranges of problematic tokens for error highlighting
/// toast_message -> Optional toast notification message
/// error_type -> Type of error if any (Lexer, Parser, Semantic)
/// completions -> List of completion suggestions
/// completion_index -> Currently selected completion index
/// show_completions -> Whether completion dropdown is visible
/// query_results -> The query results (classes) to display
/// results_scroll -> The scroll offset for results display
/// focus_mode -> Current UI focus (QueryInput, ResultsBrowse, DetailView)
/// selected_result -> Index of currently selected result in browse mode
/// cursor_visible -> Whether the input cursor is visible (for blinking)
/// menu_index -> Index of currently selected menu option
/// current_theme -> Current theme
/// settings_index -> Index of currently selected settings option
/// guide_scroll -> The scroll offset for guide display
/// guide_max_scroll -> The maximum scroll offset for guide display
/// user_query -> The last executed query string
/// cart -> Set of class IDs in the cart
/// selected_for_schedule -> Set of class IDs selected for schedule generation
/// selected_time_block_day -> Index of currently selected day in schedule viewing mode
/// selected_time_block_slot -> Index of currently selected time slot in schedule viewing mode
/// detail_return_focus -> Where to return when closing detail view
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
#[allow(clippy::too_many_arguments)]
fn render_frame(
    frame: &mut Frame,
    input: &str,
    problematic_positions: &[(usize, usize)],
    toast_message: &Option<String>,
    error_type: &Option<ErrorType>,
    completions: &[String],
    completion_index: Option<usize>,
    show_completions: bool,
    query_results: &[Class],
    results_scroll: usize,
    focus_mode: &FocusMode,
    selected_result: usize,
    cursor_visible: bool,
    menu_index: usize,
    current_theme: ThemePalette,
    settings_index: usize,
    guide_scroll: usize,
    guide_max_scroll: &mut usize,
    user_query: &str,
    cart_classes: &std::collections::HashMap<String, Class>,
    selected_for_schedule: &std::collections::HashSet<String>,
    generated_schedules: &[Vec<Class>],
    current_schedule_index: usize,
    schedule_cart_focus: bool,
    selected_cart_index: usize,
    schedule_selection_mode: bool,
    selected_time_block_day: usize,
    selected_time_block_slot: usize,
    detail_return_focus: FocusMode,
    saved_schedules: &[SavedSchedule],
    selected_saved_schedule_index: usize,
    current_saved_schedule_name: Option<&str>,
    selected_class_for_details: Option<&Class>,
    save_name_input: &str,
    save_name_cursor_visible: bool,
    available_schools: &[School],
    selected_school_index: usize,
    selected_school_id: Option<&str>,
    school_scroll_offset: usize,
    available_terms: &[Term],
    selected_term_index: usize,
    selected_term_id: Option<&str>,
    term_scroll_offset: usize,
    last_sync_time: Option<&str>,
    is_syncing: bool,
    school_picker_open: bool,
    term_picker_open: bool,
) -> (usize, usize) {
    let theme = current_theme.to_theme();

    // check window size
    let buffer_height = frame.area().height;
    let buffer_width = frame.area().width;

    // check minimum window size requirements
    let min_height = 25_u16;
    let min_width = 80_u16;

    if buffer_height < min_height || buffer_width < min_width {
        // display error message in center of screen
        let error_msg = format!(
            "Window too small! Minimum size: {}x{} (Current: {}x{})",
            min_width, min_height, buffer_width, buffer_height
        );
        // ensure error message fits on screen and handle width constraints
        let error_width = error_msg.len() as u16;
        let error_x = if buffer_width >= error_width {
            (buffer_width.saturating_sub(error_width)) / 2
        } else {
            0 // if window is too narrow, start at left edge
        };
        let error_area = Rect {
            x: error_x,
            y: buffer_height.saturating_sub(1) / 2, // center vertically, ensure in bounds
            width: error_width.min(buffer_width.saturating_sub(error_x)), // don't exceed available width
            height: 1,
        };
        let error_paragraph = ratatui::widgets::Paragraph::new(error_msg)
            .style(ratatui::style::Style::default().fg(theme.error_color));
        frame.render_widget(error_paragraph, error_area);
        return (0, 0);
    }

    // clear the entire frame with the theme background color
    // use saturating_sub to ensure we don't go out of bounds
    let buffer = frame.buffer_mut();
    // use buffer's actual dimensions to prevent index errors
    let actual_width = buffer.area.width;
    let actual_height = buffer.area.height;

    // double-check dimensions match and are valid
    if actual_width < min_width || actual_height < min_height {
        // buffer dimensions don't meet minimum - show error and return early
        let error_msg = format!(
            "Window too small! Minimum size: {}x{} (Current: {}x{})",
            min_width, min_height, actual_width, actual_height
        );
        let error_width = error_msg.len() as u16;
        let error_x = if actual_width >= error_width {
            (actual_width.saturating_sub(error_width)) / 2
        } else {
            0
        };
        let error_y = actual_height.saturating_sub(1) / 2;
        let error_area = Rect {
            x: error_x,
            y: error_y.min(actual_height.saturating_sub(1)),
            width: error_width.min(actual_width.saturating_sub(error_x)),
            height: 1,
        };
        let error_paragraph = ratatui::widgets::Paragraph::new(error_msg)
            .style(ratatui::style::Style::default().fg(theme.error_color));
        frame.render_widget(error_paragraph, error_area);
        return (0, 0);
    }

    // use exclusive range: 0..actual_width gives 0 to actual_width-1
    // add extra safety check to ensure we never access out of bounds
    let safe_width = actual_width.min(buffer.area.width);
    let safe_height = actual_height.min(buffer.area.height);
    for y in 0..safe_height {
        for x in 0..safe_width {
            // double-check bounds before access
            if x < buffer.area.width && y < buffer.area.height {
                let cell = &mut buffer[(x, y)];
                cell.set_bg(theme.background_color);
            }
        }
    }

    // only render widgets if window is large enough
    render_logo(frame, &theme);

    // render main menu if in menu mode
    if *focus_mode == FocusMode::MainMenu {
        render_main_menu(frame, menu_index, &theme);
        render_search_helpers_with_data(
            frame,
            input,
            toast_message,
            query_results,
            focus_mode,
            &theme,
            None,
        );
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }

    // render settings if in settings mode
    if *focus_mode == FocusMode::Settings {
        let settings_state = SettingsState {
            current_theme,
            selected_index: settings_index,
            available_schools,
            selected_school_index,
            selected_school_id,
            school_scroll_offset,
            available_terms,
            selected_term_index,
            selected_term_id,
            term_scroll_offset,
            last_sync_time,
            is_syncing,
            school_picker_open,
            term_picker_open,
        };
        render_settings(frame, &theme, &settings_state);
        render_search_helpers_with_data(
            frame,
            input,
            toast_message,
            query_results,
            focus_mode,
            &theme,
            None,
        );
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }

    // render help/query guide as a full-screen overlay (no query box),
    if *focus_mode == FocusMode::QueryGuide {
        let (_total_lines, max_scroll) = render_query_guide(frame, &theme, guide_scroll);
        // store max_scroll for clamping in keyboard handlers
        *guide_max_scroll = max_scroll;
        render_search_helpers_with_data(
            frame,
            input,
            toast_message,
            query_results,
            focus_mode,
            &theme,
            None,
        );
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }

    // render save name input popup
    if *focus_mode == FocusMode::SaveNameInput {
        render_save_name_popup(frame, save_name_input, save_name_cursor_visible, &theme);
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }

    // render my schedules view
    if *focus_mode == FocusMode::MySchedules {
        render_my_schedules(frame, saved_schedules, selected_saved_schedule_index, &theme);
        render_toast_with_data(frame, toast_message, error_type, &theme);
        return (0, 0);
    }

    // render schedule creation if in schedule creation mode OR detail view from schedule
    let should_render_schedule = *focus_mode == FocusMode::ScheduleCreation 
        || (*focus_mode == FocusMode::DetailView && (detail_return_focus == FocusMode::ScheduleCreation || detail_return_focus == FocusMode::MySchedules));
    
    if should_render_schedule {
        // determine if viewing from saved schedules and get saved schedule info
        let (saved_schedule_idx, total_saved) = if detail_return_focus == FocusMode::MySchedules {
            (Some(selected_saved_schedule_index), Some(saved_schedules.len()))
        } else {
            (None, None)
        };
        
        crate::tui::widgets::schedule::render_schedule_creation(
            frame,
            cart_classes,
            selected_for_schedule,
            generated_schedules,
            current_schedule_index,
            schedule_cart_focus,
            selected_cart_index,
            schedule_selection_mode,
            selected_time_block_day,
            selected_time_block_slot,
            current_saved_schedule_name,
            saved_schedule_idx,
            total_saved,
            &theme,
        );
        render_search_helpers_with_data(
            frame,
            input,
            toast_message,
            query_results,
            focus_mode,
            &theme,
            Some(schedule_selection_mode),
        );
        render_toast_with_data(frame, toast_message, error_type, &theme);
        
        // if in detail view, render it as overlay (don't return early)
        if *focus_mode == FocusMode::DetailView {
            // detail view will be rendered below
        } else {
            return (0, 0);
        }
    }

    // render query interface (skip if detail view from schedule or saved schedule)
    let skip_query_interface = *focus_mode == FocusMode::DetailView 
        && (detail_return_focus == FocusMode::ScheduleCreation || detail_return_focus == FocusMode::MySchedules);
    
    let max_items_that_fit = if !skip_query_interface {
        render_search_bar_with_data(
            frame,
            input,
            problematic_positions,
            focus_mode,
            cursor_visible,
            &theme,
        );
        let (_items_rendered, max_items) = render_query_results(
            frame,
            query_results,
            results_scroll,
            focus_mode,
            selected_result,
            &theme,
        );
        
        // Show "No results" message if a query was executed but returned no results
        if query_results.is_empty() && !user_query.is_empty() 
            && (*focus_mode == FocusMode::QueryInput || *focus_mode == FocusMode::ResultsBrowse) {
            let logo_height = 7;
            let search_y = logo_height + 6;
            let search_height = 3;
            let results_y = search_y + search_height + 1;
            
            let no_results_msg = "No results";
            let msg_width = no_results_msg.len() as u16;
            let msg_x = (frame.area().width.saturating_sub(msg_width)) / 2;
            let msg_area = Rect {
                x: msg_x,
                y: results_y + 2,
                width: msg_width,
                height: 1,
            };
            
            let no_results_paragraph = ratatui::widgets::Paragraph::new(no_results_msg)
                .style(ratatui::style::Style::default().fg(theme.error_color));
            frame.render_widget(no_results_paragraph, msg_area);
        }
        
        render_search_helpers_with_data(
            frame,
            input,
            toast_message,
            query_results,
            focus_mode,
            &theme,
            None,
        );
        render_toast_with_data(frame, toast_message, error_type, &theme);
        render_completion_dropdown(
            frame,
            completions,
            completion_index,
            show_completions,
            &theme,
        );
        max_items
    } else {
        // still render toast and helpers even when skipping query interface
        render_search_helpers_with_data(
            frame,
            input,
            toast_message,
            query_results,
            focus_mode,
            &theme,
            None,
        );
        render_toast_with_data(frame, toast_message, error_type, &theme);
        0 // no items rendered
    };

    // render detail view overlay if in detail mode
    if *focus_mode == FocusMode::DetailView {
        let class_option = if detail_return_focus == FocusMode::ScheduleCreation 
            || detail_return_focus == FocusMode::MySchedules 
        {
            // coming from schedule viewing mode (saved or generated)
            // use selected_class_for_details if available, otherwise find from time block
            if let Some(class) = selected_class_for_details {
                Some((class, false))
            } else if !generated_schedules.is_empty() 
                && current_schedule_index < generated_schedules.len() 
            {
                let schedule = &generated_schedules[current_schedule_index];
                find_class_at_time_block(schedule, selected_time_block_day, selected_time_block_slot)
                    .map(|class| (class, false))
            } else {
                None
            }
        } else if selected_result < query_results.len() {
            // coming from results browse mode
            Some((&query_results[selected_result], true))
        } else {
            None
        };

        if let Some((class, show_cart_option)) = class_option {
            let class_id = class.unique_id();
            let is_in_cart = cart_classes.contains_key(&class_id);
            render_detail_view(frame, class, &theme, is_in_cart, show_cart_option);
        }
    }

    (0, max_items_that_fit)
}

/// Render the save name input popup
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// input -> The current input text
/// cursor_visible -> Whether the cursor should be visible
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_save_name_popup(
    frame: &mut Frame,
    input: &str,
    cursor_visible: bool,
    theme: &crate::tui::themes::Theme,
) {
    let popup_width = 50;
    let popup_height = 5;
    
    // position at same height as main menu (below logo)
    let logo_height = 7_u16;
    let spacing = 6_u16;
    let popup_y = logo_height + spacing;
    
    let frame_width = frame.area().width;
    let frame_height = frame.area().height;
    
    let area = ratatui::layout::Rect {
        x: (frame_width.saturating_sub(popup_width.min(frame_width))) / 2,
        y: popup_y.min(frame_height.saturating_sub(popup_height.min(frame_height))),
        width: popup_width.min(frame_width),
        height: popup_height.min(frame_height),
    }.intersection(frame.area());

    // show placeholder text when input is empty, or add cursor
    let (display_text, text_style) = if input.is_empty() {
        let placeholder_style = ratatui::style::Style::default().fg(theme.muted_color);
        if cursor_visible {
            ("enter schedule name|".to_string(), placeholder_style)
        } else {
            ("enter schedule name ".to_string(), placeholder_style)
        }
    } else {
        let base_style = ratatui::style::Style::default().fg(theme.text_color);
        if cursor_visible {
            (format!("{input}|"), base_style)
        } else {
            (format!("{input} "), base_style)
        }
    };

    let block = ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title(" Save Schedule ")
        .title_style(
            ratatui::style::Style::default()
                .fg(theme.title_color)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .border_style(ratatui::style::Style::default().fg(theme.border_color));

    // center the text within the paragraph
    let paragraph = ratatui::widgets::Paragraph::new(display_text)
        .block(block)
        .style(text_style)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render the My Schedules view
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// saved_schedules -> List of saved schedules
/// selected_index -> The index of the currently selected schedule
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_my_schedules(
    frame: &mut Frame,
    saved_schedules: &[SavedSchedule],
    selected_index: usize,
    theme: &crate::tui::themes::Theme,
) {
    // position properly
    let logo_height = 7_u16;
    let spacing = 6_u16;
    let menu_y = logo_height + spacing;
    
    let menu_width = 50_u16;
    let menu_height = (saved_schedules.len() as u16 + 4).min(20); // schedules + borders + title
    
    let frame_width = frame.area().width;
    let frame_height = frame.area().height;
    
    // clamp menu dimensions to fit within frame
    let area = ratatui::layout::Rect {
        x: (frame_width.saturating_sub(menu_width.min(frame_width))) / 2,
        y: menu_y.min(frame_height.saturating_sub(menu_height.min(frame_height))),
        width: menu_width.min(frame_width),
        height: menu_height.min(frame_height),
    }.intersection(frame.area());
    
    // create list of schedule entries
    let mut lines = Vec::new();
    for (i, schedule) in saved_schedules.iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        
        let style = if is_selected {
            ratatui::style::Style::default()
                .fg(theme.selected_color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            ratatui::style::Style::default().fg(theme.text_color)
        };
        
        let line = format!("{}{}", prefix, schedule.name);
        lines.push(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(line, style),
        ]));
    }
    
    if lines.is_empty() {
        lines.push(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                "No saved schedules. Press 's' in schedule view to save one.",
                ratatui::style::Style::default().fg(theme.text_color),
            ),
        ]));
    }
    
    let block = ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title(" My Schedules ")
        .title_style(
            ratatui::style::Style::default()
                .fg(theme.title_color)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .border_style(ratatui::style::Style::default().fg(theme.border_color));
    
    let list = ratatui::widgets::Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    frame.render_widget(list, area);
}
