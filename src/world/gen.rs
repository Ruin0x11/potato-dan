use ecs::prefab;
use point::*;
use rand::{thread_rng, Rng};
use super::World;

const BLOCK_SIZE: u32 = 32;
const ROAD_WIDTH: u32 = 4;

fn blocks(size: (u32, u32)) -> Vec<(u32, u32)> {
    let mut vec = Vec::new();
    for i in 0..(size.0 / BLOCK_SIZE) {
        for j in 0..(size.1 / BLOCK_SIZE) {
            vec.push((i * BLOCK_SIZE, j * BLOCK_SIZE));
        }
    }

    vec
}

pub fn city(world: &mut World) {
    for block in blocks(world.size()) {
        paint_road(world, block);

        use rand::{thread_rng, Rng};

        let choices = [1, 2, 3];
        let mut rng = thread_rng();
        match rng.choose(&choices) {
            Some(&1) => park(world, block),
            Some(&2) => house(world, block),
            Some(&3) => house(world, block),
            _ => unreachable!(),
        }
    }
}

fn park(world: &mut World, block: (u32, u32)) {
}

fn house(world: &mut World, block: (u32, u32)) {
    let mut rng = thread_rng();
    let sx = rng.gen_range(2, 4);
    let sy = rng.gen_range(2, 4);
    let ex = rng.gen_range(BLOCK_SIZE - 16, BLOCK_SIZE - 8);
    let ey = rng.gen_range(BLOCK_SIZE - 16, BLOCK_SIZE - 8);
    for x in sx..ex {
        for y in sy..ey {
            if x == sx || x == ex-1 || y == sy {
                world.spawn(prefab::wall(), Point::new((block.0 + x) as f32, 0.0, (block.1 + y) as f32));
            }
        }
    }
}

fn paint_road(world: &mut World, block: (u32, u32)) {
    let siz = BLOCK_SIZE - ROAD_WIDTH;
    for x in siz..BLOCK_SIZE {
        for y in 0..BLOCK_SIZE {
            world.tiles.set((block.0 + x, block.1 + y), 2);
        }
    }

    for x in 0..BLOCK_SIZE {
        for y in siz..BLOCK_SIZE {
            world.tiles.set((block.0 + x, block.1 + y), 2);
        }
    }
}
