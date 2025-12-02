use crate::inventory::ItemType;
use crate::GameContext;
use crate::TextureManager;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

const RENDER_WIDTH: u32 = 400;
const RENDER_HEIGHT: u32 = 225;

pub struct ChestScreen {
    render_target: Option<RenderTexture2D>,
    scroll_offset_player: usize,
    scroll_offset_chest: usize,
    chest_position: (i32, i32),
}

impl ChestScreen {
    pub fn new(chest_position: (i32, i32)) -> Self {
        ChestScreen {
            render_target: None,
            scroll_offset_player: 0,
            scroll_offset_chest: 0,
            chest_position,
        }
    }

    fn handle_mouse_click(&mut self, ctx: &mut GameContext, shift_held: bool) {
        // Get normalized mouse position (0.0-1.0) from controller
        let mouse_pos = ctx.controller.mouse_position;

        // Convert normalized coordinates to render texture space (0-400, 0-225)
        let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;
        let mouse_y = mouse_pos.y * RENDER_HEIGHT as f32;

        let slot_size = 24.0;
        let slot_padding = 4.0;
        let grid_cols = 6;
        let grid_rows = 4;

        // Player inventory grid (left side)
        let player_grid_x = 8.0;
        let player_grid_y = 40.0;

        // Chest inventory grid (right side)
        let chest_grid_x = 210.0;
        let chest_grid_y = 40.0;

        // Check player inventory grid
        for row in 0..grid_rows {
            for col in 0..grid_cols {
                let slot_x = player_grid_x + (col as f32 * (slot_size + slot_padding));
                let slot_y = player_grid_y + (row as f32 * (slot_size + slot_padding));

                if mouse_x >= slot_x
                    && mouse_x <= slot_x + slot_size
                    && mouse_y >= slot_y
                    && mouse_y <= slot_y + slot_size
                {
                    let slot_index = row * grid_cols + col;
                    let actual_index = slot_index + self.scroll_offset_player;

                    // Get item at this slot
                    let inventory_items: Vec<_> = ctx.world_state.player.inventory.iter().collect();
                    if let Some(item_stack) = inventory_items.get(actual_index) {
                        // Transfer from player to chest
                        if let Some(chest_inv) =
                            ctx.world_state.chests.get_mut(&self.chest_position)
                        {
                            Self::transfer_item(
                                &mut ctx.world_state.player.inventory,
                                chest_inv,
                                item_stack.item_type,
                                shift_held,
                            );
                        }
                    }
                    return;
                }
            }
        }

        // Check chest inventory grid
        for row in 0..grid_rows {
            for col in 0..grid_cols {
                let slot_x = chest_grid_x + (col as f32 * (slot_size + slot_padding));
                let slot_y = chest_grid_y + (row as f32 * (slot_size + slot_padding));

                if mouse_x >= slot_x
                    && mouse_x <= slot_x + slot_size
                    && mouse_y >= slot_y
                    && mouse_y <= slot_y + slot_size
                {
                    let slot_index = row * grid_cols + col;
                    let actual_index = slot_index + self.scroll_offset_chest;

                    // Get item at this slot in chest
                    if let Some(chest_inv) = ctx.world_state.chests.get(&self.chest_position) {
                        let chest_items: Vec<_> = chest_inv.iter().collect();
                        if let Some(item_stack) = chest_items.get(actual_index) {
                            let item_type = item_stack.item_type;
                            // Transfer from chest to player
                            if let Some(chest_inv) =
                                ctx.world_state.chests.get_mut(&self.chest_position)
                            {
                                Self::transfer_item(
                                    chest_inv,
                                    &mut ctx.world_state.player.inventory,
                                    item_type,
                                    shift_held,
                                );
                            }
                        }
                    }
                    return;
                }
            }
        }
    }

    fn transfer_item(
        from: &mut crate::inventory::Inventory,
        to: &mut crate::inventory::Inventory,
        item: ItemType,
        shift_held: bool,
    ) {
        let available = from.count(item);
        if available == 0 {
            return;
        }

        let amount = if shift_held { available } else { 1 };

        // Check weight limit
        let item_weight = item.weight();
        let can_add = (to.available_weight() / item_weight).floor() as u32;
        let actual = amount.min(can_add);

        if actual > 0 {
            from.take(item, actual);
            to.add(item, actual);
        }
    }
}

fn get_item_texture<'a>(item_type: &ItemType, textures: &'a TextureManager) -> &'a Texture2D {
    item_type.get_texture(textures)
}

impl Screen for ChestScreen {
    type Context = GameContext;

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        // Update music streams (keep game music playing)
        ctx.music.update_streams();

