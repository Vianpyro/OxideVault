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
            return Err("Missing DISCORD_TOKEN environment variable. Provide it via environment or a .env file containing DISCORD_TOKEN=...".into());
        }
    };

    let intents = serenity::GatewayIntents::non_privileged();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "oxidevault.db".to_string());
    // Initialize DB (creates file and tables if needed)
    db::init_db(&db_path).await?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), uuid()],
            ..Default::default()
        })
        .setup(move |context, _ready, framework| {
            let db_path = db_path.clone();
            Box::pin(async move {
                poise::builtins::register_globally(context, &framework.options().commands).await?;
                Ok(Data { db_path })
            })
        })
        .build();


    let mut client = serenity::ClientBuilder::new(token, intents).framework(framework).await?;

    client.start().await?;

    Ok(())
}
