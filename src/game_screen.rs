use crate::inventory_screen::InventoryScreen;
use crate::player::{Player, SPIKE_IMMUNITY_COOLDOWN};
use crate::terrain::{Block, Terrain, CELL_RESOLUTION, NO_LIQUID_THRESHOLD};
use crate::{pixels_per_world_unit, GameContext, Neighbors, ShaderLocs};
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

// Maps to uniforms (original_0, replace_0)
const DEFAULT_SPRITE_PALLETTE: &[f32; 4] = &[172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0];
const COLOR_PALETTES: &[[f32; 4]] = &[
    [0.0, 0.5, 1.0, 1.0],                             // Blue scarf
    [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // Red scarf
    [1.0, 0.0, 1.0, 1.0],                             // Pink scarf
    [0.6, 0.9, 0.3, 1.0],                             // Green scarf
];

fn smooth_camera_to_target(
    camera: &mut Camera2D,
    camera_velocity: &mut Vector2,
    target_x: f32,
    target_y: f32,
    dt: f32,
    smooth_time: f32,
) {
    let max_speed = 10000.0;

    let omega = 2.0 / smooth_time;
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    let change_x = camera.target.x - target_x;
    let temp_x = (camera_velocity.x + omega * change_x) * dt;
    camera_velocity.x = (camera_velocity.x - omega * temp_x) * exp;

    if camera_velocity.x.abs() > max_speed {
        camera_velocity.x = camera_velocity.x.signum() * max_speed;
    }

    camera.target.x = target_x + (change_x + temp_x) * exp;

    let change_y = camera.target.y - target_y;
    let temp_y = (camera_velocity.y + omega * change_y) * dt;
    camera_velocity.y = (camera_velocity.y - omega * temp_y) * exp;

    if camera_velocity.y.abs() > max_speed {
        camera_velocity.y = camera_velocity.y.signum() * max_speed;
    }

    camera.target.y = target_y + (change_y + temp_y) * exp;
}

fn block_texture<'a>(block: Block, textures: &'a crate::TextureManager) -> &'a Texture2D {
    match block {
        Block::Dirt => &textures.tiles.dirt,
        Block::Stone => &textures.tiles.stone,
        Block::Grass => &textures.tiles.grass,
        Block::Sand => &textures.tiles.sand,
        Block::Lava => &textures.tiles.lava,
        Block::Water => &textures.tiles.water,
        Block::Log => &textures.tiles.log,
        Block::Leaf => &textures.tiles.leaves,

        Block::Air => &textures.fallback,
        Block::Tide => &textures.fallback,
        Block::Stalagmite => &textures.fallback,
        Block::Stalactite => &textures.fallback,
    }
}

fn render_tile(
    d: &mut RaylibDrawHandle,
    x: f32,
    y: f32,
    texture: &Texture2D,
    src_rect: Option<Rectangle>,
) {
    let source = src_rect
        .unwrap_or_else(|| Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32));

    d.draw_texture_pro(
        texture,
        source,
        Rectangle::new(
            x * pixels_per_world_unit(),
            y * pixels_per_world_unit(),
            pixels_per_world_unit(),
            pixels_per_world_unit(),
        ),
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );
}

fn render_water(d: &mut RaylibDrawHandle, terrain: &Terrain, x: f32, y: f32) {
    let ld = terrain.liquid_at(x, y);
    let amount = ld.volume;
    if amount > NO_LIQUID_THRESHOLD {
        let volume_clamped = amount.min(1.0).max(0.0);

        let intensity = volume_clamped.powf(0.5);
        let r = (100.0 * (1.0 - intensity)) as u8;
        let g = (150.0 * (1.0 - intensity) + 100.0 * intensity) as u8;
        let b = (255.0 * (0.5 + 0.5 * intensity)) as u8;
        let a = (255.0 * intensity.max(0.3)) as u8;

        let cell_size: i32 = (pixels_per_world_unit() / CELL_RESOLUTION as f32) as i32;
        let fill_height: i32 = if ld.flow_down {
            cell_size
        } else {
            (cell_size as f32 * volume_clamped) as i32
        };
        let y_offset: i32 = cell_size - fill_height;

        d.draw_rectangle(
            (x * pixels_per_world_unit()) as i32,
            (y * pixels_per_world_unit()) as i32 + y_offset,
            cell_size,
            fill_height,
            Color::new(r, g, b, a),
        );
    }
}

