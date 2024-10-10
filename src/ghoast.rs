use image::{DynamicImage, GenericImageView};
use windows::{core::{w, PCWSTR}, 
    Win32::{Foundation::{self as WFound, 
            HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            self, InvalidateRect, RedrawWindow, UpdateWindow, HBRUSH, HRGN}, 
        System::LibraryLoader::GetModuleHandleA, 
        UI::WindowsAndMessaging::{self as WandM, 
            CreateWindowExW, DestroyWindow, DispatchMessageW, GetLayeredWindowAttributes, GetMessageW, GetWindowLongPtrW, PostQuitMessage, RegisterClassW, SendMessageW, SetLayeredWindowAttributes, SetWindowLongPtrW, ShowWindow, TranslateMessage, HCURSOR, MSG, WNDCLASSW}}};
//I do not know why this glob import is necessary. but without it the window behaves incorrectly despite the compiler being happy.
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::{debug};
use crate::props::*;
use std::time::Duration;
use std::{sync::{Arc, Once}, thread};

static mut TOAST_INSTANCE: Option<Arc<GhoastClass>> = None;
static INITIALIZE_ONCE: Once = Once::new(); // Once to ensure Toast is initialized only once

unsafe extern "system" fn custom_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let hdc = Gdi::GetDC(hwnd);
            if hdc.is_invalid() {
                println!("Failed to get device context.");
                Gdi::DeleteDC(hdc);
                return LRESULT(0);
            } 
            // Create a memory device context
            let mem_dc = Gdi::CreateCompatibleDC(hdc);
            if mem_dc.0.is_null() {
                println!("Failed to create memory device context.");
                Gdi::DeleteDC(hdc);
                return LRESULT(0);
            }
            let thumbnail_ptr = unsafe { GetWindowLongPtrW(hwnd, WandM::GWLP_USERDATA) as *const DynamicImage };
            if !thumbnail_ptr.is_null() {
                let thumb = unsafe { &*thumbnail_ptr };
            
            //debug::view_image(Some(&var), "Ghoast");
            let bitmap = match dynamic_image_to_bitmap(hdc, thumb) {
                Ok(bmp) => bmp, // If successful, assign to bitmap
                Err(e) => {
                    println!("{}", e);
                    Gdi::DeleteDC(mem_dc);
                    Gdi::DeleteDC(hdc);
                    return LRESULT(0); // Return early on error
                }
            };

            // Select the bitmap into the device context
            Gdi::SelectObject(mem_dc, bitmap);

            // Use the dimensions of the image for the BitBlt
            let (width, height) = thumb.dimensions();
            // Draw the bitmap on the window
            let blit_result = Gdi::BitBlt(
                hdc,             // Destination device context
                0, 0, // Destination coordinates
                width as i32, height as i32, // Width and height of the bitmap
                mem_dc,            // Source device context
                0, 0,          // Source coordinates (from the bitmap)
                Gdi::SRCCOPY,
            );
            if blit_result.is_err() {
                println!("Failed to draw bitmap.");
                println!("{:?}", blit_result)
            } else {
                println!("Bitmap drawn successfully.");
            }

            Gdi::DeleteObject(bitmap); // Delete the bitmap object
            Gdi::DeleteDC(mem_dc); // Delete the memory DC
            Gdi::DeleteDC(hdc);
            return LRESULT(0);
        } else {
            print!("thumbnail ptr null");
            return LRESULT(1);
        }
    }
        WM_CLOSE => {
            DestroyWindow(hwnd); // Destroy the window
            println!("CLOSE");
            LRESULT(0) // Indicate the message was handled
        }
        WM_DESTROY => {
            // Post a quit message to the message queue
            PostQuitMessage(0);
            println!("DESTROY");
            LRESULT(0) // Indicate the message was handled
        }
        _ => WandM::DefWindowProcW(hwnd, msg, wparam, lparam), // Default handling
    }
}

use windows::Win32::Foundation::COLORREF;

pub fn make_color_ref(r: u8, g: u8, b: u8) -> COLORREF {
    // Combine the RGB components into a COLORREF value
    let color = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
    COLORREF(color)
}
#[derive(Debug)]
pub struct GhoastClass {
    pub class: WNDCLASSW,
    pub atom: u16,
    pub h_instance: HINSTANCE,
}
impl GhoastClass {
    
