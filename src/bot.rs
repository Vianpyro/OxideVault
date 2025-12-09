//! Discord bot initialization and configuration.
//!
//! This module handles the setup and execution of the Discord bot,
//! including command registration and framework initialization.

use crate::types::Data;
use crate::commands::{ping, uuid, online};
use crate::database;
use crate::config::Config;
use poise::serenity_prelude as serenity;
use rand::Rng;

/// Event handler for non-command events.
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    _data: &Data,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            // Check if the bot is mentioned
            if new_message.mentions.iter().any(|user| user.id == ctx.cache.current_user().id) {
                // Randomly choose between wave and eyes emoji
                let emoji = if rand::thread_rng().gen_bool(0.5) { "👋" } else { "👀" };

                // React to the message
                if let Err(e) = new_message
                    .react(&ctx.http, serenity::ReactionType::Unicode(emoji.to_string()))
                    .await
                {
                    eprintln!("Failed to react to message: {}", e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

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

    // Enable necessary intents for receiving messages and mentions
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), uuid(), online()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
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
