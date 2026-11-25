//! The level editor scene (level-editor.js): build/edit/delete tabs, object palette, placing, selecting, moving, rotating,
//! flipping, duplicating, deleting, camera pan/zoom, save/exit/playtest hooks. The editor works on an `EdLevel`; the view is a
//! `Game` rebuilt from it whenever something changed (`ed_frame`).
use crate::editor::*;
use crate::js::parse_int;
use crate::jsonread::Json;
use crate::level::{self, Level};
use crate::ui::*;
use crate::visual::def_of;

const GUI: f64 = 1000.0;

#[derive(Default)]
pub struct EditorState {
    pub on: bool,
    pub level: EdLevel,
    pub created_id: String,
    pub cam_x: f64, pub cam_y: f64, pub zoom: f64,
    /// 0 build, 1 edit, 2 delete
    pub tab: u8,
    pub category: usize,
    pub page: usize,
    pub selected_id: i64,
    pub sel: Vec<usize>,
    pub sel_tint: f64,
    pub dirty: bool,
    pub swipe: bool,
    pub down: Option<(f64, f64, f64, f64)>,
    pub dragging: bool,
    pub box_start: Option<(f64, f64)>,
    pub box_moved: bool,
    pub last_cell: (i64, i64),
    pub paused: bool,
    pub zoom_text: usize,
    pub zoom_clock: f64,
    pub tab_nodes: Vec<usize>,
    pub btn_copy: usize, pub btn_edit: usize, pub btn_group: usize, pub btn_desel: usize, pub btn_trash: usize, pub btn_del_grid: Option<usize>,
    pub timeline_thumb: usize,
    pub box_node: usize,
    pub swipe_node: usize,
    // playtest
    pub pt: bool, pub pt_paused: bool, pub pt_press: bool,
    pub pm: bool, pub pm_glow_check: usize, pub glow: bool,
    pub ls: crate::editor_ui::EdLs, pub ls_open: bool, pub ho: crate::editor_ui::EdHo, pub so_open: bool, pub so_checks: Vec<(usize, &'static str)>,
    pub layer_idx: usize, pub layer_nodes: [usize; 6],
    pub songs: Vec<String>, pub song_id: i64, pub pending_song: Option<(i64, String)>,
    pub fields: crate::editor_fields::EdFields,
    pub cp: crate::editor_color::EdCp,
    pub gp: crate::editor_objects::EdGp, pub trig: crate::editor_trigger::EdTrig, pub ct_focus: bool, pub ct_input: String, pub ct_text: usize, pub sp_root: usize, pub sp_checks: Vec<(usize, &'static str)>,
    pub pt_saved: (f64, f64, f64),
    pub pt_hidden: Vec<(usize, bool)>,
    pub pt_btns: [usize; 3],
    /// trail segments and death marks for the host to draw: (kind 0 trail / 1 death cross, x0, y0, x1, y1, colour), y already in screen space
    pub marks: Vec<[f64; 6]>,
    pub last_trail: [Option<(f64, f64)>; 2],
    /// bumped on start / stop / pause / resume so the host can start and stop the music (value, music offset ms)
    pub pt_event: u32, pub pt_music: f64,
}

const CATEGORIES: [(&str, &str, &[&str]); 8] = [
    ("blocks", "tab1", &["solid", "soliddeco", "dblock"]),
    ("slopes", "tab6", &["slope"]),
    ("hazards", "tab2", &["hazard", "spike"]),
    ("orbs", "tab3", &["ring", "pad", "portal", "speed", "coin"]),
    ("pixel", "tab7", &["pixel"]),
    ("particle", "tab8", &["particle"]),
    ("deco", "tab4", &["deco"]),
    ("triggers", "tab5", &["trigger"]),
];

/// (dx, dy, icon, angle degrees, scale)
const MOVES: [(f64, f64, &str, f64, f64); 20] = [
    (0.0, -3.0, "edit_upBtn_001.png", 0.0, 1.0), (0.0, 3.0, "edit_upBtn_001.png", 180.0, 1.0), (-3.0, 0.0, "edit_upBtn_001.png", 270.0, 1.0), (3.0, 0.0, "edit_upBtn_001.png", 90.0, 1.0),
    (0.0, -60.0, "edit_upBtn2_001.png", 0.0, 1.0), (0.0, 60.0, "edit_upBtn2_001.png", 180.0, 1.0), (-60.0, 0.0, "edit_upBtn2_001.png", 270.0, 1.0), (60.0, 0.0, "edit_upBtn2_001.png", 90.0, 1.0),
    (0.0, -300.0, "edit_upBtn3_001.png", 0.0, 1.0), (0.0, 300.0, "edit_upBtn3_001.png", 180.0, 1.0), (-300.0, 0.0, "edit_upBtn3_001.png", 270.0, 1.0), (300.0, 0.0, "edit_upBtn3_001.png", 90.0, 1.0),
    (0.0, -1.0, "edit_upBtn_001.png", 0.0, 0.7), (0.0, 1.0, "edit_upBtn_001.png", 180.0, 0.7), (-1.0, 0.0, "edit_upBtn_001.png", 270.0, 0.7), (1.0, 0.0, "edit_upBtn_001.png", 90.0, 0.7),
    (0.0, -30.0, "edit_upBtn5_001.png", 0.0, 1.0), (0.0, 30.0, "edit_upBtn5_001.png", 180.0, 1.0), (-30.0, 0.0, "edit_upBtn5_001.png", 270.0, 1.0), (30.0, 0.0, "edit_upBtn5_001.png", 90.0, 1.0),
];

fn fnum(v: f64) -> f64 { if v.is_nan() { 0.0 } else { v } }

impl Ui {
    // ---- life cycle ----
    /// Opens the editor on a level string (the way `loadLevel` prepares `window.levelObjects`: teleport portals folded, coins filtered).
    pub fn ed_open(&mut self, level_string: &str, created_id: &str) {
        let parsed = EdLevel::parse(level_string);
        let mut ed = EdLevel { settings: parsed.settings, objects: Vec::new() };
        // `_normalizeTeleportPortals`
        let mut exits: Vec<EdObj> = Vec::new();
        for o in parsed.objects { if o.o.id == Some(749) { exits.push(o); } else { ed.objects.push(o); } }
        for exit in &exits {
            let exit_x = fnum(exit.o.x);
            let mut best: Option<usize> = None; let mut bd = f64::INFINITY;
            for (i, c) in ed.objects.iter().enumerate() {
                if c.o.id != Some(747) { continue; }
                let d = (fnum(c.o.x) - exit_x).abs();
                if d < bd { bd = d; best = Some(i); }
            }
            if let Some(i) = best {
                let enter_y = fnum(ed.objects[i].o.y);
                let ey = exit.o.y;
                let exit_y = if ey == 0.0 || ey.is_nan() { enter_y } else { ey };
                let v = js_num(exit_y - enter_y);
                ed.objects[i].raw.set("54", v.clone());
                ed.objects[i].o.raw.insert("54".into(), v);
            }
        }
        // coin filter for a non-official level: no secret coins, at most 3 user coins
        let mut user = 0;
        ed.objects.retain(|o| match o.o.id { Some(142) => false, Some(1329) => { if user >= 3 { false } else { user += 1; true } } _ => true });
        // the editor replaces the main menu / HUD / menus entirely
        for slot in self.nodes.iter_mut() { *slot = None; }
        self.menu_active = false; self.starting = false;
        self.ed = EditorState { on: true, level: ed, created_id: created_id.to_string(), zoom: 1.0, selected_id: 1, sel_tint: 0x00ff00 as f64, dirty: true, category: 0, last_cell: (-1, -1), ..Default::default() };
        self.ed_build_gui();
        self.ed_build_grid();
    }

    pub fn ed_close(&mut self) {
        self.kill_tag("ed_");
        self.ed.on = false;
    }

    pub fn ed_level_string(&self) -> String { self.ed.level.serialize() }

    fn ed_level_width(&self) -> f64 {
        let mut far = 0.0f64;
        for o in &self.ed.level.objects { let w = (fnum(o.o.x) - 15.0) * 2.0; if w > far { far = w; } }
        self.screen_width.max(far + self.screen_width / 2.0)
    }

    /// One editor frame: arrows move the camera (`keys` bit 0 left, 1 right, 2 up, 3 down), the view is rebuilt when the level changed.
    pub fn ed_frame(&mut self, dt: f64, keys: u32) {
        if !self.ed.on { return; }
        if !self.ed.paused && !self.ed.playtest_active() {
            if keys & 1 != 0 { self.ed.cam_x -= 15.0; } else if keys & 2 != 0 { self.ed.cam_x += 15.0; }
            if keys & 4 != 0 { self.ed.cam_y -= 15.0; } else if keys & 8 != 0 { self.ed.cam_y += 15.0; }
        }
        if self.ed.pt { self.ed_pt_frame(dt, keys & 16 != 0 || self.ed.pt_press); return; }
        if self.ed.dirty { self.ed_rebuild_view(); self.ed.dirty = false; }
        let (cx, cy, z) = (self.ed.cam_x, self.ed.cam_y, self.ed.zoom);
        let sel = self.ed.sel.clone(); let tint = self.ed.sel_tint; let glow = self.ed.glow;
        let dim = self.ed_layer_dim();
        crate::with_game(|g| { g.fx.ed_glow = glow; g.editor_set_dim(&dim); g.editor_set_selection(&sel, tint); g.editor_view(cx, cy, z); });
        // zoom text fades out
        if self.ed.zoom_clock > 0.0 {
            self.ed.zoom_clock -= dt;
            let a = if self.ed.zoom_clock > 500.0 { 1.0 } else { (self.ed.zoom_clock / 500.0).max(0.0) };
            let zt = self.ed.zoom_text; if let Some(n) = self.nodes.get_mut(zt).and_then(|n| n.as_mut()) { n.alpha = a; }
        }
        // timeline thumb follows the camera
        let w = self.screen_width / 3.0;
        let start = self.screen_width / 2.0 - w / 2.0;
        let pct = (self.ed.cam_x / self.ed_level_width()).max(0.0).min(1.0);
        let th = self.ed.timeline_thumb; if let Some(n) = self.nodes.get_mut(th).and_then(|n| n.as_mut()) { if !n.pressed { n.x = start + pct * w; } }
    }

    fn ed_rebuild_view(&mut self) {
        let level = Level { settings: self.ed.level.settings.clone(), objects: self.ed.level.objects.iter().map(|o| o.o.clone()).collect() };
        crate::editor_rebuild(&level);
    }

    // ---- GUI ----
    fn ed_img(&mut self, atlas: &str, frame: &str, x: f64, y: f64, scale: f64) -> usize {
        let i = self.image(atlas, frame, x, y, GUI, scale);
        self.node(i).tag = "ed_g";
        i
    }

    fn ed_build_gui(&mut self) {
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        // top right
        let st = self.ed_img("GJ_GameSheet03", "GJ_optionsBtn02_001.png", sw - 40.0 - 87.0, 40.0, 1.0);
        self.make_bouncy(st, 1.0, Cond::Always, Action::Pop("EdSettings", 0));
        let pause = self.ed_img("GJ_GameSheet03", "GJ_pauseBtn_001.png", sw - 40.0, 40.0, 1.0);
        self.make_bouncy(pause, 1.0, Cond::Always, Action::Pop("EdPause", 0));
        let trash = self.ed_img("GJ_GameSheet03", "GJ_trashBtn_001.png", 50.0, 40.0, 1.0);
        self.make_bouncy(trash, 0.9, Cond::Always, Action::Pop("EdDelete", 0));
        self.node(trash).sx = 0.9; self.node(trash).sy = 0.9;
        self.ed.btn_trash = trash;
        // toolbox
        let bg = self.shape(Kind::Rect, 0.0, sh - 200.0, sw, 200.0, 0, 0.75, GUI - 0.5, "ed_g");
        { let n = self.node(bg); n.interactive = true; n.width = sw; n.height = 200.0; }
        let l = self.ed_img("GJ_GameSheet03", "GJ_arrow_02_001.png", cx - 330.0, sh - 100.0, 0.7);
        self.make_bouncy(l, 0.8, Cond::Always, Action::Pop("EdPage", 0)); self.node(l).tag = "ed_arrow";
        let r = self.ed_img("GJ_GameSheet03", "GJ_arrow_02_001.png", cx + 330.0, sh - 100.0, 0.7);
        self.node(r).flip_x = true; self.make_bouncy(r, 0.8, Cond::Always, Action::Pop("EdPage", 1)); self.node(r).tag = "ed_arrow";
        // swipe
        let sz = self.shape(Kind::Rect, cx + 450.0, sh - 100.0, 64.0, 64.0, 0, 0.0, GUI, "ed_g");
        { let n = self.node(sz); n.ox = 0.5; n.oy = 0.5; n.width = 64.0; n.height = 64.0; }
        let sb = self.shape(Kind::Nine, 0.0, 0.0, 64.0, 64.0, 0, 1.0, GUI, "ed_g");
        { let n = self.node(sb); n.frame = "GJ_button01".into(); n.radius = 24.0 * 0.8; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(sz) ; }
        let stx = self.text_node("bigFont", 16.0, "swipe", -1.0, -2.0, GUI + 0.1, 0.5, 0.5, 1.0); { let n = self.node(stx); n.tag = "ed_g"; n.parent = Some(sz); }
        self.make_bouncy(sz, 1.0, Cond::Always, Action::Pop("EdSwipe", 0));
        self.ed.btn_del_grid = None;
        self.ed.swipe_node = sb;
        // tabs
        let (tab_x, tab_y, ts) = (cx - 475.0, sh - 100.0, 1.15);
        let (gx, gy) = (80.0 * ts, 88.0 * ts);
        self.ed_img("GJ_GameSheet03", "edit_vLine_001.png", tab_x + gx / 2.0 + 42.0 * ts + 12.0, sh - 100.0, 1.0);
        self.ed_img("GJ_GameSheet03", "edit_vLine_001.png", cx + 375.0, sh - 100.0, 1.0);
        self.ed.tab_nodes.clear();
        for (i, (x, y)) in [(tab_x - gx / 2.0, tab_y - gy / 2.0), (tab_x + gx / 2.0 + 5.0, tab_y - gy / 2.0), (tab_x - gx / 2.0, tab_y + gy / 2.0)].into_iter().enumerate() {
            let id = ["build", "edit", "delete"][i];
            let b = self.ed_img("GJ_EditorSheet01", &format!("{}BtnOff.png", id), x, y, ts);
            self.make_bouncy(b, ts, Cond::Always, Action::Pop("EdTab", i));
            self.ed.tab_nodes.push(b);
        }
        // side buttons (sw-48, 120)
        let (sx, sy) = (sw - 48.0, 120.0);
        self.ed.btn_copy = self.ed_img("GJ_GameSheet03", "GJ_duplicateObjectBtn2_001.png", sx, sy, 1.0);
        self.make_bouncy(self.ed.btn_copy, 1.0, Cond::Always, Action::Pop("EdDuplicate", 0));
        self.ed.btn_edit = self.ed_img("GJ_GameSheet03", "GJ_editObjBtn3_001.png", sx, sy + 75.0, 1.0);
        self.make_bouncy(self.ed.btn_edit, 1.0, Cond::Always, Action::Pop("EdEditObject", 0));
        self.ed.btn_group = self.ed_img("GJ_GameSheet03", "GJ_groupIDBtn2_001.png", sx - 75.0, sy + 75.0, 1.0);
        self.make_bouncy(self.ed.btn_group, 1.0, Cond::Always, Action::Pop("EdGroup", 0));
        self.ed.btn_desel = self.ed_img("GJ_GameSheet03", "GJ_deSelBtn2_001.png", sx, sy + 150.0, 1.0);
        self.make_bouncy(self.ed.btn_desel, 1.0, Cond::Always, Action::Pop("EdDeselect", 0));
        // zoom
        let zi = self.ed_img("GJ_GameSheet03", "GJ_zoomInBtn_001.png", 48.0, sh / 2.0 - 20.0, 0.9);
        self.make_bouncy(zi, 0.9, Cond::Always, Action::Pop("EdZoom", 1));
        let zo = self.ed_img("GJ_GameSheet03", "GJ_zoomOutBtn_001.png", 48.0, sh / 2.0 + 55.0, 0.9);
        self.make_bouncy(zo, 0.9, Cond::Always, Action::Pop("EdZoom", 0));
        // the layer selector under the side buttons (`_editorLayerSelector`)
        {
            let (lx, ly) = (sw - 48.0 - 75.0, 120.0 + 275.0);
            let bg = self.shape(Kind::RoundRect, lx - 30.0, ly - 22.0, 60.0, 43.0, 0x000000, 0.4, GUI, "ed_g"); self.node(bg).radius = 10.0;
            let t = self.text_node("bigFont", 25.0, "All", lx, ly, GUI + 0.1, 0.5, 0.5, 1.0); self.node(t).tag = "ed_g";
            let first = self.ed_img("GJ_GameSheet03", "GJ_arrow_02_001.png", lx - 90.0, ly, 0.45); self.node(first).alpha = 0.5; self.make_bouncy(first, 0.42, Cond::Always, Action::Pop("EdLayer", 0));
            let prev = self.ed_img("GJ_GameSheet03", "GJ_arrow_03_001.png", lx - 55.0, ly, 0.55); self.make_bouncy(prev, 0.55, Cond::Always, Action::Pop("EdLayer", 1));
            let next = self.ed_img("GJ_GameSheet03", "GJ_arrow_03_001.png", lx + 55.0, ly, 0.55); self.node(next).flip_x = true; self.make_bouncy(next, 0.55, Cond::Always, Action::Pop("EdLayer", 2));
            let last = self.ed_img("GJ_GameSheet03", "GJ_arrow_02_001.png", lx + 90.0, ly, 0.45); self.node(last).flip_x = true; self.node(last).alpha = 0.5; self.make_bouncy(last, 0.42, Cond::Always, Action::Pop("EdLayer", 3));
            self.ed.layer_nodes = [bg, t, first, prev, next, last];
        }
        // playtest
        let pp = self.ed_img("GJ_GameSheet03", "GJ_playEditorBtn_001.png", 48.0, sh / 2.0 - 110.0, 1.1);
        self.make_bouncy(pp, 1.1, Cond::Always, Action::Pop("EdPlaytest", 0));
        { let n = self.node(pp); n.depth = GUI + 500.0; n.tag = "ed_pt"; }
        let pa = self.image("GJ_GameSheet03", "GJ_pauseEditorBtn_001.png", 48.0, sh / 2.0 - 110.0, GUI + 500.0, 1.1);
        self.make_bouncy(pa, 1.1, Cond::Always, Action::Pop("EdPtPause", 0));
        { let n = self.node(pa); n.tag = "ed_pt"; n.visible = false; }
        let st = self.image("GJ_GameSheet03", "GJ_stopEditorBtn_001.png", 138.0, sh / 2.0 - 110.0, GUI + 500.0, 1.1);
        self.make_bouncy(st, 1.1, Cond::Always, Action::Pop("EdPtStop", 0));
        { let n = self.node(st); n.tag = "ed_pt"; n.visible = false; }
        self.ed.pt_btns = [pp, pa, st];
        let zt = self.text_node("bigFont", 40.0, "Zoom: 1.00x", cx, 80.0, GUI + 1000.0, 0.5, 0.5, 1.0);
        { let n = self.node(zt); n.tag = "ed_g"; n.alpha = 0.0; }
        self.ed.zoom_text = zt;
        // timeline
        let (tw, ty) = (sw / 3.0, 40.0);
        let gh = self.sizes.get("/slidergroove2").map_or(1.0, |s| s.1);
        let gw = self.sizes.get("/slidergroove2").map_or(1.0, |s| s.0);
        let g = self.image("", "slidergroove2", cx, ty, GUI + 500.0, 1.0);
        { let n = self.node(g); n.tag = "ed_g"; n.sx = tw / gw; n.sy = 26.0 / gh; }
        let th = self.image("", "GJ_moveBtn", cx - tw / 2.0, ty, GUI + 501.0, 0.4);
        let dummy = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, GUI, "ed_g");
        { let n = self.node(th); n.tag = "ed_g"; n.interactive = true; n.hit_scale = 1.5; n.drag = Some((cx - tw / 2.0, cx + tw / 2.0, dummy, 9, 0.0)); }
        self.ed.timeline_thumb = th;
        // box select rectangle
        let bx = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0x00ff00, 0.0, GUI + 1501.0, "ed_g");
        self.ed.box_node = bx;
        self.ed_update_tab_visuals();
        self.ed_update_buttons();
    }

