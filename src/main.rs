mod img;
mod debug;
use img::*;
#[allow(unused_imports)]
use windows::{core::*, Data, 
    Media::{ MediaPlaybackType as MPT, 
        Control::{
            GlobalSystemMediaTransportControlsSession as TCS, GlobalSystemMediaTransportControlsSessionManager as TCSManager, GlobalSystemMediaTransportControlsSessionMediaProperties as TCSProperties, *}}};
#[allow(unused_imports)]
use std::{io::{Error, ErrorKind}, result::Result};
use futures::executor::block_on;
use indexmap::IndexMap;

async fn async_main() -> Result<TCSManager, Error> {
    let manager: TCSManager = TCSManager::RequestAsync()?.await?;
    Ok(manager)
}
/// Gets the media properties for the provided `TCS` (Global System Media Transport Controls Session).
/// 
/// This function retrieves the media properties for the given `TCS` session, such as title, artist, album, etc.
/// 
/// # Arguments
/// * `sesh` - The `TCS` session to get the media properties for.
/// 
/// # Returns
/// A `Result` containing the `TCSProperties` for the provided `TCS` session, or an `Error` if the operation fails.
async fn get_props(sesh: TCS) -> Result<TCSProperties, Error> {
    let props: TCSProperties = sesh.TryGetMediaPropertiesAsync()?.await?;
    Ok(props)
}
fn main() {
    let manager: TCSManager = block_on(async_main()).unwrap();
    let sessions = manager.GetSessions().unwrap();
    println!("\n-----------------start-----------------");
    for sesh in sessions.into_iter() {
        let props = block_on(get_props(sesh)).unwrap();
        let mut spectre_p = SpectreProps::new();
        spectre_p.sync(props);
        let thumb_copy = spectre_p.thumbnail.clone();
        let title_copy = spectre_p.title.clone();

        for prop in spectre_p.into_iter() {
            println!("{}: {}", prop.0, prop.1.as_deref().unwrap_or("None"));
        }
        let thumb_file = debug::view_image(Some(thumb_copy), &title_copy).unwrap();
        println!("Thumbnail: {thumb_file}");

        println!();
    }
    println!("------------------end------------------")
}


/// `SpectreProps` is a struct that holds all of the media metadata found in a 'TCSProperties', but in a more rusty way.
///
/// The `new()` and `new_async()` methods can be used to create new instances of the `SpectreProps` struct, while the `sync()` method can be used to update the properties of an existing instance based on the provided `TCSProperties`.
#[derive(Debug)]
pub struct SpectreProps {
    title: String,
    artist: String,
    album: String,
    album_artist: Option<String>,
    genres: Vec<String>,
    thumbnail: DynamicImage,
    track_number: Option<i32>,
    track_count: Option<i32>,
    playback_type: MPT,
    subtitle: Option<String>,
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
            playback_type: MPT::Unknown,
            subtitle: None,
        }
    }
}

/// Provides methods for creating and synchronizing a `SpectreProps` struct with `TCSProperties`.
///
/// The `new()` method creates a new `SpectreProps` instance with default values.
/// The `new_async()` method creates a new `SpectreProps` instance and synchronizes it with the provided `TCSProperties`.
/// The `sync()` method updates the properties of an existing `SpectreProps` instance based on the provided `TCSProperties`.
impl SpectreProps {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn new_async(sesh: TCSProperties) -> Self {
        let mut spectre_props = Self::default();
        spectre_props.sync(sesh);
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
            Ok(playback_type) => playback_type.Value().unwrap(),
            Err(_) => MPT::Unknown,
        };
        self.subtitle = match properties.Subtitle() {
            Ok(subtitle) => Some(subtitle.to_string()),
            Err(_) => None,
        };

    }
}

/// Implements the `IntoIterator` trait for `SpectreProps`, allowing it to be iterated over as a collection of key-value pairs.
/// 
/// The iterator yields tuples of `(String, Option<String>)`, where the `String` represents the property name and the `Option<String>`
/// represents the property value. This allows the `SpectreProps` struct to be easily converted to a collection of its properties,
/// which can be useful for tasks like serialization or display.
///
/// The properties included in the iterator are:
/// - "Title"
/// - "Artist"
/// - "Album"
/// - "AlbumArtist"
/// - "Genres" (a comma-separated string of genres)
/// 
/// Additional properties can be added to the iterator as needed.
impl IntoIterator for SpectreProps {
    type Item = (String, Option<String>);
    type IntoIter = indexmap::map::IntoIter<String, Option<String>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut map = IndexMap::new();
        map.insert("Title".to_string(), Some(self.title));
        map.insert("Artist".to_string(), Some(self.artist));
        map.insert("Album".to_string(), Some(self.album));
        map.insert("AlbumArtist".to_string(), self.album_artist);
        map.insert("Genres".to_string(), Some(self.genres.join(", ")));
        // Add other fields here as needed
        map.into_iter()
    }
}