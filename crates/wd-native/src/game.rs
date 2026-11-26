//! The game as the web host runs it: load a level into wd-core, step it, and draw its render buffer on the canvas.
use crate::assets::{load_png, Assets, Frame, Image};
use crate::canvas::{Canvas, Sprite, H, W};
use std::path::Path;
use crate::player_vis::PlayerVis;
use wd_core as core;

fn result(n: usize) -> String {
    let p = core::wd_result_ptr();
    unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() }
}

pub struct RenderInfo { pub cam_x: f64, pub cam_y: f64, pub dead: bool, pub percent: f64 }

pub struct Game {
    pub assets: Assets,
    bg: Option<Image>,
    ground: Option<Image>,
    ground2: Option<Image>,
    /// the level's sprite images by the index the render buffer uses: (sheet, frame)
    table: Vec<Option<(String, Frame)>>,
    pub held: bool,
    pub menu: bool,
    pub pv: PlayerVis,
    pub fx: crate::endfx::EndFx,
    pub shake: (f64, f64),
    pub ldm: bool,
    pub restart_flag: bool,
    pub level: String,
    pub music_offset: f64,
    pub coins: (String, String, f64, f64),
    pub ground_w: f64,
    pub has_ground2: bool,
}

impl Game {
    pub fn new(assets: Assets) -> Game { Game { assets, bg: None, ground: None, ground2: None, table: Vec::new(), held: false, menu: false, pv: PlayerVis::new(), fx: crate::endfx::EndFx::new(), shake: (0.0, 0.0), ldm: false, restart_flag: true, level: String::new(), music_offset: 0.0, coins: (String::new(), String::new(), 0.0, 0.0), ground_w: 1012.0, has_ground2: false } }

    /// `loadLevelText` of main.js for a shipped level.
    pub fn load_level(&mut self, id: &str) -> Result<(), String> {
        let text = std::fs::read_to_string(self.assets.root.join("levels").join(format!("{id}.txt"))).map_err(|e| format!("level {id}: {e}"))?;
        self.load_text(id, &text, true)
    }

