//! Online levels (game-scene.js `_openSearchMenu`, `_openOnlineLevelsScene`, `_openSavedLevelsScene`, `_buildSpecialLevelPopup`):
//! the screens. The host (web/online.js) talks to the Geometry Dash servers and hands the results over as JSON; every button raises a
//! `Pop("On...", n)` event for it.
use crate::jsonread;
use crate::ui::*;

#[derive(Clone, Debug, Default)]
pub struct OnLevel {
    pub id: String, pub name: String, pub author: String, pub desc: String,
    /// difficulty face frame (GJ_GameSheet03)
    pub face: String,
    pub stars: i64, pub downloads: i64, pub likes: i64, pub length: i64, pub song: String,
    pub featured: bool, pub epic: bool, pub coins: i64,
}

#[derive(Default)]
pub struct On {
    pub levels: Vec<OnLevel>,
    /// 0 search results, 1 featured, 2 saved
    pub kind: u8,
    pub page: usize, pub pages: usize,
    pub loading: bool, pub error: String,
    pub query: String, pub input: bool, pub blink: f64,
    pub field_text: usize, pub field_cursor: usize,
    pub view: Option<usize>,
    /// the daily / weekly / event popup (0, 1, 2) and its level
    pub special: Option<u8>, pub special_level: Option<OnLevel>,
    pub spin: usize,
}

const LENGTHS: [&str; 6] = ["Tiny", "Short", "Medium", "Long", "XL", "Plat."];
fn length_name(v: i64) -> &'static str { LENGTHS.get(v.max(0) as usize).copied().unwrap_or("Tiny") }

impl Ui {
    /// `[ {id, name, author, desc, face, stars, downloads, likes, length, song, featured, epic, coins}, ... ]`, the page count (0 = unknown)
    pub fn on_set_results(&mut self, json: &str, pages: usize) {
        let mut out = Vec::new();
        if let Ok(j) = jsonread::parse(json) {
            for e in j.arr().unwrap_or(&[]) { out.push(Self::on_parse_level(e)); }
        }
        self.on.levels = out;
        self.on.pages = pages.max(1);
        self.on.loading = false; self.on.error.clear();
        if self.get_tag_count("on_l") > 0 { self.on_open_list(); }
    }

    fn on_parse_level(e: &jsonread::Json) -> OnLevel {
        let s = |k: &str| e.get(k).map(|v| v.str().map(|x| x.to_string()).or_else(|| v.num().map(|n| format!("{}", n as i64))).unwrap_or_default()).unwrap_or_default();
        let n = |k: &str| e.get_num(k).unwrap_or(0.0) as i64;
        OnLevel {
            id: s("id"), name: s("name"), author: s("author"), desc: s("desc"),
            face: { let f = s("face"); if f.is_empty() { "diffIcon_00_btn_001.png".into() } else { f } },
            stars: n("stars"), downloads: n("downloads"), likes: n("likes"), length: n("length"), song: s("song"),
            featured: n("featured") != 0, epic: n("epic") != 0, coins: n("coins"),
        }
    }

    pub fn on_set_error(&mut self, msg: &str) {
        self.on.loading = false; self.on.error = msg.to_string();
        if self.get_tag_count("on_l") > 0 { self.on_open_list(); }
        if self.on.special.is_some() { self.on_open_special(); }
    }

    /// the daily / weekly / event level for the open popup
    pub fn on_set_special(&mut self, json: &str) {
        if let Ok(j) = jsonread::parse(json) { self.on.special_level = Some(Self::on_parse_level(&j)); }
        self.on.loading = false; self.on.error.clear();
        if self.on.special.is_some() { self.on_open_special(); }
    }

    pub fn on_query(&self) -> String { self.on.query.clone() }

    pub fn on_key(&mut self, code: u32) {
        match code {
            8 => { self.on.query.pop(); }
            13 => { self.on.input = false; self.events.push(Action::Pop("OnSearch", 0)); self.po_action("OnSearch", 0); return; }
            c => if let Some(c) = char::from_u32(c) { if !c.is_control() && self.on.query.chars().count() < 40 { self.on.query.push(c); } },
        }
        self.on_refresh_field();
    }

