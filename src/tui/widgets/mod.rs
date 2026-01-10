pub mod completion;
pub mod detail_view;
pub mod helpers;
/// src/tui/widgets/mod.rs
///
/// Widgets module for the TUI
///
/// Contains all individual widget rendering functions
pub mod logo;
pub mod menu;
pub mod query_guide;
pub mod results;
pub mod schedule;
pub mod search_bar;
pub mod settings;
pub mod toast;

pub use completion::render_completion_dropdown;
pub use detail_view::render_detail_view;
pub use helpers::render_search_helpers_with_data;
pub use logo::render_logo;
pub use menu::{render_main_menu, MenuOption};
pub use query_guide::render_query_guide;
pub use results::render_query_results;
pub use schedule::{generate_schedules, find_valid_schedules, render_schedule_creation};
pub use search_bar::render_search_bar_with_data;
pub use settings::render_settings;
pub use toast::render_toast_with_data;
