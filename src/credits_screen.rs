use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

pub struct CreditsScreen {
    back_button: Rectangle,
    hovered: bool,
}

impl CreditsScreen {
    pub fn new() -> Self {
        let button_width = 200.0;
        let button_height = 50.0;
        let margin = 20.0;

        CreditsScreen {
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

impl Screen for CreditsScreen {
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
        let title = "CREDITS";
        let title_size = 80;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            (1600 - title_width) / 2,
            100,
            title_size,
            Color::new(91, 110, 225, 255),
        );

        // Credits text
        let credits = ["Jacob Reckhard", "ManoftheHill7", "And music by Trollslayer"];
        let text_size = 40;
        let line_height = 60;
        let start_y = 300;

        for (i, credit) in credits.iter().enumerate() {
            let text_width = d.measure_text(credit, text_size);
            d.draw_text(
                credit,
                (1600 - text_width) / 2,
                start_y + (i as i32 * line_height),
                text_size,
                Color::WHITE,
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
