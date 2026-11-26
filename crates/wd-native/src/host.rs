//! The game session: what `main.js` does around the game logic. Menus, levels, pause, the end screen, HUD and input.
use crate::assets::{load_png, Assets};
use crate::audio::Audio;
use crate::store::Store;
use crate::online_host::Net;
use crate::canvas::Canvas;
use crate::game::Game;
use crate::levels::{self, Level};
use crate::ui_draw::UiDraw;
use wd_core as core;

const OPT_DEFAULTS: [(&str, f64); 25] = [("noclip", 0.0), ("showPercentage", 1.0), ("percentDecimals", 0.0), ("showHitboxes", 0.0), ("progressBar", 0.0), ("hitboxTrail", 0.0), ("showFPS", 0.0), ("hidePauseBtn", 0.0), ("hidePracticeUI", 0.0), ("solidWaveTrail", 0.0), ("noclipAccuracy", 0.0), ("hitboxesOnDeath", 0.0), ("createObjectIds", 0.0), ("showObjectIds", 0.0), ("showCPS", 0.0), ("speedHack", 1.0), ("respawnDelay", 1.0), ("macroBot", 0.0), ("practiceMusicSync", 0.0), ("enableLDM", 0.0), ("showGlow", 1.0), ("enablePortalGuide", 1.0), ("enableOrbGuide", 0.0), ("enableMiniIcon", 0.0), ("cullDistance", 3.0)];

