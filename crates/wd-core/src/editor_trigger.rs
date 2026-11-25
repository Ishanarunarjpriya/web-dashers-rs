//! The trigger popups: "Setup Move Command" and "Spawn Group" (level-editor.js `_openEditorMoveTriggerOptionsPopup`,
//! `_openEditorSpawnTriggerOptionsPopup`) with their shared widgets: a stepper (box with arrows), a slider with a typed value, and check boxes.
//! Every widget reads and writes one raw property of the selected trigger.
use crate::editor::*;
use crate::ui::*;

const D_OP: f64 = 2550.0;

/// one value of the selected trigger
pub struct Fld {
    pub key: &'static str, pub stepper: bool, pub dec: usize,
    /// clamp of the stored value, and the range of a typed value
    pub lo: f64, pub hi: f64, pub typed_lo: f64, pub typed_hi: f64,
    pub slider_lo: f64, pub slider_hi: f64,
    pub bg: usize, pub text: usize, pub thumb: usize, pub start_x: f64, pub width: f64, pub root_x: f64,
    pub value: String, pub before: String, pub max_len: usize,
}
#[derive(Default)]
pub struct EdTrig { pub tag: &'static str, pub flds: Vec<Fld>, pub focus: Option<usize>, pub checks: Vec<(usize, &'static str, Option<&'static str>)>, pub drag: Option<usize>, pub root: usize }

fn fmt(v: f64, dec: usize) -> String {
    let p = 10f64.powi(dec as i32);
    let r = (v * p).round() / p;
    js_num(if r == 0.0 { 0.0 } else { r })
}

impl Ui {
    fn ed_tr_obj(&self) -> Option<&EdObj> { self.ed.sel.first().and_then(|&i| self.ed.level.objects.get(i)) }

    /// the stored value of a field (target group: the first group id of the raw text)
    fn ed_tr_get(&self, key: &str, lo: f64, hi: f64) -> f64 {
        let raw = self.ed_tr_obj().and_then(|o| o.raw.get(key).cloned()).unwrap_or_default();
        let v = if key == "51" { raw.split(|c| c == ',' || c == '.').filter_map(|p| crate::js::parse_int(p.trim())).find(|v| *v > 0).unwrap_or(0) as f64 }
            else { crate::js::parse_float(&raw).filter(|v| v.is_finite()).unwrap_or(0.0) };
        v.max(lo).min(hi)
    }

    fn ed_tr_set(&mut self, key: &'static str, v: f64, lo: f64, hi: f64, dec: usize) {
        let v = if key == "51" { v.max(0.0).floor() } else { v.max(lo).min(hi) };
        let s = if key == "51" { format!("{}", v as i64) } else { fmt(v, dec) };
        self.ed_set_selected_first(move |o| { o.raw.set(key, s.clone()); });
    }

    pub(crate) fn ed_tr_refresh_pub(&mut self, i: usize) { self.ed_tr_refresh(i); }

    fn ed_tr_refresh(&mut self, i: usize) {
        let (key, lo, hi, dec, focused) = { let f = &self.ed.trig.flds[i]; (f.key, f.lo, f.hi, f.dec, self.ed.trig.focus == Some(i)) };
        let v = self.ed_tr_get(key, lo, hi);
        if !focused { self.ed.trig.flds[i].value = fmt(v, dec); }
        let (text, thumb, start, w, slider, shown) = { let f = &self.ed.trig.flds[i]; (f.text, f.thumb, f.start_x, f.width, !f.stepper, format!("{}{}", f.value, if focused { "|" } else { "" })) };
        if let Some(n) = self.nodes.get_mut(text).and_then(|n| n.as_mut()) { n.text = shown; }
        if slider {
            let (slo, shi) = { let f = &self.ed.trig.flds[i]; (f.slider_lo, f.slider_hi) };
            let pct = ((v - slo) / (shi - slo)).max(0.0).min(1.0);
            if let Some(n) = self.nodes.get_mut(thumb).and_then(|n| n.as_mut()) { n.x = start + pct * w; }
        }
    }

