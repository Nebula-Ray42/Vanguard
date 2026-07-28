// crates/core/src/camera.rs
use crate::input::InputState;
use nalgebra::{Matrix4, Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::new(0.0, 3.0, 5.0),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.5,
        }
    }

    pub fn update(&mut self, input: &InputState, delta_time: f32) {
        let sensitivity = 0.002;

        self.yaw += input.mouse_dx * sensitivity;
        self.pitch += input.mouse_dy * sensitivity;

        let max_pitch = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        let front = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        let right = front.cross(&Vector3::y()).normalize();

        let speed = 5.0 * delta_time;
        if input.move_forward {
            self.position += front * speed;
        }
        if input.move_backward {
            self.position -= front * speed;
        }
        if input.move_right {
            self.position += right * speed;
        }
        if input.move_left {
            self.position -= right * speed;
        }
        if input.move_up {
            self.position.y += speed;
        }
        if input.move_down {
            self.position.y -= speed;
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let front = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        Matrix4::look_at_rh(&self.position, &(self.position + front), &Vector3::y())
    }
}
