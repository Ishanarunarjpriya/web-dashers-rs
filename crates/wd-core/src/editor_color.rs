//! The editor colour picker (level-editor.js `_openEditorColorPickerPopup`): a hue ring and a colour disc whose colours come from the
//! pixels of the picker artwork (the host sends them once: `wd_ed_cp_tex`), R / G / B / HEX fields, the current and the original colour.
use crate::ui::*;

const D_CP: f64 = 2700.0;
const SCALE: f64 = 0.42;
const RING_IN: f64 = 94.0;
const RING_OUT: f64 = 126.0;
const INNER_R: f64 = 220.0 * SCALE - 1.0;
/// side of the sampled images: 600 * 0.42 and 440 * 0.42, rounded up
pub const HUE_SIZE: usize = 252;
pub const DISC_SIZE: usize = 185;
const ATLAS: &str = "CCControlColourPickerSpriteSheet-uhd";

#[derive(Default, Clone)]
pub struct CpOpts { pub initial: Option<(i64, i64, i64)>, pub commit: bool, pub panel_w: f64, pub panel_h: f64, pub content_y: f64, pub picker_y: Option<f64>, pub title: String }

#[derive(Default)]
pub struct EdCp {
    pub open: bool, pub channel: i64, pub opts: CpOpts,
    pub h: f64, pub inner: (f64, f64), pub hue: (f64, f64), pub color: (i64, i64, i64), pub orig: (i64, i64, i64),
    pub values: [String; 4], pub focus: Option<usize>, pub drag: u8,
    pub nodes: CpNodes,
    /// RGBA pixels of huePickerBackground (252 x 252), colourPickerBackground, colourPickerShadow, colourPickerOverlay (185 x 185)
    pub tex: [Vec<u8>; 4],
}
#[derive(Default)]
pub struct CpNodes { pub fill: usize, pub hue_h: usize, pub sv_h: usize, pub cur: usize, pub vals: [usize; 4], pub bgs: [usize; 4] }

fn clamp(v: f64, lo: f64, hi: f64) -> f64 { v.max(lo).min(hi) }
fn hex_str(c: (i64, i64, i64)) -> String { format!("{:02X}{:02X}{:02X}", c.0, c.1, c.2) }
fn norm(c: (i64, i64, i64)) -> (i64, i64, i64) { (c.0.max(0).min(255), c.1.max(0).min(255), c.2.max(0).min(255)) }
fn hue_dist(a: f64, b: f64) -> f64 { let d = (a - b).abs() % 360.0; if d > 180.0 { 360.0 - d } else { d } }
/// `_rgbToHsv`: hue in degrees, saturation and value 0..1
fn hsv_of(c: (i64, i64, i64)) -> (f64, f64, f64) {
    let (r, g, b) = (c.0 as f64 / 255.0, c.1 as f64 / 255.0, c.2 as f64 / 255.0);
    let max = r.max(g).max(b); let min = r.min(g).min(b); let d = max - min;
    let mut h = 0.0;
    if d != 0.0 {
        h = if max == r { ((g - b) / d) % 6.0 } else if max == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 };
        h *= 60.0;
        if h < 0.0 { h += 360.0; }
    }
    (h, if max == 0.0 { 0.0 } else { d / max }, max)
}
/// `_hsvToRgb` for a full-saturation, full-value hue, rounded to bytes
struct Rgbf { r: f64, g: f64, b: f64 }
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Rgbf {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let c = v * s; let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs()); let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) } else if h < 120.0 { (x, c, 0.0) } else if h < 180.0 { (0.0, c, x) } else if h < 240.0 { (0.0, x, c) } else if h < 300.0 { (x, 0.0, c) } else { (c, 0.0, x) };
    Rgbf { r: ((r + m) * 255.0).round(), g: ((g + m) * 255.0).round(), b: ((b + m) * 255.0).round() }
}

