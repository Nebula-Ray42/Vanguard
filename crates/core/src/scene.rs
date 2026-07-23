// crates/core/src/scene.rs
use nalgebra::{Matrix4, Vector3};

/// 空間上の位置や姿勢を示す純粋なデータ (Value Object)
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    // 将来的に rotation(回転) や scale(拡大縮小) もここに追加します
}

impl Transform {
    pub fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u32);

/// 世界に存在する物体 (Entity)
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    // 将来的に mesh_id や physics_body_id などを持たせます
    pub transform: Matrix4<f32>,
}

/// 空間全体を管理する集約ルート
#[derive(Debug, Clone)]
pub struct Scene {
    pub entities: Vec<Entity>,
}

impl Scene {
    pub fn create_test_scene() -> Self {
        let positions = [
            Vector3::new(0.0, 0.0, 0.0),    // 中央 (原点)
            Vector3::new(5.0, 0.0, 0.0),    // 右
            Vector3::new(-5.0, 0.0, 0.0),   // 左
            Vector3::new(0.0, 0.0, -5.0),   // 奥
            Vector3::new(0.0, 0.0, 5.0),    // 手前
        ];
        

        let entities = positions
            .into_iter()
            .enumerate()
            .map(|(i, pos)| Entity {
                // 動的オブジェクトとIDが被らないよう、1000番から連番を振る
                id: EntityId(1000 + i as u32),
                // Vector3から直接Matrix4を生成
                transform: Matrix4::new_translation(&pos),
            })
            .collect();

        Self { entities }
    }
}