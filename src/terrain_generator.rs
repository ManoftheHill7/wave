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

        let a = coord.x.abs();
        let b = coord.y.abs();
        let idx = (a + b) * (a + b + 1) / 2 + a;
        for i in 0..3 {
            let (mut x, mut y) = halton_2d(i + idx as u32);
            add_ore_vein(&mut chunk, x as i32, y as i32, Block::Coal);
        }
        for i in 4..6 {
            let (mut x, mut y) = halton_2d(i + idx as u32);
            add_ore_vein(&mut chunk, x as i32, y as i32, Block::Copper);
        }


        chunk
    }
}

pub fn halton(index: u32, base: u32) -> f64 {
    let mut result = 0.0;
    let mut f = 1.0 / base as f64;
    let mut i = index;

    while i > 0 {
        result += f * (i % base) as f64;
        i /= base;
        f /= base as f64;
    }

    result
}
pub fn halton_2d(index: u32) -> (usize, usize) {
    let base_a = 2;
    let base_b = 3;
    (
        (halton(index, base_a) * CHUNK_SIZE as f64) as usize,
        (halton(index, base_b) * CHUNK_SIZE as f64) as usize,
    )
}

/// Helper function to check if a block position is adjacent to air
fn is_adjacent_to_air(chunk: &Chunk, x: i32, y: i32) -> bool {
    (x > 0 && chunk.get((x - 1) as usize, y as usize) == Block::Air)
        || (x < CHUNK_SIZE as i32 - 1 && chunk.get((x + 1) as usize, y as usize) == Block::Air)
        || (y > 0 && chunk.get(x as usize, (y - 1) as usize) == Block::Air)
        || (y < CHUNK_SIZE as i32 - 1 && chunk.get(x as usize, (y + 1) as usize) == Block::Air)
}

pub fn add_ore_vein(chunk: &mut Chunk, x: i32, y: i32, ore: Block) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    // Create a seeded RNG based on chunk position and ore position
    let seed = ((chunk.coord.x as u64) << 32)
        | ((chunk.coord.y as u64) << 16)
        | ((x as u64) << 8)
        | (y as u64);
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate vein size: random number between 2 - 6
    let vein_size = rng.gen_range(2..=6);

    // Find nearest tile on border of Block::Air and not Block::Air
    let mut current_x = x;
    let mut current_y = y;

    // Check if starting position is valid (in bounds)
    if current_x < 0
        || current_x >= CHUNK_SIZE as i32
        || current_y < 0
        || current_y >= CHUNK_SIZE as i32
    {
        return;
    }

    let current_block = chunk.get(current_x as usize, current_y as usize);

    // Determine if we need to search for a stone block adjacent to air
    let needs_search = match current_block {
        Block::Air => true, // Need to find stone
        Block::Stone => !is_adjacent_to_air(chunk, current_x, current_y), // Need better position if not adjacent to air
        _ => true, // For any other block type, search for valid position
    };

    if needs_search {
        // Search in expanding radius for a solid block adjacent to air
        let mut found = false;
        'outer: for radius in 1..10 {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    let check_x = (current_x + dx).max(0).min(CHUNK_SIZE as i32 - 1);
                    let check_y = (current_y + dy).max(0).min(CHUNK_SIZE as i32 - 1);

                    let block = chunk.get(check_x as usize, check_y as usize);
                    if block == Block::Stone && is_adjacent_to_air(chunk, check_x, check_y) {
                        current_x = check_x;
                        current_y = check_y;
                        found = true;
                        break 'outer;
                    }
                }
            }
        }

        if !found {
            return; // No suitable starting position found
        }
    }

    // For each vein size, do a "drunk walk" to place ore
    for _ in 0..vein_size {
        // Replace current block if it's stone
        if current_x >= 0
            && current_x < CHUNK_SIZE as i32
            && current_y >= 0
            && current_y < CHUNK_SIZE as i32
        {
            let block = chunk.get(current_x as usize, current_y as usize);
            if block == Block::Stone {
                chunk.set(current_x as usize, current_y as usize, ore);
            }
        }

        // Move in a random direction to find next stone block
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut attempts = 0;
        let max_attempts = 8;

        while attempts < max_attempts {
            let dir = directions[rng.gen_range(0..directions.len())];
            let new_x = current_x + dir.0;
            let new_y = current_y + dir.1;

            // Check bounds
            if new_x >= 0 && new_x < CHUNK_SIZE as i32 && new_y >= 0 && new_y < CHUNK_SIZE as i32 {
                let block = chunk.get(new_x as usize, new_y as usize);
                if block == Block::Stone || block == ore {
                    current_x = new_x;
                    current_y = new_y;
                    break;
                }
            }

            attempts += 1;
        }

        // If we couldn't find a valid direction, stop the vein
        if attempts >= max_attempts {
            break;
        }
    }
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
