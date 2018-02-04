use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;

use glium;
use glium::backend::Facade;
use image::{self, DynamicImage, GenericImage, Rgba};
use texture_packer::Rect;
use texture_packer::SkylinePacker;
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::importer::ImageImporter;
use texture_packer::exporter::ImageExporter;

use graphics::color::{self, Color};

mod config;
pub mod font;
pub mod texture_atlas;

use self::config::TileAtlasConfig;

pub type Texture2d = glium::texture::CompressedSrgbTexture2d;

type AnimFrames = u64;
type AnimMillisDelay = u64;
#[derive(Serialize, Deserialize, Clone)]
pub enum TileKind {
    Static,
    Animated(AnimFrames, AnimMillisDelay),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl From<Rect> for AtlasRect {
    fn from(rect: Rect) -> AtlasRect {
        AtlasRect {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
        }
    }
}

pub type AtlasTextureRegion = (f32, f32, f32, f32);

#[derive(Serialize, Deserialize, Clone)]
pub struct AtlasTileData {
    /// Offset of start of sprite chunk
    pub offset: (u32, u32),
    /// Number of sprite variations in terms of width and height
    pub count: (u32, u32),
    /// Size of all sprites in the chunk
    pub size: (u32, u32),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AtlasTile {
    pub data: AtlasTileData,
    pub cached_rect: RefCell<Option<AtlasTextureRegion>>,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AtlasFrame {
    texture_idx: usize,
    pub rect: AtlasRect,
    offsets: HashMap<String, AtlasTile>,
}

impl AtlasFrame {
    pub fn new(texture_idx: usize, rect: Rect) -> Self {
        AtlasFrame {
            texture_idx: texture_idx,
            rect: AtlasRect::from(rect),
            offsets: HashMap::new(),
        }
    }
}

pub type TilePacker<'a> = TexturePacker<'a, DynamicImage, SkylinePacker<Rgba<u8>>>;

pub struct TileAtlas {
    config: TileAtlasConfig,
    textures: Vec<Texture2d>,
    indices: Vec<String>,
}

pub struct TileAtlasBuilder<'a> {
    locations: HashMap<String, String>,
    frames: HashMap<String, AtlasFrame>,
    packers: Vec<TilePacker<'a>>,
    pub file_hash: String,
}

impl<'a> TileAtlasBuilder<'a> {
    pub fn new() -> Self {
        let mut builder = TileAtlasBuilder {
            locations: HashMap::new(),
            frames: HashMap::new(),
            packers: Vec::new(),
            file_hash: String::new(),
        };
        builder.add_packer();
        builder
    }

    pub fn add_tile(&mut self, path_str: &str, index: String, tile_data: AtlasTileData) {
        let key = path_str.to_string();
        assert!(self.frames.contains_key(&path_str.to_string()));

        {
            let frame = self.frames.get_mut(&key).unwrap();
            assert!(!frame.offsets.contains_key(&index));

            let tile = AtlasTile {
                data: tile_data,
                cached_rect: RefCell::new(None),
                color: color::BLACK,
            };
            frame.offsets.insert(index.clone(), tile);

            self.locations.insert(index, key);
        }
    }

    pub fn add_frame(&mut self, path_string: &str) {
        if self.frames.contains_key(path_string) {
            return;
        }

        let path = Path::new(&path_string);
        let texture = ImageImporter::import_from_file(path).unwrap();

        for (idx, packer) in self.packers.iter_mut().enumerate() {
            if packer.can_pack(&texture) {
                packer.pack_own(path_string.to_string(), texture).unwrap();
                let rect = packer.get_frame(path_string).unwrap().frame;
                self.frames
                    .insert(path_string.to_string(), AtlasFrame::new(idx, rect));
                // cannot return self here, since self already borrowed, so
                // cannot use builder pattern.
                return;
            }
        }

        self.add_packer();

        {
            // complains that borrow doesn't last long enough
            // len mut packer = self.newest_packer_mut();

            let packer_idx = self.packers.len() - 1;
            let packer = &mut self.packers[packer_idx];
            println!("{:?}", path_string);
            packer.pack_own(path_string.to_string(), texture).unwrap();

            let rect = packer.get_frame(&path_string).unwrap().frame;
            self.frames
                .insert(path_string.to_string(), AtlasFrame::new(packer_idx, rect));
        }
    }

    fn add_packer(&mut self) {
        let config = TexturePackerConfig {
            max_width: 2048,
            max_height: 2048,
            allow_rotation: false,
            texture_outlines: false,
            trim: false,
            texture_padding: 0,
            ..Default::default()
        };

        self.packers.push(TexturePacker::new_skyline(config));
    }

    pub fn build<F: Facade>(&mut self, display: &F, packed_tex_folder: &str) -> TileAtlas {
        let mut textures = Vec::new();

        // Create cached atlas texture directory
        let packed_folder_path = config::get_config_cache_path(packed_tex_folder);

        if Path::exists(packed_folder_path.as_path()) {
            fs::remove_dir_all(packed_folder_path.as_path()).unwrap();
        }

        fs::create_dir_all(packed_folder_path.as_path()).unwrap();

        let mut images = Vec::new();

        // Save the packed atlas textures to a cached location
        for (idx, packer) in self.packers.iter().enumerate() {
            let packer_image = ImageExporter::export(packer).unwrap();

            let mut file_path = packed_folder_path.clone();
            file_path.push(&format!("{}.png", idx));

            let mut file = File::create(file_path).unwrap();

            packer_image.save(&mut file, image::PNG).unwrap();
            images.push(packer_image.clone());
            textures.push(make_texture(display, packer_image));
        }

        println!("Saved {}", packed_tex_folder);

        // Generate minimap colors for tiles
        // for frame in self.frames.values_mut() {
        //     for offset in frame.offsets.values_mut() {
        //         let image = &images[frame.texture_idx];
        //         let color = color::BLACK;
        //         offset.color = color;
        //     }
        // }

        let config = TileAtlasConfig {
            locations: self.locations.clone(),
            frames: self.frames.clone(),
            file_hash: self.file_hash.clone(),
        };

        config::write_tile_atlas_config(&config, packed_tex_folder);

        TileAtlas::new(config, textures)
    }
}

impl TileAtlas {
    pub fn new(config: TileAtlasConfig, textures: Vec<Texture2d>) -> Self {
        let mut atlas = TileAtlas {
            config: config,
            textures: textures,
            indices: Vec::new(),
        };

        atlas.cache_tile_regions();
        atlas
    }

