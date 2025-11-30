use crate::controller::Controller;
use crate::inventory::Inventory;
use crate::terrain::{Block, Terrain};
use crate::tools::{
    load_dash, load_glider, load_pick, ToolDash, ToolGlider, ToolPickaxe, ToolTideClock, ToolType,
};
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

pub const ACCEL: f32 = 50.0;
pub const SPEED: f32 = 12.0;
pub const MIN_SPEED: f32 = 2.0;
pub const CLIMB_SPEED: f32 = 3.125;
pub const CLIMB_STAMINA: f32 = 4.0;
pub const JUMP_VELOCITY: f32 = -15.0;
pub const TERMINAL_VELOCITY: f32 = 50.0;
pub const GRAVITY: f32 = 40.625;
pub const JUMP_GRAVITY_REDUCTION: f32 = 0.85;
pub const JUMP_RELEASE_REDUCTION: f32 = 0.1;
pub const JUMP_COYOTE_TIME: f32 = 0.20;
pub const JUMP_BUFFER_TIME: f32 = 0.1;
pub const WALLJUMP_EFFECT_STRENGTH: f32 = 0.15;
pub const WALLJUMP_EFFECT_TIME: f32 = 0.3;
pub const WALLJUMP_LOCK_TIME: f32 = 0.1;
pub const WALLJUMP_X_RELATIVE_STRENGTH: f32 = 1.0;
pub const WALLSLIDE_FRICTION: f32 = 0.85;
pub const DASH_VELOCITY: f32 = SPEED * 1.7;
pub const DASHJUMP_COOLDOWN: f32 = 0.075;
pub const CORNER_CORRECTION_AMOUNT: i32 = 5;
pub const WALLJUMP_DETECT_DISTANCE: f32 = 0.125;
pub const CLIMBING_TOPOUT_ASSIST: f32 = 10.0;

pub const BASE_HEIGHT: f32 = 1.99;
pub const DASH_HEIGHT: f32 = 0.9;
pub const BASE_WIDTH: f32 = 1.29;
pub const DASH_WIDTH: f32 = 0.9;

pub const WATER_BUOYANCY: f32 = -GRAVITY / 4.0;
pub const WATER_DRAG: f32 = 0.9;
pub const SWIM_SPEED: f32 = 8.0;
pub const SWIM_EXIT_TIME: f32 = 0.10;

pub const MAX_RAYCAST_PICKAXE: f32 = 3.0;
pub const MAX_RAYCAST_HOOK: f32 = 10.0;
pub const MAX_RAYCAST_SPEAR: f32 = 3.0;
pub const MAX_RAYCAST_DASH: f32 = 1.0;
pub const MAX_RAYCAST_PLACE_BLOCK: f32 = 3.5;

pub const INVENTORY_STARTING_WEIGHT: f32 = 100.0;
pub const STARTING_HEALTH: i32 = 12; // 4 frames of heart * 3 hearts
pub const MAX_BREATH_HOLD: f32 = 10.0;
pub const DROWN_DAMAGE_INTERVAL: f32 = 1.0; // Lose 1 heart per second when out of breath

pub const SPIKE_IMMUNITY_COOLDOWN: f32 = 0.3;

#[derive(Debug)]
struct RaycastResult {
    final_position: Vector2,
    last_free_position: Vector2,
    hit: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    #[serde(with = "vector2_serde")]
    pub position: Vector2,
    #[serde(with = "vector2_serde")]
    pub velocity: Vector2,
    pub height: f32,
    pub width: f32,
    pub facing_dir: i32,

    pub on_ground: bool,
    pub is_jumping: bool,
    pub is_dashing: bool,
    pub is_climbing: bool,
    pub is_on_ladder: bool,
    pub is_sliding: bool,
    pub is_swimming: bool,
    pub is_mining: bool,
    pub just_landed: bool,
    pub just_finished_dashing: bool,

    pub last_on_ground: f32,
    pub last_in_water: f32,
    pub try_jumped_at: f32,
    pub wall_jumped_at: f32,
    pub spike_touched_at: f32,
    pub started_mining_at: f32,
    pub dashed_at: f32,
    pub last_action_at: f32,
    pub time: f32,

    pub dashes: i32,
    pub climb_stamina: f32,

    pub gravity_reduction: f32,

    #[serde(with = "vector2_serde")]
    pub dash_dir: Vector2,

    #[serde(with = "vector2_serde")]
    pub landing_speed: Vector2,
    #[serde(with = "vector2_serde")]
    pub last_velocity: Vector2,

    pub currently_mining: Option<(f32, f32)>,

    pub raycast_left_tile: Option<(f32, f32)>,
    pub raycast_right_tile: Option<(f32, f32)>,
    #[serde(with = "vector2_serde")]
    pub raycast_left_end: Vector2,
    #[serde(with = "vector2_serde")]
    pub raycast_right_end: Vector2,

    pub health: i32,
    pub breath: f32,
    #[serde(default)]
    pub last_drown_damage: f32,

    pub inventory: Inventory,

