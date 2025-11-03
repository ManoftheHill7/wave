use crate::terrain::{Block, BlockType, Chunk, ChunkCoord, CHUNK_SIZE};
use noise::{NoiseFn, Perlin};

pub struct TerrainGenerator {
    seed: u64,
}

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self {
        TerrainGenerator { seed }
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE.try_into().unwrap();
        let noiseDetail = 150;
        let noiseHeight = 10;
        let noise = Perlin::new(1);

        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;
            let height_f = noiseHeight as f64 * noise.get([wx as f64 * (1.0 / noiseDetail as f64)]);
            let height = height_f as i32 + coord.x;
            dbg!(height);
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;

                let block_type = if wx < 0 {
                    if wy > 0 {
                        BlockType::Water
                    } else {
                        BlockType::Air
                    }
                } else if wx < 10 && wy < 10 {
                    if wy > -1 {
                        BlockType::Sand
                    } else {
                        BlockType::Air
                    }
                } else if wy == height {
                    BlockType::Grass
                } else if wy > height {
                    BlockType::Dirt
                } else if wy > height + 10 {
                    BlockType::Stone
                } else {
                    BlockType::Air
                };

                chunk.set_block(lx, ly, Block { block_type });
            }
        }
        // self.add_tree(&mut chunk, 15, 1, 10, 20, coord);
        // self.add_tree(&mut chunk, -10, 1, 10, 60, coord);
        chunk
    }

    fn add_tree(&self, chunk: &mut Chunk, x: i32, y: i32, width: i32, height: i32, chunk_coord: ChunkCoord) {
        if height < 2 || width < 1 {
            return;
        }

        let trunk_height = (height as f32 * 0.6).ceil() as i32;
        let canopy_height = height - trunk_height;

        // Generate main trunk
        for i in 0..trunk_height {
            let trunk_y = y - i;

            if self.is_in_chunk(x, trunk_y, chunk_coord) {
                let (lx, ly) = self.world_to_local(x, trunk_y, chunk_coord);
                chunk.set_block(lx, ly, Block { block_type: BlockType::Log });
            }
        }

        let canopy_base_y = y - trunk_height;
        for layer in 0..canopy_height {
            let canopy_y = canopy_base_y - layer;

            let layer_ratio = 1.0 - (layer as f32 / canopy_height as f32) * 0.5;
            let layer_width = ((width as f32 * layer_ratio).ceil() as i32).max(1);

            let layer_width = if layer_width % 2 == 0 { layer_width + 1 } else { layer_width };
            let half_width = layer_width / 2;

            for dx in -half_width..=half_width {
                let leaf_x = x + dx;

                if self.is_in_chunk(leaf_x, canopy_y, chunk_coord) {
                    let (lx, ly) = self.world_to_local(leaf_x, canopy_y, chunk_coord);
                    chunk.set_block(lx, ly, Block { block_type: BlockType::Leaf });
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

        wx >= chunk_start_x && wx < chunk_end_x && wy >= chunk_start_y && wy <
            chunk_end_y
    }

    fn world_to_local(&self, wx: i32, wy: i32, chunk_coord: ChunkCoord) ->
        (usize, usize) {
            let chunk_size = CHUNK_SIZE as i32;
            let lx = (wx - chunk_coord.x * chunk_size) as usize;
            let ly = (wy - chunk_coord.y * chunk_size) as usize;
            (lx, ly)
    }

}
