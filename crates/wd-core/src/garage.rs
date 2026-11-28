//! The icon kit ("Character Select"): game-scene.js `_openIconSelector` / `_closeIconSelector`.
//! Tabs for every game mode, a 12x3 icon grid with paging, two colour rows, the player name box and the custom cube import popup.
//! The host loads the per-icon atlases (`GarageNeed` event) and stores the choices (`GarageSave` event).
use crate::ui::*;
use std::collections::HashSet;

pub const GD_COLORS: [i64; 107] = [
    0x7DFF00, 0x00FF00, 0x00FF7D, 0x00FFFF, 0x007DFF, 0x0000FF, 0x7D00FF, 0xFF00FF, 0xFF007D, 0xFF0000, 0xFF7D00, 0xFFFF00,
    0xFFFFFF, 0xB900FF, 0xFFB900, 0x000000, 0x00C8FF, 0xAFAFAF, 0x5A5A5A, 0xFF7D7D, 0x00AF4B, 0x007D7D, 0x004BAF, 0x4B00AF,
    0x7D007D, 0xAF004B, 0xAF4B00, 0x7D7D00, 0x4BAF00, 0xFF4B00, 0x963200, 0x966400, 0x649600, 0x009664, 0x006496, 0x640096,
    0x960064, 0x960000, 0x009600, 0x000096, 0x7DFFAF, 0x7D7DFF, 0xFFFA7F, 0xFA7FFF, 0x00FFC0, 0x50320E, 0xCDA576, 0xB680FF,
    0xFF3A3A, 0x4D4D8F, 0x000A4C, 0xFDD4CE, 0xBEB5FF, 0x700000, 0x520200, 0x380106, 0x804F4F, 0x7A3535, 0x512424, 0xA36246,
    0x754936, 0x563528, 0xFFB972, 0xFFA040, 0x66311E, 0x5B2700, 0x472000, 0xA77B4D, 0x6D5339, 0x513E2A, 0xFFFFC0, 0xFDE0A0,
    0xC0FFA0, 0xB1FF6D, 0xC0FFE0, 0x94FFE4, 0x43A18A, 0x316D5F, 0x265449, 0x006000, 0x004000, 0x006060, 0x004040, 0xA0FFFF,
    0x010770, 0x00496D, 0x00324C, 0x002638, 0x5080AD, 0x335375, 0x243C56, 0xE0E0E0, 0x3D068C, 0x370860, 0x404040, 0x6F49A4,
    0x54367F, 0x422A63, 0xFCB5FF, 0xAF57AF, 0x824382, 0x5E315E, 0x808080, 0x66033E, 0x470134, 0xD2FF32, 0x76BDFF,
];
/// colours saved before the palette changed: value -> index (for the selection markers)
const OLD_COLOR_IDS: [(i64, usize); 18] = [(0xAF004C, 25), (0x4AAF00, 28), (0xFF3A3B, 48), (0xFCD5CE, 51), (0x530000, 54), (0x390006, 55), (0xFEA040, 63), (0xFCE8A8, 71), (0x95FFE4, 75), (0x000770, 84), (0x6E49A4, 95), (0x432A63, 97), (0xAF57AE, 99), (0x66003E, 103), (0x470034, 104), (0xD2FF31, 105), (0x04FF00, 1), (0x00FBFF, 3)];
/// the colour popup's swatch groups: 8 blocks of 3 (or 4) rows of 4 colours
const COLOR_GROUPS: [&[[usize; 4]]; 8] = [
    &[[51, 19, 48, 9], [37, 53, 54, 55], [25, 56, 57, 58]],
    &[[62, 63, 10, 29], [26, 59, 60, 61], [30, 64, 65, 66]],
    &[[70, 42, 11, 27], [71, 14, 31, 45], [46, 67, 68, 69]],
    &[[72, 73, 0, 1], [105, 28, 32, 20], [2, 38, 79, 80]],
    &[[74, 75, 3, 44], [40, 76, 77, 78], [33, 21, 81, 82]],
    &[[83, 16, 4, 5], [22, 39, 84, 50], [34, 85, 86, 87], [106, 88, 89, 90]],
    &[[52, 41, 6, 35], [47, 23, 92, 93], [49, 95, 96, 97]],
    &[[98, 8, 36, 103], [7, 13, 24, 104], [43, 99, 100, 101]],
];
const GRAYSCALE: [usize; 7] = [12, 91, 17, 102, 18, 94, 15];
/// (tab key, frame prefix, icon count, saved key, tab button frame stem, preview size, preview y); the last two tabs are the effect pages
const TABS: [(&str, &str, usize, &str, &str, f64, f64); 10] = [
    ("icon", "player", 485, "iconCurrentPlayer", "icon", 88.0, 0.0),
    ("ship", "ship", 169, "iconCurrentShip", "ship", 106.0, 0.0),
    ("ball", "player_ball", 119, "iconCurrentBall", "ball", 88.0, 0.0),
    ("ufo", "bird", 149, "iconCurrentBird", "bird", 106.0, 21.0),
    ("wave", "dart", 96, "iconCurrentWave", "dart", 70.0, 0.0),
    ("robot", "robot", 68, "iconCurrentRobot", "robot", 88.0, 0.0),
    ("spider", "spider", 69, "iconCurrentSpider", "spider", 100.0, 0.0),
    ("swing", "swing", 43, "iconCurrentSwing", "swing", 100.0, 0.0),
    ("streak", "", 0, "", "streak", 0.0, 0.0),
    ("explosion", "", 0, "", "explosion", 0.0, 0.0),
];
const ICON_TABS: usize = 8;
const PREFIXES: [&str; 8] = ["player_ball", "player", "ship", "dart", "bird", "spider", "robot", "swing"];
const PLAYER_PRIORITY: [&str; 11] = ["GJ_GameSheet03", "GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet04", "GJ_GameSheetEditor", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_WebSheet", "GJ_LaunchSheet", "player_ball_00", "player_dart_00"];
const ATLAS_LIST: [&str; 15] = ["GJ_WebSheet", "GJ_GameSheet", "GJ_GameSheet02", "GJ_GameSheet03", "GJ_GameSheet04", "GJ_GameSheetEditor", "GJ_GameSheetGlow", "GJ_GameSheetIcons", "GJ_LaunchSheet", "player_ball_00", "player_dart_00", "GJ_ParticleSheet", "PixelSheet_01", "FireSheet_01-hd", "Wavesheet"];
/// the grid is 1.2 times the old size
const G: f64 = 1.2;
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
    /// current icon per icon tab (`player_01`, ...)
    pub cur: [String; 8],
    pub main: i64,
    pub second: i64,
    pub glow: i64,
    pub glow_on: bool,
    pub trail: usize,
    pub fire: usize,
    pub death: usize,
    pub explode: bool,
    pub run_flags: [bool; 3],
    pub name: String,
    pub focused: bool,
    pub ready: HashSet<String>,
    pub failed: HashSet<String>,
    pub need: Vec<String>,
    pub changes: Vec<String>,
    pub anim: [Vec<Part>; 2],
    pub grid_colored: bool,
    pub custom: bool,
    pub coins: (i64, i64),
    /// the colour popup: open, active tab (0 col1, 1 col2, 2 glow)
    pub paint_open: bool,
    pub paint_tab: usize,
    /// the icon tab the preview shows while an effect page is open
    pub last_preview_tab: usize,
    blink: f64,
    // nodes
    title: usize,
    cursor: usize,
    tab_btns: Vec<usize>,
    sel_label: usize,
    plus: usize,
    paint: usize,
    arrows: [usize; 2],
    paint_root: usize,
    paint_tabs: Vec<usize>,
    paint_markers: [usize; 3],
    paint_swatches: Vec<(usize, f64, f64)>,
    paint_toggle: usize,
    paint_toggle_label: usize,
    effect_markers: Vec<usize>,
    effect_buttons: Vec<Vec<usize>>,
    explode_toggle: usize,
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

    /// (x, y, w, h) of the icon grid's frame (grid sizes are 1.2x the old ones)
    fn gar_container(&self) -> (f64, f64, f64, f64) {
        let (cols, rows, size, pad, mx, my) = (12.0, 3.0, 45.0 * G, 12.0 * G, 30.0 * G, 15.0 * G);
        let w = cols * size + (cols - 1.0) * pad + mx * 2.0;
        let h = rows * size + (rows - 1.0) * pad + my * 2.0;
        (self.screen_width / 2.0 - w / 2.0, self.screen_height - h - 10.0 * G - 80.0, w, h)
    }
    /// centre of the first grid cell and the step between cells
    fn gar_cells(&self) -> (f64, f64, f64, f64) {
        let (cx, cy, _, _) = self.gar_container();
        let (size, pad) = (45.0 * G, 12.0 * G);
        (cx + 30.0 * G + size / 2.0, cy + 15.0 * G + size / 2.0, size, size + pad)
    }

    fn gar_tab_frame(tab: usize, on: bool) -> String { format!("gj_{}Btn_{}_001.png", TABS[tab].4, if on { "on" } else { "off" }) }

    pub fn gar_open(&mut self) {
        if self.gar.open { return; }
        self.gar.open = true; self.gar.closing = None; self.gar.focused = false; self.gar.page = 0; self.gar.paint_open = false;
        if self.gar.name.is_empty() { self.gar.name = "Player".into(); }
        let (sw, sh) = (self.screen_width, self.screen_height);
        self.gar.tab = 0; self.gar.last_preview_tab = 0;
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
        // player name and its blinking caret
        let title = self.text_node("bigFont", 64.0, &self.gar.name.clone(), sw / 2.0, 70.0, 105.0, 0.5, 0.5, 1.0);
        { let n = self.node(title); n.tag = "gar_ui"; n.interactive = true; n.action = Action::Pop("GarageTitle", 0); }
        self.gar.title = title;
        let cursor = self.shape(Kind::Rect, 0.0, 55.0, 3.0, 40.0, 0xffffff, 1.0, 105.1, "gar_ui");
        { let n = self.node(cursor); n.visible = false; }
        self.gar.cursor = cursor;
        self.gar_place_cursor();
        // the coin totals
        for (i, (frame, n)) in [("GJ_coinsIcon_001.png", self.gar.coins.0), ("GJ_coinsIcon2_001.png", self.gar.coins.1)].into_iter().enumerate() {
            let y = 35.0 + i as f64 * 45.0;
            let ic = self.image("GJ_GameSheet03", frame, sw - 60.0, y, 105.0, 0.8); self.node(ic).tag = "gar_ui";
            let t = self.text_node("bigFont", 32.0, &n.to_string(), sw - 86.0, y, 105.0, 1.0, 0.5, 1.0); self.node(t).tag = "gar_ui";
        }
        let back = self.image("GJ_GameSheet03", "GJ_arrow_03_001.png", 50.0, 48.0, 104.0, 1.0);
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("GarageClose", 0)); self.node(back).tag = "gar_ui";
        let shards = self.image("GJ_GameSheet03", "GJ_shardsBtn_001.png", 60.0, 165.0, 104.0, 1.0);
        self.node(shards).tag = "gar_ui"; self.lock_gray(shards);
        // fading line under the preview
        let (line_y, pad) = (210.0, 150.0);
        let (ls, le) = (pad, sw - pad);
        let lw = le - ls;
        let fade_zone = lw * 0.25;
        for li in 0..40 {
            let (t0, t1) = (li as f64 / 40.0, (li + 1) as f64 / 40.0);
            let (x0, x1) = (ls + t0 * lw, ls + t1 * lw);
            let mid = (t0 + t1) / 2.0 * lw;
            let alpha = if mid < fade_zone { mid / fade_zone } else if mid > lw - fade_zone { (lw - mid) / fade_zone } else { 1.0 };
            self.shape(Kind::Rect, x0, line_y - 2.0, x1 - x0, 4.0, 0xffffff, alpha, 102.0, "gar_ui");
        }
        // icon grid frame
        let (cx, cy, cw, ch) = self.gar_container();
        let grid_bg = self.shape(Kind::RoundRect, cx, cy, cw, ch, 0x000000, 0.3, 102.0, "gar_ui");
        self.node(grid_bg).radius = 10.0 * G;
        let tl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, 0.0, 100.0, 1.0);
        { let n = self.node(tl); n.ox = 0.0; n.oy = 0.0; n.flip_y = true; n.tag = "gar_ui"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 103.0, 1.0);
        { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "gar_ui"; }
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, 103.0, 1.0);
        { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = "gar_ui"; }
        // grid arrows
        let (ay, margin, ascale) = (cy + ch / 2.0, 54.0 * G, 0.8 * G);
        let pg = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", cx - margin, ay, 106.0, ascale);
        self.make_bouncy(pg, ascale, Cond::Always, Action::Pop("GaragePage", 0)); self.node(pg).tag = "gar_ui";
        let ng = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", cx + cw + margin, ay, 106.0, ascale);
        self.make_bouncy(ng, ascale, Cond::Always, Action::Pop("GaragePage", 1)); { let n = self.node(ng); n.flip_x = true; n.tag = "gar_ui"; }
        self.gar.arrows = [pg, ng];
        // tabs
        let ty = cy - 58.0;
        self.gar.tab_btns.clear();
        for i in 0..TABS.len() {
            let off = (i as f64 - (TABS.len() as f64 - 1.0) / 2.0) * 73.0;
            let b = self.gar_img(&Self::gar_tab_frame(i, i == 0), sw / 2.0 + off, ty, 104.0, "gar_ui").unwrap_or_else(|| self.image("GJ_GameSheet03", "gj_iconBtn_off_001.png", sw / 2.0 + off, ty, 104.0, 1.0));
            self.node(b).sx = 0.95; self.node(b).sy = 0.95;
            self.make_bouncy(b, 0.95, Cond::Always, Action::Pop("GarageTab", i)); self.node(b).tag = "gar_ui";
            self.gar.tab_btns.push(b);
        }
        // selected-icon label of the grid
        let sl = self.image("GJ_GameSheet03", "GJ_select_001.png", 0.0, 0.0, 106.0, 1.0);
        { let n = self.node(sl); n.visible = false; n.tag = "gar_ui"; }
        self.gar.sel_label = sl;
        // paint button (colours) and the custom cube import button (icon tab only)
        let paint = self.image("GJ_GameSheet03", "GJ_paintBtn_001.png", 60.0, 245.0, 104.0, 1.0);
        let k = { let (w, h) = (self.node(paint).width, self.node(paint).height); (97.0 * 0.8 / if w > 0.0 { w } else { 97.0 }).min(97.0 * 0.8 / if h > 0.0 { h } else { 97.0 }) };
        { let n = self.node(paint); n.sx = k; n.sy = k; n.tag = "gar_ui"; }
        self.make_bouncy(paint, k, Cond::Always, Action::Pop("GaragePaint", 0));
        self.gar.paint = paint;
        let plus = self.image("GJ_GameSheet03", "GJ_plusBtn_001.png", 60.0, 325.0, 104.0, 1.0);
        let k = { let (w, h) = (self.node(plus).width, self.node(plus).height); (97.0 * 0.7 / if w > 0.0 { w } else { 97.0 }).min(97.0 * 0.7 / if h > 0.0 { h } else { 97.0 }) };
        { let n = self.node(plus); n.sx = k; n.sy = k; n.tag = "gar_ui"; }
        self.make_bouncy(plus, k, Cond::Always, Action::Pop("GarageImport", 0));
        self.gar.plus = plus;
        self.gar_switch_tab(0);
    }

    /// the blinking caret sits right of the name
    fn gar_place_cursor(&mut self) {
        let w = { let f = self.fonts.get("bigFont").cloned().unwrap_or_default(); f.width(&self.gar.name, 64.0) };
        let sw = self.screen_width;
        let c = self.gar.cursor;
        if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.x = sw / 2.0 + w / 2.0 + 2.0; }
    }

    fn gar_switch_tab(&mut self, tab: usize) {
        self.gar.tab = tab; self.gar.page = 0;
        if tab < ICON_TABS { self.gar.last_preview_tab = tab; }
        for i in 0..TABS.len() {
            let Some(b) = self.gar.tab_btns.get(i).copied() else { continue };
            let frame = Self::gar_tab_frame(i, i == tab);
            let fr = if self.gar_atlas(&frame).is_some() { frame } else { Self::gar_tab_frame(0, i == tab) };
            if let Some(n) = self.nodes.get_mut(b).and_then(|n| n.as_mut()) { n.frame = fr; }
        }
        let on_icon = tab == 0;
        let plus = self.gar.plus;
        if let Some(n) = self.nodes.get_mut(plus).and_then(|n| n.as_mut()) { n.visible = on_icon; }
        let effects = tab >= ICON_TABS;
        for a in self.gar.arrows { if let Some(n) = self.nodes.get_mut(a).and_then(|n| n.as_mut()) { n.visible = !effects; n.interactive = !effects; } }
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
        for b in need { if !self.gar.need.contains(&b) { self.gar.need.push(b); } }
        self.events.push(Action::Pop("GarageNeed", 0));
        true
    }

    fn gar_segmented(tab: usize) -> Option<usize> { match TABS.get(tab).map(|t| t.0) { Some("robot") => Some(0), Some("spider") => Some(1), _ => None } }

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
        let tab = if self.gar.tab >= ICON_TABS { self.gar.last_preview_tab } else { self.gar.tab };
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
        let (main, second, glow) = (self.gar.main, self.gar.second, self.gar.glow);
        if self.gar.glow_on {
            if let Some(gl) = self.gar_img(&frame.replace("_001.png", "_glow_001.png"), sw / 2.0, py, 101.9, "gar_prev") { let n = self.node(gl); n.sx = s; n.sy = s; n.tint = glow; }
        }
        let extra_name = frame.replace("_001.png", "_2_001.png");
        if let Some(ex) = self.gar_img(&extra_name, sw / 2.0, py, 102.0, "gar_prev") { let n = self.node(ex); n.sx = s; n.sy = s; n.tint = second; }
        let i = self.image(&a, &f, sw / 2.0, py, 103.0, s);
        { let n = self.node(i); n.tint = main; n.tag = "gar_prev"; }
        let white = frame.replace("_001.png", "_extra_001.png");
        if let Some(wi) = self.gar_img(&white, sw / 2.0, py, 103.5, "gar_prev") { let n = self.node(wi); n.sx = s; n.sy = s; }
    }

    /// the page dots under the grid (`_refreshPageDots`)
    fn gar_refresh_dots(&mut self) {
        self.kill_tag("gar_dots");
        let tab = self.gar.tab;
        if tab >= ICON_TABS { return; }
        let count = (Self::gar_frames(tab).len() + 35) / 36;
        let (cx, cy, cw, ch) = self.gar_container();
        let sw = self.screen_width;
        let _ = cx;
        let spacing = (cw / (count.max(1) as f64) * 1.5).min(51.0);
        let scale = (spacing / 48.0).min(0.9625);
        for i in 0..count {
            let on = i == self.gar.page;
            let x = sw / 2.0 + (i as f64 - (count as f64 - 1.0) / 2.0) * spacing;
            let d = self.image("GJ_GameSheet03", if on { "gj_navDotBtn_on_001.png" } else { "gj_navDotBtn_off_001.png" }, x, cy + ch + 45.0, 106.0, scale);
            self.node(d).tag = "gar_dots";
            self.make_bouncy(d, scale, Cond::Always, Action::Pop("GarageDot", i));
        }
    }

    fn gar_build_grid(&mut self) {
        let tab = self.gar.tab;
        if tab >= ICON_TABS {
            self.kill_tag("gar_grid"); self.kill_tag("gar_dots");
            if let Some(n) = self.nodes.get_mut(self.gar.sel_label).and_then(|n| n.as_mut()) { n.visible = false; }
            if tab == 8 { self.gar_build_streaks(); } else { self.gar_build_explosions(); }
            return;
        }
        let all = Self::gar_frames(tab);
        let per = 36;
        let pages = (all.len() + per - 1) / per;
        self.gar.page = self.gar.page.min(pages - 1);
        let slice: Vec<String> = all.iter().skip(self.gar.page * per).take(per).cloned().collect();
        if self.gar_missing(&slice) { return; }
        self.kill_tag("gar_grid");
        self.gar_refresh_dots();
        if let Some(n) = self.nodes.get_mut(self.gar.sel_label).and_then(|n| n.as_mut()) { n.visible = false; }
        let (start_x, start_y, size, step) = self.gar_cells();
        let segmented = Self::gar_segmented(tab).is_some();
        let frames: Vec<String> = slice.iter().filter(|f| if segmented { icon_base_of(f).map_or(false, |b| self.gar.ready.contains(&b)) } else { self.gar_atlas(f).is_some() }).cloned().collect();
        let cur = self.gar.cur[tab].clone();
        for (idx, frame) in frames.iter().enumerate() {
            let (col, row) = (idx % 12, idx / 12);
            let (ix, iy) = (start_x + col as f64 * step, start_y + row as f64 * step);
            let zone = self.shape(Kind::Rect, ix, iy, size, size, 0, 0.0, 104.0, "gar_grid");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = size; n.height = size; }
            let colored = self.gar.grid_colored;
            if segmented {
                let _ = self.gar_composite(tab, frame, Some(zone), 0.0, 0.0, size, 103.0, true, "gar_grid");
            } else if let Some((a, f)) = self.gar_atlas(frame) {
                let (w, h) = self.gar_size(&a, &f);
                let s = (size / if w > 0.0 { w } else { size }).min(size / if h > 0.0 { h } else { size });
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
            let num: usize = frame.trim_end_matches("_001.png").rsplit('_').next().and_then(|d| d.parse().ok()).unwrap_or(0);
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("GarageIcon", num));
            if frame.trim_end_matches("_001.png") == cur && !(tab == 0 && self.gar.custom) {
                let sl = self.gar.sel_label;
                if let Some(n) = self.nodes.get_mut(sl).and_then(|n| n.as_mut()) { n.x = ix; n.y = iy; n.sx = 0.75 * G; n.sy = 0.75 * G; n.visible = true; }
            }
        }
    }

    /// `_buildStreakGrid`: trail looks (7), ship fires (6) and the three run-animation toggles
    fn gar_build_streaks(&mut self) {
        let (start_x, start_y, size, step) = self.gar_cells();
        self.gar.effect_markers.clear(); self.gar.effect_buttons = vec![Vec::new(), Vec::new(), Vec::new()];
        for _ in 0..2 {
            let m = self.image("GJ_GameSheet03", "GJ_select_001.png", 0.0, 0.0, 106.0, 1.0);
            { let n = self.node(m); n.tag = "gar_grid"; n.visible = false; }
            self.gar.effect_markers.push(m);
        }
        for (row, count) in [7usize, 6, 3].into_iter().enumerate() {
            for col in 0..count {
                let id = col + 1;
                let frame = match row { 0 => format!("player_special_{:02}_001.png", id), 1 => format!("shipfireIcon_{:02}_001.png", id), _ => format!("gjItem_{}_001.png", 18 + col) };
                let b = self.image("GJ_GameSheet03", &frame, start_x + col as f64 * step, start_y + row as f64 * step, 104.0, 1.0);
                let (w, h) = (self.node(b).width, self.node(b).height);
                let k = (size / if w > 0.0 { w } else { size }).min(size / if h > 0.0 { h } else { size });
                { let n = self.node(b); n.sx = k; n.sy = k; n.tag = "gar_grid"; }
                self.make_bouncy(b, k, Cond::Always, Action::Pop("GarageStreak", (row << 8) | col));
                self.gar.effect_buttons[row].push(b);
            }
        }
        self.gar_refresh_effects();
    }

    fn gar_refresh_effects(&mut self) {
        let sel = [self.gar.trail, self.gar.fire];
        for row in 0..2 {
            let Some(&btn) = self.gar.effect_buttons.get(row).and_then(|r| r.get(sel[row].saturating_sub(1))) else { continue };
            let (x, y, w) = { let n = self.node(btn); (n.x, n.y, 0.0f64.max(n.width)) };
            let size = 45.0 * G;
            let m = self.gar.effect_markers[row];
            let mw = self.node(m).width.max(1.0);
            let _ = w;
            let k = size * 1.25 / mw;
            if let Some(n) = self.nodes.get_mut(m).and_then(|n| n.as_mut()) { n.x = x; n.y = y; n.sx = k; n.sy = k; n.visible = true; }
        }
        for (i, &b) in self.gar.effect_buttons.get(2).cloned().unwrap_or_default().iter().enumerate() {
            let on = self.gar.run_flags[i];
            if let Some(n) = self.nodes.get_mut(b).and_then(|n| n.as_mut()) { n.tint = if on { 0xffffff } else { 0x555555 }; }
        }
    }

    /// `_buildExplosionGrid`: 20 death effects and the "Explode" checkbox
    fn gar_build_explosions(&mut self) {
        let (start_x, start_y, size, step) = self.gar_cells();
        self.gar.effect_markers.clear(); self.gar.effect_buttons = vec![Vec::new()];
        let m = self.image("GJ_GameSheet03", "GJ_select_001.png", 0.0, 0.0, 106.0, 1.0);
        { let n = self.node(m); n.tag = "gar_grid"; n.visible = false; }
        self.gar.effect_markers.push(m);
        for i in 0..20usize {
            let (col, row) = (i % 12, i / 12);
            let b = self.image("GJ_GameSheet03", &format!("explosionIcon_{:02}_001.png", i + 1), start_x + col as f64 * step, start_y + row as f64 * step, 104.0, size / 60.0);
            { let n = self.node(b); n.tag = "gar_grid"; }
            self.make_bouncy(b, size / 60.0, Cond::Always, Action::Pop("GarageDeath", i + 1));
            self.gar.effect_buttons[0].push(b);
        }
        let ty = start_y + 2.0 * step;
        let frame = if self.gar.explode { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" };
        let tk = size * 0.85 / 62.0;
        let t = self.image("GJ_GameSheet03", frame, start_x, ty, 104.0, tk);
        { let n = self.node(t); n.tag = "gar_grid"; }
        self.make_bouncy(t, tk, Cond::Always, Action::Pop("GarageExplode", 0));
        self.gar.explode_toggle = t;
        let l = self.text_node("goldFont", 30.0, "Explode", start_x + step * 0.62, ty, 104.0, 0.0, 0.5, 1.0);
        { let n = self.node(l); n.tag = "gar_grid"; }
        self.gar_refresh_death();
    }

    fn gar_refresh_death(&mut self) {
        let Some(&btn) = self.gar.effect_buttons.get(0).and_then(|r| r.get(self.gar.death.saturating_sub(1))) else { return };
        let (x, y) = { let n = self.node(btn); (n.x, n.y) };
        let m = self.gar.effect_markers[0];
        let mw = self.node(m).width.max(1.0);
        let k = 45.0 * G * 1.25 / mw;
        if let Some(n) = self.nodes.get_mut(m).and_then(|n| n.as_mut()) { n.x = x; n.y = y; n.sx = k; n.sy = k; n.visible = true; }
    }

    // ---- the colour popup (`_openColorPopup`) ----
    fn gar_swatch_pos(id: usize) -> Option<(f64, f64)> {
        for (gi, group) in COLOR_GROUPS.iter().enumerate() {
            for (ri, row) in group.iter().enumerate() {
                for (ci, &c) in row.iter().enumerate() {
                    if c == id { return Some((-386.0 + (gi % 4) as f64 * 210.0 + ci as f64 * 47.0, -79.0 + (gi / 4) as f64 * 160.0 + ri as f64 * 47.0)); }
                }
            }
        }
        GRAYSCALE.iter().position(|&c| c == id).map(|i| (104.0 + i as f64 * 47.0, 235.0))
    }

    fn gar_color_id(&self, color: i64) -> Option<usize> {
        GD_COLORS.iter().position(|c| *c == color).or_else(|| OLD_COLOR_IDS.iter().find(|(c, _)| *c == color).map(|(_, i)| *i))
    }

    fn gar_paint_popup(&mut self) {
        if self.gar.paint_open { return; }
        self.gar.paint_open = true; self.gar.paint_tab = 0;
        self.gar_blur();
        let (sw, sh) = (self.screen_width, self.screen_height);
        let backdrop = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.65, 150.0, "gar_paint");
        { let n = self.node(backdrop); n.interactive = true; n.width = sw; n.height = sh; }
        let scale = ((sw - 80.0) / 800.0).min((sh - 55.0) / 530.0).min(1.1);
        let root = self.shape(Kind::Rect, -sw / 2.0, sh / 2.0, 0.0, 0.0, 0, 0.0, 151.0, "gar_paint");
        { let n = self.node(root); n.sx = scale; n.sy = scale; }
        self.tween_to(root, &[(Prop::X, sw / 2.0)], 400.0, Ease::ElasticOut, Then::Nothing);
        self.gar.paint_root = root;
        let corner = self.sizes.get("/GJ_square05").map_or(80.0, |s| s.0) * 0.325;
        let panel = self.shape(Kind::Nine, 0.0, 0.0, 860.0, 560.0, 0, 1.0, 151.0, "gar_paint");
        { let n = self.node(panel); n.frame = "GJ_square05".into(); n.radius = corner; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); n.interactive = true; n.width = 860.0; n.height = 560.0; }
        let pbg = self.shape(Kind::RoundRect, -378.0, -220.0, 756.0, 103.0, 0x414141, 1.0, 151.2, "gar_paint");
        { let n = self.node(pbg); n.radius = 12.0; n.parent = Some(root); }
        let close = self.image("GJ_GameSheet03", "GJ_closeBtn_001.png", -400.0, -255.0, 152.0, 0.75);
        { let n = self.node(close); n.parent = Some(root); n.tag = "gar_paint"; }
        self.make_bouncy(close, 0.75, Cond::Always, Action::Pop("GaragePaintClose", 0));
        // the three colour tabs
        self.gar.paint_tabs.clear();
        let btn_tex = self.sizes.get("/GJ_button01").map_or(80.0, |s| s.0);
        for (i, label) in ["Col1", "Col2", "Glow"].into_iter().enumerate() {
            let zone = self.shape(Kind::Rect, 140.0 + i as f64 * 97.0, -247.0, 102.0 / 1.2, 40.0 / 1.2, 0, 0.0, 152.0, "gar_paint");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 102.0 / 1.2; n.height = 40.0 / 1.2; n.parent = Some(root); }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, 102.0, 40.0, 0, 1.0, 152.0, "gar_paint");
            { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = (btn_tex * 0.3).min(18.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); n.sx = 1.0 / 1.2; n.sy = 1.0 / 1.2; }
            let t = self.text_node("bigFont", 22.0, label, -3.0, -3.0, 152.5, 0.5, 0.5, 1.0);
            { let n = self.node(t); n.parent = Some(zone); n.tag = "gar_paint"; }
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("GaragePaintTab", i));
            self.gar.paint_tabs.push(nine);
        }
        // swatches
        self.gar.paint_swatches.clear();
        let mut ids: Vec<usize> = Vec::new();
        for group in COLOR_GROUPS.iter() { for row in group.iter() { ids.extend(row.iter().copied()); } }
        ids.extend(GRAYSCALE.iter().copied());
        for id in ids {
            let Some((x, y)) = Self::gar_swatch_pos(id) else { continue };
            let zone = self.shape(Kind::Rect, x, y, 40.0, 40.0, 0, 0.0, 152.0, "gar_paint");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 40.0; n.height = 40.0; n.parent = Some(root); }
            let shadow = self.shape(Kind::RoundRect, -17.0, -16.0, 40.0, 40.0, 0, 0.35, 152.0, "gar_paint");
            { let n = self.node(shadow); n.radius = 6.0; n.parent = Some(zone); }
            let rim = self.shape(Kind::RoundRect, -20.0, -20.0, 40.0, 40.0, 0, 1.0, 152.1, "gar_paint");
            { let n = self.node(rim); n.radius = 6.0; n.parent = Some(zone); }
            let body = self.shape(Kind::RoundRect, -18.0, -18.0, 36.0, 36.0, GD_COLORS[id], 1.0, 152.2, "gar_paint");
            { let n = self.node(body); n.radius = 5.0; n.parent = Some(zone); }
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("GarageSwatch", id));
            self.gar.paint_swatches.push((id, x, y));
        }
        for i in 0..3 {
            let m = self.image("GJ_GameSheet03", "GJ_select_001.png", 0.0, 0.0, 153.0, 1.0);
            { let n = self.node(m); n.parent = Some(root); n.visible = false; n.tag = "gar_paint"; }
            self.gar.paint_markers[i] = m;
        }
        let tk = 0.55;
        let toggle = self.image("GJ_GameSheet03", if self.gar.glow_on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, -387.0, 235.0, 152.0, tk);
        { let n = self.node(toggle); n.parent = Some(root); n.tag = "gar_paint"; }
        self.make_bouncy(toggle, tk, Cond::Always, Action::Pop("GarageGlow", 0));
        let tl = self.text_node("bigFont", 28.0, "Glow", -355.0, 235.0, 152.0, 0.0, 0.5, 1.0);
        { let n = self.node(tl); n.parent = Some(root); n.tag = "gar_paint"; }
        self.gar.paint_toggle = toggle; self.gar.paint_toggle_label = tl;
        self.gar_paint_refresh();
    }

    fn gar_paint_close(&mut self) {
        if !self.gar.paint_open { return; }
        self.gar.paint_open = false;
        self.kill_tag("gar_paint");
    }

    /// markers, tab look, glow checkbox and the row of icon previews
    fn gar_paint_refresh(&mut self) {
        if !self.gar.paint_open { return; }
        let root = self.gar.paint_root;
        let active = self.gar.paint_tab;
        let colors = [self.gar.main, self.gar.second, self.gar.glow];
        let mut stacked: std::collections::HashMap<usize, f64> = Default::default();
        let order = [active].into_iter().chain((0..3).filter(|&i| i != active)).collect::<Vec<_>>();
        for i in order {
            let m = self.gar.paint_markers[i];
            let id = self.gar_color_id(colors[i]);
            let pos = id.and_then(|id| self.gar.paint_swatches.iter().find(|s| s.0 == id).map(|s| (id, s.1, s.2)));
            match pos {
                Some((id, x, y)) => {
                    let overlap = *stacked.get(&id).unwrap_or(&0.0);
                    stacked.insert(id, overlap + 1.0);
                    let k = 0.6 * 40.0 / 35.0 - overlap * 0.13;
                    if let Some(n) = self.nodes.get_mut(m).and_then(|n| n.as_mut()) { n.x = x; n.y = y; n.sx = k; n.sy = k; n.tint = if i == active { 0xffffff } else { 0x2d2e2e }; n.visible = true; }
                }
                None => { if let Some(n) = self.nodes.get_mut(m).and_then(|n| n.as_mut()) { n.visible = false; } }
            }
        }
        for (i, &b) in self.gar.paint_tabs.clone().iter().enumerate() {
            if let Some(n) = self.nodes.get_mut(b).and_then(|n| n.as_mut()) { n.frame = if i == active { "GJ_button01" } else { "GJ_button04" }.into(); }
        }
        let (tg, tl, on) = (self.gar.paint_toggle, self.gar.paint_toggle_label, active == 2);
        for n in [tg, tl] { if let Some(n) = self.nodes.get_mut(n).and_then(|n| n.as_mut()) { n.visible = on; } }
        if let Some(n) = self.nodes.get_mut(tg).and_then(|n| n.as_mut()) { n.frame = if self.gar.glow_on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }.into(); }
        // the equipped icons of every tab, in the current colours
        self.kill_tag("gar_pprev");
        for tab in 0..ICON_TABS {
            let x = (tab as f64 - (ICON_TABS as f64 - 1.0) / 2.0) * 92.0;
            let (_, _, _, _, _, size, dy) = TABS[tab];
            let (pv, y) = (size * 0.75, -167.5 + dy * 0.75);
            let frame = self.gar_current_frame(tab);
            if tab == 0 && self.gar.custom {
                let i = self.image("", "custom_cube_icon_hd", x, y, 152.5, 1.0);
                let (w, h) = (self.node(i).width.max(1.0), self.node(i).height.max(1.0));
                let s = (80.0 / w).min(80.0 / h) * 0.85 * 0.75;
                { let n = self.node(i); n.parent = Some(root); n.sx = s; n.sy = s; n.tag = "gar_pprev"; }
                continue;
            }
            if self.gar_missing(&[frame.clone()]) { continue; }
            if Self::gar_segmented(tab).is_some() {
                self.gar_composite(tab, &frame, Some(root), x, y, pv, 152.5, false, "gar_pprev");
                continue;
            }
            let Some((a, f)) = self.gar_atlas(&frame) else { continue };
            let (w, h) = self.gar_size(&a, &f);
            let s = (pv / if w > 0.0 { w } else { pv }).min(pv / if h > 0.0 { h } else { pv });
            let (main, second, glow) = (self.gar.main, self.gar.second, self.gar.glow);
            let mut layers: Vec<(String, i64, f64)> = Vec::new();
            if self.gar.glow_on { layers.push((frame.replace("_001.png", "_glow_001.png"), glow, 152.5)); }
            if tab == 3 { layers.push((frame.replace("_001.png", "_3_001.png"), second, 152.6)); }
            layers.push((frame.replace("_001.png", "_2_001.png"), second, 152.7));
            layers.push((frame.clone(), main, 152.8));
            layers.push((frame.replace("_001.png", "_extra_001.png"), -1, 152.9));
            for (name, tint, depth) in layers {
                let Some((la, lf)) = self.gar_atlas(&name) else { continue };
                let i = self.image(&la, &lf, x, y, depth, s);
                { let n = self.node(i); n.parent = Some(root); n.tint = tint; n.tag = "gar_pprev"; }
            }
        }
    }

    pub fn gar_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "GarageClose" => {
                if self.gar.paint_open { self.gar_paint_close(); return true; }
                if self.gar.open && self.gar.closing.is_none() { self.gar.closing = Some(0.0); let (sw, sh) = (self.screen_width, self.screen_height); let f = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 200.0, "gar_fadeout"); self.tween_to(f, &[(Prop::Alpha, 1.0)], 150.0, Ease::Linear, Then::Nothing); }
            }
            "GarageTitle" => { if !self.gar.paint_open { self.gar.focused = true; self.gar.blink = 0.0; let c = self.gar.cursor; if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.visible = true; } } }
            "GarageBlur" => self.gar_blur(),
            "GarageTab" => { self.gar_blur(); self.gar_switch_tab(arg); }
            "GaragePage" => {
                let all = Self::gar_frames(self.gar.tab.min(ICON_TABS - 1)).len();
                let pages = ((all + 35) / 36).max(1);
                self.gar.page = if arg == 0 { (self.gar.page + pages - 1) % pages } else { (self.gar.page + 1) % pages };
                self.gar_build_grid();
            }
            "GarageDot" => { self.gar.page = arg; self.gar_build_grid(); }
            "GaragePaint" => self.gar_paint_popup(),
            "GaragePaintClose" => self.gar_paint_close(),
            "GaragePaintTab" => { self.gar.paint_tab = arg; self.gar_paint_refresh(); }
            "GarageSwatch" => {
                let color = GD_COLORS[arg];
                match self.gar.paint_tab {
                    0 => { self.gar.main = color; self.gar.changes.push(format!("iconMainColor={:06x}", color)); }
                    1 => { self.gar.second = color; self.gar.changes.push(format!("iconSecondaryColor={:06x}", color)); }
                    _ => { self.gar.glow = color; self.gar.changes.push(format!("iconGlowColor={:06x}", color)); }
                }
                self.gar_refresh_preview();
                if self.gar.grid_colored { self.gar_build_grid(); }
                self.gar_paint_refresh();
                self.events.push(Action::Pop("GarageSave", 0));
            }
            "GarageGlow" => {
                self.gar.glow_on = !self.gar.glow_on;
                self.gar.changes.push(format!("iconGlowEnabled={}", self.gar.glow_on));
                self.gar_refresh_preview();
                self.gar_paint_refresh();
                self.events.push(Action::Pop("GarageSave", 0));
            }
            "GarageStreak" => {
                let (row, col) = (arg >> 8, arg & 0xff);
                match row {
                    0 => { self.gar.trail = col + 1; self.gar.changes.push(format!("iconCurrentTrail={}", col + 1)); }
                    1 => { self.gar.fire = col + 1; self.gar.changes.push(format!("iconCurrentShipFire={}", col + 1)); }
                    _ => {
                        self.gar.run_flags[col] = !self.gar.run_flags[col];
                        let key = ["iconSlowRobotRun", "iconFastRobotRun", "iconFastSpiderRun"][col];
                        self.gar.changes.push(format!("{}={}", key, self.gar.run_flags[col]));
                    }
                }
                self.gar_refresh_effects();
                self.events.push(Action::Pop("GarageSave", 0));
            }
            "GarageDeath" => {
                self.gar.death = arg;
                self.gar.changes.push(format!("iconCurrentDeathEffect={}", arg));
                self.gar_refresh_death();
                self.events.push(Action::Pop("GarageSave", 0));
            }
            "GarageExplode" => {
                self.gar.explode = !self.gar.explode;
                self.gar.changes.push(format!("iconExplode={}", self.gar.explode));
                let t = self.gar.explode_toggle;
                if let Some(n) = self.nodes.get_mut(t).and_then(|n| n.as_mut()) { n.frame = if self.gar.explode { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }.into(); }
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
                let (sx0, sy0, _, step) = self.gar_cells();
                let (ix, iy) = (sx0 + (idx % 12) as f64 * step, sy0 + (idx / 12) as f64 * step);
                let sl = self.gar.sel_label;
                if let Some(n) = self.nodes.get_mut(sl).and_then(|n| n.as_mut()) { n.x = ix; n.y = iy; n.sx = 0.75 * G; n.sy = 0.75 * G; n.visible = true; }
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
        let c = self.gar.cursor;
        if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.visible = false; }
        if !self.gar.focused { return; }
        self.gar.focused = false;
        if self.gar.name.trim().is_empty() { self.gar.name = "Player".into(); let t = self.gar.name.clone(); if let Some(n) = self.nodes.get_mut(self.gar.title).and_then(|n| n.as_mut()) { n.text = t; } self.gar.changes.push("playerName=Player".into()); self.gar_place_cursor(); }
    }

    /// A typed key for the name box (printable characters, backspace 8, enter 13)
    pub fn gar_key(&mut self, code: u32) {
        if !self.gar.open || !self.gar.focused { return; }
        match code {
            8 => { self.gar.name.pop(); }
            13 | 27 => { self.gar_blur(); return; }
            _ => {
                let Some(c) = char::from_u32(code) else { return };
                if NAME_CHARS.contains(c) && self.gar.name.chars().count() < 20 { self.gar.name.push(c); } else { return; }
            }
        }
        let t = self.gar.name.clone();
        let w = { let f = self.fonts.get("bigFont").cloned().unwrap_or_default(); f.width(&t, 64.0) };
        if let Some(n) = self.nodes.get_mut(self.gar.title).and_then(|n| n.as_mut()) { n.text = t.clone(); n.width = w; }
        self.gar.changes.push(format!("playerName={}", if t.is_empty() { "Player" } else { &t }));
        self.gar_place_cursor();
        self.gar.blink = 0.0;
        let c = self.gar.cursor;
        if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.visible = true; }
    }

    pub fn gar_step(&mut self, dt: f64) {
        if self.gar.open && self.gar.focused {
            self.gar.blink += dt;
            if self.gar.blink >= 500.0 {
                self.gar.blink -= 500.0;
                let c = self.gar.cursor;
                if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.visible = !n.visible; }
            }
        }
        if let Some(c) = self.gar.closing.as_mut() {
            *c += dt;
            if *c >= 150.0 {
                self.gar.closing = None; self.gar.open = false; self.gar.focused = false; self.gar.import_open = false; self.gar.paint_open = false;
                self.kill_tag("gar_bg"); self.kill_tag("gar_ui"); self.kill_tag("gar_col"); self.kill_tag("gar_grid"); self.kill_tag("gar_prev"); self.kill_tag("gar_imp"); self.kill_tag("gar_dots"); self.kill_tag("gar_paint"); self.kill_tag("gar_pprev");
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
        self.gar_paint_refresh();
        if self.gar.import_open { self.kill_tag("gar_imp"); self.gar.import_open = false; }
    }

    pub fn gar_set(&mut self, key: &str, val: &str) {
        let flag = val == "true" || val == "1";
        match key {
            "iconMainColor" => self.gar.main = i64::from_str_radix(val, 16).unwrap_or(0x04ff00),
            "iconSecondaryColor" => self.gar.second = i64::from_str_radix(val, 16).unwrap_or(0x00fbff),
            "iconGlowColor" => self.gar.glow = i64::from_str_radix(val, 16).unwrap_or(0x7dff00),
            "iconGlowEnabled" => self.gar.glow_on = flag,
            "iconCurrentTrail" => self.gar.trail = val.parse().ok().filter(|v| (1..=7).contains(v)).unwrap_or(1),
            "iconCurrentShipFire" => self.gar.fire = val.parse().ok().filter(|v| (1..=6).contains(v)).unwrap_or(1),
            "iconCurrentDeathEffect" => self.gar.death = val.parse().ok().filter(|v| (1..=20).contains(v)).unwrap_or(1),
            "iconExplode" => self.gar.explode = val != "false",
            "iconSlowRobotRun" => self.gar.run_flags[0] = flag,
            "iconFastRobotRun" => self.gar.run_flags[1] = flag,
            "iconFastSpiderRun" => self.gar.run_flags[2] = flag,
            "secretCoins" => self.gar.coins.0 = val.parse().unwrap_or(0),
            "userCoins" => self.gar.coins.1 = val.parse().unwrap_or(0),
            "playerName" => self.gar.name = val.chars().take(20).collect(),
            "customCubeIcon" => self.gar.custom = val == "1",
            "iconGridColored" => self.gar.grid_colored = val == "1",
            _ => { for (i, t) in TABS.iter().enumerate().take(ICON_TABS) { if t.3 == key { self.gar.cur[i] = val.to_string(); } } }
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
