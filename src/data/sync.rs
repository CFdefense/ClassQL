/*
    src/data/sync.rs

    Module for syncing class data from classy-sync
    Handles synchronization with the classy server and database management
*/

use std::path::PathBuf;
use std::fs;

use classy_sync::argument_parser::SyncResources;
use classy_sync::data_stores::replicate_datastore::Datastore;
use classy_sync::data_stores::sqlite::storage::Sqlite;
use classy_sync::data_stores::sync_requests::{AllSyncResult, SyncOptions};

/// Configuration for classy-sync
///
/// Fields:
/// --- ---
/// server_url -> URL of the classy server (default: http://localhost)
/// server_port -> Port of the classy server (default: from env or 8080)
/// db_path -> Path to store the synced database
/// --- ---
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub server_url: String,
    pub server_port: u16,
    pub db_path: PathBuf,
}

impl SyncConfig {
    /// Create a new SyncConfig from environment variables
    ///
    /// Returns:
    /// --- ---
    /// Result<Self, String> -> SyncConfig or error message
    /// --- ---
    pub fn from_env() -> Result<Self, String> {
        let server_url = std::env::var("CLASSY_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost".to_string());
        
        let server_port = std::env::var("CLASSY_SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "Invalid CLASSY_SERVER_PORT in .env file".to_string())?;

        // use /classy directory relative to cargo manifest directory for database storage
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let db_dir = manifest_dir.join("classy");
        fs::create_dir_all(&db_dir)
            .map_err(|e| format!("Failed to create classy directory: {}", e))?;
        
        let db_path = db_dir.join("classes.db");

        Ok(SyncConfig {
            server_url,
            server_port,
            db_path,
        })
    }

    /// Get the full server URL with port
    /// 
    /// Returns:
    /// --- ---
    /// String -> Full server URL with port
    /// --- ---
    pub fn server_url_with_port(&self) -> String {
        format!("{}:{}", self.server_url, self.server_port)
    }

    /// Get the sync API endpoint for "all" sync
    /// 
    /// Returns:
    /// --- ---
    /// String -> All sync endpoint URL
    /// --- ---
    pub fn all_sync_endpoint(&self) -> String {
        // allow override via CLASSY_SYNC_ENDPOINT environment variable
        std::env::var("CLASSY_SYNC_ENDPOINT")
            .unwrap_or_else(|_| format!("{}/sync/all", self.server_url_with_port()))
    }
}

/// Fetch sync data from the classy server
///
/// Parameters:
/// --- ---
/// endpoint -> The API endpoint URL
/// sync_options -> The sync options to send to the server
/// --- ---
///
/// Returns:
/// --- ---
/// Result<AllSyncResult, String> -> Sync result data or error message
/// --- ---
fn fetch_all_sync_data(endpoint: &str, sync_options: &SyncOptions) -> Result<AllSyncResult, String> {
    // extract the AllSync request from SyncOptions
    let all_sync = match sync_options {
        SyncOptions::All(all_sync) => all_sync,
        SyncOptions::Select(_) => {
            return Err("Expected AllSync options but got SelectSync".to_string());
        }
    };

    // create a blocking HTTP client
    let client = reqwest::blocking::Client::new();
    
    // build GET request with query parameters (classy server uses GET for /sync/all)
    let url = format!(
        "{}?lastSyncSequence={}&maxRecordsCount={}",
        endpoint,
        all_sync.last_sync,
        all_sync.max_records_count.unwrap_or(500)
    );
    
    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to connect to classy server: {}", e))?;

    // check if the request was successful
    if !response.status().is_success() {
        return Err(format!(
            "Classy server returned error: {} - {}",
            response.status(),
            response.text().unwrap_or_else(|_| "Unknown error".to_string())
        ));
    }

    // parse the response as AllSyncResult
    let sync_result: AllSyncResult = response
        .json()
        .map_err(|e| format!("Failed to parse sync response: {}", e))?;

    Ok(sync_result)
}

/// Sync all class data from classy server
///
/// Parameters:
/// --- ---
/// config -> Sync configuration
/// --- ---
///
/// Returns:
/// --- ---
/// Result<PathBuf, String> -> Path to the synced database or error message
/// --- ---
pub fn sync_all(config: &SyncConfig) -> Result<PathBuf, String> {
    // set server URL and port in environment for classy-sync to use
    std::env::set_var("CLASSY_SERVER_URL", &config.server_url);
    std::env::set_var("CLASSY_SERVER_PORT", config.server_port.to_string());
    
    // ensure the database directory exists
    if let Some(parent) = config.db_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    // set database path in environment for classy-sync's Sqlite to use
    let db_path_str = config.db_path.to_str()
        .ok_or_else(|| "Invalid database path".to_string())?;
    std::env::set_var("SQLITE_DB_PATH", db_path_str);

    // initialize SQLite datastore
    let mut datastore = Sqlite::new()
        .map_err(|e| format!("Failed to initialize SQLite datastore: {}", e))?;

    // set sync resources to sync everything
    datastore
        .set_request_sync_resources(SyncResources::Everything)
        .map_err(|e| format!("Failed to set sync resources: {}", e))?;

    // generate sync options (this reads the current state from the database)
    let sync_options = datastore
        .generate_sync_options()
        .map_err(|e| format!("Failed to generate sync options: {}", e))?;

    // fetch sync data from the classy server
    let endpoint = config.all_sync_endpoint();
    let sync_result = fetch_all_sync_data(&endpoint, &sync_options)?;

    // execute the sync (applies the data to the local database)
    datastore
        .execute_all_request_sync(sync_result)
        .map_err(|e| format!("Failed to execute sync: {}", e))?;

    Ok(config.db_path.clone())
}

/// Sync data for specific schools from classy server
///
/// Parameters:
/// --- ---
/// config -> Sync configuration
/// schools -> Comma-separated list of school IDs and optional term collection IDs
///            Format: "school1;school2,term1;school3,term2"
/// --- ---
///
/// Returns:
/// --- ---
/// Result<PathBuf, String> -> Path to the synced database or error message
/// --- ---
pub fn sync_schools(config: &SyncConfig, schools: &str) -> Result<PathBuf, String> {
    use classy_sync::argument_parser::SelectSyncOptions;
    use classy_sync::data_stores::sync_requests::TermSyncResult;

    // set environment variables
    std::env::set_var("CLASSY_SERVER_URL", &config.server_url);
    std::env::set_var("CLASSY_SERVER_PORT", config.server_port.to_string());
    
    if let Some(parent) = config.db_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    let db_path_str = config.db_path.to_str()
        .ok_or_else(|| "Invalid database path".to_string())?;
    std::env::set_var("SQLITE_DB_PATH", db_path_str);

    // initialize datastore
    let mut datastore = Sqlite::new()
        .map_err(|e| format!("Failed to initialize SQLite datastore: {}", e))?;

    // parse the schools string and create SelectSyncOptions
    // format: "school1;school2,term1;school3,term2"
    let select_options = SelectSyncOptions::from_input(schools.to_string());

    // set sync resources for selected schools/terms
    datastore
        .set_request_sync_resources(SyncResources::Select(select_options))
        .map_err(|e| format!("Failed to set sync resources: {}", e))?;

    // generate sync options
    let sync_options = datastore
        .generate_sync_options()
        .map_err(|e| format!("Failed to generate sync options: {}", e))?;

    // extract SelectSync from options
    let select_sync = match sync_options {
        SyncOptions::Select(select) => select,
        SyncOptions::All(_) => {
            return Err("Expected SelectSync options but got AllSync".to_string());
        }
    };

    // fetch sync data from server
    let endpoint = format!("{}/sync/select", config.server_url_with_port());
    let client = reqwest::blocking::Client::new();
    
    let response = client
        .post(&endpoint)
        .json(&select_sync)
        .send()
        .map_err(|e| format!("Failed to connect to classy server: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Classy server returned error: {} - {}",
            response.status(),
            response.text().unwrap_or_else(|_| "Unknown error".to_string())
        ));
    }

    let sync_result: TermSyncResult = response
        .json()
        .map_err(|e| format!("Failed to parse sync response: {}", e))?;

    // execute the sync
    datastore
        .execute_select_request_sync(select_sync, sync_result)
        .map_err(|e| format!("Failed to execute sync: {}", e))?;

    Ok(config.db_path.clone())
}

/// Get the synced database path
///
/// Returns:
/// --- ---
/// PathBuf -> Path to the synced database
/// --- ---
pub fn get_synced_db_path() -> PathBuf {
    // try to get from environment or use default
    if let Ok(config) = SyncConfig::from_env() {
        config.db_path
    } else {
        // fallback to default location relative to cargo manifest directory
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("classy").join("classes.db")
    }
}

/// Check if the sync database exists and is populated
///
/// Returns:
/// --- ---
/// bool -> true if database exists, false otherwise
/// --- ---
pub fn is_synced() -> bool {
    let db_path = get_synced_db_path();
    db_path.exists() && db_path.metadata().map(|m| m.len() > 0).unwrap_or(false)
}
