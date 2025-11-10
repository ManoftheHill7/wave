use crate::controller::Controller;
use crate::inventory::{Inventory, ItemType};
use crate::terrain::{Block, Terrain};
use crate::tools::*;
use raylib::prelude::*;

pub const ACCEL: f32 = 50.0;
pub const SPEED: f32 = 12.0;
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
pub const WALLJUMP_DETECT_DISTANCE: f32 = 0.05;

pub const BASE_HEIGHT: f32 = 2.0;
pub const DASH_HEIGHT: f32 = 0.9;
pub const BASE_WIDTH: f32 = 1.3;
pub const DASH_WIDTH: f32 = 0.9;

pub const WATER_BUOYANCY: f32 = -GRAVITY / 4.0;
pub const WATER_DRAG: f32 = 0.9;
pub const SWIM_SPEED: f32 = 8.0;
pub const SWIM_EXIT_TIME: f32 = 0.10;

pub const MAX_RAYCAST_PICKAXE: f32 = 2.0;
pub const MAX_RAYCAST_HOOK: f32 = 10.0;
pub const MAX_RAYCAST_SPEAR: f32 = 3.0;

pub const INVENTORY_STARTING_WEIGHT: f32 = 100.0;
pub const STARTING_HEALTH: i32 = 12; // 4 frames of heart * 3 hearts
pub const MAX_BREATH_HOLD: f32 = 10.0;

pub const SPIKE_IMMUNITY_COOLDOWN: f32 = 0.3;

#[derive(Debug)]
struct RaycastResult {
    final_position: Vector2,
    last_free_position: Vector2,
    hit: bool,
}

pub struct Player {
    pub position: Vector2,
    pub velocity: Vector2,
    pub height: f32,
    pub width: f32,
    pub facing_dir: i32,

    pub on_ground: bool,
    pub is_jumping: bool,
    pub is_dashing: bool,
    pub is_climbing: bool,
    pub is_sliding: bool,
    pub is_swimming: bool,
    pub just_landed: bool,
    pub just_finished_dashing: bool,

    pub last_on_ground: f32,
    pub last_in_water: f32,
    pub try_jumped_at: f32,
    pub wall_jumped_at: f32,
    pub spike_touched_at: f32,
    pub dashed_at: f32,
    pub last_action_at: f32,
    pub time: f32,

    pub dashes: i32,
    pub climb_stamina: f32,

    pub gravity_reduction: f32,

    pub dash_dir: Vector2,

    pub landing_speed: Vector2,
    pub last_velocity: Vector2,

    pub raycast_max_length: f32,
    pub raycast_end_pos: Vector2,
    pub raycast_hit_tile: Option<(f32, f32)>,
    pub raycast_last_free_tile: Option<(f32, f32)>,

    pub health: i32,
    pub breath: f32,

    pub inventory: Inventory,
    pub place_block_type: Option<ItemType>,

