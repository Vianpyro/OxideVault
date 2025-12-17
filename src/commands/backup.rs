//! Backup command.
//!
//! Publishes the most recent backup via an HTTPS link (served by your reverse proxy).
//! Avoids Discord file size limits by sharing a downloadable URL instead of attachments.

use crate::types::{Context, Error};
use rand::Rng;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Publish the most recent backup file and provide a download link.
///
/// The backup is published under a tokenized path served by your reverse proxy.
/// This approach avoids external size limits and keeps transfers on your own infrastructure.
///
/// Publishing is restricted to administrators to prevent unauthorized access to backups.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn backup(context: Context<'_>) -> Result<(), Error> {
    // Global rate limiting: 2 hours cooldown between all publishes
    const GLOBAL_COOLDOWN: Duration = Duration::from_secs(2 * 60 * 60);

    // Per-user rate limiting: 1 day cooldown between publishes by the same user
    const COOLDOWN_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

    let user_id = context.author().id.get();
    let mut last_backup_map = context.data().last_backup_time.write().await;

    if let Some(last_time) = last_backup_map.get(&user_id) {
        let elapsed = last_time.elapsed();
        if elapsed < COOLDOWN_DURATION {
            let remaining = COOLDOWN_DURATION - elapsed;
            let hours = remaining.as_secs() / 3600;
            let minutes = (remaining.as_secs() % 3600) / 60;

            context
                .say(format!(
                    "⏳ Backup command is on cooldown. Please wait {} hour{} and {} minute{}.",
                    hours,
                    if hours == 1 { "" } else { "s" },
                    minutes,
                    if minutes == 1 { "" } else { "s" }
                ))
                .await?;
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

            context
                .say(format!(
                    "⏳ Backup command is globally on cooldown. Please wait {} hour{} and {} minute{}.",
                    hours,
                    if hours == 1 { "" } else { "s" },
                    minutes,
                    if minutes == 1 { "" } else { "s" }
                ))
                .await?;
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

    // Get backup and publish settings
    let backup_folder = context.data().backup_folder.clone();
    let publish_root = context.data().backup_publish_root.clone();
    let publish_base_url = context.data().backup_public_base_url.clone();

    // Find the most recent backup file
    let backup_file = tokio::task::spawn_blocking(move || find_most_recent_backup(&backup_folder))
        .await?;

    let file_path = match backup_file {
        Some(p) => p,
        None => {
            context
                .say("❌ No backup found. The backup folder may not exist, is not accessible, or contains no files. Please check your BACKUP_FOLDER configuration.")
                .await?;
            return Ok(());
        }
    };

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("backup")
        .to_string();

    // Publish backup: create tokenized link (hard-link or copy for portability)
    let publish_result = tokio::task::spawn_blocking(move || {
        publish_backup(&file_path, &publish_root, &publish_base_url)
    })
    .await??;

    let size_mb = publish_result.size_bytes as f64 / (1024.0 * 1024.0);

    context
        .say(format!(
            "📦 Backup ready for download: **{}** ({:.2} MB)\n\
            🔗 Link: {}",
            file_name, size_mb, publish_result.url
        ))
        .await?;

    Ok(())
}

/// Locate the most recent backup file in the specified directory.
///
/// Returns the path to the most recently modified file by modification timestamp,
/// or None if the directory is missing, inaccessible, or contains no files.
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

struct PublishedBackup {
    url: String,
    local_path: PathBuf,
    size_bytes: u64,
}

/// Publish a backup by creating a tokenized subdirectory and hard-linking (or copying) the file.
/// Returns a PublishedBackup with the public URL and metadata.
fn publish_backup(
    file_path: &PathBuf,
    publish_root: &str,
    base_url: &str,
) -> Result<PublishedBackup, Box<dyn std::error::Error + Send + Sync>> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file name")?;

    // Generate a random 12-character token for obfuscation and easy revocation
    let mut rng = rand::thread_rng();
    let token: String = (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHANUMERIC.len());
            ALPHANUMERIC[idx] as char
        })
        .collect();

    let target_dir = PathBuf::from(publish_root).join(&token);
    fs::create_dir_all(&target_dir)?;

    let target_path = target_dir.join(file_name);

    // Attempt hard-link for efficiency; fall back to copy if on different filesystems
    match fs::hard_link(file_path, &target_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "Warning: Failed to create hard link from '{}' to '{}': {}. Falling back to file copy.",
                file_path.display(),
                target_path.display(),
                e
            );
            fs::copy(file_path, &target_path)?;
        }
    }

    let meta = fs::metadata(file_path)?;
    let size_bytes = meta.len();

    let base = base_url.trim_end_matches('/');
    let url = format!("{}/{}/{}", base, token, file_name);

    Ok(PublishedBackup {
        url,
        local_path: target_path,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Verify a backup file is located and matches the expected filename.
    fn assert_backup_found(temp_dir: &TempDir, expected_name: &str) {
        let result = find_most_recent_backup(temp_dir.path().to_str().unwrap());
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), expected_name);
    }

    /// Helper to set up common test fixtures for publish_backup tests.
    fn setup_publish_test() -> (TempDir, String, String) {
        let temp_dir = TempDir::new().unwrap();
        let publish_root = temp_dir.path().join("public");
        let base_url = "http://example.com/backups".to_string();

        // Create a sample backup file
        let file_path = temp_dir.path().join("backup1.tgz");
        fs::write(&file_path, b"test data").unwrap();

        (temp_dir, publish_root.to_str().unwrap().to_string(), base_url)
    }

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

        assert_backup_found(&temp_dir, "backup1.tgz");
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

        assert_backup_found(&temp_dir, "backup.tgz");
    }

    #[test]
    fn test_publish_backup_creates_tokenized_copy() {
        let (temp_dir, publish_root, base_url) = setup_publish_test();
        let file_path = temp_dir.path().join("backup1.tgz");

        let result = publish_backup(&file_path, &publish_root, &base_url);
        assert!(result.is_ok());

        let published = result.unwrap();
        assert!(published.url.contains("http://example.com/backups"));

        // Ensure file exists at published path
        assert!(published.local_path.exists());
        let metadata = fs::metadata(&published.local_path).unwrap();
        assert_eq!(metadata.len(), b"test data".len() as u64);
    }

    #[test]
    fn test_publish_backup_invalid_file() {
        let (temp_dir, publish_root, base_url) = setup_publish_test();

        // Try to publish a non-existent file
        let file_path = temp_dir.path().join("nonexistent.tgz");

        let result = publish_backup(&file_path, &publish_root, &base_url);
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_backup_token_uniqueness() {
        let (temp_dir, publish_root, base_url) = setup_publish_test();
        let file_path = temp_dir.path().join("backup1.tgz");

        // Publish multiple times and ensure tokens are different
        let result1 = publish_backup(&file_path, &publish_root, &base_url).unwrap();
        let result2 = publish_backup(&file_path, &publish_root, &base_url).unwrap();
        let result3 = publish_backup(&file_path, &publish_root, &base_url).unwrap();

        assert_ne!(result1.url, result2.url, "Tokens should be unique");
        assert_ne!(result1.url, result3.url, "Tokens should be unique");
        assert_ne!(result2.url, result3.url, "Tokens should be unique");
    }

    #[test]
    fn test_publish_backup_url_format() {
        let temp_dir = TempDir::new().unwrap();
        let publish_root = temp_dir.path().join("public");

        // Create a sample backup file
        let file_path = temp_dir.path().join("backup1.tgz");
        fs::write(&file_path, b"test data").unwrap();

        // Test with URL without trailing slash
        let base_url1 = "http://example.com/backups";
        let result1 = publish_backup(&file_path, publish_root.to_str().unwrap(), base_url1).unwrap();
        assert!(!result1.url.contains("//backups"), "Should not have double slashes");
        assert!(result1.url.ends_with("/backup1.tgz"), "Should end with filename");

        // Test with URL with trailing slash
        let base_url2 = "http://example.com/backups/";
        let result2 = publish_backup(&file_path, publish_root.to_str().unwrap(), base_url2).unwrap();
        assert!(!result2.url.contains("backups//"), "Should not have double slashes");
        assert!(result2.url.ends_with("/backup1.tgz"), "Should end with filename");
    }

    #[test]
    fn test_publish_backup_size_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let publish_root = temp_dir.path().join("public");
        let base_url = "http://example.com/backups";

        // Create a sample backup file with known size
        let test_data = vec![0u8; 1024 * 10]; // 10 KB
        let file_path = temp_dir.path().join("backup1.tgz");
        fs::write(&file_path, &test_data).unwrap();

        let result = publish_backup(&file_path, publish_root.to_str().unwrap(), base_url).unwrap();
        assert_eq!(result.size_bytes, test_data.len() as u64);
    }

    #[test]
    fn test_publish_backup_preserves_content() {
        let temp_dir = TempDir::new().unwrap();
        let publish_root = temp_dir.path().join("public");
        let base_url = "http://example.com/backups";

        // Create a sample backup file with specific content
        let test_content = b"This is a test backup file with specific content";
        let file_path = temp_dir.path().join("backup1.tgz");
        fs::write(&file_path, test_content).unwrap();

        let result = publish_backup(&file_path, publish_root.to_str().unwrap(), base_url).unwrap();

        // Read the published file and verify content
        let published_content = fs::read(&result.local_path).unwrap();
        assert_eq!(published_content, test_content, "Published file should have same content as original");
    }
}
