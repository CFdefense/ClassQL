/// src/tui/widgets/search.rs
///
/// Search widget with encapsulated state, input handling, and rendering
///
/// Handles query input, tab completion, results browsing, and search bar rendering
///
/// Contains:
/// --- ---
/// SearchWidget -> Widget for search functionality
/// CompletionState -> State for tab completion dropdown
/// --- ---
use crate::data::sql::Class;
use crate::dsl::compiler::{Compiler, CompilerResult};
use crate::tui::state::{ErrorType, FocusMode};
use crate::tui::themes::Theme;
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::cell::Cell;
use std::time::Instant;

/// State for tab completion dropdown
///
/// Tracks the completion suggestions, current selection, and visibility
/// of the tab completion dropdown in the search interface.
///
/// Fields:
/// --- ---
/// completions -> List of completion suggestions
/// completion_index -> Currently selected completion index
/// show_completions -> Whether completion dropdown is visible
/// partial_word -> The partial word being completed
/// --- ---
///
#[derive(Debug, Clone)]
pub struct CompletionState {
    pub completions: Vec<String>,
    pub completion_index: Option<usize>,
    pub show_completions: bool,
    pub partial_word: String,
}

impl CompletionState {
    pub fn new() -> Self {
        Self {
            completions: Vec::new(),
            completion_index: None,
            show_completions: false,
            partial_word: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.completions.clear();
        self.completion_index = None;
        self.show_completions = false;
        self.partial_word.clear();
    }
}

/// Search widget with encapsulated state
///
/// Manages the query input interface including text entry, cursor blinking,
/// tab completion dropdown, query execution, and results browsing with
/// keyboard navigation and add-to-cart functionality.
///
/// Fields:
/// --- ---
/// input -> The current user input string
/// user_query -> The last executed query string
/// problematic_positions -> Byte ranges of problematic tokens for highlighting
/// completion -> Tab completion state (suggestions, selection, visibility)
/// query_results -> The list of Class results from the last query
/// results_scroll -> Scroll offset for results display
/// selected_result -> Index of currently selected result
/// cursor_visible -> Whether the input cursor is visible (for blinking)
/// last_cursor_blink -> Timestamp of last cursor blink toggle
/// max_items_that_fit -> Maximum number of items that fit on screen
/// focus -> Current focus mode (QueryInput or ResultsBrowse)
/// is_searching -> Whether a query is currently being executed
/// --- ---
///
pub struct SearchWidget {
    pub input: String,
    pub user_query: String,
    pub problematic_positions: Vec<(usize, usize)>,
    pub completion: CompletionState,
    pub query_results: Vec<Class>,
    pub results_scroll: usize,
    pub selected_result: usize,
    pub cursor_visible: bool,
    pub last_cursor_blink: Instant,
    pub max_items_that_fit: Cell<usize>,
    /// Internal focus: QueryInput or ResultsBrowse
    focus: SearchFocus,
    /// Whether a search is currently in progress
    pub is_searching: bool,
}

/// Internal focus state for SearchWidget
#[derive(Debug, Clone, PartialEq)]
pub enum SearchFocus {
    QueryInput,
    ResultsBrowse,
}

impl SearchWidget {
    /// Create a new SearchWidget
    pub fn new() -> Self {
        Self {
            input: String::new(),
            user_query: String::new(),
            problematic_positions: Vec::new(),
            completion: CompletionState::new(),
            query_results: Vec::new(),
            results_scroll: 0,
            selected_result: 0,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            max_items_that_fit: Cell::new(0),
            focus: SearchFocus::QueryInput,
            is_searching: false,
        }
    }

    /// Update cursor blink state
    pub fn update_cursor_blink(&mut self) {
        if self.last_cursor_blink.elapsed() > std::time::Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_blink = Instant::now();
        }
    }

    /// Get the current focus mode
    pub fn current_focus_mode(&self) -> FocusMode {
        match self.focus {
            SearchFocus::QueryInput => FocusMode::QueryInput,
            SearchFocus::ResultsBrowse => FocusMode::ResultsBrowse,
        }
    }

