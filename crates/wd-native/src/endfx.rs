//! The effects of a completed level and the end portal (port of `web/endfx.js`).
use crate::assets::Assets;
use crate::canvas::{Canvas, Sprite, W};

const T: f64 = 460.0;

struct Circle { x: f64, y: f64, r0: f64, r1: f64, dur: f64, filled: bool, mode: bool, color: u32, t: f64 }
struct Ray { x: f64, y: f64, color: u32, delay: f64, len: f64, w1: f64, dur: f64, alpha: f64, angle: f64, fade: f64, fade_delay: f64, fade_dur: f64, age: f64 }
struct Emitter { x: f64, y: f64, ex: (f64, f64), ey: (f64, f64), qty: usize, left: usize, speed: (f64, f64), life: (f64, f64), s0: f64, s1: f64, a0: f64, a1: f64, color: u32 }
struct Part { x: f64, y: f64, vx: f64, vy: f64, age: f64, life: f64, s0: f64, s1: f64, a0: f64, a1: f64, color: u32 }
#[derive(Clone, Copy)]
enum Ev { Circle(f64, f64, f64, f64, f64, bool, bool, u32), Rays, Text, Firework, End }
struct Queued { abs: f64, ev: Ev }
struct PortalPart { x: f64, y: f64, vx: f64, vy: f64, age: f64, life: f64 }

#[derive(Clone, Copy)]
pub struct Opts { pub px: f64, pub py: f64, pub main: u32, pub second: u32, pub ldm: bool }

pub struct EndFx {
    t: f64, pub active: bool, o: Option<Opts>, queue: Vec<Queued>, circles: Vec<Circle>, parts: Vec<Part>, rays: Vec<Ray>, emitters: Vec<Emitter>,
    shake_until: f64, rng: u64, portal: Vec<PortalPart>, portal_acc: f64, portal_main: u32,
    /// set when the banner should appear / the end layer should open
    pub text_due: bool, pub end_due: bool,
}

fn quad_out(v: f64) -> f64 { v * (2.0 - v) }

impl EndFx {
    pub fn new() -> EndFx { EndFx { t: 0.0, active: false, o: None, queue: Vec::new(), circles: Vec::new(), parts: Vec::new(), rays: Vec::new(), emitters: Vec::new(), shake_until: 0.0, rng: 0x1234_5678_9abc_def1, portal: Vec::new(), portal_acc: 0.0, portal_main: 0x04ff00, text_due: false, end_due: false } }
    fn rnd(&mut self, a: f64, b: f64) -> f64 { self.rng ^= self.rng << 13; self.rng ^= self.rng >> 7; self.rng ^= self.rng << 17; a + (self.rng >> 11) as f64 / (1u64 << 53) as f64 * (b - a) }
    fn at(&mut self, ms: f64, ev: Ev) { self.queue.push(Queued { abs: self.t + ms, ev }); }
    fn circle(&mut self, x: f64, y: f64, r0: f64, r1: f64, dur: f64, filled: bool, mode: bool, color: u32) { self.circles.push(Circle { x, y, r0, r1, dur, filled, mode, color, t: 0.0 }); }

    pub fn reset(&mut self) { self.t = 0.0; self.active = false; self.o = None; self.queue.clear(); self.circles.clear(); self.parts.clear(); self.rays.clear(); self.emitters.clear(); self.shake_until = 0.0; self.text_due = false; self.end_due = false; }

    pub fn begin(&mut self, o: Opts) {
        self.reset(); self.active = true; self.o = Some(o);
        for i in 0..5 { self.at(i as f64 * 50.0, Ev::Circle(o.px, o.py, 10.0, W as f64, 500.0, false, true, o.main)); }
        self.circle(o.px, o.py, 10.0, 1000.0, 500.0, true, false, o.main);
        self.make_rays(o.px + 60.0, o.py, o.main);
        self.shake_until = 1950.0;
        self.at(1950.0, Ev::Text);
    }

