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
    /// Create a programmatically drawn icon as a drawable widget
    pub fn create_icon_widget(icon_name: &str) -> Option<gtk::DrawingArea> {
        info!("🎨 Creating programmatic icon widget: {}", icon_name);
        
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_content_width(ICON_SIZE);
        drawing_area.set_content_height(ICON_SIZE);
        
        let icon_name = icon_name.to_string();
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
            let _success = match icon_name.as_str() {
                "media-playlist-consecutive-symbolic" => Self::draw_consecutive(cr),
                "media-playlist-repeat-symbolic" => Self::draw_repeat_all(cr),
                "media-playlist-repeat-song-symbolic" => Self::draw_repeat_one(cr),
                "media-playlist-shuffle-symbolic" => Self::draw_shuffle(cr),
                "media-playback-start-symbolic" => Self::draw_play(cr),
                "media-playback-pause-symbolic" => Self::draw_pause(cr),
                "media-skip-backward-symbolic" => Self::draw_skip_backward(cr),
                "media-skip-forward-symbolic" => Self::draw_skip_forward(cr),
                "view-queue-symbolic" => Self::draw_queue(cr),
                "view-queue-rtl-symbolic" => Self::draw_queue_rtl(cr),
                _ => false
            };
        });
        
        info!("✅ Successfully created programmatic icon widget: {}", icon_name);
        Some(drawing_area)
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
}