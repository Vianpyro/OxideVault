//! Configuration management for OxideVault.
//!
//! This module handles loading and validating environment variables and application settings.

use crate::error::{OxideVaultError, Result};
use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Discord bot token
    pub discord_token: String,
    /// Path to SQLite database file
    pub db_path: String,
    /// Minecraft server address (host:port)
    pub mc_server_address: String,
    /// Backup folder path
    pub backup_folder: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// This will attempt to load a .env file if present using dotenv,
    /// then read required environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any required environment variable is missing or invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidevault::config::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load configuration");
    /// println!("Server: {}", config.mc_server_address);
    /// ```
    pub fn from_env() -> Result<Self> {
        // Load .env file if present (ignore errors - it's optional)
        dotenv::dotenv().ok();

        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| OxideVaultError::Config(
                "Missing DISCORD_TOKEN environment variable. Set it in your environment or create a .env file (never commit this file).".to_string()
            ))?;

        let db_path = Self::get_db_path()?;

        let mc_server_address = env::var("MC_SERVER_ADDRESS")
            .map_err(|_| OxideVaultError::Config(
                "Missing MC_SERVER_ADDRESS environment variable. Set it in your environment or .env file (e.g., MC_SERVER_ADDRESS=localhost:25565).".to_string()
            ))?;

        // Validate server address format
        Self::validate_server_address(&mc_server_address)?;

        let backup_folder = env::var("BACKUP_FOLDER")
            .map_err(|_| OxideVaultError::Config(
                "Missing BACKUP_FOLDER environment variable. Set it in your environment or .env file (e.g., BACKUP_FOLDER=/path/to/backups).".to_string()
            ))?;

        // Validate backup folder path
        Self::validate_backup_folder(&backup_folder)?;

        Ok(Self {
            discord_token,
            db_path,
            mc_server_address,
            backup_folder,
        })
    }

    /// Get the database path from environment or use default.
    fn get_db_path() -> Result<String> {
        match env::var("DB_PATH") {
            Ok(path) => Ok(path),
            Err(_) => {
                let mut path = env::current_dir()
                    .map_err(|e| OxideVaultError::Config(
                        format!("Failed to determine current directory: {}", e)
                    ))?;

                path.push("data");
                path.push("oxidevault.db");

                path.into_os_string()
                    .into_string()
                    .map_err(|os_str| OxideVaultError::Config(
                        format!("Database path contains invalid Unicode: {:?}", os_str)
                    ))
            }
        }
    }

    /// Validate that the server address has a valid format.
    fn validate_server_address(address: &str) -> Result<()> {
        if !address.contains(':') {
            return Err(OxideVaultError::Config(
                format!("Invalid MC_SERVER_ADDRESS format: '{}'. Expected 'host:port' format.", address)
            ));
        }

        // Try to parse port
        if let Some((_, port_str)) = address.rsplit_once(':') {
            port_str.parse::<u16>()
                .map_err(|_| OxideVaultError::Config(
                    format!("Invalid port in MC_SERVER_ADDRESS: '{}'", port_str)
                ))?;
        }

        Ok(())
    }

    /// Validate that the backup folder path exists and is a directory.
    fn validate_backup_folder(path: &str) -> Result<()> {
        use std::path::Path;
        
        let backup_path = Path::new(path);
        
        if !backup_path.is_absolute() {
            return Err(OxideVaultError::Config(
                format!("BACKUP_FOLDER must be an absolute path, got: '{}'", path)
            ));
        }
        
        if !backup_path.exists() {
            return Err(OxideVaultError::Config(
                format!("BACKUP_FOLDER path does not exist: '{}'", path)
            ));
        }
        
        if !backup_path.is_dir() {
            return Err(OxideVaultError::Config(
                format!("BACKUP_FOLDER path is not a directory: '{}'", path)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_validate_server_address() {
        assert!(Config::validate_server_address("localhost:25565").is_ok());
        assert!(Config::validate_server_address("127.0.0.1:25565").is_ok());
        assert!(Config::validate_server_address("example.com:25565").is_ok());

        assert!(Config::validate_server_address("localhost").is_err());
        assert!(Config::validate_server_address("localhost:abc").is_err());
        assert!(Config::validate_server_address("localhost:99999").is_err());
    }

    #[test]
    fn test_get_db_path_with_env_var() {
        // Save original value (if any)
        let original_value = env::var("DB_PATH").ok();

        // Set custom path
        let custom_path = "/custom/path/to/database.db";
        env::set_var("DB_PATH", custom_path);

        let result = Config::get_db_path();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path);

        // Restore original value
        match original_value {
            Some(val) => env::set_var("DB_PATH", val),
            None => env::remove_var("DB_PATH"),
        }
    }

    #[test]
    fn test_get_db_path_default() {
        // Save original value (if any)
        let original_value = env::var("DB_PATH").ok();

        // Remove DB_PATH env var to test default behavior
        env::remove_var("DB_PATH");

        let result = Config::get_db_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        // Verify the path contains expected components
        assert!(path.contains("data"));
        assert!(path.contains("oxidevault.db"));
        assert!(path.ends_with("data/oxidevault.db") || path.ends_with("data\\oxidevault.db"));

        // Restore original value
        match original_value {
            Some(val) => env::set_var("DB_PATH", val),
            None => {}, // Already removed
        }
    }
}
