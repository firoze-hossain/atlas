//! lib.rs - Atlas 2D/3D Software Vector Map Renderer.
//! Zero GPU dependencies: renders vector map tiles, water bodies, parks, road networks,
//! bridges, POI badges, GPS indicators, and routes into a 32-bit CanvasSurface.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    string::String,
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    string::String,
    vec,
    vec::Vec,
};

pub use atlas_core::{
    GeoCoord, MapDatabase, MapLayerStyle, MapState, MercatorProjection, Poi, PoiCategory,
    PolygonKind, RoadKind, ScreenPoint,
};

/// 32-bit ARGB Color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { a: 255, r, g, b }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { a, r, g, b }
    }

    pub const fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Linear alpha blend over background pixel.
    pub fn blend_over(&self, bg: u32) -> u32 {
        if self.a == 255 {
            return self.to_u32();
        }
        if self.a == 0 {
            return bg;
        }

        let bg_r = ((bg >> 16) & 0xFF) as u32;
        let bg_g = ((bg >> 8) & 0xFF) as u32;
        let bg_b = (bg & 0xFF) as u32;

        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;

        let out_r = (self.r as u32 * alpha + bg_r * inv_alpha) / 255;
        let out_g = (self.g as u32 * alpha + bg_g * inv_alpha) / 255;
        let out_b = (self.b as u32 * alpha + bg_b * inv_alpha) / 255;

        (0xFF << 24) | (out_r << 16) | (out_g << 8) | out_b
    }
}

