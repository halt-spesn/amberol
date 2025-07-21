// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows-specific functionality for Amberol

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::System::Power::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::Storage::FileSystem::*,
    Win32::UI::Shell::*,
    Win32::Media::Audio::*,
};

use log::debug;

/// Windows-specific power management
#[cfg(target_os = "windows")]
pub struct WindowsPowerManager {
    execution_state: EXECUTION_STATE,
}

#[cfg(target_os = "windows")]
impl WindowsPowerManager {
    pub fn new() -> Self {
        Self {
            execution_state: ES_CONTINUOUS,
        }
    }

    /// Prevent system sleep when playing music
    pub fn prevent_sleep(&mut self) {
        unsafe {
            self.execution_state = SetThreadExecutionState(
                ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED
            );
            debug!("Windows: Preventing system sleep for music playback");
        }
    }

    /// Allow system sleep when not playing music
    pub fn allow_sleep(&mut self) {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
            debug!("Windows: Allowing system sleep");
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for WindowsPowerManager {
    fn drop(&mut self) {
        self.allow_sleep();
    }
}

/// Get the Windows music folder path
#[cfg(target_os = "windows")]
pub fn get_music_folder() -> Option<std::path::PathBuf> {
    unsafe {
        let mut path = [0u16; 260]; // MAX_PATH
        let result = SHGetFolderPathW(
            HWND(std::ptr::null_mut()),
            CSIDL_MYMUSIC as i32,
            HANDLE(std::ptr::null_mut()),
            SHGFP_TYPE_CURRENT.0,
            &mut path,
        );
        
        if result.is_ok() {
            let len = path.iter().position(|&x| x == 0).unwrap_or(path.len());
            let path_str = String::from_utf16_lossy(&path[..len]);
            Some(std::path::PathBuf::from(path_str))
        } else {
            None
        }
    }
}

/// Windows-specific file associations
#[cfg(target_os = "windows")]
pub fn register_file_associations() -> Result<(), Box<dyn std::error::Error>> {
    // This would register Amberol as a handler for music files
    // Implementation would involve Windows Registry operations
    debug!("Windows: File associations would be registered here");
    Ok(())
}

/// Get Windows-specific application data directory
#[cfg(target_os = "windows")]
pub fn get_app_data_dir() -> Option<std::path::PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|path| std::path::PathBuf::from(path).join("Amberol"))
}

/// Windows media session integration
#[cfg(target_os = "windows")]
pub struct WindowsMediaSession {
    // This would hold Windows Media Session objects
}

#[cfg(target_os = "windows")]
impl WindowsMediaSession {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update_metadata(&self, title: &str, artist: &str, album: &str) {
        debug!("Windows: Updating media session metadata - Title: {}, Artist: {}, Album: {}", title, artist, album);
        // Implementation would use Windows Runtime Media Session APIs
    }

    pub fn set_playback_state(&self, playing: bool) {
        debug!("Windows: Setting playback state to {}", if playing { "playing" } else { "paused" });
        // Implementation would update Windows Media Transport Controls
    }
}

// Non-Windows platforms - provide stub implementations
#[cfg(not(target_os = "windows"))]
pub struct WindowsPowerManager;

#[cfg(not(target_os = "windows"))]
impl WindowsPowerManager {
    pub fn new() -> Self { Self }
    pub fn prevent_sleep(&mut self) {}
    pub fn allow_sleep(&mut self) {}
}

#[cfg(not(target_os = "windows"))]
pub fn get_music_folder() -> Option<std::path::PathBuf> { None }

#[cfg(not(target_os = "windows"))]
pub fn register_file_associations() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }

#[cfg(not(target_os = "windows"))]
pub fn get_app_data_dir() -> Option<std::path::PathBuf> { None }

#[cfg(not(target_os = "windows"))]
pub struct WindowsMediaSession;

#[cfg(not(target_os = "windows"))]
impl WindowsMediaSession {
    pub fn new() -> Self { Self }
    pub fn update_metadata(&self, _title: &str, _artist: &str, _album: &str) {}
    pub fn set_playback_state(&self, _playing: bool) {}
}