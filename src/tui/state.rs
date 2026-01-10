/// src/tui/state.rs
///
/// State management for the TUI
///
/// Contains enums and types that represent the application state
///
/// Contains:
/// --- ---
/// ErrorType -> Type of error (Lexer, Parser, Semantic)
/// FocusMode -> Current UI focus mode
/// --- ---

/// ErrorType enum
///
/// ErrorType types:
/// --- ---
/// Lexer -> Lexer error
/// Parser -> Parser error
/// Semantic -> Semantic error
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for ErrorType
/// Clone -> Clone trait for ErrorType
/// --- ---
///
#[derive(Debug, Clone)]
pub enum ErrorType {
    Lexer,
    Parser,
    Semantic,
}

/// FocusMode enum - tracks which element has keyboard focus
///
/// FocusMode types:
/// --- ---
/// MainMenu -> User is in the main menu
/// QueryInput -> User is typing in the query box
/// ResultsBrowse -> User is browsing/selecting results
/// DetailView -> User is viewing detailed class info
/// Settings -> User is in the settings menu
/// QueryGuide -> User is viewing the query guide
/// Help -> User is viewing the help page
/// ScheduleCreation -> User is creating a schedule
/// --- ---
///
#[derive(Debug, Clone, PartialEq)]
pub enum FocusMode {
    MainMenu,
    QueryInput,
    ResultsBrowse,
    DetailView,
    Settings,
    QueryGuide,
    Help,
    ScheduleCreation,
}
