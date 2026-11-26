//! Bitmap fonts (the `.fnt` files of `ref/assets/fonts`).
use crate::assets::{load_png, Image};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct Glyph { pub x: f64, pub y: f64, pub w: f64, pub h: f64, pub xo: f64, pub yo: f64, pub adv: f64 }

pub struct Font { pub img: Image, pub chars: HashMap<u32, Glyph>, pub kern: HashMap<(u32, u32), f64>, pub size: f64, pub source: String }

fn kv(line: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut rest = line;
    while let Some(eq) = rest.find('=') {
        let key = rest[..eq].rsplit(char::is_whitespace).next().unwrap_or("").to_string();
        let after = &rest[eq + 1..];
        let (val, next) = if let Some(stripped) = after.strip_prefix('"') { let end = stripped.find('"').unwrap_or(stripped.len()); (stripped[..end].to_string(), &stripped[(end + 1).min(stripped.len())..]) }
            else { let end = after.find(char::is_whitespace).unwrap_or(after.len()); (after[..end].to_string(), &after[end..]) };
        out.insert(key, val);
        rest = next;
    }
    out
}

impl Font {
    pub fn load(dir: &Path, name: &str) -> Result<Font, String> {
        let source = std::fs::read_to_string(dir.join(format!("{name}.fnt"))).map_err(|e| format!("{name}.fnt: {e}"))?;
        let img = load_png(&dir.join(format!("{name}.png")))?;
        let (mut chars, mut kern, mut size) = (HashMap::new(), HashMap::new(), 0.0);
        let num = |m: &HashMap<String, String>, k: &str| m.get(k).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        for line in source.lines() {
            let m = kv(line);
            if line.starts_with("info ") { size = num(&m, "size"); }
            else if line.starts_with("char ") { chars.insert(num(&m, "id") as u32, Glyph { x: num(&m, "x"), y: num(&m, "y"), w: num(&m, "width"), h: num(&m, "height"), xo: num(&m, "xoffset"), yo: num(&m, "yoffset"), adv: num(&m, "xadvance") }); }
            else if line.starts_with("kerning ") { kern.insert((num(&m, "first") as u32, num(&m, "second") as u32), num(&m, "amount")); }
        }
        Ok(Font { img, chars, kern, size, source })
    }
}
