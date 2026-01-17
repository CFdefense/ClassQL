/*
    src/data/sql.rs

    For sql code execution - contains the Class struct and query execution logic
*/
use rusqlite::Connection;
use std::path::Path;
use crate::data::sync::get_synced_db_path;
use crate::tui::widgets::helpers::{format_day_for_display, get_day_order};

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
/// meeting_type -> Type of meeting (e.g., "Lecture", "Lab")
/// days -> Days the class meets (formatted string like "MWF" or "TTH")
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
    pub meeting_type: Option<String>,
    pub days: String,
    pub meeting_times: Option<String>,
}

impl Class {
    /// Get a unique identifier for this class
    ///
    /// Parameters:
    /// --- ---
    /// self -> The class instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// String -> Unique identifier combining subject_code, course_number, and section_sequence
    /// --- ---
    ///
    pub fn unique_id(&self) -> String {
        format!(
            "{}:{}-{}",
            self.subject_code, self.course_number, self.section_sequence
        )
    }

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

        // line 1: course code (e.g., "CS 101-001")
        lines.push(format!(
            "{} {}-{}",
            self.subject_code, self.course_number, self.section_sequence
        ));

        // line 2: title (truncated to ~25 chars)
        let title = if self.title.len() > 25 {
            format!("{}...", &self.title[..22])
        } else {
            self.title.clone()
        };
        lines.push(title);

        // line 3: professor
        let prof = self.professor_name.as_deref().unwrap_or("TBA");
        let prof_display = if prof.len() > 20 {
            format!("{}...", &prof[..17])
        } else {
            prof.to_string()
        };
        lines.push(prof_display);

