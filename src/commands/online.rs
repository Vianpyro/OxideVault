//! Online players command.
//!
//! Queries the Minecraft server for status and online player information.

use crate::types::{Context, Error};
use crate::mc_server;

/// Check the status and online players of the configured Minecraft server.
#[poise::command(slash_command)]
pub async fn online(
    context: Context<'_>,
) -> Result<(), Error> {
    // Defer reply since server ping might take a moment
    context.defer().await?;

    // Get server address from bot data
    let server_address = context.data().mc_server_address.clone();

    // Ping the server in a blocking task
    let result = tokio::task::spawn_blocking(move || {
        mc_server::ping_server(&server_address)
    }).await;

    match result {
        Ok(Ok(status)) => {
            let player_list = if !status.players.sample.is_empty() {
                let players: Vec<&str> = status.players.sample
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect();
                format!("\n**Players online:** {}", players.join(", "))
            } else {
                String::new()
            };

            let response = format!(
                "**Minecraft Server Status** 🎮\n\
                **Version:** {}\n\
                **Players:** {}/{}\n\
                **Description:** {}{}",
                status.version.name,
                status.players.online,
                status.players.max,
                status.description.text(),
                player_list
            );

            context.say(response).await?;
        }
        Ok(Err(e)) => {
            context.say(format!("❌ Failed to connect to server: {}", e)).await?;
        }
        Err(e) => {
            context.say(format!("❌ Error: {}", e)).await?;
        }
    }

    Ok(())
}