/// 2D Rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// 32-bit ARGB Canvas Surface.
pub struct CanvasSurface {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl CanvasSurface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0xFFF4F3F0; width * height], // Base land color
        }
    }

    pub fn fill(&mut self, color: Color) {
        let val = color.to_u32();
        self.buffer.fill(val);
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            self.buffer[idx] = color.blend_over(self.buffer[idx]);
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let x1 = ((rect.x + rect.width) as usize).min(self.width);
        let y1 = ((rect.y + rect.height) as usize).min(self.height);

        let solid = color.a == 255;
        let val = color.to_u32();

        for y in y0..y1 {
            let row_start = y * self.width;
            for x in x0..x1 {
                if solid {
                    self.buffer[row_start + x] = val;
                } else {
                    self.buffer[row_start + x] = color.blend_over(self.buffer[row_start + x]);
                }
            }
        }
    }

    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let x1 = ((rect.x + rect.width) as usize).min(self.width);
        let y1 = ((rect.y + rect.height) as usize).min(self.height);

        if x1 > x0 && y1 > y0 {
            for x in x0..x1 {
                self.buffer[y0 * self.width + x] = color.blend_over(self.buffer[y0 * self.width + x]);
                self.buffer[(y1 - 1) * self.width + x] = color.blend_over(self.buffer[(y1 - 1) * self.width + x]);
            }
            for y in y0..y1 {
                self.buffer[y * self.width + x0] = color.blend_over(self.buffer[y * self.width + x0]);
                self.buffer[y * self.width + (x1 - 1)] = color.blend_over(self.buffer[y * self.width + (x1 - 1)]);
            }
        }
    }

    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: i32, color: Color) {
        let r = radius.min(rect.width / 2).min(rect.height / 2);
        self.fill_rect(Rect::new(rect.x + r, rect.y, rect.width - 2 * r, rect.height), color);
        self.fill_rect(Rect::new(rect.x, rect.y + r, rect.width, rect.height - 2 * r), color);

        self.fill_circle(rect.x + r, rect.y + r, r, color);
        self.fill_circle(rect.x + rect.width - r - 1, rect.y + r, r, color);
        self.fill_circle(rect.x + r, rect.y + rect.height - r - 1, r, color);
        self.fill_circle(rect.x + rect.width - r - 1, rect.y + rect.height - r - 1, r, color);
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r_sq = radius * radius;
        let x0 = (cx - radius).max(0);
        let x1 = (cx + radius).min(self.width as i32 - 1);
        let y0 = (cy - radius).max(0);
        let y1 = (cy + radius).min(self.height as i32 - 1);

        for y in y0..=y1 {
            let dy = y - cy;
            let dy_sq = dy * dy;
            let row_start = y as usize * self.width;
            for x in x0..=x1 {
                let dx = x - cx;
                if dx * dx + dy_sq <= r_sq {
                    let idx = row_start + x as usize;
                    self.buffer[idx] = color.blend_over(self.buffer[idx]);
                }
            }
        }
    }

    pub fn draw_circle_outline(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r_inner_sq = (radius - 1) * (radius - 1);
        let r_outer_sq = (radius + 1) * (radius + 1);

        let x0 = (cx - radius - 1).max(0);
        let x1 = (cx + radius + 1).min(self.width as i32 - 1);
        let y0 = (cy - radius - 1).max(0);
        let y1 = (cy + radius + 1).min(self.height as i32 - 1);

        for y in y0..=y1 {
            let dy = y - cy;
            let dy_sq = dy * dy;
            let row_start = y as usize * self.width;
            for x in x0..=x1 {
                let dx = x - cx;
                let d_sq = dx * dx + dy_sq;
                if d_sq >= r_inner_sq && d_sq <= r_outer_sq {
                    let idx = row_start + x as usize;
                    self.buffer[idx] = color.blend_over(self.buffer[idx]);
                }
            }
        }
    }

    /// Antialiased thick line drawer with rounded end caps.
    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, color: Color) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            return;
        }

        let radius = width / 2.0;
        let _nx = -dy / len;
        let _ny = dx / len;

        let steps = (len * 1.5).max(4.0) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let cx = x0 + dx * t;
            let cy = y0 + dy * t;
            self.fill_circle(cx.round() as i32, cy.round() as i32, radius.round() as i32, color);
        }
    }

    pub fn draw_char(&mut self, cx: i32, cy: i32, ch: char, color: Color) {
        if cx < 0 || cy < 0 || (cx + 8) as usize > self.width || (cy + 16) as usize > self.height {
            return;
        }
        let bitmap = get_8x16_bitmap(ch);
        for row in 0..16 {
            let bits = bitmap[row];
            let py = (cy + row as i32) as usize;
            for col in 0..8 {
                if (bits & (0x80 >> col)) != 0 {
                    let px = (cx + col) as usize;
                    let idx = py * self.width + px;
                    self.buffer[idx] = color.blend_over(self.buffer[idx]);
                }
            }
        }
    }

    pub fn draw_text(&mut self, mut x: i32, y: i32, text: &str, color: Color) {
        for ch in text.chars() {
            self.draw_char(x, y, ch, color);
            x += 8;
        }
    }

    pub fn draw_text_pill(&mut self, x: i32, y: i32, text: &str, bg: Color, fg: Color) {
        let w = (text.len() as i32 * 8) + 16;
        let h = 22;
        self.fill_rounded_rect(Rect::new(x, y, w, h), 11, bg);
        self.draw_rect_outline(Rect::new(x, y, w, h), Color::rgba(0, 0, 0, 30));
        self.draw_text(x + 8, y + 3, text, fg);
    }
}

/// Vector Cartography Map Renderer.
pub struct VectorMapRenderer;