    /// Precalculates the UV rectangles for individual tiles to avoid the
    /// overhead of recalculating them on lookup. It must be done before the
    /// tile atlas can be used.
    fn cache_tile_regions(&mut self) {
        for frame in self.config.frames.values() {
            let (frame_w, frame_h) = self.frame_size(frame);

            for (_, tile) in frame.offsets.iter() {
                let add_offset = (frame.rect.x, frame.rect.y);

                let tx = (tile.data.offset.0 + add_offset.0) as f32 / frame_w as f32;
                let ty = (tile.data.offset.1 + add_offset.1) as f32 / frame_h as f32;

                let tw = tile.data.size.0 as f32 / frame_w as f32;
                let th = tile.data.size.1 as f32 / frame_h as f32;

                *tile.cached_rect.borrow_mut() = Some((tx, ty, tw, th));
            }
        }

        self.indices = self.config
                           .locations
                           .keys()
                           .map(|l| l.to_string())
                           .collect();
    }

    fn frame_size(&self, frame: &AtlasFrame) -> (u32, u32) {
        self.texture_size(frame.texture_idx)
    }

    fn texture_size(&self, texture_idx: usize) -> (u32, u32) {
        self.textures[texture_idx].dimensions()
    }

    fn get_frame(&self, tile_type: &str) -> &AtlasFrame {
        let tex_name = &self.config.locations[tile_type];
        &self.config.frames[tex_name]
    }

    pub fn get_tile_texture_idx(&self, tile_type: &str) -> usize {
        self.get_frame(tile_type).texture_idx
    }

    pub fn get_tilemap_tex_ratio(&self, texture_idx: usize) -> [f32; 2] {
        let dimensions = self.texture_size(texture_idx);

        let cols: u32 = dimensions.0 / 24;
        let rows: u32 = dimensions.1 / 24;
        [1.0 / cols as f32, 1.0 / rows as f32]
    }

    pub fn get_sprite_tex_ratio(&self, tile_type: &str) -> [f32; 2] {
        let frame = self.get_frame(tile_type);
        let (sx, sy) = frame.offsets[tile_type].data.size;

        let dimensions = self.frame_size(frame);

        let cols: f32 = dimensions.0 as f32 / sx as f32;
        let rows: f32 = dimensions.1 as f32 / sy as f32;
        [1.0 / cols, 1.0 / rows]
    }

    pub fn get_tile_texture_size(&self, tile_type: &str) -> (u32, u32) {
        self.get_frame(tile_type).offsets[tile_type].data.size
    }

    pub fn get_tile(&self, tile_type: &str) -> &AtlasTile {
        let frame = self.get_frame(tile_type);
        &frame.offsets[tile_type]
    }

    pub fn get_texture_offset(&self, tile_type: &str, sprite_idx: u32) -> (f32, f32) {
        let frame = self.get_frame(tile_type);
        let tile = &frame.offsets[tile_type];

        let (mut tx, mut ty, tw, th) =
            tile.cached_rect
                .borrow()
                .expect("Texture atlas regions weren't cached yet.");

        let count = tile.data.count;

        let dx = if sprite_idx == 0 {
            0
        } else {
            sprite_idx % count.0
        };
        let dy = if sprite_idx == 0 {
            0
        } else {
            (sprite_idx / count.0) % count.1
        };

        tx += dx as f32 * tw;
        ty += dy as f32 * (th / 1.0);

        (tx, ty)
    }

    pub fn get_tile_index(&self, tile_kind: &str) -> usize {
        self.indices
            .iter()
            .enumerate()
            .find(|&(_, i)| i == tile_kind)
            .unwrap()
            .0
    }

    fn get_tile_kind_indexed(&self, tile_idx: usize) -> &String {
        &self.indices[tile_idx]
    }

    pub fn get_texture_offset_indexed(&self, tile_idx: usize, variant: u32) -> (f32, f32) {
        let kind = self.get_tile_kind_indexed(tile_idx);
        self.get_texture_offset(kind, variant)
    }

    pub fn get_texture(&self, idx: usize) -> &Texture2d {
        &self.textures[idx]
    }

    pub fn passes(&self) -> usize {
        self.textures.len()
    }
}

/// Returns the number of tiles by which an atlas frame is offset. Note that this could be a
/// fractional value if the tile sizes differ.
fn get_add_offset(rect: &AtlasRect, tile_size: &(u32, u32)) -> (f32, f32) {
    let cols = rect.x as f32 / tile_size.0 as f32;
    let rows = rect.y as f32 / tile_size.1 as f32;
    (cols, rows)
}

pub fn make_texture<F: Facade>(display: &F, image: DynamicImage) -> Texture2d {
    let dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.to_rgba().into_raw(),
                                                                   dimensions);
    Texture2d::new(display, image).unwrap()
}