fn render_terrain(
    d: &mut RaylibDrawHandle,
    terrain: &Terrain,
    px: i32,
    py: i32,
    textures: &crate::TextureManager,
) {
    let range = 900 / pixels_per_world_unit() as i32;
    for x in (px - range)..(px + range) {
        for y in (py - range)..(py + range) {
            let block = terrain.at(x, y);

            if block.is_solid() {
                // Check if this is a J11 tileset block
                if block == Block::Stone {
                    // Gather 8-directional neighbors for autotiling
                    let neighbors = Neighbors {
                        up: terrain.solid_terrain_at(x, y - 1),
                        up_right: terrain.solid_terrain_at(x + 1, y - 1),
                        right: terrain.solid_terrain_at(x + 1, y),
                        down_right: terrain.solid_terrain_at(x + 1, y + 1),
                        down: terrain.solid_terrain_at(x, y + 1),
                        down_left: terrain.solid_terrain_at(x - 1, y + 1),
                        left: terrain.solid_terrain_at(x - 1, y),
                        up_left: terrain.solid_terrain_at(x - 1, y - 1),
                    };

                    let src_rect = textures.tiles.stone_tile.get_tile_rect(x, y, &neighbors);
                    render_tile(
                        d,
                        x as f32,
                        y as f32,
                        textures.tiles.stone_tile.texture(),
                        Some(src_rect),
                    );
                } else {
                    let texture = block_texture(block, textures);
                    render_tile(d, x as f32, y as f32, texture, None);
                }
            } else if block == Block::Stalagmite || block == Block::Stalactite {
                let neighbors = Neighbors {
                    up: terrain.spike_at(x, y - 1),
                    up_right: terrain.spike_at(x + 1, y - 1),
                    right: terrain.spike_at(x + 1, y),
                    down_right: terrain.spike_at(x + 1, y + 1),
                    down: terrain.spike_at(x, y + 1),
                    down_left: terrain.spike_at(x - 1, y + 1),
                    left: terrain.spike_at(x - 1, y),
                    up_left: terrain.spike_at(x - 1, y - 1),
                };
                let src_rect = textures.tiles.spikes.get_tile_rect(x, y, &neighbors);
                let final_src_rect = if block == Block::Stalactite {
                    Rectangle::new(src_rect.x, src_rect.y + src_rect.height, src_rect.width, -src_rect.height)
                } else {
                    src_rect
                };
                render_tile(d, x as f32, y as f32, &textures.tiles.spikes.texture(), Some(final_src_rect));
            } else {
                for cell_y in 0..CELL_RESOLUTION {
                    for cell_x in 0..CELL_RESOLUTION {
                        render_water(
                            d,
                            terrain,
                            x as f32 + cell_x as f32 / CELL_RESOLUTION as f32,
                            y as f32 + cell_y as f32 / CELL_RESOLUTION as f32,
                        );
                    }
                }
            }
        }
    }
}

