# Amberol for Windows

![Application icon](./data/icons/hicolor/scalable/apps/io.bassi.Amberol.svg)

A small and simple sound and music player that is well integrated with modern Windows.

## System Requirements

- **Operating System**: Windows 10 or Windows 11 (64-bit)
- **RAM**: At least 512 MB (1 GB recommended)
- **Storage**: 100 MB free disk space
- **Audio**: Windows-compatible audio device

## Installation

### Option 1: Windows Installer (Recommended)

1. Download the latest `amberol-windows-installer.exe` from the releases page
2. Run the installer as Administrator
3. Follow the installation wizard
4. Choose whether to create desktop shortcuts and file associations
5. Launch Amberol from the Start Menu or desktop shortcut

### Option 2: Portable Version

1. Download the `amberol-windows-portable.zip` archive
2. Extract to your preferred location (e.g., `C:\Apps\Amberol`)
3. Run `amberol.bat` to start the application

### Option 3: Build from Source

#### Prerequisites

Install MSYS2 from https://www.msys2.org/

#### Build Steps

1. Open MSYS2 terminal and update the system:
   ```bash
   pacman -Syu
   ```

2. Install build dependencies:
   ```bash
   pacman -S --needed \
     mingw-w64-x86_64-gtk4 \
     mingw-w64-x86_64-libadwaita \
     mingw-w64-x86_64-gstreamer \
     mingw-w64-x86_64-gst-plugins-base \
     mingw-w64-x86_64-gst-plugins-good \
     mingw-w64-x86_64-gst-plugins-bad \
     mingw-w64-x86_64-gst-plugins-ugly \
     mingw-w64-x86_64-rust \
     mingw-w64-x86_64-meson \
     mingw-w64-x86_64-ninja \
     mingw-w64-x86_64-pkg-config
   ```

3. Clone the repository and build:
   ```bash
   git clone https://gitlab.gnome.org/World/amberol.git
   cd amberol
   meson setup _build --buildtype=release
   meson compile -C _build
   ```

4. Or use the PowerShell build script (from Windows PowerShell):
   ```powershell
   .\build_windows.ps1 -Profile release
   ```

## Supported Audio Formats

Amberol for Windows supports the following audio formats through GStreamer:

- **MP3** - MPEG Audio Layer 3
- **MP4/M4A** - MPEG-4 Audio
- **AAC** - Advanced Audio Coding
- **FLAC** - Free Lossless Audio Codec
- **OGG/Vorbis** - Ogg Vorbis
- **WAV** - Waveform Audio File Format
- **WMA** - Windows Media Audio (via DirectShow)

## Features

### Core Features
- **Simple Interface**: Clean, uncluttered design following modern Windows UI principles
- **Drag & Drop**: Simply drag music files or folders onto the window to play them
- **Queue Management**: Add songs, shuffle, repeat, and clear your playlist
- **Search**: Quick search through your current playlist
- **Waveform Visualization**: See the audio waveform of the current track
- **Album Art**: Displays embedded album artwork

