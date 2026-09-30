//! PresenterKit — Lightweight portable presenter overlay for teaching.
//!
//! No install, no EULA, just double-click and present.
//! Runs as a transparent overlay with spotlight, magnifier, freeze, and trail features.

#![windows_subsystem = "windows"]

mod types;
mod settings;
mod overlay;
mod hotkey;
mod capture;
mod features;
mod tray;


use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

use types::*;
use settings::Settings;
use overlay::OverlayWindow;
use hotkey::KeyboardHook;
use capture::ScreenCapture;
use tray::TrayIcon;

/// Timer ID for the render/update loop
const TIMER_RENDER: usize = 1;
const RENDER_INTERVAL_MS: u32 = 16; // ~60fps

/// Global app state — stored via SetWindowLongPtrW(GWLP_USERDATA)
struct App {
    state: AppState,
    overlay: OverlayWindow,
    bitmap: Bitmap,
    _hook: KeyboardHook,
    _tray: TrayIcon,
    timer_active: bool,
}

fn main() {
    unsafe {
        if let Err(e) = run() {
            let msg = format!("PresenterKit failed to start: {:?}\0", e);
            let wide: Vec<u16> = msg.encode_utf16().collect();
            let _ = MessageBoxW(None, PCWSTR(wide.as_ptr()), w!("Error"), MB_OK | MB_ICONERROR);
        }
    }
}

unsafe fn run() -> windows::core::Result<()> {
    // Load settings
    let settings = Settings::load();

    // Get screen dimensions
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);

    // Create overlay window
    let overlay = OverlayWindow::create(Some(wndproc))?;

    // Install keyboard hook
    let hook = KeyboardHook::install()?;

    // Create tray icon
    let tray = TrayIcon::create(overlay.hwnd)?;

    // Initialize app state
    let mut state = AppState::new(screen_w, screen_h);
    state.spotlight_radius = settings.spotlight.radius;
    state.spotlight_feather = settings.spotlight.feather;
    state.spotlight_opacity = settings.spotlight.opacity;
    state.default_zoom = settings.magnifier.default_zoom;
    state.chromatic_strength = settings.magnifier.chromatic_strength;
    state.max_magnifiers = settings.magnifier.max_instances;
    state.max_freezes = settings.freeze.max_regions;
    state.trail_enabled = settings.trail.enabled;
    state.trail_length = settings.trail.length;
    state.trail_fade_ms = settings.trail.fade_ms;
    state.trail_opacity = settings.trail.opacity;

    let bitmap = Bitmap::new(screen_w, screen_h);

    // Store app state with the window
    let app = Box::new(App {
        state,
        overlay,
        bitmap,
        _hook: hook,
        _tray: tray,
        timer_active: false,
    });

    let app_ptr = Box::into_raw(app);
    SetWindowLongPtrW((*app_ptr).overlay.hwnd, GWLP_USERDATA, app_ptr as isize);

    // Message loop
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    // Cleanup
    let _ = Box::from_raw(app_ptr);

    Ok(())
}

