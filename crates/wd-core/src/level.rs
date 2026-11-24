//! Port of `parseObject` / `parseLevel` / `_decodeTextObjectString` from level.js.
use crate::js::{parse_float, parse_int};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LevelObject {
    pub id: Option<i64>,
    pub x: f64,
    pub y: f64,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rot: f64,
    pub scale: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub warp: f64,
    pub editor_layer: Option<i64>,
    pub z_layer: Option<i64>,
    pub z_order: Option<i64>,
    pub editor_layer2: Option<i64>,
    pub groups: String,
    pub color1: Option<i64>,
    pub color2: Option<i64>,
    pub text: String,
    pub game_mode: Option<i64>,
    pub mini_mode: Option<i64>,
    pub speed: Option<i64>,
    pub dual_mode: Option<i64>,
    pub mirrored: Option<i64>,
    pub flip_gravity: bool,
    pub raw: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub settings: String,
    pub objects: Vec<LevelObject>,
}

fn legacy_19_color(k: i64) -> Option<i64> {
    Some(match k {
        1 => 1005,
        2 => 1006,
        3 => 1,
        4 => 2,
        5 => 1007,
        6 => 3,
        7 => 4,
        8 => 1003,
        _ => return None,
    })
}

/// Forgiving base64 like `atob`: ignores ASCII whitespace, tolerates missing/extra padding rules of the spec.
fn atob(input: &str) -> Option<Vec<u8>> {
    let mut s: Vec<u8> = input.bytes().filter(|b| !matches!(b, b' ' | b'\t' | b'\n' | 0x0c | b'\r')).collect();
    if s.len() % 4 == 0 {
        for _ in 0..2 {
            if s.last() == Some(&b'=') {
                s.pop();
            }
        }
    }
    if s.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in &s {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

fn url_to_std(s: &str) -> String {
    let mut b: String = s.chars().map(|c| match c { '-' => '+', '_' => '/', c => c }).collect();
    while b.len() % 4 != 0 {
        b.push('=');
    }
    b
}

pub fn decode_text_object_string(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let body = value.trim_end_matches('=');
    let pad = value.len() - body.len();
    let valid = !body.is_empty() && pad <= 2 && body.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-');
    if !valid {
        return value.to_string();
    }
    match atob(&url_to_std(value)).and_then(|b| String::from_utf8(b).ok()) {
        Some(s) => s,
        None => value.to_string(),
    }
}

fn read_scale(raw: Option<&String>) -> Option<f64> {
    let v = parse_float(raw?)?;
    if v.is_finite() && v != 0.0 { Some(v) } else { None }
}

fn or_zero(v: Option<&String>) -> &str {
    match v {
        Some(s) if !s.is_empty() => s.as_str(),
        _ => "0",
    }
}

pub fn parse_object(object_string: &str) -> Option<LevelObject> {
    let parts: Vec<&str> = object_string.split(',').collect();
    let mut data: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    while i + 1 < parts.len() {
        data.insert(parts[i].to_string(), parts[i + 1].to_string());
        i += 2;
    }
    let g = |k: &str| data.get(k);
    let object_id = parse_int(or_zero(g("1")));
    let uniform = read_scale(g("32")).unwrap_or(1.0);
    let sx = read_scale(g("128")).unwrap_or(uniform);
    let sy = read_scale(g("129")).unwrap_or(uniform);
    let mut color1 = parse_int(or_zero(g("21")));
    let mut color2 = parse_int(or_zero(g("22")));
    if let Some(k) = parse_int(or_zero(g("19"))).and_then(legacy_19_color) {
        color1 = Some(k);
        color2 = Some(0);
    }
    let read_float = |k: &str, fallback: f64| -> f64 {
        match g(k).and_then(|s| parse_float(s)) {
            Some(v) if v.is_finite() => v,
            _ => fallback,
        }
    };
    let mut groups: Vec<i64> = Vec::new();
    for key in ["33", "57"] {
        for part in g(key).map(|s| s.as_str()).unwrap_or("").split('.') {
            if let Some(v) = parse_int(part) {
                if v > 0 && !groups.contains(&v) {
                    groups.push(v);
                }
            }
        }
    }
    let group_string = groups.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(".");
    let base_rot = read_float("6", 0.0);
    let rot_x = read_float("131", base_rot);
    let rot_y = read_float("132", base_rot);
    let warp = ((rot_y - rot_x) % 360.0 + 540.0) % 360.0 - 180.0;
    let rotation = rot_x + warp / 2.0;
    if object_id == Some(0) {
        return None;
    }
    let int = |k: &str| parse_int(or_zero(g(k)));
    Some(LevelObject {
        id: object_id,
        x: parse_float(or_zero(g("2"))).unwrap_or(f64::NAN),
        y: parse_float(or_zero(g("3"))).unwrap_or(f64::NAN),
        flip_x: g("4").map(|s| s == "1").unwrap_or(false),
        flip_y: g("5").map(|s| s == "1").unwrap_or(false),
        rot: rotation,
        scale: if g("32").is_some() { uniform } else { (sx + sy) / 2.0 },
        scale_x: sx,
        scale_y: sy,
        warp,
        editor_layer: int("20"),
        z_layer: int("24"),
        z_order: int("25"),
        editor_layer2: int("61"),
        groups: group_string,
        color1,
        color2,
        text: decode_text_object_string(g("31").map(|s| s.as_str()).unwrap_or("")),
        game_mode: int("kA2"),
        mini_mode: int("kA3"),
        speed: int("kA4"),
        dual_mode: int("kA8"),
        mirrored: int("kA28"),
        flip_gravity: g("kA11").map(|s| s == "1").unwrap_or(false),
        raw: data.clone(),
    })
}

pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 { crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 }; }
    }
    !crc
}

