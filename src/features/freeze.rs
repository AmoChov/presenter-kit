//! Freeze region renderer.
//! Displays captured screen snapshots at their original positions,
//! creating a "time freeze" effect on parts of the screen.

use crate::types::*;

/// Render a single frozen region onto the overlay bitmap.
/// The frozen content is drawn with a subtle border to indicate it's frozen.
pub fn render_freeze(bmp: &mut Bitmap, freeze: &FreezeRegion) {
    let r = &freeze.rect;

    // Draw the captured content at its original position
    for y in 0..r.h {
        for x in 0..r.w {
            let px = freeze.captured.get_pixel(x, y);
            let dst_x = r.x + x;
            let dst_y = r.y + y;
            if dst_x >= 0 && dst_x < bmp.width && dst_y >= 0 && dst_y < bmp.height {
                bmp.set_pixel(dst_x, dst_y, px[0], px[1], px[2], px[3]);
            }
        }
    }

    // Draw a subtle blue-tinted border (2px) to indicate frozen state
    let border = 2;
    let border_color: u8 = 140; // semi-transparent
    let blue_tint: u8 = 200;

    // Top & bottom
    for y_off in [0..border, (r.h - border)..r.h] {
        for y in y_off {
            for x in 0..r.w {
                let dst_x = r.x + x;
                let dst_y = r.y + y;
                // Premultiplied: blue-ish semi-transparent
                let ba = (blue_tint as u16 * border_color as u16 / 255) as u8;
                let ga = (120_u16 * border_color as u16 / 255) as u8;
                let ra = (80_u16 * border_color as u16 / 255) as u8;
                bmp.blend_pixel(dst_x, dst_y, ba, ga, ra, border_color);
            }
        }
    }
    // Left & right
    for x_off in [0..border, (r.w - border)..r.w] {
        for x in x_off {
            for y in 0..r.h {
                let dst_x = r.x + x;
                let dst_y = r.y + y;
                let ba = (blue_tint as u16 * border_color as u16 / 255) as u8;
                let ga = (120_u16 * border_color as u16 / 255) as u8;
                let ra = (80_u16 * border_color as u16 / 255) as u8;
                bmp.blend_pixel(dst_x, dst_y, ba, ga, ra, border_color);
            }
        }
    }

    // Corner frost indicators (small snowflake-like dots)
    render_frost_indicator(bmp, Pos::new(r.x + 8, r.y + 8));
    render_frost_indicator(bmp, Pos::new(r.x + r.w - 12, r.y + 8));
}

/// Render a small frost indicator (4px pulsing dot) at the given position
fn render_frost_indicator(bmp: &mut Bitmap, pos: Pos) {
    let radius = 3.0_f32;
    for dy in -4..=4 {
        for dx in -4..=4 {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= radius {
                let t = 1.0 - dist / radius;
                let a = (t * 180.0) as u8;
                // Light blue frost color, premultiplied
                let b = (220_u16 * a as u16 / 255) as u8;
                let g = (240_u16 * a as u16 / 255) as u8;
                let r = (255_u16 * a as u16 / 255) as u8;
                bmp.blend_pixel(pos.x + dx, pos.y + dy, b, g, r, a);
            }
        }
    }
}
