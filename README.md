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

- `.devcontainer/` - Development container configuration for VS Code
  - `devcontainer.json` - Dev container settings
  - `Dockerfile` - Container image definition
- `.github/` - GitHub-specific configurations
  - `ISSUE_TEMPLATE/` - Issue templates (bug reports, feature requests)
  - `pull_request_template.md` - Pull request template
  - `workflows/` - GitHub Actions workflow files
- `.vscode/` - VS Code workspace settings and tasks
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

## 🔒 Sauvegardes via lien HTTPS

Le bot ne pousse plus les backups sur Discord. Il publie le fichier le plus récent dans un dossier tokenisé, servi par votre reverse-proxy (ex: Caddy), puis envoie l’URL et les commandes de téléchargement.

Variables d’environnement (défauts pensés pour un volume Docker monté sur `/backups`) :

```bash
BACKUP_FOLDER=/backups
BACKUP_PUBLISH_ROOT=/backups/public
BACKUP_PUBLIC_BASE_URL=https://drop.example.com/backups
```

Exemple de flux :
1. Le bot crée `/backups/public/<token>/mon_backup.tgz` (hard-link si possible, sinon copie).
2. Caddy sert `/backups/public` sur `https://drop.example.com/backups`.
3. Le bot envoie `https://drop.example.com/backups/<token>/mon_backup.tgz` et les commandes `curl` / `Invoke-WebRequest`.

### Exemple Caddy (HTTPS + option Basic Auth)

```caddyfile
drop.example.com {
  root * /backups/public
  file_server browse

  # Facultatif : auth basique
  basicauth /* {
    user JDJhJDEwJHVkL1Y2d3pzZk5IUUV0ZThQcnA0TTQuU3g0dC52cWlvUmFrZDFYOHhHTlFaQ2lUSmFwRE5v
  }

  # Sécurité minimale
  header /* {
    Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
    X-Content-Type-Options "nosniff"
  }
}
```

## 📡 PairDrop (auto-hébergé)

PairDrop est pratique pour des transferts ponctuels entre navigateurs. Le bot ne peut pas publier automatiquement via PairDrop (WebRTC côté navigateur), mais vous pouvez :

1. Faire tourner PairDrop sur votre LAN derrière Caddy (HTTPS).
2. Restreindre l’accès : LAN seulement, ou Basic Auth/IP allowlist.
3. Utiliser PairDrop manuellement pour des échanges ad-hoc ; pour les backups du bot, préférez le lien HTTPS décrit plus haut.

## 📝 License

Specify your license here (if any). For example: MIT, Apache 2.0, etc.

Happy coding! 🎉