/// Window procedure for the overlay window
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
    if app_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let app = &mut *app_ptr;

    match msg {
        // --- Custom key action from keyboard hook ---
        overlay::WM_KEY_ACTION => {
            let (vk, is_down) = hotkey::unpack_key_action(wparam.0);
            handle_key_action(app, vk, is_down);
            LRESULT(0)
        }

        // --- Tray icon callback ---
        overlay::WM_TRAY_CALLBACK => {
            app._tray.handle_message(lparam);
            LRESULT(0)
        }

        // --- Render timer ---
        WM_TIMER => {
            if wparam.0 == TIMER_RENDER {
                update_and_render(app);
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Handle a key press/release action
unsafe fn handle_key_action(app: &mut App, vk: u32, is_down: bool) {
    let state = &mut app.state;

    match vk {
        // --- Spotlight (ScrollLock hold) ---
        hotkey::VK_SPOTLIGHT => {
            state.spotlight_active = is_down;
            state.dirty = true;
        }

        // --- Circle Marker (Home hold → drag → release) ---
        hotkey::VK_CIRCLE_MARKER => {
            if is_down {
                state.drawing_mode = DrawingMode::CircleMarker;
                // Get current cursor position as start
                let mut pt = windows::Win32::Foundation::POINT::default();
                let _ = GetCursorPos(&mut pt);
                state.draw_start = Some(Pos::new(pt.x, pt.y));
            } else {
                // Release: finalize circle marker → create magnifier
                if state.drawing_mode == DrawingMode::CircleMarker {
                    if let Some(start) = state.draw_start {
                        let mut pt = windows::Win32::Foundation::POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        let end = Pos::new(pt.x, pt.y);
                        let radius = start.distance_to(&end) as i32;

                        if radius > 10 && state.magnifiers.len() < state.max_magnifiers {
                            // Capture the region
                            let cap_rect = Rect::new(
                                start.x - radius, start.y - radius,
                                radius * 2, radius * 2,
                            );
                            if let Some(captured) = ScreenCapture::capture_region(cap_rect) {
                                let mag = features::magnifier::create_magnifier(
                                    state.next_mag_id,
                                    start,
                                    radius,
                                    state.default_zoom,
                                    state.chromatic_strength,
                                    captured,
                                    state.screen_w,
                                );
                                state.magnifiers.push(mag);
                                state.next_mag_id += 1;
                            }
                        }
                    }
                }
                state.drawing_mode = DrawingMode::None;
                state.draw_start = None;
            }
            state.dirty = true;
        }

        // --- Freeze Region (Insert hold → drag → release) ---
        hotkey::VK_FREEZE_REGION => {
            if is_down {
                state.drawing_mode = DrawingMode::RectFreeze;
                let mut pt = windows::Win32::Foundation::POINT::default();
                let _ = GetCursorPos(&mut pt);
                state.draw_start = Some(Pos::new(pt.x, pt.y));
            } else {
                if state.drawing_mode == DrawingMode::RectFreeze {
                    if let Some(start) = state.draw_start {
                        let mut pt = windows::Win32::Foundation::POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        let end = Pos::new(pt.x, pt.y);
                        let rect = Rect::from_points(start, end);

                        if rect.w > 10 && rect.h > 10 && state.freezes.len() < state.max_freezes {
                            if let Some(captured) = ScreenCapture::capture_region(rect) {
                                state.freezes.push(FreezeRegion {
                                    id: state.next_freeze_id,
                                    rect,
                                    captured,
                                });
                                state.next_freeze_id += 1;
                            }
                        }
                    }
                }
                state.drawing_mode = DrawingMode::None;
                state.draw_start = None;
            }
            state.dirty = true;
        }

        // --- Freeze Half Screen (End toggle) ---
        hotkey::VK_FREEZE_HALF => {
            if is_down && state.freezes.len() < state.max_freezes {
                let rect = match state.half_freeze_side {
                    HalfSide::Left => Rect::left_half(state.screen_w, state.screen_h),
                    HalfSide::Right => Rect::right_half(state.screen_w, state.screen_h),
                };
                if let Some(captured) = ScreenCapture::capture_region(rect) {
                    state.freezes.push(FreezeRegion {
                        id: state.next_freeze_id,
                        rect,
                        captured,
                    });
                    state.next_freeze_id += 1;
                }
                // Toggle side for next press
                state.half_freeze_side = match state.half_freeze_side {
                    HalfSide::Left => HalfSide::Right,
                    HalfSide::Right => HalfSide::Left,
                };
                state.dirty = true;
            }
        }

        // --- Size up/down ---
        hotkey::VK_SIZE_UP => {
            if is_down {
                state.spotlight_radius = (state.spotlight_radius + 20.0).min(800.0);
                state.dirty = true;
            }
        }
        hotkey::VK_SIZE_DOWN => {
            if is_down {
                state.spotlight_radius = (state.spotlight_radius - 20.0).max(40.0);
                state.dirty = true;
            }
        }

        // --- Remove last ---
        hotkey::VK_REMOVE_LAST => {
            if is_down {
                state.remove_last_effect();
            }
        }

        // --- Clear all ---
        hotkey::VK_CLEAR_ALL => {
            if is_down {
                state.clear_all_effects();
            }
        }

        _ => {}
    }

    state.update_active_state();
    ensure_timer(app);
}

/// Start or stop the render timer based on whether any effects are active
unsafe fn ensure_timer(app: &mut App) {
    let need_timer = app.state.any_active || app.state.trail_enabled;
    if need_timer && !app.timer_active {
        SetTimer(app.overlay.hwnd, TIMER_RENDER, RENDER_INTERVAL_MS, None);
        app.timer_active = true;
        app.overlay.show();
    } else if !need_timer && app.timer_active {
        let _ = KillTimer(app.overlay.hwnd, TIMER_RENDER);
        app.timer_active = false;
        // Clear overlay and hide
        app.bitmap.clear();
        app.overlay.present(&app.bitmap);
        app.overlay.hide();
    }
}

/// Update state and render the overlay
unsafe fn update_and_render(app: &mut App) {
    // Update cursor position
    let mut pt = windows::Win32::Foundation::POINT::default();
    let _ = GetCursorPos(&mut pt);
    let new_cursor = Pos::new(pt.x, pt.y);

    // Check if cursor moved
    if new_cursor.x != app.state.cursor.x || new_cursor.y != app.state.cursor.y {
        app.state.prev_cursor = app.state.cursor;
        app.state.cursor = new_cursor;
        app.state.dirty = true;

        // Add trail point
        if app.state.trail_enabled {
            let velocity = app.state.cursor.distance_to(&app.state.prev_cursor);
            let now_ms = windows::Win32::System::SystemInformation::GetTickCount64();
            app.state.trail_points.push_back(TrailPoint {
                pos: app.state.cursor,
                time_ms: now_ms,
                velocity,
            });
            // Trim old points
            while app.state.trail_points.len() > app.state.trail_length {
                app.state.trail_points.pop_front();
            }
        }
    }

    // Remove expired trail points
    if !app.state.trail_points.is_empty() {
        let now_ms = windows::Win32::System::SystemInformation::GetTickCount64();
        while let Some(front) = app.state.trail_points.front() {
            if now_ms.saturating_sub(front.time_ms) >= app.state.trail_fade_ms {
                app.state.trail_points.pop_front();
                app.state.dirty = true;
            } else {
                break;
            }
        }
    }

    if !app.state.dirty && !app.state.spotlight_active {
        return; // Nothing changed, skip render
    }
    app.state.dirty = false;

    // --- Render ---
    app.bitmap.clear();

    // 1. Spotlight (background darkening)
    if app.state.spotlight_active {
        features::spotlight::render_spotlight(
            &mut app.bitmap,
            app.state.cursor,
            app.state.spotlight_radius,
            app.state.spotlight_feather,
            app.state.spotlight_opacity,
        );
    }

    // 2. Frozen regions
    for freeze in &app.state.freezes {
        features::freeze::render_freeze(&mut app.bitmap, freeze);
    }

    // 3. Magnifier lenses
    for mag in &app.state.magnifiers {
        features::magnifier::render_magnifier(&mut app.bitmap, mag);
    }

    // 4. Drawing feedback (in-progress circle or rectangle)
    match app.state.drawing_mode {
        DrawingMode::CircleMarker => {
            if let Some(start) = app.state.draw_start {
                let radius = start.distance_to(&app.state.cursor);
                features::spotlight::render_circle_outline(
                    &mut app.bitmap, start, radius, 2.0,
                );
            }
        }
        DrawingMode::RectFreeze => {
            if let Some(start) = app.state.draw_start {
                let rect = Rect::from_points(start, app.state.cursor);
                features::spotlight::render_rect_outline(
                    &mut app.bitmap, rect, 2,
                );
            }
        }
        DrawingMode::None => {}
    }

    // 5. Mouse trail
    if app.state.trail_enabled && !app.state.trail_points.is_empty() {
        let now_ms = windows::Win32::System::SystemInformation::GetTickCount64();
        features::trail::render_trail(
            &mut app.bitmap,
            &app.state.trail_points,
            now_ms,
            app.state.trail_fade_ms,
            app.state.trail_opacity,
        );
    }

    // 6. Cursor highlight (always when trail enabled)
    if app.state.trail_enabled && app.state.any_active {
        features::trail::render_cursor_highlight(&mut app.bitmap, app.state.cursor);
    }

    // Present
    app.overlay.present(&app.bitmap);

    // Update active state (trail points may have expired)
    app.state.update_active_state();
    ensure_timer(app);
}