    fn ed_update_tab_visuals(&mut self) {
        for (i, id) in ["build", "edit", "delete"].iter().enumerate() {
            let on = self.ed.tab as usize == i;
            let nd = self.ed.tab_nodes[i];
            self.node(nd).frame = format!("{}Btn{}.png", id, if on { "On" } else { "Off" });
        }
    }

    pub fn ed_update_buttons(&mut self) {
        let has = !self.ed.sel.is_empty();
        let trigger = self.ed.sel.len() == 1 && self.ed.level.objects.get(self.ed.sel[0]).and_then(|o| o.o.id).and_then(def_of).map_or(false, |d| d.get_str("type") == Some("trigger"));
        let mut set = |me: &mut Ui, i: usize, on: bool| { if let Some(n) = me.nodes.get_mut(i).and_then(|n| n.as_mut()) { n.alpha = if on { 1.0 } else { 0.5 }; n.interactive = on; } };
        let (c, e, g, d, t) = (self.ed.btn_copy, self.ed.btn_edit, self.ed.btn_group, self.ed.btn_desel, self.ed.btn_trash);
        set(self, c, has); set(self, e, trigger); set(self, g, has); set(self, d, has); set(self, t, has);
        if let Some(dg) = self.ed.btn_del_grid { set(self, dg, has); }
    }

