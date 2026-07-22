// crates/render_api/src/lib.rs

/// 3D空間の座標を示す純粋なデータ
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// 1つのオブジェクトを描画するための命令（データ指向）
#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub mesh_id: u32,   // どの3Dモデルを描画するか（今は仮で数値）
    pub position: Vec3, // どこに描画するか
}

/// 1フレーム分の描画命令のリスト
/// Coreがこれを出力し、Rendererがこれを受け取る
#[derive(Debug, Clone)]
pub struct RenderCommandList {
    pub commands: Vec<RenderCommand>,
}

impl RenderCommandList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}
