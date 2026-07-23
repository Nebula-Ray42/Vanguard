pub mod physics;
pub mod state;
pub mod camera;

use nalgebra::{UnitQuaternion, Vector3};
use physics::PhysicsWorld;
use rapier3d::prelude::RigidBodyHandle;

/// エンティティの空間的な状態（位置・回転・スケール）
/// 将来的に Rapier3D の RigidBody の状態と同期する対象
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

/// ゲーム全体のロジックと物理状態を管理する構造体
pub struct GameState {
    pub physics: PhysicsWorld,
    pub cube_handle: RigidBodyHandle,
    pub entities: ()
}

impl GameState {
    pub fn new() -> Self {
        let mut physics = PhysicsWorld::new();
        let cube_handle = physics.spawn_dynamic_cube(5.0);

        Self { physics, cube_handle, entities: () }
    }

    pub fn tick(&mut self) {
        self.physics.step();
    }
}