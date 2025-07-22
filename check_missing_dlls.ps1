# Amberol Windows Portable - Missing DLL Checker
# This script helps identify missing DLLs in your portable build

param(
    [string]$AmberolPath = ".\amberol-windows-static\bin\amberol.exe"
)

Write-Host "🔍 Amberol Portable - Missing DLL Checker" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

if (-not (Test-Path $AmberolPath)) {
    Write-Host "❌ Error: Could not find amberol.exe at: $AmberolPath" -ForegroundColor Red
    Write-Host "Usage: .\check_missing_dlls.ps1 [path_to_amberol.exe]" -ForegroundColor Yellow
    exit 1
}

$binDir = Split-Path $AmberolPath -Parent
Write-Host "📁 Checking: $AmberolPath" -ForegroundColor Blue
Write-Host "📁 Bin directory: $binDir" -ForegroundColor Blue

# Check if objdump is available
try {
    $null = & objdump --version 2>$null
    $objdumpAvailable = $true
} catch {
    $objdumpAvailable = $false
    Write-Host "⚠️ objdump not available - using basic checks only" -ForegroundColor Yellow
}

$missingDlls = @()
$foundDlls = @()

if ($objdumpAvailable) {
    Write-Host "🔍 Analyzing dependencies with objdump..." -ForegroundColor Blue
    
    try {
        $deps = & objdump -p $AmberolPath 2>$null | Select-String "DLL Name:"
        
        foreach ($dep in $deps) {
            if ($dep -match "DLL Name:\s*(.+)") {
                $dllName = $matches[1].Trim()
                
                # Skip system DLLs
                if ($dllName -match "^(kernel32|user32|ntdll|msvcrt|advapi32|ole32|shell32|ws2_32|winmm|version|comctl32|gdi32|comdlg32|winspool|oleaut32|uuid)\.dll$") {
                    continue
                }
                
                $dllPath = Join-Path $binDir $dllName
                if (Test-Path $dllPath) {
                    $foundDlls += $dllName
                } else {
                    $missingDlls += $dllName
                }
            }
        }
    } catch {
        Write-Host "❌ Error analyzing dependencies: $_" -ForegroundColor Red
    }
}

# Manual check for commonly needed DLLs
$commonDlls = @(
    "libgtk-4-1.dll", "libadwaita-1-0.dll", "libgraphene-1.0-0.dll",
    "libgstreamer-1.0-0.dll", "libgstaudio-1.0-0.dll", "libgstplayer-1.0-0.dll",
    "libglib-2.0-0.dll", "libgobject-2.0-0.dll", "libgio-2.0-0.dll",
    "libcairo-2.dll", "libpango-1.0-0.dll", "libharfbuzz-0.dll",
    "libffi-8.dll", "liborc-0.4-0.dll", "libappstream-5.dll", "libtiff-6.dll",
    
    # Common GStreamer codec dependencies
    "libavcodec-61.dll", "libavformat-61.dll", "libavutil-59.dll",
    "libfaad-2.dll", "libmodplug-1.dll", "libmpeg2-0.dll", "libopenjp2-7.dll",
    "libspeex-1.dll", "libtheora-0.dll", "libvpx-8.dll", "libwavpack-1.dll",
    "libx264-164.dll", "libass-9.dll", "libcdio-19.dll"
)

Write-Host "📋 Checking common required DLLs..." -ForegroundColor Blue
foreach ($dll in $commonDlls) {
    $dllPath = Join-Path $binDir $dll
    if (Test-Path $dllPath) {
        if ($dll -notin $foundDlls) { $foundDlls += $dll }
    } else {
        if ($dll -notin $missingDlls) { $missingDlls += $dll }
    }
}

# Results
Write-Host ""
Write-Host "📊 Results:" -ForegroundColor Yellow
Write-Host "============" -ForegroundColor Yellow

if ($foundDlls.Count -gt 0) {
    Write-Host "✅ Found DLLs ($($foundDlls.Count)):" -ForegroundColor Green
    $foundDlls | Sort-Object | ForEach-Object { Write-Host "  ✅ $_" -ForegroundColor Green }
}

if ($missingDlls.Count -gt 0) {
    Write-Host "❌ Missing DLLs ($($missingDlls.Count)):" -ForegroundColor Red
    $missingDlls | Sort-Object | ForEach-Object { Write-Host "  ❌ $_" -ForegroundColor Red }
    
    Write-Host ""
    Write-Host "🔧 Solutions:" -ForegroundColor Yellow
    Write-Host "1. Download a fresh portable build from the latest release" -ForegroundColor White
    Write-Host "2. If you have MSYS2 installed, copy the missing DLLs from:" -ForegroundColor White
    Write-Host "   C:\msys64\mingw64\bin\" -ForegroundColor Gray
    Write-Host "3. Report this issue at: https://github.com/your-repo/issues" -ForegroundColor White
} else {
    Write-Host "🎉 All required DLLs appear to be present!" -ForegroundColor Green
    Write-Host "The portable build should work correctly." -ForegroundColor Green
}

# Check GStreamer plugins
$gstPluginDir = Join-Path (Split-Path $binDir -Parent) "lib\gstreamer-1.0"
if (Test-Path $gstPluginDir) {
    $pluginCount = (Get-ChildItem $gstPluginDir -Filter "*.dll").Count
    Write-Host "🎵 GStreamer plugins: $pluginCount found" -ForegroundColor Green
} else {
    Write-Host "⚠️ GStreamer plugin directory not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Press any key to continue..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")