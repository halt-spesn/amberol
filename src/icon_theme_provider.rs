// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{gdk, glib, prelude::*};
use log::{info, warn};

/// Custom icon theme provider that intercepts icon lookups and provides programmatic alternatives
pub struct IconThemeProvider;

impl IconThemeProvider {
    /// Setup the global icon theme override
    pub fn setup_global_override() {
        info!("🎨 Setting up global icon theme override for programmatic icons");
        
        // Get the default display and icon theme
        if let Some(display) = gdk::Display::default() {
            let icon_theme = gtk::IconTheme::for_display(&display);
            
            // Add our custom search path first (highest priority)
            if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
                let custom_icons_dir = temp_dir.join("amberol-icons");
                
                // Create the directory if it doesn't exist
                if !custom_icons_dir.exists() {
                    let _ = std::fs::create_dir_all(&custom_icons_dir);
                }
                
                // Add to icon theme search path
                icon_theme.add_search_path(&custom_icons_dir);
                info!("📁 Added custom icon search path: {:?}", custom_icons_dir);
                
                // Generate programmatic icons on-demand
                Self::generate_missing_icons(&custom_icons_dir);
            }
            
            // Connect to icon theme changed signal to regenerate icons
            icon_theme.connect_changed(|theme| {
                info!("🔄 Icon theme changed, ensuring programmatic icons are available");
                Self::ensure_programmatic_icons_available(theme);
            });
            
            info!("✅ Global icon theme override setup complete");
        } else {
            warn!("⚠️ Could not get default display for icon theme setup");
        }
    }
    
    /// Generate missing icons as SVG files in the custom icons directory
    fn generate_missing_icons(icons_dir: &std::path::Path) {
        let icons_to_generate = [
            "io.bassi.Amberol",
            "io.bassi.Amberol.Devel", 
            "web-browser-symbolic",
            "user-home-symbolic",
            "document-edit-symbolic", 
            "bug-symbolic",
            "system-search-symbolic",
            "open-menu-symbolic",
            "audio-only-symbolic",
            "folder-music-symbolic",
        ];
        
        for icon_name in &icons_to_generate {
            Self::generate_icon_svg(icons_dir, icon_name);
        }
    }
    
    /// Generate a single icon as SVG file
    fn generate_icon_svg(icons_dir: &std::path::Path, icon_name: &str) {
        let svg_content = Self::create_svg_for_icon(icon_name);
        let file_path = icons_dir.join(format!("{}.svg", icon_name));
        
        match std::fs::write(&file_path, svg_content) {
            Ok(_) => {
                info!("🎨 Generated programmatic icon: {} -> {:?}", icon_name, file_path);
            }
            Err(e) => {
                warn!("⚠️ Failed to write icon {}: {}", icon_name, e);
            }
        }
    }
    
    /// Create SVG content for a specific icon
    fn create_svg_for_icon(icon_name: &str) -> String {
        let svg_header = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
<g fill="currentColor" stroke="currentColor" stroke-width="1" fill-rule="evenodd">"#;
        
        let svg_footer = r#"</g>
</svg>"#;
        
        let icon_content = match icon_name {
            "io.bassi.Amberol" | "io.bassi.Amberol.Devel" => {
                // Musical note
                r#"<path d="M5 14 c-1.1 0 -2 -0.9 -2 -2 s0.9 -2 2 -2 s2 0.9 2 2 s-0.9 2 -2 2 z"/>
<path d="M12 11 c-1.1 0 -2 -0.9 -2 -2 s0.9 -2 2 -2 s2 0.9 2 2 s-0.9 2 -2 2 z"/>
<path d="M7 12 L7 4 L12 2 L12 9" stroke-width="1.5" fill="none"/>"#
            }
            "web-browser-symbolic" | "user-home-symbolic" => {
                // Globe
                r#"<circle cx="8" cy="8" r="6" fill="none" stroke-width="1"/>
<path d="M8 2 L8 14" stroke-width="0.8"/>
<path d="M2 8 L14 8" stroke-width="0.8"/>
<path d="M5 3.5 Q8 5 8 8 Q8 11 11 12.5" fill="none" stroke-width="0.6"/>
<path d="M11 3.5 Q8 5 8 8 Q8 11 5 12.5" fill="none" stroke-width="0.6"/>
<path d="M3 5.5 Q6 6 10 6 Q13 5.5 13 5.5" fill="none" stroke-width="0.6"/>
<path d="M3 10.5 Q6 10 10 10 Q13 10.5 13 10.5" fill="none" stroke-width="0.6"/>"#
            }
            "document-edit-symbolic" | "bug-symbolic" => {
                // Bug
                r#"<ellipse cx="8" cy="8.5" rx="4" ry="4.5"/>
<line x1="6.5" y1="4" x2="5.5" y2="2" stroke-width="1"/>
<line x1="9.5" y1="4" x2="10.5" y2="2" stroke-width="1"/>
<line x1="4" y1="6" x2="2" y2="5" stroke-width="1"/>
<line x1="4" y1="8.5" x2="2" y2="8.5" stroke-width="1"/>
<line x1="4" y1="11" x2="2" y2="12" stroke-width="1"/>
<line x1="12" y1="6" x2="14" y2="5" stroke-width="1"/>
<line x1="12" y1="8.5" x2="14" y2="8.5" stroke-width="1"/>
<line x1="12" y1="11" x2="14" y2="12" stroke-width="1"/>"#
            }
            "system-search-symbolic" => {
                // Magnifying glass
                r#"<circle cx="6" cy="6" r="4" fill="none" stroke-width="1.5"/>
<line x1="9" y1="9" x2="13" y2="13" stroke-width="2"/>"#
            }
            "open-menu-symbolic" => {
                // Hamburger menu
                r#"<line x1="3" y1="5" x2="13" y2="5" stroke-width="1.5"/>
<line x1="3" y1="8" x2="13" y2="8" stroke-width="1.5"/>
<line x1="3" y1="11" x2="13" y2="11" stroke-width="1.5"/>"#
            }
            "audio-only-symbolic" => {
                // Music note
                r#"<path d="M6 13 c-1 0 -1.5 -0.5 -1.5 -1.5 s0.5 -1.5 1.5 -1.5 s1.5 0.5 1.5 1.5 s-0.5 1.5 -1.5 1.5 z"/>
<path d="M7.5 11.5 L7.5 5 L11 4 L11 8.5" stroke-width="1.2" fill="none"/>
<path d="M11 10 c-0.8 0 -1.2 -0.4 -1.2 -1.2 s0.4 -1.2 1.2 -1.2 s1.2 0.4 1.2 1.2 s-0.4 1.2 -1.2 1.2 z"/>"#
            }
            "folder-music-symbolic" => {
                // Folder with music note
                r#"<path d="M2 3 L2 13 L14 13 L14 5 L8 5 L6 3 Z" fill="none" stroke-width="1"/>
<path d="M6 10 c-0.5 0 -1 -0.5 -1 -1 s0.5 -1 1 -1 s1 0.5 1 1 s-0.5 1 -1 1 z"/>
<path d="M7 9 L7 6.5 L9.5 6 L9.5 8" stroke-width="0.8" fill="none"/>"#
            }
            _ => {
                // Default fallback
                r#"<text x="8" y="12" text-anchor="middle" font-size="12" font-family="monospace">?</text>"#
            }
        };
        
        format!("{}{}{}", svg_header, icon_content, svg_footer)
    }
    
    /// Ensure programmatic icons are available when theme changes
    fn ensure_programmatic_icons_available(icon_theme: &gtk::IconTheme) {
        if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
            let custom_icons_dir = temp_dir.join("amberol-icons");
            
            if !custom_icons_dir.exists() {
                let _ = std::fs::create_dir_all(&custom_icons_dir);
                Self::generate_missing_icons(&custom_icons_dir);
                
                // Add to search path if not already added
                icon_theme.add_search_path(&custom_icons_dir);
            }
        }
    }
}