#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    // 移動 (WASDなど)
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,

    // 視点移動 (マウスの相対移動量)
    pub mouse_dx: f32,
    pub mouse_dy: f32,
}

impl InputState {
    pub fn reset_relative_state(&mut self) {
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
    }
}