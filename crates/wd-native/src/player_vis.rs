//! The player's picture (port of the drawing side of `player.js` as `web/player_vis.js` has it): the icon layers for every mode,
//! the wave trail and the death burst. State comes from `wd_pv_state`.
use crate::assets::{load_png, Assets, Frame, Image};
use crate::canvas::{Canvas, Sprite, W};
use crate::plist;
use std::collections::HashMap;
use wd_core as core;

const T: f64 = 460.0;

pub struct IconSheet { pub img: Image, pub frames: HashMap<String, Frame> }

#[derive(Clone)]
struct Item { kind: &'static str, name: String, depth: i32, order: usize, x: f64, y: f64, rot: f64, sx: f64, sy: f64, tint: i64, additive: bool }

struct Pt { x: f64, y: f64, age: f64 }

struct Burst { parts: Vec<(f64, f64, f64, f64, f64, f64)>, tint: i64, flash: f64, fx: f64, fy: f64 }

pub struct PlayerVis {
    icons: HashMap<String, Option<IconSheet>>,
    items: Vec<Item>,
    wave: Vec<Pt>,
    wave_active: bool,
    wave_mini: f64,
    was_dead: bool,
    burst: Option<Burst>,
    rng: u64,
    pub main: i64,
    pub second: i64,
    cfg_player: String, cfg_ship: String, cfg_ball: String, cfg_wave: String, cfg_bird: String,
    hidden: bool,
    ufo_last: Option<(f64, f64)>, ufo_tilt: f64, ufo_rot: f64,
    mirror_p: f64,
    ox: f64, oy: f64,
}

/// `iconBaseOf`: `player_01_2_001.png` -> `player_01`
pub fn icon_base(name: &str) -> Option<String> {
    for prefix in ["player_ball", "player", "ship", "dart", "bird", "spider", "robot"] {
        let Some(rest) = name.strip_prefix(prefix).and_then(|r| r.strip_prefix('_')) else { continue };
        let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if !(2..=3).contains(&digits) { continue; }
        let tail = &rest[digits..];
        if tail == "_001.png" || (tail.starts_with('_') && tail.ends_with("_001.png")) { return Some(format!("{prefix}_{}", &rest[..digits])); }
    }
    None
}

fn variant(prefix: &str, kind: &str) -> String {
    match kind { "glow" => format!("{prefix}_glow_001.png"), "overlay" => format!("{prefix}_2_001.png"), "extra" => format!("{prefix}_extra_001.png"), _ => format!("{prefix}_001.png") }
}

impl PlayerVis {
    pub fn new() -> PlayerVis {
        PlayerVis { icons: HashMap::new(), items: Vec::new(), wave: Vec::new(), wave_active: false, wave_mini: 1.0, was_dead: false, burst: None, rng: 0x9e3779b97f4a7c15,
            main: 0x04ff00, second: 0x00fbff, cfg_player: "player_01".into(), cfg_ship: "ship_01".into(), cfg_ball: "player_ball_01".into(), cfg_wave: "dart_01".into(), cfg_bird: "bird_01".into(),
            hidden: false, ufo_last: None, ufo_tilt: 0.0, ufo_rot: f64::NAN, mirror_p: 0.0, ox: 0.0, oy: 0.0 }
    }

    fn rnd(&mut self, a: f64, b: f64) -> f64 {
        self.rng ^= self.rng << 13; self.rng ^= self.rng >> 7; self.rng ^= self.rng << 17;
        a + (self.rng >> 11) as f64 / (1u64 << 53) as f64 * (b - a)
    }

    fn icon(&mut self, root: &std::path::Path, base: &str) -> Option<&IconSheet> {
        if !self.icons.contains_key(base) {
            let sheet = (|| {
                let xml = std::fs::read_to_string(root.join("icons").join(format!("{base}-hd.plist"))).ok()?;
                let img = load_png(&root.join("icons").join(format!("{base}-hd.png"))).ok()?;
                let frames = plist::frames(&plist::parse(&xml)?);
                if frames.is_empty() { None } else { Some(IconSheet { img, frames }) }
            })();
            self.icons.insert(base.to_string(), sheet);
        }
        self.icons.get(base).and_then(|s| s.as_ref())
    }

