//! Level editor data model: objects with JS property order preserved, and the exact serialization of
//! `LevelEditor._serializeObject` / `_serializeLevel` (level-editor.js).
use crate::js::{parse_float, parse_int};
use crate::level::{self, LevelObject};

/// JavaScript `Number.prototype.toString()` for a double.
pub fn js_num(v: f64) -> String {
    if v.is_nan() { return "NaN".into(); }
    if v == 0.0 { return "0".into(); }
    if v.is_infinite() { return if v < 0.0 { "-Infinity".into() } else { "Infinity".into() }; }
    let neg = v < 0.0;
    let a = v.abs();
    // shortest round-trip digits and decimal exponent from Rust's `{:e}`
    let e = format!("{:e}", a);
    let (mant, exp) = e.split_once('e').unwrap();
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let n = exp.parse::<i32>().unwrap() + 1; // position of the decimal point relative to digits
    let k = digits.len() as i32;
    let body = if k <= n && n <= 21 {
        format!("{}{}", digits, "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{}", "0".repeat((-n) as usize), digits)
    } else {
        let e = n - 1;
        let sign = if e < 0 { '-' } else { '+' };
        if k == 1 { format!("{}e{}{}", digits, sign, e.abs()) } else { format!("{}.{}e{}{}", &digits[..1], &digits[1..], sign, e.abs()) }
    };
    if neg { format!("-{}", body) } else { body }
}

/// A JS object with string values: integer-like keys iterate first (ascending), then the others in insertion order.
#[derive(Debug, Clone, Default)]
pub struct JsObj {
    ints: std::collections::BTreeMap<u32, String>,
    strs: Vec<(String, String)>,
}

fn array_index(key: &str) -> Option<u32> {
    if key.is_empty() || key.len() > 10 || !key.bytes().all(|b| b.is_ascii_digit()) { return None; }
    if key.len() > 1 && key.starts_with('0') { return None; }
    let n: u64 = key.parse().ok()?;
    if n < 4_294_967_295 { Some(n as u32) } else { None }
}

impl JsObj {
    pub fn set(&mut self, key: &str, v: String) {
        if let Some(i) = array_index(key) { self.ints.insert(i, v); return; }
        match self.strs.iter_mut().find(|(k, _)| k == key) { Some(e) => e.1 = v, None => self.strs.push((key.to_string(), v)) }
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        if let Some(i) = array_index(key) { return self.ints.get(&i); }
        self.strs.iter().find(|(k, _)| k == key).map(|e| &e.1)
    }
    pub fn delete(&mut self, key: &str) {
        if let Some(i) = array_index(key) { self.ints.remove(&i); return; }
        self.strs.retain(|(k, _)| k != key);
    }
    pub fn entries(&self) -> Vec<(String, &String)> {
        self.ints.iter().map(|(k, v)| (k.to_string(), v)).chain(self.strs.iter().map(|(k, v)| (k.clone(), v))).collect()
    }
}

/// `parseObject`'s `objectData` (the `_raw` of an object), in JS property order.
pub fn raw_object(object_string: &str) -> JsObj {
    let parts: Vec<&str> = object_string.split(',').collect();
    let mut o = JsObj::default();
    let mut i = 0;
    while i + 1 < parts.len() { o.set(parts[i], parts[i + 1].to_string()); i += 2; }
    o
}

/// An object as the editor holds it: the parsed fields plus the raw property bag.
#[derive(Debug, Clone)]
pub struct EdObj { pub o: LevelObject, pub raw: JsObj }

impl EdObj {
    pub fn parse(s: &str) -> Option<EdObj> { Some(EdObj { o: level::parse_object(s)?, raw: raw_object(s) }) }
}

fn is_text_object(id: i64) -> bool { id == 914 }

fn encode_text(s: &str) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let b = s.as_bytes();
    let mut out = String::new();
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        if c.len() > 1 { out.push(T[(n >> 6 & 63) as usize] as char); }
        if c.len() > 2 { out.push(T[(n & 63) as usize] as char); }
    }
    out
}

fn finite_or(v: f64, fb: f64) -> f64 { if v.is_finite() { v } else { fb } }
fn int_str(v: Option<i64>) -> String { match v { Some(v) => v.to_string(), None => "NaN".into() } }

