<div align="center">

# 🐦 KurA

**Mastering the Discord Pulse: Lightweight, Stable, and Secure.**

[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/TOTO-sys28/KurA)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![DAVE Support](https://img.shields.io/badge/Discord-DAVE%20Verified-green.svg)](https://discord.com/blog/how-discord-is-securing-voice-and-video-with-end-to-end-encryption)

[**Features**](#features) | [**How It Works**](#how-it-works) | [**Advanced: The Encryption Problem**](#advanced-the-encryption-problem) | [**Quick Setup**](#quick-setup)

</div>

---

## Features

-   🚀 **Ultra-Low CPU**: Zero-transcoding playback using pre-converted `.opus` files.
-   🛡️ **Modern Security**: Full support for **DAVE (Discord Advanced Voice Encryption)** and MLS-based E2EE.
-   📡 **Rock-Solid Connectivity**: Compliant with **Voice Gateway V8**, resolving common 4006 session invalidations.
-   🔊 **Precision Volume**: Per-guild persistent volume control (`0.0`-`2.0`).
-   💎 **One-Click Privacy**: Verify end-to-end encryption with our unique `!privacy` command.

---

## How It Works

KurA achieves unparalleled performance by offloading audio processing to a "convert-once" workflow. Instead of burning CPU cycles re-encoding audio on the fly, it streams raw `.opus` packets directly to Discord's voice servers.

---

## 🔬 Advanced: The Encryption Problem

During development, we identified and solved two critical hurdles that cause most custom Discord bots to fail on modern channels:

### 1. The Gateway V8 "Sequence Gap"
Discord's latest Voice Gateway (V8) is extremely strict. Every heartbeat **must** acknowledge the latest inbound sequence number from the server (`seq_ack`). Traditional libraries often omit this, leading to the dreaded **4006: Session No Longer Valid** error every 14 seconds. KurA implements a custom JSON transformer that precisely tracks and injects these sequence numbers.

### 2. DAVE & MLS Handshake
End-to-End Encryption (E2EE) requires a complex multi-step handshake using **Messaging Layer Security (MLS)**. KurA handles binary handshake frames (Opcodes 25-30) and performs media encryption using **XChaCha20Poly1305** before the transport layer ever sees the packet.

---

## 🛠 Quick Setup

### 1. Requirements
-   **FFmpeg**: Required for the one-time conversion process.
-   **Discord Bot Token**: Ensure **Message Content Intent** is enabled in the Developer Portal.

<details>
<summary><b>🐧 Linux (Arch / Ubuntu / Fedora)</b></summary>

```bash
# Install dependencies (Ubuntu)
sudo apt install ffmpeg libopus-dev

# Use our AUR package (Arch)
makepkg -si ./packaging/arch/PKGBUILD

# Run with token
export DISCORD_TOKEN="your_token"
cargo run --release --features dave
```
</details>

<details>
<summary><b>🪟 Windows (Powershell)</b></summary>

```powershell
# Install FFmpeg
winget install Gyan.FFmpeg

# Run
$env:DISCORD_TOKEN="your_token"
cargo run --release --features dave --bin kura_voice
```
</details>

<details>
<summary><b>🍎 macOS (Homebrew)</b></summary>

```bash
# Install FFmpeg
brew install ffmpeg

# Run
export DISCORD_TOKEN="your_token"
cargo run --release --features dave
```
</details>

---

## 🎼 Commands

| Command | Action |
| --- | --- |
| `!join` | Join your current voice channel |
| `!play <prefix>` | Play a song (random match if ambiguous) |
| `!volume <val>` | Set volume (0.0 to 2.0) |
| `!privacy` | **DAVE Only**: Display the unique 30-digit Privacy Verification Code |
| `!skip` | Skip to a random next song |
| `!help` | Show all available commands |

---

## 💎 Contributor Cleanup & Standardization

KurA is designed to be cross-distro compatible. All file paths use Rust's `PathBuf` to ensure seamless operation on both Windows backslashes and Unix forward-slashes. 

**Commit-ready?** Run:
```bash
cargo clean
cargo build --features dave
```

---

<div align="center">
Made with ❤️ for the low-latency community.
</div>
