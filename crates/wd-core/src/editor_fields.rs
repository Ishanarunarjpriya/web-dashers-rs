//! Text / number fields of the editor popups (focus, typing). Filled in with the object popups.
use crate::ui::*;

#[derive(Default)]
pub struct EdFields { pub focus: Option<usize> }

impl Ui {
    pub fn ed_fields_key(&mut self, code: u32) -> bool { if self.ed.cp.open { return self.ed_cp_key(code); } false }
}
