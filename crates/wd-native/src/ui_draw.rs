//! The menu/HUD layer: draws the node buffer `Ui::build` produces (the port of `drawUi` and `drawBitmapText` of the web host).
use crate::assets::{load_png, Assets, Image};
use crate::canvas::{Canvas, Mat, Paint, H, W};
use crate::fonts::Font;
use std::collections::HashMap;
use wd_core as core;

pub struct UiDraw {
    pub fonts: Vec<Option<Font>>, // index = the node's font id: 0 none, 1 bigFont, 2 goldFont
    standalone: HashMap<String, Option<Image>>,
    table: Vec<(String, String)>,
    table_len: usize,
    pub cam: (f64, f64),
    pv_ptr: *const crate::player_vis::PlayerVis,
}

fn text_result(n: usize) -> String {
    let p = core::wd_result_ptr();
    unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() }
}

impl UiDraw {
    pub fn new(root: &std::path::Path) -> Result<UiDraw, String> {
        let dir = root.join("fonts");
        let big = Font::load(&dir, "bigFont")?;
        let gold = Font::load(&dir, "goldFont")?;
        for f in [&big, &gold] { let name = if std::ptr::eq(f, &big) { "bigFont" } else { "goldFont" }; unsafe { core::wd_ui_font(name.as_ptr(), name.len(), f.source.as_ptr(), f.source.len()); } }
        Ok(UiDraw { fonts: vec![None, Some(big), Some(gold)], standalone: HashMap::new(), table: Vec::new(), table_len: usize::MAX, cam: (0.0, 0.0), pv_ptr: std::ptr::null() })
    }

    /// The loose images (`standalone[...]` of the web host): `sprites/<key>.png`, or `<key>` with `_` before its last digits (`GJ_button04` -> `GJ_button_04`).
    pub fn set_standalone(&mut self, key: &str, img: Option<Image>) { self.standalone.insert(key.to_string(), img); }
    fn standalone(&mut self, root: &std::path::Path, key: &str) -> Option<&Image> {
        if !self.standalone.contains_key(key) {
            let mut cands = vec![format!("{key}.png")];
            let digits = key.chars().rev().take_while(|c| c.is_ascii_digit()).count();
            if digits >= 2 && key.len() > digits { let (a, b) = key.split_at(key.len() - 2); cands.push(format!("{a}_{b}.png")); }
            let img = cands.iter().find_map(|c| load_png(&root.join("sprites").join(c)).ok());
            self.standalone.insert(key.to_string(), img);
        }
        self.standalone.get(key).and_then(|i| i.as_ref())
    }

    /// `drawBitmapText`: lines centred on the widest one, origin (ox, oy) as fractions of the block.
    pub fn bitmap_text(&self, c: &mut Canvas, font: usize, s: &str, size: f64, x: f64, y: f64, scale: f64, ox: f64, oy: f64, color: i64, alpha: f64) {
        let Some(Some(f)) = self.fonts.get(font) else { return };
        let k = size / f.size;
        struct Line { glyphs: Vec<(crate::fonts::Glyph, f64)>, width: f64 }
        let lines: Vec<Line> = s.split('\n').map(|line| {
            let (mut cur, mut prev): (f64, Option<u32>) = (0.0, None);
            let mut glyphs = Vec::new();
            for ch in line.chars() {
                let id = ch as u32;
                if let Some(p) = prev { cur += f.kern.get(&(p, id)).copied().unwrap_or(0.0); }
                if let Some(g) = f.chars.get(&id) { glyphs.push((*g, cur)); cur += g.adv; }
                prev = Some(id);
            }
            Line { glyphs, width: cur * k }
        }).collect();
        let width = lines.iter().map(|l| l.width).fold(0.0, f64::max);
        let line_h = 67.0 * k;
        let m = Mat::id().translate(x, y).scale(scale, scale).translate(-width * ox, -line_h * lines.len() as f64 * oy);
        let paint = Paint { alpha, tint: color, additive: false, gray: false };
        for (li, ln) in lines.iter().enumerate() {
            let off = (width - ln.width) / 2.0;
            let top = li as f64 * line_h;
            for (g, gx) in &ln.glyphs {
                if g.w == 0.0 { continue; }
                let (dx, dy) = (off + (gx + g.xo) * k, top + g.yo * k);
                c.draw_region(&f.img, (g.x, g.y, g.w, g.h), (dx, dy, g.w * k, g.h * k), m, paint, (g.x as usize, g.y as usize, g.w as usize, g.h as usize));
            }
        }
    }

