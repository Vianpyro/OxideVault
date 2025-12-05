use rusqlite::Connection;
use std::error::Error;

fn init_db_sync(path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = Connection::open(path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Minecraft users table - primary source of truth
    // Note: PRIMARY KEY implies uniqueness, so no UNIQUE constraint is needed
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

// Initialize the database file (creates tables if necessary).
pub async fn init_db(path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || init_db_sync(&path))
        .await??;
    Ok(())
}