fn text(n: usize) -> String {
    let p = core::wd_result_ptr();
    unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FadeCmd { None, QuitBack }
pub struct Fade { from: f64, to: f64, t0: f64, ms: f64, cmd: FadeCmd }

pub struct Host {
    pub game: Game,
    pub(crate) ui: UiDraw,
    pub(crate) levels: Vec<Level>,
    pub(crate) visible: Vec<usize>,
    pub menu_mode: bool,
    pub(crate) paused: bool,
    pub(crate) sliding: bool,
    pub(crate) won: bool,
    pub(crate) win_clock: f64,
    pub(crate) win_stage: u8,
    pub(crate) attempts: u32,
    pub(crate) jumps: u32,
    pub(crate) play_time: f64,
    pub(crate) practice: bool,
    pub(crate) was_dead: bool,
    pub(crate) was_held: bool,
    pub(crate) level_id: String,
    pub(crate) delta_hist: [f64; 10],
    pub(crate) delta_idx: usize,
    pub quit: bool,
    pub(crate) cam: (f64, f64),
    pub(crate) audio: Audio,
    pub(crate) store: Store,
    pub(crate) anims_sent: bool,
    pub(crate) net: Net,
    pub(crate) editor_on: bool,
    pub(crate) ed_id: String,
    pub(crate) ed_keys: u32,
    pub(crate) ed_jump: bool,
    pub(crate) ed_pt_event: f64,
    pub(crate) ed_frames_ver: u32,
    pub(crate) created_play_id: Option<String>,
    pub(crate) t_now: f64,
    pub(crate) fade_a: f64,
    pub(crate) fade: Option<Fade>,
    pub(crate) pending_created: Option<(String, String)>,
    pub(crate) ed_custom_song: Option<std::sync::Arc<Vec<u8>>>,
    pub(crate) ed_song_key: String,
    pub(crate) current_song: Option<std::sync::Arc<Vec<u8>>>,
    pub(crate) online_offset: f64,
    pub(crate) last_save: f64,
}

impl Host {
    pub fn new(assets: Assets) -> Result<Host, String> {
        let root = assets.root.clone();
        core::wd_ui_new(crate::canvas::W as f64);
        let ui = UiDraw::new(&root)?;
        let levels = levels::load(&root);
        let hidden = ["level_22", "level_67"];
        let visible: Vec<usize> = (0..levels.len()).filter(|&i| !hidden.contains(&levels[i].id.as_str())).collect();
        let game = Game::new(assets);
        let mut host = Host { game, ui, levels, visible, menu_mode: true, paused: false, sliding: false, won: false, win_clock: 0.0, win_stage: 0, attempts: 1, jumps: 0, play_time: 0.0, practice: false, was_dead: false, was_held: false, level_id: "1".into(), delta_hist: [1000.0 / 60.0; 10], delta_idx: 0, quit: false, cam: (0.0, 0.0), audio: Audio::new(root.clone()), store: Store::open(), anims_sent: false, net: Net::new(), editor_on: false, ed_id: String::new(), ed_keys: 0, ed_jump: false, ed_pt_event: 0.0, ed_frames_ver: 0, created_play_id: None, t_now: 0.0, fade_a: 0.0, fade: None, pending_created: None, ed_custom_song: None, ed_song_key: String::new(), current_song: None, online_offset: 0.0, last_save: 0.0 };
        host.pv_cfg();
        host.load_custom_cube();
        host.audio.user_music = host.store.get_f("userMusicVol", 1.0);
        host.audio.user_sfx = host.store.get_f("userSfxVol", 1.0);
        host.register_standalone_sizes();
        host.send_cp_textures();
        host.push_levels(0);
        host.load_level("1")?;
        host.push_progress();
        core::wd_menu_mode(1);
        core::wd_ui_main_menu();
        if host.menu_music_enabled() { host.audio.start_menu_music(); }
        Ok(host)
    }

    fn pv_main(&self) -> u32 { i64::from_str_radix(self.store.get("iconMainColor").unwrap_or("04FF00"), 16).unwrap_or(0x04ff00) as u32 }
    fn pv_second(&self) -> u32 { i64::from_str_radix(self.store.get("iconSecondaryColor").unwrap_or("00FBFF"), 16).unwrap_or(0x00fbff) as u32 }
    /// The size of every loose image the UI may use (`wd_ui_size` in the web host's loader), under its file name and without the `_` before the last two digits.
    fn register_standalone_sizes(&mut self) {
        let dir = self.game.assets.root.join("sprites");
        let Ok(rd) = std::fs::read_dir(&dir) else { return };
        for e in rd.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("png") { continue; }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()) else { continue };
            let Ok(file) = std::fs::File::open(&path) else { continue };
            let Ok(reader) = png::Decoder::new(std::io::BufReader::new(file)).read_info() else { continue };
            let (w, h) = (reader.info().width as f64, reader.info().height as f64);
            unsafe { core::wd_ui_size(stem.as_ptr(), stem.len(), w, h); }
            let b = stem.as_bytes();
            if b.len() > 3 && b[b.len() - 3] == b'_' && b[b.len() - 2].is_ascii_digit() && b[b.len() - 1].is_ascii_digit() {
                let alias = format!("{}{}", &stem[..stem.len() - 3], &stem[stem.len() - 2..]);
                unsafe { core::wd_ui_size(alias.as_ptr(), alias.len(), w, h); }
            }
        }
    }

    fn menu_music_enabled(&self) -> bool { self.store.get("menuMusicEnabled").map_or(true, |v| v == "true") }
    fn song_file(&self) -> Option<String> {
        let l = self.levels.iter().find(|l| l.id == format!("level_{}", self.level_id))?;
        Some(l.song_file.clone().unwrap_or_else(|| l.name.replace(' ', "")))
    }
    fn music_now(&self) -> f64 { self.game.music_offset + core::wd_game_music_sync_offset() }
    fn start_music(&mut self) {
        if self.menu_mode { return; }
        let o = self.music_now() + if self.net.active { self.online_offset } else { 0.0 };
        if self.net.active { if let Some(b) = self.current_song.clone() { self.audio.start_song_bytes(b, o); } }
        else if let Some(f) = self.song_file() { self.audio.start_song(&f, o); }
    }
    fn best_key(&self) -> String { format!("{}level_{}", if self.practice { "practiceBestPercent_" } else { "bestPercent_" }, self.level_id) }
    fn save_best(&mut self) {
        let pct = core::wd_game_percent(0).floor();
        let key = self.best_key();
        if pct > self.store.get_f(&key, 0.0) { self.store.set(&key, &pct.to_string()); }
    }
    fn flush_coins(&mut self) {
        let json = text(core::wd_coins_outbox());
        if let Ok(j) = core::jsonread::parse(&json) { for e in j.arr().unwrap_or(&[]) { if let Some(p) = e.arr() { if let (Some(k), Some(v)) = (p.first().and_then(|x| x.str()), p.get(1).and_then(|x| x.str())) { self.store.set(k, v); } } } }
    }
    fn coin_ids(&self, kind: &str, id: &str) -> String {
        let a = self.store.get(&format!("gd_{kind}Coins_{id}")).unwrap_or("[]");
        core::jsonread::parse(a).ok().map(|j| j.arr().unwrap_or(&[]).iter().filter_map(|v| v.num().map(|n| n.to_string()).or_else(|| v.str().map(|s| s.to_string()))).collect::<Vec<_>>().join(",")).unwrap_or_default()
    }

    pub(crate) fn json_str(s: &str) -> String { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) }

    fn push_levels(&mut self, page: usize) {
        let items: Vec<String> = self.visible.iter().map(|&i| { let l = &self.levels[i]; format!("[{},{},{},{}]", Self::json_str(&l.internal), Self::json_str(&l.name), Self::json_str(&l.id), l.song_file.as_ref().map_or("null".to_string(), |s| Self::json_str(s))) }).collect();
        let json = format!("[{}]", items.join(","));
        unsafe { core::wd_ui_levels(json.as_ptr(), json.len(), page); }
    }

    fn push_progress(&mut self) {
        let mut total = 0.0;
        for &i in &self.visible {
            let id = self.levels[i].id.clone();
            let slots: Vec<i64> = self.store.get(&format!("gd_secretCoins_{id}_slots")).and_then(|s| core::jsonread::parse(s).ok()).map(|j| j.arr().unwrap_or(&[]).iter().filter_map(|v| v.num().map(|n| n as i64)).collect()).unwrap_or_default();
            let mask = slots.iter().fold(0u32, |m, &s| if (0..3).contains(&s) { m | (1 << s) } else { m });
            total += slots.len() as f64;
            let (n, p) = (self.store.get_f(&format!("bestPercent_{id}"), 0.0), self.store.get_f(&format!("practiceBestPercent_{id}"), 0.0));
            unsafe { core::wd_ui_progress(id.as_ptr(), id.len(), n, p, mask); }
        }
        let ref_h = load_png(&self.game.assets.root.join("game-ground/groundSquare_01_001.png")).map_or(270.0, |g| g.h as f64);
        core::wd_ui_ground(self.game.ground_w, ref_h, self.game.has_ground2 as u32, total);
    }

    fn sync_ground_art(&mut self) { let (a, b) = self.game.ground_images(); self.ui.set_standalone("groundSquare_main", a); self.ui.set_standalone("groundSquare_second", b); }

    /// A level given as text (an online level or a created one), played at once.
    pub(crate) fn play_text(&mut self, id: &str, text: &str) -> Result<(), String> {
        self.net.active = self.net.active;
        self.set_coins(id);
        self.game.ldm = self.store.get_f("opt_enableLDM", 0.0) != 0.0;
        self.game.load_text(id, text, false)?;
        self.level_id = id.to_string();
        self.sync_ground_art();
        self.start_playing();
        Ok(())
    }

    fn set_coins(&mut self, id: &str) {
        let key = if id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 5 { id.to_string() } else { format!("level_{id}") };
        self.game.coins = (self.coin_ids("secret", &key), self.coin_ids("user", &key), self.store.get_f("gd_totalsecretcoins", 0.0), self.store.get_f("gd_totalusercoins", 0.0));
    }

    fn load_level(&mut self, id: &str) -> Result<(), String> {
        self.set_coins(id);
        self.net.active = false;
        self.game.ldm = self.store.get_f("opt_enableLDM", 0.0) != 0.0;
        self.game.load_level(id)?;
        self.level_id = id.to_string();
        self.sync_ground_art();
        Ok(())
    }

    /// Starts a shipped level straight away (the `?play=1` mode of the web host).
    pub fn play(&mut self, id: &str) -> Result<(), String> { self.load_level(id)?; self.start_playing(); Ok(()) }

    fn apply_options(&mut self) {
        for (k, d) in OPT_DEFAULTS { let v = self.store.get_f(&format!("opt_{k}"), d); unsafe { core::wd_ui_opt_set(k.as_ptr(), k.len(), v); } }
        let o = |s: &Self, k: &str, d: f64| s.store.get_f(&format!("opt_{k}"), d);
        core::wd_game_set_options(o(self, "noclip", 0.0) as u32, o(self, "noclipAccuracy", 0.0) as u32, o(self, "speedHack", 1.0), o(self, "respawnDelay", 1.0), 0);
        core::wd_game_set_hitbox_opts(o(self, "showHitboxes", 0.0) as u32, o(self, "hitboxTrail", 0.0) as u32, o(self, "hitboxesOnDeath", 0.0) as u32);
        core::wd_set_ldm(o(self, "enableLDM", 0.0) as u32);
        core::wd_ui_apply_hud();
    }

    fn start_playing(&mut self) {
        self.menu_mode = false; self.paused = false; self.won = false; self.win_clock = 0.0; self.win_stage = 0;
        self.attempts = 1; self.jumps = 0; self.play_time = 0.0; self.practice = false;
        self.store.bump(&format!("levelAttempts_level_{}", self.level_id)); self.store.bump("gd_totalAttempts");
        core::wd_ui_close_end();
        core::wd_ui_start_game();
        self.apply_options();
        core::wd_menu_mode(0);
        core::wd_game_begin_slide();
        self.sliding = true;
        self.game.restart_flag = true;
        core::wd_ui_hud();
        core::wd_ui_hud_pause(0);
        core::wd_ui_attempt(1, core::wd_game_scene(0));
    }

    fn do_pause(&mut self) {
        if self.paused || self.menu_mode || self.sliding || self.won { return; }
        self.paused = true;
        core::wd_ui_set_options(self.audio.user_music, self.audio.user_sfx, self.menu_music_enabled() as u32);
        self.audio.pause();
        let id = self.level_id.clone();
        let (bn, bp) = (self.store.get_f(&format!("bestPercent_level_{id}"), 0.0), self.store.get_f(&format!("practiceBestPercent_level_{id}"), 0.0));
        let name = if self.net.active { self.net.name.clone() } else { String::new() };
        let name = if self.net.active { name } else { self.levels.iter().find(|l| l.id == format!("level_{}", self.level_id)).map_or("Level".to_string(), |l| l.name.clone()) };
        unsafe { core::wd_ui_pause(1, name.as_ptr(), name.len(), bn, bp); }
    }
    fn do_resume(&mut self) { if !self.paused { return; } self.paused = false; self.audio.resume(); unsafe { core::wd_ui_pause(0, "".as_ptr(), 0, 0.0, 0.0); } }

    pub(crate) fn fade_to(&mut self, to: f64, ms: f64, cmd: FadeCmd) {
        if ms <= 0.0 { self.fade_a = to; self.fade = None; return; }
        self.fade = Some(Fade { from: self.fade_a, to, t0: self.t_now, ms, cmd });
    }

    fn step_fade(&mut self) {
        let Some(f) = &self.fade else { return };
        let k = ((self.t_now - f.t0) / f.ms).clamp(0.0, 1.0);
        self.fade_a = f.from + (f.to - f.from) * k;
        if k >= 1.0 { let cmd = f.cmd; self.fade = None; if cmd == FadeCmd::QuitBack { self.quit_back(); self.fade_to(0.0, 400.0, FadeCmd::None); } }
    }

    /// The pause menu quits at once; the level-complete menu fades out for 400 ms and back in.
    fn quit_to_menu(&mut self) {
        self.audio.effect("quitSound_01", 1.0);
        if self.won { self.audio.stop_music(); self.fade_to(1.0, 400.0, FadeCmd::QuitBack); } else { self.quit_back(); }
    }

    fn quit_back(&mut self) {
        self.save_best();
        self.audio.stop_music();
        if self.menu_music_enabled() { self.audio.start_menu_music(); }
        self.sliding = false; self.do_resume(); self.won = false; core::wd_ui_close_end();
        self.menu_mode = true; core::wd_menu_mode(1); core::wd_ui_main_menu_again();
        if self.net.active { self.net_reopen(); return; }
        let idx = self.visible.iter().position(|&i| self.levels[i].id == format!("level_{}", self.level_id));
        if let Some(i) = idx { self.push_levels(i); self.push_progress(); core::wd_ui_open_level_select(); }
    }

    fn restarted(&mut self) {
        self.save_best();
        self.game.restart_flag = true;
        self.attempts += 1;
        core::wd_ui_attempt(self.attempts, core::wd_game_scene(0));
        if !(self.practice && self.store.get_f("opt_practiceMusicSync", 0.0) == 0.0) { self.start_music(); }
        self.store.bump(&format!("levelAttempts_level_{}", self.level_id)); self.store.bump("gd_totalAttempts");
    }

    fn handle_ui(&mut self) {
        loop {
            let n = core::wd_ui_event();
            if n == 0 { break; }
            let ev = text(n);
            match ev.as_str() {
                "OpenLevelSelect" => { self.push_progress(); core::wd_ui_open_level_select(); }
                "CloseLevelSelect" => core::wd_ui_close_level_select(),
                "PlayLevel" => {
                    let page = core::wd_ui_page();
                    if let Some(&i) = self.visible.get(page) {
                        let id = self.levels[i].id.replace("level_", "");
                        self.audio.effect("playSound_01", 1.0);
                        self.audio.stop_music();
                        core::wd_ui_close_level_select();
                        if let Err(e) = self.load_level(&id) { eprintln!("{e}"); continue; }
                        self.start_playing();
                    }
                }
                "Pause" => self.do_pause(),
                "PauseResume" => self.do_resume(),
                "PauseReplay" => { self.do_resume(); core::wd_ui_close_end(); self.won = false; core::wd_game_restart_current(); self.restarted(); }
                "PauseMenu" => self.quit_to_menu(),
                "PausePractice" => { self.practice = !self.practice; core::wd_game_practice(0); core::wd_ui_practice(self.practice as u32); self.do_resume(); self.do_pause(); }
                "Checkpoint" => { if self.practice { core::wd_game_practice(1); } }
                "ClearCheckpoint" => { if self.practice { core::wd_game_practice(2); } }
                "Volume" => { let (m, s) = (core::wd_ui_option(0), core::wd_ui_option(1)); self.audio.set_music_volume(m); self.audio.user_sfx = s; self.store.set("userMusicVol", &m.to_string()); self.store.set("userSfxVol", &s.to_string()); }
                "ToggleMenuMusic" => { let on = core::wd_ui_option(2) != 0.0; self.store.set("menuMusicEnabled", if on { "true" } else { "false" }); if on { if !self.audio.playing_menu() { self.audio.start_menu_music(); } } else { self.audio.stop_music(); } }
                e if e.starts_with("Sfx(") => { let n = &e[5..e.len() - 2]; self.audio.effect(n, 1.0); }
                "Pop(\"GaragePickFile\", 0)" => self.garage_pick(),
                "Garage" | "Creator" => { if ev == "Creator" { self.creator_event(&ev); } else { self.garage_open(); } }
                e if e.starts_with("Pop(\"Em") || e.starts_with("Pop(\"Ed") => { self.creator_event(e); }
                e if e.starts_with("Pop(\"") && self.online_event(e) => {}
                "Pop(\"GarageNeed\", 0)" => self.garage_need(),
                "Pop(\"GarageSave\", 0)" => self.garage_save(),
                "Settings" => { core::wd_ui_set_options(self.audio.user_music, self.audio.user_sfx, self.menu_music_enabled() as u32); core::wd_ui_open_settings(); }
                "Popup(\"Options\")" => { self.apply_options(); core::wd_ui_open_options(); }
                "OptChanged" => self.apply_options_from_ui(),
                _ => {}
            }
        }
    }

    /// `saveOptions`: the options popup's values go to the game.
    fn apply_options_from_ui(&mut self) {
        let json = text(core::wd_ui_opts_json());
        let Ok(j) = core::jsonread::parse(&json) else { return };
        if let Some(e) = j.entries() { for (k, v) in e { let n = match v { core::jsonread::Json::Bool(b) => *b as u8 as f64, v => v.num().unwrap_or(0.0) }; self.store.set(&format!("opt_{k}"), &n.to_string()); } }
        self.apply_options();
    }

    pub fn dump_ui(&mut self) { self.ui.dump = true; }

    pub fn pointer(&mut self, kind: u32, x: f64, y: f64) {
        core::wd_ui_pointer(kind, x, y);
        if kind == 0 && core::wd_ui_menu_active() == 0 { self.jump(true); }
        if kind == 2 { self.jump(false); }
    }
    /// Typed characters (search boxes, level names): `Backspace` = 8, `Enter` = 13, printable characters by code point.
    pub fn text_input(&mut self, ch: u32) { core::wd_ui_key(ch); core::wd_em_key(ch); }
    pub fn wheel(&mut self, x: f64, y: f64, dy: f64) { core::wd_ui_wheel(x, y, dy); }

    pub fn jump(&mut self, down: bool) {
        if down && !self.menu_mode && !self.paused && !self.sliding && !self.game.held { self.jumps += 1; self.store.bump(&format!("levelJumps_level_{}", self.level_id)); self.store.bump("gd_totalJumps"); }
        self.game.held = down;
    }

    /// A key by its DOM `KeyboardEvent.code` name. `jump` is whether it is a jump key.
    pub fn key(&mut self, code: &str, down: bool, repeat: bool, jump: bool, shift: bool) {
        if self.editor_on {
            if jump { self.ed_jump = down; }
            if self.editor_key(code, down, repeat, shift) && !jump { return; }
            if down { let hit = unsafe { core::wd_ui_menu_key(code.as_ptr(), code.len()) } != 0; if hit { return; } }
            return;
        }
        if !down { if jump { self.jump(false); } return; }
        if self.menu_mode || self.paused || self.won || core::wd_ui_menu_active() != 0 {
            let mapped = if jump && !self.won { "Enter" } else { code };
            if jump && repeat { return; }
            let hit = unsafe { core::wd_ui_menu_key(mapped.as_ptr(), mapped.len()) } != 0;
            if hit { return; }
        }
        if code == "Escape" && !self.menu_mode && !self.editor_on {
            if self.paused { self.quit_to_menu(); } else if !self.sliding && !self.won { self.do_pause(); } else if self.won { self.quit_to_menu(); }
            return;
        }
        if self.menu_mode { return; }
        if jump { if !repeat { self.jump(true); } return; }
        match code {
            "KeyR" if !self.paused && !self.sliding && !self.won && !repeat => { core::wd_game_restart_current(); self.restarted(); }
            "KeyP" if !self.paused && !self.sliding && !self.won && !repeat => { self.practice = !self.practice; core::wd_game_practice(0); core::wd_ui_practice(self.practice as u32); }
            "KeyZ" if self.practice && !self.paused && !repeat => { core::wd_game_practice(1); }
            "KeyX" if self.practice && !self.paused && !repeat => { core::wd_game_practice(2); }
            _ => {}
        }
    }

    fn smooth_delta(&mut self, mut t: f64) -> f64 {
        if t > 200.0 { t = self.delta_hist[self.delta_idx].min(200.0); }
        self.delta_hist[self.delta_idx] = t; self.delta_idx = (self.delta_idx + 1) % 10;
        self.delta_hist.iter().sum::<f64>() / 10.0
    }

    /// One frame: logic, then the picture. `raw_dt` is the real time since the last frame in milliseconds.
    pub fn frame(&mut self, raw_dt: f64, now: f64, canvas: &mut Canvas) {
        let dt = self.smooth_delta(raw_dt.max(0.0)).min(100.0);
        self.net_poll();
        self.t_now = now;
        core::wd_ui_step(dt);
        self.game.menu = self.menu_mode;
        if self.editor_on { self.editor_frame(dt); }
        else if !self.paused {
            core::wd_trig_time(now);
            let r = core::wd_game_update(dt, self.game.held as u32);
            if (r >> 30) & 1 != 0 && !self.menu_mode { self.restarted(); }
        }
        if self.sliding && core::wd_game_slide_state() == 2 { self.sliding = false; core::wd_ui_hud_pause(1); self.start_music(); }
        if !self.paused { self.game.fx.update(dt); }
        if !self.won && self.game.fx.active { self.game.fx.reset(); }
        self.game.shake = if self.paused { (0.0, 0.0) } else { self.game.fx.shake() };
        { let (ex, ey, m) = (core::wd_game_scene(5), core::wd_game_scene(4), self.pv_main()); self.game.fx.portal_update(if self.paused { 0.0 } else { dt }, ex, ey, m, !self.menu_mode && self.store.get_f("opt_enableLDM", 0.0) == 0.0); }
        self.game.pv_step(dt / 1000.0, self.paused);
        let info = self.game.draw(canvas);
        self.cam = (info.cam_x, info.cam_y);
        self.ui.cam = self.cam;
        if !self.menu_mode && info.dead && !self.was_dead {
            self.audio.effect("explode_11", 0.65);
            if !(self.practice && self.store.get_f("opt_practiceMusicSync", 0.0) == 0.0) { self.audio.stop_music(); }
        }
        self.was_dead = info.dead;
        if !self.menu_mode { core::wd_ui_hud_update(info.percent.clamp(0.0, 100.0)); }
        if !self.menu_mode && !self.paused {
            if !self.won { self.play_time += dt / 1000.0; }
            if !self.won && core::wd_game_percent(2) != 0.0 { self.won = true; self.win_clock = 0.0; self.win_stage = 0; }
            if self.won {
                self.win_clock += dt;
                if self.win_stage == 0 && self.win_clock >= 1000.0 {
                    self.win_stage = 1;
                    if !self.practice { core::wd_coins_commit(); self.flush_coins(); }
                    self.store.set(&self.best_key(), "100");
                    self.audio.effect("endStart_02", 0.8);
                    let (m, s2) = (self.pv_main(), self.pv_second());
                    let o = crate::endfx::Opts { px: core::wd_game_scene(5) - core::wd_game_scene(0), py: 460.0 - core::wd_game_scene(4) + core::wd_game_scene(1), main: m, second: s2, ldm: self.store.get_f("opt_enableLDM", 0.0) != 0.0 };
                    self.game.fx.begin(o);
                }
            }
        }
        if self.game.fx.text_due { self.game.fx.text_due = false; core::wd_ui_banner(self.practice as u32); }
        if self.game.fx.end_due {
            self.game.fx.end_due = false;
            let main = self.levels.iter().any(|l| l.id == format!("level_{}", self.level_id));
            let cnt = core::wd_game_coin_count(main as u32);
            core::wd_ui_end_coins(main as u32, cnt, 0, 0);
            let msgs = ["Awesome!", "Good\nJob!", "Well\nDone!", "Impressive!", "Amazing!", "Incredible!", "Skillful!", "Brilliant!", "Not\nbad!", "Warp\nSpeed!", "Challenge\nBreaker!", "Reflex\nMaster!", "I am\nspeechless...", "You are...\nThe One!", "How is this\npossible!?", "You beat\nme..."];
            let msg = msgs[(now as usize / 7) % msgs.len()];
            unsafe { core::wd_ui_end_layer(self.attempts, self.jumps, self.play_time, self.practice as u32, msg.as_ptr(), msg.len()); }
        }
        let assets = &self.game.assets;
        self.ui.draw(canvas, assets, &self.game.pv);
        if self.fade_a > 0.0 { canvas.fill_round(0.0, 0.0, crate::canvas::W as f64, crate::canvas::H as f64, 0.0, 0, self.fade_a.min(1.0), crate::canvas::Mat::id()); }
        self.step_fade();
        self.handle_ui();
        if now - self.last_save > 2000.0 { self.last_save = now; self.store.flush(); }
    }
}
