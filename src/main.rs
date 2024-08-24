use windows::{core::*, Data, 
    Storage::Streams::{IRandomAccessStreamReference as StreamRef, *},
    Media::{ MediaPlaybackType as MPT, 
        Control::{
            GlobalSystemMediaTransportControlsSession as TCS, GlobalSystemMediaTransportControlsSessionManager as TCSManager, GlobalSystemMediaTransportControlsSessionMediaProperties as TCSProperties, *}}};

use std::{fs, result::Result, path::Path,
    process::{Command, Stdio},
    io::{Error, Cursor, ErrorKind}};
    
use futures::{executor::block_on, future::err};
use image::{DynamicImage, ImageBuffer, Rgba, Rgb};
use indexmap::IndexMap;
use base64::{engine::general_purpose::STANDARD, Engine as _};

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

        let tst = ref_to_thumb(thumb_copy);
        let thumb_file = debug_view_image(tst.ok(), &title_copy).unwrap();
        println!("Thumbnail: {thumb_file}");

        println!();
    }
    println!("------------------end------------------")
}

#[derive(Default)]
/// `SpectreProps` is a struct that holds all of the media metadata found in a 'TCSProperties', but in a more rusty way.
///
/// The `new()` and `new_async()` methods can be used to create new instances of the `SpectreProps` struct, while the `sync()` method can be used to update the properties of an existing instance based on the provided `TCSProperties`.
pub struct SpectreProps {
    title: String,
    artist: String,
    album: String,
    album_artist: Option<String>,
    genres: Vec<String>,
    thumbnail: Option<IRandomAccessStreamReference>,
    track_number: Option<i32>,
    track_count: Option<i32>,
    playback_type: MPT,
    subtitle: Option<String>,
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
            Ok(title) => title.to_string(),
            Err(_) => "Unknown Title".to_string(),
        };
        self.artist = match properties.Artist() {
            Ok(artist) => artist.to_string(),
            Err(_) => "Unknown Artist".to_string(),
        };
        self.album = match properties.AlbumTitle() {
            Ok(album) => album.to_string(),
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
        self.thumbnail = properties.Thumbnail().ok();
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

/// Creates a thumbnail image from a stream reference.

/// Creates a thumbnail image from a stream reference.
///
/// This function takes an optional `StreamRef` and returns a `DynamicImage` representing a thumbnail image. If the `StreamRef` is `None`, it creates a default pink-colored image. If the `StreamRef` is valid, it reads the stream data and attempts to load the image. If there is an error loading the image, it returns the default pink-colored image.
///
/// # Arguments
/// * `reference` - An optional `StreamRef` containing the image data.
///
/// # Returns
/// A `Result` containing a `DynamicImage` representing the thumbnail, or an `Error` if there was a problem that wasn't caught, otherwise returns a default pink image.
fn always_fail() -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Simulated failure"))
}
fn ref_to_thumb(reference: Option<StreamRef>) -> Result<DynamicImage, Error> {
    if let Some(img) = __ref_to_thumb(reference)
    {
        Ok(img)
    } else {
        let error_pink = Rgb([255, 0, 255]);
        let mut img_buffer = ImageBuffer::<Rgb<u8>, _>::new(300, 300);
        
        for pixel in img_buffer.pixels_mut() {
            *pixel = error_pink;
        }
        let img = DynamicImage::ImageRgb8(img_buffer);
        Ok(img)
    }
}
fn __ref_to_thumb(reference: Option<StreamRef>) -> Option<DynamicImage>  {
        let stream = reference?.OpenReadAsync().ok()?.get().ok()?; 
        let stream_len = stream.Size().ok()?;
        let mut img_data = vec![0u8; stream_len as usize];
        let reader= DataReader::CreateDataReader(&stream).ok()?; 
        reader.LoadAsync(stream_len as u32).ok()?.get().ok()?;
        reader.ReadBytes(&mut img_data).ok()?;
        match image::load_from_memory(&img_data) {
            Ok(new_img) => {
            let mut file = std::fs::File::create("refto.png").ok()?;
            let _ = new_img.write_to(&mut file, image::ImageFormat::Png);
            return Some(new_img)},
            Err(e) => {
                eprintln!("Error loading image: {:?}", e);
            }
        }    
    return None;
}
use tempfile::NamedTempFile;
/// Opens an image in a browser window for debug purposes.
///
/// This function takes an optional `DynamicImage` and a title string, and opens the image in a browser window for debugging purposes. If the `DynamicImage` is `None`, it prints a message indicating a null thumbnail.
///
/// # Arguments
/// * `img` - An optional `DynamicImage` to be displayed in the browser.
/// * `title` - A string title for the image.
///
/// # Returns
/// A `Result` containing the path to the generated HTML file, or an `Error` if there was a problem creating or opening the file.
fn debug_view_image(img: Option<DynamicImage>, title: &str) -> Result<String, Error> {
    if let Some(img) = img {
        let mut file = std::fs::File::create("debug_image.png")?;
        let _ = img.write_to(&mut file, image::ImageFormat::Png);
        let mut cursor = Cursor::new(Vec::new());
        let _ = img.write_to(&mut cursor, image::ImageFormat::Png);

        let base64_str = STANDARD.encode(cursor.get_ref());
        let html = format!("<img src='data:image/png;base64,{}' />", base64_str);

        let temp_dir = Path::new("C:/Users/ghost/AppData/Local/Temp/Spectre/");
        let html_file = temp_dir.join(format!(
            "Spectre-{}-thumb.html",
            title.replace(" ", "_").replace("&", "and")
        ));
        fs::write(&html_file, html)?;

        Command::new("cmd")
            .args(&["/C", "start", &html_file.to_string_lossy()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let var_name = html_file.to_str().unwrap().to_owned();
        Ok(var_name)
    } else {
        println!("{} NULL THUMB", title);
        // Beep() equivalent is not available in Rust
        Ok("None".to_string())
    }
}