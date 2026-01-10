#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# Usage: ./convert_all_to_opus.sh /path/to/music /path/to/output_cache 64k 1
SRC=${1:-./music}
OUT=${2:-./music_opus}
BITRATE=${3:-64k}
THREADS=${4:-1}

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

mkdir -p "$OUT"

# Use find -print0 so filenames with newlines/spaces are safe
find "$SRC" -type f \( -iname '*.mp3' -o -iname '*.flac' -o -iname '*.wav' -o -iname '*.m4a' -o -iname '*.ogg' \) -print0 |
while IFS= read -r -d '' f; do
  # Robust relpath using python -c
  rel=$(python3 -c 'import os,sys
try:
    print(os.path.relpath(sys.argv[1], sys.argv[2]))
except Exception:
    print(os.path.basename(sys.argv[1]))' "$f" "$SRC")

  dst="$OUT/${rel%.*}.opus"
  mkdir -p "$(dirname "$dst")"

  if [ -f "$dst" ]; then
    echo "Skip (exists): $dst"
    continue
  fi

  echo "Converting: $f -> $dst"

  ffmpeg -nostdin -hide_banner -loglevel error -y -threads "$THREADS" -i "$f" \
    -c:a libopus -b:a "$BITRATE" -vbr on -application audio -compression_level 10 -ar 48000 -ac 2 \
    "$dst" || {
      echo "FAILED -> $f" >&2
  }
done

echo "Done."
