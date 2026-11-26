//! The player's picture (port of the drawing side of `player.js` as `web/player_vis.js` has it): the icon layers for every mode,
//! the wave trail and the death burst. State comes from `wd_pv_state`.
use crate::assets::{load_png, Assets, Frame, Image};
use crate::canvas::{Canvas, Sprite, W};
use crate::plist;
use std::collections::HashMap;
use wd_core as core;

const T: f64 = 460.0;

struct AnimPart { tag: i64, texture: String, pos: (f64, f64), scale: (f64, f64), flip: (bool, bool), z: f64, rot: f64 }
struct Anim { frames: HashMap<String, Vec<AnimPart>>, groups: HashMap<&'static str, Vec<String>>, tags: std::collections::HashSet<i64>, textures: HashMap<i64, String> }

fn pair(v: Option<&wd_core::jsonread::Json>, fx: f64, fy: f64) -> (f64, f64) {
    let s = v.and_then(|v| v.str()).unwrap_or("");
    let n: Vec<f64> = s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).filter(|t| !t.is_empty()).filter_map(|t| t.parse().ok()).collect();
    if n.len() >= 2 { (n[0], n[1]) } else { (fx, fy) }
}
fn suffix_of(s: &str) -> i64 { s.trim_end_matches(".png").rsplit('_').next().and_then(|d| d.parse().ok()).unwrap_or(0) }

