//! Mirror portals (game-scene.js `toggleMirror`, `_stepMirror`, `_mirrorProgress`, `_applyMirrorEffect`, `_mirrorTransformObjects`).
//! The simulation only logs `mirror:on/off` when a mirror portal is touched; the scene owns the animation and flips `state.mirrored`
//! half way through it. `enabled` is off in the physics test harnesses (their scene mock never flips anything).

use crate::game::Game;
use crate::jsmath;

#[derive(Clone, Debug, Default)]
pub struct Mirror { pub enabled: bool, pub animating: bool, pub target: bool, pub from: f64, pub p: f64, pub time: f64 }

impl Game {
    /// `_mirrorProgress`
    pub fn mirror_progress(&self) -> f64 {
        if self.mirror.animating { self.mirror.p } else if self.p.mirrored { 1.0 } else { 0.0 }
    }

    /// `toggleMirror`: false when the mirror already is (or is heading to) that state.
    pub fn toggle_mirror(&mut self, on: bool) -> bool {
        let current = if self.mirror.animating { self.mirror.target } else { self.p.mirrored };
        if current == on { return false; }
        self.mirror.from = self.mirror_progress();
        self.mirror.p = self.mirror.from;
        self.mirror.target = on;
        self.mirror.time = 0.0;
        self.mirror.animating = true;
        true
    }

    /// `_stepMirror(dt)` (seconds)
    pub fn step_mirror(&mut self, dt: f64) {
        if !self.mirror.animating { return; }
        self.mirror.time += dt;
        let k = (self.mirror.time / 0.4).min(1.0);
        let t = k * 2.0;
        let eased = if t < 1.0 { 0.5 * jsmath::pow(t, 1.2) } else { 1.0 - 0.5 * jsmath::pow(2.0 - t, 1.2) };
        let to = if self.mirror.target { 1.0 } else { 0.0 };
        self.mirror.p = self.mirror.from + (to - self.mirror.from) * eased;
        if k >= 1.0 { self.mirror.p = to; self.mirror.animating = false; }
        let visual = if self.mirror.animating { self.mirror.p >= 0.5 } else { self.mirror.target };
        self.p.mirrored = visual;
        self.p2.mirrored = visual;
    }

    /// The attempt start/respawn paths set `state.mirrored` themselves and drop any running animation.
    pub fn mirror_cancel(&mut self) { self.mirror.animating = false; }

    /// Turns the `mirror:` log entries of a simulation call into `toggleMirror` calls, in order.
    pub(crate) fn consume_mirror_log(&mut self, from: usize) {
        if !self.mirror.enabled { return; }
        let ons: Vec<bool> = self.log[from..].iter().filter_map(|l| l.strip_prefix("mirror:").map(|v| v == "true")).collect();
        for on in ons { self.toggle_mirror(on); }
    }

    /// `_getMirrorXOffset`
    pub fn mirror_x_offset(&self, x: f64) -> f64 { x + (self.screen_width - 2.0 * x) * self.mirror_progress() }

    /// What `_applyMirrorEffect` leaves a level sprite looking like: (x, scale_x, rotation, alpha, visible).
    pub fn mirror_view(&self, si: usize) -> (f64, f64, f64, f64, bool) {
        let s = &self.fx.sprites[si];
        let plain = (s.x, s.scale_x, s.rotation, s.alpha, true);
        if !self.mirror.enabled { return plain; }
        let p = self.mirror_progress();
        if p <= 0.0 { return plain; }
        let sw = self.screen_width;
        let shake = self.trig.shake_off.0;
        let top = s.num("_eeLayer") == Some(2.0);
        if p >= 1.0 {
            // the containers are flipped: screen x = cameraX + sw - shake - x
            return (sw - s.x + 2.0 * self.camera_x - 2.0 * shake, -s.scale_x, -s.rotation, s.alpha, true);
        }
        // direct children of the top container only move when they know their world x
        if top && s.num("_eeWorldX").is_none() { return plain; }
        let ox = -self.camera_x + shake;
        let sx = s.x + ox;
        let dont_fade = s.flag("_eeDontFade");
        let fade = if dont_fade { 1.0 } else { 1f64.min(sx.min(sw - sx) / 140.0) };
        if fade <= 0.0 { return (s.x, s.scale_x, s.rotation, s.alpha, false); }
        let alpha = if fade < 1.0 { s.alpha * fade } else { s.alpha };
        let x = s.x + p * (sw - 2.0 * sx);
        if p >= 0.5 { (x, -s.scale_x, -s.rotation, alpha, true) } else { (x, s.scale_x, s.rotation, alpha, true) }
    }
}
