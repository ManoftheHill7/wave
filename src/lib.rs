use std::sync::OnceLock;
use texture_manager_macro::generate_texture_manager;

generate_texture_manager!("assets/art");

static PIXELS_PER_WORLD_UNIT: OnceLock<f32> = OnceLock::new();

pub fn pixels_per_world_unit() -> f32 {
    *PIXELS_PER_WORLD_UNIT.get_or_init(|| {
        std::env::var("PPW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24.0)
    })
}

// Export public modules that can be used by other binaries
pub mod controller;
pub mod crafting_screen;
pub mod game_screen;
pub mod inventory;
pub mod inventory_screen;
pub mod lighting;
pub mod maps;
pub mod menu_screen;
pub mod player;
pub mod save_load;
pub mod terrain;
pub mod terrain_generator;
pub mod tools;
pub mod world;

// Re-export commonly used types
pub use controller::Controller;
pub use game_screen::GameScreen;
pub use menu_screen::MenuScreen;
pub use world::WorldState;

// Re-export map system types
pub use maps::{
    EdgeConstraint, EdgeType, Map, MapEdges, MapOrientation, MapQuery, MapSet, MapStats,
};

// Export types and structs from main
use raylib::prelude::*;
use std::cell::RefCell;

pub struct GameContext {
    pub textures: TextureManager,
    pub render_state: GameRenderState,
    pub controller: Controller,
    pub world_state: WorldState,
    pub debug_enabled: bool,
    pub updating: bool,
}

pub struct GameRenderState {
    pub player_shader: RefCell<Shader>,
    pub shader_locs: ShaderLocs,
    pub lighting_shader: RefCell<Shader>,
    pub lighting_shader_locs: LightingShaderLocs,
}

pub type ShaderLocs = (i32, i32, i32, i32);

pub struct LightingShaderLocs {
    pub ambient_darkness: i32,
    pub texture_size: i32,
}

impl GameContext {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, map_path: Option<String>) -> Self {
        let textures = TextureManager::load(rl, thread);

        let player_shader = rl.load_shader(
            thread,
            Some("shaders/playerShader.vert"),
            Some("shaders/playerShader.frag"),
        );

        let loc_original_0 = player_shader.get_shader_location("original_0");
        let loc_replace_0 = player_shader.get_shader_location("replace_0");
        let loc_exhustion = player_shader.get_shader_location("exhustion");
        let loc_whiteout = player_shader.get_shader_location("whiteout");

        let lighting_shader = rl.load_shader(
            thread,
            Some("shaders/lighting.vert"),
            Some("shaders/lighting.frag"),
        );

        let lighting_locs = LightingShaderLocs {
            ambient_darkness: lighting_shader.get_shader_location("ambientDarkness"),
            texture_size: lighting_shader.get_shader_location("textureSize"),
        };

        let mut world_state = WorldState::new();

        if let Some(path) = map_path {
            match terrain_generator::MapGenerator::from_image(&path) {
                Ok(map_gen) => {
                    let center_x = map_gen.map_width() as f32 / 2.0;
                    let center_y = map_gen.map_height() as f32 / 2.0;
                    world_state.player = player::Player::new(center_x, center_y);
                    world_state.terrain.generator = terrain_generator::Generator::Map(map_gen);
                }
                Err(e) => {
                    eprintln!("Failed to load map: {}", e);
                    eprintln!("Falling back to procedural generation");
                }
            }
        }

        world_state.update(0.0, &Controller::new());

        GameContext {
            textures,
            render_state: GameRenderState {
                player_shader: RefCell::new(player_shader),
                shader_locs: (loc_original_0, loc_replace_0, loc_exhustion, loc_whiteout),
                lighting_shader: RefCell::new(lighting_shader),
                lighting_shader_locs: lighting_locs,
            },
            controller: Controller::new(),
            world_state,
            debug_enabled: true,
            updating: true,
        }
    }

    pub fn handle_debug_input(&mut self, rl: &RaylibHandle) {
        if rl.is_key_pressed(KeyboardKey::KEY_SLASH) {
            self.debug_enabled = !self.debug_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_APOSTROPHE) {
            self.world_state.ghost_mode = !self.world_state.ghost_mode;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.updating = !self.updating;
        }

        // Save/Load with F5/F9
        if rl.is_key_pressed(KeyboardKey::KEY_F5) {
            match save_load::save_game(&self.world_state, 0) {
                Ok(()) => println!("✓ Game saved successfully!"),
                Err(e) => eprintln!("✗ Failed to save game: {}", e),
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F9) {
            if save_load::save_exists(0) {
                match save_load::load_game(0) {
                    Ok(save_data) => {
                        save_load::apply_save_data(&mut self.world_state, save_data);
                        println!("✓ Game loaded successfully!");
                    }
                    Err(e) => eprintln!("✗ Failed to load game: {}", e),
                }
            } else {
                println!("✗ No save file found!");
            }
        }

        // Delete all saves with F1
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            match save_load::delete_all_saves() {
                Ok(()) => println!("✓ All save data deleted!"),
                Err(e) => eprintln!("✗ Failed to delete saves: {}", e),
            }
        }
    }
}
