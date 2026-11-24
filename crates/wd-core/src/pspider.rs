//! Spider jump update (player.js `_updateSpiderJump` and its teleport-search helpers).
use crate::jsmath as m;
use crate::pcol::Col;
use crate::physics::{gd_trunc, Sim, P};

struct Bounds { left: f64, right: f64, lower: f64, upper: f64 }

fn or0(v: f64) -> f64 { if v == 0.0 || v.is_nan() { 0.0 } else { v } }

fn search_bounds(o: &Col) -> Bounds {
    if let Some(r) = o.hitbox_radius {
        if r.is_finite() && r > 0.0 {
            return Bounds { left: o.x - r, right: o.x + r, lower: o.y - r, upper: o.y + r };
        }
    }
    let rad = or0(o.rotation_degrees) * std::f64::consts::PI / 180.0;
    let (cos, sin) = (m::cos(rad), m::sin(rad));
    let (half_w, half_h) = (o.w / 2.0, o.h / 2.0);
    let rhw = (half_w * cos).abs() + (half_h * sin).abs();
    let rhh = (half_w * sin).abs() + (half_h * cos).abs();
    Bounds { left: o.x - rhw, right: o.x + rhw, lower: o.y - rhh, upper: o.y + rhh }
}

impl<'a> Sim<'a> {
    fn find_spider_teleport_hazard(&self, going_up: bool, world_x: f64, player_size: f64, target_center_y: f64, cols: &[Col]) -> Option<Bounds> {
        if !target_center_y.is_finite() { return None; }
        let x_pad = 9f64.max(player_size - 5.0);
        let y_pad = x_pad;
        let start_y = self.p.y;
        let from_y = start_y.min(target_center_y);
        let to_y = start_y.max(target_center_y);
        let mut nearest: Option<Bounds> = None;
        let mut nearest_distance = f64::INFINITY;
        for o in cols {
            if o.ty != 1.0 { continue; }
            let b = search_bounds(o);
            if world_x + x_pad <= b.left || world_x - x_pad >= b.right { continue; }
            if b.upper < from_y - y_pad || b.lower > to_y + y_pad { continue; }
            let entry_y = if going_up { start_y.max(b.lower) } else { start_y.min(b.upper) };
            let distance = (entry_y - start_y).abs();
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = Some(b);
            }
        }
        nearest
    }

    fn find_spider_teleport_surface(&self, going_up: bool, world_x: f64, player_size: f64, cols: &[Col]) -> Option<f64> {
        let x_pad = 9f64.max(player_size - 5.0);
        let mut best: Option<f64> = None;
        if going_up {
            if let Some(c) = self.e.ceiling_y { if c > self.p.y { best = Some(c); } }
        } else {
            let f = self.e.floor_y;
            if f < self.p.y { best = Some(f); }
        }
        for o in cols {
            if o.ty != 0.0 { continue; }
            let b = search_bounds(o);
            if world_x + x_pad <= b.left || world_x - x_pad >= b.right { continue; }
            if going_up {
                let under = b.lower;
                if under <= self.p.y + 0.5 { continue; }
                if best.map_or(true, |bs| under < bs) { best = Some(under); }
            } else {
                let top = b.upper;
                if top >= self.p.y - 0.5 { continue; }
                if best.map_or(true, |bs| top > bs) { best = Some(top); }
            }
        }
        best
    }

    pub fn update_spider_jump(&mut self, dt: f64, cols: &[Col]) {
        if !self.p.rotate_action_active { self.update_ground_rotation(dt); }
        let player_size = if self.p.is_mini { 18.0 } else { 30.0 };
        let grav_amt = P * 0.6;
        let can_teleport = self.p.can_jump || self.p.on_ground || self.p.on_ceiling;
        let prioritize_orb = self.should_prioritize_ground_orb_input();
        if !prioritize_orb && self.p.up_key_pressed && can_teleport {
            self.p.up_key_pressed = false;
            self.p.queued_hold = false;
            let world_x = self.e.player_world_x;
            let going_up = !self.p.gravity_flipped;
            let nearest = self.find_spider_teleport_surface(going_up, world_x, player_size, cols);
            if let Some(surface) = nearest.filter(|v| v.is_finite()) {
                let old_teleport_y = self.p.y;
                let normal_landing_y = if going_up { surface - player_size } else { surface + player_size };
                let blocking = self.find_spider_teleport_hazard(going_up, world_x, player_size, normal_landing_y, cols);
                if blocking.is_some() && !self.g.no_clip {
                    let b = blocking.unwrap();
                    let hazard_center_y = (b.lower + b.upper) / 2.0;
                    self.p.y = if hazard_center_y.is_finite() { hazard_center_y } else { normal_landing_y };
                    self.vlog.push(format!("tp:{},{}", old_teleport_y, self.p.y));
                    self.p.y_velocity = 0.0;
                    self.p.on_ground = false;
                    self.p.can_jump = false;
                    self.p.is_jumping = false;
                    self.kill_player();
                    return;
                }
                self.p.y = normal_landing_y;
                if going_up {
                    self.flip_gravity(true, 1.0);
                    self.p.on_ceiling = true;
                } else {
                    self.flip_gravity(false, 1.0);
                    self.p.on_ceiling = false;
                }
                if blocking.is_some() && self.g.no_clip {
                    self.p.spider_teleport_noclip_death_pending = true;
                    self.p.died_this_frame = true;
                }
                self.vlog.push(format!("tp:{},{}", old_teleport_y, self.p.y));
                self.p.y_velocity = 0.0;
                self.p.on_ground = true;
                self.p.can_jump = true;
                self.p.is_jumping = false;
            } else {
                self.flip_gravity(going_up, 1.0);
                self.p.y_velocity = 0.0;
                self.p.on_ground = false;
                self.p.can_jump = false;
                self.p.is_jumping = false;
            }
            self.p.spider_teleport_anim_timer = 0.0;
            self.p.spider_flash_duration = 0.5;
            self.p.spider_flash_timer = 0.5;
            self.run_rotate_action(1.0);
            return;
        }
        if self.player_is_falling() { self.p.can_jump = false; }
        self.p.y_velocity = gd_trunc(self.p.y_velocity - grav_amt * dt * self.flip_mod());
        self.p.y_velocity = if self.p.gravity_flipped { self.p.y_velocity.min(30.0) } else { self.p.y_velocity.max(-30.0) };
        if self.player_is_falling() {
            let past = if self.p.gravity_flipped { self.p.y_velocity > P * 2.0 } else { self.p.y_velocity < -(P * 2.0) };
            if past { self.p.on_ground = false; }
        }
    }
}
