//! Pl3xmap integration.
//!
//! This module provides functions for converting settings into Pl3xmap-compatible
//! decimal color formats.

use crate::error::{OxideVaultError, Result};

/// Validate that a parameter is within the 0.0 to 1.0 range.
///
/// # Arguments
///
/// * `value` - The value to validate
/// * `param_name` - The name of the parameter for error messages
///
/// # Returns
///
/// Returns Ok(()) if the value is valid, or an error if out of range.
fn validate_range_0_to_1(value: f32, param_name: &str) -> Result<()> {
    if !(0.0..=1.0).contains(&value) {
        return Err(OxideVaultError::InvalidInput(
            format!("{} must be between 0.0 and 1.0, got {}", param_name, value)
        ));
    }
    Ok(())
}

/// Pl3xmap color settings converted to decimal format.
#[derive(Debug, Clone)]
pub struct Pl3xmapColors {
    /// Saturation value (0.0 to 1.0) converted to decimal (0-255)
    pub saturation: u8,
    /// Lightness value (0.0 to 1.0) converted to decimal (0-255)
    pub lightness: u8,
    /// Stroke color in decimal ARGB format
    pub stroke_color: u32,
    /// Fill color in decimal ARGB format
    pub fill_color: u32,
}

/// Convert Pl3xmap settings into decimal colors.
///
/// # Arguments
///
/// * `saturation` - Saturation value (0.0 to 1.0)
/// * `lightness` - Lightness value (0.0 to 1.0)
/// * `stroke_hex` - Stroke color as hex string (e.g., "FF5733" or "#FF5733")
/// * `stroke_opacity` - Stroke opacity (0.0 to 1.0)
/// * `fill_hex` - Fill color as hex string (e.g., "00AAFF" or "#00AAFF")
/// * `fill_opacity` - Fill opacity (0.0 to 1.0)
///
/// # Returns
///
/// Returns `Pl3xmapColors` with all values converted to decimal format.
///
/// # Errors
///
/// Returns an error if hex color strings are invalid or values are out of range.
///
/// # Examples
///
/// ```no_run
/// use oxidevault::pl3xmap::convert_pl3xmap_colors;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let colors = convert_pl3xmap_colors(
///     0.8,
///     0.5,
///     "FF5733",
///     0.75,
///     "00AAFF",
///     0.5
/// )?;
///
/// println!("Stroke color (decimal): {}", colors.stroke_color);
/// println!("Fill color (decimal): {}", colors.fill_color);
/// # Ok(())
/// # }
/// ```
pub fn convert_pl3xmap_colors(
    saturation: f32,
    lightness: f32,
    stroke_hex: &str,
    stroke_opacity: f32,
    fill_hex: &str,
    fill_opacity: f32,
) -> Result<Pl3xmapColors> {
    // Validate ranges
    validate_range_0_to_1(saturation, "Saturation")?;
    validate_range_0_to_1(lightness, "Lightness")?;
    validate_range_0_to_1(stroke_opacity, "Stroke opacity")?;
    validate_range_0_to_1(fill_opacity, "Fill opacity")?;

    // Convert saturation and lightness to 0-255 range
    let saturation_decimal = (saturation * 255.0).round() as u8;
    let lightness_decimal = (lightness * 255.0).round() as u8;

    // Parse hex colors and combine with opacity
    let stroke_color = parse_hex_to_argb(stroke_hex, stroke_opacity)?;
    let fill_color = parse_hex_to_argb(fill_hex, fill_opacity)?;

    Ok(Pl3xmapColors {
        saturation: saturation_decimal,
        lightness: lightness_decimal,
        stroke_color,
        fill_color,
    })
}

/// Parse hex color string and opacity into ARGB decimal format.
///
/// # Arguments
///
/// * `hex` - Hex color string (e.g., "FF5733" or "#FF5733")
/// * `opacity` - Opacity value (0.0 to 1.0)
///
/// # Returns
///
/// Returns a u32 in ARGB format where:
/// - Bits 24-31: Alpha (opacity)
/// - Bits 16-23: Red
/// - Bits 8-15: Green
/// - Bits 0-7: Blue
///
/// # Errors
///
/// Returns an error if the hex string is invalid.
fn parse_hex_to_argb(hex: &str, opacity: f32) -> Result<u32> {
    // Remove '#' prefix if present
    let hex = hex.trim_start_matches('#');

    // Validate hex string length
    if hex.len() != 6 {
        return Err(OxideVaultError::InvalidInput(
            format!("Hex color must be 6 characters, got: {}", hex)
        ));
    }

    // Parse RGB components
    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| OxideVaultError::InvalidInput(
            format!("Invalid hex color (red component): {}", hex)
        ))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| OxideVaultError::InvalidInput(
            format!("Invalid hex color (green component): {}", hex)
        ))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| OxideVaultError::InvalidInput(
            format!("Invalid hex color (blue component): {}", hex)
        ))?;

    // Convert opacity to alpha (0-255)
    let a = (opacity * 255.0).round() as u8;

    // Combine into ARGB format
    let argb = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);

    Ok(argb)
}

