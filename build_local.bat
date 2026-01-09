@echo off
REM Amberol Local Build Script for Windows
REM Usage: build_local.bat [debug|release] [clean] [run] [build]

setlocal enabledelayedexpansion

set "PROFILE=debug"
set "CLEAN=0"
set "RUN=0"
set "BUILD=1"
set "MSYS2_PATH=C:\msys64"
set "DIST=dist-windows"
set "BIN=%DIST%\bin"
set "LIB=%DIST%\lib"
set "SHARE=%DIST%\share"

:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="release" set "PROFILE=release"
if /i "%~1"=="debug" set "PROFILE=debug"
if /i "%~1"=="clean" set "CLEAN=1"
if /i "%~1"=="run" (
    set "RUN=1"
    set "BUILD=0"
)
if /i "%~1"=="build" set "BUILD=1"
shift
goto parse_args
:end_parse

echo.
echo ===================================================
echo    AMBEROL LOCAL BUILD SCRIPT FOR WINDOWS
echo ===================================================
echo.
echo Build Profile: %PROFILE%
echo.

REM Check for MSYS2
if not exist "%MSYS2_PATH%" (
    echo ERROR: MSYS2 not found at %MSYS2_PATH%
    echo Please install MSYS2 from: https://www.msys2.org/
    exit /b 1
)
echo MSYS2 found at %MSYS2_PATH%

REM Clean if requested
if "%CLEAN%"=="1" (
    echo Cleaning previous builds...
    if exist "target" rmdir /s /q "target" 2>nul
    if exist "_build" rmdir /s /q "_build" 2>nul
    if exist "%DIST%" rmdir /s /q "%DIST%" 2>nul
    echo Cleaned.
)

REM Skip build if only running
if "%BUILD%"=="0" goto skip_build

REM Configure Meson
echo.
echo Configuring Meson build...
"%MSYS2_PATH%\usr\bin\bash.exe" -lc "export MSYSTEM=MINGW64 && export PATH=/mingw64/bin:$PATH && export PKG_CONFIG_PATH=/mingw64/lib/pkgconfig && cd '%CD%' && if [ ! -d '_build' ]; then meson setup _build --buildtype=%PROFILE% --prefix=/mingw64 -Dprofile=default; else meson setup --reconfigure _build --buildtype=%PROFILE% --prefix=/mingw64 -Dprofile=default; fi"

if errorlevel 1 (
    echo ERROR: Meson configuration failed
    exit /b 1
)
echo Meson configuration completed.

REM Copy generated config.rs to src directory
echo Copying generated config.rs...
copy /y "_build\src\config.rs" "src\config.rs" >nul
if errorlevel 1 (
    echo WARNING: Could not copy config.rs
)

REM Build
echo.
echo Building Amberol...
"%MSYS2_PATH%\usr\bin\bash.exe" -lc "export MSYSTEM=MINGW64 && export PATH=/mingw64/bin:$PATH && export PKG_CONFIG_PATH=/mingw64/lib/pkgconfig && cd '%CD%' && meson compile -C _build"

if errorlevel 1 (
    echo ERROR: Build failed
    exit /b 1
)

echo.
echo Build completed successfully!

REM Check if executable exists
if not exist "_build\src\amberol.exe" (
    echo ERROR: Executable not found at _build\src\amberol.exe
    exit /b 1
)

REM Create distribution layout
echo.
echo Creating distribution bundle...
if not exist "%BIN%" mkdir "%BIN%"
if not exist "%LIB%" mkdir "%LIB%"
if not exist "%SHARE%" mkdir "%SHARE%"
if not exist "%SHARE%\amberol" mkdir "%SHARE%\amberol"

REM Copy executable
copy /y "_build\src\amberol.exe" "%BIN%\amberol.exe" >nul

REM Copy ICO file to bin directory for tray icon
echo Copying ICO file for tray icon...
if exist "data\icons\hicolor\scalable\apps\io.bassi.Amberol.ico" (
    copy /y "data\icons\hicolor\scalable\apps\io.bassi.Amberol.ico" "%BIN%\io.bassi.Amberol.ico" >nul
    echo   ICO file copied to bin directory.
)

REM Copy ALL DLLs from MSYS2 mingw64/bin
echo Copying DLLs from MSYS2...
copy /y "%MSYS2_PATH%\mingw64\bin\*.dll" "%BIN%\" >nul 2>&1

REM Copy GStreamer plugins
echo Copying GStreamer plugins...
if exist "%MSYS2_PATH%\mingw64\lib\gstreamer-1.0" (
    xcopy "%MSYS2_PATH%\mingw64\lib\gstreamer-1.0" "%LIB%\gstreamer-1.0\" /E /I /Y /Q >nul
)

