Param(
  [string]$SRC = "./music",
  [string]$OUT = "./music_opus",
  [string]$BITRATE = "64k"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
  Write-Host "ffmpeg not found. Install it first (see README)." -ForegroundColor Red
  exit 1
}

New-Item -ItemType Directory -Force -Path $OUT | Out-Null

$files = Get-ChildItem -Path $SRC -Recurse -File | Where-Object {
  $_.Extension -match "(?i)\.(mp3|flac|wav|m4a|ogg)$"
}

foreach ($f in $files) {
  $rel = Resolve-Path -LiteralPath $f.FullName
  $relStr = $f.FullName.Substring((Resolve-Path -LiteralPath $SRC).Path.Length).TrimStart('\\','/')
  $dst = Join-Path $OUT ([System.IO.Path]::ChangeExtension($relStr, ".opus"))
  $dstDir = Split-Path -Parent $dst
  New-Item -ItemType Directory -Force -Path $dstDir | Out-Null

  if (Test-Path -LiteralPath $dst) {
    Write-Host "Skip (exists): $dst"
    continue
  }

  Write-Host "Converting: $($f.FullName) -> $dst"
  ffmpeg -nostdin -hide_banner -loglevel error -y -i "$($f.FullName)" -vn -ar 48000 -ac 2 -c:a libopus -b:a $BITRATE -vbr on -application audio -compression_level 10 "$dst"
}

Write-Host "Done."
