//! Hitbox drawing (player.js `drawHitboxes`): outlines of every collider near the player, the player's own boxes and the hitbox trail,
//! as polygons and circles in screen space for the host to stroke. Records: `[0, colour, alpha, line width, n, x1, y1, ...]` (closed
//! polygon) and `[1, colour, alpha, line width, x, y, radius]`.
use crate::game::Game;
use crate::pcol::T_SLOPE;

const T: f64 = 460.0;
fn b(y: f64) -> f64 { T - y }

#[derive(Clone, Copy, Debug, Default)]
pub struct TrailPoint { pub x: f64, pub y: f64, pub rot: f64, pub size: f64, pub wave: bool }

impl Game {
    /// `_pushHitboxTrail`: 180 positions at most
    pub fn push_hitbox_trail(&mut self) {
        if !self.hitbox_trail_on || self.fx.slide.is_some() { return; }
        let x = self.e.player_world_x;
        for (i, dead, mini, wave, y, rot) in [(0usize, self.p.is_dead, self.p.is_mini, self.p.is_wave, self.p.y, self.p.rotation), (1, self.p2.is_dead, self.p2.is_mini, self.p2.is_wave, self.p2.y, self.p2.rotation)] {
            if dead || (i == 1 && !self.e.is_dual) { continue; }
            let t = &mut self.hitbox_trail[i];
            if t.len() >= 180 { t.remove(0); }
            t.push(TrailPoint { x, y, rot, size: if mini { 18.0 } else { 30.0 }, wave });
        }
    }

    pub fn clear_hitbox_trails(&mut self) { self.hitbox_trail[0].clear(); self.hitbox_trail[1].clear(); }