fn render_player(
    d: &mut RaylibDrawHandle,
    player: &Player,
    textures: &crate::TextureManager,
    shader: &mut Shader,
    shader_locs: ShaderLocs,
) {
    macro_rules! animate {
        ($frames:expr, $frame_length:expr) => {
            &$frames[(player.time / $frame_length) as usize % $frames.len()]
        };
    }

    const WALK_FRAME_LENGTH: f32 = 0.1;
    const FALLING_FRAME_LENGTH: f32 = 0.2;
    const IDLE_FRAME_LENGTH: f32 = 0.25;
    const SWIMMING_FRAME_LENGTH: f32 = 0.18;
    const MINING_FRAME_LENGTH: f32 = 0.1;

    let mut rotation = 0.0;

    let pt = &textures.player.sammi;
    let texture = if player.is_dashing {
        rotation = player.velocity.y.atan2(player.velocity.x).to_degrees();
        animate!(&pt.dash, WALK_FRAME_LENGTH)
    } else if player.is_mining {
        animate!(pt.mining, MINING_FRAME_LENGTH)
    } else if player.is_climbing {
        if player.velocity.y != 0.0 {
            animate!(pt.climb, WALK_FRAME_LENGTH)
        } else {
            &pt.climb[0]
        }
    } else if player.is_swimming {
        animate!(pt.swimming, SWIMMING_FRAME_LENGTH)
    } else if player.is_sliding {
        &pt.sliding
    } else if player.on_ground {
        if player.velocity.x != 0.0 {
            animate!(pt.walk, WALK_FRAME_LENGTH)
        } else {
            animate!(pt.idle, IDLE_FRAME_LENGTH)
        }
    } else {
        if player.velocity.y > 0.0 {
            animate!(pt.falling, FALLING_FRAME_LENGTH)
        } else {
            &pt.jumping
        }
    };
    let rw = 1.3;
    let rh = 2.0;
    let fudge_x = 1.3;
    let fudge_y = 0.5;
    let fudge_x_off = 0.75;
    let fudge_y_off = 1.0;
    let tw = rw * pixels_per_world_unit() * (1.0 + fudge_x);
    let th = rh * pixels_per_world_unit() * (1.0 + fudge_y);

    let exhaustion_level = 1.0 - (player.climb_stamina / crate::player::CLIMB_STAMINA);
    let palette = &COLOR_PALETTES[player.dashes as usize];
    let flicker = (exhaustion_level > 0.5)
        && player.is_climbing
        && ((exhaustion_level > 0.5 && exhaustion_level < 0.51)
            || (exhaustion_level > 0.6 && exhaustion_level < 0.61)
            || (exhaustion_level > 0.7 && exhaustion_level < 0.71)
            || (exhaustion_level > 0.8 && exhaustion_level < 0.81)
            || (exhaustion_level > 0.85 && exhaustion_level < 0.86)
            || (exhaustion_level > 0.9 && exhaustion_level < 0.91)
            || (exhaustion_level > 0.95 && exhaustion_level < 0.96))
        || (player.within_grace(player.spike_touched_at, SPIKE_IMMUNITY_COOLDOWN)
            && ((player.time - player.spike_touched_at) * 100.0).rem_euclid(100.0) < 10.0);

    let (loc_original_0, loc_replace_0, loc_exhustion, loc_whiteout) = shader_locs;

    unsafe {
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_original_0,
            DEFAULT_SPRITE_PALLETTE.as_ptr() as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
        );
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_replace_0,
            palette.as_ptr() as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
        );

        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_exhustion,
            &exhaustion_level as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
        );

        let whiteout = if flicker { 1.0f32 } else { 0.0f32 };
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_whiteout,
            &whiteout as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
        );
    }

    {
        let mut d_shader = d.begin_shader_mode(shader);

        d_shader.draw_texture_pro(
            texture,
            Rectangle::new(
                0.0,
                0.0,
                if player.facing_dir < 0 {
                    -1 * texture.width
                } else {
                    texture.width
                } as f32,
                texture.height as f32,
            ),
            Rectangle::new(
                player.position.x * pixels_per_world_unit() - pixels_per_world_unit() * fudge_x_off
                    + tw / 2.0,
                player.position.y * pixels_per_world_unit() - pixels_per_world_unit() * fudge_y_off
                    + th / 2.0,
                tw,
                th,
            ),
            Vector2::new(tw / 2.0, th / 2.0),
            rotation,
            Color::WHITE,
        );
    }

    let player_center_x = (player.position.x + player.width / 2.0) * pixels_per_world_unit();
    let player_center_y = (player.position.y + player.height / 2.0) * pixels_per_world_unit();
    let raycast_end_x = player.raycast_end_pos.x * pixels_per_world_unit();
    let raycast_end_y = player.raycast_end_pos.y * pixels_per_world_unit();

    if let Some((hit_x, hit_y)) = player.raycast_hit_tile {
        d.draw_rectangle_lines_ex(
            Rectangle::new(
                hit_x.floor() * pixels_per_world_unit(),
                hit_y.floor() * pixels_per_world_unit(),
                pixels_per_world_unit(),
                pixels_per_world_unit(),
            ),
            4.0,
            Color::RED,
        );
    }
    if let Some(item) = player.place_block_type {
        if let Some((free_x, free_y)) = player.raycast_last_free_tile {
            let texture = crate::inventory_screen::get_item_texture(&item, textures);
            d.draw_texture_pro(
                texture,
                Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                Rectangle::new(
                    free_x.floor() * pixels_per_world_unit(),
                    free_y.floor() * pixels_per_world_unit(),
                    pixels_per_world_unit(),
                    pixels_per_world_unit(),
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::new(255, 255, 255, 128),
            );
        }
    }
    if false {
        d.draw_line_ex(
            Vector2::new(player_center_x, player_center_y),
            Vector2::new(raycast_end_x, raycast_end_y),
            2.0,
            Color::GREEN,
        );

        d.draw_rectangle_lines(
            (player.position.x * pixels_per_world_unit()) as i32,
            (player.position.y * pixels_per_world_unit()) as i32,
            (player.width * pixels_per_world_unit()) as i32,
            (player.height * pixels_per_world_unit()) as i32,
            Color::RED,
        );
    }
}

