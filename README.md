# OxideVault

Welcome to the **OxideVault** repository! This repository was generated from a template to get you started quickly.

## 🚀 Getting Started

To get started with this project:

1. Clone the repository:
   ```bash
   git clone https://github.com/Vianpyro/OxideVault.git
   cd OxideVault
   ```
2. Install any dependencies (if applicable).
3. Follow the instructions in the relevant documentation or project files to start working.

## 📁 Project Structure

The repository contains the following directories and files:

- `.devcontainer/` - Development container configuration for Visual Studio Code
  - `devcontainer.json` - Dev container settings
  - `Dockerfile` - Container image definition
- `.github/` - GitHub-specific configurations
  - `ISSUE_TEMPLATE/` - Issue templates (bug reports, feature requests)
  - `pull_request_template.md` - Pull request template
  - `workflows/` - GitHub Actions workflow files
- `.vscode/` - Visual Studio Code workspace settings and tasks
- `.dockerignore` - Docker build exclusions
- `.gitattributes` - Git attributes configuration
- `.gitignore` - Git ignore patterns
- `README.md` - This file

## 🛠 Features

- Initialized from a reusable template for quick setup.
- Pre-configured workflows for automation and CI/CD.
- Placeholder sections for documentation, testing, and development.

## 📖 Documentation

Check the project files and comments for guidance. You can expand this section as your project grows.

## 🤝 Contributing

Contributions are welcome! Feel free to open issues, submit pull requests, or suggest improvements.

## 🔒 Backups via HTTPS Links

The bot no longer pushes backups to Discord. Instead, it publishes the most recent backup file to a tokenized directory, served by your reverse proxy (e.g., Caddy), and sends the download URL and commands.

Environment variables (defaults designed for a Docker volume mounted at `/backups`):

```bash
BACKUP_FOLDER=/backups
BACKUP_PUBLISH_ROOT=/backups/public
BACKUP_PUBLIC_BASE_URL=https://drop.example.com/backups
```

Example workflow:

1. The bot creates `/backups/public/<token>/my_backup.tgz` (hard-linked if possible, otherwise copied).
2. Caddy serves `/backups/public` at `https://drop.example.com/backups`.
3. The bot sends `https://drop.example.com/backups/<token>/my_backup.tgz` with `curl` / `Invoke-WebRequest` commands.

### Example Caddy Configuration (HTTPS + optional Basic Auth)

```caddyfile
drop.example.com {
  root * /backups/public
  file_server

  # Optional: Basic authentication
  basicauth /* {
    user JDJhJDEwJHVkL1Y2d3pzZk5IUUV0ZThQcnA0TTQuU3g0dC52cWlvUmFrZDFYOHhHTlFaQ2lUSmFwRE5v
  }

  # Minimal security headers
  header /* {
    Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
    X-Content-Type-Options "nosniff"
  }
}
```

**Security Note**: Do NOT enable `browse` directive on the file_server, as this would allow anyone to list and access all tokenized backup directories without needing the individual tokens.

## 📡 PairDrop (Self-hosted)

PairDrop is useful for ad-hoc transfers between browsers. The bot cannot automatically publish via PairDrop (WebRTC browser-side), but you can:

1. Run PairDrop on your LAN behind Caddy (HTTPS).
2. Restrict access: LAN only, or Basic Auth/IP allowlist.
3. Use PairDrop manually for ad-hoc exchanges; for bot backups, prefer the HTTPS link described above.

## 📝 License

Specify your license here (if any). For example: MIT, Apache 2.0, etc.

Happy coding! 🎉
