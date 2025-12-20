//! Custom error types for OxideVault.
//!
//! This module provides a centralized error handling system with specific error types
//! for different parts of the application.

use std::fmt;

/// Main error type for OxideVault operations.
#[derive(Debug)]
pub enum OxideVaultError {
    /// Configuration errors (missing env vars, invalid values)
    Config(String),
    /// Database operation errors
    Database(String),
    /// Minecraft server protocol errors
    ServerProtocol(String),
    /// Mojang API errors
    MojangApi(String),
    /// Network/HTTP errors
    Network(String),
    /// Discord bot errors
    Discord(String),
    /// Validation errors (invalid usernames, etc.)
    Validation(String),
    /// Generic I/O errors
    Io(std::io::Error),
    /// Invalid input errors
    InvalidInput(String),
}

impl fmt::Display for OxideVaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Database(msg) => write!(f, "Database error: {}", msg),
            Self::ServerProtocol(msg) => write!(f, "Server protocol error: {}", msg),
            Self::MojangApi(msg) => write!(f, "Mojang API error: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Discord(msg) => write!(f, "Discord error: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for OxideVaultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OxideVaultError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// Implement From traits for automatic error conversion
impl From<std::io::Error> for OxideVaultError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<rusqlite::Error> for OxideVaultError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<reqwest::Error> for OxideVaultError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<serde_json::Error> for OxideVaultError {
    fn from(err: serde_json::Error) -> Self {
        Self::ServerProtocol(format!("JSON parsing error: {}", err))
    }
}

impl From<std::env::VarError> for OxideVaultError {
    fn from(err: std::env::VarError) -> Self {
        Self::Config(err.to_string())
    }
}

impl From<tokio::task::JoinError> for OxideVaultError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Discord(format!("Task join error: {}", err))
    }
}

/// Result type alias for OxideVault operations.
pub type Result<T> = std::result::Result<T, OxideVaultError>;
