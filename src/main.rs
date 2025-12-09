//! OxideVault - A Minecraft Discord Bot
//!
//! OxideVault is a Discord bot for managing and monitoring Minecraft servers.
//! It provides commands for checking server status, looking up player information,
//! and more.

mod error;
mod config;
mod types;
mod mojang;
mod database;
mod commands;
mod bot;
mod mc_server;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot::run().await
}
