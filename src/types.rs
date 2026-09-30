//! Core shared types for PresenterKit.
//! All modules import from here for consistency.

use std::collections::VecDeque;

/// Screen position in pixels
#[derive(Clone, Copy, Debug, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Pos) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Axis-aligned rectangle in pixels
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    /// Create from two corner points, normalizing negative dimensions
    pub fn from_points(p1: Pos, p2: Pos) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let w = (p1.x - p2.x).abs();
        let h = (p1.y - p2.y).abs();
        Self { x, y, w, h }
    }

    pub fn center(&self) -> Pos {
        Pos::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    pub fn contains(&self, p: Pos) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }

    pub fn left_half(screen_w: i32, screen_h: i32) -> Self {
        Self::new(0, 0, screen_w / 2, screen_h)
    }

    pub fn right_half(screen_w: i32, screen_h: i32) -> Self {
        Self::new(screen_w / 2, 0, screen_w - screen_w / 2, screen_h)
    }
}

/// BGRA premultiplied-alpha bitmap buffer for software rendering
pub struct Bitmap {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

impl Bitmap {
    pub fn new(width: i32, height: i32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0u8; size],
        }
    }

    pub fn stride(&self) -> i32 {
        self.width * 4
    }

    /// Clear entire bitmap to transparent black
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Fill entire bitmap with a solid BGRA premultiplied color
    pub fn fill(&mut self, b: u8, g: u8, r: u8, a: u8) {
        let pixel = u32::from_le_bytes([b, g, r, a]);
        let data32 = unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut u32,
                self.data.len() / 4,
            )
        };
        data32.fill(pixel);
    }

    /// Set a single pixel (BGRA premultiplied). No bounds check.
    #[inline(always)]
    pub fn set_pixel_unchecked(&mut self, x: i32, y: i32, b: u8, g: u8, r: u8, a: u8) {
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = b;
        self.data[idx + 1] = g;
        self.data[idx + 2] = r;
        self.data[idx + 3] = a;
    }

    /// Set pixel with bounds checking
    #[inline(always)]
    pub fn set_pixel(&mut self, x: i32, y: i32, b: u8, g: u8, r: u8, a: u8) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.set_pixel_unchecked(x, y, b, g, r, a);
        }
    }

    /// Alpha-blend a pixel on top of existing content (source-over, premultiplied alpha)
    #[inline(always)]
    pub fn blend_pixel(&mut self, x: i32, y: i32, sb: u8, sg: u8, sr: u8, sa: u8) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        let inv_sa = 255 - sa as u16;
        // Source-over with premultiplied alpha: out = src + dst * (1 - src_alpha)
        self.data[idx]     = (sb as u16 + (self.data[idx] as u16 * inv_sa / 255)) as u8;
        self.data[idx + 1] = (sg as u16 + (self.data[idx + 1] as u16 * inv_sa / 255)) as u8;
        self.data[idx + 2] = (sr as u16 + (self.data[idx + 2] as u16 * inv_sa / 255)) as u8;
        self.data[idx + 3] = (sa as u16 + (self.data[idx + 3] as u16 * inv_sa / 255)) as u8;
    }

    /// Get pixel at position (returns [B, G, R, A])
    #[inline(always)]
    pub fn get_pixel(&self, x: i32, y: i32) -> [u8; 4] {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return [0, 0, 0, 0];
        }
        let idx = ((y * self.width + x) * 4) as usize;
        [self.data[idx], self.data[idx+1], self.data[idx+2], self.data[idx+3]]
    }

    /// Blit a source bitmap region onto this bitmap at dst position
    pub fn blit(&mut self, src: &Bitmap, src_rect: Rect, dst_x: i32, dst_y: i32) {
        for sy in 0..src_rect.h {
            for sx in 0..src_rect.w {
                let px = src.get_pixel(src_rect.x + sx, src_rect.y + sy);
                self.set_pixel(dst_x + sx, dst_y + sy, px[0], px[1], px[2], px[3]);
            }
        }
    }

    /// Blit with alpha blending
    pub fn blit_blend(&mut self, src: &Bitmap, src_rect: Rect, dst_x: i32, dst_y: i32) {
        for sy in 0..src_rect.h {
            for sx in 0..src_rect.w {
                let px = src.get_pixel(src_rect.x + sx, src_rect.y + sy);
                self.blend_pixel(dst_x + sx, dst_y + sy, px[0], px[1], px[2], px[3]);
            }
        }
    }
}

