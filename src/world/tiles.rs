use point::Point2d;

pub struct Tiles {
    size: (u32, u32),
    tiles: Vec<u32>,
}

impl Tiles {
    pub fn new(size: (u32, u32), default: u32) -> Self {
        let mut tiles = Vec::new();

        for _ in 0..size.0 {
            for _ in 0..size.1 {
                tiles.push(default);
            }
        }

        Tiles {
            size: size,
            tiles: tiles,
        }
    }

    fn index(&self, pos: (u32, u32)) -> usize {
        (pos.0 * self.size.0 + pos.1) as usize
    }

    pub fn get(&self, pos: (u32, u32)) -> Option<&u32> {
        let index = self.index(pos);
        self.tiles.get(index)
    }

    pub fn set(&mut self, pos: (u32, u32), val: u32) {
        let index = self.index(pos);
        self.tiles.get_mut(index).map(|t| *t = val);
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }
}
