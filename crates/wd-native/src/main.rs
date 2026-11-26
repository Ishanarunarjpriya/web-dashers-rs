//! Web Dashers, native: a window around wd-core with a software renderer.
mod assets;
mod canvas;
mod game;

use canvas::{Canvas, H, W};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    game: game::Game,
    canvas: Canvas,
    t0: Instant,
    last: Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
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
                let down = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(k) = event.physical_key {
                    match k {
                        KeyCode::Escape if down => el.exit(),
                        KeyCode::Space | KeyCode::ArrowUp | KeyCode::KeyW | KeyCode::KeyG | KeyCode::KeyL => self.game.held = down,
                        KeyCode::KeyR if down && !event.repeat => self.game.restart(),
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => self.game.held = state == ElementState::Pressed,
            WindowEvent::RedrawRequested => self.frame(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(w) = &self.window { w.request_redraw(); }
    }
}

impl App {
    fn frame(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f64() * 1000.0;
        self.last = now;
        self.game.step(dt.min(100.0), (now - self.t0).as_secs_f64() * 1000.0);
        self.game.draw(&mut self.canvas);
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else { return };
        let size = window.inner_size();
        let (ww, wh) = (size.width.max(1) as usize, size.height.max(1) as usize);
        surface.resize(NonZeroU32::new(ww as u32).unwrap(), NonZeroU32::new(wh as u32).unwrap()).ok();
        let Ok(mut buf) = surface.buffer_mut() else { return };
        // the 1138x640 picture, scaled to fit the window with black bars
        let scale = (ww as f64 / W as f64).min(wh as f64 / H as f64);
        let (dw, dh) = ((W as f64 * scale) as usize, (H as f64 * scale) as usize);
        let (x0, y0) = ((ww - dw.min(ww)) / 2, (wh - dh.min(wh)) / 2);
        buf.fill(0);
        for y in 0..dh.min(wh) {
            let sy = ((y as f64 / scale) as usize).min(H - 1);
            let row = &self.canvas.px[sy * W..sy * W + W];
            let out = &mut buf[(y0 + y) * ww + x0..(y0 + y) * ww + x0 + dw.min(ww)];
            for (x, o) in out.iter_mut().enumerate() { *o = row[((x as f64 / scale) as usize).min(W - 1)]; }
        }
        buf.present().ok();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let root = match assets::Assets::find(arg("--assets")) { Ok(r) => r, Err(e) => { eprintln!("{e}"); std::process::exit(1); } };
    let assets = match assets::Assets::load(root) { Ok(a) => a, Err(e) => { eprintln!("{e}"); std::process::exit(1); } };
    let mut game = game::Game::new(assets);
    let level = arg("--level").unwrap_or_else(|| "1".into());
    if let Err(e) = game.load_level(&level) { eprintln!("{e}"); std::process::exit(1); }
    // `--shot file.png [frames]`: no window, run some frames and save the picture (used by the build checks)
    if let Some(path) = arg("--shot") {
        let frames: usize = arg("--frames").and_then(|f| f.parse().ok()).unwrap_or(120);
        let mut canvas = Canvas::new();
        for i in 0..frames { game.held = i % 40 < 6; game.step(1000.0 / 60.0, i as f64 * 1000.0 / 60.0); }
        game.draw(&mut canvas);
        let file = std::fs::File::create(&path).expect("shot file");
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), W as u32, H as u32);
        enc.set_color(png::ColorType::Rgb); enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().unwrap();
        let mut data = Vec::with_capacity(W * H * 3);
        for p in &canvas.px { data.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]); }
        w.write_image_data(&data).unwrap();
        return;
    }
    let el = EventLoop::new().expect("event loop");
    let mut app = App { window: None, surface: None, game, canvas: Canvas::new(), t0: Instant::now(), last: Instant::now() };
    el.run_app(&mut app).expect("run");
}
