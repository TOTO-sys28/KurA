Param(
  [string]$OPUS_CACHE = "./music_opus",
  [string]$RUST_LOG = "warn"
)

if (-not $env:DISCORD_TOKEN) {
  Write-Host "DISCORD_TOKEN is required" -ForegroundColor Red
  exit 1
}

$env:OPUS_CACHE = $OPUS_CACHE
$env:RUST_LOG = $RUST_LOG

cargo build --release
.\target\release\kura_voice.exe
