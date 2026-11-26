//! Online levels (`web/online.js`): the Geometry Dash server calls behind Search, Featured, Saved and Daily/Weekly/Event, through the proxy.
//! Requests run on their own threads; answers come back as messages the host feeds to the UI.
use crate::host::Host;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use wd_core as core;

const SECRET: &str = "Wmfd2893gb7";
const PROXY: &str = "https://gd-proxy.theblehlollipop.workers.dev/?url=";
const GD: &str = "https://www.boomlings.com/database/";

#[derive(Clone, Default)]
pub struct OLevel {
    pub id: String, pub name: String, pub author: String, pub desc: String, pub song_name: String, pub custom_song: String,
    pub difficulty: i64, pub stars: i64, pub downloads: i64, pub likes: i64, pub length: i64, pub featured: i64, pub epic: i64, pub coins: i64,
    pub best: f64, pub best_practice: f64, pub viewed: bool, pub daily_number: Option<i64>,
}

pub enum NetMsg {
    Results { levels: Vec<OLevel>, total: f64, offset: f64, count: f64 },
    Error(String),
    Special(OLevel),
    Song(String, String),
    Play { id: String, name: String, text: String, song: Option<Vec<u8>>, offset: f64, level: OLevel, official_song: i64 },
    PlayFailed(String),
    Copied(OLevel, String),
    SongData { id: String, bytes: Option<Vec<u8>>, purpose: u8 },
}

pub struct Net { pub tx: Sender<NetMsg>, pub rx: Receiver<NetMsg>, pub list: Vec<OLevel>, pub special: Option<OLevel>, pub busy: bool, pub active: bool, pub name: String }

impl Net {
    pub fn new() -> Net { let (tx, rx) = std::sync::mpsc::channel(); Net { tx, rx, list: Vec::new(), special: None, busy: false, active: false, name: String::new() } }
}

fn agent() -> ureq::Agent { ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build() }

fn post(path: &str, body: &str) -> Result<String, String> {
    let url = format!("{PROXY}{GD}{path}");
    agent().post(&url).set("Content-Type", "application/x-www-form-urlencoded").send_string(body).map_err(|e| e.to_string())?.into_string().map_err(|e| e.to_string())
}
fn get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let r = agent().get(url).call().map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut r.into_reader(), &mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn enc(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() { if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { o.push(b as char); } else { o.push_str(&format!("%{b:02X}")); } }
    o
}
fn dec(s: &str) -> String {
    let b = s.as_bytes(); let mut o = Vec::new(); let mut i = 0;
    while i < b.len() { if b[i] == b'%' && i + 2 < b.len() + 0 && i + 2 <= b.len() - 1 + 0 { if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) { o.push(v); i += 3; continue; } } if b[i] == b'+' { o.push(b' '); } else { o.push(b[i]); } i += 1; }
    String::from_utf8_lossy(&o).into_owned()
}
fn b64(s: &str) -> String {
    let t: Vec<u8> = s.bytes().map(|c| match c { b'-' => b'+', b'_' => b'/', c => c }).filter(|c| *c != b'=').collect();
    let val = |c: u8| match c { b'A'..=b'Z' => c - b'A', b'a'..=b'z' => c - b'a' + 26, b'0'..=b'9' => c - b'0' + 52, b'+' => 62, b'/' => 63, _ => 255 };
    let mut out = Vec::new(); let (mut acc, mut bits) = (0u32, 0);
    for c in t { let v = val(c); if v == 255 { return String::new(); } acc = (acc << 6) | v as u32; bits += 6; if bits >= 8 { bits -= 8; out.push((acc >> bits) as u8); acc &= (1 << bits) - 1; } }
    String::from_utf8_lossy(&out).into_owned()
}
fn kv(s: &str, sep: &str) -> std::collections::HashMap<String, String> {
    let p: Vec<&str> = s.split(sep).collect(); let mut o = std::collections::HashMap::new();
    let mut i = 0; while i + 1 < p.len() { o.insert(p[i].to_string(), p[i + 1].to_string()); i += 2; }
    o
}
fn num(m: &std::collections::HashMap<String, String>, k: &str) -> i64 { m.get(k).and_then(|v| v.trim().parse::<f64>().ok()).map_or(0, |v| v as i64) }