/// Inflate like `pako.inflate`: gzip or zlib framing picked from the header.
/// A truncated stream yields what was decoded so far; corrupt data is an error.
fn inflate(bytes: &[u8]) -> Option<Vec<u8>> {
    use miniz_oxide::inflate::{decompress_to_vec, decompress_to_vec_zlib, TINFLStatus};
    let partial = |r: Result<Vec<u8>, miniz_oxide::inflate::DecompressError>| match r {
        Ok(v) => Some(v),
        Err(e) if e.status == TINFLStatus::NeedsMoreInput => Some(e.output),
        Err(_) => None,
    };
    if bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        let hdr = (|| {
            if bytes.len() < 10 { return None; }
            let flags = bytes[3];
            let mut pos = 10usize;
            if flags & 4 != 0 {
                let n = *bytes.get(pos)? as usize | (*bytes.get(pos + 1)? as usize) << 8;
                pos += 2 + n;
            }
            for bit in [8u8, 16u8] {
                if flags & bit != 0 {
                    while *bytes.get(pos)? != 0 { pos += 1; }
                    pos += 1;
                }
            }
            if flags & 2 != 0 { pos += 2; }
            if pos <= bytes.len() { Some(pos) } else { None }
        })();
        let pos = match hdr { Some(p) => p, None => return Some(Vec::new()) };
        let body = &bytes[pos..];
        return match decompress_to_vec(body) {
            Ok(v) => {
                // gzip trailer: crc32 + length of the decoded data
                let used_trailer = bytes.len() >= pos + 8;
                if used_trailer {
                    let t = &bytes[bytes.len() - 8..];
                    let want = u32::from_le_bytes([t[0], t[1], t[2], t[3]]);
                    if want != crc32(&v) { return None; }
                }
                Some(v)
            }
            Err(e) => partial(Err(e)),
        };
    }
    partial(decompress_to_vec_zlib(bytes))
}

pub(crate) fn decompress(level: &str) -> Option<String> {
    let bytes = atob(&url_to_std(level.trim()))?;
    inflate(&bytes).map(|out| String::from_utf8_lossy(&out).into_owned())
}

pub fn parse_level(level_string: &str) -> Level {
    let text = decompress(level_string).unwrap_or_else(|| level_string.to_string());
    let mut parts = text.split(';');
    let settings = parts.next().unwrap_or("").to_string();
    let objects = parts.filter(|p| !p.is_empty()).filter_map(parse_object).collect();
    Level { settings, objects }
}

// ---- JSON output (hand-written so the crate has no build-script dependencies) ----
use crate::json::{opt, str_lit, num, Obj};

