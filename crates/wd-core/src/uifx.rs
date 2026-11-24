//! Effects of the level-complete screen: the star and secret/user coin awards (`_playStarAward`, `_playsecretcoinanimation`,
//! `_playusercoinanimation`), the particle bursts and expanding glow circles that go with them, and the delayed sounds.
use crate::ui::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FxAct {
    Star,
    StarBurst(f64, f64),
    Coin(usize),
    CoinBurst(usize),
}

#[derive(Clone, Debug)]
pub struct Part { pub node: usize, pub vx: f64, pub vy: f64, pub life: f64, pub age: f64, pub s0: f64, pub s1: f64 }

#[derive(Clone, Debug)]
pub struct Glow { pub node: usize, pub age: f64, pub r0: f64, pub grow: f64 }

#[derive(Default)]
pub struct UiFx {
    pub timers: Vec<(f64, FxAct)>,
    pub parts: Vec<Part>,
    pub glows: Vec<Glow>,
    /// 1 = main level (secret coins), 0 = other level (user coins)
    pub coin_main: bool,
    pub coin_count: u32,
    pub run_mask: u32,
    pub saved_mask: u32,
    pub practice: bool,
    pub star: (f64, f64),
    pub coin_pos: Vec<(f64, f64)>,
    pub awards_started: bool,
    pub seed: u64,
}

impl UiFx {
    fn rand(&mut self) -> f64 {
        if self.seed == 0 { self.seed = 0x9E3779B97F4A7C15; }
        let mut x = self.seed;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.seed = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }
}

impl Ui {
    /// Coins shown on the end screen: how many the level has (max 3), which were picked up this run and which were saved earlier (bit per slot).
    pub fn set_end_coins(&mut self, main: bool, count: u32, run_mask: u32, saved_mask: u32) {
        self.ufx.coin_main = main; self.ufx.coin_count = count.min(3); self.ufx.run_mask = run_mask; self.ufx.saved_mask = saved_mask;
    }

    /// The coin slots of the end screen and the star position; draws the empty/saved coin placeholders. Called by `open_end_layer`.
    pub fn end_coin_layout(&mut self, cx: f64, time_y: f64) {
        let count = self.ufx.coin_count.min(3) as usize;
        let offsets: &[f64] = match count { 0 => &[], 1 => &[0.0], 2 => &[-60.0, 60.0], _ => &[-120.0, 0.0, 120.0] };
        self.ufx.coin_pos = offsets.iter().map(|o| (cx + o, 445.0)).collect();
        self.ufx.star = (cx + 235.0, time_y - 77.5);
        self.ufx.awards_started = false;
        self.ufx.timers.clear();
        let (main, run, saved) = (self.ufx.coin_main, self.ufx.run_mask, self.ufx.saved_mask);
        let pos = self.ufx.coin_pos.clone();
        for (slot, &(x, y)) in pos.iter().enumerate() {
            let (is_run, is_saved) = (run >> slot & 1 != 0, saved >> slot & 1 != 0);
            if is_saved && !is_run { continue; }
            if main {
                let c = self.image("GJ_GameSheet02", "secretCoin_b_01_001.png", x, y, 201.5, 1.1);
                self.node(c).tag = "end";
            } else {
                let c = self.image("GJ_GameSheet02", "secretCoin_2_b_01_001.png", x, y, 201.5, 1.0);
                self.node(c).tag = "end";
            }
        }
        if main {
            for (slot, &(x, y)) in pos.iter().enumerate() {
                if saved >> slot & 1 != 0 && run >> slot & 1 == 0 {
                    let c = self.image("GJ_GameSheet03", "secretCoinUI_001.png", x, y, 201.5, 1.1);
                    self.node(c).tag = "end";
                }
            }
        }
        // (a saved *user* coin is drawn from a texture that does not exist in the original, so nothing is shown)
    }

    /// Called once the end layer has finished dropping in (`onComplete` of the 1 s bounce tween).
    fn start_awards(&mut self) {
        self.ufx.awards_started = true;
        let main = self.ufx.coin_main;
        let slots: Vec<usize> = (0..self.ufx.coin_pos.len()).filter(|s| self.ufx.run_mask >> s & 1 != 0).collect();
        if !main || !self.ufx.practice {
            for (i, slot) in slots.iter().enumerate() { self.ufx.timers.push((i as f64 * 375.0, FxAct::Coin(*slot))); }
        }
        self.ufx.timers.push((250.0, FxAct::Star));
    }

    fn fx_burst(&mut self, x: f64, y: f64, tint: i64, s0: f64, glow_r0: f64, glow_grow: f64) {
        for _ in 0..30 {
            let speed = 200.0 + self.ufx.rand() * 400.0;
            let ang = self.ufx.rand() * std::f64::consts::TAU;
            let life = 200.0 + self.ufx.rand() * 400.0;
            let n = self.image("GJ_WebSheet", "square.png", x, y, 202.0, s0);
            { let nd = self.node(n); nd.additive = true; nd.tint = tint; nd.tag = "fxp"; }
            self.ufx.parts.push(Part { node: n, vx: ang.cos() * speed, vy: ang.sin() * speed, life, age: 0.0, s0, s1: 0.0 });
        }
        let g = self.shape(Kind::Circle, x, y, glow_r0, glow_r0, tint, 1.0, 202.0, "fxp");
        self.node(g).additive = true;
        self.ufx.glows.push(Glow { node: g, age: 0.0, r0: glow_r0, grow: glow_grow });
    }

