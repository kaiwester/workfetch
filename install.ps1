<#
.SYNOPSIS
    Install or uninstall workfetch on Windows.

.DESCRIPTION
    Downloads the latest workfetch release from GitHub and installs it to
    $env:LOCALAPPDATA\workfetch. Adds the install directory to the user PATH.

.PARAMETER Version
    Install a specific version (e.g. "v0.2.0"). Defaults to latest.

.PARAMETER Uninstall
    Remove workfetch and clean up PATH.

.EXAMPLE
    # Install latest
    irm https://raw.githubusercontent.com/kaiwester/workfetch/main/install.ps1 | iex

    # Install specific version
    & ([scriptblock]::Create((irm https://raw.githubusercontent.com/kaiwester/workfetch/main/install.ps1))) -Version v0.2.0

    # Uninstall
    & ([scriptblock]::Create((irm https://raw.githubusercontent.com/kaiwester/workfetch/main/install.ps1))) -Uninstall
#>
param(
    [string]$Version,
    [switch]$Uninstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repo = "kaiwester/workfetch"
$BinName = "workfetch.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "workfetch"

function Get-LatestVersion {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    return $release.tag_name
}

function Add-ToUserPath {
    param([string]$Dir)
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -split ";" | Where-Object { $_ -eq $Dir }) {
        Write-Host "  Already on PATH." -ForegroundColor DarkGray
        return
    }
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$Dir", "User")
    $env:Path = "$env:Path;$Dir"
    Write-Host "  Added to user PATH." -ForegroundColor Green
}

function Remove-FromUserPath {
    param([string]$Dir)
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $newPath = ($currentPath -split ";" | Where-Object { $_ -ne $Dir }) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "  Removed from user PATH." -ForegroundColor Yellow
}

# --- Uninstall ---
if ($Uninstall) {
    Write-Host ""
    Write-Host "Uninstalling workfetch..." -ForegroundColor Cyan
    if (Test-Path $InstallDir) {
        Remove-Item -Recurse -Force $InstallDir
        Write-Host "  Removed $InstallDir" -ForegroundColor Yellow
    } else {
        Write-Host "  Install directory not found, skipping." -ForegroundColor DarkGray
    }
    Remove-FromUserPath $InstallDir
    Write-Host ""
    Write-Host "workfetch has been uninstalled." -ForegroundColor Green
    Write-Host "Restart your terminal for PATH changes to take effect." -ForegroundColor DarkGray
    return
}

# --- Install ---
Write-Host ""
Write-Host "Installing workfetch..." -ForegroundColor Cyan

# Determine version
if (-not $Version) {
    Write-Host "  Fetching latest version..." -ForegroundColor DarkGray
    $Version = Get-LatestVersion
}
Write-Host "  Version: $Version" -ForegroundColor White

# Determine asset URL
$Target = "x86_64-pc-windows-msvc"
$AssetName = "workfetch-$Version-$Target.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$AssetName"

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Download
$ZipPath = Join-Path $env:TEMP $AssetName
Write-Host "  Downloading $AssetName..." -ForegroundColor DarkGray
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
} catch {
    Write-Host "  Failed to download from: $DownloadUrl" -ForegroundColor Red
    Write-Host "  Please check that version '$Version' exists." -ForegroundColor Red
    exit 1
}

# Extract
Write-Host "  Extracting..." -ForegroundColor DarkGray
$TempExtract = Join-Path $env:TEMP "workfetch-extract"
if (Test-Path $TempExtract) { Remove-Item -Recurse -Force $TempExtract }
Expand-Archive -Path $ZipPath -DestinationPath $TempExtract -Force

# Find the binary (may be in a subdirectory)
$Binary = Get-ChildItem -Path $TempExtract -Recurse -Filter $BinName | Select-Object -First 1
if (-not $Binary) {
    Write-Host "  Could not find $BinName in the archive." -ForegroundColor Red
    exit 1
}
Copy-Item -Path $Binary.FullName -Destination (Join-Path $InstallDir $BinName) -Force

# Cleanup temp files
Remove-Item -Force $ZipPath -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force $TempExtract -ErrorAction SilentlyContinue

# Add to PATH
Add-ToUserPath $InstallDir

Write-Host ""
Write-Host "workfetch $Version installed successfully!" -ForegroundColor Green
Write-Host "  Location: $InstallDir\$BinName" -ForegroundColor White
Write-Host ""
Write-Host "Restart your terminal, then run:" -ForegroundColor DarkGray
Write-Host "  workfetch" -ForegroundColor White
Write-Host ""
