use crate::controller::Controller;
use crate::inventory::Inventory;
use crate::lighting::LightingSystem;
use crate::player::Player;
use crate::terrain::{ChunkCoord, Terrain, CHUNK_SIZE};
use rand::Rng;
use std::collections::HashMap;

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

pub const LIGHTING_RANGE: i32 = 55;
pub const RENDER_RANGE: i32 = 40;

pub const CHEST_WEIGHT_LIMIT: f32 = 1000.0;

// Bomb configuration
pub const BOMB_FUSE_TIME: f32 = 3.0;
pub const BOMB_BLAST_STRENGTH: f32 = 50.0;
pub const BOMB_BLAST_RADIUS: i32 = 6;
pub const BOMB_PLAYER_DAMAGE: f32 = 6.0; // Max damage at center (in health points)

#[derive(Clone)]
pub struct ActiveBomb {
    pub x: i32,
    pub y: i32,
    pub timer: f32,
    pub fuse_time: f32,
    pub blast_strength: f32,
    pub blast_radius: i32,
}

impl ActiveBomb {
    pub fn new(x: i32, y: i32) -> Self {
        ActiveBomb {
            x,
            y,
            timer: 0.0,
            fuse_time: BOMB_FUSE_TIME,
            blast_strength: BOMB_BLAST_STRENGTH,
            blast_radius: BOMB_BLAST_RADIUS,
        }
    }

    /// Get the animation frame (0-8) based on timer progress
    pub fn get_frame(&self) -> usize {
        let progress = (self.timer / self.fuse_time).min(1.0);
        (progress * 8.0).floor() as usize
    }
}

pub struct WorldState {
    pub player: Player,
    pub terrain: Terrain,
    pub lighting_system: LightingSystem,
    pub ghost_mode: bool,
    pub chests: HashMap<(i32, i32), Inventory>,
    pub active_bombs: Vec<ActiveBomb>,
    tick_timer: f32,
    flow_timer: f32,
    tide_timer: f32,
    illuminate_timer: f32,
}

impl WorldState {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let seed = rng.gen::<u64>();
        println!("World seed: {}", seed);

        WorldState {
            player: Player::new(46.0, -6.0),
            terrain: Terrain::new(seed),
            lighting_system: LightingSystem::new(),
            ghost_mode: false,
            chests: HashMap::new(),
            active_bombs: Vec::new(),
            tick_timer: 0.0,
            flow_timer: 0.0,
            illuminate_timer: 0.0,
            tide_timer: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.terrain.tick();
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

        // Add lights from lumosite ore in the world
        // Use LIGHTING_RANGE to ensure all found lumosite ore get lighting calculated
        for dx in -LIGHTING_RANGE..=LIGHTING_RANGE {
            for dy in -LIGHTING_RANGE..=LIGHTING_RANGE {
                let tx = px + dx;
                let ty = py + dy;
                let block = self.terrain.at(tx, ty);

                let light_type = match block {
                    Block::LumositeOre => Some(LightType::LumositeOre),
                    _ => None,
                };

                if let Some(lt) = light_type {
                    let ore_pos = Vector2::new(tx as f32 + 0.5, ty as f32 + 0.5);
                    let ore_light = Light::new(ore_pos, lt).with_flicker(self.player.time);
                    self.lighting_system.add_light(ore_light);
                }
            }
        }

        // Calculate shadows for all lights
        self.lighting_system
            .calculate_shadows(&self.terrain, px, py, RENDER_RANGE);

        // Calculate opaque block lighting
        self.lighting_system
            .calculate_opaque_lighting(&self.terrain, px, py, LIGHTING_RANGE);
    }

