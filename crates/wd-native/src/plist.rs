//! The icon atlases: Apple property lists (`icons/<icon>-hd.plist`) next to a png.
use crate::assets::Frame;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum P { D(Vec<(String, P)>), A(Vec<P>), S(String), N(f64), B(bool) }

impl P {
    pub fn get(&self, k: &str) -> Option<&P> { if let P::D(d) = self { d.iter().find(|(n, _)| n == k).map(|(_, v)| v) } else { None } }
    pub fn s(&self) -> &str { if let P::S(s) = self { s } else { "" } }
    pub fn n(&self) -> f64 { if let P::N(n) = self { *n } else { 0.0 } }
    pub fn b(&self) -> bool { matches!(self, P::B(true)) }
}

fn parse_node(toks: &mut std::iter::Peekable<std::vec::IntoIter<(String, String)>>, open: (String, String)) -> P {
    match open.0.as_str() {
        "dict" => {
            let mut out = Vec::new();
            while let Some((t, _)) = toks.peek().cloned() {
                if t == "/dict" { toks.next(); break; }
                let Some((kt, key)) = toks.next() else { break };
                if kt != "key" { continue; }
                let Some(v) = toks.next() else { break };
                out.push((key, parse_node(toks, v)));
            }
            P::D(out)
        }
        "array" => {
            let mut out = Vec::new();
            while let Some((t, _)) = toks.peek().cloned() {
                if t == "/array" { toks.next(); break; }
                let Some(v) = toks.next() else { break };
                out.push(parse_node(toks, v));
            }
            P::A(out)
        }
        "integer" | "real" => P::N(open.1.trim().parse().unwrap_or(0.0)),
        "true" => P::B(true),
        "false" => P::B(false),
        _ => P::S(open.1),
    }
}

pub fn parse(src: &str) -> Option<P> {
    // tokens: (tag, text) where text is the content of leaf tags (key, string, integer, real)
    let mut toks: Vec<(String, String)> = Vec::new();
    let mut rest = src;
    while let Some(lt) = rest.find('<') {
        let Some(gt) = rest[lt..].find('>') else { break };
        let tag = rest[lt + 1..lt + gt].trim();
        rest = &rest[lt + gt + 1..];
        if tag.starts_with('?') || tag.starts_with('!') { continue; }
        let selfclose = tag.ends_with('/');
        let name = tag.trim_end_matches('/').split_whitespace().next().unwrap_or("").to_string();
        match name.as_str() {
            "key" | "string" | "integer" | "real" => {
                let end = rest.find('<').unwrap_or(rest.len());
                let text = rest[..end].replace("&amp;", "&");
                rest = &rest[end..];
                if let Some(c) = rest.find('>') { rest = &rest[c + 1..]; }
                toks.push((name, text));
            }
            "true" | "false" => toks.push((name, String::new())),
            "dict" | "array" | "/dict" | "/array" => { if !selfclose { toks.push((name, String::new())); } }
            _ => {}
        }
    }
    let mut it = toks.into_iter().peekable();
    // skip <plist>, find the first dict
    while let Some((t, _)) = it.peek() { if t == "dict" { break; } it.next(); }
    let first = it.next()?;
    Some(parse_node(&mut it, first))
}

fn nums(s: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() || ch == '.' || (ch == '-' && cur.is_empty()) { cur.push(ch); }
        else { if !cur.is_empty() { if let Ok(v) = cur.parse() { out.push(v); } cur.clear(); } if ch == '-' { cur.push(ch); } }
    }
    out
}

/// The frames of a cocos2d plist as atlas frames (`plistFrames` of player_vis.js).
pub fn frames(p: &P) -> HashMap<String, Frame> {
    let fmt = p.get("metadata").and_then(|m| m.get("format")).map_or(3.0, |f| f.n());
    let mut out = HashMap::new();
    let Some(P::D(fr)) = p.get("frames") else { return out };
    for (name, d) in fr {
        let get = |k: &str| d.get(k).map(|v| v.s().to_string()).unwrap_or_default();
        let (x, y, w, h, rotated, ox, oy, sw, sh);
        if fmt >= 3.0 {
            let r = nums(&get("textureRect")); let s = nums(&get("spriteSize")); let o = nums(&get("spriteOffset")); let ss = nums(&get("spriteSourceSize"));
            (x, y) = (r.first().copied().unwrap_or(0.0), r.get(1).copied().unwrap_or(0.0));
            (w, h) = (s.first().copied().unwrap_or(0.0), s.get(1).copied().unwrap_or(0.0));
            rotated = d.get("textureRotated").map_or(false, |v| v.b());
            (ox, oy) = (o.first().copied().unwrap_or(0.0), o.get(1).copied().unwrap_or(0.0));
            (sw, sh) = (ss.first().copied().unwrap_or(w), ss.get(1).copied().unwrap_or(h));
        } else {
            let r = nums(&get("frame")); let o = nums(&get("offset")); let ss = nums(&get("sourceSize"));
            (x, y, w, h) = (r.first().copied().unwrap_or(0.0), r.get(1).copied().unwrap_or(0.0), r.get(2).copied().unwrap_or(0.0), r.get(3).copied().unwrap_or(0.0));
            rotated = d.get("rotated").map_or(false, |v| v.b());
            (ox, oy) = (o.first().copied().unwrap_or(0.0), o.get(1).copied().unwrap_or(0.0));
            (sw, sh) = (ss.first().copied().unwrap_or(w), ss.get(1).copied().unwrap_or(h));
        }
        out.insert(name.clone(), Frame { x, y, w, h, rotated, src_w: sw, src_h: sh, ss_x: ((sw - w) / 2.0 + ox).round(), ss_y: ((sh - h) / 2.0 - oy).round() });
    }
    out
}
