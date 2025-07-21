// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use adw::subclass::prelude::*;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use ashpd::{desktop::background::Background, WindowIdentifier};
use async_channel::Receiver;
use glib::clone;
use gtk::{gio, glib, prelude::*};
use log::{debug, warn};

use crate::{
    audio::AudioPlayer,
    config::{APPLICATION_ID, VERSION},
    i18n::i18n,
    utils,
    window::Window,
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
        pub background_hold: RefCell<Option<ApplicationHoldGuard>>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "AmberolApplication";
        type Type = super::Application;
        type ParentType = adw::Application;

        fn new() -> Self {
            let (sender, r) = async_channel::unbounded();
            let receiver = RefCell::new(Some(r));

            Self {
                player: AudioPlayer::new(sender),
                receiver,
                background_hold: RefCell::default(),
                settings: gio::Settings::new(APPLICATION_ID),
            }
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

            // Set up CSS
            utils::load_css();

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
        self.add_action_entries([
            gio::ActionEntry::builder("quit")
                .activate(|app: &Application, _, _| {
                    app.quit();
                })
                .build(),
            gio::ActionEntry::builder("about")
                .activate(|app: &Application, _, _| {
                    app.show_about();
                })
                .build(),
        ]);

        let background_play = self.imp().settings.boolean("background-play");
        self.add_action_entries([gio::ActionEntry::builder("background-play")
            .state(background_play.to_variant())
            .activate(|this: &Application, action, _| {
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let background_play = !action_state;
                action.set_state(&background_play.to_variant());

                this.imp()
                    .settings
                    .set_boolean("background-play", background_play)
                    .expect("Unable to store background-play setting");
            })
            .build()]);
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