impl EdCp {
    fn px(&self, which: usize, size: usize, x: i64, y: i64) -> [f64; 4] {
        let t = &self.tex[which];
        if x < 0 || y < 0 || x as usize >= size || y as usize >= size || t.len() < size * size * 4 { return [0.0; 4]; }
        let o = (y as usize * size + x as usize) * 4;
        [t[o] as f64, t[o + 1] as f64, t[o + 2] as f64, t[o + 3] as f64 / 255.0]
    }
    fn have_tex(&self) -> bool { self.tex[0].len() >= HUE_SIZE * HUE_SIZE * 4 && self.tex[1].len() >= DISC_SIZE * DISC_SIZE * 4 }

    /// the canvas of `rebuildInnerSampleCanvas`: the background, the pure hue disc, the shadow, the overlay (source-over, in this order)
    fn disc_px(&self, hue_rgb: (f64, f64, f64), x: i64, y: i64) -> [f64; 4] {
        let half = DISC_SIZE as f64 / 2.0;
        let over = |dst: [f64; 4], src: [f64; 4]| -> [f64; 4] {
            let a = src[3] + dst[3] * (1.0 - src[3]);
            if a <= 0.0 { return [0.0; 4]; }
            let c = |i: usize| (src[i] * src[3] + dst[i] * dst[3] * (1.0 - src[3])) / a;
            [c(0), c(1), c(2), a]
        };
        let mut p = self.px(1, DISC_SIZE, x, y);
        let (dx, dy) = (x as f64 + 0.5 - half, y as f64 + 0.5 - half);
        if dx * dx + dy * dy <= half * half { p = over(p, [hue_rgb.0, hue_rgb.1, hue_rgb.2, 1.0]); }
        p = over(p, self.px(2, DISC_SIZE, x, y));
        over(p, self.px(3, DISC_SIZE, x, y))
    }
}

impl Ui {
    pub fn ed_cp_set_tex(&mut self, which: usize, data: Vec<u8>) { if which < 4 { self.ed.cp.tex[which] = data; } }

    fn ed_cp_rgb8(p: [f64; 4]) -> (i64, i64, i64) { (p[0].round() as i64, p[1].round() as i64, p[2].round() as i64) }

    /// `sampleHuePixel`: the colour of the ring at the hue handle; its hue becomes the disc's when it is saturated enough
    fn ed_cp_sample_hue(&mut self) -> (i64, i64, i64) {
        let half = HUE_SIZE as f64 / 2.0;
        let cx = clamp((half + self.ed.cp.hue.0).round(), 0.0, HUE_SIZE as f64 - 1.0) as i64;
        let cy = clamp((half + self.ed.cp.hue.1).round(), 0.0, HUE_SIZE as f64 - 1.0) as i64;
        let c = Self::ed_cp_rgb8(self.ed.cp.px(0, HUE_SIZE, cx, cy));
        let (h, s, _) = hsv_of(c);
        if s > 0.08 { self.ed.cp.h = h; }
        c
    }

    /// `sampleInnerPixel`
    fn ed_cp_sample_inner(&mut self) -> (i64, i64, i64) {
        let half = DISC_SIZE as f64 / 2.0;
        let hue = hsv_to_rgb(self.ed.cp.h, 1.0, 1.0);
        let cx = clamp((half + self.ed.cp.inner.0).round(), 1.0, DISC_SIZE as f64 - 2.0) as i64;
        let cy = clamp((half + self.ed.cp.inner.1).round(), 1.0, DISC_SIZE as f64 - 2.0) as i64;
        let p = self.ed.cp.disc_px((hue.r, hue.g, hue.b), cx, cy);
        if p[3] * 255.0 < 12.0 { return self.ed.cp.color; }
        Self::ed_cp_rgb8(p)
    }

