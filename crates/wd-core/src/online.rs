//! Online levels, laid out like game-scene.js: `_openSearchMenu`, `_openListScene` + `_openOnlineLevelsScene` (the list and its level cells),
//! `_openPlayMenu` (the level screen). The host (web/online.js) talks to the Geometry Dash servers and hands the results over as JSON; every
//! button raises a `Pop("On...", n)` event for it.
use crate::jsonread;
use crate::ui::*;

#[derive(Clone, Debug, Default)]
pub struct OnLevel {
    pub id: String, pub name: String, pub author: String, pub desc: String, pub song_name: String, pub custom_song: String,
    /// 0 N/A .. 5 insane, 6-10 the demons, 11 auto
    pub difficulty: usize,
    pub stars: i64, pub downloads: i64, pub likes: i64, pub length: i64,
    pub featured: bool, pub epic: i64, pub coins: i64, pub daily_number: i64,
    /// best normal / practice percent saved for this level, and whether it was viewed before
    pub best: f64, pub best_practice: f64, pub viewed: bool,
}

#[derive(Default)]
pub struct On {
    pub levels: Vec<OnLevel>,
    /// 0 text search, 1 featured (type 6), 2 saved, 10 + n the quick search of type n
    pub kind: u8,
    pub page: usize, pub pages: usize, pub total: i64, pub offset: i64, pub count: i64,
    pub loading: bool, pub error: String,
    pub query: String, pub input: bool,
    pub typed: usize, pub placeholder: usize, pub cursor: usize,
    /// the active difficulty filters (the eight icons) and their nodes
    pub filters: [bool; 8], pub filter_nodes: Vec<usize>, pub plus: usize,
    pub spin: usize, pub sp_root: usize,
    /// a level asked for by its number: just that level, without the page controls
    pub single: bool,
    /// the demon filter: the chosen tier (0 none, 1-5), the one picked in the open popup and its icons
    pub demon_tier: usize, pub demon_sel: usize, pub demon_icons: Vec<usize>,
    pub song_name_node: usize, pub song_author_node: usize,
    /// the level shown on the play menu (index into `levels`)
    pub view: Option<usize>,
    pub special: Option<u8>, pub special_level: Option<OnLevel>,
}

const LENGTHS: [&str; 5] = ["Tiny", "Short", "Medium", "Long", "XL"];
fn length_name(v: i64) -> &'static str { if v == 5 { "Plat." } else { LENGTHS.get(v.max(0) as usize).copied().unwrap_or("Tiny") } }

/// `_formatStatCount`
fn stat_count(n: i64) -> String {
    if n < 1000 { return n.to_string(); }
    let fmt = |v: f64| if v >= 100.0 { format!("{}", v.round() as i64) } else { let s = format!("{:.1}", v); s.trim_end_matches('0').trim_end_matches('.').to_string() };
    if n < 1_000_000 { format!("{}K", fmt(n as f64 / 1000.0)) } else { format!("{}M", fmt(n as f64 / 1_000_000.0)) }
}
/// `toLocaleString("en-US")`
fn grouped(n: i64) -> String {
    let s = n.abs().to_string(); let mut out = String::new();
    for (i, c) in s.chars().enumerate() { if i > 0 && (s.len() - i) % 3 == 0 { out.push(','); } out.push(c); }
    if n < 0 { format!("-{}", out) } else { out }
}
/// (width, height) of the difficulty faces
fn diff_size(idx: usize) -> (f64, f64) {
    [(60.0, 85.0), (60.0, 85.0), (86.0, 85.0), (60.0, 84.0), (85.0, 84.0), (76.0, 85.0), (72.0, 88.0), (72.0, 85.0), (72.0, 85.0), (74.0, 88.0), (80.0, 91.0), (60.0, 85.0)][idx.min(11)]
}
fn diff_frame(idx: usize, play_menu: bool) -> String {
    match idx { 11 => "difficulty_auto_btn_001.png".into(), 6..=10 if play_menu => format!("difficulty_{:02}_btn2_001.png", idx), i => format!("difficulty_{:02}_btn_001.png", i.min(10)) }
}
fn coin_frame(l: &OnLevel) -> Option<&'static str> {
    if l.epic >= 3 { Some("GJ_epicCoin3_001.png") } else if l.epic == 2 { Some("GJ_epicCoin2_001.png") } else if l.epic == 1 { Some("GJ_epicCoin_001.png") } else if l.featured { Some("GJ_featuredCoin_001.png") } else { None }
}
const OFFICIAL: [&str; 22] = ["Stereo Madness", "Back On Track", "Polargeist", "Dry Out", "Base After Base", "Can't Let Go", "Jumper", "Time Machine", "Cycles", "xStep", "Clutterfunk",
    "Theory of Everything", "Electroman Adventures", "Clubstep", "Electrodynamix", "Hexagon Force", "Blast Processing", "Theory of Everything 2", "Geometrical Dominator", "Deadlocked", "Fingerdash", "Dash"];

impl Ui {
    /// `[ {id, name, author, desc, songName, customSong, difficulty, stars, downloads, likes, length, featured, epic, coins, best, bestPractice, viewed}, ... ]`
    /// plus the page info (`total:offset:count`)
    pub fn on_set_results(&mut self, json: &str, total: i64, offset: i64, count: i64) {
        let mut out = Vec::new();
        if let Ok(j) = jsonread::parse(json) { for e in j.arr().unwrap_or(&[]) { out.push(Self::on_parse_level(e)); } }
        self.on.levels = out;
        self.on.single = total < 0;
        let total = total.abs();
        self.on.total = total; self.on.offset = offset; self.on.count = count.max(1);
        self.on.pages = if self.on.kind == 1 { 10 } else if self.on.kind == 2 { ((self.on.levels.len() as f64 / 10.0).ceil() as usize).max(1) } else { ((total as f64 / count.max(1) as f64).ceil() as usize).max(1) };
        self.on.loading = false; self.on.error.clear();
        if self.get_tag_count("on_l") > 0 { self.on_open_list(); }
    }

    fn on_parse_level(e: &jsonread::Json) -> OnLevel {
        let s = |k: &str| e.get(k).map(|v| v.str().map(|x| x.to_string()).or_else(|| v.num().map(|n| format!("{}", n as i64))).unwrap_or_default()).unwrap_or_default();
        let n = |k: &str| e.get_num(k).unwrap_or(0.0) as i64;
        OnLevel {
            id: s("id"), name: s("name"), author: s("author"), desc: s("desc"), song_name: s("songName"), custom_song: s("customSong"),
            difficulty: (n("difficulty").max(0) as usize).min(11), stars: n("stars"), downloads: n("downloads"), likes: n("likes"), length: n("length"),
            featured: n("featured") != 0, epic: n("epic"), coins: n("coins"), daily_number: n("dailyNumber"),
            best: e.get_num("best").unwrap_or(0.0), best_practice: e.get_num("bestPractice").unwrap_or(0.0), viewed: n("viewed") != 0,
        }
    }

    pub fn on_set_error(&mut self, msg: &str) {
        self.on.loading = false; self.on.error = msg.to_string();
        if self.get_tag_count("on_l") > 0 { self.on_open_list(); }
        if self.on.special.is_some() { self.on_special_cell(); }
    }

    /// the daily / weekly / event level for the open popup
    pub fn on_set_special(&mut self, json: &str) {
        if let Ok(j) = jsonread::parse(json) { self.on.special_level = Some(Self::on_parse_level(&j)); }
        self.on.loading = false; self.on.error.clear();
        if self.on.special.is_some() { self.on_special_cell(); }
    }

