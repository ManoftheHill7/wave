use raylib::prelude::*;
use crate::world::WorldState;
use crate::player::Player;
use crate::terrain::{Terrain, BlockType};
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

            if block.block_type != BlockType::Air {
                let texture = match block.block_type {
                    BlockType::Dirt => &textures.tiles.dirt,
                    BlockType::Stone => &textures.tiles.stone,
                    BlockType::Grass => &textures.tiles.grass,
                    BlockType::Sand => &textures.tiles.sand,
                    BlockType::Lava => &textures.tiles.lava,
                    BlockType::Water => &textures.tiles.water,
                    BlockType::Log => &textures.tiles.log,
                    BlockType::Leaf => &textures.tiles.leaves,
                    BlockType::Air => continue,
                };

                render_tile(d, x as f32, y as f32, texture);
            }
        }
    }
}

// Color palettes: (original_0, replace_0)
const COLOR_PALETTES: &[([f32; 4], [f32; 4])] = &[
    // Blue scarf
    (
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // original_0: red scarf
        [0.0, 0.5, 1.0, 1.0],      // replace_0: blue scarf
    ),
    // Red scarf
    (
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // original_0: red scarf
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // replace_0: red scarf
    ),
    // Extra dash palette - pink scarf
    (
        [172.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 1.0], // original_0: red scarf
        [172.0 / 255.0, 50.0 / 255.0, 172.0 / 255.0, 1.0], // replace_0: red scarf
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

    const walkFrameLength: f32 = 0.1;
    const fallingFrameLength: f32 = 0.2;
    const idleFrameLength: f32 = 0.25;
    const swimmingFrameLength: f32 = 0.18;

    let mut rotation = 0.0;

    let pt = &textures.player.sammi;
    let texture = if player.is_dashing {
        rotation = player.velocity.y.atan2(player.velocity.x).to_degrees();
        animate!(&pt.dash, walkFrameLength)
    } else if player.is_climbing {
        if player.velocity.y != 0.0 {
            animate!(pt.climb, walkFrameLength)
        } else {
            &pt.climb[0]
        }
    } else if player.is_swimming {
        animate!(pt.swimming, swimmingFrameLength)
    } else if player.is_sliding {
        &pt.sliding
    } else if player.on_ground {
        if player.velocity.x != 0.0 {
            animate!(pt.walk, walkFrameLength)
        } else {
            animate!(pt.idle, idleFrameLength)
        }
    } else {
        if player.velocity.y > 0.0 {
            animate!(pt.falling, fallingFrameLength)
        } else {
            &pt.jumping
        }
    };
    let rw = 1.3;
    let rh = 2.0;
    let fudgeX = 1.3;
    let fudgeY = 0.5;
    let fudgeX_off = 0.75;
    let fudgeY_off = 1.0;
    let tw = rw * pixels_per_world_unit() * (1.0 + fudgeX);
    let th = rh * pixels_per_world_unit() * (1.0 + fudgeY);

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
                player.position.x * pixels_per_world_unit() - pixels_per_world_unit() * fudgeX_off + tw / 2.0,
                player.position.y * pixels_per_world_unit() - pixels_per_world_unit() * fudgeY_off + th / 2.0,
                tw, th
            ),
            Vector2::new(tw / 2.0, th / 2.0),
            rotation,
            Color::WHITE
        );
    }

    // Selected tile
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
    if true {
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
