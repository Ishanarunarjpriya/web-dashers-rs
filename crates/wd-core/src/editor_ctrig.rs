//! The colour trigger popup (level-editor.js `_openEditorColorTriggerOptionsPopup`): the colour picker with the trigger's colour and
//! extras: the colour channel (id, arrows, typed), the fade time (slider and typed) and the touch / spawn trigger check boxes.
use crate::editor_color::CpOpts;
use crate::ui::*;

fn channel_label(ch: i64) -> String {
    match ch { 1000 => "BG", 1001 => "G1", 1002 => "L", 1003 => "3DL", 1004 => "Obj", 1009 => "G2", 1013 => "MG", c if c > 0 => return c.to_string(), _ => "None" }.to_string()
}

impl Ui {
    fn ed_ct_obj(&self) -> Option<&crate::editor::EdObj> { self.ed.sel.first().and_then(|&i| self.ed.level.objects.get(i)) }

    /// `_getEditorColorTriggerTargetChannel`
    fn ed_ct_channel(&self) -> i64 {
        let Some(o) = self.ed_ct_obj() else { return 1 };
        match o.o.id { Some(29) => 1000, Some(30) => 1001, _ => { let c = o.raw.get("23").and_then(|v| crate::js::parse_int(v)).unwrap_or(0); if c > 0 { c } else { 1 } } }
    }

    pub fn ed_ct_open(&mut self, id: i64) {
        self.ed_op_close();
        let ch = self.ed_ct_channel();
        let initial = self.ed_ct_obj().map(|o| {
            let g = |k: &str, d: i64| o.raw.get(k).and_then(|v| crate::js::parse_int(v)).unwrap_or(d);
            let c = self.ed_channel_color(ch);
            (g("7", c.0), g("8", c.1), g("9", c.2))
        });
        let _ = id;
        self.ed.ct_focus = false; self.ed.ct_input = String::new();
        self.ed_cp_open_with(ch, CpOpts { trigger: true, initial, commit: false, panel_w: 920.0, panel_h: 620.0, content_y: -50.0, picker_y: Some(-80.0), title: "Select Color".into() });
    }

    /// the colour the picker shows goes into the trigger (`_setEditorObjectColor`)
    pub(crate) fn ed_ct_write_color(&mut self, c: (i64, i64, i64)) {
        let Some(&i) = self.ed.sel.first() else { return };
        if let Some(o) = self.ed.level.objects.get_mut(i) { o.raw.set("7", c.0.to_string()); o.raw.set("8", c.1.to_string()); o.raw.set("9", c.2.to_string()); }
    }

