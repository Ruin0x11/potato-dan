use glium;
use glium::backend::Facade;

use point::Direction;
use point::Point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Vertex, Viewport};

#[derive(Copy, Clone, Debug)]
struct Instance {
    tile_idx: usize,
    map_coord: [i32; 2],
    tex_offset: [f32; 2],
}

implement_vertex!(Instance, map_coord, tex_offset);

#[derive(Debug)]
struct DrawTile {
    kind: &'static str,
    edges: u8,
}

pub struct TileMap {
    tiles: Vec<(DrawTile, Point)>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,

    tile_atlas: TileAtlas,
    valid: bool,
}

use point::Direction::*;

impl TileMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_atlas = TileAtlas::from_config(display, "data/tiles.toml");

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
                    let texture_idx = self.tile_atlas.get_tile_texture_idx(tile.kind);
                    texture_idx == pass
                })
                .flat_map(|&(ref tile, c)| {
                    let mut res = Vec::new();
                    let (x, y) = (c.x, c.y);
                    let (tx, ty) = self.tile_atlas.get_texture_offset(tile.kind, msecs);

                    let tile_idx = self.tile_atlas.get_tile_index(tile.kind);

                    res.push(Instance { tile_idx: tile_idx,
                                        map_coord: [x, y],
                                        tex_offset: [tx, ty]});
                    res
                }).collect::<Vec<Instance>>();
            instances.push(glium::VertexBuffer::dynamic(display, &data).unwrap());
        }

        self.instances = instances;
    }

    fn update_instances(&mut self, msecs:u64) {
        for buffer in self.instances.iter_mut() {
            for instance in buffer.map().iter_mut() {
                let (tx, ty) = self.tile_atlas.get_texture_offset_indexed(instance.tile_idx, msecs);

                instance.tex_offset = [tx, ty];
            }
        }
    }
}

impl<'a> Renderable for TileMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        for pass in 0..self.tile_atlas.passes() {
            let texture = self.tile_atlas.get_texture(pass);
            let tex_ratio = self.tile_atlas.get_tilemap_tex_ratio(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
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

use GameContext;

fn make_map(context: &GameContext, viewport: &Viewport) -> Vec<(DrawTile, Point)> {
    let mut res = Vec::new();
    res
}


impl RenderUpdate for TileMap {
    fn should_update(&self, _context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, viewport: &Viewport) {
        self.tiles = make_map(context, viewport);
        self.valid = false;
    }

    fn redraw<F: Facade>(&mut self, display: &F, msecs: u64) {
        if !self.valid {
            self.make_instances(display, msecs);
            self.valid = true;
        } else {
            self.update_instances(msecs);
        }
    }
}
