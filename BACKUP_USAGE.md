# Backup Command Usage

## Overview

The `/backup` command sends the most recent backup file from your configured backup folder through Discord. Large files are automatically split into 24MB chunks to fit Discord's file size limits.

## Setup

Add to your `.env` file:
```bash
BACKUP_FOLDER=/path/to/your/minecraft/backups
```

## Usage

In Discord, run:
```
/backup
```

The bot will:
1. Find the most recent file in the backup folder
2. Split into 24MB chunks if needed
3. Send all chunks through Discord

## File Size Examples

For a 4GB `.tgz` backup:
- Split into ~170 chunks of 24MB each
- Takes a few minutes to upload all chunks

## Restoring a Backup

### Single File (No Chunks)
The file is ready to use as-is.

### Multiple Chunks

**Linux/macOS:**
```bash
# Combine all parts in order
cat backup.tgz.part* > backup.tgz

# Extract
tar -xzf backup.tgz
```

**Windows (PowerShell):**
```powershell
# Combine all parts in order
Get-Content backup.tgz.part* -Raw | Set-Content backup.tgz -Encoding Byte

# Extract (requires tar - built into Windows 10+)
tar -xzf backup.tgz
```

**Windows (Command Prompt):**
```cmd
REM Combine all parts
copy /b backup.tgz.part001+backup.tgz.part002+backup.tgz.part003 backup.tgz

REM Extract (Windows 10+)
tar -xzf backup.tgz
```

**Alternative (Any OS):**
- Use 7-Zip, WinRAR, or any archive tool that supports `.tgz` files
- Most GUI archive tools can open `.tgz` directly

## Troubleshooting

**"No backup found"**
- Check your `BACKUP_FOLDER` path is correct
- Ensure the folder contains files (not subdirectories)
- Verify the bot has read permissions

**Missing chunks**
- Ensure all chunks are downloaded from Discord
- They must be combined in the correct order (part001, part002, etc.)

## Example

```bash
# Bot sends in Discord:
📦 Sending backup: world_2024-12-13.tgz (167.33 MB, 7 chunks).
📦 Chunk 1/7
📦 Chunk 2/7
...
📦 Chunk 7/7
✅ Backup sent successfully!
To restore: cat world_2024-12-13.tgz.part* > world_2024-12-13.tgz

# On your machine:
cat world_2024-12-13.tgz.part* > world_2024-12-13.tgz
tar -xzf world_2024-12-13.tgz
```
