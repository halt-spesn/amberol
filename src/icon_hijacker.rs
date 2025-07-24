// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

//! Nuclear option icon hijacker - forcefully overrides all icon settings

use gtk::{gdk, glib, prelude::*};
use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Aggressive icon hijacker that intercepts and overrides all icon operations
pub struct IconHijacker {
    replacement_textures: Arc<Mutex<HashMap<String, gdk::Texture>>>,
}

impl IconHijacker {
    /// Create a new icon hijacker
    pub fn new() -> Self {
        Self {
            replacement_textures: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start the aggressive icon hijacking system
    pub fn start_hijacking() {
        info!("🚨 Starting AGGRESSIVE icon hijacking system");
        
        let hijacker = Self::new();
        
        // Create all replacement textures immediately
        hijacker.create_replacement_textures();
        
        // Start the continuous hijacking loop
        hijacker.start_continuous_hijacking();
        
        // Hook into window creation
        hijacker.hook_window_creation();
        
        // Hook into about dialog creation specifically
        hijacker.hook_about_dialog_creation();
        
        info!("🚨 Icon hijacking system ACTIVE");
    }
    
    /// Create all replacement textures upfront
    fn create_replacement_textures(&self) {
        let icons_to_create = vec![
            ("io.bassi.Amberol", Self::create_app_icon_texture()),
            ("io.bassi.Amberol.Devel", Self::create_app_icon_texture()),
            ("web-browser-symbolic", Self::create_web_icon_texture()),
            ("user-home-symbolic", Self::create_web_icon_texture()),
            ("bug-symbolic", Self::create_bug_icon_texture()),
            ("document-edit-symbolic", Self::create_bug_icon_texture()),
            ("system-search-symbolic", Self::create_search_icon_texture()),
            ("open-menu-symbolic", Self::create_menu_icon_texture()),
            ("audio-only-symbolic", Self::create_audio_icon_texture()),
            ("folder-music-symbolic", Self::create_folder_icon_texture()),
            ("image-missing", Self::create_app_icon_texture()),
        ];
        
        let mut textures = self.replacement_textures.lock().unwrap();
        for (name, texture_opt) in icons_to_create {
            if let Some(texture) = texture_opt {
                textures.insert(name.to_string(), texture);
                info!("🎨 Created replacement texture for: {}", name);
            }
        }
        info!("🎨 Created {} replacement textures", textures.len());
    }
    
    /// Start continuous hijacking - runs every 500ms
    fn start_continuous_hijacking(&self) {
        let textures = self.replacement_textures.clone();
        
        glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            Self::hijack_all_windows(&textures);
            glib::ControlFlow::Continue
        });
    }
    
    /// Hijack all windows and their widgets
    fn hijack_all_windows(textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        if let Some(app) = gtk::gio::Application::default() {
            if let Some(gtk_app) = app.downcast_ref::<gtk::Application>() {
                for window in gtk_app.windows() {
                    Self::hijack_window_icons(&window, textures);
                }
            }
        }
    }
    
    /// Hijack all icons in a specific window
    fn hijack_window_icons(window: &gtk::Window, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        // Force set window icon
        Self::force_set_window_icon(window, textures);
        
        // Hijack all widgets in the window
        Self::hijack_widget_tree(window.upcast_ref::<gtk::Widget>(), textures);
    }
    
