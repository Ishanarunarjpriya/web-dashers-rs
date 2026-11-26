//! Image and atlas loading (the files of `ref/assets`).
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use wd_core::jsonread::{parse, Json};

/// RGBA8, straight alpha.
pub struct Image { pub w: usize, pub h: usize, pub px: Vec<[u8; 4]> }

pub fn load_png(path: &Path) -> Result<Image, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut dec = png::Decoder::new(std::io::BufReader::new(file));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("{}: {e}", path.display()))?;
    let (w, h) = (info.width as usize, info.height as usize);
    let n = match info.color_type { png::ColorType::Rgba => 4, png::ColorType::Rgb => 3, png::ColorType::GrayscaleAlpha => 2, _ => 1 };
    let mut px = Vec::with_capacity(w * h);
    for i in 0..w * h {
        let b = &buf[i * n..i * n + n];
        px.push(match n { 4 => [b[0], b[1], b[2], b[3]], 3 => [b[0], b[1], b[2], 255], 2 => [b[0], b[0], b[0], b[1]], _ => [b[0], b[0], b[0], 255] });
    }
    Ok(Image { w, h, px })
}

#[derive(Clone, Copy, Default)]
pub struct Frame { pub x: f64, pub y: f64, pub w: f64, pub h: f64, pub rotated: bool, pub src_w: f64, pub src_h: f64, pub ss_x: f64, pub ss_y: f64 }

pub struct Sheet { pub img: Image, pub frames: HashMap<String, Frame> }

pub struct Assets { pub root: PathBuf, pub sheets: HashMap<String, Sheet> }

pub const SHEETS: [&str; 12] = ["GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet03", "GJ_GameSheet04", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_WebSheet", "GJ_EditorSheet01", "GJ_GameSheetEditor", "PixelSheet_01", "GJ_ParticleSheet", "GJ_LaunchSheet"];

fn frames_of(j: &Json) -> HashMap<String, Frame> {
    let mut out = HashMap::new();
    let list = match j.get("frames") { Some(f) => Some(f), None => j.get("textures").and_then(|t| t.arr()).and_then(|a| a.first()).and_then(|t| t.get("frames")) };
    let mut add = |name: &str, d: &Json| {
        let fr = d.get("frame");
        let g = |k: &str| fr.and_then(|f| f.get_num(k)).unwrap_or(0.0);
        let (w, h) = (g("w"), g("h"));
        let src = d.get("sourceSize");
        let ss = d.get("spriteSourceSize");
        out.insert(name.to_string(), Frame {
            x: g("x"), y: g("y"), w, h, rotated: d.get_bool("rotated").unwrap_or(false),
            src_w: src.and_then(|s| s.get_num("w")).unwrap_or(w), src_h: src.and_then(|s| s.get_num("h")).unwrap_or(h),
            ss_x: ss.and_then(|s| s.get_num("x")).unwrap_or(0.0), ss_y: ss.and_then(|s| s.get_num("y")).unwrap_or(0.0),
        });
    };
    match list {
        Some(Json::Obj(entries)) => for (k, v) in entries { add(k, v); },
        Some(Json::Arr(items)) => for v in items { if let Some(n) = v.get_str("filename") { add(n, v); } },
        _ => {}
    }
    out
}

impl Assets {
    /// Looks for `assets/` next to the program, then `ref/assets` (a checkout of the repository).
    pub fn find(arg: Option<String>) -> Result<PathBuf, String> {
        let mut tries: Vec<PathBuf> = Vec::new();
        if let Some(a) = arg { tries.push(PathBuf::from(a)); }
        if let Ok(exe) = std::env::current_exe() { if let Some(d) = exe.parent() { tries.push(d.join("assets")); } }
        tries.push(PathBuf::from("assets"));
        tries.push(PathBuf::from("ref/assets"));
        for t in &tries { if t.join("sheets").is_dir() { return Ok(t.clone()); } }
        Err(format!("assets folder not found (looked in {:?}); pass --assets <folder>", tries))
    }

    pub fn load(root: PathBuf) -> Result<Assets, String> {
        let mut sheets = HashMap::new();
        for name in SHEETS {
            let json = std::fs::read_to_string(root.join("sheets").join(format!("{name}.json"))).map_err(|e| format!("{name}.json: {e}"))?;
            let img = load_png(&root.join("sheets").join(format!("{name}.png")))?;
            let j = parse(&json).map_err(|e| format!("{name}.json: {e}"))?;
            unsafe { wd_core::wd_atlas_add(name.as_ptr(), name.len(), json.as_ptr(), json.len()); }
            sheets.insert(name.to_string(), Sheet { img, frames: frames_of(&j) });
        }
        for k in 1..=5 {
            let name = format!("GJBeast0{k}");
            if let Ok(json) = std::fs::read_to_string(root.join("sheets").join(format!("{name}_AnimDesc.json"))) {
                unsafe { wd_core::wd_beast_desc_add(name.as_ptr(), name.len(), json.as_ptr(), json.len()); }
            }
        }
        Ok(Assets { root, sheets })
    }
}
