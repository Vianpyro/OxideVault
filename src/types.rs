//! Type definitions and aliases for the bot.
//!
//! This module contains shared types used throughout the application.

use crate::database::PlayerRepository;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;

/// Bot application data shared across all commands.
///
/// This data is accessible in all command handlers through the context.
pub struct Data {
    /// Path to the SQLite database file
    pub db_path: String,
    /// HTTP client for making API requests
    pub http_client: reqwest::Client,
    /// Minecraft server address to query
    pub mc_server_address: String,
    /// Backup folder path
    pub backup_folder: String,
    /// Rate limiter for backup command: tracks last backup time per user
    pub last_backup_time: Arc<RwLock<HashMap<u64, Instant>>>,
    /// Global rate limiter: tracks last backup time (any user)
    pub last_global_backup_time: Arc<RwLock<Option<Instant>>>,
    /// Folder where downloadable backups are published (served by reverse proxy)
    pub backup_publish_root: String,
    /// Public base URL where published backups are accessible
    pub backup_public_base_url: String,
}

impl Data {
    /// Create a new player repository for database operations.
    pub fn player_repository(&self) -> PlayerRepository {
        PlayerRepository::new(self.db_path.clone())
    }
}

/// Error type for bot commands (maintains compatibility with poise).
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Command context type alias for easier usage.
pub type Context<'a> = poise::Context<'a, Data, Error>;
