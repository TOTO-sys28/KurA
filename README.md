# KurA

A lightweight Discord music bot optimized for low CPU usage.

## How It Works

KurA keeps CPU usage minimal by using pre-converted `.opus` files instead of live transcoding. Just convert your music library once, and playback is simple file streaming.

**Tech stack:** Rust + Songbird for efficient voice handling

## Commands

- `!join` / `!leave` - Join or leave voice channel
- `!play <prefix>` - Play a song starting with the prefix (e.g., `!play A` or `!play ال`)
  - Multiple matches? Picks randomly
  - No prefix? Plays random song
- `!random` - Play random song
- `!skip` - Skip to random next song
- `!stop` - Stop playback
- `!loop` - Toggle loop mode
- `!list [prefix]` - List all songs (or filter by prefix)
- `!reindex` - Reload music library

## Quick Setup

### 1. Download the Bot

Grab the prebuilt binary for your system from the [Releases](https://github.com/TOTO-sys28/KurA/releases) page.

### 2. Get Your Discord Token

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create an application → **Bot** section
3. Enable **MESSAGE CONTENT INTENT**
4. Copy your bot token

**⚠️ Never share your token publicly. If leaked, reset it immediately.**

### 3. Set Environment Variables & Run

**Linux/macOS:**
```bash
export DISCORD_TOKEN="YOUR_TOKEN_HERE"
export OPUS_CACHE="./music_opus"
export RUST_LOG=warn
chmod +x ./kura_voice
./kura_voice
```

**Windows:**
```bat
set DISCORD_TOKEN=YOUR_TOKEN_HERE
set OPUS_CACHE=./music_opus
set RUST_LOG=warn
kura_voice.exe
```
```bat
powershell.exe -ExecutionPolicy Bypass -File .\convert_all_to_opus.ps1
```
## Converting Your Music Library

### Install FFmpeg

- **Ubuntu/Debian:** `sudo apt-get install ffmpeg`
- **Fedora:** `sudo dnf install ffmpeg`
- **Arch:** `sudo pacman -S ffmpeg`
- **macOS:** `brew install ffmpeg`
- **Windows:** `winget install Gyan.FFmpeg`

### Convert to Opus

Place your music files in `./music/`, then convert:

**Linux/macOS:**
```bash
bash scripts/convert_all_to_opus.sh ./music ./music_opus 64k 1
```

**Windows PowerShell:**
```powershell
powershell -ExecutionPolicy Bypass -File scripts\convert_all_to_opus.ps1 -SRC ./music -OUT ./music_opus -BITRATE 64k
```

## Optional: Build from Source

If you prefer to compile it yourself:

```bash
cargo build --release
```

## Monitoring CPU Usage

**Linux:**
```bash
htop
```

Or for specific process monitoring:
```bash
pidstat -p $(pgrep -n kura_voice) 1
```

---

That's it! Your bot should now be running and ready to play music with minimal CPU overhead.
