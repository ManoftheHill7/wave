use crate::terrain::{Terrain, ChunkCoord};
use crate::player::Player;
use crate::controller::Controller;

pub struct WorldState {
    pub player: Player,
    pub terrain: Terrain,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            player: Player::new(0.0, -2.0),
            terrain: Terrain::new(12345),
        }
    }

    pub fn update(&mut self, dt: f32, controller: &Controller) {
        self.player.update(dt, &self.terrain, controller);

        let px = self.player.position.x as i32;
        let py = self.player.position.y as i32;

        for dx in -3..=3 {
            for dy in -3..=3 {
                let cx = (px + dx * 32) / 32;
                let cy = (py + dy * 32) / 32;
                let chunk_coord = ChunkCoord { x: cx, y: cy };

                if !self.terrain.chunks.contains_key(&chunk_coord) {
                    self.terrain.load_chunk(chunk_coord);
                }
            }
        }

        self.terrain.unload_distant_chunks(px, py, 8);
    }
}
