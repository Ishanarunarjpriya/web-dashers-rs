//! A software canvas: the 1138x640 game picture as 0x00RRGGBB pixels, with the few operations the game needs
//! (textured, tinted, rotated sprites with alpha or additive blending, and tiled images).
use crate::assets::{Frame, Image};

pub const W: usize = 1138;
pub const H: usize = 640;

pub struct Canvas { pub px: Vec<u32>, pub clip: (i64, i64, i64, i64) }

#[derive(Clone, Copy)]
pub struct Sprite { pub x: f64, pub y: f64, pub rot: f64, pub sx: f64, pub sy: f64, pub alpha: f64, pub tint: i64, pub additive: bool, pub flip_x: bool, pub flip_y: bool }

fn tint_rgb(tint: i64) -> [u32; 3] {
    if tint < 0 || tint == 0xffffff { [255; 3] } else { [((tint >> 16) & 255) as u32, ((tint >> 8) & 255) as u32, (tint & 255) as u32] }
}

#[inline]
fn put(dst: &mut u32, r: u32, g: u32, b: u32, a: u32, additive: bool) {
    // r, g, b are the colour already multiplied by the source alpha (premultiplied), a is the alpha 0..255
    let (dr, dg, db) = ((*dst >> 16) & 255, (*dst >> 8) & 255, *dst & 255);
    let (nr, ng, nb) = if additive { ((dr + r).min(255), (dg + g).min(255), (db + b).min(255)) } else {
        let ia = 255 - a;
        (r + (dr * ia + 127) / 255, g + (dg * ia + 127) / 255, b + (db * ia + 127) / 255)
    };
    *dst = (nr.min(255) << 16) | (ng.min(255) << 8) | nb.min(255);
}

impl Canvas {
    pub fn new() -> Canvas { Canvas { px: vec![0; W * H], clip: (0, 0, W as i64, H as i64) } }

    pub fn fill(&mut self, color: u32) { for p in self.px.iter_mut() { *p = color & 0xffffff; } }

    /// Bilinear sample of a rectangle of `img` at (u, v) in texel units from the rectangle's corner; premultiplied RGBA 0..255 as floats.
    #[inline]
    fn sample(img: &Image, rx: usize, ry: usize, rw: usize, rh: usize, u: f64, v: f64) -> [f64; 4] {
        let fx = (u - 0.5).max(0.0).min(rw as f64 - 1.0);
        let fy = (v - 0.5).max(0.0).min(rh as f64 - 1.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(rw - 1), (y0 + 1).min(rh - 1));
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let at = |x: usize, y: usize| -> [f64; 4] { let p = img.px[(ry + y) * img.w + rx + x]; let a = p[3] as f64; [p[0] as f64 * a / 255.0, p[1] as f64 * a / 255.0, p[2] as f64 * a / 255.0, a] };
        let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
        let mut o = [0.0; 4];
        for i in 0..4 { o[i] = (a[i] * (1.0 - tx) + b[i] * tx) * (1.0 - ty) + (c[i] * (1.0 - tx) + d[i] * tx) * ty; }
        o
    }

