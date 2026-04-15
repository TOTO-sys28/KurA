# KurA

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-orange?logo=rust">
  <img alt="Discord Voice" src="https://img.shields.io/badge/Discord-Voice%20Bot-5865F2?logo=discord&logoColor=white">
  <img alt="Platforms" src="https://img.shields.io/badge/Platforms-Windows%20%7C%20Linux%20%7C%20macOS-0ea5e9">
  <img alt="Encryption" src="https://img.shields.io/badge/Voice-DAVE%2FE2EE-22c55e">
</p>

A low-CPU Discord music bot that streams pre-converted `.opus` files and supports modern Discord voice behavior, including DAVE/E2EE-aware handling.

## Why KurA

- Low runtime CPU usage by avoiding live transcoding.
- Works across Windows, Linux, and macOS.
- Includes distro packaging helpers (`.deb`, `.rpm`, Arch `PKGBUILD`).
- Handles E2EE/DAVE voice channels with clear fallback behavior.

## Quick Start

1. Create a bot in the Discord Developer Portal.
2. Enable **MESSAGE CONTENT** intent.
3. Set environment variables.
4. Build and run.

### Required Environment Variables

| Variable | Description | Default |
|---|---|---|
| `DISCORD_TOKEN` | Bot token (required) | none |
| `OPUS_CACHE` | Folder containing `.opus` files | `./music_opus` |
| `RUST_LOG` | Log level (`warn`, `info`, `debug`) | `warn` |

### Run (Windows)

```powershell
$env:DISCORD_TOKEN="YOUR_TOKEN_HERE"
$env:OPUS_CACHE="./music_opus"
powershell -ExecutionPolicy Bypass -File scripts/run_windows.ps1 -EnableDave
```

### Run (Linux / macOS)

```bash
export DISCORD_TOKEN="YOUR_TOKEN_HERE"
export OPUS_CACHE="./music_opus"
bash ./run.sh
```

## Build From Source (All Platforms)

### Standard build

```bash
cargo build --release --bin kura_voice
```

### DAVE/E2EE-capable build

```bash
cargo build --release --features dave --bin kura_voice
```

## Commands

- `!join`, `!leave`
- `!play <prefix>`, `!random`, `!skip`, `!stop`
- `!loop`, `!volume <0.0..2.0>`
- `!list [prefix]`, `!reindex`
- `!privacy`, `!help`, `!ping`

## Music Conversion (`.opus`)

Install FFmpeg:
- Ubuntu/Debian: `sudo apt-get install ffmpeg`
- Fedora/RHEL: `sudo dnf install ffmpeg`
- Arch: `sudo pacman -S ffmpeg`
- macOS: `brew install ffmpeg`
- Windows: `winget install Gyan.FFmpeg`

Use helpers:

```bash
cargo run --bin kura_tui
```

```bash
bash scripts/convert_all_to_opus.sh ./music ./music_opus
```

```powershell
powershell -File scripts\convert_all_to_opus.ps1 -SRC ./music -OUT ./music_opus
```

## E2EE / DAVE: How We Skip the Encryption Problem

Discord can require DAVE (E2EE) in some voice channels. KurA handles this in two modes:

- Built **with** `--features dave`: KurA attempts DAVE-capable voice setup and exposes `!privacy` verification code support.
- Built **without** `dave`: if channel encryption is required, KurA fails safely and tells you to use a non-E2EE channel or disable E2EE on that channel.

This keeps behavior explicit instead of silently breaking playback.

## Executables and Packaging

- **Windows executable**: `target/release/kura_voice.exe`
- **Linux/macOS binary**: `target/release/kura_voice`
- **GitHub Releases**: prebuilt archives for Windows/Linux/macOS via workflow.
- **Linux distro packages**:
  - Debian/Ubuntu: `.deb`
  - Fedora/RHEL: `.rpm`
  - Arch: `PKGBUILD` (source) and `PKGBUILD-bin` template for AUR binary package

See `packaging/README.md` for package layout and service files.

## Clean Commit Workflow

For commit hygiene and generated/local file cleanup, follow `docs/CLEAN_GUIDE.md`.
