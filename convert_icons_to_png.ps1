#!/usr/bin/env pwsh
# Convert SVG icons to PNG format for better Windows compatibility
# This script converts all SVG icons in src/assets/icons to 16x16 PNG with transparent background

param(
    [string]$SourceDir = "src/assets/icons",
    [string]$OutputDir = "src/assets/icons",
    [int]$Size = 16,
    [switch]$Force
)

Write-Host "🎨 Converting SVG icons to PNG format..." -ForegroundColor Blue
Write-Host "Source: $SourceDir" -ForegroundColor Gray
Write-Host "Output: $OutputDir" -ForegroundColor Gray
Write-Host "Size: ${Size}x${Size}px" -ForegroundColor Gray
Write-Host ""

# Check for conversion tools
$InkscapeCmd = $null
$MagickCmd = $null

# Try to find Inkscape
$InkscapePaths = @(
    "inkscape",
    "C:\Program Files\Inkscape\bin\inkscape.exe",
    "C:\Program Files (x86)\Inkscape\bin\inkscape.exe"
)

foreach ($Path in $InkscapePaths) {
    if (Get-Command $Path -ErrorAction SilentlyContinue) {
        $InkscapeCmd = $Path
        Write-Host "✅ Found Inkscape: $Path" -ForegroundColor Green
        break
    }
}

# Try to find ImageMagick
$MagickPaths = @(
    "magick",
    "convert",
    "C:\Program Files\ImageMagick-*\magick.exe"
)

foreach ($Path in $MagickPaths) {
    if (Get-Command $Path -ErrorAction SilentlyContinue) {
        $MagickCmd = $Path
        Write-Host "✅ Found ImageMagick: $Path" -ForegroundColor Green
        break
    }
}

if (-not $InkscapeCmd -and -not $MagickCmd) {
    Write-Host "❌ No conversion tools found!" -ForegroundColor Red
    Write-Host "Please install one of the following:" -ForegroundColor Yellow
    Write-Host "  • Inkscape: https://inkscape.org/release/" -ForegroundColor Yellow
    Write-Host "  • ImageMagick: https://imagemagick.org/script/download.php#windows" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "💡 Alternative: Use the embedded PNG data below in the CI workflow" -ForegroundColor Cyan
    exit 1
}

# Get all SVG files
$SvgFiles = Get-ChildItem -Path $SourceDir -Filter "*.svg"
Write-Host "📁 Found $($SvgFiles.Count) SVG files to convert" -ForegroundColor Cyan
Write-Host ""

$ConvertedCount = 0
$SkippedCount = 0
$ErrorCount = 0

foreach ($SvgFile in $SvgFiles) {
    $BaseName = [System.IO.Path]::GetFileNameWithoutExtension($SvgFile.Name)
    $PngPath = Join-Path $OutputDir "$BaseName.png"
    
    # Check if PNG already exists and is newer
    if ((Test-Path $PngPath) -and (-not $Force)) {
        $SvgTime = $SvgFile.LastWriteTime
        $PngTime = (Get-Item $PngPath).LastWriteTime
        if ($PngTime -gt $SvgTime) {
            Write-Host "⏭️ Skipping $($SvgFile.Name) (PNG is newer)" -ForegroundColor Gray
            $SkippedCount++
            continue
        }
    }
    
    Write-Host "🔄 Converting $($SvgFile.Name)..." -ForegroundColor Yellow
    
    $Success = $false
    
    # Try Inkscape first (better SVG support)
    if ($InkscapeCmd -and -not $Success) {
        try {
            & $InkscapeCmd --export-type=png --export-width=$Size --export-height=$Size --export-filename="$PngPath" "$($SvgFile.FullName)" 2>$null
            if ($LASTEXITCODE -eq 0 -and (Test-Path $PngPath)) {
                $Success = $true
                Write-Host "  ✅ Converted with Inkscape" -ForegroundColor Green
            }
        } catch {
            Write-Host "  ⚠️ Inkscape failed: $_" -ForegroundColor Yellow
        }
    }
    
    # Try ImageMagick as fallback
    if ($MagickCmd -and -not $Success) {
        try {
            & $MagickCmd "$($SvgFile.FullName)" -background transparent -size "${Size}x${Size}" "$PngPath" 2>$null
            if ($LASTEXITCODE -eq 0 -and (Test-Path $PngPath)) {
                $Success = $true
                Write-Host "  ✅ Converted with ImageMagick" -ForegroundColor Green
            }
        } catch {
            Write-Host "  ⚠️ ImageMagick failed: $_" -ForegroundColor Yellow
        }
    }
    
    if ($Success) {
        $ConvertedCount++
        # Verify the PNG file
        $PngSize = (Get-Item $PngPath).Length
        Write-Host "  📏 Size: $([math]::Round($PngSize / 1KB, 1)) KB" -ForegroundColor Gray
    } else {
        $ErrorCount++
        Write-Host "  ❌ Conversion failed!" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "📊 Conversion Summary:" -ForegroundColor Blue
Write-Host "  ✅ Converted: $ConvertedCount" -ForegroundColor Green
Write-Host "  ⏭️ Skipped: $SkippedCount" -ForegroundColor Gray
Write-Host "  ❌ Errors: $ErrorCount" -ForegroundColor Red

if ($ConvertedCount -gt 0) {
    Write-Host ""
    Write-Host "🎉 Conversion complete! Update the GResource file to use .png extensions." -ForegroundColor Green
}