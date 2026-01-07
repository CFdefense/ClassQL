/// src/tui/widgets/detail_view.rs
///
/// Detail view widget rendering
///
/// Renders detailed class information overlay

use crate::data::sql::Class;
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Render detailed view of a selected class as an overlay
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// class -> The class to display in detail
/// --- ---
///
/// Returns:
/// --- ---
/// None
/// --- ---
///
pub fn render_detail_view(frame: &mut Frame, class: &Class, theme: &Theme) {
    let detail_width = 60_u16;
    
    // calculate description lines needed (before building content)
    let content_width = (detail_width.saturating_sub(4)) as usize; // -4 for borders and padding
    let desc_lines = if let Some(desc) = &class.description {
        if !desc.trim().is_empty() {
            // calculate how many lines the description will take
            let mut remaining = desc.as_str();
            let mut lines_count = 0;
            let max_desc_lines = 8; // maximum description lines
            
            while !remaining.is_empty() && lines_count < max_desc_lines {
                if remaining.len() <= content_width {
                    lines_count += 1;
                    break;
                } else {
                    let mut break_point = content_width;
                    if let Some(space_pos) = remaining[..content_width.min(remaining.len())].rfind(' ') {
                        break_point = space_pos;
                    } else if let Some(comma_pos) = remaining[..content_width.min(remaining.len())].rfind(',') {
                        break_point = comma_pos + 1;
                    } else if let Some(period_pos) = remaining[..content_width.min(remaining.len())].rfind('.') {
                        break_point = period_pos + 1;
                    }
                    remaining = remaining[break_point..].trim_start();
                    lines_count += 1;
                }
            }
            lines_count
        } else {
            1 // "(No description available)" line
        }
    } else {
        1 // "(No description available)" line
    };
    
    // calculate base content lines (without description)
    let mut base_lines = 2; // course code + title
    base_lines += 1; // blank line
    base_lines += 1; // professor
    if class.professor_email.is_some() {
        base_lines += 1; // email
    }
    base_lines += 1; // blank line
    base_lines += 1; // "Schedule:" label
    // count schedule lines
    if let Some(meeting_times_str) = &class.meeting_times {
        if !meeting_times_str.is_empty() {
            base_lines += meeting_times_str.split('|').filter(|mt| !mt.is_empty()).count();
        } else {
            base_lines += 1; // "TBD"
        }
    } else {
        base_lines += 1; // "TBD"
    }
    if class.meeting_type.is_some() {
        base_lines += 1; // type
    }
    if class.campus.is_some() {
        base_lines += 1; // campus
    }
    base_lines += 1; // method
    base_lines += 1; // blank line
    base_lines += 1; // enrollment
    base_lines += 1; // credits
    base_lines += 2; // blank line + "Description:" label
    
    // total content lines = base + description lines
    let total_content_lines = base_lines + desc_lines;
    
    // calculate height: content + borders (2) + title (1)
    let min_height = 20_u16; // minimum height when no description
    let max_height = 35_u16; // maximum height
    let calculated_height = (total_content_lines as u16 + 3).min(max_height).max(min_height);
    let detail_height = calculated_height;

    let detail_area = Rect {
        x: (frame.area().width.saturating_sub(detail_width)) / 2,
        y: (frame.area().height.saturating_sub(detail_height)) / 2,
        width: detail_width,
        height: detail_height,
    };

    // build detailed content
    let mut lines: Vec<Line> = Vec::new();

    // course code and title
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} {} - {}", class.subject_code, class.course_number, class.section_sequence),
            Style::default().fg(theme.info_color).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        class.title.clone(),
        Style::default().fg(theme.text_color).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("")); // blank line

    // professor
    lines.push(Line::from(vec![
        Span::styled("Professor: ", Style::default().fg(theme.warning_color)),
        Span::styled(
            class.professor_name.as_deref().unwrap_or("TBA"),
            Style::default().fg(theme.text_color),
        ),
    ]));

    // email
    if let Some(email) = &class.professor_email {
        lines.push(Line::from(vec![
            Span::styled("Email: ", Style::default().fg(theme.warning_color)),
            Span::styled(email, Style::default().fg(theme.text_color)),
        ]));
    }

    lines.push(Line::from("")); // blank line

    // schedule
    lines.push(Line::from(vec![
        Span::styled("Schedule:", Style::default().fg(theme.success_color)),
    ]));
    
    // helper function to format time
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
    
    // parse meeting_times if available, otherwise fall back to old format
    if let Some(meeting_times_str) = &class.meeting_times {
        if !meeting_times_str.is_empty() {
            // parse meeting times: "M:08:00:00-10:45:00|R:08:00:00-09:15:00"
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
                        if !days_part.is_empty() && !start.is_empty() && !end.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::default().fg(theme.text_color)), // 4 spaces for indentation
                                Span::styled(format!("{} {}-{}", days_part, start, end), Style::default().fg(theme.text_color)),
                            ]));
                        }
                    }
                }
            }
        } else {
            // empty meeting_times
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default().fg(theme.text_color)), // 4 spaces for indentation
                Span::styled("TBD", Style::default().fg(theme.text_color)),
            ]));
        }
    } else {
        // no meeting_times available
        lines.push(Line::from(vec![
            Span::styled("    ", Style::default().fg(theme.text_color)), // 4 spaces for indentation
            Span::styled("TBD", Style::default().fg(theme.text_color)),
        ]));
    }

    // meeting type
    if let Some(meeting_type) = &class.meeting_type {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(theme.success_color)),
            Span::styled(meeting_type, Style::default().fg(theme.text_color)),
        ]));
    }

    // location/campus
    if let Some(campus) = &class.campus {
        lines.push(Line::from(vec![
            Span::styled("Campus: ", Style::default().fg(theme.success_color)),
            Span::styled(campus, Style::default().fg(theme.text_color)),
        ]));
    }

    // instruction method
    lines.push(Line::from(vec![
        Span::styled("Method: ", Style::default().fg(theme.success_color)),
        Span::styled(
            class.instruction_method.as_deref().unwrap_or("N/A"),
            Style::default().fg(theme.text_color),
        ),
    ]));

    lines.push(Line::from("")); // blank line

    // enrollment
    let enrollment_str = match (class.enrollment, class.max_enrollment) {
        (Some(e), Some(m)) => format!("{} / {} ({:.0}% full)", e, m, (e as f64 / m as f64) * 100.0),
        _ => "Unknown".to_string(),
    };
    lines.push(Line::from(vec![
        Span::styled("Enrollment: ", Style::default().fg(theme.info_color)),
        Span::styled(enrollment_str, Style::default().fg(theme.text_color)),
    ]));

    // credit hours
    lines.push(Line::from(vec![
        Span::styled("Credits: ", Style::default().fg(theme.info_color)),
        Span::styled(format!("{}", class.credit_hours), Style::default().fg(theme.text_color)),
    ]));

    // description
    lines.push(Line::from("")); // blank line
    lines.push(Line::from(vec![
        Span::styled("Description: ", Style::default().fg(theme.success_color)),
    ]));
    
    if let Some(desc) = &class.description {
        if !desc.trim().is_empty() {
            // wrap description to fit within detail width (account for borders and padding)
            let content_width = (detail_width.saturating_sub(4)) as usize; // -4 for borders and padding
            let mut remaining = desc.as_str();
            let mut desc_lines_added = 0;
            let max_desc_lines = 8; // maximum description lines to show
            
            while !remaining.is_empty() && desc_lines_added < max_desc_lines {
                if remaining.len() <= content_width {
                    lines.push(Line::from(Span::styled(remaining.to_string(), Style::default().fg(theme.muted_color))));
                    break;
                } else {
                    // find a good break point (space, comma, period, etc.)
                    let mut break_point = content_width;
                    if let Some(space_pos) = remaining[..content_width.min(remaining.len())].rfind(' ') {
                        break_point = space_pos;
                    } else if let Some(comma_pos) = remaining[..content_width.min(remaining.len())].rfind(',') {
                        break_point = comma_pos + 1;
                    } else if let Some(period_pos) = remaining[..content_width.min(remaining.len())].rfind('.') {
                        break_point = period_pos + 1;
                    }
                    
                    let line_text = if desc_lines_added == max_desc_lines - 1 {
                        // last line, truncate if needed
                        if remaining.len() > content_width {
                            format!("{}...", &remaining[..content_width.saturating_sub(3)])
                        } else {
                            remaining.to_string()
                        }
                    } else {
                        remaining[..break_point].to_string()
                    };
                    
                    lines.push(Line::from(Span::styled(line_text, Style::default().fg(theme.muted_color))));
                    remaining = remaining[break_point..].trim_start();
                    desc_lines_added += 1;
                }
            }
        } else {
            // description exists but is empty/whitespace
            lines.push(Line::from(Span::styled("(No description available)", Style::default().fg(theme.muted_color))));
        }
    } else {
        // description is None
        lines.push(Line::from(Span::styled("(No description available)", Style::default().fg(theme.muted_color))));
    }

    // first, clear the area to cover results below with solid background
    frame.render_widget(Clear, detail_area);


    let detail_paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Class Details ")
            .title_style(Style::default().fg(theme.title_color).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(theme.border_color))
            .style(Style::default().bg(theme.background_color)),
    );

    frame.render_widget(detail_paragraph, detail_area);
    
    // force white background on empty/border cells, preserve styled text cells
    let buffer = frame.buffer_mut();
    for y in detail_area.top()..detail_area.bottom() {
        for x in detail_area.left()..detail_area.right() {
            let cell = &mut buffer[(x, y)];
            // Only set white background if cell is empty or a border character
            // This preserves the text colors and backgrounds set by the paragraph
            if cell.symbol() == " " || cell.symbol() == "│" || cell.symbol() == "─" || 
               cell.symbol() == "┌" || cell.symbol() == "┐" || cell.symbol() == "└" || 
               cell.symbol() == "┘" || cell.symbol() == "├" || cell.symbol() == "┤" ||
               cell.symbol() == "┬" || cell.symbol() == "┴" {
                cell.set_bg(theme.background_color);
            }
        }
    }
}