    // ---- palette ----
    fn ed_palette_ids(&self) -> Vec<i64> {
        let (_, _, types) = CATEGORIES[self.ed.category];
        let slopes = CATEGORIES[self.ed.category].0 == "slopes";
        let mut out = Vec::new();
        for i in 1..=1329i64 {
            if i == 749 { continue; }
            if CATEGORIES[self.ed.category].0 == "orbs" && i == 142 { continue; }
            let Some(d) = def_of(i) else { continue };
            let ty = d.get_str("type").unwrap_or("");
            let mut ok = types.contains(&ty);
            if !ok && slopes {
                let mut frames: Vec<&str> = Vec::new();
                if let Some(f) = d.get_str("frame") { frames.push(f); }
                if let Some(f) = d.get_str("glow_frame") { frames.push(f); }
                if let Some(ch) = d.get("children").and_then(Json::arr) { for c in ch { if let Some(f) = c.get_str("frame") { frames.push(f); } } }
                ok = frames.iter().any(|f| f.to_lowercase().contains("slope"));
            }
            if ok { out.push(i); }
        }
        out
    }

    fn ed_total_pages(&self) -> usize {
        match self.ed.tab { 0 => (self.ed_palette_ids().len() + 11) / 12, 1 => 3, _ => 1 }
    }

