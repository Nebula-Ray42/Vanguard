

pub struct InputState {
    // 移動 (WASDなど)
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,

    // 視点移動 (マウスの相対移動量)
    pub mouse_dx: f32,
    pub mouse_dy: f32,
}