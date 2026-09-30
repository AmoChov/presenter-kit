//! Low-level keyboard hook for intercepting presenter hotkeys.
//! Uses SetWindowsHookExW(WH_KEYBOARD_LL) to catch key events globally.
//! Posts custom WM_KEY_ACTION messages to the overlay window.

use std::sync::atomic::Ordering;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::overlay::{OVERLAY_HWND, WM_KEY_ACTION};

/// Virtual key codes for our hotkeys
pub const VK_SPOTLIGHT: u32 = VK_SCROLL.0 as u32;    // ScrollLock
pub const VK_CIRCLE_MARKER: u32 = VK_HOME.0 as u32;  // Home
pub const VK_FREEZE_REGION: u32 = VK_INSERT.0 as u32; // Insert
pub const VK_FREEZE_HALF: u32 = VK_END.0 as u32;     // End
pub const VK_SIZE_UP: u32 = VK_PRIOR.0 as u32;       // Page Up
pub const VK_SIZE_DOWN: u32 = VK_NEXT.0 as u32;      // Page Down
pub const VK_REMOVE_LAST: u32 = VK_DELETE.0 as u32;  // Delete
pub const VK_CLEAR_ALL: u32 = VK_ESCAPE.0 as u32;    // Escape

/// Packed key action: high word = vkCode, low word = 1 for keydown, 0 for keyup
pub fn pack_key_action(vk: u32, is_down: bool) -> usize {
    ((vk as usize) << 16) | (is_down as usize)
}

pub fn unpack_key_action(wparam: usize) -> (u32, bool) {
    let vk = (wparam >> 16) as u32;
    let is_down = (wparam & 1) != 0;
    (vk, is_down)
}

fn is_our_key(vk: u32) -> bool {
    matches!(vk,
        VK_SPOTLIGHT | VK_CIRCLE_MARKER | VK_FREEZE_REGION | VK_FREEZE_HALF |
        VK_SIZE_UP | VK_SIZE_DOWN | VK_REMOVE_LAST | VK_CLEAR_ALL
    )
}

/// The low-level keyboard hook callback
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = kbd.vkCode;

        if is_our_key(vk) {
            let hwnd_val = OVERLAY_HWND.load(Ordering::Relaxed);
            if hwnd_val != 0 {
                let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
                let is_down = wparam.0 == WM_KEYDOWN as usize
                    || wparam.0 == WM_SYSKEYDOWN as usize;

                let _ = PostMessageW(
                    hwnd,
                    WM_KEY_ACTION,
                    WPARAM(pack_key_action(vk, is_down)),
                    LPARAM(0),
                );

                // Eat the key — prevent ScrollLock LED toggle, prevent
                // Home/End from interfering with Photoshop text editing.
                // Exception: Escape is passed through (too important to eat)
                if vk != VK_CLEAR_ALL {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

pub struct KeyboardHook {
    hook: HHOOK,
}

impl KeyboardHook {
    /// Install the global low-level keyboard hook
    pub unsafe fn install() -> windows::core::Result<Self> {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)?;
        Ok(Self { hook })
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWindowsHookEx(self.hook);
        }
    }
}
