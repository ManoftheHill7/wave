use raylib::prelude::*;

pub struct Controller {
    pub dash_pressed: bool,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub dash_held: bool,
    pub climb_pressed: bool,
    pub input_dir: Vector2,
    pub raycast_direction: Vector2,
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
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        self.dash_pressed = rl.is_key_pressed(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_pressed(KeyboardKey::KEY_X);
        self.dash_held = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_X);
        self.jump_pressed = rl.is_key_pressed(KeyboardKey::KEY_SPACE) || rl.is_key_pressed(KeyboardKey::KEY_Z);
        self.jump_held = rl.is_key_down(KeyboardKey::KEY_SPACE) || rl.is_key_down(KeyboardKey::KEY_Z);
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
    }

    pub fn set_raycast_direction(&mut self, direction: Vector2) {
        self.raycast_direction = direction;
    }
}
