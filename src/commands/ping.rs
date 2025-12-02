use crate::types::{Context, Error};

#[poise::command(slash_command)]
pub async fn ping(context: Context<'_>) -> Result<(), Error> {
    context.say("Pong! 🏓").await?;
    Ok(())
}