    /// An icon atlas by base name (loaded on first use).
    pub fn icon_sheet(&mut self, root: &std::path::Path, base: &str) -> Option<&IconSheet> { self.icon(root, base) }
    pub fn icon_loaded(&self, base: &str) -> Option<&IconSheet> { self.icons.get(base).and_then(|s| s.as_ref()) }
    /// Colours and icon names from the saved choices (`loadCfg`).
    pub fn set_cfg(&mut self, main: i64, second: i64, player: &str, ship: &str, ball: &str, wave: &str, bird: &str) {
        self.main = main; self.second = second; self.cfg_player = player.into(); self.cfg_ship = ship.into(); self.cfg_ball = ball.into(); self.cfg_wave = wave.into(); self.cfg_bird = bird.into();
    }

    /// `getFrame`: the icon's own atlas, else the shared sheets.
    fn find<'a>(&'a self, assets: &'a Assets, name: &str) -> Option<(&'a Image, Frame)> {
        if let Some(base) = icon_base(name) {
            if let Some(Some(s)) = self.icons.get(&base) { return s.frames.get(name).map(|f| (&s.img, *f)); }
        }
        const PRIORITY: [&str; 9] = ["GJ_GameSheet03", "GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet04", "GJ_GameSheetEditor", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_WebSheet", "GJ_LaunchSheet"];
        const ALL: [&str; 12] = ["GJ_WebSheet", "GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet03", "GJ_GameSheet04", "GJ_GameSheetEditor", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_LaunchSheet", "GJ_ParticleSheet", "PixelSheet_01", "GJ_EditorSheet01"];
        let list: &[&str] = if name.starts_with("player_") { &PRIORITY } else { &ALL };
        for a in list { if let Some(sh) = assets.sheets.get(*a) { if let Some(f) = sh.frames.get(name) { return Some((&sh.img, *f)); } } }
        None
    }

    fn tint_of(&self, kind: &str, inverted: bool) -> i64 {
        let (p, s) = if inverted { (self.second, self.main) } else { (self.main, self.second) };
        match kind { "base" => p, "overlay" | "glow" => s, _ => -1 }
    }

    fn push(&mut self, assets: &Assets, kind: &'static str, name: &str, depth: i32, x: f64, y: f64, rot: f64, sx: f64, sy: f64, inverted: bool) {
        if self.find(assets, name).is_none() { return; }
        let tint = self.tint_of(kind, inverted);
        let order = self.items.len();
        self.items.push(Item { kind, name: name.to_string(), depth, order, x, y, rot, sx, sy, tint, additive: false });
    }

    fn layers(prefix: &str, bird: bool) -> Vec<(&'static str, String, i32)> {
        if bird { vec![("glow", format!("{prefix}_2_001.png"), 9), ("base", format!("{prefix}_001.png"), 10), ("overlay", format!("{prefix}_3_001.png"), 8), ("extra", format!("{prefix}_extra_001.png"), 12)] }
        else { vec![("glow", variant(prefix, "glow"), 9), ("base", variant(prefix, "base"), 10), ("overlay", variant(prefix, "overlay"), 8), ("extra", variant(prefix, "extra"), 12)] }
    }

    /// Reads the player state and rebuilds the layers; `dt` in seconds.
    pub fn frame(&mut self, assets: &Assets, dt: f64, cam: (f64, f64), menu: bool, hidden: bool, restart: bool, mirror_p: f64) {
        self.hidden = hidden;
        self.mirror_p = mirror_p;
        self.ox = -cam.0; self.oy = cam.1;
        let root = assets.root.clone();
        for n in [self.cfg_player.clone(), self.cfg_ship.clone(), self.cfg_ball.clone(), self.cfg_wave.clone(), self.cfg_bird.clone()] {
            if let Some(b) = icon_base(&format!("{n}_001.png")) { self.icon(&root, &b); }
        }
        let a: Vec<f64> = unsafe { std::slice::from_raw_parts(core::wd_pv_state(0), 26) }.to_vec();
        let dual = a[25] != 0.0;
        self.items.clear();
        let dead = a[3] != 0.0;
        if dead && !self.was_dead && !menu { self.kill(&a); }
        if !dead && self.was_dead { self.burst = None; }
        if restart { self.burst = None; self.wave.clear(); }
        self.was_dead = dead;
        self.layout(assets, &a, dt, false);
        if dual {
            let b: Vec<f64> = unsafe { std::slice::from_raw_parts(core::wd_pv_state(1), 26) }.to_vec();
            self.layout(assets, &b, dt, true);
        }
        if menu || dead { self.items.clear(); }
        // the wave trail
        let mini = a[4] != 0.0;
        let wave_now = a[6] != 0.0 && !dead && !menu;
        if wave_now && !self.wave_active { self.wave.clear(); self.wave_active = true; } else if !wave_now && self.wave_active { self.wave_active = false; self.wave.clear(); }
        self.wave_update(dt, cam.0, (a[1], T - a[0]), mini);
        // the burst
        if let Some(b) = &mut self.burst {
            let ms = dt * 1000.0;
            for p in b.parts.iter_mut() { p.4 += ms; p.0 += p.2 * dt; p.1 += p.3 * dt; }
            b.parts.retain(|p| p.4 < p.5);
            b.flash += ms / 300.0;
        }
    }

    fn layout(&mut self, assets: &Assets, p: &[f64], dt: f64, inverted: bool) {
        let (y, x, rot) = (p[0], p[1], p[2]);
        let (mini_f, gf, mir) = (if p[4] != 0.0 { 0.6 } else { 1.0 }, p[11] != 0.0, p[12] != 0.0);
        let (ship, wave, ball, ufo) = (p[5] != 0.0, p[6] != 0.0, p[7] != 0.0, p[8] != 0.0);
        let mrot = if mir { -rot } else { rot };
        let (px, py) = (x, T - y);
        let sxs = if mir { -1.0 } else { 1.0 };
        let sys = if gf { -1.0 } else { 1.0 };
        let player = self.cfg_player.clone();
        if ship || ufo {
            let off = 10.0;
            let (cs, sn) = (rot.cos(), rot.sin());
            let mm = if mir { -1.0 } else { 1.0 };
            let (ship_x, ship_y) = (-off * sn * mm, off * cs);
            let player_offset = if gf { -30.0 * mini_f } else { 10.0 * mini_f };
            let (cube_dx, cube_dy) = (player_offset * sn * mm, -player_offset * cs);
            let ufo_mode = ufo && !ship;
            if ship { for (k, n, d) in Self::layers(&self.cfg_ship.clone(), false) { if k != "glow" { self.push(assets, k, &n, d, px + ship_x, py + ship_y + if gf { -20.0 * mini_f } else { 0.0 }, mrot, sxs * mini_f, sys * mini_f, inverted); } } }
            let (mut bird_x, mut bird_y, mut cube_x, mut cube_y, mut cube_rot, mut bird_rot) = (0.0, 0.0, 0.0, 0.0, mrot, mrot);
            if ufo { bird_x = px + ship_x; bird_y = py + ship_y + if gf { -15.0 } else { 5.0 }; }
            if ship { let side = if gf { -1.0 } else { 1.0 }; cube_x = px + ship_x + side * 20.0 * mini_f * sn * mm; cube_y = py + ship_y + if gf { -20.0 * mini_f } else { 0.0 } - side * 20.0 * mini_f * cs; }
            else { cube_x = px + cube_dx; cube_y = py + cube_dy + if p[4] != 0.0 { 8.0 * mini_f } else { 0.0 } + if gf { -20.0 * mini_f } else { 0.0 }; }
            if ufo_mode {
                let last = self.ufo_last.get_or_insert((x, y));
                let (dx, dy) = (x - last.0, y - last.1);
                if dx * dx + dy * dy >= 4.8 {
                    let mut aim = 0.4 * dy.atan2(dx.abs());
                    aim = if gf { aim.min(0.1) } else { aim.max(-0.1) };
                    self.ufo_tilt = aim; *last = (x, y);
                }
                let target = if mir { -self.ufo_tilt } else { self.ufo_tilt };
                if !self.ufo_rot.is_finite() { self.ufo_rot = target; }
                if p[20].is_finite() {
                    let st = if mir { -p[20] } else { p[20] };
                    let sp = p[21];
                    let gd = if (sp - 9.30222544655).abs() < 1e-9 { 0.7 } else if (sp - 14.3488938625).abs() < 1e-9 { 1.1 } else if (sp - 17.3333393414).abs() < 1e-9 { 1.3 } else if (sp - 21.3333407279).abs() < 1e-9 { 1.6 } else { 0.9 };
                    let mix = (gd * 0.175 * dt * 60.0).min(1.0);
                    self.ufo_rot += (st - self.ufo_rot) * mix;
                } else {
                    let mix = 1.0 - 0.85f64.powf((dt * 60.0).max(0.0));
                    self.ufo_rot += (target - self.ufo_rot) * mix;
                    self.ufo_rot = self.ufo_rot.clamp(-0.11, 3.0);
                }
                let ur = self.ufo_rot;
                let delta = ur - mrot;
                let (c, s) = (delta.cos(), delta.sin());
                let (ox, oy) = (cube_x - bird_x, cube_y - bird_y);
                cube_x = bird_x + ox * c - oy * s; cube_y = bird_y + ox * s + oy * c; cube_rot = ur; bird_rot = ur;
            } else if !ufo_mode { self.ufo_last = None; }
            if ufo { for (k, n, d) in Self::layers(&self.cfg_bird.clone(), true) { if k != "glow" || true { self.push(assets, k, &n, d, bird_x, bird_y, bird_rot, sxs * mini_f, sys * mini_f, inverted); } } }
            let cm = mini_f * 0.55;
            for (k, n, d) in Self::layers(&player, false) { if k != "glow" { self.push(assets, k, &n, d, cube_x, cube_y, cube_rot, sxs * cm, sys * cm, inverted); } }
            return;
        }
        if ball { for (k, n, d) in Self::layers(&self.cfg_ball.clone(), false) { if k != "glow" { self.push(assets, k, &n, d, px, py, rot, sxs * mini_f, mini_f, inverted); } } return; }
        if wave {
            let m = mini_f * 0.94;
            for (k, n, d) in Self::layers(&self.cfg_wave.clone(), false) {
                if k == "glow" { continue; }
                let off = if k == "base" { (1.5 * if mir { 1.0 } else { -1.0 }, -1.0) } else { (0.0, 0.0) };
                self.push(assets, k, &n, d, px + off.0, py + off.1, mrot, sxs * m, sys * m, inverted);
            }
            return;
        }
        // cube (the robot and spider are drawn as the cube until their animations are ported)
        for (k, n, d) in Self::layers(&player, false) { if k != "glow" { self.push(assets, k, &n, d, px, py, mrot, sxs * mini_f, sys * mini_f, inverted); } }
    }

    fn wave_update(&mut self, dt: f64, cam_x: f64, pos: (f64, f64), mini: bool) {
        let decay = (dt / 1000.0) / 0.6;
        for p in self.wave.iter_mut() { p.age += decay; }
        self.wave.retain(|p| p.age < 1.0);
        let cutoff = cam_x - 200.0;
        let mut drop = 0;
        while self.wave.len() - drop.min(self.wave.len()) > 2 && self.wave.get(drop + 1).map_or(false, |p| p.x < cutoff) { drop += 1; }
        if drop > 0 { self.wave.drain(0..drop); }
        if self.wave_active {
            let add = match self.wave.last() { Some(l) => (pos.0 - l.x).powi(2) + (pos.1 - l.y).powi(2) >= 2.25, None => true };
            if add { self.wave.push(Pt { x: pos.0, y: pos.1, age: 0.0 }); }
        }
        self.wave_mini = if mini { 0.6 } else { 1.0 };
    }

    fn kill(&mut self, a: &[f64]) {
        let (sx, sy) = (a[1] + self.ox, T - a[0] + self.oy);
        let mut parts = Vec::new();
        for _ in 0..100 {
            let ang = self.rnd(0.0, 360.0).to_radians();
            let sp = self.rnd(200.0, 800.0);
            let (jx, jy) = (self.rnd(-20.0, 20.0), self.rnd(-20.0, 20.0));
            let life = self.rnd(50.0, 800.0);
            parts.push((sx + jx, sy + jy, ang.cos() * sp, ang.sin() * sp, 0.0, life));
        }
        self.burst = Some(Burst { parts, tint: self.main, flash: 0.0, fx: sx, fy: sy });
        self.wave_active = false; self.wave.clear();
    }

    /// The icon layers, in world space (drawn after the level's normal layer).
    pub fn draw_world(&self, c: &mut Canvas, assets: &Assets) {
        if self.hidden { return; }
        let mut items: Vec<&Item> = self.items.iter().collect();
        items.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.order.cmp(&b.order)));
        for it in items {
            let Some((img, f)) = self.find(assets, &it.name) else { continue };
            let sxp = it.x + self.ox;
            let mxp = if self.mirror_p != 0.0 { sxp + (W as f64 - 2.0 * sxp) * self.mirror_p } else { sxp };
            c.frame(img, &f, Sprite { x: mxp, y: it.y + self.oy, rot: it.rot, sx: it.sx, sy: it.sy, alpha: 1.0, tint: it.tint, additive: it.additive, flip_x: false, flip_y: false });
        }
    }

