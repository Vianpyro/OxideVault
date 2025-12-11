//! Ping command for bot health check.

use crate::types::{Context, Error};

/// Simple ping command to check if the bot is responsive.
#[poise::command(slash_command)]
pub async fn ping(context: Context<'_>) -> Result<(), Error> {
    context.say("Pong! 🏓").await?;
    Ok(())
}
