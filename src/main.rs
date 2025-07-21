// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

mod application;
mod audio;
mod config;
mod cover_picture;
mod drag_overlay;
mod i18n;
mod playback_control;
mod playlist_view;
mod queue_row;
mod search;
mod song_cover;
mod song_details;
mod sort;
mod utils;
mod volume_control;
mod waveform_view;
mod window;
#[cfg(target_os = "windows")]
mod windows;

use std::env;

use config::{APPLICATION_ID, GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR, PROFILE};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, setlocale, textdomain, LocaleCategory};
use gtk::{gio, glib, prelude::*};
use log::{debug, error, info, LevelFilter};

use self::application::Application;

fn main() -> glib::ExitCode {
    let mut builder = pretty_env_logger::formatted_builder();
    if APPLICATION_ID.ends_with("Devel") {
        builder.filter(Some("amberol"), LevelFilter::Debug);
    } else {
        builder.filter(Some("amberol"), LevelFilter::Info);
    }
    builder.init();

    // Set up gettext translations
    debug!("Setting up locale data");
    setlocale(LocaleCategory::LcAll, "");

    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Platform-specific setup
    #[cfg(not(target_os = "windows"))]
    setup_pulseaudio();

    #[cfg(target_os = "windows")]
    setup_windows_audio();

    debug!("Loading resources");
    
    // Try multiple locations for the GResource file for portable compatibility
    let mut resource_locations = Vec::new();
    
    // Add portable/relative locations first (for Windows portable builds)
    if let Ok(exe_path) = env::current_exe() {
        let exe_dir = exe_path.parent().unwrap();
        
        // Try in the same directory as the executable
        resource_locations.push(exe_dir.join("amberol.gresource"));
        
        // Try in ../share/ relative to executable (portable structure)
        resource_locations.push(exe_dir.parent().unwrap_or(exe_dir).join("share").join("amberol.gresource"));
        
        // Try in ../share/amberol/ relative to executable
        resource_locations.push(exe_dir.parent().unwrap_or(exe_dir).join("share").join("amberol").join("amberol.gresource"));
    }
    
    // Add system locations (for installed builds)
    resource_locations.push(std::path::PathBuf::from(PKGDATADIR.to_owned() + "/amberol.gresource"));
    
    // For development builds
    if env::var("MESON_DEVENV").is_ok() {
        if let Ok(exe_path) = env::current_exe() {
            let exe_dir = exe_path.parent().unwrap();
            resource_locations.insert(0, exe_dir.join("amberol.gresource"));
        }
    }
    
    let mut resources = None;
    let mut last_error = None;
    
    for location in &resource_locations {
        debug!("Trying to load resources from: {:?}", location);
        match gio::Resource::load(location) {
            Ok(res) => {
                info!("Successfully loaded resources from: {:?}", location);
                resources = Some(res);
                break;
            }
            Err(err) => {
                debug!("Failed to load from {:?}: {}", location, err);
                last_error = Some(err);
            }
        }
    }
    
    let resources = resources.unwrap_or_else(|| {
        eprintln!("Unable to find amberol.gresource in any of these locations:");
        for location in &resource_locations {
            eprintln!("  - {:?}", location);
        }
        if let Some(err) = last_error {
            panic!("Last error: {}", err);
        } else {
            panic!("No resource locations were tried");
        }
    });
    gio::resources_register(&resources);

    debug!("Setting up application (profile: {})", &PROFILE);
    glib::set_application_name("Amberol");
    glib::set_program_name(Some("amberol"));

    gst::init().expect("Failed to initialize gstreamer");

    let ctx = glib::MainContext::default();
    let _guard = ctx.acquire().unwrap();

    Application::new().run()
}

#[cfg(not(target_os = "windows"))]
fn setup_pulseaudio() {
    debug!("Setting up pulseaudio environment");
    let app_id = APPLICATION_ID.trim_end_matches(".Devel");
    env::set_var("PULSE_PROP_application.icon_name", app_id);
    env::set_var("PULSE_PROP_application.name", "Amberol");
    env::set_var("PULSE_PROP_media.role", "music");
}

#[cfg(target_os = "windows")]
fn setup_windows_audio() {
    debug!("Setting up Windows audio environment");
    // Set GStreamer to use Windows audio sink
    env::set_var("GSK_RENDERER", "gl");
    
    // Register file associations if we have permissions
    if let Err(e) = windows::register_file_associations() {
        debug!("Could not register file associations: {}", e);
    }
    
    debug!("Windows audio setup complete");
}
