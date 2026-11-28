//! Popups group B: StartPos, Keybinds, Vault, level Stats popup, Stats screen.
//! Ported from game-scene.js (`_buildStartPosPopup`, `_buildKeybindsPopup`, `_openVaultMenu`, `_buildLevelStatsPopup`, `_showStatsScreen`).
use crate::ui::*;
use std::collections::BTreeMap;

/// delayed work, ticked from `Ui::step` (see `pb_step`)
#[derive(Clone, Copy, Debug, PartialEq)]
enum Timer { VaultBuild, VaultCloseBuild, StatsClosed }

const KB_DEFS: [(&str, &str, &str); 24] = [
    ("jump", "Jump P1", "gameplay"), ("jump2", "Jump P2", "gameplay"), ("pause", "Pause", "gameplay"), ("restart", "Restart", "gameplay"),
    ("Noclip", "Noclip", "gameplay"), ("showHitboxes", "Show Hitboxes", "gameplay"),
    ("practice", "Practice Mode", "practice"), ("startPosPrev", "StartPos Prev", "practice"), ("startPosNext", "StartPos Next", "practice"),
    ("saveCheckpoint", "Place Checkpoint", "practice"), ("deleteCheckpoint", "Delete Checkpoint", "practice"),
    ("editorTab1", "Build Tab", "editor"), ("editorTab2", "Edit Tab", "editor"), ("editorTab3", "Delete Tab", "editor"),
    ("editorMoveUp", "Move Up", "editor"), ("editorMoveLeft", "Move Left", "editor"), ("editorMoveDown", "Move Down", "editor"), ("editorMoveRight", "Move Right", "editor"),
    ("toggleMusic", "Toggle Music", "tools"), ("screenshot", "Screenshot", "tools"), ("debugMode", "Debug Mode", "tools"),
    ("zoomIn", "Zoom In", "view"), ("zoomOut", "Zoom Out", "view"), ("toggleUI", "Toggle UI", "view"),
];

const KB_DEFAULTS: [(&str, &[&str]); 24] = [
    ("jump", &["Space", "KeyG", "KeyL", "KeyW"]), ("jump2", &["ArrowUp", "KeyH"]), ("pause", &[]), ("restart", &["KeyR"]), ("noclip", &["KeyN"]),
    ("showHitboxes", &["KeyH"]), ("practice", &["KeyP"]), ("startPosPrev", &["KeyQ"]), ("startPosNext", &["KeyE"]), ("saveCheckpoint", &["KeyZ"]),
    ("deleteCheckpoint", &["KeyX"]), ("editorTab1", &["Digit1"]), ("editorTab2", &["Digit2"]), ("editorTab3", &["Digit3"]),
    ("editorMoveUp", &["KeyW"]), ("editorMoveLeft", &["KeyA"]), ("editorMoveDown", &["KeyS"]), ("editorMoveRight", &["KeyD"]),
    ("toggleMusic", &["KeyM"]), ("screenshot", &["F2"]), ("debugMode", &["F1"]), ("zoomIn", &["Equal"]), ("zoomOut", &["Minus"]), ("toggleUI", &["KeyU"]),
];

const KB_TABS: [(&str, &str, f64); 3] = [("Gameplay", "gameplay", 150.0), ("Practice", "practice", 140.0), ("Editor", "editor", 120.0)];

/// `window.kbLabel`
pub fn kb_label(code: &str) -> String {
    let names = [("Space", "Space"), ("Enter", "Enter"), ("NumpadEnter", "Num Enter"), ("Tab", "Tab"), ("Backspace", "Bksp"), ("Delete", "Del"), ("Escape", "Esc"),
        ("ArrowUp", "Up"), ("ArrowDown", "Down"), ("ArrowLeft", "Left"), ("ArrowRight", "Right"), ("ShiftLeft", "L Shift"), ("ShiftRight", "R Shift"),
        ("ControlLeft", "L Ctrl"), ("ControlRight", "R Ctrl"), ("AltLeft", "L Alt"), ("AltRight", "R Alt"), ("MetaLeft", "L Win"), ("MetaRight", "R Win"),
        ("Comma", ","), ("Period", "."), ("Slash", "/"), ("Backslash", "\\"), ("IntlBackslash", "\\"), ("Semicolon", ";"), ("Quote", "'"),
        ("BracketLeft", "["), ("BracketRight", "]"), ("Minus", "-"), ("Equal", "="), ("Backquote", "`"), ("NumpadAdd", "Num +"), ("NumpadSubtract", "Num -"),
        ("NumpadMultiply", "Num *"), ("NumpadDivide", "Num /"), ("NumpadDecimal", "Num ."), ("PageUp", "Pg Up"), ("PageDown", "Pg Dn"), ("CapsLock", "Caps"),
        ("ScrollLock", "Scr Lk"), ("NumLock", "Num Lk"), ("PrintScreen", "PrtSc"), ("ContextMenu", "Menu")];
    if let Some((_, v)) = names.iter().find(|(k, _)| *k == code) { return v.to_string(); }
    let b = code.as_bytes();
    if code.len() == 4 && code.starts_with("Key") && b[3].is_ascii_uppercase() { return code[3..].to_string(); }
    if code.len() == 6 && code.starts_with("Digit") && b[5].is_ascii_digit() { return code[5..].to_string(); }
    if code.len() == 7 && code.starts_with("Numpad") && b[6].is_ascii_digit() { return format!("Num {}", &code[6..]); }
    // "$1 $2" between a lower-case and an upper-case letter
    let mut out = String::new();
    let chars: Vec<char> = code.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && chars[i - 1].is_ascii_lowercase() && c.is_ascii_uppercase() { out.push(' '); }
        out.push(*c);
    }
    out
}

/// Helvetica/Arial advance widths (1/1000 em) for 32..=126, to lay out the Arial lines without a canvas
fn arial_width(text: &str, size: f64) -> f64 {
    const W: [u16; 95] = [278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
        1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556,
        333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584];
    text.chars().map(|c| { let u = c as u32; if (32..=126).contains(&u) { W[(u - 32) as usize] as f64 } else { 556.0 } }).sum::<f64>() * size / 1000.0
}

struct Sub { def: usize, ki: i64, pending: Option<String>, key_lbl: usize }

pub struct PopB {
    timers: Vec<(f64, Timer)>,
    rng: u64,
    // StartPos
    pub sp_open: bool,
    pub sp_switcher: bool,
    pub sp_smart: bool,
    pub sp_opacity: f64,
    sp_checks: [usize; 2],
    sp_pct: usize,
    sp_preview: Vec<usize>,
    sp_dirty: bool,
    // Keybinds
    kb_open: bool,
    pub kb_keys: BTreeMap<String, Vec<String>>,
    kb_tab: usize,
    kb_search: bool,
    kb_query: String,
    pub(crate) kb_focus: bool,
    kb_blink: f64,
    kb_cursor_vis: bool,
    kb_defs: Vec<usize>,
    kb_scroll: Option<usize>,
    kb_tabs: Vec<usize>,
    kb_search_nine: usize,
    kb_search_ui: Vec<usize>,
    kb_ph: usize,
    kb_typed: usize,
    kb_cursor: usize,
    kb_thumb: usize,
    kb_off: f64,
    kb_vis: f64,
    kb_sub: Option<Sub>,
    kb_root: usize,
    // Vault
    va_busy: bool,
    va_open: bool,
    va_input: String,
    va_focus: bool,
    va_msg: usize,
    va_alt: Option<(usize, usize)>,
    va_last_alt: i64,
    va_msg_node: usize,
    va_ph: usize,
    va_typed: usize,
    va_cursor: usize,
    va_bar: (f64, f64, f64),
    va_fade: usize,
    // Stats screen
    st_open: bool,
    st_closing: bool,
    st_vals: [String; 7],
    st_nodes: [usize; 7],
    // Level stats popup
    pub ls_name: String,
    pub ls_attempts: u64,
    pub ls_jumps: u64,
    pub ls_best: i64,
    pub ls_practice: i64,
    ls_open: bool,
}

