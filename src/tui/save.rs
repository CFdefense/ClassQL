/// src/tui/save.rs
///
/// Schedule save/load functionality
///
/// Handles saving and loading schedules to/from .sav files
use crate::data::sql::Class;
use std::fs;
use std::path::{Path, PathBuf};

/// Saved schedule information
/// 
/// Fields:
/// --- ---
/// name -> Name of the schedule
/// timestamp -> Timestamp of the schedule
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
/// classes -> Classes in the schedule
/// --- ---
///
/// Returns:
/// --- ---
/// Result<(), String> -> Success or error message
/// --- ---
///
pub fn save_schedule(name: &str, classes: &[Class]) -> Result<(), String> {
    let save_dir = ensure_save_dir()?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs();
    
    let filename = format!("{}.sav", timestamp);
    let file_path = save_dir.join(&filename);
    
    // format: name on first line, then class IDs (one per line)
    let mut content = format!("{}\n", name);
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
/// cart_classes -> Map of all available classes (to look up class objects)
/// --- ---
///
/// Returns:
/// --- ---
/// Result<Vec<SavedSchedule>, String> -> List of saved schedules or error
/// --- ---
///
pub fn load_all_schedules(cart_classes: &std::collections::HashMap<String, Class>) -> Result<Vec<SavedSchedule>, String> {
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
            if let Ok(schedule) = load_schedule(&path, cart_classes) {
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
/// cart_classes -> Map of all available classes (to look up class objects)
/// --- ---
///
/// Returns:
/// --- ---
/// Result<SavedSchedule, String> -> The saved schedule or error
/// --- ---
///
fn load_schedule(file_path: &Path, cart_classes: &std::collections::HashMap<String, Class>) -> Result<SavedSchedule, String> {
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
    
    // load classes from IDs
    let mut classes = Vec::new();
    for line in lines.iter().skip(1) {
        if !line.is_empty() {
            if let Some(class) = cart_classes.get(*line) {
                classes.push(class.clone());
            }
        }
    }
    
    Ok(SavedSchedule {
        name,
        timestamp,
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
