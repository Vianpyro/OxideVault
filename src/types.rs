//! Type definitions and aliases for the bot.
//!
//! This module contains shared types used throughout the application.

use crate::database::PlayerRepository;

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
