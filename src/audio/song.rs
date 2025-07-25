// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    path::PathBuf,
    time::Instant,
};

use glib::{ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, ParamSpecUInt, Value};
use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use lofty::{prelude::*, file::TaggedFileExt};
use log::{debug, warn};
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};

use crate::{
    audio::cover_cache::{CoverArt, CoverCache},
    i18n::i18n,
};

#[derive(Debug, Clone)]
pub struct SongData {
    artist: Option<String>,
    title: Option<String>,
    album: Option<String>,
    cover_art: Option<CoverArt>,
    cover_uuid: Option<String>,
    uuid: Option<String>,
    duration: u64,
    file: gio::File,
}

impl SongData {
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn cover_uuid(&self) -> Option<&str> {
        self.cover_uuid.as_deref()
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub fn cover_texture(&self) -> Option<&gdk::Texture> {
        if let Some(cover) = &self.cover_art {
            return Some(cover.texture());
        }

        None
    }

    pub fn cover_palette(&self) -> Option<&Vec<gdk::RGBA>> {
        if let Some(cover) = &self.cover_art {
            return Some(cover.palette());
        }

        None
    }

    pub fn cover_cache(&self) -> Option<&PathBuf> {
        if let Some(cover) = &self.cover_art {
            return cover.cache();
        }

        None
    }

    pub fn from_uri(uri: &str) -> Self {
        let now = Instant::now();

        let file = gio::File::for_uri(uri);
        let path = file.path().expect("Unable to find file");

        let tagged_file = match lofty::read_from_path(&path) {
            Ok(f) => {
                debug!("Successfully read metadata from: {:?}", path);
                Some(f)
            },
            Err(e) => {
                warn!("Unable to read metadata from file {:?}: {} - will create basic entry", path, e);
                // Still try to create a basic song entry without metadata
                None
            }
        };

        let mut cover_cache = CoverCache::global().lock().unwrap();

        let mut artist = None;
        let mut title = None;
        let mut album = None;
        let mut cover_art = None;
        let mut cover_uuid = None;
        
        if let Some(ref tagged_file) = tagged_file {
            if let Some(tag) = tagged_file.primary_tag() {
            debug!("Found primary tag");
            artist = tag.artist().map(|s| s.to_string());
            title = tag.title().map(|s| s.to_string());
            album = tag.album().map(|s| s.to_string());
                if let Some(res) = cover_cache.cover_art(&path, tag) {
                    cover_art = Some(res.0);
                    cover_uuid = Some(res.1);
                }
            } else {
                warn!("Unable to load primary tag for: {}", uri);
                for tag in tagged_file.tags() {
                debug!("Found tag: {:?}", tag.tag_type());
                artist = tag.artist().map(|s| s.to_string());
                title = tag.title().map(|s| s.to_string());
                album = tag.album().map(|s| s.to_string());
                if let Some(res) = cover_cache.cover_art(&path, tag) {
                    cover_art = Some(res.0);
                    cover_uuid = Some(res.1);
                }

                    if artist.is_some() && title.is_some() {
                        break;
                    }
                }
            }
        }
        
        // If we couldn't parse metadata, try to extract basic info from filename
        if tagged_file.is_none() {
            warn!("No metadata available, using filename for basic info");
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                // Try to extract artist - title from filename patterns like "Artist - Title.mp3"
                if let Some(dash_pos) = filename.find(" - ") {
                    artist = Some(filename[..dash_pos].to_string());
                    title = Some(filename[dash_pos + 3..].to_string());
                } else {
                    // Just use the filename as title
                    title = Some(filename.to_string());
                }
            }
        }

        let uuid = match file.query_info(
            "standard::display-name",
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            Ok(info) => {
                let mut hasher = Sha256::new();

                hasher.update(info.display_name().as_str());

                if let Some(ref artist) = artist {
                    hasher.update(artist);
                }
                if let Some(ref title) = title {
                    hasher.update(title);
                }
                if let Some(ref album) = album {
                    hasher.update(album);
                }

                Some(format!("{:x}", hasher.finalize()))
            }
            _ => None,
        };

        let duration = if let Some(ref tagged_file) = tagged_file {
            let properties = tagged_file.properties();
            properties.duration().as_secs()
        } else {
            // Default duration when metadata parsing failed
            0
        };

        debug!(
            "Song {:?} ('{:?}') loading time: {} ms",
            &uuid,
            &title,
            now.elapsed().as_millis()
        );

        SongData {
            artist,
            title,
            album,
            cover_art,
            cover_uuid,
            uuid,
            duration,
            file,
        }
    }

    pub fn uri(&self) -> String {
        self.file.uri().to_string()
    }

    pub fn file(&self) -> gio::File {
        self.file.clone()
    }
}

