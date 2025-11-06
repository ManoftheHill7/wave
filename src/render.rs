use raylib::prelude::*;
use crate::world::WorldState;
use crate::player::Player;
use crate::terrain::{Terrain, Block, CELL_RESOLUTION, NO_LIQUID_THRESHOLD};
use crate::{TextureManager, pixels_per_world_unit};

type ShaderLocs = (i32, i32, i32, i32);

pub fn render(
    d: &mut RaylibDrawHandle,
    world: &WorldState,
    textures: &TextureManager,
    player_shader: &mut Shader,
    shader_locs: ShaderLocs
) {
    let px = world.player.position.x as i32;
    let py = world.player.position.y as i32;

    render_terrain(d, &world.terrain, px, py, textures);
    render_player(d, &world.player, textures, player_shader, shader_locs);
}

pub fn render_terrain(d: &mut RaylibDrawHandle, terrain: &Terrain, px: i32, py: i32, textures: &TextureManager) {
    let range = 900 / pixels_per_world_unit() as i32;
    for x in (px - range)..(px + range) {
        for y in (py - range)..(py + range) {
            let block = terrain.at(x, y);

            if block != Block::Air {
                let texture = match block {
                    Block::Dirt => &textures.tiles.dirt,
                    Block::Stone => &textures.tiles.stone,
                    Block::Grass => &textures.tiles.grass,
                    Block::Sand => &textures.tiles.sand,
                    Block::Lava => &textures.tiles.lava,
                    Block::Water => &textures.tiles.water,
                    Block::Log => &textures.tiles.log,
                    Block::Leaf => &textures.tiles.leaves,
                    Block::Air => continue,
                };

                render_tile(d, x as f32, y as f32, texture);
            } else {
                for cell_y in 0..CELL_RESOLUTION {
                    for cell_x in 0..CELL_RESOLUTION {
                        render_water(d, terrain,
                            x as f32 + cell_x as f32 / CELL_RESOLUTION as f32,
                            y as f32 + cell_y as f32 / CELL_RESOLUTION as f32
                        );
                    }
                }

            }
        }
    }
}

// Maps to uniforms (original_0, replace_0)
const COLOR_PALETTES: &[([f32; 4], [f32; 4])] = &[
    // Blue scarf
    (
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0],
        [0.0, 0.5, 1.0, 1.0],
    ),
    // Red scarf
    (
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0],
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0],
    ),
    // Pink scarf
    (
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0],
        [172.0 / 255.0, 50.0 / 255.0, 172.0 / 255.0, 1.0],
    ),
];

pub fn render_player(
    d: &mut RaylibDrawHandle,
    player: &Player,
    textures: &TextureManager,
    shader: &mut Shader,
    shader_locs: ShaderLocs
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

    let mut rotation = 0.0;

    let pt = &textures.player.sammi;
    let texture = if player.is_dashing {
        rotation = player.velocity.y.atan2(player.velocity.x).to_degrees();
        animate!(&pt.dash, WALK_FRAME_LENGTH)
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
    let flicker = (exhaustion_level > 0.5) && player.is_climbing && (
        (exhaustion_level > 0.5 && exhaustion_level < 0.51) ||
        (exhaustion_level > 0.6 && exhaustion_level < 0.61) ||
        (exhaustion_level > 0.7 && exhaustion_level < 0.71) ||
        (exhaustion_level > 0.8 && exhaustion_level < 0.81) ||
        (exhaustion_level > 0.85 && exhaustion_level < 0.86) ||
        (exhaustion_level > 0.9 && exhaustion_level < 0.91) ||
        (exhaustion_level > 0.95 && exhaustion_level < 0.96)
    );

    let (loc_original_0, loc_replace_0, loc_exhustion, loc_whiteout) = shader_locs;


    unsafe {
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_original_0,
            palette.0.as_ptr() as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32
        );
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_replace_0,
            palette.1.as_ptr() as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32
        );

        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_exhustion,
            &exhaustion_level as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32
        );

        let whiteout = if flicker { 1.0f32 } else { 0.0f32 };
        raylib::ffi::SetShaderValue(
            shader.as_ref().clone(),
            loc_whiteout,
            &whiteout as *const f32 as *const std::ffi::c_void,
            raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32
        );
    }

    {
        let mut d_shader = d.begin_shader_mode(shader);

        d_shader.draw_texture_pro(
            texture,
            Rectangle::new(
                0.0,
                0.0,
                if player.facing_dir < 0 {-1 * texture.width } else { texture.width } as f32,
                texture.height as f32
            ),
            Rectangle::new(
                player.position.x * pixels_per_world_unit() - pixels_per_world_unit() * fudge_x_off + tw / 2.0,
                player.position.y * pixels_per_world_unit() - pixels_per_world_unit() * fudge_y_off + th / 2.0,
                tw, th
            ),
            Vector2::new(tw / 2.0, th / 2.0),
            rotation,
            Color::WHITE
        );
    }

    let player_center_x = (player.position.x + player.width / 2.0) *
        pixels_per_world_unit();
    let player_center_y = (player.position.y + player.height / 2.0) *
        pixels_per_world_unit();
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
            Color::GREEN
        );
    }
    if false {
        d.draw_line_ex(
            Vector2::new(player_center_x, player_center_y),
            Vector2::new(raycast_end_x, raycast_end_y),
            2.0,
            Color::GREEN
        );

        d.draw_rectangle_lines(
            (player.position.x * pixels_per_world_unit()) as i32,
            (player.position.y * pixels_per_world_unit()) as i32,
            (player.width * pixels_per_world_unit()) as i32,
            (player.height * pixels_per_world_unit()) as i32,
            Color::RED);
    }
}

pub fn render_water(d: &mut RaylibDrawHandle, terrain: &Terrain, x: f32, y: f32) {
    let ld = terrain.liquid_at(x, y);
    let amount = ld.volume;
    if amount > NO_LIQUID_THRESHOLD {
        let volume_clamped = amount.min(1.0).max(0.0);

        let intensity = volume_clamped.powf(0.5);
        let r = (100.0 * (1.0 - intensity)) as u8;
        let g = (150.0 * (1.0 - intensity) + 100.0 * intensity) as u8;
        let b = (255.0 * (0.3 + 0.7 * intensity)) as u8;
        let a = (255.0 * intensity.max(0.3)) as u8;

        let cell_size: i32 = (pixels_per_world_unit() / CELL_RESOLUTION as f32) as i32;
        let fill_height: i32 = if ld.flow_down { cell_size } else { (cell_size as f32 * volume_clamped) as i32 };
        let y_offset: i32 = cell_size - fill_height;

        d.draw_rectangle(
            (x * pixels_per_world_unit()) as i32,
            (y * pixels_per_world_unit()) as i32 + y_offset,
            cell_size,
            fill_height,
            Color::new(r, g, b, a)
        );
    }
}

pub fn render_tile(d: &mut RaylibDrawHandle, x: f32, y: f32, texture: &Texture2D) {
    d.draw_texture_pro(
        texture,
        Rectangle::new(
            0.0,
            0.0,
            texture.width as f32,
            texture.height as f32
        ),
        Rectangle::new(
            x * pixels_per_world_unit(),
            y * pixels_per_world_unit(),
            pixels_per_world_unit(),
            pixels_per_world_unit()
        ),
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE
    );
}
