use raylib::prelude::*;
use screen_manager::ScreenManager;
use std::cell::RefCell;
use std::sync::OnceLock;
use texture_manager_macro::generate_texture_manager;

generate_texture_manager!("assets");

static PIXELS_PER_WORLD_UNIT: OnceLock<f32> = OnceLock::new();

pub fn pixels_per_world_unit() -> f32 {
    *PIXELS_PER_WORLD_UNIT.get_or_init(|| {
        std::env::var("PPW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24.0)
    })
}

mod controller;
mod game_screen;
mod inventory;
mod inventory_screen;
mod player;
mod terrain;
mod terrain_generator;
mod tools;
mod world;

use controller::Controller;
use game_screen::GameScreen;
use world::WorldState;

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
}

pub type ShaderLocs = (i32, i32, i32, i32);

impl GameContext {
    fn new(rl: &mut RaylibHandle, thread: &RaylibThread, map_path: Option<String>) -> Self {
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
            },
            controller: Controller::new(),
            world_state,
            debug_enabled: true,
            updating: true,
        }
    }

    fn handle_global_input(&mut self, rl: &RaylibHandle) {
        if rl.is_key_pressed(KeyboardKey::KEY_SLASH) {
            self.debug_enabled = !self.debug_enabled;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_APOSTROPHE) {
            self.world_state.ghost_mode = !self.world_state.ghost_mode;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.updating = !self.updating;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let map_path = args.get(1).cloned();

    let (mut rl, thread) = raylib::init().size(1600, 900).title("JGame").build();

    rl.set_target_fps(60);

    let mut ctx = GameContext::new(&mut rl, &thread, map_path);
    let mut manager = ScreenManager::new(Box::new(GameScreen::new(&ctx)), &mut ctx);

    use crate::inventory::ItemType;
    ctx.world_state.player.inventory.add(ItemType::Stone, 3);
    ctx.world_state.player.inventory.add(ItemType::Dirt, 10);

    while !rl.window_should_close() && !manager.is_empty() {
        let dt = rl.get_frame_time();

        ctx.handle_global_input(&rl);
        ctx.controller.update(&rl);

        manager.update(dt, &mut ctx);
        manager.render(&mut rl, &thread, &ctx);
    }
}
