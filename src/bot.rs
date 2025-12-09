//! Discord bot initialization and configuration.
//!
//! This module handles the setup and execution of the Discord bot,
//! including command registration and framework initialization.

use crate::types::Data;
use crate::commands::{ping, uuid, online};
use crate::database;
use crate::config::Config;
use poise::serenity_prelude as serenity;

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
            commands: vec![ping(), uuid(), online()],
            ..Default::default()
        })
        .setup(move |context, _ready, framework| {
            let db_path = config.db_path.clone();
            let http_client = http_client.clone();
            let mc_server_address = config.mc_server_address.clone();
            Box::pin(async move {
                poise::builtins::register_globally(context, &framework.options().commands).await?;
                Ok(Data { db_path, http_client, mc_server_address })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
