use crate::types::{Context, Error};
use crate::mojang;

#[poise::command(slash_command)]
pub async fn uuid(
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
            let uuid = &profile.id;
            let formatted_uuid = format!(
                "{}-{}-{}-{}-{}",
                &uuid[0..8], &uuid[8..12], &uuid[12..16], &uuid[16..20], &uuid[20..32]
            );

            context
                .say(format!("✅ **Player:** {}\n**UUID:** `{}`", profile.name, formatted_uuid))
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
