use crate::inventory_screen::InventoryScreen;
use crate::player::{Player, SPIKE_IMMUNITY_COOLDOWN};
use crate::terrain::{Block, Terrain, CELL_RESOLUTION, CHUNK_SIZE, NO_LIQUID_THRESHOLD};
use crate::world::{LIGHTING_RANGE, RENDER_RANGE};
use crate::{pixels_per_world_unit, GameContext, Neighbors, ShaderLocs};
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

// Sound effect intervals (in seconds)
const PICKAXE_SOUND_INTERVAL: f32 = 1.0;
const FOOTSTEP_SOUND_INTERVAL: f32 = 0.3;
const FOOTSTEP_MIN_SPEED: f32 = 0.5;

// Maps to uniforms (original_0, replace_0)
const DEFAULT_SPRITE_PALLETTE: &[f32; 4] = &[172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0]; // Red scarf
const COLOR_PALETTES: &[[f32; 4]] = &[
    [99.0 / 255.0, 155.0 / 255.0, 1.0, 1.0], // Blue scarf
    [55.0 / 255.0, 148.0 / 255.0, 110.0 / 255.0, 1.0], // Green scarf
    [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // Red scarf
    [118.0 / 255.0, 66.0 / 255.0, 138.0 / 255.0, 1.0], // Purple scarf

                                             //[1.0, 1.0, 1.0, 1.0]; // Air boots primary
                                             //[105.0 / 255.0, 106.0 / 255.0, 106.0 / 255.0, 1.0]; // Air boots secondary
                                             //[172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0]; // Dash boots primary
                                             //[69.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0]; // Dash boots secondary
                                             //[95.0 / 255.0, 205.0 / 255.0, 228.0 / 255.0, 1.0]; // Flippers primary
                                             //[48.0 / 255.0, 96.0 / 255.0, 130.0 / 255.0, 1.0]; // Flippers secondary
                                             //[215.0 / 255.0, 123.0 / 255.0, 186.0 / 255.0, 1.0]; // Floatie boots primary
                                             //[69.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0]; // Floatie boots secondary
                                             //[118.0 / 255.0, 66.0 / 255.0, 138.0 / 255.0, 1.0]; // Gravity boots primary
                                             //[69.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0]; // Gravity boots secondary
                                             //[143.0 / 255.0, 86.0 / 255.0, 59.0 / 255.0, 1.0]; // Hover boots primary
                                             //[102.0 / 255.0, 57.0 / 255.0, 49.0 / 255.0, 1.0]; // Hover boots secondary
                                             //[223.0 / 255.0, 113.0 / 255.0, 38.0 / 255.0, 1.0]; // Rocket boots primary
                                             //[102.0 / 255.0, 57.0 / 255.0, 49.0 / 255.0, 1.0]; // Rocket boots secondary
                                             //[251.0 / 255.0, 242.0 / 255.0, 54.0 / 255.0, 1.0]; // Rubber boots primary
                                             //[82.0 / 255.0, 75.0 / 255.0, 36.0 / 255.0, 1.0]; // Rubber boots secondary
                                             //[203.0 / 255.0, 219.0 / 255.0, 252.0 / 255.0, 1.0]; // Steel boots primary
                                             //[105.0 / 255.0, 106.0 / 255.0, 106.0 / 255.0, 1.0]; // Steel boots secondary
                                             //[75.0 / 255.0, 105.0 / 255.0, 47.0 / 255.0, 1.0]; // Wall boots primary
                                             //[50.0 / 255.0, 60.0 / 255.0, 57.0 / 255.0, 1.0]; // Wall boots secondary
];

// Tool sprite original colors to swap from
const TOOL_ORIGINAL_PRIMARY: &[f32; 4] = &[132.0 / 255.0, 126.0 / 255.0, 135.0 / 255.0, 1.0];
const TOOL_ORIGINAL_SECONDARY: &[f32; 4] = &[105.0 / 255.0, 106.0 / 255.0, 106.0 / 255.0, 1.0];

// Tool color palettes by pickaxe level: (primary, secondary)
fn get_tool_colors(level: &str) -> ([f32; 4], [f32; 4]) {
    match level {
        // Pickaxes
        "stone_pickaxe" => (
            [132.0 / 255.0, 126.0 / 255.0, 135.0 / 255.0, 1.0], // Stone primary (same as original)
            [105.0 / 255.0, 106.0 / 255.0, 106.0 / 255.0, 1.0], // Stone secondary (same as original)
        ),
        "copper_pickaxe" => (
            [223.0 / 255.0, 113.0 / 255.0, 38.0 / 255.0, 1.0], // Copper primary
            [102.0 / 255.0, 57.0 / 255.0, 49.0 / 255.0, 1.0],  // Copper secondary
        ),
        "bronze_pickaxe" => (
            [143.0 / 255.0, 86.0 / 255.0, 59.0 / 255.0, 1.0], // Bronze primary
            [102.0 / 255.0, 57.0 / 255.0, 49.0 / 255.0, 1.0], // Bronze secondary
        ),
        "iron_pickaxe" => (
            [155.0 / 255.0, 173.0 / 255.0, 183.0 / 255.0, 1.0], // Iron primary
            [105.0 / 255.0, 106.0 / 255.0, 106.0 / 255.0, 1.0], // Iron secondary
        ),
        "steel_pickaxe" => (
            [203.0 / 255.0, 219.0 / 255.0, 252.0 / 255.0, 1.0], // Steel primary
            [155.0 / 255.0, 173.0 / 255.0, 183.0 / 255.0, 1.0], // Steel secondary
        ),
        "platinum_pickaxe" => (
            [1.0, 1.0, 1.0, 1.0],                               // Platinum primary
            [203.0 / 255.0, 219.0 / 255.0, 252.0 / 255.0, 1.0], // Platinum secondary
        ),
        "mithril_pickaxe" => (
            [91.0 / 255.0, 110.0 / 255.0, 225.0 / 255.0, 1.0], // Mithril primary
            [63.0 / 255.0, 63.0 / 255.0, 116.0 / 255.0, 1.0],  // Mithril secondary
        ),
        "ozathinum_pickaxe" => (
            [55.0 / 255.0, 148.0 / 255.0, 110.0 / 255.0, 1.0], // Ozathinum primary
            [50.0 / 255.0, 60.0 / 255.0, 57.0 / 255.0, 1.0],   // Ozathinum secondary
        ),
        "torzite_pickaxe" => (
            [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // Torzite primary
            [69.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0],  // Torzite secondary
        ),
        "etherealite_pickaxe" => (
            [118.0 / 255.0, 66.0 / 255.0, 138.0 / 255.0, 1.0], // Etherealite primary
            [69.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0],   // Etherealite secondary
        ),
        // Gliders (gem-based)
        "linen_glider" => (
            [155.0 / 255.0, 173.0 / 255.0, 183.0 / 255.0, 1.0], // Linen primary (natural/default)
            [203.0 / 255.0, 219.0 / 255.0, 252.0 / 255.0, 1.0], // Linen secondary
        ),
        "pearl_glider" => (
            [203.0 / 255.0, 219.0 / 255.0, 252.0 / 255.0, 1.0], // Pearl primary (creamy white)
            [1.0, 1.0, 1.0, 1.0],                               // Pearl secondary
        ),
        "amethyst_glider" => (
            [118.0 / 255.0, 66.0 / 255.0, 138.0 / 255.0, 1.0], // Amethyst primary (purple)
            [69.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0],   // Amethyst secondary
        ),
        "emerald_glider" => (
            [106.0 / 255.0, 190.0 / 255.0, 48.0 / 255.0, 1.0], // Emerald primary (green)
            [75.0 / 255.0, 105.0 / 255.0, 47.0 / 255.0, 1.0],  // Emerald secondary
        ),
        "topaz_glider" => (
            [91.0 / 255.0, 110.0 / 255.0, 225.0 / 255.0, 1.0], // Topaz primary (blue)
            [48.0 / 255.0, 96.0 / 255.0, 130.0 / 255.0, 1.0],  // Topaz secondary
        ),
        "ruby_glider" => (
            [217.0 / 255.0, 87.0 / 255.0, 99.0 / 255.0, 1.0], // Ruby primary (red)
            [172.0 / 255.0, 55.0 / 255.0, 50.0 / 255.0, 1.0], // Ruby secondary
        ),
        "diamond_glider" => (
            [95.0 / 255.0, 205.0 / 255.0, 228.0 / 255.0, 1.0], // Diamond primary (light blue)
            [99.0 / 255.0, 155.0 / 255.0, 225.0 / 255.0, 1.0], // Diamond secondary
        ),
        _ => (
            [132.0 / 255.0, 126.0 / 255.0, 135.0 / 255.0, 1.0], // Default primary
            [105.0 / 255.0, 106.0 / 255.0, 106.0 / 255.0, 1.0], // Default secondary
        ),
    }
}

// LIGHTING_RANGE and RENDER_RANGE now imported from world.rs

fn smooth_axis(
    current: f32,
    velocity: &mut f32,
    target: f32,
    omega: f32,
    dt: f32,
    exp: f32,
    max_speed: f32,
) -> f32 {
    let change = current - target;
    let temp = (*velocity + omega * change) * dt;
    *velocity = (*velocity - omega * temp) * exp;

    if velocity.abs() > max_speed {
        *velocity = velocity.signum() * max_speed;
    }

    target + (change + temp) * exp
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
    let omega = 2.0 / smooth_time;
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    camera.target.x = smooth_axis(
        camera.target.x,
        &mut camera_velocity.x,
        target_x,
        omega,
        dt,
        exp,
        max_speed,
    );
    camera.target.y = smooth_axis(
        camera.target.y,
        &mut camera_velocity.y,
        target_y,
        omega,
        dt,
        exp,
        max_speed,
    );
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

fn render_lighting(
    d: &mut RaylibMode2D<RaylibDrawHandle>,
    lighting: &crate::lighting::LightingSystem,
    lighting_texture: &Texture2D,
    lighting_shader: &mut Shader,
    px: i32,
    py: i32,
) {
    let ppw = pixels_per_world_unit();
    let range = LIGHTING_RANGE;

    // Early exit if no lights and no darkness
    if lighting.lights().is_empty() && lighting.ambient_darkness < 0.01 {
        return;
    }

    // Draw fullscreen quad with lighting shader and texture
    let world_size = (range * 2) as f32;
    let min_x = (px - range) as f32;
    let min_y = (py - range) as f32;

    {
        let mut shader_mode = d.begin_shader_mode(lighting_shader);

        // Enable alpha blending for darkness overlay
        unsafe {
            raylib::ffi::BeginBlendMode(raylib::ffi::BlendMode::BLEND_ALPHA as i32);
        }

        shader_mode.draw_texture_pro(
            lighting_texture,
            Rectangle::new(
                0.0,
                0.0,
                lighting_texture.width as f32,
                lighting_texture.height as f32,
            ),
            Rectangle::new(min_x * ppw, min_y * ppw, world_size * ppw, world_size * ppw),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

        unsafe {
            raylib::ffi::EndBlendMode();
        }
    }
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
    world_state: &crate::world::WorldState,
    px: i32,
    py: i32,
    textures: &crate::TextureManager,
) {
    let range = RENDER_RANGE;
    for x in (px - range)..(px + range) {
        for y in (py - range)..(py + range) {
            let block = terrain.at(x, y);

            // Render water in air, tide, ladder, spike, and pickup blocks (water shows behind them)
            if block == Block::Air
                || block == Block::Tide
                || block.is_ladder()
                || block.is_spike()
                || block.is_pickup()
            {
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

            if block.is_spike() {
                let neighbors = Neighbors {
                    up: terrain.spike_at(x, y - 1) || terrain.solid_terrain_at(x, y - 1),
                    up_right: terrain.spike_at(x + 1, y - 1)
                        || terrain.solid_terrain_at(x + 1, y - 1),
                    right: terrain.spike_at(x + 1, y) || terrain.solid_terrain_at(x + 1, y),
                    down_right: terrain.spike_at(x + 1, y + 1)
                        || terrain.solid_terrain_at(x + 1, y + 1),
                    down: terrain.spike_at(x, y + 1) || terrain.solid_terrain_at(x, y + 1),
                    down_left: terrain.spike_at(x - 1, y + 1)
                        || terrain.solid_terrain_at(x - 1, y + 1),
                    left: terrain.spike_at(x - 1, y) || terrain.solid_terrain_at(x - 1, y),
                    up_left: terrain.spike_at(x - 1, y - 1)
                        || terrain.solid_terrain_at(x - 1, y - 1),
                };
                let src_rect = textures.tiles.spikes.get_tile_rect(x, y, &neighbors);
                let final_src_rect = if false && block == Block::Stalactite {
                    Rectangle::new(
                        src_rect.x,
                        src_rect.y + src_rect.height,
                        src_rect.width,
                        -src_rect.height,
                    )
                } else {
                    src_rect
                };
                render_tile(
                    d,
                    x as f32,
                    y as f32,
                    &textures.tiles.spikes.texture(),
                    Some(final_src_rect),
                );
            } else if block != Block::Air && block != Block::Tide {
                // Don't render Air or Tide blocks (they're just water/empty)

                // Handle multi-tile blocks - only render from anchor position
                if block.is_multi_tile() {
                    // Only render if this is the anchor tile
                    if terrain.is_multi_tile_anchor(x, y) {
                        let texture = block.get_texture(textures);
                        let width = block.width();
                        let height = block.height();

                        // Render multi-tile block spanning multiple tiles
                        let ppw = pixels_per_world_unit();
                        d.draw_texture_pro(
                            texture,
                            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                            Rectangle::new(
                                x as f32 * ppw,
                                (y - height as i32 + 1) as f32 * ppw,
                                width as f32 * ppw,
                                height as f32 * ppw,
                            ),
                            Vector2::new(0.0, 0.0),
                            0.0,
                            Color::WHITE,
                        );
                    }
                    continue;
                }

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
                } else if block == Block::Bomb {
                    // Check if this is an active bomb with animation
                    if let Some(frame) = world_state.get_bomb_frame(x, y) {
                        let texture = match frame {
                            0 => &textures.tiles.bomb1,
                            1 => &textures.tiles.bomb2,
                            2 => &textures.tiles.bomb3,
                            3 => &textures.tiles.bomb4,
                            4 => &textures.tiles.bomb5,
                            5 => &textures.tiles.bomb6,
                            6 => &textures.tiles.bomb7,
                            7 => &textures.tiles.bomb8,
                            _ => &textures.tiles.bomb9,
                        };
                        render_tile(d, x as f32, y as f32, texture, None);
                    } else {
                        // Fallback to default bomb texture
                        let texture = block.get_texture(textures);
                        render_tile(d, x as f32, y as f32, texture, None);
                    }
                } else {
                    let texture = block.get_texture(textures);
                    render_tile(d, x as f32, y as f32, texture, None);
                }
            }
        }
    }
}

fn render_player(
    d: &mut RaylibDrawHandle,
    player: &Player,
    terrain: &Terrain,
    textures: &crate::TextureManager,
    shader: &mut Shader,
    shader_locs: &ShaderLocs,
    debug_render: bool,
) {
    macro_rules! animate {
        ($frames:expr, $frame_length:expr) => {
            &$frames[(player.time / $frame_length) as usize % $frames.len()]
        };
    }

    macro_rules! animate_from {
        ($frames:expr, $frame_length:expr, $offset:expr) => {
            &$frames[((player.time - $offset) / $frame_length) as usize % $frames.len()]
        };
    }

    const WALK_FRAME_LENGTH: f32 = 0.1;
    const CLIMBING_FRAME_LENGTH: f32 = 0.2;
    const FALLING_FRAME_LENGTH: f32 = 0.15;
    const SLIDING_FRAME_LENGTH: f32 = 0.2;
    const IDLE_FRAME_LENGTH: f32 = 0.25;
    const SWIMMING_FRAME_LENGTH: f32 = 0.18;
    const MINING_FRAME_LENGTH: f32 = 0.1;

    let mut rotation = 0.0;

    let pt = &textures.player.sammi;
    let texture = if player.is_dashing {
        rotation = player.velocity.y.atan2(player.velocity.x).to_degrees();
        animate!(&pt.dash, WALK_FRAME_LENGTH)
    } else if player.is_mining {
        animate_from!(pt.mining, MINING_FRAME_LENGTH, player.started_mining_at)
    } else if player.is_on_ladder {
        // Ladder climbing animation - use ladder frames when moving vertically
        if player.velocity.y != 0.0 {
            animate!(pt.ladder, CLIMBING_FRAME_LENGTH)
        } else {
            &pt.ladder[0]
        }
    } else if player.is_climbing {
        if player.velocity.y != 0.0 {
            animate!(pt.climb, CLIMBING_FRAME_LENGTH)
        } else {
            &pt.climb[0]
        }
    } else if player.is_swimming {
        animate!(pt.swimming, SWIMMING_FRAME_LENGTH)
    } else if player.is_gliding {
        animate!(pt.gliding, FALLING_FRAME_LENGTH)
    } else if player.is_sliding {
        animate!(pt.sliding, SLIDING_FRAME_LENGTH)
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

    unsafe {
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            shader_locs.original_0,
            DEFAULT_SPRITE_PALLETTE.as_ptr() as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
        );
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            shader_locs.replace_0,
            palette.as_ptr() as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
        );

        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            shader_locs.exhustion,
            &exhaustion_level as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
        );

        let whiteout = if flicker { 1.0f32 } else { 0.0f32 };
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            shader_locs.whiteout,
            &whiteout as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
        );

        // Set tool color swap uniforms when mining or gliding
        let tool_swap_enabled = if player.is_mining || player.is_gliding {
            1.0f32
        } else {
            0.0f32
        };
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            shader_locs.tool_swap_enabled,
            &tool_swap_enabled as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
        );

        if player.is_mining {
            let (tool_primary, tool_secondary) = player
                .tool_pickaxe
                .as_ref()
                .map(|p| get_tool_colors(&p.level))
                .unwrap_or_else(|| get_tool_colors("stone_pickaxe"));

            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_original_primary,
                TOOL_ORIGINAL_PRIMARY.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_original_secondary,
                TOOL_ORIGINAL_SECONDARY.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_replace_primary,
                tool_primary.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_replace_secondary,
                tool_secondary.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
        } else if player.is_gliding {
            let (tool_primary, tool_secondary) = player
                .tool_glider
                .as_ref()
                .map(|g| get_tool_colors(&g.level))
                .unwrap_or_else(|| get_tool_colors("linen_glider"));

            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_original_primary,
                TOOL_ORIGINAL_PRIMARY.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_original_secondary,
                TOOL_ORIGINAL_SECONDARY.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_replace_primary,
                tool_primary.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
            raylib::ffi::SetShaderValue(
                shader.as_ref().clone(),
                shader_locs.tool_replace_secondary,
                tool_secondary.as_ptr() as *const std::ffi::c_void,
                raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
            );
        }
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

    // Determine highlight color based on whether both tiles are the same
    let both_same_tile = player.raycast_left_tile.is_some()
        && player.raycast_right_tile.is_some()
        && player.raycast_left_tile == player.raycast_right_tile;

    let mut draw_hand_preview =
        |tile: Option<(f32, f32)>, tool: Option<crate::tools::ToolType>, color: Color| {
            if let Some((tile_x, tile_y)) = tile {
                let ppw = pixels_per_world_unit();

                // Draw highlight - if placing a multi-tile block, show full bounds
                if let Some(crate::tools::ToolType::PlaceBlock(block)) = tool {
                    // Check if placement would be valid
                    let x = tile_x.floor() as i32;
                    let y = tile_y.floor() as i32;
                    if !player.can_place_block_at(terrain, block, x, y) {
                        return;
                    }

                    if block.is_multi_tile() {
                        let width = block.width();
                        let height = block.height();
                        d.draw_rectangle_lines_ex(
                            Rectangle::new(
                                tile_x.floor() * ppw,
                                (tile_y.floor() - height as f32 + 1.0) * ppw,
                                width as f32 * ppw,
                                height as f32 * ppw,
                            ),
                            4.0,
                            color,
                        );
                    } else {
                        d.draw_rectangle_lines_ex(
                            Rectangle::new(tile_x.floor() * ppw, tile_y.floor() * ppw, ppw, ppw),
                            4.0,
                            color,
                        );
                    }

                    // Draw preview for place block tools
                    let texture = block.get_texture(textures);

                    // Handle multi-tile blocks
                    if block.is_multi_tile() {
                        let width = block.width();
                        let height = block.height();
                        d.draw_texture_pro(
                            texture,
                            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                            Rectangle::new(
                                tile_x.floor() * ppw,
                                (tile_y.floor() - height as f32 + 1.0) * ppw,
                                width as f32 * ppw,
                                height as f32 * ppw,
                            ),
                            Vector2::new(0.0, 0.0),
                            0.0,
                            Color::new(255, 255, 255, 128),
                        );
                    } else {
                        d.draw_texture_pro(
                            texture,
                            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                            Rectangle::new(tile_x.floor() * ppw, tile_y.floor() * ppw, ppw, ppw),
                            Vector2::new(0.0, 0.0),
                            0.0,
                            Color::new(255, 255, 255, 128),
                        );
                    }
                } else {
                    d.draw_rectangle_lines_ex(
                        Rectangle::new(tile_x.floor() * ppw, tile_y.floor() * ppw, ppw, ppw),
                        4.0,
                        color,
                    );
                }
            }
        };

    // Draw left hand tile highlight (red or yellow if both same)
    let left_color = if both_same_tile {
        Color::YELLOW
    } else {
        Color::RED
    };
    draw_hand_preview(player.raycast_left_tile, player.left_hand, left_color);

    // Draw right hand tile highlight (green or skip if both same)
    if !both_same_tile {
        draw_hand_preview(player.raycast_right_tile, player.right_hand, Color::GREEN);
    }

    if debug_render {
        // Draw left hand raycast line
        if player.left_hand.is_some() {
            let raycast_left_end_x = player.raycast_left_end.x * pixels_per_world_unit();
            let raycast_left_end_y = player.raycast_left_end.y * pixels_per_world_unit();
            d.draw_line_ex(
                Vector2::new(player_center_x, player_center_y),
                Vector2::new(raycast_left_end_x, raycast_left_end_y),
                2.0,
                Color::RED,
            );
        }

        // Draw right hand raycast line
        if player.right_hand.is_some() {
            let raycast_right_end_x = player.raycast_right_end.x * pixels_per_world_unit();
            let raycast_right_end_y = player.raycast_right_end.y * pixels_per_world_unit();
            d.draw_line_ex(
                Vector2::new(player_center_x, player_center_y),
                Vector2::new(raycast_right_end_x, raycast_right_end_y),
                2.0,
                Color::GREEN,
            );
        }

        d.draw_rectangle_lines(
            (player.position.x * pixels_per_world_unit()) as i32,
            (player.position.y * pixels_per_world_unit()) as i32,
            (player.width * pixels_per_world_unit()) as i32,
            (player.height * pixels_per_world_unit()) as i32,
            Color::RED,
        );
    }

    // Render block break progress overlay
    if player.is_mining {
        if let Some((bx, by)) = player.currently_mining {
            let block = terrain.at(bx as i32, by as i32);
            if block != Block::Air {
                let durability = block.durability();
                // Get pickaxe speed (default 1.0 if no pickaxe equipped)
                let pickaxe_speed = player.tool_pickaxe.as_ref().map(|p| p.speed).unwrap_or(1.0);
                let mining_time = durability / pickaxe_speed;
                let progress =
                    ((player.time - player.started_mining_at) / mining_time).clamp(0.0, 1.0);
                let frame = (progress * 4.0).floor().min(3.0) as usize;
                let break_texture = &textures.ui.block_break[frame];

                render_tile(d, bx, by, break_texture, None);
            }
        }
    }
}

pub struct GameScreen {
    camera: Camera2D,
    camera_velocity: Vector2,
    screen_width: f32,
    screen_height: f32,
    help_button: Rectangle,
    help_hovered: bool,
}

impl GameScreen {
    pub fn new(ctx: &GameContext) -> Self {
        let button_size = 40.0;
        let margin = 10.0;
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
            help_button: Rectangle::new(
                1600.0 - button_size - margin,
                900.0 - button_size - margin,
                button_size,
                button_size,
            ),
            help_hovered: false,
        }
    }
}