pub struct GameScreen {
    camera: Camera2D,
    camera_velocity: Vector2,
    screen_width: f32,
    screen_height: f32,
}

impl GameScreen {
    pub fn new(ctx: &GameContext) -> Self {
        GameScreen {
            camera: Camera2D {
                target: Vector2::new(
                    ctx.world_state.player.position.x * pixels_per_world_unit(),
                    ctx.world_state.player.position.y * pixels_per_world_unit(),
                ),
                offset: Vector2::new(800.0, 450.0),
                rotation: 0.0,
                zoom: 1.0,
            },
            camera_velocity: Vector2::zero(),
            screen_width: 1600.0,
            screen_height: 900.0,
        }
    }
}

impl Screen for GameScreen {
    type Context = GameContext;

    fn update(&mut self, dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        self.camera.offset = Vector2::new(self.screen_width / 2.0, self.screen_height / 2.0);

        if ctx.updating {
            ctx.world_state.update(dt, &ctx.controller);
        }

        smooth_camera_to_target(
            &mut self.camera,
            &mut self.camera_velocity,
            ctx.world_state.player.position.x * pixels_per_world_unit(),
            ctx.world_state.player.position.y * pixels_per_world_unit(),
            dt,
            0.12,
        );

        // Check if Tab is pressed to open inventory
        if ctx.controller.menu_pressed {
            return ScreenCommand::Push(Box::new(InventoryScreen::new()));
        }

        ScreenCommand::None
    }

    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, ctx: &Self::Context) {
        self.screen_width = rl.get_screen_width() as f32;
        self.screen_height = rl.get_screen_height() as f32;

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);

        {
            let mut d2 = d.begin_mode2D(self.camera);
            let mut shader = ctx.render_state.player_shader.borrow_mut();

            let px = ctx.world_state.player.position.x as i32;
            let py = ctx.world_state.player.position.y as i32;

            render_terrain(&mut d2, &ctx.world_state.terrain, px, py, &ctx.textures);
            render_player(
                &mut d2,
                &ctx.world_state.player,
                &ctx.textures,
                &mut shader,
                ctx.render_state.shader_locs,
            );
        }
        // Draw HUD
        let heart_size = 32.0;
        let heart_spacing = 4.0;
        let hearts_x = self.screen_width - 150.0;
        let hearts_y = 10.0;
        let max_health = 12;
        let health_frames = 4;
        let health = ctx.world_state.player.health.max(0).min(max_health);
        let mut remaining_health = health;
        for i in 0..(max_health / health_frames) {
            let heart_x = hearts_x + (heart_size + heart_spacing) * i as f32;

            let heart_texture = if remaining_health > health_frames {
                &ctx.textures.ui.hearts[0]
            } else if remaining_health > 0 {
                &ctx.textures.ui.hearts[(health_frames - remaining_health) as usize]
            } else {
                continue;
            };

            d.draw_texture_ex(
                heart_texture,
                Vector2::new(heart_x, hearts_y),
                0.0,
                heart_size / heart_texture.width as f32,
                Color::WHITE,
            );

            remaining_health = remaining_health.saturating_sub(4);
        }

        // Draw breath bar when swimming or breath not full
        let max_breath = crate::player::MAX_BREATH_HOLD;
        if ctx.world_state.player.is_swimming || ctx.world_state.player.breath < max_breath {
            let breath_bar_x = hearts_x;
            let breath_bar_y = hearts_y + heart_size + 8.0;
            let breath_bar_width = 120.0;
            let breath_bar_height = 8.0;

            // Background (dark)
            d.draw_rectangle(
                breath_bar_x as i32,
                breath_bar_y as i32,
                breath_bar_width as i32,
                breath_bar_height as i32,
                Color::new(50, 50, 50, 180),
            );

            // Calculate breath percentage
            let breath_percent = (ctx.world_state.player.breath / max_breath)
                .max(0.0)
                .min(1.0);
            let filled_width = breath_bar_width * breath_percent;

            // Color changes as breath depletes
            let breath_color = if breath_percent > 0.5 {
                Color::new(100, 200, 255, 255) // Light blue
            } else if breath_percent > 0.25 {
                Color::new(255, 200, 100, 255) // Orange warning
            } else {
                Color::new(255, 100, 100, 255) // Red danger
            };

            // Draw filled portion
            d.draw_rectangle(
                breath_bar_x as i32,
                breath_bar_y as i32,
                filled_width as i32,
                breath_bar_height as i32,
                breath_color,
            );

            // Border
            d.draw_rectangle_lines(
                breath_bar_x as i32,
                breath_bar_y as i32,
                breath_bar_width as i32,
                breath_bar_height as i32,
                Color::WHITE,
            );
        }

        // Draw vignette effect when breath is critical
        if ctx.world_state.player.is_swimming {
            let breath_percent = (ctx.world_state.player.breath / max_breath)
                .max(0.0)
                .min(1.0);

            if breath_percent < 0.50 {
                // Base vignette intensity
                let base_alpha = ((1.0 - breath_percent * 2.0) * 255.0) as u8;
                let color = Color::new(0, 0, 100, 0);
                let color_alpha = Color::new(0, 0, 100, base_alpha);
                let vin_size = 500;
                // Top vignette
                d.draw_rectangle_gradient_v(
                    0,
                    0,
                    self.screen_width as i32,
                    vin_size,
                    color_alpha,
                    color,
                );

                // Bottom vignette
                d.draw_rectangle_gradient_v(
                    0,
                    self.screen_height as i32 - vin_size,
                    self.screen_width as i32,
                    vin_size,
                    color,
                    color_alpha,
                );

                // Left vignette
                d.draw_rectangle_gradient_h(
                    0,
                    0,
                    vin_size,
                    self.screen_height as i32,
                    color_alpha,
                    color,
                );

                // Right vignette
                d.draw_rectangle_gradient_h(
                    self.screen_width as i32 - vin_size,
                    0,
                    vin_size,
                    self.screen_height as i32,
                    color,
                    color_alpha,
                );
            }
        }

        // Draw HUD boxes for current tool and block type in bottom right
        let hud_box_size = 40.0;
        let mut hud_x = self.screen_width - hud_box_size - 70.0;
        let hud_y = self.screen_height - hud_box_size - 20.0;

        // Draw tool slot background
        d.draw_rectangle(
            hud_x as i32,
            hud_y as i32,
            hud_box_size as i32,
            hud_box_size as i32,
            Color::new(50, 50, 50, 200),
        );
        d.draw_rectangle_lines(
            hud_x as i32,
            hud_y as i32,
            hud_box_size as i32,
            hud_box_size as i32,
            Color::WHITE,
        );

        // Draw selected tool icon if present
        if let Some(selected_tool) = ctx.world_state.player.selected_tool {
            let tool_texture = match selected_tool {
                crate::tools::ToolType::Dash => Some(&ctx.textures.items.dashamulet),
                crate::tools::ToolType::Pickaxe => None, // TODO: add pickaxe texture
            };

            if let Some(texture) = tool_texture {
                let scale = hud_box_size / texture.width as f32;
                d.draw_texture_ex(
                    texture,
                    Vector2::new(hud_x, hud_y),
                    0.0,
                    scale,
                    Color::WHITE,
                );
            }

            // Draw durability bar for selected tool
            let (current_durability, max_durability) = match selected_tool {
                crate::tools::ToolType::Dash => {
                    if let Some(dash) = &ctx.world_state.player.tool_dash {
                        (dash.durability, dash.max_durability)
                    } else {
                        (0.0, 100.0)
                    }
                }
                crate::tools::ToolType::Pickaxe => (100.0, 100.0),
            };

            let durability_bar_y = hud_y + hud_box_size + 2.0;
            let durability_bar_width = hud_box_size;
            let durability_bar_height = 4.0;

            // Background (dark)
            d.draw_rectangle(
                hud_x as i32,
                durability_bar_y as i32,
                durability_bar_width as i32,
                durability_bar_height as i32,
                Color::new(50, 50, 50, 200),
            );

            // Calculate durability percentage
            let durability_percent = (current_durability / max_durability).max(0.0).min(1.0);
            let filled_width = durability_bar_width * durability_percent;

            // Color changes based on durability
            let durability_color = if durability_percent > 0.5 {
                Color::new(100, 255, 100, 255) // Green - good condition
            } else if durability_percent > 0.25 {
                Color::new(255, 255, 100, 255) // Yellow - wearing out
            } else {
                Color::new(255, 100, 100, 255) // Red - almost broken
            };

            // Draw filled portion
            d.draw_rectangle(
                hud_x as i32,
                durability_bar_y as i32,
                filled_width as i32,
                durability_bar_height as i32,
                durability_color,
            );

            // Border
            d.draw_rectangle_lines(
                hud_x as i32,
                durability_bar_y as i32,
                durability_bar_width as i32,
                durability_bar_height as i32,
                Color::new(200, 200, 200, 255),
            );
        }

        hud_x += 50.0;
        d.draw_rectangle(
            hud_x as i32,
            hud_y as i32,
            hud_box_size as i32,
            hud_box_size as i32,
            Color::new(50, 50, 50, 200),
        );
        d.draw_rectangle_lines(
            hud_x as i32,
            hud_y as i32,
            hud_box_size as i32,
            hud_box_size as i32,
            Color::WHITE,
        );

        // Draw selected block icon if present
        if let Some(block_type) = ctx.world_state.player.place_block_type {
            let block_texture =
                crate::inventory_screen::get_item_texture(&block_type, &ctx.textures);
            let scale = hud_box_size / block_texture.width as f32;
            d.draw_texture_ex(
                block_texture,
                Vector2::new(hud_x, hud_y),
                0.0,
                scale,
                Color::WHITE,
            );
        }

        if ctx.debug_enabled {
            d.draw_text(
                &format!(
                    "Pos: ({:.2}, {:.2})",
                    ctx.world_state.player.position.x, ctx.world_state.player.position.y
                ),
                10,
                10,
                20,
                Color::DARKGRAY,
            );
            d.draw_text(
                "Controls: WASD/Arrows=Move, Space=Jump, C=Climb, Shift/X=Dash, Toggle Controls=/, Toggle Ghost=', Play/Pause=Enter",
                500, 10, 16, Color::BLACK
            );
            d.draw_text(&format!("FPS: {}", d.get_fps()), 1500, 10, 20, Color::GRAY);

            d.draw_text(
                &format!(
                    "Vel: ({:.2}, {:.2})",
                    ctx.world_state.player.velocity.x, ctx.world_state.player.velocity.y
                ),
                10,
                35,
                20,
                Color::DARKGRAY,
            );
            d.draw_text(
                &format!(
                    "On Ground: {}   Is Climbing {}   Is Sliding {}   Is Dashing {}   Is Swimming {}",
                    ctx.world_state.player.on_ground,
                    ctx.world_state.player.is_climbing,
                    ctx.world_state.player.is_sliding,
                    ctx.world_state.player.is_dashing,
                    ctx.world_state.player.is_swimming
                ),
                10, 60, 20, Color::DARKGRAY
            );
            d.draw_text(
                &format!("Tide: {}", ctx.world_state.tide_level()),
                10,
                85,
                20,
                Color::DARKGRAY,
            );
        }
    }
}
