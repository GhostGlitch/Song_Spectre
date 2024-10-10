use std::sync::LazyLock;
#[allow(unused_imports)]
use crate::debug;
use windows::Win32::{Foundation::GetLastError, 
    Graphics::Gdi::{
        CreateDIBitmap, BITMAPINFOHEADER, BI_RGB, CBM_INIT, RGBQUAD}};
use image::GenericImageView;
pub(crate) use windows::Win32::Graphics::Gdi::{
    HBITMAP, HDC, BITMAP, BITMAPINFO, DIB_RGB_COLORS};

pub(crate) const THUMB_W: u32 = 300;
pub(crate) const THUMB_H: u32 = 300;
const IMAGE_DATA: &[u8] = include_bytes!("error_thumb.png"); // Embed the PNG file at compile time
pub(crate) static ERROR_THUMB: LazyLock<DynamicImage> = LazyLock::new(|| {
    image::load_from_memory(IMAGE_DATA).unwrap_or_else(|_| DynamicImage::new_rgb8(THUMB_W, THUMB_H))
});


pub(crate) use img_traits::*;
mod img_traits{
pub(crate) use windows::Storage::Streams::IRandomAccessStreamReference as StreamRef;
pub(crate) use image::imageops::FilterType;
pub(crate) use image::DynamicImage;
    use std::io::{Error, ErrorKind}; 
    use windows::Storage::Streams::DataReader;
    use image::{ GenericImage, ImageResult, ImageError};
    use super::ERROR_THUMB;

    
    trait WinToImgErrExt<T> { fn map_err_img(self) -> Result<T, ImageError>; }

    impl<T> WinToImgErrExt<T> for windows::core::Result<T> {
        fn map_err_img(self) -> Result<T, ImageError> {
            self.map_err(|e| ImageError::IoError(e.into()))
    }} 

    pub(super) trait ImgExt {
        fn resize_centered(&self, nwidth: u32, nheight: u32, filter: FilterType) -> Self;
        fn from_stream_ref(reference: Option<StreamRef>) -> ImageResult<DynamicImage>;
    }

    impl ImgExt for DynamicImage {
        fn resize_centered(&self, nwidth: u32, nheight: u32, filter: FilterType) -> Self {
            let inner_image = self.resize(nwidth, nheight, filter);
            let x_offset = (nwidth - inner_image.width()) /2;
            let y_offset = (nheight - inner_image.height()) /2;
            // Create a new image with the target dimensions and a transparent background
            let mut output_image = Self::new_rgba8(nwidth, nheight);
            //returns either the properly resized image, 
            //or in the unlikely event inner image is somehow too large for the output image it returns a resized version of the error image.
            //may remove this error handling later. might be too both unlikely and indicitive enough that i've done something wrong to bother catching?
            if output_image.copy_from(&inner_image, x_offset, y_offset).is_err() {
                output_image = ERROR_THUMB.clone().resize_exact(nwidth, nheight, filter);
            }
            output_image
        }
        fn from_stream_ref(reference: Option<StreamRef>) -> ImageResult<DynamicImage> {
            let stream = reference.ok_or(ImageError::IoError(Error::new(ErrorKind::InvalidInput, "No Stream")))?.OpenReadAsync().map_err_img()?.get().map_err_img()?; 
            let stream_len = stream.Size().map_err_img()?;
            let mut img_data = vec![0u8; stream_len as usize];
            let reader = DataReader::CreateDataReader(&stream).map_err_img()?; 
            reader.LoadAsync(stream_len as u32).map_err_img()?.get().map_err_img()?;
            reader.ReadBytes(&mut img_data).map_err_img()?;
            let _ = reader.Close();
            let img = image::load_from_memory(&img_data)?;
            Ok(img)
        }
    }
} 




/// Creates a thumbnail image from a stream reference. If the stream reference is `None` or an error occurs, a default pink image is returned.
///
/// # Arguments
/// * `reference` - An optional `StreamRef` that contains the image data.
///
/// # Returns
/// A `DynamicImage` containing the thumbnail image, Or a placeholder image if something goes wrong.
pub fn ref_to_thumb(reference: Option<StreamRef>) -> DynamicImage {
    match DynamicImage::from_stream_ref(reference) {
        Ok(mut img) => { 
            if img.height() != THUMB_H || img.width() != THUMB_W { 
                img = img.resize_centered(THUMB_W, THUMB_H, FilterType::Lanczos3); 
            } img 
        },
        Err(_) => ERROR_THUMB.clone(),
    }
}

// Function to convert DynamicImage to a GDI bitmap
pub fn dynamic_image_to_bitmap(hdc: HDC, image: &DynamicImage) -> Result<HBITMAP, String> {
    let (width, height) = (image.dimensions());
    let image_data = image.to_rgba8(); // Convert to RGBA format
    
    // Prepare bitmap info header
    let mut bmi_h: BITMAPINFOHEADER = BITMAPINFOHEADER::default();
    bmi_h.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi_h.biWidth = width as i32;
    bmi_h.biHeight = -(height as i32); // Negative to indicate a top-down DIB
    bmi_h.biPlanes = 1;
    bmi_h.biBitCount = 32; // 32 bits for RGBA
    bmi_h.biCompression = BI_RGB.0;

    let bmi = Box::new(BITMAPINFO {
        bmiHeader: bmi_h,
        bmiColors: [RGBQUAD::default(); 1],
    });
    //let mut pb: *mut std::ffi::c_void = std::ptr::null_mut();

    let image_data_ptr: *const std::ffi::c_void = image_data.as_ptr() as *const std::ffi::c_void;
    let h_bitmap = unsafe {
        CreateDIBitmap(
            hdc,
            Some(&bmi.bmiHeader),
            CBM_INIT as u32,
            Some(image_data_ptr), // Use the padded image data
            Some(bmi.as_ref()), // Full bitmap info (including header)
            DIB_RGB_COLORS // Use RGB color data
        )
    };

    
    #[cfg(debug_assertions)]

    match debug::check_hbitmap(h_bitmap, *bmi, hdc, width, height, 32) {
       Ok(k) => print!(""),
       Err(e) => return Err(e),        
    };

    /* 
    match debug::check_hbitmap(h_bitmap, *bmi, hdc, width, height, 32) {
       Ok(k) => println!("{}", k),
       Err(e) => return Err(e),        
    };
    */
    
    
    if h_bitmap.is_invalid() {
        // Get the last error if the bitmap creation failed
        let error_code = unsafe { GetLastError() };
        return Err(format!("Failed to create bitmap. Error code: {}", error_code.0));
    }
    Ok(h_bitmap)

}