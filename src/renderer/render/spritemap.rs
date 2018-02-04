use glium;
use glium::backend::Facade;

use renderer::atlas::*;
use renderer::render::{self, Renderable, Viewport, Vertex};

#[derive(Copy, Clone, Debug)]
struct Instance {
    tile_idx: usize,
    map_coord: [u32; 2],
    tex_offset: [f32; 2],
    tex_ratio: [f32; 2],
    sprite_size: [u32; 2],
}

implement_vertex!(Instance, map_coord, tex_offset, tex_ratio, sprite_size);

pub struct SpriteMap {
    sprites: Vec<(DrawSprite, (u32, u32))>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,

    tile_atlas: TileAtlas,
    valid: bool,
}

struct DrawSprite {
    kind: String,
    variant: u32
}

impl SpriteMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_atlas = TileAtlas::from_config(display, "data/sprites.toml");

        let (vertices, indices) = render::make_quad_buffers(display);

        let program = render::load_program(display, "sprite.vert", "sprite.frag").unwrap();

        let mut spritemap = SpriteMap {
            sprites: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: Vec::new(),
            program: program,
            tile_atlas: tile_atlas,
            valid: false,
        };

        spritemap.redraw(display, 0);
        spritemap
    }

    fn make_instances<F>(&mut self, display: &F)
        where F: glium::backend::Facade {

        let mut instances = Vec::new();

        for pass in 0..self.tile_atlas.passes() {
            let data = self.sprites.iter()
                .filter(|&&(ref sprite, _)| {
                    let texture_idx = self.tile_atlas.get_tile_texture_idx(&sprite.kind);
                    texture_idx == pass
                })
                .map(|&(ref sprite, pos)| {
                    let (x, y) = pos;

                    let (tx, ty) = self.tile_atlas.get_texture_offset(&sprite.kind, sprite.variant);
                    let (sx, sy) = self.tile_atlas.get_tile_texture_size(&sprite.kind);
                    let tex_ratio = self.tile_atlas.get_sprite_tex_ratio(&sprite.kind);

                    // To store the tile kind without putting a string in the
                    // index vertex, a mapping from a numeric index to a string
                    // is used in the tile atlas. Then, when the tile kind needs
                    // to be checked, the numeric index can retrieve a tile kind.
                    let tile_idx = self.tile_atlas.get_tile_index(&sprite.kind);

                    Instance { tile_idx: tile_idx,
                               map_coord: [x, y],
                               tex_offset: [tx, ty],
                               tex_ratio: tex_ratio,
                               sprite_size: [sx, sy], }
                }).collect::<Vec<Instance>>();
            instances.push(glium::VertexBuffer::dynamic(display, &data).unwrap());
        }

        self.instances = instances;
    }
}

impl<'a> Renderable for SpriteMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, _time: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        for pass in 0..self.tile_atlas.passes() {
            let texture = self.tile_atlas.get_texture(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [1u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: Some(scissor),
                .. Default::default()
            };

            let instances = &self.instances[pass];

            target.draw((&self.vertices, instances.per_instance().unwrap()),
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

fn make_sprites(world: &World, _viewport: &Viewport) -> Vec<(DrawSprite, (u32, u32))> {
    let mut res = Vec::new();

    {
        let mut push_sprite = |variant: u32, pos: (u32, u32), kind: &str| {
            let sprite = DrawSprite { kind: kind.to_string(), variant: variant };
            res.push((sprite, pos));
        };

        for entity in world.entities() {
            if world.ecs().positions.has(*entity) &&
                world.ecs().appearances.has(*entity) {
                    let pos = world.ecs().positions.get_or_err(*entity);
                    //let app = world.ecs().appearances.get_or_err(*entity);

                    let variant = pos.direction.ordinal() as u32 * 10;
                    push_sprite(variant, ((pos.pos.x * 10.0) as u32, (pos.pos.y * 10.0) as u32), "chara");

                    let mut variant = pos.direction.ordinal() as u32 * 7;

                    // TODO: move to movement logic
                    if pos.movement_frames != 0 {
                        variant += ((pos.movement_frames / 5) % 6) + 1;
                    }
                    push_sprite(variant, ((pos.pos.x * 10.0) as u32, (pos.pos.y * 10.0) as u32 + 22), "feet");
                }
        }
    }

    res
}

impl RenderUpdate for SpriteMap {
    fn should_update(&self, _world: &World) -> bool {
        true
    }

    fn update(&mut self, world: &World, viewport: &Viewport) {
        self.sprites = make_sprites(world, viewport);
        self.valid = false;
    }

    fn redraw<F: Facade>(&mut self, display: &F, _msecs: u64) {
        if !self.valid {
            self.make_instances(display);
            self.valid = true;
        } else {
            //self.update_instances(msecs);
        }
    }
}
