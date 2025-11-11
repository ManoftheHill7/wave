use crate::controller::Controller;
use crate::player::Player;
use crate::terrain::{ChunkCoord, Terrain, CHUNK_SIZE};

#[cfg(debug_assertions)]
const LIQUID_UPDATE_TIMER: f32 = 0.04;
#[cfg(not(debug_assertions))]
const LIQUID_UPDATE_TIMER: f32 = 0.01;

const MAX_TIDE_DEPTH: f32 = 1000.0;
const TIDE_FREQUENCY: f32 = 1.0 / 120.0;

pub struct WorldState {
    pub player: Player,
    pub terrain: Terrain,
    pub ghost_mode: bool,
    flow_timer: f32,
    tide_timer: f32,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            // player: Player::new(85.0, -1.0),
            player: Player::new(213.0, 37.0),
            terrain: Terrain::new(12345),
            ghost_mode: false,
            flow_timer: 0.0,
            tide_timer: 0.0,
        }
    }

    pub fn tide_level(&self) -> i32 {
        (((self.tide_timer * TIDE_FREQUENCY - std::f32::consts::PI / 2.0 + 0.4).sin() + 1.0)
            * MAX_TIDE_DEPTH
            / 2.0) as i32
    }

    fn initialize_chunk_tides(&mut self, coord: ChunkCoord) {
        use crate::terrain::{Block, LiquidData, CELL_RESOLUTION};

        let tide_level = self.tide_level();

        if let Some(chunk) = self.terrain.chunks.get_mut(&coord) {
            let chunk_size = CHUNK_SIZE as i32;
            for lx in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    if !chunk.get(lx, ly).is_solid() {
                        let wy = coord.y * chunk_size + ly as i32;
                        if wy < tide_level {
                            continue;
                        }
                        chunk.set(lx, ly, Block::Water);
                    }
                }
            }
        }
    }

    fn update_tides(&mut self) {
        use crate::terrain::{Block, LiquidData, CELL_RESOLUTION};

        let tide_level = self.tide_level();
        let chunk_coords: Vec<_> = self.terrain.chunks.keys().copied().collect();

        for coord in chunk_coords {
            if let Some(chunk) = self.terrain.chunks.get_mut(&coord) {
                let chunk_size = CHUNK_SIZE as i32;
                for lx in 0..CHUNK_SIZE {
                    for ly in 0..CHUNK_SIZE {
                        let wy = coord.y * chunk_size + ly as i32;
                        if chunk.get(lx, ly) == Block::Tide {
                            if wy > tide_level {
                                chunk.set(lx, ly, Block::Water);
                            } else {
                                // Remove water
                                for cell_y in 0..CELL_RESOLUTION {
                                    for cell_x in 0..CELL_RESOLUTION {
                                        chunk.liquid_set(
                                            lx as f32 + cell_x as f32 / CELL_RESOLUTION as f32,
                                            ly as f32 + cell_y as f32 / CELL_RESOLUTION as f32,
                                            LiquidData::empty(),
                                        );
                                    }
                                }
                            }
                        } else if wy > tide_level + 100 {
                            chunk.set(lx, ly, Block::Water);
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32, controller: &Controller) {
        if self.ghost_mode {
            self.player.update_ghost(dt, &self.terrain, controller);
        } else {
            self.player.update(dt, &mut self.terrain, controller);
        }

        self.tide_timer += dt;
        self.flow_timer += dt;
        while self.flow_timer > LIQUID_UPDATE_TIMER {
            self.flow_timer -= LIQUID_UPDATE_TIMER;
            self.terrain.flow();
        }

        let px = self.player.position.x as i32;
        let py = self.player.position.y as i32;

        let loaded_chunk_radius = 2;

        let chunk_size = CHUNK_SIZE as i32;
        for dx in -loaded_chunk_radius..=loaded_chunk_radius {
            for dy in -loaded_chunk_radius..=loaded_chunk_radius {
                let cx = (px + dx * chunk_size) / chunk_size;
                let cy = (py + dy * chunk_size) / chunk_size;
                let chunk_coord = ChunkCoord { x: cx, y: cy };

                if !self.terrain.chunks.contains_key(&chunk_coord) {
                    self.terrain.load_chunk(chunk_coord);
                    self.initialize_chunk_tides(chunk_coord);
                }
            }
        }

        self.terrain
            .unload_distant_chunks(px, py, loaded_chunk_radius + 1);

        self.update_tides();
    }
}