/// `_getEditorObjectGroupIds`
pub fn group_ids(e: &EdObj) -> Vec<i64> {
    let mut vals: Vec<i64> = Vec::new();
    let mut add = |s: &str| for p in s.split('.') { if let Some(v) = parse_int(p) { if v > 0 { vals.push(v); } } };
    add(e.raw.get("33").map(|s| s.as_str()).unwrap_or(""));
    add(e.raw.get("57").map(|s| s.as_str()).unwrap_or(""));
    add(&e.o.groups);
    let mut out: Vec<i64> = Vec::new();
    for v in vals { if !out.contains(&v) { out.push(v); } }
    out
}

/// `_serializeObject`; empty string when the object is skipped.
pub fn serialize_object(e: &EdObj) -> String {
    let o = &e.o;
    let id = match o.id { Some(i) if i != 0 => i, _ => return String::new() };
    if id == 749 { return String::new(); }
    let mut d = e.raw.clone();
    d.set("1", id.to_string());
    d.set("2", js_num(o.x));
    d.set("3", js_num(o.y));
    d.set("4", if o.flip_x { "1" } else { "0" }.into());
    d.set("5", if o.flip_y { "1" } else { "0" }.into());
    let rotation = finite_or(o.rot, 0.0);
    let warp = finite_or(o.warp, 0.0);
    if warp != 0.0 {
        d.delete("6");
        d.set("131", js_num(rotation - warp / 2.0));
        d.set("132", js_num(rotation + warp / 2.0));
    } else {
        d.set("6", js_num(rotation));
        d.delete("131"); d.delete("132");
    }
    let uniform = finite_or(o.scale, 1.0);
    let sx = finite_or(o.scale_x, uniform);
    let sy = finite_or(o.scale_y, uniform);
    let had_axis = e.raw.get("128").is_some() || e.raw.get("129").is_some();
    if sx != sy || had_axis {
        d.delete("32");
        d.set("128", js_num(sx));
        d.set("129", js_num(sy));
    } else {
        d.set("32", js_num(sx));
        d.delete("128"); d.delete("129");
    }
    d.set("20", o.editor_layer.map(|v| v.to_string()).unwrap_or_else(|| "NaN".into()));
    d.set("24", int_str(o.z_layer));
    d.set("25", int_str(o.z_order));
    d.set("61", o.editor_layer2.map(|v| v.to_string()).unwrap_or_else(|| "NaN".into()));
    d.set("57", group_ids(e).iter().map(|v| v.to_string()).collect::<Vec<_>>().join("."));
    d.delete("33");
    d.set("21", int_str(o.color1));
    d.set("22", int_str(o.color2));
    // parsed objects always have `text` defined
    d.set("31", if is_text_object(id) { encode_text(&o.text) } else { o.text.clone() });
    d.set("kA2", int_str(o.game_mode));
    d.set("kA3", int_str(o.mini_mode));
    d.set("kA4", int_str(o.speed));
    d.set("kA8", int_str(o.dual_mode));
    d.set("kA28", int_str(o.mirrored));
    d.set("kA11", if o.flip_gravity { "1" } else { "0" }.into());
    if id == 1346 {
        let parts = |v: Option<&String>| -> Vec<i64> {
            v.map(|s| s.as_str()).unwrap_or("").split(|c| c == ',' || c == '.').filter_map(|p| parse_int(p.trim())).filter(|v| *v > 0).collect()
        };
        let target = parts(d.get("51")).first().copied().unwrap_or(0);
        let center = parts(d.get("71")).first().copied().unwrap_or(0);
        d.set("51", target.to_string());
        if center > 0 { d.set("71", center.to_string()); } else { d.delete("71"); }
    }
    d.entries().iter().map(|(k, v)| format!("{},{}", k, v)).collect::<Vec<_>>().join(",")
}

/// gzip like `pako.gzip` (header and trailer the same; the deflate stream may differ byte for byte).
pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    out.extend(miniz_oxide::deflate::compress_to_vec(data, 6));
    out.extend(level::crc32(data).to_le_bytes());
    out.extend((data.len() as u32).to_le_bytes());
    out
}

fn base64url(b: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        if c.len() > 1 { out.push(T[(n >> 6 & 63) as usize] as char); }
        if c.len() > 2 { out.push(T[(n & 63) as usize] as char); }
    }
    out
}

#[derive(Debug, Clone, Default)]
pub struct EdLevel { pub settings: String, pub objects: Vec<EdObj> }

