//! BeastAnim objects (animations.js): the animated monsters (ids 918, 1327, 1328, 1584, 2012). A beast is a set of part sprites
//! whose positions, rotation and scale follow the frames of its `*_AnimDesc.json`; the state machine picks which animation plays.
//! The descriptions come from the host (`wd_beast_desc_add`); without one the object keeps its static sprites.

use crate::game::Game;
use crate::jsmath as m;
use crate::jsonread::{parse, Json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub const IDS: [i64; 5] = [918, 1327, 1328, 1584, 2012];
const START_DELAY_MAX: f64 = 0.4;
const UNIT_SCALE: f64 = 2.0;
const DELAY_30FPS: f64 = 1.0 / 30.0;

pub fn anim_name(id: i64) -> Option<&'static str> {
    Some(match id { 918 => "GJBeast01", 1327 => "GJBeast02", 1328 => "GJBeast03", 1584 => "GJBeast04", 2012 => "GJBeast05", _ => return None })
}
/// (delay, frames, looped) of `BEAST_ANIM_DEFS[name].animations[base]`
fn anim_def(name: &str, base: &str) -> Option<(f64, f64, bool)> {
    let t: &[(&str, f64, f64, bool)] = match name {
        "GJBeast01" => &[("bite", 0.025, 16.0, true), ("attack01", 0.025, 3.0, false), ("attack01_loop", 0.025, 7.0, true), ("attack01_end", 0.04, 6.0, false), ("idle01", 0.05, 8.0, true)],
        "GJBeast02" | "GJBeast03" => &[("idle01", 0.06, 9.0, false), ("idle02", 0.06, 9.0, false)],
        "GJBeast04" => &[("idle01", 0.06, 14.0, false), ("idle02", 0.06, 14.0, false), ("idle03", 0.06, 14.0, false), ("attack01", 0.06, 9.0, false), ("attack02", 0.06, 3.0, false),
            ("attack02_loop", 0.05, 6.0, true), ("attack02_end", 0.06, 5.0, false), ("sleep", 0.06, 9.0, false), ("sleep_loop", 0.05, 13.0, true), ("sleep_end", 0.06, 9.0, false)],
        "GJBeast05" => &[("idle01", 0.06, 13.0, false), ("idle02", 0.06, 12.0, false), ("idle03", 0.06, 9.0, false), ("toAttack01", 0.045, 8.0, false), ("attack01", 0.06, 13.0, true),
            ("attack02", 0.06, 13.0, false), ("toAttack03", 0.045, 8.0, false), ("attack03", 0.04, 4.0, true), ("fromAttack03", 0.05, 9.0, false)],
        _ => return None,
    };
    t.iter().find(|e| e.0 == base).map(|e| (e.1, e.2, e.3))
}
fn default_animation(name: &str) -> Option<&'static str> { Some(if name == "GJBeast01" { "bite" } else { "idle01" }) }
fn names_by_id(id: i64) -> Option<&'static [&'static str]> {
    Some(match id {
        918 => &["bite", "attack01", "attack01_end", "idle01", "attack01_loop"],
        1584 => &["idle01", "idle02", "idle03", "attack01", "attack02", "attack02_end", "sleep", "sleep_loop", "sleep_end", "attack02_loop"],
        2012 => &["idle01", "idle02", "toAttack01", "attack01", "attack02", "toAttack03", "attack03", "idle03", "fromAttack03"],
        _ => return None,
    })
}
pub fn eye_for(id: i64) -> Option<(&'static str, &'static str)> { if id == 918 { Some(("GJBeast01_03_001.png", "GJBeast01_01_001.png")) } else { None } }
pub fn secondary_tags(id: i64) -> Option<&'static [f64]> {
    Some(match id { 1327 | 1328 | 1584 => &[0.0], 2012 => &[1.0, 2.0], _ => return None })
}

#[derive(Debug, Clone)]
pub struct Part { pub texture: String, pub x: f64, pub y: f64, pub sx: f64, pub sy: f64, pub rot: f64, pub z: f64 }
#[derive(Debug, Clone)]
pub struct Frame { pub parts: Vec<Part>, pub tex_set: Vec<String> }
#[derive(Debug, Clone)]
pub struct RawPart { pub key: String, pub texture: Option<String>, pub tag: f64 }
#[derive(Debug)]
pub struct Desc {
    /// `groups`: `"<name>_<base>"` -> frame keys sorted by their number
    pub groups: BTreeMap<String, Vec<String>>,
    pub frames: BTreeMap<String, Frame>,
    /// `container` in file order: frame key -> its `sprite_*` entries
    pub container: Vec<(String, Vec<RawPart>)>,
}

