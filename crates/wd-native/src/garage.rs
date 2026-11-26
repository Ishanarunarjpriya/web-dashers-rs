//! The icon kit's host side (`web/garage.js`): choices are saved like the web version saves them, atlases load on request.
use crate::host::Host;
use wd_core as core;
use wd_core::jsonread::{parse, Json};

fn pair(v: Option<&Json>, fx: f64, fy: f64) -> (f64, f64) {
    let s = v.and_then(|v| v.str()).unwrap_or("");
    let n: Vec<f64> = s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).filter(|t| !t.is_empty()).filter_map(|t| t.parse().ok()).collect();
    if n.len() >= 2 { (n[0], n[1]) } else { (fx, fy) }
}
fn suffix(s: &str) -> i64 { s.trim_end_matches(".png").rsplit('_').next().and_then(|d| d.parse().ok()).unwrap_or(0) }

fn set_kv(k: &str, v: &str) { unsafe { core::wd_gar_set(k.as_ptr(), k.len(), v.as_ptr(), v.len()); } }
fn text(n: usize) -> String { let p = core::wd_result_ptr(); unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() } }

impl Host {
    fn send_anims(&mut self) {
        if self.anims_sent { return; }
        self.anims_sent = true;
        for (i, kind, file) in [(0u32, "robot", "Robot_AnimDesc.json"), (1, "spider", "Spider_AnimDesc.json")] {
            let Ok(src) = std::fs::read_to_string(self.game.assets.root.join("sheets").join(file)) else { continue };
            let Ok(j) = parse(&src) else { continue };
            let Some(Json::Obj(cont)) = j.get("animationContainer") else { continue };
            let mut keys: Vec<&(String, Json)> = cont.iter().collect();
            keys.sort_by_key(|(k, _)| suffix(k));
            let pats: &[&str] = if kind == "robot" { &["Robot_idle_001.png", "Robot_idle_", "Robot_idle01_", "Robot_idle02_", "Robot_run_001.png", "Robot_run_"] } else { &["Spider_idle01_001.png", "Spider_idle01_", "Spider_idle02_", "Spider_idle_", "Spider_walk_"] };
            let mut frame = None;
            for p in pats { if let Some((_, f)) = keys.iter().find(|(k, _)| if p.ends_with(".png") { k == p } else { k.starts_with(p) && k.ends_with(".png") && k[p.len()..k.len() - 4].chars().all(|c| c.is_ascii_digit()) }) { frame = Some(f); break; } }
            let frame = frame.or_else(|| keys.first().map(|(_, f)| f));
            let Some(Json::Obj(parts)) = frame else { continue };
            let mut lines = Vec::new();
            for (key, sd) in parts {
                if !key.starts_with("sprite_") { continue; }
                let tag = sd.get_str("tag").and_then(|t| t.parse::<i64>().ok()).unwrap_or(0);
                let (px, py) = pair(sd.get("position"), 0.0, 0.0);
                let (sx, sy) = pair(sd.get("scale"), 1.0, 1.0);
                let (fx, fy) = pair(sd.get("flipped"), 0.0, 0.0);
                let z = sd.get_str("zValue").and_then(|t| t.parse::<f64>().ok()).unwrap_or(tag as f64);
                let rot = sd.get_str("rotation").and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
                lines.push(format!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}", tag, sd.get_str("texture").unwrap_or(""), px, py, sx, sy, (fx != 0.0) as u8, (fy != 0.0) as u8, z, rot));
            }
            let t = lines.join("\n");
            unsafe { core::wd_gar_anim(i, t.as_ptr(), t.len()); }
        }
    }

    pub(crate) fn pv_cfg(&mut self) {
        let hexv = |s: &Store2, k: &str, d: &str| i64::from_str_radix(s.get(k).unwrap_or(d), 16).unwrap_or(0);
        let st = Store2(&self.store);
        let (m, s2) = (hexv(&st, "iconMainColor", "04FF00"), hexv(&st, "iconSecondaryColor", "00FBFF"));
        let g = |k: &str, d: &str| self.store.get(k).unwrap_or(d).to_string();
        let (p, sh, b, w, bi) = (g("iconCurrentPlayer", "player_01"), g("iconCurrentShip", "ship_01"), g("iconCurrentBall", "player_ball_01"), g("iconCurrentWave", "dart_01"), g("iconCurrentBird", "bird_01"));
        self.game.pv.set_cfg(m, s2, &p, &sh, &b, &w, &bi);
    }

    pub(crate) fn garage_open(&mut self) {
        self.send_anims();
        for (k, d) in [("iconMainColor", "04FF00"), ("iconSecondaryColor", "00FBFF"), ("iconCurrentPlayer", "player_01"), ("iconCurrentShip", "ship_01"), ("iconCurrentBall", "player_ball_01"), ("iconCurrentWave", "dart_01"), ("iconCurrentBird", "bird_01"), ("iconCurrentSpider", "spider_01"), ("iconCurrentRobot", "robot_01")] {
            let v = self.store.get(k).unwrap_or(d).to_string();
            set_kv(k, &v);
        }
        let mut name: String = self.store.get("playerName").unwrap_or("Player").chars().filter(|c| *c != '\r' && *c != '\n').take(20).collect();
        if name.trim().is_empty() { name = "Player".into(); }
        set_kv("playerName", &name);
        set_kv("customCubeIcon", "0");
        core::wd_gar_open();
    }

    pub(crate) fn garage_need(&mut self) {
        let need = text(core::wd_gar_need());
        let list: Vec<String> = parse(&need).ok().map(|j| j.arr().unwrap_or(&[]).iter().filter_map(|v| v.str().map(|s| s.to_string())).collect()).unwrap_or_default();
        let root = self.game.assets.root.clone();
        for b in list {
            let ok = match self.game.pv.icon_sheet(&root, &b) {
                Some(sh) => {
                    for (name, f) in &sh.frames { let key = format!("icon_{b}/{name}"); unsafe { core::wd_ui_size(key.as_ptr(), key.len(), f.src_w, f.src_h); } }
                    true
                }
                None => false,
            };
            unsafe { core::wd_gar_ready(b.as_ptr(), b.len(), ok as u32); }
        }
        core::wd_gar_rebuild();
    }

    pub(crate) fn garage_save(&mut self) {
        let changes = text(core::wd_gar_changes());
        for line in changes.lines() {
            if let Some(i) = line.find('=') { let (k, v) = (&line[..i], &line[i + 1..]); if k != "customCubeIconSrc" { self.store.set(k, v); } }
        }
        self.pv_cfg();
    }
}

pub struct Store2<'a>(&'a crate::store::Store);
impl<'a> Store2<'a> { fn get(&self, k: &str) -> Option<&str> { self.0.get(k) } }