    pub fn draw(&mut self, c: &mut Canvas, assets: &Assets, pv: &crate::player_vis::PlayerVis) {
        self.pv_ptr = pv as *const _;
        let n = core::wd_ui_build(0);
        if n == 0 { return; }
        let ub: Vec<f64> = unsafe { std::slice::from_raw_parts(core::wd_ui_ptr(), n * 16) }.to_vec();
        let texts: Vec<String> = match core::jsonread::parse(&text_result(core::wd_ui_texts())) {
            Ok(j) => j.arr().unwrap_or(&[]).iter().map(|t| t.str().unwrap_or("").to_string()).collect(), Err(_) => Vec::new() };
        let fl = core::wd_ui_frames_len();
        if fl != self.table_len {
            self.table = core::jsonread::parse(&text_result(core::wd_ui_frames())).ok().map(|j| j.arr().unwrap_or(&[]).iter().map(|e| {
                let a = e.arr().unwrap_or(&[]); (a.first().and_then(|x| x.str()).unwrap_or("").to_string(), a.get(1).and_then(|x| x.str()).unwrap_or("").to_string())
            }).collect()).unwrap_or_default();
            self.table_len = fl;
        }
        let mut clips: Vec<(i64, i64, i64, i64)> = Vec::new();
        let full = (0, 0, W as i64, H as i64);
        for k in 0..n {
            let o = k * 16;
            let kind = ub[o] as i32;
            if kind == 6 {
                let cur = c.clip;
                clips.push(cur);
                let (x0, y0) = (ub[o + 1].floor() as i64, ub[o + 2].floor() as i64);
                let (x1, y1) = ((ub[o + 1] + ub[o + 12]).ceil() as i64, (ub[o + 2] + ub[o + 13]).ceil() as i64);
                c.clip = (cur.0.max(x0), cur.1.max(y0), cur.2.min(x1), cur.3.min(y1));
                continue;
            }
            if kind == 7 { c.clip = clips.pop().unwrap_or(full); continue; }
            let alpha = ub[o + 6].clamp(0.0, 1.0);
            let flags = ub[o + 8] as i64;
            let additive = kind != 0 && flags & 4 != 0;
            let cam = flags & 8 != 0;
            match kind {
                0 => self.draw_image(c, assets, &ub[o..o + 16], &texts, alpha, flags),
                1 => {
                    let t = texts.get(ub[o + 11] as usize).cloned().unwrap_or_default();
                    let (cx, cy) = (if cam { -self.cam.0 } else { 0.0 }, if cam { self.cam.1 } else { 0.0 });
                    self.bitmap_text(c, ub[o + 15] as usize, &t, ub[o + 12], ub[o + 1] + cx, ub[o + 2] + cy, ub[o + 3], ub[o + 9], ub[o + 10], ub[o + 14] as i64, alpha);
                }
                2 => c.fill_round(ub[o + 1], ub[o + 2], ub[o + 12], ub[o + 13], 0.0, ub[o + 14] as u32, alpha, Mat::id()),
                3 => c.fill_round(ub[o + 1], ub[o + 2], ub[o + 12], ub[o + 13], ub[o + 15], ub[o + 14] as u32, alpha, Mat::id()),
                4 => c.fill_circle(ub[o + 1], ub[o + 2], ub[o + 12], ub[o + 14] as u32, alpha, Mat::id()),
                5 => self.draw_nine(c, assets, &ub[o..o + 16], &texts, alpha, flags, additive),
                _ => {}
            }
        }
        c.clip = full;
    }

