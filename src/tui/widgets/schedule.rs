/// src/tui/widgets/schedule.rs
///
/// schedule creation widget rendering
///
/// renders the schedule creation interface with cart and generated schedules
/// also contains schedule generation logic for finding non-conflicting schedules
use crate::data::sql::Class;
use crate::tui::themes::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// render the schedule creation interface
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// cart -> Set of class IDs in the cart
/// selected_for_schedule -> Set of class IDs selected for schedule generation
/// query_results -> All query results to look up classes by ID
/// generated_schedules -> All generated non-conflicting schedules
/// current_schedule_index -> Index of currently displayed schedule
/// cart_focused -> Whether the cart section is focused
/// selected_cart_index -> Index of currently selected cart item
/// selection_mode -> Whether in class selection mode (true) or schedule viewing mode (false)
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
    cart_classes: &std::collections::HashMap<String, crate::data::sql::Class>,
    selected_for_schedule: &std::collections::HashSet<String>,
    generated_schedules: &[Vec<Class>],
    current_schedule_index: usize,
    cart_focused: bool,
    selected_cart_index: usize,
    selection_mode: bool,
    selected_time_block_day: usize,
    selected_time_block_slot: usize,
    schedule_name: Option<&str>,
    saved_schedule_index: Option<usize>,
    total_saved_schedules: Option<usize>,
    theme: &Theme,
) {
    let frame_width = frame.area().width;
    let frame_height = frame.area().height;

    // position below logo at top (logo is 7 lines tall, add spacing)
    let logo_height = 7_u16;
    let spacing = 6_u16;
    let start_y = logo_height + spacing;

    // calculate size - use full available height for schedule viewing
    let max_width = 90_u16.min(frame_width.saturating_sub(4)); // leave margins, max 90 chars wide
    let max_height = if selection_mode {
        // in selection mode, limit height for cart
        (frame_height.saturating_sub(start_y + 3)).min(20)
    } else {
        // in viewing mode, use full available height for calendar
        // only reserve minimal space for help text (1 line) and gap/counter (2 lines)
        frame_height.saturating_sub(start_y + 1 + 2) // start_y + help text + gap/counter
    };
    
    let schedule_x = (frame_width.saturating_sub(max_width)) / 2;
    
    let area = Rect {
        x: schedule_x,
        y: start_y,
        width: max_width,
        height: max_height,
    };

    if selection_mode {
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
        render_cart_section(frame, cart_area, message_area, cart_classes, selected_for_schedule, cart_focused, selected_cart_index, theme);
    } else {
        // in viewing mode, show time-block calendar
        // if schedule name is provided, render it above the schedule with a gap
        let schedule_area = if let Some(name) = schedule_name {
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
            let name_para = Paragraph::new(name)
                .style(Style::default()
                    .fg(theme.title_color)
                    .add_modifier(Modifier::BOLD))
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
        
        if !generated_schedules.is_empty() && current_schedule_index < generated_schedules.len() {
            render_time_block_calendar(
                frame,
                schedule_area,
                &generated_schedules[current_schedule_index],
                current_schedule_index,
                generated_schedules.len(),
                selected_time_block_day,
                selected_time_block_slot,
                saved_schedule_index,
                total_saved_schedules,
                theme,
            );
        } else {
            render_empty_schedule_section(frame, schedule_area, true, theme);
        }
    }
}

/// render cart section
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// cart_area -> The area to render the cart section in
/// message_area -> The area to render messages below the cart
/// cart -> Set of class IDs in the cart
/// selected_for_schedule -> Set of class IDs selected for schedule generation
/// query_results -> All query results to look up classes by ID
/// focused -> Whether the cart section is focused
/// selected_index -> Index of currently selected cart item
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_cart_section(
    frame: &mut Frame,
    cart_area: Rect,
    message_area: Rect,
    cart_classes: &std::collections::HashMap<String, Class>,
    selected_for_schedule: &std::collections::HashSet<String>,
    focused: bool,
    selected_index: usize,
    theme: &Theme,
) {
    let cart_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(cart_area);
    
    let message_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(message_area);

    let border_color = if focused {
        theme.selected_color
    } else {
        theme.border_color
    };

    // cart items - get classes from cart_classes map, sorted by ID for consistent ordering
    let mut cart_classes_vec: Vec<&Class> = cart_classes.values().collect();
    // Sort by unique_id for consistent ordering
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
                let is_selected = focused && idx == selected_index;
                let class_id = class.unique_id();
                let checkbox = if selected_for_schedule.contains(&class_id) { "☑ " } else { "☐ " };
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
                            class.subject_code,
                            class.course_number,
                            class.section_sequence
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
    let empty_line = Paragraph::new("")
        .style(Style::default().fg(theme.background_color));
    frame.render_widget(empty_line, message_chunks[1]);
    
    // message to press enter to continue
    let message2 = Paragraph::new("Press Enter to continue")
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(message2, message_chunks[2]);
}

/// find class at a specific time block
///
/// Parameters:
/// --- ---
/// schedule -> The schedule classes
/// day -> Day index (0-6 for Mon-Sun)
/// slot -> Time slot index (0-23 for 8am-8pm in 30-min intervals)
/// --- ---
///
/// Returns:
/// --- ---
/// Option<&Class> -> The class at that time block, if any
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

/// render time-block calendar view
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// area -> The area to render the calendar in
/// schedule -> The schedule classes to display
/// current_index -> Index of currently displayed schedule
/// total_schedules -> Total number of schedules available
/// selected_day -> Selected day index (0-6 for Mon-Sun)
/// selected_slot -> Selected time slot index
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
fn render_time_block_calendar(
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
    // split area: calendar on top, gap, schedule counter at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let calendar_area = chunks[0];
    let _gap_area = chunks[1]; // gap between calendar and counter
    let counter_area = chunks[2];

    // time slots: 8am to 10:30pm, 30-minute intervals = 29 slots
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
            // Format with leading zero for single-digit hours (1-9) to make all times 5 digits
            let time_str = format!("{:02}:{:02}{}", display_hour, minutes, period);
            (half_hour * 30, time_str) // minutes since midnight
        })
        .collect();

    // day names
    let day_names = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let day_codes = vec!["M", "T", "W", "TH", "F", "S", "SU"];

    // build time block grid: map (day, slot) -> class
    let mut time_blocks: std::collections::HashMap<(usize, usize), &Class> = std::collections::HashMap::new();
    
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
        // Day headers are never highlighted, only time slots are highlighted
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

    // render time slots
    for (slot_idx, (_, time_str)) in time_slots.iter().enumerate() {
        let slot_y = header_y + 1 + slot_idx as u16;
        if slot_y >= calendar_area.y + calendar_area.height {
            break;
        }

        // render time label
        let time_area = Rect {
            x: calendar_area.x,
            y: slot_y,
            width: time_col_width,
            height: 1,
        };
        let time_para = Paragraph::new(time_str.clone())
            .style(Style::default().fg(theme.muted_color));
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
                let block_para = Paragraph::new(" ")
                    .style(style);
                frame.render_widget(block_para, block_area);
            }
        }
    }

    // render schedule counter at bottom
    // if viewing from saved schedules, show saved schedule index instead
    let counter_text = if let (Some(saved_idx), Some(total_saved)) = (saved_schedule_index, total_saved_schedules) {
        format!("Schedule {} of {}", saved_idx + 1, total_saved)
    } else {
        format!("Schedule {} of {}", current_index + 1, total_schedules)
    };
    let counter_para = Paragraph::new(counter_text)
        .style(Style::default().fg(theme.info_color))
        .alignment(Alignment::Center);
    frame.render_widget(counter_para, counter_area);
}

