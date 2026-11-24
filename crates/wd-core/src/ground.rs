//! Ground / ceiling tile layout and the fly-mode floor animation (level.js `_buildGround`,
//! `updateGroundTiles`, `setFlyMode`, `stepGroundAnimation`, `shiftGroundTiles`, `resetGroundTiles`).

pub const SHAKE_MAX_OFFSET: f64 = 200.0;
const T: f64 = 460.0;
fn b(y: f64) -> f64 { T - y }

#[derive(Clone, Debug)]
pub struct Ground {
    pub screen_width: f64,
    pub tile_w: f64,
    pub world_x: Vec<f64>,
    pub max_world_x: f64,
    pub ground_shift: f64,
    pub fly_ground_active: bool,
    pub ground_target: f64,
    pub animating: bool,
    pub anim_from: f64,
    pub anim_to: f64,
    pub anim_time: f64,
    pub anim_duration: f64,
    pub fly_floor_y: Option<f64>,
    pub fly_ceiling_y: Option<f64>,
    pub fly_visual_only: bool,
    pub floor_inset: f64,
    pub ceil_inset: f64,
    pub fly_camera_target: Option<f64>,
}

/// Everything `updateGroundTiles` writes, for the renderer.
#[derive(Clone, Debug, Default)]
pub struct GroundFrame {
    pub tile_x: Vec<f64>,
    pub ground_y: f64,
    pub ceiling_y: f64,
    pub ceiling_visible: bool,
    pub line_x: f64,
    pub ground2_visible: bool,
    pub ceiling2_visible: bool,
}

impl Ground {
    /// `centre_x` is `screenWidth / 2 - 150`; `tile_w` is the ground texture width (1012 when missing).
    pub fn new(screen_width: f64, tile_w: f64) -> Self {
        let centre_x = screen_width / 2.0 - 150.0;
        let count = (screen_width / tile_w).ceil() as usize + 2;
        let start_x = -centre_x;
        let world_x = (0..count).map(|i| start_x + i as f64 * tile_w).collect();
        Ground {
            screen_width, tile_w, world_x,
            max_world_x: start_x + (count as f64 - 1.0) * tile_w,
            ground_shift: ((start_x % tile_w) + tile_w) % tile_w,
            fly_ground_active: false, ground_target: 0.0, animating: false,
            anim_from: 0.0, anim_to: 0.0, anim_time: 0.0, anim_duration: 0.5,
            fly_floor_y: None, fly_ceiling_y: None, fly_visual_only: false,
            floor_inset: 0.0, ceil_inset: 0.0, fly_camera_target: None,
        }
    }

    pub fn reset_state(&mut self) {
        self.fly_ground_active = false;
        self.ground_target = 0.0;
        self.animating = false;
        self.fly_ceiling_y = None;
        self.fly_visual_only = false;
        self.floor_inset = 0.0;
        self.ceil_inset = 0.0;
        self.fly_camera_target = None;
    }

    pub fn shift(&mut self, amount: f64) {
        for w in &mut self.world_x { *w += amount; }
        self.max_world_x += amount;
        self.ground_shift = (((self.ground_shift + amount) % self.tile_w) + self.tile_w) % self.tile_w;
    }

    pub fn reset_tiles(&mut self, camera_x: f64) {
        let tw = self.tile_w;
        for (i, w) in self.world_x.iter_mut().enumerate() { *w = camera_x + i as f64 * tw; }
        self.max_world_x = camera_x + (self.world_x.len() as f64 - 1.0) * tw;
        self.ground_shift = ((camera_x % tw) + tw) % tw;
        self.reset_state();
    }

    fn compute_fly_bounds(centre_y: f64, height: f64, is_portal: bool) -> (f64, f64) {
        let mut floor = if is_portal { centre_y - 600.0 / 2.0 } else { centre_y - height / 2.0 };
        floor = (floor / 60.0).floor() * 60.0;
        floor = floor.max(0.0);
        (floor, floor + height)
    }

