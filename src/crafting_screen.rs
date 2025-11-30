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
    recipe_filter: Option<RecipeType>,
}

impl CraftingScreen {
    pub fn new(filter: Option<RecipeType>) -> Self {
        CraftingScreen {
            render_target: None,
            scroll_offset: 0.0,
            show_craftable_only: false,
            hovered_recipe_index: None,
            recipe_filter: filter,
        }
    }

    fn get_filtered_recipes(&self, ctx: &GameContext) -> Vec<&'static Recipe> {
        let mut recipes: Vec<&'static Recipe> = ALL_RECIPES
            .iter()
            .filter(|recipe| {
                // Filter by recipe type first
                let type_matches = match self.recipe_filter {
                    None => recipe.recipe_type == RecipeType::Always,
                    Some(RecipeType::Workbench) => {
                        recipe.recipe_type == RecipeType::Always
                            || recipe.recipe_type == RecipeType::Workbench
                    }
                    Some(RecipeType::Anvil) => recipe.recipe_type == RecipeType::Anvil,
                    Some(RecipeType::Furnace) => recipe.recipe_type == RecipeType::Furnace,
                    Some(RecipeType::Always) => recipe.recipe_type == RecipeType::Always,
                };

                if !type_matches {
                    return false;
                }

                // Special filtering for tool recipes
                match recipe.output {
                    RecipeIOType::ToolPickaxe {
                        level: output_level,
                    } => {
                        // Check if this is a crafting, repair, or upgrade recipe
                        let has_tool_input = recipe
                            .inputs
                            .iter()
                            .any(|input| matches!(input, RecipeIOType::ToolPickaxe { .. }));

                        if !has_tool_input {
                            // This is a crafting recipe (e.g., stone_pickaxe from scratch)
                            // Don't show if player has ANY pickaxe
                            if ctx.world_state.player.tool_pickaxe.is_some() {
                                return false;
                            }
                        } else {
                            // This is either a repair or upgrade recipe
                            // Check if it's a repair (input level == output level)
                            let is_repair = recipe.inputs.iter().any(|input| {
                                if let RecipeIOType::ToolPickaxe { level: input_level } = input {
                                    input_level == &output_level
                                } else {
                                    false
                                }
                            });

                            if is_repair {
                                // Repair recipe: only show if player has this exact level
                                let has_exact_level = ctx
                                    .world_state
                                    .player
                                    .tool_pickaxe
                                    .as_ref()
                                    .map(|p| p.level == output_level)
                                    .unwrap_or(false);
                                if !has_exact_level {
                                    return false;
                                }
                            } else {
                                // Upgrade recipe: only show if player has the required input level
                                if !recipe.can_craft(&ctx.world_state.player) {
                                    return false;
                                }
                            }
                        }
                    }
                    RecipeIOType::ToolDash {
                        level: output_level,
                    } => {
                        // Same logic for dash tools
                        let has_tool_input = recipe
                            .inputs
                            .iter()
                            .any(|input| matches!(input, RecipeIOType::ToolDash { .. }));

                        if !has_tool_input {
                            // Crafting recipe: don't show if player has ANY dash tool
                            if ctx.world_state.player.tool_dash.is_some() {
                                return false;
                            }
                        } else {
                            // Repair or upgrade recipe
                            let is_repair = recipe.inputs.iter().any(|input| {
                                if let RecipeIOType::ToolDash { level: input_level } = input {
                                    input_level == &output_level
                                } else {
                                    false
                                }
                            });

                            if is_repair {
                                // Repair: only show if player has this exact level
                                let has_exact_level = ctx
                                    .world_state
                                    .player
                                    .tool_dash
                                    .as_ref()
                                    .map(|d| d.level == output_level)
                                    .unwrap_or(false);
                                if !has_exact_level {
                                    return false;
                                }
                            } else {
                                // Upgrade: only show if player can craft it
                                if !recipe.can_craft(&ctx.world_state.player) {
                                    return false;
                                }
                            }
                        }
                    }
                    RecipeIOType::ToolGlider {
                        level: output_level,
                    } => {
                        // Same logic for glider tools
                        let has_tool_input = recipe
                            .inputs
                            .iter()
                            .any(|input| matches!(input, RecipeIOType::ToolGlider { .. }));

                        if !has_tool_input {
                            // Crafting recipe: don't show if player has ANY glider
                            if ctx.world_state.player.tool_glider.is_some() {
                                return false;
                            }
                        } else {
                            // Repair or upgrade recipe
                            let is_repair = recipe.inputs.iter().any(|input| {
                                if let RecipeIOType::ToolGlider { level: input_level } = input {
                                    input_level == &output_level
                                } else {
                                    false
                                }
                            });

                            if is_repair {
                                // Repair: only show if player has this exact level
                                let has_exact_level = ctx
                                    .world_state
                                    .player
                                    .tool_glider
                                    .as_ref()
                                    .map(|g| g.level == output_level)
                                    .unwrap_or(false);
                                if !has_exact_level {
                                    return false;
                                }
                            } else {
                                // Upgrade: only show if player can craft it
                                if !recipe.can_craft(&ctx.world_state.player) {
                                    return false;
                                }
                            }
                        }
                    }
                    RecipeIOType::ToolTideClock => {
                        // TideClock is a unique tool - you can only have one
                        // Don't show the recipe if player already has it
                        if ctx.world_state.player.tool_tideclock.is_some() {
                            return false;
                        }
                    }
                    _ => {} // Non-tool recipes: no special filtering
                }

                // Then filter by craftability if toggle is on
                if self.show_craftable_only {
                    recipe.can_craft(&ctx.world_state.player)
                } else {
                    true
                }
            })
            .collect();

