use crate::terrain::{Terrain, ChunkCoord, CHUNK_SIZE};
use crate::player::Player;
use crate::controller::Controller;

pub struct WorldState {
    pub player: Player,
    pub terrain: Terrain,
    pub ghost_mode: bool,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            player: Player::new(85.0, -1.0),
            terrain: Terrain::new(12345),
            ghost_mode: false,
        }
    }

    pub fn update(&mut self, dt: f32, controller: &Controller) {
        if self.ghost_mode {
            self.player.update_ghost(dt, &self.terrain, controller);
        } else {
            self.player.update(dt, &self.terrain, controller);
        }

        let px = self.player.position.x as i32;
        let py = self.player.position.y as i32;

        let loaded_chunk_radiusy = 5;
        let loaded_chunk_radiusx = 7;
        let unload_chunk_radius = loaded_chunk_radiusy.max(loaded_chunk_radiusx);

        let chunk_size = CHUNK_SIZE as i32;
        for dx in -loaded_chunk_radiusx..=loaded_chunk_radiusx {
            for dy in -1..=loaded_chunk_radiusy {
                let cx = (px + dx * chunk_size) / chunk_size;
                let cy = (py + dy * chunk_size) / chunk_size;
                let chunk_coord = ChunkCoord { x: cx, y: cy };

                if !self.terrain.chunks.contains_key(&chunk_coord) {
                    self.terrain.load_chunk(chunk_coord);
                }
            }
        }

        self.terrain.unload_distant_chunks(px, py, unload_chunk_radius);
    }
}