    /// `loadLevelText`: `official` levels are the shipped ones (their coins are saved by name).
    pub fn load_text(&mut self, id: &str, text: &str, official: bool) -> Result<(), String> {
        self.level = id.to_string();
        let official = official && !(id.len() >= 5 && id.chars().all(|c| c.is_ascii_digit()));
        let key = if official { format!("level_{id}") } else { id.to_string() };
        unsafe {
            core::wd_set_ldm(self.ldm as u32);
            core::wd_set_official(official as u32);
            let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1);
            core::wd_visual_seed(seed);
            let (sec, usr, ts, tu) = self.coins.clone();
            core::wd_coins_config(official as u32, id.starts_with("local_") as u32, 0, key.as_ptr(), key.len(), sec.as_ptr(), sec.len(), usr.as_ptr(), usr.len(), ts, tu);
            core::wd_game_load_level(text.as_ptr(), text.len());
            core::wd_mirror_enable(1);
            core::wd_game_after_load();
            core::wd_game_restart_level(-1.0);
        }
        let n = core::wd_render_info();
        let info = core::jsonread::parse(&result(n)).map_err(|e| format!("render info: {e}"))?;
        self.music_offset = info.get_num("musicOffset").unwrap_or(0.0);
        let bg_id = info.get_str("background").unwrap_or("01").to_string();
        let ground_idx = format!("{:02}", info.get_str("ground").and_then(|g| g.parse::<i64>().ok()).unwrap_or(0) + 1);
        let root = self.assets.root.clone();
        self.bg = load_png(&root.join("game-bg").join(format!("game_bg_{bg_id}_001-hd.png"))).ok();
        self.ground = load_png(&root.join("game-ground").join(format!("groundSquare_{ground_idx}_001.png"))).ok();
        self.ground2 = load_png(&root.join("game-ground").join(format!("groundSquare_{ground_idx}_2_001.png"))).ok();
        self.ground_w = self.ground.as_ref().map_or(1012.0, |g| g.w as f64);
        self.has_ground2 = self.ground2.is_some();
        core::wd_render_setup(self.ground.as_ref().map_or(1012.0, |g| g.w as f64), self.ground2.is_some() as u32, self.bg.as_ref().map_or(0.0, |b| b.h as f64));
        unsafe { core::wd_game_restart_level(-1.0); }
        self.load_frame_table()?;
        Ok(())
    }

    pub fn reload_art(&mut self) -> Result<(), String> {
        let n = core::wd_render_info();
        let info = core::jsonread::parse(&result(n)).map_err(|e| format!("render info: {e}"))?;
        let bg_id = info.get_str("background").unwrap_or("01").to_string();
        let ground_idx = format!("{:02}", info.get_str("ground").and_then(|g| g.parse::<i64>().ok()).unwrap_or(0) + 1);
        let root = self.assets.root.clone();
        self.bg = load_png(&root.join("game-bg").join(format!("game_bg_{bg_id}_001-hd.png"))).ok();
        self.ground = load_png(&root.join("game-ground").join(format!("groundSquare_{ground_idx}_001.png"))).ok();
        self.ground2 = load_png(&root.join("game-ground").join(format!("groundSquare_{ground_idx}_2_001.png"))).ok();
        core::wd_render_setup(self.ground.as_ref().map_or(1012.0, |g| g.w as f64), self.ground2.is_some() as u32, self.bg.as_ref().map_or(0.0, |b| b.h as f64));
        Ok(())
    }

    pub fn refresh_frames(&mut self) { let _ = self.load_frame_table(); }

    fn load_frame_table(&mut self) -> Result<(), String> {
        let n = core::wd_render_frames();
        let j = core::jsonread::parse(&result(n)).map_err(|e| format!("frame table: {e}"))?;
        self.table = j.arr().unwrap_or(&[]).iter().map(|e| {
            let a = e.arr()?;
            let (sheet, name) = (a.first()?.str()?, a.get(1)?.str()?);
            let f = self.assets.sheets.get(sheet)?.frames.get(name)?;
            Some((sheet.to_string(), *f))
        }).collect();
        Ok(())
    }

    pub fn restart(&mut self) { unsafe { core::wd_game_restart_level(-1.0); } }

    pub fn step(&mut self, dt_ms: f64, now_ms: f64) {
        core::wd_trig_time(now_ms);
        core::wd_game_update(dt_ms, self.held as u32);
    }

    /// `pvStep`: the player's layers for this frame (`dt` seconds).
    pub fn pv_step(&mut self, dt: f64, paused: bool) {
        let (cx, cy) = (core::wd_game_scene(0), core::wd_game_scene(1));
        let hidden = false;
        let mp = core::wd_mirror_progress();
        let n = core::wd_pv_events();
        let events = if n > 0 { let p = core::wd_result_ptr(); unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() } } else { String::new() };
        self.pv.frame(&self.assets, if paused { 0.0 } else { dt }, (cx, cy), self.menu, hidden, self.restart_flag, mp, paused, self.ldm, &events);
        self.restart_flag = false;
    }

    /// The level seen through the editor camera: sprites at the zoom, the playtest path, the grid.
    fn draw_editor_view(&mut self, c: &mut Canvas, buf: &[f64], count: usize, tiles: usize) {
        let z = buf[33];
        let (ex, ey) = (-buf[34] / z, -buf[35] / z);
        let base = 40 + tiles;
        for k in 0..count {
            let o = base + k * 12;
            let Some(Some((sheet, f))) = self.table.get(buf[o] as usize) else { continue };
            let Some(sh) = self.assets.sheets.get(sheet) else { continue };
            let (sx, sy) = ((buf[o + 1] + ex) * z, (buf[o + 2] + ey) * z);
            if sx < -400.0 || sx > W as f64 + 400.0 || sy < -400.0 || sy > H as f64 + 400.0 { continue; }
            let fl = buf[o + 9] as i64;
            c.frame(&sh.img, f, Sprite { x: sx, y: sy, rot: buf[o + 3], sx: buf[o + 4] * z, sy: buf[o + 5] * z, alpha: buf[o + 6], tint: buf[o + 7] as i64, additive: buf[o + 8] == 1.0, flip_x: fl & 1 != 0, flip_y: fl & 2 != 0 });
        }
        // playtest path and death crosses
        let n = core::wd_ed_marks();
        if n > 0 {
            let a = unsafe { std::slice::from_raw_parts(core::wd_ed_marks_ptr(), n * 6) };
            for i in 0..n {
                let o = i * 6;
                if a[o] == 0.0 { c.line((a[o + 1] + ex) * z, (a[o + 2] + ey) * z, (a[o + 3] + ex) * z, (a[o + 4] + ey) * z, 2.0 * z, a[o + 5] as u32 & 0xffffff, 0.95); }
                else { let (x, y, s) = ((a[o + 1] + ex) * z, (a[o + 2] + ey) * z, 28.0 * z); c.line(x - s, y - s, x + s, y + s, 13.0 * z, 0xff0000, 1.0); c.line(x - s, y + s, x + s, y - s, 13.0 * z, 0xff0000, 1.0); }
            }
        }
        // grid (`_updateEditorGrid`)
        let gs = 60.0;
        let (cam_x, cam_y2) = (buf[34] / z, buf[35] / z);
        let mut x = (-cam_x) % gs;
        while x < W as f64 / z + gs { c.line(x * z + 0.5, 0.0, x * z + 0.5, H as f64, 1.0, 0x000000, 0.4); x += gs; }
        let mut y = (-cam_y2 - 20.0) % gs;
        while y < H as f64 / z + gs { c.line(0.0, y * z + 0.5, W as f64, y * z + 0.5, 1.0, 0x000000, 0.4); y += gs; }
        let sl = -buf[34];
        if (-50.0..=W as f64 + 50.0).contains(&sl) { c.line(sl + 0.5, 0.0, sl + 0.5, H as f64, 1.0, 0xffffff, 1.0); }
        let gl = (-20.0 + 60.0 * 8.0) * z - buf[35];
        if (-50.0..=H as f64 + 50.0).contains(&gl) { c.line(0.0, gl + 0.5, W as f64, gl + 0.5, 1.0, 0xffffff, 1.0); }
    }

    pub fn draw(&mut self, c: &mut Canvas) -> RenderInfo {
        let n = core::wd_render_build();
        let buf = unsafe { std::slice::from_raw_parts(core::wd_render_ptr(), n) };
        let (cam_x, cam_y, count, tiles) = (buf[0], buf[1], buf[2] as usize, buf[18] as usize);
        let (shake_x, shake_y) = (buf[31], buf[32]);
        let menu_hidden = false;
        let _ = menu_hidden;
        let (shake_x, shake_y) = (shake_x + self.shake.0, shake_y + self.shake.1);
        let (ox, oy) = (-cam_x + shake_x, cam_y + shake_y);
        c.fill(buf[22] as u32);
        // background: the image over its own mirror, tiled
        if let Some(bg) = &self.bg {
            let (tw, ih) = (bg.w as f64, bg.h as f64);
            let th = ih * 2.0;
            let tint = buf[22] as i64 & 0xffffff;
            let tx = ((buf[20] % tw) + tw) % tw;
            let ty = ((buf[21] % th) + th) % th;
            let mut xx = -tx;
            while xx < W as f64 {
                let mut yy = -ty;
                while yy < H as f64 { c.image(bg, xx, yy, tint, false); c.image(bg, xx, yy + ih, tint, true); yy += th; }
                xx += tw;
            }
        }
        if buf[33] > 0.0 { self.draw_editor_view(c, buf, count, tiles); return RenderInfo { cam_x, cam_y, dead: false, percent: 0.0 }; }
        if !self.menu { self.fx.draw_back(c); }
        let base = 40 + tiles;
        let mut i = 0;
        let mut draw_sprite = |c: &mut Canvas, o: usize, table: &Vec<Option<(String, Frame)>>, assets: &Assets| {
            let Some(Some((sheet, f))) = table.get(buf[o] as usize) else { return };
            let Some(sh) = assets.sheets.get(sheet) else { return };
            let fl = buf[o + 9] as i64;
            c.frame(&sh.img, f, Sprite { x: buf[o + 1] + ox, y: buf[o + 2] + oy, rot: buf[o + 3], sx: buf[o + 4], sy: buf[o + 5], alpha: buf[o + 6], tint: buf[o + 7] as i64, additive: buf[o + 8] == 1.0, flip_x: fl & 1 != 0, flip_y: fl & 2 != 0 });
        };
        while i < count { let o = base + i * 12; if buf[o + 10] == 1.0 { break; } draw_sprite(c, o, &self.table, &self.assets); i += 1; }
        let on = !self.menu;
        self.fx.draw_wall(c, &self.assets, ox, oy, core::wd_game_scene(5), core::wd_game_scene(4), 0x04ff00, on);
        if on { self.fx.draw_portal_particles(c, &self.assets, ox, oy); }
        if !self.menu { self.pv.draw_world(c, &self.assets); }
        while i < count { let o = base + i * 12; draw_sprite(c, o, &self.table, &self.assets); i += 1; }
        if !self.menu { self.pv.draw_top(c, &self.assets); }
        // ground
        if let Some(g) = &self.ground {
            let (gy, tint) = (buf[15], buf[23] as i64);
            let tint2 = buf[24] as i64;
            for t in 0..tiles {
                let tx = buf[40 + t];
                c.image(g, tx, gy, tint, false);
                if let (Some(g2), true) = (&self.ground2, buf[25] != 0.0) { c.image(g2, tx, gy, tint2, false); }
                if buf[17] != 0.0 { c.image(g, tx, buf[16] - g.h as f64, tint, true); }
            }
        }
        if !self.menu { self.pv.draw_screen(c, &self.assets); self.fx.draw_front(c, &self.assets); }
        RenderInfo { cam_x, cam_y, dead: buf[6] != 0.0, percent: buf[30] * 100.0 }
    }
}

pub fn assets_ok(root: &Path) -> bool { root.join("sheets").is_dir() }
