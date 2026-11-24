//! Start positions beyond the object list: smart start pos (`_getSmartStartPosSettings`), the start-position switcher
//! (`changeStartPos`) and the song offset a start position implies (`calculateSongOffsetForX`, `_getStartPosMusicOffset`).
use crate::game::Game;

/// What `_getSmartStartPosSettings` returns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmartState { pub game_mode: i64, pub mini: bool, pub speed: i64, pub dual: bool, pub gravity_flipped: bool, pub mirrored: bool }

#[derive(Default)]
pub struct StartPosState {
    /// `window.smartStartPos`
    pub smart: bool,
    /// `window.levelObjects` reduced to the (x, id) pairs the two scans look at, in level order
    pub objects: Vec<(f64, i64)>,
}

fn game_mode_of(id: i64) -> Option<i64> { match id { 12 => Some(0), 13 => Some(1), 47 => Some(2), 111 => Some(3), 660 => Some(4), 745 => Some(5), 1331 => Some(6), _ => None } }
fn speed_of(id: i64) -> Option<i64> { match id { 200 => Some(1), 201 => Some(0), 202 => Some(2), 203 => Some(3), 1334 => Some(4), _ => None } }

/// `GD_SPEED_PIXELS_PER_SECOND` by speed key (`getSpeedPixelsPerSecondFromKey`)
fn px_per_second_for_key(key: i64) -> f64 {
    const V: [f64; 5] = [623.16, 502.38, 774.84, 936.0, 1152.0];
    if key >= 0 && (key as usize) < V.len() { V[key as usize] } else { 623.16 }
}
fn px_per_second_for_portal(id: i64) -> Option<f64> {
    match id { 200 => Some(502.38), 201 => Some(623.16), 202 => Some(774.84), 203 => Some(936.0), 1334 => Some(1152.0), _ => None }
}

impl Game {
    /// Remember the level objects the smart start pos and song offset scans need.
    pub fn sp_load(&mut self, objects: &[crate::level::LevelObject]) {
        self.sp.objects = objects.iter().map(|o| (o.x, o.id.unwrap_or(i64::MIN))).collect();
    }

    /// `_getSmartStartPosSettings(pos)`: the mode, size, speed, gravity... the level has at `pos_x`, from the header and the portals before it.
    pub fn smart_start_settings(&self, pos_x: f64) -> SmartState {
        let mut st = SmartState { game_mode: self.start.game_mode, mini: self.start.mini, speed: self.start.speed_key, dual: self.start.dual, gravity_flipped: self.start.gravity_flipped, mirrored: self.start.mirrored };
        let mut events: Vec<(f64, i64)> = Vec::new();
        for &(x, id) in &self.sp.objects {
            if !x.is_finite() || x * 2.0 >= pos_x { continue; }
            if game_mode_of(id).is_some() || speed_of(id).is_some() || matches!(id, 99 | 101 | 286 | 287 | 10 | 11 | 2926 | 45 | 46) { events.push((x, id)); }
        }
        events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, id) in events {
            if let Some(m) = game_mode_of(id) { st.game_mode = m; }
            else if let Some(s) = speed_of(id) { st.speed = s; }
            else { match id {
                99 => st.mini = false, 101 => st.mini = true, 286 => st.dual = true, 287 => st.dual = false,
                10 => st.gravity_flipped = false, 11 => st.gravity_flipped = true, 2926 => st.gravity_flipped = !st.gravity_flipped,
                45 => st.mirrored = true, 46 => st.mirrored = false, _ => {}
            } }
        }
        st
    }

    /// `calculateSongOffsetForX(targetX, startSpeedKey, levelObjects)`: seconds of song before world x.
    pub fn song_offset_for_x(&self, target_x: f64) -> f64 {
        let target = { let t = if target_x.is_nan() { 0.0 } else { target_x }; t.max(0.0) };
        let mut events: Vec<(f64, f64)> = Vec::new();
        for &(raw_x, id) in &self.sp.objects {
            let Some(pps) = px_per_second_for_portal(id) else { continue };
            if !raw_x.is_finite() { continue; }
            let wx = raw_x * 2.0;
            if wx < 0.0 || wx > target { continue; }
            events.push((wx, pps));
        }
        events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let (mut cur_x, mut cur_pps, mut secs) = (0.0f64, px_per_second_for_key(self.start.speed_key), 0.0f64);
        for (x, pps) in events {
            let next = 0f64.max(target.min(x));
            secs += 0f64.max(next - cur_x) / cur_pps.max(0.000001);
            cur_x = next;
            cur_pps = pps;
        }
        secs += 0f64.max(target - cur_x) / cur_pps.max(0.000001);
        secs.max(0.0)
    }

    /// `_getSongOffsetForWorldX`
    pub fn song_offset_for_world_x(&self, x: f64) -> f64 { self.song_offset_for_x(if x.is_finite() { x } else { 0.0 }) }

    /// `_getStartPosMusicOffset`
    pub fn start_pos_music_offset(&self) -> f64 {
        match self.start_pos_index.and_then(|i| self.start_positions.get(i)) { Some(p) => self.song_offset_for_world_x(p.x), None => 0.0 }
    }

    /// `_getCurrentMusicSyncOffset`
    pub fn current_music_sync_offset(&self) -> f64 {
        if self.start_pos_index.is_some() && self.e.player_world_x <= 0.0 { self.start_pos_music_offset() } else { self.song_offset_for_world_x(self.e.player_world_x) }
    }

    /// `changeStartPos(direction)`: step through the start positions (-1 = the level start) and restart. The scene's own
    /// paused / menu / slide-in guards are the caller's.
    pub fn change_start_pos(&mut self, direction: i64) {
        if self.level_won { return; }
        let total = self.start_positions.len() as i64;
        if total == 0 { return; }
        let mut idx = self.start_pos_index.map_or(-1, |i| i as i64) + direction;
        if idx < -1 { idx = total - 1; } else if idx >= total { idx = -1; }
        self.start_pos_index = if idx < 0 { None } else { Some(idx as usize) };
        self.checkpoints.clear();
        let sp = self.start_pos_index;
        self.restart_level(sp);
    }
}
