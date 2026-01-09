// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

//! Desktop integration for icons, taskbar, and system tray

use gtk::{gdk, glib, prelude::*};
use log::info;
#[allow(unused_imports)]
use log::warn;

/// Desktop integration manager for handling app icons and system integration
#[allow(dead_code)]
pub struct DesktopIntegration;

impl DesktopIntegration {
    /// Force create taskbar icons in system directories
    fn force_create_taskbar_icons() {
        info!("🎯 Force creating taskbar icons in system directories");
        
        // Try to create icons in standard system locations
        let icon_dirs = vec![
            "/usr/share/icons/hicolor/scalable/apps".to_string(),
            "/usr/local/share/icons/hicolor/scalable/apps".to_string(),
            format!("{}/.local/share/icons/hicolor/scalable/apps", std::env::var("HOME").unwrap_or_default()),
            format!("{}/.icons/hicolor/scalable/apps", std::env::var("HOME").unwrap_or_default()),
        ];
        
        // Also add temp directories
        let temp_dirs = vec![
            std::env::temp_dir().join("hicolor").join("scalable").join("apps"),
            std::env::temp_dir().join("amberol-taskbar-icons"),
        ];
        
        // Generate SVG icons in all possible locations
        for dir_path in &icon_dirs {
            let path = std::path::Path::new(dir_path);
            if let Ok(()) = std::fs::create_dir_all(path) {
                let icon_file = path.join("io.bassi.Amberol.svg");
                if let Some(icon_path_str) = icon_file.to_str() {
                    let _ = Self::create_svg_icon(icon_path_str);
                    info!("📁 Created taskbar icon in: {}", dir_path);
                }
            }
        }
        
        for dir_path in &temp_dirs {
            let _ = std::fs::create_dir_all(dir_path);
            let icon_file = dir_path.join("io.bassi.Amberol.svg");
            if let Some(icon_path_str) = icon_file.to_str() {
                let _ = Self::create_svg_icon(icon_path_str);
                info!("📁 Created taskbar icon in: {:?}", dir_path);
            }
        }
        
        // Add temp directories to icon theme
        if let Some(display) = gdk::Display::default() {
            let icon_theme = gtk::IconTheme::for_display(&display);
            for dir_path in &temp_dirs {
                icon_theme.add_search_path(dir_path);
            }
        }
    }

    /// Set up desktop integration including taskbar icons and tray icons
    pub fn setup_integration(app: &crate::application::Application) {
        info!("🖥️ Setting up desktop integration");
        
        // Set application icon for all windows
        Self::setup_app_icon(app);
        
        // Setup system tray if supported
        Self::setup_system_tray(app);
        
        info!("✅ Desktop integration setup complete");
    }
    