    pub fn set_fly_mode(&mut self, enabled: bool, centre_y: f64, height: f64, visual_only: bool, camera_height: f64, scene_camera_y: f64) {
        if enabled {
            let (floor, ceiling) = Self::compute_fly_bounds(centre_y, height, visual_only);
            self.fly_floor_y = Some(floor);
            self.fly_ceiling_y = Some(ceiling);
            self.fly_visual_only = visual_only;
            let span = if camera_height.is_finite() { camera_height } else { height };
            let shrink = (600.0 - span).max(0.0);
            self.floor_inset = if visual_only { 0.0 } else { shrink / 2.0 };
            self.ceil_inset = self.floor_inset;
            self.fly_ground_active = true;
            let centre = floor + span / 2.0;
            let mut target = centre - 320.0 + 180.0;
            if target < 0.0 { target = 0.0; }
            self.fly_camera_target = Some(target);
            let _ = scene_camera_y;
            self.anim_from = self.ground_target;
            self.anim_to = 1.0;
            self.anim_time = 0.0;
            self.anim_duration = 0.5;
            self.animating = true;
        } else {
            self.fly_camera_target = None;
            self.fly_ceiling_y = None;
            self.fly_floor_y = None;
            self.fly_visual_only = false;
            self.floor_inset = 0.0;
            self.ceil_inset = 0.0;
            if self.fly_ground_active {
                self.anim_from = self.ground_target;
                self.anim_to = 0.0;
                self.anim_time = 0.0;
                self.anim_duration = 0.5;
                self.animating = true;
                self.fly_ground_active = false;
            } else {
                self.animating = false;
                self.ground_target = 0.0;
            }
        }
    }

    pub fn step_animation(&mut self, dt: f64) {
        if !self.animating { return; }
        self.anim_time += dt;
        let progress = if self.anim_duration > 0.0 { (self.anim_time / self.anim_duration).min(1.0) } else { 1.0 };
        self.ground_target = self.anim_from + (self.anim_to - self.anim_from) * progress;
        if progress >= 1.0 {
            self.animating = false;
            self.ground_target = self.anim_to;
            if self.anim_to == 0.0 { self.fly_ground_active = false; }
        }
    }

    pub fn floor_y(&self) -> f64 {
        if self.fly_ground_active { if self.fly_visual_only { 0.0 } else { self.fly_floor_y.unwrap_or(f64::NAN) } } else { 0.0 }
    }

    pub fn update(&mut self, camera_x: f64, camera_y: f64, shake_x: f64, shake_y: f64, has_ground2: bool) -> GroundFrame {
        let tw = self.tile_w;
        let ceiling_active = !self.fly_ground_active && self.fly_ceiling_y.is_some();
        let (left, right);
        if self.fly_visual_only && self.fly_ceiling_y.is_some() {
            left = b(0.0) + camera_y;
            right = b(self.fly_ceiling_y.unwrap()) + camera_y;
        } else if self.fly_ground_active && self.ground_target > 0.001 {
            let gt = self.ground_target;
            let target_ground = 620.0 - self.floor_inset;
            let target_ceiling = 20.0 + self.ceil_inset;
            let ground_screen = b(0.0) + camera_y;
            let mut l = ground_screen + (target_ground - ground_screen) * gt;
            right = 0.0 + (target_ceiling - 0.0) * gt;
            if l > ground_screen { l = ground_screen; }
            left = l;
        } else {
            left = b(0.0) + camera_y;
            right = if ceiling_active { 20.0 } else { 0.0 };
        }
        let n = self.world_x.len();
        let nf = n as f64;
        let first_col = ((camera_x - SHAKE_MAX_OFFSET - self.ground_shift) / tw).floor();
        let mut max_w = f64::NEG_INFINITY;
        let ceil_vis = (self.fly_ground_active && self.ground_target > 0.0) || ceiling_active;
        let mut tile_x = Vec::with_capacity(n);
        for i in 0..n {
            let col = first_col + ((((i as f64 - first_col) % nf) + nf) % nf);
            let wx = col * tw + self.ground_shift;
            self.world_x[i] = wx;
            if wx > max_w { max_w = wx; }
            self.max_world_x = max_w;
            tile_x.push(wx - camera_x + shake_x);
        }
        GroundFrame {
            tile_x, ground_y: left + shake_y, ceiling_y: right + shake_y,
            ceiling_visible: ceil_vis, line_x: self.screen_width / 2.0 + shake_x,
            ground2_visible: has_ground2, ceiling2_visible: has_ground2 && ceil_vis,
        }
    }
}
