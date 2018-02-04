use glium;
use glium::backend::Facade;

use point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Viewport, Vertex};
use renderer::render::viewport::FACTOR;

#[derive(Copy, Clone, Debug)]
struct Instance {
    tile_idx: usize,
    map_coord: [i32; 2],
    inner_offset: [i32; 2],
    tex_offset: [f32; 2],
    tex_ratio: [f32; 2],
    sprite_size: [u32; 2],
}

implement_vertex!(Instance, map_coord, inner_offset, tex_offset, tex_ratio, sprite_size);

pub struct SpriteMap {
    sprites: Vec<(DrawSprite, (i32, i32, i32, i32))>,

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

    pub fn reload_shaders<F: Facade>(&mut self, display: &F) {
        match render::load_program(display, "sprite.vert", "sprite.frag") {
            Ok(program) => self.program = program,
            Err(e) => println!("Shader error: {:?}", e),
        }
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
                    let (x, y, ix, iy) = pos;

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
                               inner_offset: [ix, iy],
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
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, time: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window(1);

        for pass in 0..self.tile_atlas.passes() {
            let texture = self.tile_atlas.get_texture(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [1u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                time: time as u32,
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
use ecs::components::Appearance;
use point::Direction;

const TAIL_COUNT: u32 = 10;
const BODY_COUNT: u32 = 10;
const FEET_COUNT: u32 = 7;
const JACKET_COUNT: u32 = 7;
const HAIR_COUNT: u32 = 28;
const EAR_COUNT: u32 = 10;
const FACE_COUNT: u32 = 9;

fn make_sprites(world: &World, viewport: &Viewport) -> Vec<(DrawSprite, (i32, i32, i32, i32))> {
    let mut res = Vec::new();

    {
        for entity in world.entities() {
            if !world.ecs().positions.has(*entity) {
                continue;
            }

            let pos = world.ecs().positions.get_or_err(*entity);
            let screen_x = (pos.x * 32.0) as i32;
            let screen_y = (pos.z * 32.0) as i32;

            let mut push_sprite = |variant: u32, pos: (i32, i32), kind: &str| {
                let sprite = DrawSprite { kind: kind.to_string(), variant: variant };
                res.push((sprite, (screen_x, screen_y, pos.0, pos.1)));
            };

            match world.ecs().appearances.get(*entity) {
                Some(&Appearance::Chara(ref chara)) => {
                    let phys = world.ecs().physics.get_or_err(*entity);
                    let tail_occluded = phys.direction != Direction::N &&
                        phys.direction != Direction::NE &&
                        phys.direction != Direction::NW;
                    let ord = phys.direction.ordinal() as u32;

                    let tail_kind = (chara.tail_kind % TAIL_COUNT) + ord * TAIL_COUNT;
                    let body_kind = (chara.body_kind % BODY_COUNT) + ord * BODY_COUNT;
                    let mut feet_kind = (chara.feet_kind % FEET_COUNT) + ord * FEET_COUNT;
                    let jacket_kind = (chara.jacket_kind % JACKET_COUNT) + ord * JACKET_COUNT;
                    let hair_kind = (chara.hair_kind % HAIR_COUNT) + ord * HAIR_COUNT;
                    let ear_kind = (chara.ear_kind % EAR_COUNT) + ord * EAR_COUNT;
                    let face_kind = (chara.face_kind % FACE_COUNT) + ord * FACE_COUNT;

                    if tail_occluded {
                        //push_sprite(tail_kind, (0,10), "tail");
                    }

                    push_sprite(body_kind, (0, 0), "body");

                    // TODO: move to movement logic
                    if phys.movement_frames != 0 {
                        feet_kind += ((phys.movement_frames / 5) % 6) + 1;
                    }
                    push_sprite(feet_kind, (0, 64), "feet");
                    push_sprite(jacket_kind, (0,  -10), "jacket");
                    push_sprite(hair_kind, (-16, 8), "hair");
                    push_sprite(chara.helmet_kind, (-14, -16), "helmet");
                    push_sprite(ear_kind, (-16, -48), "ears");
                    //push_sprite(face_kind, (0, -8), "face");

                    if !tail_occluded {
                        //push_sprite(tail_kind, (0, 10), "tail");
                    }
                },
                Some(&Appearance::Object(ref object)) => {
                    push_sprite(object.variant, (0, 0), &object.kind);
                },
                _ => (),
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
