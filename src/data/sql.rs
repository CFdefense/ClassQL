/*
    src/data/sql.rs

    For sql code execution - contains the Class struct and query execution logic
*/

use rusqlite::Connection;
use std::path::Path;

/// Class struct
///
/// Represents a class/section returned from a database query
/// Contains all relevant information about a course section
///
/// Class fields:
/// --- ---
/// subject_code -> Subject code (e.g., "CS", "MATH")
/// course_number -> Course number (e.g., "101", "201")
/// title -> Course title
/// description -> Course description
/// credit_hours -> Number of credit hours
/// prerequisites -> Prerequisites text
/// corequisites -> Corequisites text
/// section_sequence -> Section identifier (e.g., "001", "002")
/// max_enrollment -> Maximum enrollment capacity
/// enrollment -> Current enrollment count
/// instruction_method -> Instruction method (e.g., "In Person", "Online")
/// campus -> Campus location
/// professor_name -> Professor's name
/// professor_email -> Professor's email address
/// start_time -> Class start time
/// end_time -> Class end time
/// meeting_type -> Type of meeting (e.g., "Lecture", "Lab")
/// days -> Days the class meets (formatted string like "MWF" or "TR")
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for Class
/// Clone -> Clone trait for Class
/// Default -> Default trait for Class
/// --- ---
///
#[derive(Debug, Clone, Default)]
pub struct Class {
    pub subject_code: String,
    pub course_number: String,
    pub title: String,
    pub description: Option<String>,
    pub credit_hours: f64,
    pub prerequisites: Option<String>,
    pub corequisites: Option<String>,
    pub section_sequence: String,
    pub max_enrollment: Option<i32>,
    pub enrollment: Option<i32>,
    pub instruction_method: Option<String>,
    pub campus: Option<String>,
    pub professor_name: Option<String>,
    pub professor_email: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub meeting_type: Option<String>,
    pub days: String,
}

impl Class {
    /// Format the class for display in a table cell
    ///
    /// Parameters:
    /// --- ---
    /// self -> The class instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Vec<String> -> Multi-line display with course code, title, professor, days/time, enrollment
    /// --- ---
    ///
    pub fn format_for_display(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Line 1: Course code (e.g., "CS 101-001")
        lines.push(format!(
            "{} {}-{}",
            self.subject_code, self.course_number, self.section_sequence
        ));

        // Line 2: Title (truncated to ~25 chars)
        let title = if self.title.len() > 25 {
            format!("{}...", &self.title[..22])
        } else {
            self.title.clone()
        };
        lines.push(title);

        // Line 3: Professor
        let prof = self
            .professor_name
            .as_deref()
            .unwrap_or("TBA");
        let prof_display = if prof.len() > 20 {
            format!("{}...", &prof[..17])
        } else {
            prof.to_string()
        };
        lines.push(prof_display);

        // Line 4: Days and time
        let time_str = match (&self.start_time, &self.end_time) {
            (Some(start), Some(end)) => {
                let start_short = format_time_short(start);
                let end_short = format_time_short(end);
                format!("{} {}-{}", self.days, start_short, end_short)
            }
            _ => format!("{} TBA", self.days),
        };
        lines.push(time_str);

        // Line 5: Enrollment
        let enrollment_str = match (self.enrollment, self.max_enrollment) {
            (Some(e), Some(m)) => format!("{}/{} enrolled", e, m),
            _ => String::new(),
        };
        if !enrollment_str.is_empty() {
            lines.push(enrollment_str);
        }

        lines
    }
}

/// Format time from "HH:MM:SS" to "H:MMam/pm"
///
/// Parameters:
/// --- ---
/// time -> Time string in "HH:MM:SS" format
/// --- ---
///
/// Returns:
/// --- ---
/// String -> Formatted time string (e.g., "9:00am", "2:30pm")
/// --- ---
///
fn format_time_short(time: &str) -> String {
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
}