REM Copy GStreamer helper executables (needed for plugin scanning)
echo Copying GStreamer helpers...
if not exist "%LIB%\gstreamer-1.0\helpers" mkdir "%LIB%\gstreamer-1.0\helpers"
if exist "%MSYS2_PATH%\mingw64\libexec\gstreamer-1.0\gst-plugin-scanner.exe" (
    copy /y "%MSYS2_PATH%\mingw64\libexec\gstreamer-1.0\gst-plugin-scanner.exe" "%LIB%\gstreamer-1.0\helpers\" >nul
    echo   gst-plugin-scanner.exe copied.
)

REM Copy GLib schemas from MSYS2
echo Copying GLib schemas...
if exist "%MSYS2_PATH%\mingw64\share\glib-2.0\schemas" (
    xcopy "%MSYS2_PATH%\mingw64\share\glib-2.0\schemas" "%SHARE%\glib-2.0\schemas\" /E /I /Y /Q >nul
)

REM Copy app schema from source and build
echo Copying app schema files...
if exist "data\io.bassi.Amberol.gschema.xml" (
    copy /y "data\io.bassi.Amberol.gschema.xml" "%SHARE%\glib-2.0\schemas\" >nul
    echo   Found: data\io.bassi.Amberol.gschema.xml
)
for /r "_build" %%f in (*.gschema.xml) do (
    echo   Found: %%f
    copy /y "%%f" "%SHARE%\glib-2.0\schemas\" >nul
)

REM Compile schemas
echo Compiling GLib schemas...
"%MSYS2_PATH%\mingw64\bin\glib-compile-schemas.exe" "%SHARE%\glib-2.0\schemas"
if exist "%SHARE%\glib-2.0\schemas\gschemas.compiled" (
    echo Schema compilation successful.
) else (
    echo WARNING: Schema compilation may have failed!
)

REM Copy icons (Adwaita and hicolor)
echo Copying icon themes...
if exist "%MSYS2_PATH%\mingw64\share\icons\Adwaita" (
    xcopy "%MSYS2_PATH%\mingw64\share\icons\Adwaita" "%SHARE%\icons\Adwaita\" /E /I /Y /Q >nul
    echo   Adwaita icons copied.
)
if exist "%MSYS2_PATH%\mingw64\share\icons\hicolor" (
    xcopy "%MSYS2_PATH%\mingw64\share\icons\hicolor" "%SHARE%\icons\hicolor\" /E /I /Y /Q >nul
    echo   hicolor icons copied.
)

REM Copy Amberol app icons to hicolor theme
echo Copying Amberol app icons...
if not exist "%SHARE%\icons\hicolor\scalable\apps" mkdir "%SHARE%\icons\hicolor\scalable\apps"
if exist "data\icons\hicolor\scalable\apps\io.bassi.Amberol.svg" (
    copy /y "data\icons\hicolor\scalable\apps\io.bassi.Amberol.svg" "%SHARE%\icons\hicolor\scalable\apps\" >nul
    echo   Amberol scalable icon copied.
)
if exist "data\icons\hicolor\scalable\apps\io.bassi.Amberol.Devel.svg" (
    copy /y "data\icons\hicolor\scalable\apps\io.bassi.Amberol.Devel.svg" "%SHARE%\icons\hicolor\scalable\apps\" >nul
)
if not exist "%SHARE%\icons\hicolor\symbolic\apps" mkdir "%SHARE%\icons\hicolor\symbolic\apps"
if exist "data\icons\hicolor\symbolic\apps" (
    xcopy "data\icons\hicolor\symbolic\apps\*" "%SHARE%\icons\hicolor\symbolic\apps\" /E /I /Y /Q >nul
    echo   Amberol symbolic icons copied.
)

REM Rebuild icon theme caches
echo Rebuilding icon theme caches...
if exist "%MSYS2_PATH%\mingw64\bin\gtk4-update-icon-cache.exe" (
    "%MSYS2_PATH%\mingw64\bin\gtk4-update-icon-cache.exe" -f -t "%SHARE%\icons\Adwaita" >nul 2>&1
    "%MSYS2_PATH%\mingw64\bin\gtk4-update-icon-cache.exe" -f -t "%SHARE%\icons\hicolor" >nul 2>&1
    echo   Icon caches updated.
) else if exist "%MSYS2_PATH%\mingw64\bin\gtk-update-icon-cache.exe" (
    "%MSYS2_PATH%\mingw64\bin\gtk-update-icon-cache.exe" -f -t "%SHARE%\icons\Adwaita" >nul 2>&1
    "%MSYS2_PATH%\mingw64\bin\gtk-update-icon-cache.exe" -f -t "%SHARE%\icons\hicolor" >nul 2>&1
    echo   Icon caches updated.
)