    fn fire(&mut self, a: FxAct) {
        match a {
            FxAct::Star => {
                if !self.end_open { return; }
                let (x, y) = self.ufx.star;
                let star = self.image("GJ_WebSheet", "GJ_bigStar_001.png", x, y, 201.6, 3.0);
                { let n = self.node(star); n.alpha = 0.0; n.tag = "end"; }
                self.tween_to(star, &[(Prop::ScaleX, 0.8), (Prop::ScaleY, 0.8), (Prop::Alpha, 1.0)], 300.0, Ease::BounceOut, Then::Nothing);
                self.ufx.timers.push((125.0, FxAct::StarBurst(x, y)));
            }
            FxAct::StarBurst(x, y) => {
                if !self.end_open { return; }
                self.events.push(Action::Sfx("gold02"));
                let ey = y + (self.end_p * 650.0 - 640.0);
                self.fx_burst(x, ey, 0xffff00, 0.5, 20.0, 200.0);
            }
            FxAct::Coin(slot) => {
                if !self.end_open { return; }
                let (x, y) = self.ufx.coin_pos[slot];
                let frame = if self.ufx.coin_main { "secretCoinUI_001.png" } else { "secretCoinUI2_001.png" };
                let coin = self.image("GJ_GameSheet03", frame, x, y, 201.6, 3.0);
                { let n = self.node(coin); n.alpha = 0.0; n.tag = "end"; }
                self.events.push(Action::Sfx("highscoreGet02"));
                self.tween_to(coin, &[(Prop::ScaleX, 1.1), (Prop::ScaleY, 1.1), (Prop::Alpha, 1.0)], 300.0, Ease::BounceOut, Then::Nothing);
                self.ufx.timers.push((100.0, FxAct::CoinBurst(slot)));
            }
            FxAct::CoinBurst(slot) => {
                if !self.end_open { return; }
                let (x, y) = self.ufx.coin_pos[slot];
                let ey = y + (self.end_p * 650.0 - 640.0);
                self.fx_burst(x, ey, 0xd0d0d0, 0.35, 12.0, 130.0);
            }
        }
    }

    /// The pause button of the HUD: hidden during the slide-in, then it fades in to 75/255 over 500 ms (`update`, end of the slide).
    pub fn hud_pause_fade(&mut self, show: bool) {
        let ids: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.get(i).map_or(false, |n| n.tag == "hud_pause")).collect();
        for i in ids {
            if show { self.node(i).visible = true; self.node(i).alpha = 0.0; self.tween_to(i, &[(Prop::Alpha, 75.0 / 255.0)], 500.0, Ease::Linear, Then::Nothing); }
            else { self.kill_tweens(i); self.node(i).alpha = 0.0; }
        }
    }

    pub fn ufx_reset(&mut self) {
        self.ufx.timers.clear();
        self.kill_tag("fxp");
        self.ufx.parts.clear(); self.ufx.glows.clear();
        self.ufx.awards_started = false;
    }

    pub fn ufx_step(&mut self, dt_ms: f64) {
        if self.end_open && !self.ufx.awards_started && self.end_clock >= 1000.0 { self.start_awards(); }
        // delayed actions
        let mut due = Vec::new();
        for t in self.ufx.timers.iter_mut() { t.0 -= dt_ms; }
        let mut i = 0;
        while i < self.ufx.timers.len() { if self.ufx.timers[i].0 <= 0.0 { due.push(self.ufx.timers.remove(i).1); } else { i += 1; } }
        for a in due { self.fire(a); }
        // particles
        let mut i = 0;
        while i < self.ufx.parts.len() {
            let (node, done) = {
                let p = &mut self.ufx.parts[i];
                p.age += dt_ms;
                (p.node, p.age >= p.life)
            };
            if done { self.nodes[node] = None; self.ufx.parts.remove(i); continue; }
            let p = self.ufx.parts[i].clone();
            let t = p.age / p.life;
            if let Some(n) = self.nodes.get_mut(node).and_then(|n| n.as_mut()) {
                n.x += p.vx * dt_ms / 1000.0; n.y += p.vy * dt_ms / 1000.0;
                let s = p.s0 + (p.s1 - p.s0) * t;
                n.sx = s; n.sy = s; n.alpha = 1.0 - t;
            }
            i += 1;
        }
        // glows
        let mut i = 0;
        while i < self.ufx.glows.len() {
            self.ufx.glows[i].age += dt_ms;
            let g = self.ufx.glows[i].clone();
            let t = (g.age / 400.0).min(1.0);
            let k = ease(Ease::QuadOut, t);
            if t >= 1.0 { self.nodes[g.node] = None; self.ufx.glows.remove(i); continue; }
            if let Some(n) = self.nodes.get_mut(g.node).and_then(|n| n.as_mut()) { n.alpha = 1.0 - k; n.w = g.r0 + k * g.grow; n.h = n.w; }
            i += 1;
        }
    }
}