    pub fn ed_build_grid(&mut self) {
        self.kill_tag("ed_grid");
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        let tab = self.ed.tab;
        let pages = self.ed_total_pages().max(1);
        if self.ed.page >= pages { self.ed.page = 0; }
        let page = self.ed.page;
        let show_arrows = tab != 2;
        for slot in self.nodes.iter_mut().flatten() { if slot.tag == "ed_arrow" { slot.visible = show_arrows; slot.interactive = show_arrows; } }
        if tab == 0 {
            let sp = 85.0;
            let start = -((CATEGORIES.len() as f64 - 1.0) * sp) / 2.0;
            for (i, c) in CATEGORIES.iter().enumerate() {
                let sel = self.ed.category == i;
                let b = self.image("", c.1, cx + start + i as f64 * sp, sh - 218.0, GUI, 0.12);
                { let n = self.node(b); n.tag = "ed_grid"; n.tint = if sel { 0x888888 } else { 0xffffff }; n.alpha = if sel { 1.0 } else { 0.33 }; }
                self.make_bouncy(b, 0.12, Cond::Always, Action::Pop("EdCategory", i));
            }
        }
        if tab != 2 {
            let (ds, dy) = (18.0, sh - 10.0);
            let start = cx - ((pages as f64 - 1.0) * ds) / 2.0;
            for i in 0..pages {
                let c = self.shape(Kind::Circle, start + i as f64 * ds, dy, 4.0, 4.0, if i == page { 0xffffff } else { 0x888888 }, 1.0, GUI, "ed_grid");
                let _ = c;
            }
        }
        let (px, py, cols) = (90.0, 80.0, 6usize);
        let (sx, sy) = (cx - ((cols as f64 - 1.0) * px) / 2.0, sh - 140.0);
        // items of this page
        enum Item { Obj(i64), Move(usize), Flip(u8), Rot(f64, u8), Del }
        let mut items: Vec<Item> = Vec::new();
        match tab {
            0 => { let ids = self.ed_palette_ids(); for id in ids { items.push(Item::Obj(id)); } }
            1 => {
                for i in 0..MOVES.len() { items.push(Item::Move(i)); }
                items.push(Item::Flip(0)); items.push(Item::Flip(1));
                items.push(Item::Rot(90.0, 0)); items.push(Item::Rot(-90.0, 1)); items.push(Item::Rot(45.0, 2)); items.push(Item::Rot(-45.0, 3));
            }
            _ => items.push(Item::Del),
        }
        let page_items: Vec<Item> = if tab == 2 { items } else { items.into_iter().skip(page * 12).take(12).collect() };
        for (index, item) in page_items.into_iter().enumerate() {
            let (col, row) = (index % cols, index / cols);
            let (x, y) = (sx + col as f64 * px, sy + row as f64 * py);
            match item {
                Item::Obj(id) => {
                    let zone = self.shape(Kind::Rect, x, y, 60.0, 60.0, 0, 0.0, GUI, "ed_grid");
                    { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 60.0; n.height = 60.0; }
                    let bg = self.image("GJ_GameSheet03", "GJ_checkOff_001.png", 0.0, 0.0, GUI, 1.2);
                    let picked = self.ed.selected_id == id;
                    { let n = self.node(bg); n.tag = "ed_grid"; n.parent = Some(zone); if picked { n.tint = 0x888888; } }
                    self.ed_preview(id, zone);
                    self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("EdSelectObj", id as usize));
                }
                Item::Move(i) => {
                    let m = MOVES[i];
                    let zone = self.ed_action_button(x, y, m.2, m.3, m.4, false, Action::Pop("EdMove", i));
                    let _ = zone;
                }
                Item::Flip(k) => { let (icon, _) = if k == 0 { ("edit_flipYBtn_001.png", 0) } else { ("edit_flipXBtn_001.png", 1) }; self.ed_action_button(x, y, icon, 90.0, 0.8, false, Action::Pop("EdFlip", k as usize)); }
                Item::Rot(deg, k) => {
                    let (icon, flip, scale) = match k { 0 => ("edit_ccwBtn_001.png", true, 0.8), 1 => ("edit_ccwBtn_001.png", false, 0.8), 2 => ("edit_rotate45rBtn_001.png", false, 0.85), _ => ("edit_rotate45lBtn_001.png", false, 0.85) };
                    let _ = deg;
                    self.ed_action_button(x, y, icon, 0.0, scale, flip, Action::Pop("EdRotate", k as usize));
                }
                Item::Del => {
                    let zone = self.shape(Kind::Rect, x, y, 60.0, 60.0, 0, 0.0, GUI, "ed_grid");
                    { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 60.0; n.height = 60.0; }
                    let bg = self.image("GJ_GameSheet03", "GJ_checkOff_001.png", 0.0, 0.0, GUI, 1.2); { let n = self.node(bg); n.tag = "ed_grid"; n.parent = Some(zone); }
                    let ic = self.image("GJ_GameSheet03", "edit_delBtn_001.png", 0.0, 0.0, GUI + 0.1, 1.0); { let n = self.node(ic); n.tag = "ed_grid"; n.parent = Some(zone); }
                    self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("EdDelete", 0));
                    self.ed.btn_del_grid = Some(zone);
                }
            }
        }
        self.ed_update_buttons();
    }

    fn ed_action_button(&mut self, x: f64, y: f64, icon: &str, angle: f64, scale: f64, flip_x: bool, action: Action) -> usize {
        let zone = self.shape(Kind::Rect, x, y, 74.0, 74.0, 0, 0.0, GUI, "ed_grid");
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 74.0; n.height = 74.0; n.sx = 0.92; n.sy = 0.92; }
        let b = self.image("", "GJ_button01", 0.0, 0.0, GUI, 1.0); { let n = self.node(b); n.tag = "ed_grid"; n.parent = Some(zone); }
        let ic = self.image("GJ_GameSheet03", icon, 0.0, 0.0, GUI + 0.1, scale);
        { let n = self.node(ic); n.tag = "ed_grid"; n.parent = Some(zone); n.rot = angle.to_radians(); n.flip_x = flip_x; }
        self.make_bouncy(zone, 0.92, Cond::Always, action);
        zone
    }

    /// `_createEditorObjectButtonPreview`, simplified: the object's frames (main, overlay, children) shrunk into the button.
    fn ed_preview(&mut self, id: i64, parent: usize) {
        let Some(d) = def_of(id) else { return };
        let mut frame = d.get_str("frame").map(String::from);
        if let Some(rf) = d.get("randomFrames").and_then(Json::arr) { if let Some(f) = rf.first().and_then(|v| v.str()) { frame = Some(f.to_string()); } }
        let frame = frame.filter(|f| !f.is_empty());
        let Some(frame) = frame else {
            if d.get("textObject").is_some() { let t = self.text_node("bigFont", 60.0, "A", -3.0, -3.0, GUI + 0.1, 0.5, 0.5, 0.85); let n = self.node(t); n.tag = "ed_grid"; n.parent = Some(parent); }
            return;
        };
        let mut parts: Vec<(String, f64, f64, f64)> = Vec::new(); // frame, dx, dy, depth
        let ty = d.get_str("type").unwrap_or("");
        if (ty == "portal" || ty == "speed") && frame.contains("_front_") { parts.push((frame.replace("_front_", "_back_"), 0.0, 0.0, -0.005)); }
        parts.push((frame.clone(), 0.0, 0.0, 0.0));
        if ty == "solid" || ty == "hazard" { parts.push((frame.replace("_001.png", "_2_001.png"), 0.0, 0.0, 0.002)); }
        if let Some(ch) = d.get("children").and_then(Json::arr) {
            for c in ch {
                if c.get("portalGuide").is_some() || c.get("orbGuide").is_some() { continue; }
                let Some(f) = c.get_str("frame") else { continue };
                let (dx, dy) = (c.get_num("dx").unwrap_or(0.0), c.get_num("dy").unwrap_or(0.0));
                let (dx, dy) = if c.get("localDx").is_some() || c.get("localDy").is_some() { (c.get_num("localDx").unwrap_or(0.0), c.get_num("localDy").unwrap_or(0.0)) } else { (dx, dy) };
                parts.push((f.to_string(), dx, dy, if c.get_num("z").unwrap_or(-1.0) < 0.0 { -0.003 } else { 0.001 }));
            }
        }
        parts.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
        let ring = ty == "ring";
        let mut made: Vec<(usize, f64, f64, f64, f64)> = Vec::new();
        for (f, dx, dy, _) in parts {
            let Some(atlas) = crate::atlas_for_frame(&f) else { continue };
            let sc = if ring { 0.75 } else { 1.0 };
            let i = self.image(&atlas, &f, dx, dy, GUI + 0.1, sc);
            let (w, h) = { let n = self.node(i); (n.width * sc, n.height * sc) };
            { let n = self.node(i); n.tag = "ed_grid"; n.parent = Some(parent); }
            made.push((i, dx, dy, w, h));
        }
        if made.is_empty() { return; }
        let (mut minx, mut maxx, mut miny, mut maxy) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for &(_, dx, dy, w, h) in &made { minx = minx.min(dx - w / 2.0); maxx = maxx.max(dx + w / 2.0); miny = miny.min(dy - h / 2.0); maxy = maxy.max(dy + h / 2.0); }
        let md = (maxx - minx).max(maxy - miny).max(1.0);
        let k = (64.0 / md).min(0.725);
        for (i, dx, dy, _, _) in made { let n = self.node(i); n.x = dx * k; n.y = dy * k; n.sx *= k; n.sy *= k; }
    }

    // ---- actions ----
    pub fn ed_action(&mut self, name: &str, arg: usize) -> bool {
        if !self.ed.on { return false; }
        match name {
            "EdTab" => { if self.ed.tab as usize != arg { self.ed.tab = arg as u8; self.ed.page = 0; self.ed_update_tab_visuals(); self.ed_build_grid(); } true }
            "EdCategory" => { self.ed.category = arg; self.ed.page = 0; self.ed_build_grid(); true }
            "EdPage" => {
                let pages = self.ed_total_pages().max(1);
                if arg == 0 { self.ed.page = if self.ed.page > 0 { self.ed.page - 1 } else { pages - 1 }; } else { self.ed.page = if self.ed.page + 1 < pages { self.ed.page + 1 } else { 0 }; }
                self.ed_build_grid(); true
            }
            "EdSelectObj" => { self.ed.selected_id = arg as i64; self.ed_build_grid(); true }
            "EdSwipe" => { self.ed.swipe = !self.ed.swipe; let sb = self.ed.swipe_node; self.node(sb).frame = if self.ed.swipe { "GJ_button02" } else { "GJ_button01" }.into(); true }
            "EdMove" => { let m = MOVES[arg]; self.ed_move(m.0, m.1); true }
            "EdFlip" => { self.ed_flip(arg == 0); true }
            "EdRotate" => { let d = [90.0, -90.0, 45.0, -45.0][arg]; self.ed_rotate(d); true }
            "EdDuplicate" => { self.ed_duplicate(); true }
            "EdDelete" => { self.ed_delete_selected(); true }
            "EdDeselect" => { self.ed_clear_selection(); true }
            "EdPause" | "EdPm" | "EdPmGlow" => self.ed_pm_action(name, arg),
            "EdNoop" => true,
            "EdLayer" => { self.ed_layer_step(arg); true }
            n if n == "EdEditObject" || n == "EdGroup" || n.starts_with("EdGp") || n == "EdOpClose" => self.ed_objects_action(name, arg),
            n if n.starts_with("EdNum") => self.ed_fields_action(name, arg),
            n if n.starts_with("EdTf") => self.ed_trig_action(name, arg),
            n if n.starts_with("EdCt") => self.ed_ct_action(name, arg),
            n if n.starts_with("EdCp") => self.ed_cp_action(name, arg),
            n if n.starts_with("EdLs") || n.starts_with("EdHo") || n.starts_with("EdSo") || n.starts_with("EdSp") || n == "EdSettings" => self.ed_ls_action(name, arg),
            "EdPlaytest" => { self.ed_pt_start(); true }
            "EdPtPause" => { self.ed_pt_toggle_pause(); true }
            "EdPtStop" => { self.ed_pt_stop(false); true }
            "EdZoom" => { self.ed_zoom(if arg == 1 { 0.1 } else { -0.1 }, self.screen_width / 2.0, 320.0); true }
            _ => false,
        }
    }

    pub fn ed_clear_selection(&mut self) { self.ed.sel.clear(); self.ed_update_buttons(); }
    fn ed_select(&mut self, ids: Vec<usize>, tint: f64) {
        let mut u: Vec<usize> = Vec::new();
        for i in ids { if i < self.ed.level.objects.len() && !u.contains(&i) { u.push(i); } }
        self.ed.sel = u; self.ed.sel_tint = tint; self.ed_update_buttons();
    }

    /// Applies `f` to the typed fields and the raw bag of every selected object, then reparses so both agree.
    fn ed_edit_selected(&mut self, f: impl Fn(&mut EdObj)) {
        for &i in &self.ed.sel.clone() {
            if let Some(o) = self.ed.level.objects.get_mut(i) {
                f(o);
                // keep the parsed view and the raw bag consistent (the serialized form is the truth)
                let s = serialize_object(o);
                if let Some(n) = EdObj::parse(&s) { let keep_text = o.o.text.clone(); *o = n; if o.o.id == Some(914) { o.o.text = keep_text; } }
            }
        }
        self.ed.dirty = true;
    }

    fn ed_move(&mut self, dx: f64, dy: f64) {
        self.ed_edit_selected(|o| { o.o.x += dx / 2.0; o.o.y -= dy / 2.0; });
    }
    fn ed_rotate(&mut self, deg: f64) { self.ed_edit_selected(|o| { o.o.rot = fnum(o.o.rot) + deg; }); }
    fn ed_flip(&mut self, x_axis: bool) {
        self.ed_edit_selected(|o| { if x_axis { o.o.flip_x = !o.o.flip_x; } else { o.o.flip_y = !o.o.flip_y; } o.o.rot = -fnum(o.o.rot); });
    }
    fn ed_duplicate(&mut self) {
        let mut new_ids = Vec::new();
        for &i in &self.ed.sel.clone() {
            let Some(src) = self.ed.level.objects.get(i).cloned() else { continue };
            if src.o.id == Some(142) && self.ed.level.objects.iter().filter(|o| o.o.id == Some(142)).count() >= 3 { continue; }
            self.ed.level.objects.push(src);
            new_ids.push(self.ed.level.objects.len() - 1);
        }
        if new_ids.is_empty() { self.ed_clear_selection(); } else { self.ed_select(new_ids, 0x00ffff as f64); }
        self.ed.dirty = true;
    }
    fn ed_delete_selected(&mut self) {
        let mut sel = self.ed.sel.clone();
        if sel.is_empty() { return; }
        sel.sort(); sel.dedup();
        self.ed.sel.clear();
        for &i in sel.iter().rev() { if i < self.ed.level.objects.len() { self.ed.level.objects.remove(i); } }
        self.ed.dirty = true;
        self.ed_build_grid();
    }

    fn ed_zoom(&mut self, delta: f64, ax: f64, ay: f64) {
        let old = self.ed.zoom;
        let nz = (old + delta).max(0.1).min(4.0);
        if (old - nz).abs() < 1e-12 { return; }
        let (wx, wy) = ((self.ed.cam_x + ax) / old, (self.ed.cam_y + ay) / old);
        self.ed.zoom = nz;
        self.ed.cam_x = wx * nz - ax; self.ed.cam_y = wy * nz - ay;
        let zt = self.ed.zoom_text;
        { let n = self.node(zt); n.text = format!("Zoom: {:.2}x", nz); n.alpha = 1.0; }
        self.ed.zoom_clock = 1000.0;
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let t = self.node(zt).text.clone(); self.node(zt).width = f.width(&t, 40.0);
    }

    pub fn ed_wheel(&mut self, x: f64, y: f64, dy: f64) { self.ed_zoom(if dy > 0.0 { -0.1 } else { 0.1 }, x, y); }
    pub fn ed_timeline_drag(&mut self, pct: f64) {
        self.ed.cam_x = pct * self.ed_level_width();
    }

    // ---- the world ----
    fn ed_world(&self, x: f64, y: f64) -> (f64, f64) { ((x + self.ed.cam_x) / self.ed.zoom, (y + self.ed.cam_y) / self.ed.zoom) }

    /// topmost object whose sprite contains the pointer (`getBounds` of the sprites, in world space)
    fn ed_object_at(&self, x: f64, y: f64) -> Option<usize> {
        let (wx, wy) = self.ed_world(x, y);
        crate::with_game(|g| {
            for o in (0..g.fx.obj_ranges.len()).rev() {
                let (a, b) = g.fx.obj_ranges[o];
                for si in (a..b).rev() {
                    let Some(s) = g.fx.sprites.get(si as usize) else { continue };
                    if !s.active || !s.visible { continue; }
                    let (w, h) = self.sizes.get(&format!("{}/{}", s.atlas, s.frame)).copied().unwrap_or((0.0, 0.0));
                    let (hw, hh) = (w * s.scale_x.abs() / 2.0, h * s.scale_y.abs() / 2.0);
                    let (c, sn) = (s.rotation.cos().abs(), s.rotation.sin().abs());
                    let (ex, ey) = (hw * c + hh * sn, hw * sn + hh * c);
                    if wx >= s.x - ex && wx <= s.x + ex && wy >= s.y - ey && wy <= s.y + ey { return Some(o); }
                }
            }
            None
        })
    }

    fn ed_objects_in_rect(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<usize> {
        let (ax, ay) = self.ed_world(x0.min(x1), y0.min(y1));
        let (bx, by) = self.ed_world(x0.max(x1), y0.max(y1));
        crate::with_game(|g| {
            let mut out = Vec::new();
            for o in 0..g.fx.obj_ranges.len() {
                let (a, b) = g.fx.obj_ranges[o];
                let mut hit = false;
                for si in a..b {
                    let Some(s) = g.fx.sprites.get(si as usize) else { continue };
                    if !s.active || !s.visible { continue; }
                    let (w, h) = self.sizes.get(&format!("{}/{}", s.atlas, s.frame)).copied().unwrap_or((0.0, 0.0));
                    let (hw, hh) = (w * s.scale_x.abs() / 2.0, h * s.scale_y.abs() / 2.0);
                    let (c, sn) = (s.rotation.cos().abs(), s.rotation.sin().abs());
                    let (ex, ey) = (hw * c + hh * sn, hw * sn + hh * c);
                    if s.x + ex >= ax && s.x - ex <= bx && s.y + ey >= ay && s.y - ey <= by { hit = true; break; }
                }
                if hit { out.push(o); }
            }
            out
        })
    }

    fn ed_place(&mut self, x: f64, y: f64) {
        let (wx, wy) = self.ed_world(x, y);
        let snap_x = (wx / 60.0).floor() * 60.0;
        let snap_y = ((wy + 20.0) / 60.0).floor() * 60.0;
        let (tx, ty) = (snap_x / 2.0 + 15.0, -(snap_y / 2.0) + 225.0);
        let id = self.ed.selected_id;
        let Some(d) = def_of(id) else { return };
        if id == 142 && self.ed.level.objects.iter().filter(|o| o.o.id == Some(142)).count() >= 3 { return; }
        if id == 749 { return; }
        let ox = d.get_num("editorOffsetX").or_else(|| d.get_num("offsetX")).unwrap_or(0.0);
        let oy = d.get_num("editorOffsetY").or_else(|| d.get_num("offsetY")).unwrap_or(0.0);
        let (px, py) = (tx + ox, ty + oy);
        let zl = d.get_num("default_z_layer").unwrap_or(0.0) as i64;
        let zo = d.get_num("default_z_order").unwrap_or(0.0) as i64;
        let mut s = format!("1,{},2,{},3,{},4,0,5,0,6,0,21,0,20,0,22,0,24,{},25,{},32,1,61,0,57,,kA2,0,kA3,0,kA4,0,kA8,0,kA28,0,kA11,0", id, js_num(px), js_num(py), zl, zo);
        // colour triggers (except bg/ground ones) start on channel 1
        if matches!(id, 105 | 744 | 899 | 900 | 915) { s.push_str(",23,1"); }
        if id == 747 { s.push_str(",54,90"); }
        if d.get("textObject").is_some() { s.push_str(&format!(",31,{}", d.get_str("defaultText").unwrap_or("A"))); }
        let Some(mut o) = EdObj::parse(&s) else { return };
        if d.get("textObject").is_some() { o.o.text = d.get_str("defaultText").unwrap_or("A").to_string(); }
        self.ed.level.objects.push(o);
        self.ed.dirty = true;
        let idx = self.ed.level.objects.len() - 1;
        self.ed_select(vec![idx], 0x00ff00 as f64);
    }

    pub fn ed_world_down(&mut self, x: f64, y: f64) {
        if self.ed.paused || self.ed.playtest_active() { return; }
        self.ed.down = Some((x, y, self.ed.cam_x, self.ed.cam_y));
        self.ed.dragging = false;
        if self.ed.tab == 1 && self.ed.swipe { self.ed.box_start = Some((x, y)); self.ed.box_moved = false; }
        if self.ed.swipe && self.ed.tab != 1 { self.ed_swipe_cell(x, y); }
    }

    fn ed_swipe_cell(&mut self, x: f64, y: f64) {
        let cell = (((x + self.ed.cam_x) / 60.0).floor() as i64, ((y + self.ed.cam_y + 20.0) / 60.0).floor() as i64);
        if cell != self.ed.last_cell { self.ed.last_cell = cell; self.ed_action_at(x, y); }
    }

    fn ed_action_at(&mut self, x: f64, y: f64) {
        match self.ed.tab {
            0 => self.ed_place(x, y),
            1 => self.ed_pick(x, y, false),
            _ => { if let Some(i) = self.ed_object_at(x, y) { self.ed.sel = vec![i]; self.ed_delete_selected(); } }
        }
    }

    fn ed_pick(&mut self, x: f64, y: f64, add: bool) {
        match self.ed_object_at(x, y) {
            None => { if !(add || (self.ed.tab == 1 && self.ed.swipe)) { self.ed_clear_selection(); } }
            Some(i) => {
                if add || (self.ed.tab == 1 && self.ed.swipe) { let mut s = self.ed.sel.clone(); if !s.contains(&i) { s.push(i); } self.ed_select(s, 0x00ff00 as f64); }
                else { self.ed_select(vec![i], 0x00ff00 as f64); }
            }
        }
    }

    pub fn ed_world_move(&mut self, x: f64, y: f64) {
        let Some((dx0, dy0, cx0, cy0)) = self.ed.down else { return };
        if self.ed.box_start.is_some() {
            let (sx, sy) = self.ed.box_start.unwrap();
            let (dx, dy) = (x - sx, y - sy);
            if !self.ed.box_moved && dx * dx + dy * dy < 16.0 { return; }
            self.ed.box_moved = true;
            let bn = self.ed.box_node;
            let n = self.node(bn); n.x = sx.min(x); n.y = sy.min(y); n.w = dx.abs(); n.h = dy.abs(); n.alpha = 0.25;
            return;
        }
        if self.ed.swipe { if self.ed.tab != 1 { self.ed_swipe_cell(x, y); } return; }
        let (dx, dy) = (x - dx0, y - dy0);
        if (dx * dx + dy * dy).sqrt() > 10.0 { self.ed.dragging = true; self.ed.cam_x = cx0 - dx; self.ed.cam_y = cy0 - dy; }
    }

    pub fn ed_world_up(&mut self, x: f64, y: f64) {
        let Some(_) = self.ed.down.take() else { return };
        if let Some((sx, sy)) = self.ed.box_start.take() {
            let bn = self.ed.box_node; self.node(bn).alpha = 0.0;
            if self.ed.box_moved {
                self.ed.box_moved = false;
                let found = self.ed_objects_in_rect(sx, sy, x, y);
                let mut merged = self.ed.sel.clone();
                for i in found { if !merged.contains(&i) { merged.push(i); } }
                self.ed_select(merged, 0x00ff00 as f64);
                return;
            }
            self.ed_pick(x, y, true);
            return;
        }
        self.ed.last_cell = (-1, -1);
        if !self.ed.swipe && !self.ed.dragging { self.ed_action_at(x, y); }
        self.ed.dragging = false;
    }
}