    fn ed_tr_commit(&mut self, i: usize) {
        let (key, lo, hi, tlo, thi, dec, stepper, text) = { let f = &self.ed.trig.flds[i]; (f.key, f.lo, f.hi, f.typed_lo, f.typed_hi, f.dec, f.stepper, f.value.clone()) };
        let parsed = if stepper { crate::js::parse_int(&text).map_or(0.0, |v| v as f64) } else { crate::js::parse_float(&text).filter(|v| v.is_finite()).unwrap_or(0.0) };
        let v = parsed.max(tlo).min(thi);
        self.ed_tr_set(key, v, lo, hi, dec);
    }

    pub(crate) fn ed_tr_blur(&mut self, commit: bool) {
        let Some(i) = self.ed.trig.focus else { return };
        if commit { self.ed_tr_commit(i); }
        self.ed.trig.focus = None;
        self.ed_tr_refresh(i);
    }

    pub(crate) fn ed_tr_key(&mut self, code: u32) -> bool {
        let Some(i) = self.ed.trig.focus else { return false };
        let c = char::from_u32(code).unwrap_or('\0');
        let (stepper, max_len, neg_ok) = { let f = &self.ed.trig.flds[i]; (f.stepper, f.max_len, f.typed_lo < 0.0) };
        match code {
            13 => { self.ed_tr_blur(true); return true; }
            8 => { self.ed.trig.flds[i].value.pop(); }
            _ => {
                let f = &mut self.ed.trig.flds[i];
                if c == '-' && f.value.is_empty() && neg_ok { f.value.push('-'); }
                else if c.is_ascii_digit() && f.value.len() < max_len { f.value.push(c); }
                else if c == '.' && !stepper && !f.value.contains('.') && f.value.len() < max_len { f.value.push('.'); }
            }
        }
        self.ed_tr_refresh(i);
        true
    }

