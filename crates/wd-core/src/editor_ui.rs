//! Shared widgets of the editor popups, and the level settings popup with the pickers it opens
//! (level-editor.js `_openEditorLevelSettingsPopup`, `_openEditorHorizontalOptionPopup`, `_openEditorStartOptionsPopup`).
//! Popups are trees of nodes under one root per popup (tag `ed_ls`, `ed_ho`, `ed_so`, ...), centred on the screen like a Phaser container.
use crate::editor::*;
use crate::jsonread::Json;
use crate::ui::*;
use crate::visual::def_of;

pub const D_LS: f64 = 2300.0;
pub const D_HO: f64 = 2600.0;
pub const D_SO: f64 = 2650.0;

#[derive(Default)]
pub struct EdLs { pub cat: u8, pub idx: i64, pub custom: String, pub before: String, pub focused: bool }

/// the options of the horizontal picker: what each button shows
#[derive(Clone, Debug)]
pub enum HoKind { Speed, Mode, Bg, Ground }
#[derive(Default)]
pub struct EdHo { pub kind: Option<u8>, pub keys: Vec<i64>, pub selected: i64, pub nodes: Vec<(usize, f64)> }

const SPEEDS: [(i64, &str, i64); 5] = [(1, "0.5x", 200), (0, "1x", 201), (2, "2x", 202), (3, "3x", 203), (4, "4x", 1334)];
const MODES: [(i64, &str, &str); 8] = [
    (0, "Cube", "gj_iconBtn_off_001.png"), (1, "Ship", "gj_shipBtn_off_001.png"), (2, "Ball", "gj_ballBtn_off_001.png"), (3, "UFO", "gj_birdBtn_off_001.png"),
    (4, "Wave", "gj_dartBtn_off_001.png"), (5, "Robot", "gj_robotBtn_off_001.png"), (6, "Spider", "gj_spiderBtn_off_001.png"), (7, "Swing", "gj_swingBtn_off_001.png"),
];
const COLOR_TARGETS: [(&str, i64, f64); 8] = [("BG:", 1000, -280.0), ("G1:", 1001, -200.0), ("G2:", 1009, -120.0), ("Line:", 1002, -40.0), ("3DL:", 1003, 40.0), ("Obj:", 1004, 120.0), ("MG:", 1013, 200.0), ("MG2:", 1014, 280.0)];

fn art_frame(prefix: &str, i: i64) -> String { format!("{}Icon_{:02}_001.png", prefix, i) }

