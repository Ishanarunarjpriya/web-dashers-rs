//! The slide-in after pressing play (`_startGame` → `_slideIn` branch of `update`): the cube glides in from the left edge of the
//! screen while the camera creeps along at a quarter of the speed; when it reaches its start position the attempt begins.
use crate::game::Game;
use crate::physics::{Sim, D};

impl Game {
    /// `_startGame` for a normal level: reset for a fresh attempt, then put the camera and player at the left edge.
    pub fn begin_slide(&mut self) {
        self.log.clear();
        self.restart_level(None);
        let cx = self.g.center_x;
        let menu_cam = self.fx.menu_camera_x;
        self.camera_x = -cx;
        self.camera_y = 0.0;
        self.e.player_world_x = self.camera_x;
        self.delta_buffer = 0.0;
        if let Some(g) = self.fx.ground.as_mut() { g.shift(self.camera_x - menu_cam); }
        self.fx.slide = Some(self.camera_x);
        self.fx.slide_done = false;
        self.fx.bg.prev_camera_x = self.camera_x;
    }

    /// Coins the level has for the end screen: secret coins (main levels) or user coins, at most 3.
    pub fn coin_count(&self, main: bool) -> u32 {
        let key = if main { "_secretCoinSlot" } else { "_userCoinSlot" };
        (self.fx.sprites.iter().filter(|s| s.num(key).is_some()).count() as u32).min(3)
    }

    pub fn slide_active(&self) -> bool { self.fx.slide.is_some() }

    /// One frame of the slide-in. Returns true when it ended this frame.
    pub fn slide_step(&mut self, delta_ms: f64) -> bool {
        let ground_x = self.fx.slide.unwrap_or(self.camera_x);
        let delta = self.slide_quantize(delta_ms);
        let speed = if self.g.player_speed.is_nan() { crate::physics::speed_portal::ONE_TIMES } else { self.g.player_speed };
        self.e.player_world_x += delta * speed * D;
        let ground_x = ground_x + delta * speed * D * 0.25;
        // `_cameraX` stays where `_startGame` put it: objects and background stand still, only the ground moves (`_slideGroundX`)
        self.fx.slide = Some(ground_x);
        { let mut sim = Sim::new(&mut self.p, &mut self.g, &mut self.e); sim.update_ground_rotation(delta * D); }
        self.update_visibility(self.camera_x);
        self.fx.player_x = self.e.player_world_x;
        self.render_step(delta_ms);
        if self.e.player_world_x >= 0.0 {
            self.fx.slide = None;
            self.fx.slide_done = true;
            self.delta_buffer = 0.0;
            self.e.player_world_x = 0.0;
            self.p.last_collision_world_x = None; self.p.last_collision_world_y = None;
            self.p2.last_collision_world_x = None; self.p2.last_collision_world_y = None;
            self.camera_x = self.e.player_world_x - self.g.center_x;
            if let Some(g) = self.fx.ground.as_mut() { g.shift(self.camera_x - ground_x); }
            self.fx.player_x = self.e.player_world_x;
            return true;
        }
        false
    }
}
