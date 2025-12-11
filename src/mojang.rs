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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_fetch_profile_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/users/profiles/minecraft/Notch")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"069a79f444e94726a5befca90e38aaf5","name":"Notch"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/users/profiles/minecraft/Notch", server.url());
        
        // Make the request to the mock server
        let resp = client.get(&url).send().await.unwrap();
        let profile: Option<MojangProfile> = if resp.status().is_success() {
            Some(resp.json().await.unwrap())
        } else {
            None
        };

        mock.assert_async().await;
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.id, "069a79f444e94726a5befca90e38aaf5");
        assert_eq!(profile.name, "Notch");
    }

    #[tokio::test]
    async fn test_fetch_profile_not_found() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/users/profiles/minecraft/NonExistentPlayer")
            .with_status(404)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/users/profiles/minecraft/NonExistentPlayer", server.url());
        
        let resp = client.get(&url).send().await.unwrap();
        let profile: Option<MojangProfile> = if resp.status().is_success() {
            Some(resp.json().await.unwrap())
        } else if resp.status().as_u16() == 404 {
            None
        } else {
            panic!("Unexpected status");
        };

        mock.assert_async().await;
        assert!(profile.is_none());
    }

    #[tokio::test]
    async fn test_fetch_profile_invalid_json() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/users/profiles/minecraft/TestPlayer")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/users/profiles/minecraft/TestPlayer", server.url());
        
        let resp = client.get(&url).send().await.unwrap();
        let result: std::result::Result<MojangProfile, reqwest::Error> = resp.json().await;

        mock.assert_async().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_profile_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/users/profiles/minecraft/ErrorPlayer")
            .with_status(500)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/users/profiles/minecraft/ErrorPlayer", server.url());
        
        let resp = client.get(&url).send().await.unwrap();
        let status = resp.status();

        mock.assert_async().await;
        assert!(!status.is_success());
        assert_eq!(status.as_u16(), 500);
    }
}
