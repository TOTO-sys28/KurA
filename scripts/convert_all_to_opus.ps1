Param(
    [string]$SRC = "./music",
    [string]$OUT = "./music_opus",
    [string]$BITRATE = "64k"
)

$ErrorActionPreference = "Stop"

# 1. Check for ffmpeg
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    Write-Error "ffmpeg not found. Install it first."
    exit 1
}

# 2. Get absolute paths safely
$SRC = (Resolve-Path -LiteralPath $SRC).Path
$OUT = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OUT)

if (-not (Test-Path -Path $OUT)) {
    New-Item -ItemType Directory -Force -Path $OUT | Out-Null
}

# 3. Corrected Extension Regex
$files = Get-ChildItem -Path $SRC -Recurse -File | Where-Object {
    $_.Extension -match "(?i)^\.(mp3|flac|wav|m4a|ogg)$"
}

foreach ($f in $files) {
    # 4. Compatible Relative Path logic (Works in PS 5.1 and 7)
    $relPath = $f.FullName.Substring($SRC.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar)
    $dst = Join-Path $OUT ([System.IO.Path]::ChangeExtension($relPath, ".opus"))
    $dstDir = Split-Path -Parent $dst

    if (-not (Test-Path -Path $dstDir)) {
        New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
    }

    if (Test-Path -LiteralPath $dst) {
        Write-Host "Skip (exists): $dst" -ForegroundColor Cyan
        continue
    }

    Write-Host "Converting: $($f.Name)" -ForegroundColor Green

    # 5. Optimized ffmpeg call
    & ffmpeg -nostdin -hide_banner -loglevel error -y `
        -i "$($f.FullName)" `
        -vn -ar 48000 -ac 2 `
        -c:a libopus -b:a $BITRATE -vbr on `
        -application audio -compression_level 10 `
        "$dst"
}

Write-Host "Done." -ForegroundColor Yellow
