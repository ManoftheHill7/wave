use crate::terrain::{Block, Chunk, ChunkCoord, CHUNK_SIZE};
use noise::{NoiseFn, Perlin};

const SEA_LEVEL: i32 = 0;
const SEA_FLOOR: i32 = 30;
const BEACH_HEIGHT: i32 = -10;

pub enum Generator {
    Procedural(TerrainGenerator),
    Map(MapGenerator),
}

impl Generator {
    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        match self {
            Generator::Procedural(gen) => gen.generate_chunk(coord),
            Generator::Map(gen) => gen.generate_chunk(coord),
        }
    }
}

pub struct MapGenerator {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
}

impl MapGenerator {
    pub fn from_image(path: &str) -> Result<Self, String> {
        use raylib::prelude::*;
        let mut image =
            Image::load_image(path).map_err(|e| format!("Failed to load image: {}", e))?;

        let width = image.width;
        let height = image.height;

        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let color = image.get_color(x, y);
                pixels.push(color.r);
                pixels.push(color.g);
                pixels.push(color.b);
            }
        }

        Ok(MapGenerator {
            pixels,
            width,
            height,
        })
    }

    pub fn map_width(&self) -> i32 {
        self.width
    }

    pub fn map_height(&self) -> i32 {
        self.height
    }

    fn get_block_at(&self, x: i32, y: i32) -> Block {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Block::Stone;
        }

        let idx = ((y * self.width + x) * 3) as usize;
        let r = self.pixels[idx];
        let g = self.pixels[idx + 1];
        let b = self.pixels[idx + 2];

        if r == 0 && g == 0 && b == 0 {
            Block::Stone
        } else if r == 127 && g == 127 && b == 127 {
            if self.get_block_at(x, y - 1).is_solid() {
                Block::Stalactite
            } else {
                Block::Stalagmite
            }
        } else if r == 255 && g == 255 && b == 255 {
            Block::Air
        } else if r == 0 && g == 149 && b == 199 {
            Block::Tide
        } else {
            panic!("Unknown color in map: (R: {}, G: {}, B {})", r, g, b);
        }
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size = CHUNK_SIZE as i32;

        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                let wx = coord.x * chunk_size + lx as i32;
                let wy = coord.y * chunk_size + ly as i32;
                let block = self.get_block_at(wx, wy);
                chunk.set(lx, ly, block);
            }
        }

        add_ore_vein(&mut chunk, 30, 15, Block::Coal);

        chunk
    }
}

pub fn add_ore_vein(chunk: &mut Chunk, x: i32, y: i32, ore: Block) {
    // Algorithm
    // Generate vein size: random number between 2 - 6
    // Find nearest tile on border of Block::Air not Block::Air
    // Set x, y to this tile
    // For each vein size.
    //   Replace the nearest Stone block in a random direction with the ore type
    //   Set new x, y to this tile
}