    /// The wave trail (drawn after the top layer).
    pub fn draw_top(&self, c: &mut Canvas) {
        if self.wave.len() < 2 || self.hidden { return; }
        let m = self.wave_mini;
        let (hw, ghw) = (7.0 * m, 14.0 * m);
        let col = self.main as u32;
        self.ribbon(c, ghw, col, 0.22, true);
        self.ribbon(c, hw, col, 0.95, false);
        self.ribbon(c, (hw * 0.32).round(), 0xffffff, 0.5, false);
    }

    fn ribbon(&self, c: &mut Canvas, half: f64, color: u32, base_alpha: f64, additive: bool) {
        let pts = &self.wave;
        let n = pts.len();
        let (ox, oy) = (self.ox, self.oy);
        let mut nx = vec![0.0; n]; let mut ny = vec![0.0; n];
        for i in 0..n - 1 { let (dx, dy) = (pts[i + 1].x - pts[i].x, pts[i + 1].y - pts[i].y); let len = (dx * dx + dy * dy).sqrt().max(1e-9); nx[i] = -dy / len; ny[i] = dx / len; }
        let isect = |p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), p4: (f64, f64)| -> (f64, f64) {
            let (d1x, d1y, d2x, d2y) = (p2.0 - p1.0, p2.1 - p1.1, p4.0 - p3.0, p4.1 - p3.1);
            let den = d1x * d2y - d1y * d2x;
            if den.abs() < 1e-6 { return p2; }
            let t = (((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / den).clamp(-3.0, 3.0);
            (p1.0 + d1x * t, p1.1 + d1y * t)
        };
        let mut up = vec![(0.0, 0.0); n]; let mut lo = vec![(0.0, 0.0); n];
        for i in 0..n {
            let p = (pts[i].x, pts[i].y);
            if i == 0 || i == n - 1 { let k = if i == 0 { 0 } else { n - 2 }; up[i] = (p.0 + nx[k] * half, p.1 + ny[k] * half); lo[i] = (p.0 - nx[k] * half, p.1 - ny[k] * half); }
            else {
                let (n1, n2) = ((nx[i - 1], ny[i - 1]), (nx[i], ny[i]));
                let (pr, nt) = ((pts[i - 1].x, pts[i - 1].y), (pts[i + 1].x, pts[i + 1].y));
                up[i] = isect((pr.0 + n1.0 * half, pr.1 + n1.1 * half), (p.0 + n1.0 * half, p.1 + n1.1 * half), (p.0 + n2.0 * half, p.1 + n2.1 * half), (nt.0 + n2.0 * half, nt.1 + n2.1 * half));
                lo[i] = isect((pr.0 - n1.0 * half, pr.1 - n1.1 * half), (p.0 - n1.0 * half, p.1 - n1.1 * half), (p.0 - n2.0 * half, p.1 - n2.1 * half), (nt.0 - n2.0 * half, nt.1 - n2.1 * half));
            }
        }
        let s = |q: (f64, f64)| (q.0 + ox, q.1 + oy);
        let steps = 32.0;
        let mut cur = -1.0; let mut group: Vec<[f64; 6]> = Vec::new();
        for i in 0..n - 1 {
            let a = (1.0 - (pts[i].age + pts[i + 1].age) * 0.5).max(0.0);
            if a * base_alpha <= 0.01 { continue; }
            let q = (a * steps).round().max(1.0);
            if q != cur { c.fill_tris(&group, color, cur / steps * base_alpha, additive); group.clear(); cur = q; }
            let (u0, u1, l0, l1) = (s(up[i]), s(up[i + 1]), s(lo[i]), s(lo[i + 1]));
            group.push([u0.0, u0.1, u1.0, u1.1, l0.0, l0.1]);
            group.push([u1.0, u1.1, l1.0, l1.1, l0.0, l0.1]);
        }
        c.fill_tris(&group, color, cur.max(0.0) / steps * base_alpha, additive);
    }

    /// Death burst and flash (screen space, drawn last).
    pub fn draw_screen(&self, c: &mut Canvas, assets: &Assets) {
        let Some(b) = &self.burst else { return };
        if let Some((img, f)) = self.find(assets, "square.png") {
            for p in &b.parts {
                let t = p.4 / p.5;
                c.frame(img, &f, Sprite { x: p.0, y: p.1, rot: 0.0, sx: 18.0 / 32.0 * (1.0 - t), sy: 18.0 / 32.0 * (1.0 - t), alpha: 1.0 - t, tint: b.tint, additive: true, flip_x: false, flip_y: false });
            }
        }
        if b.flash < 1.0 {
            let t = b.flash;
            c.fill_circle(b.fx, b.fy, 14.0 + t * 76.0, b.tint as u32, (1.0 - t * t).max(0.0), crate::canvas::Mat::id());
        }
    }
}
