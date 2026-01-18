/// src/tui/widgets/guide.rs
///
/// Query guide widget with encapsulated state, input handling, and rendering
///
/// Displays a scrollable guide showing the ClassQL grammar and query examples
///
/// Contains:
/// --- ---
/// QueryGuideWidget -> Widget for the scrollable query guide
/// --- ---
use crate::tui::state::FocusMode;
use crate::tui::themes::Theme;
use crate::tui::widgets::traits::{KeyAction, Widget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Query guide widget with encapsulated scroll state
///
/// Displays the scrollable query guide/help documentation with keyboard
/// navigation for scrolling and returning to the previous view.
///
/// Fields:
/// --- ---
/// scroll -> Current scroll position in the guide content
/// max_scroll -> Maximum scroll value (computed during render)
/// return_focus -> Focus mode to return to when closing the guide
/// --- ---
///
pub struct QueryGuideWidget {
    pub scroll: usize,
    pub max_scroll: usize,
    pub return_focus: FocusMode,
}

impl QueryGuideWidget {
    /// Create a new QueryGuideWidget
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Self -> new QueryGuideWidget with default state
    /// --- ---
    ///
    pub fn new() -> Self {
        Self {
            scroll: 0,
            max_scroll: 0,
            return_focus: FocusMode::MainMenu,
        }
    }

    /// Open the guide with a specific return focus mode
    ///
    /// Arguments:
    /// --- ---
    /// return_focus -> the focus mode to return to when closing
    /// --- ---
    ///
    /// Returns: None
    ///
    pub fn open(&mut self, return_focus: FocusMode) {
        self.scroll = 0;
        self.return_focus = return_focus;
    }

    /// Reset scroll position
    ///
    /// Arguments: None
    ///
    /// Returns: None
    ///
    pub fn reset(&mut self) {
        self.scroll = 0;
    }

    /// Render the query guide as an overlay with scrolling
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the theme to use for styling
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// (usize, usize) -> (total number of lines, max_scroll value)
    /// --- ---
    ///
    pub fn render_guide(&self, frame: &mut Frame, theme: &Theme) -> (usize, usize) {
        let frame_area = frame.area();
        let guide_width = 74_u16.min(frame_area.width.saturating_sub(4));
        let guide_height = 40_u16.min(frame_area.height.saturating_sub(4));

        let guide_area = Rect {
            x: (frame_area.width.saturating_sub(guide_width)) / 2,
            y: (frame_area.height.saturating_sub(guide_height)) / 2,
            width: guide_width,
            height: guide_height,
        }
        .intersection(frame_area);

        let lines = self.build_guide_lines(theme);
        let total_lines = lines.len();
        let content_height = (guide_height.saturating_sub(2)) as usize;

        let max_scroll = if total_lines > content_height {
            total_lines.saturating_sub(content_height)
        } else {
            0
        };
        let clamped_scroll = if max_scroll > 0 {
            self.scroll.min(max_scroll)
        } else {
            0
        };

        let start_line = clamped_scroll;
        let end_line = (start_line + content_height).min(total_lines);
        let visible_lines: Vec<Line> = lines[start_line..end_line].to_vec();

        frame.render_widget(Clear, guide_area);

        let guide_paragraph = Paragraph::new(visible_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Query Guide ")
                    .title_style(
                        Style::default()
                            .fg(theme.title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(theme.border_color))
                    .style(Style::default().bg(theme.background_color)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(guide_paragraph, guide_area);

        // render scrollbar if content exceeds visible area
        if total_lines > content_height {
            self.render_scrollbar(
                frame,
                guide_area,
                total_lines,
                content_height,
                clamped_scroll,
                max_scroll,
                theme,
            );
        }

        // force background color on empty/border cells
        self.fill_background(frame, guide_area, theme);

        (total_lines, max_scroll)
    }

    /// Render the scrollbar for the guide
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// guide_area -> the area of the guide
    /// total_lines -> total number of content lines
    /// content_height -> visible content height
    /// clamped_scroll -> current scroll position (clamped)
    /// max_scroll -> maximum scroll value
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render_scrollbar(
        &self,
        frame: &mut Frame,
        guide_area: Rect,
        total_lines: usize,
        content_height: usize,
        clamped_scroll: usize,
        max_scroll: usize,
        theme: &Theme,
    ) {
        let buffer = frame.buffer_mut();
        let buffer_height = buffer.area.height;
        let buffer_width = buffer.area.width;

        let scrollbar_x = guide_area.right().saturating_sub(1);
        let scrollbar_y_start = guide_area.top() + 1;
        let scrollbar_height = guide_area.height.saturating_sub(2);

        if scrollbar_x >= buffer_width {
            return;
        }

        let max_scrollable = total_lines.saturating_sub(content_height);
        let track_height = scrollbar_height.saturating_sub(2);

        let thumb_size =
            ((content_height as f64 / total_lines as f64) * track_height as f64).ceil() as u16;
        let thumb_size = thumb_size.max(1).min(track_height);

        let thumb_position = if max_scrollable > 0 {
            let ratio = clamped_scroll as f64 / max_scrollable as f64;
            (ratio * (track_height.saturating_sub(thumb_size)) as f64).round() as u16
        } else {
            0
        };

        let thumb_position = if clamped_scroll >= max_scroll && max_scroll > 0 {
            track_height.saturating_sub(thumb_size)
        } else {
            thumb_position
        };

        // render begin symbol
        if scrollbar_y_start < buffer_height && scrollbar_x < buffer_width {
            let cell = &mut buffer[(scrollbar_x, scrollbar_y_start)];
            cell.set_symbol("↑");
            cell.set_style(Style::default().fg(theme.border_color));
        }

        // render track and thumb
        let track_start_y = scrollbar_y_start + 1;
        let track_end_y = (scrollbar_y_start + 1 + track_height).min(buffer_height);

        for y in track_start_y..track_end_y {
            if y < buffer_height && scrollbar_x < buffer_width {
                let cell = &mut buffer[(scrollbar_x, y)];
                let track_y = (y - track_start_y) as u16;

                if track_y >= thumb_position && track_y < thumb_position + thumb_size {
                    cell.set_symbol("█");
                    cell.set_style(Style::default().fg(theme.selected_color));
                } else {
                    cell.set_symbol("│");
                    cell.set_style(Style::default().fg(theme.border_color));
                }
            }
        }

        // render end symbol
        let end_y = scrollbar_y_start + 1 + track_height;
        if end_y < buffer_height && scrollbar_x < buffer_width {
            let cell = &mut buffer[(scrollbar_x, end_y)];
            cell.set_symbol("↓");
            cell.set_style(Style::default().fg(theme.border_color));
        }
    }

    /// Fill background color on empty/border cells
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to modify
    /// guide_area -> the area of the guide
    /// theme -> the current theme
    /// --- ---
    ///
    /// Returns: None
    ///
    fn fill_background(&self, frame: &mut Frame, guide_area: Rect, theme: &Theme) {
        let buffer = frame.buffer_mut();
        let buffer_width = buffer.area.width;
        let buffer_height = buffer.area.height;

        let start_y = guide_area.top();
        let start_x = guide_area.left();
        let end_y = guide_area.bottom().min(buffer_height);
        let end_x = guide_area.right().min(buffer_width);

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

    /// Build the guide content lines
    ///
    /// Arguments:
    /// --- ---
    /// theme -> the theme to use for styling
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Vec<Line> -> the guide content lines
    /// --- ---
    ///
    fn build_guide_lines(&self, theme: &Theme) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(""));

        // basic query structure
        lines.push(Line::from(vec![Span::styled(
            "BASIC QUERY STRUCTURE",
            Style::default()
                .fg(theme.success_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Combine queries with 'and' and 'or' operators:",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof contains Smith and subject = CMPT",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  course contains CS or course contains MATH",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Use parentheses for grouping:",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  (prof contains Smith or prof contains Jones) and credit hours > 3",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  (subject = CMPT or subject = MATH) and enrollment < 30",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // professor queries
        lines.push(Line::from(vec![Span::styled(
            "PROFESSOR QUERIES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Examples:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  prof contains Smith",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof equals \"John Doe\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof starts with A",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof ends with son",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof is \"Smith, John\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof != \"Unknown\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prof has \"PhD\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // course queries
        lines.push(Line::from(vec![Span::styled(
            "COURSE QUERIES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Subject Code:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  subject equals CMPT  (or 'sub' for short)",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  sub = MATH",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  subject contains CS",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Course Number:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  course number = 424N",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  course number contains 101",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  course = 203L",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Title:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  title contains \"Data Structures\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  title starts with \"Introduction\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  description contains \"machine learning\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  description has \"prerequisites\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Credit Hours:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  credit hours > 3",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  credit hours = 4",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  credit hours >= 3",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  credit hours at least 3",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Prerequisites:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  prereqs contains \"CMPT 101\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  prereqs has \"MATH 201\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Corequisites:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  coreqs contains \"LAB 101\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // time queries
        lines.push(Line::from(vec![Span::styled(
            "TIME QUERIES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Time Comparison:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  start > 9:00am",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  start >= 8:00am",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  end < 5:00pm",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  end <= 4:30pm",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  start = 10:00am",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Time Range:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  start 8:00am to 10:00am",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  end 2:00pm to 4:00pm",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  start 9:30am to 11:00am",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Note: Times must include 'am' or 'pm' suffix",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(""));

        // day queries
        lines.push(Line::from(vec![Span::styled(
            "DAY QUERIES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Day Keywords:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  Monday:   monday, monda, mond, mon, mo, m",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  Tuesday:  tuesday, tuesda, tuesd, tues, tue, tu",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  Wednesday: wednesday, wednesda, wednesd, wednes, wedne, wedn, wed, we, w",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  Thursday:  thursday, thursda, thurs, thur, thu, th",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  Friday:    friday, frida, frid, fri, fr, f",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  Saturday:  saturday, saturda, saturd, satur, satu, sat, sa",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  Sunday:    sunday, sunda, sund, sun, su",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Examples:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  monday = true",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  tuesday",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  wednesday = false",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  thursday = true",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  friday",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // enrollment queries
        lines.push(Line::from(vec![Span::styled(
            "ENROLLMENT QUERIES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Current Enrollment:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  enrollment < 30",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  enrollment >= 20",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  size < 25",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  size at least 15",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Enrollment Cap:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  cap > 50",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  enrollment cap <= 30",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  cap at most 25",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // other queries
        lines.push(Line::from(vec![Span::styled(
            "OTHER QUERIES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Instruction Method:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  method contains Online",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  method = \"In Person\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Campus:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  campus equals Main",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  campus contains \"North\"",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Meeting Type:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  type = Lecture",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  type contains Lab",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Full Status:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  full = true",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  full = false",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // conditions and operators
        lines.push(Line::from(vec![Span::styled(
            "CONDITIONS & OPERATORS",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "String Conditions:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  =, !=, contains, has, starts with, ends with,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  is, is not, equals, not equals, does not equal,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  doesn't equal, doesnt equal, does not contain,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  doesn't contain, doesnt contain",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Numeric Operators:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  =, !=, <, >, <=, >=, equals, is, is not,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  not equals, does not equal, less than,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  greater than, less than or equal to,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  greater than or equal to, at least, at most,",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(Span::styled(
            "  more than, fewer than",
            Style::default().fg(theme.text_color),
        )));
        lines.push(Line::from(""));

        // complex examples
        lines.push(Line::from(vec![Span::styled(
            "COMPLEX EXAMPLES",
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Multiple conditions:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  subject = CMPT and credit hours > 3 and enrollment < 30",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Time-based queries:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  start > 9:00am and end < 5:00pm",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  start 8:00am to 12:00pm and monday = true",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Day combinations:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  monday = true and wednesday = true and friday = true",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Professor and course:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  prof contains Smith and subject = CMPT",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Complex grouping:",
            Style::default()
                .fg(theme.warning_color)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  (subject = CMPT or subject = MATH) and (credit hours >= 3)",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "  (prof contains Smith or prof contains Jones) and enrollment < 25",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(""));

        // footer
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "PROJECT & LICENSE",
            Style::default()
                .fg(theme.success_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "GitHub: https://github.com/CFdefense/ClassQL",
            Style::default().fg(theme.muted_color),
        )));
        lines.push(Line::from(Span::styled(
            "Author: christian farrell (cfdefense) | License: MIT",
            Style::default().fg(theme.muted_color),
        )));

        lines
    }
}

impl Widget for QueryGuideWidget {
    /// Render the query guide
    ///
    /// Arguments:
    /// --- ---
    /// frame -> the frame to render to
    /// theme -> the theme to use for styling
    /// --- ---
    ///
    /// Returns: None
    ///
    fn render(&self, frame: &mut Frame, theme: &Theme) {
        self.render_guide(frame, theme);
    }

    /// Handle key event
    ///
    /// Arguments:
    /// --- ---
    /// key -> the key event to handle
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// KeyAction -> the action to take in response to the key
    /// --- ---
    ///
    fn handle_key(&mut self, key: KeyEvent) -> KeyAction {
        match key.code {
            KeyCode::Esc => {
                let return_to = self.return_focus.clone();
                self.reset();
                KeyAction::Navigate(return_to)
            }
            KeyCode::Char('g') | KeyCode::Char('G')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                let return_to = self.return_focus.clone();
                self.reset();
                KeyAction::Navigate(return_to)
            }
            KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                if self.max_scroll > 0 {
                    self.scroll = (self.scroll + 1).min(self.max_scroll);
                } else {
                    self.scroll += 1;
                }
                KeyAction::Continue
            }
            KeyCode::PageUp => {
                if self.scroll >= 10 {
                    self.scroll -= 10;
                } else {
                    self.scroll = 0;
                }
                KeyAction::Continue
            }
            KeyCode::PageDown => {
                if self.max_scroll > 0 {
                    self.scroll = (self.scroll + 10).min(self.max_scroll);
                } else {
                    self.scroll += 10;
                }
                KeyAction::Continue
            }
            KeyCode::Home => {
                self.scroll = 0;
                KeyAction::Continue
            }
            KeyCode::End => {
                if self.max_scroll > 0 {
                    self.scroll = self.max_scroll;
                } else {
                    self.scroll = 10000;
                }
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        }
    }

    /// Get the focus modes this widget handles
    ///
    /// Arguments: None
    ///
    /// Returns:
    /// --- ---
    /// Vec<FocusMode> -> the focus modes this widget handles
    /// --- ---
    ///
    fn focus_modes(&self) -> Vec<FocusMode> {
        vec![FocusMode::QueryGuide]
    }
}