/// render empty schedule section
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// area -> The area to render the empty schedule section in
/// focused -> Whether the schedule section is focused
/// theme -> The current theme
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
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

/// generate all possible non-conflicting schedules from classes in the cart
///
/// Parameters:
/// --- ---
/// cart_classes -> Map of all classes in the cart (ID -> Class)
/// selected_for_schedule -> Set of class IDs selected for schedule generation
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> All valid schedule combinations
/// --- ---
///
pub fn generate_schedules(
    cart_classes: &std::collections::HashMap<String, Class>,
    selected_for_schedule: &std::collections::HashSet<String>,
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

/// generate all possible schedules from classes (including conflicting ones)
///
/// Parameters:
/// --- ---
/// classes -> List of classes to generate schedules from
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> All schedule combinations (including conflicts)
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
            // We've considered all classes
            if !current_schedule.is_empty() {
                all_schedules.push(current_schedule.clone());
            }
            return;
        }

        // Try adding current class (no conflict check)
        current_schedule.push(classes[index].clone());
        backtrack(classes, current_schedule, index + 1, all_schedules);
        current_schedule.pop();

        // Try without adding current class
        backtrack(classes, current_schedule, index + 1, all_schedules);
    }

    let mut current = Vec::new();
    backtrack(classes, &mut current, 0, &mut all_schedules);

    all_schedules
}

