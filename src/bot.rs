//! Discord bot initialization and configuration.
//!
//! This module handles the setup and execution of the Discord bot,
//! including command registration and framework initialization.

use crate::types::Data;
use crate::commands::{ping, uuid, online, backup};
use crate::database;
use crate::config::Config;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Run the Discord bot.
///
/// This function initializes the bot with configuration from environment variables,
/// sets up the database, and starts the Discord client.
///
/// # Errors
///
/// Returns an error if configuration is invalid, database initialization fails,
/// or the Discord client cannot be started.
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration from environment
    let config = Config::from_env()?;

    // Initialize database
    database::init_db(&config.db_path).await?;

    // Create HTTP client for API requests (reused across requests for better performance)
    let http_client = reqwest::Client::new();

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), uuid(), online(), backup()],
            ..Default::default()
        })
        .setup(move |context, _ready, framework| {
            let db_path = config.db_path.clone();
            let http_client = http_client.clone();
            let mc_server_address = config.mc_server_address.clone();
            let backup_folder = config.backup_folder.clone();
            let backup_publish_root = config.backup_publish_root.clone();
            let backup_public_base_url = config.backup_public_base_url.clone();
            Box::pin(async move {
                poise::builtins::register_globally(context, &framework.options().commands).await?;
                Ok(Data {
                    db_path,
                    http_client,
                    mc_server_address,
                    backup_folder,
                    last_backup_time: Arc::new(RwLock::new(HashMap::new())),
                    last_global_backup_time: Arc::new(RwLock::new(None)),
                    backup_publish_root,
                    backup_public_base_url,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
