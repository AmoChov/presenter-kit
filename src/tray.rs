//! System tray icon management.
//! Shows a tray icon with a context menu for accessing settings and quitting.

use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

use crate::overlay::WM_TRAY_CALLBACK;

const TRAY_ICON_ID: u32 = 1;

pub struct TrayIcon {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    /// Create and show the system tray icon
    pub unsafe fn create(hwnd: HWND) -> windows::core::Result<Self> {
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_ICON_ID,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAY_CALLBACK,
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            ..Default::default()
        };

        // Set tooltip text
        let tip = "PresenterKit — Press ScrollLock for Spotlight";
        let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
        let len = tip_wide.len().min(nid.szTip.len());
        nid.szTip[..len].copy_from_slice(&tip_wide[..len]);

        Shell_NotifyIconW(NIM_ADD, &nid).ok()?;

        Ok(Self { hwnd, nid })
    }

    /// Handle tray icon callback messages
    pub unsafe fn handle_message(&self, lparam: LPARAM) {
        let event = (lparam.0 & 0xFFFF) as u32;
        match event {
            WM_RBUTTONUP => {
                self.show_context_menu();
            }
            WM_LBUTTONDBLCLK => {
                // Double-click: show welcome/help
                // TODO: Show welcome window
            }
            _ => {}
        }
    }

    /// Show the right-click context menu
    unsafe fn show_context_menu(&self) {
        let menu = CreatePopupMenu().unwrap_or_default();
        let _ = AppendMenuW(menu, MF_STRING, 1001, w!("About PresenterKit"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, 1002, w!("Quit"));

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);

        // Required to make the menu disappear when clicking elsewhere
        SetForegroundWindow(self.hwnd);

        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            pt.x, pt.y,
            0,
            self.hwnd,
            None,
        );

        let _ = DestroyMenu(menu);

        if cmd.0 != 0 {
            match cmd.0 as u32 {
                1001 => {
                    // Show about / welcome
                    let _ = MessageBoxW(
                        Some(self.hwnd),
                        w!("PresenterKit v1.0\n\nLightweight presenter overlay for teachers.\n\nHotkeys:\nScrollLock - Spotlight\nHome - Circle Magnifier\nInsert - Freeze Region\nEnd - Freeze Half Screen\nPage Up/Down - Resize\nDelete - Remove Last\nEscape - Clear All"),
                        w!("PresenterKit"),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                1002 => {
                    // Quit
                    PostQuitMessage(0);
                }
                _ => {}
            }
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
    }
}