/// Smooth interpolation (cubic Hermite)
#[inline(always)]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Mouse trail point
#[derive(Clone, Copy, Debug)]
pub struct TrailPoint {
    pub pos: Pos,
    pub time_ms: u64,
    pub velocity: f32,
}

/// A magnifier lens instance
pub struct MagnifierInstance {
    pub id: u32,
    pub source_center: Pos,
    pub source_radius: i32,
    pub display_pos: Pos,
    pub zoom: f32,
    pub chromatic_strength: f32,
    pub captured: Bitmap,
}

/// A frozen region instance
pub struct FreezeRegion {
    pub id: u32,
    pub rect: Rect,
    pub captured: Bitmap,
}

/// Current interactive drawing mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawingMode {
    None,
    CircleMarker,
    RectFreeze,
}

/// Half-screen side for freeze
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HalfSide {
    Left,
    Right,
}

/// The global application state
pub struct AppState {
    pub screen_w: i32,
    pub screen_h: i32,
    pub cursor: Pos,
    pub prev_cursor: Pos,

    // Spotlight
    pub spotlight_active: bool,
    pub spotlight_radius: f32,
    pub spotlight_feather: f32,
    pub spotlight_opacity: f32,

    // Magnifier
    pub magnifiers: Vec<MagnifierInstance>,
    pub max_magnifiers: usize,
    pub default_zoom: f32,
    pub chromatic_strength: f32,
    pub next_mag_id: u32,

    // Freeze
    pub freezes: Vec<FreezeRegion>,
    pub max_freezes: usize,
    pub next_freeze_id: u32,
    pub half_freeze_side: HalfSide,

    // Trail
    pub trail_enabled: bool,
    pub trail_points: VecDeque<TrailPoint>,
    pub trail_length: usize,
    pub trail_fade_ms: u64,
    pub trail_opacity: f32,

    // Drawing interaction
    pub drawing_mode: DrawingMode,
    pub draw_start: Option<Pos>,

    // Dirty flag — true when overlay needs repaint
    pub dirty: bool,
    pub any_active: bool,
}

impl AppState {
    pub fn new(screen_w: i32, screen_h: i32) -> Self {
        Self {
            screen_w,
            screen_h,
            cursor: Pos::default(),
            prev_cursor: Pos::default(),
            spotlight_active: false,
            spotlight_radius: 200.0,
            spotlight_feather: 60.0,
            spotlight_opacity: 0.85,
            magnifiers: Vec::new(),
            max_magnifiers: 8,
            default_zoom: 2.5,
            chromatic_strength: 0.5,
            next_mag_id: 0,
            freezes: Vec::new(),
            max_freezes: 8,
            next_freeze_id: 0,
            half_freeze_side: HalfSide::Left,
            trail_enabled: true,
            trail_points: VecDeque::new(),
            trail_length: 40,
            trail_fade_ms: 600,
            trail_opacity: 0.5,
            drawing_mode: DrawingMode::None,
            draw_start: None,
            dirty: true,
            any_active: false,
        }
    }

    pub fn update_active_state(&mut self) {
        self.any_active = self.spotlight_active
            || !self.magnifiers.is_empty()
            || !self.freezes.is_empty()
            || !self.trail_points.is_empty()
            || self.drawing_mode != DrawingMode::None;
    }

    pub fn remove_last_effect(&mut self) {
        if let Some(_) = self.magnifiers.pop() {
            self.dirty = true;
        } else if let Some(_) = self.freezes.pop() {
            self.dirty = true;
        }
        self.update_active_state();
    }

    pub fn clear_all_effects(&mut self) {
        self.magnifiers.clear();
        self.freezes.clear();
        self.spotlight_active = false;
        self.drawing_mode = DrawingMode::None;
        self.draw_start = None;
        self.trail_points.clear();
        self.dirty = true;
        self.update_active_state();
    }
}
