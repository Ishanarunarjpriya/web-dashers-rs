//! Web Dashers, native: a window around wd-core with a software renderer.
mod assets;
mod audio;
mod store;
mod canvas;
mod fonts;
mod game;
mod garage;
mod host;
mod levels;
mod plist;
mod player_vis;
mod ui_draw;

use canvas::{Canvas, H, W};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

const JUMP_KEYS: [&str; 6] = ["Space", "KeyG", "KeyL", "KeyW", "ArrowUp", "KeyH"];

struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    host: host::Host,
    canvas: Canvas,
    t0: Instant,
    last: Instant,
    mouse: (f64, f64),
}

impl App {
    /// window pixels -> game coordinates (the picture is centred and scaled to fit)
    fn to_game(&self, x: f64, y: f64) -> (f64, f64) {
        let Some(w) = &self.window else { return (x, y) };
        let s = w.inner_size();
        let scale = (s.width as f64 / W as f64).min(s.height as f64 / H as f64).max(1e-6);
        let ox = (s.width as f64 - W as f64 * scale) / 2.0;
        let oy = (s.height as f64 - H as f64 * scale) / 2.0;
        ((x - ox) / scale, (y - oy) / scale)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = Window::default_attributes().with_title("Web Dashers").with_inner_size(LogicalSize::new(W as f64, H as f64));
        let window = Rc::new(el.create_window(attrs).expect("window"));
        let ctx = softbuffer::Context::new(window.clone()).expect("graphics context");
        self.surface = Some(softbuffer::Surface::new(&ctx, window.clone()).expect("surface"));
        self.window = Some(window);
        el.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    let code = format!("{k:?}");
                    let down = event.state == ElementState::Pressed;
                    let jump = JUMP_KEYS.contains(&code.as_str());
                    self.host.key(&code, down, event.repeat, jump);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = self.to_game(position.x, position.y);
                self.mouse = (x, y);
                self.host.pointer(1, x, y);
            }
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                let (x, y) = self.mouse;
                self.host.pointer(if state == ElementState::Pressed { 0 } else { 2 }, x, y);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta { MouseScrollDelta::LineDelta(_, y) => -y as f64 * 100.0, MouseScrollDelta::PixelDelta(p) => -p.y };
                self.host.wheel(self.mouse.0, self.mouse.1, dy);
            }
            WindowEvent::RedrawRequested => self.frame(el),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(w) = &self.window { w.request_redraw(); }
    }
}

impl App {
    fn frame(&mut self, el: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f64() * 1000.0;
        self.last = now;
        self.host.frame(dt, (now - self.t0).as_secs_f64() * 1000.0, &mut self.canvas);
        if self.host.quit { el.exit(); return; }
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else { return };
        let size = window.inner_size();
        let (ww, wh) = (size.width.max(1) as usize, size.height.max(1) as usize);
        surface.resize(NonZeroU32::new(ww as u32).unwrap(), NonZeroU32::new(wh as u32).unwrap()).ok();
        let Ok(mut buf) = surface.buffer_mut() else { return };
        // the 1138x640 picture, scaled to fit the window with black bars
        let scale = (ww as f64 / W as f64).min(wh as f64 / H as f64);
        let (dw, dh) = (((W as f64 * scale) as usize).min(ww), ((H as f64 * scale) as usize).min(wh));
        let (x0, y0) = ((ww - dw) / 2, (wh - dh) / 2);
        buf.fill(0);
        for y in 0..dh {
            let sy = ((y as f64 / scale) as usize).min(H - 1);
            let row = &self.canvas.px[sy * W..sy * W + W];
            let out = &mut buf[(y0 + y) * ww + x0..(y0 + y) * ww + x0 + dw];
            for (x, o) in out.iter_mut().enumerate() { *o = row[((x as f64 / scale) as usize).min(W - 1)]; }
        }
        buf.present().ok();
    }
}

fn save_png(path: &str, canvas: &Canvas) {
    let file = std::fs::File::create(path).expect("shot file");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), W as u32, H as u32);
    enc.set_color(png::ColorType::Rgb); enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().unwrap();
    let mut data = Vec::with_capacity(W * H * 3);
    for p in &canvas.px { data.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]); }
    w.write_image_data(&data).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let root = match assets::Assets::find(arg("--assets")) { Ok(r) => r, Err(e) => { eprintln!("{e}"); std::process::exit(1); } };
    let assets = match assets::Assets::load(root) { Ok(a) => a, Err(e) => { eprintln!("{e}"); std::process::exit(1); } };
    let mut host = match host::Host::new(assets) { Ok(h) => h, Err(e) => { eprintln!("{e}"); std::process::exit(1); } };
    if let Some(level) = arg("--play") { if let Err(e) = host.play(&level) { eprintln!("{e}"); std::process::exit(1); } }
    // `--shot a.png[,b.png...] --frames N --clicks x,y,x,y --every M`: no window; run frames, click now and then, save pictures (used by the build checks)
    if let Some(paths) = arg("--shot") {
        let frames: usize = arg("--frames").and_then(|f| f.parse().ok()).unwrap_or(120);
        let clicks: Vec<f64> = arg("--clicks").map(|c| c.split(',').filter_map(|v| v.parse().ok()).collect()).unwrap_or_default();
        let every: usize = arg("--every").and_then(|f| f.parse().ok()).unwrap_or(30);
        let hold: usize = arg("--hold").and_then(|f| f.parse().ok()).unwrap_or(0);
        let shots: Vec<&str> = paths.split(',').collect();
        let mut canvas = Canvas::new();
        let mut ci = 0;
        let per = (frames / shots.len()).max(1);
        for f in 0..frames {
            if ci + 1 < clicks.len() && f % every == every - 1 { let (x, y) = (clicks[ci], clicks[ci + 1]); ci += 2; host.pointer(0, x, y); host.pointer(2, x, y); }
            if hold > 0 { host.jump(f % hold < hold / 4); }
            host.frame(1000.0 / 60.0, f as f64 * 1000.0 / 60.0, &mut canvas);
            if (f + 1) % per == 0 { let k = (f + 1) / per - 1; if k < shots.len() { save_png(shots[k], &canvas); } }
        }
        return;
    }
    let el = EventLoop::new().expect("event loop");
    let mut app = App { window: None, surface: None, host, canvas: Canvas::new(), t0: Instant::now(), last: Instant::now(), mouse: (0.0, 0.0) };
    el.run_app(&mut app).expect("run");
}
