// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{gdk, glib, prelude::*};
use log::{info, warn};

/// Aggressive icon replacer that scans widget trees and replaces missing icons
pub struct IconReplacer;

impl IconReplacer {
    /// Setup periodic icon replacement scanning
    pub fn setup_periodic_replacement() {
        info!("🔄 Setting up periodic icon replacement scanning");
        
        // Run icon replacement every 2 seconds to catch dynamically created widgets
        glib::timeout_add_seconds_local(2, || {
            Self::scan_and_replace_icons();
            glib::ControlFlow::Continue
        });
        
        // Also run immediately
        Self::scan_and_replace_icons();
    }
    
    /// Scan all windows and replace missing icons
    fn scan_and_replace_icons() {
        info!("🔍 Scanning for missing icons to replace");
        
        // Get all GTK applications
        if let Some(app) = gtk::gio::Application::default() {
            if let Some(gtk_app) = app.downcast_ref::<gtk::Application>() {
                // Scan all windows
                for window in gtk_app.windows() {
                    Self::scan_widget_tree(&window);
                }
            }
        }
        
        // Also scan any top-level windows we can find
        let display = gdk::Display::default().unwrap();
        // Note: GTK4 doesn't provide direct access to all windows, so we rely on the application
    }
    
    /// Recursively scan a widget tree and replace icons
    fn scan_widget_tree(widget: &impl IsA<gtk::Widget>) {
        let widget = widget.as_ref();
        
        // Check if this widget is an Image that might have a missing icon
        if let Some(image) = widget.downcast_ref::<gtk::Image>() {
            Self::replace_image_icon(image);
        }
        
        // Check if this widget is a Button with an icon
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            Self::replace_button_icon(button);
        }
        