    pub left_hand: Option<ToolType>,
    pub right_hand: Option<ToolType>,
    pub head_slot: Option<ToolType>,
    pub tool_dash: Option<ToolDash>,
    pub tool_pickaxe: Option<ToolPickaxe>,
    pub tool_glider: Option<ToolGlider>,
    pub tool_tideclock: Option<ToolTideClock>,

    pub is_gliding: bool,

    // Sound timing (not saved)
    #[serde(skip)]
    pub last_pickaxe_sound: f32,
    #[serde(skip)]
    pub last_footstep_sound: f32,
    #[serde(skip)]
    pub jumped_this_frame: bool,
}

// Custom serialization for raylib Vector2
mod vector2_serde {
    use raylib::prelude::Vector2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct V2 {
        x: f32,
        y: f32,
    }

    pub fn serialize<S>(v: &Vector2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        V2 { x: v.x, y: v.y }.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vector2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = V2::deserialize(deserializer)?;
        Ok(Vector2::new(v.x, v.y))
    }
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            position: Vector2::new(x, y),
            velocity: Vector2::zero(),
            height: BASE_HEIGHT,
            width: BASE_WIDTH,
            facing_dir: -1,

            on_ground: false,
            is_jumping: false,
            is_dashing: false,
            is_climbing: false,
            is_on_ladder: false,
            is_sliding: false,
            is_swimming: false,
            is_mining: false,
            just_landed: false,
            just_finished_dashing: false,

            last_on_ground: -999.0,
            last_in_water: -999.0,
            try_jumped_at: -999.0,
            wall_jumped_at: -999.0,
            spike_touched_at: -999.0,
            started_mining_at: -999.0,
            dashed_at: -999.0,
            last_action_at: -999.0,
            time: 0.0,

            dashes: 0,
            climb_stamina: CLIMB_STAMINA,

            gravity_reduction: 1.0,
            dash_dir: Vector2::zero(),
            landing_speed: Vector2::zero(),
            last_velocity: Vector2::zero(),

            currently_mining: None,

            raycast_left_tile: None,
            raycast_right_tile: None,
            raycast_left_end: Vector2::zero(),
            raycast_right_end: Vector2::zero(),

            health: STARTING_HEALTH,
            breath: MAX_BREATH_HOLD,
            last_drown_damage: 0.0,

            inventory: Inventory::new(INVENTORY_STARTING_WEIGHT),

            left_hand: Some(ToolType::Pickaxe),
            right_hand: None,
            head_slot: None,

            tool_dash: None,
            tool_pickaxe: Some(load_pick("stone_pickaxe")),
            tool_glider: None,
            tool_tideclock: None,

            is_gliding: false,

