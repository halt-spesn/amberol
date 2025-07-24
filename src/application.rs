// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use adw::subclass::prelude::*;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use ashpd::{desktop::background::Background, WindowIdentifier};
use async_channel::Receiver;
use glib::clone;
use gtk::{gdk, gio, glib, prelude::*};
use log::{debug, info, warn, error};

use crate::{
    audio::AudioPlayer,
    config::{APPLICATION_ID, VERSION},
    i18n::i18n,
    utils,
    window::Window,
    system_tray::SystemTray,
};

pub enum ApplicationAction {
    Present,
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct Application {
        pub player: Rc<AudioPlayer>,
        pub receiver: RefCell<Option<Receiver<ApplicationAction>>>,
        pub background_hold: RefCell<Option<gio::ApplicationHoldGuard>>,
        pub settings: gio::Settings,
        #[cfg(target_os = "windows")]
        pub system_tray: RefCell<Option<SystemTray>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "AmberolApplication";
        type Type = super::Application;
        type ParentType = adw::Application;

        fn new() -> Self {
            let (sender, r) = async_channel::unbounded();
            let receiver = RefCell::new(Some(r));

            // Try to create settings with fallback for schema ID issues
            let settings = Self::create_settings_with_fallback();
            
            Self {
                player: AudioPlayer::new(sender),
                receiver,
                background_hold: RefCell::default(),
                settings,
                #[cfg(target_os = "windows")]
                system_tray: RefCell::new(None),
            }
        }
    }
    
