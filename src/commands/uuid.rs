use crate::types::{Context, Error};
use crate::mojang;

/// Formats a 32-character UUID string into the standard 8-4-4-4-12 format.
/// Returns None if the UUID is not exactly 32 characters.
fn format_uuid(uuid: &str) -> Option<String> {
    if uuid.len() != 32 {
        return None;
    }
    Some(format!(
        "{}-{}-{}-{}-{}",
        &uuid[0..8], &uuid[8..12], &uuid[12..16], &uuid[16..20], &uuid[20..32]
    ))
}

#[poise::command(slash_command)]
pub async fn uuid(
    context: Context<'_>,
    #[description = "Minecraft username"]
    #[min_length = 1]
    #[max_length = 16]
    name: String,
) -> Result<(), Error> {
    // Validate username characters only; Discord validates length via min_length/max_length attributes
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        context
            .say("❌ Invalid username! Minecraft usernames can only contain letters, numbers, and underscores.")
            .await?;
        return Ok(());
    }

    context.defer().await?;

    match mojang::fetch_profile(&context.data().http_client, &name).await {
        Ok(Some(profile)) => {
            if let Some(formatted_uuid) = format_uuid(&profile.id) {
                context
                    .say(format!("✅ **Player:** {}\n**UUID:** `{}`", profile.name, formatted_uuid))
                    .await?;
            } else {
                context
                    .say("❌ Unexpected UUID format returned from Mojang API.")
                    .await?;
            }
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
