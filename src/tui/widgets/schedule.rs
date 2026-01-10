/// src/tui/widgets/schedule.rs
///
/// Schedule creation widget rendering
///
/// Renders the schedule creation interface with cart and generated schedules
/// Also contains schedule generation logic for finding non-conflicting schedules
use crate::data::sql::Class;
use crate::tui::themes::Theme;
use crate::tui::widgets::helpers::{format_day_for_display, get_day_order};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the schedule creation interface
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// cart -> Set of class IDs in the cart
/// query_results -> All query results to look up classes by ID
/// generated_schedules -> All generated non-conflicting schedules
/// current_schedule_index -> Index of currently displayed schedule
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_schedule_creation(
    frame: &mut Frame,
    cart: &std::collections::HashSet<String>,
    selected_for_schedule: &std::collections::HashSet<String>,
    query_results: &[Class],
    generated_schedules: &[Vec<Class>],
    current_schedule_index: usize,
    cart_focused: bool,
    selected_cart_index: usize,
    theme: &Theme,
) {
    let frame_width = frame.area().width;
    let frame_height = frame.area().height;

    // Position below logo at top (logo is 7 lines tall, add spacing)
    let logo_height = 7_u16;
    let spacing = 2_u16;
    let start_y = logo_height + spacing;

    // Calculate compact size - limit width and height
    let max_width = 90_u16.min(frame_width.saturating_sub(4)); // leave margins, max 90 chars wide
    let max_height = (frame_height.saturating_sub(start_y + 3)).min(20); // leave room for help text, max 20 lines
    
    let schedule_x = (frame_width.saturating_sub(max_width)) / 2;
    
    let area = Rect {
        x: schedule_x,
        y: start_y,
        width: max_width,
        height: max_height,
    };

    // Create main layout: cart on left, schedule on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Render cart section
    render_cart_section(frame, chunks[0], cart, selected_for_schedule, query_results, cart_focused, selected_cart_index, theme);

    // Render schedule section
    if !generated_schedules.is_empty() && current_schedule_index < generated_schedules.len() {
        render_schedule_section(
            frame,
            chunks[1],
            &generated_schedules[current_schedule_index],
            current_schedule_index,
            generated_schedules.len(),
            !cart_focused,
            theme,
        );
    } else {
        render_empty_schedule_section(frame, chunks[1], !cart_focused, theme);
    }
}

/// Render the cart section
fn render_cart_section(
    frame: &mut Frame,
    area: Rect,
    cart: &std::collections::HashSet<String>,
    selected_for_schedule: &std::collections::HashSet<String>,
    query_results: &[Class],
    focused: bool,
    selected_index: usize,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(area);

    // Title
    let border_color = if focused {
        theme.selected_color
    } else {
        theme.border_color
    };
    let title = Paragraph::new("")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Shopping Cart ")
                .title_style(
                    Style::default()
                        .fg(theme.title_color)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(border_color)),
        );
    frame.render_widget(title, chunks[0]);

    // Cart items - show class names with checkboxes
    let cart_classes: Vec<(usize, &Class)> = query_results
        .iter()
        .enumerate()
        .filter(|(_, class)| cart.contains(&class.unique_id()))
        .collect();

    let cart_text = if cart_classes.is_empty() {
        vec![Line::from(Span::styled(
            "Cart is empty",
            Style::default().fg(theme.muted_color),
        ))]
    } else {
        cart_classes
            .iter()
            .enumerate()
            .map(|(idx, (_, class))| {
                let is_selected = focused && idx == selected_index;
                let class_id = class.unique_id();
                let checkbox = if selected_for_schedule.contains(&class_id) { "☑" } else { "☐" };
                let prefix = if is_selected { "> " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(theme.selected_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_color)
                };
                Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(
                        format!(
                            "{} {} {}-{}",
                            checkbox,
                            class.subject_code,
                            class.course_number,
                            class.section_sequence
                        ),
                        style,
                    ),
                ])
            })
            .collect()
    };

    let cart_widget = Paragraph::new(cart_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().bg(theme.background_color));
    frame.render_widget(cart_widget, chunks[1]);
}

