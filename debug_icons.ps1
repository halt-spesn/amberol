#!/usr/bin/env pwsh
# Amberol Icon Debugging Script
# This script helps diagnose why icons are not showing in Amberol

Write-Host "🔍 Amberol Icon Debugging Script" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

$amberolDir = $PSScriptRoot
Write-Host "📁 Amberol Directory: $amberolDir" -ForegroundColor Blue

# Check for GResource files
Write-Host ""
Write-Host "🎯 Checking for GResource files..." -ForegroundColor Yellow
$gresourceLocations = @(
    "$amberolDir\bin\amberol.gresource",
    "$amberolDir\share\amberol.gresource", 
    "$amberolDir\share\amberol\amberol.gresource"
)

$foundGResource = $false
foreach ($location in $gresourceLocations) {
    if (Test-Path $location) {
        Write-Host "  ✅ Found: $location" -ForegroundColor Green
        $foundGResource = $true
        
        # Try to analyze GResource contents if gresource.exe is available
        $gresourceExe = "$amberolDir\bin\gresource.exe"
        if (Test-Path $gresourceExe) {
            Write-Host "     📊 Analyzing contents..." -ForegroundColor Gray
            try {
                $contents = & $gresourceExe list $location 2>$null
                $iconResources = $contents | Where-Object { $_ -like "*icon*" -or $_ -like "*svg*" }
                Write-Host "     📈 Found $($iconResources.Count) icon resources" -ForegroundColor Cyan
                
                # Check for specific icons that should be there
                $requiredIcons = @(
                    "/io/bassi/Amberol/icons/scalable/actions/media-playlist-consecutive-symbolic.svg",
                    "/io/bassi/Amberol/icons/scalable/actions/media-playlist-repeat-symbolic.svg",
                    "/io/bassi/Amberol/icons/scalable/actions/media-playback-start-symbolic.svg",
                    "/io/bassi/Amberol/icons/scalable/actions/media-playback-pause-symbolic.svg"
                )
                
                Write-Host "     🎯 Checking for required icons:" -ForegroundColor Gray
                foreach ($icon in $requiredIcons) {
                    if ($contents -contains $icon) {
                        Write-Host "       ✅ $icon" -ForegroundColor Green
                    } else {
                        Write-Host "       ❌ $icon (MISSING)" -ForegroundColor Red
                    }
                }
                
                Write-Host "     📋 All icon resources:" -ForegroundColor Gray
                foreach ($icon in $iconResources) {
                    Write-Host "       • $icon" -ForegroundColor DarkGray
                }
            } catch {
                Write-Host "     ⚠️ Could not analyze GResource contents" -ForegroundColor Yellow
            }
        } else {
            Write-Host "     ⚠️ gresource.exe not found, cannot analyze contents" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ❌ Missing: $location" -ForegroundColor Red
    }
}

if (-not $foundGResource) {
    Write-Host ""
    Write-Host "🚨 CRITICAL: No GResource files found!" -ForegroundColor Red
    Write-Host "   This explains why icons are missing." -ForegroundColor Red
    Write-Host "   The GResource file contains all embedded icons." -ForegroundColor Red
}

# Check for fallback icon theme directories
Write-Host ""
Write-Host "🎨 Checking fallback icon themes..." -ForegroundColor Yellow
$iconThemePaths = @(
    "$amberolDir\share\icons\Adwaita\scalable\actions",
    "$amberolDir\share\icons\hicolor\scalable\actions"
)

foreach ($path in $iconThemePaths) {
    if (Test-Path $path) {
        $iconFiles = Get-ChildItem $path -Filter "*symbolic.svg" -ErrorAction SilentlyContinue
        Write-Host "  ✅ Found: $path ($($iconFiles.Count) icons)" -ForegroundColor Green
        foreach ($icon in $iconFiles) {
            Write-Host "    • $($icon.Name)" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "  ❌ Missing: $path" -ForegroundColor Red
    }
}

# Check GTK environment variables
Write-Host ""
Write-Host "🌍 Checking GTK environment..." -ForegroundColor Yellow
$gtkVars = @(
    "GTK_DATA_PREFIX",
    "GTK_EXE_PREFIX", 
    "XDG_DATA_DIRS",
    "GSK_RENDERER"
)

foreach ($var in $gtkVars) {
    $value = [Environment]::GetEnvironmentVariable($var)
    if ($value) {
        Write-Host "  ✅ $var = $value" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $var = (not set)" -ForegroundColor Red
    }
}

# Check for libadwaita theme resources
Write-Host ""
Write-Host "🎭 Checking libadwaita theme resources..." -ForegroundColor Yellow
$libadwaitaPath = "$amberolDir\share\libadwaita-1"
if (Test-Path $libadwaitaPath) {
    Write-Host "  ✅ Found: $libadwaitaPath" -ForegroundColor Green
    
    $stylesPath = "$libadwaitaPath\styles"
    if (Test-Path $stylesPath) {
        Write-Host "  ✅ Found: $stylesPath" -ForegroundColor Green
    } else {
        Write-Host "  ❌ Missing: $stylesPath" -ForegroundColor Red
    }
} else {
    Write-Host "  ❌ Missing: $libadwaitaPath" -ForegroundColor Red
}

# Summary and recommendations
Write-Host ""
Write-Host "📋 SUMMARY & RECOMMENDATIONS" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan

if (-not $foundGResource) {
    Write-Host "🚨 PRIMARY ISSUE: Missing GResource file" -ForegroundColor Red
    Write-Host "   → Download a fresh CI build from GitHub Actions" -ForegroundColor Yellow
    Write-Host "   → The GResource contains all embedded icons" -ForegroundColor Yellow
} else {
    Write-Host "✅ GResource file found - icons should be embedded" -ForegroundColor Green
    Write-Host "🔍 If icons still missing, possible causes:" -ForegroundColor Yellow
    Write-Host "   → Icon name mismatches in GResource aliases" -ForegroundColor Gray
    Write-Host "   → GTK not finding icons in the resource path" -ForegroundColor Gray
    Write-Host "   → Theme/rendering issues" -ForegroundColor Gray
}

Write-Host ""
Write-Host "🛠️ IMMEDIATE FIXES TO TRY:" -ForegroundColor Blue
Write-Host "1. Set environment variables manually:" -ForegroundColor White
Write-Host "   set GTK_DATA_PREFIX=$amberolDir" -ForegroundColor Gray
Write-Host "   set XDG_DATA_DIRS=$amberolDir\share" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Try different GTK renderer:" -ForegroundColor White
Write-Host "   set GSK_RENDERER=cairo" -ForegroundColor Gray
Write-Host "   (instead of gl)" -ForegroundColor Gray
Write-Host ""
Write-Host "3. Enable GTK debug:" -ForegroundColor White
Write-Host "   set GTK_DEBUG=icon-theme" -ForegroundColor Gray
Write-Host "   (to see icon loading messages)" -ForegroundColor Gray

Write-Host ""
Write-Host "Press any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")