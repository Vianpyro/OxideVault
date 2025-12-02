use crate::types::{Context, Error};
use crate::mojang;
use crate::db;

#[poise::command(slash_command)]
pub async fn link(
    context: Context<'_>,
    #[description = "Minecraft username"]
    #[min_length = 1]
    #[max_length = 16]
    name: String,
) -> Result<(), Error> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        context
            .say("❌ Invalid username! Minecraft usernames can only contain letters, numbers, and underscores.")
            .await?;
        return Ok(());
    }

    context.defer().await?;

    match mojang::fetch_profile(&name).await {
        Ok(Some(profile)) => {
            let uuid = profile.id.clone();
            let discord_id = context.author().id.to_string();
            let db_path = &context.data().db_path;

            if let Err(e) = db::link_user(db_path, &discord_id, &profile.name, &uuid).await {
                context
                    .say(format!("❌ Failed to link user into DB: {}", e))
                    .await?;
                return Ok(());
            }

            context
                .say(format!("✅ Linked Discord <@{}> to Minecraft `{}` ({})", discord_id, profile.name, uuid))
                .await?;
        }
        Ok(None) => {
            context
                .say("❌ Player not found! Make sure the username is correct.")
                .await?;
        }
        Err(e) => {
            context
                .say(format!("❌ Failed to connect to Mojang API: {}", e))
                .await?;
        }
    }

    Ok(())
}