    pub(crate) fn ed_ct_extras(&mut self, root: usize, pw: f64, ph: f64, d: f64) {
        self.ed.trig = Default::default();
        self.ed.trig.tag = "ed_cp";
        let (cx, cy) = ((pw / 2.0) - 160.0, (ph / 2.0) - 120.0);
        let label_y = cy - 57.0;
        let can_retarget = !matches!(self.ed_ct_obj().and_then(|o| o.o.id), Some(29) | Some(30));
        if can_retarget {
            self.pa_txt(root, "goldFont", 35.0, "Color ID", cx, label_y, 0.5, 0.5, 1.0, d, "ed_cp");
            let plus = self.pa_img(root, "GJ_GameSheet03", "GJ_plus2Btn_001.png", cx + 120.0, label_y + 3.0, 1.0, d, "ed_cp");
            self.make_bouncy(plus, 1.0, Cond::Always, Action::Pop("EdCtPlus", 0));
            let bg = self.shape(Kind::RoundRect, cx - 46.0, cy - 30.0, 92.0, 60.0, 0x000000, 0.43, d, "ed_cp"); { let n = self.node(bg); n.radius = 10.0; n.parent = Some(root); }
            let t = self.pa_txt(root, "bigFont", 35.0, "", cx, cy, 0.5, 0.5, 1.0, d + 0.01, "ed_cp");
            self.ed.ct_text = t;
            let hit = self.shape(Kind::Rect, cx, cy, 92.0, 64.0, 0xffffff, 0.001, d + 0.02, "ed_cp");
            { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdCtFocus", 0); n.width = 92.0; n.height = 64.0; }
            let l = self.pa_img(root, "GJ_GameSheet03", "edit_leftBtn_001.png", cx - 86.0, cy + 1.0, 1.08, d, "ed_cp");
            self.make_bouncy(l, 1.08, Cond::Always, Action::Pop("EdCtArrow", 0));
            let r = self.pa_img(root, "GJ_GameSheet03", "edit_leftBtn_001.png", cx + 86.0, cy + 1.0, 1.08, d, "ed_cp");
            self.node(r).flip_x = true;
            self.make_bouncy(r, 1.08, Cond::Always, Action::Pop("EdCtArrow", 1));
            self.ed_ct_draw();
        }
        // fade time
        let fy = 140.0;
        self.pa_txt(root, "goldFont", 32.0, "Fade Time:", -70.0, fy, 0.5, 0.5, 1.0, d, "ed_cp");
        let (fw, fh) = (110.0, 55.0);
        let bg = self.shape(Kind::RoundRect, 100.0 - fw / 2.0, fy - fh / 2.0, fw, fh, 0x000000, 0.43, d, "ed_cp"); { let n = self.node(bg); n.radius = 10.0; n.parent = Some(root); }
        let text = self.pa_txt(root, "bigFont", 35.0, "", 100.0, fy, 0.5, 0.5, 1.0, d + 0.01, "ed_cp");
        let i = self.ed.trig.flds.len();
        let hit = self.shape(Kind::Rect, 100.0, fy, fw + 8.0, fh + 8.0, 0xffffff, 0.001, d + 0.02, "ed_cp");
        { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdTf", i); n.width = fw + 8.0; n.height = fh + 8.0; }
        let groove_w = self.sizes.get("GJ_WebSheet/slidergroove.png").map_or(420.0, |s| s.0);
        let sc = 0.8;
        let slider_w = (groove_w - 8.0) * sc;
        let sy = fy + 55.0;
        let start = -slider_w / 2.0 + 2.0;
        self.pa_img(root, "GJ_WebSheet", "slidergroove.png", 0.0, sy, sc, d, "ed_cp");
        let sh = self.shape(Kind::Rect, 0.0, sy, slider_w + 24.0, 34.0, 0xffffff, 0.001, d + 0.02, "ed_cp");
        { let n = self.node(sh); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdTfSlide", i); n.width = slider_w + 24.0; n.height = 34.0; }
        let thumb = self.pa_img(root, "GJ_WebSheet", "sliderthumb.png", start, sy, sc, d + 0.01, "ed_cp");
        let root_x = self.get(root).map_or(0.0, |r| r.x);
        self.ed.trig.flds.push(crate::editor_trigger::Fld { key: "10", stepper: false, dec: 2, lo: 0.0, hi: f64::INFINITY, typed_lo: 0.0, typed_hi: 10.0, slider_lo: 0.0, slider_hi: 10.0, bg, text, thumb, start_x: start, width: slider_w, root_x, value: String::new(), before: String::new(), max_len: 5 });
        self.ed_tr_refresh_pub(i);
        // touch / spawn trigger (turning one on turns the other off)
        self.ed_tr_toggle(root, (pw / 2.0) - 350.0, fy + 120.0, "Touch\nTrigger", "11", Some("62"), 0.82, 24.0, d);
        self.ed_tr_toggle(root, (pw / 2.0) - 165.0, fy + 120.0, "Spawn\nTrigger", "62", Some("11"), 0.82, 24.0, d);
    }

    fn ed_ct_draw(&mut self) {
        let t = self.ed.ct_text;
        let ch = self.ed_ct_channel();
        let shown = if self.ed.ct_focus { format!("{}|", self.ed.ct_input) } else { channel_label(ch) };
        if let Some(n) = self.nodes.get_mut(t).and_then(|n| n.as_mut()) { n.text = shown; n.tint = 0xffffff; }
    }

    fn ed_ct_set_channel(&mut self, v: i64) {
        let v = v.max(1);
        self.ed_set_selected_first(move |o| { o.raw.set("23", v.to_string()); });
        self.ed.ct_input = self.ed_ct_channel().to_string();
        self.ed_ct_draw();
    }

    pub(crate) fn ed_ct_key(&mut self, code: u32) -> bool {
        let c = char::from_u32(code).unwrap_or('\0');
        match code {
            13 => { self.ed_ct_commit(); return true; }
            8 => { self.ed.ct_input.pop(); }
            _ => { if c.is_ascii_digit() && self.ed.ct_input.len() < 4 { self.ed.ct_input.push(c); } }
        }
        self.ed_ct_draw();
        true
    }

    fn ed_ct_commit(&mut self) {
        if let Some(v) = crate::js::parse_int(&self.ed.ct_input).filter(|v| *v > 0) { self.ed_ct_set_channel(v); }
        self.ed.ct_focus = false;
        self.ed_ct_draw();
    }

    pub fn ed_ct_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdCtFocus" => { self.ed_tr_blur(true); self.ed.ct_focus = true; self.ed.ct_input = self.ed_ct_channel().to_string(); self.ed_ct_draw(); true }
            "EdCtArrow" => {
                self.ed_ct_commit();
                let cur = self.ed_ct_channel();
                let next = if arg == 0 { if cur <= 1 { 1 } else { cur - 1 } } else { cur + 1 };
                self.ed_ct_set_channel(next); true
            }
            "EdCtPlus" => true,
            _ => false,
        }
    }
}
