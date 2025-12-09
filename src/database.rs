//! Database operations and data access layer.
//!
//! This module provides a repository pattern for database operations,
//! separating database concerns from business logic.

use rusqlite::Connection;
use crate::error::{OxideVaultError, Result};
use std::path::Path;

/// Minecraft player information.
#[derive(Debug, Clone)]
pub struct MinecraftPlayer {
    pub uuid: String,
    pub username: String,
}

/// Player statistics entry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerStat {
    pub mc_uuid: String,
    pub stat_name: String,
    pub stat_value: i64,
    pub timestamp: i64,
}

/// Initialize the database schema.
///
/// Creates the necessary tables and indices if they don't already exist.
/// Also creates the parent directory if needed.
///
/// # Arguments
///
/// * `path` - Path to the SQLite database file
///
/// # Errors
///
/// Returns an error if the database cannot be created or initialized.
pub async fn init_db(path: &str) -> Result<()> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || init_db_sync(&path))
        .await
        .map_err(|e| OxideVaultError::Database(format!("Task join error: {}", e)))??;
    Ok(())
}

fn init_db_sync(path: &str) -> Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Minecraft users table - primary source of truth
    conn.execute(
        "CREATE TABLE IF NOT EXISTS minecraft_users (
            mc_uuid TEXT NOT NULL PRIMARY KEY,
            mc_username TEXT NOT NULL
        )",
        [],
    )?;

    // Add index on mc_username for faster lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_mc_username ON minecraft_users(mc_username)",
        [],
    )?;

    // Stats table - linked to MC users
    conn.execute(
        "CREATE TABLE IF NOT EXISTS player_stats (
            mc_uuid TEXT NOT NULL,
            stat_name TEXT NOT NULL,
            stat_value INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            PRIMARY KEY (mc_uuid, stat_name),
            FOREIGN KEY (mc_uuid) REFERENCES minecraft_users(mc_uuid) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

/// Repository for player database operations.
pub struct PlayerRepository {
    db_path: String,
}

impl PlayerRepository {
    /// Create a new player repository.
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    /// Get a connection to the database.
    #[allow(dead_code)]
    fn connect(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
            .map_err(|e| OxideVaultError::Database(format!("Failed to connect to database: {}", e)))
    }

    /// Insert or update a player in the database.
    ///
    /// # Arguments
    ///
    /// * `player` - The player information to save
    pub async fn upsert_player(&self, player: MinecraftPlayer) -> Result<()> {
        let db_path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;
            conn.execute(
                "INSERT INTO minecraft_users (mc_uuid, mc_username)
                 VALUES (?1, ?2)
                 ON CONFLICT(mc_uuid) DO UPDATE SET mc_username = ?2",
                rusqlite::params![player.uuid, player.username],
            )?;
            Ok::<_, OxideVaultError>(())
        })
        .await
        .map_err(|e| OxideVaultError::Database(format!("Task join error: {}", e)))??;
        Ok(())
    }

    /// Get a player by UUID.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The player's UUID
    ///
    /// # Returns
    ///
    /// Returns `Some(player)` if found, `None` otherwise.
    #[allow(dead_code)]
    pub async fn get_player_by_uuid(&self, uuid: &str) -> Result<Option<MinecraftPlayer>> {
        let db_path = self.db_path.clone();
        let uuid = uuid.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;
            let mut stmt = conn.prepare(
                "SELECT mc_uuid, mc_username FROM minecraft_users WHERE mc_uuid = ?1"
            )?;

            let mut rows = stmt.query(rusqlite::params![uuid])?;

            if let Some(row) = rows.next()? {
                Ok(Some(MinecraftPlayer {
                    uuid: row.get(0)?,
                    username: row.get(1)?,
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| OxideVaultError::Database(format!("Task join error: {}", e)))?
    }

    /// Get a player by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The player's username
    ///
    /// # Returns
    ///
    /// Returns `Some(player)` if found, `None` otherwise.
    #[allow(dead_code)]
    pub async fn get_player_by_username(&self, username: &str) -> Result<Option<MinecraftPlayer>> {
        let db_path = self.db_path.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;
            let mut stmt = conn.prepare(
                "SELECT mc_uuid, mc_username FROM minecraft_users WHERE mc_username = ?1"
            )?;

            let mut rows = stmt.query(rusqlite::params![username])?;

            if let Some(row) = rows.next()? {
                Ok(Some(MinecraftPlayer {
                    uuid: row.get(0)?,
                    username: row.get(1)?,
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| OxideVaultError::Database(format!("Task join error: {}", e)))?
    }

    /// Get all players from the database.
    #[allow(dead_code)]
    pub async fn get_all_players(&self) -> Result<Vec<MinecraftPlayer>> {
        let db_path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;
            let mut stmt = conn.prepare(
                "SELECT mc_uuid, mc_username FROM minecraft_users ORDER BY mc_username"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(MinecraftPlayer {
                    uuid: row.get(0)?,
                    username: row.get(1)?,
                })
            })?;

            let mut players = Vec::new();
            for player in rows {
                players.push(player?);
            }
            Ok(players)
        })
        .await
        .map_err(|e| OxideVaultError::Database(format!("Task join error: {}", e)))?
    }

    /// Delete a player from the database.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The player's UUID
    #[allow(dead_code)]
    pub async fn delete_player(&self, uuid: &str) -> Result<()> {
        let db_path = self.db_path.clone();
        let uuid = uuid.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path)?;
            conn.execute(
                "DELETE FROM minecraft_users WHERE mc_uuid = ?1",
                rusqlite::params![uuid],
            )?;
            Ok::<_, OxideVaultError>(())
        })
        .await
        .map_err(|e| OxideVaultError::Database(format!("Task join error: {}", e)))??;
        Ok(())
    }
}
