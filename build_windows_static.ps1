# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

# Amberol Windows Static Build Script
# This script builds Amberol with all dependencies statically linked

param(
    [switch]$Clean = $false
)

Write-Host "🚀 Building Amberol for Windows (Static)" -ForegroundColor Green

# Check if we're in MSYS2 environment
if (-not $env:MSYSTEM) {
    Write-Error "This script must be run in MSYS2 environment"
    exit 1
}

# Clean previous builds if requested
if ($Clean) {
    Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force _build -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force target -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force amberol-windows-static -ErrorAction SilentlyContinue
}

# Required MSYS2 packages for static building
$required_packages = @(
    "mingw-w64-x86_64-gtk4",
    "mingw-w64-x86_64-libadwaita", 
    "mingw-w64-x86_64-gstreamer",
    "mingw-w64-x86_64-gst-plugins-base",
    "mingw-w64-x86_64-gst-plugins-good",
    "mingw-w64-x86_64-gst-plugins-bad",
    "mingw-w64-x86_64-gst-plugins-ugly",
    "mingw-w64-x86_64-gst-libav",
    "mingw-w64-x86_64-rust",
    "mingw-w64-x86_64-meson",
    "mingw-w64-x86_64-ninja",
    "mingw-w64-x86_64-pkg-config",
    "mingw-w64-x86_64-gettext",
    "mingw-w64-x86_64-gcc",
    "mingw-w64-x86_64-gcc-libs"
)

Write-Host "📦 Installing required packages..." -ForegroundColor Blue
foreach ($package in $required_packages) {
    Write-Host "  - $package"
}

# Update MSYS2 and install packages
pacman -Syu --noconfirm
pacman -S --needed --noconfirm $required_packages

# Set up environment for static linking
$env:PKG_CONFIG_ALL_STATIC = "1"
$env:PKG_CONFIG_ALLOW_CROSS = "1"
$env:GSTREAMER_1_0_STATIC = "1"
$env:GTK4_STATIC = "1"
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "x86_64-w64-mingw32-gcc"

# Set up Rust flags for static linking
$env:RUSTFLAGS = "-C target-feature=+crt-static -C link-arg=-static -C link-arg=-static-libgcc -C link-arg=-static-libstdc++"

Write-Host "🔧 Configuring Meson for static build..." -ForegroundColor Blue
meson setup _build --buildtype=release --default-library=static --prefer-static

Write-Host "🏗️ Building Amberol..." -ForegroundColor Blue
meson compile -C _build

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed!"
    exit 1
}

Write-Host "📁 Creating static distribution..." -ForegroundColor Blue
$distDir = "amberol-windows-static"
$binDir = "$distDir\bin"

New-Item -ItemType Directory -Force -Path $binDir | Out-Null

# Copy the statically linked executable
Copy-Item "_build\src\amberol.exe" $binDir -Force

# Copy only essential files (no DLLs needed for static build)
Copy-Item "amberol.bat" $distDir -Force
Copy-Item "README-Windows.md" $distDir -Force
Copy-Item "LICENSES\GPL-3.0-or-later.txt" "$distDir\LICENSE.txt" -Force

# Create a simple launcher for static build
$staticLauncher = @"
@echo off
REM Amberol Static Windows Launcher
REM All dependencies are statically linked into the executable

setlocal

REM Get the directory where this batch file is located
set AMBEROL_DIR=%~dp0

REM Set application data directory
if not defined LOCALAPPDATA (
    set LOCALAPPDATA=%USERPROFILE%\AppData\Local
)

REM Create app data directory if it doesn't exist
if not exist "%LOCALAPPDATA%\io.bassi.Amberol" (
    mkdir "%LOCALAPPDATA%\io.bassi.Amberol"
)

REM Launch Amberol (static build - no DLL dependencies)
echo Starting Amberol (Static Build)...
"%AMBEROL_DIR%bin\amberol.exe" %*

REM Check if the application started successfully
if %ERRORLEVEL% neq 0 (
    echo.
    echo Error: Amberol failed to start with error code %ERRORLEVEL%
    echo.
    echo This is a static build - no external DLLs should be required.
    echo If you're getting errors, please check:
    echo 1. Windows version compatibility (Windows 10/11 required)
    echo 2. Audio drivers are working
    echo 3. Try running as Administrator
    echo.
    pause
)

endlocal
"@

Set-Content -Path "$distDir\amberol-static.bat" -Value $staticLauncher

# Check executable dependencies
Write-Host "🔍 Checking executable dependencies..." -ForegroundColor Blue
& objdump -p "$binDir\amberol.exe" | Select-String "DLL Name:"

$exeSize = (Get-Item "$binDir\amberol.exe").Length / 1MB
Write-Host "📊 Static executable size: $($exeSize.ToString('F2')) MB" -ForegroundColor Green

# Create ZIP package
Write-Host "📦 Creating static distribution package..." -ForegroundColor Blue
if (Test-Path "amberol-windows-static.zip") {
    Remove-Item "amberol-windows-static.zip" -Force
}
Compress-Archive -Path "$distDir\*" -DestinationPath "amberol-windows-static.zip"

Write-Host "✅ Static build completed successfully!" -ForegroundColor Green
Write-Host "📁 Distribution: amberol-windows-static.zip" -ForegroundColor Yellow
Write-Host "🚀 Executable: $binDir\amberol.exe" -ForegroundColor Yellow
Write-Host "💾 Size: $($exeSize.ToString('F2')) MB" -ForegroundColor Yellow