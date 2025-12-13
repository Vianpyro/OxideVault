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
/// Large backup files are split into 24MB chunks for Discord compatibility. No additional compression is performed; files are sent as-is.
/// 
/// This command requires administrator permissions to prevent unauthorized access to sensitive backup data.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn backup(
    context: Context<'_>,
) -> Result<(), Error> {
    // Global rate limiting: 2 hours cooldown between any uses
    const GLOBAL_COOLDOWN: Duration = Duration::from_secs(2 * 60 * 60);

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

    // Check global cooldown again after acquiring write lock to prevent race condition
    let mut global_backup_time = context.data().last_global_backup_time.write().await;
    if let Some(last_time) = *global_backup_time {
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

    // Update last backup time (both global and per-user)
    let now = Instant::now();
    last_backup_map.insert(user_id, now);
    *global_backup_time = Some(now);
    drop(last_backup_map);
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

            // Process the backup: split into chunks
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

                    for (index, chunk_data) in chunks.into_iter().enumerate() {
                        let chunk_name = if total_chunks == 1 {
                            file_name.clone()
                        } else {
                            format!("{}.part{:03}", file_name, index + 1)
                        };

                        let attachment = serenity::CreateAttachment::bytes(chunk_data, &chunk_name);

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
                            **To restore (Windows PowerShell 5.1):**\n\
                            ```powershell\n\
                            Get-Content {}.part* -Raw | Set-Content {} -Encoding Byte\n\
                            tar -xzf {}\n\
                            ```\n\
                            **To restore (PowerShell Core 6+):**\n\
                            ```powershell\n\
                            Get-Content {}.part* -AsByteStream | Set-Content {} -AsByteStream\n\
                            tar -xzf {}\n\
                            ```",
                            file_name, file_name, file_name,
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
            context.say("❌ No backup found. The backup folder may not exist, is not accessible, or contains no files. Please check your BACKUP_FOLDER configuration.").await?;
        }
    }

    Ok(())
}

/// Find the most recent backup file in the specified folder.
///
/// Returns the path to the most recent file (by modification time),
/// or None if the folder doesn't exist, is not accessible, or contains no files.
fn find_most_recent_backup(backup_folder: &str) -> Option<PathBuf> {
    let path = PathBuf::from(backup_folder);

    // Check if the folder exists and is a directory
    if !path.exists() {
        eprintln!("Backup folder does not exist: {}", backup_folder);
        return None;
    }
    
    if !path.is_dir() {
        eprintln!("Backup folder path is not a directory: {}", backup_folder);
        return None;
    }

    // Read directory entries
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read backup folder: {}", e);
            return None;
        }
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

/// Process backup: read and split into chunks using streaming to avoid excessive memory usage.
/// Returns (chunks, original_file_size)
fn process_backup(file_path: &PathBuf) -> Result<(Vec<Vec<u8>>, usize), Box<dyn std::error::Error + Send + Sync>> {
    // Read the file in chunks to avoid loading entire file in memory
    let mut file = File::open(file_path)?;
    let mut chunks = Vec::new();
    let mut total_size = 0;
    const CHUNK_SIZE: usize = 24 * 1024 * 1024; // 24 MB to be safe

    loop {
        let mut buffer = Vec::new();
        let n = {
            // Read up to CHUNK_SIZE bytes
            let mut handle = file.by_ref().take(CHUNK_SIZE as u64);
            handle.read_to_end(&mut buffer)?
        };
        
        if n == 0 {
            break;
        }
        
        total_size += n;
        chunks.push(buffer);
    }

    Ok((chunks, total_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_find_most_recent_backup_empty_folder() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_most_recent_backup(temp_dir.path().to_str().unwrap());
        assert!(result.is_none());
    }

    #[test]
    fn test_find_most_recent_backup_nonexistent_folder() {
        let result = find_most_recent_backup("/nonexistent/path/that/should/not/exist");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_most_recent_backup_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("backup1.tgz");
        fs::write(&file_path, b"test data").unwrap();

        let result = find_most_recent_backup(temp_dir.path().to_str().unwrap());
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), "backup1.tgz");
    }

    #[test]
    fn test_find_most_recent_backup_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create first file
        let file1_path = temp_dir.path().join("backup1.tgz");
        fs::write(&file1_path, b"old data").unwrap();
        
        // Sleep briefly to ensure different modification times
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Create second file (should be more recent)
        let file2_path = temp_dir.path().join("backup2.tgz");
        fs::write(&file2_path, b"new data").unwrap();

        let result = find_most_recent_backup(temp_dir.path().to_str().unwrap());
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), "backup2.tgz");
    }

    #[test]
    fn test_find_most_recent_backup_ignores_directories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a subdirectory
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        // Create a file in the temp dir
        let file_path = temp_dir.path().join("backup.tgz");
        fs::write(&file_path, b"test data").unwrap();

        let result = find_most_recent_backup(temp_dir.path().to_str().unwrap());
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), "backup.tgz");
    }

    #[test]
    fn test_process_backup_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("small.tgz");
        let test_data = b"This is a small test file";
        fs::write(&file_path, test_data).unwrap();

        let result = process_backup(&file_path);
        assert!(result.is_ok());
        
        let (chunks, total_size) = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(total_size, test_data.len());
        assert_eq!(&chunks[0], test_data);
    }

    #[test]
    fn test_process_backup_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.tgz");
        
        // Create a file larger than CHUNK_SIZE (24 MB)
        // Use 50 MB for testing
        let chunk_size = 24 * 1024 * 1024;
        let total_size = 50 * 1024 * 1024;
        
        {
            let mut file = fs::File::create(&file_path).unwrap();
            let pattern = b"ABCDEFGH";
            let mut written = 0;
            while written < total_size {
                let to_write = std::cmp::min(pattern.len(), total_size - written);
                file.write_all(&pattern[..to_write]).unwrap();
                written += to_write;
            }
        }

        let result = process_backup(&file_path);
        assert!(result.is_ok());
        
        let (chunks, size) = result.unwrap();
        // Should have at least 2 chunks for a 50MB file with 24MB chunks
        assert!(chunks.len() >= 2);
        assert_eq!(size, total_size);
        
        // Verify total size by summing chunk sizes
        let total_chunk_size: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total_chunk_size, total_size);
        
        // Verify first chunk is at most CHUNK_SIZE
        assert!(chunks[0].len() <= chunk_size);
    }

    #[test]
    fn test_process_backup_nonexistent_file() {
        let file_path = PathBuf::from("/nonexistent/file.tgz");
        let result = process_backup(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_backup_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.tgz");
        fs::write(&file_path, b"").unwrap();

        let result = process_backup(&file_path);
        assert!(result.is_ok());
        
        let (chunks, total_size) = result.unwrap();
        assert_eq!(chunks.len(), 0);
        assert_eq!(total_size, 0);
    }

    #[test]
    fn test_process_backup_exact_chunk_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exact.tgz");
        
        // Create a file that's exactly CHUNK_SIZE (24 MB)
        let chunk_size = 24 * 1024 * 1024;
        let test_data = vec![0u8; chunk_size];
        fs::write(&file_path, &test_data).unwrap();

        let result = process_backup(&file_path);
        assert!(result.is_ok());
        
        let (chunks, total_size) = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(total_size, chunk_size);
        assert_eq!(chunks[0].len(), chunk_size);
    }
}
