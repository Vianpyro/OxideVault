//! Minecraft server status checking and communication.
//!
//! This module provides high-level functions for querying Minecraft servers,
//! including status checks and player information retrieval.

mod protocol;

use protocol::{send_packet, read_packet, write_varint, write_string, read_string};
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::error::{OxideVaultError, Result};

/// Server status information returned by a Minecraft server.
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerStatus {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: Description,
}

/// Version information for the Minecraft server.
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: u16,
}

/// Player count and list information.
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayersInfo {
    pub max: u16,
    pub online: u16,
    #[serde(default)]
    pub sample: Vec<PlayerSample>,
}

/// Individual player information in the server list.
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

/// Server description/MOTD.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Description {
    String(String),
    Object { text: String },
}

impl Description {
    /// Get the text content of the description.
    pub fn text(&self) -> &str {
        match self {
            Description::String(s) => s,
            Description::Object { text } => text,
        }
    }
}

/// Ping a Minecraft server and retrieve its status.
///
/// # Arguments
///
/// * `address` - Server address in "host:port" format (e.g., "localhost:25565")
///
/// # Returns
///
/// Returns the server status information including version, player count, and description.
///
/// # Errors
///
/// Returns an error if the connection fails, times out, or the server responds with invalid data.
///
/// # Examples
///
/// ```no_run
/// use oxidevault::mc_server::ping_server;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let status = tokio::task::spawn_blocking(|| {
///     ping_server("localhost:25565")
/// }).await??;
///
/// println!("Players: {}/{}", status.players.online, status.players.max);
/// # Ok(())
/// # }
/// ```
pub fn ping_server(address: &str) -> Result<ServerStatus> {
    // Resolve address and connect with timeout
    let mut addrs = address.to_socket_addrs()
        .map_err(|e| OxideVaultError::ServerProtocol(format!("Failed to resolve address: {}", e)))?;

    let addr = addrs.next()
        .ok_or_else(|| OxideVaultError::ServerProtocol("Could not resolve address".to_string()))?;

    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
        .map_err(|e| OxideVaultError::ServerProtocol(format!("Connection failed: {}", e)))?;

    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    // Build handshake packet
    let mut handshake = Vec::new();
    write_varint(&mut handshake, 0)?; // Packet ID: handshake
    write_varint(&mut handshake, -1)?; // Protocol version (-1 for auto-detection)

    // Use the resolved IP address and port
    let host_str = addr.ip().to_string();
    let port = addr.port();

    write_string(&mut handshake, &host_str)?;
    handshake.write_all(&port.to_be_bytes())?; // Port
    write_varint(&mut handshake, 1)?; // Next state: status

    // Send handshake
    send_packet(&mut stream, &handshake)?;

    // Send status request
    let mut status_request = Vec::new();
    write_varint(&mut status_request, 0)?; // Packet ID: request
    send_packet(&mut stream, &status_request)?;

    // Read response
    let response = read_packet(&mut stream)?;
    let json_str = read_string(&response[1..])?;

    // Parse JSON response
    let status: ServerStatus = serde_json::from_str(&json_str)
        .map_err(|e| OxideVaultError::ServerProtocol(format!("Failed to parse server response: {}", e)))?;

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_server_invalid_address() {
        // Test with invalid address format
        let result = ping_server("invalid-address-no-port");
        assert!(result.is_err());
        
        // Test with non-resolvable address
        let result = ping_server("nonexistent.invalid.domain.test:25565");
        assert!(result.is_err());
    }

    #[test]
    fn test_ping_server_connection_refused() {
        // Test with localhost on a port that's likely closed
        // This should fail with connection refused
        let result = ping_server("127.0.0.1:1");
        assert!(result.is_err());
        match result {
            Err(OxideVaultError::ServerProtocol(msg)) => {
                assert!(msg.contains("Connection failed") || msg.contains("connection"));
            }
            Err(OxideVaultError::Io(_)) => {
                // Also acceptable - IO error for connection issues
            }
            _ => panic!("Expected ServerProtocol or Io error"),
        }
    }

    #[test]
    fn test_description_text() {
        let desc_string = Description::String("A Minecraft Server".to_string());
        assert_eq!(desc_string.text(), "A Minecraft Server");

        let desc_object = Description::Object {
            text: "Another Server".to_string(),
        };
        assert_eq!(desc_object.text(), "Another Server");
    }

    // Note: Testing successful ping_server connections requires a running Minecraft server
    // In a real CI/CD environment, you would either:
    // 1. Set up a test Minecraft server in your CI pipeline
    // 2. Use integration tests that run separately from unit tests
    // 3. Mock the TcpStream for more detailed testing
    //
    // Example test that would work with a real server:
    // #[test]
    // #[ignore] // Ignored by default, run with --ignored flag when server is available
    // fn test_ping_server_success() {
    //     let result = ping_server("localhost:25565");
    //     assert!(result.is_ok());
    //     let status = result.unwrap();
    //     assert!(status.players.max > 0);
    // }
}
