use crate::camera::Camera;
use crate::physics::PhysicsWorld;
use crate::scene::{Entity, EntityId, Scene};
use nalgebra::Matrix4;
use rapier3d::prelude::RigidBodyHandle;
use slotmap::{SecondaryMap, SlotMap, new_key_type};

new_key_type! {
    pub struct DynamicId;
}

pub struct GameState {
    pub dynamic_bodies: SlotMap<DynamicId, RigidBodyHandle>,
    pub dynamic_transforms: SecondaryMap<DynamicId, [f32; 16]>,
    pub camera: Camera,
    pub scene: Scene,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            dynamic_bodies: SlotMap::with_key(),
            dynamic_transforms: SecondaryMap::new(),

            camera: Camera::new(),
            scene: Scene::create_test_scene(),
        }
    }

    pub fn get_renderable_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();

        entities.extend(self.scene.entities.clone());

        // TODO ※現状はメッシュ登録の都合上、全ての動的オブジェクトを暫定で EntityId(1) として扱う
        for raw_matrix_array in self.dynamic_transforms.values() {
            entities.push(Entity {
                id: EntityId(1),
                transform: Matrix4::from_column_slice(raw_matrix_array),
            });
        }

        entities
    }

    pub fn sync_transforms(&mut self, physics: &PhysicsWorld) {
        for (dynamic_id, handle) in self.dynamic_bodies.iter() {
            if let Some(body) = physics.rigid_body_set.get(*handle) {
                let mat = body.position().to_mat4();

                let array: [f32; 16] = mat.to_cols_array();

                self.dynamic_transforms.insert(dynamic_id, array);
            }
        }
    }
}
