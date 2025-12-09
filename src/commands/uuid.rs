//! UUID lookup command.
//!
//! Allows users to look up Minecraft player UUIDs by username.

use crate::types::{Context, Error};
use crate::mojang;
use crate::utils::validation::{validate_minecraft_username, format_uuid};
use crate::database::MinecraftPlayer;

/// Look up a Minecraft player's UUID by their username.
///
/// This command queries the Mojang API and optionally stores the result in the database.
#[poise::command(slash_command)]
pub async fn uuid(
    context: Context<'_>,
    #[description = "Minecraft username"]
    #[min_length = 1]
    #[max_length = 16]
    name: String,
) -> Result<(), Error> {
    // Validate username format
    if let Err(e) = validate_minecraft_username(&name) {
        context
            .say(format!("❌ {}", e))
            .await?;
        return Ok(());
    }

    context.defer().await?;

    match mojang::fetch_profile(&context.data().http_client, &name).await {
        Ok(Some(profile)) => {
            // Try to store in database (non-fatal if it fails)
            let repo = context.data().player_repository();
            let _ = repo.upsert_player(MinecraftPlayer {
                uuid: profile.id.clone(),
                username: profile.name.clone(),
            }).await;

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