    /// One frame of an atlas (`drawFrame` of the web host): `s.x, s.y` is where the frame's centre of the untrimmed image lands.
    pub fn frame(&mut self, img: &Image, f: &Frame, s: Sprite) {
        let (fw, fh) = (f.w, f.h);
        if fw <= 0.0 || fh <= 0.0 || s.alpha <= 0.0 { return; }
        let ox = -f.src_w / 2.0 + f.ss_x;
        let oy = -f.src_h / 2.0 + f.ss_y;
        let sx = if s.flip_x { -s.sx } else { s.sx };
        let sy = if s.flip_y { -s.sy } else { s.sy };
        if sx == 0.0 || sy == 0.0 { return; }
        let (cs, sn) = if s.rot != 0.0 { (s.rot.cos(), s.rot.sin()) } else { (1.0, 0.0) };
        // screen bounding box of the four corners
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (lx, ly) in [(ox, oy), (ox + fw, oy), (ox + fw, oy + fh), (ox, oy + fh)] {
            let (ax, ay) = (lx * sx, ly * sy);
            let (px, py) = (s.x + ax * cs - ay * sn, s.y + ax * sn + ay * cs);
            x0 = x0.min(px); x1 = x1.max(px); y0 = y0.min(py); y1 = y1.max(py);
        }
        let (bx0, by0) = (x0.floor().max(self.clip.0 as f64) as usize, y0.floor().max(self.clip.1 as f64) as usize);
        let (bx1, by1) = ((x1.ceil().min(self.clip.2 as f64)) as usize, (y1.ceil().min(self.clip.3 as f64)) as usize);
        if bx0 >= bx1 || by0 >= by1 { return; }
        let t = tint_rgb(s.tint);
        let alpha = s.alpha.clamp(0.0, 1.0);
        let (rx, ry) = (f.x as usize, f.y as usize);
        let (rw, rh) = if f.rotated { (fh as usize, fw as usize) } else { (fw as usize, fh as usize) };
        if rw == 0 || rh == 0 || rx + rw > img.w || ry + rh > img.h { return; }
        for py in by0..by1 {
            let dy = py as f64 + 0.5 - s.y;
            for px in bx0..bx1 {
                let dx = px as f64 + 0.5 - s.x;
                let ax = dx * cs + dy * sn;
                let ay = -dx * sn + dy * cs;
                let (lx, ly) = (ax / sx, ay / sy);
                let (u, v) = (lx - ox, ly - oy);
                if u < 0.0 || v < 0.0 || u >= fw || v >= fh { continue; }
                let (tu, tv) = if f.rotated { (fh - v, u) } else { (u, v) };
                let c = Canvas::sample(img, rx, ry, rw, rh, tu, tv);
                if c[3] <= 0.0 { continue; }
                let r = (c[0] * t[0] as f64 / 255.0 * alpha) as u32;
                let g = (c[1] * t[1] as f64 / 255.0 * alpha) as u32;
                let b = (c[2] * t[2] as f64 / 255.0 * alpha) as u32;
                put(&mut self.px[py * W + px], r, g, b, (c[3] * alpha) as u32, s.additive);
            }
        }
    }

    /// A whole image at 1:1 (or `flip_v` mirrored vertically), multiplied by `tint`, with its top-left corner at (x, y).
    pub fn image(&mut self, img: &Image, x: f64, y: f64, tint: i64, flip_v: bool) {
        let t = tint_rgb(tint);
        let (xi, yi) = (x.round() as i64, y.round() as i64);
        for iy in 0..img.h as i64 {
            let sy = yi + iy;
            if sy < 0 || sy >= H as i64 { continue; }
            let row = if flip_v { img.h as i64 - 1 - iy } else { iy } as usize;
            for ix in 0..img.w as i64 {
                let sx = xi + ix;
                if sx < 0 || sx >= W as i64 { continue; }
                let p = img.px[row * img.w + ix as usize];
                if p[3] == 0 { continue; }
                let a = p[3] as u32;
                let (r, g, b) = (p[0] as u32 * t[0] / 255 * a / 255, p[1] as u32 * t[1] / 255 * a / 255, p[2] as u32 * t[2] / 255 * a / 255);
                put(&mut self.px[sy as usize * W + sx as usize], r, g, b, a, false);
            }
        }
    }

    /// A flat rectangle outline or fill helper for the placeholder player.
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: u32, alpha: f64) {
        let (r, g, b) = ((color >> 16) & 255, (color >> 8) & 255, color & 255);
        let a = (alpha.clamp(0.0, 1.0) * 255.0) as u32;
        for py in (y.max(0.0) as usize)..((y + h).min(H as f64) as usize) {
            for px in (x.max(0.0) as usize)..((x + w).min(W as f64) as usize) { put(&mut self.px[py * W + px], r * a / 255, g * a / 255, b * a / 255, a, false); }
        }
    }
}