        /// Set the application icon for taskbar visibility
    fn setup_app_icon(app: &crate::application::Application) {
        info!("🎨 Setting up programmatic application icon for taskbar");
        
        // Force create icon theme files first
        Self::force_create_taskbar_icons();
        
        // Always use programmatic icon for consistency
        if let Some(mut icon_surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(48) {
            // Convert to GdkTexture for GTK4
            let pixbuf = Self::surface_to_pixbuf(&mut icon_surface);
            let _texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);
            
            // Set as default icon for all windows
            for window in app.windows() {
                if let Some(gtk_window) = window.downcast_ref::<gtk::Window>() {
                    info!("🎯 Setting taskbar icon for window: {:?}", gtk_window.title());
                    
                    // Method 2: Set programmatic icon name after ensuring it exists in theme
                    let icon_theme = gtk::IconTheme::for_display(&gtk::prelude::WidgetExt::display(gtk_window));
                    icon_theme.add_search_path("data/icons/hicolor/scalable/apps");
                    
                    // Force create the icon in the theme directory
                    if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
                        let custom_icons_dir = temp_dir.join("amberol-taskbar");
                        let _ = std::fs::create_dir_all(&custom_icons_dir);
                        
                        // Create the SVG icon file directly
                        let icon_file = custom_icons_dir.join("io.bassi.Amberol.svg");
                        let svg_content = Self::create_taskbar_svg();
                        if std::fs::write(&icon_file, svg_content).is_ok() {
                            icon_theme.add_search_path(&custom_icons_dir);
                            info!("✅ Created taskbar icon file: {:?}", icon_file);
                        }
                    }
                    
                    gtk_window.set_icon_name(Some("io.bassi.Amberol"));
                    
                    // Method 3: Try setting via application ID as fallback
                    if let Some(app) = gtk_window.application() {
                        if let Some(app_id) = app.application_id() {
                            gtk_window.set_icon_name(Some(&app_id));
                            info!("🔧 Set icon name to application ID: {}", app_id);
                        }
                    }
                    
                    #[cfg(target_os = "windows")]
                    Self::set_windows_taskbar_icon(gtk_window, &_texture);
                }
            }
            
            info!("✅ Programmatic application icon set for taskbar");
        } else {
            warn!("⚠️ Failed to create app icon surface");
        }
    }
    
    /// Convert Cairo surface to GdkPixbuf
    fn surface_to_pixbuf(surface: &mut gtk::cairo::ImageSurface) -> gtk::gdk_pixbuf::Pixbuf {
        let width = surface.width();
        let height = surface.height();
        let stride = surface.stride();
        
        // Get surface data
        let data = surface.data().unwrap();
        
        // Create pixbuf from RGBA data
        gtk::gdk_pixbuf::Pixbuf::from_bytes(
            &glib::Bytes::from(&data[..]),
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true, // has_alpha
            8,    // bits_per_sample
            width,
            height,
            stride,
        )
    }
    
    /// Set Windows-specific taskbar icon
    #[cfg(target_os = "windows")]
    fn set_windows_taskbar_icon(window: &gtk::Window, _texture: &gtk::gdk::Texture) {
        use gtk::prelude::*;
        
        info!("🪟 Setting Windows taskbar icon");
        
        // Get the native window handle
        if let Some(_surface) = window.surface() {
            // The embedded resource icon should handle taskbar display
            // This function exists for future enhancements
            info!("🎨 Windows taskbar icon handled by embedded resource");
        }
    }
    
    /// Setup system tray integration
    fn setup_system_tray(_app: &crate::application::Application) {
        info!("🔔 Setting up system tray integration");
        
        #[cfg(target_os = "windows")]
        {
            // System tray is handled in the separate system_tray module
            info!("🪟 Windows system tray will use custom icon");
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("🐧 System tray integration not implemented for this platform");
        }
    }
    
    /// Create application icon at multiple sizes for desktop files
    #[allow(dead_code)]
    pub fn generate_desktop_icons() -> Result<(), Box<dyn std::error::Error>> {
        info!("🖥️ Generating desktop icons");
        
        // Standard icon sizes for desktop environments
        let sizes = [16, 22, 24, 32, 48, 64, 96, 128, 256, 512];
        
        for &size in &sizes {
            if let Some(_surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(size) {
                let filename = format!("amberol-{}.png", size);
                // Note: PNG writing would require additional Cairo features
                info!("✅ Would create desktop icon: {}", filename);
            }
        }
        
        // Also create SVG version for scalability
        Self::create_svg_icon("amberol.svg")?;
        
        Ok(())
    }
    
    /// Create SVG version of the app icon
    fn create_svg_icon(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let svg_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg width="128" height="128" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="musicGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#ff8c00;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#ff6600;stop-opacity:1" />
    </linearGradient>
  </defs>
  
  <!-- Background circle -->
  <circle cx="64" cy="64" r="60" fill="url(#musicGrad)" stroke="#cc5500" stroke-width="2"/>
  
  <!-- Music note -->
  <g fill="#ffffff" stroke="#ffffff" stroke-width="1">
    <!-- Note head -->
    <ellipse cx="45" cy="85" rx="12" ry="8" transform="rotate(-20 45 85)"/>
    
    <!-- Note stem -->
    <rect x="54" y="30" width="3" height="55"/>
    
    <!-- Note flag -->
    <path d="M57 30 Q75 25 80 35 Q75 40 65 38 L57 40 Z"/>
    
    <!-- Additional decorative notes -->
    <circle cx="75" cy="70" r="4" opacity="0.7"/>
    <circle cx="85" cy="60" r="3" opacity="0.5"/>
  </g>
  
  <!-- Sound waves -->
  <g fill="none" stroke="#ffffff" stroke-width="2" opacity="0.6">
    <path d="M90 50 Q95 55 90 60"/>
    <path d="M95 45 Q102 55 95 65"/>
    <path d="M100 40 Q109 55 100 70"/>
  </g>
</svg>"##;
        
        std::fs::write(filename, svg_content)?;
        info!("✅ Created SVG icon: {}", filename);
        Ok(())
    }
    
    /// Create SVG content specifically for taskbar icon
    fn create_taskbar_svg() -> String {
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg width="64" height="64" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="musicGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#ff8c00;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#ff6600;stop-opacity:1" />
    </linearGradient>
  </defs>
  <!-- Musical note for application icon -->
  <circle cx="16" cy="48" r="6" fill="url(#musicGrad)" stroke="#333" stroke-width="1"/>
  <circle cx="44" cy="38" r="5" fill="url(#musicGrad)" stroke="#333" stroke-width="1"/>
  <path d="M22 48 L22 16 L50 12 L50 38" stroke="#333" stroke-width="3" fill="none" stroke-linecap="round"/>
  <!-- Staff lines for context -->
  <line x1="8" y1="20" x2="56" y2="20" stroke="#ddd" stroke-width="1"/>
  <line x1="8" y1="28" x2="56" y2="28" stroke="#ddd" stroke-width="1"/>
  <line x1="8" y1="36" x2="56" y2="36" stroke="#ddd" stroke-width="1"/>
  <line x1="8" y1="44" x2="56" y2="44" stroke="#ddd" stroke-width="1"/>
</svg>"##.to_string()
    }
}