use rusqlite::{params, Connection, OptionalExtension};
use std::error::Error;

fn init_db_sync(path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_links (
            discord_id TEXT PRIMARY KEY,
            mc_username TEXT NOT NULL,
            mc_uuid TEXT NOT NULL
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
    conn.execute(
        "INSERT INTO user_links (discord_id, mc_username, mc_uuid) VALUES (?1, ?2, ?3)
         ON CONFLICT(discord_id) DO UPDATE SET mc_username=excluded.mc_username, mc_uuid=excluded.mc_uuid",
        params![discord_id, mc_username, mc_uuid],
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
    let mut stmt = conn.prepare("SELECT mc_username, mc_uuid FROM user_links WHERE discord_id = ?1")?;
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
    conn.execute("DELETE FROM user_links WHERE discord_id = ?1", params![discord_id])?;
    Ok(())
}

pub async fn unlink_by_discord(path: &str, discord_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = path.to_string();
    let discord = discord_id.to_string();
    tokio::task::spawn_blocking(move || unlink_by_discord_sync(&path, &discord)).await??;
    Ok(())
}