/// An affine transform: X = a x + c y + e, Y = b x + d y + f (the canvas `setTransform` matrix).
#[derive(Clone, Copy)]
pub struct Mat { pub a: f64, pub b: f64, pub c: f64, pub d: f64, pub e: f64, pub f: f64 }
impl Mat {
    pub fn id() -> Mat { Mat { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 } }
    /// `self` followed by the local transform `o` (canvas `transform`: later calls apply to the local space).
    pub fn then(self, o: Mat) -> Mat {
        Mat { a: self.a * o.a + self.c * o.b, b: self.b * o.a + self.d * o.b, c: self.a * o.c + self.c * o.d, d: self.b * o.c + self.d * o.d,
              e: self.a * o.e + self.c * o.f + self.e, f: self.b * o.e + self.d * o.f + self.f }
    }
    pub fn translate(self, x: f64, y: f64) -> Mat { self.then(Mat { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: x, f: y }) }
    pub fn scale(self, x: f64, y: f64) -> Mat { self.then(Mat { a: x, b: 0.0, c: 0.0, d: y, e: 0.0, f: 0.0 }) }
    pub fn rotate(self, r: f64) -> Mat { let (s, c) = r.sin_cos(); self.then(Mat { a: c, b: s, c: -s, d: c, e: 0.0, f: 0.0 }) }
    fn apply(&self, x: f64, y: f64) -> (f64, f64) { (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f) }
    fn inverse(&self) -> Option<Mat> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-12 { return None; }
        let (ia, ib, ic, id) = (self.d / det, -self.b / det, -self.c / det, self.a / det);
        Some(Mat { a: ia, b: ib, c: ic, d: id, e: -(ia * self.e + ic * self.f), f: -(ib * self.e + id * self.f) })
    }
}

#[derive(Clone, Copy)]
pub struct Paint { pub alpha: f64, pub tint: i64, pub additive: bool, pub gray: bool }

