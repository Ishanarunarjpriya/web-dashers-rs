//! `localStorage` of the web host: a small key/value file (`webdashers_save.json`) next to the program.
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct Store { path: PathBuf, map: BTreeMap<String, String>, dirty: bool }

impl Store {
    pub fn open() -> Store {
        let dir = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
        let path = if std::env::var("WD_SAVE").is_ok() { PathBuf::from(std::env::var("WD_SAVE").unwrap()) } else { dir.join("webdashers_save.json") };
        let mut map = BTreeMap::new();
        if let Ok(src) = std::fs::read_to_string(&path) {
            if let Ok(j) = wd_core::jsonread::parse(&src) {
                if let Some(e) = j.entries() { for (k, v) in e { if let Some(s) = v.str() { map.insert(k.clone(), s.to_string()); } } }
            }
        }
        Store { path, map, dirty: false }
    }
    pub fn get(&self, k: &str) -> Option<&str> { self.map.get(k).map(|s| s.as_str()) }
    pub fn get_f(&self, k: &str, d: f64) -> f64 { self.get(k).and_then(|v| v.parse().ok()).unwrap_or(d) }
    pub fn set(&mut self, k: &str, v: &str) { if self.map.get(k).map(|s| s.as_str()) != Some(v) { self.map.insert(k.to_string(), v.to_string()); self.dirty = true; } }
    pub fn bump(&mut self, k: &str) { let n = self.get_f(k, 0.0) as i64 + 1; self.set(k, &n.to_string()); }
    pub fn flush(&mut self) {
        if !self.dirty { return; }
        let body: Vec<String> = self.map.iter().map(|(k, v)| format!("{}:{}", crate::host::Host::json_str(k), crate::host::Host::json_str(v))).collect();
        let _ = std::fs::write(&self.path, format!("{{{}}}", body.join(",")));
        self.dirty = false;
    }
}
