// crates/core/src/camera.rs
use crate::input::InputState;
use nalgebra::{Matrix4, Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Point3<f32>, // 現在地
    pub pitch: f32,            // 上下の首振り角度 (ラジアン)
    pub yaw: f32,              // 左右の首振り角度 (ラジアン)
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::new(0.0, 3.0, 5.0),
            yaw: -std::f32::consts::FRAC_PI_2, // -90度 (初期状態で原点を向く)
            pitch: -0.5,                       // 少し下を向く
        }
    }

    /// 入力状態と経過時間(dt)を受け取り、自身の状態を更新する（純粋な関数的アプローチ）
    pub fn update(&mut self, input: &InputState, delta_time: f32) {
        // --- 1. 視点移動 (マウス) ---
        let mouse_sensitivity = 0.005;
        self.yaw -= input.mouse_dx * mouse_sensitivity;
        self.pitch -= input.mouse_dy * mouse_sensitivity;

        // ジンバルロック対策: 真上・真下を向くと計算が破綻するため、±89度付近で制限する
        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        // --- 2. 向きベクトルの計算 ---
        // PitchとYawから、カメラが現在向いている方向(フロントベクトル)を計算
        let front = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();

        // フロントベクトルと「真上」のベクトルから、カメラの右方向(ライトベクトル)を計算
        let right = front.cross(&Vector3::y()).normalize();

        // --- 3. 移動処理 (キーボード) ---
        let speed = 5.0 * delta_time;
        if input.move_forward { self.position += front * speed; }
        if input.move_backward { self.position -= front * speed; }
        if input.move_right { self.position += right * speed; }
        if input.move_left { self.position -= right * speed; }
    }

    /// レンダラーに渡すための View行列 を生成する
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let front = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();

        Matrix4::look_at_rh(&self.position, &(self.position + front), &Vector3::y())
    }
}