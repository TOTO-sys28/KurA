#!/usr/bin/env bash
set -euo pipefail

printf "KurA setup\n\n"

OS="unix"
if [[ "${OSTYPE:-}" == "darwin"* ]]; then
  OS="macos"
fi

echo "Detected: $OS"

read -r -p "Discord bot token (DISCORD_TOKEN): " TOKEN
if [[ -z "$TOKEN" ]]; then
  echo "Token is required." >&2
  exit 1
fi

read -r -p "OPUS cache folder [./music_opus]: " OPUS_CACHE
OPUS_CACHE=${OPUS_CACHE:-./music_opus}

read -r -p "Log level (warn/info) [warn]: " RUST_LOG
RUST_LOG=${RUST_LOG:-warn}

echo
read -r -p "Write system-wide env file to /etc/kura_voice.env? (recommended for packages) [y/N]: " SYS

if [[ "$SYS" =~ ^[Yy]$ ]]; then
  sudo sh -c "umask 077; cat > /etc/kura_voice.env <<EOF
DISCORD_TOKEN=$TOKEN
OPUS_CACHE=$OPUS_CACHE
RUST_LOG=$RUST_LOG
EOF"
  echo "Wrote /etc/kura_voice.env"
  echo "If installed as a service: sudo systemctl enable --now kura_voice"
else
  umask 077
  cat > .env <<EOF
DISCORD_TOKEN=$TOKEN
OPUS_CACHE=$OPUS_CACHE
RUST_LOG=$RUST_LOG
EOF
  echo "Wrote ./.env"
  echo "Run: source ./.env && ./kura_voice"
fi
