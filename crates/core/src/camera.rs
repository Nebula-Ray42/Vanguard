// ==========================================
// 1. カメラのデータ構造（Entity/VO）を定義する
// ==========================================
// カメラは「自分の位置(eye)」「見ているターゲット(target)」「上方向(up)」の3つの情報（ベクトル）を持つ。
// これらは状態なので変更可能(mut)にする。

// ==========================================
// 2. カメラデータから「描画用の行列」を計算する純粋関数を作る
// ==========================================
// Rendererはカメラの細かい概念（位置やターゲット）を知る必要はない。
// 最終的に必要なのは「ビュープロジェクション行列」と呼ばれる [f32; 16] の生データだけ。
// nalgebra（またはglam）を使って計算し、生の配列を返すメソッドを作る。

// ==========================================
// 3. GameState にカメラの状態を持たせる
// ==========================================
// ゲームの世界に1つ（または複数）のカメラを配置するため、
// GameStateの構造体の中にカメラのフィールドを追加する。

// ==========================================
// 4. メインループからスナップショットへ渡す
// ==========================================
// 毎フレーム、GameStateからカメラの [f32; 16] を抽出し、
// RenderSnapshot（描画用の純粋なデータ箱）に詰め込んでRendererに渡す。

use glam::Vec3;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect_ratio: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            eye: Vec3::new(0.0, 5.0, 10.0), // 少し上から斜め下を見下ろす位置
            target: Vec3::new(0.0, 0.0, 0.0), // 原点を見る
            up: Vec3::Y, // Y軸が上
            aspect_ratio,
        }
    }

    pub fn build_view_projection_matrix(&self) -> [f32; 16] {
        // 1. ビュー行列（カメラの位置と向き）
        let view = glam::camera::rh::view::look_at_mat4(self.eye, self.target, self.up);

        // 2. プロジェクション行列（遠近法・視野角）
        // 視野角45度(PI/4)、ニアクリップ0.1、ファークリップ100.0
        let proj = glam::camera::rh::proj::opengl::perspective(std::f32::consts::FRAC_PI_4, self.aspect_ratio, 0.1, 100.0);

        // 3. 掛け合わせてRenderer用の生配列に変換
        let view_proj = proj * view;
        view_proj.to_cols_array()
    }
}
