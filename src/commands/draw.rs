//! Draw command for testing Pl3xmap input JSON output.
//!
//! This command accepts coordinates, radius, scale and colors, validates them
//! against environment-provided bounds and allowed values, then logs a JSON
//! representation to the console.

use crate::types::{Context, Error};
use serde_json::json;
use std::env;

/// Create a draw JSON from given parameters and log it to stdout.
///
/// - `x`, `y` must be between environment-provided min/max values named
///   `DRAW_MIN_X`, `DRAW_MAX_X`, `DRAW_MIN_Y`, `DRAW_MAX_Y` (all integers).
/// - `scale` must be one of: tiny, small, normal, large, huge, mega, giga.
/// - `colors` must be two RGB hex values separated by a comma: `stroke,fill`.
#[poise::command(slash_command)]
pub async fn draw(
    context: Context<'_>,
    #[description = "X coordinate"]
    x: i32,
    #[description = "Y coordinate"]
    y: i32,
    #[description = "Radius (positive integer)"]
    radius: i32,
    #[description = "Scale: tiny|small|normal|large|huge|mega|giga"]
    scale: String,
    #[description = "Colors as two hex RGB values 'stroke,fill' (e.g. FF5733,00AAFF)"]
    colors: String,
) -> Result<(), Error> {
    // Allowed scales
    let allowed_scales = ["tiny", "small", "normal", "large", "huge", "mega", "giga"];
    if !allowed_scales.contains(&scale.as_str()) {
        context
            .say(format!(
                "❌ Invalid scale '{}'. Allowed: {}",
                scale,
                allowed_scales.join(", ")
            ))
            .await?;
        return Ok(());
    }

    // Validate radius
    if radius <= 0 {
        context.say("❌ Radius must be a positive integer.").await?;
        return Ok(());
    }

    // Parse and validate colors
    let parts: Vec<&str> = colors.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        context
            .say("❌ `colors` must be two hex RGB values separated by a comma, e.g. FF5733,00AAFF")
            .await?;
        return Ok(());
    }

    let stroke = parts[0];
    let fill = parts[1];

    fn valid_hex6(s: &str) -> bool {
        if s.len() != 6 { return false; }
        s.chars().all(|c| c.is_ascii_hexdigit())
    }

    if !valid_hex6(stroke) || !valid_hex6(fill) {
        context
            .say("❌ Colors must be 6-character hex RGB values (0-9, A-F).")
            .await?;
        return Ok(());
    }

    // Read environment bounds
    macro_rules! read_bound {
        ($name:expr) => {{
            match env::var($name) {
                Ok(v) => match v.parse::<i32>() {
                    Ok(n) => Ok(n),
                    Err(_) => Err(format!("Environment var {} is not a valid integer: {}", $name, v)),
                },
                Err(_) => Err(format!("Missing environment variable: {}", $name)),
            }
        }};
    }

    let min_x = match read_bound!("DRAW_MIN_X") {
        Ok(v) => v,
        Err(e) => {
            context.say(format!("❌ {}", e)).await?;
            return Ok(());
        }
    };
    let max_x = match read_bound!("DRAW_MAX_X") {
        Ok(v) => v,
        Err(e) => {
            context.say(format!("❌ {}", e)).await?;
            return Ok(());
        }
    };
    let min_y = match read_bound!("DRAW_MIN_Y") {
        Ok(v) => v,
        Err(e) => {
            context.say(format!("❌ {}", e)).await?;
            return Ok(());
        }
    };
    let max_y = match read_bound!("DRAW_MAX_Y") {
        Ok(v) => v,
        Err(e) => {
            context.say(format!("❌ {}", e)).await?;
            return Ok(());
        }
    };

    if x < min_x || x > max_x {
        context
            .say(format!(
                "❌ `x` out of range ({}..={}): got {}",
                min_x, max_x, x
            ))
            .await?;
        return Ok(());
    }

    if y < min_y || y > max_y {
        context
            .say(format!(
                "❌ `y` out of range ({}..={}): got {}",
                min_y, max_y, y
            ))
            .await?;
        return Ok(());
    }

    // Build JSON and log it
    let obj = json!({
        "x": x,
        "y": y,
        "radius": radius,
        "scale": scale,
        "stroke": stroke,
        "fill": fill,
    });

    println!("[draw] {}", obj.to_string());

    context.say("✅ Draw JSON logged to console.").await?;

    Ok(())
}
