//! Keyboard shortcuts of the menus: Escape closes the top-most popup or goes one screen back, Enter presses the main button of the
//! screen, the arrow keys (or A / D) page through the level select. The host asks `wd_ui_menu_key` first and keeps the key when it was used.
use crate::ui::*;

impl Ui {
    fn has_tag(&self, tag: &str) -> bool { self.nodes.iter().flatten().any(|n| n.tag.starts_with(tag)) }

    /// a text field has the keyboard: keys belong to it
    fn typing(&self) -> bool {
        self.opt_focus.is_some() || self.em.input != 0 || self.on.input || self.pb.kb_focus || self.ed.fields.focus.is_some() || self.ed.fields.text_on
            || self.ed.trig.focus.is_some() || self.ed.ct_focus || self.ed.ls.focused || self.ed.cp.focus.is_some() || self.gar.open
    }

    /// true when the key did something
    pub fn menu_key(&mut self, code: &str) -> bool {
        if self.typing() { return false; }
        if self.ed.on { return self.menu_key_editor(code); }
        match code {
            "Escape" => self.menu_escape(),
            "Enter" | "NumpadEnter" => self.menu_enter(),
            "ArrowLeft" | "KeyA" if self.level_select_open && !self.has_popup() => { self.emit(Action::PrevLevel); true }
            "ArrowRight" | "KeyD" if self.level_select_open && !self.has_popup() => { self.emit(Action::NextLevel); true }
            _ => false,
        }
    }

    /// a popup is open over the screen
    fn has_popup(&self) -> bool {
        ["pa_", "pb_", "opt_", "set_", "info_", "on_"].iter().any(|t| self.has_tag(t)) && !self.has_tag("on_s") && !self.has_tag("on_l")
    }

    fn menu_escape(&mut self) -> bool {
        // popups first, the top-most kind first
        let closers: [(&str, Action); 12] = [
            ("info_", Action::InfoClose), ("pa_htp", Action::Pop("HowToPlay", 0)), ("pa_cr", Action::Pop("Credits", 0)), ("pa_news", Action::Pop("News", 0)),
            ("pa_ng", Action::Pop("Newgrounds", 0)), ("pa_songs", Action::Pop("Songs", 0)), ("pa_req", Action::Pop("Req", 0)), ("pa_help", Action::Pop("Help", 0)),
            ("pb_sp", Action::Pop("sp_close", 0)), ("pb_st", Action::Pop("st_close", 0)), ("opt_", Action::OptClose), ("set_", Action::SettingsClose),
        ];
        for (tag, a) in closers {
            if self.has_tag(tag) { self.emit(a); return true; }
        }
        if self.level_select_open { self.emit(Action::CloseLevelSelect); return true; }
        // the creator screens, one step back each
        if self.has_tag("lv_") { self.kill_tag("lv_"); self.em.view = None; self.em.input = 0; self.em_open_my_levels(); return true; }
        if self.has_tag("my_") { self.emit(Action::Pop("EmMyBack", 0)); return true; }
        if self.has_tag("cm_") { self.emit(Action::Pop("EmBack", 0)); return true; }
        false
    }

    fn menu_enter(&mut self) -> bool {
        if self.has_popup() { return false; }
        if self.level_select_open { self.emit(Action::PlayLevel); return true; }
        if self.end_open { self.emit(Action::PauseReplay); return true; }
        if self.paused { self.emit(Action::PauseResume); return true; }
        if self.menu_active && !self.has_tag("lv_") && !self.has_tag("my_") && !self.has_tag("cm_") { self.emit(Action::OpenLevelSelect); return true; }
        false
    }

    /// Escape in the editor closes what is open on top, innermost first; the tabs and the movement keys come from the keybinds
    fn menu_key_editor(&mut self, code: &str) -> bool {
        if code != "Escape" { return false; }
        if self.ed.cp.open { if self.ed.cp.focus.is_some() { self.ed.cp.focus = None; } else { self.ed_cp_close(); } return true; }
        if self.has_tag("ed_ho") { self.ed_ho_close(); return true; }
        if self.has_tag("ed_sps") { self.kill_tag("ed_sps"); return true; }
        if self.has_tag("ed_so") { self.kill_tag("ed_so"); self.ed.so_open = false; return true; }
        if self.has_tag("ed_op") || self.has_tag("ed_gp") { self.ed_op_close(); return true; }
        if self.ed.ls_open { self.ed_ls_close(); return true; }
        if self.ed.pm { self.ed_pm_close(); return true; }
        if self.ed.pt { self.ed_pt_stop(false); return true; }
        self.ed_pm_open();
        true
    }

    /// the editor actions of the keybinds: 0 / 1 / 2 build, edit, delete tab; 3..6 move the selection up, left, down, right (`big` = 60, else 1)
    pub fn ed_key_action(&mut self, which: u32, big: bool) {
        if !self.ed.on || self.ed.pt || self.ed.pm || self.typing() { return; }
        let a = if big { 60.0 } else { 1.0 };
        match which {
            0..=2 => { self.ed_action("EdTab", which as usize); }
            7 => { self.ed_action("EdZoom", 1); }
            8 => { self.ed_action("EdZoom", 0); }
            3 => self.ed_move(0.0, -a), 4 => self.ed_move(-a, 0.0), 5 => self.ed_move(0.0, a), 6 => self.ed_move(a, 0.0),
            _ => {}
        }
    }
}