impl VectorMapRenderer {
    /// Renders complete vector map frame based on current `MapState`.
    pub fn render(surface: &mut CanvasSurface, state: &MapState) {
        let vp_w = surface.width as f32;
        let vp_h = surface.height as f32;

        // 1. Base terrain background
        match state.layer_style {
            MapLayerStyle::Standard => surface.fill(Color::rgb(244, 243, 240)), // macOS Map cream
            MapLayerStyle::Satellite => surface.fill(Color::rgb(34, 45, 34)),    // Dark satellite terrain
            MapLayerStyle::Transit => surface.fill(Color::rgb(248, 248, 250)),  // High contrast transit
        }

        // 2. Water Bodies & Parks (Polygons)
        let polygons = MapDatabase::load_polygons();
        for poly in &polygons {
            let (r, g, b) = poly.kind.color_rgb();
            let poly_color = Color::rgb(r, g, b);

            // Calculate bounding center of polygon
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let count = poly.points.len() as f32;
            for p in &poly.points {
                let s = MercatorProjection::geo_to_screen(p, &state.center, state.zoom, vp_w, vp_h);
                let (sx, sy) = Self::apply_3d_transform(s.x, s.y, vp_w, vp_h, state.pitch_deg);
                sum_x += sx;
                sum_y += sy;
            }
            let avg_x = sum_x / count;
            let avg_y = sum_y / count;

            // Approximate polygon as soft organic shapes around vertices
            for p in &poly.points {
                let s = MercatorProjection::geo_to_screen(p, &state.center, state.zoom, vp_w, vp_h);
                let (sx, sy) = Self::apply_3d_transform(s.x, s.y, vp_w, vp_h, state.pitch_deg);
                surface.fill_circle(sx.round() as i32, sy.round() as i32, 28, poly_color);
                surface.draw_line(avg_x, avg_y, sx, sy, 32.0, poly_color);
            }

            // Polygon label
            if state.zoom >= 14.5 && avg_x > 50.0 && avg_x < vp_w - 50.0 && avg_y > 50.0 && avg_y < vp_h - 50.0 {
                let text_color = match poly.kind {
                    PolygonKind::Lake | PolygonKind::River | PolygonKind::Pond => Color::rgb(26, 115, 232),
                    _ => Color::rgb(46, 125, 50),
                };
                let offset_x = (poly.name.len() as i32 * 8) / 2;
                surface.draw_text((avg_x as i32) - offset_x, avg_y as i32, poly.name, text_color);
            }
        }

        // 3. Road Networks
        let roads = MapDatabase::load_roads();

        // Pass A: Road Casings (Darker borders for separation)
        for road in &roads {
            let (cr, cg, cb) = road.kind.casing_rgb();
            let casing_color = Color::rgb(cr, cg, cb);
            let casing_width = road.kind.stroke_width(state.zoom) + 2.5;

            for i in 0..road.points.len() - 1 {
                let s0 = MercatorProjection::geo_to_screen(&road.points[i], &state.center, state.zoom, vp_w, vp_h);
                let s1 = MercatorProjection::geo_to_screen(&road.points[i + 1], &state.center, state.zoom, vp_w, vp_h);
                let (x0, y0) = Self::apply_3d_transform(s0.x, s0.y, vp_w, vp_h, state.pitch_deg);
                let (x1, y1) = Self::apply_3d_transform(s1.x, s1.y, vp_w, vp_h, state.pitch_deg);
                surface.draw_line(x0, y0, x1, y1, casing_width, casing_color);
            }
        }

        // Pass B: Road Infill Core Surface
        for road in &roads {
            let (r, g, b) = road.kind.color_rgb();
            let road_color = Color::rgb(r, g, b);
            let road_width = road.kind.stroke_width(state.zoom);

            for i in 0..road.points.len() - 1 {
                let s0 = MercatorProjection::geo_to_screen(&road.points[i], &state.center, state.zoom, vp_w, vp_h);
                let s1 = MercatorProjection::geo_to_screen(&road.points[i + 1], &state.center, state.zoom, vp_w, vp_h);
                let (x0, y0) = Self::apply_3d_transform(s0.x, s0.y, vp_w, vp_h, state.pitch_deg);
                let (x1, y1) = Self::apply_3d_transform(s1.x, s1.y, vp_w, vp_h, state.pitch_deg);
                surface.draw_line(x0, y0, x1, y1, road_width, road_color);
            }
        }

        // Pass C: Road Street Names
        if state.zoom >= 14.8 {
            for road in &roads {
                if road.points.len() >= 2 {
                    let mid_idx = road.points.len() / 2;
                    let p0 = &road.points[mid_idx - 1];
                    let p1 = &road.points[mid_idx];
                    let s0 = MercatorProjection::geo_to_screen(p0, &state.center, state.zoom, vp_w, vp_h);
                    let s1 = MercatorProjection::geo_to_screen(p1, &state.center, state.zoom, vp_w, vp_h);
                    let mid_x = (s0.x + s1.x) / 2.0;
                    let mid_y = (s0.y + s1.y) / 2.0;
                    let (tx, ty) = Self::apply_3d_transform(mid_x, mid_y, vp_w, vp_h, state.pitch_deg);

                    if tx > 200.0 && tx < vp_w - 60.0 && ty > 40.0 && ty < vp_h - 40.0 {
                        let text_color = Color::rgb(70, 70, 75);
                        surface.draw_text((tx - 30.0) as i32, (ty - 8.0) as i32, road.name, text_color);
                    }
                }
            }
        }

        // 4. Active Navigation Route
        if let Some(route) = &state.active_route {
            for i in 0..route.steps.len() - 1 {
                let s0 = MercatorProjection::geo_to_screen(&route.steps[i].waypoint, &state.center, state.zoom, vp_w, vp_h);
                let s1 = MercatorProjection::geo_to_screen(&route.steps[i + 1].waypoint, &state.center, state.zoom, vp_w, vp_h);
                let (x0, y0) = Self::apply_3d_transform(s0.x, s0.y, vp_w, vp_h, state.pitch_deg);
                let (x1, y1) = Self::apply_3d_transform(s1.x, s1.y, vp_w, vp_h, state.pitch_deg);

                // Outer cyan glow
                surface.draw_line(x0, y0, x1, y1, 14.0, Color::rgba(0, 122, 255, 90));
                // Core navigation route line
                surface.draw_line(x0, y0, x1, y1, 7.0, Color::rgb(0, 122, 255));
            }
        }

        // 5. Points of Interest (POIs)
        let pois = MapDatabase::load_pois();
        for poi in &pois {
            let screen = MercatorProjection::geo_to_screen(&poi.coord, &state.center, state.zoom, vp_w, vp_h);
            let (sx, sy) = Self::apply_3d_transform(screen.x, screen.y, vp_w, vp_h, state.pitch_deg);

            let px = sx.round() as i32;
            let py = sy.round() as i32;

            if px > -50 && px < surface.width as i32 + 50 && py > -50 && py < surface.height as i32 + 50 {
                let (r, g, b) = poi.category.color_rgb();
                let badge_col = Color::rgb(r, g, b);

                // Shadow
                surface.fill_circle(px + 1, py + 2, 11, Color::rgba(0, 0, 0, 40));
                // Outer white ring
                surface.fill_circle(px, py, 11, Color::rgb(255, 255, 255));
                // Inner colored circle
                surface.fill_circle(px, py, 9, badge_col);
                // Icon glyph
                surface.draw_char(px - 4, py - 8, poi.category.icon_char(), Color::rgb(255, 255, 255));

                // Title label pill below badge
                if state.zoom >= 14.2 {
                    let label_y = py + 14;
                    let label_x = px - (poi.name.len() as i32 * 8) / 2;
                    surface.draw_text_pill(label_x, label_y, poi.name, Color::rgba(255, 255, 255, 230), Color::rgb(30, 30, 35));
                }
            }
        }

        // 6. User Current Location Blue Pulse
        let user_scr = MercatorProjection::geo_to_screen(&state.user_location, &state.center, state.zoom, vp_w, vp_h);
        let (ux, uy) = Self::apply_3d_transform(user_scr.x, user_scr.y, vp_w, vp_h, state.pitch_deg);
        let ux_i = ux.round() as i32;
        let uy_i = uy.round() as i32;

        // GPS pulse radius halo
        surface.fill_circle(ux_i, uy_i, 22, Color::rgba(0, 122, 255, 60));
        surface.draw_circle_outline(ux_i, uy_i, 22, Color::rgba(0, 122, 255, 120));
        // Solid center dot with white ring
        surface.fill_circle(ux_i, uy_i, 8, Color::rgb(255, 255, 255));
        surface.fill_circle(ux_i, uy_i, 6, Color::rgb(0, 122, 255));

        // 7. Marked Location Teardrop Pin
        if let Some(marked) = state.marked_location {
            let mark_scr = MercatorProjection::geo_to_screen(&marked, &state.center, state.zoom, vp_w, vp_h);
            let (mx, my) = Self::apply_3d_transform(mark_scr.x, mark_scr.y, vp_w, vp_h, state.pitch_deg);
            let mx_i = mx.round() as i32;
            let my_i = my.round() as i32;

            // Pin shadow
            surface.fill_circle(mx_i + 2, my_i + 2, 14, Color::rgba(0, 0, 0, 45));
            // Magenta / Pink Teardrop Pin Body
            let pin_col = Color::rgb(225, 29, 72); // Rose/Crimson
            surface.fill_circle(mx_i, my_i - 16, 14, Color::rgb(255, 255, 255)); // White border
            surface.fill_circle(mx_i, my_i - 16, 12, pin_col);
            // Pin stem point
            surface.draw_line(mx, my - 6.0, mx, my, 4.0, pin_col);
            // Inner white dot
            surface.fill_circle(mx_i, my_i - 16, 4, Color::rgb(255, 255, 255));
        }
    }

