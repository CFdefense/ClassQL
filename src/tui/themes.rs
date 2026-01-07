/// src/tui/themes.rs
///
/// Theme system for the TUI
///
/// Contains color palettes and theme definitions
///
/// Contains:
/// --- ---
/// Theme -> Theme structure with color definitions
/// ThemePalette -> Available theme palette options
/// --- ---

use ratatui::style::Color;

/// Theme structure
///
/// Contains color definitions for all UI elements
///
/// Fields:
/// --- ---
/// name -> Theme name
/// logo_color -> Color for the logo/ASCII art
/// border_color -> Color for borders
/// title_color -> Color for titles and headers
/// text_color -> Color for regular text
/// selected_color -> Color for selected items
/// background_color -> Color for background
/// error_color -> Color for error messages
/// warning_color -> Color for warning messages
/// success_color -> Color for success messages
/// info_color -> Color for info messages
/// muted_color -> Color for muted/secondary text
/// --- ---
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub logo_color: Color,
    pub border_color: Color,
    pub title_color: Color,
    pub text_color: Color,
    pub selected_color: Color,
    pub background_color: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub success_color: Color,
    pub info_color: Color,
    pub muted_color: Color,
}

/// ThemePalette enum
///
/// Available theme palette options
///
/// ThemePalette types:
/// --- ---
/// Default -> Default theme with cyan accents
/// Dark -> Dark theme with gray tones
/// Light -> Light theme with dark text on light background
/// Blue -> Blue-themed color scheme
/// Green -> Green-themed color scheme
/// Purple -> Purple-themed color scheme
/// --- ---
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemePalette {
    Default,
    Dark,
    Light,
    Blue,
    Green,
    Purple,
}

impl ThemePalette {
    /// Get all available theme palettes
    ///
    /// Returns:
    /// --- ---
    /// Vec<ThemePalette> -> Vector of all theme palettes
    /// --- ---
    ///
    pub fn all() -> Vec<ThemePalette> {
        vec![
            ThemePalette::Default,
            ThemePalette::Dark,
            ThemePalette::Light,
            ThemePalette::Blue,
            ThemePalette::Green,
            ThemePalette::Purple,
        ]
    }

    /// Get the string representation of the theme palette
    ///
    /// Returns:
    /// --- ---
    /// &'static str -> String name of the theme palette
    /// --- ---
    ///
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemePalette::Default => "Default",
            ThemePalette::Dark => "Dark",
            ThemePalette::Light => "Light",
            ThemePalette::Blue => "Blue",
            ThemePalette::Green => "Green",
            ThemePalette::Purple => "Purple",
        }
    }

    /// Convert ThemePalette to Theme structure
    ///
    /// Returns:
    /// --- ---
    /// Theme -> Theme structure with color definitions
    /// --- ---
    ///
    pub fn to_theme(&self) -> Theme {
        match self {
            ThemePalette::Default => Theme {
                name: "Default",
                logo_color: Color::Rgb(135, 206, 235), // Sky blue
                border_color: Color::Cyan,
                title_color: Color::Cyan,
                text_color: Color::White,
                selected_color: Color::Cyan,
                background_color: Color::Black,
                error_color: Color::Red,
                warning_color: Color::Yellow,
                success_color: Color::Green,
                info_color: Color::Blue,
                muted_color: Color::DarkGray,
            },
            ThemePalette::Dark => Theme {
                name: "Dark",
                logo_color: Color::Rgb(200, 200, 200), // Light gray
                border_color: Color::Rgb(100, 100, 100),
                title_color: Color::Rgb(220, 220, 220),
                text_color: Color::Rgb(200, 200, 200),
                selected_color: Color::Rgb(150, 150, 255),
                background_color: Color::Black,
                error_color: Color::Rgb(255, 100, 100),
                warning_color: Color::Rgb(255, 200, 100),
                success_color: Color::Rgb(100, 255, 100),
                info_color: Color::Rgb(100, 150, 255),
                muted_color: Color::Rgb(80, 80, 80),
            },
            ThemePalette::Light => Theme {
                name: "Light",
                logo_color: Color::Rgb(30, 30, 150), // Dark blue
                border_color: Color::Rgb(50, 50, 200),
                title_color: Color::Rgb(30, 30, 150),
                text_color: Color::Rgb(20, 20, 20),
                selected_color: Color::Rgb(50, 50, 200),
                background_color: Color::Rgb(255, 255, 255),
                error_color: Color::Rgb(200, 50, 50),
                warning_color: Color::Rgb(200, 150, 50),
                success_color: Color::Rgb(50, 150, 50),
                info_color: Color::Rgb(50, 100, 200),
                muted_color: Color::Rgb(120, 120, 120),
            },
            ThemePalette::Blue => Theme {
                name: "Blue",
                logo_color: Color::Rgb(100, 150, 255), // Bright blue
                border_color: Color::Rgb(100, 150, 255),
                title_color: Color::Rgb(150, 200, 255),
                text_color: Color::Rgb(200, 220, 255),
                selected_color: Color::Rgb(150, 200, 255),
                background_color: Color::Rgb(20, 30, 50),
                error_color: Color::Rgb(255, 100, 100),
                warning_color: Color::Rgb(255, 220, 100),
                success_color: Color::Rgb(100, 255, 150),
                info_color: Color::Rgb(100, 180, 255),
                muted_color: Color::Rgb(80, 100, 130),
            },
            ThemePalette::Green => Theme {
                name: "Green",
                logo_color: Color::Rgb(100, 255, 150), // Bright green
                border_color: Color::Rgb(100, 255, 150),
                title_color: Color::Rgb(150, 255, 200),
                text_color: Color::Rgb(200, 255, 220),
                selected_color: Color::Rgb(150, 255, 200),
                background_color: Color::Rgb(20, 40, 25),
                error_color: Color::Rgb(255, 100, 100),
                warning_color: Color::Rgb(255, 220, 100),
                success_color: Color::Rgb(100, 255, 150),
                info_color: Color::Rgb(100, 200, 255),
                muted_color: Color::Rgb(60, 100, 70),
            },
            ThemePalette::Purple => Theme {
                name: "Purple",
                logo_color: Color::Rgb(200, 100, 255), // Bright purple
                border_color: Color::Rgb(200, 100, 255),
                title_color: Color::Rgb(220, 150, 255),
                text_color: Color::Rgb(240, 200, 255),
                selected_color: Color::Rgb(220, 150, 255),
                background_color: Color::Rgb(30, 20, 40),
                error_color: Color::Rgb(255, 100, 100),
                warning_color: Color::Rgb(255, 220, 100),
                success_color: Color::Rgb(150, 255, 100),
                info_color: Color::Rgb(150, 150, 255),
                muted_color: Color::Rgb(100, 70, 120),
            },
        }
    }
}

