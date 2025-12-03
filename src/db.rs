use rusqlite::{params, Connection, OptionalExtension};
use std::error::Error;

fn init_db_sync(path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = Connection::open(path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Minecraft users table - primary source of truth
    conn.execute(
        "CREATE TABLE IF NOT EXISTS minecraft_users (
            mc_uuid TEXT PRIMARY KEY,
            mc_username TEXT NOT NULL
        )",
        [],
    )?;

    // Discord links - optional relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS discord_links (
            discord_id TEXT PRIMARY KEY,
            mc_uuid TEXT NOT NULL UNIQUE,
            FOREIGN KEY (mc_uuid) REFERENCES minecraft_users(mc_uuid) ON DELETE CASCADE
        )",
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

// Initialize the database file (creates tables if necessary).
pub async fn init_db(path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || init_db_sync(&path))
        .await??;
    Ok(())
}

fn link_user_sync(path: &str, discord_id: &str, mc_username: &str, mc_uuid: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = Connection::open(path)?;

    // Insert or update minecraft user
    conn.execute(
        "INSERT INTO minecraft_users (mc_uuid, mc_username) VALUES (?1, ?2)
         ON CONFLICT(mc_uuid) DO UPDATE SET mc_username=excluded.mc_username",
        params![mc_uuid, mc_username],
    )?;

    // Link to Discord account
    conn.execute(
        "INSERT INTO discord_links (discord_id, mc_uuid) VALUES (?1, ?2)
         ON CONFLICT(discord_id) DO UPDATE SET mc_uuid=excluded.mc_uuid",
        params![discord_id, mc_uuid],
    )?;

    Ok(())
}

pub async fn link_user(path: &str, discord_id: &str, mc_username: &str, mc_uuid: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = path.to_string();
    let discord = discord_id.to_string();
    let user = mc_username.to_string();
    let uuid = mc_uuid.to_string();
    tokio::task::spawn_blocking(move || link_user_sync(&path, &discord, &user, &uuid))
        .await??;
    Ok(())
}

fn get_link_by_discord_sync(path: &str, discord_id: &str) -> Result<Option<(String, String)>, Box<dyn Error + Send + Sync>> {
    let conn = Connection::open(path)?;
    let mut stmt = conn.prepare("SELECT mc_username, mc_uuid FROM discord_links JOIN minecraft_users ON discord_links.mc_uuid = minecraft_users.mc_uuid WHERE discord_id = ?1")?;
    let row = stmt.query_row(params![discord_id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
    Ok(row)
}

pub async fn get_link_by_discord(path: &str, discord_id: &str) -> Result<Option<(String, String)>, Box<dyn Error + Send + Sync>> {
    let path = path.to_string();
    let discord = discord_id.to_string();
    let res = tokio::task::spawn_blocking(move || get_link_by_discord_sync(&path, &discord)).await??;
    Ok(res)
}

fn unlink_by_discord_sync(path: &str, discord_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = Connection::open(path)?;
    conn.execute("DELETE FROM discord_links WHERE discord_id = ?1", params![discord_id])?;
    Ok(())
}

pub async fn unlink_by_discord(path: &str, discord_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = path.to_string();
    let discord = discord_id.to_string();
    tokio::task::spawn_blocking(move || unlink_by_discord_sync(&path, &discord)).await??;
    Ok(())
}
