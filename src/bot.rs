use crate::types::Data;
use crate::commands::{ping, uuid};
use crate::db;
use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use std::env;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            return Err("Missing DISCORD_TOKEN environment variable. Set it in your environment or create a .env file (never commit this file).".into());
        }
    };

    let intents = serenity::GatewayIntents::non_privileged();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "oxidevault.db".to_string());
    // Initialize DB (creates file and tables if needed)
    db::init_db(&db_path).await?;

    // Create HTTP client for API requests (reused across requests for better performance)
    let http_client = reqwest::Client::new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), uuid()],
            ..Default::default()
        })
        .setup(move |context, _ready, framework| {
            let db_path = db_path.clone();
            let http_client = http_client.clone();
            Box::pin(async move {
                poise::builtins::register_globally(context, &framework.options().commands).await?;
                Ok(Data { db_path, http_client })
            })
        })
        .build();


    let mut client = serenity::ClientBuilder::new(token, intents).framework(framework).await?;

    client.start().await?;

    Ok(())
}
