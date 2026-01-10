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

$SRC = (Resolve-Path -LiteralPath $SRC).Path
$OUT = (Resolve-Path -LiteralPath $OUT -ErrorAction SilentlyContinue) ?? $OUT

New-Item -ItemType Directory -Force -Path $OUT | Out-Null

$files = Get-ChildItem -Path $SRC -Recurse -File | Where-Object {
  $_.Extension -match "(?i)\.(mp3|flac|wav|m4a|ogg)$"
}

foreach ($f in $files) {
  $relPath = [System.IO.Path]::GetRelativePath($SRC, $f.FullName)
  $dst = Join-Path $OUT ([System.IO.Path]::ChangeExtension($relPath, ".opus"))
  $dstDir = Split-Path -Parent $dst

  New-Item -ItemType Directory -Force -Path $dstDir | Out-Null

  if (Test-Path -LiteralPath $dst) {
    Write-Host "Skip (exists): $dst"
    continue
  }

  Write-Host "Converting: $($f.FullName) -> $dst"

  ffmpeg -nostdin -hide_banner -loglevel error -y `
    -i "$($f.FullName)" `
    -vn -ar 48000 -ac 2 `
    -c:a libopus -b:a $BITRATE -vbr on `
    -application audio -compression_level 10 `
    "$dst"
}

Write-Host "Done."
