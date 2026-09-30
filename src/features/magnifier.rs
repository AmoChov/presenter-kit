//! Magnifier lens renderer.
//! Renders captured screen regions zoomed in with:
//! - Circular lens shape with smooth border
//! - Chromatic aberration effect (RGB channel offset)
//! - Drop shadow
//! - Arrow pointing from lens to source location

use crate::types::*;

/// Render a single magnifier instance onto the bitmap
pub fn render_magnifier(bmp: &mut Bitmap, mag: &MagnifierInstance) {
    let display_cx = mag.display_pos.x as f32;
    let display_cy = mag.display_pos.y as f32;
    let lens_radius = (mag.source_radius as f32 * mag.zoom).min(300.0);
    let shadow_offset = 4.0_f32;
    let shadow_radius = lens_radius + 6.0;

    // --- Drop shadow ---
    render_circle_shadow(
        bmp,
        Pos::new(mag.display_pos.x + shadow_offset as i32, mag.display_pos.y + shadow_offset as i32),
        shadow_radius,
        40,
    );

    // --- Lens content with chromatic aberration ---
    let x_min = ((display_cx - lens_radius) as i32).max(0);
    let x_max = ((display_cx + lens_radius) as i32 + 1).min(bmp.width);
    let y_min = ((display_cy - lens_radius) as i32).max(0);
    let y_max = ((display_cy + lens_radius) as i32 + 1).min(bmp.height);

    let ca_offset = (mag.chromatic_strength * 3.0) as i32; // chromatic aberration pixel offset

    for y in y_min..y_max {
        let dy = y as f32 - display_cy;
        let dy2 = dy * dy;
        for x in x_min..x_max {
            let dx = x as f32 - display_cx;
            let dist = (dx * dx + dy2).sqrt();

            if dist <= lens_radius {
                // Map display position back to source position
                let src_x = mag.source_center.x as f32 + dx / mag.zoom;
                let src_y = mag.source_center.y as f32 + dy / mag.zoom;

                let sx = src_x as i32;
                let sy = src_y as i32;

                // Sample with chromatic aberration: offset R and B channels
                let px_r = mag.captured.get_pixel(
                    (sx + ca_offset) - mag.source_center.x + mag.captured.width / 2,
                    sy - mag.source_center.y + mag.captured.height / 2,
                );
                let px_g = mag.captured.get_pixel(
                    sx - mag.source_center.x + mag.captured.width / 2,
                    sy - mag.source_center.y + mag.captured.height / 2,
                );
                let px_b = mag.captured.get_pixel(
                    (sx - ca_offset) - mag.source_center.x + mag.captured.width / 2,
                    sy - mag.source_center.y + mag.captured.height / 2,
                );

                // Anti-alias the edge
                let edge_aa = 1.0 - smoothstep(lens_radius - 2.0, lens_radius, dist);
                let alpha = (edge_aa * 255.0) as u8;

                // Combine channels: R from px_r, G from px_g, B from px_b
                let r = px_r[2]; // R channel (index 2 in BGRA)
                let g = px_g[1]; // G channel (index 1 in BGRA)
                let b = px_b[0]; // B channel (index 0 in BGRA)

                // Premultiply
                let af = alpha as f32 / 255.0;
                bmp.set_pixel(
                    x, y,
                    (b as f32 * af) as u8,
                    (g as f32 * af) as u8,
                    (r as f32 * af) as u8,
                    alpha,
                );
            }
        }
    }

    // --- Lens border (thin white ring) ---
    render_lens_border(bmp, mag.display_pos, lens_radius, 2.0);

    // --- Arrow from source to lens ---
    render_arrow(bmp, mag.source_center, mag.display_pos, mag.source_radius as f32, lens_radius);
}

/// Render a soft circular drop shadow
fn render_circle_shadow(bmp: &mut Bitmap, center: Pos, radius: f32, max_alpha: u8) {
    let cx = center.x as f32;
    let cy = center.y as f32;
    let blur = 12.0_f32;
    let outer = radius + blur;

    let x_min = ((cx - outer) as i32).max(0);
    let x_max = ((cx + outer) as i32 + 1).min(bmp.width);
    let y_min = ((cy - outer) as i32).max(0);
    let y_max = ((cy + outer) as i32 + 1).min(bmp.height);

    for y in y_min..y_max {
        let dy = y as f32 - cy;
        for x in x_min..x_max {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < outer {
                let t = smoothstep(radius, outer, dist);
                let a = ((1.0 - t) * max_alpha as f32) as u8;
                // Black shadow, premultiplied
                bmp.blend_pixel(x, y, 0, 0, 0, a);
            }
        }
    }
}

