use windows::{core::*, Data, 
    Storage::Streams::{IRandomAccessStreamReference as StreamRef, *},
    Media::{ MediaPlaybackType as MPT, 
        Control::{
            GlobalSystemMediaTransportControlsSession as TCS, GlobalSystemMediaTransportControlsSessionManager as TCSManager, GlobalSystemMediaTransportControlsSessionMediaProperties as TCSProperties, *}}};

use std::{fs, result::Result, path::Path,
    process::{Command, Stdio},
    io::{Error, Cursor}};
    
use futures::executor::block_on;
use image::{DynamicImage, ImageBuffer, Rgba};
use indexmap::IndexMap;
use base64::{engine::general_purpose::STANDARD, Engine as _};

async fn async_main() -> Result<TCSManager, Error> {
    let manager: TCSManager = TCSManager::RequestAsync()?.await?;
    Ok(manager)
}
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

fn ref_to_thumb(stream_ref: Option<StreamRef>) -> Result<DynamicImage, Error> {
    let magenta_color = Rgba([255, 0, 255, 255]);
    let mut img_buffer = ImageBuffer::<Rgba<u8>, _>::new(300, 300);

    for pixel in img_buffer.pixels_mut() {
        *pixel = magenta_color;
    }
    let error_thumb = DynamicImage::ImageRgba8(img_buffer);

    let stream = stream_ref.unwrap().OpenReadAsync()?.get()?;

    let stream_len = stream.Size()? as usize;
    let mut vec = vec![0u8; stream_len];
    let reader = DataReader::CreateDataReader(&stream)?;
    reader.LoadAsync(stream_len as u32)?.get()?;
    reader.ReadBytes(&mut vec)?;

    reader.Close().ok();
    stream.Close().ok();

    let img = match image::load_from_memory(&vec) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error loading image: {:?}", e);
            error_thumb
        }
    };
    let mut file = std::fs::File::create("refto.png")?;
    let _ = img.write_to(&mut file, image::ImageFormat::Png);
    Ok(img)
}
use tempfile::NamedTempFile;
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