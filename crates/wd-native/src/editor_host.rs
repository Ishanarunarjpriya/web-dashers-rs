//! The creator menu and the level editor (`web/editor.js` and the editor part of `main.js`).
use crate::host::Host;
use wd_core as core;
use wd_core::jsonread::{parse, Json};

const NEW_LEVEL_STRING: &str = include_str!("new_level.txt");
const KEY: &str = "created_levels";

fn text(n: usize) -> String { let p = core::wd_result_ptr(); unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() } }

impl Host {
    fn created(&self) -> Vec<Json> { self.store.get(KEY).and_then(|s| parse(s).ok()).and_then(|j| j.arr().map(|a| a.to_vec())).unwrap_or_default() }
    fn write_created(&mut self, items: &[String]) { self.store.set(KEY, &format!("[{}]", items.join(","))); self.store.flush(); }
    fn created_raw(&self) -> Vec<String> { self.created().iter().map(Self::json_of_pub).collect() }
    pub(crate) fn json_of_pub(j: &Json) -> String {
        match j {
            Json::Str(s) => Host::json_str(s), Json::Num(n) => format!("{n}"), Json::Bool(b) => b.to_string(), Json::Null => "null".into(),
            Json::Arr(a) => format!("[{}]", a.iter().map(Self::json_of_pub).collect::<Vec<_>>().join(",")),
            Json::Obj(o) => format!("{{{}}}", o.iter().map(|(k, v)| format!("{}:{}", Host::json_str(k), Self::json_of_pub(v))).collect::<Vec<_>>().join(",")),
        }
    }
    fn push_created(&mut self) { let j = format!("[{}]", self.created_raw().join(",")); unsafe { core::wd_em_levels(j.as_ptr(), j.len()); } }
    fn next_local_id(&self) -> String {
        let max = self.created().iter().filter_map(|l| l.get_str("createdId")).filter_map(|id| id.split('_').nth(1).and_then(|n| n.parse::<i64>().ok())).max().unwrap_or(0);
        format!("local_{}", max + 1)
    }
    fn view_id(&self) -> String { text(core::wd_em_view_id()) }
    /// Replaces or adds a field of the created level `id`.
    fn set_field(&mut self, id: &str, fields: &[(&str, String)]) -> bool {
        let mut found = false;
        let items: Vec<String> = self.created().iter().map(|l| {
            if l.get_str("createdId") != Some(id) { return Self::json_of_pub(l); }
            found = true;
            let Json::Obj(o) = l else { return Self::json_of_pub(l) };
            let mut parts: Vec<String> = o.iter().filter(|(k, _)| !fields.iter().any(|(f, _)| f == k)).map(|(k, v)| format!("{}:{}", Host::json_str(k), Self::json_of_pub(v))).collect();
            for (k, v) in fields { parts.push(format!("{}:{}", Host::json_str(k), v)); }
            format!("{{{}}}", parts.join(","))
        }).collect();
        if found { self.write_created(&items); }
        found
    }

