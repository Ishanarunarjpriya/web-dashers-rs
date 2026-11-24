//! Level-select card interaction (game-scene.js `_openLevelSelect`): press-and-bounce, drag/swipe, the spring snap-back and the
//! slide-out / slide-in when the level changes. Everything here is ported from `dragState`, `onDragMove`, `onDragUp` and `switchLevel`.
use crate::ui::*;

/// Where the card currently is in a slide.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LsAnim {
    #[default]
    None,
    /// sliding out; the page was already changed, the content is rebuilt once the target is reached
    Out { dir: f64, x: f64, speed: f64, target: f64, in_vel: f64 },
    /// sliding in with a spring
    In { x: f64, vel: f64 },
    /// spring back to the middle after a short drag
    Snap { x: f64, vel: f64 },
}

#[derive(Clone, Debug, Default)]
pub struct LsState {
    /// `cardSlideContainer.x`
    pub x: f64,
    /// `cardBounceContainer` scale
    pub bounce: f64,
    pub bounce_tw: Option<(f64, f64, f64, f64, Ease)>, // from, to, duration, elapsed, ease
    pub pressed: bool,
    pub dragging: bool,
    pub start_x: f64,
    pub last_x: f64,
    pub vel_samples: Vec<f64>,
    pub anim: LsAnim,
}

impl LsState {
    pub fn fresh() -> LsState { LsState { bounce: 1.0, ..Default::default() } }
    fn vel(&self) -> f64 { if self.vel_samples.is_empty() { 0.0 } else { self.vel_samples.iter().sum::<f64>() / self.vel_samples.len() as f64 } }
}

impl Ui {
    fn ls_card_rect(&self) -> (f64, f64, f64, f64) {
        let (sw, sh) = (self.screen_width, self.screen_height);
        let card_w = 700f64.min(sw - 180.0);
        (sw / 2.0, sh / 2.0 - 100.0, card_w, 185.0)
    }

    fn ls_coming_soon(&self) -> bool { self.page >= self.levels.len() }

    fn ls_locked(&self) -> bool {
        let Some(l) = self.levels.get(self.page) else { return false };
        let required = match l.id.as_str() { "level_14" => 10.0, "level_18" => 20.0, "level_20" => 30.0, _ => 0.0 };
        required > self.total_coins
    }

    fn ls_bounce_to(&mut self, to: f64, dur: f64, e: Ease) {
        let from = self.ls.bounce;
        self.ls.bounce_tw = Some((from, to, dur, 0.0, e));
    }

    /// Called from `pointer_down` when the level select is open. `hit` is the node under the pointer.
    pub fn ls_pointer_down(&mut self, x: f64, hit: Option<usize>) {
        let Some(h) = hit else { return };
        let tag = self.get(h).map_or("", |n| n.tag);
        if tag != "ls_block" && tag != "ls_hit" { return; }
        self.ls.pressed = true;
        self.ls.start_x = x; self.ls.last_x = x;
        self.ls.dragging = false;
        self.ls.vel_samples.clear();
        if tag == "ls_hit" && !self.ls_coming_soon() && !self.ls_locked() {
            self.ls_bounce_to(1.26, 300.0, Ease::BounceOut);
        }
    }

    pub fn ls_pointer_move(&mut self, x: f64) {
        if !self.ls.pressed { return; }
        let dx = x - self.ls.start_x;
        let frame_delta = x - self.ls.last_x;
        self.ls.vel_samples.push(frame_delta);
        if self.ls.vel_samples.len() > 5 { self.ls.vel_samples.remove(0); }
        self.ls.last_x = x;
        if !self.ls.dragging && dx.abs() > 12.0 {
            self.ls.dragging = true;
            self.ls_bounce_to(1.0, 200.0, Ease::QuadOut);
        }
        if self.ls.dragging { self.ls.x = dx; }
    }

    pub fn ls_pointer_up(&mut self, x: f64, y: f64) {
        if !self.ls.pressed { return; }
        let was_dragging = self.ls.dragging;
        let total_dx = x - self.ls.start_x;
        let vel = self.ls.vel();
        self.ls.pressed = false; self.ls.dragging = false; self.ls.vel_samples.clear();
        let (card_x, card_y, card_w, card_h) = self.ls_card_rect();
        if was_dragging {
            let threshold = card_w * 0.18;
            if total_dx.abs() > threshold || vel.abs() > 3.0 {
                let dir = if total_dx < 0.0 { 1 } else { -1 };
                let sx = self.ls.x;
                self.ls_switch(dir, Some(sx), vel);
            } else {
                self.ls.anim = LsAnim::Snap { x: self.ls.x, vel: vel * 40.0 };
            }
        } else if x >= card_x - card_w / 2.0 && x <= card_x + card_w / 2.0 && y >= card_y - card_h / 2.0 && y <= card_y + card_h / 2.0 {
            if self.ls_coming_soon() { return; }
            if self.ls_locked() { return; }
            // the original disables input and loads the level here; the host does the loading
            self.ls.bounce_tw = None; self.ls.bounce = 1.0;
            self.events.push(Action::PlayLevel);
        }
    }

