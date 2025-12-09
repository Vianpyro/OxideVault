//! Mojang API integration.
//!
//! This module provides functions for interacting with the Mojang API
//! to retrieve player profile information.

use serde::Deserialize;
use crate::error::{OxideVaultError, Result};

/// Player profile information from Mojang API.
#[derive(Deserialize, Debug, Clone)]
pub struct MojangProfile {
    /// Player UUID (without dashes)
    pub id: String,
    /// Current player username
    pub name: String,
}

/// Fetch a player profile from the Mojang API.
///
/// # Arguments
///
/// * `client` - HTTP client to use for the request
/// * `name` - Minecraft username to look up
///
/// # Returns
///
/// Returns `Some(profile)` if the player exists, `None` if not found.
///
/// # Errors
///
/// Returns an error if the API request fails or returns an unexpected status code.
///
/// # Examples
///
/// ```no_run
/// use oxidevault::mojang::fetch_profile;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = reqwest::Client::new();
/// let profile = fetch_profile(&client, "Notch").await?;
///
/// if let Some(p) = profile {
///     println!("UUID: {}, Name: {}", p.id, p.name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch_profile(client: &reqwest::Client, name: &str) -> Result<Option<MojangProfile>> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);
    let resp = client.get(&url).send().await
        .map_err(|e| OxideVaultError::MojangApi(format!("Request failed: {}", e)))?;

    if resp.status().is_success() {
        let profile = resp.json::<MojangProfile>().await
            .map_err(|e| OxideVaultError::MojangApi(format!("Invalid response: {}", e)))?;
        Ok(Some(profile))
    } else if resp.status().as_u16() == 404 {
        Ok(None)
    } else {
        Err(OxideVaultError::MojangApi(
            format!("API returned error: {}", resp.status())
        ))
    }
}
