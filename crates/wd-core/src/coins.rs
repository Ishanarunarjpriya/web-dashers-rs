//! Secret and user coins (level.js: `tryCollectSecretCoin`, `tryCollectUserCoin`, `commitSecretCoins`, `commitUserCoins`, the saved
//! progress in localStorage, the coin sprite animations; player.js: the pickup tween). Writes to localStorage go through `outbox`
//! (the host drains it), reads come in through `CoinsInit`.
//! `enabled` is off in the physics harnesses: there `Env::try_collect_*` answer the question as the test dictates.

use crate::game::Game;
use crate::visual::Val;
use std::collections::BTreeMap;

pub const MAX_PER_LEVEL: usize = 3;
pub const SECRET_B: [&str; 4] = ["secretCoin_b_01_001.png", "secretCoin_b_01_002.png", "secretCoin_b_01_003.png", "secretCoin_b_01_004.png"];
pub const USER_B: [&str; 4] = ["secretCoin_2_b_01_001.png", "secretCoin_2_b_01_002.png", "secretCoin_2_b_01_003.png", "secretCoin_2_b_01_004.png"];
pub const SECRET_A: [&str; 4] = ["secretCoin_01_001.png", "secretCoin_01_002.png", "secretCoin_01_003.png", "secretCoin_01_004.png"];
pub const USER_A: [&str; 4] = ["secretCoin_2_01_001.png", "secretCoin_2_01_002.png", "secretCoin_2_01_003.png", "secretCoin_2_01_004.png"];

/// What the host knows about the saved progress when a level is loaded (`window.currentlevel[2]`, localStorage).
#[derive(Clone, Debug, Default)]
pub struct CoinsInit {
    pub official: bool,
    pub stored_user: bool,
    pub editor: bool,
    pub key: String,
    /// `gd_secretCoins_<key>` / `gd_userCoins_<key>` as saved (ids as strings in the original; numbers here)
    pub secret_done: Vec<i64>,
    pub user_done: Vec<i64>,
    pub total_secret: u64,
    pub total_user: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CoinState {
    pub enabled: bool,
    pub init: CoinsInit,
    /// collider uid -> (is user coin, linked object id)
    pub link: BTreeMap<usize, (bool, i64)>,
    pub secret_done: Vec<i64>,
    pub user_done: Vec<i64>,
    pub secret_run: Vec<i64>,
    pub user_run: Vec<i64>,
    pub total_secret: u64,
    pub total_user: u64,
    /// localStorage writes for the host: (key, value)
    pub outbox: Vec<(String, String)>,
    /// uids of coins picked up since the last `take_pickups` (only filled while `enabled`)
    pub pickups: Vec<usize>,
}

fn id_list_json(ids: &[i64]) -> String { format!("[{}]", ids.iter().map(|i| format!("\"{}\"", i)).collect::<Vec<_>>().join(",")) }

impl CoinState {
    pub fn configure(&mut self, init: CoinsInit) {
        // `_loadSecretCoinProgress` / `_loadUserCoinProgress`
        self.secret_done = if init.official { init.secret_done.iter().copied().take(MAX_PER_LEVEL).collect() } else { Vec::new() };
        self.user_done = if init.editor || init.official { Vec::new() } else { init.user_done.iter().copied().take(MAX_PER_LEVEL).collect() };
        self.total_secret = init.total_secret;
        self.total_user = init.total_user;
        self.secret_run.clear();
        self.user_run.clear();
        self.init = init;
    }

    /// `tryCollectSecretCoin` / `tryCollectUserCoin` for the collider with this uid
    pub fn try_collect(&mut self, user: bool, uid: usize) -> bool {
        let Some(&(_, id)) = self.link.get(&uid) else { return false };
        let (done, run) = if user { (&self.user_done, &mut self.user_run) } else { (&self.secret_done, &mut self.secret_run) };
        if done.contains(&id) { return false; }
        if run.len() >= MAX_PER_LEVEL { return false; }
        if !run.contains(&id) { run.push(id); }
        true
    }

    /// The coin checks of the coin branch of `checkCollisions` (practice mode is dealt with by the caller)
    pub fn gate(&mut self, col: &crate::pcol::Col) -> bool {
        let uid = col.uid as usize;
        if col.secret_coin && !col.secret_completed && !self.try_collect(false, uid) { return false; }
        if col.user_coin && !col.user_completed && !self.try_collect(true, uid) { return false; }
        true
    }

    pub fn is_done(&self, user: bool, id: i64) -> bool { if user { self.user_done.contains(&id) } else { self.secret_done.contains(&id) } }
}

#[derive(Clone, Debug)]
pub struct CoinTween { pub sprite: usize, pub elapsed: f64, pub start_y: f64, pub sx: f64, pub sy: f64 }

fn quad_out(t: f64) -> f64 { -t * (t - 2.0) }
fn quad_in(t: f64) -> f64 { t * t }

impl Game {
    /// Player.js coin pickup: the matching coin sprite hops up, then drops and fades.
    pub fn coin_pickups(&mut self) {
        if self.coins.pickups.is_empty() { return; }
        let ups = std::mem::take(&mut self.coins.pickups);
        for uid in ups {
            let (cx, cy) = (self.cols[uid].x, self.cols[uid].y);
            let found = self.fx.lists.coins.iter().copied().find(|&si| {
                let s = &self.fx.sprites[si];
                s.active && (s.num("_coinWorldX").unwrap_or(f64::NAN) - cx).abs() < 2.0 && (s.num("_coinWorldY").unwrap_or(f64::NAN) - cy).abs() < 2.0
            });
            if let Some(si) = found {
                let s = &self.fx.sprites[si];
                let (sx, sy) = (if s.scale_x == 0.0 { 1.0 } else { s.scale_x }, if s.scale_y == 0.0 { 1.0 } else { s.scale_y });
                self.fx.coin_tweens.push(CoinTween { sprite: si, elapsed: 0.0, start_y: s.y, sx, sy });
            }
        }
    }