fn diff_of(m: &std::collections::HashMap<String, String>) -> i64 {
    if num(m, "25") == 1 { return 11; }
    if num(m, "17") == 1 {
        let d9 = m.get("9").and_then(|v| v.parse::<i64>().ok()); let d43 = m.get("43").and_then(|v| v.parse::<i64>().ok());
        if let Some(d) = d9 { if (1..=5).contains(&d) { return [7, 8, 6, 9, 10][(d - 1) as usize]; } }
        if let Some(d) = d43 { return match d { 3 => 7, 4 => 8, 0 => 6, 5 => 9, 6 => 10, _ => 8 }; }
        return 8;
    }
    (((num(m, "9") as f64) / 10.0).round() as i64).clamp(0, 5)
}

fn to_level(m: &std::collections::HashMap<String, String>, author: &str, song: &str) -> OLevel {
    OLevel { id: m.get("1").cloned().unwrap_or_default(), name: m.get("2").map_or("Unknown".to_string(), |s| s.trim().to_string()), author: if author.is_empty() { "Unknown".into() } else { author.into() },
        desc: b64(m.get("3").map_or("", |s| s)), song_name: song.into(), custom_song: m.get("35").map_or(String::new(), |s| s.trim().to_string()), difficulty: diff_of(m), stars: num(m, "18"), downloads: num(m, "10"),
        likes: num(m, "14"), length: num(m, "15"), featured: if num(m, "19") > 0 { 1 } else { 0 }, epic: num(m, "42"), coins: num(m, "37"), ..Default::default() }
}

fn parse_list(raw: &str, official: &dyn Fn(i64) -> String) -> (Vec<OLevel>, f64, f64, f64) {
    let sec: Vec<&str> = raw.split('#').collect();
    let mut players = std::collections::HashMap::new(); let mut songs = std::collections::HashMap::new();
    for ps in sec.get(1).unwrap_or(&"").split('|').filter(|s| !s.is_empty()) { let p: Vec<&str> = ps.split(':').collect(); if p.len() >= 2 { players.insert(p[0].to_string(), p[1].to_string()); } }
    for ss in sec.get(2).unwrap_or(&"").split("~:~").filter(|s| !s.is_empty()) { let sm = kv(ss, "~|~"); if let Some(id) = sm.get("1") { songs.insert(id.clone(), sm.get("2").cloned().unwrap_or_default()); } }
    let info: Vec<f64> = sec.get(3).unwrap_or(&"0:0:10").split(':').map(|v| v.parse().unwrap_or(0.0)).collect();
    let levels = sec.first().unwrap_or(&"").split('|').filter(|s| !s.is_empty()).map(|s| {
        let m = kv(s, ":"); let c = m.get("35").map_or("", |v| v.trim());
        let custom = !c.is_empty() && c != "0";
        let author = players.get(m.get("6").map_or("", |v| v.as_str())).cloned().unwrap_or_default();
        to_level(&m, &author, &if custom { songs.get(c).cloned().unwrap_or_else(|| format!("Song #{c}")) } else { official(num(&m, "12")) })
    }).collect();
    (levels, info.first().copied().unwrap_or(0.0), info.get(1).copied().unwrap_or(0.0), info.get(2).copied().unwrap_or(10.0))
}

fn author_of(raw: &str) -> String {
    for seg in raw.split('#').skip(1) {
        let p: Vec<&str> = seg.split(':').collect();
        if p.len() == 3 && p[0].chars().all(|c| c.is_ascii_digit()) && !p[0].is_empty() && !p[1].is_empty() && !p[1].contains(['#', '|']) && p[2].chars().all(|c| c.is_ascii_digit()) && !p[2].is_empty() { return p[1].to_string(); }
    }
    "Unknown".into()
}

fn download(id: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let raw = post("downloadGJLevel22.php", &format!("levelID={id}&secret={SECRET}"))?;
    if raw.trim() == "-1" || !raw.contains(':') { return Err("level not found".into()); }
    let m = kv(raw.split('#').next().unwrap_or(""), ":");
    if !m.contains_key("4") { return Err("no level data".into()); }
    Ok(m)
}

fn song_bytes(custom: &str) -> Option<Vec<u8>> {
    let info = post("getGJSongInfo.php", &format!("songID={custom}&secret={SECRET}")).ok()?;
    let url = dec(info.split("~|~10~|~").nth(1)?.split("~|~").next()?);
    if url.is_empty() { return None; }
    get_bytes(&format!("{PROXY}{url}")).ok()
}