impl Default for PopB {
    fn default() -> PopB {
        let mut kb_keys = BTreeMap::new();
        for (id, keys) in KB_DEFAULTS.iter() { kb_keys.insert(id.to_string(), keys.iter().map(|k| k.to_string()).collect()); }
        PopB {
            timers: Vec::new(), rng: 0x2545F4914F6CDD1D,
            sp_open: false, sp_switcher: false, sp_smart: false, sp_opacity: 1.0, sp_checks: [0; 2], sp_pct: 0, sp_preview: Vec::new(), sp_dirty: false,
            kb_open: false, kb_keys, kb_tab: 0, kb_search: false, kb_query: String::new(), kb_focus: false, kb_blink: 0.0, kb_cursor_vis: false, kb_defs: Vec::new(),
            kb_scroll: None, kb_tabs: Vec::new(), kb_search_nine: 0, kb_search_ui: Vec::new(), kb_ph: 0, kb_typed: 0, kb_cursor: 0, kb_thumb: 0, kb_off: 0.0, kb_vis: 0.0,
            kb_sub: None, kb_root: 0,
            va_busy: false, va_open: false, va_input: String::new(), va_focus: false, va_msg: 0, va_alt: None, va_last_alt: -1, va_msg_node: 0, va_ph: 0, va_typed: 0,
            va_cursor: 0, va_bar: (0.0, 0.0, 0.0), va_fade: 0,
            st_open: false, st_closing: false, st_vals: Default::default(), st_nodes: [0; 7],
            ls_name: String::new(), ls_attempts: 0, ls_jumps: 0, ls_best: 0, ls_practice: 0, ls_open: false,
        }
    }
}

const VAULT_MESSAGES: [&str; 30] = ["How did you get in here?!", "What are you poking around for?", "Don't you have better things to do?", "There is no spoon", "Go away!",
    "I should have hid this room better...", "You're not supposed to be in here...", "RubRub won't like this...", "zzzZZZZ...", "Don't touch that!", "Why U touch my stuff?",
    "RubRub better not find you in here...", "Can't you just leave?", "This is not the room you are looking for...", "Sneaky sneaky...", "It's my precious...",
    "You shall not pass!", "Don't push the button!", "You're gonna get me in trouble...", "This is getting ridiculus...", "Go collect some stars", "Maybe there are new levels?",
    "Just, stop bothering me", "I'm gonna stop talking", "...", "......", "GAH!", "You're hopeless...", "Really, still here?", "Fine, press the button"];
const VAULT_OPENING: [&str; 4] = ["I should have hid this room better...", "RubRub won't like this...", "Don't you have better things to do?", "How did you get in here?!"];
const VAULT_ALT: [&[&str]; 9] = [
    &["Don't you get tired of clicking?", "I mean, isn't it boring?", "Click click click", "Is that all you do?"],
    &["I heard a joke the other day", "Two cubes walk into a bar", "Wait... did they walk or slide?", "I don't think cubes can walk", "Nevermind..."],
    &["Are you always like this?", "I mean, what's the point?", "Let me guess, you're gonna push the button?", "What a surprise..."],
    &["Roses are red", "Violets are blue", "Click the back button", "So I get rid of you"],
    &["So... what's up?", "The back button is in the corner.", "Ready to leave yet?", "Whenever you're ready...", "Fine, I can wait."],
    &["I have an idea for a gamemode", "You play as a cube", "that can jump", "...and there are spikes", "nevermind..."],
    &["Look behind you!", "I thought I saw something", "Maybe it's just my imagination", "Or was it?", "DUN DUN DUN!"],
    &["Hey...", "Who created this menu?", "Maybe it was him.", "Or was it?", "DUN DUN DUN!"],
    &["Don't you get fed up?", "With the lack of updates I mean.", "I mean, arent they lazy?", "Who are they...", "..."],
];

const SEARCH_ALLOWED: &str = " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
const VAULT_ALLOWED: &str = " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const SP_INFO_SWITCHER: &str = "Switches between start positions in a level.";
const SP_INFO_SMART: &str = "This is the info message";

impl Ui {
    // ------------------------------------------------------------------ shared helpers
    fn pbr(&mut self) -> u64 { let mut x = self.pb.rng; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.pb.rng = x; x }
    fn pb_rand(&mut self) -> f64 { (self.pbr() >> 11) as f64 / (1u64 << 53) as f64 }

    /// a full-screen click-blocking dim (2x the screen like the original)
    fn pb_dim(&mut self, tag: &'static str, depth: f64, alpha: f64, action: Action) -> usize {
        let sw = self.screen_width;
        let d = self.shape(Kind::Rect, sw / 2.0 - sw, 320.0 - 640.0, sw * 2.0, 1280.0, 0, alpha, depth, tag);
        { let n = self.node(d); n.interactive = true; n.width = sw * 2.0; n.height = 1280.0; n.action = action; }
        d
    }

    /// the (invisible) container node every child is parented to; sits at the screen centre
    fn pb_root(&mut self, tag: &'static str, depth: f64, scale: f64) -> usize {
        let cx = self.screen_width / 2.0;
        let r = self.shape(Kind::Rect, cx, 320.0, 0.0, 0.0, 0, 0.0, depth, tag);
        { let n = self.node(r); n.sx = scale; n.sy = scale; }
        r
    }

    #[allow(clippy::too_many_arguments)]
    fn pb_nine(&mut self, root: usize, tag: &'static str, depth: f64, frame: &str, x: f64, y: f64, w: f64, h: f64, radius: f64) -> usize {
        let t = self.sizes.get(&format!("/{}", frame)).map_or(80.0, |s| s.0);
        let n = self.shape(Kind::Nine, x, y, w, h, 0, 1.0, depth, tag);
        { let nd = self.node(n); nd.frame = frame.into(); nd.radius = radius.min(t / 2.0).min(w / 2.0); nd.ox = 0.5; nd.oy = 0.5; nd.tint = 0xffffff; nd.parent = Some(root); }
        n
    }

    #[allow(clippy::too_many_arguments)]
    fn pb_text(&mut self, parent: usize, tag: &'static str, depth: f64, font: &str, size: f64, text: &str, x: f64, y: f64, ox: f64, oy: f64, color: i64) -> usize {
        let t = self.text_node(font, size, text, x, y, depth, ox, oy, 1.0);
        let w = if font == "arial" { arial_width(text, size) } else { self.node(t).width };
        let n = self.node(t); n.tag = tag; n.parent = Some(parent); n.color = color; n.width = w;
        t
    }

    #[allow(clippy::too_many_arguments)]
    fn pb_image(&mut self, parent: usize, tag: &'static str, depth: f64, atlas: &str, frame: &str, x: f64, y: f64, scale: f64) -> usize {
        let i = self.image(atlas, frame, x, y, depth, scale);
        { let n = self.node(i); n.tag = tag; n.parent = Some(parent); }
        i
    }

    #[allow(clippy::too_many_arguments)]
    fn pb_rect(&mut self, parent: usize, tag: &'static str, depth: f64, kind: Kind, x: f64, y: f64, w: f64, h: f64, color: i64, alpha: f64, radius: f64) -> usize {
        let r = self.shape(kind, x, y, w, h, color, alpha, depth, tag);
        { let n = self.node(r); n.parent = Some(parent); n.radius = radius; }
        r
    }

    /// bouncy close button used by the popups
    fn pb_close(&mut self, root: usize, tag: &'static str, depth: f64, x: f64, y: f64, scale: f64, action: Action) -> usize {
        let c = self.pb_image(root, tag, depth, "GJ_WebSheet", "GJ_closeBtn_001.png", x, y, scale);
        self.make_bouncy(c, scale, Cond::Always, action);
        c
    }

