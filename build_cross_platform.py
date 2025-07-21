#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

"""
Cross-platform build script for Amberol
Supports building for Linux (native), Windows (via MSYS2), and creating distributions
"""

import argparse
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

def run_command(cmd, shell=False, check=True, **kwargs):
    """Run a command and handle errors"""
    print(f"Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    try:
        result = subprocess.run(cmd, shell=shell, check=check, **kwargs)
        return result
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e}")
        sys.exit(1)

def detect_platform():
    """Detect the current platform"""
    system = platform.system().lower()
    if system == "windows":
        return "windows"
    elif system == "linux":
        return "linux"
    elif system == "freebsd":
        return "freebsd"
    else:
        return "unknown"

def check_dependencies_linux():
    """Check if Linux dependencies are available"""
    deps = [
        ("meson", "meson"),
        ("ninja", "ninja-build"),
        ("cargo", "rust")
    ]
    
    missing = []
    for cmd, package in deps:
        if not shutil.which(cmd):
            missing.append((cmd, package))
    
    if missing:
        print("Missing dependencies:")
        for cmd, package in missing:
            print(f"  {cmd} (install {package})")
        print("\nInstall missing dependencies and try again.")
        return False
    return True

def check_dependencies_windows():
    """Check if Windows/MSYS2 dependencies are available"""
    msys2_path = Path("C:/msys64")
    if not msys2_path.exists():
        print("MSYS2 not found at C:/msys64")
        print("Please install MSYS2 from https://www.msys2.org/")
        return False
    return True

def build_linux(args):
    """Build for Linux using meson"""
    print("Building for Linux...")
    
    if not check_dependencies_linux():
        return False
    
    build_dir = Path("_build_linux")
    
    if args.clean and build_dir.exists():
        print("Cleaning previous build...")
        shutil.rmtree(build_dir)
    
    # Configure meson
    meson_args = [
        "meson", "setup", str(build_dir),
        f"--buildtype={args.buildtype}",
        f"--prefix={args.prefix}"
    ]
    
    if args.profile == "development":
        meson_args.append("-Dprofile=development")
    
    run_command(meson_args)
    
    # Build
    run_command(["meson", "compile", "-C", str(build_dir)])
    
    if args.install:
        run_command(["sudo", "meson", "install", "-C", str(build_dir)])
    
    print("Linux build completed successfully!")
    return True

def build_windows(args):
    """Build for Windows using MSYS2"""
    print("Building for Windows...")
    
    if not check_dependencies_windows():
        return False
    
    # Use PowerShell script for Windows build
    ps_script = Path("build_windows.ps1")
    if not ps_script.exists():
        print("build_windows.ps1 not found!")
        return False
    
    ps_args = [
        "powershell", "-ExecutionPolicy", "Bypass", "-File", str(ps_script),
        "-Profile", args.buildtype
    ]
    
    if args.clean:
        ps_args.append("-Clean")
    
    if args.install:
        ps_args.append("-Install")
    
    run_command(ps_args, shell=True)
    
    print("Windows build completed successfully!")
    return True

def create_flatpak(args):
    """Create Flatpak package for Linux"""
    print("Creating Flatpak package...")
    
    manifest = Path("io.bassi.Amberol.json")
    if not manifest.exists():
        print("Flatpak manifest not found!")
        return False
    
    flatpak_dir = Path("_flatpak_build")
    
    if args.clean and flatpak_dir.exists():
        shutil.rmtree(flatpak_dir)
    
    run_command([
        "flatpak-builder", "--force-clean", "--user", "--install-deps-from=flathub",
        str(flatpak_dir), str(manifest)
    ])
    
    print("Flatpak package created successfully!")
    return True

def package_windows(args):
    """Create Windows distribution packages"""
    print("Packaging Windows distribution...")
    
    portable_dir = Path("amberol-windows-portable")
    if not portable_dir.exists():
        print("Windows portable build not found!")
        print("Run build with Windows target first.")
        return False
    
    # Create ZIP archive
    import zipfile
    
    zip_path = Path("dist/amberol-windows-portable.zip")
    zip_path.parent.mkdir(exist_ok=True)
    
    print("Creating portable ZIP archive...")
    with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(portable_dir):
            for file in files:
                file_path = Path(root) / file
                arc_path = file_path.relative_to(portable_dir.parent)
                zipf.write(file_path, arc_path)
    
    # Create installer if Inno Setup is available
    inno_setup = shutil.which("iscc")
    if inno_setup:
        print("Creating Windows installer...")
        iss_file = Path("amberol-installer.iss")
        if iss_file.exists():
            run_command([inno_setup, str(iss_file)])
        else:
            print("Installer script not found, skipping installer creation")
    else:
        print("Inno Setup not found, skipping installer creation")
    
    print("Windows packaging completed!")
    return True

def main():
    parser = argparse.ArgumentParser(description="Cross-platform build script for Amberol")
    parser.add_argument("target", choices=["linux", "windows", "flatpak", "package-windows", "all"],
                       help="Build target")
    parser.add_argument("--buildtype", choices=["release", "debug"], default="release",
                       help="Build type")
    parser.add_argument("--profile", choices=["default", "development"], default="default",
                       help="Build profile")
    parser.add_argument("--prefix", default="/usr/local",
                       help="Installation prefix")
    parser.add_argument("--clean", action="store_true",
                       help="Clean previous build")
    parser.add_argument("--install", action="store_true",
                       help="Install after building")
    
    args = parser.parse_args()
    
    current_platform = detect_platform()
    print(f"Current platform: {current_platform}")
    print(f"Build target: {args.target}")
    
    success = True
    
    if args.target == "linux" or args.target == "all":
        success &= build_linux(args)
    
    if args.target == "windows" or args.target == "all":
        if current_platform == "windows":
            success &= build_windows(args)
        else:
            print("Windows builds can only be created on Windows with MSYS2")
            success = False
    
    if args.target == "flatpak":
        if current_platform == "linux":
            success &= create_flatpak(args)
        else:
            print("Flatpak packages can only be created on Linux")
            success = False
    
    if args.target == "package-windows":
        success &= package_windows(args)
    
    if success:
        print("\nBuild process completed successfully!")
    else:
        print("\nBuild process failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()