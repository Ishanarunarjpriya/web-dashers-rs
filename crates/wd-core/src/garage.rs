//! The icon kit ("Character Select"): game-scene.js `_openIconSelector` / `_closeIconSelector`.
//! Tabs for every game mode, a 12x3 icon grid with paging, two colour rows, the player name box and the custom cube import popup.
//! The host loads the per-icon atlases (`GarageNeed` event) and stores the choices (`GarageSave` event).
use crate::ui::*;
use std::collections::HashSet;

pub const GD_COLORS: [i64; 107] = [
    0x7DFF00, 0x00FF00, 0x00FF7D, 0x00FFFF, 0x007DFF, 0x0000FF, 0x7D00FF, 0xFF00FF, 0xFF007D, 0xFF0000, 0xFF7D00, 0xFFFF00,
    0xFFFFFF, 0xB900FF, 0xFFB900, 0x000000, 0x00C8FF, 0xAFAFAF, 0x5A5A5A, 0xFF7D7D, 0x00AF4B, 0x007D7D, 0x004BAF, 0x4B00AF,
    0x7D007D, 0xAF004C, 0xAF4B00, 0x7D7D00, 0x4AAF00, 0xFF4B00, 0x963200, 0x966400, 0x649600, 0x009664, 0x006496, 0x640096,
    0x960064, 0x960000, 0x009600, 0x000096, 0x7DFFAF, 0x7D7DFF, 0xFFFA7F, 0xFA7FFF, 0x00FFC0, 0x50320E, 0xCDA576, 0xB680FF,
    0xFF3A3B, 0x4D4D8F, 0x000A4C, 0xFCD5CE, 0xBEB5FF, 0x700000, 0x530000, 0x390006, 0x804F4F, 0x7A3535, 0x512424, 0xA36246,
    0x754936, 0x563528, 0xFFB972, 0xFEA040, 0x66311E, 0x5B2700, 0x472000, 0xA77B4D, 0x6D5339, 0x513E2A, 0xFFFFC0, 0xFCE8A8,
    0xC0FFA0, 0xB1FF6D, 0xC0FFE0, 0x95FFE4, 0x43A18A, 0x316D5F, 0x265449, 0x006000, 0x004000, 0x006060, 0x004040, 0xA0FFFF,
    0x000770, 0x00496D, 0x00324C, 0x002638, 0x5080AD, 0x335375, 0x243C56, 0xE0E0E0, 0x3D068C, 0x370860, 0x404040, 0x6E49A4,
    0x54367F, 0x432A63, 0xFCB5FF, 0xAF57AE, 0x824382, 0x5E315E, 0x808080, 0x66003E, 0x470034, 0xD2FF31, 0x76BDFF,
];
/// (tab key, frame prefix, icon count, saved key, tab button frame stem, preview size, preview y)
const TABS: [(&str, &str, usize, &str, &str, f64, f64); 7] = [
    ("icon", "player", 485, "iconCurrentPlayer", "icon", 88.0, 0.0),
    ("ship", "ship", 169, "iconCurrentShip", "ship", 106.0, 0.0),
    ("ball", "player_ball", 119, "iconCurrentBall", "ball", 88.0, 0.0),
    ("ufo", "bird", 149, "iconCurrentBird", "bird", 106.0, 21.0),
    ("wave", "dart", 96, "iconCurrentWave", "dart", 70.0, 0.0),
    ("robot", "robot", 68, "iconCurrentRobot", "robot", 88.0, 0.0),
    ("spider", "spider", 69, "iconCurrentSpider", "spider", 100.0, 0.0),
];
const PREFIXES: [&str; 7] = ["player_ball", "player", "ship", "dart", "bird", "spider", "robot"];
const PLAYER_PRIORITY: [&str; 11] = ["GJ_GameSheet03", "GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet04", "GJ_GameSheetEditor", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_WebSheet", "GJ_LaunchSheet", "player_ball_00", "player_dart_00"];
const ATLAS_LIST: [&str; 15] = ["GJ_WebSheet", "GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet03", "GJ_GameSheet04", "GJ_GameSheetEditor", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_LaunchSheet", "player_ball_00", "player_dart_00", "GJ_ParticleSheet", "PixelSheet_01", "FireSheet_01-hd", "Wavesheet"];
const NAME_CHARS: &str = " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

/// One sprite of a robot / spider idle pose
#[derive(Clone, Debug, Default)]
pub struct Part { pub tag: i64, pub texture: String, pub pos: (f64, f64), pub scale: (f64, f64), pub flip: (bool, bool), pub z: f64, pub rot: f64 }

#[derive(Default)]
pub struct Gar {
    pub open: bool,
    pub closing: Option<f64>,
    pub tab: usize,
    pub page: usize,
    pub color_page: usize,
    /// current icon per tab (`player_01`, ...)
    pub cur: [String; 7],
    pub main: i64,
    pub second: i64,
    pub name: String,
    pub focused: bool,
    pub ready: HashSet<String>,
    pub failed: HashSet<String>,
    pub need: Vec<String>,
    pub changes: Vec<String>,
    pub anim: [Vec<Part>; 2],
    pub grid_colored: bool,
    pub custom: bool,
    // nodes
    title: usize,
    tab_btns: Vec<usize>,
    sel_label: usize,
    c1: usize,
    c2: usize,
    plus: usize,
    import_open: bool,
}

/// `iconBaseOf`: `player_ball_01_2_001.png` -> `player_ball_01`
pub fn icon_base_of(frame: &str) -> Option<String> {
    for p in PREFIXES {
        let Some(rest) = frame.strip_prefix(p).and_then(|r| r.strip_prefix('_')) else { continue };
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() < 2 || digits.len() > 3 { continue; }
        let tail = &rest[digits.len()..];
        if tail == "_001.png" || (tail.starts_with('_') && tail.ends_with("_001.png")) { return Some(format!("{}_{}", p, digits)); }
    }
    None
}

impl Ui {
    fn gar_frames(tab: usize) -> Vec<String> { let (_, prefix, n, ..) = TABS[tab]; (1..=n).map(|i| format!("{}_{:02}_001.png", prefix, i)).collect() }

    /// `getAtlasFrame`: (atlas, frame) when the frame can be shown
    pub fn gar_atlas(&self, frame: &str) -> Option<(String, String)> {
        if let Some(base) = icon_base_of(frame) {
            if self.gar.ready.contains(&base) {
                let key = format!("icon_{}", base);
                return if self.sizes.contains_key(&format!("{}/{}", key, frame)) { Some((key, frame.to_string())) } else { None };
            }
        }
        let list: &[&str] = if frame.starts_with("player_") { &PLAYER_PRIORITY } else { &ATLAS_LIST };
        for a in list { if self.sizes.contains_key(&format!("{}/{}", a, frame)) { return Some((a.to_string(), frame.to_string())); } }
        None
    }

    fn gar_size(&self, atlas: &str, frame: &str) -> (f64, f64) { self.sizes.get(&format!("{}/{}", atlas, frame)).copied().unwrap_or((0.0, 0.0)) }

    fn gar_img(&mut self, frame: &str, x: f64, y: f64, depth: f64, tag: &'static str) -> Option<usize> {
        let (a, f) = self.gar_atlas(frame)?;
        let i = self.image(&a, &f, x, y, depth, 1.0);
        self.node(i).tag = tag;
        Some(i)
    }

    pub fn gar_open(&mut self) {
        if self.gar.open { return; }
        self.gar.open = true; self.gar.closing = None; self.gar.focused = false; self.gar.page = 0;
        if self.gar.name.is_empty() { self.gar.name = "Player".into(); }
        let (sw, sh) = (self.screen_width, self.screen_height);
        self.gar.tab = 0;
        // fade in from black
        let fade = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 1.0, 200.0, "gar_fade");
        self.tween_to(fade, &[(Prop::Alpha, 0.0)], 300.0, Ease::Linear, Then::Destroy);
        // grey gradient backdrop
        for gi in 0..80 {
            let t = gi as f64 / 79.0;
            let c = (0x92 as f64 + (0x3a as f64 - 0x92 as f64) * t).round() as i64;
            let y = (gi as f64 * sh / 80.0).floor();
            let h = (sh / 80.0).ceil() + 1.0;
            self.shape(Kind::Rect, 0.0, y, sw, h, (c << 16) | (c << 8) | c, 1.0, 100.0, "gar_bg");
        }
        let blocker = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 100.0, "gar_bg");
        { let n = self.node(blocker); n.interactive = true; n.width = sw; n.height = sh; n.action = Action::Pop("GarageBlur", 0); }
        // player name
        let title = self.text_node("bigFont", 64.0, &self.gar.name.clone(), sw / 2.0, 70.0, 105.0, 0.5, 0.5, 1.0);
        { let n = self.node(title); n.tag = "gar_ui"; n.interactive = true; n.action = Action::Pop("GarageTitle", 0); }
        self.gar.title = title;
        let back = self.image("GJ_GameSheet03", "GJ_arrow_03_001.png", 50.0, 48.0, 104.0, 1.0);
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("GarageClose", 0)); self.node(back).tag = "gar_ui";
        let shards = self.image("GJ_GameSheet03", "GJ_shardsBtn_001.png", 60.0, 165.0, 104.0, 1.0);
        self.node(shards).tag = "gar_ui"; self.lock_gray(shards);
        // fading line under the preview
        let (line_y, pad) = (210.0, 230.0);
        let (ls, le) = (pad, sw - pad);
        let lw = le - ls;
        let fade_zone = lw * 0.25;
        for li in 0..40 {
            let (t0, t1) = (li as f64 / 40.0, (li + 1) as f64 / 40.0);
            let (x0, x1) = (ls + t0 * lw, ls + t1 * lw);
            let mid = (t0 + t1) / 2.0 * lw;
            let alpha = if mid < fade_zone { mid / fade_zone } else if mid > lw - fade_zone { (lw - mid) / fade_zone } else { 1.0 };
            self.shape(Kind::Rect, x0, line_y - 1.5, x1 - x0, 3.0, 0xffffff, alpha, 102.0, "gar_ui");
        }
        // icon grid frame
        let (cw, ch) = (self.gar_container().2, self.gar_container().3);
        let (cx, cy) = (self.gar_container().0, self.gar_container().1);
        let grid_bg = self.shape(Kind::RoundRect, cx, cy, cw, ch, 0x454444, 1.0, 102.0, "gar_ui");
        self.node(grid_bg).radius = 10.0;
        let tl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, 0.0, 100.0, 1.0);
        { let n = self.node(tl); n.ox = 0.0; n.oy = 0.0; n.flip_y = true; n.tag = "gar_ui"; }
        let tr = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, 0.0, 103.0, 1.0);
        { let n = self.node(tr); n.ox = 1.0; n.oy = 0.0; n.flip_y = true; n.flip_x = true; n.tag = "gar_ui"; }
        // colour rows
        let (btn, pad_c) = (44.0, 6.0);
        let row_w = 12.0 * (btn + pad_c) - pad_c;
        let row2 = sh - 39.0 - btn / 2.0;
        let row1 = row2 - btn - 10.0;
        let box_y = row1 - btn / 2.0 - 10.0;
        self.shape(Kind::Rect, 0.0, box_y, sw, sh - box_y, 0, 0.5, 101.0, "gar_ui");
        let sel_scale = 0.6 * btn / 35.0;
        for (slot, _) in [(0usize, 0), (1, 1)] {
            let l = self.image("GJ_GameSheet03", "GJ_select_001.png", 0.0, 0.0, 106.0, sel_scale);
            { let n = self.node(l); n.visible = false; n.tag = "gar_ui"; }
            if slot == 0 { self.gar.c1 = l; } else { self.gar.c2 = l; }
        }
        let arrow_y = (row1 + row2) / 2.0;
        let margin = row_w / 2.0 + 45.0;
        let pa = self.image("GJ_GameSheet03", "GJ_arrow_02_001.png", sw / 2.0 - margin, arrow_y, 106.0, 0.8);
        self.make_bouncy(pa, 0.8, Cond::Always, Action::Pop("GarageColorPage", 0)); self.node(pa).tag = "gar_ui";
        let na = self.image("GJ_GameSheet03", "GJ_arrow_02_001.png", sw / 2.0 + margin, arrow_y, 106.0, 0.8);
        self.make_bouncy(na, 0.8, Cond::Always, Action::Pop("GarageColorPage", 1)); { let n = self.node(na); n.flip_x = true; n.tag = "gar_ui"; }
        let mi = GD_COLORS.iter().position(|c| *c == self.gar.main);
        self.gar_build_colors(mi.map_or(0, |i| i / 12));
        // grid arrows
        let ay = cy + ch / 2.0;
        let pg = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", cx - 54.0, ay, 106.0, 0.8);
        self.make_bouncy(pg, 0.8, Cond::Always, Action::Pop("GaragePage", 0)); self.node(pg).tag = "gar_ui";
        let ng = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", cx + cw + 54.0, ay, 106.0, 0.8);
        self.make_bouncy(ng, 0.8, Cond::Always, Action::Pop("GaragePage", 1)); { let n = self.node(ng); n.flip_x = true; n.tag = "gar_ui"; }
        // tabs
        let ty = cy - 40.0;
        self.gar.tab_btns.clear();
        for i in 0..7usize {
            let off = (i as f64 - 3.0) * 66.0;
            let b = self.gar_img(&Self::gar_tab_frame(i, i == 0), sw / 2.0 + off, ty, 104.0, "gar_ui").unwrap_or_else(|| self.image("GJ_GameSheet03", "gj_iconBtn_off_001.png", sw / 2.0 + off, ty, 104.0, 1.0));
            self.node(b).sx = 0.85; self.node(b).sy = 0.85;
            self.make_bouncy(b, 0.85, Cond::Always, Action::Pop("GarageTab", i)); self.node(b).tag = "gar_ui";
            self.gar.tab_btns.push(b);
        }
        // selected-icon label of the grid
        let sl = self.image("GJ_GameSheet03", "GJ_select_001.png", 0.0, 0.0, 106.0, 1.0);
        { let n = self.node(sl); n.visible = false; n.tag = "gar_ui"; }
        self.gar.sel_label = sl;
        // custom cube import button (icon tab only)
        let plus = self.image("GJ_GameSheet03", "GJ_plusBtn_001.png", 60.0, 245.0, 104.0, 1.0);
        let k = { let (w, h) = (self.node(plus).width, self.node(plus).height); (97.0 * 0.7 / if w > 0.0 { w } else { 97.0 }).min(97.0 * 0.7 / if h > 0.0 { h } else { 97.0 }) };
        { let n = self.node(plus); n.sx = k; n.sy = k; n.tag = "gar_ui"; }
        self.make_bouncy(plus, k, Cond::Always, Action::Pop("GarageImport", 0));
        self.gar.plus = plus;
        self.gar_switch_tab(0);
    }

    fn gar_container(&self) -> (f64, f64, f64, f64) {
        let (cols, rows, size, pad, mx, my) = (12.0, 3.0, 45.0, 12.0, 30.0, 15.0);
        let w = cols * size + (cols - 1.0) * pad + mx * 2.0;
        let h = rows * size + (rows - 1.0) * pad + my * 2.0;
        (self.screen_width / 2.0 - w / 2.0, self.screen_height - h - 10.0 - 150.0, w, h)
    }

    fn gar_tab_frame(tab: usize, on: bool) -> String { format!("gj_{}Btn_{}_001.png", TABS[tab].4, if on { "on" } else { "off" }) }

    fn gar_build_colors(&mut self, page: usize) {
        let pages = (GD_COLORS.len() + 11) / 12;
        self.gar.color_page = page.min(pages - 1);
        self.kill_tag("gar_col");
        let (sw, sh) = (self.screen_width, self.screen_height);
        let (btn, pad) = (44.0, 6.0);
        let row_w = 12.0 * (btn + pad) - pad;
        let row2 = sh - 39.0 - btn / 2.0;
        let row1 = row2 - btn - 10.0;
        let start_x = sw / 2.0 - row_w / 2.0 + btn / 2.0;
        let first = self.gar.color_page * 12;
        for ci in 0..12 {
            let Some(&color) = GD_COLORS.get(first + ci) else { break };
            let cx = start_x + ci as f64 * (btn + pad);
            for (row, slot) in [(row1, 0usize), (row2, 1)] {
                let zone = self.shape(Kind::Rect, cx, row, btn, btn, 0, 0.0, 104.0, "gar_col");
                { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = btn; n.height = btn; }
                let r = self.shape(Kind::Rect, -btn / 2.0, -btn / 2.0, btn, btn, color, 1.0, 104.0, "gar_col");
                self.node(r).parent = Some(zone);
                self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("GarageColor", (slot << 24) | (first + ci)));
            }
        }
        for i in 0..pages {
            let x = sw / 2.0 - (pages as f64 - 1.0) * 29.0 / 2.0 + i as f64 * 29.0;
            self.shape(Kind::Circle, x, sh - 20.0, 7.0, 0.0, 0xffffff, if i == self.gar.color_page { 1.0 } else { 0.3 }, 153.0, "gar_col");
        }
        self.gar_move_color_labels();
    }

    fn gar_move_color_labels(&mut self) {
        let (sw, sh) = (self.screen_width, self.screen_height);
        let (btn, pad) = (44.0, 6.0);
        let row_w = 12.0 * (btn + pad) - pad;
        let row2 = sh - 39.0 - btn / 2.0;
        let row1 = row2 - btn - 10.0;
        let start_x = sw / 2.0 - row_w / 2.0 + btn / 2.0;
        for (label, color, row) in [(self.gar.c1, self.gar.main, row1), (self.gar.c2, self.gar.second, row2)] {
            let idx = GD_COLORS.iter().position(|c| *c == color);
            let col = idx.map(|i| i as i64 - (self.gar.color_page * 12) as i64);
            let Some(n) = self.nodes.get_mut(label).and_then(|n| n.as_mut()) else { continue };
            match col { Some(c) if (0..12).contains(&c) => { n.x = start_x + c as f64 * (btn + pad); n.y = row; n.visible = true; } _ => n.visible = false }
        }
    }

    fn gar_switch_tab(&mut self, tab: usize) {
        self.gar.tab = tab; self.gar.page = 0;
        for i in 0..7 {
            let Some(b) = self.gar.tab_btns.get(i).copied() else { continue };
            let frame = Self::gar_tab_frame(i, i == tab);
            let fr = if self.gar_atlas(&frame).is_some() { frame } else { Self::gar_tab_frame(0, i == tab) };
            if let Some(n) = self.nodes.get_mut(b).and_then(|n| n.as_mut()) { n.frame = fr; }
        }
        let on_icon = tab == 0;
        let plus = self.gar.plus;
        if let Some(n) = self.nodes.get_mut(plus).and_then(|n| n.as_mut()) { n.visible = on_icon; }
        self.gar_refresh_preview();
        self.gar_build_grid();
    }

    fn gar_current_frame(&self, tab: usize) -> String {
        let frames = Self::gar_frames(tab);
        let want = format!("{}_001.png", self.gar.cur[tab]);
        frames.iter().find(|f| **f == want).cloned().unwrap_or_else(|| frames[0].clone())
    }

    /// The bases of a page that still have to be loaded; asks the host for them
    fn gar_missing(&mut self, frames: &[String]) -> bool {
        let mut need: Vec<String> = Vec::new();
        for f in frames { if let Some(b) = icon_base_of(f) { if !self.gar.ready.contains(&b) && !self.gar.failed.contains(&b) && !need.contains(&b) { need.push(b); } } }
        if need.is_empty() { return false; }
        self.gar.need = need;
        self.events.push(Action::Pop("GarageNeed", 0));
        true
    }

    fn gar_segmented(tab: usize) -> Option<usize> { match TABS[tab].0 { "robot" => Some(0), "spider" => Some(1), _ => None } }

    /// A robot / spider icon built from its parts (`_createSegmentedIconComposite`); returns (container, scale)
    fn gar_composite(&mut self, tab: usize, frame: &str, parent: Option<usize>, x: f64, y: f64, max: f64, depth: f64, muted: bool, tag: &'static str) -> Option<(usize, f64)> {
        let kind = Self::gar_segmented(tab)?;
        let parts = self.gar.anim[kind].clone();
        if parts.is_empty() { return None; }
        let prefix = TABS[tab].1;
        let base_name = frame.trim_end_matches("_001.png").to_string();
        let container = self.shape(Kind::Rect, x, y, 0.0, 0.0, 0, 0.0, depth, tag);
        if let Some(p) = parent { self.node(container).parent = Some(p); }
        let mut made: Vec<(f64, usize, bool, f64, f64)> = Vec::new(); // (z, node, is_base, w, h)
        for part in &parts {
            let tag_i = part.tag;
            let leg_spider = tab == 6 && [0, 1, 4, 5].contains(&tag_i);
            let leg_robot = tab == 5 && [0, 2, 4, 6].contains(&tag_i);
            let arm_robot = tab == 5 && [1, 5].contains(&tag_i);
            let local_y = if tab == 6 { if tag_i == 3 { 5.0 } else if leg_spider { -9.0 } else { 0.0 } } else if tag_i == 3 { 5.0 } else if arm_robot { -1.0 } else if leg_robot { -9.0 } else { 0.0 };
            let local_x = if tab == 6 { if leg_spider { 1.8 } else { 1.0 } } else if leg_robot || arm_robot { 1.8 } else { 1.0 };
            // texture names are `<prefix>_NN_PP_001.png`; the icon number is replaced
            let src = match part.texture.strip_prefix(prefix) { Some(rest) => { let rest = rest.trim_start_matches('_'); let tail = rest.trim_start_matches(|c: char| c.is_ascii_digit()); format!("{}{}", base_name, tail) } None => part.texture.clone() };
            let z = if part.z == 0.0 && part.tag != 0 && false { part.tag as f64 } else { part.z };
            for (variant, tint, dz) in [("base", if muted && !self.gar.grid_colored { 0xAFAFAF } else { self.gar.main }, 0.0), ("overlay", if muted && !self.gar.grid_colored { 0xffffff } else { self.gar.second }, -0.04), ("extra", -1, 0.08)] {
                let name = match variant { "overlay" => src.replace("_001.png", "_2_001.png"), "extra" => src.replace("_001.png", "_extra_001.png"), _ => src.clone() };
                let Some((a, f)) = self.gar_atlas(&name) else { continue };
                let (w, h) = self.gar_size(&a, &f);
                let img = self.image(&a, &f, part.pos.0 * local_x, -(part.pos.1 + local_y), depth, 1.0);
                { let n = self.node(img); n.parent = Some(container); n.rot = part.rot * std::f64::consts::PI / 180.0; n.sx = part.scale.0 * if part.flip.0 { -1.0 } else { 1.0 }; n.sy = part.scale.1 * if part.flip.1 { -1.0 } else { 1.0 }; n.tint = tint; n.tag = tag; }
                made.push((z * 0.1 + dz, img, variant == "base", w * part.scale.0.abs(), h * part.scale.1.abs()));
            }
        }
        if made.is_empty() { return None; }
        made.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (rank, m) in made.iter().enumerate() { self.node(m.1).depth = depth + rank as f64 * 1e-4; }
        let has_base = made.iter().any(|m| m.2);
        let (mut minx, mut miny, mut maxx, mut maxy) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for m in made.iter().filter(|m| !has_base || m.2) {
            let n = self.get(m.1).unwrap();
            minx = minx.min(n.x - m.3 / 2.0); maxx = maxx.max(n.x + m.3 / 2.0); miny = miny.min(n.y - m.4 / 2.0); maxy = maxy.max(n.y + m.4 / 2.0);
        }
        let (width, height) = ((maxx - minx).max(1.0), (maxy - miny).max(1.0));
        let fit = if muted { max / height } else { (max / width).min(max / height) };
        { let n = self.node(container); n.sx = fit; n.sy = fit; }
        Some((container, fit))
    }

    fn gar_refresh_preview(&mut self) {
        self.kill_tag("gar_prev");
        let tab = self.gar.tab;
        let sw = self.screen_width;
        let frame = self.gar_current_frame(tab);
        if self.gar_missing(&[frame.clone()]) { return; }
        let (_, _, _, _, _, size, dy) = TABS[tab];
        let py = 165.0 + dy;
        if tab == 0 && self.gar.custom {
            let i = self.image("", "custom_cube_icon_hd", sw / 2.0, 165.0, 103.0, 1.0);
            let (w, h) = (self.node(i).width.max(1.0), self.node(i).height.max(1.0));
            let s = (80.0 / w).min(80.0 / h) * 0.85;
            { let n = self.node(i); n.sx = s; n.sy = s; n.tag = "gar_prev"; }
            return;
        }
        if Self::gar_segmented(tab).is_some() {
            self.gar_composite(tab, &frame, None, sw / 2.0, py, size, 103.0, false, "gar_prev");
            return;
        }
        let Some((a, f)) = self.gar_atlas(&frame) else { return };
        let (w, h) = self.gar_size(&a, &f);
        let s = (size / if w > 0.0 { w } else { size }).min(size / if h > 0.0 { h } else { size });
        let (main, second) = (self.gar.main, self.gar.second);
        let extra_name = frame.replace("_001.png", "_2_001.png");
        if let Some(ex) = self.gar_img(&extra_name, sw / 2.0, py, 102.0, "gar_prev") { let n = self.node(ex); n.sx = s; n.sy = s; n.tint = second; }
        let i = self.image(&a, &f, sw / 2.0, py, 103.0, s);
        { let n = self.node(i); n.tint = main; n.tag = "gar_prev"; }
        let white = frame.replace("_001.png", "_extra_001.png");
        if let Some(wi) = self.gar_img(&white, sw / 2.0, py, 103.5, "gar_prev") { let n = self.node(wi); n.sx = s; n.sy = s; }
    }

    fn gar_build_grid(&mut self) {
        let tab = self.gar.tab;
        let all = Self::gar_frames(tab);
        let per = 36;
        let pages = (all.len() + per - 1) / per;
        self.gar.page = self.gar.page.min(pages - 1);
        let slice: Vec<String> = all.iter().skip(self.gar.page * per).take(per).cloned().collect();
        if self.gar_missing(&slice) { return; }
        self.kill_tag("gar_grid");
        if let Some(n) = self.nodes.get_mut(self.gar.sel_label).and_then(|n| n.as_mut()) { n.visible = false; }
        let (cx, cy, _, _) = self.gar_container();
        let (start_x, start_y) = (cx + 30.0 + 22.5, cy + 15.0 + 22.5);
        let segmented = Self::gar_segmented(tab).is_some();
        let frames: Vec<String> = slice.iter().filter(|f| if segmented { icon_base_of(f).map_or(false, |b| self.gar.ready.contains(&b)) } else { self.gar_atlas(f).is_some() }).cloned().collect();
        let cur = self.gar.cur[tab].clone();
        for (idx, frame) in frames.iter().enumerate() {
            let (col, row) = (idx % 12, idx / 12);
            let (ix, iy) = (start_x + col as f64 * 57.0, start_y + row as f64 * 57.0);
            let zone = self.shape(Kind::Rect, ix, iy, 45.0, 45.0, 0, 0.0, 104.0, "gar_grid");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 45.0; n.height = 45.0; }
            let colored = self.gar.grid_colored;
            let mut selected_scale = 1.0;
            if segmented {
                match self.gar_composite(tab, frame, Some(zone), 0.0, 0.0, 45.0, 103.0, true, "gar_grid") { Some((_, s)) => selected_scale = s, None => {} }
            } else if let Some((a, f)) = self.gar_atlas(frame) {
                let (w, h) = self.gar_size(&a, &f);
                let s = (45.0 / if w > 0.0 { w } else { 45.0 }).min(45.0 / if h > 0.0 { h } else { 45.0 });
                selected_scale = s;
                let (main, second) = (self.gar.main, self.gar.second);
                let img = self.image(&a, &f, 0.0, 0.0, 103.0, s);
                { let n = self.node(img); n.parent = Some(zone); n.tint = if colored { main } else { 0xAFAFAF }; n.tag = "gar_grid"; }
                if let Some(ex) = self.gar_atlas(&frame.replace("_001.png", "_2_001.png")) {
                    let e = self.image(&ex.0, &ex.1, 0.0, 0.0, 102.0, s);
                    { let n = self.node(e); n.parent = Some(zone); n.tint = if colored { second } else { 0xffffff }; n.tag = "gar_grid"; }
                }
                if let Some(wh) = self.gar_atlas(&frame.replace("_001.png", "_extra_001.png")) {
                    let e = self.image(&wh.0, &wh.1, 0.0, 0.0, 103.5, s);
                    { let n = self.node(e); n.parent = Some(zone); n.tag = "gar_grid"; }
                }
            }
            let _ = selected_scale;
            let num: usize = frame.trim_end_matches("_001.png").rsplit('_').next().and_then(|d| d.parse().ok()).unwrap_or(0);
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("GarageIcon", num));
            if frame.trim_end_matches("_001.png") == cur && !(tab == 0 && self.gar.custom) {
                let sl = self.gar.sel_label;
                if let Some(n) = self.nodes.get_mut(sl).and_then(|n| n.as_mut()) { n.x = ix; n.y = iy; n.sx = 0.8; n.sy = 0.8; n.visible = true; }
            }
        }
    }

    pub fn gar_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "GarageClose" => { if self.gar.open && self.gar.closing.is_none() { self.gar.closing = Some(0.0); let (sw, sh) = (self.screen_width, self.screen_height); let f = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 200.0, "gar_fadeout"); self.tween_to(f, &[(Prop::Alpha, 1.0)], 150.0, Ease::Linear, Then::Nothing); } }
            "GarageTitle" => self.gar.focused = true,
            "GarageBlur" => self.gar_blur(),
            "GarageTab" => { self.gar_blur(); self.gar_switch_tab(arg); }
            "GaragePage" => {
                let all = Self::gar_frames(self.gar.tab).len();
                let pages = (all + 35) / 36;
                self.gar.page = if arg == 0 { (self.gar.page + pages - 1) % pages } else { (self.gar.page + 1) % pages };
                self.gar_build_grid();
            }
            "GarageColorPage" => {
                let pages = (GD_COLORS.len() + 11) / 12;
                let p = if arg == 0 { (self.gar.color_page + pages - 1) % pages } else { (self.gar.color_page + 1) % pages };
                self.gar_build_colors(p);
            }
            "GarageColor" => {
                let (slot, idx) = (arg >> 24, arg & 0xffffff);
                let color = GD_COLORS[idx];
                if slot == 0 { self.gar.main = color; self.gar.changes.push(format!("iconMainColor={:06x}", color)); } else { self.gar.second = color; self.gar.changes.push(format!("iconSecondaryColor={:06x}", color)); }
                self.gar_move_color_labels();
                self.gar_refresh_preview();
                if self.gar.grid_colored { self.gar_build_grid(); }
                self.events.push(Action::Pop("GarageSave", 0));
            }
            "GarageIcon" => {
                let tab = self.gar.tab;
                let (_, prefix, _, key, ..) = TABS[tab];
                let base = format!("{}_{:02}", prefix, arg);
                self.gar.cur[tab] = base.clone();
                self.gar.changes.push(format!("{}={}", key, base));
                if tab == 0 && self.gar.custom { self.gar.custom = false; self.gar.changes.push("customCubeIconSrc=".into()); }
                // move the selection label onto the chosen icon
                let want = format!("{}_001.png", base);
                let frames = Self::gar_frames(tab);
                let pos = frames.iter().position(|f| *f == want).unwrap_or(0);
                let idx = pos - self.gar.page * 36;
                let (cx, cy, _, _) = self.gar_container();
                let (ix, iy) = (cx + 30.0 + 22.5 + (idx % 12) as f64 * 57.0, cy + 15.0 + 22.5 + (idx / 12) as f64 * 57.0);
                let sl = self.gar.sel_label;
                if let Some(n) = self.nodes.get_mut(sl).and_then(|n| n.as_mut()) { n.x = ix; n.y = iy; n.sx = 0.75; n.sy = 0.75; n.visible = true; }
                self.gar_refresh_preview();
                self.events.push(Action::Pop("GarageSave", 0));
            }
            "GarageImport" => self.gar_import_popup(),
            "GarageImportClose" => { self.kill_tag("gar_imp"); self.gar.import_open = false; }
            "GarageImportPick" => self.events.push(Action::Pop("GaragePickFile", 0)),
            "GarageImportClear" => {
                self.gar.custom = false; self.gar.changes.push("customCubeIconSrc=".into()); self.events.push(Action::Pop("GarageSave", 0));
                self.kill_tag("gar_imp"); self.gar.import_open = false; self.gar_refresh_preview(); self.gar_build_grid();
            }
            _ => return false,
        }
        true
    }

    fn gar_blur(&mut self) {
        if !self.gar.focused { return; }
        self.gar.focused = false;
        if self.gar.name.trim().is_empty() { self.gar.name = "Player".into(); let t = self.gar.name.clone(); if let Some(n) = self.nodes.get_mut(self.gar.title).and_then(|n| n.as_mut()) { n.text = t; } self.gar.changes.push("playerName=Player".into()); }
    }

    /// A typed key for the name box (printable characters, backspace 8, enter 13)
    pub fn gar_key(&mut self, code: u32) {
        if !self.gar.open || !self.gar.focused { return; }
        match code {
            8 => { self.gar.name.pop(); }
            13 => { self.gar_blur(); return; }
            _ => {
                let Some(c) = char::from_u32(code) else { return };
                if NAME_CHARS.contains(c) && self.gar.name.chars().count() < 20 { self.gar.name.push(c); } else { return; }
            }
        }
        let t = self.gar.name.clone();
        let w = { let f = self.fonts.get("bigFont").cloned().unwrap_or_default(); f.width(&t, 64.0) };
        if let Some(n) = self.nodes.get_mut(self.gar.title).and_then(|n| n.as_mut()) { n.text = t.clone(); n.width = w; }
        self.gar.changes.push(format!("playerName={}", if t.is_empty() { "Player" } else { &t }));
    }

    pub fn gar_step(&mut self, dt: f64) {
        if let Some(c) = self.gar.closing.as_mut() {
            *c += dt;
            if *c >= 150.0 {
                self.gar.closing = None; self.gar.open = false; self.gar.focused = false; self.gar.import_open = false;
                self.kill_tag("gar_bg"); self.kill_tag("gar_ui"); self.kill_tag("gar_col"); self.kill_tag("gar_grid"); self.kill_tag("gar_prev"); self.kill_tag("gar_imp");
                // fade back in
                let mut fade_node = None;
                for (i, n) in self.nodes.iter().enumerate() { if matches!(n, Some(n) if n.tag == "gar_fadeout") { fade_node = Some(i); } }
                if let Some(i) = fade_node { self.node(i).tag = "gar_fade"; self.tween_to(i, &[(Prop::Alpha, 0.0)], 150.0, Ease::Linear, Then::Destroy); }
            }
        }
    }

    fn gar_import_popup(&mut self) {
        if self.gar.import_open { return; }
        self.gar.import_open = true;
        let (sw, sh) = (self.screen_width, self.screen_height);
        let (cx, cy) = (sw / 2.0, sh / 2.0);
        let (pw, ph) = ((560.0f64).min(sw * 0.88), (300.0f64).min(sh * 0.72));
        let back = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.65, 150.0, "gar_imp");
        { let n = self.node(back); n.interactive = true; n.width = sw; n.height = sh; n.action = Action::Pop("GarageImportClose", 0); }
        let root = self.shape(Kind::Rect, cx - sw, cy, 0.0, 0.0, 0, 0.0, 151.0, "gar_imp");
        self.tween_to(root, &[(Prop::X, cx)], 600.0, Ease::ElasticOut, Then::Nothing);
        let corner = self.sizes.get("/GJ_square01").map_or(80.0, |s| s.0) * 0.325;
        let panel = self.shape(Kind::Nine, 0.0, 0.0, pw, ph, 0, 1.0, 151.0, "gar_imp");
        { let n = self.node(panel); n.frame = "GJ_square01".into(); n.radius = corner; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); n.interactive = true; n.width = pw; n.height = ph; }
        for (x, y, ox, oy, fx, fy) in [(-pw / 2.0, -ph / 2.0, 0.0, 0.0, false, true), (pw / 2.0, -ph / 2.0, 1.0, 0.0, true, true), (-pw / 2.0, ph / 2.0, 0.0, 1.0, false, false), (pw / 2.0, ph / 2.0, 1.0, 1.0, true, false)] {
            let c = self.image("GJ_GameSheet03", "dailyLevelCorner_001.png", x, y, 151.5, 1.0);
            { let n = self.node(c); n.ox = ox; n.oy = oy; n.flip_x = fx; n.flip_y = fy; n.parent = Some(root); n.tag = "gar_imp"; }
        }
        let t = self.text_node("bigFont", 42.0, "Import Icon", 0.0, -ph / 2.0 + 38.0, 152.0, 0.5, 0.5, 1.0);
        { let n = self.node(t); n.parent = Some(root); n.tag = "gar_imp"; }
        let (iw, ih) = ((180.0f64).min(pw * 0.38), (64.0f64).min(ph * 0.26));
        let btn_tex = self.sizes.get("/GJ_button01").map_or(80.0, |s| s.0);
        let zone = self.shape(Kind::Rect, -pw * 0.22, 8.0, iw, ih, 0, 0.0, 152.0, "gar_imp");
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = iw; n.height = ih; n.parent = Some(root); }
        let nine = self.shape(Kind::Nine, 0.0, 0.0, iw, ih, 0, 1.0, 152.0, "gar_imp");
        { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = (btn_tex * 0.3).min(ih / 2.0 - 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
        let lb = self.text_node("goldFont", 38.0, "Import", 0.0, -2.0, 152.5, 0.5, 0.5, 1.0);
        { let n = self.node(lb); n.parent = Some(zone); n.tag = "gar_imp"; }
        self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("GarageImportPick", 0));
        let psize = (112.0f64).min(pw * 0.24).min(ph * 0.4);
        let (px, py) = (pw * 0.22, 8.0);
        let sq = self.sizes.get("/GJ_square02").map_or(80.0, |s| s.0) * 0.325;
        let pb = self.shape(Kind::Nine, px, py, psize, psize, 0, 1.0, 152.0, "gar_imp");
        { let n = self.node(pb); n.frame = "GJ_square02".into(); n.radius = sq.min(psize / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); }
        if self.gar.custom {
            let i = self.image("", "custom_cube_icon_hd", px, py, 152.5, 1.0);
            let (w, h) = (self.node(i).width.max(1.0), self.node(i).height.max(1.0));
            let s = (psize * 0.68 / w).min(psize * 0.68 / h);
            { let n = self.node(i); n.parent = Some(root); n.sx = s; n.sy = s; n.tag = "gar_imp"; }
            let del = self.image("GJ_GameSheet03", "edit_delBtnSmall_001.png", px + psize * 0.4 + 4.0, py - psize * 0.4 - 4.0, 153.0, 1.05);
            { let n = self.node(del); n.parent = Some(root); n.hit_scale = 2.0; n.tag = "gar_imp"; }
            self.make_bouncy(del, 1.05, Cond::Always, Action::Pop("GarageImportClear", 0));
        } else {
            let i = self.image("GJ_GameSheet03", "GJ_deleteIcon_001.png", px, py, 152.5, 1.0);
            let (w, h) = (self.node(i).width.max(1.0), self.node(i).height.max(1.0));
            let s = (psize * 0.5 / w).min(psize * 0.5 / h);
            { let n = self.node(i); n.parent = Some(root); n.sx = s; n.sy = s; n.tag = "gar_imp"; }
        }
        let close = self.image("GJ_WebSheet", "GJ_closeBtn_001.png", pw / 2.0 - 19.0, -ph / 2.0 + 19.0, 153.0, 0.8);
        { let n = self.node(close); n.parent = Some(root); n.hit_scale = 2.0; n.tag = "gar_imp"; }
        self.make_bouncy(close, 0.8, Cond::Always, Action::Pop("GarageImportClose", 0));
    }

    /// Re-draws after the host loaded atlases or a custom icon
    pub fn gar_rebuild(&mut self) {
        if !self.gar.open { return; }
        self.gar_refresh_preview();
        self.gar_build_grid();
        if self.gar.import_open { self.kill_tag("gar_imp"); self.gar.import_open = false; }
    }

    pub fn gar_set(&mut self, key: &str, val: &str) {
        match key {
            "iconMainColor" => self.gar.main = i64::from_str_radix(val, 16).unwrap_or(0x04ff00),
            "iconSecondaryColor" => self.gar.second = i64::from_str_radix(val, 16).unwrap_or(0x00fbff),
            "playerName" => self.gar.name = val.chars().take(20).collect(),
            "customCubeIcon" => self.gar.custom = val == "1",
            "iconGridColored" => self.gar.grid_colored = val == "1",
            _ => { for (i, t) in TABS.iter().enumerate() { if t.3 == key { self.gar.cur[i] = val.to_string(); } } }
        }
    }

    /// `x|y` pose text from the host: `tag|texture|px|py|sx|sy|flipX|flipY|z|rot` per line
    pub fn gar_set_anim(&mut self, kind: usize, text: &str) {
        let mut parts = Vec::new();
        for line in text.lines() {
            let f: Vec<&str> = line.split('|').collect();
            if f.len() < 10 { continue; }
            let n = |i: usize| f[i].parse::<f64>().unwrap_or(0.0);
            parts.push(Part { tag: n(0) as i64, texture: f[1].to_string(), pos: (n(2), n(3)), scale: (n(4), n(5)), flip: (n(6) != 0.0, n(7) != 0.0), z: n(8), rot: n(9) });
        }
        if kind < 2 { self.gar.anim[kind] = parts; }
    }
}