impl EdLevel {
    /// `_editorSettingsToMap`: the `key,value,key,value` list in its order
    pub fn pairs(&self) -> Vec<(String, String)> {
        let parts: Vec<&str> = self.settings.split(',').collect();
        let mut v = Vec::new();
        let mut i = 0;
        while i + 1 < parts.len() { v.push((parts[i].to_string(), parts[i + 1].to_string())); i += 2; }
        v
    }
    pub fn put_pairs(&mut self, p: &[(String, String)]) { self.settings = p.iter().map(|(k, v)| format!("{},{}", k, v)).collect::<Vec<_>>().join(","); }
    pub fn get_str(&self, key: &str) -> Option<String> { self.pairs().into_iter().rev().find(|(k, _)| k == key).map(|(_, v)| v) }
    /// `_getEditorStartValue`
    pub fn get_i(&self, key: &str, fallback: i64) -> i64 { self.get_str(key).and_then(|v| parse_int(&v)).unwrap_or(fallback) }
    /// `_setEditorStartValueDraft`: replaces the value in place, or appends the key
    pub fn set_str(&mut self, key: &str, value: &str) {
        let mut p = self.pairs();
        let mut found = false;
        for e in p.iter_mut() { if e.0 == key { e.1 = value.to_string(); found = true; } }
        if !found { p.push((key.to_string(), value.to_string())); }
        self.put_pairs(&p);
    }
    /// `_getEditorColorChannel`: the colour of a channel from `kS38`, the legacy keys or the colour manager's initial one
    pub fn color_channel(&self, ch: i64) -> Option<(i64, i64, i64)> {
        let entry = |e: &str| -> (i64, i64, i64, i64) {
            let parts: Vec<&str> = e.split('_').collect();
            let mut m = std::collections::BTreeMap::new();
            let mut i = 0;
            while i + 1 < parts.len() { m.insert(parts[i].to_string(), parts[i + 1].to_string()); i += 2; }
            let g = |k: &str, d: i64| m.get(k).and_then(|v| parse_int(v)).filter(|v| *v != 0 || k != "6").unwrap_or(d);
            (m.get("6").and_then(|v| parse_int(v)).unwrap_or(0), m.get("1").and_then(|v| parse_int(v)).unwrap_or(255), m.get("2").and_then(|v| parse_int(v)).unwrap_or(255), { let _ = g; m.get("3").and_then(|v| parse_int(v)).unwrap_or(255) })
        };
        if let Some(k) = self.get_str("kS38") {
            for e in k.split('|').filter(|e| !e.is_empty()) { let (c, r, g, b) = entry(e); if c == ch { return Some((r, g, b)); } }
        }
        let legacy = match ch { 1000 => Some("kS29"), 1001 => Some("kS30"), _ => None };
        if let Some(k) = legacy.and_then(|k| self.get_str(k)).filter(|v| !v.is_empty()) { let (_, r, g, b) = entry(&k); return Some((r, g, b)); }
        None
    }
    /// `_setEditorColorChannelDraft` (the settings part)
    pub fn set_color_channel(&mut self, ch: i64, (r, g, b): (i64, i64, i64)) {
        let make = |c: i64| format!("1_{}_2_{}_3_{}_6_{}_5_0_7_0", r, g, b, c);
        let mut entries: Vec<String> = self.get_str("kS38").map(|k| k.split('|').filter(|e| !e.is_empty()).map(String::from).collect()).unwrap_or_default();
        let mut replaced = false;
        for e in entries.iter_mut() {
            let parts: Vec<&str> = e.split('_').collect();
            let mut i = 0; let mut c = 0;
            while i + 1 < parts.len() { if parts[i] == "6" { c = parse_int(parts[i + 1]).unwrap_or(0); } i += 2; }
            if c == ch { *e = make(ch); replaced = true; break; }
        }
        if !replaced { entries.push(make(ch)); }
        self.set_str("kS38", &entries.join("|"));
        if ch == 1000 { self.set_str("kS29", &make(1000)); } else if ch == 1001 { self.set_str("kS30", &make(1001)); }
    }
    /// Same decompress + split as `parseLevel`, keeping each object's property order.
    pub fn parse(level_string: &str) -> EdLevel {
        let text = level::decompress(level_string).unwrap_or_else(|| level_string.to_string());
        let mut parts = text.split(';');
        let settings = parts.next().unwrap_or("").to_string();
        EdLevel { settings, objects: parts.filter(|p| !p.is_empty()).filter_map(EdObj::parse).collect() }
    }

    /// The uncompressed `settings;obj;obj...` text.
    pub fn plain(&self) -> String {
        let mut v = vec![self.settings.clone()];
        v.extend(self.objects.iter().map(serialize_object).filter(|s| !s.is_empty()));
        v.join(";")
    }

    /// `_serializeLevel`
    pub fn serialize(&self) -> String { base64url(&gzip(self.plain().as_bytes())) }
}

#[allow(dead_code)]
fn _unused() { let _ = parse_float; }