    /// Set focus mode from external FocusMode
    pub fn set_focus(&mut self, mode: FocusMode) {
        match mode {
            FocusMode::QueryInput => self.focus = SearchFocus::QueryInput,
            FocusMode::ResultsBrowse => self.focus = SearchFocus::ResultsBrowse,
            _ => {} // Ignore other modes
        }
    }

    /// Check if in query input focus
    pub fn is_query_input(&self) -> bool {
        self.focus == SearchFocus::QueryInput
    }

    /// Check if in results browse focus
    pub fn is_results_browse(&self) -> bool {
        self.focus == SearchFocus::ResultsBrowse
    }

    /// Clear error state
    pub fn clear_error_state(&mut self) {
        self.problematic_positions.clear();
    }

    /// Render the "Searching..." indicator in the results area
    pub fn render_searching_indicator(frame: &mut Frame, theme: &Theme) {
        use ratatui::layout::{Alignment, Rect};
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::Paragraph;

        // position in the results area (below search bar)
        let logo_height = 7_u16;
        let search_y = logo_height + 6;
        let search_height = 3_u16;
        let results_y = search_y + search_height + 3;

        let text = "Searching...";
        let msg_width = text.len() as u16;
        let msg_x = (frame.area().width.saturating_sub(msg_width)) / 2;
        let msg_area = Rect {
            x: msg_x,
            y: results_y,
            width: msg_width,
            height: 1,
        };

        let para = Paragraph::new(text).alignment(Alignment::Center).style(
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(para, msg_area);
    }

    /// Execute a query using the compiler
    pub fn execute_query(&mut self, compiler: &mut Compiler) -> Option<KeyAction> {
        self.user_query = self.input.clone();

        match compiler.run(&self.input) {
            CompilerResult::Success { classes, .. } => {
                self.problematic_positions.clear();
                self.query_results = classes;
                self.results_scroll = 0;
                self.selected_result = 0;
                None
            }
            CompilerResult::LexerError {
                message,
                problematic_positions,
            } => {
                self.problematic_positions = problematic_positions;
                Some(KeyAction::ShowToast {
                    message,
                    error_type: ErrorType::Lexer,
                })
            }
            CompilerResult::ParserError {
                message,
                problematic_positions,
            } => {
                self.problematic_positions = problematic_positions;
                Some(KeyAction::ShowToast {
                    message,
                    error_type: ErrorType::Parser,
                })
            }
            CompilerResult::SemanticError {
                message,
                problematic_positions,
            } => {
                self.problematic_positions = problematic_positions;
                Some(KeyAction::ShowToast {
                    message,
                    error_type: ErrorType::Semantic,
                })
            }
            CompilerResult::CodeGenError { message } => {
                self.problematic_positions.clear();
                Some(KeyAction::ShowToast {
                    message,
                    error_type: ErrorType::Semantic,
                })
            }
        }
    }

    /// Handle tab completion
    ///
    /// Returns a toast message if no completions are available
    pub fn handle_tab_completion(&mut self, compiler: &mut Compiler) -> Option<String> {
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
        let suggestions = compiler.get_tab_completion(self.input.clone());

        // if there's a potential partial word, check if any suggestions match it
        if !potential_partial.is_empty() {
            let matching: Vec<String> = suggestions
                .iter()
                .filter(|s| s.to_lowercase().starts_with(&potential_partial))
                .cloned()
                .collect();

            if !matching.is_empty() {
                // partial word matches some suggestions - filter to those
                self.completion.partial_word = potential_partial;
                self.completion.completions = matching;
            } else {
                // no matches - the "partial" is actually a complete value
                self.completion.partial_word = String::new();
                self.completion.completions = suggestions;
            }
        } else {
            self.completion.partial_word = String::new();
            self.completion.completions = suggestions;
        }

        if !self.completion.completions.is_empty() {
            self.completion.show_completions = true;
            self.completion.completion_index = Some(0);
            None
        } else if !self.input.trim().is_empty() {
            // no completions available - return helpful hint
            let hint = self.get_completion_hint();
            if !hint.is_empty() {
                Some(hint)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get helpful hint when no completions available
    pub fn get_completion_hint(&self) -> String {
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

    /// Apply selected completion to input
    pub fn apply_completion(&mut self) {
        if let Some(index) = self.completion.completion_index {
            if index < self.completion.completions.len() {
                let completion = &self.completion.completions[index].clone();
                // don't add placeholders like <value>
                if !completion.starts_with('<') {
                    // only replace if there's a partial word that matches
                    if !self.completion.partial_word.is_empty()
                        && completion
                            .to_lowercase()
                            .starts_with(&self.completion.partial_word)
                    {
                        // remove the partial word from input
                        let trim_len = self.completion.partial_word.len();
                        let new_len = self.input.len().saturating_sub(trim_len);
                        self.input.truncate(new_len);
                    } else {
                        // no partial word - just append with space
                        if !self.input.is_empty() && !self.input.ends_with(' ') {
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
        self.completion.clear();
    }

    /// Handle completion navigation
    fn handle_completion_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Esc => {
                self.completion.clear();
                KeyAction::Continue
            }
            KeyCode::Up => {
                if let Some(index) = self.completion.completion_index {
                    if index > 0 {
                        self.completion.completion_index = Some(index - 1);
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                if let Some(index) = self.completion.completion_index {
                    let len = self.completion.completions.len();
                    if index < len - 1 {
                        self.completion.completion_index = Some(index + 1);
                    } else {
                        self.completion.completion_index = Some(0);
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                self.apply_completion();
                KeyAction::Continue
            }
            KeyCode::Tab => {
                // tab moves down through completions
                if let Some(index) = self.completion.completion_index {
                    let len = self.completion.completions.len();
                    if index < len - 1 {
                        self.completion.completion_index = Some(index + 1);
                    } else {
                        self.completion.completion_index = Some(0);
                    }
                }
                KeyAction::Continue
            }
            _ => {
                // any other key hides completions
                self.completion.clear();
                KeyAction::Continue
            }
        }
    }

    /// Handle results browse navigation
    fn handle_results_browse_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Exit,
            KeyCode::Esc => KeyAction::Navigate(FocusMode::MainMenu),
            KeyCode::Char('g') | KeyCode::Char('G')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                KeyAction::Navigate(FocusMode::QueryGuide)
            }
            KeyCode::Up => {
                if self.selected_result == 0 {
                    self.focus = SearchFocus::QueryInput;
                } else {
                    let cols = 3;
                    if self.selected_result >= cols {
                        self.selected_result -= cols;
                        if self.selected_result < self.results_scroll {
                            let target_row = self.selected_result / cols;
                            self.results_scroll = target_row * cols;
                        }
                    } else {
                        self.focus = SearchFocus::QueryInput;
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                let cols = 3;
                if self.selected_result + cols < self.query_results.len() {
                    self.selected_result += cols;
                    let total_results = self.query_results.len();
                    let max_visible = self.max_items_that_fit.get();
                    if total_results <= max_visible || max_visible == 0 {
                        self.results_scroll = 0;
                    } else if self.selected_result >= self.results_scroll + max_visible {
                        let rows_visible = max_visible / cols;
                        let current_row = self.selected_result / cols;
                        let scroll_row = current_row.saturating_sub(rows_visible.saturating_sub(1));
                        self.results_scroll = scroll_row * cols;
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Left => {
                if self.selected_result > 0 {
                    self.selected_result -= 1;
                    if self.selected_result < self.results_scroll {
                        self.results_scroll = self.selected_result;
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Right => {
                if self.selected_result + 1 < self.query_results.len() {
                    self.selected_result += 1;
                    let total_results = self.query_results.len();
                    let max_visible = self.max_items_that_fit.get();
                    if total_results <= max_visible || max_visible == 0 {
                        self.results_scroll = 0;
                    } else if self.selected_result >= self.results_scroll + max_visible {
                        self.results_scroll = self
                            .selected_result
                            .saturating_sub(max_visible.saturating_sub(1));
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                if self.selected_result < self.query_results.len() {
                    KeyAction::Navigate(FocusMode::DetailView)
                } else {
                    KeyAction::Continue
                }
            }
            KeyCode::Char(c) => {
                // typing goes back to query input
                self.focus = SearchFocus::QueryInput;
                self.clear_error_state();
                self.input.push(c);
                KeyAction::Continue
            }
            KeyCode::Backspace => {
                self.focus = SearchFocus::QueryInput;
                self.clear_error_state();
                self.input.pop();
                KeyAction::Continue
            }
            KeyCode::Tab => {
                self.focus = SearchFocus::QueryInput;
                // caller should handle tab completion
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    /// Handle query input key events
    fn handle_query_input_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Exit,
            KeyCode::Esc => KeyAction::Navigate(FocusMode::MainMenu),
            KeyCode::Char('g') | KeyCode::Char('G')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                KeyAction::Navigate(FocusMode::QueryGuide)
            }
            KeyCode::Down => {
                if !self.query_results.is_empty() {
                    self.focus = SearchFocus::ResultsBrowse;
                    self.selected_result = 0;
                    if self.results_scroll > 0 {
                        self.results_scroll = 0;
                    }
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                // signal that we need to execute the query
                // the caller will handle this by calling execute_query
                KeyAction::Continue
            }
            KeyCode::Backspace => {
                self.clear_error_state();
                self.input.pop();
                KeyAction::Continue
            }
            KeyCode::Tab => {
                // signal that we need tab completion
                // the caller will handle this
                KeyAction::Continue
            }
            KeyCode::Char(c) => {
                self.clear_error_state();
                self.input.push(c);
                KeyAction::Continue
            }
            KeyCode::PageUp => {
                if self.results_scroll >= 3 {
                    self.results_scroll -= 3;
                } else {
                    self.results_scroll = 0;
                }
                KeyAction::Continue
            }
            KeyCode::PageDown => {
                let max_scroll = self.query_results.len().saturating_sub(3);
                if self.results_scroll + 3 < max_scroll {
                    self.results_scroll += 3;
                } else {
                    self.results_scroll = max_scroll;
                }
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    /// Get the currently selected class (for detail view)
    pub fn selected_class(&self) -> Option<&Class> {
        self.query_results.get(self.selected_result)
    }

    /// Render the search bar with syntax highlighting
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_search_bar(&self, frame: &mut Frame, theme: &Theme) {
        let search_width = 50;
        let is_focused = self.focus == SearchFocus::QueryInput;

        // position search bar below the logo with same vertical gap as menus
        let logo_height = 7;
        let search_y = logo_height + 6;

        let frame_width = frame.area().width;
        let frame_height = frame.area().height;
        let search_height = 3_u16;

        // clamp search bar dimensions to fit within frame
        let search_area = Rect {
            x: (frame_width.saturating_sub(search_width.min(frame_width) as u16)) / 2,
            y: search_y.min(frame_height.saturating_sub(search_height)),
            width: (search_width as u16).min(frame_width),
            height: search_height.min(frame_height),
        }
        .intersection(frame.area());

        // calculate visible width (minus borders and "> " prefix and cursor)
        let visible_width = search_width.saturating_sub(5) as usize;
        let input_len = self.input.chars().count();

        // calculate scroll offset to keep cursor (end of input) visible
        let scroll_offset = if input_len > visible_width {
            input_len - visible_width
        } else {
            0
        };

        // create styled spans for the input with highlighted problematic positions
        let mut styled_spans = Vec::new();

        // start with the "> " prefix (or "…" if scrolled)
        if scroll_offset > 0 {
            styled_spans.push(Span::styled("…", Style::default().fg(theme.muted_color)));
        } else {
            styled_spans.push(Span::styled(
                "> ",
                Style::default().fg(if is_focused {
                    theme.selected_color
                } else {
                    theme.muted_color
                }),
            ));
        }

        // process only the visible portion of the input
        for (i, ch) in self.input.chars().enumerate().skip(scroll_offset) {
            if i - scroll_offset >= visible_width {
                break;
            }

            let is_problematic = self
                .problematic_positions
                .iter()
                .any(|&(start, end)| i >= start && i < end);

            let style = if is_problematic {
                Style::default().fg(theme.error_color)
            } else {
                Style::default().fg(theme.text_color)
            };

            styled_spans.push(Span::styled(ch.to_string(), style));
        }

        // add flashing cursor if focused
        if is_focused && self.cursor_visible {
            styled_spans.push(Span::styled("|", Style::default().fg(theme.selected_color)));
        } else if is_focused {
            styled_spans.push(Span::styled(" ", Style::default()));
        }

        let styled_line = Line::from(styled_spans);

        // border color depends on focus state
        let border_color = if is_focused {
            theme.border_color
        } else {
            theme.muted_color
        };

        let title_style = if is_focused {
            Style::default()
                .fg(theme.title_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted_color)
        };

        let search_paragraph = Paragraph::new(styled_line)
            .style(Style::default().fg(theme.text_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("ClassQL Query")
                    .title_style(title_style)
                    .border_style(Style::default().fg(border_color)),
            );

        frame.render_widget(search_paragraph, search_area);
    }

    /// Render the query results in a 3-column grid
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// usize -> maximum number of items that fit on screen
    /// --- ---
    ///
    fn render_query_results(&self, frame: &mut Frame, theme: &Theme) -> usize {
        if self.query_results.is_empty() {
            return 0;
        }

        let is_browse_mode = self.focus == SearchFocus::ResultsBrowse;

        // position the results grid below the search bar
        let logo_height = 7;
        let search_y = logo_height + 6;
        let search_height = 3;
        let results_y = search_y + search_height + 1;

        // calculate available space for results
        let available_height = frame.area().height.saturating_sub(results_y + 10);
        let cell_height = 7_u16;
        let rows_to_show = (available_height / cell_height).max(1) as usize;

        // calculate grid dimensions
        let cell_width = 26_u16;
        let cols = 3_usize;
        let grid_width = cell_width * cols as u16 + (cols as u16 - 1) * 2;
        let grid_x = frame.area().width.saturating_sub(grid_width) / 2;

        // calculate how many items can actually fit
        let max_items_that_fit = rows_to_show * cols;

        // apply scroll offset and get visible classes
        let visible_classes: Vec<(usize, &Class)> = self
            .query_results
            .iter()
            .enumerate()
            .skip(self.results_scroll)
            .take(max_items_that_fit)
            .collect();

        // update max_items_that_fit (we'll need to store this, but for now just render)
        // render each class in a 3-column grid
        for (global_idx, class) in visible_classes.iter() {
            let idx = global_idx - self.results_scroll;
            let row = idx / cols;
            let col = idx % cols;

            let cell_x = grid_x + (col as u16 * (cell_width + 2));
            let cell_y = results_y + (row as u16 * cell_height);

            let is_selected = is_browse_mode && *global_idx == self.selected_result;

            // create the class card
            let display_lines = class.format_for_display();

            // build styled lines for the card
            let mut styled_lines: Vec<Line> = Vec::new();

            // line 1: course code (bold title color)
            if let Some(line) = display_lines.first() {
                let style = Style::default()
                    .fg(theme.title_color)
                    .add_modifier(Modifier::BOLD);
                styled_lines.push(Line::from(Span::styled(line.clone(), style)));
            }

            // line 2: title (text color)
            if let Some(line) = display_lines.get(1) {
                let style = Style::default().fg(theme.text_color);
                styled_lines.push(Line::from(Span::styled(line.clone(), style)));
            }

            // line 3: professor (warning color)
            if let Some(line) = display_lines.get(2) {
                let style = Style::default().fg(theme.warning_color);
                styled_lines.push(Line::from(Span::styled(line.clone(), style)));
            }

            // line 4: days/time (success color)
            if let Some(line) = display_lines.get(3) {
                let style = Style::default().fg(theme.success_color);
                styled_lines.push(Line::from(Span::styled(line.clone(), style)));
            }

            // line 5: enrollment (muted color)
            if let Some(line) = display_lines.get(4) {
                let style = Style::default().fg(theme.muted_color);
                styled_lines.push(Line::from(Span::styled(line.clone(), style)));
            }

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: cell_width,
                height: cell_height,
            }
            .intersection(frame.area());

            // border color depends on selection state
            let border_color = if is_selected {
                theme.selected_color
            } else {
                theme.muted_color
            };

            let card = Paragraph::new(styled_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );

            frame.render_widget(card, cell_area);
        }

        max_items_that_fit
    }

    /// Render the completion dropdown
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_completion_dropdown(&self, frame: &mut Frame, theme: &Theme) {
        if !self.completion.show_completions {
            return;
        }

        let dropdown_width = 50;

        // position below the search bar
        let logo_height = 7;
        let search_y = logo_height + 6;
        let search_height = 3;
        let dropdown_y = search_y + search_height + 1;

        // calculate max available height (leave some space at bottom)
        let max_available_height = frame.area().height.saturating_sub(dropdown_y + 2);

        // height = number of completions + 2 for borders, capped by available space
        let dropdown_height =
            (self.completion.completions.len() as u16 + 2).min(max_available_height);

        let dropdown_area = Rect {
            x: frame.area().width.saturating_sub(dropdown_width) / 2,
            y: dropdown_y,
            width: dropdown_width,
            height: dropdown_height,
        }
        .intersection(frame.area());

        let mut styled_lines = Vec::new();
        for (i, completion) in self.completion.completions.iter().enumerate() {
            let style = if Some(i) == self.completion.completion_index {
                Style::default()
                    .fg(theme.background_color)
                    .bg(theme.selected_color)
            } else {
                Style::default()
                    .fg(theme.text_color)
                    .bg(theme.background_color)
            };
            styled_lines.push(Line::from(Span::styled(completion.clone(), style)));
        }

        // first, clear the area to cover results below with solid background
        frame.render_widget(Clear, dropdown_area);

        let dropdown_paragraph = Paragraph::new(styled_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Suggestions (↑↓ to navigate, Enter to select)")
                .title_style(Style::default().fg(theme.warning_color))
                .border_style(Style::default().fg(theme.warning_color))
                .style(Style::default().bg(theme.background_color)),
        );

        frame.render_widget(dropdown_paragraph, dropdown_area);

        // force background color on empty/border cells
        let buffer = frame.buffer_mut();
        let buffer_width = buffer.area.width;
        let buffer_height = buffer.area.height;

        let start_y = dropdown_area.top();
        let start_x = dropdown_area.left();
        let end_y = dropdown_area.bottom().min(buffer_height);
        let end_x = dropdown_area.right().min(buffer_width);

        if start_y < buffer_height && start_x < buffer_width && end_y > start_y && end_x > start_x {
            for y in start_y..end_y {
                for x in start_x..end_x.min(buffer_width) {
                    if x < buffer_width && y < buffer_height {
                        let cell = &mut buffer[(x, y)];

                        if cell.symbol() == " "
                            || cell.symbol() == "│"
                            || cell.symbol() == "─"
                            || cell.symbol() == "┌"
                            || cell.symbol() == "┐"
                            || cell.symbol() == "└"
                            || cell.symbol() == "┘"
                            || cell.symbol() == "├"
                            || cell.symbol() == "┤"
                            || cell.symbol() == "┬"
                            || cell.symbol() == "┴"
                        {
                            cell.set_bg(theme.background_color);
                        }
                    }
                }
            }
        }
    }
}

impl Widget for SearchWidget {
    fn render(&self, frame: &mut Frame, theme: &Theme) {
        // render search bar
        self.render_search_bar(frame, theme);

        // show "Searching..." indicator OR results
        if self.is_searching {
            Self::render_searching_indicator(frame, theme);
        } else {
            // render results and update max_items_that_fit
            let max_items = self.render_query_results(frame, theme);
            self.max_items_that_fit.set(max_items);
        }

        // render completion dropdown if visible
        if self.completion.show_completions {
            self.render_completion_dropdown(frame, theme);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyAction {
        // handle completion dropdown first if visible
        if self.completion.show_completions {
            return self.handle_completion_key(key);
        }

        // handle based on current focus
        match self.focus {
            SearchFocus::QueryInput => self.handle_query_input_key(key),
            SearchFocus::ResultsBrowse => self.handle_results_browse_key(key),
        }
    }

    fn focus_modes(&self) -> Vec<FocusMode> {
        vec![FocusMode::QueryInput, FocusMode::ResultsBrowse]
    }
}
