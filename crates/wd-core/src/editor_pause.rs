//! The editor pause menu (level-editor.js `_initEditorPauseMenu`, `_showEditorPauseMenu`). The buttons that leave the editor or write to
//! storage raise `Pop("EdPm", n)` for the host: 1 Save and Play, 2 Save and Exit, 3 Save, 4 Exit (0, Resume, is handled here).
use crate::ui::*;

const PD: f64 = 1900.0;
const BUTTONS: [&str; 5] = ["Resume", "Save and Play", "Save and Exit", "Save", "Exit"];

impl Ui {
    pub fn ed_pm_open(&mut self) {
        if self.ed.pm { return; }
        self.ed.pm = true; self.ed.paused = true;
        let root = self.pa_popup("ed_pm", PD, 0.6, false);
        let sw = self.screen_width;
        for (i, text) in BUTTONS.iter().enumerate() {
            let y = 320.0 - 150.0 + i as f64 * 70.0;
            // a 450 x 65 nine-slice at scale 0.75, its label at bitmap size 40, scale 0.8
            self.pa_btn(root, "GJ_button01", sw / 2.0, y, 450.0 * 0.75, 65.0 * 0.75, text, "goldFont", 40.0 * 0.8, 0.0, -2.0, Action::Pop("EdPm", i), PD + 0.1, "ed_pm");
        }
        // the "Show Glow" toggle
        let (x, y) = (200.0, 640.0 - 60.0);
        let on = self.ed.glow;
        let check = self.pa_img(root, "GJ_GameSheet03", if on { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" }, x - 120.0, y, 0.8, PD + 0.1, "ed_pm");
        self.pa_txt(root, "bigFont", 25.0, "Show Glow", x - 70.0, y, 0.0, 0.5, 1.0, PD + 0.1, "ed_pm");
        self.make_bouncy(check, 0.8, Cond::Always, Action::Pop("EdPmGlow", 0));
        self.ed.pm_glow_check = check;
    }

    pub fn ed_pm_close(&mut self) {
        if !self.ed.pm { return; }
        self.kill_tag("ed_pm");
        self.ed.pm = false; self.ed.paused = false;
    }

    /// `Pop("EdPm", n)` and `Pop("EdPmGlow", 0)`; returns true when the action was an editor pause menu one
    pub fn ed_pm_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdPause" => { self.ed_pm_open(); true }
            "EdPm" => {
                if arg != 3 { self.ed_pm_close(); }
                if matches!(arg, 1 | 2 | 4) { self.ed_pt_stop(false); }
                true
            }
            "EdPmGlow" => {
                self.ed.glow = !self.ed.glow;
                let f = if self.ed.glow { "GJ_checkOn_001.png" } else { "GJ_checkOff_001.png" };
                let c = self.ed.pm_glow_check;
                if let Some(n) = self.nodes.get_mut(c).and_then(|n| n.as_mut()) { n.frame = f.into(); }
                true
            }
            _ => false,
        }
    }
}
