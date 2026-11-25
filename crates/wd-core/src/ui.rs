//! Menus and HUD: a small retained scene (images, text, shapes), a tween engine with Phaser's easings, bouncy buttons and pointer
//! hit-testing. The screens are built the way game-scene.js builds them; the host just draws `Ui::nodes` in depth order.

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind { Image, Text, Rect, RoundRect, Circle, Nine, Clip, ClipEnd }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cond { Always, MenuActive, MenuNoSelect }

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub atlas: String,
    pub frame: String,
    pub text: String,
    pub font: String,
    pub font_size: f64,
    pub x: f64, pub y: f64,
    pub sx: f64, pub sy: f64,
    pub rot: f64,
    pub alpha: f64,
    pub depth: f64,
    pub visible: bool,
    pub ox: f64, pub oy: f64,
    pub flip_x: bool, pub flip_y: bool,
    pub tint: i64,
    pub additive: bool,
    /// Phaser `postFX.addColorMatrix().grayscale(1)`: drawn in grey (each channel the average of the three), then tinted
    pub gray: bool,
    pub color: i64,
    pub w: f64, pub h: f64, pub radius: f64,
    pub width: f64, pub height: f64,   // texture frame size (image) — what Phaser calls `.width/.height`
    pub interactive: bool,
    pub hit_scale: f64,
    pub bouncy: Option<(f64, f64, Cond)>,
    /// how far a bouncy button grows while pressed (1.26 unless the button says otherwise)
    pub press: f64,
    pub pressed: bool,
    pub action: Action,
    pub tag: &'static str,
    /// frame-pixel crop width for progress bars (0 = none)
    pub crop_w: f64,
    /// drawn in world space (moves with the level)
    pub world: bool,
    /// positioned relative to (and scaled with) this node, like a Phaser container child
    pub parent: Option<usize>,
    /// slider thumb: (min x, max x, fill node, which slider, grab offset)
    pub drag: Option<(f64, f64, usize, u8, f64)>,
    /// index into `Ui::scrolls` when this node is a scroll area's touch zone
    pub scroll: Option<usize>,
}

impl Node {
    pub(crate) fn new(kind: Kind) -> Node {
        Node { kind, atlas: String::new(), frame: String::new(), text: String::new(), font: String::new(), font_size: 0.0, x: 0.0, y: 0.0, sx: 1.0, sy: 1.0, rot: 0.0, alpha: 1.0, depth: 0.0,
            visible: true, ox: 0.5, oy: 0.5, flip_x: false, flip_y: false, tint: -1, additive: false, gray: false, color: 0xffffff, w: 0.0, h: 0.0, radius: 0.0, width: 0.0, height: 0.0,
            interactive: false, hit_scale: 1.0, bouncy: None, press: 1.26, pressed: false, action: Action::None, tag: "", crop_w: 0.0, world: false, parent: None, drag: None, scroll: None }
    }
}

/// What a pressed button asks the game to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Action { None, Pause, PauseResume, PauseMenu, PauseReplay, PausePractice, Checkpoint, ClearCheckpoint, NextAfterWin, OpenLevelSelect, CloseLevelSelect, OpenUrl(&'static str), Settings, Stats, Info, News, Creator, Garage, Fullscreen, Soundtrack, PrevLevel, NextLevel, PlayLevel, SettingsClose, ToggleMenuMusic, Volume, Vault, Popup(&'static str), OptTab(usize), OptClose, OptToggle(usize), OptFocus(usize), OptChanged, InfoBox(&'static str), InfoClose, Pop(&'static str, usize), LevelStats, Sfx(&'static str) }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prop { X, Y, ScaleX, ScaleY, Alpha }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease { Linear, QuadIn, QuadOut, BounceOut, SineInOut, ElasticOut }

#[derive(Clone, Debug)]
pub struct Tween { pub node: usize, pub props: Vec<(Prop, f64, f64)>, pub duration: f64, pub elapsed: f64, pub ease: Ease, pub then: Then }

#[derive(Clone, Debug, PartialEq)]
pub enum Then { Nothing, Destroy }

/// Phaser's easing functions
pub fn ease(e: Ease, v: f64) -> f64 {
    match e {
        Ease::Linear => v,
        Ease::QuadIn => v * v,
        Ease::QuadOut => v * (2.0 - v),
        Ease::SineInOut => 0.5 * (1.0 - (std::f64::consts::PI * v).cos()),
        Ease::ElasticOut => { if v == 0.0 { 0.0 } else if v == 1.0 { 1.0 } else { let period = 0.6; let s = period / 4.0; 2f64.powf(-10.0 * v) * ((v - s) * (std::f64::consts::PI * 2.0) / period).sin() + 1.0 } }
        Ease::BounceOut => {
            let mut v = v;
            if v < 1.0 / 2.75 { 7.5625 * v * v }
            else if v < 2.0 / 2.75 { v -= 1.5 / 2.75; 7.5625 * v * v + 0.75 }
            else if v < 2.5 / 2.75 { v -= 2.25 / 2.75; 7.5625 * v * v + 0.9375 }
            else { v -= 2.625 / 2.75; 7.5625 * v * v + 0.984375 }
        }
    }
}

#[derive(Clone, Debug)]
pub struct LevelInfo { pub name: String, pub display: String, pub id: String, pub song: Option<String> }

/// A BMFont: the parts needed to measure text.
#[derive(Default, Clone)]
pub struct Font { pub size: f64, pub line_height: f64, pub adv: HashMap<u32, f64>, pub kern: HashMap<(u32, u32), f64> }

impl Font {
    pub fn parse(s: &str) -> Font {
        let mut f = Font::default();
        let num = |line: &str, key: &str| -> Option<f64> { line.split_whitespace().find_map(|p| p.strip_prefix(&format!("{}=", key))).and_then(|v| v.parse().ok()) };
        for line in s.lines() {
            if line.starts_with("info ") { f.size = num(line, "size").unwrap_or(0.0); }
            else if line.starts_with("common ") { f.line_height = num(line, "lineHeight").unwrap_or(0.0); }
            else if line.starts_with("char ") { if let (Some(id), Some(a)) = (num(line, "id"), num(line, "xadvance")) { f.adv.insert(id as u32, a); } }
            else if line.starts_with("kerning ") { if let (Some(a), Some(b), Some(k)) = (num(line, "first"), num(line, "second"), num(line, "amount")) { f.kern.insert((a as u32, b as u32), k); } }
        }
        f
    }
    /// Width of `text` at font size `size` (what `bitmapText.width` reports before scaling).
    pub fn width(&self, text: &str, size: f64) -> f64 {
        let scale = if self.size > 0.0 { size / self.size } else { 1.0 };
        let mut x = 0.0; let mut prev: Option<u32> = None;
        for ch in text.chars() {
            let c = ch as u32;
            if let Some(p) = prev { x += self.kern.get(&(p, c)).copied().unwrap_or(0.0); }
            x += self.adv.get(&c).copied().unwrap_or(0.0);
            prev = Some(c);
        }
        x * scale
    }
    pub fn height(&self, size: f64) -> f64 { if self.size > 0.0 { self.line_height * size / self.size } else { size } }
}

/// Saved progress the level select shows.
#[derive(Clone, Default, Debug)]
pub struct Progress { pub normal: f64, pub practice: f64, pub coins: [bool; 3] }

/// a numeric box on the options popup
#[derive(Clone, Debug)]
pub struct InputDef { pub text: usize, pub key: &'static str, pub min: f64, pub max: f64, pub integer: bool }

fn fmt_num(v: f64) -> String { if v.fract() == 0.0 && v.abs() < 1e15 { format!("{}", v as i64) } else { format!("{}", v) } }

const INFO: [(&str, &str); 21] = [
    ("Enable Portal Guide", "Enables extra indicators on portals."),
    ("Enable Orb Guide", "Enables extra indicators on orbs."),
    ("Practice Music Bypass", "Plays normal mode music in practice mode."),
    ("Show Percentage", "Shows the percentage you are at in a level."),
    ("Percentage Decimals", "Shows decimals in level progress."),
    ("Progress Bar", "Shows a progress bar at the top of the screen."),
    ("Ignore Damage", "Allows you to phase throught mostly anything that would kill you normally."),
    ("Ignore Damage Accuracy", "Flashes your screen red when you would've died with noclip."),
    ("Macro Bot", "Lets you record and replay your inputs for a level."),
    ("Show Hitboxes", "Shows you the hitboxes for objects and your player."),
    ("Hitbox Trail", "Shows the hitbox of exactly where your hitbox has been."),
    ("Hitboxes on Death", "Shows hitboxes upon death in both normal and practice mode."),
    ("Show FPS", "Shows the frames per second your game is running at."),
    ("Solid Wave Trail", "Removes the extra details of the wave trail."),
    ("Show CPS", "Shows when you click in a level in the top left of your screen."),
    ("Show Glow", "Shows glow for basic object sets."),
    ("Cull Distance", "Changes how many objects are shown. [DOES NOT SAVE!!]"),
    ("Default Mini Icon", "Sets player icon in min mode to default."),
    ("Hide Pause Button", "Hides the pause button during gameplay. Prevents accidental clicks."),
    ("Hide Practice UI", "Hides the checkpoint buttons in practice mode."),
    ("Startpos Switcher", "Switches between start positions in a level."),
];

/// (label, key, second key, font size, info key) — `None` info means no (i) button
type ToggleDef = (&'static str, &'static str, &'static str, f64, Option<&'static str>);
const GAMEPLAY_L: [ToggleDef; 6] = [
    ("Show Percentage", "showPercentage", "", 25.0, Some("Show Percentage")),
    ("Percentage Decimals", "percentDecimals", "", 25.0, Some("Percentage Decimals")),
    ("Progress Bar", "progressBar", "", 25.0, Some("Progress Bar")),
    ("Ignore Damage", "noclip", "", 25.0, Some("Ignore Damage")),
    ("Ignore Damage Accuracy", "noclipAccuracy", "", 22.0, Some("Ignore Damage Accuracy")),
    ("Macro Bot", "macroBot", "", 25.0, Some("Macro Bot")),
];
const VISUAL_L: [ToggleDef; 6] = [
    ("Show Hitboxes", "showHitboxes", "", 25.0, Some("Show Hitboxes")),
    ("Hitbox Trail", "hitboxTrail", "", 25.0, Some("Hitbox Trail")),
    ("Hitboxes on Death", "hitboxesOnDeath", "", 25.0, Some("Hitboxes on Death")),
    ("Show FPS", "showFPS", "", 25.0, Some("Show FPS")),
    ("Solid Wave Trail", "solidWaveTrail", "", 25.0, Some("Solid Wave Trail")),
    ("Show CPS", "showCPS", "", 25.0, Some("Show CPS")),
];
const VISUAL_R: [ToggleDef; 5] = [
    ("Show Glow", "showGlow", "", 25.0, Some("Show Glow")),
    ("Object ID Labels", "createObjectIds", "showObjectIds", 20.0, None),
    ("Enable Portal Guide", "enablePortalGuide", "", 22.0, Some("Enable Portal Guide")),
    ("Enable Orb Guide", "enableOrbGuide", "", 25.0, Some("Enable Orb Guide")),
    ("Default Mini Icon", "enableMiniIcon", "", 25.0, Some("Default Mini Icon")),
];

/// a vertically scrolling list: `content` is moved up by `y` (wheel or drag on `zone`)
#[derive(Clone, Debug, Default)]
pub struct Scroll { pub zone: usize, pub content: usize, pub base_y: f64, pub y: f64, pub max: f64, pub drag_from: Option<(f64, f64)> }

#[derive(Default)]
pub struct Ui {
    pub levels: Vec<LevelInfo>,
    pub page: usize,
    pub fonts: HashMap<String, Font>,
    pub progress: HashMap<String, Progress>,
    pub total_coins: f64,
    pub ground_tile: (f64, f64),
    pub has_ground2: bool,
    pub nodes: Vec<Option<Node>>,
    pub tweens: Vec<Tween>,
    pub screen_width: f64,
    pub screen_height: f64,
    pub menu_active: bool,
    pub level_select_open: bool,
    pub sizes: HashMap<String, (f64, f64)>,
    pub events: Vec<Action>,
    pub pointer_down_node: Option<usize>,
    pub starting: bool,
    pub paused: bool,
    pub end_open: bool,
    pub end_p: f64,
    pub end_clock: f64,
    pub set_open: bool,
    pub set_closing: bool,
    pub set_clock: f64,
    pub set_p: f64,
    pub music_vol: f64,
    pub sfx_vol: f64,
    pub menu_music: bool,
    pub set_check: usize,
    pub set_overlay: usize,
    /// saved game options (gd_settings keys; booleans as 0/1)
    pub opts: HashMap<String, f64>,
    pub hud_pct_value: f64,
    pub opt_open: bool,
    pub opt_page: usize,
    pub opt_tabs: Vec<usize>,
    pub opt_toggles: Vec<(usize, &'static str, &'static str)>,
    pub opt_inputs: Vec<InputDef>,
    pub opt_focus: Option<usize>,
    pub opt_str: String,
    pub opt_root: usize,
    pub scrolls: Vec<Scroll>,
    pub ls: crate::lscard::LsState,
    pub ufx: crate::uifx::UiFx,
    pub pa: crate::popups_a::PopA,
    pub gar: crate::garage::Gar,
    pub pb: crate::popups_b::PopB,
    pub em: crate::editor_menu::EdMenu,
    pub on: crate::online::On,
    pub ed: crate::editor_scene::EditorState,
    pub practice: bool,
    pub attempt: u32,
    pub best_normal: f64,
    pub best_practice: f64,
    pub level_name: String,
    pub start_requested: bool,
    pub buf: Vec<f64>,
    pub strings: Vec<String>,
}

impl Ui {
    pub fn new(screen_width: f64) -> Ui { Ui { screen_width, screen_height: 640.0, ..Default::default() } }