impl Default for SongData {
    fn default() -> Self {
        SongData {
            artist: Some("Invalid Artist".to_string()),
            title: Some("Invalid Title".to_string()),
            album: Some("Invalid Album".to_string()),
            cover_art: None,
            cover_uuid: None,
            uuid: None,
            duration: 0,
            file: gio::File::for_path("/does-not-exist"),
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Song {
        pub data: RefCell<SongData>,
        pub playing: Cell<bool>,
        pub selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Song {
        const NAME: &'static str = "AmberolSong";
        type Type = super::Song;
    }

    impl ObjectImpl for Song {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("uri").construct_only().build(),
                    ParamSpecString::builder("artist").read_only().build(),
                    ParamSpecString::builder("title").read_only().build(),
                    ParamSpecString::builder("album").read_only().build(),
                    ParamSpecUInt::builder("duration").read_only().build(),
                    ParamSpecObject::builder::<gdk::Texture>("cover")
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("playing").build(),
                    ParamSpecBoolean::builder("selected").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "uri" => {
                    let obj = self.obj();
                    if let Ok(p) = value.get::<&str>() {
                        self.data.replace(SongData::from_uri(p));
                        obj.notify("artist");
                        obj.notify("title");
                        obj.notify("album");
                        obj.notify("duration");
                        obj.notify("cover");
                    }
                }
                "playing" => {
                    let p = value.get::<bool>().expect("Value must be a boolean");
                    self.playing.set(p);
                }
                "selected" => {
                    let p = value.get::<bool>().expect("Value must be a boolean");
                    self.selected.set(p);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            let obj = self.obj();
            match pspec.name() {
                "artist" => obj.artist().to_value(),
                "title" => obj.title().to_value(),
                "album" => obj.album().to_value(),
                "duration" => obj.duration().to_value(),
                "uri" => obj.uri().to_value(),
                "cover" => obj.cover_texture().to_value(),
                "playing" => self.playing.get().to_value(),
                "selected" => self.selected.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Song(ObjectSubclass<imp::Song>);
}

impl Song {
    pub fn new(uri: &str) -> Self {
        glib::Object::builder::<Self>().property("uri", uri).build()
    }

    pub fn from_uri(uri: &str) -> Result<Song, &'static str> {
        let res = Song::new(uri);
        if res.equals(&Song::default()) {
            Err("Invalid song")
        } else {
            Ok(res)
        }
    }

    pub fn empty() -> Self {
        glib::Object::new()
    }

    pub fn equals(&self, other: &Self) -> bool {
        if self.uuid().is_some() && other.uuid().is_some() {
            self.uuid() == other.uuid()
        } else {
            self.uri() == other.uri()
        }
    }

    pub fn uri(&self) -> String {
        self.imp().data.borrow().uri()
    }

    pub fn artist(&self) -> String {
        match self.imp().data.borrow().artist() {
            Some(artist) => artist.to_string(),
            None => i18n("Unknown artist"),
        }
    }

    pub fn title(&self) -> String {
        match self.imp().data.borrow().title() {
            Some(title) => title.to_string(),
            None => i18n("Unknown title"),
        }
    }

    pub fn album(&self) -> String {
        match self.imp().data.borrow().album() {
            Some(album) => album.to_string(),
            None => i18n("Unknown album"),
        }
    }

    pub fn cover_texture(&self) -> Option<gdk::Texture> {
        self.imp().data.borrow().cover_texture().cloned()
    }

    pub fn cover_color(&self) -> Option<gdk::RGBA> {
        self.imp().data.borrow().cover_palette().map(|p| p[0])
    }

    pub fn cover_palette(&self) -> Option<Vec<gdk::RGBA>> {
        self.imp().data.borrow().cover_palette().cloned()
    }

    pub fn cover_uuid(&self) -> Option<String> {
        self.imp().data.borrow().cover_uuid().map(|s| s.to_string())
    }

    pub fn cover_cache(&self) -> Option<PathBuf> {
        self.imp().data.borrow().cover_cache().cloned()
    }

    pub fn duration(&self) -> u64 {
        self.imp().data.borrow().duration()
    }

    pub fn playing(&self) -> bool {
        self.imp().playing.get()
    }

    pub fn set_playing(&self, playing: bool) {
        let was_playing = self.imp().playing.replace(playing);
        if was_playing != playing {
            self.notify("playing");
        }
    }

    pub fn selected(&self) -> bool {
        self.imp().selected.get()
    }

    pub fn set_selected(&self, selected: bool) {
        let was_selected = self.imp().selected.replace(selected);
        if was_selected != selected {
            self.notify("selected");
        }
    }

    pub fn uuid(&self) -> Option<String> {
        self.imp().data.borrow().uuid().map(|s| s.to_string())
    }

    pub fn search_key(&self) -> String {
        format!("{} {} {}", self.artist(), self.album(), self.title())
    }

    pub fn file(&self) -> gio::File {
        self.imp().data.borrow().file()
    }
}

impl Default for Song {
    fn default() -> Self {
        Self::empty()
    }
}

impl Display for Song {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Song {{ uuid: {}, uri: {}, artist: '{}', title: '{}' }}",
            self.uuid().unwrap(),
            self.uri(),
            self.artist(),
            self.title()
        )
    }
}
