# KurA

**KurA** is a lightweight Discord voice music bot built for **low CPU usage**.

It’s designed around one simple rule:

- **Pre-cache your library as `.opus`** → playback is just streaming (no live transcoding).

## Why it feels fast

- **Rust + Songbird** (compiled voice stack)
- **Opus passthrough-friendly** setup (volume `1.0`, single track)
- **Prefix search** that works well with Arabic / non‑Latin names

## Commands

- `!join` / `!leave`
- `!play <prefix>`
  - Example: `!play A` or `!play ال`
  - Picks a song whose name **starts with** the prefix
  - If multiple match → **random**
- `!play` (no args) → random
- `!random` → random
- `!skip` → stop + random next
- `!stop`
- `!loop` → toggle loop on/off
- `!list [prefix]` → list all (or prefix-first matches)
- `!reindex`

## Quickstart (recommended): GitHub Releases (prebuilt binaries)

1. Download the right file from **Releases**.
2. Set `DISCORD_TOKEN`.
3. Run the binary.

### Linux x64
```bash
export DISCORD_TOKEN="YOUR_TOKEN"
export OPUS_CACHE="./music_opus"
export RUST_LOG=warn

chmod +x ./kura_voice
./kura_voice
```

### Windows
```bat
set DISCORD_TOKEN=YOUR_TOKEN
set OPUS_CACHE=./music_opus
set RUST_LOG=warn

kura_voice.exe
```

### macOS
```bash
export DISCORD_TOKEN="YOUR_TOKEN"
export OPUS_CACHE="./music_opus"
export RUST_LOG=warn

chmod +x ./kura_voice
./kura_voice
```

## Setup: Discord bot token (required)

1. Discord Developer Portal → your application → **Bot**
2. Enable:
   - **MESSAGE CONTENT INTENT**
3. Copy the token.

**Security**

- Never paste your token into issues / screenshots.
- If you leaked it once: **reset token immediately**.

## Interactive setup (recommended)

- **Linux/macOS:**
```bash
bash scripts/setup.sh
```

- **Windows:**
```bat
setup.bat
```

## Build your Opus cache (recommended)

Put music in `./music/` (or any folder), convert to `./music_opus/`.

### Install ffmpeg

- **Ubuntu/Debian:** `sudo apt-get install -y ffmpeg`
- **Fedora:** `sudo dnf install -y ffmpeg`
- **Arch:** `sudo pacman -S ffmpeg`
- **macOS:** `brew install ffmpeg`
- **Windows:** `winget install Gyan.FFmpeg`

### Convert (Linux/macOS)
```bash
bash scripts/convert_all_to_opus.sh ./music ./music_opus 64k 1
```

### Convert (Windows PowerShell)
```powershell
powershell -ExecutionPolicy Bypass -File scripts\convert_all_to_opus.ps1 -SRC ./music -OUT ./music_opus -BITRATE 64k
```

## CPU monitoring

Linux:

```bash
htop
```

or:

```bash
pidstat -p $(pgrep -n kura_voice) 1
```

## Source build (optional)

If you want to build locally:

```bash
cargo build --release
```

## Publishing to GitHub (your account)

From inside the `KurA/` folder:

```bash
git init
git add .
git commit -m "KurA: initial release"
git branch -M main
git remote add origin https://github.com/TOTO-sys28/KurA.git
git push -u origin main
```

## Releases (auto-built downloads)

This repo includes GitHub Actions workflows that build and attach:

- Prebuilt binaries (Linux/Windows/macOS)
- Linux packages: `.deb` + `.rpm`

When you push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

After the workflow finishes, download everything from the GitHub Release page.

Before you run `git add .`, make sure you don’t commit local build outputs (they’re ignored, but delete them if already created):

```bash
rm -f ./*.deb
rm -rf packaging/rpm/dist
```

## Linux packages (Option 2)

If you want system packages (service + `/etc/kura_voice.env`):

### Debian/Ubuntu (.deb)

Build locally:
```bash
sudo apt-get install -y dpkg-dev
bash packaging/deb/build_deb.sh
```

Install:
```bash
sudo dpkg -i kura-voice_0.1.0_amd64.deb
sudo nano /etc/kura_voice.env
sudo systemctl enable --now kura_voice
```

### Fedora/RHEL (.rpm)

Build locally (best-effort helper):
```bash
sudo dnf install -y rpm-build
bash packaging/rpm/build_rpm.sh
```

### Arch (PKGBUILD / pacman / yay)

Local build:
```bash
cd packaging/arch
makepkg -si
```

Then:
```bash
sudo nano /etc/kura_voice.env
sudo systemctl enable --now kura_voice
```