    pub fn new() -> Option<Self> {
        let name = w!("Ghoast");
        let h_instance = unsafe { GetModuleHandleA(None).unwrap_or_default().into() };
        let class = {
            WNDCLASSW {
                lpszClassName: name,
                lpfnWndProc: Some(custom_window_proc),
                hInstance: h_instance,
                hbrBackground: HBRUSH::default(), //GetSysColorBrush(COLOR_BACKGROUND),
                hCursor: HCURSOR::default(),
                ..Default::default()
            }
        };
        let atom = unsafe { RegisterClassW(&class) };
        //Needs To Be Propper Error eventually.
        if atom == 0 {
            println!("Window class registration failed.");
            thread::sleep(Duration::from_secs(1));
            return None;
        }
        Some(Self { class, atom, h_instance})
    }
    pub fn instance() -> Arc<GhoastClass> { unsafe {
            INITIALIZE_ONCE.call_once(|| {
                TOAST_INSTANCE = Some(Arc::new(GhoastClass::new().unwrap()));
            });
            TOAST_INSTANCE.clone().expect("Failed to get Toast instance")
        }
    }
}
#[derive(Debug)]
pub struct Ghoast {
    pub hwnd: HWND,
    pub h_instance: HINSTANCE,
    pub c_name: String,
    pub is_good: bool,
    pub title: String,
    pub props: SpectreProps,
}
impl Ghoast {
    pub fn new(title: &str, props: SpectreProps) -> Self {
        let inst = GhoastClass::instance();
        let name = inst.class.lpszClassName;
            // Create the window using the registered class
            let hwnd = unsafe {
                CreateWindowExW(
                    WandM::WS_EX_TOPMOST | WandM::WS_EX_TRANSPARENT |
                    WandM::WS_EX_LAYERED | WandM::WS_EX_NOACTIVATE,
                    name,
                    PCWSTR::from_raw(title.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr()),
                    WandM::WS_POPUP,
                    WandM::CW_USEDEFAULT, WandM::CW_USEDEFAULT,
                    300, 300, 
                    HWND::default(), // Parent window
                    None, // Menu
                    inst.h_instance, // Instance handle
                    None, // Additional data
                )
            }.unwrap();
        let thumbnail_ptr = Box::into_raw(Box::new(props.thumbnail.clone()));
        unsafe { SetWindowLongPtrW(hwnd, WandM::GWLP_USERDATA, thumbnail_ptr as _) };
        Self { hwnd , h_instance: inst.h_instance, c_name: unsafe { name.to_string().unwrap_or_default() }, is_good: true, title: title.to_string(), props}
    }    // Method to show the window
    pub fn init(&self) {
            self.show();
            self.update();
            self.set_transparency(make_color_ref(126, 126, 126), 126);
            self.check_messages();          
    }  
    fn check_messages(&self)->bool {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            if  GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                true
            } else {
                false
            }
        }
    }
    pub fn message_loop(&mut self)->bool {
        let boo = self.check_messages();
        if !boo { self.is_good = false;}
        boo
    }
    pub fn fade_out(&mut self, seconds: f32) -> bool {
        let cref = make_color_ref(126, 126, 126);
        let mut alpha = self.get_current_alpha().unwrap();
        let dur = Duration::from_secs_f32(seconds/alpha as f32);
        while self.message_loop() {
            alpha -= 1;
            println!("{}", alpha);
            if alpha < 1 {
                self.destruct();
                break;
            } else {
            let _ = self.set_transparency(cref, alpha);
            self.redraw();
            thread::sleep(dur);
        }}
        return false;
    }

    pub fn destruct(&mut self) {
        // Send the WM_CLOSE message to the window
        self.message_self(WandM::WM_CLOSE);
        while self.check_messages() { 
            println!("SUICIDE")
        }
        self.is_good = false;
    }

    fn show(&self) -> bool{
        unsafe {ShowWindow(self.hwnd, WandM::SW_SHOW)}.into()
    }
    fn update(&self) -> bool{
        unsafe {UpdateWindow(self.hwnd)}.into()
    }
    fn set_transparency(&self, crkey: COLORREF, alpha: u8) -> Result<(), windows::core::Error> {
        unsafe {SetLayeredWindowAttributes(self.hwnd, crkey, alpha, WandM::LWA_ALPHA)}
    }
    pub fn redraw(&self) -> bool{
        unsafe { RedrawWindow(self.hwnd, None, HRGN::default(), Gdi::RDW_INVALIDATE | Gdi::RDW_ALLCHILDREN) }.into()
    }
    pub fn message_self(&self, msg: u32) -> LRESULT {
        unsafe {SendMessageW(self.hwnd, msg, WPARAM(0), LPARAM(0))}
    }
    pub fn request_paint(&self) {
        unsafe {
            // Invalidate the window's client area
            InvalidateRect(self.hwnd, None, WFound::TRUE);

            // Update the window to send a WM_PAINT message immediately
            UpdateWindow(self.hwnd);
        }
    }
    pub fn get_current_alpha(&self) -> Option<u8> {
        let mut alpha: u8 = 0;
        let mut crkey: COLORREF = COLORREF(0);
    
        // Call GetLayeredWindowAttributes to retrieve current attributes
        if unsafe { GetLayeredWindowAttributes(self.hwnd, None, Some(&mut alpha), None).is_ok() } {
            Some(alpha)
        } else {
            None
        }
    }
}



