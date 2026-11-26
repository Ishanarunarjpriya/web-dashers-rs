//! `assets/scripts/game/allLevels.js`: `window.allLevels = [[internal, name, "level_N", [maker, song], songFile?], ...]`.
pub struct Level { pub internal: String, pub name: String, pub id: String, pub song_file: Option<String> }

pub fn load(root: &std::path::Path) -> Vec<Level> {
    let Ok(src) = std::fs::read_to_string(root.join("scripts/game/allLevels.js")) else { return Vec::new() };
    // tokens: '[' ']' and string literals; // comments are skipped
    let mut stack: Vec<Vec<Item>> = Vec::new();
    let mut top: Vec<Item> = Vec::new();
    #[derive(Clone)] enum Item { S(String), A(Vec<Item>) }
    let cs: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        match cs[i] {
            '/' if cs.get(i + 1) == Some(&'/') => { while i < cs.len() && cs[i] != '\n' { i += 1; } continue; }
            '[' => { stack.push(std::mem::take(&mut top)); }
            ']' => { let done = std::mem::take(&mut top); top = stack.pop().unwrap_or_default(); top.push(Item::A(done)); }
            '"' | '\'' => {
                let q = cs[i]; i += 1; let mut s = String::new();
                while i < cs.len() && cs[i] != q { if cs[i] == '\\' { i += 1; } if i < cs.len() { s.push(cs[i]); } i += 1; }
                top.push(Item::S(s));
            }
            _ => {}
        }
        i += 1;
    }
    let all = match top.into_iter().next() { Some(Item::A(a)) => a, _ => return Vec::new() };
    all.into_iter().filter_map(|e| {
        let Item::A(f) = e else { return None };
        let s = |k: usize| match f.get(k) { Some(Item::S(s)) => Some(s.clone()), _ => None };
        Some(Level { internal: s(0)?, name: s(1)?, id: s(2)?, song_file: if f.len() > 4 { s(4) } else { None } })
    }).collect()
}
