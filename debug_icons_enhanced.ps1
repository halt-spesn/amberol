#!/usr/bin/env pwsh
# Enhanced Amberol Icon Debugging Script
# This script provides detailed analysis of icon loading and SVG rendering

Write-Host "🔍 Enhanced Amberol Icon Debugging" -ForegroundColor Blue
Write-Host "=================================" -ForegroundColor Blue
Write-Host ""

$AmberolDir = $PSScriptRoot
Write-Host "📁 Amberol Directory: $AmberolDir" -ForegroundColor Gray

# Check for GResource files
Write-Host "🔍 Checking GResource files..." -ForegroundColor Blue
$GResourceFiles = @()
$SearchPaths = @("bin\", "share\", "share\amberol\")

foreach ($SearchPath in $SearchPaths) {
    $FullPath = Join-Path $AmberolDir $SearchPath
    if (Test-Path $FullPath) {
        $GResourceFile = Join-Path $FullPath "amberol.gresource"
        if (Test-Path $GResourceFile) {
            $GResourceFiles += $GResourceFile
            $Size = [math]::Round((Get-Item $GResourceFile).Length / 1KB, 2)
            Write-Host "  ✅ Found: $GResourceFile ($Size KB)" -ForegroundColor Green
        }
    }
}

if ($GResourceFiles.Count -eq 0) {
    Write-Host "  ❌ No GResource files found!" -ForegroundColor Red
    Write-Host "     This is likely the cause of missing icons." -ForegroundColor Red
    Write-Host ""
    return
}

# Analyze GResource contents
Write-Host ""
Write-Host "📋 Analyzing GResource contents..." -ForegroundColor Blue

$GResourceTool = Join-Path $AmberolDir "bin\gresource.exe"
if (-not (Test-Path $GResourceTool)) {
    Write-Host "  ⚠️ gresource.exe not found - trying system PATH" -ForegroundColor Yellow
    $GResourceTool = "gresource"
}

$MainGResource = $GResourceFiles[0]
try {
    $Resources = & $GResourceTool list $MainGResource 2>$null
    $IconResources = $Resources | Where-Object { $_ -like "*icons*" -or $_ -like "*svg*" }
    
    Write-Host "  📊 Total resources: $($Resources.Count)" -ForegroundColor Cyan
    Write-Host "  🎨 Icon resources: $($IconResources.Count)" -ForegroundColor Cyan
    
    # Check specific problematic icons
    $ProblematicIcons = @(
        "/io/bassi/Amberol/icons/scalable/actions/media-playlist-consecutive-symbolic.svg",
        "/io/bassi/Amberol/icons/scalable/actions/media-playlist-repeat-symbolic.svg",
        "/io/bassi/Amberol/icons/scalable/actions/media-playlist-shuffle-symbolic.svg"
    )
    
    Write-Host ""
    Write-Host "🎯 Checking specific repeat mode icons:" -ForegroundColor Blue
    foreach ($IconPath in $ProblematicIcons) {
        $IconName = Split-Path $IconPath -Leaf
        if ($Resources -contains $IconPath) {
            Write-Host "  ✅ $IconName - Found in GResource" -ForegroundColor Green
            
            # Try to extract and analyze the icon
            try {
                $TempFile = Join-Path $env:TEMP "$IconName.tmp"
                & $GResourceTool extract $MainGResource $IconPath > $TempFile 2>$null
                if (Test-Path $TempFile) {
                    $Content = Get-Content $TempFile -Raw
                    $Size = $Content.Length
                    Write-Host "    📏 Size: $Size bytes" -ForegroundColor Gray
                    
                    # Check for common SVG issues
                    if ($Content -match 'fill="#[0-9a-fA-F]{6}"') {
                        Write-Host "    🎨 Has explicit fill color" -ForegroundColor Gray
                    } else {
                        Write-Host "    ⚠️ No explicit fill color - may not show on all themes" -ForegroundColor Yellow
                    }
                    
                    if ($Content -match 'viewBox="0 0 16 16"') {
                        Write-Host "    📐 Standard 16x16 viewBox" -ForegroundColor Gray
                    } else {
                        Write-Host "    ⚠️ Non-standard viewBox" -ForegroundColor Yellow
                    }
                    
                    # Check path complexity
                    $PathMatches = [regex]::Matches($Content, '<path[^>]*d="([^"]*)"')
                    if ($PathMatches.Count -gt 0) {
                        $PathData = $PathMatches[0].Groups[1].Value
                        Write-Host "    🛣️ Path length: $($PathData.Length) characters" -ForegroundColor Gray
                        if ($PathData.Length -gt 500) {
                            Write-Host "    ℹ️ Complex path - should render fine" -ForegroundColor Gray
                        } else {
                            Write-Host "    ℹ️ Simple path" -ForegroundColor Gray
                        }
                    }
                    
                    Remove-Item $TempFile -ErrorAction SilentlyContinue
                }
            } catch {
                Write-Host "    ❌ Failed to extract icon: $_" -ForegroundColor Red
            }
        } else {
            Write-Host "  ❌ $IconName - NOT found in GResource!" -ForegroundColor Red
        }
    }
    
} catch {
    Write-Host "  ❌ Failed to analyze GResource: $_" -ForegroundColor Red
}

# Check GTK environment variables
Write-Host ""
Write-Host "🌍 GTK Environment Variables:" -ForegroundColor Blue
$GTKVars = @(
    "GTK_THEME", "GTK_DATA_PREFIX", "GTK_EXE_PREFIX", 
    "GTK_DEBUG", "GSK_RENDERER", "XDG_DATA_DIRS"
)

foreach ($Var in $GTKVars) {
    $Value = [Environment]::GetEnvironmentVariable($Var)
    if ($Value) {
        Write-Host "  ✅ $Var = $Value" -ForegroundColor Green
    } else {
        Write-Host "  ⚪ $Var = (not set)" -ForegroundColor Gray
    }
}

# Icon theme debugging suggestions
Write-Host ""
Write-Host "🔧 Troubleshooting Suggestions:" -ForegroundColor Blue
Write-Host "  1. If icons show as 'missing image' placeholder:" -ForegroundColor Cyan
Write-Host "     • SVG parsing failure - icons exist but can't be rendered" -ForegroundColor Gray
Write-Host "     • Try: set GSK_RENDERER=cairo (software rendering)" -ForegroundColor Gray
Write-Host "     • Try: set GTK_THEME=Default (remove theme overrides)" -ForegroundColor Gray
Write-Host "  2. If no icons show at all:" -ForegroundColor Cyan  
Write-Host "     • GResource not loaded - check file paths above" -ForegroundColor Gray
Write-Host "     • Missing icon theme - check GTK_DATA_PREFIX" -ForegroundColor Gray
Write-Host "  3. If icons are completely missing:" -ForegroundColor Cyan
Write-Host "     • Icon names don't match - check aliases in GResource" -ForegroundColor Gray
Write-Host "     • Wrong icon theme path - verify XDG_DATA_DIRS" -ForegroundColor Gray
Write-Host "  4. For detailed debugging:" -ForegroundColor Cyan
Write-Host "     • Enable: set GTK_DEBUG=icon-theme" -ForegroundColor Gray
Write-Host "     • Enable: set G_MESSAGES_DEBUG=all" -ForegroundColor Gray
Write-Host "     • Check Amberol logs with: set RUST_LOG=amberol=info" -ForegroundColor Gray

Write-Host ""
Write-Host "🎵 Run this script whenever icons aren't displaying correctly!" -ForegroundColor Blue