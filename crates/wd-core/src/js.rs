//! JavaScript number-parsing semantics, so ported code behaves like the original.

/// `parseInt(s, 10)`: optional whitespace and sign, then leading digits. `None` is NaN.
pub fn parse_int(s: &str) -> Option<i64> {
    let t = s.trim_start_matches(is_js_space);
    let (neg, rest) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let n: f64 = digits.parse().ok()?;
    let n = if neg { -n } else { n };
    Some(n as i64)
}

/// `parseFloat(s)`: longest valid decimal prefix (or Infinity). `None` is NaN.
pub fn parse_float(s: &str) -> Option<f64> {
    let t = s.trim_start_matches(is_js_space);
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    if t[i..].starts_with("Infinity") {
        return Some(if b.first() == Some(&b'-') { f64::NEG_INFINITY } else { f64::INFINITY });
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - int_start;
    if i < b.len() && b[i] == b'.' {
        let j = i + 1;
        let mut k = j;
        while k < b.len() && b[k].is_ascii_digit() {
            k += 1;
        }
        if digits > 0 || k > j {
            digits += k - j;
            i = k;
        }
    }
    if digits == 0 {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut k = i + 1;
        if k < b.len() && (b[k] == b'+' || b[k] == b'-') {
            k += 1;
        }
        let ds = k;
        while k < b.len() && b[k].is_ascii_digit() {
            k += 1;
        }
        if k > ds {
            i = k;
        }
    }
    t[..i].parse::<f64>().ok()
}

fn is_js_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// `Number(s)` for a string: whitespace-trimmed decimal / hex / octal / binary / `Infinity`; anything else is NaN.
pub fn to_number(s: &str) -> f64 {
    let t = s.trim_matches(is_js_space);
    if t.is_empty() { return 0.0; }
    let radix = |digits: &str, r: u32| -> f64 {
        if digits.is_empty() { return f64::NAN; }
        let mut v = 0f64;
        for c in digits.chars() {
            match c.to_digit(r) { Some(d) => v = v * r as f64 + d as f64, None => return f64::NAN }
        }
        v
    };
    if let Some(r) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) { return radix(r, 16); }
    if let Some(r) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) { return radix(r, 8); }
    if let Some(r) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) { return radix(r, 2); }
    let (neg, body) = match t.as_bytes()[0] { b'-' => (true, &t[1..]), b'+' => (false, &t[1..]), _ => (false, t) };
    if body == "Infinity" { return if neg { f64::NEG_INFINITY } else { f64::INFINITY }; }
    let b = body.as_bytes();
    let mut i = 0;
    let int0 = i;
    while i < b.len() && b[i].is_ascii_digit() { i += 1; }
    let mut digits = i - int0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let f0 = i;
        while i < b.len() && b[i].is_ascii_digit() { i += 1; }
        digits += i - f0;
    }
    if digits == 0 { return f64::NAN; }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') { i += 1; }
        let e0 = i;
        while i < b.len() && b[i].is_ascii_digit() { i += 1; }
        if i == e0 { return f64::NAN; }
    }
    if i != b.len() { return f64::NAN; }
    match body.parse::<f64>() { Ok(v) => if neg { -v } else { v }, Err(_) => f64::NAN }
}