    /// nine-slice button with a bitmap label that scales as one (`_makeCompositeBouncyButton`); returns (zone, nine, label)
    #[allow(clippy::too_many_arguments)]
    fn pb_btn(&mut self, parent: usize, tag: &'static str, depth: f64, x: f64, y: f64, w: f64, h: f64, tex: &str, border: f64, label: &str, font: &str, size: f64, lx: f64, ly: f64, max_w: Option<f64>, action: Action) -> (usize, usize, usize) {
        let zone = self.pb_rect(parent, tag, depth, Kind::Rect, x, y, w, h, 0, 0.0, 0.0);
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = h; }
        let nine = self.pb_nine(zone, tag, depth, tex, 0.0, 0.0, w, h, border);
        self.node(nine).radius = border;
        let lbl = self.pb_text(zone, tag, depth, font, size, label, lx, ly, 0.5, 0.5, 0xffffff);
        if let Some(m) = max_w { let lw = self.node(lbl).width; if lw > m { let k = m / lw; let n = self.node(lbl); n.sx = k; n.sy = k; } }
        self.make_bouncy(zone, 1.0, Cond::Always, action);
        (zone, nine, lbl)
    }

    fn pb_tex_w(&self, name: &str) -> f64 { self.sizes.get(&format!("/{}", name)).map_or(40.0, |s| s.0) }

    fn pb_hide_pause_btn(&mut self, to: f64, ms: f64) {
        let hud: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "hud_pause")).collect();
        for i in hud { self.tween_to(i, &[(Prop::Alpha, to)], ms, Ease::Linear, Then::Nothing); }
    }

    fn pb_pop_in(&mut self, root: usize) { self.tween_to(root, &[(Prop::ScaleX, 1.0), (Prop::ScaleY, 1.0)], 660.0, Ease::ElasticOut, Then::Nothing); }

    // ------------------------------------------------------------------ entry points
    pub fn pb_open(&mut self, name: &str) -> bool {
        match name {
            "StartPos" => { self.open_startpos(); true }
            "Keys" => { self.open_keybinds(); true }
            "Vault" => { self.open_vault(); true }
            "Stats" => { self.open_stats(); true }
            "LevelStats" => { self.open_level_stats(); true }
            _ => false,
        }
    }

    pub fn pb_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "sp_close" => { self.kill_tag("pb_sp"); self.pb.sp_open = false; self.pb.sp_dirty = false; true }
            "sp_tog" => {
                if arg == 0 { self.pb.sp_switcher = !self.pb.sp_switcher; } else { self.pb.sp_smart = !self.pb.sp_smart; }
                let on = if arg == 0 { self.pb.sp_switcher } else { self.pb.sp_smart };
                let c = self.pb.sp_checks[arg.min(1)];
                if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.frame = if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }.into(); }
                self.events.push(Action::Pop("sp_save", 0));
                true
            }
            "kb_close" => { self.close_keybinds(); true }
            "kb_tab" => { self.pb.kb_search = false; self.pb.kb_tab = arg; self.pb.kb_query.clear(); self.kb_blur(); self.kb_refresh_tabs(); self.kb_build_list(true); true }
            "kb_search" => { if !self.pb.kb_search { self.pb.kb_search = true; self.kb_refresh_tabs(); self.kb_build_list(true); } true }
            "kb_cancel" => { self.pb.kb_query.clear(); self.kb_update_search_display(); self.kb_build_list(true); true }
            "kb_blur" => { self.kb_blur(); true }
            "kb_focus" => { if self.pb.kb_search && !self.pb.kb_focus { self.pb.kb_focus = true; self.pb.kb_cursor_vis = true; self.pb.kb_blink = 0.0; self.kb_update_search_display(); } true }
            "kb_edit" => { let (def, k) = (arg / 16, (arg % 16) as i64 - 1); self.kb_open_sub(def, k); true }
            "kb_sub_close" => { self.kb_close_sub(); true }
            "kb_sub_set" => { self.kb_sub_set(); true }
            "kb_sub_remove" => { self.kb_sub_remove(); true }
            "va_back" => { self.close_vault(); true }
            "va_submit" => { self.va_submit(); true }
            "va_focus" => { self.pb.va_focus = true; self.va_update_input(); true }
            "va_blur" => { if self.pb.va_focus { self.pb.va_focus = false; self.va_update_input(); } true }
            "st_close" => { self.close_stats(); true }
            "ls_close" => { self.kill_tag("pb_ls"); self.pb.ls_open = false; true }
            _ => false,
        }
    }

    /// called every frame from `Ui::step`
    pub fn pb_step(&mut self, dt: f64) {
        if !self.pb.timers.is_empty() {
            for t in self.pb.timers.iter_mut() { t.0 -= dt; }
            let due: Vec<Timer> = self.pb.timers.iter().filter(|t| t.0 <= 0.0).map(|t| t.1).collect();
            self.pb.timers.retain(|t| t.0 > 0.0);
            for t in due { self.pb_timer(t); }
        }
        if self.pb.sp_dirty && self.pointer_down_node.is_none() { self.pb.sp_dirty = false; self.events.push(Action::Pop("sp_save", 0)); }
        if self.pb.kb_open {
            if self.pb.kb_focus && self.pb.kb_search {
                self.pb.kb_blink += dt;
                if self.pb.kb_blink >= 530.0 { self.pb.kb_blink -= 530.0; self.pb.kb_cursor_vis = !self.pb.kb_cursor_vis; self.kb_update_search_display(); }
            }
            self.kb_update_thumb();
        }
    }

    fn pb_timer(&mut self, t: Timer) {
        match t {
            Timer::VaultBuild => { self.build_vault(); self.pb.va_busy = false; self.vault_fade_out(); }
            Timer::VaultCloseBuild => {
                self.kill_tag("pb_va_s");
                self.pb.va_open = false; self.pb.va_busy = false; self.pb.va_focus = false;
                self.events.push(Action::Pop("va_music", 0));
                self.vault_fade_out();
            }
            Timer::StatsClosed => { self.kill_tag("pb_st"); self.pb.st_open = false; self.pb.st_closing = false; self.pb_hide_pause_btn(1.0, 300.0); }
        }
    }

    fn vault_fade_out(&mut self) {
        let f = self.pb.va_fade;
        self.tween_to(f, &[(Prop::Alpha, 0.0)], 150.0, Ease::Linear, Then::Destroy);
    }

    /// slider thumb dragged (which >= 2): `pct` is the thumb position as 0..1
    pub fn pb_drag(&mut self, which: u8, pct: f64) {
        if which != 2 { return; }
        self.pb.sp_opacity = ((0.05 + pct * 0.95) * 100.0).round() / 100.0;
        self.pb_update_opacity();
        self.pb.sp_dirty = true;
    }

    fn pb_update_opacity(&mut self) {
        let o = self.pb.sp_opacity;
        let t = format!("{}%", (o * 100.0).round() as i64);
        let p = self.pb.sp_pct; self.node(p).text = t;
        for i in self.pb.sp_preview.clone() { self.node(i).alpha = o; }
    }

    // ------------------------------------------------------------------ StartPos
    pub fn open_startpos(&mut self) {
        if self.pb.sp_open { return; }
        self.pb.sp_open = true;
        let d = 500.0; let tag = "pb_sp";
        let (pw, ph) = (720.0, 540.0);
        self.pb_dim(tag, d, 150.0 / 255.0, Action::None);
        let root = self.pb_root(tag, d, 1.0);
        let t67 = self.pb_tex_w("GJ_square67");
        self.pb_nine(root, tag, d, "GJ_square67", 0.0, 0.0, pw, ph, 0.325 * t67);
        self.pb_close(root, tag, d, -pw / 2.0 + 10.0, -ph / 2.0 + 10.0, 0.8, Action::Pop("sp_close", 0));
        self.pb_text(root, tag, d, "goldFont", 34.0, "StartPos", 0.0, -ph / 2.0 + 45.0 - 2.0, 0.5, 0.5, 0xffffff);
        let (check_x, label_x) = (-270.0, -220.0);
        for (idx, (y, label, info, on)) in [(-135.0, "StartPos Switcher", SP_INFO_SWITCHER, self.pb.sp_switcher), (-65.0, "Smart StartPos", SP_INFO_SMART, self.pb.sp_smart)].into_iter().enumerate() {
            let check = self.pb_image(root, tag, d, "GJ_GameSheet03", if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, check_x, y, 0.8);
            self.pb_text(root, tag, d, "bigFont", 25.0, label, label_x, y, 0.0, 0.5, 0xffffff);
            let inf = self.pb_image(root, tag, d, "GJ_GameSheet03", "GJ_infoIcon_001.png", check_x - 34.0, y - 30.0, 0.45);
            self.make_bouncy(inf, 0.45, Cond::Always, Action::InfoBox(info));
            self.make_bouncy(check, 0.8, Cond::Always, Action::Pop("sp_tog", idx));
            self.pb.sp_checks[idx] = check;
        }
        let (slider_y, sc) = (15.0, 0.7);
        self.pb_text(root, tag, d, "bigFont", 25.0, "Opacity", label_x - 50.0, slider_y, 0.0, 0.5, 0xffffff);
        let groove_w = self.sizes.get("GJ_WebSheet/slidergroove.png").map_or(420.0, |s| s.0);
        let bar_max = (groove_w - 8.0) * sc;
        let bar_start = -groove_w * sc / 2.0 + 2.8;
        let frac = ((self.pb.sp_opacity - 0.05) / 0.95).max(0.0).min(1.0);
        let fill = self.pb_image(root, tag, d, "", "sliderBar", bar_start, slider_y, 1.0);
        { let n = self.node(fill); n.ox = 0.0; n.oy = 0.5; n.sy = 11.2 / 16.0; n.crop_w = (frac * bar_max).max(1.0); }
        self.pb_image(root, tag, d, "GJ_WebSheet", "slidergroove.png", 0.0, slider_y, sc);
        let thumb = self.pb_image(root, tag, d, "GJ_WebSheet", "sliderthumb.png", bar_start + frac * bar_max, slider_y, sc);
        { let n = self.node(thumb); n.interactive = true; n.drag = Some((bar_start, bar_start + bar_max, fill, 2, 0.0)); }
        let pct = format!("{}%", (self.pb.sp_opacity * 100.0).round() as i64);
        self.pb.sp_pct = self.pb_text(root, tag, d, "bigFont", 25.0, &pct, groove_w * sc / 2.0 + 60.0, slider_y, 0.5, 0.5, 0xffffff);
        let (py, pw2, ph2) = (170.0, 560.0, 150.0);
        self.pb_nine(root, tag, d, "GJ_square02", 0.0, py, pw2, ph2, 16.0);
        self.pb_text(root, tag, d, "goldFont", 22.0, "Preview", -pw2 / 2.0 + 16.0, py - ph2 / 2.0 + 20.0, 0.0, 0.5, 0xffffff);
        let gy = py + 14.0;
        let l = self.pb_image(root, tag, d, "GJ_GameSheet03", "GJ_arrow_01_001.png", -90.0, gy, 0.6);
        let r = self.pb_image(root, tag, d, "GJ_GameSheet03", "GJ_arrow_01_001.png", 90.0, gy, 0.6);
        self.node(r).flip_x = true;
        let t = self.pb_text(root, tag, d, "bigFont", 40.0, "1/3", 0.0, gy, 0.5, 0.5, 0xffffff);
        self.pb.sp_preview = vec![l, r, t];
        let o = self.pb.sp_opacity;
        for i in [l, r, t] { self.node(i).alpha = o; }
    }

    // ------------------------------------------------------------------ Keybinds
    fn kb_keys_of(&self, def: usize) -> Vec<String> { self.pb.kb_keys.get(KB_DEFS[def].0).cloned().unwrap_or_default() }

    fn kb_set_keys(&mut self, def: usize, keys: Vec<String>) {
        self.pb.kb_keys.insert(KB_DEFS[def].0.to_string(), keys);
        self.events.push(Action::Pop("kb_save", 0));
    }

    pub fn open_keybinds(&mut self) {
        if self.pb.kb_open { return; }
        self.pb.kb_open = true;
        self.pb.kb_tab = 0; self.pb.kb_search = false; self.pb.kb_query.clear(); self.pb.kb_focus = false; self.pb.kb_scroll = None; self.pb.kb_sub = None;
        let d = 500.0; let tag = "pb_kb";
        let (pw, ph) = (800.0, 560.0);
        self.pb_dim(tag, d, 150.0 / 255.0, Action::Pop("kb_blur", 0));
        let root = self.pb_root(tag, d, 1.0);
        self.pb.kb_root = root;
        let t67 = self.pb_tex_w("GJ_square67");
        self.pb_nine(root, tag, d, "GJ_square67", 0.0, 0.0, pw, ph, 0.325 * t67);
        // clicking the empty panel drops the search focus (inner.on("pointerdown"))
        let blur = self.pb_rect(root, tag, d, Kind::Rect, 0.0, 0.0, pw, ph, 0, 0.0, 0.0);
        { let n = self.node(blur); n.ox = 0.5; n.oy = 0.5; n.width = pw; n.height = ph; n.interactive = true; n.action = Action::Pop("kb_blur", 0); }
        self.pb_close(root, tag, d, -pw / 2.0 + 10.0, -ph / 2.0 + 10.0, 0.8, Action::Pop("kb_close", 0));

        let (tab_y, tab_h, gap, search_w) = (-ph / 2.0 + 45.0, 55.0, 5.0, 50.0);
        let tab_border = self.pb_tex_w("GJ_button05") * 0.3;
        let group_w: f64 = KB_TABS.iter().map(|t| t.2 + gap).sum::<f64>() + search_w;
        let mut x = -group_w / 2.0;
        self.pb.kb_tabs.clear();
        for (i, (label, _, w)) in KB_TABS.iter().enumerate() {
            let (_, nine, _) = self.pb_btn(root, tag, d, x + w / 2.0, tab_y, *w, tab_h, if i == 0 { "GJ_button02" } else { "GJ_button05" }, tab_border, label, "bigFont", 30.0, 0.0, -2.0, Some(w - 24.0), Action::Pop("kb_tab", i));
            self.pb.kb_tabs.push(nine);
            x += w + gap;
        }
        let zone = self.pb_rect(root, tag, d, Kind::Rect, x + search_w / 2.0, tab_y, search_w, tab_h, 0, 0.0, 0.0);
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = search_w; n.height = tab_h; }
        let nine = self.pb_nine(zone, tag, d, "GJ_button05", 0.0, 0.0, search_w, tab_h, tab_border);
        self.node(nine).radius = tab_border;
        self.pb.kb_search_nine = nine;
        let icon = self.pb_image(zone, tag, d, "GJ_GameSheet04", "searchIcon_001.png", 0.0, -1.0, 1.0);
        let ih = self.sizes.get("GJ_GameSheet04/searchIcon_001.png").map_or(60.0, |s| s.1);
        { let k = (tab_h * 0.6 / ih) * 0.8; let n = self.node(icon); n.sx = k; n.sy = k; }
        self.make_bouncy(zone, 1.0, Cond::Always, Action::Pop("kb_search", 0));

        let (list_top, list_w) = (-ph / 2.0 + 100.0, pw - 60.0);
        // search bar
        let (bar_h, cancel_h) = (70.0, 40.0);
        let by = list_top + bar_h / 2.0;
        let mut ui = Vec::new();
        ui.push(self.pb_rect(root, tag, d, Kind::RoundRect, -list_w / 2.0, by - bar_h / 2.0, list_w, bar_h, 0x252933, 1.0, 10.0));
        let ph_t = self.pb_text(root, tag, d, "bigFont", 28.0, "Search...", 0.0, by, 0.5, 0.5, 0x9a9a9a);
        let typed = self.pb_text(root, tag, d, "bigFont", 28.0, "", 0.0, by, 0.5, 0.5, 0xffffff);
        self.node(typed).visible = false;
        let cur = self.pb_text(root, tag, d, "arial", 30.0, "|", 0.0, by + 1.0, 0.5, 0.5, 0x92a7c0);
        self.node(cur).visible = false;
        let hz = self.pb_rect(root, tag, d, Kind::Rect, 0.0, by, list_w - 90.0, bar_h, 0, 0.0, 0.0);
        { let n = self.node(hz); n.ox = 0.5; n.oy = 0.5; n.width = list_w - 90.0; n.height = bar_h; n.interactive = true; n.action = Action::Pop("kb_focus", 0); }
        let cb = self.pb_image(root, tag, d, "GJ_GameSheet03", "GJ_longBtn07_001.png", list_w / 2.0 - 55.0, by, 1.0);
        let chh = self.sizes.get("GJ_GameSheet03/GJ_longBtn07_001.png").map_or(40.0, |s| s.1);
        let cs = cancel_h / chh;
        { let n = self.node(cb); n.sx = cs; n.sy = cs; }
        self.make_bouncy(cb, cs, Cond::Always, Action::Pop("kb_cancel", 0));
        ui.extend([ph_t, typed, cur, hz, cb]);
        self.pb.kb_ph = ph_t; self.pb.kb_typed = typed; self.pb.kb_cursor = cur;
        self.pb.kb_search_ui = ui;
        let opt = self.pb_image(root, tag, d, "GJ_GameSheet03", "GJ_optionsBtn02_001.png", pw / 2.0 - 45.0, tab_y, 0.8);
        self.lock_gray(opt);
        let th = self.pb_rect(root, tag, d + 0.2, Kind::Rect, list_w / 2.0 - 4.0, list_top, 7.0, 30.0, 0xffffff, 0.45, 0.0);
        { let n = self.node(th); n.ox = 0.5; n.oy = 0.0; }
        self.pb.kb_thumb = th;
        self.kb_refresh_tabs();
        self.kb_build_list(true);
    }

    fn close_keybinds(&mut self) {
        self.kb_close_sub();
        self.kb_unreg_scroll();
        self.kill_tag("pb_kb");
        self.pb.kb_open = false; self.pb.kb_focus = false; self.pb.kb_scroll = None;
    }

    fn kb_unreg_scroll(&mut self) {
        if let Some(si) = self.pb.kb_scroll { if let Some(sc) = self.scrolls.get_mut(si) { sc.zone = usize::MAX; sc.max = 0.0; sc.y = 0.0; } }
    }

    fn kb_refresh_tabs(&mut self) {
        let (search, cur) = (self.pb.kb_search, self.pb.kb_tab);
        for (i, nine) in self.pb.kb_tabs.clone().into_iter().enumerate() {
            self.node(nine).frame = if !search && i == cur { "GJ_button02" } else { "GJ_button05" }.into();
        }
        let sn = self.pb.kb_search_nine;
        self.node(sn).frame = if search { "GJ_button02" } else { "GJ_button05" }.into();
    }

    fn kb_blur(&mut self) {
        if self.pb.kb_focus { self.pb.kb_focus = false; self.kb_update_search_display(); }
    }

    fn kb_update_search_display(&mut self) {
        let (has, focus, vis) = (!self.pb.kb_query.is_empty(), self.pb.kb_focus, self.pb.kb_cursor_vis);
        let (ph, ty, cu) = (self.pb.kb_ph, self.pb.kb_typed, self.pb.kb_cursor);
        let search = self.pb.kb_search;
        self.node(ph).visible = !has && !focus && search;
        let q = self.pb.kb_query.clone();
        let max_w = 740.0 - 220.0;
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let w = f.width(&q, 28.0);
        let k = if w > max_w { max_w / w } else { 1.0 };
        { let n = self.node(ty); n.visible = has && search; n.text = q; n.width = w; n.sx = k; n.sy = k; }
        let tw = if has { w * k } else { 0.0 };
        { let n = self.node(cu); n.x = tw / 2.0 + 4.0; n.visible = focus && vis && search; }
    }

    /// `_buildList`: rebuilds tab/search content; `reset` goes back to the top
    fn kb_build_list(&mut self, reset: bool) {
        self.kill_tag("pb_kbl");
        let root = self.pb.kb_root;
        let (list_top, list_bot, list_w) = (-560.0 / 2.0 + 100.0, 560.0 / 2.0 - 20.0, 740.0);
        let list_h = list_bot - list_top;
        let (row_h, row_gap) = (72.0, 6.0);
        let defs: Vec<usize> = if self.pb.kb_search {
            let q = self.pb.kb_query.trim().to_lowercase();
            if q.is_empty() { vec![] } else { (0..KB_DEFS.len()).filter(|&i| KB_DEFS[i].1.to_lowercase().contains(&q)).collect() }
        } else { (0..KB_DEFS.len()).filter(|&i| KB_DEFS[i].2 == KB_TABS[self.pb.kb_tab].1).collect() };
        self.pb.kb_defs = defs.clone();
        let off = if self.pb.kb_search { 70.0 + 8.0 } else { 0.0 };
        let vis = list_h - off;
        self.pb.kb_off = off; self.pb.kb_vis = vis;
        for i in self.pb.kb_search_ui.clone() { self.node(i).visible = self.pb.kb_search; }
        self.kb_update_search_display();
        let n = defs.len() as f64;
        let total = n * row_h + (n - 1.0).max(0.0) * row_gap;
        let max_scroll = (total - vis).max(0.0);
        let d = 500.0;
        let tag = "pb_kbl";
        // clip rect = the visible list; the rows are drawn between the clip nodes (depth 500.00 .. 500.05)
        let (content, si) = self.scroll_area(root, 0.0, list_top + off + vis / 2.0, list_w, vis, 0.0, d, tag);
        let mut si = si;
        if let Some(old) = self.pb.kb_scroll { if old != si { let e = self.scrolls.pop().unwrap(); self.scrolls[old] = e; si = old; } }
        let zone = self.scrolls[si].zone;
        { let z = self.node(zone); z.scroll = Some(si); z.interactive = false; }
        self.pb.kb_scroll = Some(si);
        let prev_y = if reset { 0.0 } else { self.scrolls[si].y };
        self.node(content).y = 0.0;
        self.scrolls[si].base_y = 0.0; self.scrolls[si].max = max_scroll;
        self.set_scroll(si, prev_y);
        for i in 0..defs.len() {
            let ry = list_top + off + row_h / 2.0 + i as f64 * (row_h + row_gap);
            let col = if i % 2 == 0 { 0x2d313c } else { 0x343946 };
            self.pb_rect(content, tag, d + 0.01, Kind::RoundRect, -list_w / 2.0, ry - row_h / 2.0, list_w, row_h, col, 1.0, 10.0);
        }
        for i in 0..defs.len() { self.kb_build_row(content, i); }
        let th = self.pb.kb_thumb;
        let thumb_h = (vis * (vis / total.max(1.0)).min(1.0)).max(30.0);
        { let t = self.node(th); t.h = thumb_h; t.visible = max_scroll > 0.0; }
        self.kb_update_thumb();
    }

    fn kb_update_thumb(&mut self) {
        let Some(si) = self.pb.kb_scroll else { return };
        let Some(sc) = self.scrolls.get(si) else { return };
        let pct = if sc.max > 0.0 { sc.y / sc.max } else { 0.0 };
        let (vis, off) = (self.pb.kb_vis, self.pb.kb_off);
        let th = self.pb.kb_thumb;
        let h = self.get(th).map_or(30.0, |n| n.h);
        let y = -280.0 + 100.0 + off + pct * (vis - h);
        if let Some(n) = self.nodes.get_mut(th).and_then(|n| n.as_mut()) { n.y = y; }
    }

    fn kb_build_row(&mut self, content: usize, i: usize) {
        let def = self.pb.kb_defs[i];
        let (row_h, row_gap, list_w) = (72.0, 6.0, 740.0);
        let list_top = -280.0 + 100.0;
        let ry = list_top + self.pb.kb_off + row_h / 2.0 + i as f64 * (row_h + row_gap);
        let (d, tag) = (500.02, "pb_kbl");
        let name = self.pb_text(content, tag, d, "bigFont", 28.0, KB_DEFS[def].1, -list_w / 2.0 + 22.0, ry, 0.0, 0.5, 0xffffff);
        { let w = self.node(name).width; if w > 230.0 { let k = 230.0 / w; let n = self.node(name); n.sx = k; n.sy = k; } }
        let keys = self.kb_keys_of(def);
        let (btn_h, plus_w, gap) = (48.0, 58.0, 10.0);
        let b04 = (self.pb_tex_w("GJ_button04") * 0.3).min(btn_h / 2.0 - 2.0);
        let b01 = (self.pb_tex_w("GJ_button01") * 0.3).min(btn_h / 2.0 - 2.0);
        let mut cx = list_w / 2.0 - 16.0;
        let act = if keys.len() < 4 { Action::Pop("kb_edit", i * 16) } else { Action::None };
        self.pb_btn(content, tag, d, cx - plus_w / 2.0, ry, plus_w, btn_h, "GJ_button01", b01, "+", "bigFont", 38.0, -4.0, -2.5, None, act);
        cx -= plus_w + gap;
        for ki in (0..keys.len()).rev() {
            let label = kb_label(&keys[ki]);
            let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
            let lw = f.width(&label, 24.0);
            let k = if lw > 110.0 { 110.0 / lw } else { 1.0 };
            let bw = btn_h.max((lw * k).ceil() + 28.0);
            let bx = cx - bw / 2.0;
            self.pb_btn(content, tag, d, bx, ry, bw, btn_h, "GJ_button04", b04, &label, "bigFont", 24.0, 0.0, -2.0, Some(110.0), Action::Pop("kb_edit", i * 16 + ki + 1));
            cx -= bw + gap;
        }
    }

    fn kb_open_sub(&mut self, i: usize, ki: i64) {
        if self.pb.kb_sub.is_some() { return; }
        let Some(&def) = self.pb.kb_defs.get(i) else { return };
        let (d, tag) = (510.0, "pb_kbsub");
        let (sw_, sh_) = (600.0, 475.0);
        self.pb_dim(tag, d, 150.0 / 255.0, Action::None);
        let root = self.pb_root(tag, d, 1.0);
        let t67 = self.pb_tex_w("GJ_square67");
        self.pb_nine(root, tag, d, "GJ_square67", 0.0, 0.0, sw_, sh_, 0.325 * t67);
        self.pb_close(root, tag, d, -sw_ / 2.0 + 10.0, -sh_ / 2.0 + 10.0, 0.8, Action::Pop("kb_sub_close", 0));
        self.pb_text(root, tag, d, "bigFont", 36.0, "Edit Keybind", 0.0, -sh_ / 2.0 + 30.0, 0.5, 0.5, 0xffffff);
        let name = self.pb_text(root, tag, d, "bigFont", 44.0, KB_DEFS[def].1, 0.0, -136.0, 0.5, 0.5, 0x26cbff);
        { let w = self.node(name).width; let m = sw_ - 140.0; if w > m { let k = m / w; let n = self.node(name); n.sx = k; n.sy = k; } }
        let pending = if ki < 0 { None } else { self.kb_keys_of(def).get(ki as usize).cloned() };
        let key_txt = pending.as_ref().map(|k| kb_label(k)).unwrap_or_default();
        let key_lbl = self.pb_text(root, tag, d, "goldFont", 48.0, &key_txt, 0.0, -54.0, 0.5, 0.5, 0xffffff);
        let hint = self.pb_text(root, tag, d, "bigFont", 32.0, "Press Keys...", 0.0, 25.0, 0.5, 0.5, 0xffffff);
        self.node(hint).alpha = 0.6;
        let (btn_h, by) = (60.0, sh_ / 2.0 - 50.0);
        let is_add = ki < 0;
        let b1 = (self.pb_tex_w("GJ_button01") * 0.3).min(btn_h / 2.0 - 2.0);
        self.pb_btn(root, tag, d, if is_add { 0.0 } else { -110.0 }, by, 150.0, btn_h, "GJ_button01", b1, "Set", "bigFont", 49.0, -5.0, -2.0, None, Action::Pop("kb_sub_set", 0));
        if !is_add {
            let b6 = (self.pb_tex_w("GJ_button06") * 0.3).min(btn_h / 2.0 - 2.0);
            self.pb_btn(root, tag, d, 110.0, by, 210.0, btn_h, "GJ_button06", b6, "Remove", "bigFont", 46.0, -5.0, -2.0, None, Action::Pop("kb_sub_remove", 0));
        }
        self.pb.kb_sub = Some(Sub { def: i, ki, pending, key_lbl });
    }

    fn kb_close_sub(&mut self) { self.kill_tag("pb_kbsub"); self.pb.kb_sub = None; }

    fn kb_sub_set(&mut self) {
        let Some(sub) = self.pb.kb_sub.as_ref() else { return };
        let Some(pending) = sub.pending.clone() else { return };
        let (i, ki) = (sub.def, sub.ki);
        let def = self.pb.kb_defs[i];
        let mut cur = self.kb_keys_of(def);
        if ki < 0 {
            if !cur.contains(&pending) && cur.len() < 4 { cur.push(pending); }
        } else {
            let ki = ki as usize;
            if ki < cur.len() { cur[ki] = pending.clone(); }
            cur = cur.into_iter().enumerate().filter(|(idx, k)| *idx == ki || *k != pending).map(|(_, k)| k).collect();
        }
        self.kb_set_keys(def, cur);
        self.kb_close_sub();
        self.kb_build_list(false);
    }

    fn kb_sub_remove(&mut self) {
        let Some(sub) = self.pb.kb_sub.as_ref() else { return };
        let (i, ki) = (sub.def, sub.ki);
        if ki < 0 { return; }
        let def = self.pb.kb_defs[i];
        let mut cur = self.kb_keys_of(def);
        if (ki as usize) < cur.len() { cur.remove(ki as usize); }
        self.kb_set_keys(def, cur);
        self.kb_close_sub();
        self.kb_build_list(false);
    }

    // ------------------------------------------------------------------ Vault
    pub fn open_vault(&mut self) {
        if self.pb.va_open || self.pb.va_busy { return; }
        self.pb.va_busy = true;
        let (sw, sh) = (self.screen_width, 640.0);
        let f = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 600.0, "pb_va_f");
        self.pb.va_fade = f;
        self.tween_to(f, &[(Prop::Alpha, 1.0)], 150.0, Ease::Linear, Then::Nothing);
        self.pb.timers.push((150.0, Timer::VaultBuild));
    }

    fn close_vault(&mut self) {
        if !self.pb.va_open || self.pb.va_busy { return; }
        self.pb.va_busy = true; self.pb.va_focus = false;
        let (sw, sh) = (self.screen_width, 640.0);
        let f = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 600.0, "pb_va_f");
        self.pb.va_fade = f;
        self.tween_to(f, &[(Prop::Alpha, 1.0)], 150.0, Ease::Linear, Then::Nothing);
        self.pb.timers.push((150.0, Timer::VaultCloseBuild));
    }

    fn build_vault(&mut self) {
        let (sw, sh) = (self.screen_width, 640.0);
        let tag = "pb_va_s";
        self.pb.va_open = true; self.pb.va_input.clear(); self.pb.va_focus = false; self.pb.va_alt = None; self.pb.va_last_alt = -1;
        self.events.push(Action::Pop("va_music", 1));
        // gradient backdrop: 80 bands from 0x004c66 darkening to 44.5%
        let (br, bg, bb) = (0x00 as f64, 0x4c as f64, 0x66 as f64);
        let steps = 80;
        for gi in 0..steps {
            let t = gi as f64 / (steps - 1) as f64;
            let k = 1.0 + (0.445 - 1.0) * t;
            let col = (((br * k).round() as i64) << 16) | (((bg * k).round() as i64) << 8) | ((bb * k).round() as i64);
            let y = (gi as f64 * sh / steps as f64).floor();
            let h = (sh / steps as f64).ceil() + 1.0;
            self.shape(Kind::Rect, 0.0, y, sw, h, col, 1.0, 300.0, tag);
        }
        let blocker = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 301.0, tag);
        { let n = self.node(blocker); n.interactive = true; n.width = sw; n.height = sh; n.action = Action::Pop("va_blur", 0); }
        let back = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", 50.0, 48.0, 304.0, 1.0);
        { let n = self.node(back); n.tag = tag; n.flip_x = true; n.flip_y = true; n.rot = std::f64::consts::PI; }
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("va_back", 0));
        let title = self.text_node("goldFont", 46.0, "The Vault", sw / 2.0, 48.0, 304.0, 0.5, 0.5, 1.0);
        self.node(title).tag = tag;
        let title_h = self.node(title).height;
        let lock = self.image("GJ_GameSheet03", "GJ_secretLock_001.png", sw / 2.0, sh / 2.0, 304.0, 1.0);
        let lock_h = self.node(lock).height;
        let title_bottom = 48.0 + title_h / 2.0;
        let lock_top = sh / 2.0 - lock_h / 2.0;
        let gap = lock_top - title_bottom;
        let (msg_size, msg_gap) = (36.0, 26.0);
        { let n = self.node(lock); n.y += 50.0; n.tag = tag; }
        let bar_w = sw * 0.362;
        let bar_h = 40.0f64.max((sh * 0.125).min(gap - 30.0));
        let stack_h = msg_size + msg_gap + bar_h;
        let mut bar_bottom = title_bottom + (gap - stack_h).max(0.0) / 2.0 + stack_h;
        bar_bottom = bar_bottom.min(lock_top - 12.0);
        let bar_cx = sw / 2.0;
        let bar_base = bar_bottom - bar_h / 2.0;
        let msg_base = (title_bottom + (bar_base - bar_h / 2.0)) / 2.0;
        let bar_cy = bar_base + 70.0;
        let msg_cy = msg_base + 40.0;
        let bar = self.shape(Kind::RoundRect, bar_cx - bar_w / 2.0, bar_cy - bar_h / 2.0, bar_w, bar_h, 0x002632, 1.0, 303.0, tag);
        self.node(bar).radius = 10.0;
        let opening = VAULT_OPENING[(self.pb_rand() * 4.0) as usize % 4];
        self.pb.va_msg = VAULT_MESSAGES.iter().position(|m| *m == opening).unwrap_or(0);
        let m = self.text_node("bigFont", msg_size, opening, bar_cx, msg_cy, 305.0, 0.5, 0.5, 1.0);
        self.node(m).tag = tag;
        self.pb.va_msg_node = m;
        self.va_fit_message();
        let ph = self.text_node("bigFont", 50.0, "...", bar_cx, bar_cy, 305.0, 0.5, 0.5, 1.0);
        { let n = self.node(ph); n.tag = tag; n.color = 0x5a9bdd; }
        let ty = self.text_node("bigFont", 44.0, "", bar_cx, bar_cy, 305.0, 0.5, 0.5, 1.0);
        { let n = self.node(ty); n.tag = tag; n.visible = false; }
        let cu = self.text_node("arial", 33.0, "|", bar_cx, bar_cy, 305.0, 0.5, 0.5, 1.0);
        { let n = self.node(cu); n.tag = tag; n.visible = false; n.color = 0x92a7c0; }
        let hz = self.shape(Kind::Rect, bar_cx, bar_cy, bar_w, bar_h, 0, 0.0, 306.0, tag);
        { let n = self.node(hz); n.ox = 0.5; n.oy = 0.5; n.interactive = true; n.width = bar_w; n.height = bar_h; n.action = Action::Pop("va_focus", 0); }
        self.make_bouncy(lock, 1.0, Cond::Always, Action::Pop("va_submit", 0));
        self.pb.va_ph = ph; self.pb.va_typed = ty; self.pb.va_cursor = cu; self.pb.va_bar = (bar_cx, bar_cy, bar_w);
    }

    fn va_fit_message(&mut self) {
        let m = self.pb.va_msg_node;
        let max_w = self.screen_width * 0.9;
        let w = self.node(m).width;
        let k = if w > max_w { max_w / w } else { 1.0 };
        let n = self.node(m); n.sx = k; n.sy = k;
    }

    fn va_show_message(&mut self, text: &str, alt: bool) {
        let m = self.pb.va_msg_node;
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let w = f.width(text, 36.0);
        { let n = self.node(m); n.text = text.into(); n.width = w; n.color = if alt { 0x00ffc5 } else { 0xffffff }; }
        self.va_fit_message();
    }

    fn va_next_message(&mut self) {
        if let Some((seq, pos)) = self.pb.va_alt {
            let np = pos + 1;
            if np < VAULT_ALT[seq].len() { self.pb.va_alt = Some((seq, np)); self.va_show_message(VAULT_ALT[seq][np], true); return; }
            self.pb.va_alt = None;
        } else if self.pb_rand() < 0.15 {
            let mut idx = self.pb.va_last_alt;
            while idx == self.pb.va_last_alt { idx = (self.pb_rand() * VAULT_ALT.len() as f64) as i64; }
            self.pb.va_last_alt = idx;
            self.pb.va_alt = Some((idx as usize, 0));
            self.va_show_message(VAULT_ALT[idx as usize][0], true);
            return;
        }
        let mut next = self.pb.va_msg;
        while next == self.pb.va_msg { next = (self.pb_rand() * VAULT_MESSAGES.len() as f64) as usize; }
        self.pb.va_msg = next;
        self.va_show_message(VAULT_MESSAGES[next], false);
    }

    fn va_submit(&mut self) { self.pb.va_input.clear(); self.va_update_input(); self.va_next_message(); }

    fn va_update_input(&mut self) {
        let (ph, ty, cu) = (self.pb.va_ph, self.pb.va_typed, self.pb.va_cursor);
        let (bar_cx, _, bar_w) = self.pb.va_bar;
        let text = self.pb.va_input.clone();
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        let w = f.width(&text, 44.0);
        let max = bar_w - 40.0;
        let k = if w > max { max / w } else { 1.0 };
        if text.is_empty() { self.node(ty).visible = false; self.node(ph).visible = true; }
        else { self.node(ph).visible = false; let n = self.node(ty); n.visible = true; n.text = text.clone(); n.width = w; n.sx = k; n.sy = k; }
        let tw = if text.is_empty() { 0.0 } else { w * k };
        let focus = self.pb.va_focus;
        let n = self.node(cu); n.x = bar_cx + tw / 2.0 + 3.0; n.visible = focus;
    }

    // ------------------------------------------------------------------ Stats screen
    /// `jumps, attempts, completed levels, deaths, secret coins, user coins` (the seventh row is "?")
    pub fn pb_set_stats(&mut self, v: [u64; 6]) {
        for (i, x) in v.iter().enumerate() { self.pb.st_vals[i] = x.to_string(); }
        self.pb.st_vals[6] = "?".into();
        if self.pb.st_open { for i in 0..7 { let (n, t) = (self.pb.st_nodes[i], self.pb.st_vals[i].clone()); self.node(n).text = t; } }
    }

    pub fn open_stats(&mut self) {
        if self.pb.st_open { return; }
        self.pb.st_open = true; self.pb.st_closing = false;
        if self.pb.st_vals[6].is_empty() { for i in 0..6 { self.pb.st_vals[i] = if i == 1 { "1".into() } else { "0".into() }; } self.pb.st_vals[6] = "?".into(); }
        self.pb_hide_pause_btn(0.0, 300.0);
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let tag = "pb_st";
        let ov = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.0, 200.0, "pb_st_ov");
        { let n = self.node(ov); n.interactive = true; n.width = sw; n.height = 640.0; }
        self.tween_to(ov, &[(Prop::Alpha, 100.0 / 255.0)], 1000.0, Ease::Linear, Then::Nothing);
        let d = 201.0;
        // the slide-in container: its y goes from -640 to 10 (p * 650 - 640)
        let root = self.shape(Kind::Rect, 0.0, -640.0, 0.0, 0.0, 0, 0.0, d, tag);
        self.tween_to(root, &[(Prop::Y, 10.0)], 500.0, Ease::QuadOut, Then::Nothing);
        let (pw, ph) = (712.0, 460.0);
        let a = (sw - pw) / 2.0;
        self.pb_rect(root, tag, d, Kind::Rect, a, 80.0, pw, ph, 0xac531e, 1.0, 0.0);
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = if side_h > 0.0 { ph / side_h } else { 1.0 };
        let l = self.pb_image(root, tag, d, "GJ_WebSheet", "GJ_table_side_001.png", a - 40.0, 80.0, 1.0); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; }
        let r = self.pb_image(root, tag, d, "GJ_WebSheet", "GJ_table_side_001.png", a + pw + 40.0, 80.0, 1.0); { let n = self.node(r); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; }
        self.pb_image(root, tag, d, "GJ_WebSheet", "GJ_table_top_001.png", a + 356.0, 70.0, 1.0);
        self.pb_image(root, tag, d, "GJ_WebSheet", "GJ_table_bottom_001.png", a + 356.0, 560.0, 1.0);
        for dx in [-312.0, 312.0] { let c = self.pb_image(root, tag, d, "GJ_WebSheet", "chain_01_001.png", cx + dx, 35.0, 1.0); self.node(c).oy = 1.0; }
        self.pb_text(root, tag, d, "bigFont", 55.0, "Stats", cx, 65.0, 0.5, 0.5, 0xffffff);
        let (top, bottom) = (102.0, 528.0);
        let (left, right) = (a + 7.8, a + pw - 7.8);
        let (rw, rows) = (right - left, 7usize);
        let rh = (bottom - top) / rows as f64;
        let labels = ["Total Jumps:", "Total Attempts:", "Completed Levels:", "Total Deaths:", "Total Secret Coins:", "Total User Coins:", "???:"];
        for i in 0..rows {
            let cy = top + i as f64 * rh + rh / 2.0;
            let col = if i % 2 == 0 { 0xac531e } else { 0xcf6d30 };
            self.pb_rect(root, tag, d, Kind::Rect, cx - rw / 2.0, cy - rh / 2.0, rw, rh, col, 1.0, 0.0);
            if i > 0 { self.pb_rect(root, tag, d, Kind::Rect, cx - rw / 2.0, top + i as f64 * rh - 0.25, rw, 0.5, 0, 1.0, 0.0); }
            self.pb_text(root, tag, d, "goldFont", 34.0, labels[i], left + 20.0, cy, 0.0, 0.5, 0xffffff);
            let v = self.pb.st_vals[i].clone();
            self.pb.st_nodes[i] = self.pb_text(root, tag, d, "goldFont", 34.0, &v, right - 20.0, cy, 1.0, 0.5, 0xffffff);
        }
        let back = self.pb_image(root, tag, d, "GJ_GameSheet03", "GJ_arrow_03_001.png", cx - 535.0, 30.0, 1.0);
        self.make_bouncy(back, 1.0, Cond::Always, Action::Pop("st_close", 0));
    }

    fn close_stats(&mut self) {
        if !self.pb.st_open || self.pb.st_closing { return; }
        self.pb.st_closing = true;
        let ids: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "pb_st_ov")).collect();
        for i in ids { self.tween_to(i, &[(Prop::Alpha, 0.0)], 500.0, Ease::Linear, Then::Nothing); }
        let roots: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "pb_st" && n.parent.is_none())).collect();
        for i in roots { self.tween_to(i, &[(Prop::Y, -640.0)], 500.0, Ease::QuadIn, Then::Nothing); }
        self.pb.timers.push((500.0, Timer::StatsClosed));
    }

    // ------------------------------------------------------------------ Level stats popup
    /// Opens the level stats popup for a level (name, attempts, jumps, best normal %, best practice %).
    pub fn pb_open_level_stats(&mut self, name: &str, attempts: u64, jumps: u64, best: i64, practice: i64) {
        self.pb.ls_name = name.into(); self.pb.ls_attempts = attempts; self.pb.ls_jumps = jumps; self.pb.ls_best = best; self.pb.ls_practice = practice;
        self.open_level_stats();
    }

    pub fn open_level_stats(&mut self) {
        if self.pb.ls_open { return; }
        self.pb.ls_open = true;
        let (d, tag) = (700.0, "pb_ls");
        let (pw, ph) = (600.0, 420.0);
        self.pb_dim(tag, d, 150.0 / 255.0, Action::None);
        let root = self.pb_root(tag, d, 0.0);
        self.pb_nine(root, tag, d, "square01_001", 0.0, 0.0, pw, ph, 53.0);
        self.pb_text(root, tag, d, "goldFont", 48.0, "Level Stats", 0.0, -ph / 2.0 + 60.0, 0.5, 0.5, 0xffffff);
        let name = if self.pb.ls_name.is_empty() { "Unnamed".to_string() } else { self.pb.ls_name.clone() };
        let rows: [(String, i64, Option<String>); 5] = [
            (name, 0xffff00, None),
            ("Total Attempts".into(), 0x3cdc50, Some(self.pb.ls_attempts.to_string())),
            ("Total Jumps".into(), 0x5aa0ff, Some(self.pb.ls_jumps.to_string())),
            ("Normal".into(), 0xff00ff, Some(format!("{}%", self.pb.ls_best))),
            ("Practice".into(), 0xffa050, Some(format!("{}%", self.pb.ls_practice))),
        ];
        let (start, gap) = (-ph / 2.0 + 125.0, 42.0);
        for (i, (text, color, value)) in rows.into_iter().enumerate() {
            let y = start + i as f64 * gap;
            match value {
                None => {
                    let t = self.pb_text(root, tag, d, "arial", 30.0, &text, 0.0, y, 0.5, 0.5, color);
                    let w = self.node(t).width;
                    if w > pw - 100.0 { let k = (pw - 100.0) / w; let n = self.node(t); n.sx = k; n.sy = k; }
                }
                Some(v) => {
                    let label = format!("{}:", text);
                    let lv = format!(" {}", v);
                    let (lw, vw) = (arial_width(&label, 30.0), arial_width(&lv, 30.0));
                    let total = lw + vw;
                    self.pb_text(root, tag, d, "arial", 30.0, &label, -total / 2.0, y, 0.0, 0.5, color);
                    self.pb_text(root, tag, d, "arial", 30.0, &lv, -total / 2.0 + lw, y, 0.0, 0.5, 0xffffff);
                }
            }
        }
        self.pb_btn(root, tag, d, 0.0, ph / 2.0 - 55.0, 100.0, 60.0, "GJ_button01", 24.0, "OK", "goldFont", 46.0, -3.0, -4.0, None, Action::Pop("ls_close", 0));
        self.pb_pop_in(root);
    }

    // ------------------------------------------------------------------ keyboard
    /// A key went down while a group-B popup may be open. `code` is `KeyboardEvent.code`, `key` is `KeyboardEvent.key`.
    /// Returns true when the key was consumed.
    pub fn pb_key(&mut self, code: &str, key: &str, ctrl: bool) -> bool {
        if self.pb.kb_open {
            // sub-popup: capture the next key as the binding
            if self.pb.kb_sub.is_some() {
                if code == "Escape" { self.kb_close_sub(); return true; }
                if code.is_empty() || code == "Unidentified" { return true; }
                let lbl = kb_label(code);
                let kl = self.pb.kb_sub.as_ref().unwrap().key_lbl;
                self.node(kl).text = lbl;
                if let Some(s) = self.pb.kb_sub.as_mut() { s.pending = Some(code.to_string()); }
                return true;
            }
            if code == "Escape" { if self.pb.kb_focus { self.kb_blur(); } else { self.close_keybinds(); } return true; }
            if self.pb.kb_search && self.pb.kb_focus {
                if key == "Backspace" { if self.pb.kb_query.pop().is_some() { self.kb_update_search_display(); self.kb_build_list(true); } return true; }
                if !ctrl && key.chars().count() == 1 {
                    let c = key.chars().next().unwrap();
                    if SEARCH_ALLOWED.contains(c) && self.pb.kb_query.chars().count() < 24 { self.pb.kb_query.push(c); self.kb_update_search_display(); self.kb_build_list(true); }
                    return true;
                }
                return ctrl;
            }
            return false;
        }
        if self.pb.va_open && self.pb.va_focus {
            if key == "Backspace" { self.pb.va_input.pop(); self.va_update_input(); }
            else if key == "Enter" { self.va_submit(); }
            else if !ctrl && key.chars().count() == 1 {
                let c = key.chars().next().unwrap();
                if VAULT_ALLOWED.contains(c) && self.pb.va_input.chars().count() < 19 { self.pb.va_input.push(c); self.va_update_input(); }
            }
            return true;
        }
        false
    }

    /// pasted text for the focused search/vault box
    pub fn pb_paste(&mut self, text: &str) {
        if self.pb.kb_open && self.pb.kb_search && self.pb.kb_focus && self.pb.kb_sub.is_none() {
            let f: String = text.chars().filter(|c| SEARCH_ALLOWED.contains(*c)).collect();
            if !f.is_empty() { let space = 24usize.saturating_sub(self.pb.kb_query.chars().count()); self.pb.kb_query.extend(f.chars().take(space)); self.kb_update_search_display(); self.kb_build_list(true); }
        } else if self.pb.va_open && self.pb.va_focus {
            let f: String = text.chars().filter(|c| VAULT_ALLOWED.contains(*c)).collect();
            if !f.is_empty() { let space = 19usize.saturating_sub(self.pb.va_input.chars().count()); self.pb.va_input.extend(f.chars().take(space)); self.va_update_input(); }
        }
    }

    /// the text a copy shortcut should put on the clipboard
    pub fn pb_copy_text(&self) -> String { if self.pb.va_open && self.pb.va_focus { self.pb.va_input.clone() } else { self.pb.kb_query.clone() } }

    /// rebuild whatever is open from the current state (host calls this after loading saved settings)
    pub fn pb_refresh(&mut self) {
        if self.pb.sp_open { self.kill_tag("pb_sp"); self.pb.sp_open = false; self.open_startpos(); }
        if self.pb.kb_open && self.pb.kb_sub.is_none() {
            let tab = self.pb.kb_tab;
            self.kb_unreg_scroll(); self.kill_tag("pb_kb"); self.pb.kb_open = false;
            self.open_keybinds();
            self.pb.kb_tab = tab; self.kb_refresh_tabs(); self.kb_build_list(true);
        }
    }

    pub fn pb_keybinds_json(&self) -> String {
        let parts: Vec<String> = self.pb.kb_keys.iter().map(|(k, v)| format!("\"{}\":[{}]", k, v.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(","))).collect();
        format!("{{{}}}", parts.join(","))
    }
}