    /// the song's real name and artist, once `getGJSongInfo.php` answered
    pub fn on_set_song(&mut self, name: &str, artist: &str) {
        let (sn, sa) = (self.on.song_name_node, self.on.song_author_node);
        let box_w = self.screen_width * 0.55;
        if self.get(sn).is_none() || self.get_tag_count("on_m") == 0 { return; }
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default(); let g = self.fonts.get("goldFont").cloned().unwrap_or_default();
        if !artist.is_empty() { let w = g.width(&format!("By: {}", artist), 36.0); let n = self.node(sa); n.text = format!("By: {}", artist); n.width = w; self.on_fit(sa, box_w - 100.0, 0.3); }
        if !name.is_empty() { let w = f.width(name, 50.0); let n = self.node(sn); n.text = name.to_string(); n.width = w; self.on_fit(sn, box_w - 100.0, 0.3); }
    }

    pub fn on_query(&self) -> String { self.on.query.clone() }
    /// the `diff` search parameter of the active filters (`_getActiveDiffParam`), "" when none
    pub fn on_diff_param(&self) -> String {
        const CODES: [i32; 8] = [-1, 1, 2, 3, 4, 5, -2, -3];
        self.on.filters.iter().enumerate().filter(|(_, a)| **a).map(|(i, _)| CODES[i].to_string()).collect::<Vec<_>>().join(",")
    }