    pub(crate) fn creator_event(&mut self, ev: &str) -> bool {
        if ev == "Creator" { self.push_created(); return true; }
        let Some(rest) = ev.strip_prefix("Pop(\"") else { return false };
        let Some(q) = rest.find('"') else { return false };
        let name = &rest[..q];
        let n: usize = rest[q + 1..].trim_start_matches(',').trim().trim_end_matches(')').trim().parse().unwrap_or(0);
        match name {
            "EmNew" => {
                let levels = self.created();
                let mut counter = 0;
                while levels.iter().any(|l| l.get_str("levelName") == Some(&format!("Unnamed {counter}"))) { counter += 1; }
                let id = self.next_local_id();
                let obj = format!("{{\"levelName\":{},\"song\":\"Stereo Madness\",\"songId\":-1,\"levelId\":null,\"levelString\":{},\"levelLength\":0,\"normalBest\":0,\"practiceBest\":0,\"description\":\"\",\"version\":1,\"status\":\"Unverified\",\"createdId\":{},\"lastModified\":0}}", Host::json_str(&format!("Unnamed {counter}")), Host::json_str(NEW_LEVEL_STRING), Host::json_str(&id));
                let mut items = self.created_raw(); items.push(obj); self.write_created(&items);
                self.push_created();
                unsafe { core::wd_em_open_new(id.as_ptr(), id.len()); }
            }
            "EmField" => { let id = self.view_id(); if !id.is_empty() { let v = text(core::wd_em_field(n as u32)); self.set_field(&id, &[(if n == 0 { "levelName" } else { "description" }, Host::json_str(&v))]); } }
            "EmDelete" => {
                let id = self.view_id();
                if self.created().iter().any(|l| l.get_str("createdId") == Some(&id)) {
                    let items: Vec<String> = self.created().iter().filter(|l| l.get_str("createdId") != Some(&id)).map(Self::json_of_pub).collect();
                    self.write_created(&items); self.push_created(); core::wd_em_after_delete();
                }
            }
            "EmEdit" => {
                let id = self.view_id();
                if let Some(l) = self.created().into_iter().find(|l| l.get_str("createdId") == Some(&id)) { let s = l.get_str("levelString").unwrap_or("").to_string(); let _ = self.start_editor(&s, &id); }
            }
            "EmPlay" => { let id = self.view_id(); self.play_created(&id); }
            "EdArt" => { let _ = self.game.reload_art(); }
            "EdPm" => match n {
                1 => { if self.save_editor() { self.leave_editor(true); } }
                2 => { if self.save_editor() { self.leave_editor(false); } }
                3 => { self.save_editor(); }
                4 => self.leave_editor(false),
                _ => {}
            },
            "EdPmGlow" => { let g = core::wd_ed_glow(); self.store.set("opt_showEditorGlow", &g.to_string()); }
            _ => return false,
        }
        true
    }

    pub(crate) fn play_created(&mut self, id: &str) {
        let Some(l) = self.created().into_iter().find(|l| l.get_str("createdId") == Some(id)) else { return };
        let (txt, song_id) = (l.get_str("levelString").unwrap_or("").to_string(), l.get_num("songId").unwrap_or(-1.0));
        let song_name = l.get_str("song").unwrap_or("").to_string();
        self.audio.stop_music();
        self.current_song = None;
        self.net.name = l.get_str("levelName").unwrap_or("Level").to_string();
        if song_id > 0.0 {
            // a downloaded custom song
            let sid = format!("{}", song_id as i64);
            if let Some(b) = self.cached_song(&sid) { self.current_song = Some(std::sync::Arc::new(b)); }
            else { self.pending_created = Some((id.to_string(), txt)); self.request_custom_song(sid, 1); return; }
        } else if let Some(lv) = self.levels.iter().find(|x| x.name == song_name) {
            let f = lv.song_file.clone().unwrap_or_else(|| lv.name.replace(' ', ""));
            self.current_song = std::fs::read(self.game.assets.root.join("music").join(format!("{f}.mp3"))).ok().map(std::sync::Arc::new);
        }
        self.finish_play_created(id, &txt);
    }

    pub(crate) fn finish_play_created(&mut self, id: &str, txt: &str) {
        self.net.active = true; self.online_offset = 0.0;
        self.created_play_id = Some(id.to_string());
        core::wd_em_close();
        if let Err(e) = self.play_text(id, txt) { eprintln!("{e}"); }
    }

