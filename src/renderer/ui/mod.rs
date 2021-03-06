use glium;
use glium::glutin;
use glium::backend::Facade;

use renderer::render::{Renderable, Viewport};

pub mod elements;
mod layer;
pub mod layers;
mod renderer;
mod subrenderer;
mod traits;

pub use self::elements::UiElement;
pub use self::renderer::{UiRenderer, UiSubRenderer};
pub use self::layer::{EventResult, UiLayer, UiQuery};
pub use self::traits::*;

use renderer::render::{SCREEN_WIDTH, SCREEN_HEIGHT};
use renderer::ui::elements::{UiBar, UiMessageLog, UiText};

pub struct MainLayer {
    pub text: UiText,
}

impl MainLayer {
    pub fn new(viewport: &Viewport) -> Self {
        let mut text = UiText::new(String::new());
        text.shadow = true;
        MainLayer {
            text: text,
        }
    }
}

impl UiElement for MainLayer {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        {
            let mut sub =
                renderer.sub_renderer((20, 20), (SCREEN_WIDTH * 2 - 40, SCREEN_HEIGHT * 2 - 40));
            self.text.draw(&mut sub);
        }
    }
}

impl UiLayer for MainLayer {
    fn on_event(&mut self, _event: glutin::WindowEvent) -> EventResult {
        EventResult::Ignored
    }
}

pub struct Ui {
    renderer: UiRenderer,
    valid: bool,
    layers: Vec<Box<UiLayer>>,
    pub main_layer: MainLayer,
}

impl Ui {
    pub fn new<F: Facade>(display: &F, viewport: &Viewport) -> Self {
        Ui {
            renderer: UiRenderer::new(display),
            valid: false,
            layers: Vec::new(),
            main_layer: MainLayer::new(viewport),
        }
    }

    pub fn draw_layer<T: 'static + UiLayer>(&mut self, layer: &T) {
        layer.draw(&mut UiSubRenderer::new(&mut self.renderer))
    }

    pub fn push_layer<T: 'static + UiLayer>(&mut self, layer: T) {
        self.layers.push(Box::new(layer));
        self.invalidate();
    }

    pub fn pop_layer(&mut self) {
        self.layers.pop();
        self.invalidate();
    }

    pub fn set_text(&mut self, text: String) {
        self.main_layer.text.text = text
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
        self.valid = true;
    }

    pub fn on_event(&mut self, event: glutin::WindowEvent) {
        let result = match self.layers.last_mut() {
            None => EventResult::Ignored,
            Some(layer) => layer.on_event(event),
        };

        match result {
            EventResult::Ignored => (),
            EventResult::Consumed(callback) => {
                self.invalidate();
                match callback {
                    None => (),
                    Some(cb) => cb(self),
                }
            },
            EventResult::Done | EventResult::Canceled => self.pop_layer(),
        }
    }

    pub fn render_all(&mut self) {
        self.renderer.clear();

        self.main_layer
            .draw(&mut UiSubRenderer::new(&mut self.renderer));

        for layer in self.layers.iter() {
            layer.draw(&mut UiSubRenderer::new(&mut self.renderer));
        }
    }

    fn redraw(&mut self) {
        if !self.valid {
            self.render_all();
            self.valid = true;
        }
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }
}

impl<'a> Renderable for Ui {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, time: u64)
    where
        F: glium::backend::Facade,
        S: glium::Surface,
    {

        self.renderer.render(display, target, viewport, time);
    }
}

use renderer::RenderUpdate;
use world::World;

impl RenderUpdate for Ui {
    fn should_update(&self, _world: &World) -> bool {
        true
    }

    fn update(&mut self, world: &World, _viewport: &Viewport) {
        self.invalidate();
        self.redraw();
    }
}
