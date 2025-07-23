// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

//! Desktop integration for icons, taskbar, and system tray

use gtk::{gio, glib, prelude::*};
use log::{info, warn};

#[cfg(target_os = "windows")]
use windows::Win32::{
    Graphics::Gdi::*,
    UI::WindowsAndMessaging::*,
};

/// Desktop integration manager for handling app icons and system integration
pub struct DesktopIntegration;

impl DesktopIntegration {
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
        info!("🎨 Setting up application icon for taskbar");
        
        // Create programmatic app icon
        let _icon_theme = gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap());
        
        // Try to set a custom icon
        if let Some(icon_surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(48) {
            // Convert to GdkTexture for GTK4
            let _texture = gtk::gdk::Texture::for_pixbuf(&Self::surface_to_pixbuf(&icon_surface));
            
            // Set as default icon for all windows
            for window in app.windows() {
                if let Some(gtk_window) = window.downcast_ref::<gtk::Window>() {
                    gtk_window.set_icon_name(Some("io.bassi.Amberol"));
                    
                    // Also try to set custom icon directly
                    #[cfg(target_os = "windows")]
                    Self::set_windows_taskbar_icon(gtk_window);
                }
            }
            
            info!("✅ Application icon set for taskbar");
        } else {
            warn!("⚠️ Failed to create app icon surface");
        }
    }
    
    /// Convert Cairo surface to GdkPixbuf
    fn surface_to_pixbuf(surface: &cairo::ImageSurface) -> gtk::gdk_pixbuf::Pixbuf {
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
    fn set_windows_taskbar_icon(window: &gtk::Window) {
        use gtk::prelude::*;
        
        info!("🪟 Setting Windows taskbar icon");
        
        // Get the native window handle
        if let Some(surface) = window.surface() {
            // Try to create and set a custom icon
            if let Some(hicon) = crate::icon_renderer::IconRenderer::create_tray_icon() {
                unsafe {
                    // This would require more complex window handle extraction
                    // For now, we'll rely on the build-time embedded icon
                    info!("🎨 Custom Windows icon created");
                }
            }
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
    pub fn generate_desktop_icons() -> Result<(), Box<dyn std::error::Error>> {
        info!("🖥️ Generating desktop icons");
        
        // Standard icon sizes for desktop environments
        let sizes = [16, 22, 24, 32, 48, 64, 96, 128, 256, 512];
        
        for &size in &sizes {
            if let Some(surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(size) {
                let filename = format!("amberol-{}.png", size);
                surface.write_to_png(&mut std::fs::File::create(&filename)?)?;
                info!("✅ Created desktop icon: {}", filename);
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
}