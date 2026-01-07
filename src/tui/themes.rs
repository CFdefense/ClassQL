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
/// Default -> Default light theme with dark text on light background
/// Dark -> Traditional dark theme with light text on dark background
/// Pastel -> Pastel theme with soft colors
/// Blue -> Blue-themed color scheme
/// Green -> Green-themed color scheme
/// Purple -> Purple-themed color scheme
/// Orange -> Orange-themed color scheme
/// Red -> Red-themed color scheme
/// Monochrome -> Monochrome black and white theme
/// Cyberpunk -> Cyberpunk-inspired neon theme
/// Forest -> Forest/nature-inspired green theme
/// Ocean -> Ocean-inspired blue-green theme
/// Sunset -> Sunset-inspired warm color theme
/// --- ---
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemePalette {
    Default,
    Dark,
    Pastel,
    Blue,
    Green,
    Purple,
    Orange,
    Red,
    Monochrome,
    Cyberpunk,
    Forest,
    Ocean,
    Sunset,
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
            ThemePalette::Pastel,
            ThemePalette::Blue,
            ThemePalette::Green,
            ThemePalette::Purple,
            ThemePalette::Orange,
            ThemePalette::Red,
            ThemePalette::Monochrome,
            ThemePalette::Cyberpunk,
            ThemePalette::Forest,
            ThemePalette::Ocean,
            ThemePalette::Sunset,
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
            ThemePalette::Pastel => "Pastel",
            ThemePalette::Blue => "Blue",
            ThemePalette::Green => "Green",
            ThemePalette::Purple => "Purple",
            ThemePalette::Orange => "Orange",
            ThemePalette::Red => "Red",
            ThemePalette::Monochrome => "Monochrome",
            ThemePalette::Cyberpunk => "Cyberpunk",
            ThemePalette::Forest => "Forest",
            ThemePalette::Ocean => "Ocean",
            ThemePalette::Sunset => "Sunset",
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
                logo_color: Color::Rgb(30, 30, 150), // Dark blue
                border_color: Color::Red, // Red
                title_color: Color::Rgb(30, 30, 150), // Dark blue
                text_color: Color::Rgb(20, 20, 20), // Very dark gray / black
                selected_color: Color::Red, // Red
                background_color: Color::Rgb(255, 255, 255), // White
                error_color: Color::Rgb(200, 50, 50), // Red
                warning_color: Color::Rgb(200, 150, 50), // Orange
                success_color: Color::Rgb(50, 150, 50), // Green
                info_color: Color::Rgb(50, 100, 200), // Blue
                muted_color: Color::Rgb(120, 120, 120), // Medium gray
            },
            ThemePalette::Dark => Theme {
                name: "Dark",
                logo_color: Color::Rgb(200, 200, 200), // Light gray
                border_color: Color::Rgb(100, 100, 100), // Medium gray
                title_color: Color::Rgb(255, 255, 255), // White
                text_color: Color::Rgb(220, 220, 220), // Light gray
                selected_color: Color::Rgb(100, 150, 255), // Bright blue for selection
                background_color: Color::Rgb(18, 18, 18), // Very dark gray, almost black
                error_color: Color::Rgb(255, 85, 85), // Light red
                warning_color: Color::Rgb(255, 184, 108), // Light orange
                success_color: Color::Rgb(85, 255, 85), // Light green
                info_color: Color::Rgb(100, 150, 255), // Bright blue
                muted_color: Color::Rgb(100, 100, 100), // Medium gray
            },
            ThemePalette::Pastel => Theme {
                name: "Pastel",
                logo_color: Color::Rgb(135, 206, 235), // Sky blue
                border_color: Color::Rgb(200, 180, 220), // Soft purple
                title_color: Color::Rgb(150, 150, 200), // Soft blue
                text_color: Color::Rgb(60, 60, 80), // Dark gray
                selected_color: Color::Rgb(150, 200, 255), // Soft blue
                background_color: Color::Rgb(250, 248, 255), // Very light purple/white
                error_color: Color::Rgb(255, 150, 150), // Soft red
                warning_color: Color::Rgb(255, 220, 150), // Soft yellow
                success_color: Color::Rgb(150, 255, 200), // Soft green
                info_color: Color::Rgb(150, 200, 255), // Soft blue
                muted_color: Color::Rgb(180, 180, 200), // Soft gray
            },
            ThemePalette::Blue => Theme {
                name: "Blue",
                logo_color: Color::Rgb(100, 150, 255), // Bright blue
                border_color: Color::Rgb(100, 150, 255), // Bright blue
                title_color: Color::Rgb(150, 200, 255), // Light blue
                text_color: Color::Rgb(200, 220, 255), // Very light blue
                selected_color: Color::Rgb(150, 200, 255), // Light blue
                background_color: Color::Rgb(20, 30, 50), // Dark blue-gray
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 220, 100), // Light yellow
                success_color: Color::Rgb(100, 255, 150), // Light green
                info_color: Color::Rgb(100, 180, 255), // Bright blue
                muted_color: Color::Rgb(80, 100, 130), // Blue-gray
            },
            ThemePalette::Green => Theme {
                name: "Green",
                logo_color: Color::Rgb(100, 255, 150), // Bright green
                border_color: Color::Rgb(100, 255, 150), // Bright green
                title_color: Color::Rgb(150, 255, 200), // Light green
                text_color: Color::Rgb(200, 255, 220), // Very light green
                selected_color: Color::Rgb(150, 255, 200), // Light green
                background_color: Color::Rgb(20, 40, 25), // Dark green-gray
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 220, 100), // Light yellow
                success_color: Color::Rgb(100, 255, 150), // Bright green
                info_color: Color::Rgb(100, 200, 255), // Bright blue
                muted_color: Color::Rgb(60, 100, 70), // Green-gray
            },
            ThemePalette::Purple => Theme {
                name: "Purple",
                logo_color: Color::Rgb(200, 100, 255), // Bright purple
                border_color: Color::Rgb(200, 100, 255), // Bright purple
                title_color: Color::Rgb(220, 150, 255), // Light purple
                text_color: Color::Rgb(240, 200, 255), // Very light purple
                selected_color: Color::Rgb(220, 150, 255), // Light purple
                background_color: Color::Rgb(30, 20, 40), // Dark purple-gray
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 220, 100), // Light yellow
                success_color: Color::Rgb(150, 255, 100), // Light green
                info_color: Color::Rgb(150, 150, 255), // Light purple-blue
                muted_color: Color::Rgb(100, 70, 120), // Purple-gray
            },
            ThemePalette::Orange => Theme {
                name: "Orange",
                logo_color: Color::Rgb(255, 165, 0), // Bright orange
                border_color: Color::Rgb(255, 140, 0), // Dark orange
                title_color: Color::Rgb(255, 200, 100), // Light orange
                text_color: Color::Rgb(255, 240, 220), // Very light orange
                selected_color: Color::Rgb(255, 200, 100), // Light orange
                background_color: Color::Rgb(40, 25, 15), // Dark brown-gray
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 200, 50), // Bright yellow
                success_color: Color::Rgb(100, 255, 150), // Light green
                info_color: Color::Rgb(100, 180, 255), // Bright blue
                muted_color: Color::Rgb(120, 80, 50), // Brown-gray
            },
            ThemePalette::Red => Theme {
                name: "Red",
                logo_color: Color::Rgb(255, 80, 80), // Bright red
                border_color: Color::Rgb(200, 50, 50), // Dark red
                title_color: Color::Rgb(255, 150, 150), // Light red
                text_color: Color::Rgb(255, 220, 220), // Very light red
                selected_color: Color::Rgb(255, 150, 150), // Light red
                background_color: Color::Rgb(25, 10, 10), // Very dark red-gray
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 200, 100), // Light yellow
                success_color: Color::Rgb(100, 255, 150), // Light green
                info_color: Color::Rgb(150, 150, 255), // Light purple-blue
                muted_color: Color::Rgb(100, 50, 50), // Red-gray
            },
            ThemePalette::Monochrome => Theme {
                name: "Monochrome",
                logo_color: Color::Rgb(200, 200, 200), // Light gray
                border_color: Color::Rgb(150, 150, 150), // Medium gray
                title_color: Color::Rgb(255, 255, 255), // White
                text_color: Color::Rgb(220, 220, 220), // Light gray
                selected_color: Color::Rgb(255, 255, 255), // White
                background_color: Color::Rgb(0, 0, 0), // Black
                error_color: Color::Rgb(200, 200, 200), // Light gray
                warning_color: Color::Rgb(180, 180, 180), // Medium-light gray
                success_color: Color::Rgb(220, 220, 220), // Light gray
                info_color: Color::Rgb(200, 200, 200), // Light gray
                muted_color: Color::Rgb(100, 100, 100), // Medium gray
            },
            ThemePalette::Cyberpunk => Theme {
                name: "Cyberpunk",
                logo_color: Color::Rgb(0, 255, 255), // Cyan
                border_color: Color::Rgb(255, 0, 255), // Magenta
                title_color: Color::Rgb(0, 255, 255), // Bright cyan
                text_color: Color::Rgb(255, 255, 255), // White
                selected_color: Color::Rgb(255, 0, 255), // Magenta
                background_color: Color::Rgb(10, 5, 20), // Very dark purple
                error_color: Color::Rgb(255, 0, 100), // Bright pink-red
                warning_color: Color::Rgb(255, 200, 0), // Bright yellow
                success_color: Color::Rgb(0, 255, 150), // Bright cyan-green
                info_color: Color::Rgb(0, 200, 255), // Bright cyan-blue
                muted_color: Color::Rgb(80, 40, 100), // Dark purple-gray
            },
            ThemePalette::Forest => Theme {
                name: "Forest",
                logo_color: Color::Rgb(100, 200, 100), // Forest green
                border_color: Color::Rgb(80, 150, 80), // Dark forest green
                title_color: Color::Rgb(150, 255, 150), // Light green
                text_color: Color::Rgb(200, 255, 200), // Very light green
                selected_color: Color::Rgb(150, 255, 150), // Light green
                background_color: Color::Rgb(15, 25, 15), // Very dark green-gray
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 220, 100), // Light yellow
                success_color: Color::Rgb(100, 255, 100), // Bright green
                info_color: Color::Rgb(100, 200, 255), // Bright blue
                muted_color: Color::Rgb(60, 100, 60), // Green-gray
            },
            ThemePalette::Ocean => Theme {
                name: "Ocean",
                logo_color: Color::Rgb(64, 224, 208), // Turquoise
                border_color: Color::Rgb(0, 191, 255), // Deep sky blue
                title_color: Color::Rgb(135, 206, 250), // Light sky blue
                text_color: Color::Rgb(200, 230, 255), // Very light blue
                selected_color: Color::Rgb(135, 206, 250), // Light sky blue
                background_color: Color::Rgb(5, 15, 30), // Very dark blue
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 220, 100), // Light yellow
                success_color: Color::Rgb(100, 255, 200), // Light cyan-green
                info_color: Color::Rgb(100, 200, 255), // Bright blue
                muted_color: Color::Rgb(50, 100, 130), // Blue-gray
            },
            ThemePalette::Sunset => Theme {
                name: "Sunset",
                logo_color: Color::Rgb(255, 140, 0), // Dark orange
                border_color: Color::Rgb(255, 165, 0), // Orange
                title_color: Color::Rgb(255, 200, 100), // Light orange
                text_color: Color::Rgb(255, 240, 200), // Very light orange
                selected_color: Color::Rgb(255, 200, 100), // Light orange
                background_color: Color::Rgb(30, 15, 25), // Dark purple-red
                error_color: Color::Rgb(255, 100, 100), // Light red
                warning_color: Color::Rgb(255, 180, 80), // Orange-yellow
                success_color: Color::Rgb(150, 255, 100), // Light green
                info_color: Color::Rgb(255, 150, 150), // Light pink
                muted_color: Color::Rgb(120, 70, 80), // Purple-red-gray
            },
        }
    }
}

