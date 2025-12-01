use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

pub struct HelpScreen {
    back_button: Rectangle,
    hovered: bool,
}

impl HelpScreen {
    pub fn new() -> Self {
        let button_width = 200.0;
        let button_height = 50.0;
        let margin = 20.0;

        HelpScreen {
            back_button: Rectangle::new(
                margin,
                900.0 - button_height - margin,
                button_width,
                button_height,
            ),
            hovered: false,
        }
    }
}

impl Screen for HelpScreen {
    type Context = GameContext;

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        ctx.music.update_streams();

        let mouse_pos = Vector2::new(
            ctx.controller.mouse_position.x * 1600.0,
            ctx.controller.mouse_position.y * 900.0,
        );

        self.hovered = self.back_button.check_collision_point_rec(mouse_pos);

        if ctx.controller.left_hand_pressed && self.hovered {
            return ScreenCommand::Pop;
        }

        // Also allow escape/menu to close
        if ctx.controller.menu_pressed {
            return ScreenCommand::Pop;
        }

        ScreenCommand::None
    }

    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, ctx: &Self::Context) {
        let mut d = rl.begin_drawing(thread);

        // Stone background
        let stone_background = &ctx.textures.tiles.stone;
        let stone_scale = 16.0;
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

        // Title
        let title = "CONTROLS";
        let title_size = 80;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            (1600 - title_width) / 2,
            50,
            title_size,
            Color::new(91, 110, 225, 255),
        );

        // Controls text - two columns
        let controls_left = [
            ("Movement", ""),
            ("Walk", "WASD  or  Arrow Keys"),
            ("", ""),
            ("Actions", ""),
            ("Left Hand", "Left Click (hold)"),
            ("Right Hand", "Right Click"),
        ];

        let controls_right = [
            ("Menus", ""),
            ("Inventory", "Tab"),
            ("Crafting", "M"),
            ("", ""),
            ("Special", ""),
            ("Dash", "Space (with dash amulet)"),
            ("Glide", "Hold Space (with glider)"),
            ("", ""),
            ("", ""),
        ];

        let text_size = 28;
        let line_height = 38;
        let start_y = 160;
        let left_x = 150;
        let right_x = 850;
        let label_color = Color::new(91, 110, 225, 255);

        // Draw left column
        for (i, (label, value)) in controls_left.iter().enumerate() {
            let y = start_y + (i as i32 * line_height);
            if value.is_empty() && !label.is_empty() {
                // Section header
                d.draw_text(label, left_x, y, text_size + 4, label_color);
            } else if !label.is_empty() {
                // Key-value pair
                d.draw_text(label, left_x, y, text_size, Color::WHITE);
                d.draw_text(value, left_x + 200, y, text_size, Color::LIGHTGRAY);
            }
        }

        // Draw right column
        for (i, (label, value)) in controls_right.iter().enumerate() {
            let y = start_y + (i as i32 * line_height);
            if value.is_empty() && !label.is_empty() {
                // Section header
                d.draw_text(label, right_x, y, text_size + 4, label_color);
            } else if !label.is_empty() {
                // Key-value pair
                d.draw_text(label, right_x, y, text_size, Color::WHITE);
                d.draw_text(value, right_x + 200, y, text_size, Color::LIGHTGRAY);
            }
        }

        // Tips section at bottom
        let tips = [
            "Tip: Mine ores and craft better pickaxes to dig deeper!",
            "Tip: Watch your breath meter when underwater.",
        ];
        let tip_y = 680;
        for (i, tip) in tips.iter().enumerate() {
            let tip_width = d.measure_text(tip, 24);
            d.draw_text(
                tip,
                (1600 - tip_width) / 2,
                tip_y + (i as i32 * 30),
                24,
                Color::YELLOW,
            );
        }

        // Back button
        let button_color = if self.hovered {
            Color::GRAY
        } else {
            Color::LIGHTGRAY
        };
        d.draw_rectangle_rec(self.back_button, button_color);
        d.draw_rectangle_lines_ex(self.back_button, 4.0, Color::BLACK);

        let back_text = "Back";
        let back_width = d.measure_text(back_text, 30);
        d.draw_text(
            back_text,
            self.back_button.x as i32 + ((self.back_button.width as i32 - back_width) / 2),
            self.back_button.y as i32 + 10,
            30,
            Color::BLACK,
        );
    }
}