    /// Applies 3D perspective pitch transform around horizontal horizon.
    fn apply_3d_transform(x: f32, y: f32, vp_w: f32, vp_h: f32, pitch_deg: f32) -> (f32, f32) {
        if pitch_deg <= 0.1 {
            return (x, y);
        }
        let rad = (pitch_deg as f64) * (atlas_core::PI / 180.0);
        let cos_p = rad.cos() as f32;

        let cy = vp_h / 2.0;
        let dy = y - cy;
        let transformed_y = cy + dy * cos_p;

        // Perspective horizontal scale: closer points spread, farther converge
        let factor = 1.0 + (dy / vp_h) * 0.35;
        let cx = vp_w / 2.0;
        let transformed_x = cx + (x - cx) * factor;

        (transformed_x, transformed_y)
    }
}

/// Fallback 8x16 font bitmap generator.
fn get_8x16_bitmap(ch: char) -> [u8; 16] {
    let mut b = [0u8; 16];
    let ascii = ch as u8;
    if ascii >= 32 && ascii <= 126 {
        let pattern = (ascii.wrapping_mul(41)) ^ 0x55;
        b[2] = pattern & 0x7E;
        b[3] = (pattern >> 1) & 0x7E;
        b[4] = pattern & 0x7E;
        b[5] = 0x66;
        b[6] = 0x66;
        b[7] = 0x7E;
        b[8] = 0x66;
        b[9] = 0x66;
        b[10] = 0x66;
        b[11] = pattern & 0x7E;
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation_and_rasterization() {
        let mut surface = CanvasSurface::new(1024, 768);
        let state = MapState::default();

        VectorMapRenderer::render(&mut surface, &state);

        let non_default = surface.buffer.iter().filter(|&&px| px != 0xFFF4F3F0).count();
        assert!(non_default > 1000, "Expected rich vector map rasterization");
    }

    #[test]
    fn test_color_blend() {
        let fg = Color::rgba(255, 0, 0, 128);
        let bg = 0xFF0000FF; // Pure Blue
        let out = fg.blend_over(bg);
        let r = (out >> 16) & 0xFF;
        let b = out & 0xFF;
        assert!(r > 100 && r < 150);
        assert!(b > 100 && b < 150);
    }
}
