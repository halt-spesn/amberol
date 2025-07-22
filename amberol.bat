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

REM Set application data directory
if not defined LOCALAPPDATA (
    set LOCALAPPDATA=%USERPROFILE%\AppData\Local
)

REM Create app data directory if it doesn't exist
if not exist "%LOCALAPPDATA%\io.bassi.Amberol" (
    mkdir "%LOCALAPPDATA%\io.bassi.Amberol"
)

REM Show environment info for debugging
echo Amberol Portable Launcher
echo ========================
echo Application Directory: %AMBEROL_DIR%
echo GStreamer Plugins: %GST_PLUGIN_PATH%
echo GSettings Schemas: %GSETTINGS_SCHEMA_DIR%
echo.

REM Launch Amberol
echo Starting Amberol...
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
    echo.
    pause
)

endlocal