    /// `findHuePositionForLoadedColor`
    fn ed_cp_find_hue(&self, target: (i64, i64, i64)) -> (f64, f64) {
        let th = hsv_of(target).0;
        let mid = (RING_IN + RING_OUT) / 2.0;
        let a = (th - 90.0).to_radians();
        let fallback = (a.cos() * mid, a.sin() * mid);
        if !self.ed.cp.have_tex() { return fallback; }
        let half = HUE_SIZE as f64 / 2.0;
        let mut best: Option<(f64, f64, f64)> = None;
        let mut y = 0usize;
        while y < HUE_SIZE {
            let mut x = 0usize;
            while x < HUE_SIZE {
                let (dx, dy) = (x as f64 - half, y as f64 - half);
                let dist = (dx * dx + dy * dy).sqrt();
                if dist >= RING_IN && dist <= RING_OUT {
                    let p = self.ed.cp.px(0, HUE_SIZE, x as i64, y as i64);
                    if p[3] * 255.0 >= 12.0 {
                        let (h, s, _) = hsv_of(Self::ed_cp_rgb8(p));
                        if s >= 0.08 {
                            let score = hue_dist(h, th) + (dist - mid).abs() * 0.08;
                            if best.map_or(true, |b| score < b.2) { best = Some((dx, dy, score)); }
                        }
                    }
                }
                x += 2;
            }
            y += 2;
        }
        best.map_or(fallback, |b| (b.0, b.1))
    }

    /// `findInnerPositionForLoadedColor`
    fn ed_cp_find_inner(&self, target: (i64, i64, i64)) -> (f64, f64) {
        if !self.ed.cp.have_tex() { return (0.0, 0.0); }
        let half = DISC_SIZE as f64 / 2.0;
        let hue = hsv_to_rgb(self.ed.cp.h, 1.0, 1.0);
        let mut best: Option<(f64, f64, f64)> = None;
        for y in 1..DISC_SIZE - 1 {
            for x in 1..DISC_SIZE - 1 {
                let (dx, dy) = (x as f64 - half, y as f64 - half);
                if (dx * dx + dy * dy).sqrt() > INNER_R { continue; }
                let p = self.ed.cp.disc_px((hue.r, hue.g, hue.b), x as i64, y as i64);
                if p[3] * 255.0 < 12.0 { continue; }
                let c = Self::ed_cp_rgb8(p);
                let score = ((c.0 - target.0).pow(2) + (c.1 - target.1).pow(2) + (c.2 - target.2).pow(2)) as f64;
                if best.map_or(true, |b| score < b.2) { best = Some((dx, dy, score)); }
            }
        }
        best.map_or((0.0, 0.0), |b| (b.0, b.1))
    }

    /// `setPickerPositionForColor`
    fn ed_cp_place(&mut self, c: (i64, i64, i64)) {
        let c = norm(c);
        self.ed.cp.hue = self.ed_cp_find_hue(c);
        self.ed_cp_sample_hue();
        self.ed.cp.inner = self.ed_cp_find_inner(c);
    }

    pub fn ed_cp_open(&mut self, channel: i64) {
        self.ed_cp_open_with(channel, CpOpts { commit: true, panel_w: 820.0, panel_h: 520.0, title: "Select Color".into(), ..Default::default() });
    }

