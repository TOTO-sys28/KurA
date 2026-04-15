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

# Ensure we don't crash on high-core systems and handle VS mismatch
$env:CMAKE_GENERATOR = "NMake Makefiles"
cargo build --release -j 2
.\target\release\kura.exe
