use crate::save_load;
use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

pub struct MenuScreen {
    new_game_button: Rectangle,
    load_game_button: Rectangle,
    hovered_button: Option<ButtonType>,
}

#[derive(PartialEq)]
enum ButtonType {
    NewGame,
    LoadGame,
}

impl MenuScreen {
    pub fn new() -> Self {
        let screen_width = 1600.0;
        let screen_height = 900.0;

        let button_width = 300.0;
        let button_height = 60.0;
        let button_x = (screen_width - button_width) / 2.0;

        MenuScreen {
            new_game_button: Rectangle::new(
                button_x,
                screen_height / 2.0 - 40.0,
                button_width,
                button_height,
            ),
            load_game_button: Rectangle::new(
                button_x,
                screen_height / 2.0 + 40.0,
                button_width,
                button_height,
            ),
            hovered_button: None,
        }
    }
}

impl Screen for MenuScreen {
    type Context = GameContext;

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        // Convert normalized mouse position (0.0-1.0) to pixel coordinates
        let mouse_pos = Vector2::new(
            ctx.controller.mouse_position.x * 1600.0,
            ctx.controller.mouse_position.y * 900.0,
        );

        // Check button hovers
        self.hovered_button = None;
        if self.new_game_button.check_collision_point_rec(mouse_pos) {
            self.hovered_button = Some(ButtonType::NewGame);
        } else if self.load_game_button.check_collision_point_rec(mouse_pos) {
            self.hovered_button = Some(ButtonType::LoadGame);
        }

        // Check button clicks
        if ctx.controller.left_hand_pressed {
            if self.hovered_button == Some(ButtonType::NewGame) {
                return ScreenCommand::Push(Box::new(crate::GameScreen::new(ctx)));
            } else if self.hovered_button == Some(ButtonType::LoadGame) {
                if save_load::save_exists(0) {
                    match save_load::load_game(0) {
                        Ok(save_data) => {
                            save_load::apply_save_data(&mut ctx.world_state, save_data);
                            println!("✓ Game loaded successfully!");
                            return ScreenCommand::Push(Box::new(crate::GameScreen::new(ctx)));
                        }
                        Err(e) => {
                            eprintln!("✗ Failed to load game: {}", e);
                        }
                    }
                } else {
                    println!("✗ No save file found!");
                }
            }
        }

        ScreenCommand::None
    }

    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, _ctx: &Self::Context) {
        let mut d = rl.begin_drawing(thread);

        // White background
        d.clear_background(Color::WHITE);

        // Title
        let title = "WAVES";
        let title_size = 80;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            (1600 - title_width) / 2,
            200,
            title_size,
            Color::BLACK,
        );

        // New Game button
        let new_game_color = if self.hovered_button == Some(ButtonType::NewGame) {
            Color::GRAY
        } else {
            Color::LIGHTGRAY
        };
        d.draw_rectangle_rec(self.new_game_button, new_game_color);
        d.draw_rectangle_lines_ex(self.new_game_button, 2.0, Color::BLACK);

        let new_game_text = "New Game";
        let new_game_width = d.measure_text(new_game_text, 30);
        d.draw_text(
            new_game_text,
            self.new_game_button.x as i32
                + ((self.new_game_button.width as i32 - new_game_width) / 2),
            self.new_game_button.y as i32 + 15,
            30,
            Color::BLACK,
        );

        // Load Game button
        let load_game_enabled = save_load::save_exists(0);
        let load_game_color = if !load_game_enabled {
            Color::new(200, 200, 200, 255)
        } else if self.hovered_button == Some(ButtonType::LoadGame) {
            Color::GRAY
        } else {
            Color::LIGHTGRAY
        };
        d.draw_rectangle_rec(self.load_game_button, load_game_color);
        d.draw_rectangle_lines_ex(self.load_game_button, 2.0, Color::BLACK);

        let load_game_text = if load_game_enabled {
            "Load Game"
        } else {
            "(No Saves found)"
        };
        let load_game_width = d.measure_text(load_game_text, 30);
        let text_color = if load_game_enabled {
            Color::BLACK
        } else {
            Color::new(100, 100, 100, 255)
        };
        d.draw_text(
            load_game_text,
            self.load_game_button.x as i32
                + ((self.load_game_button.width as i32 - load_game_width) / 2),
            self.load_game_button.y as i32 + 15,
            30,
            text_color,
        );
    }
}
