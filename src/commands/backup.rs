//! Backup command.
//!
//! Sends the most recent backup file from the configured backup folder through Discord.
//! Files are split into chunks to fit Discord's 25MB limit.
//! Already compressed files (.gz, .tgz, .zip, etc.) are not re-compressed.

use crate::types::{Context, Error};
use poise::serenity_prelude as serenity;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Send the most recent backup file from the backup folder.
///
/// Large backups are compressed with gzip and split into 24MB chunks for Discord compatibility.
#[poise::command(slash_command)]
pub async fn backup(
    context: Context<'_>,
) -> Result<(), Error> {
    // Global rate limiting: 2 hours cooldown between any uses
    const GLOBAL_COOLDOWN: Duration = Duration::from_secs(2 * 60 * 60);

    let global_last_backup = context.data().last_global_backup_time.read().await;
    if let Some(last_time) = *global_last_backup {
        let elapsed = last_time.elapsed();
        if elapsed < GLOBAL_COOLDOWN {
            let remaining = GLOBAL_COOLDOWN - elapsed;
            let hours = remaining.as_secs() / 3600;
            let minutes = (remaining.as_secs() % 3600) / 60;

            context.say(format!(
                "⏳ Backup command is globally on cooldown. Please wait {} hour{} and {} minute{}.",
                hours,
                if hours == 1 { "" } else { "s" },
                minutes,
                if minutes == 1 { "" } else { "s" }
            )).await?;
            return Ok(());
        }
    }
    drop(global_last_backup);

    // Per-user rate limiting: 1 day cooldown per user
    const COOLDOWN_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

    let user_id = context.author().id.get();
    let mut last_backup_map = context.data().last_backup_time.write().await;

    if let Some(last_time) = last_backup_map.get(&user_id) {
        let elapsed = last_time.elapsed();
        if elapsed < COOLDOWN_DURATION {
            let remaining = COOLDOWN_DURATION - elapsed;
            let hours = remaining.as_secs() / 3600;
            let minutes = (remaining.as_secs() % 3600) / 60;

            context.say(format!(
                "⏳ Backup command is on cooldown. Please wait {} hour{} and {} minute{}.",
                hours,
                if hours == 1 { "" } else { "s" },
                minutes,
                if minutes == 1 { "" } else { "s" }
            )).await?;
            return Ok(());
        }
    }

    // Update last backup time (both global and per-user)
    let now = Instant::now();
    last_backup_map.insert(user_id, now);
    drop(last_backup_map); // Release the lock

    let mut global_backup_time = context.data().last_global_backup_time.write().await;
    *global_backup_time = Some(now);
    drop(global_backup_time);

    // Defer reply since processing might take a while
    context.defer().await?;

    // Get backup folder from bot data
    let backup_folder = context.data().backup_folder.clone();

    // Find the most recent backup file
    let backup_file = tokio::task::spawn_blocking(move || {
        find_most_recent_backup(&backup_folder)
    }).await?;

    match backup_file {
        Some(file_path) => {
            // Get the file name for display
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("backup")
                .to_string();

            // Process the backup: compress and split
            let result = tokio::task::spawn_blocking(move || {
                process_backup(&file_path)
            }).await?;

            match result {
                Ok((chunks, original_size)) => {
                    // Send all chunks
                    let total_chunks = chunks.len();

                    let size_mb = original_size as f64 / (1024.0 * 1024.0);
                    context.say(format!(
                        "📦 Sending backup: **{}** ({:.2} MB, {} chunk{}).",
                        file_name,
                        size_mb,
                        total_chunks,
                        if total_chunks == 1 { "" } else { "s" }
                    )).await?;

                    for (index, chunk_data) in chunks.iter().enumerate() {
                        let chunk_name = if total_chunks == 1 {
                            file_name.clone()
                        } else {
                            format!("{}.part{:03}", file_name, index + 1)
                        };

                        let attachment = serenity::CreateAttachment::bytes(chunk_data.clone(), &chunk_name);

                        context.send(
                            poise::CreateReply::default()
                                .content(format!(
                                    "📦 Chunk {}/{}",
                                    index + 1,
                                    total_chunks
                                ))
                                .attachment(attachment)
                        ).await?;
                    }

                    // Send decompression instructions
                    if total_chunks > 1 {
                        context.say(format!(
                            "✅ Backup sent successfully!\n\n\
                            **To restore (Linux/macOS):**\n\
                            ```bash\n\
                            cat {}.part* > {}\n\
                            tar -xzf {}\n\
                            ```\n\
                            **To restore (Windows PowerShell):**\n\
                            ```powershell\n\
                            Get-Content {}.part* -Raw | Set-Content {} -Encoding Byte\n\
                            tar -xzf {}\n\
                            ```",
                            file_name, file_name, file_name,
                            file_name, file_name, file_name
                        )).await?;
                    } else {
                        context.say("✅ Backup sent successfully!").await?;
                    }
                }
                Err(e) => {
                    context.say(format!("❌ Failed to process backup: {}", e)).await?;
                }
            }
        }
        None => {
            context.say("❌ No backup found in the configured backup folder.").await?;
        }
    }

    Ok(())
}

/// Find the most recent backup file in the specified folder.
///
/// Returns the path to the most recent file (by modification time),
/// or None if no files are found.
fn find_most_recent_backup(backup_folder: &str) -> Option<PathBuf> {
    let path = PathBuf::from(backup_folder);

    // Check if the folder exists
    if !path.exists() || !path.is_dir() {
        return None;
    }

    // Read directory entries
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    // Find the most recent file
    let mut most_recent: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in entries.flatten() {
        let entry_path = entry.path();

        // Skip directories, only consider files
        if !entry_path.is_file() {
            continue;
        }

        // Get modification time
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                match &most_recent {
                    None => {
                        most_recent = Some((entry_path, modified));
                    }
                    Some((_, current_time)) => {
                        if modified > *current_time {
                            most_recent = Some((entry_path, modified));
                        }
                    }
                }
            }
        }
    }

    most_recent.map(|(path, _)| path)
}

/// Process backup: read and split into chunks.
/// Returns (chunks, original_file_size)
fn process_backup(file_path: &PathBuf) -> Result<(Vec<Vec<u8>>, usize), Box<dyn std::error::Error + Send + Sync>> {
    // Read the file
    let mut file = File::open(file_path)?;
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)?;

    let original_size = file_data.len();

    // Split into chunks (25 MB limit for Discord)
    const CHUNK_SIZE: usize = 24 * 1024 * 1024; // 24 MB to be safe
    let mut chunks = Vec::new();

    for chunk_data in file_data.chunks(CHUNK_SIZE) {
        chunks.push(chunk_data.to_vec());
    }

    Ok((chunks, original_size))
}