fn json_s(s: &str) -> String { Host::json_str(s) }
impl OLevel {
    pub fn json(&self) -> String {
        let mut j = format!("{{\"id\":{},\"name\":{},\"author\":{},\"desc\":{},\"songName\":{},\"customSong\":{},\"difficulty\":{},\"stars\":{},\"downloads\":{},\"likes\":{},\"length\":{},\"featured\":{},\"epic\":{},\"coins\":{},\"best\":{},\"bestPractice\":{},\"viewed\":{}",
            json_s(&self.id), json_s(&self.name), json_s(&self.author), json_s(&self.desc), json_s(&self.song_name), json_s(&self.custom_song), self.difficulty, self.stars, self.downloads, self.likes, self.length, self.featured, self.epic, self.coins, self.best, self.best_practice, self.viewed as u8);
        if let Some(n) = self.daily_number { j.push_str(&format!(",\"dailyNumber\":{n}")); }
        j.push('}'); j
    }
}

impl Host {
    fn official_name(&self, n: i64) -> String { self.levels.get(n as usize).map_or("Unknown".into(), |l| l.name.clone()) }

    fn fill(&self, l: &mut OLevel) {
        l.best = self.store.get_f(&format!("bestPercent_level_{}", l.id), 0.0);
        l.best_practice = self.store.get_f(&format!("practiceBestPercent_level_{}", l.id), 0.0);
        l.viewed = self.store.get(&format!("viewedLevel_{}", l.id)).is_some();
    }

