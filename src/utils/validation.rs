//! Validation utilities for user input.
//!
//! This module provides reusable validation functions for various types of input.

use crate::error::{OxideVaultError, Result};

/// Validate a Minecraft username.
///
/// Minecraft usernames must:
/// - Be between 1 and 16 characters
/// - Contain only alphanumeric characters and underscores
///
/// # Arguments
///
/// * `username` - The username to validate
///
/// # Returns
///
/// Returns `Ok(())` if the username is valid, otherwise returns an error describing the issue.
///
/// # Examples
///
/// ```
/// use oxidevault::utils::validation::validate_minecraft_username;
///
/// assert!(validate_minecraft_username("Steve").is_ok());
/// assert!(validate_minecraft_username("Player_123").is_ok());
/// assert!(validate_minecraft_username("").is_err());
/// assert!(validate_minecraft_username("Invalid Name").is_err());
/// ```
pub fn validate_minecraft_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(OxideVaultError::Validation(
            "Username cannot be empty".to_string()
        ));
    }

    if username.len() > 16 {
        return Err(OxideVaultError::Validation(
            format!("Username too long: {} characters (max 16)", username.len())
        ));
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(OxideVaultError::Validation(
            "Username can only contain letters, numbers, and underscores".to_string()
        ));
    }

    Ok(())
}

/// Format a 32-character UUID string into the standard 8-4-4-4-12 format.
///
/// # Arguments
///
/// * `uuid` - A 32-character hex string without dashes
///
/// # Returns
///
/// Returns the formatted UUID string, or `None` if the input is not exactly 32 characters.
///
/// # Examples
///
/// ```
/// use oxidevault::utils::validation::format_uuid;
///
/// let uuid = format_uuid("069a79f444e94726a5befca90e38aaf5");
/// assert_eq!(uuid, Some("069a79f4-44e9-4726-a5be-fca90e38aaf5".to_string()));
///
/// assert_eq!(format_uuid("invalid"), None);
/// ```
pub fn format_uuid(uuid: &str) -> Option<String> {
    if uuid.len() != 32 {
        return None;
    }
    Some(format!(
        "{}-{}-{}-{}-{}",
        &uuid[0..8], &uuid[8..12], &uuid[12..16], &uuid[16..20], &uuid[20..32]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_minecraft_username() {
        // Valid usernames
        assert!(validate_minecraft_username("Steve").is_ok());
        assert!(validate_minecraft_username("Player_123").is_ok());
        assert!(validate_minecraft_username("a").is_ok());
        assert!(validate_minecraft_username("1234567890123456").is_ok()); // 16 chars

        // Invalid usernames
        assert!(validate_minecraft_username("").is_err());
        assert!(validate_minecraft_username("12345678901234567").is_err()); // 17 chars
        assert!(validate_minecraft_username("Invalid Name").is_err()); // space
        assert!(validate_minecraft_username("Player@123").is_err()); // @
        assert!(validate_minecraft_username("Player-123").is_err()); // dash
    }

    #[test]
    fn test_format_uuid() {
        assert_eq!(
            format_uuid("069a79f444e94726a5befca90e38aaf5"),
            Some("069a79f4-44e9-4726-a5be-fca90e38aaf5".to_string())
        );

        assert_eq!(format_uuid("invalid"), None);
        assert_eq!(format_uuid(""), None);
        assert_eq!(format_uuid("069a79f444e94726a5befca90e38aaf5extra"), None);
    }
}
