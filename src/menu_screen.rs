use crate::save_load;
use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

pub struct MenuScreen {
    new_game_button: Rectangle,
    load_game_button: Rectangle,
    mute_button: Rectangle,
    hovered_button: Option<ButtonType>,
}

#[derive(PartialEq)]
enum ButtonType {
    NewGame,
    LoadGame,
    Mute,
}

impl MenuScreen {
    pub fn new() -> Self {
        let screen_width = 1600.0;
        let screen_height = 900.0;

        let button_width = 300.0;
        let button_height = 60.0;
        let button_x = (screen_width - button_width) / 2.0;

        // Mute button in top-right corner
        let mute_size = 48.0;
        let mute_margin = 20.0;

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
            mute_button: Rectangle::new(
                screen_width - mute_size - mute_margin,
                mute_margin,
                mute_size,
                mute_size,
            ),
            hovered_button: None,
        }
    }
}

impl Screen for MenuScreen {
    type Context = GameContext;

    fn on_resume(&mut self, ctx: &mut Self::Context) {
        // Start menu music when entering menu
        ctx.music.play_static("menu");
    }

    fn update(&mut self, _dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
        // Update music streams
        ctx.music.update_streams();
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
        } else if self.mute_button.check_collision_point_rec(mouse_pos) {
            self.hovered_button = Some(ButtonType::Mute);
        }

        // Check button clicks
        if ctx.controller.left_hand_pressed {
            if self.hovered_button == Some(ButtonType::NewGame) {
                // Save immediately so chunks can be saved/loaded from files
                match save_load::save_game(&mut ctx.world_state, 0) {
                    Ok(()) => println!("✓ New game saved!"),
                    Err(e) => eprintln!("✗ Failed to save new game: {}", e),
                }
                return ScreenCommand::Push(Box::new(crate::GameScreen::new(ctx)));
            } else if self.hovered_button == Some(ButtonType::LoadGame) {
                if save_load::save_exists(0) {
                    match save_load::load_game(0) {
                        Ok(save_data) => {
                            save_load::apply_save_data(&mut ctx.world_state, save_data, 0);
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
            } else if self.hovered_button == Some(ButtonType::Mute) {
                ctx.music.toggle_mute();
                ctx.sounds.set_muted(ctx.music.is_muted());
            }
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
            Vector2::new(stone_grid_x,stone_grid_y),
            0.0,
            stone_scale, 
            Color::GRAY
             );
            }
        }

        // Title
        let title = "TIDALCAVE";
        let title_size = 160;
        let title_width = d.measure_text(title, title_size);
         d.draw_text(
            title,
            (1600 - title_width + 16) / 2,
            192,
            title_size,
            Color::new(0, 0, 0, 127),
        );
        d.draw_text(
            title,
            (1600 - title_width) / 2,
            200,
            title_size,
            Color::new(91, 110, 225, 255),
        );
       

        // New Game button
        let new_game_color = if self.hovered_button == Some(ButtonType::NewGame) {
            Color::GRAY
        } else {
            Color::LIGHTGRAY
        };
        d.draw_rectangle_rec(self.new_game_button, new_game_color);
        d.draw_rectangle_lines_ex(self.new_game_button, 4.0, Color::BLACK);

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
        d.draw_rectangle_lines_ex(self.load_game_button, 4.0, Color::BLACK);

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

        // Mute button
        let mute_color = if ctx.music.is_muted() {
            Color::new(255, 100, 100, 255) // Red when muted
        } else {
            Color::new(100, 255, 100, 255) // Green when not muted
        };

        // Draw the music icon with tint
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

        // Draw border if hovered
        if self.hovered_button == Some(ButtonType::Mute) {
            d.draw_rectangle_lines_ex(self.mute_button, 3.0, Color::BLACK);
        }
    }
}