    pub fn ed_cp_open_with(&mut self, channel: i64, mut opts: CpOpts) {
        if self.ed.cp.open { self.ed_cp_close(); }
        if opts.panel_w <= 0.0 { opts.panel_w = 820.0; }
        if opts.panel_h <= 0.0 { opts.panel_h = 520.0; }
        let cur = norm(opts.initial.unwrap_or_else(|| self.ed_channel_color(channel)));
        let (pw, ph) = (opts.panel_w, opts.panel_h);
        let (picker_y, cy) = (opts.picker_y.unwrap_or(opts.content_y), opts.content_y);
        let hsv = hsv_of(cur);
        let a = (hsv.0 - 90.0).to_radians();
        let tex = std::mem::take(&mut self.ed.cp.tex);
        self.ed.cp = EdCp { open: true, channel, opts: opts.clone(), h: hsv.0, hue: (a.cos() * 110.0, a.sin() * 110.0), color: cur, orig: cur, values: [cur.0.to_string(), cur.1.to_string(), cur.2.to_string(), hex_str(cur)], tex, ..Default::default() };
        let root = self.ed_pop("ed_cp", D_CP, 0.32, 0.0, Action::Pop("EdCpBlur", 0));
        let d = D_CP + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_cp");
        let title = if opts.title.is_empty() { "Select Color" } else { &opts.title };
        self.pa_txt(root, "goldFont", 34.0, title, 0.0, -(ph / 2.0) + 40.0, 0.5, 0.5, 1.0, d, "ed_cp");
        let bg = self.pa_img(root, ATLAS, "menuColourPanelBackground.png", 0.0, picker_y, SCALE, d, "ed_cp"); self.node(bg).flip_x = true;
        self.pa_img(root, ATLAS, "huePickerBackground.png", 0.0, picker_y, SCALE, d + 0.01, "ed_cp");
        self.pa_img(root, ATLAS, "colourPickerBackground.png", 0.0, picker_y, SCALE, d + 0.02, "ed_cp");
        let fill = self.shape(Kind::Circle, 0.0, picker_y, INNER_R, INNER_R, 0xffffff, 1.0, d + 0.03, "ed_cp"); self.node(fill).parent = Some(root);
        self.pa_img(root, ATLAS, "colourPickerShadow.png", 0.0, picker_y, SCALE, d + 0.04, "ed_cp");
        self.pa_img(root, ATLAS, "colourPickerOverlay.png", 0.0, picker_y, SCALE, d + 0.05, "ed_cp");
        let hue_h = self.pa_img(root, ATLAS, "colourPicker.png", 0.0, picker_y, SCALE, d + 0.06, "ed_cp");
        let sv_h = self.pa_img(root, ATLAS, "colourPicker.png", 0.0, picker_y, SCALE, d + 0.07, "ed_cp");
        // the picker area takes presses and drags
        let hit = self.shape(Kind::Rect, 0.0, picker_y, (RING_OUT + 20.0) * 2.0, (RING_OUT + 20.0) * 2.0, 0xffffff, 0.001, d + 0.08, "ed_cp");
        { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdCpDown", 0); n.width = (RING_OUT + 20.0) * 2.0; n.height = (RING_OUT + 20.0) * 2.0; }
        // current / original colour
        let (pxx, pyy, ps) = (-(pw / 2.0) + 46.0, -(ph / 2.0) + 43.0, 42.0);
        let cur_r = self.shape(Kind::Rect, pxx - ps / 2.0, pyy - ps / 2.0, ps, ps, ((cur.0 << 16) | (cur.1 << 8) | cur.2) as i64, 1.0, d + 0.03, "ed_cp"); self.node(cur_r).parent = Some(root);
        let org = self.shape(Kind::Rect, pxx - ps / 2.0, pyy + ps - ps / 2.0, ps, ps, ((cur.0 << 16) | (cur.1 << 8) | cur.2) as i64, 1.0, d + 0.03, "ed_cp"); self.node(org).parent = Some(root);
        // the input fields
        let (lx, ry, gap) = (-(pw / 2.0) + 60.0, -34.0 + cy, 76.0);
        let mut vals = [0usize; 4]; let mut bgs = [0usize; 4];
        for (k, (label, x, y, w)) in [("R:", lx, ry, 68.0), ("G:", lx + gap, ry, 68.0), ("B:", lx + gap * 2.0, ry, 68.0), ("HEX:", lx + gap, 50.0 + cy, 152.0)].iter().enumerate() {
            let t = self.pa_txt(root, "goldFont", 22.0, label, *x, y - 37.0, 0.5, 0.5, 1.0, d + 0.03, "ed_cp"); self.node(t).alpha = 0.95;
            let b = self.shape(Kind::Rect, x - w / 2.0, y - 17.0, *w, 34.0, 0, 0.38, d + 0.03, "ed_cp");
            { let n = self.node(b); n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdCpField", k); n.width = *w; n.height = 34.0; }
            vals[k] = self.pa_txt(root, "bigFont", 20.0, &self.ed.cp.values[k].clone(), *x, *y, 0.5, 0.5, 1.0, d + 0.04, "ed_cp");
            bgs[k] = b;
        }
        self.ed.cp.nodes = CpNodes { fill, hue_h, sv_h, cur: cur_r, vals, bgs };
        self.ed_ok(root, 0.0, (ph / 2.0) - 55.0, "OK", Action::Pop("EdCpOk", 0), d, "ed_cp");
        if self.ed.cp.have_tex() { self.ed_cp_place(cur); }
        self.ed.cp.color = cur;
        self.ed_cp_refresh(false);
    }

    pub fn ed_cp_close(&mut self) { self.kill_tag("ed_cp"); self.ed.cp.open = false; self.ed.cp.focus = None; self.ed.cp.drag = 0; }

    /// `updateColorUi`: handles, fill, preview and field texts follow the colour; the channel and the host hear about a change
    fn ed_cp_refresh(&mut self, changed: bool) {
        let n = self.ed.cp.nodes.fill;
        let hue = hsv_to_rgb(self.ed.cp.h, 1.0, 1.0);
        let c = self.ed.cp.color;
        let py = self.ed.cp.opts.picker_y.unwrap_or(self.ed.cp.opts.content_y);
        let (hh, sh, cur) = (self.ed.cp.nodes.hue_h, self.ed.cp.nodes.sv_h, self.ed.cp.nodes.cur);
        let (hx, hy, ix, iy) = (self.ed.cp.hue.0, self.ed.cp.hue.1, self.ed.cp.inner.0, self.ed.cp.inner.1);
        let col = |r: i64, g: i64, b: i64| (r << 16) | (g << 8) | b;
        if let Some(x) = self.nodes.get_mut(n).and_then(|x| x.as_mut()) { x.color = col(hue.r.round() as i64, hue.g.round() as i64, hue.b.round() as i64); }
        if let Some(x) = self.nodes.get_mut(hh).and_then(|x| x.as_mut()) { x.x = hx; x.y = py + hy; }
        if let Some(x) = self.nodes.get_mut(sh).and_then(|x| x.as_mut()) { x.x = ix; x.y = py + iy; }
        if let Some(x) = self.nodes.get_mut(cur).and_then(|x| x.as_mut()) { x.color = col(c.0, c.1, c.2); }
        self.ed_cp_fields();
        if changed {
            if self.ed.cp.opts.commit { self.ed_set_channel(self.ed.cp.channel, c); }
            self.events.push(Action::Pop("EdCpChanged", 0));
        }
    }

    /// the texts and backgrounds of the four fields
    fn ed_cp_fields(&mut self) {
        let (vals, bgs) = (self.ed.cp.nodes.vals, self.ed.cp.nodes.bgs);
        for k in 0..4 {
            let focused = self.ed.cp.focus == Some(k);
            let text = format!("{}{}", self.ed.cp.values[k], if focused { "|" } else { "" });
            if let Some(x) = self.nodes.get_mut(vals[k]).and_then(|x| x.as_mut()) { x.text = text; }
            if let Some(x) = self.nodes.get_mut(bgs[k]).and_then(|x| x.as_mut()) { x.alpha = if focused { 0.56 } else { 0.38 }; }
        }
    }

    /// `_setEditorColorChannelDraft`: the settings string, the colour manager and with them the view
    pub(crate) fn ed_set_channel(&mut self, ch: i64, c: (i64, i64, i64)) {
        self.ed.level.set_color_channel(ch, c);
        crate::with_game(|g| { g.cm.set_initial_color(ch, crate::colors::Rgb { r: c.0 as f64, g: c.1 as f64, b: c.2 as f64 }); g.fx.color_dirty = true; });
    }

    fn ed_cp_set_color(&mut self, c: (i64, i64, i64), set_values: bool) {
        let c = norm(c);
        self.ed.cp.color = c;
        if set_values { self.ed.cp.values = [c.0.to_string(), c.1.to_string(), c.2.to_string(), hex_str(c)]; }
        self.ed_cp_refresh(true);
    }

    /// presses and drags on the picker (`beginPickerDrag`, `continuePickerDrag`)
    pub fn ed_cp_pointer(&mut self, x: f64, y: f64, phase: u8) {
        if !self.ed.cp.open { return; }
        let py = self.ed.cp.opts.picker_y.unwrap_or(self.ed.cp.opts.content_y);
        let (dx, dy) = (x - self.screen_width / 2.0, y - 320.0 - py);
        let dist = (dx * dx + dy * dy).sqrt();
        if phase == 0 {
            self.ed.cp.drag = if dist <= INNER_R { 1 } else if dist >= RING_IN && dist <= RING_OUT { 2 } else { 0 };
        } else if phase == 2 { self.ed.cp.drag = 0; return; }
        match self.ed.cp.drag {
            1 => {
                self.ed.cp.focus = None;
                let (mut px, mut py2) = (dx, dy);
                if dist > INNER_R && dist > 0.0001 { px = dx / dist * INNER_R; py2 = dy / dist * INNER_R; }
                self.ed.cp.inner = (px, py2);
                let c = self.ed_cp_sample_inner();
                self.ed_cp_set_color(c, true);
            }
            2 => {
                self.ed.cp.focus = None;
                let (mut ddx, mut ddy, mut d) = (dx, dy, dist);
                if d < 0.0001 { ddx = 0.0; ddy = -RING_IN; d = RING_IN; }
                let cd = clamp(d, RING_IN, RING_OUT);
                self.ed.cp.hue = (ddx / d * cd, ddy / d * cd);
                self.ed_cp_sample_hue();
                let c = self.ed_cp_sample_inner();
                self.ed_cp_set_color(c, true);
            }
            _ => {}
        }
    }

    pub fn ed_cp_key(&mut self, code: u32) -> bool {
        let Some(k) = self.ed.cp.focus else { return false };
        let c = char::from_u32(code).unwrap_or('\0');
        match code {
            13 => { self.ed.cp.focus = None; self.ed_cp_fields(); return true; }
            8 => { self.ed.cp.values[k].pop(); }
            _ => {
                if k == 3 { if c.is_ascii_hexdigit() && self.ed.cp.values[3].len() < 6 { self.ed.cp.values[3].push(c.to_ascii_uppercase()); } else { return true; } }
                else if c.is_ascii_digit() && self.ed.cp.values[k].len() < 3 { self.ed.cp.values[k].push(c); } else { return true; }
            }
        }
        if k == 3 {
            let v = &self.ed.cp.values[3];
            if v.len() == 6 {
                let p = |i: usize| i64::from_str_radix(&v[i..i + 2], 16).unwrap_or(0);
                let c = (p(0), p(2), p(4));
                self.ed_cp_apply_typed(c, true);
            } else { self.ed_cp_fields(); }
        } else {
            let v = &self.ed.cp.values;
            if v[0].is_empty() || v[1].is_empty() || v[2].is_empty() { self.ed_cp_fields(); }
            else {
                let c = (v[0].parse().unwrap_or(0), v[1].parse().unwrap_or(0), v[2].parse().unwrap_or(0));
                self.ed_cp_apply_typed(c, false);
            }
        }
        true
    }

    /// `applyTypedColor`: the picker handles move to the colour; the field the user types in keeps its text
    fn ed_cp_apply_typed(&mut self, c: (i64, i64, i64), _hex: bool) {
        let c = norm(c);
        self.ed_cp_place(c);
        let keep = self.ed.cp.values.clone();
        self.ed.cp.color = c;
        self.ed.cp.values = [c.0.to_string(), c.1.to_string(), c.2.to_string(), hex_str(c)];
        // the focused field shows what was typed; the others show the normalised colour
        if let Some(k) = self.ed.cp.focus { self.ed.cp.values[k] = keep[k].clone(); }
        self.ed_cp_refresh(true);
    }

    pub fn ed_cp_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdCpOk" => { self.ed_cp_close(); true }
            "EdCpBlur" => { self.ed.cp.focus = None; self.ed_cp_fields(); true }
            "EdCpField" => { self.ed.cp.focus = Some(arg); self.ed_cp_fields(); true }
            "EdCpDown" | "EdCpChanged" => true,
            _ => false,
        }
    }
}
