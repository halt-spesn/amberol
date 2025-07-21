#!/bin/bash
# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

# Amberol True Static Build Script
# This script attempts multiple strategies to achieve true static linking

set -e

echo "🚀 Building Amberol with True Static Linking"

# Function to check if static libraries exist
check_static_libs() {
    echo "🔍 Checking for static libraries..."
    
    local libs=("libgtk-4.a" "libadwaita-1.a" "libgstreamer-1.0.a" "libgstaudio-1.0.a" "libgstplayer-1.0.a" "libgraphene-1.0.a")
    local missing=()
    
    for lib in "${libs[@]}"; do
        if ! find /mingw64/lib -name "$lib" | grep -q .; then
            missing+=("$lib")
        else
            echo "  ✅ Found: $lib"
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "  ❌ Missing static libraries:"
        printf '    %s\n' "${missing[@]}"
        echo "  🔧 Attempting to install -dev packages..."
        
        # Try to install development packages that might contain static libraries
        pacman -S --noconfirm --needed \
            mingw-w64-x86_64-gtk4-devel \
            mingw-w64-x86_64-libadwaita-devel \
            mingw-w64-x86_64-gstreamer-devel \
            mingw-w64-x86_64-gst-plugins-base-devel 2>/dev/null || true
    fi
}

# Strategy 1: Build from source with static linking
build_from_source() {
    echo "🏗️ Strategy 1: Building dependencies from source..."
    
    # This would involve downloading and building GTK4, libadwaita, etc. from source
    # For now, we'll try with available packages
    echo "  ⚠️ Source build not implemented yet, using package strategy"
}

# Strategy 2: Force static linking with available libraries
force_static_linking() {
    echo "🔧 Strategy 2: Aggressive static linking configuration..."
    
    # Set up comprehensive static environment
    export PKG_CONFIG_ALL_STATIC=1
    export PKG_CONFIG_ALLOW_CROSS=1
    export GSTREAMER_1_0_STATIC=1
    export GTK4_STATIC=1
    export LIBADWAITA_STATIC=1
    export GRAPHENE_STATIC=1
    
    # More aggressive Rust flags
    export RUSTFLAGS="-C target-feature=+crt-static \
        -C link-arg=-static \
        -C link-arg=-static-libgcc \
        -C link-arg=-static-libstdc++ \
        -C link-arg=-Wl,--allow-multiple-definition \
        -C link-arg=-Wl,--whole-archive \
        -L /mingw64/lib \
        -C link-arg=-Wl,--no-whole-archive"
    
    # Set up linker environment
    export LDFLAGS="-static -static-libgcc -static-libstdc++ -L/mingw64/lib"
    export CFLAGS="-static"
    export CXXFLAGS="-static"
    
    # Configure pkg-config for static mode
    export PKG_CONFIG_PATH="/mingw64/lib/pkgconfig:/mingw64/share/pkgconfig"
    export PKG_CONFIG_LIBDIR="/mingw64/lib/pkgconfig:/mingw64/share/pkgconfig"
    
    echo "  📋 Environment configured:"
    echo "    RUSTFLAGS: $RUSTFLAGS"
    echo "    LDFLAGS: $LDFLAGS"
    
    # Test pkg-config
    echo "  🧪 Testing pkg-config static mode..."
    pkg-config --exists --print-errors gtk4 && echo "    ✅ gtk4 found" || echo "    ❌ gtk4 not found"
    pkg-config --exists --print-errors libadwaita-1 && echo "    ✅ libadwaita found" || echo "    ❌ libadwaita not found"
    pkg-config --exists --print-errors gstreamer-1.0 && echo "    ✅ gstreamer found" || echo "    ❌ gstreamer not found"
    
    # Configure meson with aggressive static options
    echo "  🔧 Configuring meson..."
    meson setup _build_static \
        --buildtype=release \
        --default-library=static \
        --prefer-static \
        --strip \
        -Doptimization=3 \
        -Db_lto=true \
        -Db_staticpic=false
    
    echo "  🏗️ Building..."
    meson compile -C _build_static -v
}

# Strategy 3: Manual library collection and linking
manual_static_link() {
    echo "🔧 Strategy 3: Manual static library collection..."
    
    # Find all required static libraries
    local static_libs=""
    local lib_dirs="/mingw64/lib"
    
    # Core libraries that must be static
    local required_libs=(
        "gtk-4" "adwaita-1" "gstreamer-1.0" "gstaudio-1.0" "gstplayer-1.0"
        "graphene-1.0" "glib-2.0" "gobject-2.0" "gio-2.0"
        "cairo" "pango-1.0" "pangocairo-1.0" "harfbuzz"
    )
    
    echo "  🔍 Collecting static libraries..."
    for lib in "${required_libs[@]}"; do
        local static_lib="$lib_dirs/lib${lib}.a"
        if [ -f "$static_lib" ]; then
            static_libs="$static_libs $static_lib"
            echo "    ✅ $static_lib"
        else
            echo "    ❌ Missing: $static_lib"
        fi
    done
    
    if [ -n "$static_libs" ]; then
        echo "  🔗 Building with manual library linking..."
        export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static $static_libs"
        meson setup _build_manual --buildtype=release
        meson compile -C _build_manual -v
    else
        echo "  ❌ No static libraries found for manual linking"
        return 1
    fi
}

# Main execution
main() {
    echo "🧪 Checking static library availability..."
    check_static_libs
    
    echo "🎯 Attempting static build strategies..."
    
    # Try strategy 2 first (most likely to work)
    if force_static_linking; then
        echo "✅ Strategy 2 (aggressive static linking) succeeded!"
        return 0
    fi
    
    echo "⚠️ Strategy 2 failed, trying strategy 3..."
    if manual_static_link; then
        echo "✅ Strategy 3 (manual linking) succeeded!"
        return 0
    fi
    
    echo "❌ All static linking strategies failed!"
    echo "📋 Falling back to regular build..."
    
    # Fallback to regular build
    meson setup _build_fallback --buildtype=release
    meson compile -C _build_fallback
    
    echo "⚠️ Built with dynamic linking - DLLs will be required"
    return 1
}

# Run main function
main "$@"