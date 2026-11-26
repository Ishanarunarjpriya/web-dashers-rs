//! A software canvas: the 1138x640 game picture as 0x00RRGGBB pixels, with the few operations the game needs
//! (textured, tinted, rotated sprites with alpha or additive blending, and tiled images).
use crate::assets::{Frame, Image};

pub const W: usize = 1138;
pub const H: usize = 640;

pub struct Canvas { pub px: Vec<u32> }

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
    pub fn new() -> Canvas { Canvas { px: vec![0; W * H] } }

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
        let (bx0, by0) = (x0.floor().max(0.0) as usize, y0.floor().max(0.0) as usize);
        let (bx1, by1) = ((x1.ceil().min(W as f64)) as usize, (y1.ceil().min(H as f64)) as usize);
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
