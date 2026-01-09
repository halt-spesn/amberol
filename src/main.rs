// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

// Hide console window on Windows - must be at the very top
#![windows_subsystem = "windows"]

// Run console hiding code BEFORE main() using CRT initialization
#[cfg(target_os = "windows")]
#[used]
#[link_section = ".CRT$XCU"]
static HIDE_CONSOLE: unsafe extern "C" fn() = {
    unsafe extern "C" fn hide_console_early() {
        use winapi::um::wincon::{FreeConsole, GetConsoleWindow};
        use winapi::um::winuser::{ShowWindow, SW_HIDE};
        let console_window = GetConsoleWindow();
        if !console_window.is_null() {
            ShowWindow(console_window, SW_HIDE);
        }
        FreeConsole();
    }
    hide_console_early
};

mod application;
mod audio;
mod config;
mod cover_picture;
mod desktop_integration;
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
mod system_tray;
mod icon_renderer;

use std::env;

use config::{APPLICATION_ID, GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR, PROFILE};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, setlocale, textdomain, LocaleCategory};
use gtk::{gio, glib, prelude::*};
use log::{debug, error, info, LevelFilter};

use self::application::Application;

fn main() -> glib::ExitCode {
    // On Windows, set up portable environment BEFORE anything else
    // (Console hiding is done earlier via CRT initialization hook)
    #[cfg(target_os = "windows")]
    setup_windows_portable_environment();
    
    let mut builder = pretty_env_logger::formatted_builder();
    if APPLICATION_ID.ends_with("Devel") {
        builder.filter(Some("amberol"), LevelFilter::Debug);
    } else {
        builder.filter(Some("amberol"), LevelFilter::Info);
    }
    builder.init();

    // Set up gettext translations
    debug!("Setting up locale data");
    
    // Set locale - on Windows with MinGW/MSYS2, we need to use POSIX-style locale names
    // But Windows C runtime doesn't fully support them, so we rely on LANGUAGE env var
    #[cfg(target_os = "windows")]
    {
        // Try empty string first to initialize from environment
        let result = setlocale(LocaleCategory::LcAll, "");
        if result.is_none() {
            // If that fails, try "C" locale as fallback and let LANGUAGE env var handle the rest
            setlocale(LocaleCategory::LcAll, "C");
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    setlocale(LocaleCategory::LcAll, "");

    // For portable builds, try to find locale directory in this order:
    // 1. LOCALEDIR environment variable (set by launcher script)
    // 2. Relative to executable (portable builds)
    // 3. Compiled-in LOCALEDIR (system install)
    let locale_dir = {
        // First check environment variable
        if let Ok(env_locale) = env::var("LOCALEDIR") {
            let path = std::path::PathBuf::from(&env_locale);
            if path.exists() && path.is_dir() {
                info!("Using LOCALEDIR from environment: {}", env_locale);
                // Convert to forward slashes for libintl compatibility on Windows
                #[cfg(target_os = "windows")]
                let locale_str = path.to_string_lossy().replace('\\', "/");
                #[cfg(not(target_os = "windows"))]
                let locale_str = path.to_string_lossy().to_string();
                Some(locale_str)
            } else {
                None
            }
        } else {
            None
        }
        .or_else(|| {
            // Try portable locations (relative to exe)
            if let Ok(exe_path) = env::current_exe() {
                let exe_dir = exe_path.parent().unwrap();
                
                let portable_locations = [
                    exe_dir.parent().unwrap_or(exe_dir).join("share").join("locale"),
                    exe_dir.join("..").join("share").join("locale"),
                ];
                
                for location in &portable_locations {
                    if location.exists() && location.is_dir() {
                        info!("Found portable locale directory: {:?}", location);
                        // Convert to forward slashes for libintl compatibility on Windows
                        #[cfg(target_os = "windows")]
                        let locale_str = location.to_string_lossy().replace('\\', "/");
                        #[cfg(not(target_os = "windows"))]
                        let locale_str = location.to_string_lossy().to_string();
                        return Some(locale_str);
                    }
                }
            }
            None
        })
        // Fall back to compiled-in LOCALEDIR
        .unwrap_or_else(|| LOCALEDIR.to_string())
    };
    
    info!("Using locale directory: {}", locale_dir);
    
    bindtextdomain(GETTEXT_PACKAGE, &locale_dir).expect("Failed to bind text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").expect("Failed to set text domain codeset");
    textdomain(GETTEXT_PACKAGE).expect("Failed to set text domain");

    // Platform-specific setup
    #[cfg(not(target_os = "windows"))]
    setup_pulseaudio();

    #[cfg(target_os = "windows")]
    setup_windows_audio();

    debug!("Loading resources");
    info!("🎯 Amberol Resource Loading Debug");
    info!("Application ID: {}", APPLICATION_ID);
    info!("Profile: {}", PROFILE);
    info!("Package Data Dir: {}", PKGDATADIR);
    
    // Try multiple locations for the GResource file for portable compatibility
    let mut resource_locations = Vec::new();
    
    // Add portable/relative locations first (for Windows portable builds)
    if let Ok(exe_path) = env::current_exe() {
        let exe_dir = exe_path.parent().unwrap();
        info!("📁 Executable directory: {:?}", exe_dir);
        
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
    
    info!("🔍 Will try {} resource locations:", resource_locations.len());
    for (i, location) in resource_locations.iter().enumerate() {
        info!("  {}. {:?}", i + 1, location);
    }
    
    let mut resources = None;
    let mut last_error = None;
    
    for (i, location) in resource_locations.iter().enumerate() {
        info!("📍 Attempt {}/{}: {:?}", i + 1, resource_locations.len(), location);
        
        // Check if file exists first
        if location.exists() {
            info!("  ✅ File exists ({} bytes)", location.metadata().map(|m| m.len()).unwrap_or(0));
        } else {
            info!("  ❌ File not found");
            continue;
        }
        
        match gio::Resource::load(location) {
            Ok(res) => {
                info!("  🎉 Successfully loaded GResource!");
                
                // Debug: List some resources to verify icon loading
                let resource_paths = res.enumerate_children("/io/bassi/Amberol/icons/scalable/actions/", gio::ResourceLookupFlags::NONE);
                match resource_paths {
                    Ok(paths) => {
                        info!("  📋 Found {} icon resources:", paths.len());
                        for (j, path) in paths.iter().take(5).enumerate() {
                            info!("    {}. {}", j + 1, path);
                        }
                        if paths.len() > 5 {
                            info!("    ... and {} more", paths.len() - 5);
                        }
                    }
                    Err(e) => {
                        error!("  ⚠️ Could not enumerate icon resources: {}", e);
                    }
                }
                
                resources = Some(res);
                break;
            }
            Err(err) => {
                error!("  💥 Failed to load GResource: {}", err);
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
fn detect_windows_language() -> Option<String> {
    // Use winapi crate for locale detection
    use winapi::um::winnls::GetUserDefaultUILanguage;
    
    unsafe {
        // Get the user's default UI language
        let langid = GetUserDefaultUILanguage();
        
        // Extract primary language ID (lower 10 bits)
        let primary_lang = langid & 0x3FF;
        // Extract sublanguage ID (upper 6 bits)
        let sub_lang = (langid >> 10) & 0x3F;
        
        info!("Windows LANGID: 0x{:04X} (primary: 0x{:02X}, sub: 0x{:02X})", langid, primary_lang, sub_lang);
        
        // Map primary language IDs to language codes
        // Reference: https://docs.microsoft.com/en-us/windows/win32/intl/language-identifiers
        let lang_code = match primary_lang {
            0x22 => "uk",       // Ukrainian
            0x19 => "ru",       // Russian
            0x09 => "en",       // English
            0x07 => "de",       // German
            0x0C => "fr",       // French
            0x10 => "it",       // Italian
            0x0A => "es",       // Spanish
            0x15 => "pl",       // Polish
            0x04 => {           // Chinese
                if sub_lang == 0x02 { "zh_CN" } else { "zh_TW" }
            },
            0x11 => "ja",       // Japanese
            0x12 => "ko",       // Korean
            0x16 => {           // Portuguese
                if sub_lang == 0x01 { "pt_BR" } else { "pt" }
            },
            0x1D => "sv",       // Swedish
            0x13 => "nl",       // Dutch
            0x05 => "cs",       // Czech
            0x0E => "hu",       // Hungarian
            0x08 => "el",       // Greek
            0x1F => "tr",       // Turkish
            0x14 => "nb",       // Norwegian Bokmål
            0x0B => "fi",       // Finnish
            0x06 => "da",       // Danish
            0x18 => "ro",       // Romanian
            0x02 => "bg",       // Bulgarian
            0x1A => "hr",       // Croatian/Serbian
            0x24 => "sl",       // Slovenian
            0x27 => "lt",       // Lithuanian
            0x0D => "he",       // Hebrew
            0x29 => "fa",       // Persian/Farsi
            0x39 => "hi",       // Hindi
            0x03 => "ca",       // Catalan
            0x2D => "eu",       // Basque
            0x56 => "gl",       // Galician
            0x21 => "id",       // Indonesian
            0x0F => "is",       // Icelandic
            0x23 => "be",       // Belarusian
            0x36 => "af",       // Afrikaans
            0x57 => "oc",       // Occitan (not standard LANGID but mapped)
            0x78 => "ia",       // Interlingua (mapped)
            _ => return None,
        };
        
        Some(lang_code.to_string())
    }
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

/// Set up all environment variables for portable Windows build
/// This must be called BEFORE any GTK/GLib initialization
#[cfg(target_os = "windows")]
fn setup_windows_portable_environment() {
    use std::path::PathBuf;
    
    // Get the executable's directory
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return, // Can't determine exe path, skip setup
    };
    
    let exe_dir = match exe_path.parent() {
        Some(dir) => dir,
        None => return,
    };
    
    // Determine the base directory (parent of bin/)
    // Structure: base/bin/amberol.exe -> base is exe_dir.parent()
    let base_dir = exe_dir.parent().unwrap_or(exe_dir);
    
    // Check if we're in a portable structure (has share/ directory)
    let share_dir = base_dir.join("share");
    if !share_dir.exists() {
        return; // Not a portable build, use system paths
    }
    
    let lib_dir = base_dir.join("lib");
    
    // Use glib::setenv for C-level environment variables (needed for libintl)
    // This ensures the C library sees the environment changes
    let set_env_glib = |key: &str, value: &str| {
        // Set at both C level (glib) and Rust level
        glib::setenv(key, value, false).ok();
        if env::var(key).is_err() {
            env::set_var(key, value);
        }
    };
    
    let set_env_glib_override = |key: &str, value: &str| {
        // Set at both C level (glib) with override, and Rust level
        glib::setenv(key, value, true).ok();
        env::set_var(key, value);
    };
    
    // Helper to set env var only if not already set (PathBuf version)
    let set_env = |key: &str, value: &PathBuf| {
        let value_str = value.to_string_lossy();
        glib::setenv(key, &*value_str, false).ok();
        if env::var(key).is_err() {
            env::set_var(key, value);
        }
    };
    
    // GStreamer paths
    let gst_plugin_path = lib_dir.join("gstreamer-1.0");
    if gst_plugin_path.exists() {
        set_env("GST_PLUGIN_PATH", &gst_plugin_path);
        set_env("GST_PLUGIN_SYSTEM_PATH", &gst_plugin_path);
    }
    
    // GStreamer registry (writable cache)
    let gst_registry = base_dir.join("gst-registry.bin");
    set_env("GST_REGISTRY", &gst_registry);
    
    // GSettings schemas
    let schemas_dir = share_dir.join("glib-2.0").join("schemas");
    if schemas_dir.exists() {
        set_env("GSETTINGS_SCHEMA_DIR", &schemas_dir);
    }
    
    // XDG data dirs (for icons, themes, etc.)
    set_env("XDG_DATA_DIRS", &share_dir);
    set_env("XDG_DATA_HOME", &share_dir);
    
    // GDK Pixbuf loaders
    let pixbuf_loaders = lib_dir.join("gdk-pixbuf-2.0").join("2.10.0").join("loaders.cache");
    if pixbuf_loaders.exists() {
        set_env("GDK_PIXBUF_MODULE_FILE", &pixbuf_loaders);
    }
    let pixbuf_loaders_dir = lib_dir.join("gdk-pixbuf-2.0").join("2.10.0").join("loaders");
    if pixbuf_loaders_dir.exists() {
        set_env("GDK_PIXBUF_MODULEDIR", &pixbuf_loaders_dir);
    }
    
    // GTK paths
    set_env("GTK_DATA_PREFIX", &base_dir.to_path_buf());
    set_env("GTK_EXE_PREFIX", &base_dir.to_path_buf());
    
    // Locale directory
    let locale_dir = share_dir.join("locale");
    if locale_dir.exists() {
        set_env("LOCALEDIR", &locale_dir);
        // Also set TEXTDOMAINDIR for gettext compatibility
        set_env("TEXTDOMAINDIR", &locale_dir);
    }
    
    // Renderer and portal settings
    set_env_glib("GSK_RENDERER", "gl");
    set_env_glib("GTK_USE_PORTAL", "0");
    set_env_glib("ADW_DISABLE_PORTAL", "1");
    
    // Detect Windows language and set for gettext
    // Always detect and set the Windows UI language, overriding any system defaults
    // Use glib::setenv with override=true to ensure C library sees these values
    if let Some(lang) = detect_windows_language() {
        // Set all locale-related environment variables at C level
        let lang_utf8 = format!("{}.UTF-8", lang);
        set_env_glib_override("LANGUAGE", &lang);
        set_env_glib_override("LANG", &lang_utf8);
        set_env_glib_override("LC_ALL", &lang_utf8);
        set_env_glib_override("LC_MESSAGES", &lang_utf8);
    }
    
    // Add bin directory to PATH for DLL loading
    if let Some(current_path) = env::var_os("PATH") {
        let mut new_path = std::ffi::OsString::from(exe_dir);
        new_path.push(";");
        new_path.push(current_path);
        env::set_var("PATH", new_path);
    } else {
        env::set_var("PATH", exe_dir);
    }
}