impl Ui {
    // ---- widgets ----
    /// A popup: a dim that swallows clicks and a root node at the screen centre (+ `dy`); returns the root.
    pub(crate) fn ed_pop(&mut self, tag: &'static str, depth: f64, dim_a: f64, dy: f64, dim_action: Action) -> usize {
        let sw = self.screen_width;
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, dim_a, depth, tag);
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; n.action = dim_action; }
        self.shape(Kind::Rect, sw / 2.0, 320.0 + dy, 0.0, 0.0, 0, 0.0, depth, tag)
    }

    /// `_makeEditorOkButton`: a 124 x 62 nine-slice with a gold label, growing to 1.16 while pressed
    pub(crate) fn ed_ok(&mut self, root: usize, x: f64, y: f64, text: &str, action: Action, depth: f64, tag: &'static str) -> usize {
        let tex = self.sizes.get("/GJ_button01").map_or(80.0, |s| s.0);
        let zone = self.shape(Kind::Rect, x, y, 124.0, 62.0, 0, 0.0, depth, tag);
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 124.0; n.height = 62.0; n.parent = Some(root); }
        let nine = self.shape(Kind::Nine, 0.0, 0.0, 124.0, 62.0, 0, 1.0, depth, tag);
        { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = (tex * 0.3).min(31.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
        self.pa_txt(zone, "goldFont", 42.0, text, -5.0, -3.0, 0.5, 0.5, 1.0, depth + 0.01, tag);
        self.make_bouncy(zone, 1.0, Cond::Always, action);
        self.node(zone).press = 1.16;
        zone
    }

    /// a frame of whichever atlas has it (`_addSafeFrameImage`)
    pub(crate) fn ed_frame_img(&mut self, root: usize, frame: &str, x: f64, y: f64, scale: f64, depth: f64, tag: &'static str) -> Option<usize> {
        let atlas = crate::atlas_for_frame(frame)?;
        Some(self.pa_img(root, &atlas, frame, x, y, scale, depth, tag))
    }

    fn ed_obj_frame(id: i64) -> Option<String> {
        let d = def_of(id)?;
        let mut frame = d.get_str("frame").map(String::from);
        if let Some(rf) = d.get("randomFrames").and_then(Json::arr) { if let Some(f) = rf.first().and_then(|v| v.str()) { frame = Some(f.to_string()); } }
        frame.filter(|f| !f.is_empty())
    }

    // ---- keys ----
    /// keys typed while an editor text field has the focus; true when consumed
    pub fn ed_key(&mut self, code: u32) -> bool {
        if self.ed.ls.focused {
            let c = char::from_u32(code).unwrap_or('\0');
            match code {
                13 => { self.ed_ls_blur(true); }
                8 => { self.ed.ls.custom.pop(); self.ed_ls_refresh(); }
                _ => if c.is_ascii_digit() && self.ed.ls.custom.len() < 12 { self.ed.ls.custom.push(c); self.ed_ls_refresh(); }
            }
            return true;
        }
        self.ed_fields_key(code)
    }

    // ---- level settings ----
    pub fn ed_ls_open(&mut self) {
        if self.ed.ls_open { return; }
        self.ed.ls_open = true;
        // the song of the level starts the pickers
        let active = self.ed.pending_song.as_ref().map(|p| p.0).unwrap_or(self.ed.song_id);
        self.ed.ls = EdLs { cat: if active > 0 { 1 } else { 0 }, idx: if active < 0 { (active.abs() - 1).max(0) } else { 0 }, custom: if active > 0 { active.to_string() } else { String::new() }, before: String::new(), focused: false };
        self.ed_ls_build();
    }

    pub fn ed_ls_close(&mut self) {
        self.kill_tag("ed_ls");
        self.ed.ls_open = false; self.ed.ls.focused = false;
    }

    pub(crate) fn ed_ls_refresh(&mut self) { if self.ed.ls_open { self.kill_tag("ed_ls"); self.ed_ls_build(); } }

    fn ed_ls_blur(&mut self, commit: bool) {
        if !self.ed.ls.focused { return; }
        self.ed.ls.focused = false;
        if commit { self.ed_ls_commit_custom(); }
        self.ed_ls_refresh();
    }

    fn ed_ls_commit_custom(&mut self) {
        let id = self.ed.ls.custom.parse::<i64>().unwrap_or(0);
        if id > 0 { self.ed_set_song(id, format!("NG#{}", id)); }
    }
    fn ed_ls_commit_normal(&mut self) {
        if self.ed.songs.is_empty() { return; }
        let n = self.ed.songs.len() as i64;
        self.ed.ls.idx = ((self.ed.ls.idx % n) + n) % n;
        let title = self.ed.songs[self.ed.ls.idx as usize].clone();
        self.ed_set_song(-(self.ed.ls.idx + 1), title);
    }
    fn ed_set_song(&mut self, id: i64, name: String) {
        self.ed.pending_song = Some((id, name));
        self.events.push(Action::Pop("EdSong", 0));
    }

    fn ed_ls_build(&mut self) {
        let sw = self.screen_width;
        let _ = sw;
        let (pw, ph) = (1000.0, 600.0);
        let root = self.ed_pop("ed_ls", D_LS, 0.55, 0.0, Action::Pop("EdLsBlur", 0));
        let d = D_LS + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_ls");
        self.pa_txt(root, "goldFont", 40.0, "Select Color", 0.0, -(ph / 2.0) + 42.0, 0.5, 0.5, 1.0, d, "ed_ls");
        // the eight colour channels
        for (i, (label, ch, x)) in COLOR_TARGETS.iter().enumerate() {
            self.pa_txt(root, "bigFont", 24.0, label, *x, -170.0 - 45.0, 0.5, 0.5, 1.0, d, "ed_ls");
            let c = self.ed_channel_color(*ch);
            let hex = ((c.0 as i64) << 16) | ((c.1 as i64) << 8) | c.2 as i64;
            let sh = self.shape(Kind::RoundRect, x - 25.0 + 4.0, -170.0 - 25.0 + 5.0, 50.0, 50.0, 0x000000, 0.35, d, "ed_ls"); { let n = self.node(sh); n.radius = 8.0; n.parent = Some(root); }
            let zone = self.shape(Kind::Rect, *x, -170.0, 50.0, 50.0, 0, 0.0, d + 0.01, "ed_ls");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 50.0; n.height = 50.0; n.parent = Some(root); }
            let edge = self.shape(Kind::RoundRect, -25.0, -25.0, 50.0, 50.0, 0x000000, 1.0, d + 0.01, "ed_ls"); { let n = self.node(edge); n.radius = 8.0; n.parent = Some(zone); }
            let fill = self.shape(Kind::RoundRect, -23.0, -23.0, 46.0, 46.0, hex, 1.0, d + 0.02, "ed_ls"); { let n = self.node(fill); n.radius = 6.0; n.parent = Some(zone); }
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("EdLsColor", i));
        }
        // speed
        let left = -(pw / 2.0) + 90.0;
        self.pa_txt(root, "goldFont", 30.0, "Speed:", left, -262.0, 0.5, 0.5, 1.0, d, "ed_ls");
        let speed_key = self.ed.level.get_i("kA4", 0);
        let (si, sd) = SPEEDS.iter().enumerate().find(|(_, s)| s.0 == speed_key).map(|(i, s)| (i, *s)).unwrap_or((0, SPEEDS[0]));
        self.ed_obj_button(root, left + 5.0, -190.0, sd.2, 60.8, if si == 0 { 1.05 } else { 1.2 }, Action::Pop("EdLsPick", 0), d, "ed_ls");
        // mode
        self.pa_txt(root, "goldFont", 30.0, "Mode:", left, -105.0, 0.5, 0.5, 1.0, d, "ed_ls");
        let mode_key = self.ed.level.get_i("kA2", 0);
        let md = MODES.iter().find(|m| m.0 == mode_key).copied().unwrap_or(MODES[0]);
        if let Some(i) = self.ed_frame_img(root, md.2, left, -40.0, 0.88, d, "ed_ls") { self.make_bouncy(i, 0.88, Cond::Always, Action::Pop("EdLsPick", 1)); }
        // options
        self.pa_txt(root, "goldFont", 30.0, "Options:", left, 45.0, 0.5, 0.5, 1.0, d, "ed_ls");
        let ob = self.pa_img(root, "GJ_GameSheet03", "GJ_optionsBtn_001.png", left, 112.0, 0.7, d, "ed_ls");
        self.make_bouncy(ob, 0.7, Cond::Always, Action::Pop("EdSoOpen", 0));
        // background and ground art
        let right = (pw / 2.0) - 90.0;
        self.pa_txt(root, "goldFont", 30.0, "BG:", right, -262.0, 0.5, 0.5, 1.0, d, "ed_ls");
        let bg = self.ed.level.get_i("kA6", 1).max(1).max(1).min(59);
        self.ed_art_button(root, right, -190.0, &art_frame("bg", bg), 90.0, Action::Pop("EdLsPick", 2), d);
        self.pa_txt(root, "goldFont", 30.0, "G:", right, -112.0, 0.5, 0.5, 1.0, d, "ed_ls");
        let gr = self.ed.level.get_i("kA7", 1).max(1).min(22);
        self.ed_art_button(root, right, -40.0, &art_frame("g", gr), 90.0, Action::Pop("EdLsPick", 3), d);
        self.ed_ls_song(root, d);
        self.ed_ok(root, 0.0, (ph / 2.0) - 45.0, "OK", Action::Pop("EdLsOk", 0), d, "ed_ls");
    }

    /// `_makeCurrentSpeedPreview`: the object's frame scaled to a height
    fn ed_obj_button(&mut self, root: usize, x: f64, y: f64, id: i64, target_h: f64, mult: f64, action: Action, d: f64, tag: &'static str) {
        if let Some(f) = Self::ed_obj_frame(id) {
            if let Some(atlas) = crate::atlas_for_frame(&f) {
                let i = self.pa_img(root, &atlas, &f, x, y, 1.0, d, tag);
                let h = self.node(i).height.max(1.0);
                let sc = target_h * mult / h;
                self.node(i).sx = sc; self.node(i).sy = sc;
                self.make_bouncy(i, sc, Cond::Always, action);
                return;
            }
        }
        let t = self.pa_txt(root, "bigFont", 42.0, "?", x, y, 0.5, 0.5, 1.0, d, tag);
        self.make_bouncy(t, 1.0, Cond::Always, action);
    }

    /// `_makeEditorAtlasIconButton`: an icon scaled to a height
    fn ed_art_button(&mut self, root: usize, x: f64, y: f64, frame: &str, target_h: f64, action: Action, d: f64) {
        if let Some(i) = self.ed_frame_img(root, frame, x, y, 1.0, d, "ed_ls") {
            let h = self.node(i).height.max(1.0);
            let sc = target_h / h;
            self.node(i).sx = sc; self.node(i).sy = sc;
            self.make_bouncy(i, sc, Cond::Always, action);
        }
    }

    pub(crate) fn ed_channel_color(&self, ch: i64) -> (i64, i64, i64) {
        if let Some(c) = self.ed.level.color_channel(ch) { return c; }
        let c = crate::with_game(|g| g.cm.get_color(ch));
        (c.r as i64, c.g as i64, c.b as i64)
    }

    fn ed_ls_song(&mut self, root: usize, d: f64) {
        let (bx, by, bw, bh) = (-315.0, 40.0, 630.0, 175.0);
        let (tab_y, tab_w, text_y) = (by - 25.0, 110.0, by + bh / 2.0);
        let gold = self.fonts.get("goldFont").cloned().unwrap_or_default();
        let big = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let title_w = gold.width("Select Song:", 40.0);
        let group_w = title_w + 42.0 + tab_w + 13.0 + tab_w;
        let left = -(group_w / 2.0);
        let title_x = left + title_w;
        let (normal_x, custom_x) = (title_x + 42.0 + tab_w / 2.0, title_x + 42.0 + tab_w / 2.0 + tab_w + 13.0);
        self.pa_txt(root, "goldFont", 40.0, "Select Song:", title_x, tab_y - 2.0, 1.0, 0.5, 1.0, d, "ed_ls");
        let custom = self.ed.ls.cat == 1;
        for (x, label, which) in [(normal_x, "Normal", 0usize), (custom_x, "Custom", 1usize)] {
            let sel = (self.ed.ls.cat as usize) == which;
            let tex = self.sizes.get(if sel { "/GJ_button01" } else { "/GJ_button04" }).map_or(80.0, |s| s.0);
            let zone = self.shape(Kind::Rect, x, tab_y, tab_w, 40.0, 0, 0.0, d, "ed_ls");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = tab_w; n.height = 40.0; n.parent = Some(root); }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, tab_w, 40.0, 0, 1.0, d, "ed_ls");
            { let n = self.node(nine); n.frame = if sel { "GJ_button01" } else { "GJ_button04" }.into(); n.radius = (tex * 0.225).min(20.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
            self.pa_txt(zone, "bigFont", 22.0, label, -3.0, -3.0, 0.5, 0.5, 1.0, d + 0.01, "ed_ls");
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("EdLsTab", which));
        }
        let bg = self.shape(Kind::RoundRect, bx, by, bw, bh, 0x000000, 0.28, d, "ed_ls"); { let n = self.node(bg); n.radius = 25.0; n.parent = Some(root); }
        let (iw, ih) = (335.0, 62.0);
        if custom {
            let inp = self.shape(Kind::RoundRect, -iw / 2.0, text_y - ih / 2.0, iw, ih, 0x000000, 0.42, d + 0.01, "ed_ls"); { let n = self.node(inp); n.radius = 12.0; n.parent = Some(root); n.interactive = true; n.width = iw; n.height = ih; n.action = Action::Pop("EdLsFocus", 0); }
        }
        // the song line, shrunk to fit between the arrows or the input
        let (text, placeholder) = if custom {
            let focused = self.ed.ls.focused;
            if focused { (format!("{}|", self.ed.ls.custom), false) } else if self.ed.ls.custom.is_empty() { ("Newgrounds Song ID".to_string(), true) } else { (self.ed.ls.custom.clone(), false) }
        } else if self.ed.songs.is_empty() { ("No normal songs loaded".to_string(), false) } else {
            let n = self.ed.songs.len() as i64;
            let i = ((self.ed.ls.idx % n) + n) % n;
            (format!("{:02}: {}", i + 1, self.ed.songs[i as usize]), false)
        };
        let max_font = if custom { if !self.ed.ls.focused && self.ed.ls.custom.is_empty() { 30.0 } else { 42.0 } } else { 50.0 };
        let max_w = if custom { iw - 34.0 } else { (250.0 - 18.0 - 12.0 * 0.0) * 2.0 - 2.0 * 30.0 };
        let mut size = max_font; let mut scale = 1.0;
        while size >= 8.0 { if big.width(&text, size) <= max_w { break; } size -= 2.0; }
        if size < 8.0 { size = 8.0; let w = big.width(&text, 8.0); if w > max_w && w > 0.0 { scale = max_w / w; } }
        let t = self.pa_txt(root, "bigFont", size, &text, 0.0, text_y, 0.5, 0.5, scale, d + 0.02, "ed_ls");
        if placeholder { self.node(t).tint = 0x75aaf0; }
        if !custom {
            let a = self.pa_img(root, "GJ_GameSheet03", "edit_leftBtn_001.png", -250.0, text_y + 2.0, 1.2, d, "ed_ls");
            self.make_bouncy(a, 1.2, Cond::Always, Action::Pop("EdLsSong", 0));
            let b = self.pa_img(root, "GJ_GameSheet03", "edit_leftBtn_001.png", 250.0, text_y + 2.0, 1.2, d, "ed_ls");
            self.node(b).flip_x = true;
            self.make_bouncy(b, 1.2, Cond::Always, Action::Pop("EdLsSong", 1));
        }
    }

    // ---- actions ----
    pub fn ed_ls_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdSettings" => { self.ed_ls_open(); true }
            "EdLsBlur" => { self.ed_ls_blur(false); true }
            "EdLsOk" => { if self.ed.ls.cat == 1 { self.ed_ls_commit_custom(); } self.ed_ls_close(); true }
            "EdLsTab" => {
                let was_custom = self.ed.ls.cat == 1;
                self.ed.ls.cat = arg as u8; self.ed.ls.focused = false;
                if arg == 0 && was_custom { self.ed.ls.idx = 0; self.ed_ls_commit_normal(); }
                self.ed_ls_refresh(); true
            }
            "EdLsFocus" => { if self.ed.ls.cat == 1 { self.ed.ls.before = self.ed.ls.custom.clone(); self.ed.ls.focused = true; self.ed_ls_refresh(); } true }
            "EdLsSong" => {
                if !self.ed.songs.is_empty() { self.ed.ls.idx += if arg == 0 { -1 } else { 1 }; self.ed_ls_commit_normal(); }
                self.ed_ls_refresh(); true
            }
            "EdLsPick" => { self.ed_ho_open(arg as u8); true }
            "EdLsColor" => { self.ed_cp_open(COLOR_TARGETS[arg].1); true }
            "EdHoPick" => { self.ed_ho_pick(arg); true }
            "EdHoOk" => { self.ed_ho_close(); true }
            "EdSoOpen" => { self.ed_so_open(); true }
            "EdSoToggle" => { self.ed_so_toggle(arg); true }
            "EdSoClose" => { self.ed_so_close(); true }
            _ => false,
        }
    }

    // ---- the horizontal option picker ----
    fn ed_ho_open(&mut self, kind: u8) {
        self.ed_ho_close();
        let (title, keys, selected): (&str, Vec<i64>, i64) = match kind {
            0 => ("Select Speed", SPEEDS.iter().map(|s| s.0).collect(), self.ed.level.get_i("kA4", 0)),
            1 => ("Select Mode", MODES.iter().map(|m| m.0).collect(), self.ed.level.get_i("kA2", 0)),
            2 => ("Select Background", (1..=59).collect(), self.ed.level.get_i("kA6", 1).max(1)),
            _ => ("Select Ground", (1..=22).collect(), self.ed.level.get_i("kA7", 1).max(1)),
        };
        let art = kind >= 2;
        let (cols, rows) = if art { (10usize, (keys.len() + 9) / 10) } else { (keys.len(), 1) };
        let gap_x = if art { 76.0 } else if kind == 1 { 84.0 } else if kind == 0 { 150.0 } else { 142.0 };
        let gap_y = if art { 68.0 } else { 0.0 };
        let pw = if art { 860.0 } else { 760.0 };
        let ph = if art { (650.0f64).min(225.0 + rows as f64 * gap_y) - 45.0 } else { 340.0 };
        let root = self.ed_pop("ed_ho", D_HO, 0.25, 15.0, Action::Pop("EdNoop", 0));
        let d = D_HO + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, d, "ed_ho");
        self.pa_txt(root, "bigFont", if art { 52.0 } else { 60.0 }, title, 0.0, -(ph / 2.0) + 50.0, 0.5, 0.5, 1.0, d, "ed_ho");
        self.ed.ho = EdHo { kind: Some(kind), keys: keys.clone(), selected, nodes: Vec::new() };
        for (i, key) in keys.iter().enumerate() {
            let (row, col) = if art { (i / cols, i % cols) } else { (0, i) };
            let in_row = if art { cols.min(keys.len() - row * cols) } else { keys.len() };
            let ox = (col as f64 - (in_row as f64 - 1.0) / 2.0) * gap_x + if kind == 0 { -15.0 } else { 0.0 };
            let oy = if art { -((rows as f64 - 1.0) * gap_y) / 2.0 + row as f64 * gap_y + 16.0 - 20.0 } else { -2.0 };
            let node = match kind {
                0 => {
                    let sd = SPEEDS[i];
                    let target = 112.0 * if i == 0 { 0.8 } else { 1.0 };
                    self.ed_obj_ho(root, ox, oy, sd.2, target, d)
                }
                1 => self.ed_frame_img(root, MODES[i].2, ox, oy, 0.88, d, "ed_ho").map(|n| (n, 0.88)),
                _ => { let f = art_frame(if kind == 2 { "bg" } else { "g" }, *key); self.ed_frame_img(root, &f, ox, oy, 1.0, d, "ed_ho").map(|n| { let h = self.node(n).height.max(1.0); let sc = 48.0 / h; self.node(n).sx = sc; self.node(n).sy = sc; (n, sc) }) }
            };
            if let Some((n, sc)) = node {
                self.make_bouncy(n, sc, Cond::Always, Action::Pop("EdHoPick", i));
                self.ed.ho.nodes.push((n, sc));
            }
        }
        self.ed_ok(root, 0.0, (ph / 2.0) - 52.0, "OK", Action::Pop("EdHoOk", 0), d, "ed_ho");
        self.ed_ho_tints();
    }

    /// the speed portal frame scaled so it is `target_h` high
    fn ed_obj_ho(&mut self, root: usize, x: f64, y: f64, id: i64, target_h: f64, d: f64) -> Option<(usize, f64)> {
        let f = Self::ed_obj_frame(id)?; let atlas = crate::atlas_for_frame(&f)?;
        let i = self.pa_img(root, &atlas, &f, x, y, 1.0, d, "ed_ho");
        let h = self.node(i).height.max(1.0);
        let sc = target_h / h;
        self.node(i).sx = sc; self.node(i).sy = sc;
        Some((i, sc))
    }

    fn ed_ho_tints(&mut self) {
        let sel = self.ed.ho.selected;
        let nodes = self.ed.ho.nodes.clone(); let keys = self.ed.ho.keys.clone();
        for (k, (n, _)) in nodes.into_iter().enumerate() { if let Some(node) = self.nodes.get_mut(n).and_then(|n| n.as_mut()) { node.tint = if keys[k] == sel { -1 } else { 0x666666 }; } }
    }

    fn ed_ho_pick(&mut self, i: usize) {
        let Some(kind) = self.ed.ho.kind else { return };
        let key = self.ed.ho.keys[i];
        self.ed.ho.selected = key;
        match kind {
            0 => self.ed.level.set_str("kA4", &key.to_string()),
            1 => self.ed.level.set_str("kA2", &key.to_string()),
            2 => { self.ed.level.set_str("kA6", &key.to_string()); self.events.push(Action::Pop("EdArt", 0)); }
            _ => { self.ed.level.set_str("kA7", &key.to_string()); self.events.push(Action::Pop("EdArt", 1)); }
        }
        self.ed.dirty = true;
        self.ed_ho_tints();
        self.ed_ls_refresh_keep_ho();
    }

    /// the level settings popup behind the picker shows the new choice
    fn ed_ls_refresh_keep_ho(&mut self) {
        if !self.ed.ls_open { return; }
        self.kill_tag("ed_ls"); self.ed_ls_build();
    }

    fn ed_ho_close(&mut self) { self.kill_tag("ed_ho"); self.ed.ho = EdHo::default(); }

    // ---- start options ----
    fn ed_so_open(&mut self) {
        if self.ed.so_open { return; }
        self.ed.so_open = true;
        let root = self.ed_pop("ed_so", D_SO, 0.25, 15.0, Action::Pop("EdNoop", 0));
        let d = D_SO + 0.1;
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, 520.0, 420.0, d, "ed_so");
        let c = self.pa_img(root, "GJ_WebSheet", "GJ_closeBtn_001.png", -(520.0 / 2.0) + 10.0, -(420.0 / 2.0) + 10.0, 0.8, d, "ed_so");
        self.node(c).hit_scale = 1.0;
        self.make_bouncy(c, 0.8, Cond::Always, Action::Pop("EdSoClose", 0));
        self.ed.so_checks.clear();
        for (i, (label, key)) in [("Mini Mode", "kA3"), ("Flip Gravity", "kA11"), ("Dual Mode", "kA8"), ("Mirror Mode", "kA28")].iter().enumerate() {
            let y = -130.0 + i as f64 * 80.0;
            let on = self.ed.level.get_i(key, 0) == 1;
            let check = self.pa_img(root, "GJ_GameSheet03", if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, -150.0, y, 0.9, d, "ed_so");
            self.make_bouncy(check, 0.9, Cond::Always, Action::Pop("EdSoToggle", i));
            let t = self.pa_txt(root, "bigFont", 28.0, label, -150.0 + 48.0, y - 2.0, 0.0, 0.5, 1.0, d, "ed_so");
            self.node(t).interactive = true; self.node(t).action = Action::Pop("EdSoToggle", i);
            self.ed.so_checks.push((check, key));
        }
    }

    fn ed_so_toggle(&mut self, i: usize) {
        let Some(&(node, key)) = self.ed.so_checks.get(i) else { return };
        let on = self.ed.level.get_i(key, 0) == 1;
        self.ed.level.set_str(key, if on { "0" } else { "1" });
        if let Some(n) = self.nodes.get_mut(node).and_then(|n| n.as_mut()) { n.frame = if on { "GJ_checkOff_001.png" } else { "GJ_checkOn_001.png" }.into(); }
    }

    fn ed_so_close(&mut self) { self.kill_tag("ed_so"); self.ed.so_open = false; }
}
