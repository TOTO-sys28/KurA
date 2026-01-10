#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  ffmpeg \
  ca-certificates \
  build-essential \
  pkg-config

# Rust
if ! command -v cargo >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh
  echo "Restart your shell or run: source $HOME/.cargo/env"
fi