/// Validate a hex color string without opacity.
///
/// # Arguments
///
/// * `hex` - Hex color string (e.g., "FF5733" or "#FF5733")
///
/// # Returns
///
/// Returns Ok(()) if the hex string is valid, or an error if invalid.
///
/// # Examples
///
/// ```no_run
/// use oxidevault::pl3xmap::validate_hex_color;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// validate_hex_color("FF5733")?;  // Valid
/// validate_hex_color("#00AAFF")?; // Valid with # prefix
/// # Ok(())
/// # }
/// ```
pub fn validate_hex_color(hex: &str) -> Result<()> {
    // Use parse_hex_to_argb with a dummy opacity to validate
    parse_hex_to_argb(hex, 1.0)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_to_argb() {
        // Test basic color conversion with full opacity
        let color = parse_hex_to_argb("FF5733", 1.0).unwrap();
        assert_eq!(color, 0xFFFF5733); // Alpha=255, R=255, G=87, B=51

        // Test with half opacity
        let color = parse_hex_to_argb("00AAFF", 0.5).unwrap();
        assert_eq!(color, 0x7F00AAFF); // Alpha=127, R=0, G=170, B=255

        // Test with # prefix
        let color = parse_hex_to_argb("#123456", 0.75).unwrap();
        assert_eq!(color, 0xBF123456); // Alpha=191, R=18, G=52, B=86

        // Test with zero opacity
        let color = parse_hex_to_argb("FFFFFF", 0.0).unwrap();
        assert_eq!(color, 0x00FFFFFF); // Alpha=0, R=255, G=255, B=255
    }

    #[test]
    fn test_parse_hex_to_argb_invalid() {
        // Test invalid hex length
        assert!(parse_hex_to_argb("FFF", 1.0).is_err());
        assert!(parse_hex_to_argb("FF5733AA", 1.0).is_err());

        // Test invalid hex characters
        assert!(parse_hex_to_argb("GGGGGG", 1.0).is_err());
        assert!(parse_hex_to_argb("ZZZZZZ", 1.0).is_err());
    }

    #[test]
    fn test_convert_pl3xmap_colors() {
        let colors = convert_pl3xmap_colors(
            0.8,      // saturation
            0.5,      // lightness
            "FF5733", // stroke hex
            0.75,     // stroke opacity
            "00AAFF", // fill hex
            0.5       // fill opacity
        ).unwrap();

        assert_eq!(colors.saturation, 204); // 0.8 * 255 = 204
        assert_eq!(colors.lightness, 128);  // 0.5 * 255 = 127.5 -> 128
        assert_eq!(colors.stroke_color, 0xBFFF5733); // Alpha=191, R=255, G=87, B=51
        assert_eq!(colors.fill_color, 0x7F00AAFF);   // Alpha=127, R=0, G=170, B=255
    }

    #[test]
    fn test_convert_pl3xmap_colors_edge_cases() {
        // Test with minimum values
        let colors = convert_pl3xmap_colors(
            0.0,
            0.0,
            "000000",
            0.0,
            "000000",
            0.0
        ).unwrap();

        assert_eq!(colors.saturation, 0);
        assert_eq!(colors.lightness, 0);
        assert_eq!(colors.stroke_color, 0x00000000);
        assert_eq!(colors.fill_color, 0x00000000);

        // Test with maximum values
        let colors = convert_pl3xmap_colors(
            1.0,
            1.0,
            "FFFFFF",
            1.0,
            "FFFFFF",
            1.0
        ).unwrap();

        assert_eq!(colors.saturation, 255);
        assert_eq!(colors.lightness, 255);
        assert_eq!(colors.stroke_color, 0xFFFFFFFF);
        assert_eq!(colors.fill_color, 0xFFFFFFFF);
    }

    #[test]
    fn test_convert_pl3xmap_colors_invalid_ranges() {
        // Test invalid saturation
        assert!(convert_pl3xmap_colors(1.5, 0.5, "FF5733", 0.5, "00AAFF", 0.5).is_err());
        assert!(convert_pl3xmap_colors(-0.1, 0.5, "FF5733", 0.5, "00AAFF", 0.5).is_err());

        // Test invalid lightness
        assert!(convert_pl3xmap_colors(0.5, 1.5, "FF5733", 0.5, "00AAFF", 0.5).is_err());
        assert!(convert_pl3xmap_colors(0.5, -0.1, "FF5733", 0.5, "00AAFF", 0.5).is_err());

        // Test invalid stroke opacity
        assert!(convert_pl3xmap_colors(0.5, 0.5, "FF5733", 1.5, "00AAFF", 0.5).is_err());
        assert!(convert_pl3xmap_colors(0.5, 0.5, "FF5733", -0.1, "00AAFF", 0.5).is_err());

        // Test invalid fill opacity
        assert!(convert_pl3xmap_colors(0.5, 0.5, "FF5733", 0.5, "00AAFF", 1.5).is_err());
        assert!(convert_pl3xmap_colors(0.5, 0.5, "FF5733", 0.5, "00AAFF", -0.1).is_err());

        // Test invalid hex colors
        assert!(convert_pl3xmap_colors(0.5, 0.5, "INVALID", 0.5, "00AAFF", 0.5).is_err());
        assert!(convert_pl3xmap_colors(0.5, 0.5, "FF5733", 0.5, "BAD", 0.5).is_err());
    }
}