impl Canvas {
    /// `drawImage(img, sx, sy, sw, sh, dx, dy, dw, dh)` under the matrix `m`; the source rectangle may be fractional (the half texel inset).
    /// Texels are only read from inside the integer frame `(fx, fy, fw, fh)` so neighbours of an atlas never bleed in.
    pub fn draw_region(&mut self, img: &Image, src: (f64, f64, f64, f64), dst: (f64, f64, f64, f64), m: Mat, p: Paint, frame: (usize, usize, usize, usize)) {
        let (sx, sy, sw, sh) = src;
        let (dx, dy, dw, dh) = dst;
        if dw <= 0.0 || dh <= 0.0 || sw <= 0.0 || sh <= 0.0 || p.alpha <= 0.0 { return; }
        let Some(inv) = m.inverse() else { return };
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (lx, ly) in [(dx, dy), (dx + dw, dy), (dx + dw, dy + dh), (dx, dy + dh)] {
            let (px, py) = m.apply(lx, ly);
            x0 = x0.min(px); x1 = x1.max(px); y0 = y0.min(py); y1 = y1.max(py);
        }
        let bx0 = (x0.floor() as i64).max(self.clip.0); let by0 = (y0.floor() as i64).max(self.clip.1);
        let bx1 = (x1.ceil() as i64).min(self.clip.2); let by1 = (y1.ceil() as i64).min(self.clip.3);
        if bx0 >= bx1 || by0 >= by1 { return; }
        let t = tint_rgb(p.tint);
        let alpha = p.alpha.clamp(0.0, 1.0);
        let (fx, fy, fw, fh) = frame;
        let (kx, ky) = (sw / dw, sh / dh);
        let minify = kx.max(ky) > 1.5;
        for py in by0..by1 {
            for px in bx0..bx1 {
                let (lx, ly) = inv.apply(px as f64 + 0.5, py as f64 + 0.5);
                let (u, v) = (lx - dx, ly - dy);
                if u < 0.0 || v < 0.0 || u >= dw || v >= dh { continue; }
                let (tu, tv) = (sx - fx as f64 + u * kx, sy - fy as f64 + v * ky);
                let mut c = if minify { Canvas::sample_box(img, fx, fy, fw, fh, tu, tv, kx, ky) } else { Canvas::sample(img, fx, fy, fw, fh, tu, tv) };
                if c[3] <= 0.0 { continue; }
                if p.gray { let l = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]; c[0] = l; c[1] = l; c[2] = l; }
                let r = (c[0] * t[0] as f64 / 255.0 * alpha) as u32;
                let g = (c[1] * t[1] as f64 / 255.0 * alpha) as u32;
                let b = (c[2] * t[2] as f64 / 255.0 * alpha) as u32;
                put(&mut self.px[py as usize * W + px as usize], r, g, b, (c[3] * alpha) as u32, p.additive);
            }
        }
    }

    /// A few bilinear taps spread over the source footprint, for images drawn much smaller than they are.
    fn sample_box(img: &Image, rx: usize, ry: usize, rw: usize, rh: usize, u: f64, v: f64, kx: f64, ky: f64) -> [f64; 4] {
        let n = (kx.max(ky).ceil() as usize).clamp(2, 4);
        let mut o = [0.0; 4];
        for j in 0..n { for i in 0..n {
            let su = u + ((i as f64 + 0.5) / n as f64 - 0.5) * kx;
            let sv = v + ((j as f64 + 0.5) / n as f64 - 0.5) * ky;
            let c = Canvas::sample(img, rx, ry, rw, rh, su, sv);
            for q in 0..4 { o[q] += c[q]; }
        } }
        for q in 0..4 { o[q] /= (n * n) as f64; }
        o
    }

    /// A filled rounded rectangle (alpha blended), `radius` 0 for a plain rectangle.
    pub fn fill_round(&mut self, x: f64, y: f64, w: f64, h: f64, radius: f64, color: u32, alpha: f64, m: Mat) {
        let Some(inv) = m.inverse() else { return };
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (lx, ly) in [(x, y), (x + w, y), (x + w, y + h), (x, y + h)] { let (px, py) = m.apply(lx, ly); x0 = x0.min(px); x1 = x1.max(px); y0 = y0.min(py); y1 = y1.max(py); }
        let (r, g, b) = ((color >> 16) & 255, (color >> 8) & 255, color & 255);
        let a = (alpha.clamp(0.0, 1.0) * 255.0) as u32;
        let rad = radius.min(w / 2.0).min(h / 2.0).max(0.0);
        for py in (y0.floor() as i64).max(self.clip.1)..(y1.ceil() as i64).min(self.clip.3) {
            for px in (x0.floor() as i64).max(self.clip.0)..(x1.ceil() as i64).min(self.clip.2) {
                let (lx, ly) = inv.apply(px as f64 + 0.5, py as f64 + 0.5);
                if lx < x || ly < y || lx >= x + w || ly >= y + h { continue; }
                let mut cov = 1.0;
                if rad > 0.0 {
                    let cx = lx.clamp(x + rad, x + w - rad); let cy = ly.clamp(y + rad, y + h - rad);
                    let d = ((lx - cx).powi(2) + (ly - cy).powi(2)).sqrt();
                    cov = (rad - d + 0.5).clamp(0.0, 1.0);
                }
                if cov <= 0.0 { continue; }
                let aa = (a as f64 * cov) as u32;
                put(&mut self.px[py as usize * W + px as usize], r * aa / 255, g * aa / 255, b * aa / 255, aa, false);
            }
        }
    }

    pub fn fill_circle(&mut self, cx: f64, cy: f64, radius: f64, color: u32, alpha: f64, m: Mat) {
        let Some(inv) = m.inverse() else { return };
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (lx, ly) in [(cx - radius, cy - radius), (cx + radius, cy - radius), (cx + radius, cy + radius), (cx - radius, cy + radius)] { let (px, py) = m.apply(lx, ly); x0 = x0.min(px); x1 = x1.max(px); y0 = y0.min(py); y1 = y1.max(py); }
        let (r, g, b) = ((color >> 16) & 255, (color >> 8) & 255, color & 255);
        for py in (y0.floor() as i64).max(self.clip.1)..(y1.ceil() as i64).min(self.clip.3) {
            for px in (x0.floor() as i64).max(self.clip.0)..(x1.ceil() as i64).min(self.clip.2) {
                let (lx, ly) = inv.apply(px as f64 + 0.5, py as f64 + 0.5);
                let d = ((lx - cx).powi(2) + (ly - cy).powi(2)).sqrt();
                let cov = (radius - d + 0.5).clamp(0.0, 1.0);
                if cov <= 0.0 { continue; }
                let aa = (alpha.clamp(0.0, 1.0) * cov * 255.0) as u32;
                put(&mut self.px[py as usize * W + px as usize], r * aa / 255, g * aa / 255, b * aa / 255, aa, false);
            }
        }
    }
}