### Windows-Specific Features
- **Windows Media Integration**: Control playback from keyboard media keys
- **File Associations**: Associate audio files with Amberol (optional during installation)
- **Windows Audio**: Full support for Windows audio subsystem
- **System Tray**: Minimize to system tray for background playback
  - Click X button to minimize to tray (doesn't close the app)
  - Click tray icon to restore window
  - Music continues playing while minimized
- **Windows 11 Integration**: Modern context menus and title bar
- **Built-in Debugging**: Comprehensive diagnostics for troubleshooting
  - Icon loading verification and resource enumeration
  - GResource file detection and content analysis
  - Environment variable checking and recommendations

## Usage

### Basic Operations

1. **Adding Music**: 
   - Drag and drop music files or folders onto the window
   - Use Ctrl+O to open files
   - Use Ctrl+Shift+O to add folders

2. **Playback Controls**:
   - Spacebar: Play/Pause
   - Ctrl+Right: Next track
   - Ctrl+Left: Previous track
   - Ctrl+R: Shuffle
   - Ctrl+L: Clear queue

3. **Search**: Press Ctrl+F to search your current playlist

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Play/Pause |
| `Ctrl+Right` | Next track |
| `Ctrl+Left` | Previous track |
| `Ctrl+R` | Shuffle playlist |
| `Ctrl+L` | Clear playlist |
| `Ctrl+F` | Search playlist |
| `Ctrl+O` | Open files |
| `Ctrl+Shift+O` | Add folder |
| `Ctrl+Q` | Quit application |

## Configuration

### Settings Location
Amberol stores its configuration in:
```
%LOCALAPPDATA%\io.bassi.Amberol\
```

### Audio Settings
You can configure audio settings by setting environment variables or using the Windows audio settings:

- Use `GSK_RENDERER=gl` for better graphics performance
- Configure audio device through Windows Sound settings
- Adjust volume through the application or Windows volume mixer

## Troubleshooting

### Common Issues

#### Audio Not Playing
1. Check Windows audio settings
2. Ensure audio drivers are up to date
3. Try different audio formats
4. Restart the application

#### MP3 Files Not Loading ("invalid frame" errors)
1. **Metadata Issues**: Files with "invalid frame" errors have corrupted ID3 tags/metadata, but should still appear in the playlist
2. **Songs Still Playable**: Files with metadata errors will show filename as title but should still play
3. **Clean Metadata**: Use tools like Mp3tag or FFmpeg to fix/rebuild metadata:
   ```
   ffmpeg -i problematic.mp3 -c copy -map_metadata -1 fixed.mp3
   ```
4. **Enable Debug**: Uncomment `GST_DEBUG=3` in `amberol.bat` to see detailed GStreamer logs
5. **Check GStreamer Plugins**: Ensure MP3 codecs are present in `lib\gstreamer-1.0\`

#### Missing Icons or Theme Issues
1. **GResource Check**: Icons are embedded in `amberol.gresource` - verify this file exists:
   - `bin\amberol.gresource` (primary location)
   - `share\amberol.gresource` (fallback location)  
   - `share\amberol\amberol.gresource` (alternative location)
2. **Theme Resources**: Check if `share\libadwaita-1\` and `share\icons\` directories exist
3. **Keep Original Theme**: The launcher preserves Amberol's original look while providing missing icons
4. **Comprehensive Icon Fix**: The portable build includes all missing symbolic icons:
   - Media controls (play, pause, skip, shuffle, repeat)
   - Playlist controls (queue, selection, remove)  
   - Application icons (search, menu, volume)
   - Amberol app icon for "About" dialog
5. **Build Issues**: If GResource is missing, the build may have failed icon compilation
6. **Manual Fix**: Download a fresh portable build if icons still missing

#### Application Won't Start
1. **Missing DLL Check**: Run the included `check_missing_dlls.ps1` script to identify missing dependencies:
   ```powershell
   # From the portable build directory
   .\check_missing_dlls.ps1
   ```
2. Ensure you have the Microsoft Visual C++ 2019+ Redistributable installed
3. Check Windows Event Viewer for error details
4. Try running as Administrator
5. Reinstall the application

#### Missing DLL Errors (libffi-8.dll, liborc-0.4-0.dll, etc.)
If you get specific DLL missing errors:
1. Use the DLL checker tool: `.\check_missing_dlls.ps1`
2. Download a fresh portable build from the latest release
3. If using an older build, manually copy missing DLLs from MSYS2:
   ```
   C:\msys64\mingw64\bin\[missing-dll-name]
   ```

#### Poor Performance
1. Update graphics drivers
2. Set `GSK_RENDERER=gl` environment variable
3. Close other resource-intensive applications
4. Increase virtual memory if needed

#### File Association Problems
1. Run the installer as Administrator
2. Manually associate files through Windows Settings > Apps > Default apps
3. Use "Open with" from the context menu

### Debug Information

To get debug information for troubleshooting:

1. Open Command Prompt or PowerShell
2. Navigate to the Amberol installation directory
3. Run with debug flags:
   ```cmd
   set RUST_LOG=amberol=debug
   amberol.exe
   ```

## Uninstallation

### If Installed via Installer
1. Go to Windows Settings > Apps
2. Find "Amberol" in the list
3. Click "Uninstall"

### If Using Portable Version
Simply delete the folder containing Amberol files.

## Privacy and Data

Amberol is designed with privacy in mind:
- No data collection or telemetry
- No internet connection required for basic functionality
- Music files are played locally only
- No account registration needed

## Support

### Getting Help
- **Issues**: Report bugs on [GitLab Issues](https://gitlab.gnome.org/World/amberol/-/issues)
- **Discussions**: Join the conversation on [GNOME Discourse](https://discourse.gnome.org/c/applications/7)
- **Matrix**: Join `#amberol:gnome.org` for chat support

### Contributing
Amberol is open source! Contributions are welcome:
- Report bugs and suggest features
- Submit pull requests
- Help with translations
- Improve documentation

## License

Amberol is released under the GNU General Public License, version 3.0 or later.
See the [LICENSE.txt](LICENSE.txt) file for details.

## Credits

- **Developer**: Emmanuele Bassi
- **Windows Port**: Community contribution
- **Built with**: GTK4, libadwaita, GStreamer, Rust
- **Special Thanks**: GNOME Project, MSYS2 Project, GStreamer Project