    pub fn update(&mut self, dt: f32, controller: &Controller) {
        if self.ghost_mode {
            self.player.update_ghost(dt, &self.terrain, controller);
        } else {
            self.player.update(
                dt,
                &mut self.terrain,
                &mut self.chests,
                &mut self.active_bombs,
                controller,
            );
        }

        // Update lighting at reduced framerate for performance
        // On WASM, use a longer timer to reduce CPU usage
        #[cfg(target_arch = "wasm32")]
        let lighting_timer = LIGHTING_UPDATE_TIMER * 4.0;
        #[cfg(not(target_arch = "wasm32"))]
        let lighting_timer = LIGHTING_UPDATE_TIMER;

        if self.illuminate_timer > lighting_timer {
            self.illuminate_timer = 0.0;
            self.update_lighting();
        }

        self.illuminate_timer += dt;
        self.tide_timer += dt;
        self.flow_timer += dt;

        // On WASM, reduce water flow frequency for performance
        #[cfg(target_arch = "wasm32")]
        let flow_timer = LIQUID_UPDATE_TIMER * 3.0;
        #[cfg(not(target_arch = "wasm32"))]
        let flow_timer = LIQUID_UPDATE_TIMER;

        while self.flow_timer > flow_timer {
            self.flow_timer -= flow_timer;
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

        // Update tick every 10 sec
        self.tick_timer += dt;
        if self. tick_timer > 10.0 {
            self.tick_timer = 0.0;
            self.tick();
        }
        self.update_tides();
        self.update_bombs(dt);
    }

    fn update_bombs(&mut self, dt: f32) {

        // Update all bomb timers
        for bomb in &mut self.active_bombs {
            bomb.timer += dt;
        }

        // Collect bombs that should explode
        let exploded_bombs: Vec<ActiveBomb> = self
            .active_bombs
            .iter()
            .filter(|b| b.timer >= b.fuse_time)
            .cloned()
            .collect();

        // Process explosions
        for bomb in exploded_bombs {
            self.explode_bomb(&bomb);
        }

        // Remove exploded bombs
        self.active_bombs.retain(|b| b.timer < b.fuse_time);
    }

    fn explode_bomb(&mut self, bomb: &ActiveBomb) {
        use crate::terrain::Block;

        // Remove the bomb block itself
        self.terrain.set(bomb.x, bomb.y, Block::Air);

        // Cast rays in many directions for blast with shielding
        let num_rays = 72; // Every 5 degrees
        for i in 0..num_rays {
            let angle = (i as f32 / num_rays as f32) * std::f32::consts::PI * 2.0;
            let dir_x = angle.cos();
            let dir_y = angle.sin();

            let mut remaining_damage = bomb.blast_strength;

            // Walk along the ray
            for dist in 1..=bomb.blast_radius {
                if remaining_damage <= 0.0 {
                    break;
                }

                let bx = bomb.x + (dir_x * dist as f32).round() as i32;
                let by = bomb.y + (dir_y * dist as f32).round() as i32;

                let block = self.terrain.at(bx, by);
                let durability = block.durability();

                // Skip air and water (durability 0)
                if durability <= 0.0 {
                    continue;
                }

                // Check if we can destroy this block
                if remaining_damage >= durability {
                    // Destroy the block (no drops)
                    self.terrain.set(bx, by, Block::Air);

                    // Also remove chest inventory if it was a chest
                    if matches!(block, Block::WoodenChest) {
                        self.chests.remove(&(bx, by));
                    }

                    // Damage absorbed by the block
                    remaining_damage -= durability;
                } else {
                    // Block survives, shields everything behind it
                    break;
                }
            }
        }

        // Calculate player damage
        let player_center_x = self.player.position.x + self.player.width / 2.0;
        let player_center_y = self.player.position.y + self.player.height / 2.0;

        let dx = player_center_x - bomb.x as f32;
        let dy = player_center_y - bomb.y as f32;
        let player_dist = (dx * dx + dy * dy).sqrt();

        if player_dist <= bomb.blast_radius as f32 {
            // Check line of sight to player
            if self.has_explosion_line_of_sight(bomb.x, bomb.y, player_center_x, player_center_y) {
                // Damage falls off with distance
                let damage_ratio = 1.0 - (player_dist / bomb.blast_radius as f32);
                let damage = (BOMB_PLAYER_DAMAGE * damage_ratio).round() as i32;
                self.player.health -= damage;

                // Ensure health doesn't go below 0
                if self.player.health < 0 {
                    self.player.health = 0;
                }
            }
        }
    }

    /// Check if there's line of sight from bomb to target (for player damage)
    fn has_explosion_line_of_sight(&self, from_x: i32, from_y: i32, to_x: f32, to_y: f32) -> bool {
        let dx = to_x - from_x as f32;
        let dy = to_y - from_y as f32;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 1.0 {
            return true;
        }

        let steps = dist.ceil() as i32;
        let step_x = dx / steps as f32;
        let step_y = dy / steps as f32;

        for i in 1..steps {
            let check_x = (from_x as f32 + step_x * i as f32).round() as i32;
            let check_y = (from_y as f32 + step_y * i as f32).round() as i32;

            let block = self.terrain.at(check_x, check_y);
            // If there's a solid block in the way, no line of sight
            if block.is_solid() {
                return false;
            }
        }

        true
    }

    /// Get the animation frame for a bomb at a specific position
    pub fn get_bomb_frame(&self, x: i32, y: i32) -> Option<usize> {
        self.active_bombs
            .iter()
            .find(|b| b.x == x && b.y == y)
            .map(|b| b.get_frame())
    }
}
