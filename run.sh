#!/usr/bin/env bash
set -euo pipefail

: "${DISCORD_TOKEN:?DISCORD_TOKEN is required}"
OPUS_CACHE=${OPUS_CACHE:-./music_opus}
RUST_LOG=${RUST_LOG:-warn}

export OPUS_CACHE RUST_LOG

chmod +x ./kura_voice 2>/dev/null || true
./kura_voice
