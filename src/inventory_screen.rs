use crate::inventory::ItemType;
use crate::terrain::Block;
use crate::tools::ToolType;
use crate::GameContext;
use crate::TextureManager;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

const RENDER_WIDTH: u32 = 400;
const RENDER_HEIGHT: u32 = 225;

pub struct InventoryScreen {
    render_target: Option<RenderTexture2D>,
    mute_button: Rectangle,
    scroll_offset: usize,
}

impl InventoryScreen {
    pub fn new() -> Self {
        // Mute button in top-right corner (in render texture space)
        let mute_size = 24.0;
        let mute_margin = 8.0;

        InventoryScreen {
            render_target: None,
            mute_button: Rectangle::new(
                RENDER_WIDTH as f32 - mute_size - mute_margin,
                mute_margin,
                mute_size,
                mute_size,
            ),
            scroll_offset: 0,
        }
    }

    fn handle_mouse_click(&mut self, ctx: &mut GameContext, is_left_click: bool) {
        // Get normalized mouse position (0.0-1.0) from controller
        let mouse_pos = ctx.controller.mouse_position;

        // Convert normalized coordinates to render texture space (0-400, 0-225)
        let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;
        let mouse_y = mouse_pos.y * RENDER_HEIGHT as f32;

        // Check if mute button is clicked (only on left click)
        if is_left_click {
            if mouse_x >= self.mute_button.x
                && mouse_x <= self.mute_button.x + self.mute_button.width
                && mouse_y >= self.mute_button.y
                && mouse_y <= self.mute_button.y + self.mute_button.height
            {
                ctx.music.toggle_mute();
                return;
            }
        }

        // Check if click is in inventory grid
        let slot_size = 24.0;
        let slot_padding = 4.0;
        let grid_cols = 6;
        let grid_rows = 4;
        let grid_x = 32.0;
        let grid_y = 40.0;

        // Collect inventory items
        let inventory_items: Vec<_> = ctx.world_state.player.inventory.iter().collect();

        // Check all grid slots
        for row in 0..grid_rows {
            for col in 0..grid_cols {
                let slot_x = grid_x + (col as f32 * (slot_size + slot_padding));
                let slot_y = grid_y + (row as f32 * (slot_size + slot_padding));

                // Check if mouse is over this slot
                if mouse_x >= slot_x
                    && mouse_x <= slot_x + slot_size
                    && mouse_y >= slot_y
                    && mouse_y <= slot_y + slot_size
                {
                    let slot_index = row * grid_cols + col;
                    let actual_index = slot_index + self.scroll_offset;

                    // Check if this slot has an item
                    if let Some(item_stack) = inventory_items.get(actual_index) {
                        let bt = Block::from_item_type(item_stack.item_type);
                        if let Some(bt) = bt {
                            let new_tool = Some(ToolType::PlaceBlock(bt));
                            Self::set_hand_with_swap(
                                &mut ctx.world_state.player.left_hand,
                                &mut ctx.world_state.player.right_hand,
                                new_tool,
                                is_left_click,
                            );
                        }
                    } else {
                        // Empty slot clicked, clear selection
                        if is_left_click {
                            ctx.world_state.player.left_hand = None;
                        } else {
                            ctx.world_state.player.right_hand = None;
                        }
                    }
                    return;
                }
            }
        }

        // Check tool selection slots
        let tool_y = 190.0;
        let tool_start_x = slot_padding * 2.0;

        // Check dash tool slot (column 0)
        if ctx.world_state.player.tool_dash.is_some() {
            let tool_x = tool_start_x + (0.0 * (slot_size + slot_padding));
            if mouse_x >= tool_x
                && mouse_x <= tool_x + slot_size
                && mouse_y >= tool_y
                && mouse_y <= tool_y + slot_size
            {
                Self::set_hand_with_swap(
                    &mut ctx.world_state.player.left_hand,
                    &mut ctx.world_state.player.right_hand,
                    Some(ToolType::Dash),
                    is_left_click,
                );
                return;
            }
        }

        // Check pickaxe tool slot (column 1)
        if ctx.world_state.player.tool_pickaxe.is_some() {
            let tool_x = tool_start_x + (1.0 * (slot_size + slot_padding));
            if mouse_x >= tool_x
                && mouse_x <= tool_x + slot_size
                && mouse_y >= tool_y
                && mouse_y <= tool_y + slot_size
            {
                Self::set_hand_with_swap(
                    &mut ctx.world_state.player.left_hand,
                    &mut ctx.world_state.player.right_hand,
                    Some(ToolType::Pickaxe),
                    is_left_click,
                );
                return;
            }
        }

        // Check lamp tool slot (column 4)
        let tool_x = tool_start_x + (4.0 * (slot_size + slot_padding));
        if mouse_x >= tool_x
            && mouse_x <= tool_x + slot_size
            && mouse_y >= tool_y
            && mouse_y <= tool_y + slot_size
        {
            Self::set_hand_with_swap(
                &mut ctx.world_state.player.left_hand,
                &mut ctx.world_state.player.right_hand,
                Some(ToolType::Lamp),
                is_left_click,
            );
            return;
        }

        // Check glider tool slot (column 6)
        if ctx.world_state.player.tool_glider.is_some() {
            let tool_x = tool_start_x + (6.0 * (slot_size + slot_padding));
            if mouse_x >= tool_x
                && mouse_x <= tool_x + slot_size
                && mouse_y >= tool_y
                && mouse_y <= tool_y + slot_size
            {
                Self::set_hand_with_swap(
                    &mut ctx.world_state.player.left_hand,
                    &mut ctx.world_state.player.right_hand,
                    Some(ToolType::Glider),
                    is_left_click,
                );
                return;
            }
        }

        // Check tideclock tool slot (column 7)
        if ctx.world_state.player.tool_tideclock.is_some() {
            let tool_x = tool_start_x + (7.0 * (slot_size + slot_padding));
            if mouse_x >= tool_x
                && mouse_x <= tool_x + slot_size
                && mouse_y >= tool_y
                && mouse_y <= tool_y + slot_size
            {
                Self::set_hand_with_swap(
                    &mut ctx.world_state.player.left_hand,
                    &mut ctx.world_state.player.right_hand,
                    Some(ToolType::TideClock),
                    is_left_click,
                );
                return;
            }
        }
    }