impl EditorState { pub fn playtest_active(&self) -> bool { self.pt } }

// ---- playtest (`_startEditorPlaytest` .. `_stopEditorPlaytest`) ----
impl Ui {
    fn ed_pt_buttons(&mut self) {
        let [play, pause, stop] = self.ed.pt_btns;
        let (active, paused) = (self.ed.pt, self.ed.pt_paused);
        self.node(play).visible = !active;
        { let n = self.node(pause); n.visible = active; n.frame = if paused { "GJ_playEditorBtn_001.png" } else { "GJ_pauseEditorBtn_001.png" }.into(); }
        self.node(stop).visible = active;
    }

    /// `_setEditorPlaytestGuiVisible`: everything of the editor GUI except the three playtest buttons
    fn ed_pt_gui(&mut self, visible: bool) {
        if visible {
            for (i, v) in std::mem::take(&mut self.ed.pt_hidden) { if let Some(n) = self.nodes.get_mut(i).and_then(|n| n.as_mut()) { n.visible = v; } }
        } else {
            let mut saved = Vec::new();
            for (i, slot) in self.nodes.iter_mut().enumerate() {
                if let Some(n) = slot { if n.tag.starts_with("ed_") && n.tag != "ed_pt" { saved.push((i, n.visible)); n.visible = false; } }
            }
            self.ed.pt_hidden = saved;
        }
    }

