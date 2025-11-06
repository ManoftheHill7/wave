use std::collections::HashMap;
use crate::terrain_generator::TerrainGenerator;

pub const CHUNK_SIZE: usize = 128;
pub const CELL_RESOLUTION: usize = 1;
pub const CELLS_PER_TILE: usize = CELL_RESOLUTION * CELL_RESOLUTION;
pub const CELL_OFFSET: f32 = 1.0 / CELL_RESOLUTION as f32;

// Block types enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Dirt,
    Stone,
    Grass,
    Sand,
    Water,
    Lava,
    Log,
    Leaf,
}

#[derive(Debug, Clone, Copy)]
pub struct LiquidData {
    pub volume: f32, // 0.0 is none, 1.0 is full, more then 1.0 is pressurized
    pub flow_left: bool,
    pub flow_right: bool,
    pub flow_down: bool,
    pub flow_up: bool,
}

impl LiquidData {
    fn new(volume: f32) -> Self {
        LiquidData {
            volume,
            flow_left: false,
            flow_right: false,
            flow_down: false,
            flow_up: false,
        }
    }
}

// Chunk coordinate (not block coordinate)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    blocks: [Block; CHUNK_SIZE * CHUNK_SIZE],
    cells: [LiquidData; CELLS_PER_TILE * CHUNK_SIZE * CHUNK_SIZE],
    pub coord: ChunkCoord,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Chunk {
            blocks: [Block::Air; CHUNK_SIZE * CHUNK_SIZE],
            cells: [LiquidData::new(0.0); CELLS_PER_TILE * CHUNK_SIZE * CHUNK_SIZE],
            coord,
        }
    }

    pub fn get(&self, local_x: usize, local_y: usize) -> Block {
        let index = local_y * CHUNK_SIZE + local_x;
        self.blocks[index]
    }

    pub fn set(&mut self, local_x: usize, local_y: usize, block: Block) {
        let index = local_y * CHUNK_SIZE + local_x;
        if block == Block::Water {
            let w = CHUNK_SIZE ;
            let lx = local_x as f32;
            let ly = local_y as f32;
            for cell_y in 0..CELL_RESOLUTION {
                for cell_x in 0..CELL_RESOLUTION {
                    self.liquid_set(lx as f32 + cell_x as f32 / CELL_RESOLUTION as f32,
                        ly as f32 + cell_y as f32 / CELL_RESOLUTION as f32,
                        LiquidData::new(1.0));
                }
            }
        } else {
            self.blocks[index] = block;
        }
    }

    pub fn liquid_get(&self, local_x: f32, local_y: f32) -> LiquidData {
        let cx = local_x * CELL_RESOLUTION as f32;
        let cy = local_y * CELL_RESOLUTION as f32;
        self.cells[(cy * CHUNK_SIZE as f32 + cx) as usize]
    }

    pub fn liquid_set(&mut self, local_x: f32, local_y: f32, ld: LiquidData) {
        let cx = local_x * CELL_RESOLUTION as f32;
        let cy = local_y * CELL_RESOLUTION as f32;
        self.cells[(cy * CHUNK_SIZE as f32 + cx) as usize] = ld
    }
}

pub struct Terrain {
    pub chunks: HashMap<ChunkCoord, Chunk>,
    chunk_size: i32,
    generator: TerrainGenerator,
}

impl Terrain {
    pub fn new(seed: u64) -> Self {
        Terrain {
            chunks: HashMap::new(),
            chunk_size: CHUNK_SIZE as i32,
            generator: TerrainGenerator::new(seed),
        }
    }

    pub fn flow(&mut self) {
    }

    pub fn at(&self, x: i32, y: i32) -> Block {
        let chunk_coord = self.world_to_chunk(x, y);
        let local_coord = self.world_to_local(x, y);

        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.get(local_coord.0, local_coord.1)
        } else {
            Block::Air
        }
    }

    pub fn liquid_at(&self, x: f32, y: f32) -> LiquidData {
        let xi = x as i32;
        let yi = y as i32;
        let chunk_coord = self.world_to_chunk(xi, yi);
        let local_coord = self.world_to_local(xi, yi);

        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.liquid_get(local_coord.0 as f32 + x.fract(), local_coord.1 as f32 + y.fract())
        } else {
            LiquidData::new(0.0)
        }
    }

    pub fn solid_terrain_at(&self, x: i32, y: i32) -> bool {
        matches!(self.at(x, y),
            Block::Dirt |
            Block::Stone |
            Block::Grass |
            Block::Sand |
            Block::Log |
            Block::Leaf)
    }

    pub fn liquid_terrain_at(&self, x: i32, y: i32) -> bool {
        // matches!(self.at(x, y), Block::Water | Block::Lava)
        self.liquid_at(x as f32, y as f32).volume +
            self.liquid_at(x as f32 + CELL_OFFSET, y as f32).volume +
            self.liquid_at(x as f32, y as f32 + CELL_OFFSET).volume +
            self.liquid_at(x as f32 + CELL_OFFSET, y as f32 + CELL_OFFSET).volume > 0.5
    }

    pub fn collides_with_solid_terrain(&self, x: f32, y: f32, width: f32, height: f32) -> Option<(f32, f32)> {
        let left = x.floor() as i32;
        let right = (x + width).floor() as i32;
        let top = y.floor() as i32;
        let bottom = (y + height).floor() as i32;

        for check_y in top..=bottom {
            for check_x in left..=right {
                if self.solid_terrain_at(check_x, check_y) {
                    return Some((check_x as f32, check_y as f32));
                }
            }
        }

        None
    }

    pub fn set(&mut self, x: i32, y: i32, block: Block) {
        let chunk_coord = self.world_to_chunk(x, y);
        let local_coord = self.world_to_local(x, y);

        if !self.chunks.contains_key(&chunk_coord) {
            self.load_chunk(chunk_coord);
        }

        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            chunk.set(local_coord.0, local_coord.1, block);
        }
    }

    pub fn world_to_chunk(&self, x: i32, y: i32) -> ChunkCoord {
        ChunkCoord {
            x: x.div_euclid(self.chunk_size),
            y: y.div_euclid(self.chunk_size),
        }
    }

    fn world_to_local(&self, x: i32, y: i32) -> (usize, usize) {
        (
            x.rem_euclid(self.chunk_size) as usize,
            y.rem_euclid(self.chunk_size) as usize,
        )
    }

    pub fn load_chunk(&mut self, coord: ChunkCoord) {
        let chunk = self.generator.generate_chunk(coord);
        self.chunks.insert(coord, chunk);
    }

    pub fn get_chunks_in_range(&self, center_x: i32, center_y: i32, range: i32) -> Vec<&Chunk> {
        let center_chunk = self.world_to_chunk(center_x, center_y);
        let mut chunks = Vec::new();

        for dx in -range..=range {
            for dy in -range..=range {
                let coord = ChunkCoord {
                    x: center_chunk.x + dx,
                    y: center_chunk.y + dy,
                };
                if let Some(chunk) = self.chunks.get(&coord) {
                    chunks.push(chunk);
                }
            }
        }
        chunks
    }

    /**
     * `max_distance`: radius to keep
     */
    pub fn unload_distant_chunks(&mut self, center_x: i32, center_y: i32, max_distance: i32) {
        let center_chunk = self.world_to_chunk(center_x, center_y);

        self.chunks.retain(|coord, _| {
            let dx = (coord.x - center_chunk.x).abs();
            let dy = (coord.y - center_chunk.y).abs();
            dx <= max_distance && dy <= max_distance
        });
    }
}
