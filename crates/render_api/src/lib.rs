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
    pub transform: Matrix4<f32>,
}

// 頂点1つ分のデータ
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    // TODO 将来的に法線(normal)やUVなどもここに追加する
}

// メッシュ全体を表す純粋なデータ
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// 1フレーム分の描画指示をすべて詰め込んだスナップショット
#[derive(Debug, Clone)]
pub struct RenderSnapshot {
    pub instances: Vec<RenderInstance>,
    pub view_matrix: Matrix4<f32>,
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

impl MeshData {
    /// キューブ（立方体）を生成する
    pub fn new_cube(size: f32, color: [f32; 3]) -> Self {
        let h = size / 2.0;
        let vertices = vec![
            Vertex { position: [-h, -h,  h], color }, // 0: 左下前
            Vertex { position: [ h, -h,  h], color }, // 1: 右下前
            Vertex { position: [ h,  h,  h], color }, // 2: 右上前
            Vertex { position: [-h,  h,  h], color }, // 3: 左上前
            Vertex { position: [-h, -h, -h], color }, // 4: 左下奥
            Vertex { position: [ h, -h, -h], color }, // 5: 右下奥
            Vertex { position: [ h,  h, -h], color }, // 6: 右上奥
            Vertex { position: [-h,  h, -h], color }, // 7: 左上奥
        ];

        // 36個のインデックス（6面 × 2ポリゴン × 3頂点）
        let indices = vec![
            0, 1, 2, 2, 3, 0, // 前
            1, 5, 6, 6, 2, 1, // 右
            7, 6, 5, 5, 4, 7, // 後
            4, 0, 3, 3, 7, 4, // 左
            3, 2, 6, 6, 7, 3, // 上
            4, 5, 1, 1, 0, 4, // 下
        ];

        Self { vertices, indices }
    }

    /// 床（平面）を生成する
    pub fn new_plane(width: f32, depth: f32, color: [f32; 3]) -> Self {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        // Y=0 の平面上に4つの頂点を配置
        let vertices = vec![
            Vertex { position: [-hw, 0.0,  hd], color }, // 0: 左前
            Vertex { position: [ hw, 0.0,  hd], color }, // 1: 右前
            Vertex { position: [ hw, 0.0, -hd], color }, // 2: 右奥
            Vertex { position: [-hw, 0.0, -hd], color }, // 3: 左奥
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0,
        ];

        Self { vertices, indices }
    }
}