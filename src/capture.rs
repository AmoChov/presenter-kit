//! Screen capture via Win32 BitBlt.
//! Captures rectangular regions of the desktop into our Bitmap type.

use windows::Win32::Graphics::Gdi::*;

use crate::types::{Bitmap, Rect};

pub struct ScreenCapture;

impl ScreenCapture {
    /// Capture a rectangular region of the screen into a Bitmap.
    /// The Bitmap uses BGRA format with premultiplied alpha (alpha = 255).
    pub unsafe fn capture_region(rect: Rect) -> Option<Bitmap> {
        if rect.w <= 0 || rect.h <= 0 {
            return None;
        }

        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            return None;
        }

        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.is_invalid() {
            ReleaseDC(None, screen_dc);
            return None;
        }

        // Create a DIB section so we can read the pixel data
        let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: rect.w,
                biHeight: -rect.h, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let hbmp = CreateDIBSection(
            mem_dc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        );

        let hbmp = match hbmp {
            Ok(bmp) => bmp,
            Err(_) => {
                let _ = DeleteDC(mem_dc);
                ReleaseDC(None, screen_dc);
                return None;
            }
        };

        let old = SelectObject(mem_dc, hbmp);

        // BitBlt from screen to our DIB
        let _ = BitBlt(
            mem_dc,
            0, 0, rect.w, rect.h,
            screen_dc,
            rect.x, rect.y,
            SRCCOPY,
        );

        // Copy bits to our Bitmap
        let byte_count = (rect.w * rect.h * 4) as usize;
        let mut bitmap = Bitmap::new(rect.w, rect.h);
        std::ptr::copy_nonoverlapping(
            bits_ptr as *const u8,
            bitmap.data.as_mut_ptr(),
            byte_count,
        );

        // BitBlt gives us BGRX (alpha = 0). Set alpha to 255 for premultiplied.
        for i in (3..bitmap.data.len()).step_by(4) {
            bitmap.data[i] = 255;
        }

        // Cleanup
        SelectObject(mem_dc, old);
        let _ = DeleteObject(hbmp);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);

        Some(bitmap)
    }

    /// Capture the entire screen
    pub unsafe fn capture_full_screen(w: i32, h: i32) -> Option<Bitmap> {
        Self::capture_region(Rect::new(0, 0, w, h))
    }
}
