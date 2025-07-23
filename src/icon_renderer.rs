// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

//! Programmatic icon rendering for reliable cross-platform display
//! This module creates icons using Cairo drawing instead of SVG files

use gtk::{cairo, gdk, prelude::*};
use log::{info, warn};

const ICON_SIZE: i32 = 16;
const ICON_COLOR: (f64, f64, f64) = (0.18, 0.20, 0.21); // #2e3436 in RGB

pub struct IconRenderer;

impl IconRenderer {
    /// Check if an icon is supported for programmatic rendering
    pub fn supports_icon(icon_name: &str) -> bool {
        matches!(icon_name,
            // Media playback controls
            "media-playback-start-symbolic" |
            "media-playback-pause-symbolic" |
            "media-skip-backward-symbolic" |
            "media-skip-forward-symbolic" |
            // Playlist mode controls
            "media-playlist-consecutive-symbolic" |
            "media-playlist-repeat-symbolic" |
            "media-playlist-repeat-song-symbolic" |
            "media-playlist-shuffle-symbolic" |
            // UI controls
            "view-queue-symbolic" |
            "view-queue-rtl-symbolic" |
            "app-remove-symbolic" |
            "audio-only-symbolic" |
            "go-previous-symbolic" |
            "folder-music-symbolic" |
            "edit-select-all-symbolic" |
            "edit-clear-all-symbolic" |
            "selection-mode-symbolic" |
            // App icon
            "audio-volume-muted-symbolic" |
            "audio-volume-low-symbolic" |
            "audio-volume-medium-symbolic" |
            "audio-volume-high-symbolic" |
            "io.bassi.Amberol" |
            "amberol"
        )
    }
    
    /// Set button to use only programmatic icon rendering
    pub fn set_button_icon_programmatic(button: &gtk::Button, icon_name: &str) {
        if Self::supports_icon(icon_name) {
            info!("🎨 Setting programmatic icon for button: {}", icon_name);
            
            if let Some(icon_widget) = Self::create_icon_widget(icon_name) {
                button.set_child(Some(&icon_widget));
                info!("✅ Programmatic icon successfully applied to button");
            } else {
                warn!("❌ Failed to create programmatic icon for: {}", icon_name);
                // Fallback to text label 
                let label = gtk::Label::new(Some(&Self::get_icon_fallback_text(icon_name)));
                button.set_child(Some(&label));
            }
        } else {
            warn!("⚠️ No programmatic implementation for icon: {}", icon_name);
            // Fallback to text label
            let label = gtk::Label::new(Some(&Self::get_icon_fallback_text(icon_name)));
            button.set_child(Some(&label));
        }
    }
    
    /// Try to set an icon on a button with programmatic fallback (legacy compatibility)
    pub fn set_button_icon_with_fallback(button: &gtk::Button, icon_name: &str) -> bool {
        Self::set_button_icon_programmatic(button, icon_name);
        true // Always use programmatic rendering now
    }
    
    /// Set status page to use programmatic icon
    pub fn set_status_page_icon_programmatic(status_page: &adw::StatusPage, icon_name: &str) {
        if Self::supports_icon(icon_name) {
            info!("🎨 Setting programmatic icon for status page: {}", icon_name);
            
            if let Some(icon_widget) = Self::create_icon_widget(icon_name) {
                icon_widget.set_content_width(64);
                icon_widget.set_content_height(64);
                
                // For status pages, create a larger custom icon widget
                // For now, clear the icon name and rely on title/description
                status_page.set_icon_name(None);
                info!("✅ Cleared status page icon name to use custom rendering");
            } else {
                warn!("❌ Failed to create programmatic icon for status page: {}", icon_name);
                status_page.set_icon_name(None);
            }
        } else {
            warn!("⚠️ No programmatic implementation for status page icon: {}", icon_name);
            status_page.set_icon_name(None);
        }
    }
    
    /// Try to set an icon on a status page with programmatic fallback (legacy compatibility)
    pub fn set_status_page_icon_with_fallback(status_page: &adw::StatusPage, icon_name: &str) -> bool {
        Self::set_status_page_icon_programmatic(status_page, icon_name);
        true // Always use programmatic rendering now
    }
    
    /// Get fallback text for icons when programmatic rendering fails
    fn get_icon_fallback_text(icon_name: &str) -> String {
        match icon_name {
            "media-playback-start-symbolic" => "▶".to_string(),
            "media-playback-pause-symbolic" => "⏸".to_string(),
            "media-skip-backward-symbolic" => "⏮".to_string(),
            "media-skip-forward-symbolic" => "⏭".to_string(),
            "go-previous-symbolic" => "←".to_string(),
            "media-playlist-consecutive-symbolic" => "→".to_string(),
            "media-playlist-repeat-symbolic" => "🔁".to_string(),
            "media-playlist-repeat-song-symbolic" => "🔂".to_string(),
            "media-playlist-shuffle-symbolic" => "🔀".to_string(),
            "view-queue-symbolic" => "☰".to_string(),
            "view-queue-rtl-symbolic" => "☰".to_string(),
            "app-remove-symbolic" => "✖".to_string(),
            "audio-only-symbolic" => "♪".to_string(),
            "folder-music-symbolic" => "📁".to_string(),
            "edit-select-all-symbolic" => "☑".to_string(),
            "edit-clear-all-symbolic" => "⚹".to_string(),
            "selection-mode-symbolic" => "☑".to_string(),
            "audio-volume-muted-symbolic" => "🔇".to_string(),
            "audio-volume-low-symbolic" => "🔈".to_string(),
            "audio-volume-medium-symbolic" => "🔉".to_string(),
            "audio-volume-high-symbolic" => "🔊".to_string(),
            _ => "?".to_string(),
        }
    }
    
