//! Configuration management for OxideVault.
//!
//! This module handles loading and validating environment variables and application settings.

use crate::error::{OxideVaultError, Result};
use std::env;

/// Configuration for the application, loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Discord bot token
    pub discord_token: String,
    /// Path to SQLite database file
    pub db_path: String,
    /// Minecraft server address (host:port)
    pub mc_server_address: String,
    /// Path to the directory containing backup files
    pub backup_folder: String,
    /// Directory where backups are published for download (served by reverse proxy)
    pub backup_publish_root: String,
    /// Public URL base where published backups are served (must match reverse proxy)
    pub backup_public_base_url: String,
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

        // Use /backups as the default when running in Docker unless overridden
        let backup_folder = env::var("BACKUP_FOLDER").unwrap_or_else(|_| "/backups".to_string());

        // Validate backup folder path (will error if the path is not absolute, missing, or not a directory)
        Self::validate_backup_folder(&backup_folder)?;

        // Where we publish downloadable backups (defaults to /backups/public)
        let backup_publish_root = env::var("BACKUP_PUBLISH_ROOT").unwrap_or_else(|_| "/backups/public".to_string());
        Self::validate_publish_root(&backup_publish_root)?;
        
        // Warn if backup_folder and backup_publish_root might be on different filesystems
        Self::check_filesystem_compatibility(&backup_folder, &backup_publish_root);

        // Public URL base (must match your reverse proxy, e.g., https://drop.example.com/backups)
        let backup_public_base_url = env::var("BACKUP_PUBLIC_BASE_URL")
            .unwrap_or_else(|_| "http://localhost/backups".to_string());
        Self::validate_public_base_url(&backup_public_base_url)?;

        Ok(Self {
            discord_token,
            db_path,
            mc_server_address,
            backup_folder,
            backup_publish_root,
            backup_public_base_url,
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

    /// Validate that the publish root exists (or create it) and is a directory.
    fn validate_publish_root(path: &str) -> Result<()> {
        use std::path::Path;
        use std::fs;

        let publish_path = Path::new(path);

        if !publish_path.is_absolute() {
            return Err(OxideVaultError::Config(
                format!("BACKUP_PUBLISH_ROOT must be an absolute path, got: '{}'", path)
            ));
        }

        if !publish_path.exists() {
            fs::create_dir_all(publish_path).map_err(|e| OxideVaultError::Config(
                format!("Failed to create BACKUP_PUBLISH_ROOT '{}': {}", path, e)
            ))?;
        }

        if !publish_path.is_dir() {
            return Err(OxideVaultError::Config(
                format!("BACKUP_PUBLISH_ROOT is not a directory: '{}'", path)
            ));
        }

        Ok(())
    }

    /// Validate the public base URL format using proper URL parsing.
    fn validate_public_base_url(url_str: &str) -> Result<()> {
        use url::Url;
        
        // Parse the URL to validate its structure
        let parsed_url = Url::parse(url_str)
            .map_err(|e| OxideVaultError::Config(
                format!("Invalid BACKUP_PUBLIC_BASE_URL '{}': {}", url_str, e)
            ))?;
        
        // Ensure it's HTTP or HTTPS
        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(OxideVaultError::Config(
                format!("BACKUP_PUBLIC_BASE_URL must use http:// or https:// scheme, got: '{}'", scheme)
            ));
        }
        
        // Ensure it has a host
        if parsed_url.host_str().is_none() {
            return Err(OxideVaultError::Config(
                format!("BACKUP_PUBLIC_BASE_URL must contain a valid host: '{}'", url_str)
            ));
        }
        
        Ok(())
    }
    
    /// Check if backup_folder and backup_publish_root are on compatible filesystems.
    /// Warns if they might be on different filesystems (hard linking will fail and fall back to copying).
    fn check_filesystem_compatibility(backup_folder: &str, publish_root: &str) {
        use std::path::Path;
        
        let backup_path = Path::new(backup_folder);
        let publish_path = Path::new(publish_root);
        
        // Check if publish_root is under backup_folder (likely same filesystem)
        if publish_path.starts_with(backup_path) {
            return; // Likely same filesystem
        }
        
        // On Unix systems, we can check device IDs to determine if paths are on the same filesystem
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            
            if let (Ok(backup_meta), Ok(publish_meta)) = (
                std::fs::metadata(backup_path),
                std::fs::metadata(publish_path),
            ) {
                if backup_meta.dev() != publish_meta.dev() {
                    eprintln!(
                        "Warning: BACKUP_FOLDER ('{}') and BACKUP_PUBLISH_ROOT ('{}') appear to be on different filesystems. \
                        Hard linking will fail and backups will be copied instead, which may be slower for large files.",
                        backup_folder, publish_root
                    );
                }
            }
        }
        
        // On non-Unix systems, just warn if they're not in a parent-child relationship
        #[cfg(not(unix))]
        {
            eprintln!(
                "Note: BACKUP_FOLDER ('{}') and BACKUP_PUBLISH_ROOT ('{}') are in different directories. \
                If they're on different filesystems, hard linking will fail and backups will be copied instead.",
                backup_folder, publish_root
            );
        }
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
