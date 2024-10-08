use image::DynamicImage;
use image::GenericImageView;
use windows::core::PCWSTR;
use windows::Win32::Foundation;
use windows::Win32::Graphics::Gdi::GetSysColorBrush;
use windows::Win32::Graphics::Gdi::SYS_COLOR_INDEX;
use windows::Win32::UI::WindowsAndMessaging;
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32;
use windows as win;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::*;
use crate::debug;
use crate::dynamic_image_to_bitmap;
use crate::w;
use crate::SpectreProps;
use crate::ERROR_THUMB;
use core::time;
use std::f32::consts::E;
//////////////////////////////////////////////////////////////////
use std::sync::Arc;
use std::sync::Once;
use std::thread;
static mut TOAST_INSTANCE: Option<Arc<GhoastClass>> = None;
static INITIALIZE_ONCE: Once = Once::new(); // Once to ensure Toast is initialized only once
////////////////////////////////////////////////////////////////////////////////////////////

use windows::Win32::UI::WindowsAndMessaging::WNDCLASSW;
unsafe extern "system" fn custom_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps); // Begin painting
            if hdc.is_invalid() {
                println!("Failed to get device context.");
                EndPaint(hwnd, &ps);
                return LRESULT(0);
            } 
            // Create a memory device context
            let mem_dc = CreateCompatibleDC(hdc);
            if mem_dc.0.is_null() {
                println!("Failed to create memory device context.");
                EndPaint(hwnd, &ps);
                return LRESULT(0);
            }
            let thumbnail_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const DynamicImage };
            if !thumbnail_ptr.is_null() {
                let thumb = unsafe { &*thumbnail_ptr };
            
            //debug::view_image(Some(&var), "Ghoast");
            let bitmap = match dynamic_image_to_bitmap(hdc, thumb) {
                Ok(bmp) => bmp, // If successful, assign to bitmap
                Err(e) => {
                    println!("{}", e);
                    DeleteDC(mem_dc);
                    EndPaint(hwnd, &ps);
                    return LRESULT(0); // Return early on error
                }
            };

            // Select the bitmap into the device context
            let old_bitmap: HGDIOBJ = SelectObject(mem_dc, bitmap);

            // Use the dimensions of the image for the BitBlt
            let (width, height) = ERROR_THUMB.dimensions();
            let dest_x = 0;
            let dest_y = 0;

            // Draw the bitmap on the window
            let blit_result = BitBlt(
                hdc,             // Destination device context
                dest_x, dest_y, // Destination coordinates
                width as i32, height as i32, // Width and height of the bitmap
                mem_dc,            // Source device context
                0, 0,          // Source coordinates (from the bitmap)
                SRCCOPY,
            );
            if blit_result.is_err() {
                println!("Failed to draw bitmap.");
                println!("{:?}", blit_result)
            } else {
                //println!("Bitmap drawn successfully.");
            }

            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap); // Delete the bitmap object
            DeleteDC(mem_dc); // Delete the memory DC
            EndPaint(hwnd, &ps); // End painting
            return LRESULT(0);
        } else {
            return LRESULT(0);
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
        _ => DefWindowProcW(hwnd, msg, wparam, lparam), // Default handling
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
            thread::sleep(time::Duration::from_secs(1));
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
    pub t_inst: Arc<GhoastClass>,
    pub is_good: bool,
    pub title: String,
    pub props: SpectreProps,
    pub thumb: DynamicImage
}
impl Ghoast {
    pub fn new(title: &str, props: SpectreProps) -> Self {
        let inst = GhoastClass::instance();
        let name = inst.class.lpszClassName;
            // Create the window using the registered class
            let hwnd = unsafe {
                CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TRANSPARENT |
                    WS_EX_LAYERED | WS_EX_NOACTIVATE,
                    name,
                    PCWSTR::from_raw(title.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr()),
                    WS_VISIBLE | WS_POPUP,
                    CW_USEDEFAULT, CW_USEDEFAULT,
                    300, 300, 
                    HWND::default(), // Parent window
                    None, // Menu
                    inst.h_instance, // Instance handle
                    None, // Additional data
                )
            }.unwrap();

        let dumb  = props.clone();
        let thumb = props.thumbnail.clone();
        let window = Self { hwnd , h_instance: inst.h_instance, c_name: unsafe { name.to_string().unwrap_or_default() }, t_inst: inst, is_good: true, title: title.to_string(), props, thumb};
        let thumbnail_ptr = Box::into_raw(Box::new(dumb.thumbnail));
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, thumbnail_ptr as _) };
        window
    }    // Method to show the window
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
            SetLayeredWindowAttributes(self.hwnd, make_color_ref(126, 126, 126), 125, LWA_ALPHA);
            self.inloop();           
        }
    }
    fn inloop(&self)->bool {
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
    pub fn mloop(&mut self)->bool {
        let boo = self.inloop();
        if !boo { self.is_good = false;}
        boo
    }
    pub fn loopyloop(&mut self) -> bool {
        while self.mloop() {
            println!("loopyloop");
            thread::sleep(time::Duration::from_secs_f32(0.1));
        }
        println!("loopyloop fin");
        return false;
    }
    pub fn dbg_self_destruct(&mut self) {
        unsafe {
            // Send the WM_CLOSE message to the window
            SendMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            while self.is_good {
                self.mloop();   
                println!("SUICIDE")
            }
        }
    }
    pub fn request_paint(&self) {
        unsafe {
            // Invalidate the window's client area
            InvalidateRect(self.hwnd, None, TRUE);

            // Update the window to send a WM_PAINT message immediately
            UpdateWindow(self.hwnd);
        }
    }
}



