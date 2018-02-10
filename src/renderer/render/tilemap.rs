use glium;
use glium::backend::Facade;

use point::Direction;
use point::Point2d;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Vertex, Viewport};

#[derive(Copy, Clone, Debug)]
struct Instance {
    tile_idx: usize,
    map_coord: [f32; 2],
    tex_offset: [f32; 2],
}

implement_vertex!(Instance, map_coord, tex_offset);

#[derive(Debug)]
struct DrawTile(u32);

pub struct TileMap {
    tiles: Vec<(DrawTile, (f32, f32))>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,

    tile_atlas: TileAtlas,
    valid: bool,
    pub camera: (f32, f32),
}

use point::Direction::*;

impl TileMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_atlas = TileAtlas::from_config(display, "data/sprites.toml");

        let (vertices, indices) = render::make_quad_buffers(display);

        let program = render::load_program(display, "tile.vert", "tile.frag").unwrap();

        let mut tilemap = TileMap {
            tiles: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: Vec::new(),
            program: program,
            tile_atlas: tile_atlas,
            valid: false,
            camera: (0.0, 0.0),
        };

        tilemap.redraw(display, 0);
        tilemap
    }

    fn make_instances<F>(&mut self, display: &F, msecs: u64)
        where F: glium::backend::Facade {

        let mut instances = Vec::new();

        for pass in 0..self.tile_atlas.passes() {
            let data = self.tiles.iter()
                .filter(|&&(ref tile, _)| {
                    let texture_idx = self.tile_atlas.get_tile_texture_idx("tile");
                    texture_idx == pass
                })
                .flat_map(|&(ref tile, c)| {
                    let mut res = Vec::new();
                    let (x, y) = (c.0, c.1);
                    let (tx, ty) = self.tile_atlas.get_texture_offset("tile", tile.0);

                    let tile_idx = self.tile_atlas.get_tile_index("tile");

                    res.push(Instance { tile_idx: tile_idx,
                                        map_coord: [x, y],
                                        tex_offset: [tx, ty]});
                    res
                }).collect::<Vec<Instance>>();
            instances.push(glium::VertexBuffer::dynamic(display, &data).unwrap());
        }

        self.instances = instances;
    }

    fn update_instances(&mut self, msecs: u64) {
    }
}

impl<'a> Renderable for TileMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, time: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window(32);

        for pass in 0..self.tile_atlas.passes() {
            let texture = self.tile_atlas.get_texture(pass);
            let tex_ratio = self.tile_atlas.get_tilemap_tex_ratio(pass);

            let uniforms = uniform! {
                matrix: proj,
                camera: self.camera,
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                rotation: viewport.rot,
                tex_ratio: tex_ratio,
            };

            let instances = &self.instances[pass];

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: Some(scissor),
                .. Default::default()
            };

            target.draw((&self.vertices, instances.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}

use point;

fn make_map(world: &World, viewport: &Viewport) -> Vec<(DrawTile, (f32, f32))> {
    let mut res = Vec::new();
    let (w, h) = world.size();
    for i in 0..w {
        for j in 0..h {
            if let Some(var) = world.tiles.get((i, j)) {
                res.push((DrawTile(*var), (i as f32, j as f32)));
            }
        }
    }
    res
}

use renderer::RenderUpdate;
use world::World;
use ecs::traits::ComponentQuery;
use ecs::components::Appearance;

impl RenderUpdate for TileMap {
    fn should_update(&self, _world: &World) -> bool {
        true
    }

    fn update(&mut self, world: &World, viewport: &Viewport) {
        self.valid = self.tiles.len() == world.tiles.len();
        if !self.valid {
            self.tiles = make_map(world, viewport);
            self.instances.clear();
        }
        let camera = world.camera_pos().unwrap_or(point::zero());
        self.camera = (camera.x, camera.z);
    }

    fn redraw<F: Facade>(&mut self, display: &F, msecs: u64) {
        if self.instances.is_empty() {
            self.make_instances(display, msecs);
        }
    }
}
