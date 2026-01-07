/*
    src/data/pool.rs

    For database connection management

    This module provides a simple database path configuration.
    The actual connection is handled per-query in sql.rs.
*/

use std::path::PathBuf;

/// Database configuration
///
/// Holds the path to the SQLite database file
///
/// DbConfig fields:
/// --- ---
/// db_path -> Path to the database file
/// --- ---
///
/// Implemented Traits:
/// --- ---
/// Debug -> Debug trait for DbConfig
/// Clone -> Clone trait for DbConfig
/// --- ---
///
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub db_path: PathBuf,
}

/// DbConfig Implementation
///
/// Methods:
/// --- ---
/// new -> Create a new database configuration with the default path
/// with_path -> Create a database configuration with a custom path
/// get_path -> Get the database path
/// --- ---
///
impl DbConfig {
    /// Create a new database configuration with the default path
    ///
    /// Parameters:
    /// --- ---
    /// None
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Self -> DbConfig with default database path
    /// --- ---
    ///
    pub fn new() -> Self {
        DbConfig {
            db_path: PathBuf::from("src/data/classes.db"),
        }
    }

    /// Create a database configuration with a custom path
    ///
    /// Parameters:
    /// --- ---
    /// path -> Custom path to the database file
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// Self -> DbConfig with the specified path
    /// --- ---
    ///
    pub fn with_path(path: PathBuf) -> Self {
        DbConfig { db_path: path }
    }

    /// Get the database path
    ///
    /// Parameters:
    /// --- ---
    /// &self -> Reference to the DbConfig instance
    /// --- ---
    ///
    /// Returns:
    /// --- ---
    /// &PathBuf -> Reference to the database path
    /// --- ---
    ///
    pub fn get_path(&self) -> &PathBuf {
        &self.db_path
    }
}