    /// A copy of an online level in My Levels.
    pub(crate) fn add_copied_level(&mut self, l: &crate::online_host::OLevel, text: &str) {
        let id = self.next_local_id();
        let obj = format!("{{\"levelName\":{},\"song\":{},\"songId\":-1,\"levelId\":null,\"levelString\":{},\"levelLength\":{},\"normalBest\":0,\"practiceBest\":0,\"description\":{},\"version\":1,\"status\":\"Copy\",\"createdId\":{},\"lastModified\":0}}", Host::json_str(&l.name), Host::json_str(&l.song_name), Host::json_str(text), l.length, Host::json_str(&l.desc), Host::json_str(&id));
        let mut items = self.created_raw(); items.push(obj); self.write_created(&items);
    }

    /// Test hook: the creator menu with a few levels in it.
    pub fn open_creator(&mut self) { self.push_created(); core::wd_em_open_creator(); }

    pub(crate) fn start_editor(&mut self, level: &str, id: &str) -> Result<(), String> {
        self.audio.stop_music();
        self.game.ldm = false;
        self.game.load_text("editor", level, false)?;
        core::wd_ui_start_game();
        core::wd_menu_mode(0);
        self.menu_mode = false; self.won = false; self.sliding = false; self.paused = false;
        unsafe { core::wd_ed_open(level.as_ptr(), level.len(), id.as_ptr(), id.len()); }
        let names = format!("[{}]", self.levels.iter().map(|l| Host::json_str(&l.name)).collect::<Vec<_>>().join(","));
        let song_id = self.created().iter().find(|l| l.get_str("createdId") == Some(id)).and_then(|l| l.get_num("songId")).unwrap_or(-1.0);
        unsafe { core::wd_ed_set_songs(names.as_ptr(), names.len(), song_id); }
        core::wd_ed_set_glow(self.store.get_f("opt_showEditorGlow", 0.0) as u32);
        self.editor_on = true; self.ed_id = id.to_string(); self.ed_frames_ver = core::wd_ed_frames_ver();
        self.game.refresh_frames();
        Ok(())
    }

    fn save_editor(&mut self) -> bool {
        let id = self.ed_id.clone();
        let mut fields = vec![("levelString", Host::json_str(&text(core::wd_ed_level_string()))), ("lastModified", "0".to_string())];
        let pid = core::wd_ed_pending_song();
        if !pid.is_nan() { fields.push(("songId", format!("{pid}"))); let n = text(core::wd_ed_pending_song_name()); if !n.is_empty() { fields.push(("song", Host::json_str(&n))); } }
        let ok = self.set_field(&id, &fields);
        core::wd_ed_clear_pending_song();
        ok
    }

    fn leave_editor(&mut self, play: bool) {
        self.editor_on = false;
        core::wd_ed_close();
        self.audio.stop_music();
        self.menu_mode = true; core::wd_menu_mode(1); core::wd_ui_main_menu_again();
        let id = self.ed_id.clone();
        self.push_created();
        core::wd_ui_start_game();
        core::wd_ui_main_menu_again();
        unsafe { core::wd_em_open_new(id.as_ptr(), id.len()); }
        if play { self.play_created(&id); }
        let _ = self.game.reload_art();
    }

