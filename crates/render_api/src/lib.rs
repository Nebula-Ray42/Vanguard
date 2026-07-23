// crates/render_api/src/lib.rs
use nalgebra::Matrix4;
use std::collections::HashMap;

// ==========================================
// 1. Value Objects (ID群)
// ==========================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub u32);

// ==========================================
// 2. DTO (描画用データ構造)
// ==========================================

/// 1つのメッシュを描画するための情報
#[derive(Debug, Clone)]
pub struct RenderInstance {
    pub mesh_id: MeshId,
    /// どこに、どの向きで、どの大きさで描画するか（Model行列）
    pub transform: Matrix4<f32>,
}

/// 1フレーム分の描画指示をすべて詰め込んだスナップショット（完全なイミュータブルDTO）
#[derive(Debug, Clone)]
pub struct RenderSnapshot {
    pub instances: Vec<RenderInstance>,
}

// ==========================================
// 3. IDマッピング層
// ==========================================

/// Core側のエンティティIDと、Renderer側のメッシュ/リソースIDを紐付ける辞書
#[derive(Debug, Default)]
pub struct RenderRegistry {
    entity_to_mesh: HashMap<EntityId, MeshId>,
}

impl RenderRegistry {
    pub fn new() -> Self {
        Self {
            entity_to_mesh: HashMap::new(),
        }
    }

    /// エンティティと描画するメッシュを紐付ける
    pub fn register_entity(&mut self, entity: EntityId, mesh: MeshId) {
        self.entity_to_mesh.insert(entity, mesh);
    }

    /// エンティティの紐付けを解除する
    pub fn unregister_entity(&mut self, entity: &EntityId) {
        self.entity_to_mesh.remove(entity);
    }

    /// エンティティIDから、対応するメッシュIDを取得する
    pub fn get_mesh_for(&self, entity: &EntityId) -> Option<MeshId> {
        self.entity_to_mesh.get(entity).copied()
    }
}