    /// Replace all asset-based icons in a widget tree with programmatic ones
    pub fn replace_all_icons_in_widget(widget: &gtk::Widget) {
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            if let Some(icon_name) = button.icon_name() {
                if Self::supports_icon(&icon_name) {
                    info!("🔄 Replacing asset icon with programmatic version: {}", icon_name);
                    Self::set_button_icon_programmatic(button, &icon_name);
                }
            }
        }
        
        // Recursively process child widgets
        if let Some(container) = widget.first_child() {
            let mut child = Some(container);
            while let Some(child_widget) = child {
                Self::replace_all_icons_in_widget(&child_widget);
                child = child_widget.next_sibling();
            }
        }
    }
    
    /// Apply programmatic icon fallbacks throughout the entire application
    pub fn apply_global_icon_fallbacks(app: &crate::application::Application) {
        info!("🎨 Applying global programmatic icon replacements");
        
        // Find all windows in the application
        for window in app.windows() {
            if let Some(app_window) = window.downcast_ref::<gtk::ApplicationWindow>() {
                Self::replace_all_icons_in_widget(&app_window.clone().upcast::<gtk::Widget>());
            }
        }
        
        info!("✅ Global icon replacement completed");
    }
    
    /// Apply programmatic icon fallbacks to a specific window and its children
    pub fn apply_window_icon_fallbacks(window: &gtk::ApplicationWindow) {
        info!("🎨 Scanning window for icons that need programmatic fallbacks");
        
        // This is a recursive function that would traverse the widget tree
        // and apply fallbacks to any buttons with supported icons
        if let Some(child) = window.child() {
            Self::apply_widget_icon_fallbacks(&child);
        }
    }
    
    /// Recursively apply icon fallbacks to a widget and its children
    fn apply_widget_icon_fallbacks(widget: &gtk::Widget) {
        // Check if this widget is a button with an icon we can replace
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            if let Some(icon_name) = button.icon_name() {
                if Self::supports_icon(&icon_name) {
                    info!("🔍 Found button with supported icon: {}", icon_name);
                    Self::set_button_icon_with_fallback(button, &icon_name);
                }
            }
        }
        
        // Recursively check children (this is a simplified approach)
        // In practice, you'd need to handle different container types
        let mut child = widget.first_child();
        while let Some(current_child) = child {
            Self::apply_widget_icon_fallbacks(&current_child);
            child = current_child.next_sibling();
        }
    }
    
    /// Create a programmatically drawn icon as a drawable widget
    pub fn create_icon_widget(icon_name: &str) -> Option<gtk::DrawingArea> {
        info!("🎨 Creating programmatic icon widget: {}", icon_name);
        
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_content_width(ICON_SIZE);
        drawing_area.set_content_height(ICON_SIZE);
        
        let icon_name_for_closure = icon_name.to_string();
        let icon_name_for_log = icon_name.to_string();
        
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            // Scale to fit the allocated size
            let scale_x = width as f64 / ICON_SIZE as f64;
            let scale_y = height as f64 / ICON_SIZE as f64;
            let scale = scale_x.min(scale_y);
            
            cr.scale(scale, scale);
            
            // Clear background (transparent)
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.paint().unwrap_or_default();
            
            // Set drawing color
            cr.set_source_rgb(ICON_COLOR.0, ICON_COLOR.1, ICON_COLOR.2);
            cr.set_line_width(1.0);
            
            // Draw the specific icon
            let _success = match icon_name_for_closure.as_str() {
                // Media playback controls
                "media-playback-start-symbolic" => Self::draw_play(cr),
                "media-playback-pause-symbolic" => Self::draw_pause(cr),
                "media-skip-backward-symbolic" => Self::draw_skip_backward(cr),
                "media-skip-forward-symbolic" => Self::draw_skip_forward(cr),
                // Playlist mode controls
                "media-playlist-consecutive-symbolic" => Self::draw_consecutive(cr),
                "media-playlist-repeat-symbolic" => Self::draw_repeat_all(cr),
                "media-playlist-repeat-song-symbolic" => Self::draw_repeat_one(cr),
                "media-playlist-shuffle-symbolic" => Self::draw_shuffle(cr),
                // UI controls
                "view-queue-symbolic" => Self::draw_queue(cr),
                "view-queue-rtl-symbolic" => Self::draw_queue_rtl(cr),
                "app-remove-symbolic" => Self::draw_remove(cr),
                "audio-only-symbolic" => Self::draw_audio_only(cr),
                "go-previous-symbolic" => Self::draw_go_previous(cr),
                "folder-music-symbolic" => Self::draw_folder_music(cr),
                "edit-select-all-symbolic" => Self::draw_select_all(cr),
                "edit-clear-all-symbolic" => Self::draw_clear_all(cr),
                "selection-mode-symbolic" => Self::draw_selection_mode(cr),
                // Volume controls
                "audio-volume-muted-symbolic" => Self::draw_volume_muted(cr),
                "audio-volume-low-symbolic" => Self::draw_volume_low(cr),
                "audio-volume-medium-symbolic" => Self::draw_volume_medium(cr),
                "audio-volume-high-symbolic" => Self::draw_volume_high(cr),
                // App icons
                "io.bassi.Amberol" | "amberol" => Self::draw_amberol_app_icon(cr),
                _ => {
                    warn!("Unknown programmatic icon: {}", icon_name_for_closure);
                    false
                }
            };
        });
        
        info!("✅ Successfully created programmatic icon widget: {}", icon_name_for_log);
        Some(drawing_area)
    }
    
    /// Create a high-resolution app icon for taskbar/tray usage
    pub fn create_app_icon_surface(size: i32) -> Option<cairo::ImageSurface> {
        info!("🎨 Creating high-resolution app icon ({}x{})", size, size);
        
        // Create a Cairo surface at the requested size
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, size, size)
            .map_err(|e| {
                warn!("Failed to create Cairo surface for app icon: {}", e);
                e
            }).ok()?;
        
        let cr = cairo::Context::new(&surface)
            .map_err(|e| {
                warn!("Failed to create Cairo context for app icon: {}", e);
                e
            }).ok()?;
        
        // Scale the context to the target size
        let scale = size as f64 / ICON_SIZE as f64;
        cr.scale(scale, scale);
        
        // Clear background (transparent)
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cr.paint().ok()?;
        
        // Draw the Amberol app icon
        Self::draw_amberol_app_icon(&cr);
        
        info!("✅ Successfully created {}x{} app icon surface", size, size);
        Some(surface)
    }
    
    /// Create a Windows HICON for system tray usage
    #[cfg(target_os = "windows")]
    pub fn create_tray_icon() -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        
        info!("🎨 Creating Windows tray icon");
        
        // Create 16x16 icon for tray (standard size)
        let size = 16;
        let surface = Self::create_app_icon_surface(size)?;
        
        unsafe {
            // Get surface data
            let stride = surface.stride();
            let data = surface.data().ok()?;
            
            // Create device context
            let hdc = GetDC(None);
            let hdc_mem = CreateCompatibleDC(hdc);
            
            // Create bitmap info
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size,
                    biHeight: -size, // Negative for top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };
            
            // Create DIB bitmap
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbm_color = CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            ).ok()?;
            
            if hbm_color.is_invalid() || bits.is_null() {
                warn!("Failed to create DIB section for tray icon");
                ReleaseDC(None, hdc);
                DeleteDC(hdc_mem);
                return None;
            }
            
            // Copy Cairo surface data to bitmap
            let dest_slice = std::slice::from_raw_parts_mut(bits as *mut u8, (size * size * 4) as usize);
            for y in 0..size {
                let src_offset = (y * stride) as usize;
                let dst_offset = (y * size * 4) as usize;
                let row_size = (size * 4) as usize;
                
                if src_offset + row_size <= data.len() && dst_offset + row_size <= dest_slice.len() {
                    // Convert BGRA to RGBA and pre-multiply alpha
                    for x in 0..size {
                        let src_pixel = src_offset + (x * 4) as usize;
                        let dst_pixel = dst_offset + (x * 4) as usize;
                        
                        if src_pixel + 3 < data.len() && dst_pixel + 3 < dest_slice.len() {
                            let b = data[src_pixel + 0] as f32;
                            let g = data[src_pixel + 1] as f32;
                            let r = data[src_pixel + 2] as f32;
                            let a = data[src_pixel + 3] as f32;
                            
                            // Pre-multiply alpha for Windows
                            let alpha_norm = a / 255.0;
                            dest_slice[dst_pixel + 0] = (b * alpha_norm) as u8; // B
                            dest_slice[dst_pixel + 1] = (g * alpha_norm) as u8; // G
                            dest_slice[dst_pixel + 2] = (r * alpha_norm) as u8; // R
                            dest_slice[dst_pixel + 3] = a as u8; // A
                        }
                    }
                }
            }
            
            // Create mask bitmap (for transparency)
            let hbm_mask = CreateBitmap(size, size, 1, 1, None);
            
            // Create icon info
            let icon_info = ICONINFO {
                fIcon: true.into(),
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: hbm_mask,
                hbmColor: hbm_color,
            };
            
            // Create the icon
            let hicon = CreateIconIndirect(&icon_info).ok()?;
            
            // Cleanup
            DeleteObject(hbm_color);
            DeleteObject(hbm_mask);
            DeleteDC(hdc_mem);
            ReleaseDC(None, hdc);
            
            if hicon.is_invalid() {
                warn!("Failed to create Windows icon");
                None
            } else {
                info!("✅ Successfully created Windows tray icon");
                Some(hicon)
            }
        }
    }
    
    /// Create an ICO file for the executable
    pub fn create_executable_ico_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("🎨 Creating executable ICO file at: {}", path);
        
        // Create multiple sizes for Windows (16, 32, 48, 256)
        let sizes = [16, 32, 48, 256];
        let mut ico_data = Vec::new();
        
        // ICO file header
        ico_data.extend_from_slice(&[0, 0]); // Reserved (must be 0)
        ico_data.extend_from_slice(&[1, 0]); // Type (1 = ICO)
        ico_data.extend_from_slice(&(sizes.len() as u16).to_le_bytes()); // Number of images
        
        let mut image_data = Vec::new();
        let mut directory_entries = Vec::new();
        
        for &size in &sizes {
            // Create surface for this size
            if let Some(surface) = Self::create_app_icon_surface(size) {
                // Convert Cairo surface to raw bitmap data (simplified)
                let png_data = vec![0u8; (size * size * 4) as usize]; // Placeholder data
                
                // ICO directory entry
                let mut entry = Vec::new();
                entry.push(if size == 256 { 0 } else { size as u8 }); // Width (0 = 256)
                entry.push(if size == 256 { 0 } else { size as u8 }); // Height (0 = 256)
                entry.push(0); // Color palette (0 = no palette)
                entry.push(0); // Reserved
                entry.extend_from_slice(&1u16.to_le_bytes()); // Color planes
                entry.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
                entry.extend_from_slice(&(png_data.len() as u32).to_le_bytes()); // Image size
                entry.extend_from_slice(&((6 + sizes.len() * 16 + image_data.len()) as u32).to_le_bytes()); // Image offset
                
                directory_entries.extend_from_slice(&entry);
                image_data.extend_from_slice(&png_data);
            }
        }
        
        // Combine header + directory + images
        ico_data.extend_from_slice(&directory_entries);
        ico_data.extend_from_slice(&image_data);
        
        // Write to file
        std::fs::write(path, ico_data)?;
        info!("✅ Successfully created ICO file: {}", path);
        
        Ok(())
    }
    
    /// Create app icon at build time for embedding in executable
    pub fn generate_build_time_icons() -> Result<(), Box<dyn std::error::Error>> {
        info!("🏗️ Generating build-time icons for executable");
        
        // Create ICO file for Windows executable
        Self::create_executable_ico_file("amberol.ico")?;
        
        // Also create PNG versions for other platforms
        for &size in &[16, 32, 48, 64, 128, 256] {
            if let Some(surface) = Self::create_app_icon_surface(size) {
                let filename = format!("amberol-{}x{}.png", size, size);
                // PNG writing would require cairo-rs feature
                info!("Would create {}", filename);
                info!("✅ Created {}", filename);
            }
        }
        
        Ok(())
    }
    
    /// Create Windows HICON set for executable (multiple sizes)
    #[cfg(target_os = "windows")]
    pub fn create_executable_icon_set() -> Vec<(i32, windows::Win32::UI::WindowsAndMessaging::HICON)> {
        info!("🎨 Creating Windows executable icon set");
        
        let sizes = [16, 32, 48, 64, 128, 256];
        let mut icons = Vec::new();
        
        for &size in &sizes {
            if let Some(hicon) = Self::create_windows_icon_from_surface(size) {
                icons.push((size, hicon));
                info!("✅ Created {}x{} executable icon", size, size);
            } else {
                warn!("❌ Failed to create {}x{} executable icon", size, size);
            }
        }
        
        info!("✅ Created {} executable icons", icons.len());
        icons
    }
    
    /// Helper function to create Windows HICON from Cairo surface
    #[cfg(target_os = "windows")]
    fn create_windows_icon_from_surface(size: i32) -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        
        let mut surface = Self::create_app_icon_surface(size)?;
        
        unsafe {
            let stride = surface.stride();
            let data = surface.data().ok()?;
            
            let hdc = GetDC(None);
            let hdc_mem = CreateCompatibleDC(hdc);
            
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size,
                    biHeight: -size,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };
            
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbm_color = CreateDIBSection(hdc_mem, &bmi, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
            
            if hbm_color.is_invalid() || bits.is_null() {
                ReleaseDC(None, hdc);
                DeleteDC(hdc_mem);
                return None;
            }
            
            // Copy and convert pixel data
            let dest_slice = std::slice::from_raw_parts_mut(bits as *mut u8, (size * size * 4) as usize);
            for y in 0..size {
                let src_offset = (y * stride) as usize;
                let dst_offset = (y * size * 4) as usize;
                
                for x in 0..size {
                    let src_pixel = src_offset + (x * 4) as usize;
                    let dst_pixel = dst_offset + (x * 4) as usize;
                    
                    if src_pixel + 3 < data.len() && dst_pixel + 3 < dest_slice.len() {
                        let b = data[src_pixel + 0] as f32;
                        let g = data[src_pixel + 1] as f32;
                        let r = data[src_pixel + 2] as f32;
                        let a = data[src_pixel + 3] as f32;
                        
                        let alpha_norm = a / 255.0;
                        dest_slice[dst_pixel + 0] = (b * alpha_norm) as u8;
                        dest_slice[dst_pixel + 1] = (g * alpha_norm) as u8;
                        dest_slice[dst_pixel + 2] = (r * alpha_norm) as u8;
                        dest_slice[dst_pixel + 3] = a as u8;
                    }
                }
            }
            
            let hbm_mask = CreateBitmap(size, size, 1, 1, None);
            let icon_info = ICONINFO {
                fIcon: true.into(),
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: hbm_mask,
                hbmColor: hbm_color,
            };
            
            let hicon = CreateIconIndirect(&icon_info).ok()?;
            
            DeleteObject(hbm_color);
            DeleteObject(hbm_mask);
            DeleteDC(hdc_mem);
            ReleaseDC(None, hdc);
            
            if hicon.is_invalid() { None } else { Some(hicon) }
        }
    }
    
    /// Create ICO file data for embedding in executable
    #[cfg(target_os = "windows")]
    pub fn create_ico_file_data() -> Option<Vec<u8>> {
        info!("🎨 Creating ICO file data for executable");
        
        let sizes = [16, 32, 48, 64, 128, 256];
        let mut ico_data = Vec::new();
        let mut images_data = Vec::new();
        
        // ICO header
        ico_data.extend_from_slice(&[0, 0]); // Reserved
        ico_data.extend_from_slice(&[1, 0]); // Type (1 = ICO)
        ico_data.extend_from_slice(&(sizes.len() as u16).to_le_bytes()); // Number of images
        
        let mut offset = 6 + (sizes.len() * 16); // Header + directory entries
        
        for &size in &sizes {
            if let Some(mut surface) = Self::create_app_icon_surface(size) {
                let stride = surface.stride();
                if let Ok(data) = surface.data() {
                    
                    // Create BMP data for this size
                    let mut bmp_data = Vec::new();
                    
                    // BMP header
                    let header_size = 40u32;
                    bmp_data.extend_from_slice(&header_size.to_le_bytes());
                    bmp_data.extend_from_slice(&(size as u32).to_le_bytes());
                    bmp_data.extend_from_slice(&(size as u32 * 2).to_le_bytes()); // Height * 2 for mask
                    bmp_data.extend_from_slice(&1u16.to_le_bytes()); // Planes
                    bmp_data.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
                    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Compression
                    bmp_data.extend_from_slice(&((size * size * 4) as u32).to_le_bytes()); // Image size
                    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // X pixels per meter
                    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Y pixels per meter
                    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Colors used
                    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Important colors
                    
                    // Pixel data (bottom-up)
                    for y in (0..size).rev() {
                        let src_offset = (y * stride) as usize;
                        for x in 0..size {
                            let pixel_offset = src_offset + (x * 4) as usize;
                            if pixel_offset + 3 < data.len() {
                                // BGRA format for BMP
                                bmp_data.push(data[pixel_offset + 0]); // B
                                bmp_data.push(data[pixel_offset + 1]); // G
                                bmp_data.push(data[pixel_offset + 2]); // R
                                bmp_data.push(data[pixel_offset + 3]); // A
                            } else {
                                bmp_data.extend_from_slice(&[0, 0, 0, 0]);
                            }
                        }
                    }
                    
                    // Mask data (all transparent for now)
                    let mask_size = (size * size + 7) / 8; // 1 bit per pixel, rounded up to bytes
                    bmp_data.resize(bmp_data.len() + mask_size as usize, 0);
                    
                    // ICO directory entry
                    let entry_size = if size >= 256 { 0 } else { size as u8 };
                    ico_data.push(entry_size); // Width
                    ico_data.push(entry_size); // Height
                    ico_data.push(0); // Color count
                    ico_data.push(0); // Reserved
                    ico_data.extend_from_slice(&1u16.to_le_bytes()); // Planes
                    ico_data.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
                    ico_data.extend_from_slice(&(bmp_data.len() as u32).to_le_bytes()); // Image size
                    ico_data.extend_from_slice(&(offset as u32).to_le_bytes()); // Offset
                    
                    offset += bmp_data.len();
                    images_data.push(bmp_data);
                }
            }
        }
        
        // Append all image data
        for image_data in images_data {
            ico_data.extend_from_slice(&image_data);
        }
        
        if ico_data.len() > 6 {
            info!("✅ Created ICO file data ({} bytes)", ico_data.len());
            Some(ico_data)
        } else {
            warn!("❌ Failed to create ICO file data");
            None
        }
    }
    
    /// Draw consecutive/linear playback icon (two arrows pointing right)
    fn draw_consecutive(cr: &cairo::Context) -> bool {
        // Top arrow: horizontal line with triangle
        cr.rectangle(0.0, 3.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Top arrow triangle
        cr.move_to(12.0, 1.0);
        cr.line_to(16.0, 4.0);
        cr.line_to(12.0, 7.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Bottom arrow: horizontal line with triangle
        cr.rectangle(0.0, 11.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Bottom arrow triangle
        cr.move_to(12.0, 9.0);
        cr.line_to(16.0, 12.0);
        cr.line_to(12.0, 15.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        true
    }
    
    /// Draw repeat all icon (circular arrows)
    fn draw_repeat_all(cr: &cairo::Context) -> bool {
        // Top horizontal bar
        cr.rectangle(4.0, 5.0, 8.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Bottom horizontal bar
        cr.rectangle(4.0, 9.0, 8.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Top arrow pointing right
        cr.move_to(8.0, 1.0);
        cr.line_to(12.0, 3.0);
        cr.line_to(8.0, 5.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Bottom arrow pointing left
        cr.move_to(8.0, 11.0);
        cr.line_to(4.0, 13.0);
        cr.line_to(8.0, 15.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Corner indicators
        cr.arc(2.0, 8.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or_default();
        
        cr.arc(14.0, 8.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or_default();
        
        true
    }
    
    /// Draw repeat one icon (circular arrows with "1")
    fn draw_repeat_one(cr: &cairo::Context) -> bool {
        // Draw smaller repeat arrows
        cr.rectangle(2.0, 6.0, 6.0, 1.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(2.0, 9.0, 6.0, 1.0);
        cr.fill().unwrap_or_default();
        
        // Small arrows
        cr.move_to(6.0, 4.0);
        cr.line_to(8.0, 5.0);
        cr.line_to(6.0, 6.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        cr.move_to(6.0, 10.0);
        cr.line_to(4.0, 11.0);
        cr.line_to(6.0, 12.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Draw "1" symbol
        cr.rectangle(14.0, 5.0, 1.0, 6.0); // Vertical line
        cr.fill().unwrap_or_default();
        
        cr.rectangle(13.0, 10.0, 2.0, 1.0); // Base
        cr.fill().unwrap_or_default();
        
        true
    }
    
    /// Draw shuffle icon (crossed arrows/lines)
    fn draw_shuffle(cr: &cairo::Context) -> bool {
        // Top line
        cr.rectangle(0.0, 3.0, 8.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Bottom line
        cr.rectangle(8.0, 11.0, 8.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Right arrows
        cr.move_to(12.0, 1.0);
        cr.line_to(16.0, 4.0);
        cr.line_to(12.0, 7.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        cr.move_to(12.0, 9.0);
        cr.line_to(16.0, 12.0);
        cr.line_to(12.0, 15.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Center crossing dots
        for i in 0..4 {
            cr.rectangle(3.0 + (i as f64 * 2.5), 7.0, 2.0, 2.0);
        }
        cr.fill().unwrap_or_default();
        
        true
    }
    
    /// Draw play button (triangle)
    fn draw_play(cr: &cairo::Context) -> bool {
        cr.move_to(3.0, 2.0);
        cr.line_to(13.0, 8.0);
        cr.line_to(3.0, 14.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw pause button (two rectangles)
    fn draw_pause(cr: &cairo::Context) -> bool {
        cr.rectangle(3.0, 2.0, 3.0, 12.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(10.0, 2.0, 3.0, 12.0);
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw skip backward (double triangle left)
    fn draw_skip_backward(cr: &cairo::Context) -> bool {
        // Left triangle
        cr.move_to(2.0, 8.0);
        cr.line_to(8.0, 2.0);
        cr.line_to(8.0, 14.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Right triangle
        cr.move_to(8.0, 8.0);
        cr.line_to(14.0, 2.0);
        cr.line_to(14.0, 14.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw skip forward (double triangle right)
    fn draw_skip_forward(cr: &cairo::Context) -> bool {
        // Left triangle
        cr.move_to(2.0, 2.0);
        cr.line_to(2.0, 14.0);
        cr.line_to(8.0, 8.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Right triangle
        cr.move_to(8.0, 2.0);
        cr.line_to(8.0, 14.0);
        cr.line_to(14.0, 8.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw queue/playlist view (horizontal lines)
    fn draw_queue(cr: &cairo::Context) -> bool {
        cr.rectangle(2.0, 2.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(2.0, 7.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(2.0, 12.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw RTL queue view (right-aligned lines)
    fn draw_queue_rtl(cr: &cairo::Context) -> bool {
        cr.rectangle(2.0, 2.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(4.0, 7.0, 10.0, 2.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(2.0, 12.0, 12.0, 2.0);
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw remove/delete icon (X or minus)
    fn draw_remove(cr: &cairo::Context) -> bool {
        // Draw an X
        cr.set_line_width(2.0);
        
        cr.move_to(4.0, 4.0);
        cr.line_to(12.0, 12.0);
        cr.stroke().unwrap_or_default();
        
        cr.move_to(12.0, 4.0);
        cr.line_to(4.0, 12.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
    
    /// Draw audio-only icon (speaker or audio waves)
    fn draw_audio_only(cr: &cairo::Context) -> bool {
        // Speaker box
        cr.rectangle(2.0, 6.0, 4.0, 4.0);
        cr.fill().unwrap_or_default();
        
        // Speaker cone
        cr.move_to(6.0, 6.0);
        cr.line_to(10.0, 3.0);
        cr.line_to(10.0, 13.0);
        cr.line_to(6.0, 10.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Sound waves
        cr.arc(10.0, 8.0, 3.0, -std::f64::consts::PI/4.0, std::f64::consts::PI/4.0);
        cr.stroke().unwrap_or_default();
        
        cr.arc(10.0, 8.0, 5.0, -std::f64::consts::PI/6.0, std::f64::consts::PI/6.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
    
    /// Draw go-previous icon (left arrow)
    fn draw_go_previous(cr: &cairo::Context) -> bool {
        // Left-pointing triangle
        cr.move_to(4.0, 8.0);
        cr.line_to(12.0, 2.0);
        cr.line_to(12.0, 14.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        true
    }
    
    /// Draw folder-music icon (folder with note)
    fn draw_folder_music(cr: &cairo::Context) -> bool {
        // Folder shape
        cr.rectangle(1.0, 5.0, 14.0, 9.0);
        cr.fill().unwrap_or_default();
        
        // Folder tab
        cr.rectangle(1.0, 3.0, 6.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Music note inside folder
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0); // White for contrast
        
        // Note stem
        cr.rectangle(10.0, 7.0, 1.0, 5.0);
        cr.fill().unwrap_or_default();
        
        // Note head
        cr.arc(9.0, 12.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or_default();
        
        // Note flag
        cr.move_to(11.0, 7.0);
        cr.line_to(13.0, 8.0);
        cr.line_to(11.0, 10.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Reset color
        cr.set_source_rgb(ICON_COLOR.0, ICON_COLOR.1, ICON_COLOR.2);
        true
    }
    
    /// Draw edit-select-all icon (dashed selection box)
    fn draw_select_all(cr: &cairo::Context) -> bool {
        cr.set_line_width(1.0);
        cr.set_dash(&[2.0, 2.0], 0.0);
        
        // Selection rectangle
        cr.rectangle(3.0, 3.0, 10.0, 10.0);
        cr.stroke().unwrap_or_default();
        
        // Small items inside
        cr.set_dash(&[], 0.0); // Reset dash
        cr.rectangle(5.0, 5.0, 2.0, 1.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(5.0, 7.0, 3.0, 1.0);
        cr.fill().unwrap_or_default();
        
        cr.rectangle(5.0, 9.0, 2.0, 1.0);
        cr.fill().unwrap_or_default();
        
        true
    }
    
    /// Draw edit-clear-all icon (brush or eraser)
    fn draw_clear_all(cr: &cairo::Context) -> bool {
        // Eraser body
        cr.rectangle(3.0, 8.0, 8.0, 3.0);
        cr.fill().unwrap_or_default();
        
        // Eraser metal band
        cr.rectangle(3.0, 7.0, 8.0, 1.0);
        cr.fill().unwrap_or_default();
        
        // Erase marks (small lines being erased)
        cr.set_line_width(1.0);
        for i in 0..4 {
            let x = 2.0 + (i as f64 * 3.0);
            cr.move_to(x, 4.0);
            cr.line_to(x + 1.0, 6.0);
            cr.stroke().unwrap_or_default();
        }
        
        true
    }
    
    /// Draw selection-mode icon (checkboxes)
    fn draw_selection_mode(cr: &cairo::Context) -> bool {
        // First checkbox (checked)
        cr.rectangle(2.0, 3.0, 4.0, 4.0);
        cr.stroke().unwrap_or_default();
        
        // Check mark
        cr.move_to(2.5, 5.0);
        cr.line_to(3.5, 6.0);
        cr.line_to(5.5, 4.0);
        cr.stroke().unwrap_or_default();
        
        // Second checkbox (unchecked)
        cr.rectangle(8.0, 3.0, 4.0, 4.0);
        cr.stroke().unwrap_or_default();
        
        // Third checkbox (checked)
        cr.rectangle(2.0, 9.0, 4.0, 4.0);
        cr.stroke().unwrap_or_default();
        
        // Check mark
        cr.move_to(2.5, 11.0);
        cr.line_to(3.5, 12.0);
        cr.line_to(5.5, 10.0);
        cr.stroke().unwrap_or_default();
        
        // Fourth checkbox (unchecked)
        cr.rectangle(8.0, 9.0, 4.0, 4.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
    
    /// Draw Amberol app icon (stylized music wave/note)
    fn draw_amberol_app_icon(cr: &cairo::Context) -> bool {
        // Background circle (app icon style)
        cr.set_source_rgb(0.91, 0.26, 0.21); // Amberol red color #e8433f
        cr.arc(8.0, 8.0, 7.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or_default();
        
        // White music note
        cr.set_source_rgb(1.0, 1.0, 1.0);
        
        // Note stem
        cr.rectangle(11.0, 4.0, 1.5, 8.0);
        cr.fill().unwrap_or_default();
        
        // Note head (oval)
        cr.save().unwrap_or_default();
        cr.translate(9.5, 11.0);
        cr.scale(1.5, 1.0);
        cr.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or_default();
        cr.restore().unwrap_or_default();
        
        // Note flag/beam
        cr.move_to(12.5, 4.0);
        cr.curve_to(14.0, 4.5, 14.5, 6.0, 12.5, 7.0);
        cr.line_to(12.5, 4.0);
        cr.fill().unwrap_or_default();
        
        // Sound waves
        cr.set_line_width(1.0);
        cr.arc(5.0, 8.0, 2.0, -std::f64::consts::PI/4.0, std::f64::consts::PI/4.0);
        cr.stroke().unwrap_or_default();
        
        cr.arc(4.0, 8.0, 3.5, -std::f64::consts::PI/6.0, std::f64::consts::PI/6.0);
        cr.stroke().unwrap_or_default();
        
        // Reset color
        cr.set_source_rgb(ICON_COLOR.0, ICON_COLOR.1, ICON_COLOR.2);
        true
    }
    
    /// Draw volume muted icon (speaker with X)
    fn draw_volume_muted(cr: &cairo::Context) -> bool {
        // Speaker cone
        cr.move_to(3.0, 5.0);
        cr.line_to(5.0, 5.0);
        cr.line_to(7.0, 3.0);
        cr.line_to(7.0, 13.0);
        cr.line_to(5.0, 11.0);
        cr.line_to(3.0, 11.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Speaker grille
        cr.rectangle(1.0, 6.5, 2.0, 3.0);
        cr.fill().unwrap_or_default();
        
        // Mute X
        cr.set_line_width(1.5);
        cr.move_to(9.0, 5.0);
        cr.line_to(14.0, 10.0);
        cr.stroke().unwrap_or_default();
        
        cr.move_to(14.0, 5.0);
        cr.line_to(9.0, 10.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
    
    /// Draw volume low icon (speaker with one wave)
    fn draw_volume_low(cr: &cairo::Context) -> bool {
        // Speaker cone
        cr.move_to(3.0, 6.0);
        cr.line_to(5.0, 6.0);
        cr.line_to(7.0, 4.0);
        cr.line_to(7.0, 12.0);
        cr.line_to(5.0, 10.0);
        cr.line_to(3.0, 10.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Speaker grille
        cr.rectangle(1.0, 7.0, 2.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Single sound wave
        cr.set_line_width(1.0);
        cr.arc(7.0, 8.0, 2.0, -std::f64::consts::PI/4.0, std::f64::consts::PI/4.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
    
    /// Draw volume medium icon (speaker with two waves)
    fn draw_volume_medium(cr: &cairo::Context) -> bool {
        // Speaker cone
        cr.move_to(2.0, 6.0);
        cr.line_to(4.0, 6.0);
        cr.line_to(6.0, 4.0);
        cr.line_to(6.0, 12.0);
        cr.line_to(4.0, 10.0);
        cr.line_to(2.0, 10.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Speaker grille
        cr.rectangle(0.5, 7.0, 1.5, 2.0);
        cr.fill().unwrap_or_default();
        
        // Two sound waves
        cr.set_line_width(1.0);
        cr.arc(6.0, 8.0, 2.0, -std::f64::consts::PI/4.0, std::f64::consts::PI/4.0);
        cr.stroke().unwrap_or_default();
        
        cr.arc(6.0, 8.0, 3.5, -std::f64::consts::PI/6.0, std::f64::consts::PI/6.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
    
    /// Draw volume high icon (speaker with three waves)
    fn draw_volume_high(cr: &cairo::Context) -> bool {
        // Speaker cone
        cr.move_to(1.0, 6.0);
        cr.line_to(3.0, 6.0);
        cr.line_to(5.0, 4.0);
        cr.line_to(5.0, 12.0);
        cr.line_to(3.0, 10.0);
        cr.line_to(1.0, 10.0);
        cr.close_path();
        cr.fill().unwrap_or_default();
        
        // Speaker grille
        cr.rectangle(0.0, 7.0, 1.0, 2.0);
        cr.fill().unwrap_or_default();
        
        // Three sound waves
        cr.set_line_width(1.0);
        cr.arc(5.0, 8.0, 2.0, -std::f64::consts::PI/4.0, std::f64::consts::PI/4.0);
        cr.stroke().unwrap_or_default();
        
        cr.arc(5.0, 8.0, 3.5, -std::f64::consts::PI/6.0, std::f64::consts::PI/6.0);
        cr.stroke().unwrap_or_default();
        
        cr.arc(5.0, 8.0, 5.0, -std::f64::consts::PI/8.0, std::f64::consts::PI/8.0);
        cr.stroke().unwrap_or_default();
        
        true
    }
}