//! Scrolling background and the background/ground art-id helpers
//! (game-scene.js `_updateBackground`, `_applyMirroredBackgroundTexture`; level.js `getBackgroundTextureIndex`, `getGroundTextureId`).

use crate::js::parse_int;

/// `getBackgroundTextureIndex(setting)`; `None` stands for null/undefined (which reads as "1").
pub fn background_texture_index(setting: Option<&str>) -> i64 {
    let id = parse_int(setting.unwrap_or("1"));
    match id { Some(n) if n > 1 => n - 1, _ => 0 }
}

/// `getBackgroundDisplayId`: two digits, at least 1.
pub fn background_display_id(setting: Option<&str>) -> String {
    let id = match parse_int(setting.unwrap_or("1")) { Some(n) if n > 1 => n, _ => 1 };
    format!("{:02}", id)
}

/// `getGroundTextureId`: texture index (setting - 1, at least 0) as two digits.
pub fn ground_texture_id(setting: Option<&str>) -> String {
    let id = match parse_int(setting.unwrap_or("1")) { Some(n) if n > 1 => n - 1, _ => 0 };
    format!("{:02}", id)
}

#[derive(Clone, Debug)]
pub struct Background {
    pub tile_x: f64,
    pub tile_y: f64,
    pub speed_x: f64,
    pub speed_y: f64,
    pub init_y: f64,
    pub mirror_tile_h: f64,
    pub prev_camera_x: f64,
}

impl Default for Background { fn default() -> Self { Self::new() } }

impl Background {
    pub fn new() -> Self {
        Background { tile_x: 0.0, tile_y: 0.0, speed_x: 0.1, speed_y: 0.1, init_y: 0.0, mirror_tile_h: 0.0, prev_camera_x: 0.0 }
    }

    /// `_applyMirroredBackgroundTexture`: `source_height` is the background image height (0 when unknown).
    pub fn apply_texture(&mut self, source_height: f64) {
        self.init_y = if source_height > 0.0 { source_height - 640.0 - 180.0 } else { 0.0 };
        self.mirror_tile_h = if source_height > 0.0 { source_height * 2.0 } else { 0.0 };
    }

    /// `_updateBackground`. Returns the amount the ground tiles must be shifted by (0 when none).
    pub fn update(&mut self, camera_x: f64, camera_y: f64, mirror_progress: f64) -> f64 {
        let dx = camera_x - self.prev_camera_x;
        let p = mirror_progress;
        self.tile_x += dx * self.speed_x * (1.0 - 2.0 * p);
        let shift = if p > 0.0 && dx != 0.0 { 2.0 * p * dx } else { 0.0 };
        self.prev_camera_x = camera_x;
        let mut ty = self.init_y - camera_y * self.speed_y;
        if self.mirror_tile_h > 0.0 {
            ty = ((ty % self.mirror_tile_h) + self.mirror_tile_h) % self.mirror_tile_h;
        }
        self.tile_y = ty;
        shift
    }
}
