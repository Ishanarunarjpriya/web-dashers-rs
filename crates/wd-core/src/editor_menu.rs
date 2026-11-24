//! Creator menu, "My Levels" list and the level view (game-scene.js `_openCreatorMenu`, `_openEditorMenu`, `_openLevelView`).
//! Level storage (`created_levels` in localStorage) stays in the host; this side shows it and raises `Pop("Em...", n)` events.
use crate::js::parse_int;
use crate::jsonread;
use crate::ui::*;

#[derive(Clone, Debug, Default)]
pub struct CreatedLevel {
    pub name: String, pub song: String, pub length: i64, pub status: String, pub version: i64,
    pub created_id: String, pub level_id: String, pub description: String,
}

#[derive(Default)]
pub struct EdMenu {
    pub levels: Vec<CreatedLevel>,
    /// index into `levels` of the level shown in the level view
    pub view: Option<usize>,
    /// 0 none, 1 title, 2 description
    pub input: u8,
    pub blink: f64,
    pub cursor_on: bool,
    pub title_node: usize, pub desc_node: usize, pub title_cursor: usize, pub desc_cursor: usize,
    /// the list as displayed (sorted), indexes into `levels`
    pub order: Vec<usize>,
}

const LENGTHS: [&str; 6] = ["Tiny", "Short", "Medium", "Long", "XL", "Plat."];

fn length_name(v: i64) -> &'static str { LENGTHS.get(v.max(0) as usize).copied().unwrap_or("Tiny") }

impl Ui {
    /// `[ {createdId, levelName, song, levelLength, status, version, levelId, description}, ... ]`
    pub fn em_set_levels(&mut self, json: &str) {
        let mut out = Vec::new();
        if let Ok(j) = jsonread::parse(json) {
            for e in j.arr().unwrap_or(&[]) {
                let s = |k: &str| e.get_str(k).unwrap_or("").to_string();
                out.push(CreatedLevel {
                    name: s("levelName"), song: s("song"),
                    length: e.get_num("levelLength").or_else(|| e.get_str("levelLength").and_then(parse_int).map(|v| v as f64)).unwrap_or(0.0) as i64,
                    status: s("status"), version: e.get_num("version").unwrap_or(1.0) as i64,
                    created_id: s("createdId"), level_id: e.get("levelId").map(|v| v.str().map(|s| s.to_string()).or_else(|| v.num().map(|n| n.to_string())).unwrap_or_default()).unwrap_or_default(),
                    description: s("description"),
                });
            }
        }
        self.em.levels = out;
        // the displayed order: newest local id first
        let mut order: Vec<usize> = (0..self.em.levels.len()).collect();
        let key = |i: usize| parse_int(&self.em.levels[i].created_id.replace("local_", "")).unwrap_or(0);
        order.sort_by(|a, b| key(*b).cmp(&key(*a)));
        self.em.order = order;
    }

    pub fn em_created_id(&self, idx: usize) -> String { self.em.levels.get(idx).map(|l| l.created_id.clone()).unwrap_or_default() }
    pub fn em_field(&self, which: u32) -> String {
        match self.em.view.and_then(|i| self.em.levels.get(i)) { Some(l) => if which == 0 { l.name.clone() } else { l.description.clone() }, None => String::new() }
    }

    pub fn pe_open(&mut self, name: &str) -> bool {
        match name { "Creator" => { self.em_open_creator(); true } _ => false }
    }