static DESCS: Mutex<BTreeMap<String, Arc<Desc>>> = Mutex::new(BTreeMap::new());

fn pair(s: Option<&str>, dx: f64, dy: f64) -> (f64, f64) {
    // /\{\s*([^,]+)\s*,\s*([^}]+)\s*\}/ then parseFloat(..) || 0
    let Some(s) = s else { return (dx, dy) };
    let Some(open) = s.find('{') else { return (dx, dy) };
    let rest = &s[open + 1..];
    let Some(comma) = rest.find(',') else { return (dx, dy) };
    let first = &rest[..comma];
    let after = &rest[comma + 1..];
    let Some(close) = after.find('}') else { return (dx, dy) };
    if first.trim().is_empty() || after[..close].trim().is_empty() { return (dx, dy); }
    let f = |t: &str| { let v = crate::js::parse_float(t).unwrap_or(f64::NAN); if v.is_nan() || v == 0.0 { 0.0 } else { v } };
    (f(first), f(&after[..close]))
}
fn num_or0(j: Option<&Json>, dflt: &str) -> f64 {
    let s = match j { Some(Json::Str(s)) => s.clone(), Some(Json::Num(n)) => crate::json::num(*n), _ => dflt.to_string() };
    let s = if s.is_empty() { dflt.to_string() } else { s };
    let v = crate::js::parse_float(&s).unwrap_or(f64::NAN);
    if v.is_nan() { 0.0 } else { v }
}

pub fn add_desc(name: &str, json: &str) -> bool {
    let Ok(j) = parse(json) else { return false };
    let Some(container) = j.get("animationContainer").and_then(Json::entries) else { return false };
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, _) in container {
        // /^(.*?)_(\d+)\.png$/: the base is everything before the last `_<digits>.png`
        let Some(stem) = key.strip_suffix(".png") else { continue };
        let Some(us) = stem.rfind('_') else { continue };
        let digits = &stem[us + 1..];
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) { continue; }
        groups.entry(stem[..us].to_string()).or_default().push(key.clone());
    }
    let n = |k: &str| -> f64 { let d = k.strip_suffix(".png").and_then(|s| s.rsplit('_').next()).unwrap_or("0"); crate::js::parse_int(d).unwrap_or(0) as f64 };
    for list in groups.values_mut() { list.sort_by(|a, b| n(a).partial_cmp(&n(b)).unwrap_or(std::cmp::Ordering::Equal)); }
    let mut frames = BTreeMap::new();
    let mut raw = Vec::new();
    for (key, frame) in container {
        let mut parts = Vec::new();
        let mut tex_set: Vec<String> = Vec::new();
        let mut rparts = Vec::new();
        for (pk, part) in frame.entries().unwrap_or(&[]) {
            if !pk.starts_with("sprite_") { continue; }
            let tex = match part.get("texture") { Some(Json::Str(s)) => Some(s.clone()), _ => None };
            let tag = num_or0(part.get("tag"), "NaN");
            let tag = if part.get("tag").is_none() { f64::NAN } else { tag };
            rparts.push(RawPart { key: pk.clone(), texture: tex.clone(), tag });
            let pos = pair(part.get_str("position"), 0.0, 0.0);
            let sc = pair(part.get_str("scale"), 1.0, 1.0);
            let fl = pair(part.get_str("flipped"), 0.0, 0.0);
            let texture = tex.clone().unwrap_or_default();
            parts.push(Part {
                texture: texture.clone(), x: pos.0, y: pos.1,
                sx: sc.0 * if fl.0 != 0.0 { -1.0 } else { 1.0 }, sy: sc.1 * if fl.1 != 0.0 { -1.0 } else { 1.0 },
                rot: num_or0(part.get("rotation"), "0") * std::f64::consts::PI / 180.0,
                z: num_or0(part.get("zValue"), "0"),
            });
            if !tex_set.contains(&texture) { tex_set.push(texture); }
        }
        frames.insert(key.clone(), Frame { parts, tex_set });
        raw.push((key.clone(), rparts));
    }
    DESCS.lock().unwrap().insert(name.to_string(), Arc::new(Desc { groups, frames, container: raw }));
    true
}
pub fn desc_for(name: &str) -> Option<Arc<Desc>> { DESCS.lock().unwrap().get(name).cloned() }