    pub fn step_coin_tweens(&mut self, dt_ms: f64) {
        let mut i = 0;
        while i < self.fx.coin_tweens.len() {
            let t = &mut self.fx.coin_tweens[i];
            t.elapsed += dt_ms;
            let (si, e, y0, sx, sy) = (t.sprite, t.elapsed, t.start_y, t.sx, t.sy);
            let s = &mut self.fx.sprites[si];
            let mut done = false;
            if e < 180.0 {
                let k = quad_out(e / 180.0);
                s.y = y0 + (-70.0) * k; s.scale_x = sx + (sx * 1.3 - sx) * k; s.scale_y = sy + (sy * 1.3 - sy) * k;
            } else {
                let k = quad_in(((e - 180.0) / 1200.0).min(1.0));
                s.y = (y0 - 70.0) + 670.0 * k;
                s.scale_x = sx * 1.3; s.scale_y = sy * 1.3;
                s.alpha = 1.0 - k;
                if e - 180.0 >= 1200.0 { s.visible = false; done = true; }
            }
            if done { self.fx.coin_tweens.remove(i); } else { i += 1; }
        }
    }

    /// `resetObjects`: coins are available again and their sprites back where they were.
    pub fn reset_coins(&mut self) {
        self.coins.secret_run.clear();
        self.coins.user_run.clear();
        self.fx.coin_tweens.clear();
        for si in self.fx.lists.coins.clone() {
            let s = &mut self.fx.sprites[si];
            s.visible = true; s.alpha = 1.0;
            let base = s.num("_coinBaseScale").unwrap_or(1.0);
            let base = if base == 0.0 || base.is_nan() { 1.0 } else { base };
            s.scale_x = base; s.scale_y = base;
            if let Some(wy) = s.num("_coinWorldY") { s.y = 460.0 - wy; }
        }
    }

    /// `_setSecretCoinAnimation` / `_setUserCoinAnimation`
    fn set_coin_animation(&mut self, si: usize, user: bool, collected: bool) {
        let Some(ids) = self.fx.coin_frames.get(&(si, collected)).cloned() else { return };
        let names: &[&str; 4] = match (user, collected) { (false, false) => &SECRET_A, (false, true) => &SECRET_B, (true, false) => &USER_A, (true, true) => &USER_B };
        self.fx.sprites[si].props.insert("_animFrames", Val::List(names.iter().map(|n| n.to_string()).collect()));
        self.fx.sprites[si].set_num("_animIdx", 0.0);
        self.fx.anim_frames.insert(si, ids.clone());
        if let Some(Some(fid)) = ids.first() {
            let (a, f) = self.fx.frames[*fid as usize].clone();
            let s = &mut self.fx.sprites[si];
            s.atlas = a; s.frame = f;
            self.fx.sprite_frame[si] = *fid;
        }
        self.fx.sprites[si].set_flag(if user { "_userCoinCompleted" } else { "_secretCoinCompleted" }, collected);
    }

    /// `commitSecretCoins` + `commitUserCoins` (a level finished outside practice mode)
    pub fn commit_coins(&mut self) {
        if !self.coins.enabled { return; }
        for user in [false, true] {
            let c = &self.coins;
            if user { if c.init.editor || c.init.official || c.init.stored_user { continue; } } else if !c.init.official { continue; }
            let mut saved = if user { c.user_done.clone() } else { c.secret_done.clone() };
            let run = if user { c.user_run.clone() } else { c.secret_run.clone() };
            let room = MAX_PER_LEVEL.saturating_sub(saved.len());
            let newly: Vec<i64> = run.into_iter().filter(|id| !saved.contains(id)).take(room).collect();
            saved.extend(newly.iter().copied());
            if newly.is_empty() { continue; }
            let key = format!("{}_{}", if user { "gd_userCoins" } else { "gd_secretCoins" }, self.coins.init.key);
            let saved_cut: Vec<i64> = saved.iter().copied().take(MAX_PER_LEVEL).collect();
            self.coins.outbox.push((key.clone(), id_list_json(&saved_cut)));
            let id_prop = if user { "_userCoinId" } else { "_secretCoinId" };
            let slot_prop = if user { "_userCoinSlot" } else { "_secretCoinSlot" };
            let coin_sprites = self.fx.lists.coins.clone();
            let slots: Vec<i64> = coin_sprites.iter().filter_map(|&si| {
                let s = &self.fx.sprites[si];
                let id = s.num(id_prop)? as i64;
                if saved.contains(&id) { s.num(slot_prop).filter(|v| v.fract() == 0.0).map(|v| v as i64) } else { None }
            }).take(MAX_PER_LEVEL).collect();
            self.coins.outbox.push((format!("{}_slots", key), format!("[{}]", slots.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))));
            if user { self.coins.user_done = saved.clone(); self.coins.total_user += newly.len() as u64; } else { self.coins.secret_done = saved.clone(); self.coins.total_secret += newly.len() as u64; }
            let total = if user { self.coins.total_user } else { self.coins.total_secret };
            self.coins.outbox.push((if user { "gd_totalusercoins" } else { "gd_totalsecretcoins" }.to_string(), total.to_string()));
            for si in coin_sprites {
                let s = &self.fx.sprites[si];
                let Some(id) = s.num(id_prop).map(|v| v as i64) else { continue };
                if saved.contains(&id) {
                    self.set_coin_animation(si, user, true);
                    let s = &mut self.fx.sprites[si];
                    s.visible = true; s.alpha = 1.0;
                }
            }
        }
    }
}