    pub fn pe_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EmEdit" => { self.kill_tag("lv_"); self.em.input = 0; true }
            "EmBack" => { self.kill_tag("cm_"); true }
            "EmCreate" => { self.kill_tag("cm_"); self.em_open_my_levels(); true }
            "EmMyBack" => { self.kill_tag("my_"); self.em_open_creator(); true }
            "EmView" => { if let Some(&i) = self.em.order.get(arg) { self.kill_tag("my_"); self.em_open_view(i); } true }
            "EmViewBack" => { self.kill_tag("lv_"); self.em.view = None; self.em.input = 0; self.em_open_my_levels(); true }
            "EmTitle" => { self.em.input = 1; self.em_refresh_input(); true }
            "EmDesc" => { self.em.input = 2; self.em_refresh_input(); true }
            "EmBlock" => { self.em.input = 0; self.em_refresh_input(); true }
            "EmStub" => { self.open_info("This part needs the online servers, which the Rust port does not have yet."); true }
            _ => false,
        }
    }

    /// Opens the level view for the level with this createdId (after the host created it).
    pub fn em_open_new(&mut self, created_id: &str) {
        self.kill_tag("my_");
        if let Some(i) = self.em.levels.iter().position(|l| l.created_id == created_id) { self.em_open_view(i); }
    }

    /// back to the list after a delete
    pub fn em_after_delete(&mut self) { self.kill_tag("lv_"); self.em.view = None; self.em.input = 0; self.em_open_my_levels(); }

    pub(crate) fn em_gradient(&mut self, depth: f64, tag: &'static str) {
        let (sw, sh) = (self.screen_width, 640.0);
        let steps = 80;
        for gi in 0..steps {
            let t = gi as f64 / (steps - 1) as f64;
            let lerp = |a: f64, b: f64| (a + (b - a) * t).round() as i64;
            let c = lerp(0x00 as f64, 0x01 as f64) << 16 | lerp(0x65 as f64, 0x2c as f64) << 8 | lerp(0xff as f64, 0x71 as f64);
            let y = (gi as f64 * sh / steps as f64).floor();
            let h = (sh / steps as f64).ceil() + 1.0;
            self.shape(Kind::Rect, 0.0, y, sw, h, c, 1.0, depth, tag);
        }
    }

    pub(crate) fn em_fade_in(&mut self, depth: f64, ms: f64, tag: &'static str) {
        let f = self.shape(Kind::Rect, 0.0, 0.0, self.screen_width, 640.0, 0, 1.0, depth, tag);
        self.tween_to(f, &[(Prop::Alpha, 0.0)], ms, Ease::Linear, Then::Destroy);
    }

    pub(crate) fn em_blocker(&mut self, depth: f64, tag: &'static str) -> usize {
        let sw = self.screen_width;
        let b = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.0, depth, tag);
        { let n = self.node(b); n.interactive = true; n.width = sw; n.height = 640.0; }
        b
    }

    pub(crate) fn em_back_arrow(&mut self, action: Action, depth: f64, tag: &'static str) {
        let b = self.image("GJ_GameSheet03", "GJ_arrow_03_001.png", 50.0, 48.0, depth, 1.0);
        { let n = self.node(b); n.flip_x = true; n.flip_y = true; n.rot = std::f64::consts::PI; n.tag = tag; }
        self.make_bouncy(b, 1.0, Cond::Always, action);
    }

    // ---- creator menu ----
    pub fn em_open_creator(&mut self) {
        if self.get_tag_count("cm_") > 0 { return; }
        let (sw, sh) = (self.screen_width, 640.0);
        self.em_fade_in(200.0, 300.0, "cm_");
        self.em_gradient(100.0, "cm_");
        self.em_blocker(101.0, "cm_");
        let tl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, 0.0, 100.0, 1.0);
        { let n = self.node(tl); n.ox = 0.0; n.oy = 0.0; n.flip_y = true; n.tag = "cm_"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 152.0, 1.0);
        { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "cm_"; }
        let vault = self.image("GJ_GameSheet03", "GJ_lock_open_001.png", 1095.0, 40.0, 104.0, 1.0);
        self.node(vault).tag = "cm_"; self.make_bouncy(vault, 1.0, Cond::Always, Action::Popup("SecretVault"));
        let door = self.image("GJ_GameSheet03", "secretDoorBtn_closed_001.png", 1095.0, 601.0, 104.0, 1.0);
        self.node(door).tag = "cm_"; self.lock_gray(door);
        self.em_back_arrow(Action::Pop("EmBack", 0), 104.0, "cm_");
        let buttons = ["GJ_createBtn_001.png", "GJ_savedBtn_001.png", "GJ_highscoreBtn_001.png", "GJ_challengeBtn_001.png", "GJ_versusBtn_001.png", "GJ_mapBtn_001.png", "GJ_dailyBtn_001.png",
            "GJ_weeklyBtn_001.png", "GJ_eventBtn_001.png", "GJ_gauntletsBtn_001.png", "GJ_featuredBtn_001.png", "GJ_listsBtn_001.png", "GJ_pathsBtn_001.png", "GJ_mapPacksBtn_001.png", "GJ_searchBtn_001.png"];
        let (cols, sc, gap) = (5usize, 0.77, 18.0);
        let size = 209.0 * sc;
        let grid_w = cols as f64 * size + (cols as f64 - 1.0) * gap;
        let sx = sw / 2.0 - grid_w / 2.0 + size / 2.0;
        let rows = (buttons.len() + cols - 1) / cols;
        let grid_h = rows as f64 * size + (rows as f64 - 1.0) * gap;
        let sy = sh / 2.0 - grid_h / 2.0 + size / 2.0;
        for (idx, frame) in buttons.iter().enumerate() {
            let (col, row) = (idx % cols, idx / cols);
            let b = self.image("GJ_GameSheet04", frame, sx + col as f64 * (size + gap), sy + row as f64 * (size + gap), 104.0, sc);
            self.node(b).tag = "cm_";
            let action = match *frame {
                "GJ_createBtn_001.png" => Some(Action::Pop("EmCreate", 0)),
                // these open server-backed screens in the original (Saved is the list of saved online levels, not the editor's My Levels)
                "GJ_searchBtn_001.png" => Some(Action::Pop("OnSearchOpen", 0)),
                "GJ_featuredBtn_001.png" => Some(Action::Pop("OnFeatured", 0)),
                "GJ_savedBtn_001.png" => Some(Action::Pop("OnSaved", 0)),
                "GJ_dailyBtn_001.png" => Some(Action::Pop("OnDaily", 0)),
                "GJ_weeklyBtn_001.png" => Some(Action::Pop("OnDaily", 1)),
                "GJ_eventBtn_001.png" => Some(Action::Pop("OnDaily", 2)),
                _ => None,
            };
            match action { Some(a) => self.make_bouncy(b, sc, Cond::Always, a), None => self.lock_gray(b) }
        }
    }

    pub(crate) fn get_tag_count(&self, tag: &str) -> usize { self.nodes.iter().flatten().filter(|n| n.tag.starts_with(tag)).count() }

    // ---- My Levels ----
    pub fn em_open_my_levels(&mut self) {
        self.kill_tag("my_");
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        self.em_fade_in(200.0, 300.0, "my_");
        self.em_gradient(100.0, "my_");
        self.em_blocker(101.0, "my_");
        let root = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 102.0, "my_");
        let (tw, th) = (712.0, 460.0);
        let (tx, ty) = ((sw - tw) / 2.0, 85.0);
        self.shape(Kind::Rect, tx, ty, tw, th, 0xc2723e, 1.0, 102.0, "my_");
        let count = self.em.order.len();
        let list_h = count as f64 * 100.0;
        let (content, si) = self.scroll_area(root, cx, ty + th / 2.0, tw, th, list_h, 103.0, "my_");
        self.scrolls[si].base_y = ty + 22.0;
        self.scrolls[si].max = 32.0 + (list_h - th).max(0.0);
        self.set_scroll(si, 0.0);
        let mut name_counts: std::collections::HashMap<String, i32> = Default::default();
        for (index, &li) in self.em.order.clone().iter().enumerate() {
            let l = self.em.levels[li].clone();
            let rev = match name_counts.get_mut(&l.name) { None => { name_counts.insert(l.name.clone(), 1); String::new() } Some(c) => { let s = format!("Rev. {}", c); *c += 1; s } };
            let slot = index as f64 * 100.0 + 50.0;
            let d = 103.01;
            let stripe = self.shape(Kind::Rect, cx - (tw - 10.0) / 2.0, slot - 50.0, tw - 10.0, 100.0, if index % 2 != 0 { 0xc2723e } else { 0xa1582c }, 1.0, d, "my_");
            self.node(stripe).parent = Some(content);
            let sep = self.shape(Kind::Rect, cx - (tw - 10.0) / 2.0, slot + 50.0, tw - 10.0, 1.0, 0x502c16, 1.0, d, "my_");
            self.node(sep).parent = Some(content);
            let name = self.pa_txt(content, "bigFont", 32.0, &l.name, tx + 20.0, slot - 22.0, 0.0, 0.5, 1.0, d + 0.01, "my_");
            let nw = self.node(name).width;
            self.pa_txt(content, "goldFont", 20.0, &rev, tx + 20.0 + nw + 10.0, slot - 22.0 + 5.0, 0.0, 0.5, 1.0, d + 0.01, "my_");
            let info_y = slot + 18.0;
            self.pa_img(content, "GJ_GameSheet03", "GJ_timeIcon_001.png", tx + 35.0, info_y, 0.65, d + 0.01, "my_");
            self.pa_txt(content, "bigFont", 18.0, length_name(l.length), tx + 35.0 + 22.0, info_y, 0.0, 0.5, 1.0, d + 0.01, "my_");
            self.pa_img(content, "GJ_GameSheet03", "GJ_musicIcon_001.png", tx + 150.0, info_y, 0.65, d + 0.01, "my_");
            let song = self.pa_txt(content, "bigFont", 18.0, &l.song, tx + 150.0 + 22.0, info_y, 0.0, 0.5, 1.0, d + 0.01, "my_");
            let status_x = tx + 380.0;
            self.pa_img(content, "GJ_GameSheet03", "GJ_infoIcon_001.png", status_x, info_y, 0.65, d + 0.01, "my_");
            let max_w = (status_x - 25.0 - (tx + 172.0)).max(20.0);
            let sw_ = self.node(song).width;
            if sw_ > max_w { let k = max_w / sw_; let n = self.node(song); n.sx = k; n.sy = k; }
            self.pa_txt(content, "bigFont", 18.0, &l.status, status_x + 22.0, info_y, 0.0, 0.5, 1.0, d + 0.01, "my_");
            // View button: starts at 1.05 but its bounce base is 0.75, like the original
            let zone = self.shape(Kind::Rect, tx + tw - 80.0, slot, 115.0, 55.0, 0, 0.0, d + 0.02, "my_");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 115.0; n.height = 55.0; n.parent = Some(content); n.sx = 1.05; n.sy = 1.05; }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, 115.0, 55.0, 0, 1.0, d + 0.02, "my_");
            { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = 24.0; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
            let t = self.pa_txt(zone, "bigFont", 32.0, "View", -2.0, -1.0, 0.5, 0.5, 1.1, d + 0.03, "my_"); let _ = t;
            self.make_bouncy(zone, 0.75, Cond::Always, Action::Pop("EmView", index));
        }
        if count == 0 { self.text_node("bigFont", 38.0, "Tap New to create a level", cx, ty + th / 2.0, 104.0, 0.5, 0.5, 1.0); let n = self.nodes.len() - 1; self.node(n).tag = "my_"; }
        // frame
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = th / side_h;
        let l = self.image("GJ_WebSheet", "GJ_table_side_001.png", tx - 40.0, ty, 105.0, 1.0); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; n.tag = "my_"; }
        let r = self.image("GJ_WebSheet", "GJ_table_side_001.png", tx + tw + 40.0, ty, 105.0, 1.0); { let n = self.node(r); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; n.tag = "my_"; }
        let top = self.image("GJ_WebSheet", "GJ_table_top_001.png", cx, ty - 10.0, 105.0, 1.0); self.node(top).tag = "my_";
        let bot = self.image("GJ_WebSheet", "GJ_table_bottom_001.png", cx, ty + th + 20.0, 105.0, 1.0); self.node(bot).tag = "my_";
        let t = self.text_node("bigFont", 42.0, "My Levels", cx, ty - 15.0, 106.0, 0.5, 0.5, 1.1); self.node(t).tag = "my_";
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, 105.0, 1.0); { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = "my_"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 105.0, 1.0); { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "my_"; }
        let new = self.image("GJ_GameSheet03", "GJ_newBtn_001.png", sw - 60.0, sh - 55.0, 107.0, 0.95); self.node(new).tag = "my_";
        self.make_bouncy(new, 0.95, Cond::Always, Action::Pop("EmNew", 0));
        let imp = self.image("GJ_GameSheet04", "GJ_importBtn_001.png", 50.0, sh - 55.0 - 90.0, 107.0, 0.95); self.node(imp).tag = "my_";
        self.make_bouncy(imp, 0.95, Cond::Always, Action::Pop("EmImport", 0));
        let up = self.image("GJ_GameSheet03", "GJ_myServerBtn_001.png", 60.0, sh - 60.0, 107.0, 0.95); self.node(up).tag = "my_"; self.lock_gray(up);
        self.em_back_arrow(Action::Pop("EmMyBack", 0), 107.0, "my_");
    }

    // ---- level view ----
    pub fn em_open_view(&mut self, idx: usize) {
        let Some(l) = self.em.levels.get(idx).cloned() else { return };
        self.em.view = Some(idx); self.em.input = 0; self.em.blink = 0.0; self.em.cursor_on = true;
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        self.em_gradient(102.0, "lv_");
        let blk = self.em_blocker(101.0, "lv_");
        self.node(blk).action = Action::Pop("EmBlock", 0);
        self.em_fade_in(200.0, 350.0, "lv_");
        let bw = sw * 0.6;
        let name_y = 50.0;
        let nb = self.shape(Kind::RoundRect, cx - bw / 2.0, name_y - 28.0, bw, 70.0, 0, 0.3, 151.0, "lv_");
        { let n = self.node(nb); n.radius = 18.0; n.interactive = true; n.width = bw; n.height = 70.0; n.ox = 0.0; n.oy = 0.0; n.x = cx - bw / 2.0; n.y = name_y - 28.0; n.action = Action::Pop("EmTitle", 0); }
        self.em.title_node = self.text_node("bigFont", 45.0, &l.name, cx, name_y + 5.0, 152.0, 0.5, 0.5, 1.0);
        let tn = self.em.title_node; self.node(tn).tag = "lv_";
        self.em.title_cursor = self.text_node("bigFont", 45.0, "|", 0.0, name_y + 5.0, 153.0, 0.0, 0.5, 1.0);
        let tc = self.em.title_cursor; { let n = self.node(tc); n.tag = "lv_"; n.visible = false; }
        let (desc_y, desc_h) = (180.0, 120.0);
        let db = self.shape(Kind::RoundRect, cx - bw / 2.0, desc_y - desc_h / 2.0, bw, desc_h, 0, 0.3, 151.0, "lv_");
        { let n = self.node(db); n.radius = 18.0; n.interactive = true; n.width = bw; n.height = desc_h; n.action = Action::Pop("EmDesc", 0); }
        self.em.desc_node = self.text_node("arial", 22.0, "", cx, desc_y, 152.0, 0.5, 0.5, 1.0);
        let dn = self.em.desc_node; self.node(dn).tag = "lv_";
        self.em.desc_cursor = self.text_node("arial", 18.0, "|", 0.0, 0.0, 153.0, 0.5, 0.5, 1.0);
        let dc = self.em.desc_cursor; { let n = self.node(dc); n.tag = "lv_"; n.visible = false; }
        self.em_refresh_input();
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, 152.0, 1.0); { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = "lv_"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 152.0, 1.0); { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "lv_"; }
        let btn_y = sh * 0.58;
        for (frame, dx, act) in [("GJ_editBtn_001.png", -220.0, "EmEdit"), ("GJ_playBtn2_001.png", 0.0, "EmPlay"), ("GJ_shareBtn_001.png", 220.0, "EmExport")] {
            let b = self.image("GJ_GameSheet03", frame, cx + dx, btn_y, 153.0, 1.1);
            self.node(b).tag = "lv_"; self.make_bouncy(b, 1.1, Cond::Always, Action::Pop(act, 0));
        }
        self.em_back_arrow(Action::Pop("EmViewBack", 0), 153.0, "lv_");
        let del = self.image("GJ_GameSheet03", "GJ_deleteBtn_001.png", sw - 50.0, 48.0, 153.0, 0.8);
        self.node(del).tag = "lv_"; self.make_bouncy(del, 0.8, Cond::Always, Action::Pop("EmDelete", 0));
        let (fy, sfy) = (sh - 100.0, sh - 30.0);
        let ti = self.image("GJ_GameSheet03", "GJ_timeIcon_001.png", cx - 350.0, fy, 152.0, 1.0); self.node(ti).tag = "lv_";
        let t = self.text_node("bigFont", 33.0, length_name(l.length), cx - 310.0, fy, 152.0, 0.0, 0.5, 1.0); self.node(t).tag = "lv_";
        let mi = self.image("GJ_GameSheet03", "GJ_musicIcon_001.png", cx - 160.0, fy, 152.0, 1.0); self.node(mi).tag = "lv_";
        let song = self.text_node("bigFont", 29.0, &l.song, cx - 115.0, fy, 152.0, 0.0, 0.5, 1.0); self.node(song).tag = "lv_";
        let si = self.image("GJ_GameSheet03", "GJ_infoIcon_001.png", cx + 200.0, fy, 152.0, 1.0); self.node(si).tag = "lv_";
        let max_w = (cx + 200.0 - 30.0 - (cx - 115.0)).max(40.0);
        let w_ = self.node(song).width;
        if w_ > max_w { let k = max_w / w_; let n = self.node(song); n.sx = k; n.sy = k; }
        let t = self.text_node("bigFont", 33.0, &l.status, cx + 245.0, fy, 152.0, 0.0, 0.5, 1.0); self.node(t).tag = "lv_";
        let t = self.text_node("goldFont", 30.0, &format!("Version: {}", l.version.max(1)), cx - 180.0, sfy, 152.0, 0.5, 0.5, 1.0); self.node(t).tag = "lv_";
        let id = if l.level_id.is_empty() { "na".to_string() } else { l.level_id.clone() };
        let t = self.text_node("goldFont", 30.0, &format!("ID: {}", id), cx + 180.0, sfy, 152.0, 0.5, 0.5, 1.0); self.node(t).tag = "lv_";
    }

    fn em_refresh_input(&mut self) {
        let Some(l) = self.em.view.and_then(|i| self.em.levels.get(i)).cloned() else { return };
        let (cx, bw) = (self.screen_width / 2.0, self.screen_width * 0.6);
        let (tn, dn, tc, dc) = (self.em.title_node, self.em.desc_node, self.em.title_cursor, self.em.desc_cursor);
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let w = f.width(&l.name, 45.0);
        { let n = self.node(tn); n.text = l.name.clone(); n.width = w; }
        let show = self.em.cursor_on;
        let (input, desc) = (self.em.input, l.description.clone());
        let text = if input == 2 { desc.clone() } else if desc.is_empty() { "Description [Optional]".to_string() } else { desc.clone() };
        let lines = crate::popups_a::wrap_arial(&text, 22.0, bw - 40.0);
        { let n = self.node(dn); n.text = lines.join("\n"); }
        { let n = self.node(tc); n.visible = input == 1 && show; n.x = cx + w / 2.0 + 2.0; }
        // description cursor after the last wrapped line
        let count = lines.len() as f64;
        let last = lines.last().cloned().unwrap_or_default();
        let lw = crate::popups_a::arial_width(&last, 22.0);
        let (size, spacing) = (22.0, 4.0);
        let full = size + spacing;
        let total = count * full - spacing;
        let top = 180.0 - total / 2.0;
        { let n = self.node(dc); n.visible = input == 2 && show; n.x = cx + lw / 2.0 + 2.0; n.y = top + (count - 1.0) * full + size / 2.0; }
    }

    /// Typed key for the level view's title/description (8 = backspace, 13 = enter, otherwise a character).
    pub fn em_key(&mut self, ch: u32) {
        let Some(idx) = self.em.view else { return };
        if self.em.input == 0 { return; }
        let input = self.em.input;
        let l = &mut self.em.levels[idx];
        let field = if input == 1 { &mut l.name } else { &mut l.description };
        match ch {
            8 => { field.pop(); }
            13 => { self.em.input = 0; }
            c => if let Some(c) = char::from_u32(c) {
                if !c.is_control() { if input == 1 && field.chars().count() < 20 { field.push(c); } else if input == 2 && field.chars().count() < 150 { field.push(c); } }
            }
        }
        self.em.cursor_on = true; self.em.blink = 0.0;
        self.em_refresh_input();
        if ch != 13 { self.events.push(Action::Pop("EmField", (input - 1) as usize)); }
    }

    pub fn pe_step(&mut self, dt: f64) {
        if self.em.view.is_some() && self.em.input != 0 {
            self.em.blink += dt;
            if self.em.blink >= 500.0 { self.em.blink -= 500.0; self.em.cursor_on = !self.em.cursor_on; self.em_refresh_input(); }
        }
    }
}
