#!/usr/bin/env bash
set -euo pipefail

if [ -f ./.env ]; then
  set -a
  # shellcheck disable=SC1091
  source ./.env
  set +a
fi

: "${DISCORD_TOKEN:?DISCORD_TOKEN is required (run scripts/setup.sh or set env)}"
OPUS_CACHE=${OPUS_CACHE:-./music_opus}
RUST_LOG=${RUST_LOG:-warn}
export OPUS_CACHE RUST_LOG

./target/release/kura_voice
