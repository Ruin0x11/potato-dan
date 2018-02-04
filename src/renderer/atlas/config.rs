use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};

use bincode;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use glium::backend::Facade;
use glob;
use image;
use toml::Value;

use renderer::atlas::*;
use util;

#[derive(Serialize, Deserialize)]
pub struct TileAtlasConfig {
    pub locations: HashMap<String, String>,
    pub frames: HashMap<String, AtlasFrame>,
    pub file_hash: String,
}

pub fn get_config_cache_path(config_name: &str) -> PathBuf {
    let cache_filepath_str = format!("data/.packed/{}", config_name);
    PathBuf::from(&cache_filepath_str)
}

pub fn get_config_cache_file(config_name: &str) -> PathBuf {
    let mut path = get_config_cache_path(config_name);
    path.push("cache.bin");
    path
}

pub fn load_tile_atlas_config(config_name: &str) -> TileAtlasConfig {
    let path = get_config_cache_file(config_name);

    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    bincode::deserialize(buf.as_slice()).unwrap()
}

pub fn write_tile_atlas_config(config: &TileAtlasConfig, config_name: &str) {
    let path = get_config_cache_file(config_name);

    let data = bincode::serialize(config, bincode::Infinite).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(data.as_slice()).unwrap();
}

fn hash_str(s: &str) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(s);
    hasher.result_str()
}

impl TileAtlas {
    pub fn from_config<F: Facade>(display: &F, filename: &str) -> Self {
        let toml_str = util::toml::toml_string_from_file(filename);

        let packed_folder = Path::new(filename).file_stem().unwrap().to_str().unwrap();
        let cache_filepath = get_config_cache_file(packed_folder);

        if !Path::exists(cache_filepath.as_path()) {
            return TileAtlas::build_from_toml(display, packed_folder, &toml_str);
        }

        // check if tile definitions were changed and only repack textures if
        // so, saving startup time.

        let cached_config = load_tile_atlas_config(packed_folder);

        let hash = hash_str(&toml_str);

        if cached_config.file_hash != hash {
            return TileAtlas::build_from_toml(display, packed_folder, &toml_str);
        }

        println!("Using cached tile atlas config at {}", cache_filepath.display());

        let mut textures = Vec::new();

        let cached_texture_path = format!("{}/*.png", get_config_cache_path(packed_folder).display());

        for entry in glob::glob(&cached_texture_path).unwrap() {
            if let Ok(path) = entry {
                let image = image::open(&path).unwrap();
                let texture = make_texture(display, image);
                textures.push(texture);
            }
        }

        assert!(!textures.is_empty(), "Cached textures weren't loaded!");

        TileAtlas::new(cached_config, textures)
    }

    fn build_from_toml<F: Facade>(display: &F, packed_folder: &str, toml_str: &str) -> Self {
        println!("Rebuilding tile atlas config \"{}\"", packed_folder);

        let val = util::toml::toml_value_from_string(toml_str);

        let mut builder = TileAtlasBuilder::new();

        let maps = match util::toml::expect_value_in_table(&val, "maps") {
            Value::Array(array) => array,
            _                   => panic!("Atlas config array wasn't an array."),
        };

        for map in maps.iter() {
            let name: String = util::toml::expect_value_in_table(map, "name");
            let file_path = format!("data/texture/{}", name);
            println!("Load: {}", file_path);
            builder.add_frame(&file_path);
        }

        let tiles = match util::toml::expect_value_in_table(&val, "tiles") {
            Value::Array(array) => array,
            _                   => panic!("Atlas config array wasn't an array."),
        };

        for tile in tiles.iter() {
            let name: String = util::toml::expect_value_in_table(tile, "name");
            let map: String = util::toml::expect_value_in_table(tile, "map");

            let offset: [u32; 2] = util::toml::expect_value_in_table(tile, "offset");
            let count: [u32; 2] = util::toml::expect_value_in_table(tile, "count");
            let size: [u32; 2] = util::toml::expect_value_in_table(tile, "size");

            let tile = AtlasTileData {
                offset: (offset[0], offset[1]),
                count: (count[0], count[1]),
                size: (size[0], size[1]),
            };

            let file_path = format!("data/texture/{}", map);
            builder.add_tile(&file_path, name, tile);
        }

        let hash = hash_str(toml_str);

        builder.file_hash = hash;

        builder.build(display, packed_folder)
    }
}
