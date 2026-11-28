//! Popups group A: How To Play, Newgrounds, Credits, News, Help (+ its Req/Contact/Notes popups), Songs.
use crate::ui::*;

#[derive(Default)]
pub struct PopA {
    htp_root: usize,
    htp_page: usize,
    htp_label: usize,
    htp_img: usize,
    htp_open: bool,
    ng_open: bool,
    cr_open: bool,
    news_open: bool,
    news_root: usize,
    news_page: usize,
    news_label: usize,
    news_geom: (f64, f64, f64, f64, f64),
    news_si: usize,
    news_thumb: usize,
    news_view: (f64, f64),
    help_open: bool,
    help_closing: bool,
    help_clock: f64,
    help_root: usize,
    help_overlay: usize,
    help_check: usize,
    songs_open: bool,
    songs_clock: f64,
    songs_root: usize,
    songs_overlay: usize,
    req_open: bool,
    req_kind: u8,
    req_clock: f64,
    req_spin: usize,
    req_text: usize,
    req_panel: usize,
    req_close: usize,
    req_lock: usize,
    req_texts: [usize; 6],
    req_shown: bool,
    pending: Option<(&'static str, f64)>,
}

const WHITE: i64 = 0xffffff;
const GREEN: i64 = 0x00e719;
const YELLOW: i64 = 0xf8ff00;
const BLUE: i64 = 0x3cadf5;

/// text segments of a How To Play line
type Seg = (&'static str, Option<i64>);
const TUTORIAL: [(f64, &[&[Seg]]); 5] = [
    (40.0, &[&[("TAP", Some(GREEN)), (" the screen to jump.", None)], &[("HOLD", Some(GREEN)), (" down to keep jumping.", None)]]),
    (40.0, &[&[("Hold", Some(GREEN)), (" to fly up.", None)], &[("Release", Some(GREEN)), (" to fly down.", None)]]),
    (35.0, &[&[("You can enter ", None), ("practice mode", Some(BLUE)), (" from", None)], &[("the pause menu.", None)], &[("Practice mode lets you place", None)], &[("checkpoints", Some(GREEN)), (".", None)]]),
    (35.0, &[&[("You can place checkpoints manually, or", None)], &[("use the auto-checkpoints feature.", None)], &[("Tap the delete button to remove your", None)], &[("last checkpoint.", None)]]),
    (35.0, &[&[("Jump Orbs", Some(YELLOW)), (" activate when you are on", None)], &[("top of them.", None)], &[("TAP", Some(GREEN)), (" while touching an orb to", None)], &[("interact with it and use its effect.", None)]]),
];

/// Arial advance widths (1/1000 em) for ASCII 32..126
const ARIAL: [u16; 95] = [278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333,
    556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584];

pub(crate) fn arial_width(s: &str, size: f64) -> f64 {
    s.chars().map(|c| { let i = c as usize; if (32..127).contains(&i) { ARIAL[i - 32] as f64 } else { 556.0 } }).sum::<f64>() * size / 1000.0
}

/// Phaser's greedy word wrap
pub(crate) fn wrap_arial(text: &str, size: f64, width: f64) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let joined = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
        if line.is_empty() || arial_width(&joined, size) <= width { line = joined; } else { lines.push(std::mem::take(&mut line)); line = word.to_string(); }
    }
    lines.push(line);
    lines
}

const NEWS: [(&str, &str, &[&str]); 3] = [
    ("November 27th, 2026", "QOL Update! - V2.1", &[
        "AKA. Lasokar's super tuff return to Web Dashers development",
        "This update includes:",
        "- Swing gamemode :o",
        "- Added spider orbs and pads",
        "- New color selector and icon kit menus",
        "- Fixed being unable to delete saved lvls",
        "- Fixed dual going above the ceiling",
        "- Fixed noclip accuracy wave collisions",
        "- Player glow!",
        "- Free Mode",
        "- Custom trails, ship fires, and death fx",
        "- Fixed icon layering and ship trail",
        "- Fixed dual boundaries",
        "- Fixed D blocks in slopes",
        "- Made triggers tick based",
        "- Hitbox consistency fixes",
        "- Fixed slope snapping when it shouldnt be",
        "- Rename ignore damage back to noclip",
        "- Added the ability to import .gdlvl files",
        "- Added timewarp trigger",
        "- Added H blocks",
        "- Added Player renaming",
        "- Fixed rotated line hitboxes",
        "- Fixed editor camera up arrow bug",
        "- Fixed the coloring for some blocks",
        "- Fixed slope collisions when teleporting as the spider",
        "- Fixed physics inconsistencies when playtesting in editor",
        "- Made the news tab functional instead of falling back to having 1 entry (wtf did you do pinkdev)",
        "- Other bug fixes stuff that I might've added but i forgor",
    ]),
    ("November 23rd, 2026", "Minor Bugfix Update - V2.01", &[
        "A bunch of bugs have been fixed plus minor changes and additions.",
        "This update includes:",
        "- Revamped Create Menu UI",
        "- Fixed some physics and gameplay bugs",
        "- Revamped Stats Menu",
        "- Seperated UI from game-scene (now ui-scene) (MAINLY FOR DEVELOPERS)",
        "- Fixed dash orb cancelling",
        "- Fixed levels seeming broken for a split moment before fully loading",
        "- and more i dont remember",
    ]),
    ("November 13th, 2026", "MEGA UPDATE - V2.0", &[
        "V2.0 Mega Update. me (rohanis0000) and PinkDev have combined our updates into 1 huge update for more content for you guys to play with.",
        "This update includes:",
        "2.2 icons",
        "Fixed robot and spider icons/layering",
        "More colors for the icon kit",
        "Custom icons",
        "The Vault and Vault of Secrets",
        "A new update menu (News)",
        "Custom keybinds",
        "3D textures fixed",
        "MANY, MANY BUG FIXES",
        "2.209 UI updates",
        "Daily Levels",
        "Weekly Demons",
        "Event Levels",
        "Level Stats",
        "Progress Bar",
        "Startpos Revamp",
        "Fixed/Added a lot of triggers",
        "Texture fixes",
        "Object warping",
        "Mirror portal transitions",
        "Minor QOL additions",
        "and more i think i prolly forgot ngl",
    ]),
];

