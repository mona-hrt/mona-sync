# Mona Sync

Mona Sync is the lightweight, blazingly-fast Rust REST API backend for the [**Mona** app](https://github.com/mona-hrt/mona). It handles cross-device synchronization for HRT and general medication tracking, ensuring that your inventory, schedules, intake logs, and blood test records are always up-to-date wherever you use the app.

## Security Features

### End-to-End Encryption (E2EE)
Privacy is a core principle of Mona Sync. All synchronization items and vault data are encrypted on the client side before they are sent to the server. The backend only stores and transmits opaque, encrypted payloads. This ensures that the server operator cannot access your medical data, schedules, or personal logs.

### Transport Security (TLS/SSL)
Mona Sync supports secure communication out of the box.
- Self-Signed Certificates: The server can automatically generate a self-signed certificate for immediate encrypted communication.
- Reverse Proxy: A Caddyfile is included for production environments, facilitating automatic HTTPS with Let's Encrypt or ZeroSSL.

### Authentication and Access Control
- JWT Authentication: Access to synchronization endpoints is secured via JSON Web Tokens.
- Password Protection: Initial authentication requires a pre-shared API password.

### Rate Limiting
To prevent abuse and brute-force attacks, the API employs a rate-limiting layer (axum-governor). It restricts the number of requests per second per IP address, ensuring service stability and security.

## Architecture

- Language: Built with Rust for memory safety and high performance.
- Framework: Utilizes Axum for its robust and scalable routing system.
- Database: Uses SQLite (via sqlx) for a portable, file-based storage solution with high reliability.
- Migrations: Automated database schema management.

## Quick Install (VPS)

You can install Mona Sync on your VPS with a single command:

```bash
curl -sSfL https://raw.githubusercontent.com/mona-hrt/mona-sync/dev/install.sh |
      sudo bash -s -- --branch dev
```

This script will:
1. Install system dependencies (git, sqlite3, etc.)
2. Install the Rust toolchain
3. Clone the repository to /opt/mona-sync
4. Build the project in release mode
5. Set up a systemd service to keep the API running
