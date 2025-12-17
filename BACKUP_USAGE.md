# Backup Publishing Guide

## Overview

OxideVault publishes the most recent backup as an HTTPS link served by your reverse proxy. This approach avoids external size limits and keeps all transfers on your own infrastructure.

## Configuration (.env)

Default values (designed for a Docker volume mounted at `/backups`):

```bash
# Where your backups are stored
BACKUP_FOLDER=/backups

# Where the bot publishes tokenized links (served by your reverse proxy)
BACKUP_PUBLISH_ROOT=/backups/public

# Public URL served by your reverse proxy (Caddy/NGINX)
BACKUP_PUBLIC_BASE_URL=https://drop.example.com/backups
```

Ensure your reverse proxy serves `BACKUP_PUBLISH_ROOT` at `BACKUP_PUBLIC_BASE_URL` (see README for Caddy example).

## How Backup Publishing Works

1. Scans `BACKUP_FOLDER` to find the most recent backup file
2. Creates a tokenized directory in `BACKUP_PUBLISH_ROOT` and hard-links (or copies if on a different filesystem) the file there
3. Returns:
   - A secure download link with embedded token
   - Ready-to-copy commands for Linux/macOS and Windows restoration

## Example

After triggering a backup, you receive:
```md
📦 Backup ready for download: world_2024-12-13.tgz (1234.56 MB)
🔗 Link: https://drop.example.com/backups/abc123/world_2024-12-13.tgz
```

## Download and Restore

**Linux / macOS**
```bash
curl -L "https://drop.example.com/backups/abc123/world_2024-12-13.tgz" -o world_2024-12-13.tgz
tar -xzf world_2024-12-13.tgz
```

**Windows (PowerShell)**
```powershell
Invoke-WebRequest -Uri "https://drop.example.com/backups/abc123/world_2024-12-13.tgz" -OutFile world_2024-12-13.tgz
tar -xzf world_2024-12-13.tgz
```

## Troubleshooting

- **Link inaccessible:**
  - Verify the reverse proxy is serving `BACKUP_PUBLISH_ROOT` at the correct path
  - Check that `BACKUP_PUBLIC_BASE_URL` is set correctly and matches your reverse proxy configuration
  - Confirm the tokenized directory still exists in `BACKUP_PUBLISH_ROOT`
- **`/backups` directory missing:**
  - Ensure the volume is properly mounted in your deployment
  - Verify the application has read access to backups and write access to the publish root

## Security Notes

- **Token-based access:** Each published backup uses a random 12-character token in its URL path. While this provides obfuscation, it is not cryptographic security.
- **Additional protection:** Consider adding layer 7 security at your reverse proxy (Basic Auth, IP allowlisting, rate limiting).
- **Access revocation:** Delete the tokenized directory from `BACKUP_PUBLISH_ROOT` to immediately revoke download access to a specific backup.
- **Rate limits:** The application enforces per-user (24 hours) and global (2 hours) cooldowns on publishing backups.
