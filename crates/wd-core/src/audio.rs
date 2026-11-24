//! The music level meter (audio.js `AudioManager.update` / `getMeteringValue`): the host reads the analyser's loudest sample and
//! this smooths it into the value the audio-reactive sprites follow.

use crate::jsmath as m;

#[derive(Clone, Debug)]
pub struct Meter {
    pub value: f64,
    last_audio: f64,
    last_peak: f64,
    silence: u32,
}

impl Default for Meter { fn default() -> Self { Meter::new() } }

impl Meter {
    /// `reset()`
    pub fn new() -> Self { Meter { value: 0.1, last_audio: 0.1, last_peak: 0.0, silence: 0 } }

    /// `update(timeSeconds)` after the analyser produced `peak` (the largest absolute sample); `volume` is `_effectiveVolume()`.
    pub fn update(&mut self, time_seconds: f64, peak: f64, volume: f64) {
        let mut biggest = peak;
        if volume > 0.0 { biggest /= volume; }
        self.value = 0.1 + biggest;
        let minutes = time_seconds * 60.0;
        if self.silence < 3 || self.value < self.last_audio * 1.1 || (self.value < self.last_peak * 0.95 && self.last_audio > self.last_peak * 0.2) {
            self.value = self.last_audio * m::pow(0.92, minutes);
        } else {
            self.silence = 0;
            self.last_peak = self.value;
            self.value *= m::pow(1.46, minutes);
        }
        if self.value <= 0.1 { self.last_peak = 0.0; }
        self.last_audio = self.value;
        self.silence += 1;
    }
}