    /// The image behind a frame: (image, frame rectangle, untrimmed size, trim offset, atlas?)
    fn lookup<'a>(&'a mut self, assets: &'a Assets, a: &str, f: &str) -> Option<(&'a Image, (f64, f64, f64, f64), (f64, f64), (f64, f64), bool)> {
        if a.is_empty() {
            let root = assets.root.clone();
            let img = self.standalone(&root, f)?;
            let (w, h) = (img.w as f64, img.h as f64);
            return Some((img, (0.0, 0.0, w, h), (w, h), (0.0, 0.0), false));
        }
        if let Some(base) = a.strip_prefix("icon_") {
            // an icon atlas of the player (garage grid)
            let pv = unsafe { self.pv_ptr.as_ref()? };
            let sh = pv.icon_loaded(base)?;
            let fr = sh.frames.get(f)?;
            return Some((&sh.img, (fr.x, fr.y, fr.w, fr.h), (fr.src_w, fr.src_h), (fr.ss_x, fr.ss_y), true));
        }
        let sh = assets.sheets.get(a)?;
        let fr = sh.frames.get(f)?;
        Some((&sh.img, (fr.x, fr.y, fr.w, fr.h), (fr.src_w, fr.src_h), (fr.ss_x, fr.ss_y), true))
    }

    fn draw_image(&mut self, c: &mut Canvas, assets: &Assets, nd: &[f64], _texts: &[String], alpha: f64, flags: i64) {
        let Some((a, f)) = self.table.get(nd[13] as usize).cloned() else { return };
        let (cam_x, cam_y) = self.cam;
        let Some((img, (fx, fy, fw, fh), (sw2, sh2), (ssx, ssy), atlas)) = self.lookup(assets, &a, &f) else { return };
        let (ox, oy) = (nd[9], nd[10]);
        let mut m = Mat::id().translate(nd[1] + if flags & 8 != 0 { -cam_x } else { 0.0 }, nd[2] + if flags & 8 != 0 { cam_y } else { 0.0 });
        if nd[5] != 0.0 { m = m.rotate(nd[5]); }
        m = m.scale(nd[3], nd[4]);
        if flags & 1 != 0 { m = m.translate(sw2 * (1.0 - 2.0 * ox), 0.0).scale(-1.0, 1.0); }
        if flags & 2 != 0 { m = m.translate(0.0, sh2 * (1.0 - 2.0 * oy)).scale(1.0, -1.0); }
        let (dx, dy) = (-sw2 * ox + ssx, -sh2 * oy + ssy);
        let (tint, crop) = (nd[7] as i64, nd[12]);
        let cw = if crop > 0.0 { crop.min(fw) } else { fw };
        let paint = Paint { alpha, tint, additive: flags & 4 != 0, gray: flags & 16 != 0 };
        if crop > fw && !atlas {
            let mut tx = 0.0;
            while tx < crop { let pw = fw.min(crop - tx); c.draw_region(img, (0.0, 0.0, pw, fh), (dx + tx, dy, pw, fh), m, paint, (0, 0, img.w, img.h)); tx += fw; }
        } else {
            let ins = if atlas && cw == fw && (fw < img.w as f64 || fh < img.h as f64) { 0.5 } else { 0.0 };
            c.draw_region(img, (fx + ins, fy + ins, cw - 2.0 * ins, fh - 2.0 * ins), (dx, dy, cw, fh), m, paint, (fx as usize, fy as usize, cw.ceil() as usize, fh as usize));
        }
    }

    fn draw_nine(&mut self, c: &mut Canvas, assets: &Assets, nd: &[f64], texts: &[String], alpha: f64, flags: i64, additive: bool) {
        let key = texts.get(nd[11] as usize).cloned().unwrap_or_default();
        let (a, f) = match key.find('/') { Some(i) if i > 0 => (key[..i].to_string(), key[i + 1..].to_string()), _ => (String::new(), key.clone()) };
        let Some((img, (fx, fy, fw, fh), (uw, uh), (tx0, ty0), atlas)) = self.lookup(assets, &a, &f) else { return };
        let (x0, y0, w2, h2, b) = (nd[1], nd[2], nd[12], nd[13], nd[15]);
        let paint = Paint { alpha, tint: nd[14] as i64, additive, gray: flags & 16 != 0 };
        let us = [0.0, b, uw - b, uw];
        let vs = [0.0, b, uh - b, uh];
        let nine_ins = atlas && (fw < img.w as f64 || fh < img.h as f64);
        let xd = [x0, x0 + b, x0 + w2 - b, x0 + w2];
        let yd = [y0, y0 + b, y0 + h2 - b, y0 + h2];
        for i in 0..3 { for j in 0..3 {
            let iu0 = us[i].max(tx0); let iu1 = us[i + 1].min(tx0 + fw);
            let iv0 = vs[j].max(ty0); let iv1 = vs[j + 1].min(ty0 + fh);
            if iu1 <= iu0 || iv1 <= iv0 { continue; }
            let dl = if nine_ins && i == 0 && iu0 == tx0 { 0.5 } else { 0.0 };
            let dr = if nine_ins && i == 2 && iu1 == tx0 + fw { 0.5 } else { 0.0 };
            let dt = if nine_ins && j == 0 && iv0 == ty0 { 0.5 } else { 0.0 };
            let db = if nine_ins && j == 2 && iv1 == ty0 + fh { 0.5 } else { 0.0 };
            let kx = (xd[i + 1] - xd[i]) / (us[i + 1] - us[i]);
            let ky = (yd[j + 1] - yd[j]) / (vs[j + 1] - vs[j]);
            let ex = if i < 2 && iu1 == us[i + 1] { 0.4 } else { 0.0 };
            let ey = if j < 2 && iv1 == vs[j + 1] { 0.4 } else { 0.0 };
            c.draw_region(img,
                (fx + (iu0 - tx0) + dl, fy + (iv0 - ty0) + dt, iu1 - iu0 - dl - dr, iv1 - iv0 - dt - db),
                (xd[i] + (iu0 - us[i]) * kx, yd[j] + (iv0 - vs[j]) * ky, (iu1 - iu0) * kx + ex, (iv1 - iv0) * ky + ey),
                Mat::id(), paint, (fx as usize, fy as usize, fw as usize, fh as usize));
        } }
    }
}