REM Copy pixbuf loaders
echo Copying pixbuf loaders...
if exist "%MSYS2_PATH%\mingw64\lib\gdk-pixbuf-2.0" (
    xcopy "%MSYS2_PATH%\mingw64\lib\gdk-pixbuf-2.0" "%LIB%\gdk-pixbuf-2.0\" /E /I /Y /Q >nul
)

REM Copy GResource file to ALL locations (critical for UI)
echo Copying GResource file...
for /r "_build" %%f in (*.gresource) do (
    echo   Found: %%f
    copy /y "%%f" "%BIN%\" >nul
    copy /y "%%f" "%SHARE%\" >nul
    copy /y "%%f" "%SHARE%\amberol\" >nul
)

REM Copy GTK 4.0 resources
echo Copying GTK 4.0 resources...
if exist "%MSYS2_PATH%\mingw64\share\gtk-4.0" (
    xcopy "%MSYS2_PATH%\mingw64\share\gtk-4.0" "%SHARE%\gtk-4.0\" /E /I /Y /Q >nul
)

REM Copy locale files
echo Copying locale files...
if exist "%MSYS2_PATH%\mingw64\share\locale" (
    xcopy "%MSYS2_PATH%\mingw64\share\locale" "%SHARE%\locale\" /E /I /Y /Q >nul
)

REM Compile Amberol translations from po/*.po to locale/*/LC_MESSAGES/amberol.mo
echo Compiling Amberol translations...
set "MSGFMT=%MSYS2_PATH%\mingw64\bin\msgfmt.exe"
if exist "%MSGFMT%" (
    for %%f in (po\*.po) do (
        set "LANG=%%~nf"
        setlocal enabledelayedexpansion
        if not exist "%SHARE%\locale\!LANG!\LC_MESSAGES" mkdir "%SHARE%\locale\!LANG!\LC_MESSAGES"
        "%MSGFMT%" -o "%SHARE%\locale\!LANG!\LC_MESSAGES\amberol.mo" "%%f"
        if !ERRORLEVEL! EQU 0 (
            echo   Compiled: !LANG!
        ) else (
            echo   WARNING: Failed to compile: !LANG!
        )
        endlocal
    )
) else (
    echo WARNING: msgfmt not found, translations will not be available
)

REM Create portable launcher script (like CI does)
echo Creating launcher script...
(
echo @echo off
echo REM Amberol Portable - Self-Contained with All Dependencies
echo setlocal EnableDelayedExpansion
echo set AMBEROL_DIR=%%~dp0
echo set PATH=%%AMBEROL_DIR%%bin;%%PATH%%
echo set GST_PLUGIN_PATH=%%AMBEROL_DIR%%lib\gstreamer-1.0
echo set GST_PLUGIN_SYSTEM_PATH=%%AMBEROL_DIR%%lib\gstreamer-1.0
echo set GST_REGISTRY=%%AMBEROL_DIR%%gst-registry.bin
echo set GSETTINGS_SCHEMA_DIR=%%AMBEROL_DIR%%share\glib-2.0\schemas
echo set XDG_DATA_DIRS=%%AMBEROL_DIR%%share
echo set GDK_PIXBUF_MODULE_FILE=%%AMBEROL_DIR%%lib\gdk-pixbuf-2.0\2.10.0\loaders.cache
echo set GDK_PIXBUF_MODULEDIR=%%AMBEROL_DIR%%lib\gdk-pixbuf-2.0\2.10.0\loaders
echo set GTK_DATA_PREFIX=%%AMBEROL_DIR%%
echo set GTK_EXE_PREFIX=%%AMBEROL_DIR%%
echo set GST_PLUGIN_SCANNER=%%AMBEROL_DIR%%lib\gstreamer-1.0\helpers\gst-plugin-scanner.exe
echo set GSK_RENDERER=gl
echo set GTK_USE_PORTAL=0
echo set ADW_DISABLE_PORTAL=1
echo set XDG_DATA_HOME=%%AMBEROL_DIR%%share
echo set LOCALEDIR=%%AMBEROL_DIR%%share\locale
echo "%%AMBEROL_DIR%%bin\amberol.exe" %%*
echo endlocal
) > "%DIST%\amberol.bat"

echo.
echo ===================================================
echo    BUILD SUMMARY
echo ===================================================
echo   Profile:     %PROFILE%
echo   Executable:  %BIN%\amberol.exe
echo   Launcher:    %DIST%\amberol.bat
echo ===================================================
echo.

:skip_build

REM Run if requested
if "%RUN%"=="1" (
    echo Launching Amberol...
    echo.
    
    REM Use the launcher script which sets up all env vars properly
    pushd "%DIST%"
    call amberol.bat
    popd
    
    echo.
    echo Exit code: %ERRORLEVEL%
    pause
) else (
    echo To run Amberol:
    echo   cd %DIST% ^&^& amberol.bat
    echo.
    echo Or:
    echo   build_local.bat run
    echo.
)

endlocal
