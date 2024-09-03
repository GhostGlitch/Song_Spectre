//debug functions, make sure to remove or exclude before making release build.

// FIND A WAY TO MARK WHOLE MOD AS DEBUG NOT JUST THE FUNCTIONS
use std::{ fs, io::{Cursor, Error}, process::{Command, Stdio}, result::Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::DynamicImage;
use std::env;
//shows a DynamicImage in a browser window abd returns a string of the file location.
#[cfg(debug_assertions)]
pub(crate)  fn view_image(img: Option<DynamicImage>, title: &str) -> Result<String, Error> {
    if let Some(img) = img {
        let mut cursor = Cursor::new(Vec::new());
        let _ = img.write_to(&mut cursor, image::ImageFormat::Png);

        let base64_str = STANDARD.encode(cursor.get_ref());
        let html = format!("<img src='data:image/png;base64,{}' />", base64_str);

        // Define and create the temporary directory if it does not exist
        let temp_dir = env::temp_dir().join("GhostGlitch/Spectre");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)?;
        }

        let html_file = temp_dir.join(format!(
            "Spectre-{}-thumb.html",
            title.replace(" ", "_").replace("\"", "''")
            .replace("&", "＆")  // Fullwidth Ampersand for '&'
            .replace("?", "？")  // Fullwidth Question Mark for '?'
            .replace("/", "⁄") // Fraction slash as a replacement for '/'
            .replace("\\", "⧵") // Reverse solidus for '\'
            .replace(":", "ː") // Modifier letter colon for ':'
            .replace("*", "∗") // Asterisk operator for '*'
            .replace("|", "¦") // Broken bar for '|'
            .replace("<", "⟨") // Mathematical angle bracket for '<'
            .replace(">", "⟩") // Mathematical angle bracket for '>'
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
/// Simulates a failure by returning an `std::io::Error`.
///
/// For verifying error handling by triggering errors anywhere easily.
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub(crate) fn sim_error() -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Simulated failure"))
}