        // line 4: days and time
        let time_str = if let Some(meeting_times_str) = &self.meeting_times {
            // parse meeting times: "M:08:00:00-10:45:00|TH:08:00:00-09:15:00"
            let mut time_parts: Vec<(u8, String)> = Vec::new(); // (day_order, formatted_string)
            for mt in meeting_times_str.split('|') {
                if let Some(colon_pos) = mt.find(':') {
                    let days_part = &mt[..colon_pos];
                    let time_part = &mt[colon_pos + 1..];
                    if let Some(dash_pos) = time_part.find('-') {
                        let start = format_time_short(&time_part[..dash_pos]);
                        let end = format_time_short(&time_part[dash_pos + 1..]);
                        if !days_part.is_empty() {
                            // get the first day code for sorting (in case of multiple days like "MW")
                            let first_day = if days_part.starts_with("TH") {
                                "TH"
                            } else if days_part.starts_with("SU") {
                                "SU"
                            } else if days_part.len() > 0 {
                                &days_part[..1]
                            } else {
                                days_part
                            };
                            let day_order = get_day_order(first_day);
                            // format day code for display (add space after single letters)
                            let formatted_days = format_day_for_display(days_part);
                            time_parts
                                .push((day_order, format!("{} {}-{}", formatted_days, start, end)));
                        }
                    }
                }
            }

            // sort by day order (Monday first)
            time_parts.sort_by_key(|(day_order, _)| *day_order);

            if time_parts.is_empty() {
                format!("{} TBA", self.days)
            } else {
                // show all parsed meeting times with their days (already sorted)
                time_parts
                    .iter()
                    .map(|(_, s)| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        } else {
            // no meeting times available
            format!("{} TBA", self.days)
        };
        lines.push(time_str);

        // line 5: enrollment
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

/// Format days from boolean flags into a compact string like "MWF" or "TTH"
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
/// String -> Compact day string (e.g., "MWF", "TTH", "TBA")
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
    // build days in order (Monday first)
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
        days.push_str("TH");
    }
    if is_friday {
        days.push('F');
    }
    if is_saturday {
        days.push('S');
    }
    if is_sunday {
        days.push_str("SU");
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
    // connect to the database
    let conn =
        Connection::open(db_path).map_err(|e| format!("Database connection error: {}", e))?;

    // prepare and execute the statement
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQL preparation error: {}", e))?;

    // execute query and map results to Class structs
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
                meeting_type: row.get(15).ok(),
                days: format_days(
                    row.get::<_, i32>(16).unwrap_or(0) == 1,
                    row.get::<_, i32>(17).unwrap_or(0) == 1,
                    row.get::<_, i32>(18).unwrap_or(0) == 1,
                    row.get::<_, i32>(19).unwrap_or(0) == 1,
                    row.get::<_, i32>(20).unwrap_or(0) == 1,
                    row.get::<_, i32>(21).unwrap_or(0) == 1,
                    row.get::<_, i32>(22).unwrap_or(0) == 1,
                ),
                meeting_times: row.get(14).ok(), // meeting_times is column 14
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?;

    // collect results
    let mut classes = Vec::new();
    for class_result in class_iter {
        match class_result {
            Ok(class) => classes.push(class),
            Err(e) => return Err(format!("Error reading row: {}", e)),
        }
    }

    Ok(classes)
}

/// School struct for representing available schools
///
/// Fields:
/// --- ---
/// id -> School identifier
/// name -> School display name
/// --- ---
#[derive(Debug, Clone)]
pub struct School {
    pub id: String,
    pub name: String,
}

/// Term struct for representing available terms
///
/// Fields:
/// --- ---
/// id -> Term collection identifier
/// school_id -> School identifier
/// name -> Term display name (e.g., "2025 Fall")
/// year -> Term year
/// season -> Term season (Spring, Fall, Summer, Winter)
/// --- ---
#[derive(Debug, Clone)]
pub struct Term {
    pub id: String,
    pub school_id: String,
    pub name: String,
    pub year: i32,
    pub season: String,
}

/// Fetch all available schools from the synced database
///
/// Parameters:
/// --- ---
/// db_path -> Path to the SQLite database file
/// --- ---
///
/// Returns:
/// --- ---
/// Result<Vec<School>, String> -> Vector of schools or error message
/// --- ---
pub fn fetch_schools(db_path: &Path) -> Result<Vec<School>, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Database connection error: {}", e))?;
    
    let mut stmt = conn
        .prepare("SELECT id, name FROM schools ORDER BY name")
        .map_err(|e| format!("SQL preparation error: {}", e))?;
    
    let school_iter = stmt
        .query_map([], |row| {
            Ok(School {
                id: row.get(0).unwrap_or_default(),
                name: row.get(1).unwrap_or_default(),
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?;
    
    let mut schools = Vec::new();
    for school_result in school_iter {
        if let Ok(school) = school_result {
            schools.push(school);
        }
    }
    
    Ok(schools)
}

/// Fetch all available terms for a school from the synced database
///
/// Parameters:
/// --- ---
/// db_path -> Path to the SQLite database file
/// school_id -> The school ID to filter terms by
/// --- ---
///
/// Returns:
/// --- ---
/// Result<Vec<Term>, String> -> Vector of terms or error message
/// --- ---
pub fn fetch_terms(db_path: &Path, school_id: &str) -> Result<Vec<Term>, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Database connection error: {}", e))?;
    
    let mut stmt = conn
        .prepare("SELECT id, school_id, name, year, season FROM term_collections WHERE school_id = ? ORDER BY year DESC, season")
        .map_err(|e| format!("SQL preparation error: {}", e))?;
    
    let term_iter = stmt
        .query_map([school_id], |row| {
            Ok(Term {
                id: row.get(0).unwrap_or_default(),
                school_id: row.get(1).unwrap_or_default(),
                name: row.get(2).unwrap_or_default(),
                year: row.get(3).unwrap_or(0),
                season: row.get(4).unwrap_or_default(),
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?;
    
    let mut terms = Vec::new();
    for term_result in term_iter {
        if let Ok(term) = term_result {
            terms.push(term);
        }
    }
    
    Ok(terms)
}

/// Get the last sync timestamp from the synced database
///
/// Parameters:
/// --- ---
/// db_path -> Path to the SQLite database file
/// --- ---
///
/// Returns:
/// --- ---
/// Option<String> -> Last sync timestamp or None if never synced
/// --- ---
pub fn get_last_sync_time(db_path: &Path) -> Option<String> {
    let conn = Connection::open(db_path).ok()?;
    
    let result: Result<String, _> = conn.query_row(
        "SELECT created_at FROM _previous_all_collections ORDER BY synced_at DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    
    result.ok()
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
    // prioritize synced database from classy directory
    let synced_db = get_synced_db_path();
    if synced_db.exists() {
        return synced_db;
    }
    
    // fallback to test database location
    get_test_db_path()
}

/// Get the path to the test database
///
/// Returns:
/// --- ---
/// PathBuf -> Path to the test database file (classy/test.db)
/// --- ---
///
pub fn get_test_db_path() -> std::path::PathBuf {
    let base_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd
    } else {
        std::path::PathBuf::from(".")
    };
    base_dir.join("classy").join("test.db")
}
