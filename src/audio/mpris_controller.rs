// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::{
    cell::OnceCell,
    rc::Rc,
};

use std::cell::RefCell;

use async_channel::Sender;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use glib::clone;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use gtk::glib;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use log::error;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Player, Time};

use crate::audio::{Controller, PlaybackAction, PlaybackState, RepeatMode, Song};
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use crate::config::APPLICATION_ID;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MprisController {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    mpris: Rc<OnceCell<Player>>,
    song: RefCell<Option<Song>>,
}

#[allow(dead_code)]
impl MprisController {
    pub fn new(sender: Sender<PlaybackAction>) -> Self {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            let builder = Player::builder(APPLICATION_ID)
                .identity("Amberol")
                .desktop_entry(APPLICATION_ID)
                .can_raise(true)
                .can_play(false)
                .can_pause(true)
                .can_seek(true)
                .can_go_next(true)
                .can_go_previous(true)
                .can_set_fullscreen(false);

            let mpris = Rc::new(OnceCell::new());

            glib::spawn_future_local(clone!(
                #[weak]
                mpris,
                #[strong]
                sender,
                async move {
                    match builder.build().await {
                        Err(err) => error!("Failed to create MPRIS server: {:?}", err),
                        Ok(player) => {
                            setup_signals(sender, &player);
                            mpris.set(player).unwrap();
                        }
                    }
                }
            ));

            Self {
                mpris,
                song: RefCell::new(None),
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        {
            // Stub implementation for non-Linux platforms
            let _ = sender; // Suppress unused variable warning
            Self {
                song: RefCell::new(None),
            }
        }
    }

    pub fn stop(&self) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                if let Err(err) = mpris.set_playback_status(PlaybackStatus::Stopped) {
                    error!("Could not update playback state in MPRIS: {}", err);
                }
            }
        }
    }

    pub fn play(&self) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                if let Err(err) = mpris.set_playback_status(PlaybackStatus::Playing) {
                    error!("Could not update playback state in MPRIS: {}", err);
                }
            }
        }
    }

    pub fn pause(&self) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                if let Err(err) = mpris.set_playback_status(PlaybackStatus::Paused) {
                    error!("Could not update playback state in MPRIS: {}", err);
                }
            }
        }
    }

    pub fn skip(&self) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                mpris.next();
            }
        }
    }

    pub fn previous(&self) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                mpris.previous();
            }
        }
    }

    pub fn seek(&self, position: i64) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                let time = Time::from_micros(position);
                if let Err(err) = mpris.set_position(time) {
                    error!("Could not set position in MPRIS: {}", err);
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        {
            let _ = position; // Suppress unused variable warning
        }
    }

    pub fn update_repeat_mode(&self, repeat_mode: RepeatMode) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                let loop_status = match repeat_mode {
                    RepeatMode::Consecutive => LoopStatus::None,
                    RepeatMode::RepeatAll => LoopStatus::Playlist,
                    RepeatMode::RepeatOne => LoopStatus::Track,
                };

                if let Err(err) = mpris.set_loop_status(loop_status) {
                    error!("Could not set loop status in MPRIS: {}", err);
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        {
            let _ = repeat_mode; // Suppress unused variable warning
        }
    }

    pub fn update_song(&self, song: &Song) {
        self.song.replace(Some(song.clone()));

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                let mut metadata = Metadata::new();

                metadata = metadata.title(song.title().unwrap_or("Unknown Title"));

                if let Some(artist) = song.artist() {
                    metadata = metadata.artist([artist]);
                }

                if let Some(album) = song.album() {
                    metadata = metadata.album(album);
                }

                if let Some(duration) = song.duration() {
                    metadata = metadata.length(Time::from_nanos(duration.nseconds() as i64));
                }

                if let Some(track_number) = song.track_number() {
                    metadata = metadata.track_number(track_number as i32);
                }

                // Try to load the cover art
                if let Some(cover_texture) = song.cover_texture() {
                    if let Some(file) = song.cached_cover_path() {
                        let cover_art_uri = file.uri();
                        metadata = metadata.art_url(cover_art_uri.as_str());
                    }
                }

                if let Err(err) = mpris.set_metadata(metadata) {
                    error!("Could not set metadata in MPRIS: {}", err);
                }
            }
        }
    }
}

impl Controller for MprisController {
    fn set_playback_state(&self, state: &PlaybackState) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            let status = match state {
                PlaybackState::Playing => PlaybackStatus::Playing,
                PlaybackState::Paused => PlaybackStatus::Paused,
                _ => PlaybackStatus::Stopped,
            };

            if let Some(mpris) = self.mpris.get() {
                if let Err(err) = mpris.set_playback_status(status) {
                    error!("Could not update playback state in MPRIS: {}", err);
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        {
            let _ = state; // Suppress unused variable warning
        }
    }

    fn set_song(&self, song: &Song) {
        self.update_song(song);
    }

    fn set_position(&self, position: u64) {
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            if let Some(mpris) = self.mpris.get() {
                let time = Time::from_secs(position as i64);
                if let Err(err) = mpris.set_position(time) {
                    error!("Could not set position in MPRIS: {}", err);
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        {
            let _ = position; // Suppress unused variable warning
        }
    }

    fn set_repeat_mode(&self, repeat: RepeatMode) {
        self.update_repeat_mode(repeat);
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn setup_signals(sender: Sender<PlaybackAction>, player: &Player) {
    player.connect_raise(clone!(
        #[strong]
        sender,
        move || {
            let app = gio::Application::default().expect("No default GApplication");
            app.activate();
            sender.send_blocking(PlaybackAction::Raise).unwrap();
        }
    ));

    player.connect_quit(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Quit).unwrap()
    ));

    player.connect_play_pause(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Toggle).unwrap()
    ));

    player.connect_play(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Play).unwrap()
    ));

    player.connect_pause(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Pause).unwrap()
    ));

    player.connect_stop(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Stop).unwrap()
    ));

    player.connect_next(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Skip).unwrap()
    ));

    player.connect_previous(clone!(
        #[strong]
        sender,
        move || sender.send_blocking(PlaybackAction::Previous).unwrap()
    ));

    player.connect_seek(clone!(
        #[strong]
        sender,
        move |offset| {
            sender
                .send_blocking(PlaybackAction::Seek(offset.as_nanos() as i64))
                .unwrap()
        }
    ));

    player.connect_set_position(clone!(
        #[strong]
        sender,
        move |_track_id, position| {
            sender
                .send_blocking(PlaybackAction::Seek(position.as_nanos() as i64))
                .unwrap()
        }
    ));
}