/// Render thin white lens border
fn render_lens_border(bmp: &mut Bitmap, center: Pos, radius: f32, thickness: f32) {
    let cx = center.x as f32;
    let cy = center.y as f32;
    let outer_r = radius + thickness / 2.0;
    let inner_r = radius - thickness / 2.0;

    let x_min = ((cx - outer_r) as i32 - 1).max(0);
    let x_max = ((cx + outer_r) as i32 + 2).min(bmp.width);
    let y_min = ((cy - outer_r) as i32 - 1).max(0);
    let y_max = ((cy + outer_r) as i32 + 2).min(bmp.height);

    for y in y_min..y_max {
        let dy = y as f32 - cy;
        for x in x_min..x_max {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist >= inner_r && dist <= outer_r {
                let aa = smoothstep(inner_r - 0.5, inner_r + 0.5, dist)
                    * (1.0 - smoothstep(outer_r - 0.5, outer_r + 0.5, dist));
                let a = (aa * 220.0) as u8;
                // White border, premultiplied
                bmp.blend_pixel(x, y, a, a, a, a);
            }
        }
    }
}

/// Render a thin line with arrow from source to lens
fn render_arrow(bmp: &mut Bitmap, from: Pos, to: Pos, src_radius: f32, dst_radius: f32) {
    let dx = to.x as f32 - from.x as f32;
    let dy = to.y as f32 - from.y as f32;
    let length = (dx * dx + dy * dy).sqrt();
    if length < src_radius + dst_radius + 10.0 {
        return; // Too close, skip arrow
    }

    let nx = dx / length;
    let ny = dy / length;

    // Start and end points (offset by radii)
    let start_x = from.x as f32 + nx * src_radius;
    let start_y = from.y as f32 + ny * src_radius;
    let end_x = to.x as f32 - nx * dst_radius;
    let end_y = to.y as f32 - ny * dst_radius;

    // Draw line using Bresenham-like stepping
    let steps = ((end_x - start_x).abs().max((end_y - start_y).abs())) as i32;
    if steps <= 0 { return; }

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = (start_x + (end_x - start_x) * t) as i32;
        let y = (start_y + (end_y - start_y) * t) as i32;
        // 2px wide line, semi-transparent white
        for oy in -1..=1 {
            for ox in -1..=1 {
                bmp.blend_pixel(x + ox, y + oy, 100, 100, 100, 100);
            }
        }
    }

    // Arrowhead at the end
    let arrow_size = 8.0_f32;
    let px = -ny; // perpendicular
    let py = nx;
    let tip_x = end_x;
    let tip_y = end_y;
    let left_x = (tip_x - nx * arrow_size + px * arrow_size * 0.5) as i32;
    let left_y = (tip_y - ny * arrow_size + py * arrow_size * 0.5) as i32;
    let right_x = (tip_x - nx * arrow_size - px * arrow_size * 0.5) as i32;
    let right_y = (tip_y - ny * arrow_size - py * arrow_size * 0.5) as i32;

    // Simple filled triangle for arrowhead
    draw_line(bmp, tip_x as i32, tip_y as i32, left_x, left_y, 160);
    draw_line(bmp, tip_x as i32, tip_y as i32, right_x, right_y, 160);
    draw_line(bmp, left_x, left_y, right_x, right_y, 160);
}

/// Simple line drawing helper
fn draw_line(bmp: &mut Bitmap, x0: i32, y0: i32, x1: i32, y1: i32, alpha: u8) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        bmp.blend_pixel(x, y, alpha, alpha, alpha, alpha);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            if x == x1 { break; }
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 { break; }
            err += dx;
            y += sy;
        }
    }
}

/// Create a new magnifier from a marked circle region.
/// `center` is where the user placed the circle center.
/// `radius` is the circle radius.
/// `captured` is the screen capture of that region.
/// Returns a MagnifierInstance with display position offset to the right.
pub fn create_magnifier(
    id: u32,
    center: Pos,
    radius: i32,
    zoom: f32,
    chromatic: f32,
    captured: Bitmap,
    screen_w: i32,
) -> MagnifierInstance {
    // Place the magnifier lens to the right of the source, or left if near screen edge
    let lens_display_radius = (radius as f32 * zoom).min(300.0);
    let offset = radius as i32 + lens_display_radius as i32 + 40;
    let display_x = if center.x + offset + lens_display_radius as i32 > screen_w {
        center.x - offset  // Place left
    } else {
        center.x + offset  // Place right
    };

    MagnifierInstance {
        id,
        source_center: center,
        source_radius: radius,
        display_pos: Pos::new(display_x, center.y),
        zoom,
        chromatic_strength: chromatic,
        captured,
    }
}
