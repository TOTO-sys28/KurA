@echo off
REM KurA launcher (Windows)

IF "%DISCORD_TOKEN%"=="" (
  echo DISCORD_TOKEN is required.
  exit /b 1
)

IF "%OPUS_CACHE%"=="" set OPUS_CACHE=./music_opus
IF "%RUST_LOG%"=="" set RUST_LOG=warn

target\release\kura_voice.exe