    /// everything `drawHitboxes` strokes for one player (`second` = the second player)
    fn hitbox_for(&self, second: bool, out: &mut Vec<f64>) {
        let p = if second { &self.p2 } else { &self.p };
        let (sw, cx_screen) = (self.screen_width, self.g.center_x);
        let flipped = p.mirrored;
        let (cam_x, cam_y) = (self.camera_x, self.camera_y);
        let fx = |x: f64| if flipped { sw - x } else { x };
        let poly = |out: &mut Vec<f64>, color: f64, alpha: f64, lw: f64, pts: &[(f64, f64)]| {
            out.extend_from_slice(&[0.0, color, alpha, lw, pts.len() as f64]);
            for (x, y) in pts { out.push(*x); out.push(*y); }
        };
        let circle = |out: &mut Vec<f64>, color: f64, alpha: f64, lw: f64, x: f64, y: f64, r: f64| out.extend_from_slice(&[1.0, color, alpha, lw, x, y, r]);
        let rect = |out: &mut Vec<f64>, color: f64, alpha: f64, lw: f64, x: f64, y: f64, w: f64, h: f64| {
            out.extend_from_slice(&[0.0, color, alpha, lw, 4.0, x, y, x + w, y, x + w, y + h, x, y + h]);
        };
        // the colliders near the screen centre
        for ci in self.nearby(cam_x + cx_screen) {
            let c = &self.cols[ci];
            let color = match c.ty as usize {
                1 => 16729156.0,
                12 | 13 | 14 | 15 | 16 | 17 => 4491519.0,
                9 | 10 | 11 => 16776960.0,
                20 | 21 => 16744448.0,
                22 | 23 => 16711935.0,
                5 => 16744192.0,
                6 => 16711935.0,
                4 => 65535.0,
                2 => 0xaaaaaa as f64,
                3 => 0xffaa00 as f64,
                _ => 65280.0,
            };
            let (ox, oy) = (c.x - cam_x, b(c.y) + cam_y);
            let xpos = fx(ox);
            if let Some(r) = c.hitbox_radius { circle(out, color, 0.7, 2.0, xpos, oy, r); }
            else if c.ty == T_SLOPE && !c.slope_is_filled {
                let (hw, hh) = (c.w / 2.0, c.h / 2.0);
                let (lw_, rw_, bottom, top) = (c.x - hw, c.x + hw, c.y - hh, c.y + hh);
                if let (Some(ls), Some(rs)) = (c.slope_surface_y(lw_), c.slope_surface_y(rw_)) {
                    let solid_below = c.slope_solid_below != p.gravity_flipped;
                    let left_is_bottom = (ls - bottom).abs() < (rs - bottom).abs();
                    let third_x = if solid_below { if left_is_bottom { rw_ } else { lw_ } } else if left_is_bottom { lw_ } else { rw_ };
                    let third_y = if solid_below { bottom } else { top };
                    let sx = |wx: f64| fx(wx - cam_x);
                    poly(out, color, 0.7, 2.0, &[(sx(lw_), b(ls) + cam_y), (sx(rw_), b(rs) + cam_y), (sx(third_x), b(third_y) + cam_y)]);
                }
            } else {
                let rot = c.rotation_degrees * std::f64::consts::PI / 180.0;
                let (cos, sin) = (crate::jsmath::cos(rot), crate::jsmath::sin(rot));
                let (nw, nh, pw, ph) = (-c.w / 2.0, -c.h / 2.0, c.w / 2.0, c.h / 2.0);
                let pts: Vec<(f64, f64)> = [(nw, nh), (pw, nh), (pw, ph), (nw, ph)].iter().map(|&(px, py)| {
                    let (rx, ry) = (px * cos - py * sin, px * sin + py * cos);
                    (xpos + if flipped { -rx } else { rx }, oy + if flipped { -ry } else { ry })
                }).collect();
                poly(out, color, 0.7, 2.0, &pts);
            }
        }
        // the trail
        if self.hitbox_trail_on {
            for t in &self.hitbox_trail[second as usize] {
                let tx = fx(t.x - cam_x);
                let ty = b(t.y) + cam_y;
                let (size, outer) = (t.size, t.size * 2.0);
                if !t.wave {
                    rect(out, 0xff0000 as f64, 0.5, 1.0, tx - size, ty - size, outer, outer);
                    circle(out, 0xb30001 as f64, 0.5, 1.0, tx, ty, outer / 2.0);
                    let (cx, cy, hw) = (tx, ty, outer / 2.0);
                    let (cs, sn) = (crate::jsmath::cos(t.rot), crate::jsmath::sin(t.rot));
                    poly(out, 0xb30001 as f64, 0.5, 1.0, &[(cx - hw * cs + hw * sn, cy - hw * sn - hw * cs), (cx + hw * cs + hw * sn, cy + hw * sn - hw * cs), (cx + hw * cs - hw * sn, cy + hw * sn + hw * cs), (cx - hw * cs - hw * sn, cy - hw * sn + hw * cs)]);
                    rect(out, 0x0000ff as f64, 1.0, 1.0, tx - 9.0, ty - 9.0, 18.0, 18.0);
                } else {
                    let wo = if size == 18.0 { 6.0 } else { 9.0 };
                    let wi = wo / 3.0;
                    rect(out, 0xff0000 as f64, 0.5, 1.0, tx - wo, ty - wo, wo * 2.0, wo * 2.0);
                    rect(out, 0x0000ff as f64, 1.0, 1.0, tx - wi, ty - wi, wi * 2.0, wi * 2.0);
                }
            }
        }
        // the player
        let size = if p.is_mini { 18.0 } else { 30.0 };
        let box_size = size * 2.0;
        let py = b(p.y) + cam_y;
        let px = if flipped { sw - cx_screen } else { cx_screen };
        if !p.is_wave {
            rect(out, 0xff0000 as f64, 0.8, 2.0, px - size, py - size, box_size, box_size);
            circle(out, 0xb30001 as f64, 0.8, 2.0, px, py, box_size / 2.0);
            let hw = box_size / 2.0;
            let (cs, sn) = (crate::jsmath::cos(p.rotation), crate::jsmath::sin(p.rotation));
            poly(out, 0xb30001 as f64, 0.8, 2.0, &[(px - hw * cs + hw * sn, py - hw * sn - hw * cs), (px + hw * cs + hw * sn, py + hw * sn - hw * cs), (px + hw * cs - hw * sn, py + hw * sn + hw * cs), (px - hw * cs - hw * sn, py - hw * sn + hw * cs)]);
            rect(out, 0x0000ff as f64, 1.0, 2.0, px - 9.0, py - 9.0, 18.0, 18.0);
        } else {
            let wo = if p.is_mini { 6.0 } else { 9.0 };
            let wi = wo / 3.0;
            rect(out, 0xff0000 as f64, 0.8, 2.0, px - wo, py - wo, wo * 2.0, wo * 2.0);
            rect(out, 0x0000ff as f64, 1.0, 2.0, px - wi, py - wi, wi * 2.0, wi * 2.0);
        }
    }

    /// the frame's hitbox records (both players when dual), empty unless hitboxes are on
    pub fn hitbox_shapes(&self) -> Vec<f64> {
        let mut out = Vec::new();
        if !(self.show_hitboxes || self.hitboxes_on_death) { return out; }
        self.hitbox_for(false, &mut out);
        if self.e.is_dual { self.hitbox_for(true, &mut out); }
        out
    }
}