fn load_anim(root: &std::path::Path, file: &str, prefix: &str, names: &[(&'static str, &str)], fallback: &[&str]) -> Option<Anim> {
    let src = std::fs::read_to_string(root.join("sheets").join(file)).ok()?;
    let j = wd_core::jsonread::parse(&src).ok()?;
    let wd_core::jsonread::Json::Obj(cont) = j.get("animationContainer")? else { return None };
    let mut frames = HashMap::new();
    for (key, fr) in cont {
        let mut parts = Vec::new();
        if let wd_core::jsonread::Json::Obj(items) = fr {
            for (k, sd) in items {
                if !k.starts_with("sprite_") { continue; }
                let tag = sd.get_str("tag").and_then(|t| t.parse().ok()).unwrap_or(0);
                let (fx, fy) = pair(sd.get("flipped"), 0.0, 0.0);
                parts.push(AnimPart { tag, texture: sd.get_str("texture").unwrap_or("").to_string(), pos: pair(sd.get("position"), 0.0, 0.0), scale: pair(sd.get("scale"), 1.0, 1.0), flip: (fx != 0.0, fy != 0.0),
                    z: sd.get_str("zValue").and_then(|t| t.parse().ok()).unwrap_or(tag as f64), rot: sd.get_str("rotation").and_then(|t| t.parse().ok()).unwrap_or(0.0) });
            }
        }
        frames.insert(key.clone(), parts);
    }
    let mut entries: Vec<(i64, String)> = Vec::new();
    if let Some(wd_core::jsonread::Json::Obj(u)) = j.get("usedTextures") { for (_, e) in u { entries.push((e.get_str("tag").and_then(|t| t.parse().ok()).unwrap_or(0), e.get_str("texture").unwrap_or("").to_string())); } }
    entries.sort_by_key(|e| e.0);
    if entries.is_empty() { entries = fallback.iter().enumerate().map(|(i, t)| (i as i64, t.to_string())).collect(); }
    let tags = entries.iter().map(|e| e.0).collect();
    let textures = entries.into_iter().collect();
    let mut groups = HashMap::new();
    for (name, re) in names {
        let pre = format!("{prefix}{re}_");
        let mut keys: Vec<String> = frames.keys().filter(|k| k.starts_with(&pre) && k.ends_with(".png") && k[pre.len()..k.len() - 4].chars().all(|c| c.is_ascii_digit()) && k.len() > pre.len() + 4).cloned().collect();
        keys.sort_by_key(|k| suffix_of(k));
        groups.insert(*name, keys);
    }
    Some(Anim { frames, groups, tags, textures })
}

pub struct IconSheet { pub img: Image, pub frames: HashMap<String, Frame> }

#[derive(Clone)]
struct Item { kind: &'static str, name: String, depth: f64, order: usize, x: f64, y: f64, rot: f64, sx: f64, sy: f64, tint: i64, additive: bool }

struct Pt { x: f64, y: f64, age: f64 }

#[derive(Clone, Copy)]
enum Tint { None, Fixed(i64), Grad(i64, i64) }
struct Particle { x: f64, y: f64, vx: f64, vy: f64, age: f64, life: f64 }
struct Emitter {
    speed: (f64, f64), angle: (f64, f64), life: (f64, f64), scale: (f64, f64), alpha: (f64, f64), gx: f64, gy: f64, freq: f64,
    spread_x: Option<(f64, f64)>, tint: Tint, parts: Vec<Particle>, acc: f64, on: bool, px: f64, py: f64,
}
impl Emitter {
    fn new(speed: (f64, f64), angle: (f64, f64), life: (f64, f64), scale: (f64, f64), gx: f64, gy: f64, freq: f64, spread_x: Option<(f64, f64)>, tint: Tint) -> Emitter {
        Emitter { speed, angle, life, scale, alpha: (1.0, 0.0), gx, gy, freq, spread_x, tint, parts: Vec::new(), acc: 0.0, on: false, px: 0.0, py: 0.0 }
    }
    fn set(&mut self, on: bool) { if on && !self.on { self.acc = 0.0; } self.on = on; }
}
fn mix(a: i64, b: i64, t: f64) -> i64 {
    let t = t.clamp(0.0, 1.0);
    let ch = |s: i64| ((a >> s) & 255) as f64 + ((((b >> s) & 255) - ((a >> s) & 255)) as f64) * t;
    ((ch(16).round() as i64) << 16) | ((ch(8).round() as i64) << 8) | ch(0).round() as i64
}
struct StreakPt { x: f64, y: f64, state: f64 }

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
    robot: Option<Anim>, spider: Option<Anim>, anims_loaded: bool,
    spider_timer: f64, robot_timer: f64, robot_state: u8,
    cfg_spider: String, cfg_robot: String,
    em: Vec<Emitter>, // 0 ground, 1 fly, 2 fly2, 3 drag, 4 land1, 5 land2
    land_idx: bool,
    streak: Vec<StreakPt>, streak_active: bool, streak_pos: Option<(f64, f64)>,
    ldm: bool,
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
            hidden: false, ufo_last: None, ufo_tilt: 0.0, ufo_rot: f64::NAN, mirror_p: 0.0, ox: 0.0, oy: 0.0, robot: None, spider: None, anims_loaded: false, spider_timer: 0.0, robot_timer: 0.0, robot_state: 0, cfg_spider: "spider_01".into(), cfg_robot: "robot_01".into(),
            em: vec![
                Emitter::new((110.0, 190.0), (225.0, 315.0), (150.0, 450.0), (0.5, 0.0), 0.0, 600.0, 1000.0 / 30.0, None, Tint::None),
                Emitter::new((22.0, 38.0), (225.0, 315.0), (150.0, 450.0), (0.5, 0.0), 0.0, 600.0, 1000.0 / 30.0, None, Tint::Grad(16737280, 16711680)),
                Emitter::new((220.0, 380.0), (180.0, 360.0), (150.0, 450.0), (0.75, 0.0), 0.0, 600.0, 1000.0 / 30.0, None, Tint::Grad(16760320, 16711680)),
                Emitter::new((223.8, 343.8), (205.0, 295.0), (80.0, 220.0), (0.375, 0.0), -700.0, 600.0, 25.0, Some((-18.0, 18.0)), Tint::None),
                Emitter::new((250.0, 350.0), (210.0, 330.0), (50.0, 600.0), (0.625, 0.0), 0.0, 1000.0, 0.0, None, Tint::None),
                Emitter::new((250.0, 350.0), (210.0, 330.0), (50.0, 600.0), (0.625, 0.0), 0.0, 1000.0, 0.0, None, Tint::None),
            ], land_idx: false, streak: Vec::new(), streak_active: false, streak_pos: None, ldm: false }
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
    pub fn set_cfg(&mut self, main: i64, second: i64, player: &str, ship: &str, ball: &str, wave: &str, bird: &str, spider: &str, robot: &str) {
        self.cfg_spider = spider.into(); self.cfg_robot = robot.into();
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

    fn push(&mut self, assets: &Assets, kind: &'static str, name: &str, depth: f64, x: f64, y: f64, rot: f64, sx: f64, sy: f64, inverted: bool) {
        if self.find(assets, name).is_none() { return; }
        let tint = self.tint_of(kind, inverted);
        let order = self.items.len();
        self.items.push(Item { kind, name: name.to_string(), depth, order, x, y, rot, sx, sy, tint, additive: false });
    }

    fn layers(prefix: &str, bird: bool) -> Vec<(&'static str, String, f64)> {
        if bird { vec![("glow", format!("{prefix}_2_001.png"), 9.0), ("base", format!("{prefix}_001.png"), 10.0), ("overlay", format!("{prefix}_3_001.png"), 8.0), ("extra", format!("{prefix}_extra_001.png"), 12.0)] }
        else { vec![("glow", variant(prefix, "glow"), 9.0), ("base", variant(prefix, "base"), 10.0), ("overlay", variant(prefix, "overlay"), 8.0), ("extra", variant(prefix, "extra"), 12.0)] }
    }

    /// Reads the player state and rebuilds the layers; `dt` in seconds.
    pub fn frame(&mut self, assets: &Assets, dt: f64, cam: (f64, f64), menu: bool, hidden: bool, restart: bool, mirror_p: f64, paused: bool, ldm: bool, events: &str) {
        self.ldm = ldm;
        self.hidden = hidden;
        self.mirror_p = mirror_p;
        self.ox = -cam.0; self.oy = cam.1;
        let root = assets.root.clone();
        if !self.anims_loaded {
            self.anims_loaded = true;
            self.robot = load_anim(&root, "Robot_AnimDesc.json", "Robot_", &[("run", "run"), ("run2", "run2"), ("run3", "run3"), ("skip", "skip"), ("jump_start", "jump_start"), ("jump_loop", "jump_loop"), ("fall_start", "fall_start"), ("fall_loop", "fall_loop"), ("idle", "idle"), ("idle1", "idle01"), ("idle2", "idle02")], &["robot_01_03_001.png", "robot_01_02_001.png", "robot_01_04_001.png", "robot_01_01_001.png", "robot_01_03_001.png", "robot_01_02_001.png", "robot_01_04_001.png"]);
            self.spider = load_anim(&root, "Spider_AnimDesc.json", "Spider_", &[("run", "run"), ("run2", "run2"), ("walk", "walk"), ("fall", "fall_loop"), ("idle", "idle01"), ("idle2", "idle02")], &["spider_01_02_001.png", "spider_01_02_001.png", "spider_01_04_001.png", "spider_01_01_001.png", "spider_01_03_001.png", "spider_01_02_001.png"]);
        }
        for n in [self.cfg_player.clone(), self.cfg_ship.clone(), self.cfg_ball.clone(), self.cfg_wave.clone(), self.cfg_bird.clone(), self.cfg_spider.clone(), self.cfg_robot.clone()] {
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
        self.emitters_frame(&a, dt, cam, menu, dead, paused, restart, events);
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
        if p[9] != 0.0 { self.part_layout(assets, "spider", p, px, py, dt, inverted); return; }
        if p[10] != 0.0 { self.part_layout(assets, "robot", p, px, py, dt, inverted); return; }
        for (k, n, d) in Self::layers(&player, false) { if k != "glow" { self.push(assets, k, &n, d, px, py, mrot, sxs * mini_f, sys * mini_f, inverted); } }
    }

    fn speed_factor(sp: f64) -> f64 {
        let (one, half, four) = (11.540004, 9.30222544655, 21.3333407279);
        let sp = if sp.is_finite() { sp } else { one };
        if sp >= one { 1.75 + ((sp - one) / (four - one).max(1e-6)).clamp(0.0, 1.0) * 0.35 } else { 1.75 - ((one - sp) / (one - half).max(1e-6)).clamp(0.0, 1.0) * 0.15 }
    }

    /// Robot and spider: the animation frame by state and time, each part placed by the AnimDesc.
    fn part_layout(&mut self, assets: &Assets, kind: &'static str, p: &[f64], x: f64, y: f64, dt: f64, inverted: bool) {
        let (on_ground, on_ceiling, gf) = (p[13] != 0.0, p[14] != 0.0, p[11] != 0.0);
        let key: Option<String>;
        {
            let a = if kind == "spider" { self.spider.as_ref() } else { self.robot.as_ref() };
            let Some(a) = a else { return };
            let get = |n: &str| a.groups.get(n).cloned().unwrap_or_default();
            if kind == "spider" {
                self.spider_timer += dt.max(0.0) * Self::speed_factor(p[21]);
                let mut group = if !on_ground && !on_ceiling { get("fall") } else { get("run") };
                if group.is_empty() { group = get("walk"); }
                if group.is_empty() { group = get("idle"); }
                let fps = if on_ground || on_ceiling { 16.0 } else { 12.0 };
                key = if !group.is_empty() { Some(group[((self.spider_timer * fps).floor() as usize) % group.len()].clone()) } else if a.frames.contains_key("Spider_idle_001.png") { Some("Spider_idle_001.png".into()) } else { None };
            } else {
                let speed = Self::speed_factor(p[21]) * 1.2;
                let grounded = on_ground || on_ceiling;
                let up = if gf { p[17] < 0.0 } else { p[17] > 0.0 };
                let next: u8 = if grounded { 0 } else if up { 1 } else { 2 };
                if self.robot_state != next { self.robot_state = next; self.robot_timer = 0.0; }
                self.robot_timer += dt.max(0.0) * speed;
                let t = self.robot_timer;
                let start_loop = |start: Vec<String>, lp: Vec<String>, fps: f64| -> Option<String> {
                    if !start.is_empty() {
                        let sd = start.len() as f64 / fps;
                        if t < sd || lp.is_empty() { return Some(start[((start.len() - 1) as f64).min((t * fps).floor()) as usize].clone()); }
                        return Some(lp[(((t - sd) * fps).floor() as usize) % lp.len()].clone());
                    }
                    if lp.is_empty() { None } else { Some(lp[((t * fps).floor() as usize) % lp.len()].clone()) }
                };
                let k = if p[15] != 0.0 { let g = get("fall_loop"); if g.is_empty() { None } else { Some(g[((t * 16.0).floor() as usize) % g.len()].clone()) } }
                    else if next == 0 { let g = if !get("run").is_empty() { get("run") } else if !get("idle1").is_empty() { get("idle1") } else { get("idle") }; if g.is_empty() { None } else { Some(g[((t * 16.0).floor() as usize) % g.len()].clone()) } }
                    else if next == 1 { start_loop(get("jump_start"), get("jump_loop"), 15.0) } else { start_loop(get("fall_start"), get("fall_loop"), 15.0) };
                key = k.or_else(|| if a.frames.contains_key("Robot_idle_001.png") { Some("Robot_idle_001.png".into()) } else { None });
            }
        }
        let Some(key) = key else { return };
        let (mini, mir_sign, grav_sign, pos_y_sign) = (if p[4] != 0.0 { 0.6 } else { 1.0 }, if p[12] != 0.0 { -1.0 } else { 1.0 }, if gf { -1.0 } else { 1.0 }, if gf { 1.0 } else { -1.0 });
        let base_name = if kind == "spider" { self.cfg_spider.clone() } else { self.cfg_robot.clone() };
        let resolve = |n: &str| -> String {
            // `^spider_\d+` -> the equipped icon
            let pre = format!("{kind}_");
            if let Some(rest) = n.strip_prefix(&pre) { let d = rest.chars().take_while(|c| c.is_ascii_digit()).count(); if d > 0 { return format!("{base_name}{}", &rest[d..]); } }
            n.to_string()
        };
        let parts: Vec<(i64, String, (f64, f64), (f64, f64), (bool, bool), f64, f64)> = {
            let a = if kind == "spider" { self.spider.as_ref() } else { self.robot.as_ref() };
            let Some(a) = a else { return };
            let Some(frame) = a.frames.get(&key) else { return };
            frame.iter().filter(|pt| a.tags.contains(&pt.tag)).map(|pt| (pt.tag, if pt.texture.is_empty() { a.textures.get(&pt.tag).cloned().unwrap_or_default() } else { pt.texture.clone() }, pt.pos, pt.scale, pt.flip, pt.z, pt.rot)).collect()
        };
        for (tag, texture, pos, sc, fl, z, rot_deg) in parts {
            let (local_y, local_xs) = if kind == "spider" {
                let leg = [0, 1, 4, 5].contains(&tag);
                (if tag == 3 { 5.0 } else if leg { -9.0 } else { 0.0 }, if leg { 1.8 } else { 1.0 })
            } else {
                let leg = [0, 2, 4, 6].contains(&tag); let arm = [1, 5].contains(&tag);
                (if tag == 3 { 5.0 } else if arm { -1.0 } else if leg { -9.0 } else { 0.0 }, if leg || arm { 1.8 } else { 1.0 })
            };
            let cx = x + pos.0 * local_xs * mir_sign * mini;
            let cy = y + (pos.1 + local_y) * pos_y_sign * mini;
            let mut cr = rot_deg.to_radians();
            if p[12] != 0.0 { cr = -cr; }
            if gf { cr = -cr; }
            let bsx = sc.0 * if fl.0 { -1.0 } else { 1.0 } * mir_sign * mini;
            let bsy = sc.1 * if fl.1 { -1.0 } else { 1.0 } * grav_sign * mini;
            let tex = resolve(&texture);
            for (vk, off) in [("base", 0.0), ("overlay", -0.02), ("extra", 0.08)] {
                let fnm = resolve(&match vk { "overlay" => tex.replace("_001.png", "_2_001.png"), "extra" => tex.replace("_001.png", "_extra_001.png"), _ => tex.clone() });
                self.push(assets, vk, &fnm, 8.0 + z * 0.1 + off, cx, cy, cr, bsx, bsy, inverted);
            }
        }
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

    fn emit(e: &mut Emitter, n: usize, x: f64, y: f64, rng: &mut dyn FnMut(f64, f64) -> f64) {
        for _ in 0..n {
            let a = rng(e.angle.0, e.angle.1).to_radians(); let sp = rng(e.speed.0, e.speed.1);
            let life = rng(e.life.0, e.life.1);
            let ox = e.spread_x.map_or(0.0, |s| rng(s.0, s.1));
            e.parts.push(Particle { x: x + ox, y, vx: a.cos() * sp, vy: a.sin() * sp, age: 0.0, life });
        }
    }

    /// Particle emitters, the motion streak and the landing dust (`frame` of player_vis.js).
    fn emitters_frame(&mut self, a: &[f64], dt: f64, cam: (f64, f64), menu: bool, dead: bool, paused: bool, restart: bool, events: &str) {
        let (x, y, rot) = (a[1], a[0], a[2]);
        let (wx, wy) = (x, T - y);
        let (gf, mir, ufo, wave, ship) = (a[11] != 0.0, a[12] != 0.0, a[8] != 0.0, a[6] != 0.0, a[5] != 0.0);
        let live = !self.ldm && !menu && !dead && !self.hidden;
        if restart { for e in self.em.iter_mut() { e.parts.clear(); } self.streak.clear(); }
        let mut rng_state = self.rng;
        let mut rng = |lo: f64, hi: f64| -> f64 { rng_state ^= rng_state << 13; rng_state ^= rng_state >> 7; rng_state ^= rng_state << 17; lo + (rng_state >> 11) as f64 / (1u64 << 53) as f64 * (hi - lo) };
        if !live {
            for i in 0..4 { self.em[i].set(false); }
            self.streak_active = false;
            if self.ldm { self.streak.clear(); self.wave.clear(); }
        } else {
            let (cs, sn) = (rot.cos(), rot.sin());
            self.em[0].px = wx - 20.0; self.em[0].py = wy + if gf { -26.0 + if ufo { -5.0 } else { 0.0 } } else { 26.0 + if ufo { 5.0 } else { 0.0 } };
            let on_g = a[13] != 0.0 && !ship && !wave;
            self.em[0].set(on_g);
            let dxo = if wave { 0.0 } else if ufo { 0.0 } else { -24.0 };
            let dyo = (if wave { 4.0 } else if ufo { 5.0 } else { 18.0 }) * if gf { -1.0 } else { 1.0 };
            let (fx, fy) = (wx + dxo * cs - dyo * sn, wy + dxo * sn + dyo * cs);
            let jit = rng(-1.0, 1.0) * 4.0;
            for i in [1, 2] { self.em[i].px = fx; self.em[i].py = fy + jit; }
            self.streak_pos = Some((if wave { fx } else if ufo { fx } else { fx + 8.0 }, fy));
            let trail_mode = ship || ufo || wave;
            if trail_mode && !self.streak_active { self.streak.clear(); self.streak_active = true; } else if !trail_mode && self.streak_active { self.streak_active = false; self.streak.clear(); }
            self.streak_update(dt);
            let fly_on = ship || ufo;
            self.em[1].set(fly_on);
            let fly2 = (ship && a[19] != 0.0) || (ufo && a[18] != 0.0);
            self.em[2].set(fly2);
            self.em[3].px = wx; self.em[3].py = if gf { T - y + cam.1 + 10.0 } else { T - y + cam.1 + 30.0 };
            self.em[3].angle = if mir { (245.0, 335.0) } else { (205.0, 295.0) };
            self.em[3].gx = if mir { 700.0 } else { -700.0 };
            let drag = ship && a[13] != 0.0 && (if gf { a[14] != 0.0 } else { a[14] == 0.0 });
            self.em[3].set(drag);
        }
        if !paused {
            let ms = dt * 1000.0;
            for e in self.em.iter_mut() {
                if e.on && e.freq > 0.0 { e.acc += ms; while e.acc >= e.freq { e.acc -= e.freq; let (px, py) = (e.px, e.py); Self::emit(e, 1, px, py, &mut rng); } }
                for p in e.parts.iter_mut() { p.age += ms; p.vx += e.gx * dt; p.vy += e.gy * dt; p.x += p.vx * dt; p.y += p.vy * dt; }
                e.parts.retain(|p| p.age < p.life);
            }
        }
        // landing dust
        for ev in events.split('|') {
            let mut it = ev.splitn(2, ':');
            if it.next() == Some("land") && !self.ldm && !menu {
                let flipped = it.next().and_then(|s| s.split(',').next()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) != 0.0;
                self.land_idx = !self.land_idx;
                let i = if self.land_idx { 4 } else { 5 };
                let (lx, ly) = (x, if flipped { T - y - 30.0 } else { T - y + 30.0 });
                Self::emit(&mut self.em[i], 10, lx, ly, &mut rng);
            }
        }
        self.rng = rng_state;
        let main = self.main;
        let _ = main;
    }

    fn streak_update(&mut self, dt: f64) {
        let fade = 1.0 / 0.231;
        let max_pts = ((0.231 * 60.0 + 2.0).floor() * 5.0) as usize;
        let Some(pos) = self.streak_pos else { return };
        let d = dt * fade;
        for p in self.streak.iter_mut() { p.state -= d; }
        self.streak.retain(|p| p.state > 0.0);
        if self.streak_active && self.streak.len() < max_pts {
            let n = self.streak.len(); let mut add = true;
            if n > 0 {
                let last = &self.streak[n - 1];
                let (dx, dy) = (pos.0 - last.x, pos.1 - last.y); let sq = dx * dx + dy * dy;
                if sq.sqrt() > 100.0 { self.streak.clear(); }
                else if sq < 100.0 { add = false; }
                else if n > 1 { let l2 = &self.streak[n - 2]; let (ex, ey) = (pos.0 - l2.x, pos.1 - l2.y); if ex * ex + ey * ey < 200.0 { add = false; } }
            }
            if add { self.streak.push(StreakPt { x: pos.0, y: pos.1, state: 1.0 }); }
        }
    }

    fn draw_particles(&self, c: &mut Canvas, assets: &Assets, which: &[usize]) {
        let Some((img, f)) = self.find(assets, "square.png") else { return };
        for &i in which {
            let e = &self.em[i];
            for p in &e.parts {
                let t = p.age / p.life;
                let tint = match e.tint { Tint::None => if i == 0 || i >= 4 { self.main } else { -1 }, Tint::Fixed(c) => c, Tint::Grad(a, b) => mix(a, b, (t * 24.0).round() / 24.0) };
                let sc = e.scale.0 + (e.scale.1 - e.scale.0) * t;
                c.frame(img, &f, Sprite { x: p.x + self.ox, y: p.y + self.oy, rot: 0.0, sx: sc, sy: sc, alpha: e.alpha.0 + (e.alpha.1 - e.alpha.0) * t, tint, additive: true, flip_x: false, flip_y: false });
            }
        }
    }

    fn draw_streak(&self, c: &mut Canvas) {
        if self.streak.len() < 2 { return; }
        let col = self.second as u32;
        for i in 0..self.streak.len() - 1 {
            let (a, b) = (&self.streak[i], &self.streak[i + 1]);
            let (dx, dy) = (b.x - a.x, b.y - a.y); let len = (dx * dx + dy * dy).sqrt().max(1e-9);
            let (nx, ny) = (-dy / len * 4.0, dx / len * 4.0);
            let p = |x: f64, y: f64| (x + self.ox, y + self.oy);
            let (a1, a2, b1, b2) = (p(a.x + nx, a.y + ny), p(a.x - nx, a.y - ny), p(b.x + nx, b.y + ny), p(b.x - nx, b.y - ny));
            c.fill_tris(&[[a1.0, a1.1, b1.0, b1.1, a2.0, a2.1], [b1.0, b1.1, b2.0, b2.1, a2.0, a2.1]], col, ((a.state + b.state) * 0.5 * 0.7).clamp(0.0, 1.0), true);
        }
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
        self.draw_streak(c);
        self.draw_particles(c, assets, &[0, 1, 2, 3]);
        let mut items: Vec<&Item> = self.items.iter().collect();
        items.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal).then(a.order.cmp(&b.order)));
        for it in items {
            let Some((img, f)) = self.find(assets, &it.name) else { continue };
            let sxp = it.x + self.ox;
            let mxp = if self.mirror_p != 0.0 { sxp + (W as f64 - 2.0 * sxp) * self.mirror_p } else { sxp };
            c.frame(img, &f, Sprite { x: mxp, y: it.y + self.oy, rot: it.rot, sx: it.sx, sy: it.sy, alpha: 1.0, tint: it.tint, additive: it.additive, flip_x: false, flip_y: false });
        }
    }

    /// The wave trail (drawn after the top layer).
    pub fn draw_top(&self, c: &mut Canvas, assets: &Assets) {
        self.draw_particles(c, assets, &[4, 5]);
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