        if ctx.controller.menu_pressed {
            return ScreenCommand::Pop;
        }

        // Handle scrolling with mouse wheel
        let grid_cols = 6;
        let grid_rows = 4;
        let visible_slots = grid_cols * grid_rows;

        // Get mouse position to determine which grid to scroll
        let mouse_pos = ctx.controller.mouse_position;
        let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;

        // Left half = player inventory, right half = chest inventory
        let is_over_player = mouse_x < RENDER_WIDTH as f32 / 2.0;

        if ctx.controller.mouse_wheel_move != 0.0 {
            if is_over_player {
                let total_items = ctx.world_state.player.inventory.unique_items();
                let max_scroll_rows = if total_items > visible_slots {
                    ((total_items - visible_slots) + grid_cols - 1) / grid_cols
                } else {
                    0
                };

                if ctx.controller.mouse_wheel_move > 0.0 {
                    if self.scroll_offset_player > 0 {
                        self.scroll_offset_player =
                            self.scroll_offset_player.saturating_sub(grid_cols);
                    }
                } else {
                    let new_offset = self.scroll_offset_player + grid_cols;
                    if new_offset / grid_cols <= max_scroll_rows {
                        self.scroll_offset_player = new_offset;
                    }
                }
            } else {
                // Chest inventory scrolling
                let total_items = ctx
                    .world_state
                    .chests
                    .get(&self.chest_position)
                    .map(|c| c.unique_items())
                    .unwrap_or(0);
                let max_scroll_rows = if total_items > visible_slots {
                    ((total_items - visible_slots) + grid_cols - 1) / grid_cols
                } else {
                    0
                };

                if ctx.controller.mouse_wheel_move > 0.0 {
                    if self.scroll_offset_chest > 0 {
                        self.scroll_offset_chest =
                            self.scroll_offset_chest.saturating_sub(grid_cols);
                    }
                } else {
                    let new_offset = self.scroll_offset_chest + grid_cols;
                    if new_offset / grid_cols <= max_scroll_rows {
                        self.scroll_offset_chest = new_offset;
                    }
                }
            }
        }