    fn make_rays(&mut self, x: f64, y: f64, color: u32) {
        let n = 8; let step = 90.0 / n as f64;
        let mut ang: Vec<f64> = (0..n).map(|i| -135.0 + i as f64 * step).collect();
        for i in (1..n).rev() { let j = self.rnd(0.0, (i + 1) as f64) as usize; ang.swap(i, j.min(i)); }
        let len = ((W * W) as f64 + 102400.0).sqrt().round() + 65.0;
        let mut last: f64 = 0.0;
        for i in 0..n {
            let delay = (i as f64 * 195.0 + 40.0 + 40.0 * self.rnd(-1.0, 1.0)).max(0.0);
            let (w1, dur, alpha, extra) = (60.0 + 40.0 * self.rnd(-1.0, 1.0), 180.0 + 40.0 * self.rnd(-1.0, 1.0), (155.0 / 255.0 + 100.0 / 255.0 * self.rnd(-1.0, 1.0)).clamp(0.0, 1.0), self.rnd(0.0, 1.0));
            self.rays.push(Ray { x, y, color, delay, len, w1, dur, alpha, angle: ang[i] + step * extra + 180.0, fade: -1.0, fade_delay: 0.0, fade_dur: 0.0, age: 0.0 });
            last = last.max(delay);
        }
        self.at(last + 400.0, Ev::Rays);
    }

    fn firework(&mut self, c1: u32, c2: u32) {
        let x = 200.0 + (W as f64 - 400.0) * self.rnd(0.0, 1.0); let y = 200.0 + self.rnd(0.0, 240.0);
        let r1 = 140.0 + self.rnd(0.0, 60.0);
        self.circle(x, y, 40.0, r1, 500.0, true, true, c2);
        self.emitters.push(Emitter { x, y, ex: (-20.0, 20.0), ey: (-20.0, 20.0), qty: 1, left: 25, speed: (520.0, 920.0), life: (0.0, 500.0), s0: 0.4, s1: 0.13, a0: 1.0, a1: 0.0, color: c1 });
    }

    fn fire(&mut self, ev: Ev) {
        let Some(o) = self.o else { return };
        match ev {
            Ev::Circle(x, y, r0, r1, d, f, m, c) => self.circle(x, y, r0, r1, d, f, m, c),
            Ev::Rays => for r in self.rays.iter_mut() { r.fade = 0.0; r.fade_delay = 0.0; }
            Ev::Firework => self.firework(o.main, o.second),
            Ev::Text => {
                self.text_due = true;
                let cx = W as f64 / 2.0;
                for col in [o.main, 0xffffff] { self.emitters.push(Emitter { x: cx, y: 250.0, ex: (-800.0, 800.0), ey: (-80.0, 80.0), qty: 50, left: 200, speed: (300.0, 700.0), life: (0.0, 1000.0), s0: 0.4, s1: 0.13, a0: 1.0, a1: 1.0, color: col }); }
                self.circle(o.px, o.py, 10.0, W as f64, 800.0, true, false, o.main);
                self.circle(cx, 250.0, 10.0, 1000.0, 800.0, true, false, o.main);
                for i in 0..5 { self.at(i as f64 * 50.0, Ev::Circle(o.px, o.py, 10.0, W as f64, 500.0, false, true, o.main)); }
                if !o.ldm { for i in 0..10 { let d = (i as f64 * 150.0 + self.rnd(-80.0, 80.0)).max(0.0); self.at(d, Ev::Firework); } }
                self.at(1500.0, Ev::End);
            }
            Ev::End => self.end_due = true,
        }
    }

    pub fn shake(&mut self) -> (f64, f64) {
        if self.active && self.t < self.shake_until { (self.rnd(-1.0, 1.0) * 0.004 * W as f64, self.rnd(-1.0, 1.0) * 0.004 * 640.0) } else { (0.0, 0.0) }
    }