    pub(crate) fn add(&mut self, n: Node) -> usize { self.nodes.push(Some(n)); self.nodes.len() - 1 }

    pub fn image(&mut self, atlas: &str, frame: &str, x: f64, y: f64, depth: f64, scale: f64) -> usize {
        let mut n = Node::new(Kind::Image);
        n.atlas = atlas.into(); n.frame = frame.into(); n.x = x; n.y = y; n.depth = depth; n.sx = scale; n.sy = scale;
        let (w, h) = self.sizes.get(&format!("{}/{}", atlas, frame)).copied().unwrap_or((0.0, 0.0));
        n.width = w; n.height = h;
        self.add(n)
    }

    pub fn node(&mut self, i: usize) -> &mut Node { self.nodes[i].as_mut().unwrap() }
    pub fn get(&self, i: usize) -> Option<&Node> { self.nodes.get(i).and_then(|n| n.as_ref()) }

    pub fn tween(&mut self, node: usize, props: Vec<(Prop, f64, f64)>, duration: f64, e: Ease, then: Then) {
        self.tweens.push(Tween { node, props, duration, elapsed: 0.0, ease: e, then });
    }
    pub fn kill_tweens(&mut self, node: usize) { self.tweens.retain(|t| t.node != node); }

    pub(crate) fn get_prop(n: &Node, p: Prop) -> f64 { match p { Prop::X => n.x, Prop::Y => n.y, Prop::ScaleX => n.sx, Prop::ScaleY => n.sy, Prop::Alpha => n.alpha } }
    pub(crate) fn set_prop(n: &mut Node, p: Prop, v: f64) { match p { Prop::X => n.x = v, Prop::Y => n.y = v, Prop::ScaleX => n.sx = v, Prop::ScaleY => n.sy = v, Prop::Alpha => n.alpha = v } }

    /// Starts a tween that reads its start values from the node now (Phaser tweens `from` the current value).
    pub fn tween_to(&mut self, node: usize, props: &[(Prop, f64)], duration: f64, e: Ease, then: Then) {
        let n = match self.get(node) { Some(n) => n, None => return };
        let p: Vec<(Prop, f64, f64)> = props.iter().map(|(p, to)| (*p, Self::get_prop(n, *p), *to)).collect();
        self.tween(node, p, duration, e, then);
    }

    pub(crate) fn yoff(&self, tag: &str) -> f64 {
        match tag { "end" => self.end_p * 650.0 - 640.0, "set_scr" => self.set_p * 650.0 - 640.0, _ => 0.0 }
    }

    pub fn step(&mut self, dt_ms: f64) {
        self.pa_step(dt_ms);
        self.gar_step(dt_ms);
        self.on_step(dt_ms);
        self.ls_step(dt_ms);
        self.ufx_step(dt_ms);
        self.pb_step(dt_ms);
        self.pe_step(dt_ms);
        if self.set_open {
            self.set_clock += dt_ms;
            if self.set_closing {
                let t = (self.set_clock / 450.0).min(1.0);
                self.set_p = 1.0 - ease(Ease::QuadIn, t);
                if t >= 1.0 { self.finish_settings_close(); }
            } else { self.set_p = ease(Ease::QuadOut, (self.set_clock / 500.0).min(1.0)); }
        }
        if self.end_open && self.end_p < 1.0 { self.end_clock += dt_ms; self.end_p = ease(Ease::BounceOut, (self.end_clock / 1000.0).min(1.0)); }
        let mut i = 0;
        while i < self.tweens.len() {
            let done = {
                let t = &mut self.tweens[i];
                t.elapsed += dt_ms;
                let p = if t.duration > 0.0 { (t.elapsed / t.duration).min(1.0) } else { 1.0 };
                let k = ease(t.ease, p);
                if let Some(n) = self.nodes.get_mut(t.node).and_then(|n| n.as_mut()) {
                    for (prop, from, to) in &t.props { Self::set_prop(n, *prop, from + (to - from) * k); }
                }
                p >= 1.0
            };
            if done {
                let t = self.tweens.remove(i);
                if t.then == Then::Destroy { if let Some(slot) = self.nodes.get_mut(t.node) { *slot = None; } }
            } else { i += 1; }
        }
    }

    // ---- bouncy buttons (`_makeBouncyButton`) ----
    pub fn make_bouncy(&mut self, node: usize, base: f64, cond: Cond, action: Action) { self.make_bouncy_xy(node, base, base, cond, action); }
    pub fn make_bouncy_xy(&mut self, node: usize, bx: f64, by: f64, cond: Cond, action: Action) {
        let n = self.node(node);
        n.bouncy = Some((bx, by, cond)); n.interactive = true; n.action = action;
    }

    pub(crate) fn cond_ok(&self, c: Cond) -> bool {
        match c { Cond::Always => true, Cond::MenuActive => self.menu_active, Cond::MenuNoSelect => self.menu_active && !self.level_select_open }
    }

    /// position and scale after the container chain (`parent`) is applied
    pub(crate) fn eff(&self, i: usize) -> (f64, f64, f64, f64) {
        let Some(n) = self.get(i) else { return (0.0, 0.0, 1.0, 1.0) };
        match n.parent {
            Some(p) if p != i => { let (px, py, psx, psy) = self.eff(p); (px + n.x * psx, py + n.y * psy, n.sx * psx, n.sy * psy) }
            _ => (n.x, n.y, n.sx, n.sy),
        }
    }

    pub(crate) fn hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let mut best: Option<(f64, usize)> = None;
        // the visible window of every scroll area: what lies in its content can only be hit inside it
        let windows: Vec<(usize, f64, f64, f64, f64)> = self.scrolls.iter().filter_map(|sc| {
            let z = self.get(sc.zone)?; let (ex, ey, esx, esy) = self.eff(sc.zone);
            let (w, h) = (z.width * esx.abs(), z.height * esy.abs());
            Some((sc.content, ex - w * z.ox, ey - h * z.oy, w, h))
        }).collect();
        for (i, n) in self.nodes.iter().enumerate() {
            let Some(n) = n else { continue };
            if !n.interactive || !n.visible { continue; }
            let (ex, ey, esx, esy) = self.eff(i);
            let (w, h) = (n.width * esx.abs() * n.hit_scale, n.height * esy.abs() * n.hit_scale);
            let off = self.yoff(n.tag);
            let (l, t) = (ex - w * n.ox, ey + off - h * n.oy);
            if !(w > 0.0 && x >= l && x <= l + w && y >= t && y <= t + h) { continue; }
            if !windows.is_empty() && n.scroll.is_none() {
                let mut p = n.parent; let mut hops = 0;
                while let Some(pi) = p {
                    if let Some(&(_, wl, wt, ww, wh)) = windows.iter().find(|w| w.0 == pi) { if !(x >= wl && x <= wl + ww && y >= wt && y <= wt + wh) { p = Some(usize::MAX); } break; }
                    p = self.get(pi).and_then(|q| q.parent); hops += 1; if hops > 6 { break; }
                }
                if p == Some(usize::MAX) { continue; }
            }
            // a scroll area catches drags, but a button on its content wins a tap: it ranks just below its buttons
            let rank = if n.scroll.is_some() { n.depth - 0.05 } else { n.depth };
            if best.map_or(true, |(d, _)| rank >= d) { best = Some((rank, i)); }
        }
        best.map(|b| b.1)
    }

    // ---- level select (`_openLevelSelect`) ----
    pub(crate) fn level_color(&self, page: usize) -> i64 {
        const C: [i64; 9] = [0x0100f5, 0xf902f8, 0xf90285, 0xfa0102, 0xfa8702, 0xfcfc06, 0x03fb03, 0x02fbfb, 0x007dff];
        C[page % 9]
    }

    pub fn text_node(&mut self, font: &str, size: f64, text: &str, x: f64, y: f64, depth: f64, ox: f64, oy: f64, scale: f64) -> usize {
        let mut n = Node::new(Kind::Text);
        n.font = font.into(); n.font_size = size; n.text = text.into(); n.x = x; n.y = y; n.depth = depth; n.ox = ox; n.oy = oy; n.sx = scale; n.sy = scale;
        let f = self.fonts.get(font).cloned().unwrap_or_default();
        n.width = f.width(text, size); n.height = f.height(size);
        self.add(n)
    }

    pub(crate) fn shape(&mut self, kind: Kind, x: f64, y: f64, w: f64, h: f64, color: i64, alpha: f64, depth: f64, tag: &'static str) -> usize {
        let mut n = Node::new(kind);
        n.x = x; n.y = y; n.w = w; n.h = h; n.color = color; n.alpha = alpha; n.depth = depth; n.tag = tag; n.ox = 0.0; n.oy = 0.0;
        self.add(n)
    }