    impl Application {
        fn create_settings_with_fallback() -> gio::Settings {
            use gio::prelude::*;
            
            // Check if the schema exists before trying to create Settings
            let schema_source = gio::SettingsSchemaSource::default().unwrap();
            
            // First try the configured APPLICATION_ID
            if let Some(_schema) = schema_source.lookup(APPLICATION_ID, true) {
                debug!("Successfully found settings schema: {}", APPLICATION_ID);
                return gio::Settings::new(APPLICATION_ID);
            }
            
            warn!("Schema '{}' not found", APPLICATION_ID);
            
            // If APPLICATION_ID is a development schema, try the release version
            if APPLICATION_ID.ends_with(".Devel") {
                let release_id = APPLICATION_ID.replace(".Devel", "");
                warn!("Trying release schema '{}'", release_id);
                
                if let Some(_schema) = schema_source.lookup(&release_id, true) {
                    info!("Successfully found release schema: {}", release_id);
                    return gio::Settings::new(&release_id);
                } else {
                    error!("Release schema '{}' also not found", release_id);
                }
            }
            
            // List available schemas for debugging
            error!("Available schemas:");
            let (non_relocatable, relocatable) = schema_source.list_schemas(true);
            for schema in non_relocatable.iter().chain(relocatable.iter()) {
                if schema.contains("Amberol") || schema.contains("bassi") {
                    error!("  - {}", schema);
                }
            }
            
            error!("Cannot initialize application - no compatible settings schema found");
            error!("This is likely a packaging issue - GSettings schemas not properly installed");
            error!("Expected schema: {} or fallback: {}", APPLICATION_ID, APPLICATION_ID.replace(".Devel", ""));
            panic!("Cannot initialize application without settings schema");
        }
    }

    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_resource_base_path(Some("/io/bassi/Amberol/"));
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self) {
            debug!("Application<activate>");
            let application = self.obj();
            application.present_main_window();
        }

        fn startup(&self) {
            debug!("Application<startup>");
            self.parent_startup();
            let application = self.obj();

            // Set up system tray on Windows
            #[cfg(target_os = "windows")]
            {
                info!("🔧 Setting up Windows system tray");
                match SystemTray::new() {
                    Ok(tray) => {
                        info!("✅ System tray created successfully");
                        *self.system_tray.borrow_mut() = Some(tray);
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to create system tray: {}", e);
                    }
                }
                
                // Set up tray signal monitoring
                let app_weak = application.downgrade();
                glib::timeout_add_seconds_local(1, move || {
                    if let Some(app) = app_weak.upgrade() {
                        // Check for restore signal file
                        if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
                            let signal_file = temp_dir.join("amberol-restore-signal");
                            if signal_file.exists() {
                                info!("📱 Detected tray restore signal, presenting window");
                                app.present_main_window();
                                // Remove the signal file
                                let _ = std::fs::remove_file(&signal_file);
                            }
                        }
                        glib::ControlFlow::Continue
                    } else {
                        glib::ControlFlow::Break
                    }
                });
                info!("✅ Tray signal monitoring started");
            }

            // Set up CSS
            // utils::load_css(); // This function doesn't exist, CSS is loaded by the window

            // Handle application action receiver
            let receiver = self.receiver.take().unwrap();
            glib::spawn_future_local(clone!(@weak application => async move {
                while let Ok(action) = receiver.recv().await {
                    match action {
                        ApplicationAction::Present => application.present_main_window(),
                    }
                }
            }));
            
            // Replace all asset-based icons with programmatic rendering after a short delay
            // to ensure all widgets are properly initialized
            glib::timeout_add_seconds_local(2, clone!(@weak application => @default-return glib::ControlFlow::Break, move || {
                use crate::icon_renderer::IconRenderer;
                IconRenderer::apply_global_icon_fallbacks(&application);
                
                        // Setup global icon theme override first
        crate::icon_theme_provider::IconThemeProvider::setup_global_override();
        
        // Setup aggressive icon replacement scanning
        crate::icon_replacer::IconReplacer::setup_periodic_replacement();
        
        // Setup desktop integration (taskbar icons, tray icons)
        crate::desktop_integration::DesktopIntegration::setup_integration(&application);
                
                glib::ControlFlow::Break // Run only once
            }));
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APPLICATION_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }

    pub fn player(&self) -> Rc<AudioPlayer> {
        self.imp().player.clone()
    }

    fn present_main_window(&self) {
        let window = if let Some(window) = self.active_window() {
            window
        } else {
            let window = Window::new(self);
            window.upcast()
        };

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        self.request_background();

        #[cfg(target_os = "windows")]
        self.request_background_windows();

        window.present();
    }

    fn setup_gactions(&self) {
        // Create and add simple actions using the underlying gio::Application
        let app = self.upcast_ref::<gio::Application>();
        
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _| {
                this.quit();
            }
        ));
        app.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _| {
                // Ensure icons are available before showing about dialog
                crate::icon_theme_provider::IconThemeProvider::force_create_about_icons();
                this.show_about();
            }
        ));
        app.add_action(&about_action);

        let background_play = self.imp().settings.boolean("background-play");
        let background_play_action = gio::SimpleAction::new_stateful(
            "background-play",
            None,
            &background_play.to_variant(),
        );
        background_play_action.connect_activate(clone!(
            #[weak(rename_to = this)]
            self,
            move |action, _| {
                let state = action.state().unwrap();
                let background_play = state.get::<bool>().unwrap();
                let new_state = !background_play;
                action.set_state(&new_state.to_variant());
                this.imp().settings.set_boolean("background-play", new_state).unwrap();

                if new_state {
                    this.imp().background_hold.replace(Some(this.hold()));
                } else {
                    this.imp().background_hold.replace(None);
                }
            }
        ));
        app.add_action(&background_play_action);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_icon(APPLICATION_ID)
            .application_name("Amberol")
            .developer_name("Emmanuele Bassi")
            .version(VERSION)
            .developers(vec!["Emmanuele Bassi"])
            .copyright("© 2022 Emmanuele Bassi")
            .website("https://apps.gnome.org/Amberol/")
            .issue_url("https://gitlab.gnome.org/World/amberol/-/issues/new")
            .license_type(gtk::License::Gpl30)
            // Translators: Replace "translator-credits" with your names, one name per line
            .translator_credits(i18n("translator-credits"))
            .build();

        // Fix icons in the about dialog after it's created
        glib::timeout_add_local_once(std::time::Duration::from_millis(100), glib::clone!(@weak dialog => move || {
            Self::fix_about_dialog_icons(&dialog);
        }));

        dialog.present();
    }
    
    /// Fix icons in the about dialog by directly replacing them
    fn fix_about_dialog_icons(dialog: &adw::AboutWindow) {
        use gtk::prelude::*;
        info!("🔧 Directly fixing about dialog icons");
        
        // Try to find and replace images in the about dialog
        Self::scan_and_fix_widget_icons(dialog.upcast_ref::<gtk::Widget>());
        
        // Also try to set a custom application icon directly if possible
        if let Some(texture) = Self::create_about_app_icon() {
            // Unfortunately, AdwAboutWindow doesn't expose a direct way to set the icon
            // But our widget scanning should catch it
            info!("✅ Created custom about dialog app icon texture");
        }
    }
    
    /// Create a custom application icon for the about dialog
    fn create_about_app_icon() -> Option<gdk::Texture> {
        if let Some(mut surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(64) {
            // Convert surface to pixbuf then texture
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
        None
    }
    
    /// Recursively scan and fix icons in a widget tree
    fn scan_and_fix_widget_icons(widget: &gtk::Widget) {
        // Check if this widget is an Image that might need fixing
        if let Some(image) = widget.downcast_ref::<gtk::Image>() {
            Self::fix_image_icon(image);
        }
        
        // Check if this widget is a Button with an icon
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            Self::fix_button_icon(button);
        }
        
        // Recursively scan child widgets
        let mut child = widget.first_child();
        while let Some(current_child) = child {
            Self::scan_and_fix_widget_icons(&current_child);
            child = current_child.next_sibling();
        }
    }
    
    /// Fix a specific image widget
    fn fix_image_icon(image: &gtk::Image) {
        use gtk::prelude::*;
        
        // Check what kind of image this is and if it needs fixing
        match image.storage_type() {
            gtk::ImageType::IconName => {
                if let Some(icon_name) = image.icon_name() {
                    if Self::should_fix_icon(&icon_name) {
                        info!("🎨 Fixing image icon: {}", icon_name);
                                                 if let Some(texture) = Self::create_icon_texture(&icon_name) {
                             image.set_paintable(Some(&texture));
                         }
                    }
                }
            }
            gtk::ImageType::Gicon => {
                if let Some(gicon) = image.gicon() {
                    if let Some(themed_icon) = gicon.downcast_ref::<gtk::gio::ThemedIcon>() {
                        let names = themed_icon.names();
                        for name in names {
                            if Self::should_fix_icon(&name) {
                                info!("🎨 Fixing GIcon: {}", name);
                                                                 if let Some(texture) = Self::create_icon_texture(&name) {
                                     image.set_paintable(Some(&texture));
                                     break;
                                 }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Fix a specific button widget
    fn fix_button_icon(button: &gtk::Button) {
        use gtk::prelude::*;
        
        if let Some(icon_name) = button.icon_name() {
            if Self::should_fix_icon(&icon_name) {
                info!("🎨 Fixing button icon: {}", icon_name);
                crate::icon_renderer::IconRenderer::set_button_icon_programmatic(button, &icon_name);
            }
        }
    }
    
    /// Check if an icon should be fixed
    fn should_fix_icon(icon_name: &str) -> bool {
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
        )
    }
    
    /// Create a texture for a specific icon name
    fn create_icon_texture(icon_name: &str) -> Option<gdk::Texture> {
        if let Some(mut surface) = crate::icon_renderer::IconRenderer::create_app_icon_surface(24) {
            // Draw the specific icon on the surface
            if let Ok(cr) = gtk::cairo::Context::new(&surface) {
                cr.set_source_rgba(0.2, 0.2, 0.2, 1.0);
                
                let success = match icon_name {
                    "io.bassi.Amberol" | "io.bassi.Amberol.Devel" => Self::draw_musical_note(&cr),
                    "web-browser-symbolic" | "user-home-symbolic" => Self::draw_globe(&cr),
                    "bug-symbolic" | "document-edit-symbolic" => Self::draw_bug(&cr),
                    _ => false,
                };
                
                if success {
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
    
    /// Draw a musical note
    fn draw_musical_note(cr: &gtk::cairo::Context) -> bool {
        // Simple musical note
        cr.arc(4.0, 16.0, 3.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or(());
        cr.arc(14.0, 12.0, 2.5, 0.0, 2.0 * std::f64::consts::PI);
        cr.fill().unwrap_or(());
        cr.move_to(7.0, 16.0);
        cr.line_to(7.0, 4.0);
        cr.line_to(16.5, 2.0);
        cr.line_to(16.5, 12.0);
        cr.stroke().unwrap_or(());
        true
    }
    
    /// Draw a globe icon
    fn draw_globe(cr: &gtk::cairo::Context) -> bool {
        // Simple globe
        cr.arc(12.0, 12.0, 8.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.stroke().unwrap_or(());
        cr.move_to(12.0, 4.0);
        cr.line_to(12.0, 20.0);
        cr.stroke().unwrap_or(());
        cr.move_to(4.0, 12.0);
        cr.line_to(20.0, 12.0);
        cr.stroke().unwrap_or(());
        true
    }
    
    /// Draw a bug icon
    fn draw_bug(cr: &gtk::cairo::Context) -> bool {
        // Simple bug
        cr.arc(12.0, 12.0, 6.0, 0.0, 2.0 * std::f64::consts::PI);
        cr.stroke().unwrap_or(());
        // Legs
        for i in 0..3 {
            let y = 8.0 + i as f64 * 3.0;
            cr.move_to(6.0, y);
            cr.line_to(2.0, y - 1.0);
            cr.move_to(18.0, y);
            cr.line_to(22.0, y - 1.0);
            cr.stroke().unwrap_or(());
        }
        true
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    async fn portal_request_background(&self) {
        if let Some(window) = self.active_window() {
            let root = window.native().unwrap();
            let identifier = WindowIdentifier::from_native(&root).await;
            let request = Background::request().identifier(identifier).reason(&*i18n(
                "Amberol needs to run in the background to play music",
            ));

            match request.send().await.and_then(|r| r.response()) {
                Ok(response) => {
                    debug!("Background request successful: {:?}", response);
                    self.imp().background_hold.replace(Some(self.hold()));
                }
                Err(err) => {
                    warn!("Background request denied: {}", err);
                    self.imp()
                        .settings
                        .set_boolean("background-play", false)
                        .expect("Unable to set background-play settings key");
                }
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn request_background(&self) {
        let background_play = self.imp().settings.boolean("background-play");
        if background_play {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak self as app => async move {
                app.portal_request_background().await
            }));
        }
    }

    #[cfg(target_os = "windows")]
    fn request_background_windows(&self) {
        let background_play = self.imp().settings.boolean("background-play");
        if background_play {
            // On Windows, we can use the Power Management API to prevent sleep
            // This is a simplified approach - in a real implementation you might
            // want to use SetThreadExecutionState or other Windows APIs
            debug!("Background play enabled on Windows");
            self.imp().background_hold.replace(Some(self.hold()));
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "windows")))]
    fn request_background(&self) {}
}