    pub fn update(&mut self, dt: f64) {
        if !self.active { return; }
        self.t += dt;
        for _ in 0..8 {
            let t = self.t;
            let (ready, rest): (Vec<Queued>, Vec<Queued>) = std::mem::take(&mut self.queue).into_iter().partition(|q| q.abs <= t);
            self.queue = rest;
            if ready.is_empty() { break; }
            for q in ready { self.fire(q.ev); }
        }
        for c in self.circles.iter_mut() { c.t += dt; }
        self.circles.retain(|c| c.t < c.dur);
        for i in 0..self.rays.len() {
            let fade_rand = (self.rnd(0.0, 200.0), self.rnd(300.0, 500.0));
            let r = &mut self.rays[i];
            if r.fade >= 0.0 && r.fade_dur == 0.0 { r.fade_delay = fade_rand.0; r.fade_dur = fade_rand.1; }
            r.delay -= dt;
            if r.fade >= 0.0 { r.fade_delay -= dt; }
            if r.fade >= 0.0 && r.fade_delay <= 0.0 { r.fade += dt; }
            if r.delay < 0.0 { r.age += dt; }
        }
        self.rays.retain(|r| !(r.fade >= 0.0 && r.fade >= r.fade_dur));
        let mut new_parts = Vec::new();
        for i in 0..self.emitters.len() {
            let (qty, left) = (self.emitters[i].qty, self.emitters[i].left);
            let n = qty.min(left);
            self.emitters[i].left -= n;
            for _ in 0..n {
                let e = &self.emitters[i];
                let (esp, elife, eex, eey, ex, ey, s0, s1, a0, a1, col) = (e.speed, e.life, e.ex, e.ey, e.x, e.y, e.s0, e.s1, e.a0, e.a1, e.color);
                let a = self.rnd(0.0, 360.0).to_radians(); let sp = self.rnd(esp.0, esp.1);
                let (jx, jy, life) = (self.rnd(eex.0, eex.1), self.rnd(eey.0, eey.1), self.rnd(elife.0, elife.1));
                new_parts.push(Part { x: ex + jx, y: ey + jy, vx: a.cos() * sp, vy: a.sin() * sp, age: 0.0, life, s0, s1, a0, a1, color: col });
            }
        }
        self.parts.extend(new_parts);
        self.emitters.retain(|e| e.left > 0);
        let s = dt / 1000.0;
        for p in self.parts.iter_mut() { p.age += dt; p.x += p.vx * s; p.y += p.vy * s; }
        self.parts.retain(|p| p.age < p.life);
        if self.queue.is_empty() && self.circles.is_empty() && self.parts.is_empty() && self.rays.is_empty() && self.emitters.is_empty() && self.t > 3600.0 { self.active = false; }
    }

    /// The end wall and its shine in the level layer.
    pub fn draw_wall(&self, c: &mut Canvas, assets: &Assets, ox: f64, oy: f64, end_x: f64, end_y: f64, main: u32, on: bool) {
        if !on || !(end_x > 0.0) { return; }
        let x = end_x + ox;
        if x < -200.0 || x > W as f64 + 400.0 { return; }
        let y = T - end_y + oy;
        let Some(web) = assets.sheets.get("GJ_WebSheet") else { return };
        if let Some(f) = web.frames.get("square_02_001.png") {
            for i in 0..16 { c.frame(&web.img, f, Sprite { x, y: y + (i as f64 - 8.0) * 60.0, rot: -std::f64::consts::FRAC_PI_2, sx: 1.0, sy: 1.0, alpha: 1.0, tint: -1, additive: false, flip_x: false, flip_y: false }); }
        }
        if let Some(f) = web.frames.get("gradientBar.png") {
            c.frame(&web.img, f, Sprite { x: x - 58.0, y, rot: 0.0, sx: 1.0, sy: 960.0 / f.src_h.max(1.0), alpha: 1.0, tint: main as i64, additive: true, flip_x: false, flip_y: false });
        }
    }

