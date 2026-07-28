// crates/core/src/physics.rs
use rapier3d::prelude::*;

/// 物理エンジンのすべての状態をカプセル化するドメインオブジェクト
pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    gravity: Vector, // <f32> を削除
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    // TODO query_pipeline はレイキャスト等が必要になるまで一旦削除
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, -9.81, 0.0].into(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    /// 物理シミュレーションを1ステップ進める
    pub fn step(&mut self) {
        let physics_hooks = ();
        let event_handler = ();

        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &physics_hooks,
            &event_handler,
        );
    }
}

impl PhysicsWorld {
    pub fn spawn_dynamic_cube(&mut self, start_y: f32) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, start_y, -5.0].into())
            .build();
        let handle = self.rigid_body_set.insert(rigid_body);

        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5).build();
        self.collider_set
            .insert_with_parent(collider, handle, &mut self.rigid_body_set);

        let floor_collider = ColliderBuilder::cuboid(50.0, 0.1, 50.0)
            .translation(vector![0.0, -2.0, 0.0].into())
            .build();

        self.collider_set.insert(floor_collider);

        handle
    }

    pub fn get_transform_matrix(&self, handle: RigidBodyHandle) -> [f32; 16] {
        let body = self.rigid_body_set.get(handle).unwrap();
        let pos = body.position();

        let glam_mat = glam::Mat4::from_rotation_translation(pos.rotation, pos.translation);
        glam_mat.to_cols_array()
    }
}
