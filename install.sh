#!/bin/bash

# Mona Sync Installation Script
# This script installs Mona Sync on a Linux VPS.

set -e

REPO_URL="https://github.com/alwenyfae/mona-sync.git"
INSTALL_DIR="/opt/mona-sync"
SERVICE_NAME="mona-sync"
DEFAULT_PORT="3000"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🌟 Starting Mona Sync installation...${NC}"

if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}❌ Please run as root or with sudo.${NC}"
  exit 1
fi

ACTUAL_USER=${SUDO_USER:-$USER}
HOME_DIR=$(eval echo ~$ACTUAL_USER)

echo -e "${BLUE}📦 Installing system dependencies...${NC}"
apt-get update -y
apt-get install -y git curl build-essential libsqlite3-dev pkg-config sqlite3

if ! command -v cargo &> /dev/null; then
    echo -e "${BLUE}🦀 Installing Rust for ${ACTUAL_USER}...${NC}"
    sudo -u "$ACTUAL_USER" bash <<EOF
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
EOF
    source "$HOME_DIR/.cargo/env"
else
    echo -e "${GREEN} Rust is already installed.${NC}"
fi

if [ -d "$INSTALL_DIR" ]; then
    echo -e "${BLUE} Directory $INSTALL_DIR already exists. Updating...${NC}"
    cd "$INSTALL_DIR"
    git pull
else
    echo -e "${BLUE} Cloning repository to $INSTALL_DIR...${NC}"
    git clone "$REPO_URL" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

echo -e "${BLUE} Ensure correct ownership...${NC}"
chown -R "$ACTUAL_USER":"$ACTUAL_USER" "$INSTALL_DIR"

if [ ! -f ".env" ]; then
    echo -e "${BLUE} Creating default .env file...${NC}"
    sudo -u "$ACTUAL_USER" cat <<EOF > .env
DATABASE_URL=sqlite://database.db
SERVER_IP=0.0.0.0
SERVER_PORT=$DEFAULT_PORT
EOF
fi

echo -e "${BLUE} Setting up the database...${NC}"
DB_PATH="database.db"
MIGRATION_FILE=$(ls migrations/*.sql | head -n 1)
if [ -n "$MIGRATION_FILE" ]; then
    sudo -u "$ACTUAL_USER" sqlite3 "$DB_PATH" < "$MIGRATION_FILE"
    echo -e "${GREEN} Database migrations applied.${NC}"
else
    echo -e "${RED} No migration files found! Creating empty database...${NC}"
    sudo -u "$ACTUAL_USER" touch "$DB_PATH"
fi

echo -e "${BLUE} Building Mona Sync (this may take a few minutes)...${NC}"
sudo -u "$ACTUAL_USER" bash <<EOF
source "$HOME_DIR/.cargo/env"
cargo build --release
EOF

echo -e "${BLUE} Setting up systemd service...${NC}"
cat <<EOF > /etc/systemd/system/$SERVICE_NAME.service
[Unit]
Description=Mona Sync Backend
After=network.target

[Service]
Type=simple
User=$ACTUAL_USER
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/.env
ExecStart=$INSTALL_DIR/target/release/mona_backend
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable $SERVICE_NAME
systemctl restart $SERVICE_NAME

echo -e "${GREEN} Mona Sync is installed and running!${NC}"
echo -e " Status: ${BLUE}systemctl status $SERVICE_NAME${NC}"

PUBLIC_IP=$(curl -s https://ifconfig.me || echo "your-vps-ip")
echo -e " Server address: ${GREEN}http://$PUBLIC_IP:$DEFAULT_PORT${NC}"
echo -e " Health check: ${GREEN}http://$PUBLIC_IP:$DEFAULT_PORT/health${NC}"
