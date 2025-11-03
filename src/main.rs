use raylib::prelude::*;
use texture_manager_macro::generate_texture_manager;

generate_texture_manager!("assets");
const PIXELS_PER_WORLD_UNIT: f32 = 12.0; // 24

mod render;
mod world;
mod terrain;
mod terrain_generator;
mod player;
mod controller;

use world::WorldState;
use controller::Controller;

fn update_controller_raycast_for_mouse(
    rl: &RaylibHandle,
    camera: &Camera2D,
    controller: &mut Controller,
    player: &player::Player,
) {
    let mouse_screen = rl.get_mouse_position();
    let mouse_world = rl.get_screen_to_world2D(mouse_screen, camera);

    let dx = mouse_world.x - (player.position.x + player.width / 2.0) * PIXELS_PER_WORLD_UNIT;
    let dy = mouse_world.y - (player.position.y + player.height / 2.0) * PIXELS_PER_WORLD_UNIT;
    let distance = (dx * dx + dy * dy).sqrt();

    if distance > 0.0001 {
        controller.set_raycast_direction(Vector2::new(dx / distance, dy / distance));
    }
}

fn smooth_camera_to_target(
    camera: &mut Camera2D,
    camera_velocity: &mut Vector2,
    target_x: f32,
    target_y: f32,
    dt: f32,
    smooth_time: f32,
) {
    let max_speed = 10000.0;

    // Exponential smoothing calculation
    let omega = 2.0 / smooth_time;
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    // X-axis smoothing
    let change_x = camera.target.x - target_x;
    let temp_x = (camera_velocity.x + omega * change_x) * dt;
    camera_velocity.x = (camera_velocity.x - omega * temp_x) * exp;

    // Clamp velocity to max speed
    if camera_velocity.x.abs() > max_speed {
        camera_velocity.x = camera_velocity.x.signum() * max_speed;
    }

    camera.target.x = target_x + (change_x + temp_x) * exp;

    // Y-axis smoothing
    let change_y = camera.target.y - target_y;
    let temp_y = (camera_velocity.y + omega * change_y) * dt;
    camera_velocity.y = (camera_velocity.y - omega * temp_y) * exp;

    // Clamp velocity to max speed
    if camera_velocity.y.abs() > max_speed {
        camera_velocity.y = camera_velocity.y.signum() * max_speed;
    }

    camera.target.y = target_y + (change_y + temp_y) * exp;
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1600, 900)
        .title("JGame")
        .build();

    rl.set_target_fps(60);

    let textures = TextureManager::load(&mut rl, &thread);

    // Load player shader
    let mut player_shader = rl.load_shader(
        &thread,
        Some("shaders/playerShader.vert"),
        Some("shaders/playerShader.frag")
    );

    // Get shader uniform locations
    let loc_original_0 = player_shader.get_shader_location("original_0");
    let loc_replace_0 = player_shader.get_shader_location("replace_0");
    let loc_exhustion = player_shader.get_shader_location("exhustion");
    let loc_whiteout = player_shader.get_shader_location("whiteout");

    let mut world_state = WorldState::new();
    let mut controller = Controller::new();

    let mut camera_velocity = Vector2::zero();
    let mut camera = Camera2D {
        target: Vector2::new(
            world_state.player.position.x * PIXELS_PER_WORLD_UNIT,
            world_state.player.position.y * PIXELS_PER_WORLD_UNIT,
        ),
        offset: Vector2::new(rl.get_screen_width() as f32 / 2.0, rl.get_screen_height() as f32 / 2.0),
        rotation: 0.0,
        zoom: 1.0,
    };


    let mut debug_enabled = true;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // Toggle debug mode with '/' key
        if rl.is_key_pressed(KeyboardKey::KEY_SLASH) {
            debug_enabled = !debug_enabled;
        }

        controller.update(&rl);
        update_controller_raycast_for_mouse(&rl, &camera, &mut controller, &world_state.player);

        world_state.update(dt, &controller);

        smooth_camera_to_target(
            &mut camera,
            &mut camera_velocity,
            world_state.player.position.x * PIXELS_PER_WORLD_UNIT,
            world_state.player.position.y * PIXELS_PER_WORLD_UNIT,
            dt,
            0.12
        );

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut d2 = d.begin_mode2D(camera);
            render::render(
                &mut d2,
                &world_state,
                &textures,
                &mut player_shader,
                (loc_original_0, loc_replace_0, loc_exhustion, loc_whiteout)
            );
        }

        // Draw HUD
        if debug_enabled {
            d.draw_text(&format!("Pos: ({:.2}, {:.2})", world_state.player.position.x, world_state.player.position.y), 10, 10, 20, Color::DARKGRAY);
            d.draw_text("Controls: WASD/Arrows=Move, Space=Jump, C=Climb, Shift/X=Dash, Toggle Controls=/", 500, 10, 16, Color::BLACK);
            d.draw_text(&format!("FPS: {}", d.get_fps()), 1500, 10, 20, Color::GRAY);

            d.draw_text(&format!("Vel: ({:.2}, {:.2})", world_state.player.velocity.x, world_state.player.velocity.y), 10, 35, 20, Color::DARKGRAY);
            d.draw_text(&format!("On Ground: {}   Is Climbing {}   Is Sliding {}   Is Dashing {}   Is Swimming {}",
                    world_state.player.on_ground, world_state.player.is_climbing,
                    world_state.player.is_sliding, world_state.player.is_dashing,
                    world_state.player.is_swimming), 10, 60, 20, Color::DARKGRAY);
        }
    }
}
