#[allow(unused_imports)]
use windows::{core::*, Data, Foundation::IReference,
    Media::{ MediaPlaybackType as MPT, 
        Control::{
            GlobalSystemMediaTransportControlsSession as TCS, GlobalSystemMediaTransportControlsSessionManager as TCSManager, GlobalSystemMediaTransportControlsSessionMediaProperties as TCSProperties, *}}};
pub use crate::img::*;
use core::fmt;
use std::fmt::Display;

#[derive(PartialEq, Eq, Copy, Clone, Default)]
pub struct SPT(pub i8);
impl SPT {
    pub const UNKNOWN: Self = Self(0);
    pub const AUDIO: Self = Self(1);
    pub const VIDEO: Self = Self(2);
    pub const IMAGE: Self = Self(3);
}
impl From<MPT> for SPT {
    fn from(mpt: MPT) -> Self {
        match mpt.0 {
            0 => SPT::UNKNOWN,
            1 => SPT::AUDIO,
            2 => SPT::VIDEO,
            3 => SPT::IMAGE,
            _ => SPT::UNKNOWN,
        }
    }
}
impl From<IReference<MPT>> for SPT {
    fn from(mpt: IReference<MPT>) -> Self {
        SPT::from(mpt.Value().unwrap())
    }
}
impl Display for  SPT{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self.0{
            0 => write!(f, "UNKNOWN"),
            1 => write!(f, "AUDIO"),
            2 => write!(f, "VIDEO"),
            3 => write!(f, "IMAGE"),
            _ => write!(f, "UNKNOWN"),
        }
    }
}

// Implement Debug for SpectrePlayType
impl fmt::Debug for SPT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SpectrePlayType({})", self)
    }
}

/// `SpectreProps` is a struct that holds all of the media metadata found in a 'TCSProperties', but in a more rusty way.
///
/// The `new()` and `new_async()` methods can be used to create new instances of the `SpectreProps` struct, while the `sync()` method can be used to update the properties of an existing instance based on the provided `TCSProperties`.
#[derive(Clone)]
pub struct SpectreProps {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: Option<String>,
    pub genres: Vec<String>,
    pub thumbnail: DynamicImage,
    pub track_number: Option<i32>,
    pub track_count: Option<i32>,
    pub playback_type: SPT,
    pub subtitle: Option<String>,
}


impl Default for SpectreProps {
    fn default() -> Self {
        SpectreProps {
            title: "Unknown Title".to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            album_artist: None,
            genres: vec![],
            thumbnail: ERROR_THUMB.clone(),
            track_number: None,
            track_count: None,
            playback_type: SPT::UNKNOWN,
            subtitle: None,
        }
    }
}

/// Provides methods for creating and synchronizing a `SpectreProps` struct with `TCSProperties`.
///
/// The `new()` method creates a new `SpectreProps` instance with default values.
/// The `from_tcsp()` method creates a new `SpectreProps` instance and loads data from a provided `TCSProperties`.
/// The `sync()` method updates the properties of an existing `SpectreProps` instance based on the provided `TCSProperties`.
impl SpectreProps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tcsp(props: TCSProperties) -> Self {
        let mut spectre_props = Self::new();
        spectre_props.sync(props);
        spectre_props
    }

    pub fn sync(&mut self, properties: TCSProperties) {
        self.title = match properties.Title() {
            Ok(title) => {if title.is_empty(){
                "Unknown Title".to_string()
            } else {
                title.to_string()
            }},
            Err(_) => "Unknown Title".to_string(),
        };
        self.artist = match properties.Artist() {
            Ok(artist) => {if artist.is_empty(){
                "Unknown Artist".to_string()
            } else {
                artist.to_string()
            }},
            Err(_) => "Unknown Artist".to_string(),
        };
        self.album = match properties.Artist() {
            Ok(album) => {if album.is_empty(){
                "Unknown Album".to_string()
            } else {
                album.to_string()
            }},
            Err(_) => "Unknown Album".to_string(),
        };
        self.album_artist = match properties.AlbumArtist() {
            Ok(album_artist) => Some(album_artist.to_string()),
            Err(_) => None,
        };
        self.genres = match properties.Genres() {
            Ok(ivv_genre) => {
                let mut genres_vec = Vec::new();
                for genre in ivv_genre {
                    genres_vec.push(genre.to_string());
                }
                genres_vec
            }
            Err(_) => vec![],
        };
        self.thumbnail = ref_to_thumb(properties.Thumbnail().ok());
        self.track_number = properties.TrackNumber().ok();
        self.track_count = properties.AlbumTrackCount().ok();
        self.playback_type = match properties.PlaybackType() {
            Ok(playback_type) => playback_type.into(),
            Err(_) => SPT::UNKNOWN,
        };
        self.subtitle = match properties.Subtitle() {
            Ok(subtitle) => Some(subtitle.to_string()),
            Err(_) => None,
        };

    }
}

impl fmt::Display for SpectreProps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Title: {}", &self.title)?;
        writeln!(f, "Artist: {}", &self.artist)?;
        writeln!(f, "Album: {}", &self.album)?;
        writeln!(f, "Album Artist: {}", &self.album_artist.as_deref().unwrap_or(""))?;
        writeln!(f, "Genres: {}", &self.genres.join(", "))
    }
}

impl fmt::Debug for SpectreProps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&self.title)
            .field("title", &self.title)
            .field("artist", &self.artist)
            .field("album", &self.album)
            .field("album_artist", &self.album_artist.as_deref().unwrap_or_default())
            .field("genres", &self.genres)
            .field("thumbnail", &format!("DynamicImage [{} x {}]", self.thumbnail.width(), self.thumbnail.height())) // Summary for DynamicImage
            .field("track_number", &self.track_number.unwrap_or_default())
            .field("track_count", &self.track_count.unwrap_or_default())
            .field("playback_type", &self.playback_type)
            .field("subtitle", &self.subtitle.as_deref().unwrap_or_default())
            .finish()
    }
}