    /// `load`: Search, Featured, Saved, paging and the quick searches.
    pub(crate) fn online_load(&mut self) {
        let kind = core::wd_on_kind();
        let page = core::wd_on_page();
        let tx = self.net.tx.clone();
        if kind == 2 { // saved levels
            let list = self.saved_levels();
            let n = list.len() as f64;
            let _ = tx.send(NetMsg::Results { levels: list, total: n, offset: 0.0, count: n.max(1.0) });
            return;
        }
        let q = { let n = core::wd_on_query(); let p = core::wd_result_ptr(); unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).trim().to_string() } };
        let diff = { let n = core::wd_on_diff(); let p = core::wd_result_ptr(); unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned() } };
        let demon = core::wd_on_demon();
        let names: Vec<String> = self.levels.iter().map(|l| l.name.clone()).collect();
        std::thread::spawn(move || {
            let official = |n: i64| names.get(n as usize).cloned().unwrap_or("Unknown".into());
            if kind == 0 && !q.is_empty() && q.chars().all(|c| c.is_ascii_digit()) {
                match post("downloadGJLevel22.php", &format!("levelID={q}&secret={SECRET}")) {
                    Ok(raw) if raw.trim() != "-1" && raw.contains(':') => {
                        let m = kv(raw.split('#').next().unwrap_or(""), ":"); let c = m.get("35").map_or("", |v| v.trim()); let custom = !c.is_empty() && c != "0";
                        let l = to_level(&m, &author_of(&raw), &if custom { format!("Song #{c}") } else { official(num(&m, "12")) });
                        let _ = tx.send(NetMsg::Results { levels: vec![l], total: -1.0, offset: 0.0, count: 1.0 });
                    }
                    Ok(_) => { let _ = tx.send(NetMsg::Results { levels: vec![], total: 0.0, offset: 0.0, count: 1.0 }); }
                    Err(e) => { let _ = tx.send(NetMsg::Error(e)); }
                }
                return;
            }
            let mut body = format!("secret={SECRET}&page={page}");
            if kind == 1 { body.push_str("&type=6"); } else if kind >= 10 { body.push_str(&format!("&type={}", kind - 10)); } else if !q.is_empty() { body.push_str(&format!("&type=0&str={}", enc(&q))); } else { body.push_str("&type=2"); }
            if !diff.is_empty() { body.push_str(&format!("&diff={}", enc(&diff))); if demon != 0 { body.push_str(&format!("&demonFilter={demon}")); } }
            match post("getGJLevels21.php", &body) {
                Ok(raw) if raw.trim() != "-1" && !raw.is_empty() => { let (levels, total, offset, count) = parse_list(&raw, &official); let _ = tx.send(NetMsg::Results { levels, total, offset, count }); }
                Ok(_) => { let _ = tx.send(NetMsg::Results { levels: vec![], total: 0.0, offset: 0.0, count: 10.0 }); }
                Err(e) => { let _ = tx.send(NetMsg::Error(e)); }
            }
        });
    }

    pub(crate) fn online_daily(&mut self, k: usize) {
        let tx = self.net.tx.clone();
        let names: Vec<String> = self.levels.iter().map(|l| l.name.clone()).collect();
        std::thread::spawn(move || {
            let official = |n: i64| names.get(n as usize).cloned().unwrap_or("Unknown".into());
            let (dl, list_type) = [(-1, 21), (-2, 22), (-3, 23)][k];
            let mut lvl: Option<OLevel> = None;
            if let Ok(raw) = post("getGJLevels21.php", &format!("secret={SECRET}&page=0&type={list_type}")) { if raw.trim() != "-1" { lvl = parse_list(&raw, &official).0.into_iter().next(); } }
            if lvl.is_none() {
                match post("downloadGJLevel22.php", &format!("levelID={dl}&secret={SECRET}")) {
                    Ok(raw) if raw.trim() != "-1" && raw.contains(':') => { let m = kv(raw.split('#').next().unwrap_or(""), ":"); lvl = Some(to_level(&m, &author_of(&raw), &official(num(&m, "12")))); }
                    Ok(_) => { let _ = tx.send(NetMsg::Error("no level".into())); return; }
                    Err(e) => { let _ = tx.send(NetMsg::Error(e)); return; }
                }
            }
            let mut l = lvl.unwrap();
            if let Ok(t) = post("getGJDailyLevel.php", &format!("type={k}&secret={SECRET}")) { if let Some(n) = t.split('|').next().and_then(|v| v.parse::<i64>().ok()) { if n > 0 { l.daily_number = Some(n); } } }
            let _ = tx.send(NetMsg::Special(l));
        });
    }

    fn song_info(&mut self, l: &OLevel) {
        if l.custom_song.is_empty() || l.custom_song == "0" { return; }
        let (tx, id) = (self.net.tx.clone(), l.custom_song.clone());
        std::thread::spawn(move || {
            if let Ok(t) = post("getGJSongInfo.php", &format!("songID={id}&secret={SECRET}")) {
                if t.trim() == "-1" { return; }
                let m = kv(&t, "~|~");
                let _ = tx.send(NetMsg::Song(m.get("2").map_or(String::new(), |s| s.trim().to_string()), m.get("4").map_or("Unknown".to_string(), |s| s.trim().to_string())));
            }
        });
    }

    pub(crate) fn online_view(&mut self, i: usize) {
        if let Some(mut l) = self.net.list.get(i).cloned() { self.store.set(&format!("viewedLevel_{}", l.id), "1"); self.save_level(&l, None); self.song_info(&l); l.viewed = true; }
    }

    fn saved_path_key() -> &'static str { "gd_saved_online_levels" }

    fn saved_raw(&self) -> Vec<core::jsonread::Json> {
        self.store.get(Self::saved_path_key()).and_then(|s| core::jsonread::parse(s).ok()).and_then(|j| j.arr().map(|a| a.to_vec())).unwrap_or_default()
    }

    fn saved_levels(&self) -> Vec<OLevel> {
        self.saved_raw().iter().map(|s| {
            let g = |k: &str| s.get_str(k).unwrap_or("").to_string();
            let n = |k: &str| s.get_num(k).unwrap_or(0.0) as i64;
            let mut l = OLevel { id: g("id").trim_start_matches("online_").to_string(), name: g("name"), author: g("author"), song_name: g("songName"), custom_song: g("customSongID"), difficulty: n("difficulty"), stars: n("stars"), downloads: n("downloads"), likes: n("likes"), length: n("length"), ..Default::default() };
            self.fill(&mut l); l
        }).collect()
    }

    /// `_saveOnlineLevelToSavedList`
    fn save_level(&mut self, l: &OLevel, extra: Option<(&str, i64, f64)>) {
        let mut items: Vec<String> = Vec::new();
        let mut found = false;
        for s in self.saved_raw() {
            let id = s.get_str("id").unwrap_or("").trim_start_matches("online_").to_string();
            let mut obj = Self::json_of(&s);
            if id == l.id { found = true; if let Some((text, off_song, off)) = extra { if s.get_str("levelString").is_none() { obj = format!("{},\"levelString\":{},\"officialSong\":{},\"songOffset\":{}}}", &obj[..obj.len() - 1], json_s(text), off_song, off); } } }
            items.push(obj);
        }
        if !found {
            let mut o = format!("{{\"id\":{},\"name\":{},\"author\":{},\"customSongID\":{},\"songName\":{},\"difficulty\":{},\"downloads\":{},\"likes\":{},\"stars\":{},\"coins\":{},\"length\":{}", json_s(&l.id), json_s(&l.name), json_s(&l.author), json_s(&l.custom_song), json_s(&l.song_name), l.difficulty, l.downloads, l.likes, l.stars, l.coins, l.length);
            if let Some((text, off_song, off)) = extra { o.push_str(&format!(",\"levelString\":{},\"officialSong\":{},\"songOffset\":{}", json_s(text), off_song, off)); }
            o.push('}');
            items.insert(0, o);
        }
        self.store.set(Self::saved_path_key(), &format!("[{}]", items.join(",")));
    }

    fn json_of(j: &core::jsonread::Json) -> String {
        use core::jsonread::Json;
        match j {
            Json::Str(s) => json_s(s), Json::Num(n) => format!("{n}"), Json::Bool(b) => b.to_string(), Json::Null => "null".into(),
            Json::Arr(a) => format!("[{}]", a.iter().map(Self::json_of).collect::<Vec<_>>().join(",")),
            Json::Obj(o) => format!("{{{}}}", o.iter().map(|(k, v)| format!("{}:{}", json_s(k), Self::json_of(v))).collect::<Vec<_>>().join(",")),
        }
    }

    pub(crate) fn online_play(&mut self, i: usize, special: bool) {
        if self.net.busy { return; }
        let l = if special { self.net.special.clone() } else { self.net.list.get(i).cloned() };
        let Some(l) = l else { return };
        self.net.busy = true;
        self.audio.stop_music(); self.audio.effect("playSound_01", 1.0);
        self.fade_to(1.0, 150.0, crate::host::FadeCmd::None);
        core::wd_on_play_busy(1);
        let saved = self.saved_raw().into_iter().find(|s| s.get_str("id").unwrap_or("").trim_start_matches("online_") == l.id);
        let cached = saved.and_then(|s| s.get_str("levelString").map(|t| (t.to_string(), s.get_num("officialSong").unwrap_or(0.0) as i64, s.get_str("customSongID").unwrap_or("").to_string(), s.get_num("songOffset").unwrap_or(0.0))));
        let tx = self.net.tx.clone();
        let root = self.game.assets.root.clone();
        let songs: Vec<(String, Option<String>)> = self.levels.iter().map(|l| (l.name.clone(), l.song_file.clone())).collect();
        let cache_dir = self.store.dir().join("webdashers_songs");
        std::thread::spawn(move || {
            let (text, official, custom, offset) = match cached {
                Some(c) => c,
                None => match download(&l.id) { Ok(m) => (m["4"].clone(), num(&m, "12"), m.get("35").map_or(String::new(), |s| s.trim().to_string()), m.get("45").and_then(|v| v.parse().ok()).unwrap_or(0.0)), Err(e) => { let _ = tx.send(NetMsg::PlayFailed(e)); return; } },
            };
            let mut song = None;
            if !custom.is_empty() && custom != "0" {
                let f = cache_dir.join(&custom);
                song = std::fs::read(&f).ok().or_else(|| { let b = song_bytes(&custom); if let Some(b) = &b { let _ = std::fs::create_dir_all(&cache_dir); let _ = std::fs::write(&f, b); } b });
            } else if let Some((name, file)) = songs.get(official as usize) { song = std::fs::read(root.join("music").join(format!("{}.mp3", file.clone().unwrap_or_else(|| name.replace(' ', ""))))).ok(); }
            let _ = tx.send(NetMsg::Play { id: l.id.clone(), name: l.name.clone(), text, song, offset, level: l, official_song: official });
        });
    }

    /// Answers of the request threads.
    pub(crate) fn net_poll(&mut self) {
        while let Ok(m) = self.net.rx.try_recv() {
            match m {
                NetMsg::Results { mut levels, total, offset, count } => {
                    for l in levels.iter_mut() { self.fill(l); }
                    let json = format!("[{}]", levels.iter().map(|l| l.json()).collect::<Vec<_>>().join(","));
                    self.net.list = levels;
                    unsafe { core::wd_on_results(json.as_ptr(), json.len(), total, offset, count); }
                }
                NetMsg::Error(e) => { eprintln!("online: {e}"); let m = "Could not reach the servers"; unsafe { core::wd_on_error(m.as_ptr(), m.len()); } }
                NetMsg::Special(mut l) => { self.fill(&mut l); let json = l.json(); self.net.list = vec![l.clone()]; self.net.special = Some(l); unsafe { core::wd_on_special(json.as_ptr(), json.len()); } }
                NetMsg::Song(n, a) => unsafe { core::wd_on_song(n.as_ptr(), n.len(), a.as_ptr(), a.len()); },
                NetMsg::PlayFailed(e) => { eprintln!("online: {e}"); self.net.busy = false; self.fade_to(0.0, 150.0, crate::host::FadeCmd::None); core::wd_on_play_busy(0); let m = "Could not reach the servers"; unsafe { core::wd_on_error(m.as_ptr(), m.len()); } }
                NetMsg::Play { id, name, text, song, offset, level, official_song } => {
                    self.save_level(&level, Some((&text, official_song, offset)));
                    self.net.busy = false;
                    self.fade_a = 0.0; self.fade = None;
                    core::wd_on_close();
                    self.net.active = true; self.net.name = name;
                    self.current_song = song.map(std::sync::Arc::new);
                    self.online_offset = offset;
                    if let Err(e) = self.play_text(&id, &text) { eprintln!("{e}"); }
                }
                NetMsg::Copied(l, text) => { self.add_copied_level(&l, &text); }
                NetMsg::SongData { id: _, bytes, purpose } => {
                    match purpose {
                        0 => { if let Some(b) = bytes { self.ed_custom_song = Some(std::sync::Arc::new(b)); if self.editor_on && core::wd_ed_pt(1) != 0.0 && core::wd_ed_pt(2) == 0.0 { let off = self.game.music_offset + core::wd_ed_pt(3); self.audio.start_song_bytes(self.ed_custom_song.clone().unwrap(), off); } } }
                        _ => { if let Some((id, txt)) = self.pending_created.take() { self.current_song = bytes.map(std::sync::Arc::new); self.finish_play_created(&id, &txt); } }
                    }
                }
            }
        }
    }

    /// A custom song by its id: the saved copy, else downloaded on a thread (`purpose`: 0 editor playtest, 1 created level).
    pub(crate) fn request_custom_song(&mut self, id: String, purpose: u8) {
        let tx = self.net.tx.clone();
        let cache_dir = self.store.dir().join("webdashers_songs");
        std::thread::spawn(move || {
            let f = cache_dir.join(&id);
            let bytes = std::fs::read(&f).ok().or_else(|| { let b = song_bytes(&id); if let Some(b) = &b { let _ = std::fs::create_dir_all(&cache_dir); let _ = std::fs::write(&f, b); } b });
            let _ = tx.send(NetMsg::SongData { id, bytes, purpose });
        });
    }

    pub(crate) fn cached_song(&self, id: &str) -> Option<Vec<u8>> { std::fs::read(self.store.dir().join("webdashers_songs").join(id)).ok() }

    /// `_duplicateOnlineLevelToEditor`
    pub(crate) fn online_copy(&mut self, i: usize) {
        let Some(l) = self.net.list.get(i).cloned() else { return };
        let tx = self.net.tx.clone();
        std::thread::spawn(move || match download(&l.id) { Ok(m) => { let _ = tx.send(NetMsg::Copied(l, m["4"].clone())); } Err(e) => { let _ = tx.send(NetMsg::Error(e)); } });
    }

    pub(crate) fn net_reopen(&mut self) {
        let i = core::wd_on_reopen();
        if i >= 0 { self.online_view(i as usize); }
    }
}

impl Host {
    /// `Pop("OnX", n)` events of the online screens; false when the event is not one of them.
    pub(crate) fn online_event(&mut self, ev: &str) -> bool {
        let Some(rest) = ev.strip_prefix("Pop(\"") else { return false };
        let Some(q) = rest.find('"') else { return false };
        let name = &rest[..q];
        let n: usize = rest[q + 1..].trim_start_matches(',').trim().trim_end_matches(')').trim().parse().unwrap_or(0);
        match name {
            "OnSearch" | "OnFeatured" | "OnSaved" | "OnPage" | "OnPrev" | "OnNext" | "OnRefresh" | "OnQuick" => self.online_load(),
            "OnDaily" => self.online_daily(n.min(2)),
            "OnSpecialPlay" => { if let Some(l) = self.net.special.clone() { self.net.list = vec![l.clone()]; self.store.set(&format!("viewedLevel_{}", l.id), "1"); self.save_level(&l, None); self.song_info(&l); } }
            "OnDeleteSaved" => self.store.set("gd_saved_online_levels", "[]"),
            "OnView" => self.online_view(n),
            "OnPlay" => self.online_play(n, false),
            "OnCopy" => self.online_copy(n),
            _ => return false,
        }
        true
    }
}
