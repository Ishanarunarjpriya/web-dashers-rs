//! A small JSON reader (no serde: the crate has no dependencies that need a native build).
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self { Json::Obj(v) => v.iter().find(|(k, _)| k == key).map(|(_, v)| v), _ => None }
    }
    pub fn num(&self) -> Option<f64> { match self { Json::Num(n) => Some(*n), _ => None } }
    pub fn str(&self) -> Option<&str> { match self { Json::Str(s) => Some(s), _ => None } }
    pub fn bool(&self) -> Option<bool> { match self { Json::Bool(b) => Some(*b), _ => None } }
    pub fn arr(&self) -> Option<&[Json]> { match self { Json::Arr(a) => Some(a), _ => None } }
    pub fn entries(&self) -> Option<&[(String, Json)]> { match self { Json::Obj(o) => Some(o), _ => None } }
    pub fn get_num(&self, key: &str) -> Option<f64> { self.get(key).and_then(Json::num) }
    pub fn get_str(&self, key: &str) -> Option<&str> { self.get(key).and_then(Json::str) }
    pub fn get_bool(&self, key: &str) -> Option<bool> { self.get(key).and_then(Json::bool) }
}

pub fn parse(src: &str) -> Result<Json, String> {
    let mut p = P { b: src.as_bytes(), i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() { return Err(format!("trailing data at {}", p.i)); }
    Ok(v)
}

struct P<'a> { b: &'a [u8], i: usize }

impl<'a> P<'a> {
    fn ws(&mut self) { while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\n' | b'\r' | b'\t') { self.i += 1; } }
    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.b.get(self.i).copied() {
            None => Err("unexpected end".into()),
            Some(b'{') => {
                self.i += 1;
                let mut out = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b'}') { self.i += 1; return Ok(Json::Obj(out)); }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    if self.b.get(self.i) != Some(&b':') { return Err(format!("expected ':' at {}", self.i)); }
                    self.i += 1;
                    let v = self.value()?;
                    out.push((k, v));
                    self.ws();
                    match self.b.get(self.i) { Some(b',') => self.i += 1, Some(b'}') => { self.i += 1; break; } _ => return Err(format!("expected ',' or '}}' at {}", self.i)) }
                }
                Ok(Json::Obj(out))
            }
            Some(b'[') => {
                self.i += 1;
                let mut out = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b']') { self.i += 1; return Ok(Json::Arr(out)); }
                loop {
                    out.push(self.value()?);
                    self.ws();
                    match self.b.get(self.i) { Some(b',') => self.i += 1, Some(b']') => { self.i += 1; break; } _ => return Err(format!("expected ',' or ']' at {}", self.i)) }
                }
                Ok(Json::Arr(out))
            }
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') if self.b[self.i..].starts_with(b"true") => { self.i += 4; Ok(Json::Bool(true)) }
            Some(b'f') if self.b[self.i..].starts_with(b"false") => { self.i += 5; Ok(Json::Bool(false)) }
            Some(b'n') if self.b[self.i..].starts_with(b"null") => { self.i += 4; Ok(Json::Null) }
            Some(_) => {
                let s = self.i;
                while self.i < self.b.len() && matches!(self.b[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') { self.i += 1; }
                std::str::from_utf8(&self.b[s..self.i]).ok().and_then(|t| t.parse::<f64>().ok()).map(Json::Num).ok_or_else(|| format!("bad number at {}", s))
            }
        }
    }
    fn string(&mut self) -> Result<String, String> {
        if self.b.get(self.i) != Some(&b'"') { return Err(format!("expected string at {}", self.i)); }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.b.get(self.i).ok_or("unterminated string")?;
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or("bad escape")?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'), b't' => out.push(b'\t'), b'r' => out.push(b'\r'), b'b' => out.push(8), b'f' => out.push(12),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i + 4).ok_or("bad \\u")?).map_err(|e| e.to_string())?;
                            let mut cp = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
                            self.i += 4;
                            if (0xd800..0xdc00).contains(&cp) && self.b[self.i..].starts_with(b"\\u") {
                                let lo = u32::from_str_radix(std::str::from_utf8(&self.b[self.i + 2..self.i + 6]).map_err(|e| e.to_string())?, 16).map_err(|e| e.to_string())?;
                                self.i += 6;
                                cp = 0x10000 + ((cp - 0xd800) << 10) + (lo - 0xdc00);
                            }
                            let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                _ => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|e| e.to_string())
    }
}
