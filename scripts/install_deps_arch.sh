#!/usr/bin/env bash
set -euo pipefail

sudo pacman -Syu --noconfirm
sudo pacman -S --noconfirm \
  ffmpeg \
  base-devel \
  pkgconf \
  rust

# If you prefer rustup instead of repo rust:
# yay -S rustup