impl Ui {
    pub fn pa_open(&mut self, name: &str) -> bool {
        match name {
            "HowToPlay" => self.pa_how_to_play(),
            "Newgrounds" => self.pa_newgrounds(),
            "Credits" => self.pa_credits(),
            "News" => self.pa_news(),
            "Help" => { self.close_settings(); self.pa.pending = Some(("Help", 0.0)); }
            "Songs" => { self.close_settings(); self.pa.pending = Some(("Songs", 0.0)); }
            _ => return false,
        }
        true
    }

    pub fn pa_action(&mut self, name: &str, arg: usize) -> bool {
        match (name, arg) {
            ("HowToPlay", 0) => { self.kill_tag("pa_htp"); self.pa.htp_open = false; }
            ("HowToPlay", _) => self.pa_htp_next(),
            ("Newgrounds", 0) => { self.kill_tag("pa_ng"); self.pa.ng_open = false; }
            ("Newgrounds", _) => { self.kill_tag("pa_ng"); self.pa.ng_open = false; self.events.push(Action::OpenUrl("https://www.newgrounds.com/audio")); }
            ("Credits", _) => { self.kill_tag("pa_cr"); self.pa.cr_open = false; }
            ("News", _) => { self.kill_tag("pa_news"); self.kill_tag("pa_news_c"); self.pa.news_open = false; }
            ("NewsPage", dir) => {
                let n = NEWS.len();
                self.pa.news_page = if dir == 0 { (self.pa.news_page + n - 1) % n } else { (self.pa.news_page + 1) % n };
                self.pa_news_render();
            }
            ("Help", 0) => self.pa_close_help(),
            ("Help", 1) | ("Help", 3) => self.pa_req(2),
            ("Help", 2) => self.pa_req(1),
            ("Help", 4) => {
                let v = if self.opts.get("enableLDM").copied().unwrap_or(0.0) != 0.0 { 0.0 } else { 1.0 };
                self.opts.insert("enableLDM".into(), v);
                let f = if v != 0.0 { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" };
                let c = self.pa.help_check;
                if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.frame = f.into(); }
                self.events.push(Action::OptChanged);
            }
            ("Help", _) => self.pa_req(0),
            ("Req", 0) => { self.kill_tag("pa_req"); self.pa.req_open = false; }
            ("Req", _) => self.pa_contact_toggle(),
            ("Songs", _) => {
                self.kill_tag("pa_songs"); self.pa.songs_open = false;
                self.pa_pause_btn(1.0);
            }
            _ => return false,
        }
        true
    }