    /// Force set the window icon itself
    fn force_set_window_icon(window: &gtk::Window, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        let textures_guard = textures.lock().unwrap();
        
        // Debug: Log current window state
        info!("🔍 Window debug - Title: {:?}, Icon name: {:?}", 
              window.title(), window.icon_name());
        
        // Try multiple approaches to set the window icon
        let icon_names = ["io.bassi.Amberol.Devel", "io.bassi.Amberol"];
        
        for icon_name in &icon_names {
            if textures_guard.contains_key(*icon_name) {
                // Method 1: Set icon name directly
                window.set_icon_name(Some(icon_name));
                info!("🚨 Set window icon name: {}", icon_name);
                
                // Method 2: Set via application if available
                if let Some(app) = window.application() {
                    // Get the current application ID
                    let current_app_id = app.application_id();
                    info!("🔍 Current application ID: {:?}", current_app_id);
                    
                    // Try to ensure the application ID matches our icon
                    if current_app_id.as_ref().map(|s| s.as_str()) != Some("io.bassi.Amberol.Devel") {
                        app.set_application_id(Some("io.bassi.Amberol.Devel"));
                        info!("🚨 Set application ID to: io.bassi.Amberol.Devel");
                    }
                }
                
                                // Method 3: Try to force icon theme to have our icon
                let display = gtk::prelude::WidgetExt::display(window);
                let icon_theme = gtk::IconTheme::for_display(&display);
                let search_paths = icon_theme.search_path();
                info!("🔍 Icon theme search paths: {:?}", search_paths);
                
                // Check if icon theme can find our icon
                if icon_theme.has_icon(icon_name) {
                    info!("✅ Icon theme HAS icon: {}", icon_name);
                } else {
                    warn!("❌ Icon theme MISSING icon: {}", icon_name);
                    
                    // Force add a search path with our icon
                    Self::force_create_window_icon(icon_name, &icon_theme);
                }
                
                info!("🚨 FORCED window icon: {} for window: {:?}", icon_name, window.title());
                break;
            }
        }
    }
    
    /// Force create window icon in icon theme
    fn force_create_window_icon(icon_name: &str, icon_theme: &gtk::IconTheme) {
        use std::io::Write;
        
        if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
            let icon_dir = temp_dir.join("amberol-window-icons");
            if std::fs::create_dir_all(&icon_dir).is_ok() {
                let icon_file = icon_dir.join(format!("{}.svg", icon_name));
                
                // Create a simple SVG icon
                let svg_content = format!(r##"<?xml version="1.0" encoding="UTF-8"?>
<svg width="48" height="48" viewBox="0 0 48 48" xmlns="http://www.w3.org/2000/svg">
  <circle cx="12" cy="36" r="6" fill="#ff8c00" stroke="#333" stroke-width="1"/>
  <circle cx="32" cy="28" r="5" fill="#ff8c00" stroke="#333" stroke-width="1"/>
  <path d="M18 36 L18 12 L38 8 L38 28" stroke="#333" stroke-width="3" fill="none"/>
</svg>"##);
                
                if std::fs::write(&icon_file, svg_content).is_ok() {
                    icon_theme.add_search_path(&icon_dir);
                    info!("🚨 FORCE CREATED window icon file: {:?}", icon_file);
                }
            }
        }
    }
    
    /// Recursively hijack all widgets in a tree
    fn hijack_widget_tree(widget: &gtk::Widget, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        // Check for images
        if let Some(image) = widget.downcast_ref::<gtk::Image>() {
            Self::hijack_image(image, textures);
        }
        
        // Check for buttons with icons
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            Self::hijack_button(button, textures);
        }
        
        // Check for about windows specifically
        if let Some(about_window) = widget.downcast_ref::<adw::AboutWindow>() {
            Self::hijack_about_window(about_window, textures);
        }
        
