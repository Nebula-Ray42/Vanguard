use slotmap::{new_key_type, SecondaryMap, SlotMap};
use rapier3d::prelude::RigidBodyHandle;
use crate::physics::PhysicsWorld;
use crate::camera::Camera;

new_key_type! {
    pub struct DynamicId;
}

pub struct GameState {
    // 物理ハンドルとその生存を管理
    pub dynamic_bodies: SlotMap<DynamicId, RigidBodyHandle>,
    // 描画用の生行列データ (SoA)
    pub dynamic_transforms: SecondaryMap<DynamicId, [f32; 16]>,
    // カメラの状態
    pub camera: Camera
}

impl GameState {
    pub fn new() -> Self {
        Self {
            dynamic_bodies: SlotMap::with_key(),
            dynamic_transforms: SecondaryMap::new(),
            camera: Camera::new(1280.0 / 720.0),
        }
    }

    /// 物理ワールドから最新の行列を取得し、描画用配列を同期する (Always-Valid)
    pub fn sync_transforms(&mut self, physics: &PhysicsWorld) {
        for (id, &handle) in self.dynamic_bodies.iter() {
            // physics.rs に実装されている安全な取得メソッドを利用
            let transform = physics.get_transform_matrix(handle);
            self.dynamic_transforms.insert(id, transform);
        }
    }
}