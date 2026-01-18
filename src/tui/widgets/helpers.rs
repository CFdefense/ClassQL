/// src/tui/widgets/helpers.rs
///
/// Helper functions for widget rendering
///
/// Contains utility functions for formatting and processing data
/// Get day order for sorting (Monday = 0, Sunday = 6)
///
/// Parameters:
/// --- ---
/// day_code -> Day code string (M, T, W, TH, F, S, SU)
/// --- ---
///
/// Returns:
/// --- ---
/// u8 -> Day order (0-6 for valid days, 99 for unknown)
/// --- ---
///
pub fn get_day_order(day_code: &str) -> u8 {
    match day_code {
        "M" => 0,  // monday
        "T" => 1,  // Tuesday
        "W" => 2,  // wednesday
        "TH" => 3, // thursday
        "F" => 4,  // friday
        "S" => 5,  // saturday
        "SU" => 6, // sunday
        _ => 99,   // unknown days go last
    }
}

/// Format day code for display (add space after single-letter codes)
///
/// Parameters:
/// --- ---
/// day_code -> Day code string (M, T, W, TH, F, S, SU)
/// --- ---
///
/// Returns:
/// --- ---
/// String -> Formatted day code with space padding for alignment
/// --- ---
///
pub fn format_day_for_display(day_code: &str) -> String {
    // check if it's a single letter (not TH, SU, etc.)
    if day_code.len() == 1 {
        format!("{} ", day_code) // add space after single letter
    } else {
        day_code.to_string() // keep multi-letter codes as-is
    }
}
