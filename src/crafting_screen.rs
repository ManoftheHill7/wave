use crate::inventory::ItemType;
use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

const RENDER_WIDTH: u32 = 400;
const RENDER_HEIGHT: u32 = 225;

// Include generated recipes
include!(concat!(env!("OUT_DIR"), "/generated_recipes.rs"));

pub struct CraftingScreen {
    render_target: Option<RenderTexture2D>,
    scroll_offset: f32,
    show_craftable_only: bool,
    hovered_recipe_index: Option<usize>,
}

impl CraftingScreen {
    pub fn new() -> Self {
        CraftingScreen {
            render_target: None,
            scroll_offset: 0.0,
            show_craftable_only: false,
            hovered_recipe_index: None,
        }
    }

    fn get_filtered_recipes(&self, ctx: &GameContext) -> Vec<&'static Recipe> {
        if self.show_craftable_only {
            ALL_RECIPES
                .iter()
                .filter(|recipe| recipe.can_craft(&ctx.world_state.player.inventory))
                .collect()
        } else {
            ALL_RECIPES.iter().collect()
        }
    }

    fn handle_mouse_click(&mut self, ctx: &mut GameContext) {
        let mouse_pos = ctx.controller.mouse_position;
        let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;
        let mouse_y = mouse_pos.y * RENDER_HEIGHT as f32;

        // Check toggle button (top-right area)
        let toggle_x = 260.0;
        let toggle_y = 10.0;
        let toggle_w = 130.0;
        let toggle_h = 20.0;

        if mouse_x >= toggle_x
            && mouse_x <= toggle_x + toggle_w
            && mouse_y >= toggle_y
            && mouse_y <= toggle_y + toggle_h
        {
            self.show_craftable_only = !self.show_craftable_only;
            self.scroll_offset = 0.0;
            return;
        }

        // Check recipe list clicks
        let list_x = 10.0;
        let list_y = 40.0;
        let list_w = 230.0;
        let item_h = 28.0;

        let filtered_recipes = self.get_filtered_recipes(ctx);

        for (i, recipe) in filtered_recipes.iter().enumerate() {
            let item_y = list_y + (i as f32 * item_h) - self.scroll_offset;

            // Skip if outside visible area
            if item_y < list_y || item_y + item_h > RENDER_HEIGHT as f32 - 10.0 {
                continue;
            }

            if mouse_x >= list_x
                && mouse_x <= list_x + list_w
                && mouse_y >= item_y
                && mouse_y <= item_y + item_h
            {
                // Craft this recipe
                recipe.craft(&mut ctx.world_state.player.inventory);
                return;
            }
        }
    }

    fn handle_scroll(&mut self, ctx: &GameContext) {
        let filtered_recipes = self.get_filtered_recipes(ctx);
        let item_h = 28.0;
        let visible_h = RENDER_HEIGHT as f32 - 50.0;
        let total_h = filtered_recipes.len() as f32 * item_h;
        let max_scroll = (total_h - visible_h).max(0.0);

        // TODO: Get actual mouse wheel input from controller
        // For now, this is a placeholder
        // self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
    }
}

impl Screen for CraftingScreen {
    type Context = GameContext;

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        // ESC or M to close
        if ctx.controller.crafting_pressed {
            return ScreenCommand::Pop;
        }

        // Handle mouse clicks
        if ctx.controller.left_hand_pressed {
            self.handle_mouse_click(ctx);
        }

        // Update scroll offset
        self.handle_scroll(ctx);

        // Update hovered recipe
        let mouse_pos = ctx.controller.mouse_position;
        let mouse_x = mouse_pos.x * RENDER_WIDTH as f32;
        let mouse_y = mouse_pos.y * RENDER_HEIGHT as f32;

        let list_x = 10.0;
        let list_y = 40.0;
        let list_w = 230.0;
        let item_h = 28.0;

        self.hovered_recipe_index = None;
        let filtered_recipes = self.get_filtered_recipes(ctx);