impl Screen for GameScreen {
    type Context = GameContext;

    fn on_resume(&mut self, ctx: &mut Self::Context) {
        // Start layered game music when entering game
        ctx.music.play_game_layers();
        // Restore full volume when resuming
        ctx.music.set_volume_multiplier(1.0);
    }

    fn on_pause(&mut self, ctx: &mut Self::Context) {
        // Half the music volume when paused
        ctx.music.set_volume_multiplier(0.3);
    }

    fn update(&mut self, dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        // Update music streams
        ctx.music.update_streams();

        // Update music layers based on player depth and tide level
        if ctx.updating {
            ctx.music.update_game_depth(
                dt,
                ctx.world_state.player.position.y,
                ctx.world_state.tide_level() as f32,
            );
        }

        self.camera.offset = Vector2::new(self.screen_width / 2.0, self.screen_height / 2.0);

        if ctx.updating {
            ctx.world_state.update(dt, &ctx.controller);

            // Handle sound effects
            let player = &mut ctx.world_state.player;
            let time = player.time;

            // Pickaxe sounds while mining
            if player.is_mining && time - player.last_pickaxe_sound >= PICKAXE_SOUND_INTERVAL {
                ctx.sounds.play_random("pickaxe");
                player.last_pickaxe_sound = time;
            }

            // Footstep sounds while walking on ground
            if player.on_ground
                && !player.is_swimming
                && player.velocity.x.abs() > FOOTSTEP_MIN_SPEED
                && time - player.last_footstep_sound >= FOOTSTEP_SOUND_INTERVAL
            {
                ctx.sounds.play_random("footstep");
                player.last_footstep_sound = time;
            }

            // Jump sound (check if player just jumped this frame)
            if player.jumped_this_frame {
                ctx.sounds.play_random("jumping");
            }
        }

        smooth_camera_to_target(
            &mut self.camera,
            &mut self.camera_velocity,
            ctx.world_state.player.position.x * pixels_per_world_unit(),
            ctx.world_state.player.position.y * pixels_per_world_unit(),
            dt,
            0.12,
        );

        // Check for player death
        if ctx.world_state.player.health <= 0 {
            return ScreenCommand::Push(Box::new(crate::death_screen::DeathScreen::new()));
        }

        // Check if Tab is pressed to open inventory
        if ctx.controller.menu_pressed {
            return ScreenCommand::Push(Box::new(InventoryScreen::new()));
        }

        // Check for help button click
        let mouse_pos = Vector2::new(
            ctx.controller.mouse_position.x * self.screen_width,
            ctx.controller.mouse_position.y * self.screen_height,
        );
        self.help_hovered = self.help_button.check_collision_point_rec(mouse_pos);
        if ctx.controller.left_hand_pressed && self.help_hovered {
            return ScreenCommand::Push(Box::new(crate::help_screen::HelpScreen::new()));
        }

        // Check if M is pressed to open crafting or chest
        if ctx.controller.crafting_pressed {
            use crate::crafting_screen::RecipeType;
            use crate::terrain::Block;

            // Crafting stations take priority over chests
            let crafting_station = ctx
                .world_state
                .player
                .get_intersecting_crafting_station(&ctx.world_state.terrain);

            if let Some(station) = crafting_station {
                let filter = match station {
                    Block::Workbench => Some(RecipeType::Workbench),
                    Block::Anvil => Some(RecipeType::Anvil),
                    Block::Furnace => Some(RecipeType::Furnace),
                    _ => None,
                };
                return ScreenCommand::Push(Box::new(crate::crafting_screen::CraftingScreen::new(
                    filter,
                )));
            }

            // Check for chest interaction if no crafting station
            if let Some(chest_pos) = ctx
                .world_state
                .player
                .get_intersecting_chest(&ctx.world_state.terrain)
            {
                // Only open if this chest has an inventory (was properly placed)
                if ctx.world_state.chests.contains_key(&chest_pos) {
                    return ScreenCommand::Push(Box::new(crate::chest_screen::ChestScreen::new(
                        chest_pos,
                    )));
                }
            }

            // No crafting station or chest - open basic crafting menu
            return ScreenCommand::Push(Box::new(crate::crafting_screen::CraftingScreen::new(
                None,
            )));
        }

        ScreenCommand::None
    }

    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, ctx: &Self::Context) {
        self.screen_width = rl.get_screen_width() as f32;
        self.screen_height = rl.get_screen_height() as f32;

        // Update lighting system (this is a workaround since we can't mutate in render)
        // In the future, this should be in update()
        let lighting_system = &ctx.world_state.lighting_system;

        let px = ctx.world_state.player.position.x as i32;
        let py = ctx.world_state.player.position.y as i32;

        // Create lighting texture BEFORE begin_drawing to avoid borrow conflicts
        let lighting_texture = if !lighting_system.lights().is_empty()
            || lighting_system.ambient_darkness >= 0.01
        {
            let range = LIGHTING_RANGE;
            let texture_size = crate::lighting::LightingSystem::get_texture_size(range);
            let pixel_data =
                lighting_system.create_lighting_texture(&ctx.world_state.terrain, px, py, range);

            // Create RG texture (R=brightness, G=solid flag)
            let mut image = Image::gen_image_color(texture_size, texture_size, Color::BLACK);

            unsafe {
                let pixels = std::slice::from_raw_parts_mut(
                    (*image.as_mut()).data as *mut u8,
                    (texture_size * texture_size * 4) as usize,
                );

                for i in 0..(texture_size * texture_size) as usize {
                    let brightness = pixel_data[i * 2];
                    let is_solid = pixel_data[i * 2 + 1];
                    pixels[i * 4] = brightness; // R = brightness
                    pixels[i * 4 + 1] = is_solid; // G = solid flag
                    pixels[i * 4 + 2] = 0; // B = unused
                    pixels[i * 4 + 3] = 255; // A = opaque
                }
            }

            let mut texture = rl.load_texture_from_image(thread, &image).unwrap();

            // Enable bilinear filtering for smooth lighting
            unsafe {
                raylib::ffi::SetTextureFilter(
                    *texture.as_ref(),
                    raylib::ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
                );
            }

            // Set shader uniforms
            let lighting_shader = ctx.render_state.lighting_shader.borrow();
            unsafe {
                raylib::ffi::SetShaderValue(
                    *lighting_shader.as_ref(),
                    ctx.render_state.lighting_shader_locs.ambient_darkness,
                    &lighting_system.ambient_darkness as *const f32 as *const _,
                    raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
                );
                raylib::ffi::SetShaderValue(
                    *lighting_shader.as_ref(),
                    ctx.render_state.lighting_shader_locs.texture_size,
                    &[texture_size as f32, texture_size as f32] as *const f32 as *const _,
                    raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC2 as i32,
                );
            }

            Some(texture)
        } else {
            None
        };

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::RAYWHITE);

        // Draw background based on y position
        {
            let mut d2 = d.begin_mode2D(self.camera);
            let ppw = pixels_per_world_unit();
            let px = ctx.world_state.player.position.x as i32;
            let py = ctx.world_state.player.position.y as i32;
            let range = RENDER_RANGE as i32;

            let sky_color = Color::new(135, 206, 235, 255); // Light blue
            let cave_color = Color::new(50, 50, 55, 255); // Dark grey
            let sky_cutoff = 5.0;

            // Draw sky (y above sky_cutoff)
            if py - range < sky_cutoff as i32 {
                let sky_min_y = (py - range).max(-range * 2) as f32;
                let sky_max_y = 0.0;
                let sky_height = sky_max_y - sky_min_y;
                if sky_height > sky_cutoff {
                    d2.draw_rectangle(
                        ((px - range) as f32 * ppw) as i32,
                        (sky_min_y * ppw) as i32,
                        ((range * 2) as f32 * ppw) as i32,
                        (sky_height * ppw) as i32,
                        sky_color,
                    );
                }
            }
            // Draw cave (y below sky_cutoff)
            if py + range > sky_cutoff as i32 {
                let cave_min_y = 0.0f32.max((py - range) as f32);
                let cave_max_y = (py + range) as f32;
                let cave_height = cave_max_y - cave_min_y;
                if cave_height > sky_cutoff {
                    d2.draw_rectangle(
                        ((px - range) as f32 * ppw) as i32,
                        (cave_min_y * ppw) as i32,
                        ((range * 2) as f32 * ppw) as i32,
                        (cave_height * ppw) as i32,
                        cave_color,
                    );
                }
            }
        }

        {
            let mut d2 = d.begin_mode2D(self.camera);
            let mut shader = ctx.render_state.player_shader.borrow_mut();

            let px = ctx.world_state.player.position.x as i32;
            let py = ctx.world_state.player.position.y as i32;

            render_terrain(
                &mut d2,
                &ctx.world_state.terrain,
                &ctx.world_state,
                px,
                py,
                &ctx.textures,
            );

            // Draw chunk boundaries when debug is enabled
            if ctx.debug_enabled {
                use crate::terrain::CHUNK_SIZE;
                let range = 128;
                let chunk_size = CHUNK_SIZE as i32;

                // Calculate the range of chunks to draw boundaries for
                let min_chunk_x = (px - range) / chunk_size - 1;
                let max_chunk_x = (px + range) / chunk_size + 1;
                let min_chunk_y = (py - range) / chunk_size - 1;
                let max_chunk_y = (py + range) / chunk_size + 1;

                // Draw vertical chunk boundaries
                for chunk_x in min_chunk_x..=max_chunk_x {
                    let x_pos = chunk_x * chunk_size;
                    let screen_x = x_pos as f32 * pixels_per_world_unit();
                    let screen_y_start = (py - range) as f32 * pixels_per_world_unit();
                    let screen_y_end = (py + range) as f32 * pixels_per_world_unit();

                    d2.draw_line_ex(
                        Vector2::new(screen_x, screen_y_start),
                        Vector2::new(screen_x, screen_y_end),
                        2.0,
                        Color::new(255, 120, 120, 255),
                    );
                }

                // Draw horizontal chunk boundaries
                for chunk_y in min_chunk_y..=max_chunk_y {
                    let y_pos = chunk_y * chunk_size;
                    let screen_y = y_pos as f32 * pixels_per_world_unit();
                    let screen_x_start = (px - range) as f32 * pixels_per_world_unit();
                    let screen_x_end = (px + range) as f32 * pixels_per_world_unit();

                    d2.draw_line_ex(
                        Vector2::new(screen_x_start, screen_y),
                        Vector2::new(screen_x_end, screen_y),
                        2.0,
                        Color::new(255, 120, 120, 255),
                    );
                }
            }

            render_player(
                &mut d2,
                &ctx.world_state.player,
                &ctx.world_state.terrain,
                &ctx.textures,
                &mut shader,
                &ctx.render_state.shader_locs,
                ctx.debug_enabled,
            );

            // Draw lighting system
            if let Some(ref texture) = lighting_texture {
                let mut lighting_shader = ctx.render_state.lighting_shader.borrow_mut();
                render_lighting(
                    &mut d2,
                    &ctx.world_state.lighting_system,
                    texture,
                    &mut lighting_shader,
                    px,
                    py,
                );
            }
        }
        // Draw HUD
        let heart_texture = &ctx.textures.ui.hearts[4];
        let heart_size = heart_texture.width as f32 / 2.5;
        let heart_spacing = 10.0;
        let max_health = 32;
        let health_frames = 4;
        let hearts_x = (self.screen_width
            - (heart_size * heart_spacing) * max_health as f32 / health_frames as f32)
            / 2.0;
        let hearts_y = self.screen_height - 147.0;
        let health = ctx.world_state.player.health.max(0).min(max_health);
        let mut remaining_health = health;

        for i in 0..(max_health / health_frames) {
            let heart_x = hearts_x + (heart_size * heart_spacing) * i as f32;

            d.draw_texture_ex(
                heart_texture,
                Vector2::new(heart_x, hearts_y),
                0.0,
                heart_size,
                Color::WHITE,
            );

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
                heart_size,
                Color::WHITE,
            );

            remaining_health = remaining_health.saturating_sub(4);
        }

        // Draw breath bar when swimming or breath not full
        let max_breath = crate::player::MAX_BREATH_HOLD;
        if ctx.world_state.player.is_swimming || ctx.world_state.player.breath < max_breath {
            let breath_bar_width = 192.0;
            let breath_bar_height = 6.0;
            let breath_bar_x = (self.screen_width - breath_bar_width) / 2.0;
            let breath_bar_y = self.screen_height - 110.0;

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
                Color::BLACK,
            );
        }

        // Draw HUD boxes for current tool and block type in bottom centre
        let hud_box_texture = &ctx.textures.ui.inventory_slot;
        let hud_box_size = hud_box_texture.width as f32;
        let scale = 4.0;
        let right_offset = hud_box_size * (scale + 0.5);
        let hud_x = (self.screen_width - hud_box_size / 2.0) / 2.0 - hud_box_size * scale;
        let hud_y = self.screen_height - hud_box_size * scale - hud_box_size / 4.0;

        d.draw_texture_ex(
            hud_box_texture,
            Vector2 { x: hud_x, y: hud_y },
            0.0,
            scale,
            Color::WHITE,
        );
        d.draw_texture_ex(
            hud_box_texture,
            Vector2 {
                x: hud_x + right_offset,
                y: hud_y,
            },
            0.0,
            scale,
            Color::WHITE,
        );

        // Helper closure to draw a hand slot
        let mut draw_hand_slot = |hand: Option<crate::tools::ToolType>, x: f32, label: &str| {
            // Draw selected tool icon if present
            if let Some(selected_tool) = hand {
                let texture = match selected_tool {
                    crate::tools::ToolType::Pickaxe => ctx
                        .world_state
                        .player
                        .tool_pickaxe
                        .as_ref()
                        .map(|p| p.get_texture(&ctx.textures)),
                    crate::tools::ToolType::Dash => ctx
                        .world_state
                        .player
                        .tool_dash
                        .as_ref()
                        .map(|d| d.get_texture(&ctx.textures)),
                    crate::tools::ToolType::Glider => ctx
                        .world_state
                        .player
                        .tool_glider
                        .as_ref()
                        .map(|g| g.get_texture(&ctx.textures)),
                    _ => selected_tool.get_texture(&ctx.textures),
                };
                if let Some(texture) = texture {
                    d.draw_texture_ex(texture, Vector2::new(x, hud_y), 0.0, scale, Color::WHITE);
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
                    crate::tools::ToolType::Pickaxe => {
                        if let Some(pick) = &ctx.world_state.player.tool_pickaxe {
                            (pick.durability, pick.max_durability)
                        } else {
                            (0.0, 100.0)
                        }
                    }
                    crate::tools::ToolType::Lamp => {
                        // Lamp doesn't have durability (yet)
                        (1.0, 1.0)
                    }
                    crate::tools::ToolType::Glider => {
                        if let Some(glider) = &ctx.world_state.player.tool_glider {
                            (glider.durability, glider.max_durability)
                        } else {
                            (0.0, 100.0)
                        }
                    }
                    crate::tools::ToolType::TideClock => {
                        // TideClock has no durability
                        (1.0, 1.0)
                    }
                    crate::tools::ToolType::PlaceBlock(blk) => {
                        let count = blk
                            .to_item_type()
                            .map(|item| ctx.world_state.player.inventory.count(item))
                            .unwrap_or(0);
                        (count as f32, count.max(1) as f32)
                    }
                };

                let durability_bar_width = hud_box_size * scale / 2.0;
                let durability_bar_height = 3.0;
                let durability_bar_y = hud_y + hud_box_size * (scale - 1.0) + 9.0;

                // Background (dark)
                d.draw_rectangle(
                    x as i32 + 24,
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
                    x as i32 + 24,
                    durability_bar_y as i32,
                    filled_width as i32,
                    durability_bar_height as i32,
                    durability_color,
                );
            }
        };

        // Draw left hand slot (left click tool)
        draw_hand_slot(ctx.world_state.player.left_hand, hud_x, "L");

        // Draw right hand slot (right click tool)
        draw_hand_slot(ctx.world_state.player.right_hand, hud_x + right_offset, "R");

        // Draw tideclock HUD if player has tideclock equipped in either hand
        let has_tideclock_equipped = matches!(
            ctx.world_state.player.left_hand,
            Some(crate::tools::ToolType::TideClock)
        ) || matches!(
            ctx.world_state.player.right_hand,
            Some(crate::tools::ToolType::TideClock)
        );

        if has_tideclock_equipped && ctx.world_state.player.tool_tideclock.is_some() {
            let tide_percent = ctx.world_state.tide_percent();
            let tideclock_texture =
                crate::tools::ToolTideClock::get_frame_texture(tide_percent, &ctx.textures);

            // Draw tideclock in top left corner
            let clock_scale = 8.0;
            let clock_x = 16.0;
            let clock_y = 16.0;

            d.draw_texture_ex(
                tideclock_texture,
                Vector2::new(clock_x, clock_y),
                0.0,
                clock_scale,
                Color::WHITE,
            );
        }

        // Draw vignette effect when breath is critical
        if ctx.world_state.player.is_swimming {
            let breath_percent = (ctx.world_state.player.breath / max_breath)
                .max(0.0)
                .min(1.0);

            if breath_percent < 0.50 {
                let base_alpha = ((1.0 - breath_percent * 2.0) * 255.0) as u8;
                let color = Color::new(0, 0, 100, 0);
                let color_alpha = Color::new(0, 0, 100, base_alpha);
                let vin_size = 500;

                let screen_w = self.screen_width as i32;
                let screen_h = self.screen_height as i32;

                d.draw_rectangle_gradient_v(0, 0, screen_w, vin_size, color_alpha, color);
                d.draw_rectangle_gradient_v(
                    0,
                    screen_h - vin_size,
                    screen_w,
                    vin_size,
                    color,
                    color_alpha,
                );
                d.draw_rectangle_gradient_h(0, 0, vin_size, screen_h, color_alpha, color);
                d.draw_rectangle_gradient_h(
                    screen_w - vin_size,
                    0,
                    vin_size,
                    screen_h,
                    color,
                    color_alpha,
                );
            }
        }

        if ctx.debug_enabled {
            let player_x = ctx.world_state.player.position.x;
            let player_y = ctx.world_state.player.position.y;
            let chunk_x = (player_x as i32).div_euclid(CHUNK_SIZE as i32);
            let chunk_y = (player_y as i32).div_euclid(CHUNK_SIZE as i32);
            let local_x = (player_x as i32).rem_euclid(CHUNK_SIZE as i32);
            let local_y = (player_y as i32).rem_euclid(CHUNK_SIZE as i32);

            d.draw_text(
                &format!(
                    "Pos: ({:.2}, {:.2}) | Chunk: ({}, {}) | Local: ({}, {})",
                    player_x, player_y, chunk_x, chunk_y, local_x, local_y
                ),
                10,
                10,
                20,
                Color::DARKGRAY,
            );
            d.draw_text(
                "Controls: WASD/Arrows=Move, Space=Jump, C=Climb, Shift/X=Dash, Toggle Controls=/",
                500,
                10,
                16,
                Color::BLACK,
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
                    "On Ground: {}   Is Climbing {}   Is Sliding {}   Is Dashing {}   Is Swimming {}   On Ladder {}",
                    ctx.world_state.player.on_ground,
                    ctx.world_state.player.is_climbing,
                    ctx.world_state.player.is_sliding,
                    ctx.world_state.player.is_dashing,
                    ctx.world_state.player.is_swimming,
                    ctx.world_state.player.is_on_ladder
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
            d.draw_text(
                &format!(
                    "Darkness: {:.2}",
                    ctx.world_state.lighting_system.ambient_darkness
                ),
                10,
                110,
                20,
                Color::DARKGRAY,
            );
        }

        // Draw crafting station or chest prompt at bottom of screen
        let crafting_station = ctx
            .world_state
            .player
            .get_intersecting_crafting_station(&ctx.world_state.terrain);

        let prompt: String = if let Some((_x, _y, name)) = ctx.world_state.player.nearby_pickup {
            format!("Press E to pickup {}", name)
        } else if let Some(station) = crafting_station {
            use crate::terrain::Block;
            match station {
                Block::Workbench => "Press M to use Workbench".to_string(),
                Block::Anvil => "Press M to use Anvil".to_string(),
                Block::Furnace => "Press M to use Furnace".to_string(),
                _ => String::new(),
            }
        } else if let Some(chest_pos) = ctx
            .world_state
            .player
            .get_intersecting_chest(&ctx.world_state.terrain)
        {
            // Only show prompt if chest has an inventory
            if ctx.world_state.chests.contains_key(&chest_pos) {
                "Press M to open Chest".to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if !prompt.is_empty() {
            let text_width = d.measure_text(&prompt, 20);
            let text_x = (self.screen_width - text_width as f32) / 2.0;
            let text_y = self.screen_height - 300.0;

            // Draw background box
            d.draw_rectangle(
                text_x as i32 - 10,
                text_y as i32 - 5,
                text_width + 20,
                30,
                Color::new(0, 0, 0, 180),
            );

            // Draw text
            d.draw_text(&prompt, text_x as i32, text_y as i32, 20, Color::WHITE);
        }

        // Draw help button in bottom right corner
        let help_button_color = if self.help_hovered {
            Color::new(91, 110, 225, 255)
        } else {
            Color::new(60, 60, 80, 200)
        };
        d.draw_rectangle_rec(self.help_button, help_button_color);
        d.draw_rectangle_lines_ex(self.help_button, 2.0, Color::WHITE);

        // Draw "?" in the center of the button
        let help_text = "?";
        let help_text_size = 28;
        let help_text_width = d.measure_text(help_text, help_text_size);
        d.draw_text(
            help_text,
            self.help_button.x as i32 + (self.help_button.width as i32 - help_text_width) / 2,
            self.help_button.y as i32 + (self.help_button.height as i32 - help_text_size) / 2,
            help_text_size,
            Color::WHITE,
        );
    }
}