    const ALLOWED: &'static str = " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    pub fn on_key(&mut self, code: u32) {
        match code {
            8 => { self.on.query.pop(); }
            13 => { self.po_action("OnSearch", 0); self.events.push(Action::Pop("OnSearch", 0)); return; }
            c => if let Some(c) = char::from_u32(c) { if Self::ALLOWED.contains(c) && self.on.query.chars().count() < 20 { self.on.query.push(c); } },
        }
        self.on_refresh_field();
    }

    pub fn on_step(&mut self, dt_ms: f64) {
        if self.get_tag_count("on_") == 0 { return; }
        if self.on.spin != 0 && self.on.loading { let sp = self.on.spin; if let Some(n) = self.nodes.get_mut(sp).and_then(|n| n.as_mut()) { n.rot += dt_ms / 16.0 * 0.1; } }
    }

    pub fn po_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "OnSearchOpen" => { self.kill_tag("cm_"); self.on.kind = 0; self.on_open_search(); true }
            "OnFeatured" => { self.kill_tag("cm_"); self.on_begin_list(1); true }
            "OnSaved" => { self.kill_tag("cm_"); self.on_begin_list(2); true }
            "OnDemon" => { self.on_open_demon(); true }
            "OnDemonTier" => { self.on.demon_sel = arg; for (i, &nid) in self.on.demon_icons.clone().iter().enumerate() { if let Some(n) = self.nodes.get_mut(nid).and_then(|n| n.as_mut()) { n.tint = if i == arg { 0xffffff } else { 0x8f8f8f }; } } true }
            "OnDemonOk" => {
                const FR: [&str; 6] = ["difficulty_06_btn_001.png", "difficulty_07_btn2_001.png", "difficulty_08_btn2_001.png", "difficulty_06_btn2_001.png", "difficulty_09_btn2_001.png", "difficulty_10_btn2_001.png"];
                self.on.demon_tier = self.on.demon_sel;
                let ic = self.on.filter_nodes.get(6).copied();
                if let Some(i) = ic { if let Some(n) = self.nodes.get_mut(i).and_then(|n| n.as_mut()) { n.frame = FR[self.on.demon_sel].into(); n.sx = 0.85; n.sy = 0.85; } }
                self.kill_tag("on_p"); true
            }
            // the delete button asks the host (confirm, remove from the saved list) and the host closes the menu through `wd_on_menu_back`
            "OnDeleteLevel" => true,
            "OnDeleteSaved" => { self.on.levels.clear(); self.on.page = 0; self.on.pages = 1; self.on_open_list(); true }
            "OnDaily" => { self.on.special = Some(arg as u8); self.on.special_level = None; self.on.loading = true; self.on.error.clear(); self.on_open_special(); true }
            "OnSpecialClose" => { self.kill_tag("on_d"); self.on.special = None; true }
            "OnField" => { self.on.input = true; self.on_refresh_field(); true }
            "OnClear" => { self.on.query.clear(); self.on_refresh_field(); true }
            "OnFilter" => { self.on_filter(arg); true }
            "OnQuick" => { self.kill_tag("on_s"); self.on.input = false; self.on_begin_list(10 + arg as u8); true }
            "OnSearch" => { self.kill_tag("on_s"); self.on.input = false; self.on_begin_list(0); true }
            "OnBack" => { self.kill_tag("on_"); self.on.input = false; self.em_open_creator(); true }
            "OnListBack" => {
                self.kill_tag("on_l"); self.kill_tag("on_m");
                if self.on.kind == 1 || self.on.kind == 2 { self.em_open_creator(); } else { self.on_open_search(); }
                true
            }
            "OnPage" => { // the round number button: the next page, wrapping round
                if self.on.loading { return true; }
                let n = self.on.pages.max(1);
                self.on.page = (self.on.page + 1) % n; self.on_page_loading(); true
            }
            "OnPrev" => { if !self.on.loading && self.on.page > 0 { self.on.page -= 1; self.on_page_loading(); } true }
            "OnNext" => { if !self.on.loading { self.on.page += 1; self.on_page_loading(); } true }
            "OnRefresh" => { if !self.on.loading { self.on_page_loading(); } true }
            "OnView" => { self.kill_tag("on_l"); self.on_open_menu(arg); true }
            "OnMenuBack" => {
                self.kill_tag("on_m"); self.on.view = None;
                if self.on.kind == 3 { self.em_open_creator(); self.on_open_special(); } else { self.on_open_list(); }
                true
            }
            "OnMenuRefresh" => { if let Some(i) = self.on.view { self.on_open_menu(i); } true }
            "OnSpecialPlay" => { // the play menu of the daily / weekly / event level; back returns to the creator menu and the popup
                if let Some(l) = self.on.special_level.clone() { self.kill_tag("on_d"); self.kill_tag("cm_"); self.on.levels = vec![l]; self.on.kind = 3; self.on_open_menu(0); }
                true
            }
            "OnPlay" | "OnSave" | "OnCopy" | "OnSongDownload" | "OnInfo" => true, // the host does these
            _ => false,
        }
    }

    pub fn on_page(&self) -> usize { self.on.page }
    pub fn on_kind(&self) -> u8 { self.on.kind }

    fn on_begin_list(&mut self, kind: u8) {
        self.on.kind = kind; self.on.single = false; self.on.page = 0; self.on.pages = 1; self.on.total = 0; self.on.levels.clear();
        self.on.loading = true; self.on.error.clear();
        self.on_open_list();
    }
    fn on_page_loading(&mut self) { self.on.levels.clear(); self.on.loading = true; self.on.error.clear(); self.on_open_list(); }

    fn on_filter(&mut self, i: usize) {
        let was = self.on.filters[i];
        let exclusive = [0usize, 6, 7];
        if exclusive.contains(&i) { self.on.filters = [false; 8]; if !was { self.on.filters[i] = true; } }
        else { for e in exclusive { self.on.filters[e] = false; } self.on.filters[i] = !was; }
        self.on_paint_filters();
    }
    fn on_paint_filters(&mut self) {
        for (i, &nid) in self.on.filter_nodes.clone().iter().enumerate() { if let Some(n) = self.nodes.get_mut(nid).and_then(|n| n.as_mut()) { n.tint = if self.on.filters[i] { -1 } else { 0x666666 }; } }
        let plus = self.on.plus; let show = self.on.filters[6];
        if let Some(n) = self.nodes.get_mut(plus).and_then(|n| n.as_mut()) { n.visible = show; }
    }

    // ---- shared pieces ----
    /// the green back arrow of the search menu and the play menu (`GJ_arrow_01_001`, flipped both ways and turned half round)
    fn on_back_arrow(&mut self, action: Action, depth: f64, tag: &'static str) {
        let b = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", 50.0, 48.0, depth, 1.0);
        { let n = self.node(b); n.flip_x = true; n.flip_y = true; n.rot = std::f64::consts::PI; n.tag = tag; }
        self.make_bouncy(b, 1.0, Cond::Always, action);
    }
    fn on_corners(&mut self, depth: f64, tag: &'static str) {
        let (sw, sh) = (self.screen_width, 640.0);
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, depth, 1.0); { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = tag; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, depth, 1.0); { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = tag; }
    }
    fn on_txt(&mut self, root: Option<usize>, font: &str, size: f64, text: &str, x: f64, y: f64, ox: f64, oy: f64, depth: f64, tag: &'static str) -> usize {
        let t = self.text_node(font, size, text, x, y, depth, ox, oy, 1.0);
        { let n = self.node(t); n.parent = root; n.tag = tag; }
        t
    }
    fn on_img(&mut self, root: Option<usize>, atlas: &str, frame: &str, x: f64, y: f64, scale: f64, depth: f64, tag: &'static str) -> usize {
        let i = self.image(atlas, frame, x, y, depth, scale);
        { let n = self.node(i); n.parent = root; n.tag = tag; }
        i
    }
    /// `_fitBitmapText`: shrinks the text to `max_w`
    fn on_fit(&mut self, i: usize, max_w: f64, min_scale: f64) {
        let w = self.node(i).width;
        let k = if w > max_w { (max_w / w).max(min_scale) } else { 1.0 };
        let n = self.node(i); n.sx = k; n.sy = k;
    }
    fn on_disp_w(&self, i: usize) -> f64 { self.get(i).map_or(0.0, |n| n.width * n.sx) }

    // ---- the search menu (`_openSearchMenu`) ----
    pub fn on_open_search(&mut self) {
        self.kill_tag("on_");
        let (sw, sh) = (self.screen_width, 640.0);
        let tag = "on_s";
        self.em_fade_in(200.0, 300.0, tag);
        self.em_gradient(100.0, tag);
        self.em_blocker(101.0, tag);
        self.on_back_arrow(Action::Pop("OnBack", 0), 104.0, tag);
        self.on_corners(152.0, tag);
        let panel_left = sw * 0.18; let panel_right = sw - sw * 0.18; let panel_w = panel_right - panel_left;
        let rr = |ui: &mut Ui, x: f64, y: f64, w: f64, h: f64, color: i64| { let p = ui.shape(Kind::RoundRect, x, y, w, h, color, 0.7, 104.0, tag); ui.node(p).radius = 10.0; p };
        // the top panel: the text box and its three buttons
        let top_y = sh * 0.10 - 40.0; let top_h = sh * 0.12;
        rr(self, panel_left, top_y, panel_w, top_h, 0x00388d);
        let (ix, iy, iw, ih) = (panel_left + 10.0, top_y + 10.0, panel_w * 0.57, top_h - 20.0);
        rr(self, ix, iy, iw, ih, 0x002762);
        let (bx, by) = (ix + iw + 20.0, iy + ih / 2.0);
        for (dx, frame, action) in [(137.0, "GJ_longBtn05_001.png", Action::None), (47.0, "GJ_longBtn06_001.png", Action::Pop("OnSearch", 0)), (231.0, "GJ_longBtn07_001.png", Action::Pop("OnClear", 0))] {
            let b = self.image("GJ_GameSheet03", frame, bx + dx, by, 105.0, 1.0); self.node(b).tag = tag;
            self.make_bouncy(b, 1.0, Cond::Always, action);
        }
        let (cx_in, cy_in) = (ix + iw / 2.0, iy + ih / 2.0);
        self.on.placeholder = self.on_txt(None, "bigFont", 28.0, "Enter a level, user or id", cx_in - 5.0, cy_in, 0.5, 0.5, 106.0, tag);
        { let p = self.on.placeholder; let n = self.node(p); n.color = 0x6c99d8; n.alpha = 0.85; }
        self.on.typed = self.on_txt(None, "bigFont", 46.0, "", ix + 10.0, cy_in, 0.0, 0.5, 106.0, tag);
        self.on.cursor = self.on_txt(None, "arial", 33.0, "|", 0.0, cy_in, 0.5, 0.5, 106.0, tag);
        { let c = self.on.cursor; let n = self.node(c); n.color = 0x92a7c0; n.visible = false; }
        let hit = self.shape(Kind::Rect, ix, iy, iw, ih, 0, 0.0, 107.0, tag);
        { let n = self.node(hit); n.interactive = true; n.width = iw; n.height = ih; n.ox = 0.0; n.oy = 0.0; n.action = Action::Pop("OnField", 0); }
        // Quick Search
        let qs_label_y = sh * 0.195; let qs_y = qs_label_y + 25.0; let qs_h = sh * 0.36;
        self.on_txt(None, "bigFont", 32.0, "Quick Search", sw / 2.0, qs_label_y, 0.5, 0.5, 105.0, tag);
        rr(self, panel_left, qs_y, panel_w, qs_h, 0x002e75);
        let (pad_x, pad_y, gap_x, gap_y) = (14.0, 14.5, 10.0, 12.0);
        let btn_w = (panel_w - pad_x * 2.0 - gap_x * 2.0) / 3.0;
        let btn_h = (qs_h - pad_y * 2.0 - gap_y * 2.0) / 3.0;
        let lb_h = self.sizes.get("GJ_GameSheet03/GJ_longBtn01_001.png").map_or(60.0, |s| s.1);
        let lb_border = (lb_h * 0.28).round();
        let labels = [["Downloads", "Likes", "Sent"], ["Trending", "Recent", "Magic"], ["Awarded", "Followed", "Friends"]];
        let icons = [["GJ_sDownloadIcon_001.png", "GJ_sLikeIcon_001.png", "GJ_sModIcon_001.png"], ["GJ_sTrendingIcon_001.png", "GJ_sRecentIcon_001.png", "GJ_sMagicIcon_001.png"], ["GJ_sStarsIcon_001.png", "GJ_sFollowedIcon_001.png", "GJ_sFriendsIcon_001.png"]];
        let icon_dim = |f: &str| -> (f64, f64) { match f { "GJ_sDownloadIcon_001.png" => (30.0, 30.0), "GJ_sLikeIcon_001.png" => (31.0, 37.0), "GJ_sModIcon_001.png" => (31.0, 31.0), "GJ_sTrendingIcon_001.png" => (33.0, 25.0),
            "GJ_sRecentIcon_001.png" => (31.0, 29.0), "GJ_sMagicIcon_001.png" => (33.0, 30.0), "GJ_sStarsIcon_001.png" => (30.0, 29.0), "GJ_sFollowedIcon_001.png" => (31.0, 29.0), _ => (35.0, 29.0) } };
        for row in 0..3 { for col in 0..3 {
            let (cx, cy) = (panel_left + pad_x + col as f64 * (btn_w + gap_x) + btn_w / 2.0, qs_y + pad_y + row as f64 * (btn_h + gap_y) + btn_h / 2.0);
            let label = labels[row][col]; let icon = icons[row][col];
            let ty = match label { "Downloads" => 1, "Likes" => 2, "Trending" => 3, "Recent" => 4, "Magic" => 7, "Awarded" => 11, _ => 0 };
            let zone = self.shape(Kind::Rect, cx, cy, btn_w, btn_h, 0, 0.0, 105.0, tag);
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = btn_w; n.height = btn_h; }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, btn_w, btn_h, 0, 1.0, 105.0, tag);
            { let n = self.node(nine); n.frame = "GJ_GameSheet03/GJ_longBtn01_001.png".into(); n.radius = lb_border.min(btn_w / 2.0).min(btn_h / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
            let fs = if label == "Downloads" { 26.0 } else { 32.0 };
            let measured = self.fonts.get("bigFont").map_or(1.0, |f| f.width(label, fs));
            let (iw0, ih0) = icon_dim(icon);
            let isc = 1f64.min((btn_h - 12.0).max(0.0) / iw0.max(ih0));
            let iconw = iw0 * isc;
            let gap = 10.0;
            let lscale = ((btn_w - 24.0 - iconw - gap) / measured).min(1.0).max(0.0);
            let group_w = measured * lscale + gap + iconw;
            let left = -group_w / 2.0;
            let l = self.on_txt(Some(zone), "bigFont", fs, label, left, -1.0, 0.0, 0.5, 106.0, tag);
            { let n = self.node(l); n.sx = lscale; n.sy = lscale; }
            let icx = left + measured * lscale + gap + iconw / 2.0;
            let iy = if icon == "GJ_sLikeIcon_001.png" { 2.0 } else if label == "Downloads" { -2.0 } else { -1.0 };
            let ic = self.on_img(Some(zone), "GJ_GameSheet03", icon, icx, iy, isc, 106.0, tag);
            if ty == 0 { self.lock_gray(nine); self.lock_gray(l); self.lock_gray(ic); }
            let act = if ty == 0 { Action::None } else { Action::Pop("OnQuick", ty) };
            self.make_bouncy(zone, 1.0, Cond::Always, act);
        } }
        // Filters
        let f_label_y = qs_y + qs_h + 24.0; let f_y = f_label_y + 20.0; let f_h = sh * 0.16;
        self.on_txt(None, "bigFont", 32.0, "Filters", sw / 2.0, f_label_y, 0.5, 0.5, 105.0, tag);
        rr(self, panel_left, f_y, panel_w, f_h, 0x00245b);
        let frames = ["difficulty_00_btn_001.png", "difficulty_01_btn_001.png", "difficulty_02_btn_001.png", "difficulty_03_btn_001.png", "difficulty_04_btn_001.png", "difficulty_05_btn_001.png", "difficulty_06_btn_001.png", "difficulty_auto_btn_001.png"];
        let slot_w = panel_w / 8.0;
        self.on.filter_nodes.clear();
        for (i, f) in frames.iter().enumerate() {
            let ic = self.image("GJ_GameSheet03", f, panel_left + slot_w * i as f64 + slot_w / 2.0, f_y + f_h / 2.0, 105.0, 0.85);
            { let n = self.node(ic); n.tag = tag; n.tint = 0x666666; }
            self.make_bouncy(ic, 0.85, Cond::Always, Action::Pop("OnFilter", i));
            self.on.filter_nodes.push(ic);
        }
        let plus = self.image("GJ_GameSheet03", "GJ_plus2Btn_001.png", panel_right + 42.0, f_y + f_h / 2.0, 105.0, 1.0);
        { let n = self.node(plus); n.tag = tag; n.visible = false; }
        self.make_bouncy(plus, 1.0, Cond::Always, Action::Pop("OnDemon", 0));
        self.on.plus = plus;
        // the extra panel
        let ex_y = f_y + f_h + 18.0; let ex_h = sh * 0.11;
        rr(self, panel_left, ex_y, panel_w, ex_h, 0x001f4f);
        let cs = self.on_txt(None, "bigFont", 42.0, "Coming Soon!", sw / 2.0, ex_y + ex_h / 2.0, 0.5, 0.5, 105.0, tag);
        { let n = self.node(cs); n.color = 0xadd8e6; n.alpha = 0.75; }
        self.on_paint_filters();
        self.on.input = false;
        self.on_refresh_field();
    }

    fn on_refresh_field(&mut self) {
        let (tn, pn, cn) = (self.on.typed, self.on.placeholder, self.on.cursor);
        if self.get(tn).is_none() { return; }
        let sw = self.screen_width;
        let panel_left = sw * 0.18; let panel_w = sw - sw * 0.36;
        let (ix, iw) = (panel_left + 10.0, panel_w * 0.57);
        let q = self.on.query.clone();
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let w = f.width(&q, 46.0);
        let max_w = iw - 24.0;
        let k = if w > max_w { max_w / w } else { 1.0 };
        { let n = self.node(tn); n.text = q.clone(); n.width = w; n.sx = k; n.sy = k; n.visible = !q.is_empty(); }
        { let n = self.node(pn); n.visible = q.is_empty(); }
        let show = self.on.input;
        { let n = self.node(cn); n.visible = show; n.x = ix + 10.0 + if q.is_empty() { 0.0 } else { w * k } + 4.0; }
    }

    // ---- the list (`_openListScene` + `_openOnlineLevelsScene`) ----
    pub fn on_open_list(&mut self) {
        self.kill_tag("on_l");
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0;
        let tag = "on_l";
        self.em_fade_in(300.0, 280.0, tag);
        // the blue gradient 0x0066ff -> 0x003399
        let steps = 80;
        for gi in 0..steps {
            let t = gi as f64 / (steps - 1) as f64;
            let g = (0x66 as f64 * (1.0 - t) + 0x33 as f64 * t).round() as i64; let b = (0xff as f64 * (1.0 - t) + 0x99 as f64 * t).round() as i64;
            let y = (gi as f64 * sh / steps as f64).floor(); let h = (sh / steps as f64).ceil() + 1.0;
            self.shape(Kind::Rect, 0.0, y, sw, h, (g << 8) | b, 1.0, 200.0, tag);
        }
        self.em_blocker(200.0, tag);
        self.on_corners(201.0, tag);
        let (pw, ph) = (712.0, 460.0);
        let (list_left, list_top) = (cx - pw / 2.0, 320.0 - ph / 2.0 + 10.0);
        self.shape(Kind::Rect, list_left, list_top, pw, ph, 0xC2723E, 1.0, 201.0, tag);
        let root = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 202.0, tag);
        let saved = self.on.kind == 2;
        let (first, shown) = if saved { let a = (self.on.page * 10).min(self.on.levels.len()); (a, self.on.levels.len().min(a + 10) - a) } else { (0, self.on.levels.len()) };
        let count = shown;
        let row_h = 180.0;
        let view_top = list_top + 12.0; let view_bottom = list_top + ph - 22.0; let view_h = view_bottom - view_top;
        let list_h = count as f64 * row_h;
        let (content, si) = self.scroll_area(root, cx, view_top + view_h / 2.0, pw, view_h, list_h, 202.0, tag);
        self.scrolls[si].base_y = view_top;
        self.scrolls[si].max = (list_h - view_h).max(0.0);
        self.set_scroll(si, 0.0);
        let levels = self.on.levels.clone();
        for (idx, l) in levels.iter().skip(first).take(shown).enumerate() { self.on_cell(content, idx, first + idx, l, list_left, pw); }
        if count == 0 && !self.on.loading {
            if saved { self.on_txt(None, "bigFont", 38.0, "You have not downloaded any ", cx, 320.0, 0.5, 0.5, 204.0, tag); self.on_txt(None, "bigFont", 38.0, "levels yet! ", cx + 20.0, 360.0, 0.5, 0.5, 204.0, tag); }
            else { let m = if !self.on.error.is_empty() { self.on.error.clone() } else { "No levels found".to_string() }; self.on_txt(None, "bigFont", 40.0, &m, cx, 320.0, 0.5, 0.5, 204.0, tag); }
        }
        // the table frame
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = ph / side_h;
        let l = self.image("GJ_WebSheet", "GJ_table_side_001.png", list_left - 40.0, 90.0, 203.0, 1.0); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; n.tag = tag; }
        let r = self.image("GJ_WebSheet", "GJ_table_side_001.png", list_left + pw + 40.0, 90.0, 203.0, 1.0); { let n = self.node(r); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; n.tag = tag; }
        let top = self.image("GJ_WebSheet", "GJ_table_top_001.png", cx, 80.0, 203.0, 1.0); self.node(top).tag = tag;
        let bot = self.image("GJ_WebSheet", "GJ_table_bottom_001.png", cx, 570.0, 203.0, 1.0); self.node(bot).tag = tag;
        let featured = self.on.kind == 1;
        if featured { let h = self.image("GJ_GameSheet03", "featuredLabel_001.png", sw / 2.0, 320.0 - 265.0, 204.0, 1.0); self.node(h).tag = tag; }
        else { let title = if self.on.kind == 2 { "Saved Levels" } else { "Online Levels" }; self.on_txt(None, "bigFont", 52.0, title, cx, 320.0 - 245.0, 0.5, 0.5, 204.0, tag); }
        // page info (top right), the page button, arrows, refresh
        if self.on.single {
            let back = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", 45.0, 45.0, 204.0, 1.0); self.node(back).tag = tag;
            self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("OnListBack", 0));
            return;
        }
        if self.on.kind != 2 && !self.on.loading && self.on.total > 0 {
            let start = self.on.offset + 1; let end = self.on.count * (self.on.page as i64 + 1);
            self.on_txt(None, "goldFont", 22.0, &format!("{} to {} of {}", start, end, self.on.total), sw - 8.0, 3.0, 1.0, 0.0, 204.0, tag);
        }
        {
            let pb = self.shape(Kind::Rect, sw - 35.0, sh / 2.0 - 240.0, 70.0, 70.0, 0, 0.0, 205.0, tag);
            { let n = self.node(pb); n.ox = 0.5; n.oy = 0.5; n.width = 70.0; n.height = 70.0; }
            let bg = self.image("", "GJ_button02", 0.0, 0.0, 205.0, 0.7); { let n = self.node(bg); n.parent = Some(pb); n.tag = tag; }
            let num = self.text_node("bigFont", 35.0, &(self.on.page + 1).to_string(), -2.0, 0.0, 205.1, 0.5, 0.5, 1.0); { let n = self.node(num); n.parent = Some(pb); n.tag = tag; }
            self.make_bouncy(pb, 1.0, Cond::Always, Action::Pop("OnPage", 0));
            if self.on.page > 0 {
                let p = self.image("GJ_GameSheet03", "GJ_arrow_03_001.png", 40.0, sh / 2.0, 204.0, 1.0); self.node(p).tag = tag;
                self.make_bouncy(p, 1.0, Cond::Always, Action::Pop("OnPrev", 0));
            }
            if (!featured || saved) && self.on.page + 1 < self.on.pages && !self.on.loading {
                let n2 = self.image("GJ_GameSheet03", "GJ_arrow_03_001.png", sw - 40.0, sh / 2.0, 204.0, 1.0); { let n = self.node(n2); n.flip_x = true; n.tag = tag; }
                self.make_bouncy(n2, 1.0, Cond::Always, Action::Pop("OnNext", 0));
            }
        }
        if self.on.loading {
            let sp = self.image("", "loadingCircle", cx, 320.0, 205.0, 1.0);
            { let n = self.node(sp); n.tag = tag; n.additive = true; n.alpha = 0.5; }
            self.on.spin = sp;
        } else if saved {
            let rf = self.image("GJ_GameSheet03", "GJ_deleteBtn_001.png", sw - 55.0, sh - 55.0, 204.0, 1.0); self.node(rf).tag = tag;
            self.make_bouncy(rf, 1.0, Cond::Always, Action::Pop("OnDeleteSaved", 0));
        } else {
            let rf = self.image("GJ_GameSheet03", "GJ_updateBtn_001.png", sw - 55.0, sh - 55.0, 204.0, 1.0); self.node(rf).tag = tag;
            self.make_bouncy(rf, 1.0, Cond::Always, Action::Pop("OnRefresh", 0));
        }
        let info = self.image("GJ_GameSheet03", "GJ_infoIcon_001.png", 60.0, sh - 60.0, 204.0, 1.0); self.node(info).tag = tag;
        self.make_bouncy(info, 1.0, Cond::Always, Action::None);
        // the back arrow (not mirrored here)
        let back = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", 45.0, 45.0, 204.0, 1.0); self.node(back).tag = tag;
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("OnListBack", 0));
    }

    /// `_buildLevelCell`
    fn on_cell(&mut self, content: usize, idx: usize, global: usize, l: &OnLevel, rx: f64, pw: f64) {
        let tag = "on_l";
        let row_h = 180.0;
        let row_y = idx as f64 * row_h; let cy = row_y + row_h / 2.0;
        let d = 202.01; // inside the scroll area's clip: (202.0, 202.04)
        let stripe = self.shape(Kind::Rect, rx, row_y, pw, row_h, if idx % 2 == 0 { 0xB5652E } else { 0xC2723E }, 1.0, 202.005, tag);
        self.node(stripe).parent = Some(content);
        if idx > 0 { let dv = self.shape(Kind::Rect, rx + 5.0, row_y - 0.75, pw - 10.0, 1.5, 0, 0.6, 202.006, tag); self.node(dv).parent = Some(content); }
        let saved = self.on.kind == 2;
        let (dw, dh) = match (saved, l.difficulty) { (true, 6) | (true, 9) => (diff_size(l.difficulty).0, 90.0), (true, 10) => (80.0, 95.0), _ => diff_size(l.difficulty) };
        let sc = (85.0 / dh).min(90.0 / dw);
        let extra = match l.difficulty { 6 | 9 => 1.04, 10 => 1.1, _ => 1.0 };
        if let Some(c) = coin_frame(l) { self.on_img(Some(content), "GJ_GameSheet03", c, rx + 52.0, cy, 1.0, d, tag); }
        self.on_img(Some(content), "GJ_GameSheet03", &diff_frame(l.difficulty, false), rx + 52.0, cy, sc * extra, d + 0.001, tag);
        let name_x = rx + 105.0; let btn_x = rx + pw - 88.0;
        let complete = l.best >= 100.0; let has_best = l.best > 0.0;
        let reserve = if complete { 44.0 } else if has_best { 70.0 } else { 0.0 };
        let max_text = (btn_x - 70.0 - 15.0) - name_x - reserve;
        let name = self.on_txt(Some(content), "bigFont", 45.0, if l.name.is_empty() { "Unknown" } else { &l.name }, name_x, cy - 60.0, 0.0, 0.5, d + 0.001, tag);
        self.on_fit(name, max_text, 0.3);
        let nw = self.on_disp_w(name);
        if complete { let ci = self.on_img(Some(content), "GJ_GameSheet03", "GJ_completesIcon_001.png", name_x + 5.0 + nw + 14.0, cy - 60.0, 0.75, d + 0.001, tag); self.node(ci).ox = 0.0; }
        else if has_best { self.on_txt(Some(content), "goldFont", 28.0, &format!("{}%", l.best.floor() as i64), name_x + 5.0 + nw + 14.0, cy - 60.0, 0.0, 0.5, d + 0.001, tag); }
        let author = self.on_txt(Some(content), "goldFont", 32.0, &format!("By {}", if l.author.is_empty() { "Unknown" } else { &l.author }), name_x, cy - 15.0, 0.0, 0.5, d + 0.001, tag);
        self.on_fit(author, max_text, 0.3);
        let official = OFFICIAL.contains(&l.song_name.as_str());
        let song = self.on_txt(Some(content), "bigFont", 28.0, &l.song_name, name_x, cy + 28.0, 0.0, 0.5, d + 0.001, tag);
        { let n = self.node(song); n.color = if official { 0x00d6ff } else { 0xff79ff }; }
        self.on_fit(song, max_text, 0.3);
        let stat_y = cy + 63.0; let mut stat_x = name_x;
        for (icon, val, scale) in [("GJ_timeIcon_001.png", length_name(l.length).to_string(), 0.6), ("GJ_downloadsIcon_001.png", stat_count(l.downloads), 0.6), ("GJ_sLikeIcon_001.png", stat_count(l.likes), 0.9)] {
            let ic = self.on_img(Some(content), "GJ_GameSheet03", icon, stat_x, stat_y, scale, d + 0.001, tag);
            self.node(ic).ox = 0.0;
            let iw = self.node(ic).width * scale;
            let t = self.on_txt(Some(content), "bigFont", 26.0, &val, stat_x + iw + 8.0, stat_y, 0.0, 0.5, d + 0.001, tag);
            self.on_fit(t, max_text / 2.0, 0.3);
            let tw = self.on_disp_w(t);
            stat_x += iw + 8.0 + tw + 25.0;
        }
        // "Get It" until the level was viewed, then "View"
        let (frame, label, fs) = if l.viewed || saved { ("GJ_button01", "View", 40.0) } else { ("GJ_button02", "Get It", 30.0) };
        let zone = self.shape(Kind::Rect, btn_x, cy, 140.0, 60.0, 0, 0.0, d + 0.008, tag);
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 140.0; n.height = 60.0; n.parent = Some(content); }
        let nine = self.shape(Kind::Nine, 0.0, 0.0, 140.0, 60.0, 0, 1.0, d + 0.008, tag);
        { let n = self.node(nine); n.frame = frame.into(); n.radius = 20.0; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
        self.on_txt(Some(zone), "bigFont", fs, label, -2.0, -4.0, 0.5, 0.5, d + 0.009, tag);
        self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("OnView", global));
    }

    // ---- the play menu (`_openPlayMenu`) ----
    /// the play button while the level downloads: tinted 0x666666 and dead, restored when the start failed (`playBtnLoading`)
    pub fn on_play_busy(&mut self, busy: bool) {
        for slot in self.nodes.iter_mut() { if let Some(n) = slot {
            if matches!(n.action, Action::Pop("OnPlay", _)) { n.tint = if busy { 0x666666 } else { -1 }; n.interactive = !busy; }
        } }
    }

    /// Coming back from a played online level: the play menu of that level opens again (`_onlineReturnToPlayMenu`), with its list behind it.
    /// Returns the list index, or -1 when no level was open.
    pub fn on_reopen(&mut self) -> i64 {
        let Some(i) = self.on.view else { return -1 };
        if i >= self.on.levels.len() { return -1; }
        self.kill_tag("on_");
        if self.on.kind != 3 { self.on_open_list(); }
        self.on_open_menu(i);
        i as i64
    }

    pub fn on_open_menu(&mut self, idx: usize) {
        let Some(l) = self.on.levels.get(idx).cloned() else { return };
        self.kill_tag("on_m");
        self.on.view = Some(idx);
        let (sw, sh) = (self.screen_width, 640.0);
        let cx = sw / 2.0; let tag = "on_m";
        self.em_fade_in(600.0, 300.0, tag);
        self.em_gradient(500.0, tag);
        self.em_blocker(501.0, tag);
        self.on_corners(502.0, tag);
        self.on_back_arrow(Action::Pop("OnMenuBack", 0), 504.0, tag);
        let name = self.on_txt(None, "bigFont", 50.0, if l.name.is_empty() { "Unknown" } else { &l.name }, cx, sh * 0.11 - 30.0, 0.5, 0.5, 503.0, tag);
        self.on_fit(name, sw * 0.7, 0.3);
        let ny = self.node(name).y;
        let author = self.on_txt(None, "goldFont", 36.0, &format!("By {}", if l.author.is_empty() { "Unknown" } else { &l.author }), cx, ny + 42.0, 0.5, 0.5, 503.0, tag);
        self.on_fit(author, sw * 0.6, 0.3);
        let play_y = sh * 0.36 - 15.0;
        let play = self.image("GJ_GameSheet03", "GJ_playBtn2_001.png", cx, play_y, 504.0, 1.0); self.node(play).tag = tag;
        let play_w = self.node(play).width;
        self.make_bouncy(play, 1.0, Cond::Always, Action::Pop("OnPlay", idx));
        // downloads, likes and length on the right
        let sx = cx + play_w / 2.0 + 100.0; let gap = 57.0; let y0 = play_y - gap;
        let max_w = (sw - 30.0) - (sx + 60.0);
        for (i, (icon, val, sc)) in [("GJ_downloadsIcon_001.png", grouped(l.downloads), 0.9), ("GJ_sLikeIcon_001.png", grouped(l.likes), 1.3), ("GJ_timeIcon_001.png", length_name(l.length).to_string(), 0.9)].into_iter().enumerate() {
            let y = y0 + i as f64 * gap;
            let ic = self.on_img(None, "GJ_GameSheet03", icon, sx, y, sc, 503.0, tag);
            let iw = self.node(ic).width * sc;
            let t = self.on_txt(None, "bigFont", 30.0, &val, sx + iw / 2.0 + 12.0, y, 0.0, 0.5, 503.0, tag);
            self.on_fit(t, max_w, 0.3);
        }
        // the face and its coin on the left
        let dx = cx - play_w / 2.0 - 100.0;
        if let Some(c) = coin_frame(&l) { self.on_img(None, "GJ_GameSheet03", c, dx, play_y, 1.0, 502.0, tag); }
        self.on_img(None, "GJ_GameSheet03", &diff_frame(l.difficulty, true), dx, play_y, 1.0, 503.0, tag);
        // the two progress bars
        let bar_w = self.sizes.get("GJ_WebSheet/GJ_progressBar_001.png").map_or(680.0, |s| s.0);
        for (label, best, y, col, sy) in [("Normal Mode", l.best, sh * 0.58 - 15.0, 0x00ff00i64, 0.69), ("Practice Mode", l.best_practice, sh * 0.70 - 25.0, 0x00ffff, 0.7)] {
            let lb = self.on_txt(None, "bigFont", 32.0, label, cx, y - 30.0, 0.5, 0.5, 503.0, tag); { let n = self.node(lb); n.sx = 0.78; n.sy = 0.78; }
            let bg = self.on_img(None, "GJ_WebSheet", "GJ_progressBar_001.png", cx, y, 1.0, 503.0, tag); { let n = self.node(bg); n.tint = 0; n.alpha = 125.0 / 255.0; n.sx = 0.7; n.sy = sy; }
            let fg = self.on_img(None, "GJ_WebSheet", "GJ_progressBar_001.png", cx - bar_w * 0.695 / 2.0, y, 1.0, 503.2, tag);
            { let n = self.node(fg); n.sx = 0.695; n.sy = 0.57; n.ox = 0.0; n.tint = col; n.crop_w = (bar_w * best / 100.0).floor().max(1.0); }
            let p = self.on_txt(None, "bigFont", 30.0, &format!("{}%", best), cx, y, 0.5, 0.5, 504.0, tag); { let n = self.node(p); n.sx = 0.7; n.sy = 0.7; }
        }
        // side buttons: copy to the editor, close, refresh, and three locked ones
        let (side_x, side_gap, side_y0) = (sw - 55.0, sh * 0.16, sh * 0.08 - 100.0);
        let dup = self.image("GJ_GameSheet03", "GJ_duplicateBtn_001.png", 55.0, side_y0 + 350.0, 504.0, 1.0); self.node(dup).tag = tag;
        self.make_bouncy(dup, 1.0, Cond::Always, Action::Pop("OnCopy", idx));
        let close = self.image("GJ_GameSheet03", "GJ_deleteBtn_001.png", side_x, side_y0 + side_gap, 504.0, 1.0); self.node(close).tag = tag;
        self.make_bouncy(close, 1.0, Cond::Always, Action::Pop("OnDeleteLevel", idx));
        let rf = self.image("GJ_GameSheet03", "GJ_updateBtn_001.png", side_x, side_y0 + side_gap * 2.0, 504.0, 1.0); self.node(rf).tag = tag;
        self.make_bouncy(rf, 1.0, Cond::Always, Action::Pop("OnMenuRefresh", 0));
        for (k, frame) in [(3.0, "GJ_infoBtn_001.png"), (4.0, "GJ_levelLeaderboardBtn_001.png"), (5.0, "GJ_like2Btn_001.png")] {
            let b = self.image("GJ_GameSheet03", frame, side_x, side_y0 + side_gap * k, 504.0, 1.0); self.node(b).tag = tag; self.lock_gray(b);
            self.make_bouncy(b, 1.0, Cond::Always, Action::None);
        }
        let ci = self.image("GJ_GameSheet03", "GJ_infoIcon_001.png", 60.0, sh - 60.0, 504.0, 1.0); self.node(ci).tag = tag;
        self.make_bouncy(ci, 1.0, Cond::Always, Action::None);
        // the song box
        let (box_w, box_h, box_y) = (sw * 0.55, 190.0, sh - 95.0);
        let tex = self.sizes.get("/GJ_square01").map_or(80.0, |s| s.0);
        let root = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 503.0, tag);
        let _ = tex;
        self.pa_nine(root, "GJ_square01", cx, box_y, box_w, box_h, 503.0, tag);
        let song = if l.song_name.is_empty() { "Unknown" } else { &l.song_name };
        let sn = self.on_txt(None, "bigFont", 50.0, song, cx - box_w / 2.0 + 20.0, box_y - 60.0, 0.0, 0.5, 504.0, tag);
        self.on.song_name_node = sn;
        self.on_fit(sn, box_w - 100.0, 0.3);
        let sa = self.on_txt(None, "goldFont", 36.0, "By: Unknown", cx - box_w / 2.0 + 20.0, box_y - 15.0, 0.0, 0.5, 504.0, tag);
        self.on.song_author_node = sa;
        self.on_fit(sa, box_w - 100.0, 0.3);
        if !l.custom_song.is_empty() && l.custom_song != "0" {
            let dl = self.image("GJ_GameSheet03", "GJ_downloadBtn_001.png", cx + box_w / 2.0 - 50.0, box_y + 40.0, 504.0, 1.0); self.node(dl).tag = tag;
            self.make_bouncy(dl, 1.0, Cond::Always, Action::Pop("OnSongDownload", idx));
        }
    }

    /// the demon filter popup (`_buildDemonFilterPopup`)
    fn on_open_demon(&mut self) {
        self.kill_tag("on_p");
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let tag = "on_p";
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 100.0 / 255.0, 1000.0, tag);
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        let root = self.shape(Kind::Rect, cx, 320.0, 0.0, 0.0, 0, 0.0, 1000.0, tag);
        { let n = self.node(root); n.sx = 0.0; n.sy = 0.0; }
        self.tween_to(root, &[(Prop::ScaleX, 1.0), (Prop::ScaleY, 1.0)], 660.0, Ease::ElasticOut, Then::Nothing);
        let (pw, ph) = (760.0, 360.0);
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, pw, ph, 1001.0, tag);
        self.on_txt(Some(root), "bigFont", 64.0, "Demon Filter", 0.0, -ph / 2.0 + 40.0, 0.5, 0.5, 1002.0, tag);
        const FR: [&str; 6] = ["difficulty_06_btn_001.png", "difficulty_07_btn2_001.png", "difficulty_08_btn2_001.png", "difficulty_06_btn2_001.png", "difficulty_09_btn2_001.png", "difficulty_10_btn2_001.png"];
        let slot = (pw - 60.0) / 6.0;
        self.on.demon_sel = self.on.demon_tier; self.on.demon_icons.clear();
        for (i, f) in FR.iter().enumerate() {
            let ic = self.on_img(Some(root), "GJ_GameSheet03", f, -pw / 2.0 + 30.0 + slot * i as f64 + slot / 2.0, 0.0, 1.1, 1002.0, tag);
            self.node(ic).tint = if i == self.on.demon_sel { 0xffffff } else { 0x8f8f8f };
            self.make_bouncy(ic, 1.1, Cond::Always, Action::Pop("OnDemonTier", i));
            self.on.demon_icons.push(ic);
        }
        self.pa_btn(root, "GJ_button01", 0.0, ph / 2.0 - 45.0, 110.0, 55.0, "OK", "goldFont", 44.0, -3.0, -4.0, Action::Pop("OnDemonOk", 0), 1002.0, tag);
    }

    /// the demon tier to search for (0 none): only while the demon filter is on
    pub fn on_demon(&self) -> u32 { if self.on.filters[6] { self.on.demon_tier as u32 } else { 0 } }

    // ---- daily / weekly / event (`_buildSpecialLevelPopup`) ----
    pub fn on_open_special(&mut self) {
        self.kill_tag("on_d");
        let Some(kind) = self.on.special else { return };
        let (sw, cx, cy) = (self.screen_width, self.screen_width / 2.0, 320.0);
        let tag = "on_d";
        // per kind: panel texture and tint, cell texture and tint, corner frame (and scale / cut), label, crown
        struct St { panel: &'static str, panel_tint: i64, cell: &'static str, cell_tint: i64, corner: &'static str, corner_scale: f64, cut: f64, ins: f64, line_len: f64,
                    offs: [(f64, f64); 3], line_offs: [(f64, f64); 2], label: &'static str, label_text: &'static str, crown: &'static str }
        let st = match kind {
            0 => St { panel: "GJ_square01", panel_tint: -1, cell: "GJ_square02", cell_tint: -1, corner: "dailyLevelCorner_001.png", corner_scale: 1.0, cut: 0.0, ins: 0.0, line_len: -1.0, offs: [(0.0, 0.0); 3], line_offs: [(0.0, 0.0); 2], label: "dailyLevelLabel_001.png", label_text: "Daily Level", crown: "gj_dailyCrown_001.png" },
            1 => St { panel: "GJ_square05", panel_tint: -1, cell: "GJ_square05", cell_tint: 0xafafaf, corner: "gauntletCorner_001.png", corner_scale: 0.6, cut: 15.0, ins: 8.0, line_len: 16.0, offs: [(-1.0, -1.0), (-1.0, -1.0), (1.0, 1.0)], line_offs: [(3.5, -3.5), (-3.5, -2.0)], label: "weeklyLevelLabel_001.png", label_text: "", crown: "gj_weeklyCrown_001.png" },
            _ => St { panel: "GJ_square05", panel_tint: 0xcf16fb, cell: "GJ_square04", cell_tint: -1, corner: "dailyLevelCorner_001.png", corner_scale: 1.0, cut: 0.0, ins: 0.0, line_len: -1.0, offs: [(0.0, 0.0); 3], line_offs: [(0.0, 0.0); 2], label: "eventLevelLabel_001.png", label_text: "Event Level", crown: "gj_eventCrown_001.png" },
        };
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 100.0 / 255.0, 1000.0, tag);
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        // everything else slides in from the left
        let root = self.shape(Kind::Rect, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 1000.0, tag);
        { let n = self.node(root); n.x = -sw; }
        self.tween_to(root, &[(Prop::X, 0.0)], 600.0, Ease::ElasticOut, Then::Nothing);
        let (pw, ph) = (800.0, 550.0);
        let tex = |ui: &Ui, k: &str| ui.sizes.get(&format!("/{}", k)).map_or(80.0, |s| s.0);
        let nine = |ui: &mut Ui, frame: &'static str, x: f64, y: f64, w: f64, h: f64, tint: i64, depth: f64| {
            let r = tex(ui, if frame == "GJ_square01" { "GJ_square01" } else { "GJ_square02" }) * 0.325;
            let n = ui.shape(Kind::Nine, x, y, w, h, 0, 1.0, depth, tag);
            { let m = ui.node(n); m.frame = frame.into(); m.radius = r; m.ox = 0.5; m.oy = 0.5; m.tint = if tint < 0 { 0xffffff } else { tint }; m.parent = Some(root); }
            n
        };
        nine(self, st.panel, cx, cy, pw, ph, st.panel_tint, 1001.0);
        let (nl, nr, nt, nb) = (cx - pw / 2.0, cx + pw / 2.0, cy - ph / 2.0, cy + ph / 2.0);
        // the three corner pieces (top left mirrored upside down, bottom left, bottom right mirrored sideways)
        for (k, (ox, oy, x, y, fx, fy)) in [(0.0, 0.0, nl + st.ins + st.offs[0].0, nt + st.ins + st.offs[0].1, false, true), (0.0, 1.0, nl + st.ins + st.offs[1].0, nb - st.ins + st.offs[1].1, false, false), (1.0, 1.0, nr - st.ins + st.offs[2].0, nb - st.ins + st.offs[2].1, true, false)].into_iter().enumerate() {
            let c = self.on_img(Some(root), "GJ_GameSheet03", st.corner, x, y, st.corner_scale, 1001.5, tag);
            { let n = self.node(c); n.ox = ox; n.oy = oy; n.flip_x = fx; n.flip_y = fy; } let _ = k;
        }
        if st.cut > 0.0 { // the two black diagonal lines at the bottom corners
            let sq = self.sizes.get("GJ_WebSheet/square.png").map_or(16.0, |s| s.0);
            for (k, (x1, y1, x2, y2)) in [(nr, nb - st.cut, nr - st.cut, nb), (nl, nb - st.cut, nl + st.cut, nb)].into_iter().enumerate() {
                let (dx, dy) = st.line_offs[k];
                let full = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
                let len = if st.line_len >= 0.0 { st.line_len } else { full };
                let (mx, my) = ((x1 + x2) / 2.0 + dx, (y1 + y2) / 2.0 + dy);
                let ln = self.on_img(Some(root), "GJ_WebSheet", "square.png", mx, my, 1.0, 1001.6, tag);
                { let n = self.node(ln); n.tint = 0; n.sx = len / sq; n.sy = 4.0 / sq; n.rot = (y2 - y1).atan2(x2 - x1); }
            }
        }
        // the label, the greyed-out safe button and the close button
        if self.sizes.contains_key(&format!("GJ_GameSheet03/{}", st.label)) { self.on_img(Some(root), "GJ_GameSheet03", st.label, cx, nt + 60.0, 1.0, 1002.0, tag); }
        else { self.on_txt(Some(root), "goldFont", 52.0, st.label_text, cx, nt + 60.0, 0.5, 0.5, 1002.0, tag); }
        let safe = self.on_img(Some(root), "GJ_GameSheet04", "GJ_safeBtn_001.png", nr - 20.0, nt + 15.0, 1.0, 1002.0, tag);
        { let n = self.node(safe); n.ox = 1.0; n.oy = 0.0; } self.lock_gray(safe);
        let close = self.on_img(Some(root), "GJ_WebSheet", "GJ_closeBtn_001.png", nl + 20.0, nt + 20.0, 1.0, 1002.0, tag);
        self.node(close).hit_scale = 2.0;
        self.make_bouncy(close, 1.0, Cond::Always, Action::Pop("OnSpecialClose", 0));
        self.on.sp_root = root;
        self.on_special_cell();
    }

    /// the cell of the daily / weekly / event popup: the spinner while the level loads, then the level. It is rebuilt in place when the
    /// level arrives, so the popup does not slide in a second time (`renderCell` of `_buildSpecialLevelPopup`).
    pub fn on_special_cell(&mut self) {
        self.kill_tag("on_dc");
        let Some(kind) = self.on.special else { return };
        let root = self.on.sp_root;
        if self.get(root).is_none() { return; }
        let (cx, cy) = (self.screen_width / 2.0, 320.0);
        let tag = "on_dc";
        let (cell, cell_tint, crown): (&'static str, i64, &'static str) = match kind { 0 => ("GJ_square02", -1, "gj_dailyCrown_001.png"), 1 => ("GJ_square05", 0xafafaf, "gj_weeklyCrown_001.png"), _ => ("GJ_square04", -1, "gj_eventCrown_001.png") };
        let tex = |ui: &Ui, k: &str| ui.sizes.get(&format!("/{}", k)).map_or(80.0, |s| s.0);
        let nine = |ui: &mut Ui, frame: &'static str, x: f64, y: f64, w: f64, h: f64, tint: i64, depth: f64| {
            let r = tex(ui, if frame == "GJ_square01" { "GJ_square01" } else { "GJ_square02" }) * 0.325;
            let n = ui.shape(Kind::Nine, x, y, w, h, 0, 1.0, depth, tag);
            { let m = ui.node(n); m.frame = frame.into(); m.radius = r; m.ox = 0.5; m.oy = 0.5; m.tint = if tint < 0 { 0xffffff } else { tint }; m.parent = Some(root); }
            n
        };
        let (cell_w, cell_h) = (720.0, 220.0);
        let (ccx, ccy) = (cx, cy + 25.0);
        let (cl, ct) = (ccx - cell_w / 2.0, ccy - cell_h / 2.0);
        match self.on.special_level.clone() {
            None => {
                if self.on.loading { let sp = self.image("", "loadingCircle", ccx, ccy, 1003.0, 1.0); { let n = self.node(sp); n.tag = tag; n.parent = Some(root); n.additive = true; n.alpha = 0.5; } self.on.spin = sp; }
                else { let m = if self.on.error.is_empty() { "Nothing here".to_string() } else { self.on.error.clone() }; self.on_txt(Some(root), "bigFont", 36.0, &m, ccx, ccy, 0.5, 0.5, 1003.0, tag); }
            }
            Some(l) => {
                nine(self, cell, ccx, ccy, cell_w, cell_h, cell_tint, 1003.0);
                let (dw, dh) = diff_size(l.difficulty);
                let sc = (85.0 / dh).min(90.0 / dw);
                if let Some(c) = coin_frame(&l) { self.on_img(Some(root), "GJ_GameSheet03", c, cl + 75.0, ccy, 1.0, 1003.5, tag); }
                self.on_img(Some(root), "GJ_GameSheet03", &diff_frame(l.difficulty, false), cl + 75.0, ccy, sc, 1003.6, tag);
                let name_x = cl + 160.0; let max_w = cell_w - 160.0 - 170.0; let gap = 5.0;
                let d = 1004.0;
                let name = self.on_txt(Some(root), "bigFont", 45.0, &l.name, name_x, 0.0, 0.0, 0.0, d, tag); self.on_fit(name, max_w, 0.3);
                let author = self.on_txt(Some(root), "goldFont", 32.0, &format!("By {}", l.author), name_x, 0.0, 0.0, 0.0, d, tag); self.on_fit(author, max_w, 0.3);
                let official = OFFICIAL.contains(&l.song_name.as_str());
                let song = self.on_txt(Some(root), "bigFont", 28.0, &l.song_name, name_x, 0.0, 0.0, 0.0, d, tag);
                { let n = self.node(song); n.color = if official { 0x00d6ff } else { 0xff79ff }; } self.on_fit(song, max_w, 0.3);
                let dh_of = |ui: &Ui, i: usize| ui.get(i).map_or(0.0, |n| n.height * n.sy);
                let total = dh_of(self, name) + dh_of(self, author) + dh_of(self, song) + 30.0 + gap * 3.0;
                let mut ty = ccy - total / 2.0;
                self.node(name).y = ty; ty += dh_of(self, name) + gap;
                self.node(author).y = ty; ty += dh_of(self, author) + gap;
                self.node(song).y = ty; ty += dh_of(self, song) + gap;
                let stat_y = ty + 15.0;
                let mut sx = name_x;
                for (icon, val, scale) in [("GJ_timeIcon_001.png", length_name(l.length).to_string(), 0.6), ("GJ_downloadsIcon_001.png", stat_count(l.downloads), 0.6), ("GJ_sLikeIcon_001.png", stat_count(l.likes), 0.9)] {
                    let ic = self.on_img(Some(root), "GJ_GameSheet03", icon, sx, stat_y, scale, d, tag);
                    self.node(ic).ox = 0.0;
                    let iw = self.node(ic).width * scale;
                    let t = self.on_txt(Some(root), "bigFont", 26.0, &val, sx + iw + 8.0, stat_y, 0.0, 0.5, d, tag);
                    self.on_fit(t, max_w / 2.0, 0.3);
                    sx += iw + 8.0 + self.on_disp_w(t) + 25.0;
                }
                if l.daily_number > 0 {
                    let t = self.on_txt(Some(root), "bigFont", 24.0, &format!("#{}", l.daily_number), cl + cell_w - 18.0, ct + cell_h - 14.0, 1.0, 1.0, d, tag);
                    { let n = self.node(t); n.color = 0x8a8fb8; n.alpha = 0.6; }
                }
                let play = self.on_img(Some(root), "GJ_GameSheet03", "GJ_playBtn2_001.png", cl + cell_w - 95.0, ccy, 0.75, d, tag);
                self.make_bouncy(play, 0.75, Cond::Always, Action::Pop("OnSpecialPlay", 0));
                let crown = self.on_img(Some(root), "GJ_GameSheet04", crown, ccx, ct + 12.0, 1.0, d, tag);
                self.node(crown).oy = 0.8;
            }
        }
    }
}