    /// Per frame while the editor is open.
    pub(crate) fn editor_frame(&mut self, dt: f64) {
        // playtest music follows start / pause / stop
        let ev = core::wd_ed_pt(0);
        if ev != self.ed_pt_event {
            self.ed_pt_event = ev;
            if core::wd_ed_pt(1) != 0.0 && core::wd_ed_pt(2) == 0.0 {
                let id = self.created().iter().find(|l| l.get_str("createdId") == Some(&self.ed_id)).and_then(|l| l.get_num("songId")).unwrap_or(-1.0);
                let pid = core::wd_ed_pending_song();
                let sid = if pid.is_nan() { id } else { pid };
                let idx = (-sid - 1.0).max(0.0) as usize;
                if sid > 0.0 {
                    let key = format!("{}", sid as i64);
                    if let Some(b) = self.ed_custom_song.clone().filter(|_| self.ed_song_key == key) { let off = self.game.music_offset + core::wd_ed_pt(3); self.audio.start_song_bytes(b, off); }
                    else if let Some(b) = self.cached_song(&key) { self.ed_song_key = key; let a = std::sync::Arc::new(b); self.ed_custom_song = Some(a.clone()); let off = self.game.music_offset + core::wd_ed_pt(3); self.audio.start_song_bytes(a, off); }
                    else { self.ed_song_key = key.clone(); self.request_custom_song(key, 0); }
                } else { if let Some(lv) = self.levels.get(idx) { let f = lv.song_file.clone().unwrap_or_else(|| lv.name.replace(' ', "")); let off = self.game.music_offset + core::wd_ed_pt(3); self.audio.start_song(&f, off); } }
            } else if core::wd_ed_pt(1) != 0.0 { self.audio.pause(); } else { self.audio.stop_music(); }
        }
        if core::wd_ed_pt(1) != 0.0 { core::wd_trig_time(self.t_now); }
        core::wd_ed_frame(dt, self.ed_keys | if self.ed_jump { 16 } else { 0 });
        let v = core::wd_ed_frames_ver();
        if v != self.ed_frames_ver { self.ed_frames_ver = v; self.game.refresh_frames(); }
    }

    /// Editor keys by DOM code; true when handled.
    pub(crate) fn editor_key(&mut self, code: &str, down: bool, repeat: bool, shift: bool) -> bool {
        let bit = match code { "ArrowLeft" => 1, "ArrowRight" => 2, "ArrowUp" => 4, "ArrowDown" => 8, _ => 0 };
        if bit != 0 { if down { self.ed_keys |= bit; } else { self.ed_keys &= !bit; } return true; }
        if !down { return false; }
        let acts = [("Digit1", 0), ("Digit2", 1), ("Digit3", 2), ("KeyW", 3), ("KeyA", 4), ("KeyS", 5), ("KeyD", 6), ("Equal", 7), ("Minus", 8)];
        for (c, n) in acts { if c == code && (!repeat || n >= 3) { core::wd_ed_key_action(n, if shift { 0 } else { 1 }); return true; } }
        false
    }

    /// A custom song is not needed for the creator list; the picker textures of the colour picker are sent once.
    pub(crate) fn send_cp_textures(&mut self) {
        let Some(sh) = self.game.assets.sheets.get("CCControlColourPickerSpriteSheet-uhd") else { return };
        for (k, (name, size)) in [("huePickerBackground.png", 252usize), ("colourPickerBackground.png", 185), ("colourPickerShadow.png", 185), ("colourPickerOverlay.png", 185)].iter().enumerate() {
            let Some(f) = sh.frames.get(*name) else { continue };
            let (fw, fh) = (f.w as usize, f.h as usize);
            let mut px = vec![0u8; size * size * 4];
            for y in 0..*size { for x in 0..*size {
                let (sx, sy) = (((x as f64 + 0.5) / *size as f64 * fw as f64 - 0.5).clamp(0.0, fw as f64 - 1.0), ((y as f64 + 0.5) / *size as f64 * fh as f64 - 0.5).clamp(0.0, fh as f64 - 1.0));
                let (x0, y0) = (sx.floor() as usize, sy.floor() as usize);
                let (x1, y1) = ((x0 + 1).min(fw - 1), (y0 + 1).min(fh - 1));
                let (tx, ty) = (sx - x0 as f64, sy - y0 as f64);
                let at = |xx: usize, yy: usize| sh.img.px[(f.y as usize + yy) * sh.img.w + f.x as usize + xx];
                let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
                for ch in 0..4 { let v = (a[ch] as f64 * (1.0 - tx) + b[ch] as f64 * tx) * (1.0 - ty) + (c[ch] as f64 * (1.0 - tx) + d[ch] as f64 * tx) * ty; px[(y * size + x) * 4 + ch] = v.round() as u8; }
            } }
            unsafe { core::wd_ed_cp_tex(k as u32, px.as_ptr(), px.len()); }
        }
    }
}