#[derive(Debug, Clone)]
pub struct BState {
    pub object_id: i64,
    pub anim_name: &'static str,
    pub desc: Arc<Desc>,
    pub anim_id: i64,
    pub base: Option<String>,
    pub group: Option<Vec<String>>,
    pub frame_idx: f64,
    pub timer: f64,
    pub rng: f64,
    pub delay: f64,
    pub total: f64,
    pub looped: bool,
    pub pending: Option<(String, i64, f64)>,
}

impl BState {
    fn group_key(&self, base: &str) -> String { format!("{}_{}", self.anim_name, base) }
    fn delay_of(&self, base: &str) -> f64 {
        match anim_def(self.anim_name, base) { Some((d, _, _)) if d.is_finite() && d > 0.0 => d, _ => DELAY_30FPS }
    }
    fn frame_count(&self, base: &str) -> f64 {
        let avail = self.desc.groups.get(&self.group_key(base)).map_or(0, |g| g.len()) as f64;
        match anim_def(self.anim_name, base) { Some((_, f, _)) if f.is_finite() && f > 0.0 => f.min(avail), _ => avail }
    }
    fn is_looping(&self, base: &str) -> bool {
        match anim_def(self.anim_name, base) { Some((_, _, l)) => l, None => base.ends_with("_loop") }
    }
    pub fn frame_exists(&self, base: &str) -> bool { self.desc.groups.get(&self.group_key(base)).map_or(false, |g| !g.is_empty()) }
    fn randomize_timer(&mut self) { self.randomize_timer_with(&mut m::random); }
    fn randomize_timer_with(&mut self, rnd: &mut dyn FnMut() -> f64) {
        self.rng = rnd();
        let d = self.delay_for_current();
        self.timer = self.rng * d;
    }
    fn delay_for_current(&self) -> f64 { match &self.base { Some(b) => self.delay_of(b), None => DELAY_30FPS } }

    pub fn create(object_id: i64) -> Option<BState> { Self::create_with(object_id, &mut m::random) }
    /// `beastAnimCreateState`; `rnd` is the random source (the level builder's own stream while a level is spawned)
    pub fn create_with(object_id: i64, rnd: &mut dyn FnMut() -> f64) -> Option<BState> {
        let name = anim_name(object_id)?;
        let desc = desc_for(name)?;
        let rng = rnd();
        let mut s = BState { object_id, anim_name: name, desc, anim_id: 0, base: None, group: None, frame_idx: 0.0, timer: 0.0, rng, delay: 0.0, total: 0.0, looped: false, pending: None };
        s.default_base_with(rnd);
        Some(s)
    }

    pub fn set_base(&mut self, base: &str, anim_id: Option<i64>) -> bool {
        let Some(g) = self.desc.groups.get(&self.group_key(base)).filter(|g| !g.is_empty()).cloned() else { return false };
        self.base = Some(base.to_string());
        self.group = Some(g);
        self.delay = self.delay_of(base);
        self.total = self.frame_count(base);
        self.looped = self.is_looping(base);
        self.frame_idx = 0.0;
        self.timer = 0.0;
        if let Some(a) = anim_id { self.anim_id = a; }
        true
    }

    pub fn default_base(&mut self) { self.default_base_with(&mut m::random); }
    pub fn default_base_with(&mut self, rnd: &mut dyn FnMut() -> f64) {
        self.pending = None;
        if let Some(d) = default_animation(self.anim_name) {
            if self.set_base(d, None) {
                let total = self.frame_count(d);
                if total > 1.0 { self.frame_idx = (rnd() * total).floor(); }
                self.randomize_timer_with(rnd);
                return;
            }
        }
        // (`BEAST_ANIM_DEFAULT_ANIM_ID` is 0 for every beast, then the idle01 fallback)
        if let Some(b) = names_by_id(self.object_id).and_then(|n| n.first()) {
            if self.set_base(b, Some(0)) { self.randomize_timer_with(rnd); return; }
        }
        self.set_base("idle01", Some(0));
        let len = self.desc.groups.get(&self.group_key("idle01")).map_or(0, |g| g.len());
        if len > 1 { self.frame_idx = (rnd() * len as f64).floor(); }
        self.randomize_timer_with(rnd);
    }

