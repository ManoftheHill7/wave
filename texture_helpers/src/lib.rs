use raylib::prelude::*;

pub struct J11TileSet {
    texture: Texture2D,
    tile_size: i32,
}

pub struct Neighbors {
    pub up: bool,
    pub up_right: bool,
    pub right: bool,
    pub down_right: bool,
    pub down: bool,
    pub down_left: bool,
    pub left: bool,
    pub up_left: bool,
}

impl J11TileSet {
    pub fn new(texture: Texture2D) -> Self {
        J11TileSet {
            texture,
            tile_size: 8,
        }
    }

    #[rustfmt::skip]
    pub fn get_tile_rect(&self, x: i32, y: i32, neighbors: &Neighbors) -> Rectangle {
        let hash = ((x + y) * (x + y + 1) / 2 + x).abs() as usize; // Cantor pairing
        let v0000 = [(0,0), (0,1), (1,0), (1,1)];
        let v0001 = [(0, 4),(0, 5),(0, 6),(0, 7),(0, 8),(0, 9),(0, 10)];
        let v0010 = [(2, 4),(2, 5),(2, 6),(2, 7),(2, 8),(2, 9),(2, 10)];
        let v0011 = [(1, 4),(1, 5),(1, 6),(1, 7),(1, 8),(1, 9),(1, 10)];
        let v0100 = [(4, 0),(5, 0),(6, 0),(7, 0),(8, 0),(9, 0),(10, 0)];
        let v0101 = [(3, 3), (4, 3), (5, 3), (4, 4), (3, 4), (3, 5)]; // top left
        let v0110 = [(8, 3), (9, 3), (10, 3), (9, 4), (10, 4), (10, 5)]; // top right
        let v0111 = [(6,3), (7,3)]; // top air
        let v1000 = [(4, 2),(5, 2),(6, 2),(7, 2),(8, 2),(9, 2),(10, 2)];
        let v1001 = [(3, 10), (4, 10), (5, 10), (4, 9), (3, 9), (3, 8)]; // bottom left
        let v1010 = [(8, 10), (9, 10), (10, 10), (9, 9), (10, 9), (10, 8)]; // bottom right
        let v1011 = [(6,10), (7,10)]; // down air
        let v1100 = [(4, 1),(5, 1),(6, 1),(7, 1),(8, 1),(9, 1),(10, 1)];
        let v1101 = [(3,6), (3,7)]; // left air
        let v1110 = [(10,6), (10,7)]; // right air
        let v1111 = [(5,5),(5,6),(5,7),(5,8),(6,5),(6,6),(6,7),(6,8),(7,5),(7,6),(7,7),(7,8),(8,5),(8,6),(8,7),(8,8),(6,6),(6,7),(7,6),(7,7),(6,6),(6,7),(7,6),(7,7)]; // middle

        let loc = match (neighbors.up, neighbors.down, neighbors.left, neighbors.right, neighbors.up_left, neighbors.up_right, neighbors.down_left, neighbors.down_right) {
            (true, true, true, true, true, false, false, true) => (2, 3),
            (true, true, true, true, false, true, true, false) => (3, 2),
            (true, true, true, true, false, false, false, false) => (2, 2),

            // One corner air
            (true, true, true, true, true, true, true, false) => (0, 2),
            (true, true, true, true, true, true, false, true) => (1, 2),
            (true, true, true, true, true, false, true, true) => (0, 3),
            (true, true, true, true, false, true, true, true) => (1, 3),

            // Three corner air
            (true, true, true, true, false, false, false, true) => (3, 1),
            (true, true, true, true, false, false, true, false) => (2, 1),
            (true, true, true, true, false, true, false, false) => (3, 0),
            (true, true, true, true, true, false, false, false) => (2, 0),

            (false, false, false, false, _, _, _, _) => v0000[hash % v0000.len()],
            (false, false, false, true, _, _, _, _)  => v0001[hash % v0001.len()],
            (false, false, true, false, _, _, _, _)  => v0010[hash % v0010.len()],
            (false, false, true, true, _, _, _, _)   => v0011[hash % v0011.len()],

            (false, true, false, false, _, _, _, _)  => v0100[hash % v0100.len()],
            (false, true, false, true, _, _, _, _)   => v0101[hash % v0101.len()],
            (false, true, true, false, _, _, _, _)   => v0110[hash % v0110.len()],
            (false, true, true, true, _, _, _, _)    => v0111[hash % v0111.len()],

            (true, false, false, false, _, _, _, _)  => v1000[hash % v1000.len()],
            (true, false, false, true, _, _, _, _)   => v1001[hash % v1001.len()],
            (true, false, true, false, _, _, _, _)   => v1010[hash % v1010.len()],
            (true, false, true, true, _, _, _, _)    => v1011[hash % v1011.len()],

            (true, true, false, false, _, _, _, _)   => v1100[hash % v1100.len()],
            (true, true, false, true, _, _, _, _)    => v1101[hash % v1101.len()],
            (true, true, true, false, _, _, _, _)    => v1110[hash % v1110.len()],
            (true, true, true, true, _, _, _, _)     => v1111[hash % v1111.len()],
        };

        Rectangle::new((loc.0 * self.tile_size) as f32, (loc.1 * self.tile_size) as f32, self.tile_size as f32, self.tile_size as f32)
    }

    pub fn texture(&self) -> &Texture2D {
        &self.texture
    }
}