    pub(crate) fn pa_pause_btn(&mut self, alpha: f64) {
        let hud: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "hud_pause")).collect();
        for i in hud { self.tween_to(i, &[(Prop::Alpha, alpha)], 300.0, Ease::Linear, Then::Nothing); }
    }

    // ---- helpers ----
    /// dim + (scaled-in) container root; returns root
    pub(crate) fn pa_popup(&mut self, tag: &'static str, depth: f64, dim_a: f64, bounce: bool) -> usize {
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, dim_a, depth, tag);
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        let root = self.shape(Kind::Rect, if bounce { cx } else { 0.0 }, if bounce { 320.0 } else { 0.0 }, 0.0, 0.0, 0, 0.0, depth, tag);
        if bounce {
            { let n = self.node(root); n.sx = 0.0; n.sy = 0.0; }
            self.tween_to(root, &[(Prop::ScaleX, 1.0), (Prop::ScaleY, 1.0)], 660.0, Ease::ElasticOut, Then::Nothing);
        }
        root
    }

    pub(crate) fn pa_nine(&mut self, root: usize, frame: &str, x: f64, y: f64, w: f64, h: f64, depth: f64, tag: &'static str) -> usize {
        let tex = self.sizes.get(&format!("/{}", frame)).map_or(80.0, |s| s.0);
        let nine = self.shape(Kind::Nine, x, y, w, h, 0, 1.0, depth, tag);
        { let n = self.node(nine); n.frame = frame.into(); n.radius = (tex * 0.325).min(tex / 2.0).min(w / 2.0).min(h / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); }
        nine
    }

    pub(crate) fn pa_img(&mut self, root: usize, atlas: &str, frame: &str, x: f64, y: f64, scale: f64, depth: f64, tag: &'static str) -> usize {
        let i = self.image(atlas, frame, x, y, depth, scale);
        { let n = self.node(i); n.parent = Some(root); n.tag = tag; }
        i
    }

    pub(crate) fn pa_txt(&mut self, root: usize, font: &str, size: f64, text: &str, x: f64, y: f64, ox: f64, oy: f64, scale: f64, depth: f64, tag: &'static str) -> usize {
        let t = self.text_node(font, size, text, x, y, depth, ox, oy, scale);
        { let n = self.node(t); n.parent = Some(root); n.tag = tag; }
        t
    }

    pub(crate) fn pa_close(&mut self, root: usize, x: f64, y: f64, scale: f64, hit: f64, action: Action, depth: f64, tag: &'static str) -> usize {
        let c = self.pa_img(root, "GJ_WebSheet", "GJ_closeBtn_001.png", x, y, scale, depth, tag);
        self.node(c).hit_scale = hit;
        self.make_bouncy(c, scale, Cond::Always, action);
        c
    }

    /// green nine-slice button with a label, scaled as one group
    pub(crate) fn pa_btn(&mut self, root: usize, frame: &str, x: f64, y: f64, w: f64, h: f64, label: &str, font: &str, fs: f64, lx: f64, ly: f64, action: Action, depth: f64, tag: &'static str) -> usize {
        let tex = self.sizes.get(&format!("/{}", frame)).map_or(80.0, |s| s.0);
        let zone = self.shape(Kind::Rect, x, y, w, h, 0, 0.0, depth, tag);
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = h; n.parent = Some(root); }
        let nine = self.shape(Kind::Nine, 0.0, 0.0, w, h, 0, 1.0, depth, tag);
        { let n = self.node(nine); n.frame = frame.into(); n.radius = (tex * 0.3).min(w / 2.0).min(h / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
        let l = self.pa_txt(zone, font, fs, label, lx, ly, 0.5, 0.5, 1.0, depth + 0.01, tag);
        let _ = l;
        self.make_bouncy(zone, 1.0, Cond::Always, action);
        zone
    }

    pub(crate) fn pa_table(&mut self, root: usize, a: f64, depth: f64, tag: &'static str) {
        let (pw, ph) = (712.0, 460.0);
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = if side_h > 0.0 { ph / side_h } else { 1.0 };
        let l = self.pa_img(root, "GJ_WebSheet", "GJ_table_side_001.png", a - 40.0, 80.0, 1.0, depth, tag); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; }
        let r = self.pa_img(root, "GJ_WebSheet", "GJ_table_side_001.png", a + pw + 40.0, 80.0, 1.0, depth, tag); { let n = self.node(r); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; }
        self.pa_img(root, "GJ_WebSheet", "GJ_table_top_001.png", a + 356.0, 70.0, 1.0, depth, tag);
        self.pa_img(root, "GJ_WebSheet", "GJ_table_bottom_001.png", a + 356.0, 560.0, 1.0, depth, tag);
    }

    // ---- How To Play (`_buildHowToPlayPopup`, depth 300) ----
    pub(crate) fn pa_how_to_play(&mut self) {
        if self.pa.htp_open { return; }
        self.pa.htp_open = true; self.pa.htp_page = 0;
        let root = self.pa_popup("pa_htp", 300.0, 100.0 / 255.0, true);
        self.pa_nine(root, "GJ_square01", 0.0, 0.0, 830.0, 530.0, 300.1, "pa_htp");
        self.pa_close(root, -400.0, 172.0 - 320.0 - 110.0, 0.8, 2.0, Action::Pop("HowToPlay", 0), 300.2, "pa_htp");
        self.pa_txt(root, "bigFont", 62.0, "How To Play", 0.0, -210.0, 0.5, 0.5, 1.0, 300.2, "pa_htp");
        self.pa.htp_root = root;
        self.pa_htp_lines();
        let img = self.pa_img(root, "", "tutorial_01", -90.0, 155.0, 1.0, 300.2, "pa_htp");
        self.pa.htp_img = img;
        let btn = self.pa_btn(root, "GJ_button01", 310.0, 165.0, 125.0, 80.0, "Next", "bigFont", 35.0, -5.0, -2.5, Action::Pop("HowToPlay", 1), 300.3, "pa_htp");
        // the label is the zone's second child
        let lbl = (0..self.nodes.len()).rev().find(|&i| self.get(i).map_or(false, |n| n.parent == Some(btn) && n.kind == Kind::Text)).unwrap_or(0);
        self.pa.htp_label = lbl;
    }

    pub(crate) fn pa_htp_lines(&mut self) {
        self.kill_tag("pa_htpd");
        let root = self.pa.htp_root;
        let (fs, lines) = TUTORIAL[self.pa.htp_page];
        let font = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let line_h = fs * 1.35;
        let start = -195.0 + fs / 0.5;
        for (i, segs) in lines.iter().enumerate() {
            let y = start + i as f64 * line_h;
            if segs.len() == 1 && segs[0].1.is_none() {
                self.pa_txt(root, "bigFont", fs, segs[0].0, 0.0, y, 0.5, 0.5, 1.0, 300.2, "pa_htpd");
                continue;
            }
            let widths: Vec<f64> = segs.iter().map(|s| font.width(s.0, fs)).collect();
            let mut x = -widths.iter().sum::<f64>() / 2.0;
            for (k, s) in segs.iter().enumerate() {
                let t = self.pa_txt(root, "bigFont", fs, s.0, x, y, 0.0, 0.5, 1.0, 300.2, "pa_htpd");
                if let Some(c) = s.1 { self.node(t).color = c; }
                x += widths[k];
            }
        }
    }

    pub(crate) fn pa_htp_next(&mut self) {
        if self.pa.htp_page >= TUTORIAL.len() - 1 { self.pa_action("HowToPlay", 0); return; }
        self.pa.htp_page += 1;
        self.pa_htp_lines();
        let (img, lbl, p) = (self.pa.htp_img, self.pa.htp_label, self.pa.htp_page);
        let name = format!("tutorial_0{}", p + 1);
        let (w, h) = self.sizes.get(&format!("/{}", name)).copied().unwrap_or((0.0, 0.0));
        if let Some(n) = self.nodes.get_mut(img).and_then(|n| n.as_mut()) { n.frame = name; n.width = w; n.height = h; }
        if p >= TUTORIAL.len() - 1 {
            let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
            if let Some(n) = self.nodes.get_mut(lbl).and_then(|n| n.as_mut()) { n.text = "Exit".into(); n.width = f.width("Exit", 35.0); }
        }
    }

    // ---- Newgrounds (`_buildNewgroundsPopup`, depth 1000) ----
    pub(crate) fn pa_newgrounds(&mut self) {
        if self.pa.ng_open { return; }
        self.pa.ng_open = true;
        let root = self.pa_popup("pa_ng", 1000.0, 100.0 / 255.0, true);
        self.pa_nine(root, "square01_001", 0.0, 0.0, 460.0, 240.0, 1000.1, "pa_ng");
        self.pa_txt(root, "goldFont", 40.0, "Newgrounds", 0.0, -76.0, 0.5, 0.5, 1.0, 1000.2, "pa_ng");
        // 25px Arial, two lines centred on y = -10
        self.pa_txt(root, "arial", 25.0, "Visit Newgrounds to find awesome", 0.0, -24.5, 0.5, 0.5, 1.0, 1000.2, "pa_ng");
        self.pa_txt(root, "arial", 25.0, "music?", 0.0, 4.5, 0.5, 0.5, 1.0, 1000.2, "pa_ng");
        self.pa_btn(root, "GJ_button01", -70.0, 65.0, 165.0, 55.0, "Cancel", "goldFont", 38.0, -2.0, -3.0, Action::Pop("Newgrounds", 0), 1000.3, "pa_ng");
        self.pa_btn(root, "GJ_button01", 90.0, 65.0, 125.0, 55.0, "Open", "goldFont", 39.0, -2.0, -3.0, Action::Pop("Newgrounds", 1), 1000.3, "pa_ng");
    }

    // ---- Credits (`_buildInfoPopup`, depth 1000) ----
    pub(crate) fn pa_credits(&mut self) {
        if self.pa.cr_open { return; }
        self.pa.cr_open = true;
        let root = self.pa_popup("pa_cr", 1000.0, 100.0 / 255.0, true);
        self.pa_nine(root, "GJ_square02", 0.0, 0.0, 480.0, 336.0, 1000.1, "pa_cr");
        self.pa_close(root, -220.0, -148.0, 0.8, 2.0, Action::Pop("Credits", 0), 1000.2, "pa_cr");
        self.pa_txt(root, "bigFont", 42.0, "Credits", 0.0, -124.0, 0.5, 0.6, 1.0, 1000.2, "pa_cr");
        let (aw, ah, ay) = (420.0, 230.0, 20.0);
        let fr = self.shape(Kind::RoundRect, -aw / 2.0, ay - ah / 2.0, aw, ah, 0x000000, 0.18, 1000.3, "pa_cr");
        { let n = self.node(fr); n.radius = 8.0; n.parent = Some(root); }
        // (text, scale, font)
        let entries: [(&str, f64, &str); 14] = [
            ("Made by RobTop Games", 0.8, "goldFont"), ("Lead Developers:", 0.9, "bigFont"), ("breadbb, PinkDev, rohanis0000,", 0.7, "goldFont"), ("Lasokar, and aloaf", 0.7, "goldFont"),
            ("Former Developers:", 0.9, "bigFont"), ("bog, AntiMatter, and arbstro", 0.7, "goldFont"), ("Contributors:", 0.9, "bigFont"), ("t0nchi7, Itzar, zainojdaf,", 0.7, "goldFont"),
            ("Ameth7st, and CoraBitz", 0.7, "goldFont"), ("we love you cora <3", 0.4, "bigFont"), ("Playtesters:", 0.9, "bigFont"), ("Ameth7st, Altruist,", 0.7, "goldFont"),
            ("and Piefaceluke", 0.7, "goldFont"), ("© 2026 RobTop Games. All rights reserved.", 0.4, "Arial"),
        ];
        let total: f64 = entries.iter().map(|e| (32.0 * e.1).round() + 10.0).sum();
        let (content, _si) = self.scroll_area(root, 0.0, ay, aw, ah, total, 1000.4, "pa_cr");
        let mut y = 0.0;
        for (t, sc, font) in entries {
            let node = if font == "Arial" { self.pa_txt(content, "arial", (32.0 * sc).round(), t, 0.0, y, 0.5, 0.0, 1.0, 1000.42, "pa_cr") }
                       else { self.pa_txt(content, font, 32.0, t, 0.0, y, 0.5, 0.0, sc, 1000.42, "pa_cr") };
            let _ = node;
            y += (32.0 * sc).round() + 10.0;
        }
    }

    // ---- News (`_buildNewsPopup`, depth 1000) ----
    pub(crate) fn pa_news(&mut self) {
        if self.pa.news_open { return; }
        self.pa.news_open = true;
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let (pw, ph) = (900.0, 550.0);
        let root = self.pa_popup("pa_news", 1000.0, 100.0 / 255.0, false);
        self.pa_nine(root, "GJ_square01", cx, 320.0, pw, ph, 1000.1, "pa_news");
        let (nl, nr, nt, nb) = (cx - pw / 2.0, cx + pw / 2.0, 320.0 - ph / 2.0, 320.0 + ph / 2.0);
        for (x, y, ox, oy, fx, fy) in [(nl, nt, 0.0, 0.0, false, true), (nr, nt, 1.0, 0.0, true, true), (nl, nb, 0.0, 1.0, false, false), (nr, nb, 1.0, 1.0, true, false)] {
            let c = self.pa_img(root, "GJ_GameSheet03", "dailyLevelCorner_001.png", x, y, 1.0, 1000.2, "pa_news");
            { let n = self.node(c); n.ox = ox; n.oy = oy; n.flip_x = fx; n.flip_y = fy; }
        }
        self.pa_txt(root, "goldFont", 52.0, "News", cx, nt + 42.0, 0.5, 0.5, 1.0, 1000.3, "pa_news");
        let label = self.pa_txt(root, "goldFont", 26.0, &format!("1 of {}", NEWS.len()), nr - 90.0, nt + 62.0, 1.0, 0.5, 1.0, 1000.3, "pa_news");
        self.pa.news_label = label;
        for (x, flip, dir) in [(nl - 40.0, false, 0usize), (nr + 40.0, true, 1usize)] {
            let a = self.pa_img(root, "GJ_GameSheet03", "GJ_arrow_01_001.png", x, 320.0, 1.0, 1000.3, "pa_news");
            self.node(a).flip_x = flip;
            self.make_bouncy(a, 1.0, Cond::Always, Action::Pop("NewsPage", dir));
        }
        let (box_l, box_r, box_t, box_b) = (nl + 80.0, nr - 80.0, nt + 90.0, nb - 22.0);
        let (box_w, box_h) = (box_r - box_l, box_b - box_t);
        let bb = 4.0;
        let g1 = self.shape(Kind::RoundRect, box_l, box_t, box_w, box_h, 0x8c3b15, 1.0, 1000.4, "pa_news"); { let n = self.node(g1); n.radius = 20.0; n.parent = Some(root); }
        let g2 = self.shape(Kind::RoundRect, box_l + bb, box_t + bb, box_w - bb * 2.0, box_h - bb * 2.0, 0x844022, 1.0, 1000.41, "pa_news"); { let n = self.node(g2); n.radius = 16.0; n.parent = Some(root); }
        let (view_t, view_h) = (box_t + bb, box_h - bb * 2.0);
        self.pa.news_root = root; self.pa.news_page = 0;
        self.pa.news_geom = (box_l, box_w, view_t, view_h, bb);
        // "N of M" label created above as the 5th pa_news node: find it again by tag order
        self.pa_news_render();
        let (track_x, track_w, pad) = (box_r - bb - 14.0, 9.0, 4.0);
        let track_h = view_h - pad * 2.0;
        let thumb_h = track_h * 0.8;
        let th = self.shape(Kind::Rect, track_x - track_w / 2.0, view_t + pad, track_w, thumb_h, 0xffffff, 0.22, 1000.7, "pa_news");
        { let n = self.node(th); n.parent = Some(root); }
        self.pa.news_thumb = th; self.pa.news_view = (view_t, view_h);
        let _ = sw;
        self.pa_close(root, nl + 20.0, nt + 20.0, 0.8, 2.0, Action::Pop("News", 0), 1000.8, "pa_news");
    }


    /// `_newsRenderPage(index)`: date, title and notes of one news entry in the scroll area
    pub(crate) fn pa_news_render(&mut self) {
        self.kill_tag("pa_news_c");
        let (root, page) = (self.pa.news_root, self.pa.news_page);
        let (box_l, box_w, view_t, view_h, bb) = self.pa.news_geom;
        let label = self.pa.news_label;
        if let Some(n) = self.nodes.get_mut(label).and_then(|n| n.as_mut()) { n.text = format!("{} of {}", page + 1, NEWS.len()); }
        let (date, title, notes) = NEWS[page];
        let pad_x = 30.0;
        let text_x = box_l + bb + pad_x;
        let text_w = box_w - bb * 2.0 - pad_x - 44.0;
        let gold = self.fonts.get("goldFont").cloned().unwrap_or_default();
        let big = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let mut y = view_t + 22.0;
        let mut items: Vec<(&str, f64, String, f64)> = Vec::new(); // (font, size, text, abs y)
        items.push(("goldFont", 25.0, date.into(), y)); y += gold.height(25.0) + 22.0;
        items.push(("bigFont", 44.0, title.into(), y)); y += big.height(44.0) + 22.0;
        for note in notes {
            let lines = wrap_arial(note, 20.0, text_w);
            // Phaser: 23px line height + 6px line spacing
            for (i, l) in lines.iter().enumerate() { items.push(("arial", 20.0, l.clone(), y + i as f64 * 29.0)); }
            y += lines.len() as f64 * 23.0 + (lines.len() as f64 - 1.0) * 6.0 + 18.0;
        }
        let content_height = y + 20.0 - view_t;
        let (content, si) = self.scroll_area(root, box_l + bb + (box_w - bb * 2.0) / 2.0, view_t + view_h / 2.0, box_w - bb * 2.0, view_h, content_height - 16.0, 1000.5, "pa_news_c");
        let base = view_t + 8.0;
        for (font, size, t, ay) in items {
            self.pa_txt(content, font, size, &t, text_x, ay - base, 0.0, 0.0, 1.0, 1000.52, "pa_news_c");
        }
        self.pa.news_si = si;
    }

    // ---- Help / Support (`_buildhelppopup`) ----
    pub(crate) fn pa_help(&mut self) {
        if self.pa.help_open { return; }
        self.pa.help_open = true; self.pa.help_closing = false; self.pa.help_clock = 0.0;
        self.pa_pause_btn(0.0);
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let ov = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.0, 200.0, "pa_help");
        { let n = self.node(ov); n.interactive = true; n.width = sw; n.height = 640.0; }
        self.tween_to(ov, &[(Prop::Alpha, 180.0 / 255.0)], 250.0, Ease::Linear, Then::Nothing);
        self.pa.help_overlay = ov;
        let root = self.shape(Kind::Rect, 0.0, -640.0, 0.0, 0.0, 0, 0.0, 201.0, "pa_help");
        self.pa.help_root = root;
        let d = 201.0;
        let a = (sw - 712.0) / 2.0;
        self.pa_txt(root, "bigFont", 55.0, "Support", cx, 65.0, 0.5, 0.5, 1.0, d, "pa_help");
        let p = self.shape(Kind::Rect, a, 80.0, 712.0, 460.0, 0, 180.0 / 255.0, d, "pa_help"); self.node(p).parent = Some(root);
        self.pa_table(root, a, d, "pa_help");
        self.pa_txt(root, "bigFont", 55.0, "Support", cx, 65.0, 0.5, 0.5, 1.0, d, "pa_help");
        for dx in [-312.0, 312.0] { let c = self.pa_img(root, "GJ_WebSheet", "chain_01_001.png", cx + dx, 35.0, 1.0, d, "pa_help"); self.node(c).oy = 1.0; }
        self.pa_img(root, "GJ_GameSheet03", "developedBy_001.png", a + 258.0, 285.0, 1.0, d, "pa_help");
        let rt = self.pa_img(root, "GJ_GameSheet03", "robtoplogo_small.png", a + 440.0, 285.0, 0.95, d, "pa_help");
        self.make_bouncy(rt, 0.95, Cond::Always, Action::OpenUrl("https://geometrydash.com"));
        self.pa_img(root, "GJ_GameSheet03", "poweredBy_001.png", a + 250.0, 375.0, 1.0, d, "pa_help");
        // "Powered By": the Rust logo takes the Phaser logo's place
        let ph = self.pa_img(root, "", "RustLogo", a + 432.0, 380.0, 0.4, d, "pa_help");
        self.make_bouncy(ph, 0.4, Cond::Always, Action::OpenUrl("https://www.rust-lang.org/"));
        self.pa_txt(root, "goldFont", 22.0, "Web Dashers not associated with RobTop Games", cx, 440.0, 0.5, 0.5, 1.0, d, "pa_help");
        let (bw, bh, gap) = (195.0, 70.0, 52.5);
        for (x, label, arg) in [(cx - bw - gap / 2.0, "Links", 1), (cx, "Contact", 2), (cx + bw + gap / 2.0, "Notes", 3)] {
            self.pa_btn(root, "GJ_button01", x, 180.0, bw, bh, label, "bigFont", 40.0, 0.0, -5.0, Action::Pop("Help", arg), d, "pa_help");
        }
        let ldm = self.opts.get("enableLDM").copied().unwrap_or(0.0) != 0.0;
        let lx = a + 160.0;
        let ck = self.pa_img(root, "GJ_GameSheet03", if ldm { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, lx - 120.0, 495.0, 0.8, d, "pa_help");
        self.make_bouncy(ck, 0.8, Cond::Always, Action::Pop("Help", 4));
        self.pa.help_check = ck;
        self.pa_txt(root, "bigFont", 25.0, "Low Detail Mode", lx - 80.0, 495.0, 0.0, 0.5, 1.0, d, "pa_help");
        self.pa_btn(root, "GJ_button04", a + 655.0, 495.0, 80.0, 55.0, "Req", "bigFont", 28.0, 0.0, -2.0, Action::Pop("Help", 5), d, "pa_help");
        let back = self.pa_img(root, "GJ_GameSheet03", "GJ_arrow_03_001.png", cx - 535.0, 30.0, 1.0, d, "pa_help");
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("Help", 0));
    }

    pub(crate) fn pa_close_help(&mut self) {
        if !self.pa.help_open || self.pa.help_closing { return; }
        self.pa.help_closing = true; self.pa.help_clock = 0.0;
        let ov = self.pa.help_overlay;
        self.tween_to(ov, &[(Prop::Alpha, 0.0)], 250.0, Ease::Linear, Then::Nothing);
    }

    // ---- Songs / Soundtrack (`_buildsongspopup`) ----
    pub(crate) fn pa_songs(&mut self) {
        if self.pa.songs_open { return; }
        self.pa.songs_open = true; self.pa.songs_clock = 0.0;
        self.pa_pause_btn(0.0);
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let ov = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.0, 200.0, "pa_songs");
        { let n = self.node(ov); n.interactive = true; n.width = sw; n.height = 640.0; }
        self.tween_to(ov, &[(Prop::Alpha, 100.0 / 255.0)], 1000.0, Ease::Linear, Then::Nothing);
        self.pa.songs_overlay = ov;
        let root = self.shape(Kind::Rect, 0.0, -640.0, 0.0, 0.0, 0, 0.0, 201.0, "pa_songs");
        self.pa.songs_root = root;
        let d = 201.0;
        let a = (sw - 712.0) / 2.0;
        let p = self.shape(Kind::Rect, a, 80.0, 712.0, 460.0, 0xac531e, 1.0, d, "pa_songs"); self.node(p).parent = Some(root);
        self.pa_table(root, a, d, "pa_songs");
        self.pa_txt(root, "bigFont", 55.0, "Soundtrack", cx, 65.0, 0.5, 0.5, 1.0, d, "pa_songs");
        self.pa_txt(root, "bigFont", 42.0, "Nothing here yet... Sorry :(", cx, 300.0, 0.5, 0.5, 1.0, d, "pa_songs");
        for dx in [-312.0, 312.0] { let c = self.pa_img(root, "GJ_WebSheet", "chain_01_001.png", cx + dx, 35.0, 1.0, d, "pa_songs"); self.node(c).oy = 1.0; }
        let back = self.pa_img(root, "GJ_GameSheet03", "GJ_arrow_03_001.png", cx - 535.0, 30.0, 1.0, d, "pa_songs");
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("Songs", 0));
    }

    // ---- Req / Contact / Notes popups (`_showReqpopup`, `_showcontactpopup`, `_showwippopup`; depth 1000) ----
    pub(crate) fn pa_req(&mut self, kind: u8) {
        if self.pa.req_open { return; }
        self.pa.req_open = true; self.pa.req_kind = kind; self.pa.req_clock = 0.0; self.pa.req_shown = false;
        let root = self.pa_popup("pa_req", 1000.0, 100.0 / 255.0, true);
        let d = 1000.2;
        match kind {
            0 => {
                let (w, h) = (475.0, 230.0);
                self.pa_nine(root, "GJ_square02", 0.0, 0.0, w, h, 1000.1, "pa_req");
                self.pa_close(root, -w / 2.0 + 20.0, -h / 2.0 + 20.0, 0.95, 1.0, Action::Pop("Req", 0), d, "pa_req");
                let sp = self.pa_img(root, "", "loadingCircle", 0.0, -10.0, 1.0, d, "pa_req");
                { let n = self.node(sp); n.additive = true; n.alpha = 0.5; }
                self.pa.req_spin = sp;
                let t = self.pa_txt(root, "arial", 26.0, " Loading...", 0.0, h / 2.0 - 40.0, 0.5, 0.5, 1.0, d, "pa_req");
                self.pa.req_text = t;
            }
            1 => {
                let w = 500.0;
                let panel = self.pa_nine(root, "GJ_square02", 0.0, 0.0, w, 250.0, 1000.1, "pa_req");
                self.pa.req_panel = panel;
                self.pa.req_close = self.pa_close(root, -w / 2.0 + 20.0, -105.0, 1.0, 1.0, Action::Pop("Req", 0), d, "pa_req");
                let rows: [(&str, f64, f64, i64); 6] = [("Or, call 480-957-8838 for more info.", 35.0, 26.0, WHITE), ("Message pinkdevyt on Discord,", -5.0, 26.0, WHITE), ("Message rohanis0000 on Discord,", -45.0, 26.0, WHITE),
                    ("Message lasokar on Discord,", -85.0, 26.0, WHITE), ("(Pinkdev's real phone number btw)", 75.0, 17.0, 0x92a7c0), ("...because I created this menu.", 25.0, 17.0, 0x92a7c0)];
                for (i, (t, y, fs, c)) in rows.iter().enumerate() {
                    let n = self.pa_txt(root, "arial", *fs, t, 0.0, *y, 0.5, 0.5, 1.0, d, "pa_req");
                    self.node(n).color = *c;
                    if i == 5 { self.node(n).visible = false; }
                    self.pa.req_texts[i] = n;
                }
                let lock = self.pa_img(root, "GJ_GameSheet03", "GJ_lockGray_001.png", 0.0, 110.0, 1.0, d, "pa_req");
                self.node(lock).hit_scale = 1.5;
                self.make_bouncy(lock, 1.0, Cond::Always, Action::Pop("Req", 1));
                self.pa.req_lock = lock;
            }
            _ => {
                let (w, h) = (500.0, 200.0);
                self.pa_nine(root, "GJ_square02", 0.0, 0.0, w, h, 1000.1, "pa_req");
                self.pa_close(root, -w / 2.0 + 20.0, -h / 2.0 + 20.0, 1.0, 1.0, Action::Pop("Req", 0), d, "pa_req");
                self.pa_img(root, "GJ_GameSheet03", "GJ_timeIcon_001.png", 0.0, 100.0, 1.0, d, "pa_req");
                let rows: [(&str, f64, f64, i64); 3] = [("Coming in another pr later!-", -50.0, 22.0, 0x92a7c0), ("This is still being worked on.", 0.0, 26.0, WHITE), ("Expect this to be finished with particles.", 50.0, 17.0, 0x92a7c0)];
                for (t, y, fs, c) in rows { let n = self.pa_txt(root, "arial", fs, t, 0.0, y, 0.5, 0.5, 1.0, d, "pa_req"); self.node(n).color = c; }
            }
        }
    }

    pub(crate) fn pa_contact_toggle(&mut self) {
        self.pa.req_shown = !self.pa.req_shown;
        let shown = self.pa.req_shown;
        let h = if shown { 190.0 } else { 250.0 };
        let t = self.pa.req_texts;
        let (panel, close, lock) = (self.pa.req_panel, self.pa.req_close, self.pa.req_lock);
        if let Some(n) = self.nodes.get_mut(panel).and_then(|n| n.as_mut()) { n.h = h; let tex = 80.0f64; let _ = tex; n.radius = n.radius.min(h / 2.0); }
        if let Some(n) = self.nodes.get_mut(close).and_then(|n| n.as_mut()) { n.y = -h / 2.0 + 20.0; }
        if let Some(n) = self.nodes.get_mut(lock).and_then(|n| n.as_mut()) { n.y = if shown { 80.0 } else { 110.0 }; }
        if let Some(n) = self.nodes.get_mut(t[0]).and_then(|n| n.as_mut()) {
            n.text = if shown { "Orrrr... Message ameth7st_nya!" } else { "Or, call 480-957-8838 for more info." }.into();
            n.y = if shown { -10.0 } else { 35.0 };
        }
        for i in 1..5 { if let Some(n) = self.nodes.get_mut(t[i]).and_then(|n| n.as_mut()) { n.visible = !shown; } }
        if let Some(n) = self.nodes.get_mut(t[5]).and_then(|n| n.as_mut()) { n.visible = shown; }
    }

    /// per-frame work: delayed opens, the slide-in/out of Help and Songs, the loading spinner, the news scrollbar
    pub fn pa_step(&mut self, dt: f64) {
        if let Some((name, t)) = self.pa.pending {
            // the settings screen has to be gone first, then 150 ms
            if !self.set_open {
                let t = t + dt;
                if t >= 150.0 { self.pa.pending = None; if name == "Help" { self.pa_help(); } else { self.pa_songs(); } }
                else { self.pa.pending = Some((name, t)); }
            }
        }
        if self.pa.help_open {
            self.pa.help_clock += dt;
            let t = (self.pa.help_clock / 450.0).min(1.0);
            let p = if self.pa.help_closing { 1.0 - ease(Ease::QuadIn, t) } else { ease(Ease::QuadOut, t) };
            let r = self.pa.help_root;
            if let Some(n) = self.nodes.get_mut(r).and_then(|n| n.as_mut()) { n.y = p * 650.0 - 640.0; }
            if self.pa.help_closing && t >= 1.0 {
                self.kill_tag("pa_help"); self.pa.help_open = false; self.pa.help_closing = false;
                self.pa_pause_btn(1.0);
            }
        }
        if self.pa.songs_open {
            self.pa.songs_clock += dt;
            let p = ease(Ease::QuadOut, (self.pa.songs_clock / 500.0).min(1.0));
            let r = self.pa.songs_root;
            if let Some(n) = self.nodes.get_mut(r).and_then(|n| n.as_mut()) { n.y = p * 650.0 - 640.0; }
        }
        if self.pa.req_open && self.pa.req_kind == 0 {
            let before = self.pa.req_clock;
            self.pa.req_clock += dt;
            let sp = self.pa.req_spin;
            if before < 5000.0 {
                if self.pa.req_clock < 5000.0 {
                    if let Some(n) = self.nodes.get_mut(sp).and_then(|n| n.as_mut()) { n.rot += 0.1 * dt / 16.0; }
                } else {
                    let (w, h) = self.sizes.get("GJ_GameSheet03/exMark_001.png").copied().unwrap_or((0.0, 0.0));
                    if let Some(n) = self.nodes.get_mut(sp).and_then(|n| n.as_mut()) {
                        n.additive = false; n.alpha = 1.0; n.atlas = "GJ_GameSheet03".into(); n.frame = "exMark_001.png".into(); n.width = w; n.height = h; n.rot = 0.0; n.sx = 1.25; n.sy = 1.25; n.y = -25.0;
                    }
                    let t = self.pa.req_text;
                    if let Some(n) = self.nodes.get_mut(t).and_then(|n| n.as_mut()) { n.text = "Failed. Please try again never. ".into(); }
                }
            }
        }
        if self.pa.news_open {
            let (si, th, (view_t, view_h)) = (self.pa.news_si, self.pa.news_thumb, self.pa.news_view);
            if let Some(sc) = self.scrolls.get(si) {
                let (y, max) = (sc.y, sc.max);
                let (pad, track_h) = (4.0, view_h - 8.0);
                let thumb_h = track_h * 0.8;
                let ty = view_t + pad + if max > 0.0 { (track_h - thumb_h) * (y / max) } else { 0.0 };
                if let Some(n) = self.nodes.get_mut(th).and_then(|n| n.as_mut()) { n.y = ty; }
            }
        }
    }
}