// ---------------------------------------------------------------------- wasm exports
use crate::{out_str, with_ui};

unsafe fn s(ptr: *const u8, len: usize) -> String { String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned() }

/// saved StartPos options from gd_settings: switcher, smart, opacity (0.05..1)
#[no_mangle]
pub extern "C" fn wd_pb_startpos(switcher: u32, smart: u32, opacity: f64) {
    with_ui(|u| { u.pb.sp_switcher = switcher != 0; u.pb.sp_smart = smart != 0; u.pb.sp_opacity = if opacity.is_finite() && opacity != 0.0 { opacity.max(0.05).min(1.0) } else { 1.0 }; });
}
#[no_mangle]
pub extern "C" fn wd_pb_startpos_get(which: u32) -> f64 { with_ui(|u| match which { 0 => u.pb.sp_switcher as u32 as f64, 1 => u.pb.sp_smart as u32 as f64, _ => u.pb.sp_opacity }) }
/// one saved keybind: `keys` is a comma separated list of KeyboardEvent.code values
#[no_mangle]
pub unsafe extern "C" fn wd_pb_kb_set(id: *const u8, id_len: usize, keys: *const u8, keys_len: usize) {
    let (id, keys) = (s(id, id_len), s(keys, keys_len));
    with_ui(|u| { u.pb.kb_keys.insert(id, keys.split(',').filter(|k| !k.is_empty()).map(|k| k.to_string()).collect()); });
}
#[no_mangle]
pub extern "C" fn wd_pb_kb_json() -> usize { out_str(with_ui(|u| u.pb_keybinds_json())) }
/// returns 1 when the popup used the key
#[no_mangle]
pub unsafe extern "C" fn wd_pb_key(code: *const u8, code_len: usize, key: *const u8, key_len: usize, ctrl: u32) -> u32 {
    let (c, k) = (s(code, code_len), s(key, key_len));
    with_ui(|u| u.pb_key(&c, &k, ctrl != 0)) as u32
}
#[no_mangle]
pub unsafe extern "C" fn wd_pb_paste(ptr: *const u8, len: usize) { let t = s(ptr, len); with_ui(|u| u.pb_paste(&t)); }
#[no_mangle]
pub extern "C" fn wd_pb_copy_text() -> usize { out_str(with_ui(|u| u.pb_copy_text())) }
#[no_mangle]
pub extern "C" fn wd_pb_refresh() { with_ui(|u| u.pb_refresh()); }
#[no_mangle]
pub extern "C" fn wd_pb_seed(seed: f64) { with_ui(|u| { u.pb.rng = (seed.abs() * 4294967296.0) as u64 | 1; }); }
/// jumps, attempts, completed levels, deaths, secret coins, user coins
#[no_mangle]
pub extern "C" fn wd_pb_stats(jumps: f64, attempts: f64, completed: f64, deaths: f64, secret: f64, user: f64) {
    with_ui(|u| u.pb_set_stats([jumps as u64, attempts as u64, completed as u64, deaths as u64, secret as u64, user as u64]));
}
/// opens the level stats popup (level-select info button / in-game info button)
#[no_mangle]
pub unsafe extern "C" fn wd_pb_level_stats(name: *const u8, name_len: usize, attempts: f64, jumps: f64, best: f64, practice: f64) {
    let n = s(name, name_len);
    with_ui(|u| u.pb_open_level_stats(&n, attempts as u64, jumps as u64, best as i64, practice as i64));
}