pub struct TerrainGenerator {
    seed: u64,
    noise: Perlin,
}

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self {
        let noise = Perlin::new(seed as u32);
        TerrainGenerator { seed, noise }
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        return if coord.x < 0 {
            self.generate_ocean_chunk(coord)
        } else if coord.x == 0 && (coord.y == -1 || coord.y == 0) {
            self.generate_beach_chunk(coord)
        } else if coord.y == -1 {
            self.generate_hill_chunk(coord)
        } else if coord.y < 0 {
            self.generate_sky_chunk(coord)
        } else {
            self.generate_tunnels_chunk(coord)
        };
    }

    pub fn generate_hill_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE.try_into().unwrap();
        let noise_detail = 100;
        let noise_height = 10;

        // Starts to show flaws if x>14000
        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;
            let height_f =
                noise_height as f64 * self.noise.get([wx as f64 * (1.0 / noise_detail as f64)]);
            let height = height_f as i32 - coord.x + BEACH_HEIGHT;
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;

                let block_type = if wy == height {
                    Block::Grass
                } else if wy > height {
                    Block::Dirt
                } else {
                    Block::Air
                };

                chunk.set(lx, ly, block_type);
            }
        }
        // self.add_tree(&mut chunk, 15, 1, 10, 20, coord);
        // self.add_tree(&mut chunk, -10, 1, 10, 60, coord);
        chunk
    }

    pub fn generate_tunnels_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE.try_into().unwrap();
        let noise_detail = 8.0;
        let air_percent = 0.6;
        let water_spawn_percent = 0.05;
        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;
                let nv = (1.0
                    + self
                        .noise
                        .get([wx as f64 / noise_detail, wy as f64 / noise_detail]))
                    / 2.0;
                let block_type = if air_percent > nv {
                    if water_spawn_percent > nv {
                        Block::Tide
                    } else {
                        Block::Air
                    }
                } else {
                    Block::Stone
                };
                chunk.set(lx, ly, block_type);
            }
        }

        return chunk;
    }

    pub fn generate_ocean_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE.try_into().unwrap();

        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;
                let block_type = if wy > SEA_FLOOR {
                    Block::Sand
                } else if wy > SEA_LEVEL {
                    Block::Water
                } else {
                    Block::Air
                };

                chunk.set(lx, ly, block_type);
            }
        }
        chunk
    }

    pub fn generate_sky_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                chunk.set(lx, ly, Block::Air);
            }
        }
        chunk
    }

    pub fn generate_beach_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE as i32;

        let inflection_x = 32;
        let little_slope = -0.25;
        let big_slope = -2.0;
        let offset_right = BEACH_HEIGHT as f32 - (chunk_size - inflection_x) as f32 * little_slope;
        let offset_left = offset_right - big_slope * inflection_x as f32;

        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;
                let block_type = if wy > SEA_FLOOR {
                    Block::Sand
                } else if wx <= inflection_x {
                    if ((offset_left + wx as f32 * big_slope) as i32) < wy {
                        Block::Sand
                    } else {
                        if wy > SEA_LEVEL {
                            Block::Water
                        } else {
                            Block::Air
                        }
                    }
                } else {
                    if ((offset_right + (wx - inflection_x) as f32 * little_slope) as i32) < wy {
                        Block::Sand
                    } else {
                        if wy > SEA_LEVEL {
                            Block::Water
                        } else {
                            Block::Air
                        }
                    }
                };

                chunk.set(lx, ly, block_type);
            }
        }
        chunk
    }

    fn add_tree(
        &self,
        chunk: &mut Chunk,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        chunk_coord: ChunkCoord,
    ) {
        if height < 2 || width < 1 {
            return;
        }

        let trunk_height = (height as f32 * 0.6).ceil() as i32;
        let canopy_height = height - trunk_height;

        // Trunk
        for i in 0..trunk_height {
            let trunk_y = y - i;

            if self.is_in_chunk(x, trunk_y, chunk_coord) {
                let (lx, ly) = self.world_to_local(x, trunk_y, chunk_coord);
                chunk.set(lx, ly, Block::Log);
            }
        }

        // Leaves
        let canopy_base_y = y - trunk_height;
        for layer in 0..canopy_height {
            let canopy_y = canopy_base_y - layer;

            let layer_ratio = 1.0 - (layer as f32 / canopy_height as f32) * 0.5;
            let layer_width = ((width as f32 * layer_ratio).ceil() as i32).max(1);

            let layer_width = if layer_width % 2 == 0 {
                layer_width + 1
            } else {
                layer_width
            };
            let half_width = layer_width / 2;

            for dx in -half_width..=half_width {
                let leaf_x = x + dx;

                if self.is_in_chunk(leaf_x, canopy_y, chunk_coord) {
                    let (lx, ly) = self.world_to_local(leaf_x, canopy_y, chunk_coord);
                    chunk.set(lx, ly, Block::Leaf);
                }
            }
        }
    }

    fn is_in_chunk(&self, wx: i32, wy: i32, chunk_coord: ChunkCoord) -> bool {
        let chunk_size = CHUNK_SIZE as i32;
        let chunk_start_x = chunk_coord.x * chunk_size;
        let chunk_end_x = chunk_start_x + chunk_size;
        let chunk_start_y = chunk_coord.y * chunk_size;
        let chunk_end_y = chunk_start_y + chunk_size;

        wx >= chunk_start_x && wx < chunk_end_x && wy >= chunk_start_y && wy < chunk_end_y
    }

    fn world_to_local(&self, wx: i32, wy: i32, chunk_coord: ChunkCoord) -> (usize, usize) {
        let chunk_size = CHUNK_SIZE as i32;
        let lx = (wx - chunk_coord.x * chunk_size) as usize;
        let ly = (wy - chunk_coord.y * chunk_size) as usize;
        (lx, ly)
    }
}