    pub fn ed_pt_start(&mut self) {
        if self.ed.pt { return; }
        self.ed_clear_selection();
        if self.ed.dirty { self.ed_rebuild_view(); self.ed.dirty = false; }
        self.ed.marks.clear(); self.ed.last_trail = [None, None];
        self.ed.pt_saved = (self.ed.cam_x, self.ed.cam_y, self.ed.zoom);
        // `_getLatestEditorStartPosition`: the start position furthest right (the list is sorted by x; ties keep the later object)
        let objs: Vec<(f64, i64)> = self.ed.level.objects.iter().map(|o| (o.o.x, o.o.id.unwrap_or(i64::MIN))).collect();
        let music = crate::with_game(|g| {
            g.sp.objects = objs;
            g.fx.editor_mode = false;
            g.fx.vis_min = -1; g.fx.vis_max = -1; // the editor view shows every section; the game culls again
            g.level_won = false; g.end_camera_override = false; g.end_cam = None;
            let idx = if g.start_positions.is_empty() { None } else { Some(g.start_positions.len() - 1) };
            g.start_pos_index = idx;
            g.practice_mode = false;
            g.restart_level(idx);
            g.render_restart();
            g.p.mirrored = false; g.p2.mirrored = false;
            idx.map_or(0.0, |i| g.song_offset_for_world_x(g.start_positions[i].x))
        });
        self.ed.pt_music = music;
        self.ed.pt = true; self.ed.pt_paused = false; self.ed.pt_press = false;
        self.ed.zoom = 1.0;
        self.ed_pt_gui(false);
        self.ed_pt_buttons();
        self.ed.pt_event += 1;
    }

