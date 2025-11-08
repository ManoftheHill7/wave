use raylib::prelude::*;

pub struct Controller {
    pub dash_pressed: bool,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub dash_held: bool,
    pub climb_pressed: bool,
    pub input_dir: Vector2,
    pub raycast_direction: Vector2,
    pub menu_pressed: bool,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            dash_pressed: false,
            dash_held: false,
            jump_pressed: false,
            jump_held: false,
            climb_pressed: false,
            input_dir: Vector2::zero(),
            raycast_direction: Vector2::zero(),
            menu_pressed: false,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        self.dash_pressed =
            rl.is_key_pressed(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_pressed(KeyboardKey::KEY_X);
        self.dash_held =
            rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_X);
        self.jump_pressed =
            rl.is_key_pressed(KeyboardKey::KEY_SPACE) || rl.is_key_pressed(KeyboardKey::KEY_Z);
        self.jump_held =
            rl.is_key_down(KeyboardKey::KEY_SPACE) || rl.is_key_down(KeyboardKey::KEY_Z);
        self.climb_pressed = rl.is_key_down(KeyboardKey::KEY_C);

        self.input_dir = Vector2::new(0.0, 0.0);
        if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
            self.input_dir.y -= 1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
            self.input_dir.y += 1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
            self.input_dir.x = -1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.input_dir.x = 1.0;
        }

        // Calculate raycast direction from screen center to mouse
        let mouse_pos = rl.get_mouse_position();
        let screen_center_x = rl.get_screen_width() as f32 / 2.0;
        let screen_center_y = rl.get_screen_height() as f32 / 2.0;

        let dx = mouse_pos.x - screen_center_x;
        let dy = mouse_pos.y - screen_center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0001 {
            self.raycast_direction = Vector2::new(dx / distance, dy / distance);
        } else {
            // Default direction if mouse is exactly at center
            self.raycast_direction = Vector2::new(1.0, 0.0);
        }

        // Track Tab key for menu/inventory
        self.menu_pressed = rl.is_key_pressed(KeyboardKey::KEY_TAB);
    }

    pub fn set_raycast_direction(&mut self, direction: Vector2) {
        self.raycast_direction = direction;
    }
}
