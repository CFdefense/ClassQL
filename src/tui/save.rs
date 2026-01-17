/// src/tui/save.rs
///
/// Schedule save/load functionality
///
/// Handles saving and loading schedules to/from .sav files
use crate::data::sql::{self, Class};
use std::fs;
use std::path::{Path, PathBuf};

/// Saved schedule information
/// 
/// Fields:
/// --- ---
/// name -> Name of the schedule
/// timestamp -> Timestamp of the schedule
/// school_id -> School ID the schedule belongs to
/// term_id -> Term ID the schedule belongs to
/// classes -> Classes in the schedule
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for SavedSchedule
/// Clone -> Clone trait for SavedSchedule
/// --- ---
///
#[derive(Debug, Clone)]
pub struct SavedSchedule {
    pub name: String,
    pub timestamp: u64,
    pub school_id: Option<String>,
    pub term_id: Option<String>,
    pub classes: Vec<Class>,
}

/// Get the save directory path (current working directory/save)
///
/// Parameters:
/// --- ---
/// None
/// --- ---
///
/// Returns:
/// --- ---
/// Result<PathBuf, String> -> Path to the save directory or error
/// --- ---
///
fn get_save_dir() -> Result<PathBuf, String> {
    // try CARGO_MANIFEST_DIR first (for development), then fall back to current working directory
    let base_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
    };
    let save_dir = base_dir.join("save");
    Ok(save_dir)
}

/// Ensure the save directory exists
///
/// Parameters:
/// --- ---
/// None
/// --- ---
///
/// Returns:
/// --- ---
/// Result<PathBuf, String> -> Path to the save directory or error
/// --- ---
///
fn ensure_save_dir() -> Result<PathBuf, String> {
    let save_dir = get_save_dir()?;
    fs::create_dir_all(&save_dir)
        .map_err(|e| format!("Failed to create save directory: {}", e))?;
    Ok(save_dir)
}

/// Save a schedule to a .sav file
///
/// Parameters:
/// --- ---
/// name -> Name of the schedule
/// school_id -> School ID for the schedule
/// term_id -> Term ID for the schedule
/// classes -> Classes in the schedule
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), String> -> Success or error message
/// --- ---
///
pub fn save_schedule(
    name: &str,
    school_id: Option<&str>,
    term_id: Option<&str>,
    classes: &[Class],
) -> Result<(), String> {
    let save_dir = ensure_save_dir()?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs();
    
    let filename = format!("{}.sav", timestamp);
    let file_path = save_dir.join(&filename);
    
    // format:
    // line 1: name
    // line 2: school_id (or empty)
    // line 3: term_id (or empty)
    // remaining lines: class IDs (one per line)
    let mut content = format!("{}\n", name);
    content.push_str(&format!("{}\n", school_id.unwrap_or("")));
    content.push_str(&format!("{}\n", term_id.unwrap_or("")));
    for class in classes {
        content.push_str(&format!("{}\n", class.unique_id()));
    }
    
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write save file: {}", e))?;
    
    Ok(())
}

/// Load all saved schedules
///
/// Parameters:
/// --- ---
/// None
/// --- ---
///
/// Returns:
/// --- ---
/// Result<Vec<SavedSchedule>, String> -> List of saved schedules or error
/// --- ---
///
pub fn load_all_schedules() -> Result<Vec<SavedSchedule>, String> {
    let save_dir = get_save_dir()?;
    
    if !save_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut saved_schedules = Vec::new();
    
    let entries = fs::read_dir(&save_dir)
        .map_err(|e| format!("Failed to read save directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("sav") {
            if let Ok(schedule) = load_schedule(&path) {
                saved_schedules.push(schedule);
            }
        }
    }
    
    // sort by timestamp (newest first)
    saved_schedules.sort_by_key(|s| std::cmp::Reverse(s.timestamp));
    
    Ok(saved_schedules)
}

