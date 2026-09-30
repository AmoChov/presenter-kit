//! Spotlight effect renderer.
//! Darkens the entire overlay except a circular area around the cursor.
//! Uses a smooth radial gradient (smoothstep) for Apple-quality edges.

use crate::types::*;

/// Render the spotlight effect onto the bitmap.
/// The bitmap should be cleared before calling this.
pub fn render_spotlight(bmp: &mut Bitmap, cursor: Pos, radius: f32, feather: f32, opacity: f32) {
    let alpha_max = (opacity * 255.0) as u8;
    // Premultiplied black with alpha_max
    let pixel_dark = u32::from_le_bytes([0, 0, 0, alpha_max]);

    // Fast fill: set entire bitmap to dark
    let data32 = unsafe {
        std::slice::from_raw_parts_mut(
            bmp.data.as_mut_ptr() as *mut u32,
            bmp.data.len() / 4,
        )
    };
    data32.fill(pixel_dark);

    // Now punch a hole around the cursor with smooth gradient
    let cx = cursor.x as f32;
    let cy = cursor.y as f32;
    let outer_r = radius + feather;
    let inner_r = radius;

    // Bounding box of the gradient circle
    let x_min = ((cx - outer_r) as i32).max(0);
    let x_max = ((cx + outer_r) as i32 + 1).min(bmp.width);
    let y_min = ((cy - outer_r) as i32).max(0);
    let y_max = ((cy + outer_r) as i32 + 1).min(bmp.height);

    for y in y_min..y_max {
        let dy = y as f32 - cy;
        let dy2 = dy * dy;
        let row_offset = (y * bmp.width) as usize;

        for x in x_min..x_max {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy2).sqrt();

            if dist < outer_r {
                // smoothstep: 0.0 at inner_r (fully transparent), 1.0 at outer_r (fully dark)
                let t = smoothstep(inner_r, outer_r, dist);
                let a = (t * alpha_max as f32) as u8;
                let idx = (row_offset + x as usize) * 4;
                bmp.data[idx] = 0;     // B
                bmp.data[idx + 1] = 0; // G
                bmp.data[idx + 2] = 0; // R
                bmp.data[idx + 3] = a; // A (premultiplied, black so RGB=0)
            }
        }
    }
}

/// Render a circle outline showing the draw-in-progress for circle marker
pub fn render_circle_outline(bmp: &mut Bitmap, center: Pos, radius: f32, thickness: f32) {
    let cx = center.x as f32;
    let cy = center.y as f32;
    let outer_r = radius + thickness / 2.0;
    let inner_r = (radius - thickness / 2.0).max(0.0);

    let x_min = ((cx - outer_r) as i32 - 1).max(0);
    let x_max = ((cx + outer_r) as i32 + 2).min(bmp.width);
    let y_min = ((cy - outer_r) as i32 - 1).max(0);
    let y_max = ((cy + outer_r) as i32 + 2).min(bmp.height);

    for y in y_min..y_max {
        let dy = y as f32 - cy;
        let dy2 = dy * dy;
        for x in x_min..x_max {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy2).sqrt();

            if dist >= inner_r && dist <= outer_r {
                // Anti-aliased edges
                let aa_inner = smoothstep(inner_r - 1.0, inner_r + 1.0, dist);
                let aa_outer = 1.0 - smoothstep(outer_r - 1.0, outer_r + 1.0, dist);
                let alpha = (aa_inner * aa_outer * 200.0) as u8;
                if alpha > 0 {
                    // White circle outline, premultiplied
                    bmp.blend_pixel(x, y, alpha, alpha, alpha, alpha);
                }
            }
        }
    }
}

/// Render a rectangle outline showing the draw-in-progress for rect freeze
pub fn render_rect_outline(bmp: &mut Bitmap, rect: Rect, thickness: i32) {
    let color_a: u8 = 180;
    // Top edge
    for y in rect.y..(rect.y + thickness).min(rect.y + rect.h) {
        for x in rect.x..(rect.x + rect.w) {
            bmp.blend_pixel(x, y, color_a, color_a, color_a, color_a);
        }
    }
    // Bottom edge
    for y in (rect.y + rect.h - thickness).max(rect.y)..(rect.y + rect.h) {
        for x in rect.x..(rect.x + rect.w) {
            bmp.blend_pixel(x, y, color_a, color_a, color_a, color_a);
        }
    }
    // Left edge
    for y in rect.y..(rect.y + rect.h) {
        for x in rect.x..(rect.x + thickness).min(rect.x + rect.w) {
            bmp.blend_pixel(x, y, color_a, color_a, color_a, color_a);
        }
    }
    // Right edge
    for y in rect.y..(rect.y + rect.h) {
        for x in (rect.x + rect.w - thickness).max(rect.x)..(rect.x + rect.w) {
            bmp.blend_pixel(x, y, color_a, color_a, color_a, color_a);
        }
    }
}
