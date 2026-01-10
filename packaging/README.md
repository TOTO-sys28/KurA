# Packaging

This folder contains best-practice starter packaging for:

- Debian/Ubuntu: `.deb`
- Fedora/RHEL: `.rpm`
- Arch: `PKGBUILD`

All packages install:

- Binary: `/usr/bin/kura_voice`
- Env file: `/etc/kura_voice.env` (mode 600)
- systemd service: `kura_voice.service`

After install:

1) Edit `/etc/kura_voice.env` and set `DISCORD_TOKEN`
2) `sudo systemctl enable --now kura_voice`
