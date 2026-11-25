//! Numeric input fields of the editor popups: a dark box with the value and an arrow on each side (the `makeNumericInput` of the
//! group popup and the trigger popups). Typing goes through `ed_fields_key`; every change reaches `ed_num_commit(purpose, value)`.
use crate::ui::*;

pub struct NumInput { pub purpose: u16, pub bg: usize, pub text: usize, pub value: String, pub before: String, pub min: i64, pub max: i64, pub neg: bool, pub tag: &'static str, pub x: f64, pub y: f64 }

#[derive(Default)]
pub struct EdFields { pub inputs: Vec<NumInput>, pub focus: Option<usize>, pub text_node: usize, pub text_on: bool }

const IN_W: f64 = 92.0;
const IN_H: f64 = 58.0;

impl NumInput {
    fn normalize(&self, raw: &str) -> i64 {
        let n = raw.parse::<i64>().unwrap_or(0);
        n.max(self.min).min(self.max)
    }
}

impl Ui {
    /// `makeNumericInput`: label above, the box, and arrows (`arrow` frame of GJ_GameSheet03) on both sides
    pub(crate) fn ed_num_input(&mut self, root: usize, tag: &'static str, depth: f64, purpose: u16, x: f64, y: f64, label: &str, arrow: &str, initial: i64, min: i64, max: i64, neg: bool) -> usize {
        self.pa_txt(root, "goldFont", 28.0, label, x, y - 52.0, 0.5, 0.5, 1.0, depth, tag);
        let bg = self.shape(Kind::RoundRect, x - IN_W / 2.0, y - IN_H / 2.0, IN_W, IN_H, 0x000000, 0.43, depth, tag);
        { let n = self.node(bg); n.radius = 10.0; n.parent = Some(root); }
        let text = self.pa_txt(root, "bigFont", 32.0, "", x, y, 0.5, 0.5, 1.0, depth + 0.01, tag);
        let idx = self.ed.fields.inputs.len();
        let hit = self.shape(Kind::Rect, x, y, IN_W, IN_H + 4.0, 0xffffff, 0.001, depth + 0.02, tag);
        { let n = self.node(hit); n.ox = 0.5; n.oy = 0.5; n.parent = Some(root); n.interactive = true; n.action = Action::Pop("EdNum", idx); n.width = IN_W; n.height = IN_H + 4.0; }
        let l = self.pa_img(root, "GJ_GameSheet03", arrow, x - 82.0, y + 1.0, 0.7, depth, tag);
        self.make_bouncy(l, 0.7, Cond::Always, Action::Pop("EdNumStep", idx * 2));
        let r = self.pa_img(root, "GJ_GameSheet03", arrow, x + 82.0, y + 1.0, 0.7, depth, tag);
        self.node(r).flip_x = true;
        self.make_bouncy(r, 0.7, Cond::Always, Action::Pop("EdNumStep", idx * 2 + 1));
        let v = initial.to_string();
        self.ed.fields.inputs.push(NumInput { purpose, bg, text, value: v.clone(), before: v, min, max, neg, tag, x, y });
        self.ed_num_render(idx);
        idx
    }

    fn ed_num_render(&mut self, i: usize) {
        let Some(f) = self.ed.fields.inputs.get(i) else { return };
        let focused = self.ed.fields.focus == Some(i);
        let shown = if focused { f.value.clone() } else { f.normalize(&f.value).to_string() };
        let t = format!("{}{}", shown, if focused { "|" } else { "" });
        let text = f.text;
        if let Some(n) = self.nodes.get_mut(text).and_then(|n| n.as_mut()) { n.text = t; }
    }

    pub(crate) fn ed_num_blur(&mut self, commit: bool) {
        let Some(i) = self.ed.fields.focus else { return };
        if commit {
            let (p, v) = { let f = &self.ed.fields.inputs[i]; (f.purpose, f.normalize(&f.value)) };
            self.ed.fields.inputs[i].value = v.to_string();
            self.ed_num_commit(p, v);
        }
        self.ed.fields.focus = None;
        self.ed_num_render(i);
    }

    /// sets a value from outside (the buttons next to an input); true commit as well
    pub(crate) fn ed_num_set(&mut self, i: usize, v: i64, commit: bool) {
        let Some(f) = self.ed.fields.inputs.get_mut(i) else { return };
        let v = f.normalize(&v.to_string());
        f.value = v.to_string();
        let p = f.purpose;
        if commit { self.ed_num_commit(p, v); }
        self.ed_num_render(i);
    }

    pub(crate) fn ed_num_get(&self, i: usize) -> i64 { self.ed.fields.inputs.get(i).map_or(0, |f| f.normalize(&f.value)) }

    pub fn ed_fields_key(&mut self, code: u32) -> bool {
        if self.ed.cp.open { return self.ed_cp_key(code); }
        if self.ed.fields.text_on { return self.ed_text_key(code); }
        let Some(i) = self.ed.fields.focus else { return false };
        let c = char::from_u32(code).unwrap_or('\0');
        match code {
            13 => { self.ed_num_blur(true); }
            8 => { self.ed.fields.inputs[i].value.pop(); self.ed_num_render(i); }
            _ => {
                let f = &mut self.ed.fields.inputs[i];
                if f.neg && c == '-' && f.value.is_empty() { f.value.push('-'); }
                else if c.is_ascii_digit() && f.value.len() < 5 { f.value.push(c); }
                self.ed_num_render(i);
            }
        }
        true
    }

    pub fn ed_fields_action(&mut self, name: &str, arg: usize) -> bool {
        match name {
            "EdNum" => {
                self.ed_num_blur(true);
                if let Some(f) = self.ed.fields.inputs.get_mut(arg) { f.before = f.value.clone(); }
                self.ed.fields.focus = Some(arg);
                self.ed_num_render(arg);
                true
            }
            "EdNumBlur" => { self.ed_num_blur(true); true }
            "EdNumStep" => {
                self.ed_num_blur(true);
                let (i, dir) = (arg / 2, if arg % 2 == 0 { -1 } else { 1 });
                let cur = self.ed_num_get(i);
                self.ed_num_set(i, cur + dir, true);
                true
            }
            _ => false,
        }
    }
}