            last_pickaxe_sound: 0.0,
            last_footstep_sound: 0.0,
            jumped_this_frame: false,
        }
    }

    pub fn update_ghost(&mut self, dt: f32, terrain: &Terrain, controller: &Controller) {
        let input_dir = controller.input_dir;
        self.is_swimming = true;

        let speed = 5.0 * ACCEL * dt;
        let swim_speed = if controller.climb_pressed {
            self.time += dt;
            self.time += dt;
            SWIM_SPEED * 8.0
        } else {
            self.time += dt;
            SWIM_SPEED * 3.0
        };

        if input_dir.x != 0.0 {
            self.facing_dir = input_dir.x.signum() as i32;
        }
        self.velocity.x = Self::move_toward(self.velocity.x, input_dir.x * swim_speed, speed);
        self.velocity.y = Self::move_toward(self.velocity.y, input_dir.y * swim_speed, speed);

        self.position += self.velocity * dt;

        self.calculated_selected_blocks(terrain, controller);
    }

    fn block_would_intersect_player(&self, block_x: i32, block_y: i32) -> bool {
        let block_left = block_x as f32;
        let block_right = (block_x + 1) as f32;
        let block_top = block_y as f32;
        let block_bottom = (block_y + 1) as f32;

        let player_left = self.position.x;
        let player_right = self.position.x + self.width;
        let player_top = self.position.y;
        let player_bottom = self.position.y + self.height;

        // Check for AABB collision
        player_right > block_left
            && player_left < block_right
            && player_bottom > block_top
            && player_top < block_bottom
    }

    fn calculate_hand_raycast(
        &self,
        tool: Option<ToolType>,
        raycast_start: Vector2,
        raycast_direction: Vector2,
        terrain: &Terrain,
    ) -> (Option<(f32, f32)>, Vector2) {
        if let Some(tool) = tool {
            let max_length = match tool {
                ToolType::Pickaxe => MAX_RAYCAST_PICKAXE,
                ToolType::Dash => MAX_RAYCAST_DASH,
                ToolType::Lamp => 0.0,      // Lamp doesn't raycast
                ToolType::Glider => 0.0,    // Glider doesn't raycast
                ToolType::TideClock => 0.0, // TideClock doesn't raycast
                ToolType::PlaceBlock(_) => MAX_RAYCAST_PLACE_BLOCK,
            };
            let rayresult = self.raycast(
                raycast_start,
                raycast_start + raycast_direction * max_length,
                terrain,
            );
            let end_position = rayresult.final_position;

            if rayresult.hit {
                let tile_pos = if matches!(tool, ToolType::PlaceBlock(_)) {
                    rayresult.last_free_position
                } else {
                    rayresult.final_position
                };
                let tile_x = tile_pos.x.floor() as i32;
                let tile_y = tile_pos.y.floor() as i32;

                if matches!(tool, ToolType::PlaceBlock(_))
                    && self.block_would_intersect_player(tile_x, tile_y)
                {
                    (None, end_position)
                } else {
                    (Some((tile_x as f32, tile_y as f32)), end_position)
                }
            } else {
                (None, end_position)
            }
        } else {
            (None, Vector2::zero())
        }
    }

    pub fn calculated_selected_blocks(&mut self, terrain: &Terrain, controller: &Controller) {
        let raycast_start = self.position + Vector2::new(self.width / 2.0, self.height / 2.0);

        let (left_tile, left_end) = self.calculate_hand_raycast(
            self.left_hand,
            raycast_start,
            controller.raycast_direction,
            terrain,
        );
        self.raycast_left_tile = left_tile;
        self.raycast_left_end = left_end;

        let (right_tile, right_end) = self.calculate_hand_raycast(
            self.right_hand,
            raycast_start,
            controller.raycast_direction,
            terrain,
        );
        self.raycast_right_tile = right_tile;
        self.raycast_right_end = right_end;
    }

    pub fn get_intersecting_crafting_station(&self, terrain: &Terrain) -> Option<Block> {
        // Check multiple points in player bounding box
        let points = [
            (self.position.x, self.position.y),               // top-left
            (self.position.x + self.width, self.position.y),  // top-right
            (self.position.x, self.position.y + self.height), // bottom-left
            (self.position.x + self.width, self.position.y + self.height), // bottom-right
            (
                self.position.x + self.width / 2.0,
                self.position.y + self.height / 2.0,
            ), // center
        ];

        for (x, y) in points {
            let block = terrain.at(x as i32, y as i32);
            if matches!(block, Block::Workbench | Block::Anvil | Block::Furnace) {
                return Some(block);
            }
        }

        None
    }

    pub fn get_intersecting_chest(&self, terrain: &Terrain) -> Option<(i32, i32)> {
        // Check multiple points in player bounding box
        let points = [
            (self.position.x, self.position.y),               // top-left
            (self.position.x + self.width, self.position.y),  // top-right
            (self.position.x, self.position.y + self.height), // bottom-left
            (self.position.x + self.width, self.position.y + self.height), // bottom-right
            (
                self.position.x + self.width / 2.0,
                self.position.y + self.height / 2.0,
            ), // center
        ];

        for (x, y) in points {
            let bx = x as i32;
            let by = y as i32;
            let block = terrain.at(bx, by);
            if matches!(block, Block::WoodenChest) {
                return Some((bx, by));
            }
        }

        None
    }

    pub fn can_place_block_at(&self, terrain: &Terrain, block: Block, x: i32, y: i32) -> bool {
        // Restrict workbench and anvil placement to underground (y < 0)
        if matches!(block, Block::Workbench | Block::Anvil) && y >= 0 {
            return false;
        }

        // Check if placement is valid for multi-tile blocks
        terrain.can_place_multi_tile(x, y, block)
    }

    pub fn try_place_block(
        &mut self,
        terrain: &mut Terrain,
        chests: &mut std::collections::HashMap<(i32, i32), crate::inventory::Inventory>,
        active_bombs: &mut Vec<crate::world::ActiveBomb>,
        block: Block,
        left_hand: bool,
    ) {
        let tile = if left_hand {
            self.raycast_left_tile
        } else {
            self.raycast_right_tile
        };

        if let Some((tile_x, tile_y)) = tile {
            let x = tile_x.floor() as i32;
            let y = tile_y.floor() as i32;

            if !self.can_place_block_at(terrain, block, x, y) {
                return;
            }

            if let Some(item_type) = block.to_item_type() {
                let taken = self.inventory.take(item_type, 1);
                if taken > 0 {
                    // Use multi-tile placement (works for both single and multi-tile blocks)
                    terrain.place_multi_tile(x, y, block);

                    // Create chest inventory if placing a chest
                    if matches!(block, Block::WoodenChest) {
                        chests.insert(
                            (x, y),
                            crate::inventory::Inventory::new(crate::world::CHEST_WEIGHT_LIMIT),
                        );
                    }

                    // Start bomb timer if placing a bomb
                    if matches!(block, Block::Bomb) {
                        active_bombs.push(crate::world::ActiveBomb::new(x, y));
                    }

                    if self.inventory.count(item_type) == 0 {
                        // Unequip from hand when last item is placed
                        if left_hand {
                            self.left_hand = None;
                        } else {
                            self.right_hand = None;
                        }
                    }
                }
            }
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        terrain: &mut Terrain,
        chests: &mut std::collections::HashMap<(i32, i32), crate::inventory::Inventory>,
        active_bombs: &mut Vec<crate::world::ActiveBomb>,
        controller: &Controller,
    ) {
        let jump_pressed = controller.jump_pressed;
        let jump_held = controller.jump_held;
        let climb_pressed = controller.climb_pressed;
        let input_dir = controller.input_dir;

        // Reset per-frame flags
        self.jumped_this_frame = false;

        self.time += dt;

        let max_dashes = self.tool_dash.as_ref().map_or(0, |x| x.max_dashes);
        let dash_time = self.tool_dash.as_ref().map_or(0.0, |x| x.dash_time);
        let dash_extended_time = self
            .tool_dash
            .as_ref()
            .map_or(0.0, |x| x.dash_extended_time);
        let dash_control_modifier = self
            .tool_dash
            .as_ref()
            .map_or(0.0, |x| x.dash_control_modifier);

        // Update dashing state
        let was_dashing = self.is_dashing;
        self.is_dashing = self.within_grace(self.dashed_at, dash_time);
        self.just_finished_dashing = was_dashing && !self.is_dashing;
        self.height = if self.is_dashing {
            DASH_HEIGHT
        } else {
            BASE_HEIGHT
        };
        self.width = if self.is_dashing {
            DASH_WIDTH
        } else {
            BASE_WIDTH
        };

        if self.just_finished_dashing {
            self.exit_dash_handler(terrain);
        }

        self.is_swimming = self.check_in_water(terrain);
        if self.is_swimming {
            self.last_in_water = self.time;
            self.on_ground = false;
            self.breath -= dt;

            // Take drowning damage when out of breath
            if self.breath <= 0.0 && self.time - self.last_drown_damage >= DROWN_DAMAGE_INTERVAL {
                self.health -= 1;
                self.last_drown_damage = self.time;
            }
        } else {
            // Recover breath at twice the decay rate
            self.breath = (self.breath + 2.0 * dt).min(MAX_BREATH_HOLD);
        }

        // Check if on ladder
        self.is_on_ladder = self.check_on_ladder(terrain);

        // Apply gravity
        if !self.on_ground || self.is_on_ladder {
            if self.is_on_ladder {
                // No gravity when on ladder - only stop falling, not upward movement (jumping)
                if self.velocity.y > 0.0 {
                    self.velocity.y = 0.0;
                }
            } else if self.is_swimming {
                self.velocity.y += WATER_BUOYANCY * dt;
                self.velocity.x *= WATER_DRAG;
                self.velocity.y *= WATER_DRAG;
            } else {
                self.velocity.y += GRAVITY * self.gravity_reduction * dt;
            }

            if !self.on_ground {
                self.landing_speed = self.velocity;
            }
        }
        self.velocity.y = self.velocity.y.clamp(-TERMINAL_VELOCITY, TERMINAL_VELOCITY);

        if !self.is_swimming && self.within_grace(self.last_in_water, SWIM_EXIT_TIME) {
            self.is_swimming = true;
        }

        let on_wall = self.check_wall(terrain);
        let topout_vel = if !on_wall && self.is_climbing {
            self.facing_dir as f32 * CLIMBING_TOPOUT_ASSIST
        } else {
            0.0
        };

        // Wall slide
        self.is_sliding = false;
        if on_wall && self.velocity.y > 0.0 && !self.is_swimming {
            self.is_sliding = true;
            self.velocity.y *= WALLSLIDE_FRICTION;
        }

        // Climbing
        self.is_climbing = false;

        if on_wall && climb_pressed && self.climb_stamina > 0.0 {
            self.climb_stamina -= dt;
            self.is_climbing = true;
            self.velocity.y = 0.0;
            self.velocity.y =
                Self::move_toward(self.velocity.y, input_dir.y * CLIMB_SPEED, CLIMB_SPEED);
        }

        // Jump handling
        if jump_pressed || self.within_grace(self.try_jumped_at, JUMP_BUFFER_TIME) {
            if self.is_swimming {
                self.jump(0.7);
            } else if self.is_on_ladder {
                // Jump from ladder
                self.jump(1.0);
            } else if self.on_ground
                || (self.within_grace(self.last_on_ground, JUMP_COYOTE_TIME)
                    && !self.within_grace(self.last_action_at, DASHJUMP_COOLDOWN))
            {
                // TODO: wavedash
                self.jump(1.0);
            } else if on_wall && !self.within_grace(self.last_action_at, DASHJUMP_COOLDOWN) {
                // Wall jump
                self.last_action_at = self.time;
                self.velocity.y = JUMP_VELOCITY;
                self.velocity.x =
                    JUMP_VELOCITY * self.facing_dir as f32 * WALLJUMP_X_RELATIVE_STRENGTH;
                self.facing_dir *= -1;
                self.try_jumped_at = 0.0;
                self.wall_jumped_at = self.time;
                self.is_jumping = true;

                if self.is_dashing {
                    self.dash_dir.y -= 0.7;
                    self.dash_dir.x += 0.5 * self.facing_dir as f32;
                }
            } else if !self.within_grace(self.try_jumped_at, JUMP_BUFFER_TIME) {
                self.try_jumped_at = self.time;
            }
        }

        // Activate head slot equipment when jump pressed while truly in air
        // (not on ground, past coyote time, not on wall, not on ladder)
        let truly_in_air = !self.on_ground
            && !self.within_grace(self.last_on_ground, JUMP_COYOTE_TIME)
            && !on_wall
            && !self.is_on_ladder;

        if jump_pressed && truly_in_air && !self.is_swimming && !self.is_dashing {
            match self.head_slot {
                Some(ToolType::Glider) => {
                    if let Some(glider) = &self.tool_glider {
                        if glider.durability > 0.0 {
                            self.is_gliding = true;
                        }
                    }
                }
                Some(ToolType::Dash) => {
                    // Dash in the direction of the raycast (mouse direction)
                    self.manage_dash(controller.raycast_direction);
                }
                _ => {}
            }
        }

        // Jump release (variable jump height)
        if (!jump_held || self.velocity.y > 0.0)
            && !self.within_grace(self.spike_touched_at, SPIKE_IMMUNITY_COOLDOWN)
        {
            if self.velocity.y < 0.0 && self.is_jumping {
                self.velocity.y *= JUMP_RELEASE_REDUCTION;
                self.is_jumping = false;
            }
            self.gravity_reduction = 1.50;
        }

        // Ground detection
        self.just_landed = false;
        if self.on_ground {
            if !self.within_grace(self.last_action_at, DASHJUMP_COOLDOWN) {
                self.dashes = max_dashes;
            }
            self.last_on_ground = self.time;
            self.climb_stamina = CLIMB_STAMINA;
            if !self.just_landed && self.is_jumping {
                self.just_landed = true;
            }
            self.is_jumping = false;
        } else if self.is_swimming {
            self.is_jumping = false;
            self.climb_stamina = CLIMB_STAMINA;
            self.dashes = max_dashes;
        } else if self.is_on_ladder {
            // Reset dashes when on ladder
            self.is_jumping = false;
            self.dashes = max_dashes;
        }

        // Reset gliding state (will be set by use_tool if glider is active)
        self.is_gliding = false;

        let mut used_tool = self.use_tool(terrain, chests, active_bombs, controller, true);
        used_tool = self.use_tool(terrain, chests, active_bombs, controller, false) || used_tool;
        if !used_tool {
            self.is_mining = false;
        }

        // Glider stays active while jump is held, falling, in the air with glider equipped in head slot
        if !self.on_ground
            && !self.is_swimming
            && !self.is_dashing
            && jump_held
            && self.velocity.y > 0.0
        {
            if self.head_slot == Some(ToolType::Glider) {
                if let Some(glider) = &self.tool_glider {
                    if glider.durability > 0.0 {
                        self.is_gliding = true;
                    }
                }
            }
        }

        // Apply glider physics - clamp fall speed and consume durability
        if self.is_gliding {
            if let Some(glider) = &self.tool_glider {
                let max_fall = glider.max_fall_speed;
                if self.velocity.y > max_fall {
                    self.velocity.y = max_fall;
                }
            }
            // Consume durability over time while gliding
            if let Some(glider) = self.tool_glider.as_mut() {
                glider.durability -= dt;
                if glider.durability <= 0.0 {
                    self.break_tool(ToolType::Glider);
                    self.is_gliding = false;
                }
            }
        }

        // Apply dash velocity
        if self.within_grace(self.dashed_at, dash_time) {
            self.velocity = Vector2::new(
                self.dash_dir.x * DASH_VELOCITY,
                self.dash_dir.y * DASH_VELOCITY,
            );
        } else if self.is_on_ladder {
            // Ladder climbing - free movement up and down
            let ladder_speed = CLIMB_SPEED;
            let speed = ACCEL * dt;

            // Horizontal movement on ladder
            if input_dir.x != 0.0 {
                self.facing_dir = input_dir.x.signum() as i32;
                self.velocity.x = Self::move_toward(self.velocity.x, input_dir.x * SPEED, speed);
            } else {
                self.velocity.x = Self::move_toward(self.velocity.x, 0.0, speed);
            }

            // Vertical movement on ladder
            if input_dir.y != 0.0 {
                self.velocity.y = input_dir.y * ladder_speed;
            } else if self.velocity.y > 0.0 {
                // Only stop falling, not upward movement
                self.velocity.y = 0.0;
            }
        } else if self.is_swimming {
            let speed = ACCEL * dt;
            if input_dir.x != 0.0 {
                self.facing_dir = input_dir.x.signum() as i32;
                self.velocity.x =
                    Self::move_toward(self.velocity.x, input_dir.x * SWIM_SPEED, speed);
            }
            if input_dir.y != 0.0 {
                self.velocity.y =
                    Self::move_toward(self.velocity.y, input_dir.y * SWIM_SPEED, speed);
            }
        } else {
            // Horizontal movement
            let mut speed = ACCEL * dt;
            if self.within_grace(self.wall_jumped_at, WALLJUMP_EFFECT_TIME) {
                speed *= WALLJUMP_EFFECT_STRENGTH;
            }

            if self.within_grace(self.dashed_at, dash_extended_time) {
                speed /= (self.time - self.dashed_at) / dash_extended_time * dash_control_modifier;
            }
            if !self.within_grace(self.wall_jumped_at, WALLJUMP_LOCK_TIME) {
                // Regular movement
                if input_dir.x != 0.0 {
                    self.velocity.x =
                        Self::move_toward(self.velocity.x, input_dir.x * SPEED, speed);
                    if self.velocity.x.abs() < MIN_SPEED
                        && self.velocity.x.signum() == input_dir.x.signum()
                    {
                        self.velocity.x = self.velocity.x.signum() * MIN_SPEED;
                    }
                    self.facing_dir = input_dir.x.signum() as i32;
                } else {
                    self.velocity.x = Self::move_toward(self.velocity.x, 0.0, speed);
                }
            }
            if !self.is_jumping {
                self.velocity.x += topout_vel;
            }
        }
        self.apply_movement_and_collision(dt, terrain);
        if self.spike_check(terrain) {
            self.spike_touched_at = self.time;
        }
        self.last_velocity = self.velocity;

        self.calculated_selected_blocks(terrain, controller);
    }

    fn raycast(&self, start: Vector2, end: Vector2, terrain: &Terrain) -> RaycastResult {
        // DDA (Digital Differential Analyzer) ray-grid traversal algorithm
        // https://lodev.org/cgtutor/raycasting.html
        let pos_x = start.x;
        let pos_y = start.y;
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let distance2 = dx * dx + dy * dy;

        let buffer = 0.125;
        let delta_dist_x = (1.0 / dx).abs();
        let delta_dist_y = (1.0 / dy).abs();
        let mut map_x = start.x.floor() as i32;
        let mut map_y = start.y.floor() as i32;
        let mut hit_vertical;

        let mut side_dist_x;
        let mut side_dist_y;
        let step_x;
        let step_y;

        if dx < 0.0 {
            step_x = -1;
            side_dist_x = (pos_x - map_x as f32) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = (map_x as f32 + 1.0 - pos_x) * delta_dist_x;
        }
        if dy < 0.0 {
            step_y = -1;
            side_dist_y = (pos_y - map_y as f32) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = (map_y as f32 + 1.0 - pos_y) * delta_dist_y;
        }

        let mut vx = 0.0;
        let mut vy = 0.0;
        let mut ox = 0.0;
        let mut oy = 0.0;
        while vx * vx + vy * vy < distance2 {
            // Stop raycast at solid blocks or ladders (ladders can be targeted for placement)
            if terrain.solid_terrain_at(map_x, map_y) || terrain.at(map_x, map_y).is_ladder() {
                return RaycastResult {
                    final_position: Vector2::new(vx + start.x, vy + start.y),
                    last_free_position: Vector2::new(ox + start.x, oy + start.y),
                    hit: true,
                };
            }

            //jump to next map square, either in x-direction, or in y-direction
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                hit_vertical = true;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                hit_vertical = false;
            }

            ox = vx;
            oy = vy;
            if !hit_vertical {
                vy = (map_y + (1 - step_y) / 2) as f32
                    - start.y
                    - if end.y < start.y { buffer } else { 0.0 };
                vx = vy / (dy / dx);
            } else {
                vx = (map_x + (1 - step_x) / 2) as f32
                    - start.x
                    - if end.x < start.x { buffer } else { 0.0 };
                vy = (dy / dx) * vx;
            }
        }

        RaycastResult {
            final_position: end,
            last_free_position: end,
            hit: false,
        }
    }

    fn spike_check(&mut self, terrain: &Terrain) -> bool {
        if let Some(spike_type) = terrain.collides_with_spike_terrain(
            self.position.x,
            self.position.y,
            self.width,
            self.height,
        ) {
            if !self.within_grace(self.spike_touched_at, SPIKE_IMMUNITY_COOLDOWN) {
                self.health -= 1;
                if spike_type == Block::Stalagmite {
                    self.jump(0.7);
                } else {
                    self.jump(-0.3);
                }
                return true;
            }
        }
        return false;
    }

    fn apply_movement_and_collision(&mut self, dt: f32, terrain: &Terrain) {
        let buffer = 0.125;
        let little_buffer = 0.0001;
        let double_buffer = buffer * 2.0;
        let step_size = 1.0 / 8.0;

        let target_x = self.position.x + self.velocity.x * dt;
        let mut dx = self.position.x;
        let mut ox;
        while dx != target_x {
            ox = dx;
            dx = Self::move_toward(dx, target_x, step_size);
            if let Some((tx, _)) = terrain.collides_with_solid_terrain(
                dx,
                self.position.y + buffer,
                self.width,
                self.height - double_buffer,
            ) {
                if target_x > self.position.x {
                    dx = tx - self.width;
                } else {
                    dx = tx + 1.0;
                }
                if let Some((_, _)) = terrain.collides_with_solid_terrain(
                    dx + little_buffer,
                    self.position.y + buffer,
                    self.width - little_buffer * 2.0,
                    self.height - double_buffer,
                ) {
                    dx = ox;
                }
                self.velocity.x = 0.0;
                break;
            }
        }
        self.position.x = dx;

        let target_y = self.position.y + self.velocity.y * dt;
        let mut dy = self.position.y;
        let mut oy;
        while dy != target_y {
            oy = dy;
            dy = Self::move_toward(dy, target_y, step_size);
            if let Some((_, ty)) = terrain.collides_with_solid_terrain(
                self.position.x + buffer,
                dy,
                self.width - double_buffer,
                self.height,
            ) {
                if target_y > self.position.y {
                    self.on_ground = true;
                    dy = ty - self.height;
                } else {
                    dy = ty + 1.0;
                }
                if let Some((_, _)) = terrain.collides_with_solid_terrain(
                    dx + buffer,
                    dy + little_buffer,
                    self.width - double_buffer,
                    self.height - little_buffer * 2.0,
                ) {
                    dy = oy;
                }
                self.velocity.y = 0.0;
                break;
            }
        }
        self.position.y = dy;

        if self.on_ground {
            if let None = terrain.collides_with_solid_terrain(
                self.position.x + buffer,
                self.position.y + step_size,
                self.width - double_buffer,
                self.height,
            ) {
                self.on_ground = false
            }
        }

        self.apply_corner_correction(terrain);
    }

    fn apply_corner_correction(&mut self, _terrain: &Terrain) {
        let do_cc = false;
        if self.velocity.y >= 0.0 && do_cc {
            for _i in 0..CORNER_CORRECTION_AMOUNT {
                // Do nothing for now
            }
            return;
        }
        // TODO: Move a small amount to avoid clipping corners when jumping up
    }

    fn check_wall(&self, terrain: &Terrain) -> bool {
        let buffer = WALLJUMP_DETECT_DISTANCE;
        let y_buf = 0.0001;
        terrain
            .collides_with_solid_terrain(
                self.position.x + buffer * self.facing_dir as f32,
                self.position.y - y_buf,
                self.width,
                self.height - 2.0 * y_buf,
            )
            .is_some()
    }

    fn check_in_water(&self, terrain: &Terrain) -> bool {
        let center_x = (self.position.x + self.width / 2.0) as i32;
        let center_y = (self.position.y + self.height / 2.0) as i32;
        terrain.liquid_terrain_at(center_x, center_y)
    }

    fn check_on_ladder(&self, terrain: &Terrain) -> bool {
        // Check if player's center, near the bottom (but well inside the player) is on a ladder
        // Since larger Y is down, check at about 25% down from the top (or 75% of the way through the height)
        let center_x = (self.position.x + self.width / 2.0) as i32;
        let check_y = (self.position.y + self.height * 0.25) as i32;
        terrain.at(center_x, check_y).is_ladder()
    }

    fn exit_dash_handler(&mut self, terrain: &Terrain) {
        let mut wiggle_y = 0.0;
        let mut wiggle_x = 0.0;
        while let Some(_) = terrain.collides_with_solid_terrain(
            self.position.x + wiggle_x,
            self.position.y + wiggle_y,
            self.width,
            self.height,
        ) {
            wiggle_y = (wiggle_y.abs() + 0.1) * wiggle_y.signum() * -1.0;
            if wiggle_y.abs() > 2.0 {
                wiggle_y = 0.0;
                wiggle_x = (wiggle_x.abs() + 0.1) * wiggle_x.signum() * -1.0;
            }
        }

        self.position.x += wiggle_x;
        self.position.y += wiggle_y;
    }

    fn jump(&mut self, strength_mod: f32) {
        self.last_action_at = self.time;
        self.velocity.y = JUMP_VELOCITY * strength_mod;
        self.gravity_reduction = JUMP_GRAVITY_REDUCTION;
        self.try_jumped_at = 0.0;
        self.is_jumping = true;
        self.jumped_this_frame = true;

        if self.is_dashing {
            self.dash_dir.y -= 0.5;
            self.dash_dir.x *= 1.5;
        }
    }

    pub fn within_grace(&self, time: f32, period: f32) -> bool {
        time + period > self.time
    }

    fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
        if (target - current).abs() <= max_delta {
            target
        } else {
            current + max_delta * (target - current).signum()
        }
    }

    /// Breaks a tool while preserving its original level for repairs
    pub fn break_tool(&mut self, tool_type: ToolType) {
        match tool_type {
            ToolType::Pickaxe => {
                if let Some(pickaxe) = &self.tool_pickaxe {
                    let original_level = pickaxe.level.clone();
                    self.tool_pickaxe = Some(load_pick("broken"));
                    self.tool_pickaxe.as_mut().unwrap().level = original_level;
                }
            }
            ToolType::Dash => {
                if let Some(dash) = &self.tool_dash {
                    let original_level = dash.level.clone();
                    self.tool_dash = Some(load_dash("broken"));
                    self.tool_dash.as_mut().unwrap().level = original_level;
                }
            }
            ToolType::Glider => {
                if let Some(glider) = &self.tool_glider {
                    let original_level = glider.level.clone();
                    self.tool_glider = Some(load_glider("broken"));
                    self.tool_glider.as_mut().unwrap().level = original_level;
                }
            }
            _ => {}
        }
    }

    fn use_tool(
        &mut self,
        terrain: &mut Terrain,
        chests: &mut std::collections::HashMap<(i32, i32), crate::inventory::Inventory>,
        active_bombs: &mut Vec<crate::world::ActiveBomb>,
        controller: &Controller,
        left_hand: bool,
    ) -> bool {
        let (hand, held, pressed) = if left_hand {
            (
                self.left_hand,
                controller.left_hand_held,
                controller.left_hand_pressed,
            )
        } else {
            (
                self.right_hand,
                controller.right_hand_held,
                controller.right_hand_pressed,
            )
        };
        match (hand, held, pressed) {
            (Some(ToolType::Dash), _, _) => {
                // Dash is now activated via head slot with jump key while in air
                false
            }
            (Some(ToolType::Pickaxe), true, _) => self.manage_pickaxe(terrain, chests, left_hand),
            (Some(ToolType::Lamp), _, _) => {
                // Lamp is passive, no action needed
                false
            }
            (Some(ToolType::Glider), _, _) => {
                // Glider is now activated via head slot with jump key
                false
            }
            (Some(ToolType::PlaceBlock(blk)), _, true) => {
                self.try_place_block(terrain, chests, active_bombs, blk, left_hand);
                true
            }
            _ => false,
        }
    }

    fn manage_pickaxe(
        &mut self,
        terrain: &mut Terrain,
        chests: &mut std::collections::HashMap<(i32, i32), crate::inventory::Inventory>,
        left_hand: bool,
    ) -> bool {
        let tile = if left_hand {
            self.raycast_left_tile
        } else {
            self.raycast_right_tile
        };

        let raycast_end = if left_hand {
            self.raycast_left_end
        } else {
            self.raycast_right_end
        };

        if let Some(bt) = tile {
            let block = terrain.at(bt.0 as i32, bt.1 as i32);
            if block == Block::Air {
                self.is_mining = false;
                return false;
            }
            if !(self.is_swimming || self.is_climbing || self.is_dashing || self.is_sliding) {
                let block_durability = block.durability();
                if !self.is_mining {
                    self.is_mining = true;
                    self.started_mining_at = self.time;
                    self.last_pickaxe_sound = self.time;
                    self.currently_mining = Some(bt);
                } else {
                    self.facing_dir =
                        (raycast_end.x - (self.position.x + self.width / 2.0)).signum() as i32;
                    if let Some(ot) = self.currently_mining {
                        if ot.0 != bt.0 || ot.1 != bt.1 {
                            self.last_pickaxe_sound = self.time;
                            self.started_mining_at = self.time;
                            self.currently_mining = Some(bt);
                        }
                    }
                }

                if !self.within_grace(self.started_mining_at, block_durability) {
                    if let Some(pickaxe) = self.tool_pickaxe.as_mut() {
                        pickaxe.durability -= block_durability;
                        if pickaxe.durability <= 0.0 {
                            self.break_tool(ToolType::Pickaxe);
                        }
                    }
                    self.is_mining = false;

                    // Break the block (handles both single-tile and multi-tile blocks)
                    let block_x = bt.0 as i32;
                    let block_y = bt.1 as i32;
                    if let Some((broken_block, _, _)) = terrain.break_multi_tile(block_x, block_y) {
                        // Remove chest inventory if breaking a chest (contents are lost)
                        if matches!(broken_block, Block::WoodenChest) {
                            chests.remove(&(block_x, block_y));
                        }

                        // Add drops to inventory
                        if let Some((item_type, amount)) = broken_block.get_drops() {
                            self.inventory.add(item_type, amount);
                        }
                    }
                }
                return true;
            }
        }
        false
    }

    fn manage_dash(&mut self, dir: Vector2) {
        if self.dashes > 0 && !self.within_grace(self.last_action_at, DASHJUMP_COOLDOWN) {
            if let Some(dash) = self.tool_dash.as_mut() {
                dash.durability -= 1.0;
                if dash.durability <= 0.0 {
                    self.break_tool(ToolType::Dash);
                }
            }

            self.last_action_at = self.time;
            self.dashes -= 1;
            self.dashed_at = self.time;
            self.dash_dir = dir;

            if self.climb_stamina < CLIMB_STAMINA / 2.0 {
                self.climb_stamina = CLIMB_STAMINA / 2.0;
            }

            if self.on_ground {
                self.dash_dir.y = self.dash_dir.y.min(0.0);
            }

            if self.dash_dir.x == 0.0 && self.dash_dir.y == 0.0 {
                self.dash_dir.x = self.facing_dir as f32;
            }

            self.dash_dir = self.dash_dir.normalized();
        }
    }
}
