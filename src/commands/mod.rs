//! Discord bot commands.
//!
//! This module contains all available bot commands organized by functionality.

pub mod ping;
pub mod uuid;
pub mod online;
pub mod draw;
pub mod backup;

pub use ping::ping;
pub use uuid::uuid;
pub use online::online;
pub use draw::draw;
pub use backup::backup;
