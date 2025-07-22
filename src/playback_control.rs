// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};
use log::{info, warn};

use crate::{audio::RepeatMode, i18n::i18n, volume_control::VolumeControl};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/playback-control.ui")]
    pub struct PlaybackControl {
        // Template widgets
        #[template_child]
        pub start_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub center_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub end_box: TemplateChild<gtk::Box>,

        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub volume_control: TemplateChild<VolumeControl>,

        #[template_child]
        pub playlist_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub shuffle_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub repeat_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackControl {
        const NAME: &'static str = "AmberolPlaybackControl";
        type Type = super::PlaybackControl;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("playbackcontrol");
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            VolumeControl::static_type();
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackControl {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.menu_button.set_primary(true);
        }
    }

    impl WidgetImpl for PlaybackControl {}
}

glib::wrapper! {
    pub struct PlaybackControl(ObjectSubclass<imp::PlaybackControl>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for PlaybackControl {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl PlaybackControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play_button(&self) -> gtk::Button {
        self.imp().play_button.get()
    }

    pub fn repeat_button(&self) -> gtk::Button {
        self.imp().repeat_button.get()
    }

    pub fn volume_control(&self) -> VolumeControl {
        self.imp().volume_control.get()
    }

    pub fn set_repeat_mode(&self, repeat_mode: RepeatMode) {
        let repeat_button = self.imp().repeat_button.get();
        let (icon_name, tooltip) = match repeat_mode {
            RepeatMode::Consecutive => {
                ("media-playlist-consecutive-symbolic", i18n("Enable Repeat"))
            }
            RepeatMode::RepeatAll => {
                ("media-playlist-repeat-symbolic", i18n("Repeat All Songs"))
            }
            RepeatMode::RepeatOne => {
                ("media-playlist-repeat-song-symbolic", i18n("Repeat the Current Song"))
            }
        };
        
        info!("🎯 Setting repeat button icon: {} (mode: {:?})", icon_name, repeat_mode);
        
        // Try to load the icon to verify it exists
        let icon_theme = gtk::IconTheme::for_display(&repeat_button.display());
        
        // Debug icon theme information
        if let Some(theme_name) = icon_theme.theme_name() {
            info!("  🎨 Current icon theme: {}", theme_name);
        }
        let search_paths = icon_theme.search_path();
        info!("  📂 Icon search paths: {} directories", search_paths.len());
        for (i, path) in search_paths.iter().take(3).enumerate() {
            info!("    {}. {:?}", i + 1, path);
        }
        
        // Check if similar icons exist
        let related_icons = [
            "media-playlist-consecutive-symbolic",
            "media-playlist-repeat-symbolic", 
            "media-playlist-repeat-song-symbolic",
            "media-playlist-shuffle-symbolic"
        ];
        for related_icon in related_icons {
            let exists = icon_theme.has_icon(related_icon);
            info!("  🔍 Icon '{}': {}", related_icon, if exists { "✅" } else { "❌" });
        }
        
        if icon_theme.has_icon(icon_name) {
            info!("  ✅ Icon '{}' found in theme", icon_name);
            
            // Try to actually load the icon to see if there are issues
            match icon_theme.lookup_icon(icon_name, &[], 16, 1, gtk::TextDirection::None, gtk::IconLookupFlags::FORCE_SVG) {
                Some(icon_paintable) => {
                    info!("  🎨 Icon paintable loaded successfully");
                    // Check if it's actually an SVG
                    if let Some(file) = icon_paintable.file() {
                        if let Some(path) = file.path() {
                            info!("  📁 Icon loaded from: {:?}", path);
                        } else {
                            info!("  📦 Icon loaded from GResource");
                        }
                    }
                }
                None => {
                    warn!("  ⚠️ Icon found in theme but failed to load paintable!");
                }
            }
        } else {
            warn!("  ❌ Icon '{}' NOT found in theme!", icon_name);
            warn!("     Fallback will be used (may show as missing icon)");
        }
        
        repeat_button.set_icon_name(icon_name);
        repeat_button.set_tooltip_text(Some(&tooltip));
        
        // Additional debugging: check what icon was actually set
        if let Some(actual_icon) = repeat_button.icon_name() {
            info!("  📋 Button now shows icon: {}", actual_icon);
        } else {
            warn!("  ⚠️ Button has no icon name set!");
        }
        
        // Try alternative loading method for problematic icons
        if icon_name == "media-playlist-consecutive-symbolic" {
            info!("  🔧 Trying alternative loading for consecutive icon...");
            
            // Try loading directly from GResource
            if let Ok(resource_bytes) = gio::resources_lookup_data(
                "/io/bassi/Amberol/icons/scalable/actions/media-playlist-consecutive-symbolic.svg",
                gio::ResourceLookupFlags::NONE
            ) {
                info!("  📦 Successfully loaded icon data from GResource ({} bytes)", resource_bytes.len());
                
                // Try creating a texture from the SVG data
                match gtk::gdk::Texture::from_bytes(&resource_bytes) {
                    Ok(_texture) => {
                        info!("  ✅ SVG data can be parsed as texture");
                    }
                    Err(e) => {
                        warn!("  ❌ Failed to parse SVG as texture: {}", e);
                    }
                }
            } else {
                warn!("  ❌ Failed to load icon data from GResource");
            }
        }
        
        // Force a redraw to ensure the icon updates
        repeat_button.queue_draw();
    }
}