/// Render the schedule section
fn render_schedule_section(
    frame: &mut Frame,
    area: Rect,
    schedule: &[Class],
    current_index: usize,
    total_schedules: usize,
    focused: bool,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(area);

    // Title with schedule navigation
    let border_color = if focused {
        theme.selected_color
    } else {
        theme.border_color
    };
    let title_text = format!(
        "Schedule {} of {}",
        current_index + 1,
        total_schedules
    );
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" Generated Schedule ")
        .title_style(
            Style::default()
                .fg(theme.title_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color));
    
    let title = Paragraph::new(title_text)
        .block(title_block)
        .style(
            Style::default()
                .fg(theme.info_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Schedule classes
    let mut schedule_lines = Vec::new();

    // Helper function to format time
    let format_time = |time: &str| -> String {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() >= 2 {
            let hours: i32 = parts[0].parse().unwrap_or(0);
            let minutes: i32 = parts[1].parse().unwrap_or(0);

            let (display_hour, period) = if hours == 0 {
                (12, "am")
            } else if hours < 12 {
                (hours, "am")
            } else if hours == 12 {
                (12, "pm")
            } else {
                (hours - 12, "pm")
            };

            format!("{}:{:02}{}", display_hour, minutes, period)
        } else {
            time.to_string()
        }
    };

    for class in schedule {
        // Course code and title (compact format)
        let title = if class.title.len() > 30 {
            format!("{}...", &class.title[..27])
        } else {
            class.title.clone()
        };
        schedule_lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "{} {}-{}: {}",
                    class.subject_code, class.course_number, class.section_sequence, title
                ),
                Style::default()
                    .fg(theme.info_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Meeting times
        if let Some(meeting_times_str) = &class.meeting_times {
            if !meeting_times_str.is_empty() {
                let mut meeting_times: Vec<(u8, String, String, String)> = Vec::new();

                for mt in meeting_times_str.split('|') {
                    if mt.is_empty() {
                        continue;
                    }
                    if let Some(colon_pos) = mt.find(':') {
                        let days_part = &mt[..colon_pos];
                        let time_part = &mt[colon_pos + 1..];
                        if let Some(dash_pos) = time_part.find('-') {
                            let start = format_time(&time_part[..dash_pos]);
                            let end = format_time(&time_part[dash_pos + 1..]);
                            if !days_part.is_empty() {
                                let first_day = if days_part.starts_with("TH") {
                                    "TH"
                                } else if days_part.starts_with("SU") {
                                    "SU"
                                } else if days_part.len() > 0 {
                                    &days_part[..1]
                                } else {
                                    days_part
                                };
                                let day_order = get_day_order(first_day);
                                let formatted_days = format_day_for_display(days_part);
                                meeting_times.push((day_order, formatted_days, start, end));
                            }
                        }
                    }
                }

                meeting_times.sort_by_key(|(day_order, _, _, _)| *day_order);

                for (_, days_part, start, end) in meeting_times {
                    schedule_lines.push(Line::from(vec![
                        Span::styled("  ", Style::default().fg(theme.text_color)),
                        Span::styled(
                            format!("{} {}-{}", days_part, start, end),
                            Style::default().fg(theme.success_color),
                        ),
                    ]));
                }
            }
        }
    }

    let schedule_widget = Paragraph::new(schedule_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().bg(theme.background_color));
    frame.render_widget(schedule_widget, chunks[1]);
}

/// Render empty schedule section
fn render_empty_schedule_section(frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
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

/// Generate all possible non-conflicting schedules from classes in the cart
///
/// Parameters:
/// --- ---
/// query_results -> All available classes from query results
/// cart -> Set of class IDs in the cart
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> All valid schedule combinations
/// --- ---
///
pub fn generate_schedules(
    query_results: &[Class],
    cart: &std::collections::HashSet<String>,
) -> Vec<Vec<Class>> {
    // Get all classes from cart
    let cart_classes: Vec<Class> = query_results
        .iter()
        .filter(|class| cart.contains(&class.unique_id()))
        .cloned()
        .collect();

    if cart_classes.is_empty() {
        return Vec::new();
    }

    // Generate all possible combinations and filter out conflicts
    find_valid_schedules(&cart_classes)
}

/// Find all valid (non-conflicting) schedules from a list of classes
///
/// Parameters:
/// --- ---
/// classes -> List of classes to generate schedules from
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> All valid schedule combinations
/// --- ---
///
pub fn find_valid_schedules(classes: &[Class]) -> Vec<Vec<Class>> {
    let mut valid_schedules = Vec::new();

    // Use backtracking to generate all combinations
    fn backtrack(
        classes: &[Class],
        current_schedule: &mut Vec<Class>,
        index: usize,
        valid_schedules: &mut Vec<Vec<Class>>,
    ) {
        if index >= classes.len() {
            // We've considered all classes
            if !current_schedule.is_empty() {
                valid_schedules.push(current_schedule.clone());
            }
            return;
        }

        // Try adding current class
        let current_class = &classes[index];
        let mut can_add = true;

        // Check for conflicts with existing classes in schedule
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

        // Try without adding current class
        backtrack(classes, current_schedule, index + 1, valid_schedules);
    }

    let mut current = Vec::new();
    backtrack(classes, &mut current, 0, &mut valid_schedules);

    valid_schedules
}

/// Check if two classes conflict (overlap in time)
///
/// Parameters:
/// --- ---
/// class1 -> First class
/// class2 -> Second class
/// --- ---
///
/// Returns:
/// --- ---
/// bool -> True if classes conflict, false otherwise
/// --- ---
///
pub fn classes_conflict(class1: &Class, class2: &Class) -> bool {
    // If either class has no meeting times, they don't conflict
    let times1 = match &class1.meeting_times {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };
    let times2 = match &class2.meeting_times {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };

    // Parse meeting times for both classes
    let meetings1 = parse_meeting_times(times1);
    let meetings2 = parse_meeting_times(times2);

    // Check for any overlap
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
/// Parameters:
/// --- ---
/// times_str -> Meeting times string (e.g., "M:08:00:00-10:45:00|TH:08:00:00-09:15:00")
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<(Vec<String>, i32, i32)> -> List of (days, start_minutes, end_minutes)
/// --- ---
///
pub fn parse_meeting_times(times_str: &str) -> Vec<(Vec<String>, i32, i32)> {
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

                // Parse days (handle "MW", "TTH", etc.)
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
/// Parameters:
/// --- ---
/// days_str -> Day string (e.g., "MW", "TTH")
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<String> -> List of day codes
/// --- ---
///
pub fn parse_days(days_str: &str) -> Vec<String> {
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
/// Parameters:
/// --- ---
/// time_str -> Time string
/// --- ---
///
/// Returns:
/// --- ---
/// i32 -> Minutes since midnight
/// --- ---
///
pub fn time_to_minutes(time_str: &str) -> i32 {
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
/// Parameters:
/// --- ---
/// m1 -> First meeting (days, start, end)
/// m2 -> Second meeting (days, start, end)
/// --- ---
///
/// Returns:
/// --- ---
/// bool -> True if meetings overlap, false otherwise
/// --- ---
///
pub fn meetings_overlap(
    m1: &(Vec<String>, i32, i32),
    m2: &(Vec<String>, i32, i32),
) -> bool {
    // Check if they share any day
    let days_overlap = m1.0.iter().any(|d| m2.0.contains(d));
    if !days_overlap {
        return false;
    }

    // Check if time ranges overlap
    let (_, start1, end1) = m1;
    let (_, start2, end2) = m2;

    // Overlap if: start1 < end2 && start2 < end1
    start1 < end2 && start2 < end1
}