    pub fn portal_update(&mut self, dt: f64, end_x: f64, end_y: f64, main: u32, on: bool) {
        if !on || !(end_x > 0.0) { self.portal.clear(); self.portal_acc = 0.0; return; }
        self.portal_acc += dt;
        while self.portal_acc >= 10.0 {
            self.portal_acc -= 10.0;
            if self.portal.len() >= 100 { continue; }
            let ang = (85.0 + self.rnd(0.0, 190.0)).to_radians(); let rad = 320.0 + self.rnd(-1.0, 1.0) * 80.0;
            let (dx, dy) = (ang.cos() * rad, ang.sin() * rad);
            let life = self.rnd(200.0, 1000.0);
            let d = (dx * dx + dy * dy).sqrt().max(1e-9);
            let sp = (d - 20.0) / (life / 1000.0);
            self.portal.push(PortalPart { x: end_x - 30.0 + dx, y: T - end_y + dy, vx: -dx / d * sp, vy: -dy / d * sp, age: 0.0, life });
        }
        let s = dt / 1000.0;
        for p in self.portal.iter_mut() { p.age += dt; p.x += p.vx * s; p.y += p.vy * s; }
        self.portal.retain(|p| p.age < p.life);
        self.portal_main = main;
    }

    pub fn draw_portal_particles(&self, c: &mut Canvas, assets: &Assets, ox: f64, oy: f64) {
        let Some(web) = assets.sheets.get("GJ_WebSheet") else { return };
        let Some(f) = web.frames.get("square.png") else { return };
        for p in &self.portal {
            let k = p.age / p.life; let sc = 0.75 + (0.125 - 0.75) * k;
            c.frame(&web.img, f, Sprite { x: p.x + ox, y: p.y + oy, rot: 0.0, sx: sc, sy: sc, alpha: 1.0 - k, tint: self.portal_main as i64, additive: true, flip_x: false, flip_y: false });
        }
    }

    /// Light rays, behind the level.
    pub fn draw_back(&self, c: &mut Canvas) {
        if !self.active { return; }
        for r in &self.rays {
            if r.delay > 0.0 { continue; }
            let k = (r.age / r.dur).min(1.0); let e = quad_out(k);
            let h = 1.0 + (r.len - 1.0) * e; let w = 2.0 + (r.w1 - 2.0) * e; let w0 = 2.0 + (w - 2.0) / 4.0;
            let al = if r.fade >= 0.0 && r.fade_delay <= 0.0 { r.alpha * (1.0 - (r.fade / r.fade_dur.max(1.0)).min(1.0)) } else { r.alpha };
            let a = r.angle.to_radians(); let (co, si) = (a.cos(), a.sin());
            let pts: Vec<(f64, f64)> = [(-w0 / 2.0, 0.0), (w0 / 2.0, 0.0), (w / 2.0, h), (-w / 2.0, h)].iter().map(|&(lx, ly)| (r.x + lx * co - ly * si, r.y + lx * si + ly * co)).collect();
            c.fill_tris(&[[pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1], [pts[0].0, pts[0].1, pts[2].0, pts[2].1, pts[3].0, pts[3].1]], r.color, al, true);
        }
    }

    /// Circles and particles, above the level.
    pub fn draw_front(&self, c: &mut Canvas, assets: &Assets) {
        if !self.active { return; }
        for ci in &self.circles {
            let tt = ci.t / ci.dur; let te = if ci.filled && !ci.mode { quad_out(tt) } else { tt };
            let r = ci.r0 + (ci.r1 - ci.r0) * te;
            let al = if ci.mode { if te < 0.5 { te * 2.0 } else { (1.0 - te) * 2.0 } } else { 1.0 - te };
            if al <= 0.0 { continue; }
            if ci.filled { c.circle_fill_mode(ci.x, ci.y, r.max(0.0), ci.color, al.min(1.0), true); } else { c.circle_stroke_add(ci.x, ci.y, r.max(0.0), 4.0, ci.color, al.min(1.0)); }
        }
        let Some(web) = assets.sheets.get("GJ_WebSheet") else { return };
        let Some(f) = web.frames.get("square.png") else { return };
        for p in &self.parts {
            let k = if p.life > 0.0 { p.age / p.life } else { 1.0 };
            let sc = p.s0 + (p.s1 - p.s0) * k; let al = p.a0 + (p.a1 - p.a0) * k;
            c.frame(&web.img, f, Sprite { x: p.x, y: p.y, rot: 0.0, sx: sc, sy: sc, alpha: al, tint: p.color as i64, additive: true, flip_x: false, flip_y: false });
        }
    }
}
