//! Text / number fields of the editor popups (focus, typing). Filled in with the object popups.
use crate::ui::*;

#[derive(Default)]
pub struct EdFields { pub focus: Option<usize> }

impl Ui {
    pub fn ed_fields_key(&mut self, _code: u32) -> bool { false }
    pub fn ed_cp_open(&mut self, _channel: i64) {}
}
