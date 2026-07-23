use nalgebra::Matrix4;
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use rapier3d::prelude::RigidBodyHandle;
use crate::camera::Camera;
use crate::physics::PhysicsWorld;
use crate::scene::{Entity, EntityId, Scene};

new_key_type! {
    pub struct DynamicId;
}

pub struct GameState {
    // 物理ハンドルとその生存を管理
    pub dynamic_bodies: SlotMap<DynamicId, RigidBodyHandle>,
    // 描画用の生行列データ (SoA)
    pub dynamic_transforms: SecondaryMap<DynamicId, [f32; 16]>,
    // カメラの状態
    pub camera: Camera,
    // シーンの状態
    pub scene: Scene
}

impl GameState {

    pub fn new() -> Self {
        Self {
            // SlotMapとSecondaryMapの初期化
            dynamic_bodies: SlotMap::with_key(),
            dynamic_transforms: SecondaryMap::new(),

            camera: Camera::new(),
            scene: Scene::create_test_scene(),
        }
    }

    /// 描画対象となるすべての Entity をまとめて返す
    pub fn get_renderable_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();

        // 1. 静的オブジェクト（目印のランドマーク）をリストに追加
        entities.extend(self.scene.entities.clone());



        // 2. 動的オブジェクト（物理演算の結果）をリストに追加
        // ※現状はメッシュ登録の都合上、全ての動的オブジェクトを暫定で EntityId(1) として扱う
        for raw_matrix_array in self.dynamic_transforms.values() {
            entities.push(Entity {
                id: EntityId(1),
                transform: Matrix4::from_column_slice(raw_matrix_array),
            });
        }

        entities
    }

    pub fn sync_transforms(&mut self, physics: &PhysicsWorld) { // ※型の名前は実際の環境に合わせてください
        // dynamic_bodies に登録されている全ての物理ハンドルについて処理
        for (dynamic_id, handle) in self.dynamic_bodies.iter() {
            // Rapier3D などの剛体セットから、ハンドルの現在の位置・回転を取得
            if let Some(body) = physics.rigid_body_set.get(*handle) {
                // 1. 物理エンジンが持っている位置情報を Mat4 としてそのまま受け取る
                let mat = body.position().to_mat4();

                // 2. nalgebraを挟まずに、直接16個の数字の配列（縦方向の並び）に変換する
                let array: [f32; 16] = mat.to_cols_array();

                // 3. マップに保存する
                self.dynamic_transforms.insert(dynamic_id, array);
            }
        }
    }
}