        for (i, _recipe) in filtered_recipes.iter().enumerate() {
            let item_y = list_y + (i as f32 * item_h) - self.scroll_offset;

            if item_y < list_y || item_y + item_h > RENDER_HEIGHT as f32 - 10.0 {
                continue;
            }

            if mouse_x >= list_x
                && mouse_x <= list_x + list_w
                && mouse_y >= item_y
                && mouse_y <= item_y + item_h
            {
                self.hovered_recipe_index = Some(i);
                break;
            }
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

        // Get filtered recipes before borrowing render_target mutably
        let filtered_recipes = self.get_filtered_recipes(ctx);

        let render_target = self.render_target.as_mut().unwrap();

        // Draw to render texture
        {
            let mut d = rl.begin_texture_mode(thread, render_target);
        d.clear_background(Color::new(40, 40, 50, 255));

        // Draw title
        d.draw_text("Crafting", 10, 10, 20, Color::WHITE);

        // Draw toggle button
        let toggle_x = 260;
        let toggle_y = 10;
        let toggle_w = 130;
        let toggle_h = 20;
        let toggle_color = if self.show_craftable_only {
            Color::GREEN
        } else {
            Color::GRAY
        };
        d.draw_rectangle(toggle_x, toggle_y, toggle_w, toggle_h, toggle_color);
        d.draw_rectangle_lines(toggle_x, toggle_y, toggle_w, toggle_h, Color::WHITE);
        d.draw_text("Show Craftable", toggle_x + 5, toggle_y + 3, 14, Color::WHITE);

        // Draw recipe list
        let list_x = 10.0;
        let list_y = 40.0;
        let list_w = 230.0;
        let item_h = 28.0;

        for (i, recipe) in filtered_recipes.iter().enumerate() {
            let item_y = list_y + (i as f32 * item_h) - self.scroll_offset;

            // Skip if outside visible area
            if item_y < list_y || item_y + item_h > RENDER_HEIGHT as f32 - 10.0 {
                continue;
            }

            let can_craft = recipe.can_craft(&ctx.world_state.player.inventory);
            let is_hovered = self.hovered_recipe_index == Some(i);

            // Background
            let bg_color = if is_hovered {
                Color::new(80, 80, 100, 255)
            } else if can_craft {
                Color::new(50, 60, 70, 255)
            } else {
                Color::new(40, 40, 50, 255)
            };
            d.draw_rectangle(
                list_x as i32,
                item_y as i32,
                list_w as i32,
                item_h as i32,
                bg_color,
            );
            d.draw_rectangle_lines(
                list_x as i32,
                item_y as i32,
                list_w as i32,
                item_h as i32,
                Color::DARKGRAY,
            );

            // Draw item icon
            let icon_x = list_x + 4.0;
            let icon_y = item_y + 2.0;
            let icon_size = 24.0;

            let texture = recipe.output.get_texture(&ctx.textures);
            let scale = icon_size / texture.width as f32;
            d.draw_texture_ex(
                texture,
                Vector2::new(icon_x, icon_y),
                0.0,
                scale,
                if can_craft {
                    Color::WHITE
                } else {
                    Color::GRAY
                },
            );

            // Draw recipe name and output amount
            let text_x = icon_x + icon_size + 4.0;
            let text_y = item_y + 6.0;
            let text_color = if can_craft {
                Color::WHITE
            } else {
                Color::DARKGRAY
            };
            d.draw_text(
                &format!("{}x {}", recipe.output_amount, recipe.output.name()),
                text_x as i32,
                text_y as i32,
                12,
                text_color,
            );
        }

        // Draw recipe details panel (right side)
        if let Some(hovered_idx) = self.hovered_recipe_index {
            if let Some(recipe) = filtered_recipes.get(hovered_idx) {
                let panel_x = 250.0;
                let panel_y = 40.0;
                let panel_w = 140.0;
                let panel_h = 175.0;

                d.draw_rectangle(
                    panel_x as i32,
                    panel_y as i32,
                    panel_w as i32,
                    panel_h as i32,
                    Color::new(50, 50, 60, 255),
                );
                d.draw_rectangle_lines(
                    panel_x as i32,
                    panel_y as i32,
                    panel_w as i32,
                    panel_h as i32,
                    Color::WHITE,
                );

                // Recipe name
                d.draw_text(recipe.name, panel_x as i32 + 5, panel_y as i32 + 5, 14, Color::WHITE);

                // Output
                let output_y = panel_y + 30.0;
                d.draw_text("Output:", panel_x as i32 + 5, output_y as i32, 12, Color::LIGHTGRAY);
                let output_count = ctx.world_state.player.inventory.count(recipe.output);
                d.draw_text(
                    &format!("{}x {} ({})", recipe.output_amount, recipe.output.name(), output_count),
                    panel_x as i32 + 5,
                    output_y as i32 + 15,
                    10,
                    Color::WHITE,
                );

                // Ingredients
                let ingredients_y = output_y + 35.0;
                d.draw_text(
                    "Ingredients:",
                    panel_x as i32 + 5,
                    ingredients_y as i32,
                    12,
                    Color::LIGHTGRAY,
                );

                for (i, input) in recipe.inputs.iter().enumerate() {
                    let ing_y = ingredients_y + 15.0 + (i as f32 * 15.0);
                    let current_count = ctx.world_state.player.inventory.count(input.item_type);
                    let has_enough = current_count >= input.amount;
                    let text_color = if has_enough {
                        Color::GREEN
                    } else {
                        Color::RED
                    };
                    d.draw_text(
                        &format!("{}x {} ({})", input.amount, input.item_type.name(), current_count),
                        panel_x as i32 + 5,
                        ing_y as i32,
                        10,
                        text_color,
                    );
                }

                // Craft button
                let can_craft = recipe.can_craft(&ctx.world_state.player.inventory);
                let button_y = panel_y + panel_h - 30.0;
                let button_color = if can_craft {
                    Color::GREEN
                } else {
                    Color::DARKGRAY
                };
                d.draw_rectangle(
                    panel_x as i32 + 10,
                    button_y as i32,
                    120,
                    20,
                    button_color,
                );
                d.draw_text(
                    "Click to Craft",
                    panel_x as i32 + 20,
                    button_y as i32 + 3,
                    12,
                    Color::WHITE,
                );
            }
        }

        }

        // Draw render texture to screen
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        let scale_x = d.get_screen_width() as f32 / RENDER_WIDTH as f32;
        let scale_y = d.get_screen_height() as f32 / RENDER_HEIGHT as f32;
        let scale = scale_x.min(scale_y);

        let offset_x = (d.get_screen_width() as f32 - RENDER_WIDTH as f32 * scale) / 2.0;
        let offset_y = (d.get_screen_height() as f32 - RENDER_HEIGHT as f32 * scale) / 2.0;

        d.draw_texture_pro(
            render_target,
            Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, -(RENDER_HEIGHT as f32)),
            Rectangle::new(offset_x, offset_y, RENDER_WIDTH as f32 * scale, RENDER_HEIGHT as f32 * scale),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}
