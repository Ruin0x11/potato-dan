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
    prims: Vec<Instance>,
    lines: Vec<Vertex3f>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex3>,
    instances: glium::VertexBuffer<Instance>,
    program: glium::Program,
    line_program: glium::Program,

    line_indices: glium::index::NoIndices,
    line_vertices: glium::VertexBuffer<Vertex3f>,
}

#[derive(Clone, Copy)]
struct Vertex3f {
    position: [f32; 3]
}

implement_vertex!(Vertex3f, position);

use rand::{self, Rng};
impl Primitives {
    pub fn new<F: Facade>(display: &F) -> Self {
        let (vertices, indices) = render::make_cube_buffers(display);

        let program = render::load_program(display, "cube.vert", "cube.frag").unwrap();
        let line_program = render::load_program(display, "line.vert", "cube.frag").unwrap();

        let mut primitives = Primitives {
            prims: Vec::new(),
            lines: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: glium::VertexBuffer::dynamic(display, &[]).unwrap(),
            program: program,
            line_program: line_program,

            line_indices: glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
            line_vertices: glium::VertexBuffer::dynamic(display, &[]).unwrap(),
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
        self.line_vertices = glium::VertexBuffer::dynamic(display, &self.lines).unwrap();
        self.instances = glium::VertexBuffer::dynamic(display, &self.prims).unwrap();
    }
}

impl<'a> Renderable for Primitives {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, time: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window(32);

        let uniforms = uniform! {
            matrix: proj,
            time: time as u32,
            rotation: viewport.rot,
        };

        let params = glium::DrawParameters {
            scissor: Some(scissor),
            .. Default::default()
        };

        // cubes
        target.draw((&self.vertices, self.instances.per_instance().unwrap()),
                    &self.indices,
                    &self.program,
                    &uniforms,
                    &params).unwrap();

        // lines
        target.draw(&self.line_vertices,
                    &self.line_indices,
                    &self.line_program,
                    &uniforms,
                    &params).unwrap();
    }
}

use renderer::RenderUpdate;
use world::World;
use ecs::traits::ComponentQuery;
use ecs::components::{Appearance, PhysicsShape};
use point::Direction;

impl RenderUpdate for Primitives {
    fn should_update(&self, _world: &World) -> bool {
        true
    }

    fn update(&mut self, world: &World, viewport: &Viewport) {
        let mut instances = Vec::new();
        let mut verts = Vec::new();
        let camera = world.camera_pos().unwrap_or(point::zero());
        let min = viewport.min_point((camera.x, camera.z), 32);
        for entity in world.entities() {
            if world.ecs().physics.has(*entity) {
                let pos = world.ecs().positions.get_or_err(*entity);

                let scale = match world.ecs().physics.get_or_err(*entity).shape {
                    PhysicsShape::Chara => [1.0, 1.0, 1.0],
                    PhysicsShape::Wall => [1.0, 1.0, 20.0],
                    PhysicsShape::Bullet => [0.3, 0.3, 0.3],
                };

                instances.push(Instance {
                    offset: [pos.pos.x - camera.x + 0.5, pos.pos.z - camera.z + 0.5, pos.pos.y],
                    scale: scale,
                });

                verts.push(Vertex3f { position: [pos.pos.x - camera.x, pos.pos.z - camera.z, pos.pos.y] });
                verts.push(Vertex3f { position: [pos.pos.x - camera.x + 1.0, pos.pos.z - camera.z + 1.0, pos.pos.y] });
            }
        }

        for (pos, blocked) in world.grid.nodes.iter() {
            if *blocked {
                instances.push(Instance {
                    offset: [(pos.x*2) as f32 - camera.x, (pos.y*2) as f32 - camera.z, 0.0],
                    scale: [2.0, 2.0, 1.0],
                });
            }
        }

        self.prims = instances;
        self.lines = verts;
    }

    fn redraw<F: Facade>(&mut self, display: &F, _msecs: u64) {
        self.make_instances(display);
    }
}
