#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

import os
import sys
import subprocess
import shutil
import argparse

def main():
    parser = argparse.ArgumentParser(description='Build Amberol with Cargo')
    parser.add_argument('--cargo-home', required=True, help='Cargo home directory')
    parser.add_argument('--manifest-path', required=True, help='Path to Cargo.toml')
    parser.add_argument('--target-dir', required=True, help='Target directory for build')
    parser.add_argument('--profile', default='release', help='Build profile (release or debug)')
    parser.add_argument('--output', required=True, help='Output executable path')
    parser.add_argument('--project-name', required=True, help='Project name')
    
    args = parser.parse_args()
    
    # Set up environment
    env = os.environ.copy()
    env['CARGO_HOME'] = args.cargo_home
    
    # Build cargo command
    cargo_cmd = [
        'cargo', 'build',
        '--manifest-path', args.manifest_path,
        '--target-dir', args.target_dir
    ]
    
    if args.profile == 'release':
        cargo_cmd.append('--release')
    
    # Run cargo build
    print(f"Running: {' '.join(cargo_cmd)}")
    result = subprocess.run(cargo_cmd, env=env)
    
    if result.returncode != 0:
        print(f"Cargo build failed with exit code {result.returncode}")
        sys.exit(result.returncode)
    
    # Determine source executable path
    exe_suffix = '.exe' if os.name == 'nt' else ''
    src_exe = os.path.join(args.target_dir, args.profile, args.project_name + exe_suffix)
    
    # Copy to output location
    print(f"Copying {src_exe} to {args.output}")
    try:
        shutil.copy2(src_exe, args.output)
        # Make sure it's executable on Unix systems
        if os.name != 'nt':
            os.chmod(args.output, 0o755)
    except Exception as e:
        print(f"Failed to copy executable: {e}")
        sys.exit(1)
    
    print("Build completed successfully!")

if __name__ == '__main__':
    main()