    /// `switchLevel(dir, startX, dragVel)`
    pub fn ls_switch(&mut self, dir: i64, start_x: Option<f64>, drag_vel: f64) {
        let page_count = self.levels.len() as i64 + 1;
        if page_count <= 0 { return; }
        self.page = (((self.page as i64 + dir) % page_count + page_count) % page_count) as usize;
        let (_, _, card_w, _) = self.ls_card_rect();
        let slide_dist = card_w - 200.0;
        let dir = dir as f64;
        let boost = drag_vel.abs() * 60.0;
        let x = start_x.unwrap_or(self.ls.x);
        self.ls.anim = LsAnim::Out { dir, x, speed: slide_dist * 14.0 + boost, target: -dir * slide_dist, in_vel: slide_dist * 6.0 + boost };
    }

    /// Per-frame part (`preupdate` handlers and the scale tweens)
    pub fn ls_step(&mut self, dt_ms: f64) {
        if !self.level_select_open { return; }
        if let Some((from, to, dur, el, e)) = self.ls.bounce_tw {
            let el = el + dt_ms;
            let p = if dur > 0.0 { (el / dur).min(1.0) } else { 1.0 };
            self.ls.bounce = from + (to - from) * ease(e, p);
            self.ls.bounce_tw = if p >= 1.0 { None } else { Some((from, to, dur, el, e)) };
        }
        let dt = (dt_ms / 1000.0).min(0.05);
        match self.ls.anim {
            LsAnim::None => {}
            LsAnim::Out { dir, mut x, speed, target, in_vel } => {
                x += -dir * speed * dt;
                if (dir > 0.0 && x <= target) || (dir < 0.0 && x >= target) {
                    let (_, _, card_w, _) = self.ls_card_rect();
                    let slide_dist = card_w - 200.0;
                    if self.ls_coming_soon() { self.ls.bounce_tw = None; self.ls.bounce = 1.0; }
                    self.build_level_card();
                    let start = dir * slide_dist;
                    self.ls.anim = LsAnim::In { x: start, vel: -dir * in_vel };
                    self.ls.x = start;
                } else {
                    self.ls.anim = LsAnim::Out { dir, x, speed, target, in_vel };
                    self.ls.x = x;
                }
            }
            LsAnim::In { mut x, mut vel } => {
                let (tension, friction) = (300.0, 15.0);
                let force = -tension * x - friction * vel;
                vel += force * dt;
                x += vel * dt;
                if x.abs() < 1.0 && vel.abs() < 15.0 { x = 0.0; self.ls.anim = LsAnim::None; } else { self.ls.anim = LsAnim::In { x, vel }; }
                self.ls.x = x;
            }
            LsAnim::Snap { mut x, mut vel } => {
                let (tension, friction) = (400.0, 18.0);
                let force = -tension * x - friction * vel;
                vel += force * dt;
                x += vel * dt;
                if x.abs() < 0.5 && vel.abs() < 5.0 { x = 0.0; self.ls.anim = LsAnim::None; } else { self.ls.anim = LsAnim::Snap { x, vel }; }
                self.ls.x = x;
            }
        }
    }

    /// Position/size of a card node while sliding and bouncing: `ls_card` nodes sit in the bounce container (scaled about the card
    /// centre) inside the slide container; `ls_bar` nodes only slide.
    pub fn ls_xform(&self, n: &Node, ex: &mut f64, ey: &mut f64, esx: &mut f64, esy: &mut f64, ew: &mut f64, eh: &mut f64, er: &mut f64) {
        let shape = matches!(n.kind, Kind::Rect | Kind::RoundRect | Kind::Nine);
        if n.tag == "ls_card" {
            let (ccx, ccy, _, _) = self.ls_card_rect();
            let b = if self.ls.bounce == 0.0 { 1.0 } else { self.ls.bounce };
            *ex = ccx + (*ex - ccx) * b; *ey = ccy + (*ey - ccy) * b;
            if shape { *ew *= b; *eh *= b; *er *= b; } else { *esx *= b; *esy *= b; }
            *ex += self.ls.x;
        } else if n.tag == "ls_bar" {
            *ex += self.ls.x;
        }
    }
}