    pub selected_tool: Option<ToolType>,
    pub tool_dash: Option<ToolDash>,
    pub tool_pickaxe: Option<ToolPickaxe>,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        let initial_dash = Some(load_dash("basic"));
        Player {
            position: Vector2::new(x, y),
            velocity: Vector2::zero(),
            height: BASE_HEIGHT,
            width: BASE_WIDTH,
            facing_dir: 1,

            on_ground: false,
            is_jumping: false,
            is_dashing: false,
            is_climbing: false,
            is_sliding: false,
            is_swimming: false,
            just_landed: false,
            just_finished_dashing: false,

            last_on_ground: -999.0,
            last_in_water: -999.0,
            try_jumped_at: -999.0,
            wall_jumped_at: -999.0,
            spike_touched_at: -999.0,
            dashed_at: -999.0,
            last_action_at: -999.0,
            time: 0.0,

            dashes: 0,
            climb_stamina: CLIMB_STAMINA,

            gravity_reduction: 1.0,
            dash_dir: Vector2::zero(),
            landing_speed: Vector2::zero(),
            last_velocity: Vector2::zero(),

            raycast_max_length: MAX_RAYCAST_SPEAR,
            raycast_hit_tile: None,
            raycast_last_free_tile: None,
            raycast_end_pos: Vector2::zero(),

            health: STARTING_HEALTH,
            breath: MAX_BREATH_HOLD,

            inventory: Inventory::new(INVENTORY_STARTING_WEIGHT),
            place_block_type: None,

            selected_tool: Some(ToolType::Dash),
            tool_dash: initial_dash,
            tool_pickaxe: None,
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

    pub fn calculated_selected_blocks(&mut self, terrain: &Terrain, controller: &Controller) {
        self.raycast_max_length = MAX_RAYCAST_HOOK;
        let raycast_start = self.position + Vector2::new(self.width / 2.0, self.height / 2.0);
        let rayresult = self.raycast(
            raycast_start,
            raycast_start + controller.raycast_direction * self.raycast_max_length,
            terrain,
        );
        self.raycast_end_pos = rayresult.final_position;
        if rayresult.hit {
            self.raycast_hit_tile = Some((rayresult.final_position.x, rayresult.final_position.y));
            self.raycast_last_free_tile = Some((
                rayresult.last_free_position.x,
                rayresult.last_free_position.y,
            ));
        } else {
            self.raycast_hit_tile = None;
            self.raycast_last_free_tile = None;
        };
    }

    pub fn try_place_block(&mut self, terrain: &mut Terrain) {
        if let Some(item_type) = self.place_block_type {
            if let Some((free_x, free_y)) = self.raycast_last_free_tile {
                if let Some(block) = crate::terrain::Block::from_item_type(item_type) {
                    let taken = self.inventory.take(item_type, 1);
                    if taken > 0 {
                        terrain.set(free_x.floor() as i32, free_y.floor() as i32, block);
                        if self.inventory.count(item_type) == 0 {
                            self.place_block_type = None;
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32, terrain: &Terrain, controller: &Controller) {
        let jump_pressed = controller.jump_pressed;
        let jump_held = controller.jump_held;
        let climb_pressed = controller.climb_pressed;
        let input_dir = controller.input_dir;

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
        } else {
            self.breath = MAX_BREATH_HOLD;
        }

        // Apply gravity
        if !self.on_ground {
            if self.is_swimming {
                self.velocity.y += WATER_BUOYANCY * dt;
                self.velocity.x *= WATER_DRAG;
                self.velocity.y *= WATER_DRAG;
            } else {
                self.velocity.y += GRAVITY * self.gravity_reduction * dt;
            }

            self.landing_speed = self.velocity;
        }
        self.velocity.y = self.velocity.y.clamp(-TERMINAL_VELOCITY, TERMINAL_VELOCITY);

        if !self.is_swimming && self.within_grace(self.last_in_water, SWIM_EXIT_TIME) {
            self.is_swimming = true;
        }

        let on_wall = self.check_wall(terrain);

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

        // Jump release (variable jump height)
        if (!jump_held || self.velocity.y > 0.0) && !self.within_grace(self.spike_touched_at, SPIKE_IMMUNITY_COOLDOWN) {
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
        }

        if controller.use_tool_pressed {
            self.use_tool(terrain, controller);
        }

        // Apply dash velocity
        if self.within_grace(self.dashed_at, dash_time) {
            self.velocity = Vector2::new(
                self.dash_dir.x * DASH_VELOCITY,
                self.dash_dir.y * DASH_VELOCITY,
            );
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
                    self.facing_dir = input_dir.x.signum() as i32;
                } else {
                    self.velocity.x = Self::move_toward(self.velocity.x, 0.0, speed);
                }
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
            if terrain.solid_terrain_at(map_x, map_y) {
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
            self.height) {
            if !self.within_grace(self.spike_touched_at, SPIKE_IMMUNITY_COOLDOWN) {
                self.health -= 1;
                if spike_type == Block::Stalagmite {
                    self.jump(0.7);
                } else {
                    self.jump(-0.3);
                }
                return true
            }
        }
        return false
    }

    fn apply_movement_and_collision(&mut self, dt: f32, terrain: &Terrain) {
        let buffer = 0.125;
        let double_buffer = buffer * 2.0;
        let step_size = 1.0 / 8.0;

        let target_x = self.position.x + self.velocity.x * dt;
        let mut dx = self.position.x;
        let mut ox;
        while dx != target_x {
            ox = dx;
            dx = Self::move_toward(dx, target_x, step_size);
            if let Some((_, _)) = terrain.collides_with_solid_terrain(
                dx,
                self.position.y + buffer,
                self.width,
                self.height - double_buffer,
            ) {
                dx = ox;
                self.velocity.x = 0.0;
                break;
            }
        }
        self.position.x = dx;

        let target_y = self.position.y + self.velocity.y * dt;
        let mut dy = self.position.y;
        while dy != target_y {
            dy = Self::move_toward(dy, target_y, step_size);
            if let Some((_, ty)) = terrain.collides_with_solid_terrain(
                self.position.x + buffer,
                dy,
                self.width - double_buffer,
                self.height,
            ) {
                if self.velocity.y > 0.0 {
                    self.on_ground = true;
                    dy = ty - self.height;
                } else {
                    dy = ty + 1.0;
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
        terrain
            .collides_with_solid_terrain(
                self.position.x + buffer * self.facing_dir as f32,
                self.position.y + buffer,
                self.width,
                self.height - 2.0 * buffer,
            )
            .is_some()
    }

    fn check_in_water(&self, terrain: &Terrain) -> bool {
        let center_x = (self.position.x + self.width / 2.0) as i32;
        let center_y = (self.position.y + self.height / 2.0) as i32;
        terrain.liquid_terrain_at(center_x, center_y)
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

    fn use_tool(&mut self, _terrain: &Terrain, controller: &Controller) {
        match self.selected_tool {
            Some(ToolType::Dash) => self.manage_dash(controller.raycast_direction),
            Some(ToolType::Pickaxe) => (),
            None => (),
        }
    }

    fn manage_dash(&mut self, dir: Vector2) {
        if self.dashes > 0 && !self.within_grace(self.last_action_at, DASHJUMP_COOLDOWN) {
            self.tool_dash.as_mut().unwrap().durability -= 1.0;
            if self.tool_dash.as_mut().unwrap().durability <= 0.0 {
                self.tool_dash = Some(load_dash("broken"));
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
