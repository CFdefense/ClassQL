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
                logo_color: Color::Rgb(30, 30, 150), // dark blue
                border_color: Color::Red, // red
                title_color: Color::Rgb(30, 30, 150), // dark blue
                text_color: Color::Rgb(20, 20, 20), // very dark gray / black
                selected_color: Color::Red, // red
                background_color: Color::Rgb(255, 255, 255), // white
                error_color: Color::Rgb(200, 50, 50), // red
                warning_color: Color::Rgb(200, 150, 50), // orange
                success_color: Color::Rgb(50, 150, 50), // green
                info_color: Color::Rgb(50, 100, 200), // blue
                muted_color: Color::Rgb(120, 120, 120), // medium gray
            },
            ThemePalette::Dark => Theme {
                name: "Dark",
                logo_color: Color::Rgb(200, 200, 200), // light gray
                border_color: Color::Rgb(100, 100, 100), // medium gray
                title_color: Color::Rgb(255, 255, 255), // white
                text_color: Color::Rgb(220, 220, 220), // light gray
                selected_color: Color::Rgb(100, 150, 255), // bright blue for selection
                background_color: Color::Rgb(18, 18, 18), // very dark gray, almost black
                error_color: Color::Rgb(255, 85, 85), // light red
                warning_color: Color::Rgb(255, 184, 108), // light orange
                success_color: Color::Rgb(85, 255, 85), // light green
                info_color: Color::Rgb(100, 150, 255), // bright blue
                muted_color: Color::Rgb(100, 100, 100), // medium gray
            },
            ThemePalette::Pastel => Theme {
                name: "Pastel",
                logo_color: Color::Rgb(135, 206, 235), // sky blue
                border_color: Color::Rgb(200, 180, 220), // soft purple
                title_color: Color::Rgb(150, 150, 200), // soft blue
                text_color: Color::Rgb(60, 60, 80), // dark gray
                selected_color: Color::Rgb(150, 200, 255), // soft blue
                background_color: Color::Rgb(250, 248, 255), // very light purple/white
                error_color: Color::Rgb(255, 150, 150), // soft red
                warning_color: Color::Rgb(255, 220, 150), // soft yellow
                success_color: Color::Rgb(150, 255, 200), // soft green
                info_color: Color::Rgb(150, 200, 255), // soft blue
                muted_color: Color::Rgb(180, 180, 200), // soft gray
            },
            ThemePalette::Blue => Theme {
                name: "Blue",
                logo_color: Color::Rgb(100, 150, 255), // bright blue
                border_color: Color::Rgb(100, 150, 255), // bright blue
                title_color: Color::Rgb(150, 200, 255), // light blue
                text_color: Color::Rgb(200, 220, 255), // very light blue
                selected_color: Color::Rgb(150, 200, 255), // light blue
                background_color: Color::Rgb(20, 30, 50), // dark blue-gray
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 220, 100), // light yellow
                success_color: Color::Rgb(100, 255, 150), // light green
                info_color: Color::Rgb(100, 180, 255), // bright blue
                muted_color: Color::Rgb(80, 100, 130), // blue-gray
            },
            ThemePalette::Green => Theme {
                name: "Green",
                logo_color: Color::Rgb(100, 255, 150), // bright green
                border_color: Color::Rgb(100, 255, 150), // bright green
                title_color: Color::Rgb(150, 255, 200), // light green
                text_color: Color::Rgb(200, 255, 220), // very light green
                selected_color: Color::Rgb(150, 255, 200), // light green
                background_color: Color::Rgb(20, 40, 25), // dark green-gray
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 220, 100), // light yellow
                success_color: Color::Rgb(100, 255, 150), // bright green
                info_color: Color::Rgb(100, 200, 255), // bright blue
                muted_color: Color::Rgb(60, 100, 70), // green-gray
            },
            ThemePalette::Purple => Theme {
                name: "Purple",
                logo_color: Color::Rgb(200, 100, 255), // bright purple
                border_color: Color::Rgb(200, 100, 255), // bright purple
                title_color: Color::Rgb(220, 150, 255), // light purple
                text_color: Color::Rgb(240, 200, 255), // very light purple
                selected_color: Color::Rgb(220, 150, 255), // light purple
                background_color: Color::Rgb(30, 20, 40), // dark purple-gray
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 220, 100), // light yellow
                success_color: Color::Rgb(150, 255, 100), // light green
                info_color: Color::Rgb(150, 150, 255), // light purple-blue
                muted_color: Color::Rgb(100, 70, 120), // purple-gray
            },
            ThemePalette::Orange => Theme {
                name: "Orange",
                logo_color: Color::Rgb(255, 165, 0), // bright orange
                border_color: Color::Rgb(255, 140, 0), // dark orange
                title_color: Color::Rgb(255, 200, 100), // light orange
                text_color: Color::Rgb(255, 240, 220), // very light orange
                selected_color: Color::Rgb(255, 200, 100), // light orange
                background_color: Color::Rgb(40, 25, 15), // dark brown-gray
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 200, 50), // bright yellow
                success_color: Color::Rgb(100, 255, 150), // light green
                info_color: Color::Rgb(100, 180, 255), // bright blue
                muted_color: Color::Rgb(120, 80, 50), // brown-gray
            },
            ThemePalette::Red => Theme {
                name: "Red",
                logo_color: Color::Rgb(255, 80, 80), // bright red
                border_color: Color::Rgb(200, 50, 50), // dark red
                title_color: Color::Rgb(255, 150, 150), // light red
                text_color: Color::Rgb(255, 220, 220), // very light red
                selected_color: Color::Rgb(255, 150, 150), // light red
                background_color: Color::Rgb(25, 10, 10), // very dark red-gray
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 200, 100), // light yellow
                success_color: Color::Rgb(100, 255, 150), // light green
                info_color: Color::Rgb(150, 150, 255), // light purple-blue
                muted_color: Color::Rgb(100, 50, 50), // red-gray
            },
            ThemePalette::Monochrome => Theme {
                name: "Monochrome",
                logo_color: Color::Rgb(200, 200, 200), // light gray
                border_color: Color::Rgb(150, 150, 150), // medium gray
                title_color: Color::Rgb(255, 255, 255), // white
                text_color: Color::Rgb(220, 220, 220), // light gray
                selected_color: Color::Rgb(255, 255, 255), // white
                background_color: Color::Rgb(0, 0, 0), // black
                error_color: Color::Rgb(200, 200, 200), // light gray
                warning_color: Color::Rgb(180, 180, 180), // medium-light gray
                success_color: Color::Rgb(220, 220, 220), // light gray
                info_color: Color::Rgb(200, 200, 200), // light gray
                muted_color: Color::Rgb(100, 100, 100), // medium gray
            },
            ThemePalette::Cyberpunk => Theme {
                name: "Cyberpunk",
                logo_color: Color::Rgb(0, 255, 255), // cyan
                border_color: Color::Rgb(255, 0, 255), // magenta
                title_color: Color::Rgb(0, 255, 255), // bright cyan
                text_color: Color::Rgb(255, 255, 255), // white
                selected_color: Color::Rgb(255, 0, 255), // magenta
                background_color: Color::Rgb(10, 5, 20), // very dark purple
                error_color: Color::Rgb(255, 0, 100), // bright pink-red
                warning_color: Color::Rgb(255, 200, 0), // bright yellow
                success_color: Color::Rgb(0, 255, 150), // bright cyan-green
                info_color: Color::Rgb(0, 200, 255), // bright cyan-blue
                muted_color: Color::Rgb(80, 40, 100), // dark purple-gray
            },
            ThemePalette::Forest => Theme {
                name: "Forest",
                logo_color: Color::Rgb(100, 200, 100), // forest green
                border_color: Color::Rgb(80, 150, 80), // dark forest green
                title_color: Color::Rgb(150, 255, 150), // light green
                text_color: Color::Rgb(200, 255, 200), // very light green
                selected_color: Color::Rgb(150, 255, 150), // light green
                background_color: Color::Rgb(15, 25, 15), // very dark green-gray
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 220, 100), // light yellow
                success_color: Color::Rgb(100, 255, 100), // bright green
                info_color: Color::Rgb(100, 200, 255), // bright blue
                muted_color: Color::Rgb(60, 100, 60), // green-gray
            },
            ThemePalette::Ocean => Theme {
                name: "Ocean",
                logo_color: Color::Rgb(64, 224, 208), // turquoise
                border_color: Color::Rgb(0, 191, 255), // deep sky blue
                title_color: Color::Rgb(135, 206, 250), // light sky blue
                text_color: Color::Rgb(200, 230, 255), // very light blue
                selected_color: Color::Rgb(135, 206, 250), // light sky blue
                background_color: Color::Rgb(5, 15, 30), // very dark blue
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 220, 100), // light yellow
                success_color: Color::Rgb(100, 255, 200), // light cyan-green
                info_color: Color::Rgb(100, 200, 255), // bright blue
                muted_color: Color::Rgb(50, 100, 130), // blue-gray
            },
            ThemePalette::Sunset => Theme {
                name: "Sunset",
                logo_color: Color::Rgb(255, 140, 0), // dark orange
                border_color: Color::Rgb(255, 165, 0), // orange
                title_color: Color::Rgb(255, 200, 100), // light orange
                text_color: Color::Rgb(255, 240, 200), // very light orange
                selected_color: Color::Rgb(255, 200, 100), // light orange
                background_color: Color::Rgb(30, 15, 25), // dark purple-red
                error_color: Color::Rgb(255, 100, 100), // light red
                warning_color: Color::Rgb(255, 180, 80), // orange-yellow
                success_color: Color::Rgb(150, 255, 100), // light green
                info_color: Color::Rgb(255, 150, 150), // light pink
                muted_color: Color::Rgb(120, 70, 80), // purple-red-gray
            },
        }
    }
}

