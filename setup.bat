@echo off
setlocal enabledelayedexpansion

echo KurA setup
echo.

set /p TOKEN=Discord bot token (DISCORD_TOKEN): 
if "%TOKEN%"=="" (
  echo Token is required.
  exit /b 1
)

set /p OPUS_CACHE=OPUS cache folder [./music_opus]: 
if "%OPUS_CACHE%"=="" set OPUS_CACHE=./music_opus

set /p RUST_LOG=Log level (warn/info) [warn]: 
if "%RUST_LOG%"=="" set RUST_LOG=warn

echo DISCORD_TOKEN=%TOKEN%> kura.env
echo OPUS_CACHE=%OPUS_CACHE%>> kura.env
echo RUST_LOG=%RUST_LOG%>> kura.env

echo.
echo Wrote kura.env
echo.

echo To run in this terminal:
echo   call kura.env
echo   target\release\kura_voice.exe

echo.
echo To set permanently (optional):
echo   setx DISCORD_TOKEN "%TOKEN%"
echo   setx OPUS_CACHE "%OPUS_CACHE%"
echo   setx RUST_LOG "%RUST_LOG%"
