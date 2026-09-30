//! Fullscreen transparent click-through overlay window.
//! Uses WS_EX_LAYERED + WS_EX_TRANSPARENT + WS_EX_TOPMOST for
//! a window that floats above everything but passes all input through.

use std::sync::atomic::{AtomicIsize, Ordering};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

use crate::types::Bitmap;

/// Global handle to the overlay window (for use by hook callbacks)
pub static OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);

/// Custom message IDs
pub const WM_KEY_ACTION: u32 = WM_USER + 100;
pub const WM_TRAY_CALLBACK: u32 = WM_USER + 101;

pub struct OverlayWindow {
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,
    mem_dc: HDC,
    mem_bmp: HBITMAP,
    old_bmp: HGDIOBJ,
    bits_ptr: *mut u8,
}

impl OverlayWindow {
    /// Create the overlay window covering the entire primary monitor
    pub unsafe fn create(wndproc: WNDPROC) -> windows::core::Result<Self> {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = w!("PresenterKitOverlay");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES::default(),
            lpfnWndProc: wndproc,
            hInstance: hinstance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST
            | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            w!("PresenterKit"),
            WS_POPUP,
            0, 0, screen_w, screen_h,
            None,
            None,
            hinstance,
            None,
        )?;

        OVERLAY_HWND.store(hwnd.0 as isize, Ordering::Relaxed);

        // Create memory DC and DIB section for UpdateLayeredWindow
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);

        let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: screen_w,
                biHeight: -screen_h, // top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let mem_bmp = CreateDIBSection(
            mem_dc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        )?;
        let old_bmp = SelectObject(mem_dc, mem_bmp);
        ReleaseDC(None, screen_dc);

        // Show the window
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        Ok(Self {
            hwnd,
            width: screen_w,
            height: screen_h,
            mem_dc,
            mem_bmp,
            old_bmp,
            bits_ptr: bits_ptr as *mut u8,
        })
    }

    /// Copy bitmap data to the DIB section and call UpdateLayeredWindow
    pub unsafe fn present(&self, bitmap: &Bitmap) {
        // Copy our rendered bitmap to the DIB section memory
        let size = (self.width * self.height * 4) as usize;
        std::ptr::copy_nonoverlapping(
            bitmap.data.as_ptr(),
            self.bits_ptr,
            size,
        );

        let pos = POINT { x: 0, y: 0 };
        let size = SIZE { cx: self.width, cy: self.height };
        let src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            self.hwnd,
            None,
            Some(&pos),
            Some(&size),
            self.mem_dc,
            Some(&src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
    }

    /// Show the overlay
    pub unsafe fn show(&self) {
        let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
    }

    /// Hide the overlay
    pub unsafe fn hide(&self) {
        let _ = ShowWindow(self.hwnd, SW_HIDE);
    }

    /// Enable or disable click-through.
    /// When interactive (e.g., dragging magnifier), disable click-through.
    pub unsafe fn set_click_through(&self, click_through: bool) {
        let current = GetWindowLongW(self.hwnd, GWL_EXSTYLE) as u32;
        let new = if click_through {
            current | WS_EX_TRANSPARENT.0
        } else {
            current & !WS_EX_TRANSPARENT.0
        };
        SetWindowLongW(self.hwnd, GWL_EXSTYLE, new as i32);
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.mem_dc, self.old_bmp);
            let _ = DeleteObject(self.mem_bmp);
            let _ = DeleteDC(self.mem_dc);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