        // Handle mouse clicks for item transfer (shift = stack)
        if ctx.controller.left_hand_pressed {
            let shift_held = ctx.controller.climb_pressed;
            self.handle_mouse_click(ctx, shift_held);
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
            let stone_background = &ctx.textures.tiles.stone;
            let stone_scale = 4.0;
            let stone_size = stone_background.width as f32 * stone_scale;
            let stone_grid_cols = 13;
            let stone_grid_rows = 8;

            for row in 0..stone_grid_rows {
                for col in 0..stone_grid_cols {
                    let stone_grid_x = col as f32 * stone_size;
                    let stone_grid_y = row as f32 * stone_size;

                    d.draw_texture_ex(
                        stone_background,
                        Vector2::new(stone_grid_x, stone_grid_y),
                        0.0,
                        stone_scale,
                        Color::GRAY,
                    );
                }
            }

            let slot_texture = &ctx.textures.ui.inventory_slot;
            let slot_size = slot_texture.width as f32;
            let slot_padding = 4.0;
            let grid_cols = 6;
            let grid_rows = 4;

            // Player inventory (left side)
            let player_grid_x = 8.0;
            let player_grid_y = 40.0;

            // Chest inventory (right side)
            let chest_grid_x = 210.0;
            let chest_grid_y = 40.0;

            // Draw titles
            d.draw_text("Your Inventory", 8, 8, 12, Color::new(223, 113, 38, 255));
            d.draw_text("Chest", 210, 8, 12, Color::LIGHTSEAGREEN);

            // Draw weight info
            let player_weight_text = format!(
                "Weight: {:.1}/{:.1}",
                ctx.world_state.player.inventory.current_weight().abs(),
                ctx.world_state.player.inventory.max_weight()
            );
            if ctx.world_state.player.inventory.current_weight()
                >= ctx.world_state.player.inventory.max_weight() * 0.9
            {
                d.draw_text(&player_weight_text, 8, 22, 10, Color::RED);
            } else {
                d.draw_text(&player_weight_text, 8, 22, 10, Color::DARKGRAY);
            }

            if let Some(chest_inv) = ctx.world_state.chests.get(&self.chest_position) {
                let chest_weight_text = format!(
                    "Weight: {:.1}/{:.1}",
                    chest_inv.current_weight().abs(),
                    chest_inv.max_weight()
                );
                if chest_inv.current_weight() >= chest_inv.max_weight() * 0.9 {
                    d.draw_text(&chest_weight_text, 210, 22, 10, Color::RED);
                } else {
                    d.draw_text(&chest_weight_text, 210, 22, 10, Color::DARKGRAY);
                }
            }

            // Draw player inventory slots
            for row in 0..grid_rows {
                for col in 0..grid_cols {
                    let slot_x = player_grid_x + (col as f32 * (slot_size + slot_padding));
                    let slot_y = player_grid_y + (row as f32 * (slot_size + slot_padding));
                    d.draw_texture(slot_texture, slot_x as i32, slot_y as i32, Color::WHITE);
                }
            }

            // Draw chest inventory slots
            for row in 0..grid_rows {
                for col in 0..grid_cols {
                    let slot_x = chest_grid_x + (col as f32 * (slot_size + slot_padding));
                    let slot_y = chest_grid_y + (row as f32 * (slot_size + slot_padding));
                    d.draw_texture(slot_texture, slot_x as i32, slot_y as i32, Color::WHITE);
                }
            }

            // Draw player inventory items
            let player_items: Vec<_> = ctx.world_state.player.inventory.iter().collect();
            for (visible_index, item_stack) in player_items
                .iter()
                .skip(self.scroll_offset_player)
                .enumerate()
            {
                if visible_index >= (grid_cols * grid_rows) {
                    break;
                }

                let col = visible_index % grid_cols;
                let row = visible_index / grid_cols;
                let slot_x = player_grid_x + (col as f32 * (slot_size + slot_padding));
                let slot_y = player_grid_y + (row as f32 * (slot_size + slot_padding));

                let item_texture = get_item_texture(&item_stack.item_type, &ctx.textures);
                d.draw_texture_ex(
                    item_texture,
                    Vector2::new(slot_x, slot_y),
                    0.0,
                    1.0,
                    Color::WHITE,
                );

                // Draw item count in bottom-right corner
                let count_text = item_stack.count.to_string();
                let count_text_size = item_stack.count.ilog10() as i32 * 3;
                let text_size = 10;

                // Position text in bottom-right corner with small padding
                let text_x = slot_x + slot_size - count_text_size as f32 - 14.0; // 14px from right for padding
                let text_y = slot_y + slot_size - 4.0; // 4px from bottom for padding

                d.draw_text(
                    &count_text,
                    text_x as i32 + 1,
                    text_y as i32 - 1,
                    text_size,
                    Color::new(0, 0, 0, 127),
                );
                d.draw_text(
                    &count_text,
                    text_x as i32,
                    text_y as i32,
                    text_size,
                    Color::new(223, 113, 38, 255),
                );
            }

            // Draw chest inventory items
            if let Some(chest_inv) = ctx.world_state.chests.get(&self.chest_position) {
                let chest_items: Vec<_> = chest_inv.iter().collect();
                for (visible_index, item_stack) in chest_items
                    .iter()
                    .skip(self.scroll_offset_chest)
                    .enumerate()
                {
                    if visible_index >= (grid_cols * grid_rows) {
                        break;
                    }

                    let col = visible_index % grid_cols;
                    let row = visible_index / grid_cols;
                    let slot_x = chest_grid_x + (col as f32 * (slot_size + slot_padding));
                    let slot_y = chest_grid_y + (row as f32 * (slot_size + slot_padding));

                    let item_texture = get_item_texture(&item_stack.item_type, &ctx.textures);
                    d.draw_texture_ex(
                        item_texture,
                        Vector2::new(slot_x, slot_y),
                        0.0,
                        1.0,
                        Color::WHITE,
                    );

                    // Draw item count in bottom-right corner
                    let count_text = item_stack.count.to_string();
                    let count_text_size = item_stack.count.ilog10() as i32 * 3;
                    let text_size = 10;

                    // Position text in bottom-right corner with small padding
                    let text_x = slot_x + slot_size - count_text_size as f32 - 14.0; // 14px from right for padding
                    let text_y = slot_y + slot_size - 4.0; // 4px from bottom for padding

                    d.draw_text(
                        &count_text,
                        text_x as i32 + 1,
                        text_y as i32 - 1,
                        text_size,
                        Color::new(0, 0, 0, 127),
                    );
                    d.draw_text(
                        &count_text,
                        text_x as i32,
                        text_y as i32,
                        text_size,
                        Color::LIGHTSEAGREEN,
                    );
                }
            }

            // Draw instructions at bottom
            d.draw_text(
                "Click: transfer 1 | Shift+Click: transfer stack | Tab: close",
                8,
                RENDER_HEIGHT as i32 - 16,
                10,
                Color::DARKGRAY,
            );
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