        // Recursively scan child widgets
        let mut child = widget.first_child();
        while let Some(current_child) = child {
            Self::scan_widget_tree(&current_child);
            child = current_child.next_sibling();
        }
    }
    
    /// Replace icon in a gtk::Image widget
    fn replace_image_icon(image: &gtk::Image) {
        // Check what kind of image this is
        match image.storage_type() {
            gtk::ImageType::IconName => {
                if let Some(icon_name) = image.icon_name() {
                    if Self::should_replace_icon(&icon_name) {
                        info!("🎨 Replacing image icon: {}", icon_name);
                        Self::set_programmatic_image(image, &icon_name);
                    }
                }
            }
            gtk::ImageType::Gicon => {
                // Handle GIcon case - might be showing image-missing
                if let Some(gicon) = image.gicon() {
                    if let Some(themed_icon) = gicon.downcast_ref::<gtk::gio::ThemedIcon>() {
                        let names = themed_icon.names();
                        for name in names {
                            if Self::should_replace_icon(&name) {
                                info!("🎨 Replacing GIcon: {}", name);
                                Self::set_programmatic_image(image, &name);
                                break;
                            }
                        }
                    }
                }
            }
            _ => {
                // For other image types, we can't easily determine if they're missing
            }
        }
    }
    
    /// Replace icon in a gtk::Button widget
    fn replace_button_icon(button: &gtk::Button) {
        if let Some(icon_name) = button.icon_name() {
            if Self::should_replace_icon(&icon_name) {
                info!("🎨 Replacing button icon: {}", icon_name);
                // Use our programmatic icon renderer
                crate::icon_renderer::IconRenderer::set_button_icon_programmatic(button, &icon_name);
            }
        }
    }
    
    /// Check if an icon should be replaced
    fn should_replace_icon(icon_name: &str) -> bool {
        // List of icons we want to replace
        matches!(icon_name,
            "io.bassi.Amberol" |
            "io.bassi.Amberol.Devel" |
            "web-browser-symbolic" |
            "user-home-symbolic" |
            "document-edit-symbolic" |
            "bug-symbolic" |
            "system-search-symbolic" |
            "open-menu-symbolic" |
            "audio-only-symbolic" |
            "folder-music-symbolic" |
            "image-missing"  // Catch the fallback directly
        )
    }
    
    /// Set a programmatic image for a gtk::Image widget
    fn set_programmatic_image(image: &gtk::Image, icon_name: &str) {
        // Create a paintable for this icon and set it directly
        if let Some(paintable) = Self::create_paintable_for_icon(icon_name) {
            image.set_paintable(Some(&paintable));
            info!("✅ Successfully replaced image with programmatic icon: {}", icon_name);
        } else {
            warn!("⚠️ Failed to create paintable for icon: {}", icon_name);
        }
    }
    
    /// Create a paintable for an icon
    fn create_paintable_for_icon(icon_name: &str) -> Option<gdk::Paintable> {
        // Create a surface using our icon renderer - but we need to draw the specific icon
        if let Some(mut surface) = Self::create_icon_surface_for_name(icon_name, 16) {
            // Convert to pixbuf
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
                
                let texture = gdk::Texture::for_pixbuf(&pixbuf);
                return Some(texture.upcast::<gdk::Paintable>());
            }
        }
        None
    }
    
    /// Create a surface for a specific icon name
    fn create_icon_surface_for_name(icon_name: &str, size: i32) -> Option<gtk::cairo::ImageSurface> {
        use gtk::cairo;
        
        // Create surface
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, size, size).ok()?;
        let cr = cairo::Context::new(&surface).ok()?;
        
        // Set up drawing context
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.0); // Transparent background
        cr.paint().ok()?;
        
        // Set drawing color (use theme-appropriate color)
        cr.set_source_rgba(0.2, 0.2, 0.2, 1.0); // Dark gray for visibility
        cr.set_line_width(1.0);
        
        // Draw the appropriate icon
        let success = match icon_name {
            "io.bassi.Amberol" | "io.bassi.Amberol.Devel" => Self::draw_app_icon(&cr, size),
            "web-browser-symbolic" | "user-home-symbolic" => Self::draw_web_browser(&cr, size),
            "document-edit-symbolic" | "bug-symbolic" => Self::draw_bug(&cr, size),
            "system-search-symbolic" => Self::draw_search(&cr, size),
            "open-menu-symbolic" => Self::draw_menu(&cr, size),
            "audio-only-symbolic" => Self::draw_audio(&cr, size),
            "folder-music-symbolic" => Self::draw_folder(&cr, size),
            "image-missing" => Self::draw_fallback(&cr, size),
            _ => Self::draw_fallback(&cr, size),
        };
        
        if success {
            Some(surface)
        } else {
            None
        }
    }
    
    // Simple drawing functions for different icon types
    fn draw_app_icon(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Musical note
        cr.arc(s * 0.2, s * 0.8, s * 0.1, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or(());
        cr.arc(s * 0.7, s * 0.6, s * 0.08, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or(());
        cr.move_to(s * 0.3, s * 0.8);
        cr.line_to(s * 0.3, s * 0.2);
        cr.line_to(s * 0.78, s * 0.15);
        cr.line_to(s * 0.78, s * 0.6);
        cr.stroke().unwrap_or(());
        true
    }
    
    fn draw_web_browser(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Globe
        cr.arc(s * 0.5, s * 0.5, s * 0.35, 0.0, 2.0 * std::f64::consts::PI);
        cr.stroke().unwrap_or(());
        cr.move_to(s * 0.5, s * 0.15);
        cr.line_to(s * 0.5, s * 0.85);
        cr.stroke().unwrap_or(());
        cr.move_to(s * 0.15, s * 0.5);
        cr.line_to(s * 0.85, s * 0.5);
        cr.stroke().unwrap_or(());
        true
    }
    
    fn draw_bug(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Bug body
        cr.arc(s * 0.5, s * 0.5, s * 0.25, 0.0, 2.0 * std::f64::consts::PI);
        cr.stroke().unwrap_or(());
        // Legs
        for i in 0..3 {
            let y = s * (0.3 + i as f64 * 0.2);
            cr.move_to(s * 0.25, y);
            cr.line_to(s * 0.1, y - s * 0.05);
            cr.move_to(s * 0.75, y);
            cr.line_to(s * 0.9, y - s * 0.05);
            cr.stroke().unwrap_or(());
        }
        true
    }
    
    fn draw_search(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Magnifying glass
        cr.arc(s * 0.4, s * 0.4, s * 0.2, 0.0, 2.0 * std::f64::consts::PI);
        cr.stroke().unwrap_or(());
        cr.move_to(s * 0.55, s * 0.55);
        cr.line_to(s * 0.8, s * 0.8);
        cr.stroke().unwrap_or(());
        true
    }
    
    fn draw_menu(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Hamburger menu
        for i in 0..3 {
            let y = s * (0.3 + i as f64 * 0.2);
            cr.move_to(s * 0.2, y);
            cr.line_to(s * 0.8, y);
            cr.stroke().unwrap_or(());
        }
        true
    }
    
    fn draw_audio(cr: &gtk::cairo::Context, size: i32) -> bool {
        Self::draw_app_icon(cr, size) // Same as app icon (musical note)
    }
    
    fn draw_folder(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Folder
        cr.move_to(s * 0.1, s * 0.3);
        cr.line_to(s * 0.1, s * 0.8);
        cr.line_to(s * 0.9, s * 0.8);
        cr.line_to(s * 0.9, s * 0.4);
        cr.line_to(s * 0.6, s * 0.4);
        cr.line_to(s * 0.5, s * 0.3);
        cr.close_path();
        cr.stroke().unwrap_or(());
        // Music note inside
        cr.arc(s * 0.4, s * 0.6, s * 0.05, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or(());
        true
    }
    
    fn draw_fallback(cr: &gtk::cairo::Context, size: i32) -> bool {
        let s = size as f64;
        // Question mark
        cr.arc(s * 0.5, s * 0.3, s * 0.1, 0.0, std::f64::consts::PI);
        cr.stroke().unwrap_or(());
        cr.arc(s * 0.5, s * 0.7, s * 0.05, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or(());
        true
    }
    
    /// Force replacement of specific widgets by CSS class or ID
    pub fn force_replace_known_widgets() {
        info!("🎯 Force replacing known problematic widgets");
        
        // This is a more targeted approach for widgets we know are problematic
        if let Some(app) = gtk::gio::Application::default() {
            if let Some(gtk_app) = app.downcast_ref::<gtk::Application>() {
                for window in gtk_app.windows() {
                    // Look for about dialogs specifically
                    if window.type_().name() == "AdwAboutWindow" {
                        Self::fix_about_dialog(&window);
                    }
                }
            }
        }
    }
    
    /// Fix icons in about dialog specifically
    fn fix_about_dialog(window: &gtk::Window) {
        info!("🔧 Fixing about dialog icons");
        
        // The about dialog's application icon is set via application_icon property
        // We need to ensure our icon theme has the right icon
        
        // Find all images in the about dialog and replace them
        Self::scan_widget_tree(window);
    }
}