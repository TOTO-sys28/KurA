Param(
  [Parameter(Mandatory=$true)][string]$SourceDir,
  [Parameter(Mandatory=$true)][string]$Version,
  [string]$OutDir = "$(Join-Path $PSScriptRoot 'dist')"
)

$ErrorActionPreference = "Stop"

function Assert-File($Path, $Name) {
  if (-not (Test-Path -LiteralPath $Path)) {
    throw "$Name not found at $Path"
  }
}

# MSI version must be MAJOR.MINOR.BUILD (no prerelease/build metadata)
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
  throw "Version must be MAJOR.MINOR.PATCH (e.g. 0.1.0). Got: $Version"
}

# Locate WiX Toolset if not in PATH
$candle = "candle.exe"
$light = "light.exe"

if (-not (Get-Command $candle -ErrorAction SilentlyContinue)) {
    $wixPaths = @(
        "C:\Program Files (x86)\WiX Toolset v3.14\bin",
        "C:\Program Files (x86)\WiX Toolset v3.11\bin",
        "C:\Program Files\WiX Toolset v3.14\bin",
        "C:\Program Files\WiX Toolset v3.11\bin"
    )
    foreach ($p in $wixPaths) {
        if (Test-Path (Join-Path $p "candle.exe")) {
            $candle = Join-Path $p "candle.exe"
            $light = Join-Path $p "light.exe"
            Write-Host "Found WiX at $p" -ForegroundColor Cyan
            break
        }
    }
}

$kura = Join-Path $SourceDir "kura.exe"
$kurac = Join-Path $SourceDir "kurac.exe"
Assert-File $kura "kura.exe"
Assert-File $kurac "kurac.exe"

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$wxs = Join-Path $PSScriptRoot "kura.wxs"
Assert-File $wxs "kura.wxs"

$msiName = "KurA-$Version-x64.msi"
$obj = Join-Path $OutDir "kura.wixobj"
$msi = Join-Path $OutDir $msiName

# WiX Toolset v3: candle/light expected on PATH or located above
Write-Host "Building MSI $msiName from $SourceDir (Version=$Version)"

$ext = ""
# Check for WixUIExtension.dll in the same bin folder
if ($candle -ne "candle.exe") {
    $binDir = [System.IO.Path]::GetDirectoryName($candle)
    $extPath = Join-Path $binDir "WixUIExtension.dll"
    if (Test-Path $extPath) {
        $ext = "-ext `"$extPath`""
    }
} else {
    $ext = "-ext WixUIExtension"
}

# Resolve absolute paths to ensure they work when we change directory
$absSourceDir = [System.IO.Path]::GetFullPath($SourceDir)
$absObj = [System.IO.Path]::GetFullPath($obj)
$absMsi = [System.IO.Path]::GetFullPath($msi)
$absWxs = [System.IO.Path]::GetFullPath($wxs)

Push-Location $PSScriptRoot
try {
    Invoke-Expression "& `"$candle`" -nologo -dSourceDir=`"$absSourceDir`" -dProductVersion=`"$Version`" $ext -out `"$absObj`" `"$absWxs`""
    Invoke-Expression "& `"$light`" -nologo $ext -out `"$absMsi`" `"$absObj`""
} finally {
    Pop-Location
}

Write-Host "MSI written to $msi"
