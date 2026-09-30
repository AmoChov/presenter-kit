//! Mouse trail renderer ("comet" style).
//! Renders recent cursor positions as fading, glowing dots.
//! Points fade out over time based on trail_fade_ms.

use crate::types::*;

/// Render the mouse trail onto the bitmap.
/// `now_ms` is the current time in milliseconds (GetTickCount64).
pub fn render_trail(bmp: &mut Bitmap, points: &std::collections::VecDeque<TrailPoint>, now_ms: u64, fade_ms: u64, base_opacity: f32) {
    if points.is_empty() {
        return;
    }

    for (_i, point) in points.iter().enumerate() {
        let age = now_ms.saturating_sub(point.time_ms);
        if age >= fade_ms {
            continue; // Fully faded
        }

        // Exponential fade
        let life = 1.0 - (age as f32 / fade_ms as f32);
        let alpha = (life * life * base_opacity * 255.0) as u8; // quadratic fade for smooth decay

        if alpha == 0 {
            continue;
        }

        // Dot size varies with velocity (faster = larger trail)
        let base_radius = 3.0_f32;
        let vel_bonus = (point.velocity * 0.02).min(4.0);
        let radius = base_radius + vel_bonus;

        // Render a soft glow dot
        render_glow_dot(bmp, point.pos, radius, alpha);
    }
}

/// Render a soft circular glow dot
fn render_glow_dot(bmp: &mut Bitmap, center: Pos, radius: f32, max_alpha: u8) {
    let r_ceil = radius.ceil() as i32 + 1;
    let cx = center.x;
    let cy = center.y;

    for dy in -r_ceil..=r_ceil {
        for dx in -r_ceil..=r_ceil {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= radius {
                // Gaussian-like falloff
                let t = dist / radius;
                let intensity = (1.0 - t * t) * (1.0 - t * t); // quartic falloff
                let a = (intensity * max_alpha as f32) as u8;
                if a > 0 {
                    // White glow, premultiplied
                    bmp.blend_pixel(cx + dx, cy + dy, a, a, a, a);
                }
            }
        }
    }
}

/// Render a prominent cursor highlight ring (always visible, subtle)
pub fn render_cursor_highlight(bmp: &mut Bitmap, cursor: Pos) {
    let inner_r = 14.0_f32;
    let outer_r = 18.0_f32;
    let cx = cursor.x;
    let cy = cursor.y;

    for dy in -20..=20 {
        for dx in -20..=20 {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist >= inner_r && dist <= outer_r {
                let t_inner = smoothstep(inner_r - 1.0, inner_r + 1.0, dist);
                let t_outer = 1.0 - smoothstep(outer_r - 1.0, outer_r + 1.0, dist);
                let a = (t_inner * t_outer * 80.0) as u8;
                if a > 0 {
                    // Soft white ring, premultiplied
                    bmp.blend_pixel(cx + dx, cy + dy, a, a, a, a);
                }
            }
        }
    }
}
