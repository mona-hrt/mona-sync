# Mona Sync

Mona Sync is the lightweight, blazingly-fast Rust REST API backend for the [**Mona** app](https://github.com/mona-hrt/mona). It handles cross-device synchronization for HRT and general medication tracking, ensuring that your inventory, schedules, intake logs, and blood test records are always up-to-date wherever you use the app.

## Quick Install (VPS)

You can install Mona Sync on your VPS with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/alwenyfae/mona-sync/main/install.sh | sudo bash
```

This script will:
1. Install system dependencies (`git`, `sqlite3`, etc.)
2. Install the Rust toolchain
3. Clone the repository to `/opt/mona-sync`
4. Build the project in release mode
5. Set up a systemd service to keep the API running
