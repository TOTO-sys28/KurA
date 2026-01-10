#!/usr/bin/env bash
set -euo pipefail

sudo dnf install -y \
  ffmpeg \
  gcc \
  gcc-c++ \
  make \
  pkgconf-pkg-config \
  openssl-devel

# Rust
if ! command -v cargo >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh
  echo "Restart your shell or run: source $HOME/.cargo/env"
fi