    /// the stepper: label above, box, an arrow on each side
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ed_tr_stepper(&mut self, root: usize, x: f64, y: f64, label: &str, key: &'static str, d: f64) {
        let tag = if self.ed.trig.tag.is_empty() { "ed_op" } else { self.ed.trig.tag };
        self.pa_txt(root, "goldFont", 27.0, label, x, y - 56.0, 0.5, 0.5, 1.0, d, tag);
        let (w, h) = (102.0, 58.0);
        let bg = self.shape(Kind::RoundRect, x - w / 2.0, y - h / 2.0, w, h, 0x000000, 0.43, d, tag); { let n = self.node(bg); n.radius = 10.0; n.parent = Some(root); }
        let text = self.pa_txt(root, "bigFont", 32.0, "", x, y, 0.5, 0.5, 1.0, d + 0.01, tag);
        let i = self.ed.trig.flds.len();
        let hit = self.shape(Kind::Rect, x, y, w, h + 4.0, 0xffffff, 0.001, d + 0.02, tag);
        { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdTf", i); n.width = w; n.height = h + 4.0; }
        let l = self.pa_img(root, "GJ_GameSheet03", "edit_leftBtn_001.png", x - 88.0, y + 1.0, 0.9, d, tag);
        self.make_bouncy(l, 0.9, Cond::Always, Action::Pop("EdTfStep", i * 2));
        let r = self.pa_img(root, "GJ_GameSheet03", "edit_leftBtn_001.png", x + 88.0, y + 1.0, 0.9, d, tag);
        self.node(r).flip_x = true;
        self.make_bouncy(r, 0.9, Cond::Always, Action::Pop("EdTfStep", i * 2 + 1));
        self.ed.trig.flds.push(Fld { key, stepper: true, dec: 0, lo: 0.0, hi: 9999.0, typed_lo: 0.0, typed_hi: 9999.0, slider_lo: 0.0, slider_hi: 1.0, bg, text, thumb: 0, start_x: 0.0, width: 0.0, root_x: 0.0, value: String::new(), before: String::new(), max_len: 5 });
        self.ed_tr_refresh(i);
    }

    /// the slider with its typed value: `horizontal` puts the label, the box and the groove in a row (Arial label), else the time layout
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ed_tr_slider(&mut self, root: usize, x: f64, y: f64, label: &str, horizontal: bool, label_x: f64, width: f64, key: &'static str, g: (f64, f64), typed: (f64, f64), slider: (f64, f64), dec: usize, max_len: usize, d: f64) {
        let tag = if self.ed.trig.tag.is_empty() { "ed_op" } else { self.ed.trig.tag };
        let (iw, ih) = (118.0, 50.0);
        let (input_x, input_y) = if horizontal { (x - 250.0, y) } else { (x + 90.0, y) };
        let (slider_x, slider_y) = if horizontal { (x - 100.0, y) } else { (x, y + 50.0) };
        let lab = if horizontal { self.text_node("Arial", 24.0, label, label_x, input_y, d, 0.0, 0.5, 1.0) } else { self.text_node("goldFont", 32.0, label, label_x, input_y, d, 0.0, 0.5, 1.0) };
        { let n = self.node(lab); n.parent = Some(root); n.tag = tag; }
        let bg = self.shape(Kind::RoundRect, input_x - iw / 2.0, input_y - ih / 2.0, iw, ih, 0x000000, 0.43, d, tag); { let n = self.node(bg); n.radius = 10.0; n.parent = Some(root); }
        let text = self.pa_txt(root, "bigFont", 28.0, "", input_x, input_y, 0.5, 0.5, 1.0, d + 0.01, tag);
        let i = self.ed.trig.flds.len();
        let ih_hit = self.shape(Kind::Rect, input_x, input_y, iw + 8.0, ih + 8.0, 0xffffff, 0.001, d + 0.02, tag);
        { let n = self.node(ih_hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdTf", i); n.width = iw + 8.0; n.height = ih + 8.0; }
        let sc = 0.7;
        let groove_h = self.sizes.get("GJ_WebSheet/slidergroove.png").map_or(34.0, |s| s.1);
        let slider_w = (width - 8.0) * sc;
        let start_x = slider_x - slider_w / 2.0 + 2.0;
        // a nine-slice of 200 x 34 shown at scale 0.7, drawn at its final size
        let groove = self.shape(Kind::Nine, slider_x, slider_y, width * sc, groove_h * sc, 0, 1.0, d, tag);
        { let n = self.node(groove); n.frame = "GJ_WebSheet/slidergroove.png".into(); n.radius = 16.0 * sc; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); }
        let hit = self.shape(Kind::Rect, slider_x, slider_y, slider_w + 24.0, 34.0, 0xffffff, 0.001, d + 0.02, tag);
        { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdTfSlide", i); n.width = slider_w + 24.0; n.height = 34.0; }
        let thumb = self.pa_img(root, "GJ_WebSheet", "sliderthumb.png", start_x, slider_y, sc, d + 0.01, tag);
        let root_x = self.get(root).map_or(0.0, |r| r.x);
        self.ed.trig.flds.push(Fld { key, stepper: false, dec, lo: g.0, hi: g.1, typed_lo: typed.0, typed_hi: typed.1, slider_lo: slider.0, slider_hi: slider.1, bg, text, thumb, start_x, width: slider_w, root_x, value: String::new(), before: String::new(), max_len });
        self.ed_tr_refresh(i);
    }

    /// a check box with a (two line) label (`makeToggle`)
    pub(crate) fn ed_tr_toggle(&mut self, root: usize, x: f64, y: f64, label: &str, key: &'static str, peer: Option<&'static str>, scale: f64, fs: f64, d: f64) {
        let tag = if self.ed.trig.tag.is_empty() { "ed_op" } else { self.ed.trig.tag };
        let on = self.ed_tr_obj().map_or(false, |o| o.raw.get(key).map_or(false, |v| v == "1"));
        let check = self.pa_img(root, "GJ_GameSheet03", if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, x, y, scale, d, tag);
        let i = self.ed.trig.checks.len();
        self.make_bouncy(check, scale, Cond::Always, Action::Pop("EdTfToggle", i));
        let off = if fs > 22.5 { 42.0 } else { 40.0 };
        let t = self.pa_txt(root, "bigFont", fs, label, x + off, y - 2.0, 0.0, 0.5, 1.0, d, tag);
        self.node(t).interactive = true; self.node(t).action = Action::Pop("EdTfToggle", i);
        self.ed.trig.checks.push((check, key, peer));
    }

    fn ed_tr_toggle_press(&mut self, i: usize) {
        self.ed_tr_blur(true);
        let Some(&(_, key, peer)) = self.ed.trig.checks.get(i) else { return };
        let on = self.ed_tr_obj().map_or(false, |o| o.raw.get(key).map_or(false, |v| v == "1"));
        let next = !on;
        if next { if let Some(p) = peer { self.ed_set_selected_first(move |o| o.raw.set(p, "0".to_string())); } }
        self.ed_set_selected_first(move |o| o.raw.set(key, if next { "1" } else { "0" }.to_string()));
        for k in 0..self.ed.trig.checks.len() {
            let (node, key, _) = self.ed.trig.checks[k];
            let on = self.ed_tr_obj().map_or(false, |o| o.raw.get(key).map_or(false, |v| v == "1"));
            if let Some(n) = self.nodes.get_mut(node).and_then(|n| n.as_mut()) { n.frame = if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }.into(); }
        }
    }

    fn ed_tr_begin(&mut self, w: f64, h: f64, title: &str, tsize: f64, ty: f64, font: &str) -> (usize, f64) {
        self.ed_op_close();
        self.ed.trig = EdTrig::default();
        let root = self.ed_pop("ed_op", D_OP, 0.45, 6.0, Action::Pop("EdTfBlur", 0));
        let d = D_OP + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, w, h, d, "ed_op");
        self.pa_txt(root, font, tsize, title, 0.0, ty, 0.5, 0.5, 1.0, d, "ed_op");
        self.ed.trig.root = root;
        (root, d)
    }

    pub(crate) fn ed_move_popup(&mut self) {
        let (pw, ph) = (940.0, 620.0);
        let (root, d) = self.ed_tr_begin(pw, ph, "Setup Move Command", 42.0, -(ph / 2.0) + 50.0, "bigFont");
        self.ed_tr_stepper(root, 280.0, 230.0, "Target Group ID", "51", d);
        self.ed_tr_slider(root, 0.0, -180.0, "Move X:", true, 0.0 - 420.0, 200.0, "28", (f64::NEG_INFINITY, f64::INFINITY), (-99999.0, 99999.0), (-100.0, 100.0), 2, 7, d);
        self.ed_tr_slider(root, 450.0, -180.0, "Move Y:", true, 450.0 - 420.0, 200.0, "29", (f64::NEG_INFINITY, f64::INFINITY), (-99999.0, 99999.0), (-100.0, 100.0), 2, 7, d);
        self.ed_tr_slider(root, -225.0, 60.0, "Move Time:", false, -225.0 - 165.0, 400.0, "10", (0.0, 10.0), (0.0, 10.0), (0.0, 10.0), 2, 7, d);
        self.ed_tr_toggle(root, -410.0, 200.0, "Touch\nTrigger", "11", None, 0.78, 22.0, d);
        self.ed_tr_toggle(root, -410.0, 260.0, "Spawn\nTrigger", "62", None, 0.78, 22.0, d);
        self.pa_txt(root, "goldFont", 26.0, "Lock X:", -250.0, -130.0, 0.5, 0.5, 1.0, d, "ed_op");
        self.ed_tr_toggle(root, -380.0, -80.0, "Player", "58", Some("302"), 0.78, 22.0, d);
        self.ed_tr_toggle(root, -180.0, -80.0, "Camera", "302", Some("58"), 0.78, 22.0, d);
        self.pa_txt(root, "goldFont", 26.0, "Lock Y:", 200.0, -130.0, 0.5, 0.5, 1.0, d, "ed_op");
        self.ed_tr_toggle(root, 80.0, -80.0, "Player", "59", Some("303"), 0.78, 22.0, d);
        self.ed_tr_toggle(root, 260.0, -80.0, "Camera", "303", Some("59"), 0.78, 22.0, d);
        self.ed_ok(root, 0.0, (ph / 2.0) - 48.0, "OK", Action::Pop("EdTfOk", 0), d, "ed_op");
    }

    pub(crate) fn ed_spawn_popup(&mut self) {
        let (pw, ph) = (840.0, 580.0);
        let (root, d) = self.ed_tr_begin(pw, ph, "Spawn Group", 35.0, -(ph / 2.0) + 40.0, "bigFont");
        self.ed_tr_stepper(root, 0.0, -100.0, "Group ID", "51", d);
        self.ed_tr_slider(root, 30.0, 70.0, "Delay:", true, 30.0 - 400.0, 200.0, "63", (0.0, f64::INFINITY), (0.0, 10.0), (0.0, 10.0), 4, 9, d);
        self.ed_tr_slider(root, 400.0, 70.0, "+-", true, 400.0 - 350.0, 200.0, "556", (0.0, 9999.0), (0.0, 9999.0), (0.0, 1.0), 4, 9, d);
        self.ed_tr_toggle(root, -370.0, 180.0, "Touch\nTrigger", "11", None, 0.82, 23.0, d);
        self.ed_tr_toggle(root, -370.0, 245.0, "Spawn\nTrigger", "62", None, 0.82, 23.0, d);
        self.ed_ok(root, 0.0, (ph / 2.0) - 48.0, "OK", Action::Pop("EdTfOk", 0), d, "ed_op");
    }

    /// presses and drags on a slider groove
    pub fn ed_tf_pointer(&mut self, x: f64, phase: u8) {
        if self.ed.trig.flds.is_empty() { return; }
        match phase {
            0 => {}
            1 => if let Some(i) = self.ed.trig.drag { self.ed_tf_slide(i, x); },
            _ => self.ed.trig.drag = None,
        }
    }
    fn ed_tf_slide(&mut self, i: usize, px: f64) {
        self.ed_tr_blur(true);
        let (key, lo, hi, dec, start, w, slo, shi, root_x) = { let f = &self.ed.trig.flds[i]; (f.key, f.lo, f.hi, f.dec, f.start_x, f.width, f.slider_lo, f.slider_hi, f.root_x) };
        let lx = (px - root_x).max(start).min(start + w);
        let v = slo + ((lx - start) / w) * (shi - slo);
        self.ed_tr_set(key, v, lo, hi, dec);
        self.ed_tr_refresh(i);
    }
    pub fn ed_tf_down(&mut self, i: usize, x: f64) { if i < self.ed.trig.flds.len() { self.ed.trig.drag = Some(i); self.ed_tf_slide(i, x); } }

    pub fn ed_trig_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdTf" => {
                self.ed_tr_blur(true);
                let i = arg;
                if i < self.ed.trig.flds.len() {
                    // the box shows the stored value; typing starts from it
                    let (key, lo, hi, dec) = { let f = &self.ed.trig.flds[i]; (f.key, f.lo, f.hi, f.dec) };
                    let v = fmt(self.ed_tr_get(key, lo, hi), dec);
                    { let f = &mut self.ed.trig.flds[i]; f.value = v.clone(); f.before = v; }
                    self.ed.trig.focus = Some(i);
                    self.ed_tr_refresh(i);
                }
                true
            }
            "EdTfStep" => {
                self.ed_tr_blur(true);
                let (i, dir) = (arg / 2, if arg % 2 == 0 { -1.0 } else { 1.0 });
                if i < self.ed.trig.flds.len() {
                    let (key, lo, hi, dec) = { let f = &self.ed.trig.flds[i]; (f.key, f.lo, f.hi, f.dec) };
                    let cur = self.ed_tr_get(key, lo, hi).round();
                    self.ed_tr_set(key, cur + dir, lo, hi, dec);
                    self.ed_tr_refresh(i);
                }
                true
            }
            "EdTfSlide" => true,
            "EdTfToggle" => { self.ed_tr_toggle_press(arg); true }
            "EdTfBlur" => { self.ed_tr_blur(true); true }
            "EdTfOk" => { self.ed_tr_blur(true); self.ed_op_close(); true }
            _ => false,
        }
    }
}
