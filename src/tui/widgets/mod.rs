/// src/tui/widgets/mod.rs
///
/// Widgets module for the TUI
///
/// Contains all individual widget rendering functions

pub mod logo;
pub mod search_bar;
pub mod results;
pub mod detail_view;
pub mod toast;
pub mod completion;
pub mod menu;
pub mod helpers;
pub mod settings;

pub use logo::render_logo;
pub use search_bar::render_search_bar_with_data;
pub use results::render_query_results;
pub use detail_view::render_detail_view;
pub use toast::render_toast_with_data;
pub use completion::render_completion_dropdown;
pub use menu::{render_main_menu, MenuOption};
pub use helpers::render_search_helpers_with_data;
pub use settings::render_settings;

