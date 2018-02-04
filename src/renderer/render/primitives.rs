use glium;
use glium::backend::Facade;

use point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Viewport, Vertex3};
use renderer::render::viewport::FACTOR;

#[derive(Copy, Clone, Debug)]
struct Instance {
    offset: [i32; 3],
    scale: [f32; 3],
}

implement_vertex!(Instance, offset, scale);

pub struct Primitives {
    pos: (i32, i32),
    sprites: Vec<(DrawPrimitive, (i32, i32))>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex3>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,
}

struct DrawPrimitive {
    i: i32,
}

use rand::{self, Rng};
impl Primitives {
    pub fn new<F: Facade>(display: &F) -> Self {
        let (vertices, indices) = render::make_cube_buffers(display);

        let program = render::load_program(display, "cube.vert", "cube.frag").unwrap();

        let mut primitives = Primitives {
            pos: (0, 0),
            sprites: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: Vec::new(),
            program: program,
        };

        primitives.redraw(display, 0);
        primitives
    }

    fn make_instances<F>(&mut self, display: &F)
        where F: glium::backend::Facade {

        let mut instances = Vec::new();

        let mut v = Vec::new();
        v.push(Instance {
            offset: [self.pos.0, 0, self.pos.1],
            scale: [32.0, 32.0, 32.0]
        });
        instances.push(glium::VertexBuffer::dynamic(display, &v).unwrap());

        self.instances = instances;
    }
}

impl<'a> Renderable for Primitives {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, time: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        let uniforms = uniform! {
            matrix: viewport.make_projection_matrix((0, 0)),
            time: time as u32,
        };

        let params = glium::DrawParameters {
            scissor: Some(scissor),
            .. Default::default()
        };

        for pass in self.instances.iter() {
            target.draw((&self.vertices, pass.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}

use renderer::RenderUpdate;
use world::World;
use ecs::traits::ComponentQuery;
use ecs::components::Appearance;
use point::Direction;

impl RenderUpdate for Primitives {
    fn should_update(&self, _world: &World) -> bool {
        true
    }

    fn update(&mut self, world: &World, viewport: &Viewport) {
        let camera = world.camera_pos().unwrap_or(point::zero());
        let start_corner = viewport.camera((camera.x, camera.z));
        let pos = ((camera.x * FACTOR) as i32, (camera.z * FACTOR) as i32);

        self.pos = pos;
    }

    fn redraw<F: Facade>(&mut self, display: &F, _msecs: u64) {
        self.make_instances(display);
    }
}
