// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use adw::subclass::prelude::*;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use ashpd::{desktop::background::Background, WindowIdentifier};
use async_channel::Receiver;
use glib::clone;
use gtk::{gio, glib, prelude::*};
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

        dialog.present();
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
