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
        
        // Try multiple icon names
        for icon_name in &["io.bassi.Amberol", "io.bassi.Amberol.Devel"] {
            if let Some(_texture) = textures_guard.get(*icon_name) {
                window.set_icon_name(Some(icon_name));
                
                // Also try to set via application
                if let Some(app) = window.application() {
                    app.set_application_id(Some("io.bassi.Amberol"));
                }
                
                info!("🚨 FORCED window icon: {} for window: {:?}", icon_name, window.title());
                break;
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
            gtk::ImageType::IconName => image.icon_name(),
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
    
    /// Hijack a specific button widget
    fn hijack_button(button: &gtk::Button, textures: &Arc<Mutex<HashMap<String, gdk::Texture>>>) {
        if let Some(icon_name) = button.icon_name() {
            let textures_guard = textures.lock().unwrap();
            if textures_guard.contains_key(&icon_name) {
                // Remove the button's icon and add our own image
                button.set_icon_name(None);
                
                if let Some(texture) = textures_guard.get(&icon_name) {
                    let image = gtk::Image::new();
                    image.set_paintable(Some(texture));
                    button.set_child(Some(&image));
                    info!("🚨 HIJACKED button icon: {}", icon_name);
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
        
        if let Ok(surface) = cairo::ImageSurface::create(cairo::Format::ARgb32, size, size) {
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