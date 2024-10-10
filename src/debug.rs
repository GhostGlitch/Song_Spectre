//debug functions, make sure to remove or exclude before making release build.

// FIND A WAY TO MARK WHOLE MOD AS DEBUG NOT JUST THE FUNCTIONS
use std::{ fs, io::{Cursor, Error}, process::{Command, Stdio}, result::Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::env;
use crate::props::*;
use crate::ghoast::*;
//shows a DynamicImage in a browser window and returns a string of the file location.
pub(crate) fn view_image_rgba8 (img: Option<&image::RgbaImage>) -> Result<String, Error> {
    let fuc = img.unwrap().clone();
    let dyna = DynamicImage::from(fuc);
    view_image(Some(&dyna), "rgba")
}
#[cfg(debug_assertions)]
pub(crate)  fn view_image(img: Option<&DynamicImage>, title: &str) -> Result<String, Error> {
    if let Some(img) = img {
        let mut cursor = Cursor::new(Vec::new());
        let _ = img.write_to(&mut cursor, image::ImageFormat::Png);

        let base64_str = STANDARD.encode(cursor.get_ref());
        let html = format!("<img src='data:image\\png;base64,{}' />", base64_str);

        // Define and create the temporary directory if it does not exist
        let temp_dir = env::temp_dir().join("GhostGlitch\\Spectre");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)?;
        }

        let html_file = temp_dir.join(format!(
            "Spectre-{}-thumb.html",
            title.replace(" ", "_").replace("\"", "''")
            .replace("&", "[AND]")  
            .replace("?", "[QU]")  
            .replace("/", "[FSL]") 
            .replace("\\", "[BSL]") 
            .replace(":", "[COL]") 
            .replace("*", "[AST]") 
            .replace("|", "[PIP]") 
            .replace("<", "[LAB]") 
            .replace(">", "[RAB]") 
        ));
        fs::write(&html_file, html)?;

        Command::new("cmd")
            .args(["/C", "start", &html_file.to_string_lossy()])
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
#[cfg(debug_assertions)]
pub(crate) fn display_spec_props(spec_props: &SpectreProps) -> Result<(), Error> {
    let thumb_file = view_image(Some(&spec_props.thumbnail), &spec_props.title)?;
    print!("{}", spec_props);
    println!("Thumbnail: {thumb_file}");
    println!();
    Ok(())
}
#[cfg(debug_assertions)]
pub(crate) fn cls() {
    // Clear the console using the 'Clear-Host' command in PowerShell
    let _ = std::process::Command::new("powershell")
        .arg("-Command")
        .arg("Clear-Host")
        .output();
}

#[cfg(debug_assertions)]
pub(crate)  fn show_ghoast(title: &str, props: SpectreProps) -> Ghoast {
    let toast_window = Ghoast::new(title, props);
    toast_window.init();
    print!("new {:?}", toast_window.c_name);
    print!(" named {:?}", toast_window.title);
    print!(" | WinID: {:?}", toast_window.hwnd);
    println!(" | H Inst: {:?}", toast_window.h_instance);
    //println!(" Class: {:?}", toast_window.t_inst.class);
    println!();

    toast_window
}

use windows::Win32::{Foundation::GetLastError, Graphics::Gdi::{GetObjectW, GetStockObject, BITMAP, BITMAPINFO, DIB_RGB_COLORS, HBITMAP, HDC, OBJ_BITMAP}};
use std::ptr;
pub trait DbgStrExt {
    fn indent(&self, indent: u8) -> String;
}
impl DbgStrExt for str {
    fn indent(&self, indent: u8) -> String {
        let indent_str = " ".repeat((indent * 4) as usize); // 4 spaces per indent level
        format!("{}{}", indent_str, self)
    }
}

pub trait DbgStringExt {
    fn push_ln(&mut self, str: &str);
    fn push_ln_in(&mut self, str: &str, indent: u8);
}
impl DbgStringExt for String {
    fn push_ln(&mut self, str: &str) {
        self.push_ln_in(str, 0);
    }
    fn push_ln_in(&mut self, str: &str, indent: u8) {
        if !self.is_empty() {
            self.push_str("\n");
        }
        self.push_str(&str.indent(indent));
    }
}
pub fn check_hbitmap(h_bitmap: HBITMAP, mut bmi: BITMAPINFO,  hdc: HDC, width: u32, height: u32, depth: u32) -> Result<String, String> {
    if h_bitmap.is_invalid() {
        // Get the last error if the bitmap creation failed
        let error_code = unsafe { GetLastError() };
        return Err(format!("Failed to create bitmap. Win32 Error: {}", error_code.0));
    }

    let mut fucked = false;

    let mut bmp_inf: BITMAP = BITMAP::default();         // Allocate space for a BITMAP structure
    let mut error_string: String = String::new();


    let get_object_results = unsafe {
        GetObjectW(
            h_bitmap,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp_inf as *mut _ as *mut std::ffi::c_void),
        )
    };
    let mut bmp_bits: Vec<u8> = vec![0; (bmp_inf.bmWidth * bmp_inf.bmHeight * 4) as usize]; // Buffer for bitmap data
    let get_bits_result = unsafe {
        windows::Win32::Graphics::Gdi::GetDIBits(
            hdc,
            h_bitmap,
            0,
            bmp_inf.bmHeight as u32,
            Some(bmp_bits.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bmi,
            DIB_RGB_COLORS
        )
    };

    // Check if GetDIBits was successful
    if get_bits_result == 0 {
        error_string.push_ln_in("Failed to get bits", 1);
        fucked = true;
    }
    if bmp_bits.iter().all(|&byte| byte == 0) {
        error_string.push_ln_in("Bitmap data is zeroes", 1);
        fucked = true;
    }

    if get_object_results == 0 {
        error_string.push_ln_in("Failed to get bitmap information", 1);
        return Err(error_string);
    } else {
        // Check the bitmap's properties (e.g., dimensions and bit depth)
        if bmp_inf.bmWidth != width as i32|| bmp_inf.bmHeight != height as i32 {
            error_string.push_ln_in("Bad dimensions:", 1);
            if bmp_inf.bmWidth != width as i32 {
                error_string.push_ln_in(&format!("Width: Expected {}, Found {}", width, bmp_inf.bmWidth), 2);
            }
            if bmp_inf.bmHeight != height as i32{
                error_string.push_ln_in(&format!("Height: Expected {}, Found {}", height, bmp_inf.bmHeight), 2);
            }
            fucked = true;
        }
        if bmp_inf.bmBitsPixel != depth as u16 {
            error_string.push_ln_in("Bad bit depth:", 1);
            error_string.push_ln_in(&format!("Expected {}, Found {}", depth, bmp_inf.bmBitsPixel), 2);
            fucked = true;
        }
    }
    if fucked {
        Err(format!("Problem with bitmap:\n{}", error_string))
    } else {
        Ok(format!("Bitmap is well-formed \n    BitSize: {} Bitmap Width: {}, Height: {}, BitDepth: {}", bmp_bits.len(), bmp_inf.bmWidth, bmp_inf.bmHeight, bmp_inf.bmBitsPixel))
    }
}