    /// Helper function to set a hand with swap logic
    /// If the new_tool is already in the other hand, swap the hands
    fn set_hand_with_swap(
        left_hand: &mut Option<ToolType>,
        right_hand: &mut Option<ToolType>,
        new_tool: Option<ToolType>,
        is_left_click: bool,
    ) {
        if is_left_click {
            // Setting left hand
            if new_tool == *right_hand {
                // Tool is already in right hand, swap them
                std::mem::swap(left_hand, right_hand);
            } else {
                *left_hand = new_tool;
            }
        } else {
            // Setting right hand
            if new_tool == *left_hand {
                // Tool is already in left hand, swap them
                std::mem::swap(left_hand, right_hand);
            } else {
                *right_hand = new_tool;
            }
        }
    }
}

pub fn get_item_texture<'a>(item_type: &ItemType, textures: &'a TextureManager) -> &'a Texture2D {
    item_type.get_texture(textures)
}

impl Screen for InventoryScreen {
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
        let total_items = ctx.world_state.player.inventory.unique_items();
        let max_scroll_rows = if total_items > visible_slots {
            ((total_items - visible_slots) + grid_cols - 1) / grid_cols
        } else {
            0
        };

        if ctx.controller.mouse_wheel_move > 0.0 {
            // Scroll up
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(grid_cols);
            }
        } else if ctx.controller.mouse_wheel_move < 0.0 {
            // Scroll down
            let new_offset = self.scroll_offset + grid_cols;
            if new_offset / grid_cols <= max_scroll_rows {
                self.scroll_offset = new_offset;
            }
        }

        // Handle mouse clicks for item selection
        if ctx.controller.left_hand_pressed {
            self.handle_mouse_click(ctx, true);
        }
        if ctx.controller.right_hand_pressed {
            self.handle_mouse_click(ctx, false);
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

            // Display scroll info if there are multiple pages
            let grid_cols = 6;
            let grid_rows = 4;
            let visible_slots = grid_cols * grid_rows;
            let total_items = ctx.world_state.player.inventory.unique_items();

            // Calculate how many rows we can scroll through
            let total_rows = (total_items + grid_cols - 1) / grid_cols;
            let max_scroll_row = total_rows.saturating_sub(grid_rows);

            if total_items > visible_slots {
                let current_row = self.scroll_offset / grid_cols;
                let scroll_text = format!("Row {}/{}", current_row + 1, max_scroll_row + 1);
                d.draw_text(
                    &scroll_text,
                    weight_x as i32,
                    (weight_y + 12.0) as i32,
                    10,
                    Color::new(100, 100, 100, 255),
                );
            }

            // Draw inventory slots grid on the left side
            let slot_texture = &ctx.textures.ui.inventory_slot;
            let slot_size = slot_texture.width as f32;
            let slot_padding = 4.0;

            let grid_x = 32.0;
            let grid_y = 40.0;

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
            for (visible_index, item_stack) in
                inventory_items.iter().skip(self.scroll_offset).enumerate()
            {
                if visible_index >= (grid_cols * grid_rows) {
                    break; // Don't overflow the grid
                }

                let col = visible_index % grid_cols;
                let row = visible_index / grid_cols;
                let slot_x = grid_x + (col as f32 * (slot_size + slot_padding));
                let slot_y = grid_y + (row as f32 * (slot_size + slot_padding));

                // Get texture for this item type
                let item_texture = get_item_texture(&item_stack.item_type, &ctx.textures);

                // Draw item texture scaled to fit in slot (24x24 texture → 24x24 slot)
                d.draw_texture_ex(
                    item_texture,
                    Vector2::new(slot_x, slot_y),
                    0.0,
                    1.0,
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
                    text_size,
                    Color::RAYWHITE,
                );
                d.draw_text(
                    &count_text,
                    text_x as i32,
                    text_y as i32,
                    text_size,
                    Color::RED,
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

            // Draw selected tool in left hand slot if set
            if let Some(tool) = ctx.world_state.player.left_hand {
                let texture = match tool {
                    ToolType::Pickaxe => ctx
                        .world_state
                        .player
                        .tool_pickaxe
                        .as_ref()
                        .map(|p| p.get_texture(&ctx.textures)),
                    ToolType::Dash => ctx
                        .world_state
                        .player
                        .tool_dash
                        .as_ref()
                        .map(|d| d.get_texture(&ctx.textures)),
                    ToolType::Glider => ctx
                        .world_state
                        .player
                        .tool_glider
                        .as_ref()
                        .map(|g| g.get_texture(&ctx.textures)),
                    ToolType::TideClock => ctx
                        .world_state
                        .player
                        .tool_tideclock
                        .as_ref()
                        .map(|_| &ctx.textures.items.tidalcave_clock),
                    _ => tool.get_texture(&ctx.textures),
                };
                if let Some(texture) = texture {
                    d.draw_texture_ex(
                        texture,
                        Vector2::new(left_hand_x, left_hand_y),
                        0.0,
                        1.0,
                        Color::WHITE,
                    );
                }
            }

            // Draw selected tool in right hand slot if set
            if let Some(tool) = ctx.world_state.player.right_hand {
                let texture = match tool {
                    ToolType::Pickaxe => ctx
                        .world_state
                        .player
                        .tool_pickaxe
                        .as_ref()
                        .map(|p| p.get_texture(&ctx.textures)),
                    ToolType::Dash => ctx
                        .world_state
                        .player
                        .tool_dash
                        .as_ref()
                        .map(|d| d.get_texture(&ctx.textures)),
                    ToolType::Glider => ctx
                        .world_state
                        .player
                        .tool_glider
                        .as_ref()
                        .map(|g| g.get_texture(&ctx.textures)),
                    ToolType::TideClock => ctx
                        .world_state
                        .player
                        .tool_tideclock
                        .as_ref()
                        .map(|_| &ctx.textures.items.tidalcave_clock),
                    _ => tool.get_texture(&ctx.textures),
                };
                if let Some(texture) = texture {
                    d.draw_texture_ex(
                        texture,
                        Vector2::new(right_hand_x, right_hand_y),
                        0.0,
                        1.0,
                        Color::WHITE,
                    );
                }
            }

            // Draw tool selection
            let tool_y = 190;
            let slot_outline = &ctx.textures.ui.inventory_outline;

            let mut draw_tool_slot =
                |col: usize, tool_texture: Option<&Texture2D>, tool_type: Option<ToolType>| {
                    let tool_x = slot_padding * 2.0 + (col as f32 * (slot_size + slot_padding));

                    // Check if this tool is selected
                    let is_selected = (ctx.world_state.player.right_hand == tool_type
                        && tool_type.is_some())
                        || (ctx.world_state.player.left_hand == tool_type && tool_type.is_some());

                    // Draw slot background with highlight if selected
                    let slot_color = if is_selected {
                        Color::new(255, 255, 200, 255) // Light yellow tint
                    } else {
                        Color::WHITE
                    };
                    d.draw_texture(slot_texture, tool_x as i32, tool_y as i32, slot_color);

                    // Draw tool icon if present
                    if let Some(texture) = tool_texture {
                        d.draw_texture_ex(
                            texture,
                            Vector2::new(tool_x, tool_y as f32),
                            0.0,
                            1.0,
                            Color::WHITE,
                        );
                    }

                    // Draw selection border if selected
                    if is_selected {
                        d.draw_texture(slot_outline, tool_x as i32, tool_y as i32, Color::WHITE);
                    }
                };

            draw_tool_slot(
                0,
                ctx.world_state
                    .player
                    .tool_dash
                    .as_ref()
                    .map(|d| d.get_texture(&ctx.textures)),
                ctx.world_state
                    .player
                    .tool_dash
                    .as_ref()
                    .map(|_| ToolType::Dash),
            );
            draw_tool_slot(
                1,
                ctx.world_state
                    .player
                    .tool_pickaxe
                    .as_ref()
                    .map(|p| p.get_texture(&ctx.textures)),
                ctx.world_state
                    .player
                    .tool_pickaxe
                    .as_ref()
                    .map(|_| ToolType::Pickaxe),
            );
            draw_tool_slot(2, None, None); // Grappling hook
            draw_tool_slot(3, None, None); // Spear
            draw_tool_slot(
                4,
                Some(&ctx.textures.tools.lamp_coal1),
                Some(ToolType::Lamp),
            ); // Lamp
            draw_tool_slot(5, None, None); // Fishing rod
            draw_tool_slot(
                6,
                ctx.world_state
                    .player
                    .tool_glider
                    .as_ref()
                    .map(|g| g.get_texture(&ctx.textures)),
                ctx.world_state
                    .player
                    .tool_glider
                    .as_ref()
                    .map(|_| ToolType::Glider),
            ); // Glider
            draw_tool_slot(
                7,
                ctx.world_state
                    .player
                    .tool_tideclock
                    .as_ref()
                    .map(|_| &ctx.textures.items.tidalcave_clock),
                ctx.world_state
                    .player
                    .tool_tideclock
                    .as_ref()
                    .map(|_| ToolType::TideClock),
            ); // TideClock

            // Draw mute button
            let mute_color = if ctx.music.is_muted() {
                Color::new(255, 100, 100, 255) // Red when muted
            } else {
                Color::new(100, 255, 100, 255) // Green when not muted
            };

            d.draw_texture_pro(
                &ctx.textures.ui.music,
                Rectangle::new(
                    0.0,
                    0.0,
                    ctx.textures.ui.music.width as f32,
                    ctx.textures.ui.music.height as f32,
                ),
                self.mute_button,
                Vector2::zero(),
                0.0,
                mute_color,
            );

            // Check if mouse is hovering over mute button
            let mouse_pos = ctx.controller.mouse_position;
            let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;
            let mouse_y = mouse_pos.y * RENDER_HEIGHT as f32;
            if mouse_x >= self.mute_button.x
                && mouse_x <= self.mute_button.x + self.mute_button.width
                && mouse_y >= self.mute_button.y
                && mouse_y <= self.mute_button.y + self.mute_button.height
            {
                d.draw_rectangle_lines_ex(self.mute_button, 2.0, Color::BLACK);
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
