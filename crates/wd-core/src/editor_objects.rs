//! The object popups of the editor: group / layer popup (`_openSelectedEditorGroupPopup`), the choice of options popup for a selected
//! trigger (`_openSelectedEditorObjectOptions`) and the "Coming soon" popup.
use crate::editor::*;
use crate::ui::*;
use crate::visual::def_of;

const D_OP: f64 = 2550.0;
const PURPOSE_EL: u16 = 1;
const PURPOSE_EL2: u16 = 2;
const PURPOSE_ZO: u16 = 3;
const PURPOSE_GROUP: u16 = 4;

const Z_LAYERS: [(&str, i64); 10] = [("B5", -5), ("B4", -3), ("B3", -1), ("B2", 1), ("B1", 3), ("T1", 5), ("T2", 7), ("T3", 9), ("T4", 11), ("Default", 0)];

fn raw_i(e: &EdObj, key: &str) -> i64 { e.raw.get(key).and_then(|v| crate::js::parse_int(v)).unwrap_or(0) }

#[derive(Default)]
pub struct EdGp { pub open: bool, pub group_input: usize, pub list_root: usize, pub z_nodes: Vec<(usize, i64)> }

impl Ui {
    pub fn ed_objects_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdEditObject" => { self.ed_open_object_options(); true }
            "EdGroup" => { self.ed_gp_open(); true }
            "EdOpClose" => { self.ed_op_close(); true }
            "EdGpNextFree" => { self.ed_num_blur(true); let n = self.ed_next_free_group(); let g = self.ed.gp.group_input; self.ed_num_set(g, n, true); true }
            "EdGpAdd" => { self.ed_gp_add(); true }
            "EdGpRemove" => { self.ed_gp_remove(arg as i64); true }
            "EdGpZ" => { self.ed_num_blur(true); let v = Z_LAYERS[arg].1; self.ed_set_selected_first(|o| { o.o.z_layer = Some(v); o.raw.set("24", v.to_string()); }); self.ed_gp_z_tints(); true }
            _ => false,
        }
    }

    pub(crate) fn ed_op_close(&mut self) {
        self.ed.fields.focus = None; self.ed.fields.inputs.clear(); self.ed.fields.text_on = false; self.ed.trig = Default::default();
        self.kill_tag("ed_op"); self.kill_tag("ed_gp"); self.kill_tag("ed_sps"); self.ed_ho_close(); self.ed.gp.open = false;
    }

    /// edits one object and keeps the parsed form and the raw bag the same, as `ed_edit_selected` does for the whole selection
    pub(crate) fn ed_set_obj(&mut self, i: usize, f: impl Fn(&mut EdObj)) {
        if let Some(o) = self.ed.level.objects.get_mut(i) {
            f(o);
            let s = serialize_object(o);
            if let Some(n) = EdObj::parse(&s) { let keep = o.o.text.clone(); *o = n; if o.o.id == Some(914) { o.o.text = keep; } }
        }
        self.ed.dirty = true;
    }
    pub(crate) fn ed_set_selected_first(&mut self, f: impl Fn(&mut EdObj)) { if let Some(&i) = self.ed.sel.first() { self.ed_set_obj(i, f); } }

    pub(crate) fn ed_num_commit(&mut self, purpose: u16, v: i64) {
        match purpose {
            PURPOSE_EL => self.ed_set_selected_first(|o| { o.o.editor_layer = Some(v.max(0)); o.raw.set("20", v.max(0).to_string()); }),
            PURPOSE_EL2 => self.ed_set_selected_first(|o| { o.o.editor_layer2 = Some(v.max(0)); o.raw.set("61", v.max(0).to_string()); }),
            PURPOSE_ZO => self.ed_set_selected_first(|o| { o.o.z_order = Some(v); o.raw.set("25", v.to_string()); }),
            _ => {}
        }
    }

    /// `getNextFreeGroupId`: the smallest group id nothing uses, as a member or as the target of a trigger
    fn ed_next_free_group(&self) -> i64 {
        let mut used: std::collections::BTreeSet<i64> = Default::default();
        let first = |e: &EdObj, k: &str| -> Option<i64> { e.raw.get(k).and_then(|v| crate::js::parse_int(v)).filter(|v| *v > 0) };
        for o in &self.ed.level.objects {
            for g in group_ids(o) { used.insert(g); }
            let id = o.o.id.unwrap_or(0);
            if matches!(id, 901 | 1007 | 1268 | 1346) { if let Some(g) = first(o, "51") { used.insert(g); } }
            if id == 1346 { if let Some(g) = first(o, "71") { used.insert(g); } }
            if id == 1006 && raw_i(o, "52") == 1 { if let Some(g) = first(o, "51") { used.insert(g); } }
        }
        let mut c = 1; while used.contains(&c) { c += 1; }
        c
    }

    fn ed_set_groups(&mut self, i: usize, groups: Vec<i64>) {
        let s = groups.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(".");
        self.ed_set_obj(i, |o| { o.o.groups = s.clone(); o.raw.set("57", s.clone()); o.raw.delete("33"); });
    }

    /// the group ids of the selection with how many of the objects have each
    fn ed_group_entries(&self) -> (usize, Vec<(i64, usize)>) {
        let mut counts: std::collections::BTreeMap<i64, usize> = Default::default();
        for &i in &self.ed.sel { if let Some(o) = self.ed.level.objects.get(i) { for g in group_ids(o) { *counts.entry(g).or_insert(0) += 1; } } }
        (self.ed.sel.len(), counts.into_iter().collect())
    }

    fn ed_gp_add(&mut self) {
        self.ed_num_blur(true);
        let new = self.ed_num_get(self.ed.gp.group_input).max(1);
        let (total, entries) = self.ed_group_entries();
        if total == 0 { return; }
        if entries.iter().any(|e| e.0 == new && e.1 >= total) { return; }
        for i in self.ed.sel.clone() {
            let Some(o) = self.ed.level.objects.get(i) else { continue };
            let mut g = group_ids(o);
            if !g.contains(&new) { g.push(new); }
            g.sort();
            self.ed_set_groups(i, g);
        }
        let gi = self.ed.gp.group_input; self.ed_num_set(gi, new, false);
        self.ed_gp_list();
    }

    fn ed_gp_remove(&mut self, id: i64) {
        for i in self.ed.sel.clone() {
            let Some(o) = self.ed.level.objects.get(i) else { continue };
            let g: Vec<i64> = group_ids(o).into_iter().filter(|v| *v != id).collect();
            self.ed_set_groups(i, g);
        }
        self.ed_gp_list();
    }

    /// a gold nine-slice button of the popups (`makeGoldButton`)
    fn ed_gold_button(&mut self, root: usize, x: f64, y: f64, w: f64, h: f64, text: &str, font: &str, fs: f64, ly: f64, frame: &str, action: Action, d: f64, tag: &'static str) -> usize {
        let tex = self.sizes.get(&format!("/{}", frame)).map_or(80.0, |s| s.0);
        let zone = self.shape(Kind::Rect, x, y, w, h, 0, 0.0, d, tag);
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = h; n.parent = Some(root); }
        let nine = self.shape(Kind::Nine, 0.0, 0.0, w, h, 0, 1.0, d, tag);
        { let n = self.node(nine); n.frame = frame.into(); n.radius = (tex * 0.28).min(w / 2.0).min(h / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
        self.pa_txt(zone, font, fs, text, -3.0, ly, 0.5, 0.5, 1.0, d + 0.01, tag);
        self.make_bouncy(zone, 1.0, Cond::Always, action);
        zone
    }

    fn ed_gp_list(&mut self) {
        self.kill_tag("ed_gpl");
        let (total, entries) = self.ed_group_entries();
        if total == 0 || entries.is_empty() { return; }
        let root = self.ed.gp.list_root;
        let (cols, gx, gy, bw, bh) = (5usize, 16.0, 14.0, 100.0, 40.0);
        let sx = -(((cols - 1) as f64) * (bw + gx)) / 2.0;
        for (k, (id, count)) in entries.iter().take(10).enumerate() {
            let (col, row) = (k % cols, k / cols);
            let frame = if *count < total { "GJ_button05" } else { "GJ_button04" };
            self.ed_gold_button(root, sx + col as f64 * (bw + gx), 10.0 + row as f64 * (bh + gy), bw, bh, &id.to_string(), "goldFont", 28.0, -4.0, frame, Action::Pop("EdGpRemove", *id as usize), D_OP + 0.3, "ed_gpl");
        }
    }

    fn ed_gp_z_tints(&mut self) {
        let cur = self.ed.sel.first().and_then(|&i| self.ed.level.objects.get(i)).map_or(0, |o| o.o.z_layer.unwrap_or(0));
        let nodes = self.ed.gp.z_nodes.clone();
        for (n, v) in nodes {
            // the selected layer button is the green one
            if let Some(node) = self.nodes.get_mut(n).and_then(|x| x.as_mut()) { node.frame = if v == cur { "GJ_button02" } else { "GJ_button04" }.into(); }
        }
    }

    pub fn ed_gp_open(&mut self) {
        if self.ed.sel.is_empty() { return; }
        self.ed_op_close();
        self.ed.gp.open = true;
        let sw = self.screen_width;
        let (pw, ph) = ((860.0f64).min(sw - 60.0), (615.0f64).max(640.0 - 80.0));
        let root = self.ed_pop("ed_gp", D_OP, 0.45, 0.0, Action::Pop("EdNumBlur", 0));
        let d = D_OP + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_gp");
        let first = self.ed.sel[0];
        let (el, el2, zo, zl) = { let o = &self.ed.level.objects[first]; (o.o.editor_layer.unwrap_or(0), o.o.editor_layer2.unwrap_or(0), o.o.z_order.unwrap_or(0), o.o.z_layer.unwrap_or(0)) };
        let _ = zl;
        let (top_y, gap) = (-(ph / 2.0) + 85.0, 230.0);
        self.ed_num_input(root, "ed_gp", d, PURPOSE_EL, -gap, top_y, "Editor L", "GJ_arrow_02_001.png", el, 0, 999, false);
        self.ed_num_input(root, "ed_gp", d, PURPOSE_EL2, 0.0, top_y, "Editor L2", "GJ_arrow_03_001.png", el2, 0, 999, false);
        self.ed_num_input(root, "ed_gp", d, PURPOSE_ZO, gap, top_y, "Z Order", "GJ_arrow_02_001.png", zo, -999, 999, true);
        let cy = top_y + 110.0;
        self.pa_txt(root, "goldFont", 32.0, "Add Group ID", 0.0, cy - 52.0, 0.5, 0.5, 1.0, d, "ed_gp");
        self.ed.gp.group_input = self.ed_num_input(root, "ed_gp", d, PURPOSE_GROUP, 0.0, cy, "", "GJ_arrow_01_001.png", 1, 1, 9999, false);
        self.ed_gold_button(root, -240.0, cy, 140.0, 50.0, "Next Free", "goldFont", 20.0, -4.0, "GJ_button04", Action::Pop("EdGpNextFree", 0), d, "ed_gp");
        self.ed_gold_button(root, 205.0, cy, 140.0, 50.0, "Add", "goldFont", 34.0, -4.0, "GJ_button04", Action::Pop("EdGpAdd", 0), d, "ed_gp");
        let (lw, lh) = (pw - 110.0, 145.0);
        let ly = cy + 60.0 + lh / 2.0;
        let bg = self.shape(Kind::RoundRect, -lw / 2.0, ly - lh / 2.0, lw, lh, 0x000000, 0.2, d, "ed_gp"); { let n = self.node(bg); n.radius = 20.0; n.parent = Some(root); }
        let lr = self.shape(Kind::Rect, 0.0, ly - lh / 2.0 + 34.0, 0.0, 0.0, 0, 0.0, d, "ed_gp"); self.node(lr).parent = Some(root);
        self.ed.gp.list_root = lr;
        self.ed_gp_list();
        // Z layer buttons
        let zt = ly + lh / 2.0 + 30.0;
        self.pa_txt(root, "goldFont", 34.0, "Z Layer", 0.0, zt, 0.5, 0.5, 1.0, d, "ed_gp");
        let (def_w, gapx) = (132.0, 10.0);
        let usable = pw - 92.0;
        let sq = (54.0f64).max(((usable - def_w - gapx * 9.0) / 9.0).floor());
        let widths: Vec<f64> = Z_LAYERS.iter().map(|l| if l.0 == "Default" { def_w } else { sq }).collect();
        let total = widths.iter().sum::<f64>() + gapx * (widths.len() as f64 - 1.0);
        let mut lx = -total / 2.0;
        self.ed.gp.z_nodes.clear();
        for (k, (label, v)) in Z_LAYERS.iter().enumerate() {
            let w = widths[k];
            let z = self.ed_gold_button(root, lx + w / 2.0, zt + 55.0, w, 48.0, label, "bigFont", if *label == "Default" { 20.0 } else { 23.0 }, -2.0, "GJ_button04", Action::Pop("EdGpZ", k), d, "ed_gp");
            self.ed.gp.z_nodes.push((self.ed_first_nine(z), *v));
            lx += w + gapx;
        }
        self.ed_gp_z_tints();
        self.ed_ok(root, 0.0, (ph / 2.0) - 46.0, "OK", Action::Pop("EdOpClose", 0), d, "ed_gp");
    }

    /// the nine-slice child of a gold button zone
    fn ed_first_nine(&self, zone: usize) -> usize { (0..self.nodes.len()).find(|&i| self.get(i).map_or(false, |n| n.parent == Some(zone) && n.kind == Kind::Nine)).unwrap_or(zone) }

    // ---- the options of a selected trigger ----
    pub fn ed_open_object_options(&mut self) {
        let Some(&i) = self.ed.sel.first() else { return };
        let Some(o) = self.ed.level.objects.get(i) else { return };
        let id = o.o.id.unwrap_or(0);
        let Some(d) = def_of(id) else { return };
        if d.get_str("type") != Some("trigger") { return; }
        self.ed_op_close();
        if matches!(id, 31 | 34) { self.ed_sp_open(); }
        else if matches!(id, 29 | 30 | 105 | 744 | 899 | 900 | 915) { self.ed_ct_open(id); }
        else if id == 901 { self.ed_move_popup(); }
        else if id == 1268 { self.ed_spawn_popup(); }
        else if d.get("textObject").is_some() { self.ed_text_open(); }
        else { self.ed_coming_soon(); }
    }

    // ---- start positions ----
    /// `_getEditorStartPositionValue`
    pub(crate) fn ed_sp_value(&self, key: &str, fb: i64) -> i64 {
        let Some(o) = self.ed.sel.first().and_then(|&i| self.ed.level.objects.get(i)) else { return fb };
        match key {
            "kA2" => o.o.game_mode, "kA3" => o.o.mini_mode, "kA4" => o.o.speed, "kA8" => o.o.dual_mode, "kA28" => o.o.mirrored,
            "kA11" => return if o.raw.get("kA11").is_some() { raw_i(o, "kA11") } else { o.o.flip_gravity as i64 },
            _ => None,
        }.unwrap_or(fb)
    }
    pub(crate) fn ed_sp_set(&mut self, key: &str, v: i64) {
        let key = key.to_string();
        self.ed_set_selected_first(move |o| {
            match key.as_str() {
                "kA2" => o.o.game_mode = Some(v), "kA3" => o.o.mini_mode = Some(v), "kA4" => o.o.speed = Some(v),
                "kA8" => o.o.dual_mode = Some(v), "kA28" => o.o.mirrored = Some(v), "kA11" => o.o.flip_gravity = v == 1, _ => {}
            }
            o.raw.set(&key, v.to_string());
        });
    }

    fn ed_sp_open(&mut self) {
        let (pw, ph) = (720.0, 320.0);
        let root = self.ed_pop("ed_op", D_OP, 0.45, 10.0, Action::Pop("EdNoop", 0));
        let d = D_OP + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_op");
        let sb = self.pa_img(root, "GJ_GameSheet03", "GJ_optionsBtn_001.png", -(pw / 2.0) + 58.0, -(ph / 2.0) + 58.0, 0.64, d, "ed_op");
        self.make_bouncy(sb, 0.64, Cond::Always, Action::Pop("EdSpSettings", 0));
        self.pa_txt(root, "goldFont", 34.0, "Speed:", -165.0, -74.0, 0.5, 0.5, 1.0, d, "ed_op");
        self.pa_txt(root, "goldFont", 34.0, "Mode:", 165.0, -74.0, 0.5, 0.5, 1.0, d, "ed_op");
        self.ed.sp_root = root;
        self.ed_sp_refresh();
        self.ed_ok(root, 0.0, (ph / 2.0) - 48.0, "OK", Action::Pop("EdOpClose", 0), d, "ed_op");
    }

    /// the speed and mode buttons after a change
    pub(crate) fn ed_sp_refresh(&mut self) {
        self.kill_tag("ed_opv");
        let root = self.ed.sp_root;
        let d = D_OP + 0.2;
        let key = self.ed_sp_value("kA4", 0);
        let (si, sd) = crate::editor_ui::SPEEDS.iter().enumerate().find(|(_, s)| s.0 == key).map(|(i, s)| (i, *s)).unwrap_or((1, crate::editor_ui::SPEEDS[1]));
        self.ed_obj_button(root, -165.0, 20.0, sd.2, 67.0, if si == 0 { 1.05 } else { 1.2 }, Action::Pop("EdSpPick", 0), d, "ed_opv");
        let mk = self.ed_sp_value("kA2", 0);
        let md = crate::editor_ui::MODES.iter().find(|m| m.0 == mk).copied().unwrap_or(crate::editor_ui::MODES[0]);
        if let Some(i) = self.ed_frame_img(root, md.2, 165.0, 20.0, 0.88, d, "ed_opv") { self.make_bouncy(i, 0.88, Cond::Always, Action::Pop("EdSpPick", 1)); }
    }

    pub(crate) fn ed_sp_settings_open(&mut self) {
        self.kill_tag("ed_sps");
        let root = self.ed_pop("ed_sps", 2850.0, 0.25, 15.0, Action::Pop("EdNoop", 0));
        let d = 2850.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, 520.0, 420.0, d, "ed_sps");
        let c = self.pa_img(root, "GJ_WebSheet", "GJ_closeBtn_001.png", -(520.0 / 2.0) + 10.0, -(420.0 / 2.0) + 10.0, 0.8, d, "ed_sps");
        self.make_bouncy(c, 0.8, Cond::Always, Action::Pop("EdSpSettingsClose", 0));
        self.ed.sp_checks.clear();
        for (i, (label, key)) in [("Mini Mode", "kA3"), ("Flip Gravity", "kA11"), ("Dual Mode", "kA8"), ("Mirror Mode", "kA28")].iter().enumerate() {
            let y = -130.0 + i as f64 * 80.0;
            let on = self.ed_sp_value(key, 0) == 1;
            let check = self.pa_img(root, "GJ_GameSheet03", if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, -150.0, y, 0.9, d, "ed_sps");
            self.make_bouncy(check, 0.9, Cond::Always, Action::Pop("EdSpToggle", i));
            let t = self.pa_txt(root, "bigFont", 28.0, label, -150.0 + 48.0, y - 2.0, 0.0, 0.5, 1.0, d, "ed_sps");
            self.node(t).interactive = true; self.node(t).action = Action::Pop("EdSpToggle", i);
            self.ed.sp_checks.push((check, key));
        }
    }

    pub(crate) fn ed_sp_toggle(&mut self, i: usize) {
        let Some(&(node, key)) = self.ed.sp_checks.get(i) else { return };
        let on = self.ed_sp_value(key, 0) == 1;
        self.ed_sp_set(key, if on { 0 } else { 1 });
        if let Some(n) = self.nodes.get_mut(node).and_then(|n| n.as_mut()) { n.frame = if on { "GJ_checkOff_001.png" } else { "GJ_checkOn_001.png" }.into(); }
    }

    // ---- text objects ----
    fn ed_text_open(&mut self) {
        let (pw, ph) = (720.0, 330.0);
        let root = self.ed_pop("ed_op", D_OP, 0.45, 10.0, Action::Pop("EdNoop", 0));
        let d = D_OP + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_op");
        self.pa_txt(root, "goldFont", 42.0, "Edit Text", 0.0, -(ph / 2.0) + 46.0, 0.5, 0.5, 1.0, d, "ed_op");
        let bg = self.shape(Kind::RoundRect, -270.0, -47.0, 540.0, 70.0, 0x000000, 0.38, d, "ed_op"); { let n = self.node(bg); n.radius = 15.0; n.parent = Some(root); }
        let t = self.pa_txt(root, "bigFont", 45.0, "", 0.0, -12.0, 0.5, 0.5, 1.0, d + 0.01, "ed_op");
        self.ed.fields.text_node = t; self.ed.fields.text_on = true;
        self.ed_text_render();
        self.ed_ok(root, 0.0, (ph / 2.0) - 48.0, "OK", Action::Pop("EdOpClose", 0), d, "ed_op");
    }

    fn ed_text_render(&mut self) {
        let v = self.ed.sel.first().and_then(|&i| self.ed.level.objects.get(i)).map(|o| o.o.text.clone()).unwrap_or_default();
        let shown = format!("{}|", v);
        let n = self.ed.fields.text_node;
        let w = self.fonts.get("bigFont").map_or(0.0, |f| f.width(&shown, 45.0));
        if let Some(x) = self.nodes.get_mut(n).and_then(|x| x.as_mut()) { x.text = shown; x.width = w; }
    }

    /// typing into the text of a text object: every key changes the object, as `commit()` does after each key
    pub(crate) fn ed_text_key(&mut self, code: u32) -> bool {
        let Some(&i) = self.ed.sel.first() else { return false };
        let Some(o) = self.ed.level.objects.get(i) else { return false };
        let mut v = o.o.text.clone();
        match code {
            13 => { self.ed_op_close(); return true; }
            8 => { v.pop(); }
            c => { if let Some(ch) = char::from_u32(c) { if !ch.is_control() && v.chars().count() < 80 { v.push(ch); } } }
        }
        self.ed_set_obj(i, move |o| { o.o.text = v.clone(); });
        self.ed_text_render();
        true
    }

    fn ed_coming_soon(&mut self) {
        let (pw, ph) = (680.0, 300.0);
        let root = self.ed_pop("ed_op", D_OP, 0.45, 10.0, Action::Pop("EdNoop", 0));
        let d = D_OP + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_op");
        let t = self.pa_txt(root, "bigFont", 34.0, "Coming soon!\n- Lasokar", 0.0, -35.0, 0.5, 0.5, 1.0, d, "ed_op");
        let _ = t;
        self.ed_ok(root, 0.0, (ph / 2.0) - 48.0, "OK", Action::Pop("EdOpClose", 0), d, "ed_op");
    }
}
