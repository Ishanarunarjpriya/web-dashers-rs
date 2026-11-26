//! Sound: effects (`assets/sfx`), the menu loop and the level's song (`assets/music`), through the default output device.
//! Without an output device (a build machine) every call does nothing.
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::Duration;

pub struct Audio {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    root: PathBuf,
    cache: HashMap<String, std::sync::Arc<Vec<u8>>>,
    music: Option<Sink>,
    menu: bool,
    pub user_music: f64,
    pub user_sfx: f64,
    effects: Vec<Sink>,
}

impl Audio {
    pub fn new(root: PathBuf) -> Audio {
        let (stream, handle) = match OutputStream::try_default() { Ok((s, h)) => (Some(s), Some(h)), Err(_) => (None, None) };
        Audio { _stream: stream, handle, root, cache: HashMap::new(), music: None, menu: false, user_music: 1.0, user_sfx: 1.0, effects: Vec::new() }
    }

    fn bytes(&mut self, rel: &str) -> Option<std::sync::Arc<Vec<u8>>> {
        if let Some(b) = self.cache.get(rel) { return Some(b.clone()); }
        let data = std::fs::read(self.root.join(rel)).ok()?;
        let a = std::sync::Arc::new(data);
        if rel.starts_with("sfx/") { self.cache.insert(rel.to_string(), a.clone()); }
        Some(a)
    }

    /// `AudioManager.playEffect(name, {volume})`: the base volume times the user's effects volume.
    pub fn effect(&mut self, name: &str, volume: f64) {
        let Some(h) = self.handle.clone() else { return };
        let Some(b) = self.bytes(&format!("sfx/{name}.ogg")) else { return };
        let Ok(src) = Decoder::new(Cursor::new((*b).clone())) else { return };
        let Ok(sink) = Sink::try_new(&h) else { return };
        sink.set_volume((volume * self.user_sfx) as f32);
        sink.append(src);
        self.effects.retain(|s| !s.empty());
        self.effects.push(sink);
    }

    pub fn stop_music(&mut self) { if let Some(m) = self.music.take() { m.stop(); } self.menu = false; }

    pub fn start_menu_music(&mut self) {
        if self.menu && self.music.is_some() { return; }
        self.stop_music();
        let Some(h) = self.handle.clone() else { return };
        let Some(b) = self.bytes("music/menuLoop.mp3") else { return };
        let Ok(src) = Decoder::new(Cursor::new((*b).clone())) else { return };
        let Ok(sink) = Sink::try_new(&h) else { return };
        sink.set_volume((self.user_music * 0.8) as f32);
        sink.append(src.repeat_infinite());
        self.music = Some(sink);
        self.menu = true;
    }

    /// The level's song from `offset` seconds.
    pub fn start_song(&mut self, file: &str, offset: f64) {
        self.stop_music();
        let Some(h) = self.handle.clone() else { return };
        let Some(b) = self.bytes(&format!("music/{file}.mp3")) else { return };
        let Ok(src) = Decoder::new(Cursor::new((*b).clone())) else { return };
        let Ok(sink) = Sink::try_new(&h) else { return };
        sink.set_volume((self.user_music * 0.8) as f32);
        sink.append(src.skip_duration(Duration::from_secs_f64(offset.max(0.0))));
        self.music = Some(sink);
    }

    /// A song held in memory (a downloaded custom song).
    pub fn start_song_bytes(&mut self, bytes: std::sync::Arc<Vec<u8>>, offset: f64) {
        self.stop_music();
        let Some(h) = self.handle.clone() else { return };
        let Ok(src) = Decoder::new(Cursor::new((*bytes).clone())) else { return };
        let Ok(sink) = Sink::try_new(&h) else { return };
        sink.set_volume((self.user_music * 0.8) as f32);
        sink.append(src.skip_duration(Duration::from_secs_f64(offset.max(0.0))));
        self.music = Some(sink);
    }

    pub fn set_music_volume(&mut self, v: f64) { self.user_music = v; if let Some(m) = &self.music { m.set_volume((v * 0.8) as f32); } }
    pub fn pause(&mut self) { if let Some(m) = &self.music { m.pause(); } }
    pub fn resume(&mut self) { if let Some(m) = &self.music { m.play(); } }
    pub fn playing_menu(&self) -> bool { self.menu && self.music.is_some() }
}
