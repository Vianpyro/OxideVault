//! Low-level Minecraft server protocol implementation.
//!
//! This module handles the binary protocol for communicating with Minecraft servers,
//! including VarInt encoding/decoding and packet serialization.

use std::io::{Read, Write};
use std::net::TcpStream;

/// Send a packet to the Minecraft server.
///
/// Packets are prefixed with their length as a VarInt, followed by the packet data.
pub fn send_packet(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    let mut packet = Vec::new();
    write_varint(&mut packet, data.len() as i32)?;
    packet.extend_from_slice(data);
    stream.write_all(&packet)?;
    Ok(())
}

/// Read a complete packet from the Minecraft server.
///
/// Returns the packet data without the length prefix.
pub fn read_packet(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let length = read_varint(stream)?;
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer)?;
    Ok(buffer)
}

/// Write a VarInt to a buffer.
///
/// VarInts are variable-length encoded integers used in the Minecraft protocol.
pub fn write_varint(buf: &mut Vec<u8>, value: i32) -> std::io::Result<()> {
    // Convert to unsigned for proper bit manipulation with negative numbers
    let mut value = value as u32;

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

/// Read a VarInt from a TCP stream.
pub fn read_varint(stream: &mut TcpStream) -> std::io::Result<i32> {
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

/// Read a VarInt from a byte slice.
///
/// Returns the decoded value and the number of bytes consumed.
pub fn read_varint_from_slice(data: &[u8]) -> std::io::Result<(i32, usize)> {
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

/// Process a single byte of a VarInt.
///
/// Returns `Ok(true)` if the VarInt is complete, `Ok(false)` if more bytes are needed.
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

/// Write a string to a buffer using Minecraft protocol format.
///
/// Strings are prefixed with their length as a VarInt, followed by UTF-8 bytes.
pub fn write_string(buf: &mut Vec<u8>, s: &str) -> std::io::Result<()> {
    write_varint(buf, s.len() as i32)?;
    buf.extend_from_slice(s.as_bytes());
    Ok(())
}

/// Read a string from a byte slice using Minecraft protocol format.
///
/// Returns the decoded string.
pub fn read_string(data: &[u8]) -> std::io::Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encoding() {
        let mut buf = Vec::new();
        write_varint(&mut buf, 0).unwrap();
        assert_eq!(buf, vec![0]);

        let mut buf = Vec::new();
        write_varint(&mut buf, 127).unwrap();
        assert_eq!(buf, vec![127]);

        let mut buf = Vec::new();
        write_varint(&mut buf, 128).unwrap();
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn test_varint_decoding() {
        assert_eq!(read_varint_from_slice(&[0]).unwrap(), (0, 1));
        assert_eq!(read_varint_from_slice(&[127]).unwrap(), (127, 1));
        assert_eq!(read_varint_from_slice(&[0x80, 0x01]).unwrap(), (128, 2));
    }

    #[test]
    fn test_string_encoding() {
        let mut buf = Vec::new();
        write_string(&mut buf, "test").unwrap();
        assert_eq!(buf, vec![4, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_string_decoding() {
        let data = vec![4, b't', b'e', b's', b't'];
        assert_eq!(read_string(&data).unwrap(), "test");
    }
}