    pub fn open_level_select(&mut self) {
        if self.level_select_open { return; }
        self.level_select_open = true;
        self.ls = crate::lscard::LsState::fresh();
        let (sw, sh) = (self.screen_width, self.screen_height);
        let (cx, cy) = (sw / 2.0, sh / 2.0);
        let fade = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 1.0, 200.0, "ls");
        self.tween_to(fade, &[(Prop::Alpha, 0.0)], 300.0, Ease::Linear, Then::Destroy);
        self.build_level_select_static(cx, cy, sw, sh);
        let blocker = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 151.0, "ls_block");
        { let n = self.node(blocker); n.interactive = true; n.width = sw; n.height = sh; }
        self.build_level_card();
    }

    pub(crate) fn build_level_select_static(&mut self, cx: f64, cy: f64, sw: f64, sh: f64) {
        let color = self.level_color(self.page);
        // gradient overlay
        let every_end = self.levels.get(self.page).map_or(false, |l| l.id == "level_99");
        let (top_mul, bot_mul) = if every_end { (0.48, 0.18) } else { (0.92, 0.52) };
        for i in 0..60 {
            let t = i as f64 / 59.0;
            let mul = top_mul + (bot_mul - top_mul) * t;
            let ch = |sh_: i64| ((((color >> sh_) & 0xff) as f64 * mul).round()).min(255.0) as i64;
            let y0 = (i as f64 * sh / 60.0).floor();
            let h = (sh / 60.0).ceil() + 1.0;
            let n = self.shape(Kind::Rect, 0.0, y0, sw, h, (ch(16) << 16) | (ch(8) << 8) | ch(0), 1.0, 150.0, "ls_overlay");
            let _ = n;
        }
        let blocker = self.shape(Kind::Rect, 0.0, 0.0, sw, sh, 0, 0.0, 151.0, "ls");
        self.node(blocker).interactive = true; self.node(blocker).width = sw; self.node(blocker).height = sh;
        let bar = self.image("GJ_GameSheet03", "GJ_topBar_001.png", cx, 0.0, 152.0, 1.0); { let n = self.node(bar); n.oy = 0.0; n.tag = "ls"; }
        // ground
        let (tile_w, ref_h) = if self.ground_tile.0 > 0.0 { self.ground_tile } else { (1012.0, 270.0) };
        let ground_top = sh + 175.0 - ref_h;
        let tint = {
            let m = |sh_: i64| ((((color >> sh_) & 0xff) as f64 * 0.72).round()) as i64;
            (m(16) << 16) | (m(8) << 8) | m(0)
        };
        let tiles = (sw / tile_w).ceil() as usize + 2;
        for gi in 0..tiles {
            let g = self.image("", "groundSquare_main", gi as f64 * tile_w, ground_top, 151.0, 1.0);
            { let n = self.node(g); n.ox = 0.0; n.oy = 0.0; n.tint = tint; n.tag = "ls_ground"; n.width = tile_w; n.height = ref_h; }
            if self.has_ground2 {
                let g2 = self.image("", "groundSquare_second", gi as f64 * tile_w, ground_top, 151.5, 1.0);
                { let n = self.node(g2); n.ox = 0.0; n.oy = 0.0; n.tint = tint; n.tag = "ls_ground"; n.width = tile_w; n.height = ref_h; }
            }
        }
        let info = self.image("GJ_GameSheet03", "GJ_infoIcon_001.png", sw - 40.0, 40.0, 152.0, 1.0);
        self.make_bouncy(info, 1.0, Cond::Always, Action::LevelStats); self.node(info).tag = "ls";
        let fl_w = self.sizes.get("GJ_WebSheet/floorLine_01_001.png").map_or(888.0, |s| s.0);
        let line = self.image("GJ_WebSheet", "floorLine_01_001.png", cx, ground_top, 152.0, 1.0);
        { let n = self.node(line); n.sx = sw / fl_w; n.additive = true; n.tag = "ls"; }
        let bl = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", 0.0, sh, 152.0, 1.0); { let n = self.node(bl); n.ox = 0.0; n.oy = 1.0; n.tag = "ls"; }
        let br = self.image("GJ_GameSheet03", "GJ_sideArt_001.png", sw, sh, 152.0, 1.0); { let n = self.node(br); n.ox = 1.0; n.oy = 1.0; n.flip_x = true; n.tag = "ls"; }
        let back = self.image("GJ_GameSheet03", "GJ_arrow_01_001.png", 50.0, 48.0, 154.0, 1.0);
        { let n = self.node(back); n.flip_x = true; n.sy = -1.0; n.rot = std::f64::consts::PI; n.tag = "ls"; }
        self.make_bouncy_xy(back, 1.0, -1.0, Cond::Always, Action::CloseLevelSelect);
        let dl = self.text_node("bigFont", 56.0, "Download the soundtracks", cx, 585.0, 155.0, 0.5, 0.5, 0.52);
        { let n = self.node(dl); n.tag = "ls"; n.interactive = true; n.action = Action::Soundtrack; n.bouncy = Some((0.52, 0.52, Cond::Always)); }
        let al = self.image("GJ_GameSheet03", "navArrowBtn_001.png", 55.0, cy - 25.0, 154.0, 1.1); { let n = self.node(al); n.flip_x = true; n.tag = "ls"; }
        self.make_bouncy(al, 1.1, Cond::Always, Action::PrevLevel);
        let ar = self.image("GJ_GameSheet03", "navArrowBtn_001.png", sw - 55.0, cy - 25.0, 154.0, 1.1); self.node(ar).tag = "ls";
        self.make_bouncy(ar, 1.1, Cond::Always, Action::NextLevel);
    }

    /// The part of the level select that changes with the page: dots, card, bars.
    pub fn build_level_card(&mut self) {
        for slot in self.nodes.iter_mut() { if matches!(slot, Some(n) if matches!(n.tag, "ls_card" | "ls_bar" | "ls_dot" | "ls_hit")) { *slot = None; } }
        let (sw, sh) = (self.screen_width, self.screen_height);
        let (cx, cy) = (sw / 2.0, sh / 2.0);
        let color = self.level_color(self.page);
        // recolour the static parts
        let every_end = self.levels.get(self.page).map_or(false, |l| l.id == "level_99");
        let (top_mul, bot_mul) = if every_end { (0.48, 0.18) } else { (0.92, 0.52) };
        let mut band = 0;
        for slot in self.nodes.iter_mut() {
            if let Some(n) = slot {
                if n.tag == "ls_overlay" {
                    let t = band as f64 / 59.0; band += 1;
                    let mul = top_mul + (bot_mul - top_mul) * t;
                    let ch = |s_: i64| ((((color >> s_) & 0xff) as f64 * mul).round()).min(255.0) as i64;
                    n.color = (ch(16) << 16) | (ch(8) << 8) | ch(0);
                } else if n.tag == "ls_ground" {
                    let m = |s_: i64| ((((color >> s_) & 0xff) as f64 * 0.72).round()) as i64;
                    n.tint = (m(16) << 16) | (m(8) << 8) | m(0);
                }
            }
        }
        let page_count = self.levels.len() + 1;
        let max_dots = page_count.min(28);
        let spacing = 29.0;
        let start_x = cx - (max_dots as f64 - 1.0) * spacing / 2.0;
        for di in 0..max_dots {
            let n = self.shape(Kind::Circle, start_x + di as f64 * spacing, sh - 20.0, 7.0, 7.0, 0xffffff, if di == self.page { 1.0 } else { 0.3 }, 153.0, "ls_dot");
            let _ = n;
        }
        let card_w = 700f64.min(sw - 180.0);
        let card_h = 185.0;
        let (card_x, card_y) = (cx, cy - 100.0);
        let coming_soon = self.page >= self.levels.len();
        if coming_soon {
            let w = self.fonts.get("bigFont").map_or(0.0, |f| f.width("Coming Soon!", 56.0));
            let sc = if w > 0.0 { 1f64.min((card_w - 40.0) / w) } else { 1.0 };
            let t = self.text_node("bigFont", 56.0, "Coming Soon!", card_x, card_y, 155.0, 0.5, 0.5, sc); self.node(t).tag = "ls_card";
            return;
        }
        let lvl = self.levels[self.page].clone();
        let mul = if every_end { 0.10 } else { 0.22 };
        let cc = |s_: i64| ((((color >> s_) & 0xff) as f64 * mul).round()) as i64;
        let bg = self.shape(Kind::RoundRect, card_x - card_w / 2.0, card_y - card_h / 2.0, card_w, card_h, (cc(16) << 16) | (cc(8) << 8) | cc(0), 0.75, 152.0, "ls_card");
        self.node(bg).radius = 18.0;
        let required = match lvl.id.as_str() { "level_14" => 10.0, "level_18" => 20.0, "level_20" => 30.0, _ => 0.0 };
        let locked = required > self.total_coins;
        let diff = match lvl.id.as_str() {
            "level_1" | "level_2" => "diffIcon_01_btn_001", "level_3" | "level_4" => "diffIcon_02_btn_001", "level_5" | "level_6" => "diffIcon_03_btn_001",
            "level_7" | "level_8" | "level_9" | "level_17" | "level_19" | "level_56199846" => "diffIcon_04_btn_001",
            "level_10" | "level_11" | "level_12" | "level_13" | "level_15" | "level_16" | "level_21" | "level_22" => "diffIcon_05_btn_001",
            "level_14" | "level_18" | "level_20" | "level_4284013" => "diffIcon_06_btn_001", "level_5703070" => "diffIcon_07_btn_001",
            "level_99" | "level_100" | "level_23" => "diffIcon_10_btn_001", "level_137409445" | "level_137677336" | "level_116489424" => "diffIcon_00_btn_001",
            _ => "diffIcon_05_btn_001",
        };
        if locked {
            let lock = self.image("GJ_GameSheet03", "GJLargeLock_001.png", card_x, card_y - 8.0, 155.0, 1.0);
            let h = self.node(lock).height; if h > 0.0 { let sc = 1f64.min((card_h - 20.0) / h); let n = self.node(lock); n.sx = sc; n.sy = sc; }
            self.node(lock).tag = "ls_card";
        } else {
            let icon = self.image("GJ_GameSheet03", &format!("{}.png", diff), card_x, card_y, 155.0, 1.0);
            let (iw, ih) = { let n = self.node(icon); (n.width, n.height) };
            let sc = if iw > 0.0 && ih > 0.0 { 1.115f64.min((card_h - 16.0) / ih).min(80.0 / iw) } else { 1.115 };
            let (dw, _dh) = (if iw > 0.0 { iw } else { 80.0 } * sc, if ih > 0.0 { ih } else { 80.0 } * sc);
            let font = self.fonts.get("bigFont").cloned().unwrap_or_default();
            let (lw, lh) = (font.width(&lvl.display, 60.0), font.height(60.0));
            let gap = 25.0; let pad = 16.0;
            let max_w = card_w - pad * 2.0 - 100.0; let max_h = card_h - pad * 2.0;
            let max_label_w = (max_w - dw - gap).max(1.0);
            let label_scale = if lw > 0.0 { 1f64.min(max_label_w / lw).min(max_h / lh) } else { 1.0 };
            let total = dw + gap + lw * label_scale;
            let start = card_x - total / 2.0;
            { let n = self.node(icon); n.sx = sc; n.sy = sc; n.x = start + dw / 2.0; n.tag = "ls_card"; }
            let name = self.text_node("bigFont", 60.0, &lvl.display, start + dw + gap, card_y, 155.0, 0.0, 0.5, label_scale);
            self.node(name).tag = "ls_card";
        }
        // the card is the play button
        let hit = self.shape(Kind::Rect, card_x - card_w / 2.0, card_y - card_h / 2.0, card_w, card_h, 0, 0.0, 156.0, "ls_hit");
        { let n = self.node(hit); n.interactive = true; n.action = Action::None; n.width = card_w; n.height = card_h; n.ox = 0.0; n.oy = 0.0; }
        let prog = self.progress.get(&lvl.id).cloned().unwrap_or_default();
        if locked {
            let count = self.text_node("bigFont", 42.0, &format!("{}/{}", self.total_coins, required), card_x + card_w / 2.0 - 117.0, card_y + 63.0, 155.0, 0.5, 0.5, 1.0); self.node(count).tag = "ls_card";
            let ci = self.image("GJ_GameSheet03", "GJ_coinsIcon_001.png", card_x + card_w / 2.0 - 31.0, card_y + 63.0, 155.0, 1.0); self.node(ci).tag = "ls_card";
        } else {
            for k in 0..3 {
                let f = if prog.coins[k] { "GJ_coinsIcon_001.png" } else { "GJ_coinsIcon_gray_001.png" };
                let ci = self.image("GJ_GameSheet03", f, card_x + card_w / 2.0 - 131.0 + k as f64 * 50.0, card_y + 63.0, 155.0, 1.0); self.node(ci).tag = "ls_card";
            }
        }
        // progress bars
        let bar_area_y = card_y + card_h / 2.0 + 100.0 - 10.0;
        let (bw, bh) = (card_w, 36.0);
        let bx0 = cx - bw / 2.0;
        for (i, (label, best, col)) in [("Normal Mode", prog.normal, 0x00ff00i64), ("Practice Mode", prog.practice, 0x00ffff)].into_iter().enumerate() {
            let y = if i == 0 { bar_area_y } else { bar_area_y + bh + 48.0 };
            let l = self.text_node("bigFont", 30.0, label, cx, y - 40.0, 155.0, 0.5, 0.5, 1.0); self.node(l).tag = "ls_bar";
            let bg = self.shape(Kind::RoundRect, bx0, y - bh / 2.0, bw, bh, 0, 0.5, 154.0, "ls_bar"); self.node(bg).radius = bh / 2.0;
            if best > 0.0 {
                let pad = 3.0; let inner_h = bh - pad * 2.0; let inner_w = bw - pad * 2.0;
                let fill_w = inner_h.max(inner_w * best / 100.0);
                let fg = self.shape(Kind::RoundRect, bx0 + pad, y - bh / 2.0 + pad, fill_w, inner_h, col, 1.0, 155.0, "ls_bar");
                self.node(fg).radius = inner_h / 2.0;
            }
            let p = self.text_node("bigFont", 22.0, &format!("{}%", best.round()), cx, y, 156.0, 0.5, 0.5, 1.0); self.node(p).tag = "ls_bar";
        }
    }

    pub fn switch_level(&mut self, dir: i64) {
        let n = self.levels.len() as i64 + 1;
        self.page = (((self.page as i64 + dir) % n + n) % n) as usize;
        self.build_level_card();
    }

    pub fn close_level_select(&mut self) {
        for slot in self.nodes.iter_mut() { if matches!(slot, Some(n) if n.tag.starts_with("ls")) { *slot = None; } }
        self.tweens.retain(|t| self.nodes.get(t.node).map_or(false, |n| n.is_some()));
        self.level_select_open = false;
    }

    pub fn pointer_down(&mut self, x: f64, y: f64) {
        if self.ed.on && self.hit_test(x, y).is_none() { if self.ed.pt { self.ed.pt_press = true; } self.ed_world_down(x, y); }
        if self.level_select_open { let h = self.hit_test(x, y); self.ls_pointer_down(x, h); }
        if let Some(i) = self.hit_test(x, y) {
            if self.get(i).map_or(false, |n| n.tag == "opt_dim") { self.commit_input(); }
            self.pointer_down_node = Some(i);
            if let Some(si) = self.get(i).and_then(|n| n.scroll) { if let Some(sc) = self.scrolls.get_mut(si) { sc.drag_from = Some((y, sc.y)); } }
            if let Some(n) = self.nodes[i].as_mut() { if let Some(d) = n.drag.as_mut() { d.4 = n.x - x; } }
            if let Some((bx, by, cond)) = self.get(i).and_then(|n| n.bouncy) {
                if self.cond_ok(cond) {
                    self.node(i).pressed = true;
                    self.kill_tweens(i);
                    self.tween_to(i, &[(Prop::ScaleX, bx * self.get(i).map_or(1.26, |n| n.press)), (Prop::ScaleY, by * self.get(i).map_or(1.26, |n| n.press))], 300.0, Ease::BounceOut, Then::Nothing);
                }
            }
        }
    }

    pub fn pointer_move(&mut self, x: f64, y: f64) {
        if self.ed.on { self.ed_world_move(x, y); }
        if self.level_select_open { self.ls_pointer_move(x); }
        if let Some(i) = self.pointer_down_node {
            if self.get(i).map_or(false, |n| n.drag.is_some()) { self.drag_thumb(i, x); return; }
            if let Some(si) = self.get(i).and_then(|n| n.scroll) {
                if let Some((y0, s0)) = self.scrolls.get(si).and_then(|s| s.drag_from) { self.set_scroll(si, s0 + (y0 - y)); }
                return;
            }
            let over = self.hit_test(x, y) == Some(i);
            if !over { self.release(i, false); }
        }
    }

    pub(crate) fn release(&mut self, i: usize, fire: bool) {
        let Some(n) = self.get(i) else { return };
        if !n.pressed { return; }
        let (bx, by, _) = n.bouncy.unwrap_or((1.0, 1.0, Cond::Always));
        let action = n.action.clone();
        self.node(i).pressed = false;
        self.kill_tweens(i);
        if fire { let n = self.node(i); n.sx = bx; n.sy = by; self.emit(action); }
        else { self.tween_to(i, &[(Prop::ScaleX, bx), (Prop::ScaleY, by)], 400.0, Ease::BounceOut, Then::Nothing); }
        if !fire { self.pointer_down_node = None; }
    }

    pub fn pointer_up(&mut self, x: f64, y: f64) {
        if self.ed.on { self.ed.pt_press = false; self.ed_world_up(x, y); }
        if self.level_select_open { self.ls_pointer_up(x, y); }
        for sc in self.scrolls.iter_mut() { sc.drag_from = None; }
        if let Some(i) = self.pointer_down_node.take() {
            let over = self.hit_test(x, y) == Some(i);
            // plain (non-bouncy) hit zones just fire on release
            if let Some(n) = self.get(i) { if n.bouncy.is_none() && over && n.action != Action::None { let a = n.action.clone(); self.emit(a); return; } }
            self.release(i, over);
        }
    }

    // ---- the main menu (game-scene.js `create`, `_positionMenuItems`) ----
    pub fn build_main_menu(&mut self) {
        let sw = self.screen_width;
        let centre_x = sw / 2.0 - 150.0;
        self.menu_active = true;
        let logo = self.image("GJ_WebSheet", "GJ_logo_001.png", sw / 2.0, 100.0, 30.0, 1.2); self.node(logo).tag = "logo";
        let rob = self.image("GJ_WebSheet", "RobTopLogoBig_001.png", 110.0, 585.0, 30.0, 0.525);
        self.make_bouncy(rob, 0.525, Cond::MenuActive, Action::OpenUrl("https://geometrydash.com")); self.node(rob).tag = "rob";
        let socials: [(&str, &str, usize, usize); 5] = [
            ("gj_twIcon_001.png", "https://x.com/rohanis0000gd", 1, 0), ("gj_ytIcon_001.png", "https://www.youtube.com/@rohanis0000gd", 1, 1),
            ("gj_tiktokIcon_001.png", "https://www.tiktok.com/@rohanis00000", 1, 2), ("gj_githubIcon_001.png", "https://github.com/web-dashers/web-dashers.github.io", 1, 3),
            ("gj_discordIcon_001.png", "https://discord.gg/TfEzAVWPSJ", 2, 3),
        ];
        for (frame, url, row, col) in socials {
            let i = self.image("GJ_GameSheet03", frame, 65.0 + col as f64 * 52.0, 478.0 + row as f64 * 52.0, 30.0, 0.75);
            self.make_bouncy(i, 0.75, Cond::MenuActive, Action::OpenUrl(url)); self.node(i).tag = "social";
        }
        let mut t = Node::new(Kind::Text);
        t.text = "© 2026 RobTop Games · geometrydash.com".into(); t.font = "Arial".into(); t.font_size = 14.0; t.x = 288.0; t.y = 630.0; t.ox = 1.0; t.oy = 1.0; t.alpha = 0.3; t.depth = 30.0; t.tag = "copyright";
        self.add(t);
        let beta = self.image("", "GJ_MenuBeta", sw / 2.0 + 260.0, 150.0, 30.0, 0.75); self.node(beta).tag = "beta";
        let news = self.image("GJ_GameSheet04", "GJ_newsBtn_001.png", sw - 105.0, 550.0, 30.0, 1.0);
        self.make_bouncy(news, 1.0, Cond::MenuActive, Action::News); self.node(news).tag = "news";
        let fs = self.image("GJ_WebSheet", "toggleFullscreenOn_001.png", 33.0, 33.0, 30.0, 0.64);
        { let n = self.node(fs); n.alpha = 0.8; n.hit_scale = 1.5; }
        self.make_bouncy(fs, 0.64, Cond::MenuActive, Action::Fullscreen); self.node(fs).tag = "fs";
        let info = self.image("GJ_GameSheet03", "communityCreditsBtn_001.png", sw - 33.0, 35.0, 30.0, 0.73);
        self.node(info).hit_scale = 1.5;
        self.make_bouncy(info, 0.73, Cond::MenuActive, Action::Info); self.node(info).tag = "info";
        for (frame, x, action, tag) in [("GJ_optionsBtn_001.png", centre_x + 92.0, Action::Settings, "settings"), ("GJ_statsBtn_001.png", centre_x + 202.0, Action::Stats, "stats"),
                                        ("GJ_achBtn_001.png", centre_x - 19.0, Action::None, "ach"), ("GJ_ngBtn_001.png", centre_x + 312.0, Action::Popup("Newgrounds"), "ng")] {
            let i = self.image("GJ_GameSheet03", frame, x, 550.0, 30.0, 1.0);
            if tag == "ach" { self.lock_gray(i); }
            self.make_bouncy(i, 1.0, Cond::MenuActive, action); self.node(i).tag = tag;
        }
        let play = self.image("GJ_GameSheet04", "GJ_playBtn_001.png", sw / 2.0, 320.0, 30.0, 1.0);
        self.make_bouncy(play, 1.0, Cond::MenuNoSelect, Action::OpenLevelSelect); self.node(play).tag = "play";
        let play_w = self.node(play).width;
        let creator = self.image("GJ_GameSheet04", "GJ_creatorBtn_001.png", 0.0, 320.0, 30.0, 1.0);
        let cw = self.node(creator).width;
        self.node(creator).x = sw / 2.0 + play_w / 2.0 + 50.0 + cw / 2.0;
        self.make_bouncy(creator, 1.0, Cond::MenuNoSelect, Action::Creator); self.node(creator).tag = "creator";
        let garage = self.image("GJ_GameSheet03", "GJ_garageBtn_001.png", 0.0, 320.0, 30.0, 1.0);
        let gw = self.node(garage).width;
        self.node(garage).x = sw / 2.0 - play_w / 2.0 - 50.0 - gw / 2.0;
        self.make_bouncy(garage, 1.0, Cond::MenuNoSelect, Action::Garage); self.node(garage).tag = "garage";
        let gh = self.node(garage).height; let gx = self.node(garage).x;
        let d1 = self.image("GJ_GameSheet03", "GJ_chrSel_001.png", gx - 110.0, 320.0 - gh / 2.0 + 160.0, 31.0, 1.0); self.node(d1).tag = "decor";
        let ch = self.node(creator).height; let cx2 = self.node(creator).x;
        let d2 = self.image("GJ_GameSheet03", "GJ_lvlEdit_001.png", cx2 + 110.0, 320.0 - ch / 2.0 + 160.0, 31.0, 1.0); self.node(d2).tag = "decor";
    }

    // ---- in-game HUD (`_buildHUD`, `_createHUD...`) ----
    pub fn build_hud(&mut self) {
        let sw = self.screen_width;
        self.kill_tag("hud");
        let pct = self.text_node("bigFont", 32.0, "0%", sw / 2.0, 16.0, 100.0, 0.5, 0.5, 1.0); self.node(pct).tag = "hud_pct";
        // `_updateProgressBar`: groove (scale 0.5) over a 412 px fill that starts 206 px left of the centre
        let fill = self.image("", "sliderBar2", sw / 2.0 - 206.0, 16.0, 100.0, 1.0);
        { let n = self.node(fill); n.ox = 0.0; n.oy = 0.5; n.crop_w = 1.0; n.tint = 0x04FF00; n.tag = "hud_barfill"; n.visible = false; }
        let groove = self.image("", "slidergroove2", sw / 2.0, 16.0, 100.1, 0.5); { let n = self.node(groove); n.tag = "hud_bar"; n.visible = false; }
        let pause = self.image("GJ_WebSheet", "GJ_pauseBtn_clean_001.png", sw - 30.0, 30.0, 30.0, 1.0);
        { let n = self.node(pause); n.alpha = 75.0 / 255.0; n.hit_scale = 2.0; n.interactive = true; n.action = Action::Pause; n.tag = "hud_pause"; }
        self.hud_pct_value = 0.0;
        let att = self.text_node("bigFont", 65.0, "Attempt 1", 0.0, 150.0, 20.0, 0.5, 0.5, 1.0); { let n = self.node(att); n.tag = "hud_attempt"; n.world = true; }
        // practice-mode checkpoint buttons
        let cp = self.image("GJ_GameSheet03", "GJ_checkpointBtn_001.png", sw / 2.0 - 50.0, 640.0 - 60.0, 30.0, 0.8);
        self.make_bouncy(cp, 0.8, Cond::Always, Action::Checkpoint); { let n = self.node(cp); n.tag = "hud_cp"; n.visible = false; n.hit_scale = 2.0; }
        let cl = self.image("GJ_GameSheet03", "GJ_removeCheckBtn_001.png", sw / 2.0 + 50.0, 640.0 - 60.0, 30.0, 0.8);
        self.make_bouncy(cl, 0.8, Cond::Always, Action::ClearCheckpoint); { let n = self.node(cl); n.tag = "hud_cp"; n.visible = false; n.hit_scale = 1.5; }
        self.apply_hud_options();
    }

    fn opt_on(&self, k: &str, default: f64) -> bool { self.opts.get(k).copied().unwrap_or(default) != 0.0 }

    /// The gd_settings options that shape the HUD: Show Percentage, Percentage Decimals, Progress Bar, Hide Pause Button, Hide Practice UI.
    pub fn apply_hud_options(&mut self) {
        let sw = self.screen_width;
        let (show, bar, hide_pause, hide_prac) = (self.opt_on("showPercentage", 1.0), self.opt_on("progressBar", 0.0), self.opt_on("hidePauseBtn", 0.0), self.opt_on("hidePracticeUI", 0.0));
        for slot in self.nodes.iter_mut() { if let Some(n) = slot {
            match n.tag {
                "hud_pct" => { n.visible = show; n.ox = if bar { 0.0 } else { 0.5 }; n.x = sw / 2.0 + if bar { 220.0 } else { 0.0 }; n.y = 16.0; }
                "hud_bar" | "hud_barfill" => n.visible = bar && (n.tag == "hud_bar" || self.hud_pct_value > 0.0),
                "hud_pause" => { n.alpha = if hide_pause { 0.0 } else { 75.0 / 255.0 }; n.interactive = !hide_pause; }
                "hud_cp" => n.alpha = if hide_prac { 0.0 } else { 1.0 },
                _ => {}
            }
        } }
        let p = self.hud_pct_value; self.set_percent(p);
    }

    pub fn reset_to_main_menu(&mut self) { self.nodes.clear(); self.tweens.clear(); self.events.clear(); self.level_select_open = false; self.paused = false; self.starting = false; self.pointer_down_node = None; self.build_main_menu(); }

    pub(crate) fn kill_tag(&mut self, tag: &str) { for slot in self.nodes.iter_mut() { if matches!(slot, Some(n) if n.tag.starts_with(tag)) { *slot = None; } } }

    pub fn set_percent(&mut self, pct: f64) {
        let pct = pct.max(0.0).min(100.0);
        self.hud_pct_value = pct;
        let text = if self.opt_on("percentDecimals", 0.0) { format!("{:.2}%", pct) } else { format!("{}%", pct.floor() as i64) };
        let bar = self.opt_on("progressBar", 0.0);
        for slot in self.nodes.iter_mut() { if let Some(n) = slot {
            if n.tag == "hud_barfill" { n.crop_w = (412.0 * pct / 100.0).max(1.0); n.visible = bar && pct > 0.0; }
        } }
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        for slot in self.nodes.iter_mut() { if let Some(n) = slot { if n.tag == "hud_pct" && n.text != text { n.text = text.clone(); n.width = f.width(&n.text, 32.0); } } }
    }

    /// `_attemptsLabel.setText` + `_positionAttemptsLabel`
    pub fn set_attempt(&mut self, n: u32, camera_x: f64) {
        self.attempt = n;
        let text = format!("Attempt {}", n);
        let x = camera_x + self.screen_width / 2.0 + if n > 1 { 100.0 } else { 0.0 };
        let f = self.fonts.get("bigFont").cloned().unwrap_or_default();
        for slot in self.nodes.iter_mut() { if let Some(nd) = slot { if nd.tag == "hud_attempt" { nd.text = text.clone(); nd.width = f.width(&nd.text, 65.0); nd.x = x; nd.y = 150.0; } } }
    }

    pub fn set_practice_ui(&mut self, on: bool) {
        for slot in self.nodes.iter_mut() { if let Some(n) = slot { if n.tag == "hud_cp" { n.visible = on; } } }
    }

    // ---- pause overlay (`_buildPauseOverlay`) ----
    /// game-scene.js `_applyLockedGray`: greyscale plus the tint 0xafafaf
    pub fn lock_gray(&mut self, i: usize) { let n = self.node(i); n.gray = true; n.tint = 0xafafaf; if matches!(n.kind, Kind::Text) { n.color = 0xafafaf; } }

    pub fn open_pause(&mut self) {
        if self.paused { return; }
        self.paused = true;
        let sw = self.screen_width;
        let cx = sw / 2.0;
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 75.0 / 255.0, 100.0, "pause");
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; n.depth = 100.0; }
        let tex_w = self.sizes.get("/square04_001").map_or(64.0, |s| s.0);
        let panel_w = sw - 40.0;
        let panel = self.shape(Kind::Nine, cx - panel_w / 2.0, 320.0 - 300.0, panel_w, 600.0, 0, 150.0 / 255.0, 100.1, "pause");
        { let n = self.node(panel); n.frame = "square04_001".into(); n.radius = (tex_w * 0.325).min(tex_w / 2.0).min(panel_w / 2.0); n.tint = 0; }
        let fs = self.image("GJ_WebSheet", "toggleFullscreenOn_001.png", cx - panel_w / 2.0 + 40.0, 60.0, 100.2, 0.64);
        self.node(fs).hit_scale = 2.5; self.make_bouncy(fs, 0.64, Cond::Always, Action::Fullscreen); self.node(fs).tag = "pause";
        let st = self.image("GJ_GameSheet03", "GJ_optionsBtn_001.png", cx + panel_w / 2.0 - 60.0, 80.0, 100.2, 0.64);
        self.node(st).hit_scale = 2.5; self.make_bouncy(st, 0.64, Cond::Always, Action::Settings); self.node(st).tag = "pause";
        let title = self.level_name.clone();
        let t = self.text_node("bigFont", 40.0, &title, cx, 65.0, 100.2, 0.5, 0.5, 1.0); self.node(t).tag = "pause";
        let bar_w = self.sizes.get("GJ_WebSheet/GJ_progressBar_001.png").map_or(680.0, |s| s.0);
        let bar_h = self.sizes.get("GJ_WebSheet/GJ_progressBar_001.png").map_or(40.0, |s| s.1);
        for (label, best, y, col) in [("Normal Mode", self.best_normal, 170.0, 0x00ff00i64), ("Practice Mode", self.best_practice, 255.0, 0x00ffff)] {
            let bg = self.image("GJ_WebSheet", "GJ_progressBar_001.png", cx, y, 100.2, 1.0); { let n = self.node(bg); n.tint = 0; n.alpha = 125.0 / 255.0; n.tag = "pause"; }
            let fg = self.image("GJ_WebSheet", "GJ_progressBar_001.png", cx - bar_w * 0.992 / 2.0, y, 100.3, 1.0);
            { let n = self.node(fg); n.sx = 0.992; n.sy = 0.86; n.ox = 0.0; n.tint = col; n.crop_w = (bar_w * best / 100.0).floor().max(1.0); n.tag = "pause"; }
            let _ = bar_h;
            let p = self.text_node("bigFont", 30.0, &format!("{}%", best), cx, y, 100.4, 0.5, 0.5, 0.7); self.node(p).tag = "pause";
            let l = self.text_node("bigFont", 30.0, label, cx, y - 40.0, 100.4, 0.5, 0.5, 0.78); self.node(l).tag = "pause";
        }
        let items: [(&str, &str, Action); 4] = [
            ("GJ_GameSheet03", if self.practice { "GJ_normalBtn_001.png" } else { "GJ_practiceBtn_001.png" }, Action::PausePractice),
            ("GJ_WebSheet", "GJ_playBtn2_001.png", Action::PauseResume), ("GJ_WebSheet", "GJ_menuBtn_001.png", Action::PauseMenu), ("GJ_WebSheet", "GJ_replayBtn_001.png", Action::PauseReplay),
        ];
        let widths: Vec<f64> = items.iter().map(|(a, f, _)| self.sizes.get(&format!("{}/{}", a, f)).map_or(123.0, |s| s.0)).collect();
        let mut x = cx - (widths.iter().sum::<f64>() + 3.0 * 40.0) / 2.0;
        for (i, (a, f, act)) in items.iter().enumerate() {
            let b = self.image(a, f, x + widths[i] / 2.0, 390.0, 100.2, 1.0);
            self.make_bouncy(b, 1.0, Cond::Always, act.clone()); self.node(b).tag = "pause";
            x += widths[i] + 40.0;
        }
        // music and sound-effect volume (game-scene.js `createSlider`): icon, fill, groove, thumb at y 530
        let groove_w = self.sizes.get("GJ_WebSheet/slidergroove.png").map_or(420.0, |s| s.0);
        let sc = 0.7;
        for (which, pos_x, icon, val) in [(0u8, cx - 200.0, "gj_songIcon_001.png", self.music_vol), (1u8, cx + 200.0, "GJ_sfxIcon_001.png", self.sfx_vol)] {
            let ic = self.image("GJ_WebSheet", icon, pos_x - 185.0, 530.0, 100.2, 1.2); self.node(ic).tag = "pause";
            let bar_max = (groove_w - 8.0) * sc;
            let bar_x = pos_x - groove_w * sc / 2.0 + 2.8;
            let fill_w = val * bar_max;
            let fill = self.image("", "sliderBar", bar_x, 530.0, 100.2, 1.0);
            { let n = self.node(fill); n.ox = 0.0; n.oy = 0.5; n.crop_w = if fill_w > 0.0 { fill_w } else { 1.0 }; n.tag = "pause"; }
            let g = self.image("GJ_WebSheet", "slidergroove.png", pos_x, 530.0, 100.3, sc); self.node(g).tag = "pause";
            let th = self.image("GJ_WebSheet", "sliderthumb.png", bar_x + fill_w, 530.0, 100.4, sc);
            { let n = self.node(th); n.tag = "pause"; n.interactive = true; n.drag = Some((bar_x, bar_x + bar_max, fill, which, 0.0)); }
        }
    }

    /// A scroll area for a popup: clip rectangle (root-local), touch zone and the content node to move.
    /// Returns (content node, scroll index). Content children should use `parent = Some(content)` and depth in (depth, depth + 0.04).
    pub fn scroll_area(&mut self, root: usize, x: f64, y: f64, w: f64, h: f64, content_h: f64, depth: f64, tag: &'static str) -> (usize, usize) {
        let clip = self.shape(Kind::Clip, x - w / 2.0, y - h / 2.0, w, h, 0, 1.0, depth, tag);
        { let n = self.node(clip); n.parent = Some(root); n.ox = 0.0; n.oy = 0.0; }
        let content = self.shape(Kind::Rect, 0.0, y - h / 2.0 + 8.0, 0.0, 0.0, 0, 0.0, depth, tag);
        { let n = self.node(content); n.parent = Some(root); }
        let end = self.shape(Kind::ClipEnd, 0.0, 0.0, 0.0, 0.0, 0, 1.0, depth + 0.05, tag);
        { let n = self.node(end); n.parent = Some(root); }
        let zone = self.shape(Kind::Rect, x, y, w, h, 0, 0.0, depth + 0.06, tag);
        let si = self.scrolls.len();
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = h; n.interactive = true; n.parent = Some(root); n.scroll = Some(si); }
        let base_y = y - h / 2.0 + 8.0;
        self.scrolls.push(Scroll { zone, content, base_y, y: 0.0, max: (content_h - h + 16.0).max(0.0), drag_from: None });
        (content, si)
    }

    pub fn set_scroll(&mut self, si: usize, v: f64) {
        let Some(sc) = self.scrolls.get_mut(si) else { return };
        sc.y = v.max(0.0).min(sc.max);
        let (c, b, y) = (sc.content, sc.base_y, sc.y);
        if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.y = b - y; }
    }

    /// mouse wheel over a scroll area
    pub fn wheel(&mut self, x: f64, y: f64, dy: f64) {
        // the scroll area under the pointer, whatever sits on top of it
        let mut hit = None;
        for (si, sc) in self.scrolls.iter().enumerate() {
            let Some(z) = self.get(sc.zone) else { continue };
            let (ex, ey, esx, esy) = self.eff(sc.zone);
            let (w, h) = (z.width * esx.abs(), z.height * esy.abs());
            if x >= ex - w * z.ox && x <= ex - w * z.ox + w && y >= ey - h * z.oy && y <= ey - h * z.oy + h { hit = Some(si); }
        }
        if let Some(si) = hit { let cur = self.scrolls[si].y; self.set_scroll(si, cur + dy * 0.6); } else if self.ed.on { self.ed_wheel(x, y, dy); }
    }

    pub(crate) fn emit(&mut self, a: Action) {
        match a {
            Action::PrevLevel => { self.ls_switch(-1, None, 0.0); }
            Action::NextLevel => { self.ls_switch(1, None, 0.0); }
            Action::Soundtrack => { self.pa_open("Songs"); }
            Action::SettingsClose => self.close_settings(),
            Action::OptTab(i) => { if i != self.opt_page { self.commit_input(); self.opt_page = i; self.build_opt_page(); } }
            Action::OptClose => { self.commit_input(); self.kill_tag("opt_"); self.opt_open = false; }
            Action::OptToggle(i) => {
                if let Some(&(node, k1, k2)) = self.opt_toggles.get(i) {
                    let v = if self.opts.get(k1).copied().unwrap_or(0.0) != 0.0 { 0.0 } else { 1.0 };
                    self.opts.insert(k1.into(), v);
                    if !k2.is_empty() { self.opts.insert(k2.into(), v); }
                    if let Some(n) = self.nodes.get_mut(node).and_then(|n| n.as_mut()) { n.frame = if v != 0.0 { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }.into(); }
                    self.events.push(Action::OptChanged);
                }
            }
            Action::OptFocus(i) => { self.commit_input(); self.opt_focus = Some(i); self.opt_str.clear(); self.show_input(); }
            Action::Popup(name) if name != "Options" => { if !self.pa_open(name) { self.pb_open(name); } }
            Action::Info => { self.pa_open("Credits"); }
            Action::News => { self.pa_open("News"); }
            Action::Stats => { self.pb_open("Stats"); }
            Action::Vault => { self.pb_open("Vault"); }
            Action::Creator => { self.pe_open("Creator"); }
            Action::Pop(name, arg) => { if !self.pa_action(name, arg) && !self.gar_action(name, arg) && !self.pb_action(name, arg) && !self.pe_action(name, arg) && !self.po_action(name, arg) { self.ed_action(name, arg); } }
            Action::InfoBox(t) => self.open_info(t),
            Action::InfoClose => self.kill_tag("info_"),
            Action::ToggleMenuMusic => {
                self.menu_music = !self.menu_music;
                let f = if self.menu_music { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" };
                if let Some(n) = self.nodes.get_mut(self.set_check).and_then(|n| n.as_mut()) { n.frame = f.into(); }
            }
            _ => {}
        }
        self.events.push(a);
    }

    pub(crate) fn drag_thumb(&mut self, i: usize, px: f64) {
        let Some(n) = self.get(i) else { return };
        let Some((lo, hi, fill, which, grab)) = n.drag else { return };
        let x = (px + grab).max(lo).min(hi);
        let pct = (x - lo) / (hi - lo);
        self.node(i).x = x;
        { let f = self.node(fill); f.crop_w = (pct * (hi - lo)).max(1.0); }
        if which == 9 { self.ed_timeline_drag(pct); return; }
        if which >= 2 { self.pb_drag(which, pct); return; }
        let v = if pct < 0.03 { 0.0 } else { pct };
        if which == 0 { self.music_vol = v; } else { self.sfx_vol = v; }
        self.events.push(Action::Volume);
    }

    // ---- settings screen (`_showSettingsScreen`) ----
    pub fn open_settings(&mut self) {
        if self.set_open { return; }
        self.set_open = true; self.set_closing = false; self.set_clock = 0.0; self.set_p = 0.0;
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let hud: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "hud_pause")).collect();
        for i in hud { self.tween_to(i, &[(Prop::Alpha, 0.0)], 300.0, Ease::Linear, Then::Nothing); }
        let ov = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.0, 200.0, "set_ov");
        { let n = self.node(ov); n.interactive = true; n.width = sw; n.height = 640.0; }
        self.tween_to(ov, &[(Prop::Alpha, 180.0 / 255.0)], 400.0, Ease::Linear, Then::Nothing);
        self.set_overlay = ov;
        let (pw, ph) = (712.0, 460.0);
        let a = (sw - pw) / 2.0;
        let d = 201.0;
        self.shape(Kind::Rect, a, 80.0, pw, ph, 0, 180.0 / 255.0, d, "set_scr");
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = if side_h > 0.0 { ph / side_h } else { 1.0 };
        let l = self.image("GJ_WebSheet", "GJ_table_side_001.png", a - 40.0, 80.0, d, 1.0); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; n.tag = "set_scr"; }
        let r = self.image("GJ_WebSheet", "GJ_table_side_001.png", a + pw + 40.0, 80.0, d, 1.0); { let n = self.node(r); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; n.tag = "set_scr"; }
        let top = self.image("GJ_WebSheet", "GJ_table_top_001.png", a + 356.0, 70.0, d, 1.0); self.node(top).tag = "set_scr";
        let bot = self.image("GJ_WebSheet", "GJ_table_bottom_001.png", a + 356.0, 560.0, d, 1.0); self.node(bot).tag = "set_scr";
        for dx in [-312.0, 312.0] { let c = self.image("GJ_WebSheet", "chain_01_001.png", cx + dx, 35.0, d, 1.0); { let n = self.node(c); n.oy = 1.0; n.tag = "set_scr"; } }
        let t = self.text_node("bigFont", 55.0, "Settings", cx, 65.0, d, 0.5, 0.5, 1.0); self.node(t).tag = "set_scr";

        let tex_w = self.sizes.get("/GJ_button01").map_or(80.0, |s| s.0);
        let (border, bh, w2, w3, gap) = (tex_w * 0.3, 62.0, 310.0, 200.0, 18.0);
        let buttons: [(f64, f64, &str, f64, Option<Action>); 7] = [
            (cx - w2 / 2.0 - gap / 2.0, 155.0, "Account", w2, None),
            (cx + w2 / 2.0 + gap / 2.0, 155.0, "How To Play", w2, Some(Action::Popup("HowToPlay"))),
            (cx - w2 / 2.0 - gap / 2.0, 235.0, "Options", w2, Some(Action::Popup("Options"))),
            (cx + w2 / 2.0 + gap / 2.0, 235.0, "Graphics", w2, None),
            (cx - w3 - gap, 312.0, "Rate", w3, Some(Action::OpenUrl("https://github.com/web-dashers/web-dashers.github.io"))),
            (cx, 312.0, "Songs", w3, Some(Action::Popup("Songs"))),
            (cx + w3 + gap, 312.0, "Help", w3, Some(Action::Popup("Help"))),
        ];
        for (x, y, label, w, action) in buttons {
            let active = action.is_some();
            let zone = self.shape(Kind::Rect, x, y, w, bh, 0, 0.0, d, "set_scr");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = bh; }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, w, bh, 0, 1.0, d, "set_scr");
            { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = border; n.ox = 0.5; n.oy = 0.5; n.tint = if active { 0xffffff } else { 0x666666 }; n.parent = Some(zone); }
            let fs = if label == "How To Play" { 41.0 } else { 50.0 };
            let lb = self.text_node("goldFont", fs, label, 0.0, -5.0, d, 0.5, 0.5, 1.0);
            { let n = self.node(lb); n.tag = "set_scr"; n.parent = Some(zone); if !active { n.color = 0x666666; } }
            if let Some(a) = action { self.make_bouncy(zone, 1.0, Cond::Always, a); }
        }
        let lock = self.image("GJ_GameSheet03", "GJ_lock_open_001.png", cx + 535.0, 30.0, d, 0.9);
        self.node(lock).hit_scale = 1.5; self.make_bouncy(lock, 0.9, Cond::Always, Action::Vault); self.node(lock).tag = "set_scr";
        let groove_w = self.sizes.get("GJ_WebSheet/slidergroove.png").map_or(420.0, |s| s.0);
        let sc = 0.7 * 1.3;
        for (which, pos_y, label, val) in [(0u8, 415.0, "Music", self.music_vol), (1u8, 490.0, "SFX", self.sfx_vol)] {
            let t = self.text_node("bigFont", 33.0, label, cx, pos_y - 37.0, d, 0.5, 0.5, 1.0); self.node(t).tag = "set_scr";
            let bar_max = (groove_w - 8.0) * sc;
            let bar_x = cx - bar_max / 2.0 + 2.8;
            let fill_w = val * bar_max;
            let fill = self.image("", "sliderBar", bar_x, pos_y, d, 1.0);
            { let n = self.node(fill); n.ox = 0.0; n.oy = 0.5; n.crop_w = if fill_w > 0.0 { fill_w } else { 1.0 }; n.tag = "set_scr"; }
            let g = self.image("GJ_WebSheet", "slidergroove.png", cx, pos_y, d, sc); self.node(g).tag = "set_scr";
            let th = self.image("GJ_WebSheet", "sliderthumb.png", bar_x + fill_w, pos_y, d, sc);
            { let n = self.node(th); n.tag = "set_scr"; n.interactive = true; n.drag = Some((bar_x, bar_x + bar_max, fill, which, 0.0)); }
        }
        let (kx, ky) = (cx + 280.0, 420.0);
        let t = self.text_node("bigFont", 20.0, "Menu", kx, ky - 42.0, d, 0.5, 0.5, 1.0); self.node(t).tag = "set_scr";
        let t = self.text_node("bigFont", 20.0, "Music", kx, ky - 22.0, d, 0.5, 0.5, 1.0); self.node(t).tag = "set_scr";
        let f = if self.menu_music { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" };
        let ck = self.image("GJ_GameSheet03", f, kx, ky + 15.0, d, 0.7);
        self.make_bouncy(ck, 0.8, Cond::Always, Action::ToggleMenuMusic); self.node(ck).tag = "set_scr";
        self.set_check = ck;
        let back = self.image("GJ_GameSheet03", "GJ_arrow_03_001.png", cx - 535.0, 30.0, d, 1.0);
        self.make_bouncy(back, 1.0, Cond::Always, Action::SettingsClose); self.node(back).tag = "set_scr";
    }

    pub fn close_settings(&mut self) {
        if !self.set_open || self.set_closing { return; }
        self.set_closing = true; self.set_clock = 0.0;
        let ov = self.set_overlay;
        self.tween_to(ov, &[(Prop::Alpha, 0.0)], 250.0, Ease::Linear, Then::Nothing);
    }

    pub(crate) fn finish_settings_close(&mut self) {
        self.kill_tag("set_");
        self.set_open = false; self.set_closing = false; self.set_p = 0.0;
        // the original fades the pause button back in to full opacity
        let hud: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "hud_pause")).collect();
        for i in hud { self.tween_to(i, &[(Prop::Alpha, 1.0)], 300.0, Ease::Linear, Then::Nothing); }
    }

    // ---- options popup (`_buildSettingsPopup`) ----
    pub(crate) fn show_input(&mut self) {
        let Some(i) = self.opt_focus else { return };
        let Some(def) = self.opt_inputs.get(i).cloned() else { return };
        let t = if self.opt_str.is_empty() && false { " ".to_string() } else { format!("{}|", self.opt_str) };
        if let Some(n) = self.nodes.get_mut(def.text).and_then(|n| n.as_mut()) { n.text = t; }
    }

    pub(crate) fn commit_input(&mut self) {
        let Some(i) = self.opt_focus.take() else { return };
        let Some(def) = self.opt_inputs.get(i).cloned() else { return };
        let mut v = if def.integer { self.opt_str.parse::<f64>().ok().map(|v| v.trunc()) } else { self.opt_str.parse::<f64>().ok() }.unwrap_or(def.min);
        if v.is_nan() { v = def.min; }
        v = v.max(def.min).min(def.max);
        if def.integer { v = v.round(); }
        if let Some(n) = self.nodes.get_mut(def.text).and_then(|n| n.as_mut()) { n.text = fmt_num(v); }
        self.opts.insert(def.key.into(), v);
        self.events.push(Action::OptChanged);
    }

    /// A typed character for the focused number box: digits and '.', backspace (8), enter (13).
    pub fn key_input(&mut self, code: u32) {
        if self.ed.on && self.ed_key(code) { return; }
        if self.on.input { self.on_key(code); return; }
        if self.gar.open { self.gar_key(code); return; }
        let Some(i) = self.opt_focus else { return };
        let integer = self.opt_inputs.get(i).map_or(true, |d| d.integer);
        match code {
            13 => self.commit_input(),
            8 => { self.opt_str.pop(); self.show_input(); }
            46 => { if !integer && !self.opt_str.is_empty() && !self.opt_str.contains('.') { self.opt_str.push('.'); self.show_input(); } }
            48..=57 => { self.opt_str.push(char::from_u32(code).unwrap()); self.show_input(); }
            _ => {}
        }
    }

    pub fn open_options(&mut self) {
        if self.opt_open { return; }
        self.opt_open = true; self.opt_page = 0; self.opt_focus = None;
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 150.0 / 255.0, 250.0, "opt_dim");
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        let root = self.shape(Kind::Rect, cx, 320.0, 0.0, 0.0, 0, 0.0, 250.0, "opt_s");
        { let n = self.node(root); n.sx = 0.0; n.sy = 0.0; }
        self.opt_root = root;
        let (pw, ph) = (800.0, 550.0);
        let tex = self.sizes.get("/GJ_square67").map_or(80.0, |s| s.0);
        let panel = self.shape(Kind::Nine, 0.0, 0.0, pw, ph, 0, 1.0, 250.5, "opt_s");
        { let n = self.node(panel); n.frame = "GJ_square67".into(); n.radius = (tex * 0.325).min(tex / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); }
        let close = self.image("GJ_WebSheet", "GJ_closeBtn_001.png", -pw / 2.0 + 10.0, -ph / 2.0 + 10.0, 250.6, 0.8);
        { let n = self.node(close); n.parent = Some(root); n.tag = "opt_s"; }
        self.make_bouncy(close, 0.8, Cond::Always, Action::OptClose);
        let tab_y = -ph / 2.0 + 45.0;
        let btn_tex = self.sizes.get("/GJ_button05").map_or(80.0, |s| s.0);
        let border = btn_tex * 0.3;
        self.opt_tabs.clear();
        let mut left = -pw / 2.0 + 50.0;
        for (i, (name, w)) in [("Gameplay", 150.0), ("Visual", 120.0), ("Performance", 185.0)].into_iter().enumerate() {
            let zone = self.shape(Kind::Rect, left + w / 2.0, tab_y, w, 56.0, 0, 0.0, 250.6, "opt_s");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = 56.0; n.parent = Some(root); }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, w, 56.0, 0, 1.0, 250.6, "opt_s");
            { let n = self.node(nine); n.frame = if i == 0 { "GJ_button02" } else { "GJ_button05" }.into(); n.radius = border; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
            let lbl = self.text_node("bigFont", 25.0, name, 0.0, -2.0, 250.7, 0.5, 0.5, 1.0);
            { let tw = self.node(lbl).width; let n = self.node(lbl); n.tag = "opt_s"; n.parent = Some(zone); if tw > w - 24.0 { let k = (w - 24.0) / tw; n.sx = k; n.sy = k; } }
            self.make_bouncy(zone, 1.0, Cond::Always, Action::OptTab(i));
            self.opt_tabs.push(nine);
            left += w + 5.0;
        }
        self.build_opt_page();
        self.tween_to(root, &[(Prop::ScaleX, 1.0), (Prop::ScaleY, 1.0)], 660.0, Ease::ElasticOut, Then::Nothing);
    }

    pub(crate) fn build_opt_page(&mut self) {
        self.kill_tag("opt_pg");
        self.opt_toggles.clear(); self.opt_inputs.clear(); self.opt_focus = None;
        let root = self.opt_root;
        let (c1, c2, sp, y0, tab_y) = (-200.0, 200.0, 70.0, -150.0, -275.0 + 45.0);
        for (i, &nine) in self.opt_tabs.clone().iter().enumerate() {
            if let Some(n) = self.nodes.get_mut(nine).and_then(|n| n.as_mut()) { n.frame = if i == self.opt_page { "GJ_button02" } else { "GJ_button05" }.into(); }
        }
        match self.opt_page {
            0 => {
                for (r, d) in GAMEPLAY_L.iter().enumerate() { self.opt_toggle(root, c1, y0 + sp * r as f64, d); }
                self.opt_toggle(root, c2, y0 + sp, &("Practice Music Sync", "practiceMusicSync", "", 20.0, Some("Practice Music Bypass")));
                self.opt_toggle(root, c2, y0 + sp * 3.0, &("Hide Pause Button", "hidePauseBtn", "", 25.0, Some("Hide Pause Button")));
                self.opt_toggle(root, c2, y0 + sp * 4.0, &("Hide Practice UI", "hidePracticeUI", "", 25.0, Some("Hide Practice UI")));
                self.opt_number(root, c2, y0 + sp * 2.0, "Respawn Delay (s)", "respawnDelay", 0.0, 10.0, false, None);
                self.opt_number(root, c2, y0, "Speedhack", "speedHack", 0.1, 10.0, false, None);
            }
            1 => {
                for (r, d) in VISUAL_L.iter().enumerate() { self.opt_toggle(root, c1, y0 + sp * r as f64, d); }
                for (r, d) in VISUAL_R.iter().enumerate() { self.opt_toggle(root, c2, y0 + sp * r as f64, d); }
            }
            _ => self.opt_number(root, c1, y0, "Cull Distance", "cullDistance", 0.0, 3.0, true, Some("Cull Distance")),
        }
        let btn_tex = self.sizes.get("/GJ_button05").map_or(80.0, |s| s.0);
        for (label, x, w, fs, popup) in [("StartPos", 215.0, 130.0, 21.0, "StartPos"), ("Keys", 335.0, 90.0, 25.0, "Keys")] {
            let zone = self.shape(Kind::Rect, x, tab_y, w, 56.0, 0, 0.0, 250.6, "opt_pg");
            { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = w; n.height = 56.0; n.parent = Some(root); }
            let nine = self.shape(Kind::Nine, 0.0, 0.0, w, 56.0, 0, 1.0, 250.6, "opt_pg");
            { let n = self.node(nine); n.frame = "GJ_button05".into(); n.radius = btn_tex * 0.3; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
            let lbl = self.text_node("goldFont", fs, label, 0.0, -2.0, 250.7, 0.5, 0.5, 1.0);
            { let n = self.node(lbl); n.tag = "opt_pg"; n.parent = Some(zone); }
            self.make_bouncy(zone, 1.0, Cond::Always, Action::Popup(popup));
        }
    }

    pub(crate) fn opt_info_button(&mut self, root: usize, x: f64, y: f64, key: &'static str) {
        let text = INFO.iter().find(|(k, _)| *k == key).map(|(_, v)| *v).unwrap_or(key);
        let b = self.image("GJ_GameSheet03", "GJ_infoIcon_001.png", x, y, 250.6, 0.45);
        { let n = self.node(b); n.parent = Some(root); n.tag = "opt_pg"; }
        self.make_bouncy(b, 0.45, Cond::Always, Action::InfoBox(text));
    }

    pub(crate) fn opt_toggle(&mut self, root: usize, x: f64, y: f64, d: &ToggleDef) {
        let (label, k1, k2, fs, info) = *d;
        let on = self.opts.get(k1).copied().unwrap_or(0.0) != 0.0;
        let check = self.image("GJ_GameSheet03", if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, x - 120.0, y, 250.6, 0.8);
        { let n = self.node(check); n.parent = Some(root); n.tag = "opt_pg"; }
        let t = self.text_node("bigFont", fs, label, x - 70.0, y, 250.6, 0.0, 0.5, 1.0);
        { let n = self.node(t); n.parent = Some(root); n.tag = "opt_pg"; }
        if let Some(k) = info { self.opt_info_button(root, x - 120.0 - 34.0, y - 30.0, k); }
        let idx = self.opt_toggles.len();
        self.opt_toggles.push((check, k1, k2));
        self.make_bouncy(check, 0.8, Cond::Always, Action::OptToggle(idx));
    }

    pub(crate) fn opt_number(&mut self, root: usize, x: f64, y: f64, label: &'static str, key: &'static str, min: f64, max: f64, integer: bool, info: Option<&'static str>) {
        let t = self.text_node("bigFont", 25.0, label, x - 70.0, y, 250.6, 0.0, 0.5, 1.0);
        { let n = self.node(t); n.parent = Some(root); n.tag = "opt_pg"; }
        if let Some(k) = info { self.opt_info_button(root, x - 120.0 - 38.0, y - 32.0, k); }
        let (bx, by, bw, bh) = (x - 120.0, y, 64.0, 48.0);
        let bg = self.shape(Kind::RoundRect, bx - bw / 2.0, by - bh / 2.0, bw, bh, 0x222222, 0.5, 250.6, "opt_pg");
        { let n = self.node(bg); n.radius = 8.0; n.parent = Some(root); }
        let v = self.opts.get(key).copied().unwrap_or(min);
        let val = self.text_node("bigFont", 28.0, &fmt_num(v), bx, by, 250.7, 0.5, 0.5, 1.0);
        { let n = self.node(val); n.parent = Some(root); n.tag = "opt_pg"; }
        let idx = self.opt_inputs.len();
        self.opt_inputs.push(InputDef { text: val, key, min, max, integer });
        let hit = self.shape(Kind::Rect, bx, by, bw, bh, 0, 0.0, 250.8, "opt_pg");
        { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.width = bw; n.height = bh; n.interactive = true; n.action = Action::OptFocus(idx); n.parent = Some(root); }
    }

    /// `InfoBoxDoAThing`
    pub fn open_info(&mut self, text: &str) {
        self.kill_tag("info_");
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        let dim = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.5, 1000.0, "info_dim");
        { let n = self.node(dim); n.interactive = true; n.width = sw; n.height = 640.0; }
        let root = self.shape(Kind::Rect, cx, 320.0, 0.0, 0.0, 0, 0.0, 1000.0, "info_s");
        { let n = self.node(root); n.sx = 0.0; n.sy = 0.0; }
        let tex = self.sizes.get("/GJ_square01").or(self.sizes.get("/square01_001")).map_or(80.0, |s| s.0);
        let (bw, bh) = (720.0, 280.0);
        let panel = self.shape(Kind::Nine, 0.0, 0.0, bw, bh, 0, 1.0, 1000.5, "info_s");
        { let n = self.node(panel); n.frame = "square01_001".into(); n.radius = (tex * 0.325).min(tex / 2.0); n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(root); }
        let t = self.text_node("goldFont", 45.0, "Info", -4.0, -90.0, 1000.6, 0.5, 0.5, 1.0);
        { let n = self.node(t); n.parent = Some(root); n.tag = "info_s"; }
        // word wrap at 40 characters
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for word in text.split(' ') {
            let joined = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
            if joined.chars().count() <= 40 { line = joined; } else { if !line.is_empty() { lines.push(line.clone()); } line = word.to_string(); }
        }
        if !line.is_empty() { lines.push(line); }
        let mut y = -30.0;
        for l in lines {
            let t = self.text_node("arial", 30.0, &l, 0.0, y, 1000.6, 0.5, 0.0, 1.0);
            { let n = self.node(t); n.parent = Some(root); n.tag = "info_s"; }
            y += 32.0;
        }
        let btn_tex = self.sizes.get("/GJ_button01").map_or(80.0, |s| s.0);
        let zone = self.shape(Kind::Rect, 0.0, bh / 2.0 - 50.0, 100.0, 60.0, 0, 0.0, 1000.6, "info_s");
        { let n = self.node(zone); n.ox = 0.5; n.oy = 0.5; n.width = 100.0; n.height = 60.0; n.parent = Some(root); }
        let nine = self.shape(Kind::Nine, 0.0, 0.0, 100.0, 60.0, 0, 1.0, 1000.6, "info_s");
        { let n = self.node(nine); n.frame = "GJ_button01".into(); n.radius = btn_tex * 0.3; n.ox = 0.5; n.oy = 0.5; n.tint = 0xffffff; n.parent = Some(zone); }
        let ok = self.text_node("goldFont", 48.0, "OK", 0.0, 0.0, 1000.7, 0.55, 0.55, 1.0);
        { let n = self.node(ok); n.parent = Some(zone); n.tag = "info_s"; }
        self.make_bouncy(zone, 1.0, Cond::Always, Action::InfoClose);
        self.tween_to(root, &[(Prop::ScaleX, 1.0), (Prop::ScaleY, 1.0)], 660.0, Ease::ElasticOut, Then::Nothing);
    }

    // ---- level complete ----
    /// The "Level Complete" banner that pops in (`_showCompleteText`)
    pub fn show_complete_banner(&mut self, practice: bool) {
        let cx = self.screen_width / 2.0;
        let (atlas, frame) = if practice { ("GJ_GameSheet03", "GJ_practiceComplete_001.png") } else { ("GJ_WebSheet", "GJ_levelComplete_001.png") };
        let b = self.image(atlas, frame, cx, 250.0, 60.0, 0.01);
        self.node(b).tag = "banner";
        self.tween_to(b, &[(Prop::ScaleX, 1.1), (Prop::ScaleY, 1.1)], 660.0, Ease::ElasticOut, Then::Nothing);
        self.end_clock = 0.0;
        self.end_open = false;
    }

    /// `_showEndLayer`
    pub fn open_end_layer(&mut self, attempts: u32, jumps: u32, seconds: f64, practice: bool, message: &str) {
        if self.end_open { return; }
        self.end_open = true;
        self.end_p = 0.0;
        let (sw, cx) = (self.screen_width, self.screen_width / 2.0);
        self.kill_tag("banner");
        let ov = self.shape(Kind::Rect, 0.0, 0.0, sw, 640.0, 0, 0.0, 200.0, "end_overlay");
        { let n = self.node(ov); n.interactive = true; n.width = sw; n.height = 640.0; }
        self.tween_to(ov, &[(Prop::Alpha, 100.0 / 255.0)], 1000.0, Ease::Linear, Then::Nothing);
        let (pw, ph) = (712.0, 460.0);
        let a = (sw - pw) / 2.0;
        let r = self.shape(Kind::Rect, a, 80.0, pw, ph, 0, 180.0 / 255.0, 201.0, "end");
        let _ = r;
        let side_h = self.sizes.get("GJ_WebSheet/GJ_table_side_001.png").map_or(1.0, |s| s.1);
        let k = if side_h > 0.0 { ph / side_h } else { 1.0 };
        let l = self.image("GJ_WebSheet", "GJ_table_side_001.png", a - 40.0, 80.0, 201.0, 1.0); { let n = self.node(l); n.ox = 0.0; n.oy = 0.0; n.sy = k; n.tag = "end"; }
        let rr = self.image("GJ_WebSheet", "GJ_table_side_001.png", a + pw + 40.0, 80.0, 201.0, 1.0); { let n = self.node(rr); n.ox = 1.0; n.oy = 0.0; n.sy = k; n.flip_x = true; n.tag = "end"; }
        let top = self.image("GJ_WebSheet", "GJ_table_top_001.png", a + 356.0, 70.0, 201.0, 1.0); self.node(top).tag = "end";
        let bot = self.image("GJ_WebSheet", "GJ_table_bottom_001.png", a + 356.0, 560.0, 201.0, 1.0); self.node(bot).tag = "end";
        for dx in [-312.0, 312.0] { let c = self.image("GJ_WebSheet", "chain_01_001.png", cx + dx, 35.0, 201.0, 1.0); { let n = self.node(c); n.oy = 1.0; n.tag = "end"; } }
        let (ba, bf) = if practice { ("GJ_GameSheet03", "GJ_practiceComplete_001.png") } else { ("GJ_WebSheet", "GJ_levelComplete_001.png") };
        let banner = self.image(ba, bf, cx, 170.0, 201.0, 0.85); self.node(banner).tag = "end";
        let mut y = 250.0;
        let t = self.text_node("goldFont", 46.0, &format!("Attempts: {}", attempts), cx, y, 201.5, 0.5, 0.5, 0.8); self.node(t).tag = "end"; y += 48.0;
        let t = self.text_node("goldFont", 46.0, &format!("Jumps: {}", jumps), cx, y, 201.5, 0.5, 0.5, 0.8); self.node(t).tag = "end"; y += 48.0;
        let s = seconds.floor() as i64;
        let ts = if s / 3600 > 0 { format!("{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60) } else { format!("{:02}:{:02}", s % 3600 / 60, s % 60) };
        let t = self.text_node("goldFont", 46.0, &format!("Time: {}", ts), cx, y, 201.5, 0.5, 0.5, 0.8); self.node(t).tag = "end";
        self.ufx.practice = practice;
        self.end_coin_layout(cx, y);
        let with_coins = self.ufx.coin_count > 0;
        let m = if with_coins { self.text_node("bigFont", 40.0, message, cx + 235.0, y, 201.5, 0.5, 0.5, 0.8) } else { self.text_node("bigFont", 46.0, &message.replace('\n', " "), cx, y + 96.0, 201.5, 0.5, 0.5, 0.9) };
        self.node(m).tag = "end";
        for (frame, dx, action) in [("GJ_replayBtn_001.png", -200.0, Action::PauseReplay), ("GJ_menuBtn_001.png", 200.0, Action::PauseMenu)] {
            let b = self.image("GJ_WebSheet", frame, cx + dx, 555.0, 201.5, 1.0);
            self.make_bouncy(b, 1.0, Cond::Always, action); self.node(b).tag = "end";
        }
    }

    pub fn close_end_layer(&mut self) { self.ufx_reset(); self.kill_tag("end"); self.kill_tag("banner"); self.end_open = false; self.end_p = 0.0; }

    pub fn close_pause(&mut self) { self.kill_tag("pause"); self.paused = false; }

    /// `_startGame`: everything slides/shrinks away; the game proper starts afterwards.
    /// `_startGame` after the scene restart (`_instantLevelStart`): the menu pieces are destroyed on the spot, no slide-out tweens
    pub fn start_game_dismiss(&mut self) {
        if !self.menu_active { return; }
        self.menu_active = false;
        self.starting = true;
        let (sh, _sw) = (self.screen_height, self.screen_width);
        for i in 0..self.nodes.len() {
            let Some(n) = self.get(i) else { continue };
            let (tag, height, y) = (n.tag, n.height, n.y);
            match tag {
                "play" | "garage" | "creator" => self.nodes[i] = None,
                "decor" => self.nodes[i] = None,
                "rob" | "news" => self.nodes[i] = None,
                "copyright" => self.nodes[i] = None,
                "fs" | "info" | "beta" | "logo" => self.nodes[i] = None,
                "social" => self.nodes[i] = None,
                "settings" | "stats" | "ach" | "ng" => { self.node(i).visible = false; let _ = y; }
                _ => {}
            }
        }
    }

    /// Flat buffer for the host: per visible node (depth sorted): kind, x, y, sx, sy, rot, alpha, tint, flags, ox, oy, text index, extra, atlas/frame index.
    pub fn build(&mut self, frames: &mut Vec<(String, String)>) -> usize {
        let mut order: Vec<(f64, usize)> = self.nodes.iter().enumerate().filter_map(|(i, n)| n.as_ref().filter(|n| n.visible).map(|n| (n.depth, i))).collect();
        order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.cmp(&b.1)));
        self.buf.clear();
        self.strings.clear();
        for (_, i) in order {
            let n = self.nodes[i].as_ref().unwrap();
            let fi = if n.kind == Kind::Image { (match frames.iter().position(|f| f.0 == n.atlas && f.1 == n.frame) { Some(p) => p, None => { frames.push((n.atlas.clone(), n.frame.clone())); frames.len() - 1 } }) as f64 } else { -1.0 };
            let ti = if n.kind == Kind::Text { self.strings.push(n.text.clone()); (self.strings.len() - 1) as f64 } else if n.kind == Kind::Nine { self.strings.push(n.frame.clone()); (self.strings.len() - 1) as f64 } else { -1.0 };
            let yoff = self.yoff(n.tag);
            let (mut ex, mut ey, mut esx, mut esy) = self.eff(i);
            let (mut ew, mut eh, mut er) = (n.w, n.h, n.radius);
            if matches!(n.tag, "ls_card" | "ls_bar") { self.ls_xform(n, &mut ex, &mut ey, &mut esx, &mut esy, &mut ew, &mut eh, &mut er); }
            if matches!(n.kind, Kind::Nine | Kind::Rect | Kind::RoundRect | Kind::Clip) && n.parent.is_some() { ew *= esx; eh *= esy; er *= esx; ex -= ew * n.ox; ey -= eh * n.oy; esx = 1.0; esy = 1.0; }
            self.buf.extend_from_slice(&[
                match n.kind { Kind::Image => 0.0, Kind::Text => 1.0, Kind::Rect => 2.0, Kind::RoundRect => 3.0, Kind::Circle => 4.0, Kind::Nine => 5.0, Kind::Clip => 6.0, Kind::ClipEnd => 7.0 },
                ex, ey + yoff, esx, esy, n.rot, n.alpha, n.tint as f64, (n.flip_x as u8 | (n.flip_y as u8) << 1 | (n.additive as u8) << 2 | (n.world as u8) << 3 | (n.gray as u8) << 4) as f64, n.ox, n.oy,
                ti, match n.kind { Kind::Text => n.font_size, Kind::Image => n.crop_w, _ => ew }, if n.kind == Kind::Image { fi } else { eh }, if n.kind == Kind::Nine { n.tint as f64 } else { n.color as f64 }, if n.kind == Kind::Text { match n.font.as_str() { "bigFont" => 1.0, "goldFont" => 2.0, _ => 0.0 } } else { er },
            ]);
        }
        self.buf.len() / 16
    }
}
