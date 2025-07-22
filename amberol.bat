@echo off
REM SPDX-FileCopyrightText: 2022  Emmanuele Bassi
REM SPDX-License-Identifier: GPL-3.0-or-later

REM Amberol Windows Launcher Script
REM This batch file sets up the environment and launches Amberol

setlocal

REM Get the directory where this batch file is located
set AMBEROL_DIR=%~dp0

REM Set up GStreamer environment
set GST_PLUGIN_PATH=%AMBEROL_DIR%lib\gstreamer-1.0
set GST_PLUGIN_SYSTEM_PATH=%AMBEROL_DIR%lib\gstreamer-1.0
set GST_REGISTRY=%AMBEROL_DIR%gst-registry.bin

REM Set up GLib/GSettings environment
set GSETTINGS_SCHEMA_DIR=%AMBEROL_DIR%share\glib-2.0\schemas
set XDG_DATA_DIRS=%AMBEROL_DIR%share;%XDG_DATA_DIRS%

REM Set up GTK environment for icon themes
set GTK_DATA_PREFIX=%AMBEROL_DIR%
set GTK_EXE_PREFIX=%AMBEROL_DIR%

REM Set up icon environment (keep original Amberol theme)
REM set GTK_THEME=Adwaita
REM set ICON_THEME=Adwaita

REM Add required DLLs to PATH
set PATH=%AMBEROL_DIR%bin;%PATH%

REM Set GTK/GStreamer preferences for Windows
set GSK_RENDERER=gl
set GTK_USE_PORTAL=0

REM Enable GStreamer debugging (uncomment for troubleshooting)
REM set GST_DEBUG=3
REM set GST_DEBUG_FILE=%AMBEROL_DIR%gstreamer-debug.log

REM Enable GTK icon debugging (uncomment for troubleshooting missing icons)
REM set GTK_DEBUG=icon-theme
REM set G_MESSAGES_DEBUG=all

REM Enable Amberol debug logging (shows icon loading and tray info)
set RUST_LOG=amberol=info

REM Set application data directory
if not defined LOCALAPPDATA (
    set LOCALAPPDATA=%USERPROFILE%\AppData\Local
)

REM Create app data directory if it doesn't exist
if not exist "%LOCALAPPDATA%\io.bassi.Amberol" (
    mkdir "%LOCALAPPDATA%\io.bassi.Amberol"
)

REM Create missing devel-symbolic.svg if libadwaita themes are present but icon is missing
if exist "%AMBEROL_DIR%share\libadwaita-1\styles" (
    if not exist "%AMBEROL_DIR%share\libadwaita-1\styles\assets" (
        mkdir "%AMBEROL_DIR%share\libadwaita-1\styles\assets"
    )
    if not exist "%AMBEROL_DIR%share\libadwaita-1\styles\assets\devel-symbolic.svg" (
        echo Creating missing devel-symbolic.svg...
        echo ^<?xml version="1.0" encoding="UTF-8"?^> > "%AMBEROL_DIR%share\libadwaita-1\styles\assets\devel-symbolic.svg"
        echo ^<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"^> >> "%AMBEROL_DIR%share\libadwaita-1\styles\assets\devel-symbolic.svg"
        echo   ^<circle cx="8" cy="8" r="6" fill="currentColor" opacity="0.3"/^> >> "%AMBEROL_DIR%share\libadwaita-1\styles\assets\devel-symbolic.svg"
        echo ^</svg^> >> "%AMBEROL_DIR%share\libadwaita-1\styles\assets\devel-symbolic.svg"
    )
)

REM Show environment info for debugging
echo Amberol Portable Launcher
echo ========================
echo Application Directory: %AMBEROL_DIR%
echo GStreamer Plugins: %GST_PLUGIN_PATH%
echo GSettings Schemas: %GSETTINGS_SCHEMA_DIR%
echo.

REM Debug: Check for GResource file (contains icons and UI)
echo Checking for application resources...
if exist "%AMBEROL_DIR%bin\amberol.gresource" (echo   ✓ Found GResource in bin/) else (echo   ✗ WARNING: GResource missing in bin/)
if exist "%AMBEROL_DIR%share\amberol.gresource" (echo   ✓ Found GResource in share/) else (echo   ✗ WARNING: GResource missing in share/)
if exist "%AMBEROL_DIR%share\amberol\amberol.gresource" (echo   ✓ Found GResource in share/amberol/) else (echo   ✗ WARNING: GResource missing in share/amberol/)
echo.

REM Try icon workarounds if needed (uncomment one at a time if icons don't show)
REM set GSK_RENDERER=cairo
REM set GTK_THEME=Default

REM Launch Amberol
echo Starting Amberol...
echo   📱 Note: Window will minimize to system tray instead of closing
echo   🖱️ Click the tray icon to restore the window
"%AMBEROL_DIR%bin\amberol.exe" %*

REM Check if the application started successfully
if %ERRORLEVEL% neq 0 (
    echo.
    echo Error: Amberol failed to start with error code %ERRORLEVEL%
    echo.
    echo Troubleshooting:
    echo 1. Make sure all required files are present
    echo 2. Check if Microsoft Visual C++ Redistributable is installed
    echo 3. Verify audio drivers are working
    echo 4. Try running as Administrator
    echo 5. Check if GStreamer plugins are present in: %GST_PLUGIN_PATH%
    echo 6. Check if GSettings schemas are compiled in: %GSETTINGS_SCHEMA_DIR%
    echo 7. If icons are missing, check if amberol.gresource file exists in bin/ or share/
    echo.
    pause
)

endlocal