/// Format days from boolean flags into a compact string like "MWF" or "TR"
///
/// Parameters:
/// --- ---
/// is_monday -> Whether class meets on Monday
/// is_tuesday -> Whether class meets on Tuesday
/// is_wednesday -> Whether class meets on Wednesday
/// is_thursday -> Whether class meets on Thursday
/// is_friday -> Whether class meets on Friday
/// is_saturday -> Whether class meets on Saturday
/// is_sunday -> Whether class meets on Sunday
/// --- ---
///
/// Returns:
/// --- ---
/// String -> Compact day string (e.g., "MWF", "TR", "TBA")
/// --- ---
///
fn format_days(
    is_monday: bool,
    is_tuesday: bool,
    is_wednesday: bool,
    is_thursday: bool,
    is_friday: bool,
    is_saturday: bool,
    is_sunday: bool,
) -> String {
    let mut days = String::new();
    if is_monday {
        days.push('M');
    }
    if is_tuesday {
        days.push('T');
    }
    if is_wednesday {
        days.push('W');
    }
    if is_thursday {
        days.push('R');
    }
    if is_friday {
        days.push('F');
    }
    if is_saturday {
        days.push('S');
    }
    if is_sunday {
        days.push('U');
    }
    if days.is_empty() {
        days = "TBA".to_string();
    }
    days
}

/// Execute a SQL query against the classes database and return Class results
///
/// Parameters:
/// --- ---
/// sql -> The SQL query string to execute
/// db_path -> Path to the SQLite database file
/// --- ---
///
/// Returns:
/// --- ---
/// Result<Vec<Class>, String> -> Vector of Class results or error message
/// --- ---
///
pub fn execute_query(sql: &str, db_path: &Path) -> Result<Vec<Class>, String> {
    // Connect to the database
    let conn = Connection::open(db_path).map_err(|e| format!("Database connection error: {}", e))?;

    // Prepare and execute the statement
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQL preparation error: {}", e))?;

    // Execute query and map results to Class structs
    let class_iter = stmt
        .query_map([], |row| {
            Ok(Class {
                subject_code: row.get(0).unwrap_or_default(),
                course_number: row.get(1).unwrap_or_default(),
                title: row.get(2).unwrap_or_default(),
                description: row.get(3).ok(),
                credit_hours: row.get(4).unwrap_or(0.0),
                prerequisites: row.get(5).ok(),
                corequisites: row.get(6).ok(),
                section_sequence: row.get(7).unwrap_or_default(),
                max_enrollment: row.get(8).ok(),
                enrollment: row.get(9).ok(),
                instruction_method: row.get(10).ok(),
                campus: row.get(11).ok(),
                professor_name: row.get(12).ok(),
                professor_email: row.get(13).ok(),
                start_time: row.get(14).ok(),
                end_time: row.get(15).ok(),
                meeting_type: row.get(16).ok(),
                days: format_days(
                    row.get::<_, i32>(17).unwrap_or(0) == 1,
                    row.get::<_, i32>(18).unwrap_or(0) == 1,
                    row.get::<_, i32>(19).unwrap_or(0) == 1,
                    row.get::<_, i32>(20).unwrap_or(0) == 1,
                    row.get::<_, i32>(21).unwrap_or(0) == 1,
                    row.get::<_, i32>(22).unwrap_or(0) == 1,
                    row.get::<_, i32>(23).unwrap_or(0) == 1,
                ),
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?;

    // Collect results
    let mut classes = Vec::new();
    for class_result in class_iter {
        match class_result {
            Ok(class) => classes.push(class),
            Err(e) => return Err(format!("Error reading row: {}", e)),
        }
    }

    Ok(classes)
}

/// Get the default database path
///
/// Parameters:
/// --- ---
/// None
/// --- ---
///
/// Returns:
/// --- ---
/// PathBuf -> Path to the default database file
/// --- ---
///
pub fn get_default_db_path() -> std::path::PathBuf {
    std::path::PathBuf::from("src/data/classes.db")
}
