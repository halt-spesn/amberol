// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{glib, prelude::*};
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
        // Create a programmatic icon widget
        if let Some(icon_widget) = crate::icon_renderer::IconRenderer::create_programmatic_icon(icon_name, 16) {
            // We can't directly replace the image content, but we can try to replace the parent's child
            if let Some(parent) = image.parent() {
                // Try to replace with a drawing area if possible
                // This is tricky because we need to maintain the same layout properties
                
                // For now, let's try setting the image from a paintable
                if let Some(paintable) = Self::create_paintable_for_icon(icon_name) {
                    image.set_from_paintable(Some(&paintable));
                }
            }
        }
    }
    
    /// Create a paintable for an icon
    fn create_paintable_for_icon(icon_name: &str) -> Option<gdk::Paintable> {
        // Create a surface using our icon renderer
        if let Some(mut surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(16) {
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