/// find all valid (non-conflicting) schedules from a list of classes
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
    let mut all_valid_schedules = Vec::new();

    // use backtracking to generate all valid combinations
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
    backtrack(classes, &mut current, 0, &mut all_valid_schedules);

    // filter to keep only maximal schedules (schedules that are not subsets of other schedules)
    filter_maximal_schedules(&all_valid_schedules)
}

/// filter schedules to keep only maximal ones (remove schedules that are subsets of others)
///
/// Parameters:
/// --- ---
/// schedules -> All valid schedules
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<Vec<Class>> -> Only maximal schedules
/// --- ---
///
fn filter_maximal_schedules(schedules: &[Vec<Class>]) -> Vec<Vec<Class>> {
    let mut maximal_schedules = Vec::new();
    
    for schedule in schedules {
        let schedule_ids: std::collections::HashSet<String> = schedule
            .iter()
            .map(|c| c.unique_id())
            .collect();
        
        // check if this schedule is a subset of any other schedule
        let is_subset = schedules.iter().any(|other_schedule| {
            if other_schedule.len() <= schedule.len() {
                return false; // can't be a subset if other is same size or smaller
            }
            let other_ids: std::collections::HashSet<String> = other_schedule
                .iter()
                .map(|c| c.unique_id())
                .collect();
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

/// check if any classes in a list have conflicts
///
/// Parameters:
/// --- ---
/// classes -> List of classes to check
/// --- ---
///
/// Returns:
/// --- ---
/// bool -> True if any classes conflict, false otherwise
/// --- ---
///
pub fn has_conflicts(classes: &[Class]) -> bool {
    for i in 0..classes.len() {
        for j in (i + 1)..classes.len() {
            if classes_conflict(&classes[i], &classes[j]) {
                return true;
            }
        }
    }
    false
}

/// find all conflicting class pairs
///
/// Parameters:
/// --- ---
/// classes -> List of classes to check
/// --- ---
///
/// Returns:
/// --- ---
/// Vec<(String, String)> -> List of (class1_id, class2_id) pairs that conflict
/// --- ---
///
pub fn find_conflicting_classes(classes: &[Class]) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();
    for i in 0..classes.len() {
        for j in (i + 1)..classes.len() {
            if classes_conflict(&classes[i], &classes[j]) {
                let class1_id = format!("{} {}-{}", classes[i].subject_code, classes[i].course_number, classes[i].section_sequence);
                let class2_id = format!("{} {}-{}", classes[j].subject_code, classes[j].course_number, classes[j].section_sequence);
                conflicts.push((class1_id, class2_id));
            }
        }
    }
    conflicts
}

/// check if two classes conflict (overlap in time)
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

/// parse meeting times string into structured format
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

/// parse day codes into individual days
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

/// convert time string (HH:MM:SS) to minutes since midnight
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

/// check if two meetings overlap
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
