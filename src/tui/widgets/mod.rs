/// src/tui/widgets/mod.rs
///
/// Widgets module for the TUI
///
/// Contains all individual widget rendering functions and interactive widget structs
/// 

// trait definitions
pub mod traits;

// widget structs with encapsulated state and interaction
pub mod guide;
pub mod menu;
pub mod schedule;
pub mod search;
pub mod settings;

// render-only widget modules
pub mod detail_view;
pub mod help_bar;
pub mod helpers;
pub mod logo;
pub mod toast;

// re-export trait
pub use traits::{KeyAction, Widget};

// re-export widget structs
pub use detail_view::DetailViewWidget;
pub use guide::QueryGuideWidget;
pub use help_bar::HelpBarWidget;
pub use logo::LogoWidget;
pub use menu::MainMenuWidget;
pub use schedule::{ScheduleAction, ScheduleWidget};
pub use search::{CompletionState, SearchFocus, SearchWidget};
pub use settings::{SettingsAction, SettingsWidget};
pub use toast::ToastWidget;