        // Sort anvil recipes: repairs first, then upgrades
        if self.recipe_filter == Some(RecipeType::Anvil) {
            recipes.sort_by_key(|recipe| !recipe.is_repair());
        }

        recipes
    }

    fn get_title(&self) -> String {
        match self.recipe_filter {
            None => "Crafting".to_string(),
            Some(RecipeType::Workbench) => "Crafting - Workbench".to_string(),
            Some(RecipeType::Anvil) => "Crafting - Anvil".to_string(),
            Some(RecipeType::Furnace) => "Crafting - Furnace".to_string(),
            Some(RecipeType::Always) => "Crafting".to_string(),
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
                recipe.craft(&mut ctx.world_state.player);
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

        // Handle mouse wheel scrolling
        let scroll_speed = item_h; // Scroll one item at a time
        if ctx.controller.mouse_wheel_move > 0.0 {
            self.scroll_offset = (self.scroll_offset - scroll_speed).max(0.0);
        } else if ctx.controller.mouse_wheel_move < 0.0 {
            self.scroll_offset = (self.scroll_offset + scroll_speed).min(max_scroll);
        }
    }
}

impl Screen for CraftingScreen {
    type Context = GameContext;

    fn on_resume(&mut self, ctx: &mut Self::Context) {
        // Start crafting music when opening crafting screen
        ctx.music.play_static("crafting");
    }

    fn on_pause(&mut self, ctx: &mut Self::Context) {
        // Resume game music when closing crafting screen
        ctx.music.play_game_layers();
    }

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        // Update music streams
        ctx.music.update_streams();
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

        // Get filtered recipes
        let filtered_recipes = self.get_filtered_recipes(ctx);
        let title = self.get_title();

        let render_target = self.render_target.as_mut().unwrap();

        // Draw to render texture
        {
            let mut d = rl.begin_texture_mode(thread, render_target);
            d.clear_background(Color::new(40, 40, 50, 255));

            // Draw title
            d.draw_text(&title, 10, 10, 20, Color::WHITE);

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
            d.draw_text(
                "Show Craftable",
                toggle_x + 5,
                toggle_y + 3,
                14,
                Color::WHITE,
            );

            // Draw recipe list
            let list_x = 10.0;
            let list_y = 40.0;
            let list_w = 230.0;
            let item_h = 28.0;

            // Calculate and display scroll info if needed
            let visible_h = RENDER_HEIGHT as f32 - 50.0;
            let total_h = filtered_recipes.len() as f32 * item_h;
            let max_scroll = (total_h - visible_h).max(0.0);

            if max_scroll > 0.0 {
                // Calculate current position
                let scroll_progress = if max_scroll > 0.0 {
                    self.scroll_offset / max_scroll
                } else {
                    0.0
                };

                // Draw scroll indicator bar on the right side of the list
                let scrollbar_x = list_x + list_w + 2.0;
                let scrollbar_y = list_y;
                let scrollbar_w = 4.0;
                let scrollbar_h = visible_h;

                // Background track
                d.draw_rectangle(
                    scrollbar_x as i32,
                    scrollbar_y as i32,
                    scrollbar_w as i32,
                    scrollbar_h as i32,
                    Color::new(30, 30, 40, 255),
                );

                // Thumb
                let thumb_h = (visible_h / total_h * scrollbar_h).max(20.0);
                let thumb_y = scrollbar_y + (scroll_progress * (scrollbar_h - thumb_h));
                d.draw_rectangle(
                    scrollbar_x as i32,
                    thumb_y as i32,
                    scrollbar_w as i32,
                    thumb_h as i32,
                    Color::new(150, 150, 160, 255),
                );
            }

            for (i, recipe) in filtered_recipes.iter().enumerate() {
                let item_y = list_y + (i as f32 * item_h) - self.scroll_offset;

                // Skip if outside visible area
                if item_y < list_y || item_y + item_h > RENDER_HEIGHT as f32 - 10.0 {
                    continue;
                }

                let can_craft = recipe.can_craft(&ctx.world_state.player);
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
                    if can_craft { Color::WHITE } else { Color::GRAY },
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
                    &recipe.display_name(),
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
                    d.draw_text(
                        recipe.name,
                        panel_x as i32 + 5,
                        panel_y as i32 + 5,
                        14,
                        Color::WHITE,
                    );

                    // Output
                    let output_y = panel_y + 30.0;
                    d.draw_text(
                        "Output:",
                        panel_x as i32 + 5,
                        output_y as i32,
                        12,
                        Color::LIGHTGRAY,
                    );

                    d.draw_text(
                        &recipe.output.display_with_count(&ctx.world_state.player),
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
                        let has_enough = input.player_has(&ctx.world_state.player);
                        let text_color = if has_enough { Color::GREEN } else { Color::RED };
                        d.draw_text(
                            &input.display_with_count(&ctx.world_state.player),
                            panel_x as i32 + 5,
                            ing_y as i32,
                            10,
                            text_color,
                        );
                    }

                    // Craft button
                    let can_craft = recipe.can_craft(&ctx.world_state.player);
                    let button_y = panel_y + panel_h - 30.0;
                    let button_color = if can_craft {
                        Color::GREEN
                    } else {
                        Color::DARKGRAY
                    };
                    d.draw_rectangle(panel_x as i32 + 10, button_y as i32, 120, 20, button_color);
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
            Rectangle::new(
                offset_x,
                offset_y,
                RENDER_WIDTH as f32 * scale,
                RENDER_HEIGHT as f32 * scale,
            ),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}
