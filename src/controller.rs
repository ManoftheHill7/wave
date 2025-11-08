use raylib::prelude::*;

pub struct Controller {
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub climb_pressed: bool,
    pub input_dir: Vector2,
    pub raycast_direction: Vector2,
    pub menu_pressed: bool,
    pub use_tool_pressed: bool,
    pub place_pressed: bool,
    pub mouse_position: Vector2, // 0.0 to 1.0
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            jump_pressed: false,
            jump_held: false,
            climb_pressed: false,
            input_dir: Vector2::zero(),
            raycast_direction: Vector2::zero(),
            menu_pressed: false,
            use_tool_pressed: false,
            place_pressed: false,
            mouse_position: Vector2::zero(),
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        self.jump_pressed =
            rl.is_key_pressed(KeyboardKey::KEY_SPACE) || rl.is_key_pressed(KeyboardKey::KEY_Z);
        self.jump_held =
            rl.is_key_down(KeyboardKey::KEY_SPACE) || rl.is_key_down(KeyboardKey::KEY_Z);
        self.climb_pressed =
            rl.is_key_down(KeyboardKey::KEY_C) || rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);

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

        let mouse_pixel = rl.get_mouse_position();
        let screen_width = rl.get_screen_width() as f32;
        let screen_height = rl.get_screen_height() as f32;

        self.mouse_position =
            Vector2::new(mouse_pixel.x / screen_width, mouse_pixel.y / screen_height);

        let screen_center_x = screen_width / 2.0;
        let screen_center_y = screen_height / 2.0;

        let dx = mouse_pixel.x - screen_center_x;
        let dy = mouse_pixel.y - screen_center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0001 {
            self.raycast_direction = Vector2::new(dx / distance, dy / distance);
        } else {
            self.raycast_direction = Vector2::new(1.0, 0.0);
        }

        self.menu_pressed = rl.is_key_pressed(KeyboardKey::KEY_TAB);

        self.use_tool_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        self.place_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT);
    }

    pub fn set_raycast_direction(&mut self, direction: Vector2) {
        self.raycast_direction = direction;
    }
}