    fn finish_action(&mut self) {
        let rnd = m::random();
        let id = self.object_id;
        let prev = self.base.clone().unwrap_or_default();
        let mut go = |s: &mut BState, b: &str, a: i64| { s.set_base(b, Some(a)); s.randomize_timer(); };
        match id {
            918 => {
                if prev == "attack01" {
                    if self.frame_exists("attack01_loop") { go(self, "attack01_loop", 4); return; }
                    go(self, "bite", 0); return;
                }
                if prev == "bite" { return; }
                go(self, "idle01", 3); return;
            }
            1327 => { if rnd <= 0.75 { go(self, "idle01", 0) } else { go(self, "idle02", 0) } return; }
            1328 => { if rnd <= 0.9 { go(self, "idle01", 0) } else { go(self, "idle02", 0) } return; }
            1584 => {
                if prev == "attack02" && self.frame_exists("attack02_loop") { go(self, "attack02_loop", 9); return; }
                if prev == "sleep" && self.frame_exists("sleep_loop") { go(self, "sleep_loop", 7); return; }
                if prev == "sleep_loop" && self.frame_exists("sleep_end") { go(self, "sleep_end", 8); return; }
                if prev == "sleep_end" { go(self, "idle01", 0); return; }
                if prev == "attack02_loop" || prev == "attack02_end" { go(self, "idle01", 0); return; }
                if prev == "attack01" { go(self, "idle01", 0); return; }
                if rnd <= 0.75 { go(self, "idle01", 0) } else { go(self, "idle02", 0) }
                return;
            }
            2012 => {
                if prev == "toAttack03" { go(self, "attack03", 6); return; }
                if prev == "fromAttack03" || prev == "attack02" || prev == "toAttack01" { go(self, "attack01", 3); return; }
                if prev == "attack01" || prev == "attack03" { if rnd <= 0.75 { go(self, "idle01", 0) } else { go(self, "idle02", 0) } return; }
                if prev == "idle01" { if rnd <= 0.75 { go(self, "idle01", 0) } else { go(self, "idle02", 1) } return; }
                go(self, "idle01", 0); return;
            }
            _ => {}
        }
        go(self, "idle01", 0);
    }

    /// `beastAnimCommand`
    pub fn command(&mut self, anim_idx: i64, start_delay_override: Option<f64>) -> bool {
        let base = match names_by_id(self.object_id) { Some(n) if anim_idx >= 0 && (anim_idx as usize) < n.len() => n[anim_idx as usize], _ => return false };
        if !self.frame_exists(base) { return false; }
        let delay = match start_delay_override { Some(d) if d.is_finite() => d.min(START_DELAY_MAX).max(0.0), _ => m::random() * START_DELAY_MAX };
        if delay > 0.0 { self.pending = Some((base.to_string(), anim_idx, delay)); return true; }
        self.pending = None;
        self.set_base(base, Some(anim_idx));
        self.randomize_timer();
        true
    }

    /// `beastAnimAdvance` (`dt` in seconds)
    pub fn advance(&mut self, dt: f64) {
        if self.base.is_none() { return; }
        if self.group.is_none() { let b = self.base.clone().unwrap(); self.set_base(&b, None); }
        if !self.timer.is_finite() { self.timer = 0.0; }
        if !self.frame_idx.is_finite() { self.frame_idx = 0.0; }
        if !dt.is_finite() || dt <= 0.0 { return; }
        let dt = dt.min(0.1);
        if let Some(p) = self.pending.as_mut() {
            p.2 -= dt;
            if p.2 <= 0.0 {
                let (b, a, _) = self.pending.take().unwrap();
                self.set_base(&b, Some(a));
                self.randomize_timer();
                return;
            }
        }
        self.timer += dt;
        let mut guard = 0;
        while guard < 240 {
            guard += 1;
            let delay = self.delay;
            if self.timer < delay { break; }
            self.timer -= delay;
            self.frame_idx += 1.0;
            let total = self.total;
            if total == 0.0 { self.default_base(); break; }
            if self.frame_idx >= total {
                if self.looped { self.frame_idx = 0.0; continue; }
                self.finish_action();
                break;
            }
        }
    }

    pub fn json(&self) -> String {
        use crate::json::{num, str_lit};
        format!("[{},{},{},{},{},{},{}]", str_lit(self.base.as_deref().unwrap_or("")), num(self.anim_id as f64), num(self.frame_idx), num(self.timer), num(self.rng),
            self.pending.as_ref().map_or("null".to_string(), |p| format!("[{},{},{}]", str_lit(&p.0), p.1, num(p.2))), num(self.total))
    }
}