    pub fn on_step(&mut self, dt_ms: f64) {
        if self.get_tag_count("on_") == 0 { return; }
        self.on.blink += dt_ms;
        if self.on.input && self.get_tag_count("on_s") > 0 { self.on_refresh_field(); }
        if self.on.loading && self.on.spin != 0 { let sp = self.on.spin; if let Some(n) = self.nodes.get_mut(sp).and_then(|n| n.as_mut()) { n.rot += dt_ms / 16.0 * 0.1; } }
    }

    pub fn po_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "OnSearchOpen" => { self.kill_tag("cm_"); self.on.kind = 0; self.on_open_search(); true }
            "OnFeatured" => { self.kill_tag("cm_"); self.on.kind = 1; self.on.page = 0; self.on.pages = 10; self.on.levels.clear(); self.on.loading = true; self.on.error.clear(); self.on_open_list(); true }
            "OnSaved" => { self.kill_tag("cm_"); self.on.kind = 2; self.on.page = 0; self.on.pages = 1; self.on.levels.clear(); self.on.loading = true; self.on.error.clear(); self.on_open_list(); true }
            "OnDaily" => { self.on.special = Some(arg as u8); self.on.special_level = None; self.on.loading = true; self.on.error.clear(); self.on_open_special(); true }
            "OnSpecialClose" => { self.kill_tag("on_d"); self.on.special = None; true }
            "OnField" => { self.on.input = true; self.on.blink = 0.0; self.on_refresh_field(); true }
            "OnSearch" => {
                if self.on.query.trim().is_empty() { return true; }
                self.kill_tag("on_s"); self.on.input = false;
                self.on.kind = 0; self.on.page = 0; self.on.pages = 1; self.on.levels.clear(); self.on.loading = true; self.on.error.clear();
                self.on_open_list(); true
            }
            "OnBack" => { self.kill_tag("on_"); self.on.input = false; self.em_open_creator(); true }
            "OnListBack" => {
                self.kill_tag("on_"); self.on.view = None;
                if self.on.kind == 0 { self.on_open_search(); } else { self.em_open_creator(); }
                true
            }
            "OnPage" => {
                if self.on.loading { return true; }
                let n = self.on.pages.max(1);
                self.on.page = (self.on.page + 1) % n;
                self.on.levels.clear(); self.on.loading = true; self.on_open_list(); true
            }
            "OnView" => { self.on_open_view(arg); true }
            "OnViewBack" => { self.kill_tag("on_v"); self.on.view = None; true }
            "OnPlay" | "OnSave" | "OnSpecialPlay" => true, // the host does these
            _ => false,
        }
    }

    pub fn on_page(&self) -> usize { self.on.page }
    pub fn on_kind(&self) -> u8 { self.on.kind }

    fn on_root(&mut self, depth: f64, tag: &'static str) -> usize {
        let r = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, depth, tag);
        r
    }

    // ---- search menu ----
    pub fn on_open_search(&mut self) {
        self.kill_tag("on_");
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        self.em_fade_in(200.0, 300.0, "on_s");
        self.em_gradient(100.0, "on_s");
        let blk = self.em_blocker(101.0, "on_s"); let _ = blk;
        let root = self.on_root(102.0, "on_s");
        let t = self.pa_txt(root, "bigFont", 52.0, "Search", cx, 70.0, 0.5, 0.5, 1.0, 103.0, "on_s"); let _ = t;
        let bw = 760.0;
        let field = self.shape(Kind::RoundRect, cx - bw / 2.0, 190.0, bw, 80.0, 0, 0.35, 103.0, "on_s");
        { let n = self.node(field); n.radius = 20.0; n.interactive = true; n.width = bw; n.height = 80.0; n.action = Action::Pop("OnField", 0); }
        let q = self.on.query.clone();
        self.on.field_text = self.pa_txt(root, "bigFont", 40.0, &q, cx, 230.0, 0.5, 0.5, 1.0, 104.0, "on_s");
        self.on.field_cursor = self.pa_txt(root, "bigFont", 40.0, "|", cx, 230.0, 0.0, 0.5, 1.0, 104.0, "on_s");
        let h = self.pa_txt(root, "bigFont", 26.0, "Level name or ID", cx, 300.0, 0.5, 0.5, 1.0, 104.0, "on_s");
        { let n = self.node(h); n.alpha = 0.6; }
        self.pa_btn(root, "GJ_button01", cx, 400.0, 260.0, 90.0, "Search", "bigFont", 40.0, 0.0, -2.0, Action::Pop("OnSearch", 0), 104.0, "on_s");
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, 105.0, 1.0); { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = "on_s"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 105.0, 1.0); { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "on_s"; }
        self.em_back_arrow(Action::Pop("OnBack", 0), 106.0, "on_s");
        self.on.input = true; self.on.blink = 0.0;
        self.on_refresh_field();
    }

    fn on_refresh_field(&mut self) {
        let (tn, cn) = (self.on.field_text, self.on.field_cursor);
        if self.get(tn).is_none() { return; }
        let q = self.on.query.clone();
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let w = f.width(&q, 40.0);
        let cx = self.screen_width / 2.0;
        let max = 700.0;
        let k = if w > max { max / w } else { 1.0 };
        { let n = self.node(tn); n.text = q; n.width = w; n.sx = k; n.sy = k; }
        let show = self.on.input && ((self.on.blink / 500.0) as i64 % 2 == 0);
        { let n = self.node(cn); n.visible = show; n.x = cx + w * k / 2.0 + 2.0; }
    }

    // ---- list ----
    pub fn on_open_list(&mut self) {
        self.kill_tag("on_l"); self.kill_tag("on_v");
        self.on.view = None;
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        self.em_fade_in(200.0, 300.0, "on_l");
        self.em_gradient(100.0, "on_l");
        self.em_blocker(101.0, "on_l");
        let root = self.on_root(102.0, "on_l");
        let (tw, th) = (712.0, 460.0);
        let (tx, ty) = ((sw - tw) / 2.0, 85.0);
        self.shape(Kind::Rect, tx, ty, tw, th, 0xc2723e, 1.0, 102.0, "on_l");
        let count = self.on.levels.len();
        let list_h = count as f64 * 100.0;
        let (content, si) = self.scroll_area(root, cx, ty + th / 2.0, tw, th, list_h, 103.0, "on_l");
        self.scrolls[si].base_y = ty + 22.0;
        self.scrolls[si].max = 32.0 + (list_h - th).max(0.0);
        self.set_scroll(si, 0.0);
        for (index, l) in self.on.levels.clone().iter().enumerate() {
            let slot = index as f64 * 100.0 + 50.0;
            let d = 103.01;
            let stripe = self.shape(Kind::Rect, cx - (tw - 10.0) / 2.0, slot - 50.0, tw - 10.0, 100.0, if index % 2 != 0 { 0xc2723e } else { 0xa1582c }, 1.0, d, "on_l");
            self.node(stripe).parent = Some(content);
            let sep = self.shape(Kind::Rect, cx - (tw - 10.0) / 2.0, slot + 50.0, tw - 10.0, 1.0, 0x502c16, 1.0, d, "on_l");
            self.node(sep).parent = Some(content);
            self.pa_img(content, "GJ_GameSheet03", &l.face, tx + 50.0, slot - 8.0, 0.9, d + 0.01, "on_l");
            if l.stars > 0 {
                self.pa_txt(content, "bigFont", 20.0, &l.stars.to_string(), tx + 48.0, slot + 34.0, 1.0, 0.5, 1.0, d + 0.01, "on_l");
                self.pa_img(content, "GJ_GameSheet03", "GJ_starsIcon_001.png", tx + 62.0, slot + 34.0, 0.6, d + 0.01, "on_l");
            }
            let name = self.pa_txt(content, "bigFont", 32.0, &l.name, tx + 100.0, slot - 24.0, 0.0, 0.5, 1.0, d + 0.01, "on_l");
            let nw = self.node(name).width;
            let max_w = 350.0;
            if nw > max_w { let k = max_w / nw; let n = self.node(name); n.sx = k; n.sy = k; }
            self.pa_txt(content, "goldFont", 22.0, &format!("by {}", l.author), tx + 100.0, slot + 4.0, 0.0, 0.5, 1.0, d + 0.01, "on_l");
            let info_y = slot + 30.0;
            self.pa_img(content, "GJ_GameSheet03", "GJ_timeIcon_001.png", tx + 108.0, info_y, 0.6, d + 0.01, "on_l");
            self.pa_txt(content, "bigFont", 16.0, length_name(l.length), tx + 128.0, info_y, 0.0, 0.5, 1.0, d + 0.01, "on_l");
            self.pa_img(content, "GJ_GameSheet03", "GJ_downloadsIcon_001.png", tx + 220.0, info_y, 0.6, d + 0.01, "on_l");
            self.pa_txt(content, "bigFont", 16.0, &short(l.downloads), tx + 240.0, info_y, 0.0, 0.5, 1.0, d + 0.01, "on_l");
            self.pa_img(content, "GJ_GameSheet03", "GJ_likesIcon_001.png", tx + 340.0, info_y, 0.6, d + 0.01, "on_l");
            self.pa_txt(content, "bigFont", 16.0, &short(l.likes), tx + 360.0, info_y, 0.0, 0.5, 1.0, d + 0.01, "on_l");
            let zone = self.shape(Kind::Rect, tx + tw - 80.0, slot, 115.0, 55.0, 0, 0.0, d + 0.02, "on_l");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 115.0; n.height = 55.0; n.parent = Some(content); n.sx = 1.05; n.sy = 1.05; }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, 115.0, 55.0, 0, 1.0, d + 0.02, "on_l");
            { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = 24.0; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
            self.pa_txt(zone, "bigFont", 32.0, "View", -2.0, -1.0, 0.5, 0.5, 1.1, d + 0.03, "on_l");
            self.make_bouncy(zone, 0.75, Cond::Always, Action::Pop("OnView", index));
        }
        // status text over an empty list
        let msg = if self.on.loading { Some("Loading...".to_string()) }
            else if !self.on.error.is_empty() { Some(self.on.error.clone()) }
            else if count == 0 { Some(if self.on.kind == 2 { "No saved levels yet".to_string() } else { "No levels found".to_string() }) } else { None };
        if let Some(m) = msg {
            let t = self.text_node("bigFont", 38.0, &m, cx, ty + th / 2.0, 104.0, 0.5, 0.5, 1.0); self.node(t).tag = "on_l";
            if self.on.loading {
                let sp = self.image("", "loadingCircle", cx, ty + th / 2.0 + 70.0, 104.0, 0.7);
                { let n = self.node(sp); n.tag = "on_l"; n.additive = true; n.alpha = 0.5; }
                self.on.spin = sp;
            }
        }
        // frame
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = th / side_h;
        let l = self.image("GJ_WebSheet", "GJ_table_side_001.png", tx - 40.0, ty, 105.0, 1.0); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; n.tag = "on_l"; }
        let r = self.image("GJ_WebSheet", "GJ_table_side_001.png", tx + tw + 40.0, ty, 105.0, 1.0); { let n = self.node(r); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; n.tag = "on_l"; }
        let top = self.image("GJ_WebSheet", "GJ_table_top_001.png", cx, ty - 10.0, 105.0, 1.0); self.node(top).tag = "on_l";
        let bot = self.image("GJ_WebSheet", "GJ_table_bottom_001.png", cx, ty + th + 20.0, 105.0, 1.0); self.node(bot).tag = "on_l";
        let title = match self.on.kind { 1 => "Featured", 2 => "Saved Levels", _ => "Online Levels" };
        let t = self.text_node("bigFont", 42.0, title, cx, ty - 15.0, 106.0, 0.5, 0.5, 1.1); self.node(t).tag = "on_l";
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, 105.0, 1.0); { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = "on_l"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 105.0, 1.0); { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "on_l"; }
        if self.on.kind != 2 {
            // the page button (top right): tap for the next page
            let pb = self.shape(Kind::Rect, sw - 35.0, 80.0, 70.0, 70.0, 0, 0.0, 107.0, "on_l");
            { let n = self.node(pb); n.ox = 0.5; n.oy = 0.5; n.width = 70.0; n.height = 70.0; }
            let bg = self.image("", "GJ_button02", 0.0, 0.0, 107.0, 0.7); { let n = self.node(bg); n.parent = Some(pb); n.tag = "on_l"; }
            let num = self.text_node("bigFont", 35.0, &(self.on.page + 1).to_string(), -2.0, 0.0, 107.1, 0.5, 0.5, 1.0); { let n = self.node(num); n.parent = Some(pb); n.tag = "on_l"; }
            self.make_bouncy(pb, 1.0, Cond::Always, Action::Pop("OnPage", 0));
        }
        self.em_back_arrow(Action::Pop("OnListBack", 0), 107.0, "on_l");
    }

    // ---- one level ----
    pub fn on_open_view(&mut self, idx: usize) {
        let Some(l) = self.on.levels.get(idx).cloned() else { return };
        self.kill_tag("on_v");
        self.on.view = Some(idx);
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.55, 300.0, "on_v");
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        let root = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 300.0, "on_v");
        let (bw, bh) = (760.0, 460.0);
        self.pa_nine(root, "GJ_square01", cx, 320.0, bw, bh, 301.0, "on_v");
        self.pa_close(root, cx - bw / 2.0 + 20.0, 320.0 - bh / 2.0 + 20.0, 1.0, 1.4, Action::Pop("OnViewBack", 0), 302.0, "on_v");
        let name = self.pa_txt(root, "bigFont", 46.0, &l.name, cx, 140.0, 0.5, 0.5, 1.0, 302.0, "on_v");
        let nw = self.node(name).width; if nw > 640.0 { let k = 640.0 / nw; let n = self.node(name); n.sx = k; n.sy = k; }
        self.pa_txt(root, "goldFont", 28.0, &format!("by {}", l.author), cx, 185.0, 0.5, 0.5, 1.0, 302.0, "on_v");
        self.pa_img(root, "GJ_GameSheet03", &l.face, cx - 250.0, 270.0, 1.4, 302.0, "on_v");
        if l.stars > 0 {
            self.pa_img(root, "GJ_GameSheet03", "GJ_starsIcon_001.png", cx - 232.0, 330.0, 0.9, 302.0, "on_v");
            self.pa_txt(root, "bigFont", 28.0, &l.stars.to_string(), cx - 246.0, 330.0, 1.0, 0.5, 1.0, 302.0, "on_v");
        }
        let desc = if l.desc.is_empty() { "(No description provided)".to_string() } else { l.desc.clone() };
        let lines = crate::popups_a::wrap_arial(&desc, 22.0, 460.0);
        let shown: Vec<String> = lines.into_iter().take(4).collect();
        self.pa_txt(root, "arial", 22.0, &shown.join("\n"), cx + 40.0, 260.0, 0.5, 0.5, 1.0, 302.0, "on_v");
        let y = 335.0;
        self.pa_img(root, "GJ_GameSheet03", "GJ_timeIcon_001.png", cx - 240.0, y + 20.0, 0.8, 302.0, "on_v");
        self.pa_txt(root, "bigFont", 22.0, length_name(l.length), cx - 215.0, y + 20.0, 0.0, 0.5, 1.0, 302.0, "on_v");
        self.pa_img(root, "GJ_GameSheet03", "GJ_downloadsIcon_001.png", cx - 80.0, y + 20.0, 0.8, 302.0, "on_v");
        self.pa_txt(root, "bigFont", 22.0, &short(l.downloads), cx - 55.0, y + 20.0, 0.0, 0.5, 1.0, 302.0, "on_v");
        self.pa_img(root, "GJ_GameSheet03", "GJ_likesIcon_001.png", cx + 70.0, y + 20.0, 0.8, 302.0, "on_v");
        self.pa_txt(root, "bigFont", 22.0, &short(l.likes), cx + 95.0, y + 20.0, 0.0, 0.5, 1.0, 302.0, "on_v");
        self.pa_txt(root, "goldFont", 22.0, &format!("ID: {}", l.id), cx + 230.0, y + 20.0, 0.5, 0.5, 1.0, 302.0, "on_v");
        let play = self.pa_img(root, "GJ_GameSheet03", "GJ_playBtn2_001.png", cx - 70.0, 470.0, 0.9, 302.0, "on_v");
        self.make_bouncy(play, 0.9, Cond::Always, Action::Pop("OnPlay", idx));
        let save = self.pa_img(root, "GJ_GameSheet03", "GJ_downloadBtn_001.png", cx + 70.0, 470.0, 0.9, 302.0, "on_v");
        self.make_bouncy(save, 0.9, Cond::Always, Action::Pop("OnSave", idx));
    }

    // ---- daily / weekly / event ----
    pub fn on_open_special(&mut self) {
        self.kill_tag("on_d");
        let Some(kind) = self.on.special else { return };
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.55, 300.0, "on_d");
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        let root = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 300.0, "on_d");
        let (bw, bh) = (700.0, 420.0);
        self.pa_nine(root, "GJ_square01", cx, 320.0, bw, bh, 301.0, "on_d");
        self.pa_close(root, cx - bw / 2.0 + 20.0, 320.0 - bh / 2.0 + 20.0, 1.0, 1.4, Action::Pop("OnSpecialClose", 0), 302.0, "on_d");
        let title = match kind { 0 => "Daily Level", 1 => "Weekly Level", _ => "Event Level" };
        self.pa_txt(root, "bigFont", 46.0, title, cx, 150.0, 0.5, 0.5, 1.0, 302.0, "on_d");
        if let Some(l) = self.on.special_level.clone() {
            let name = self.pa_txt(root, "bigFont", 40.0, &l.name, cx, 225.0, 0.5, 0.5, 1.0, 302.0, "on_d");
            let nw = self.node(name).width; if nw > 580.0 { let k = 580.0 / nw; let n = self.node(name); n.sx = k; n.sy = k; }
            self.pa_txt(root, "goldFont", 26.0, &format!("by {}", l.author), cx, 270.0, 0.5, 0.5, 1.0, 302.0, "on_d");
            self.pa_img(root, "GJ_GameSheet03", &l.face, cx - 150.0, 340.0, 1.2, 302.0, "on_d");
            if l.stars > 0 { self.pa_txt(root, "bigFont", 28.0, &format!("{}", l.stars), cx - 160.0, 395.0, 1.0, 0.5, 1.0, 302.0, "on_d"); self.pa_img(root, "GJ_GameSheet03", "GJ_starsIcon_001.png", cx - 142.0, 395.0, 0.8, 302.0, "on_d"); }
            let play = self.pa_img(root, "GJ_GameSheet03", "GJ_playBtn2_001.png", cx + 120.0, 355.0, 0.9, 302.0, "on_d");
            self.make_bouncy(play, 0.9, Cond::Always, Action::Pop("OnSpecialPlay", 0));
        } else {
            let m = if self.on.loading { "Loading...".to_string() } else if !self.on.error.is_empty() { self.on.error.clone() } else { "Nothing here".to_string() };
            self.pa_txt(root, "bigFont", 36.0, &m, cx, 320.0, 0.5, 0.5, 1.0, 302.0, "on_d");
        }
    }
}

fn short(n: i64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) } else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) } else { n.to_string() }
}
