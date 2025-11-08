use crate::inventory::ItemType;
use crate::GameContext;
use crate::TextureManager;
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

    fn handle_mouse_click(&mut self, ctx: &mut GameContext) {
        // Get normalized mouse position (0.0-1.0) from controller
        let mouse_pos = ctx.controller.mouse_position;

        // Convert normalized coordinates to render texture space (0-400, 0-225)
        let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;
        let mouse_y = mouse_pos.y * RENDER_HEIGHT as f32;

        // Check if click is in inventory grid
        let slot_size = 32.0;
        let slot_padding = 8.0;
        let grid_cols = 6;
        let grid_rows = 4;
        let grid_x = 8.0;
        let grid_y = 20.0;

        // Collect inventory items
        let inventory_items: Vec<_> = ctx.world_state.player.inventory.iter().collect();

        // Check each slot
        for (slot_index, item_stack) in inventory_items.iter().enumerate() {
            if slot_index >= (grid_cols * grid_rows) {
                break;
            }

            let col = slot_index % grid_cols;
            let row = slot_index / grid_cols;
            let slot_x = grid_x + (col as f32 * (slot_size + slot_padding));
            let slot_y = grid_y + (row as f32 * (slot_size + slot_padding));

            // Check if mouse is over this slot
            if mouse_x >= slot_x
                && mouse_x <= slot_x + slot_size
                && mouse_y >= slot_y
                && mouse_y <= slot_y + slot_size
            {
                // Set the selected item
                ctx.world_state.player.place_block_type = Some(item_stack.item_type);
                break;
            }
        }
    }
}

pub fn get_item_texture<'a>(item_type: &ItemType, textures: &'a TextureManager) -> &'a Texture2D {
    match item_type {
        ItemType::Stone => &textures.tiles.stone,
        ItemType::Dirt => &textures.tiles.dirt,
    }
}

impl Screen for InventoryScreen {
    type Context = GameContext;

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        if ctx.controller.menu_pressed {
            return ScreenCommand::Pop;
        }

        // Handle mouse clicks for item selection
        if ctx.controller.use_tool_pressed {
            self.handle_mouse_click(ctx);
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

            // Display weight information above the inventory grid
            let weight_text = format!(
                "Weight: {:.1}/{:.1}",
                ctx.world_state.player.inventory.current_weight(),
                ctx.world_state.player.inventory.max_weight()
            );

            let weight_x = 8.0;
            let weight_y = 8.0;

            d.draw_text(
                &weight_text,
                weight_x as i32,
                weight_y as i32,
                10,
                Color::BLACK,
            );

            // Draw inventory slots grid on the left side
            let slot_texture = &ctx.textures.ui.inventory_slot;
            let slot_size = slot_texture.width as f32;
            let slot_padding = 8.0;

            let grid_cols = 6;
            let grid_rows = 4;

            let grid_x = 8.0;
            let grid_y = 20.0;

            for row in 0..grid_rows {
                for col in 0..grid_cols {
                    let slot_x = grid_x + (col as f32 * (slot_size + slot_padding));
                    let slot_y = grid_y + (row as f32 * (slot_size + slot_padding));

                    d.draw_texture(slot_texture, slot_x as i32, slot_y as i32, Color::WHITE);
                }
            }

            // Collect inventory items for rendering
            let inventory_items: Vec<_> = ctx.world_state.player.inventory.iter().collect();

            // Render items on top of slots
            for (slot_index, item_stack) in inventory_items.iter().enumerate() {
                if slot_index >= (grid_cols * grid_rows) {
                    break; // Don't overflow the grid
                }

                let col = slot_index % grid_cols;
                let row = slot_index / grid_cols;
                let slot_x = grid_x + (col as f32 * (slot_size + slot_padding));
                let slot_y = grid_y + (row as f32 * (slot_size + slot_padding));

                // Get texture for this item type
                let item_texture = get_item_texture(&item_stack.item_type, &ctx.textures);

                // Draw item texture scaled to fit in slot (8x8 texture → 32x32 slot = 4x scale)
                let texture_scale = slot_size / item_texture.width as f32;
                d.draw_texture_ex(
                    item_texture,
                    Vector2::new(slot_x, slot_y),
                    0.0,
                    texture_scale,
                    Color::WHITE,
                );

                // Draw item count in bottom-right corner
                let count_text = item_stack.count.to_string();
                let text_size = 10;

                // Position text in bottom-right corner with small padding
                let text_x = slot_x + slot_size - 12.0; // 12px from right for padding
                let text_y = slot_y + slot_size - 12.0; // 12px from bottom for padding

                d.draw_text(
                    &count_text,
                    text_x as i32 + 1,
                    text_y as i32 + 1,
                    10,
                    Color::WHITE,
                );
                d.draw_text(
                    &count_text,
                    text_x as i32,
                    text_y as i32,
                    text_size,
                    Color::BLACK,
                );
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

            d.draw_texture(
                slot_texture,
                left_hand_x as i32,
                left_hand_y as i32,
                Color::WHITE,
            );
            d.draw_texture(
                slot_texture,
                right_hand_x as i32,
                right_hand_y as i32,
                Color::WHITE,
            );
            d.draw_texture(slot_texture, helm_x as i32, helm_y as i32, Color::WHITE);
            d.draw_texture(slot_texture, armor_x as i32, armor_y as i32, Color::WHITE);
            d.draw_texture(slot_texture, boots_x as i32, boots_y as i32, Color::WHITE);

            // Draw selected item in right hand slot if set
            if let Some(selected_item) = ctx.world_state.player.place_block_type {
                let item_texture = get_item_texture(&selected_item, &ctx.textures);
                let texture_scale = slot_size / item_texture.width as f32;
                d.draw_texture_ex(
                    item_texture,
                    Vector2::new(right_hand_x, right_hand_y),
                    0.0,
                    texture_scale,
                    Color::WHITE,
                );
            }

            // Draw tool selection
            let tool_y = 190;

            let mut draw_tool = |col: usize, tool_texture: Option<&Texture2D>| {
                let tool_x = slot_padding * 2.0 + (col as f32 * (slot_size + slot_padding));
                d.draw_texture(slot_texture, tool_x as i32, tool_y as i32, Color::WHITE);
                if let Some(text) = tool_texture {
                    d.draw_texture_ex(text, Vector2::new(tool_x, tool_y as f32), 0.0, 4.0, Color::WHITE);
                }
            };

            draw_tool(0, ctx.world_state.player.tool_dash.as_ref().map(|_| &ctx.textures.items.dashamulet)); // TODO: get different textures based on tool level
            draw_tool(1, None); // Pickaxe
            draw_tool(2, None); // Grappling hook
            draw_tool(3, None); // Spear
            draw_tool(4, None); // Lamp
            draw_tool(5, None); // Fishing rod
            draw_tool(6, None); // Glider

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