/// `beastAnimCommandMany`
pub fn command_many(states: &mut [&mut BState], anim_idx: i64) {
    let n = states.len();
    for i in 0..n {
        let stagger = if n > 1 { (i as f64 + m::random()) / n as f64 } else { m::random() };
        states[i].command(anim_idx, Some(stagger * START_DELAY_MAX));
    }
}

/// One spawned beast (`level._beastSprites` entry).
#[derive(Debug, Clone)]
pub struct Beast {
    pub state: BState,
    /// (texture, sprite) in `groupMap` order
    pub parts: Vec<(String, usize)>,
    pub glows: BTreeMap<String, usize>,
    pub hidden: Vec<usize>,
    pub eye: Option<usize>,
    pub eye_parent: Option<String>,
    pub world_x: f64,
    pub base_y: f64,
    pub scale: f64,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rot_cos: f64,
    pub rot_sin: f64,
    pub last_frame_key: Option<String>,
    pub base_rot: Option<f64>,
}

#[derive(Debug, Default)]
pub struct Beasts { pub list: Vec<Beast>, pub of_sprite: BTreeMap<usize, usize> }

impl Game {
    /// `beastAnimUpdate`
    pub fn beast_update(&mut self, delta_ms: f64) {
        if self.beasts.list.is_empty() { return; }
        let dt = if delta_ms.is_finite() { delta_ms / 1000.0 } else { 0.0 };
        for bi in 0..self.beasts.list.len() {
            let hidden = self.beasts.list[bi].hidden.clone();
            for si in hidden { let s = &mut self.fx.sprites[si]; s.visible = false; s.active = false; }
            self.beasts.list[bi].state.advance(dt);
            let (frame_key, anchor, flip_x, flip_y, scale, world_x, base_y, rot_cos, rot_sin) = {
                let b = &self.beasts.list[bi];
                let st = &b.state;
                let Some(group) = st.group.as_ref().filter(|g| !g.is_empty()) else { continue };
                let key = group[(st.frame_idx as usize).min(group.len() - 1)].clone();
                (key, b.parts.first().map(|p| p.1), b.flip_x, b.flip_y, b.scale, b.world_x, b.base_y, b.rot_cos, b.rot_sin)
            };
            let desc = self.beasts.list[bi].state.desc.clone();
            let Some(frame) = desc.frames.get(&frame_key) else { continue };
            let Some(anchor_si) = anchor else { continue };
            if self.sprite_container_hidden(anchor_si) { continue; }
            let flip_sign_x = if flip_x { -1.0 } else { 1.0 };
            let anchor_active = self.fx.sprites[anchor_si].active;
            let (mut rig_off_x, mut rig_off_y, mut rig_rot) = (0.0, 0.0, 0.0);
            let gids: Vec<i64> = match self.fx.sprites[anchor_si].props.get("_eeGroups") { Some(crate::visual::Val::Arr(g)) => g.iter().map(|v| *v as i64).collect(), _ => Vec::new() };
            if anchor_active && !gids.is_empty() {
                let (ox, oy) = self.group_offset_sum_pub(&gids);
                rig_off_x = ox; rig_off_y = oy;
                for gid in &gids {
                    let (total, center) = match self.trig.rotations.get(gid) { Some(r) if r.total_rad != 0.0 && !r.total_rad.is_nan() => (r.total_rad, r.center), _ => continue };
                    rig_rot += total;
                    if center > 0 {
                        let (cx, cy) = self.group_center_pub(center);
                        let (pdx, pdy) = (world_x + rig_off_x - cx, base_y + rig_off_y - cy);
                        let (pc, ps) = (m::cos(total), m::sin(total));
                        rig_off_x = cx + pdx * pc - pdy * ps - world_x;
                        rig_off_y = cy + pdx * ps + pdy * pc - base_y;
                    }
                }
            }
            let (rig_cos, rig_sin) = if rig_rot != 0.0 { (m::cos(rig_rot), m::sin(rig_rot)) } else { (1.0, 0.0) };
            if self.beasts.list[bi].last_frame_key.as_deref() != Some(frame_key.as_str()) {
                self.beasts.list[bi].last_frame_key = Some(frame_key.clone());
                let parts = self.beasts.list[bi].parts.clone();
                for (tex, si) in parts {
                    if frame.tex_set.contains(&tex) { continue; }
                    let s = &mut self.fx.sprites[si]; s.scale_x = 0.0; s.scale_y = 0.0;
                    if let Some(&gi) = self.beasts.list[bi].glows.get(&tex) { let g = &mut self.fx.sprites[gi]; g.scale_x = 0.0; g.scale_y = 0.0; }
                }
            }
            if self.beasts.list[bi].base_rot.is_none() { self.beasts.list[bi].base_rot = Some(m::atan2(rot_sin, rot_cos)); }
            let unit_scale = UNIT_SCALE * scale;
            let flip_sign_y = if flip_y { -1.0 } else { 1.0 };
            let angle_sign = if flip_x != flip_y { -1.0 } else { 1.0 };
            let angle_base = self.beasts.list[bi].base_rot.unwrap() + rig_rot;
            let scale_sign_x = flip_sign_x * scale;
            let scale_sign_y = flip_sign_y * scale;
            let eye = self.beasts.list[bi].eye;
            let eye_parent = self.beasts.list[bi].eye_parent.clone();
            for part in &frame.parts {
                let Some(&(_, si)) = self.beasts.list[bi].parts.iter().find(|p| p.0 == part.texture) else { continue };
                let local_x = part.x * unit_scale * flip_sign_x;
                let local_y = -part.y * unit_scale * flip_sign_y;
                let rig_lx = local_x * rig_cos - local_y * rig_sin;
                let rig_ly = local_x * rig_sin + local_y * rig_cos;
                let wx = world_x + rig_off_x + (rig_lx * rot_cos - rig_ly * rot_sin);
                let wy = base_y + rig_off_y + (rig_lx * rot_sin + rig_ly * rot_cos);
                let s = &mut self.fx.sprites[si];
                let edx = s.num("_eeEffectOffsetX").map_or(0.0, |v| if v.is_nan() { 0.0 } else { v });
                let edy = s.num("_eeEffectOffsetY").map_or(0.0, |v| if v.is_nan() { 0.0 } else { v });
                let escale = s.num("_eeEffectScale").unwrap_or(1.0);
                s.set_num("_eeEffectOffsetX", 0.0); s.set_num("_eeEffectOffsetY", 0.0); s.set_num("_eeEffectScale", 1.0);
                let angle = part.rot * angle_sign + angle_base;
                s.x = wx + edx; s.y = wy + edy; s.rotation = angle;
                let psx = part.sx * scale_sign_x * escale;
                let psy = part.sy * scale_sign_y * escale;
                s.scale_x = psx; s.scale_y = psy;
                if s.num("_eeZDepthBase").is_none() { let z = s.num("_eeZDepth").unwrap_or(0.0); s.set_num("_eeZDepthBase", z); }
                let z_depth = s.num("_eeZDepthBase").unwrap() + part.z * 0.001;
                s.set_num("_eeZDepth", z_depth);
                if let Some(&gi) = self.beasts.list[bi].glows.get(&part.texture) {
                    let g = &mut self.fx.sprites[gi];
                    g.x = wx + edx; g.y = wy + edy; g.rotation = angle; g.scale_x = psx; g.scale_y = psy;
                    g.set_num("_eeZDepth", z_depth - 0.001);
                }
                if let (Some(ei), Some(ep)) = (eye, eye_parent.as_ref()) {
                    if *ep == part.texture {
                        let e = &mut self.fx.sprites[ei];
                        e.x = wx + edx; e.y = wy + edy; e.rotation = angle; e.scale_x = psx; e.scale_y = psy;
                    }
                }
            }
        }
    }

    /// `_executeAnimationTrigger`'s command: the beasts owning a sprite of the group get the animation
    pub fn beast_command(&mut self, g: i64, id: f64) {
        let mut recs: Vec<usize> = Vec::new();
        if let Some(list) = self.fx.group_sprites.get(&g) {
            for si in list { if let Some(&r) = self.beasts.of_sprite.get(si) { if !recs.contains(&r) { recs.push(r); } } }
        }
        let mut taken: Vec<BState> = recs.iter().map(|&r| self.beasts.list[r].state.clone()).collect();
        { let mut refs: Vec<&mut BState> = taken.iter_mut().collect(); command_many(&mut refs, id as i64); }
        for (k, &r) in recs.iter().enumerate() { self.beasts.list[r].state = taken[k].clone(); }
    }

    /// `resetAnimationTriggers`: every beast goes back to its default animation.
    pub fn beasts_default(&mut self) { for b in &mut self.beasts.list { b.state.default_base(); } }
}