    pub fn ed_pt_toggle_pause(&mut self) {
        if !self.ed.pt { return; }
        if !self.ed.pt_paused {
            self.ed.pt_paused = true;
            self.ed_pt_gui(true);
        } else {
            self.ed.pt_paused = false;
            self.ed_pt_gui(false);
        }
        self.ed_pt_buttons();
        self.ed.pt_event += 1;
    }

    pub fn ed_pt_stop(&mut self, death_mark: bool) {
        if !self.ed.pt { return; }
        if death_mark {
            let (x, y) = crate::with_game(|g| (g.e.player_world_x, 460.0 - g.p.y));
            self.ed.marks.push([1.0, x, y, 0.0, 0.0, 0.0]);
        }
        self.ed.pt = false; self.ed.pt_paused = false; self.ed.pt_press = false;
        // `_hideEditorPlaytestPlayer` / `_resetEditorPlaytestLevelState`: the level goes back to how the editor shows it
        self.ed_pt_gui(true);
        let (cx, cy, z) = self.ed.pt_saved;
        self.ed.cam_x = cx; self.ed.cam_y = cy; self.ed.zoom = z;
        self.ed_rebuild_view();
        self.ed.dirty = false;
        self.ed_pt_buttons();
        self.ed.pt_event += 1;
    }

