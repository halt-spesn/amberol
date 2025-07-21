# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

# Windows Build Script for Amberol
# This script builds Amberol using MSYS2 with GTK4, libadwaita, and GStreamer

param(
    [switch]$Install,
    [switch]$Clean,
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "Building Amberol for Windows..." -ForegroundColor Green

# Check if MSYS2 is installed
$msys2Path = "C:\msys64"
if (-not (Test-Path $msys2Path)) {
    Write-Error "MSYS2 not found at $msys2Path. Please install MSYS2 first."
    exit 1
}

# Required MSYS2 packages
$packages = @(
    "mingw-w64-x86_64-gtk4",
    "mingw-w64-x86_64-libadwaita", 
    "mingw-w64-x86_64-gstreamer",
    "mingw-w64-x86_64-gst-plugins-base",
    "mingw-w64-x86_64-gst-plugins-good",
    "mingw-w64-x86_64-gst-plugins-bad",
    "mingw-w64-x86_64-gst-plugins-ugly",
    "mingw-w64-x86_64-rust",
    "mingw-w64-x86_64-pkg-config",
    "mingw-w64-x86_64-meson",
    "mingw-w64-x86_64-ninja",
    "mingw-w64-x86_64-gettext",
    "mingw-w64-x86_64-desktop-file-utils"
)

Write-Host "Setting up MSYS2 environment..." -ForegroundColor Yellow

# Update MSYS2 and install packages
$installCmd = "pacman -Syu --noconfirm; pacman -S --needed --noconfirm " + ($packages -join " ")
& "$msys2Path\usr\bin\bash.exe" -lc $installCmd

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to install MSYS2 packages"
    exit 1
}

# Set environment variables for the build
$env:PKG_CONFIG_PATH = "$msys2Path\mingw64\lib\pkgconfig"
$env:PATH = "$msys2Path\mingw64\bin;$env:PATH"
$env:MSYSTEM = "MINGW64"

Write-Host "Building Amberol..." -ForegroundColor Yellow

if ($Clean) {
    Write-Host "Cleaning previous build..." -ForegroundColor Cyan
    Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "_build" -Recurse -Force -ErrorAction SilentlyContinue
}

# Configure meson build
$mesonArgs = @(
    "setup",
    "_build",
    "--buildtype=$Profile",
    "--prefix=C:/Program Files/Amberol"
)

& "$msys2Path\usr\bin\bash.exe" -lc "cd '$PWD'; meson $($mesonArgs -join ' ')"

if ($LASTEXITCODE -ne 0) {
    Write-Error "Meson configuration failed"
    exit 1
}

# Build the project
& "$msys2Path\usr\bin\bash.exe" -lc "cd '$PWD'; meson compile -C _build"

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit 1
}

Write-Host "Build completed successfully!" -ForegroundColor Green

if ($Install) {
    Write-Host "Installing Amberol..." -ForegroundColor Yellow
    & "$msys2Path\usr\bin\bash.exe" -lc "cd '$PWD'; meson install -C _build"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Installation completed!" -ForegroundColor Green
    } else {
        Write-Error "Installation failed"
        exit 1
    }
}

# Create a portable distribution
Write-Host "Creating portable distribution..." -ForegroundColor Yellow

$distDir = "amberol-windows-portable"
$binDir = "$distDir\bin"
$libDir = "$distDir\lib"
$shareDir = "$distDir\share"

# Create directories
New-Item -ItemType Directory -Force -Path $binDir
New-Item -ItemType Directory -Force -Path $libDir  
New-Item -ItemType Directory -Force -Path $shareDir

# Copy main executable
Copy-Item "_build\src\amberol.exe" $binDir

# Copy required DLLs from MSYS2
$dllsNeeded = @(
    "libgtk-4-1.dll",
    "libadwaita-1-0.dll", 
    "libgstreamer-1.0-0.dll",
    "libgst*",
    "libglib-2.0-0.dll",
    "libgobject-2.0-0.dll",
    "libgio-2.0-0.dll",
    "libcairo-2.dll",
    "libpango-1.0-0.dll",
    "libharfbuzz-0.dll",
    "libfreetype-6.dll",
    "libfontconfig-1.dll",
    "libexpat-1.dll",
    "libbrotlidec.dll",
    "libbrotlicommon.dll",
    "libpng16-16.dll",
    "libffi-7.dll",
    "libintl-8.dll",
    "libiconv-2.dll",
    "libpcre-1.dll",
    "zlib1.dll",
    "libwinpthread-1.dll",
    "libgcc_s_seh-1.dll",
    "libstdc++-6.dll"
)

foreach ($dll in $dllsNeeded) {
    $source = "$msys2Path\mingw64\bin\$dll"
    if (Test-Path $source) {
        Copy-Item $source $binDir -Force
    }
}

# Copy GStreamer plugins
$gstPluginDir = "$msys2Path\mingw64\lib\gstreamer-1.0"
if (Test-Path $gstPluginDir) {
    Copy-Item $gstPluginDir "$libDir\gstreamer-1.0" -Recurse -Force
}

# Copy application resources
Copy-Item "_build\data\amberol.gresource" $shareDir -ErrorAction SilentlyContinue

# Copy icon themes and other resources
$iconsDir = "$msys2Path\mingw64\share\icons"
if (Test-Path $iconsDir) {
    Copy-Item "$iconsDir\Adwaita" "$shareDir\icons\Adwaita" -Recurse -Force -ErrorAction SilentlyContinue
}

# Create a run script
$runScript = @"
@echo off
set GST_PLUGIN_PATH=%~dp0lib\gstreamer-1.0
set PATH=%~dp0bin;%PATH%
start "" "%~dp0bin\amberol.exe" %*
"@

$runScript | Out-File -FilePath "$distDir\amberol.bat" -Encoding ASCII

Write-Host "Portable distribution created in $distDir" -ForegroundColor Green
Write-Host "You can run Amberol by executing amberol.bat" -ForegroundColor Cyan

Write-Host "Build process completed!" -ForegroundColor Green