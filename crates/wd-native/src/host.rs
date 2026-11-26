//! The game session: what `main.js` does around the game logic. Menus, levels, pause, the end screen, HUD and input.
use crate::assets::{load_png, Assets};
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

pub struct Host {
    pub game: Game,
    ui: UiDraw,
    levels: Vec<Level>,
    visible: Vec<usize>,
    pub menu_mode: bool,
    paused: bool,
    sliding: bool,
    won: bool,
    win_clock: f64,
    win_stage: u8,
    attempts: u32,
    jumps: u32,
    play_time: f64,
    practice: bool,
    was_dead: bool,
    was_held: bool,
    level_id: String,
    delta_hist: [f64; 10],
    delta_idx: usize,
    pub quit: bool,
    cam: (f64, f64),
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
        let mut host = Host { game, ui, levels, visible, menu_mode: true, paused: false, sliding: false, won: false, win_clock: 0.0, win_stage: 0, attempts: 1, jumps: 0, play_time: 0.0, practice: false, was_dead: false, was_held: false, level_id: "1".into(), delta_hist: [1000.0 / 60.0; 10], delta_idx: 0, quit: false, cam: (0.0, 0.0) };
        host.push_levels(0);
        host.load_level("1")?;
        host.push_progress();
        core::wd_menu_mode(1);
        core::wd_ui_main_menu();
        Ok(host)
    }

    pub fn json_str(s: &str) -> String { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) }

    fn push_levels(&mut self, page: usize) {
        let items: Vec<String> = self.visible.iter().map(|&i| { let l = &self.levels[i]; format!("[{},{},{},{}]", Self::json_str(&l.internal), Self::json_str(&l.name), Self::json_str(&l.id), l.song_file.as_ref().map_or("null".to_string(), |s| Self::json_str(s))) }).collect();
        let json = format!("[{}]", items.join(","));
        unsafe { core::wd_ui_levels(json.as_ptr(), json.len(), page); }
    }

    fn push_progress(&mut self) {
        for &i in &self.visible { let id = &self.levels[i].id; unsafe { core::wd_ui_progress(id.as_ptr(), id.len(), 0.0, 0.0, 0); } }
        let ref_h = load_png(&self.game.assets.root.join("game-ground/groundSquare_01_001.png")).map_or(270.0, |g| g.h as f64);
        core::wd_ui_ground(self.game.ground_w, ref_h, self.game.has_ground2 as u32, 0.0);
    }

    fn load_level(&mut self, id: &str) -> Result<(), String> {
        self.game.load_level(id)?;
        self.level_id = id.to_string();
        Ok(())
    }

    /// Starts a shipped level straight away (the `?play=1` mode of the web host).
    pub fn play(&mut self, id: &str) -> Result<(), String> { self.load_level(id)?; self.start_playing(); Ok(()) }

    fn apply_options(&mut self) {
        for (k, v) in OPT_DEFAULTS { unsafe { core::wd_ui_opt_set(k.as_ptr(), k.len(), v); } }
        core::wd_game_set_options(0, 0, 1.0, 1.0, 0);
        core::wd_set_ldm(0);
        core::wd_ui_apply_hud();
    }

    fn start_playing(&mut self) {
        self.menu_mode = false; self.paused = false; self.won = false; self.win_clock = 0.0; self.win_stage = 0;
        self.attempts = 1; self.jumps = 0; self.play_time = 0.0; self.practice = false;
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
        core::wd_ui_set_options(1.0, 1.0, 1);
        let name = self.levels.iter().find(|l| l.id == format!("level_{}", self.level_id)).map_or("Level".to_string(), |l| l.name.clone());
        unsafe { core::wd_ui_pause(1, name.as_ptr(), name.len(), 0.0, 0.0); }
    }
    fn do_resume(&mut self) { if !self.paused { return; } self.paused = false; unsafe { core::wd_ui_pause(0, "".as_ptr(), 0, 0.0, 0.0); } }

    fn quit_to_menu(&mut self) {
        self.sliding = false; self.do_resume(); self.won = false; core::wd_ui_close_end();
        self.menu_mode = true; core::wd_menu_mode(1); core::wd_ui_main_menu_again();
        let idx = self.visible.iter().position(|&i| self.levels[i].id == format!("level_{}", self.level_id));
        if let Some(i) = idx { self.push_levels(i); self.push_progress(); core::wd_ui_open_level_select(); }
    }

    fn restarted(&mut self) {
        self.game.restart_flag = true;
        self.attempts += 1;
        core::wd_ui_attempt(self.attempts, core::wd_game_scene(0));
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
                "Settings" => { core::wd_ui_set_options(1.0, 1.0, 1); core::wd_ui_open_settings(); }
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
        let g = |k: &str, d: f64| match j.get(k) { Some(core::jsonread::Json::Bool(b)) => *b as u8 as f64, Some(v) => v.num().unwrap_or(d), None => d };
        core::wd_game_set_options(g("noclip", 0.0) as u32, g("noclipAccuracy", 0.0) as u32, g("speedHack", 1.0), g("respawnDelay", 1.0), 0);
        core::wd_game_set_hitbox_opts(g("showHitboxes", 0.0) as u32, g("hitboxTrail", 0.0) as u32, g("hitboxesOnDeath", 0.0) as u32);
        core::wd_ui_apply_hud();
    }

    pub fn pointer(&mut self, kind: u32, x: f64, y: f64) {
        core::wd_ui_pointer(kind, x, y);
        if kind == 0 && core::wd_ui_menu_active() == 0 { self.jump(true); }
        if kind == 2 { self.jump(false); }
    }
    pub fn wheel(&mut self, x: f64, y: f64, dy: f64) { core::wd_ui_wheel(x, y, dy); }

    pub fn jump(&mut self, down: bool) {
        if down && !self.menu_mode && !self.paused && !self.sliding && !self.game.held { self.jumps += 1; }
        self.game.held = down;
    }

    /// A key by its DOM `KeyboardEvent.code` name. `jump` is whether it is a jump key.
    pub fn key(&mut self, code: &str, down: bool, repeat: bool, jump: bool) {
        if !down { if jump { self.jump(false); } return; }
        if self.menu_mode || self.paused || self.won || core::wd_ui_menu_active() != 0 {
            let mapped = if jump && !self.won { "Enter" } else { code };
            if jump && repeat { return; }
            let hit = unsafe { core::wd_ui_menu_key(mapped.as_ptr(), mapped.len()) } != 0;
            if hit { return; }
        }
        if code == "Escape" && !self.menu_mode {
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
        core::wd_ui_step(dt);
        self.game.menu = self.menu_mode;
        if !self.paused {
            core::wd_trig_time(now);
            let r = core::wd_game_update(dt, self.game.held as u32);
            if (r >> 30) & 1 != 0 && !self.menu_mode { self.restarted(); }
        }
        if self.sliding && core::wd_game_slide_state() == 2 { self.sliding = false; core::wd_ui_hud_pause(1); }
        self.game.pv_step(dt / 1000.0, self.paused);
        let info = self.game.draw(canvas);
        self.cam = (info.cam_x, info.cam_y);
        self.ui.cam = self.cam;
        self.was_dead = info.dead;
        if !self.menu_mode { core::wd_ui_hud_update(info.percent.clamp(0.0, 100.0)); }
        if !self.menu_mode && !self.paused {
            if !self.won { self.play_time += dt / 1000.0; }
            if !self.won && core::wd_game_percent(2) != 0.0 { self.won = true; self.win_clock = 0.0; self.win_stage = 0; }
            if self.won {
                self.win_clock += dt;
                if self.win_stage == 0 && self.win_clock >= 1000.0 {
                    self.win_stage = 1;
                    if !self.practice { core::wd_coins_commit(); }
                    core::wd_ui_banner(self.practice as u32);
                }
                if self.win_stage == 1 && self.win_clock >= 2800.0 {
                    self.win_stage = 2;
                    let main = self.levels.iter().any(|l| l.id == format!("level_{}", self.level_id));
                    core::wd_ui_end_coins(main as u32, core::wd_game_coin_count(main as u32), 0, 0);
                    let msg = "Awesome!";
                    unsafe { core::wd_ui_end_layer(self.attempts, self.jumps, self.play_time, self.practice as u32, msg.as_ptr(), msg.len()); }
                }
            }
        }
        let assets = &self.game.assets;
        self.ui.draw(canvas, assets);
        self.handle_ui();
    }
}
