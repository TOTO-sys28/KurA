# KurA

<p align="center">
  <img alt="KurA Banner" src="https://capsule-render.vercel.app/api?type=waving&color=0:5865F2,100:22c55e&height=170&section=header&text=KurA%20Discord%20Music%20Bot&fontSize=38&fontColor=ffffff&animation=fadeIn">
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-orange?logo=rust">
  <img alt="Discord Voice" src="https://img.shields.io/badge/Discord-Voice%20Bot-5865F2?logo=discord&logoColor=white">
  <img alt="Platforms" src="https://img.shields.io/badge/Platforms-Windows%20%7C%20Linux%20%7C%20macOS-0ea5e9">
  <img alt="Encryption" src="https://img.shields.io/badge/Voice-DAVE%2FE2EE-22c55e">
</p>

<p align="center">
  <img alt="Typing Animation" src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&pause=1200&color=5865F2&center=true&vCenter=true&width=900&lines=Low+CPU+Discord+music+bot;Pre-converted+.opus+streaming;Windows+%7C+Linux+%7C+macOS;DAVE%2FE2EE-aware+voice+handling">
</p>

🎵 A low-CPU Discord music bot that streams pre-converted `.opus` files and supports modern Discord voice behavior, including DAVE/E2EE-aware handling.

## Why KurA

- 🚀 **Zero Transcoding:** Low runtime CPU usage by avoiding live transcoding.
- 🌍 **Cross-Platform:** Works across Windows, Linux, and macOS.
- 📦 **Simple Install:** Includes an MSI installer for Windows and native distro packages for Linux.
- 🛡️ **DAVE/E2EE Ready:** Handles modern Discord encryption with clear fallback behavior.

## Quick Start

1. 🤖 **Create a Bot:** Get your token from the [Discord Developer Portal](https://discord.com/developers/applications).
2. ✅ **Enable Intents:** Turn on **MESSAGE CONTENT** in the portal.
3. ⬇️ **Download:** Grab the latest `KurA-x64.msi` from [Releases](../../releases).
4. ▶️ **Run:** Open `kura` from your Start Menu or Terminal, paste your token, and you're done!

---

## Installation & Running

### Windows
- **Standard (Recommended):** Download and run the `.msi` installer. It adds `kura` and `kurac` to your PATH.
- **Run:** Simply type `kura` in any terminal or double-click the app. The bot will ask for your token on the first run.

### Linux / macOS
- **Binary:** Download the archive for your OS, extract, and run `./kura`.
- **Global (via npx):**
  ```bash
  npm install -g kura2-cli 
  ```

---

## Music Conversion (`.opus`)

KurA plays `.opus` files for maximum efficiency. You can convert your existing library using our built-in tool:

1. **Install FFmpeg:**
   - **Windows:** `winget install Gyan.FFmpeg`
   - **Linux:** `sudo apt/dnf/pacman -S ffmpeg`
2. **Convert:**
   ```bash
   kurac
   ```
   Follow the interactive prompts to pick your music folder and output directory.

---

## Commands

- 🎧 `!join`, `!leave`
- ▶️ `!play <prefix>`, `!random`, `!skip`, `!stop`
- 🔁 `!loop`, 🔊 `!volume <0.0..2.0>`
- 📚 `!list [prefix]`, `!reindex`
- 🛡️ `!privacy`, ❓`!help`, 🏓 `!ping`

---

## Technical Setup (Developers)

### Build From Source
```bash
cargo build --release --bins
```

### DAVE (Discord voice encryption)

**DAVE** is Discord’s encrypted voice path (MLS / “E2EE-aware” voice). The badges above describe what the **songbird** voice stack can support upstream.

- This project currently pins **songbird 0.5** from crates.io. That release **does not** ship a `dave` Cargo feature, so `cargo build --features dave` **does not apply** here (that line was outdated).
- **Songbird 0.6** wires **`davey`** into the voice driver; moving KurA to 0.6+ is how you align with the crates.io story for DAVE plumbing.
- **GitHub release tarballs and AUR `-bin`** are built from this `Cargo.toml` (same as CI): they match **0.5**, not a separate “DAVE-only” binary. If your local build used a **git** songbird, **0.6**, or a **patch**, it can behave differently from the prebuilt package.

### Distro Packaging
KurA includes build scripts for native packages in the `packaging/` folder:
- **Debian/Ubuntu:** `bash packaging/deb/build_deb.sh`
- **Fedora/RHEL:** uses `packaging/rpm/kura.spec`
- **Arch Linux:** `PKGBUILD` available in `packaging/arch/` and `kura-voice-bin-latest/`

<p align="center">
  <img alt="Footer" src="https://capsule-render.vercel.app/api?type=waving&color=0:22c55e,100:5865F2&height=120&section=footer">
</p>