/// Load a single schedule from a file
///
/// Parameters:
/// --- ---
/// file_path -> Path to the .sav file
/// --- ---
///
/// Returns:
/// --- ---
/// Result<SavedSchedule, String> -> The saved schedule or error
/// --- ---
///
fn load_schedule(file_path: &Path) -> Result<SavedSchedule, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read save file: {}", e))?;
    
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Err("Empty save file".to_string());
    }
    
    let name = lines[0].to_string();
    
    // extract timestamp from filename (e.g., "1234567890.sav" -> 1234567890)
    let filename = file_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;
    let timestamp = filename.parse::<u64>()
        .map_err(|_| "Invalid timestamp in filename".to_string())?;
    
    // format: line 1 = name, line 2 = school_id, line 3 = term_id, rest = class IDs
    if lines.len() < 3 {
        return Err("Invalid save file format".to_string());
    }
    
    let school_id_str = lines[1];
    let term_id_str = lines[2];
    let school_id = if school_id_str.is_empty() { None } else { Some(school_id_str.to_string()) };
    let term_id = if term_id_str.is_empty() { None } else { Some(term_id_str.to_string()) };
    let class_ids: Vec<&str> = lines.iter().skip(3).filter(|line| !line.is_empty()).copied().collect();
    
    // load classes from database by their unique IDs
    let mut classes = Vec::new();
    if !class_ids.is_empty() {
        // use test db if school_id is "_test", otherwise use synced db or default
        let db_path = if school_id.as_deref() == Some("_test") {
            sql::get_test_db_path()
        } else {
            sql::get_default_db_path()
        };
        
        // build SQL query to get classes by their unique IDs
        // unique_id format is "SUBJECT:COURSE-SECTION"
        let mut conditions = Vec::new();
        
        for class_id in &class_ids {
            // parse the unique_id format: "SUBJECT:COURSE-SECTION"
            let parts: Vec<&str> = class_id.split(':').collect();
            if parts.len() == 2 {
                let subject = parts[0];
                let rest: Vec<&str> = parts[1].split('-').collect();
                if rest.len() == 2 {
                    let course = rest[0];
                    let section = rest[1];
                    
                    // escape single quotes in values (SQL injection protection)
                    let subject_escaped = subject.replace("'", "''");
                    let course_escaped = course.replace("'", "''");
                    let section_escaped = section.replace("'", "''");
                    
                    // use table aliases to avoid ambiguous column names
                    // s = sections, c = courses
                    conditions.push(format!(
                        "(s.subject_code = '{}' AND s.course_number = '{}' AND s.sequence = '{}')",
                        subject_escaped, course_escaped, section_escaped
                    ));
                }
            }
        }
        
        if !conditions.is_empty() {
            // build additional filters for school and term
            let mut filters = Vec::new();
            if let Some(ref sid) = school_id {
                if sid != "_test" {
                    filters.push(format!("s.school_id = '{}'", sid.replace("'", "''")));
                }
            }
            if let Some(ref tid) = term_id {
                filters.push(format!("s.term_collection_id = '{}'", tid.replace("'", "''")));
            }
            
            // combine class conditions with school/term filters
            let class_conditions = conditions.join(" OR ");
            let where_clause = if filters.is_empty() {
                class_conditions
            } else {
                format!("({}) AND {}", class_conditions, filters.join(" AND "))
            };
            
            // query sections table with joins
            let sql = format!(
                "SELECT \
                    c.subject_code, \
                    c.number AS course_number, \
                    c.title, \
                    c.description, \
                    c.credit_hours, \
                    c.prerequisites, \
                    c.corequisites, \
                    s.sequence AS section_sequence, \
                    s.max_enrollment, \
                    s.enrollment, \
                    s.instruction_method, \
                    s.campus, \
                    p.name AS professor_name, \
                    p.email_address AS professor_email, \
                    GROUP_CONCAT( \
                        (CASE WHEN mt.is_monday = 1 THEN 'M' ELSE '' END || \
                         CASE WHEN mt.is_tuesday = 1 THEN 'T' ELSE '' END || \
                         CASE WHEN mt.is_wednesday = 1 THEN 'W' ELSE '' END || \
                         CASE WHEN mt.is_thursday = 1 THEN 'TH' ELSE '' END || \
                         CASE WHEN mt.is_friday = 1 THEN 'F' ELSE '' END || \
                         CASE WHEN mt.is_saturday = 1 THEN 'S' ELSE '' END || \
                         CASE WHEN mt.is_sunday = 1 THEN 'SU' ELSE '' END) || \
                        ':' || mt.start_minutes || '-' || mt.end_minutes, \
                        '|' \
                    ) AS meeting_times, \
                    GROUP_CONCAT(DISTINCT mt.meeting_type) AS meeting_type, \
                    MAX(mt.is_monday) AS is_monday, \
                    MAX(mt.is_tuesday) AS is_tuesday, \
                    MAX(mt.is_wednesday) AS is_wednesday, \
                    MAX(mt.is_thursday) AS is_thursday, \
                    MAX(mt.is_friday) AS is_friday, \
                    MAX(mt.is_saturday) AS is_saturday, \
                    MAX(mt.is_sunday) AS is_sunday \
                FROM sections s \
                JOIN courses c ON s.school_id = c.school_id \
                    AND s.subject_code = c.subject_code \
                    AND s.course_number = c.number \
                LEFT JOIN professors p ON s.primary_professor_id = p.id \
                    AND s.school_id = p.school_id \
                LEFT JOIN meeting_times mt ON s.sequence = mt.section_sequence \
                    AND s.term_collection_id = mt.term_collection_id \
                    AND s.school_id = mt.school_id \
                    AND s.subject_code = mt.subject_code \
                    AND s.course_number = mt.course_number \
                WHERE {} \
                GROUP BY \
                    c.subject_code, \
                    c.number, \
                    c.title, \
                    c.description, \
                    c.credit_hours, \
                    c.prerequisites, \
                    c.corequisites, \
                    s.sequence, \
                    s.term_collection_id, \
                    s.school_id, \
                    s.max_enrollment, \
                    s.enrollment, \
                    s.instruction_method, \
                    s.campus, \
                    p.name, \
                    p.email_address",
                where_clause
            );
            
            match sql::execute_query(&sql, &db_path) {
                Ok(loaded_classes) => {
                    // create a map for quick lookup
                    let mut class_map: std::collections::HashMap<String, Class> = loaded_classes
                        .into_iter()
                        .map(|c| (c.unique_id(), c))
                        .collect();
                    
                    // add classes in the order they appear in the save file
                    for class_id in class_ids {
                        if let Some(class) = class_map.remove(class_id) {
                            classes.push(class);
                        }
                    }
                }
                Err(e) => {
                    // if query fails, return empty classes but don't fail the whole load
                    eprintln!("Warning: Failed to load classes from database: {}", e);
                }
            }
        }
    }
    
    Ok(SavedSchedule {
        name,
        timestamp,
        school_id,
        term_id,
        classes,
    })
}

/// Delete a saved schedule
///
/// Parameters:
/// --- ---
/// timestamp -> Timestamp of the schedule to delete
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), String> -> Success or error message
/// --- ---
///
pub fn delete_schedule(timestamp: u64) -> Result<(), String> {
    let save_dir = get_save_dir()?;
    let filename = format!("{}.sav", timestamp);
    let file_path = save_dir.join(&filename);
    
    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete save file: {}", e))?;
    }
    
    Ok(())
}
