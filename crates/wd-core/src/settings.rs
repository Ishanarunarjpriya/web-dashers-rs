//! `LevelObject._setUpSettings`: the level header's colour channels, blending flags, background and ground.
use crate::colors::{parse_channel_hsv, Copy, Rgb};
use crate::js::{parse_float, parse_int};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct LevelSettings {
    pub map: BTreeMap<String, String>,
    pub initial_colors: BTreeMap<i64, Rgb>,
    pub initial_copies: BTreeMap<i64, Copy>,
    pub initial_opacity: BTreeMap<i64, f64>,
    pub channel_blending: BTreeMap<i64, bool>,
    pub legacy_color_level: bool,
    pub background_id: Option<String>,
    pub ground_id: Option<String>,
}

fn prop_map(s: &str) -> BTreeMap<i64, String> {
    // `cp[parseInt(props[j], 10)] = props[j + 1]`: later entries overwrite earlier ones, NaN keys collapse to one key
    let parts: Vec<&str> = s.split('_').collect();
    let mut m = BTreeMap::new();
    let mut j = 0;
    while j + 1 < parts.len() {
        m.insert(parse_int(parts[j]).unwrap_or(i64::MIN), parts[j + 1].to_string());
        j += 2;
    }
    m
}

fn int_or(v: Option<&String>, d: &str) -> f64 {
    let s = match v { Some(s) if !s.is_empty() => s.as_str(), _ => d };
    parse_int(s).map_or(f64::NAN, |v| v as f64)
}

pub fn background_display_id(setting: Option<&str>) -> String {
    let parsed = parse_int(setting.unwrap_or("1"));
    let id = match parsed { Some(v) if v > 1 => v, _ => 1 };
    format!("{:02}", id)
}
pub fn ground_texture_id(setting: Option<&str>) -> String {
    let parsed = parse_int(setting.unwrap_or("1"));
    let idx = match parsed { Some(v) if v > 1 => v - 1, _ => 0 };
    format!("{:02}", idx)
}

impl LevelSettings {
    pub fn parse(settings: &str) -> LevelSettings {
        let mut st = LevelSettings::default();
        st.channel_blending.insert(1007, true);
        if settings.is_empty() { return st; }
        let pairs: Vec<&str> = settings.split(',').collect();
        let mut i = 0;
        while i + 1 < pairs.len() { st.map.insert(pairs[i].to_string(), pairs[i + 1].to_string()); i += 2; }
        st.background_id = Some(background_display_id(st.map.get("kA6").map(|s| s.as_str())));
        st.ground_id = Some(ground_texture_id(st.map.get("kA7").map(|s| s.as_str())));
        if let Some(color_str) = st.map.get("kS38").cloned().filter(|s| !s.is_empty()) {
            for ch in color_str.split('|') {
                if ch.is_empty() { continue; }
                let cp = prop_map(ch);
                let channel = parse_int(cp.get(&6).map(|s| s.as_str()).unwrap_or("")).map_or(f64::NAN, |v| v as f64);
                if channel == 1005.0 || channel == 1006.0 { st.channel_blending.insert(channel as i64, cp.get(&5).map(|s| s.as_str()) == Some("1")); continue; }
                if channel == 1007.0 { continue; }
                if !channel.is_nan() {
                    let id = channel as i64;
                    st.initial_colors.insert(id, Rgb { r: int_or(cp.get(&1), "255"), g: int_or(cp.get(&2), "255"), b: int_or(cp.get(&3), "255") });
                    st.channel_blending.insert(id, cp.get(&5).map(|s| s.as_str()) == Some("1"));
                    if let Some(op) = cp.get(&7).and_then(|s| parse_float(s)).filter(|v| v.is_finite() && *v != 1.0) { st.initial_opacity.insert(id, 0f64.max(1f64.min(op))); }
                    let src = int_or(cp.get(&9), "0");
                    if src > 0.0 && src != channel { st.initial_copies.insert(id, Copy { source: src, hsv: parse_channel_hsv(cp.get(&10).map(|s| s.as_str())) }); }
                }
            }
        }
        const LEGACY: [(&str, i64); 9] = [("kS29", 1000), ("kS30", 1001), ("kS31", 1002), ("kS32", 1004), ("kS33", 1), ("kS34", 2), ("kS35", 3), ("kS36", 4), ("kS37", 1003)];
        for (key, ch) in LEGACY {
            if st.initial_colors.contains_key(&ch) { continue; }
            let Some(v) = st.map.get(key).cloned().filter(|s| !s.is_empty()) else { continue };
            let cp = prop_map(&v);
            st.initial_colors.insert(ch, Rgb { r: int_or(cp.get(&1), "255"), g: int_or(cp.get(&2), "255"), b: int_or(cp.get(&3), "255") });
            st.channel_blending.insert(ch, cp.get(&5).map(|s| s.as_str()) == Some("1"));
        }
        st.legacy_color_level = st.map.get("kS38").map_or(true, |s| s.is_empty()) && st.map.get("kS29").map_or(false, |s| !s.is_empty());
        if st.legacy_color_level { st.channel_blending.insert(1002, true); }
        st
    }

    pub fn to_json(&self) -> String {
        use crate::json::num;
        let colors: Vec<String> = self.initial_colors.iter().map(|(k, c)| format!("\"{}\":[{},{},{}]", k, num(c.r), num(c.g), num(c.b))).collect();
        let copies: Vec<String> = self.initial_copies.iter().map(|(k, c)| format!("\"{}\":[{},{}]", k, num(c.source), c.hsv.map_or("null".to_string(), |h| format!("[{},{},{},{},{}]", num(h.h), num(h.s), num(h.v), h.s_add, h.v_add)))).collect();
        let ops: Vec<String> = self.initial_opacity.iter().map(|(k, v)| format!("\"{}\":{}", k, num(*v))).collect();
        let bl: Vec<String> = self.channel_blending.iter().map(|(k, v)| format!("\"{}\":{}", k, v)).collect();
        format!("{{\"colors\":{{{}}},\"copies\":{{{}}},\"opacity\":{{{}}},\"blending\":{{{}}},\"legacy\":{},\"bg\":{},\"ground\":{}}}",
            colors.join(","), copies.join(","), ops.join(","), bl.join(","), self.legacy_color_level,
            self.background_id.as_ref().map_or("null".to_string(), |s| format!("\"{}\"", s)), self.ground_id.as_ref().map_or("null".to_string(), |s| format!("\"{}\"", s)))
    }
}
