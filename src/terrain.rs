use std::collections::HashMap;
use crate::terrain_generator::TerrainGenerator;

pub const CHUNK_SIZE: usize = 128;

// Block types enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
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

// Individual block
#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
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
    pub coord: ChunkCoord,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Chunk {
            blocks: [Block { block_type: BlockType::Air }; CHUNK_SIZE * CHUNK_SIZE],
            coord,
        }
    }

    pub fn get_block(&self, local_x: usize, local_y: usize) -> Block {
        let index = local_y * CHUNK_SIZE + local_x;
        self.blocks[index]
    }

    pub fn set_block(&mut self, local_x: usize, local_y: usize, block: Block) {
        let index = local_y * CHUNK_SIZE + local_x;
        self.blocks[index] = block;
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

    pub fn at(&self, x: i32, y: i32) -> Block {
        let chunk_coord = self.world_to_chunk(x, y);
        let local_coord = self.world_to_local(x, y);

        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.get_block(local_coord.0, local_coord.1)
        } else {
            Block { block_type: BlockType::Air }
        }
    }

    pub fn solid_terrain_at(&self, x: i32, y: i32) -> bool {
        matches!(self.at(x, y).block_type,
            BlockType::Dirt |
            BlockType::Stone |
            BlockType::Grass |
            BlockType::Sand |
            BlockType::Log |
            BlockType::Leaf)
    }

    pub fn liquid_terrain_at(&self, x: i32, y: i32) -> bool {
        matches!(self.at(x, y).block_type,
            BlockType::Water |
            BlockType::Lava)
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
            chunk.set_block(local_coord.0, local_coord.1, block);
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
