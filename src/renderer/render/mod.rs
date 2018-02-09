use std::time::{Duration, Instant};
use std::thread;

use util;
use world::World;

pub mod background;
pub mod spritemap;
pub mod primitives;
mod viewport;

use debug;
use point;
use self::background::Background;
use self::spritemap::SpriteMap;
use self::primitives::Primitives;
pub use self::viewport::Viewport;

use engine::MouseState;
use renderer::RenderUpdate;
use renderer::ui::*;

use glium;
use glium::glutin;
use glium::Surface;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use imgui;
use imgui_glium_renderer;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [i32; 2],
}

implement_vertex!(Vertex, position);

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];
pub const QUAD: [Vertex; 4] = [
    Vertex { position: [0, 1] },
    Vertex { position: [1, 1] },
    Vertex { position: [0, 0] },
    Vertex { position: [1, 0] },
];

#[derive(Copy, Clone)]
pub struct Vertex3 {
    pub position: [i32; 3],
}

implement_vertex!(Vertex3, position);

pub const CUBE_INDICES: [u16; 36] = [0,  1,  2,  0,  2,  3,   //front
                                     4,  5,  6,  4,  6,  7,   //right
                                     8,  9,  10, 8,  10, 11,  //back
                                     12, 13, 14, 12, 14, 15,  //left
                                     16, 17, 18, 16, 18, 19,  //upper
                                     20, 21, 22, 20, 22, 23];

