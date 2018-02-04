use glium;
use glium::backend::Facade;

use point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Viewport, Vertex3};
use renderer::render::viewport::FACTOR;

#[derive(Copy, Clone, Debug)]
struct Instance {
    offset: [f32; 3],
    scale: [f32; 3],
}

implement_vertex!(Instance, offset, scale);

pub struct Primitives {
    prims: Vec<DrawPrimitive>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex3>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,
}

struct DrawPrimitive {
    pos: (f32, f32, f32),
    scale: (f32, f32, f32),
}

use rand::{self, Rng};
impl Primitives {
    pub fn new<F: Facade>(display: &F) -> Self {
        let (vertices, indices) = render::make_cube_buffers(display);

        let program = render::load_program(display, "cube.vert", "cube.frag").unwrap();

        let mut primitives = Primitives {
            prims: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: Vec::new(),
            program: program,
        };

        primitives.redraw(display, 0);
        primitives
    }

    pub fn reload_shaders<F: Facade>(&mut self, display: &F) {
        match render::load_program(display, "cube.vert", "cube.frag") {
            Ok(program) => self.program = program,
            Err(e) => println!("Shader error: {:?}", e),
        }
    }

    fn make_instances<F>(&mut self, display: &F)
        where F: glium::backend::Facade {

        let mut instances = Vec::new();

        let mut v = Vec::new();
        for p in self.prims.iter() {
            v.push(Instance {
                offset: [p.pos.0, p.pos.2, p.pos.1],
                scale: [p.scale.0, p.scale.2, p.scale.1],
            });
        }
        instances.push(glium::VertexBuffer::dynamic(display, &v).unwrap());

        self.instances = instances;
    }
}

impl<'a> Renderable for Primitives {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, time: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window(32);

        let uniforms = uniform! {
            matrix: proj,
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
        let mut prims = Vec::new();
        for entity in world.entities() {
            if world.ecs().physics.has(*entity) {
                let pos = world.ecs().positions.get_or_err(*entity);
                prims.push(DrawPrimitive {
                    pos: (pos.x, pos.y, pos.z),
                    scale: (1.0, 1.0, 1.0),
                });
            }
        }

        prims.push(DrawPrimitive {
            pos: (0.0, -0.5, 0.0),
            scale: (64.0, 0.5, 64.0),
        });
        self.prims = prims;
    }

    fn redraw<F: Facade>(&mut self, display: &F, _msecs: u64) {
        self.make_instances(display);
    }
}
