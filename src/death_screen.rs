use crate::player::STARTING_HEALTH;
use crate::GameContext;
use raylib::prelude::*;
use screen_manager::{Screen, ScreenCommand};

pub struct DeathScreen {
    continue_button: Rectangle,
    exit_button: Rectangle,
    hovered_button: Option<ButtonType>,
}

#[derive(PartialEq, Clone, Copy)]
enum ButtonType {
    Continue,
    Exit,
}

/// Result from death screen
pub enum DeathScreenResult {
    Respawn,
    ExitToMenu,
}

impl DeathScreen {
    pub fn new() -> Self {
        let screen_width = 1600.0;
        let screen_height = 900.0;

        let button_width = 300.0;
        let button_height = 60.0;
        let button_x = (screen_width - button_width) / 2.0;

        DeathScreen {
            continue_button: Rectangle::new(
                button_x,
                screen_height / 2.0 + 20.0,
                button_width,
                button_height,
            ),
            exit_button: Rectangle::new(
                button_x,
                screen_height / 2.0 + 100.0,
                button_width,
                button_height,
            ),
            hovered_button: None,
        }
    }

    /// Reset player for respawn: clear non-tool items, reset health, reset position
    pub fn respawn_player(ctx: &mut GameContext) {
        // Reset health to full
        ctx.world_state.player.health = STARTING_HEALTH;

        // Reset breathing to full
        ctx.world_state.player.breath = 100.0;

        // Reset position to starting location
        ctx.world_state.player.position.x = 32.0;
        ctx.world_state.player.position.y = -3.0;

        // Reset velocity
        ctx.world_state.player.velocity.x = 0.0;
        ctx.world_state.player.velocity.y = 0.0;

        // Clear non-tool items from inventory
        ctx.world_state.player.inventory.clear();

        // Unequip hands
        ctx.world_state.player.left_hand = None;
        ctx.world_state.player.right_hand = None;

        // Reset various player states
        ctx.world_state.player.is_mining = false;
        ctx.world_state.player.is_swimming = false;
        ctx.world_state.player.is_climbing = false;
        ctx.world_state.player.is_dashing = false;
        ctx.world_state.player.is_gliding = false;
    }
}

impl Screen for DeathScreen {
    type Context = GameContext;

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
        if self.continue_button.check_collision_point_rec(mouse_pos) {
            self.hovered_button = Some(ButtonType::Continue);
        } else if self.exit_button.check_collision_point_rec(mouse_pos) {
            self.hovered_button = Some(ButtonType::Exit);
        }

        // Check button clicks
        if ctx.controller.left_hand_pressed {
            if self.hovered_button == Some(ButtonType::Continue) {
                // Respawn player and pop back to game
                Self::respawn_player(ctx);
                return ScreenCommand::Pop;
            } else if self.hovered_button == Some(ButtonType::Exit) {
                // Reset to menu screen (clears all screens and starts fresh)
                return ScreenCommand::Reset(Box::new(crate::menu_screen::MenuScreen::new()));
            }
        }

        ScreenCommand::None
    }

    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, _ctx: &Self::Context) {
        let mut d = rl.begin_drawing(thread);

        // Dark background with some transparency effect
        d.clear_background(Color::new(20, 20, 30, 255));

        // "You Died" text
        let title = "YOU DIED";
        let title_size = 80;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            (1600 - title_width) / 2,
            250,
            title_size,
            Color::new(200, 50, 50, 255), // Dark red
        );

        // Subtitle
        let subtitle = "Your items have been lost...";
        let subtitle_size = 30;
        let subtitle_width = d.measure_text(subtitle, subtitle_size);
        d.draw_text(
            subtitle,
            (1600 - subtitle_width) / 2,
            350,
            subtitle_size,
            Color::GRAY,
        );

        // Continue button
        let continue_color = if self.hovered_button == Some(ButtonType::Continue) {
            Color::new(80, 80, 80, 255)
        } else {
            Color::new(60, 60, 60, 255)
        };
        d.draw_rectangle_rec(self.continue_button, continue_color);
        d.draw_rectangle_lines_ex(self.continue_button, 2.0, Color::WHITE);

        let continue_text = "Continue";
        let continue_width = d.measure_text(continue_text, 30);
        d.draw_text(
            continue_text,
            self.continue_button.x as i32
                + ((self.continue_button.width as i32 - continue_width) / 2),
            self.continue_button.y as i32 + 15,
            30,
            Color::WHITE,
        );

        // Exit button
        let exit_color = if self.hovered_button == Some(ButtonType::Exit) {
            Color::new(80, 80, 80, 255)
        } else {
            Color::new(60, 60, 60, 255)
        };
        d.draw_rectangle_rec(self.exit_button, exit_color);
        d.draw_rectangle_lines_ex(self.exit_button, 2.0, Color::WHITE);

        let exit_text = "Exit to Menu";
        let exit_width = d.measure_text(exit_text, 30);
        d.draw_text(
            exit_text,
            self.exit_button.x as i32 + ((self.exit_button.width as i32 - exit_width) / 2),
            self.exit_button.y as i32 + 15,
            30,
            Color::WHITE,
        );
    }
}