pub const CUBE: [Vertex3; 24] = [
    Vertex3 { position: [-1, -1,  1] },
    Vertex3 { position: [ 1, -1,  1] },
    Vertex3 { position: [ 1,  1,  1] },
    Vertex3 { position: [-1,  1,  1] },

    Vertex3 { position: [1,   1,  1] },
    Vertex3 { position: [1,   1, -1] },
    Vertex3 { position: [1,  -1, -1] },
    Vertex3 { position: [1,  -1,  1] },

    Vertex3 { position: [-1, -1, -1] },
    Vertex3 { position: [ 1, -1, -1] },
    Vertex3 { position: [ 1,  1, -1] },
    Vertex3 { position: [-1,  1, -1] },

    Vertex3 { position: [-1, -1, -1] },
    Vertex3 { position: [-1, -1,  1] },
    Vertex3 { position: [-1,  1,  1] },
    Vertex3 { position: [-1,  1, -1] },

    Vertex3 { position: [ 1,  1,  1] },
    Vertex3 { position: [-1,  1,  1] },
    Vertex3 { position: [-1,  1, -1] },
    Vertex3 { position: [ 1,  1, -1] },

    Vertex3 { position: [-1, -1, -1] },
    Vertex3 { position: [ 1, -1, -1] },
    Vertex3 { position: [ 1, -1,  1] },
    Vertex3 { position: [-1, -1,  1] },
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

pub fn make_cube_buffers<F: Facade>(
    display: &F,
) -> (glium::VertexBuffer<Vertex3>, glium::IndexBuffer<u16>) {
    let vertices = glium::VertexBuffer::immutable(display, &CUBE).unwrap();
    let indices =
        glium::IndexBuffer::immutable(display, PrimitiveType::LineLoop, &CUBE_INDICES)
        .unwrap();
    (vertices, indices)
}

pub struct RenderContext {
    backend: glium::Display,
    events_loop: glutin::EventsLoop,

    background: Background,
    spritemap: SpriteMap,
    primitives: Primitives,
    ui: Ui,
    imgui: imgui::ImGui,
    imgui_renderer: imgui_glium_renderer::Renderer,

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

        let scale = display.gl_window().hidpi_factor();

        let viewport = Viewport {
            position: (0, 0),
            size: (SCREEN_WIDTH, SCREEN_HEIGHT),
            scale: scale,
            camera: (0.0, 0.0, 0.0),
        };

        let bg = Background::new(&display);
        let sprite = SpriteMap::new(&display);
        let prim = Primitives::new(&display);
        let ui = Ui::new(&display, &viewport);

        let mut imgui = imgui::ImGui::init();
        imgui.set_ini_filename(None);
        let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        let accumulator = FpsAccumulator::new();

        RenderContext {
            backend: display,
            events_loop: events_loop,

            background: bg,
            spritemap: sprite,
            primitives: prim,
            ui: ui,

            imgui: imgui,
            imgui_renderer: renderer,

            accumulator: accumulator,
            viewport: viewport,
        }
    }

    pub fn update(&mut self, world: &World) {
        if let Some(text) = debug::pop_text() {
            self.ui.set_text(text);
        }

        self.spritemap.update(world, &self.viewport);
        self.primitives.update(world, &self.viewport);
        self.ui.update(world, &self.viewport);
    }

    pub fn render(&mut self) {
        let mut target = self.backend.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let millis = self.accumulator.millis_since_start();

        self.background
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.spritemap.redraw(&self.backend, millis);
        self.spritemap
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.primitives.redraw(&self.backend, millis);
        self.primitives
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.ui
            .render(&self.backend, &mut target, &self.viewport, millis);

        let size_points = self.backend.gl_window().get_inner_size_points().unwrap();
        let size_pixels = self.backend.gl_window().get_inner_size_pixels().unwrap();;
        let delta = self.accumulator.delta();
        let ui = self.imgui.frame(size_points, size_pixels, delta);

        self.accumulator.fps.map(debug::set_fps);

        debug::run_ui(&ui);

        self.imgui_renderer.render(&mut target, ui).expect("Rendering failed");

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

        self.ui = Ui::new(&self.backend, &self.viewport);
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

    pub fn reload_shaders(&mut self) {
        self.primitives.reload_shaders(&self.backend);
        self.spritemap.reload_shaders(&self.backend);
    }

    pub fn set_mouse(&mut self, mouse_state: &mut MouseState) {
        let scale = self.imgui.display_framebuffer_scale();
        self.imgui.set_mouse_pos(
            mouse_state.pos.0 as f32 / scale.0,
            mouse_state.pos.1 as f32 / scale.1,
        );
        self.imgui.set_mouse_down(
            &[
                mouse_state.pressed.0,
                mouse_state.pressed.1,
                mouse_state.pressed.2,
                false,
                false,
            ],
        );
        self.imgui.set_mouse_wheel(mouse_state.wheel / scale.1);
        mouse_state.wheel = 0.0;
    }

    pub fn delta(&self) -> f32 {
        self.accumulator.delta()
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
    delta: f32,
    pub fps: Option<f32>,
    fixed_time_stamp: Duration,
}

impl FpsAccumulator {
    pub fn new() -> Self {
        FpsAccumulator {
            start: Instant::now(),
            frame_count: 0,
            last_time: 0,
            accumulator: Duration::new(0, 0),
            previous_clock: Instant::now(),
            delta: 0.0,
            fps: None,
            fixed_time_stamp: Duration::new(0, 16666667),
        }
    }

    pub fn step_frame(&mut self) {
        let now = Instant::now();
        let delta = now - self.previous_clock;
        self.accumulator += delta;
        self.delta = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.previous_clock = now;

        while self.accumulator >= self.fixed_time_stamp {
            self.accumulator -= self.fixed_time_stamp;
        }

        let millis = ::util::get_duration_millis(&Instant::now().duration_since(self.start));

        if millis - self.last_time >= 1000 {
            let fps = self.fps();
            println!("{} ms/frame | {} fps | {} delta", self.ms_per_frame(), fps, self.delta);
            self.fps = Some(fps);
            self.frame_count = 0;
            self.last_time += 1000;
        } else {
            self.fps = None;
        }

        self.frame_count += 1;
    }

    fn ms_per_frame(&self) -> f32 {
        1000.0 / self.frame_count as f32
    }

    fn fps(&self) -> f32 {
        1000.0 / self.ms_per_frame()
    }

    pub fn sleep_time(&self) -> Duration {
        Duration::new(0, 16666667) - self.accumulator
    }

    pub fn millis_since_start(&self) -> u64 {
        let duration = Instant::now().duration_since(self.start);
        ::util::get_duration_millis(&duration)
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }
}