impl LevelObject {
    pub fn to_json(&self) -> String {
        let mut raw = Obj::new();
        let mut keys: Vec<&String> = self.raw.keys().collect();
        keys.sort();
        for k in keys { raw.field(k, str_lit(&self.raw[k])); }
        let mut o = Obj::new();
        o.field("id", opt(self.id));
        o.field("x", num(self.x));
        o.field("y", num(self.y));
        o.field("flipX", self.flip_x.to_string());
        o.field("flipY", self.flip_y.to_string());
        o.field("rot", num(self.rot));
        o.field("scale", num(self.scale));
        o.field("scaleX", num(self.scale_x));
        o.field("scaleY", num(self.scale_y));
        o.field("warp", num(self.warp));
        o.field("editorLayer", opt(self.editor_layer));
        o.field("zLayer", opt(self.z_layer));
        o.field("zOrder", opt(self.z_order));
        o.field("editorLayer2", opt(self.editor_layer2));
        o.field("groups", str_lit(&self.groups));
        o.field("color1", opt(self.color1));
        o.field("color2", opt(self.color2));
        o.field("text", str_lit(&self.text));
        o.field("gameMode", opt(self.game_mode));
        o.field("miniMode", opt(self.mini_mode));
        o.field("speed", opt(self.speed));
        o.field("dualMode", opt(self.dual_mode));
        o.field("mirrored", opt(self.mirrored));
        o.field("flipGravity", self.flip_gravity.to_string());
        o.field("_raw", raw.finish());
        o.finish()
    }
}

impl Level {
    pub fn to_json(&self) -> String {
        let objs: Vec<String> = self.objects.iter().map(|o| o.to_json()).collect();
        let mut o = Obj::new();
        o.field("settings", str_lit(&self.settings));
        o.field("objects", format!("[{}]", objs.join(",")));
        o.finish()
    }
}

/// JS `String(n)` as far as a later `parseFloat` can tell (the shortest round-tripping form).
fn js_num_str(v: f64) -> String {
    if v.is_nan() { "NaN".into() } else if v == 0.0 { "0".into() } else if v.is_infinite() { if v > 0.0 { "Infinity".into() } else { "-Infinity".into() } } else { format!("{}", v) }
}

/// `_normalizeTeleportPortals`: standalone exit portals (749) are dropped and the Y offset to the nearest
/// entry portal (747, by X distance) is stored on that entry as raw key 54.
pub fn normalize_teleport_portals(objects: Vec<LevelObject>) -> Vec<LevelObject> {
    let (exits, mut normalized): (Vec<LevelObject>, Vec<LevelObject>) = objects.into_iter().partition(|o| o.id.unwrap_or(0) == 749);
    let or_zero = |v: f64| if v == 0.0 || v.is_nan() { 0.0 } else { v };
    for exit in &exits {
        let exit_x = or_zero(exit.x);
        let mut best: Option<usize> = None;
        let mut best_distance = f64::INFINITY;
        for (i, cand) in normalized.iter().enumerate() {
            if cand.id.unwrap_or(0) != 747 { continue; }
            let distance = (or_zero(cand.x) - exit_x).abs();
            if distance < best_distance { best_distance = distance; best = Some(i); }
        }
        if let Some(i) = best {
            let enter_y = or_zero(normalized[i].y);
            let ey = exit.y;
            let exit_y = if ey == 0.0 || ey.is_nan() { enter_y } else { ey };
            let s = js_num_str(exit_y - enter_y);
            normalized[i].raw.insert("54".into(), s);
        }
    }
    normalized
}

/// The coin filter of `loadLevel`: official levels have no user coins, other levels no secret coins, at most 3 of each.
pub fn filter_coins(objects: Vec<LevelObject>, official: u32) -> Vec<LevelObject> {
    if official > 1 { return objects; }
    let official = official == 1;
    let (mut secret, mut user) = (0, 0);
    objects.into_iter().filter(|o| {
        let id = o.id.unwrap_or(0);
        if id == 142 && !official { return false; }
        if id == 1329 && official { return false; }
        if id == 1329 { if user >= 3 { return false; } user += 1; return true; }
        if id != 142 { return true; }
        if secret >= 3 { return false; }
        secret += 1;
        true
    }).collect()
}
