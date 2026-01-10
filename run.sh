#!/usr/bin/env bash
set -euo pipefail

: "${DISCORD_TOKEN:?DISCORD_TOKEN is required}"
OPUS_CACHE=${OPUS_CACHE:-./music_opus}
RUST_LOG=${RUST_LOG:-warn}

export OPUS_CACHE RUST_LOG

./target/release/kura_voice