    fn ed_pt_frame(&mut self, dt: f64, held: bool) {
        if self.ed.pt_paused { return; }
        let (dead, won, x, y, y2, dual, dead2) = crate::with_game(|g| {
            g.update(dt, held);
            (g.p.is_dead, g.level_won, g.e.player_world_x, 460.0 - g.p.y, 460.0 - g.p2.y, g.e.is_dual, g.p2.is_dead)
        });
        // `_drawEditorPlaytestTrailPoint`: a green line for the first player, a cyan one for the second
        for (k, (px, py, color, on)) in [(x, y, 0x00ff00 as f64, !dead), (x, y2, 0x00ffff as f64, dual && !dead2)].into_iter().enumerate() {
            if !on { if k == 1 { self.ed.last_trail[1] = None; } continue; }
            if let Some((lx, ly)) = self.ed.last_trail[k] {
                let (dx, dy) = (px - lx, py - ly);
                if dx * dx + dy * dy >= 2.0 { self.ed.marks.push([0.0, lx, ly, px, py, color]); self.ed.last_trail[k] = Some((px, py)); }
            } else { self.ed.last_trail[k] = Some((px, py)); }
        }
        if dead { self.ed_pt_stop(true); } else if won { self.ed_pt_stop(false); }
    }
}

// ---- editor layers (`_getEditorLayerOptions`, `_setEditorActiveLayerIndex`, `_applyEditorLayerFilter`) ----
impl Ui {
    /// the options: all layers, then 0 .. the highest layer any object uses
    fn ed_layer_options(&self) -> Vec<Option<i64>> {
        let mut max = 0i64;
        for o in &self.ed.level.objects { for l in [o.o.editor_layer, o.o.editor_layer2].into_iter().flatten() { if l > max { max = l; } } }
        let mut v = vec![None];
        for l in 0..=max { v.push(Some(l)); }
        v
    }

    fn ed_layer_step(&mut self, which: usize) {
        let n = self.ed_layer_options().len();
        let cur = self.ed.layer_idx as i64;
        let next = match which { 0 => 0, 1 => cur - 1, 2 => cur + 1, _ => n as i64 - 1 };
        self.ed.layer_idx = next.max(0).min(n as i64 - 1) as usize;
        self.ed_layer_label();
    }

    pub(crate) fn ed_layer_label(&mut self) {
        let opts = self.ed_layer_options();
        self.ed.layer_idx = self.ed.layer_idx.min(opts.len() - 1);
        let t = self.ed.layer_nodes[1];
        let s = match opts[self.ed.layer_idx] { None => "All".to_string(), Some(l) => l.to_string() };
        if let Some(n) = self.nodes.get_mut(t).and_then(|n| n.as_mut()) { n.text = s; }
    }

    /// the objects outside the active layer
    fn ed_layer_dim(&mut self) -> Vec<usize> {
        let opts = self.ed_layer_options();
        let Some(active) = opts.get(self.ed.layer_idx).copied().flatten() else { return Vec::new() };
        self.ed_layer_label();
        self.ed.level.objects.iter().enumerate().filter(|(_, o)| o.o.editor_layer.unwrap_or(0) != active && o.o.editor_layer2.unwrap_or(0) != active).map(|(i, _)| i).collect()
    }
}