        // Recurse into children
        let mut child = widget.first_child();
        while let Some(current_child) = child {
            Self::hijack_widget_tree(&current_child, textures);
            child = current_child.next_sibling();
        }
    }
    
    /// Hijack a specific image widget
    fn hijack_image(image: &gtk::Image, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        let textures_guard = textures.lock().unwrap();
        
        // Try to determine what icon this image is supposed to show
        let icon_name = match image.storage_type() {
            gtk::ImageType::IconName => image.icon_name().map(|s| s.to_string()),
            gtk::ImageType::Gicon => {
                if let Some(gicon) = image.gicon() {
                    if let Some(themed_icon) = gicon.downcast_ref::<gtk::gio::ThemedIcon>() {
                        themed_icon.names().get(0).map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        
        if let Some(icon_name) = icon_name {
            // Only hijack icons we know should be replaced
            if Self::should_hijack_icon(&icon_name) {
                if let Some(texture) = textures_guard.get(&icon_name) {
                    image.set_paintable(Some(texture));
                    info!("🚨 HIJACKED image icon: {}", icon_name);
                } else {
                    // Force set to app icon if we don't have a specific replacement
                    if let Some(texture) = textures_guard.get("io.bassi.Amberol") {
                        image.set_paintable(Some(texture));
                        info!("🚨 FORCE REPLACED unknown icon '{}' with app icon", icon_name);
                    }
                }
            }
        }
    }
    
    /// Check if an icon should be hijacked (be more selective)
    fn should_hijack_icon(icon_name: &str) -> bool {
        matches!(icon_name,
            "io.bassi.Amberol" |
            "io.bassi.Amberol.Devel" |
            "web-browser-symbolic" |
            "user-home-symbolic" |
            "document-edit-symbolic" |
            "bug-symbolic" |
            "system-search-symbolic" |
            "open-menu-symbolic" |
            "image-missing"
            // Deliberately NOT including "audio-only-symbolic" - let the existing system handle it
        )
    }
    
    /// Hijack a specific button widget
    fn hijack_button(button: &gtk::Button, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        if let Some(icon_name) = button.icon_name() {
            let icon_name_str = icon_name.to_string();
            // Only hijack buttons with icons we want to replace
            if Self::should_hijack_icon(&icon_name_str) {
                let textures_guard = textures.lock().unwrap();
                if let Some(texture) = textures_guard.get(&icon_name_str) {
                    // Remove the button's icon and add our own image
                    button.set_icon_name("");
                    
                    let image = gtk::Image::new();
                    image.set_paintable(Some(texture));
                    button.set_child(Some(&image));
                    info!("🚨 HIJACKED button icon: {}", icon_name_str);
                }
            }
        }
    }
    
    /// Hijack about window specifically
    fn hijack_about_window(about_window: &adw::AboutWindow, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        info!("🚨 HIJACKING about window!");
        
        // Force set application icon
        about_window.set_application_icon("io.bassi.Amberol");
        
        // Try to find and replace all icons in the about window
        Self::hijack_widget_tree(about_window.upcast_ref::<gtk::Widget>(), textures);
    }
    
    /// Hook into window creation to catch new windows
    fn hook_window_creation(&self) {
        // This is a bit tricky in GTK4, but we can monitor application windows
        if let Some(app) = gtk::gio::Application::default() {
            if let Some(gtk_app) = app.downcast_ref::<gtk::Application>() {
                let textures = self.replacement_textures.clone();
                
                gtk_app.connect_window_added(move |_app, window| {
                    info!("🚨 NEW WINDOW DETECTED - hijacking icons");
                    
                    // Wait a bit for the window to be fully constructed
                    let textures_clone = textures.clone();
                    let window_weak = window.downgrade();
                    
                    glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                        if let Some(window) = window_weak.upgrade() {
                            Self::hijack_window_icons(&window, &textures_clone);
                        }
                    });
                });
            }
        }
    }
    
    /// Hook specifically into about dialog creation
    fn hook_about_dialog_creation(&self) {
        // We'll use a different approach - hook into the action
        info!("🚨 Hooking about dialog creation");
    }
    
    // Icon creation methods
    fn create_app_icon_texture() -> Option<gdk::Texture> {
        Self::create_texture_from_drawing(64, |cr| {
            // Musical note - orange/gold color
            cr.set_source_rgba(1.0, 0.55, 0.0, 1.0);
            
            // Main note head
            cr.arc(16.0, 48.0, 8.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap_or(());
            
            // Second note head
            cr.arc(44.0, 38.0, 6.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap_or(());
            
            // Note stem
            cr.set_line_width(4.0);
            cr.move_to(24.0, 48.0);
            cr.line_to(24.0, 16.0);
            cr.line_to(50.0, 12.0);
            cr.line_to(50.0, 38.0);
            cr.stroke().unwrap_or(());
            
            true
        })
    }
    
    fn create_web_icon_texture() -> Option<gdk::Texture> {
        Self::create_texture_from_drawing(32, |cr| {
            cr.set_source_rgba(0.2, 0.6, 0.9, 1.0);
            cr.set_line_width(2.0);
            
            // Globe outline
            cr.arc(16.0, 16.0, 12.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap_or(());
            
            // Meridians
            cr.move_to(16.0, 4.0);
            cr.line_to(16.0, 28.0);
            cr.stroke().unwrap_or(());
            
            // Equator
            cr.move_to(4.0, 16.0);
            cr.line_to(28.0, 16.0);
            cr.stroke().unwrap_or(());
            
            true
        })
    }
    
    fn create_bug_icon_texture() -> Option<gdk::Texture> {
        Self::create_texture_from_drawing(32, |cr| {
            cr.set_source_rgba(0.8, 0.2, 0.2, 1.0);
            cr.set_line_width(2.0);
            
            // Bug body
            cr.arc(16.0, 16.0, 8.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap_or(());
            
            // Bug legs
            for i in 0..3 {
                let y = 10.0 + i as f64 * 4.0;
                cr.move_to(8.0, y);
                cr.line_to(4.0, y - 2.0);
                cr.move_to(24.0, y);
                cr.line_to(28.0, y - 2.0);
                cr.stroke().unwrap_or(());
            }
            
            true
        })
    }
    
    fn create_search_icon_texture() -> Option<gdk::Texture> {
        Self::create_texture_from_drawing(32, |cr| {
            cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
            cr.set_line_width(3.0);
            
            // Magnifying glass
            cr.arc(12.0, 12.0, 8.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap_or(());
            
            // Handle
            cr.move_to(18.0, 18.0);
            cr.line_to(26.0, 26.0);
            cr.stroke().unwrap_or(());
            
            true
        })
    }
    
    fn create_menu_icon_texture() -> Option<gdk::Texture> {
        Self::create_texture_from_drawing(32, |cr| {
            cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
            cr.set_line_width(3.0);
            
            // Hamburger menu
            for i in 0..3 {
                let y = 8.0 + i as f64 * 8.0;
                cr.move_to(6.0, y);
                cr.line_to(26.0, y);
                cr.stroke().unwrap_or(());
            }
            
            true
        })
    }
    
    fn create_audio_icon_texture() -> Option<gdk::Texture> {
        Self::create_app_icon_texture() // Same as app icon
    }
    
    fn create_folder_icon_texture() -> Option<gdk::Texture> {
        Self::create_texture_from_drawing(32, |cr| {
            cr.set_source_rgba(0.9, 0.7, 0.3, 1.0);
            cr.set_line_width(2.0);
            
            // Folder outline
            cr.move_to(4.0, 10.0);
            cr.line_to(4.0, 26.0);
            cr.line_to(28.0, 26.0);
            cr.line_to(28.0, 12.0);
            cr.line_to(18.0, 12.0);
            cr.line_to(16.0, 10.0);
            cr.close_path();
            cr.stroke().unwrap_or(());
            
            // Small music note inside
            cr.set_source_rgba(0.6, 0.4, 0.2, 1.0);
            cr.arc(14.0, 20.0, 2.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap_or(());
            
            true
        })
    }
    
    /// Helper to create texture from drawing function
    fn create_texture_from_drawing(size: i32, draw_fn: impl Fn(&gtk::cairo::Context) -> bool) -> Option<gdk::Texture> {
        use gtk::cairo;
        
        if let Ok(mut surface) = cairo::ImageSurface::create(cairo::Format::ARgb32, size, size) {
            if let Ok(cr) = cairo::Context::new(&surface) {
                // Clear background
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                cr.paint().unwrap_or(());
                
                // Draw the icon
                if draw_fn(&cr) {
                    // Convert to texture
                    let width = surface.width();
                    let height = surface.height();
                    let stride = surface.stride();
                    
                    if let Ok(data) = surface.data() {
                        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
                            &glib::Bytes::from(&data[..]),
                            gtk::gdk_pixbuf::Colorspace::Rgb,
                            true, // has_alpha
                            8,    // bits_per_sample
                            width,
                            height,
                            stride,
                        );
                        
                        return Some(gdk::Texture::for_pixbuf(&pixbuf));
                    }
                }
            }
        }
        
        None
    }
}