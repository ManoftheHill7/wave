use crate::terrain::{Terrain, ChunkCoord, CHUNK_SIZE};
use crate::player::Player;
use crate::controller::Controller;

// Liquid update timer - adjust based on performance needs
const LIQUID_UPDATE_TIMER: f32 = 0.016;
pub struct WorldState {
    pub player: Player,
    pub terrain: Terrain,
    pub ghost_mode: bool,
    flow_timer: f32,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            // player: Player::new(85.0, -1.0),
            player: Player::new(215.0, 39.0),
            terrain: Terrain::new(12345),
            ghost_mode: false,
            flow_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, controller: &Controller) {
        if self.ghost_mode {
            self.player.update_ghost(dt, &self.terrain, controller);
        } else {
            self.player.update(dt, &self.terrain, controller);
        }
        self.flow_timer += dt;
        while self.flow_timer > LIQUID_UPDATE_TIMER {
            self.flow_timer -= LIQUID_UPDATE_TIMER;
            self.terrain.flow();
        }

        let px = self.player.position.x as i32;
        let py = self.player.position.y as i32;

        let loaded_chunk_radius = 1;

        let chunk_size = CHUNK_SIZE as i32;
        for dx in -loaded_chunk_radius..=loaded_chunk_radius {
            for dy in -loaded_chunk_radius..=loaded_chunk_radius {
                let cx = (px + dx * chunk_size) / chunk_size;
                let cy = (py + dy * chunk_size) / chunk_size;
                let chunk_coord = ChunkCoord { x: cx, y: cy };

                if !self.terrain.chunks.contains_key(&chunk_coord) {
                    self.terrain.load_chunk(chunk_coord);
                }
            }
        }

        self.terrain.unload_distant_chunks(px, py, loaded_chunk_radius + 1);
    }
}
