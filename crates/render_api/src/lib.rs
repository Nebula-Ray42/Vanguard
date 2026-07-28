pub mod engine_error;
#[path = "render-pass-error.rs"]
pub mod render_pass_error;
pub mod swapchain;
pub mod mapping_error;

// crates/render_api/src/lib.rs
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use nalgebra::Matrix4;
use std::collections::HashMap;
use shared::engine_data;
use crate::mapping_error::MappingError;


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
    pub normal: [f32; 3],
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuTransform {
    pub position: [f32; 3],
    pub _pad0: f32,         // 12 + 4 = 16バイト（1つ目のvec4）
    pub rotation: [f32; 4], // 16バイト（2つ目のvec4）
    pub scale: [f32; 3],
    pub _pad1: f32, // 12 + 4 = 16バイト（3つ目のvec4）
} // 合計48バイト（完全に16の倍数）

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuEntity {
    pub id: u32,
    pub mesh_id: u32,
    pub _pad0: [u32; 2], // u32が2つで8バイト。id(4) + mesh_id(4) + pad(8) = 16バイト
    pub transform: GpuTransform, // 48バイト（16の倍数なのでズレない）
} // 合計64バイト（完全に16の倍数）

// ==========================================
// 3. IDマッピング層
// ==========================================

/// Core側のエンティティIDと、Renderer側のメッシュ/リソースIDを紐付ける辞書
#[derive(Debug, Default)]
pub struct RenderRegistry {
    entity_to_mesh: HashMap<EntityId, MeshId>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PushConstants {
    pub mvp: Mat4,
}

impl PushConstants {
    /// 3つの行列からMVP行列を構築し、常に妥当な状態(Always-Valid)として生成する
    pub fn new(model: Mat4, view: Mat4, proj: Mat4) -> Self {
        // glamでは行列の乗算は * 演算子を使用し、数式通り P * V * M の順に記述する
        let mvp = proj * view * model;
        Self { mvp }
    }

    /// 検証用のバイト列表現を取得する
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
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
    /// キューブ（立方体）を生成する（フラットシェーディング用24頂点）
    pub fn new_cube(size: f32, color: [f32; 3]) -> Self {
        let h = size / 2.0;

        // 各面の法線
        let n_front = [0.0, 0.0, 1.0];
        let n_back = [0.0, 0.0, -1.0];
        let n_top = [0.0, 1.0, 0.0];
        let n_bottom = [0.0, -1.0, 0.0];
        let n_right = [1.0, 0.0, 0.0];
        let n_left = [-1.0, 0.0, 0.0];

        // 6面 × 4頂点 = 24頂点
        let vertices = vec![
            // 前 (Front)
            Vertex {
                position: [-h, -h, h],
                color,
                normal: n_front,
            }, // 0
            Vertex {
                position: [h, -h, h],
                color,
                normal: n_front,
            }, // 1
            Vertex {
                position: [h, h, h],
                color,
                normal: n_front,
            }, // 2
            Vertex {
                position: [-h, h, h],
                color,
                normal: n_front,
            }, // 3
            // 後 (Back)
            Vertex {
                position: [h, -h, -h],
                color,
                normal: n_back,
            }, // 4
            Vertex {
                position: [-h, -h, -h],
                color,
                normal: n_back,
            }, // 5
            Vertex {
                position: [-h, h, -h],
                color,
                normal: n_back,
            }, // 6
            Vertex {
                position: [h, h, -h],
                color,
                normal: n_back,
            }, // 7
            // 上 (Top)
            Vertex {
                position: [-h, h, h],
                color,
                normal: n_top,
            }, // 8
            Vertex {
                position: [h, h, h],
                color,
                normal: n_top,
            }, // 9
            Vertex {
                position: [h, h, -h],
                color,
                normal: n_top,
            }, // 10
            Vertex {
                position: [-h, h, -h],
                color,
                normal: n_top,
            }, // 11
            // 下 (Bottom)
            Vertex {
                position: [-h, -h, -h],
                color,
                normal: n_bottom,
            }, // 12
            Vertex {
                position: [h, -h, -h],
                color,
                normal: n_bottom,
            }, // 13
            Vertex {
                position: [h, -h, h],
                color,
                normal: n_bottom,
            }, // 14
            Vertex {
                position: [-h, -h, h],
                color,
                normal: n_bottom,
            }, // 15
            // 右 (Right)
            Vertex {
                position: [h, -h, h],
                color,
                normal: n_right,
            }, // 16
            Vertex {
                position: [h, -h, -h],
                color,
                normal: n_right,
            }, // 17
            Vertex {
                position: [h, h, -h],
                color,
                normal: n_right,
            }, // 18
            Vertex {
                position: [h, h, h],
                color,
                normal: n_right,
            }, // 19
            // 左 (Left)
            Vertex {
                position: [-h, -h, -h],
                color,
                normal: n_left,
            }, // 20
            Vertex {
                position: [-h, -h, h],
                color,
                normal: n_left,
            }, // 21
            Vertex {
                position: [-h, h, h],
                color,
                normal: n_left,
            }, // 22
            Vertex {
                position: [-h, h, -h],
                color,
                normal: n_left,
            }, // 23
        ];

        // 6面 × 2ポリゴン × 3頂点 = 36インデックス
        let indices = vec![
            0, 1, 2, 2, 3, 0, // 前
            4, 5, 6, 6, 7, 4, // 後
            8, 9, 10, 10, 11, 8, // 上
            12, 13, 14, 14, 15, 12, // 下
            16, 17, 18, 18, 19, 16, // 右
            20, 21, 22, 22, 23, 20, // 左
        ];

        Self { vertices, indices }
    }

    /// 床（平面）を生成する
    pub fn new_plane(width: f32, depth: f32, color: [f32; 3]) -> Self {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        // 床は真上を向いているので Y=1.0 が法線
        let n_up = [0.0, 1.0, 0.0];

        let vertices = vec![
            Vertex {
                position: [-hw, 0.0, hd],
                color,
                normal: n_up,
            }, // 0: 左前
            Vertex {
                position: [hw, 0.0, hd],
                color,
                normal: n_up,
            }, // 1: 右前
            Vertex {
                position: [hw, 0.0, -hd],
                color,
                normal: n_up,
            }, // 2: 右奥
            Vertex {
                position: [-hw, 0.0, -hd],
                color,
                normal: n_up,
            }, // 3: 左奥
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self { vertices, indices }
    }
}

// ==========================================
// 変換ロジック
// ==========================================
impl GpuEntity {
    pub fn try_from_fb(fb_entity: &engine_data::Entity) -> Result<Self, MappingError> {
        let fb_transform = fb_entity.transform().ok_or(MappingError::MissingTransform)?;

        let pos = fb_transform.position().ok_or(MappingError::MissingPosition)?;
        let rot = fb_transform.rotation().ok_or(MappingError::MissingRotation)?;
        let scale = fb_transform.scale().ok_or(MappingError::MissingScale)?;

        let gpu_transform = GpuTransform {
            position: [pos.x(), pos.y(), pos.z()],
            _pad0: 0.0,
            rotation: [rot.x(), rot.y(), rot.z(), rot.w()],
            scale: [scale.x(), scale.y(), scale.z()],
            _pad1: 0.0,
        };

        Ok(Self {
            id: fb_entity.id(),
            mesh_id: fb_entity.mesh_id(),
            _pad0: [0, 0],
            transform: gpu_transform,
        })
    }
}

