// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows-specific functionality for Amberol

use log::debug;

/// Windows-specific file associations (stub - not yet implemented)
#[cfg(target_os = "windows")]
pub fn register_file_associations() -> Result<(), Box<dyn std::error::Error>> {
    debug!("Windows: File association registration not yet implemented");
    Ok(())
}

// Non-Windows platforms - provide stub implementation
#[cfg(not(target_os = "windows"))]
pub fn register_file_associations() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
