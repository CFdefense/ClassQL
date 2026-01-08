/// src/tui/widgets/results.rs
///
/// Query results widget rendering
///
/// Renders the query results in a 3-column grid
use crate::data::sql::Class;
use crate::tui::state::FocusMode;
use crate::tui::themes::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the query results in a 3-column grid below the search bar
///
/// Parameters:
/// --- ---
/// frame -> The frame to render
/// classes -> The classes to display
/// scroll -> The scroll offset (number of classes to skip)
/// focus_mode -> Current focus mode
/// selected_result -> Index of currently selected result
/// --- ---
///
/// Returns:
/// --- ---
/// (usize, usize) -> (number_of_items_rendered, max_items_that_fit)
/// --- ---
///
pub fn render_query_results(
    frame: &mut Frame,
    classes: &[Class],
    scroll: usize,
    focus_mode: &FocusMode,
    selected_result: usize,
    theme: &Theme,
) -> (usize, usize) {
    if classes.is_empty() {
        return (0, 0);
    }

    let is_browse_mode =
        *focus_mode == FocusMode::ResultsBrowse || *focus_mode == FocusMode::DetailView;

    // position the results grid below the search bar
    let logo_height = 7; // height of the ASCII art logo
    let search_y = logo_height + 2; // search bar position
    let search_height = 3; // search bar height
    let results_y = search_y + search_height + 1; // 1 line below search bar

    // calculate available space for results
    let available_height = frame.area().height.saturating_sub(results_y + 10); // leave room for help text and logo
    let cell_height = 7_u16; // height per class box
    let rows_to_show = (available_height / cell_height).max(1) as usize;

    // calculate grid dimensions
    let cell_width = 26_u16;
    let cols = 3_usize;
    let grid_width = cell_width * cols as u16 + (cols as u16 - 1) * 2; // cells + gaps
    let grid_x = frame.area().width.saturating_sub(grid_width) / 2;

    // calculate how many items can actually fit
    let max_items_that_fit = rows_to_show * cols;

    // apply scroll offset and get visible classes
    let visible_classes: Vec<(usize, &Class)> = classes
        .iter()
        .enumerate()
        .skip(scroll)
        .take(max_items_that_fit)
        .collect();

    let items_rendered = visible_classes.len();

    // render each class in a 3-column grid
    for (global_idx, class) in visible_classes.iter() {
        let idx = global_idx - scroll; // local index in visible area
        let row = idx / cols;
        let col = idx % cols;

        let cell_x = grid_x + (col as u16 * (cell_width + 2));
        let cell_y = results_y + (row as u16 * cell_height);

        let is_selected = is_browse_mode && *global_idx == selected_result;

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
        }.intersection(frame.area()); // ensure it's within frame bounds

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

    (items_rendered, max_items_that_fit)
}
