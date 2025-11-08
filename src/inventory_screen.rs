use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

const RENDER_WIDTH: u32 = 400;
const RENDER_HEIGHT: u32 = 225;

pub struct InventoryScreen {
    render_target: Option<RenderTexture2D>,
}

impl InventoryScreen {
    pub fn new() -> Self {
        InventoryScreen {
            render_target: None,
        }
    }
}

impl Screen for InventoryScreen {
    type Context = GameContext;

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        if ctx.controller.menu_pressed {
            return ScreenCommand::Pop;
        }
        ScreenCommand::None
    }

    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, ctx: &Self::Context) {
        if self.render_target.is_none() {
            self.render_target = Some(
                rl.load_render_texture(thread, RENDER_WIDTH, RENDER_HEIGHT)
                    .unwrap(),
            );
        }

        let render_target = self.render_target.as_mut().unwrap();

        {
            let mut d = rl.begin_texture_mode(thread, render_target);
            d.clear_background(Color::RAYWHITE);

            // Draw inventory slots grid on the left side
            let slot_texture = &ctx.textures.ui.inventory_slot;
            let slot_size = slot_texture.width as f32;
            let slot_padding = 8.0;

            let grid_cols = 6;
            let grid_rows = 4;

            let grid_width =
                (slot_size * grid_cols as f32) + (slot_padding * (grid_cols - 1) as f32);
            let grid_height =
                (slot_size * grid_rows as f32) + (slot_padding * (grid_rows - 1) as f32);

            let grid_x = 8.0;
            let grid_y = 8.0;

            for row in 0..grid_rows {
                for col in 0..grid_cols {
                    let slot_x = grid_x + (col as f32 * (slot_size + slot_padding));
                    let slot_y = grid_y + (row as f32 * (slot_size + slot_padding));

                    // TODO: check and draw item over box
                    d.draw_texture(slot_texture, slot_x as i32, slot_y as i32, Color::WHITE);
                }
            }

            // Draw equip UI on the right side
            let equip_texture = &ctx.textures.ui.equip;
            let texture_width = equip_texture.width as f32;
            let texture_height = equip_texture.height as f32;

            let equip_x = RENDER_WIDTH as f32 - texture_width;
            let equip_y = (RENDER_HEIGHT as f32 - texture_height) / 2.0;

            d.draw_texture(equip_texture, equip_x as i32, equip_y as i32, Color::WHITE);


            // Draw equiped tools
            let left_hand_x = 268.0;
            let left_hand_y = 40.0;
            let right_hand_x = 350.0;
            let right_hand_y = left_hand_y;
            let helm_x = (right_hand_x + left_hand_x) / 2.0;
            let helm_y = 40.0;
            let armor_x = helm_x;
            let armor_y = 80.0;
            let boots_x = helm_x;
            let boots_y = 120.0;

            d.draw_texture(slot_texture, left_hand_x as i32, left_hand_y as i32, Color::WHITE);
            d.draw_texture(slot_texture, right_hand_x as i32, right_hand_y as i32, Color::WHITE);
            d.draw_texture(slot_texture, helm_x as i32, helm_y as i32, Color::WHITE);
            d.draw_texture(slot_texture, armor_x as i32, armor_y as i32, Color::WHITE);
            d.draw_texture(slot_texture, boots_x as i32, boots_y as i32, Color::WHITE);


            // Draw tool selection
            let tool_cols = 9;
            let tool_y = 190;
            for col in 0..tool_cols {
                let tool_x = slot_padding * 2.0 + (col as f32 * (slot_size + slot_padding));
                // TODO: check and draw item over box
                d.draw_texture(slot_texture, tool_x as i32, tool_y as i32, Color::WHITE);
            }
        }



        {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);

            let screen_width = d.get_screen_width() as f32;
            let screen_height = d.get_screen_height() as f32;

            d.draw_texture_pro(
                &render_target.texture(),
                Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, -(RENDER_HEIGHT as f32)),
                Rectangle::new(0.0, 0.0, screen_width, screen_height),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
    }
}
