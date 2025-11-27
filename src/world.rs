use crate::controller::Controller;
use crate::lighting::LightingSystem;
use crate::player::Player;
use crate::terrain::{ChunkCoord, Terrain, CHUNK_SIZE};

#[cfg(debug_assertions)]
const LIQUID_UPDATE_TIMER: f32 = 0.04;
#[cfg(not(debug_assertions))]
const LIQUID_UPDATE_TIMER: f32 = 0.01;

#[cfg(debug_assertions)]
const LIGHTING_UPDATE_TIMER: f32 = 1.0 / 24.0;
#[cfg(not(debug_assertions))]
const LIGHTING_UPDATE_TIMER: f32 = 1.0 / 48.0;

const MAX_TIDE_DEPTH: f32 = 1000.0;
const TIDE_FREQUENCY: f32 = 1.0 / 120.0;

pub const LIGHTING_RANGE: i32 = 35;
pub const RENDER_RANGE: i32 = 35;

pub struct WorldState {
    pub player: Player,
    pub terrain: Terrain,
    pub lighting_system: LightingSystem,
    pub ghost_mode: bool,
    flow_timer: f32,
    tide_timer: f32,
    illuminate_timer: f32,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            // player: Player::new(38.74, -3.99), // This spot reproduces the getting stuck bug
            player: Player::new(32.0, -3.0),
            terrain: Terrain::new(12345),
            lighting_system: LightingSystem::new(),
            ghost_mode: false,
            flow_timer: 0.0,
            illuminate_timer: 0.0,
            tide_timer: 0.0,
        }
    }

    pub fn get_flow_timer(&self) -> f32 {
        self.flow_timer
    }

    pub fn get_tide_timer(&self) -> f32 {
        self.tide_timer
    }

    pub fn set_flow_timer(&mut self, value: f32) {
        self.flow_timer = value;
    }

    pub fn set_tide_timer(&mut self, value: f32) {
        self.tide_timer = value;
    }

    pub fn tide_level(&self) -> i32 {
        let initial_offset = 3.14;
        (((self.tide_timer * TIDE_FREQUENCY - std::f32::consts::PI / 2.0 + initial_offset).sin()
            + 1.0)
            * MAX_TIDE_DEPTH
            / 2.0) as i32
    }

    /// Returns tide as a percentage from 0.0 (low tide) to 1.0 (high tide)
    pub fn tide_percent(&self) -> f32 {
        self.tide_level() as f32 / MAX_TIDE_DEPTH
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

    fn update_lighting(&mut self) {
        use crate::lighting::{Light, LightType};
        use crate::terrain::Block;
        use crate::tools::ToolType;
        use raylib::prelude::Vector2;

        // Update ambient darkness based on player depth
        self.lighting_system
            .update_ambient_darkness(self.player.position.y);

        // Update lights (clear and re-add each frame)
        self.lighting_system.clear_lights();

        let px = self.player.position.x as i32;
        let py = self.player.position.y as i32;

        // Add player's lamp if equipped
        let player_has_lamp = self.player.left_hand == Some(ToolType::Lamp)
            || self.player.right_hand == Some(ToolType::Lamp);

        if player_has_lamp {
            let lamp_pos = Vector2::new(
                self.player.position.x + self.player.width / 2.0,
                self.player.position.y + self.player.height / 2.0,
            );
            let lamp_light = Light::new(lamp_pos, LightType::Lamp).with_flicker(self.player.time);
            self.lighting_system.add_light(lamp_light);
        }

        // Add lights from placed torches in the world
        // Use LIGHTING_RANGE to ensure all found torches get lighting calculated
        for dx in -LIGHTING_RANGE..=LIGHTING_RANGE {
            for dy in -LIGHTING_RANGE..=LIGHTING_RANGE {
                let tx = px + dx;
                let ty = py + dy;
                let block = self.terrain.at(tx, ty);

                let light_type = match block {
                    Block::Torch => Some(LightType::CoalTorch),
                    Block::Lumostorch => Some(LightType::LumostoneTorch),
                    _ => None,
                };

                if let Some(lt) = light_type {
                    let torch_pos = Vector2::new(tx as f32 + 0.5, ty as f32 + 0.5);
                    let torch_light = Light::new(torch_pos, lt).with_flicker(self.player.time);
                    self.lighting_system.add_light(torch_light);
                }
            }
        }

        // Calculate shadows for all lights
        self.lighting_system
            .calculate_shadows(&self.terrain, px, py, RENDER_RANGE);

        // Calculate solid block lighting
        self.lighting_system
            .calculate_solid_lighting(&self.terrain, px, py, LIGHTING_RANGE);
    }

    pub fn update(&mut self, dt: f32, controller: &Controller) {
        if self.ghost_mode {
            self.player.update_ghost(dt, &self.terrain, controller);
        } else {
            self.player.update(dt, &mut self.terrain, controller);
        }

        // Update lighting at reduced framerate for performance
        if self.illuminate_timer > LIGHTING_UPDATE_TIMER {
            self.illuminate_timer = 0.0;
            self.update_lighting();
        }

        self.illuminate_timer += dt;
        self.tide_timer += dt;
        self.flow_timer += dt;
        while self.flow_timer > LIQUID_UPDATE_TIMER {
            self.flow_timer -= LIQUID_UPDATE_TIMER;
            self.terrain.flow();
        }

        let px = self.player.position.x as i32;
        let py = self.player.position.y as i32;

        let load_chunk_radius = 2;
        let unload_chunk_radius = load_chunk_radius + 1;

        let chunk_size = CHUNK_SIZE as i32;
        for dx in -load_chunk_radius..=load_chunk_radius {
            for dy in -load_chunk_radius..=load_chunk_radius {
                let cx = (px + dx * chunk_size).div_euclid(chunk_size);
                let cy = (py + dy * chunk_size).div_euclid(chunk_size);
                let chunk_coord = ChunkCoord { x: cx, y: cy };

                if !self.terrain.chunks.contains_key(&chunk_coord) {
                    self.terrain.load_chunk(chunk_coord);
                    self.initialize_chunk_tides(chunk_coord);
                }
            }
        }

        self.terrain
            .unload_distant_chunks(px, py, unload_chunk_radius);

        self.update_tides();
    }
}
