use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerStatus {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: Description,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayersInfo {
    pub max: u16,
    pub online: u16,
    #[serde(default)]
    pub sample: Vec<PlayerSample>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Description {
    String(String),
    Object { text: String },
}

impl Description {
    pub fn text(&self) -> &str {
        match self {
            Description::String(s) => s,
            Description::Object { text } => text,
        }
    }
}

// Ping a Minecraft server and retrieve its status
pub fn ping_server(address: &str) -> Result<ServerStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(address)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Build handshake packet
    let mut handshake = Vec::new();
    write_varint(&mut handshake, 0)?; // Packet ID: handshake
    write_varint(&mut handshake, 47)?; // Protocol version (1.8+)

    // Extract host and port from address
    let (host, port) = if address.contains(':') {
        let parts: Vec<&str> = address.split(':').collect();
        (parts[0], parts.get(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(25565))
    } else {
        (address, 25565)
    };

    write_string(&mut handshake, host)?;
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
    let status: ServerStatus = serde_json::from_str(&json_str)?;

    Ok(status)
}

fn send_packet(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    let mut packet = Vec::new();
    write_varint(&mut packet, data.len() as i32)?;
    packet.extend_from_slice(data);
    stream.write_all(&packet)?;
    Ok(())
}

fn read_packet(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let length = read_varint(stream)?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn write_varint(buf: &mut Vec<u8>, mut value: i32) -> std::io::Result<()> {
    loop {
        let mut temp = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0x80;
        }
        buf.push(temp);
        if value == 0 {
            break;
        }
    }
    Ok(())
}

// Process a VarInt byte and update the result and shift values
// Returns Ok(true) if VarInt is complete, Ok(false) if more bytes needed
fn process_varint_byte(byte: u8, result: &mut i32, shift: &mut i32) -> std::io::Result<bool> {
    *result |= ((byte & 0x7F) as i32) << *shift;
    if byte & 0x80 == 0 {
        return Ok(true);
    }
    *shift += 7;
    if *shift >= 35 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "VarInt is too big",
        ));
    }
    Ok(false)
}

fn read_varint(stream: &mut TcpStream) -> std::io::Result<i32> {
    let mut result = 0;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte)?;
        if process_varint_byte(byte[0], &mut result, &mut shift)? {
            break;
        }
    }
    Ok(result)
}

fn write_string(buf: &mut Vec<u8>, s: &str) -> std::io::Result<()> {
    write_varint(buf, s.len() as i32)?;
    buf.extend_from_slice(s.as_bytes());
    Ok(())
}

fn read_string(data: &[u8]) -> std::io::Result<String> {
    let (len, offset) = read_varint_from_slice(data)?;
    if offset + len as usize > data.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "String length exceeds data size",
        ));
    }
    let s = String::from_utf8_lossy(&data[offset..offset + len as usize]);
    Ok(s.to_string())
}

fn read_varint_from_slice(data: &[u8]) -> std::io::Result<(i32, usize)> {
    let mut result = 0;
    let mut shift = 0;
    let mut pos = 0;
    loop {
        if pos >= data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Unexpected end of data while reading VarInt",
            ));
        }
        let byte = data[pos];
        pos += 1;
        if process_varint_byte(byte, &mut result, &mut shift)? {
            break;
        }
    }
    Ok((result, pos))
}
