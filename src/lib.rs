//! OxideVault library.
//!
//! This library provides the core functionality for the OxideVault Discord bot,
//! including Minecraft server monitoring, player management, and API integrations.

pub mod error;
pub mod config;
pub mod database;
pub mod mojang;
pub mod mc_server;
pub mod utils;

pub use error::{OxideVaultError, Result};
pub use config::Config;
