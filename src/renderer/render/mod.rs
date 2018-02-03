use std::time::{Duration, Instant};
use std::thread;

use util;
use world::World;

pub mod background;
mod viewport;

use self::background::Background;
pub use self::viewport::Viewport;

use glium;
use glium::glutin;
use glium::Surface;
use glium::backend::Facade;
use glium::index::PrimitiveType;

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];
pub const QUAD: [Vertex; 4] = [
    Vertex { position: [0, 1] },
    Vertex { position: [1, 1] },
    Vertex { position: [0, 0] },
    Vertex { position: [1, 0] },
];

pub fn load_program<F: Facade>(
    display: &F,
    vert: &str,
    frag: &str,
) -> Result<glium::Program, glium::ProgramCreationError> {
    let vertex_shader = util::read_string(&format!("data/shaders/{}", vert));
    let fragment_shader = util::read_string(&format!("data/shaders/{}", frag));

    glium::Program::from_source(display, &vertex_shader, &fragment_shader, None)
}

pub fn make_quad_buffers<F: Facade>(
    display: &F,
) -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer<u16>) {
    let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
    let indices =
        glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES)
            .unwrap();
    (vertices, indices)
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [i32; 2],
}

implement_vertex!(Vertex, position);

pub struct RenderContext {
    backend: glium::Display,
    events_loop: glutin::EventsLoop,
    background: Background,
    pub viewport: Viewport,
    accumulator: FpsAccumulator,
}

pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_HEIGHT: u32 = 1024;

impl RenderContext {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();

        let window = glutin::WindowBuilder::new()
            .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
            .with_title("ポテト団");

        let context = glutin::ContextBuilder::new();

        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let bg = Background::new(&display);

        let scale = display.gl_window().hidpi_factor();

        let accumulator = FpsAccumulator::new();

        let viewport = Viewport {
            position: (0, 0),
            size: (SCREEN_WIDTH, SCREEN_HEIGHT),
            scale: scale,
            camera: (0, 0),
        };

        RenderContext {
            backend: display,
            events_loop: events_loop,

            background: bg,
            accumulator: accumulator,
            viewport: viewport,
        }
    }

    pub fn update(&mut self, world: &World) {

    }

    pub fn render(&mut self) {
        let mut target = self.backend.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let millis = self.accumulator.millis_since_start();

        self.background
            .render(&self.backend, &mut target, &self.viewport, millis);

        target.finish().unwrap();
    }

    pub fn set_viewport(&mut self, w: u32, h: u32) {
        let scale = self.backend.gl_window().hidpi_factor();
        self.viewport = Viewport {
            position: (0, 0),
            size: (w, h),
            scale: scale,
            camera: self.viewport.camera,
        };

        //self.ui = Ui::new(&self.backend, &self.viewport);
    }

    pub fn poll_events<F>(&mut self, callback: F)
    where
        F: FnMut(glutin::Event),
    {
        self.events_loop.poll_events(callback)
    }

    pub fn step_frame(&mut self) {
        self.accumulator.step_frame();

        thread::sleep(self.accumulator.sleep_time());
    }
}

pub trait Renderable {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, time: u64)
    where
        F: glium::backend::Facade,
        S: glium::Surface;
}

pub struct FpsAccumulator {
    start: Instant,
    frame_count: u32,
    last_time: u64,
    accumulator: Duration,
    previous_clock: Instant,
}

impl FpsAccumulator {
    pub fn new() -> Self {
        FpsAccumulator {
            start: Instant::now(),
            frame_count: 0,
            last_time: 0,
            accumulator: Duration::new(0, 0),
            previous_clock: Instant::now(),
        }
    }

    pub fn step_frame(&mut self) {
        let now = Instant::now();
        self.accumulator += now - self.previous_clock;
        self.previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667);
        while self.accumulator >= fixed_time_stamp {
            self.accumulator -= fixed_time_stamp;
        }

        let millis = ::util::get_duration_millis(&Instant::now().duration_since(self.start));

        if millis - self.last_time >= 1000 {
            let ms_per_frame = 1000.0 / self.frame_count as f32;
            println!("{} ms/frame | {} fps", ms_per_frame, 1000.0 / ms_per_frame);
            self.frame_count = 0;
            self.last_time += 1000;
        }

        self.frame_count += 1;
    }

    pub fn sleep_time(&self) -> Duration {
        Duration::new(0, 16666667) - self.accumulator
    }

    pub fn millis_since_start(&self) -> u64 {
        let duration = Instant::now().duration_since(self.start);
        ::util::get_duration_millis(&duration)
    }
}
