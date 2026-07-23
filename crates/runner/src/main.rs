use nalgebra::Matrix4;
use render_api::{EntityId, MeshId, RenderInstance, RenderRegistry, RenderSnapshot};
use renderer::VulkanRenderer;
use std::thread;
use std::time::{Duration, Instant};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use core::physics::PhysicsWorld;
use core::state::GameState;

// ==========================================
// 1. 初期化関数の抽出 (変更なし)
// ==========================================
fn setup_engine() -> (EventLoop<()>, Window, VulkanRenderer) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Rey Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();

    let renderer = VulkanRenderer::new(
        window.display_handle().unwrap().as_raw(),
        window.window_handle().unwrap().as_raw(),
        1280,
        720,
    ).expect("Vulkanレンダラーの初期化に失敗しました");

    (event_loop, window, renderer)
}

// ==========================================
// 2. データ変換関数 (純粋関数)
// ==========================================
/// GameStateのSoA配列から、レンダラー用のスナップショットを生成する
fn build_render_snapshot(game_state: &GameState, registry: &RenderRegistry) -> RenderSnapshot {
    let mut snapshot = RenderSnapshot {
        camera_matrix: game_state.camera.build_view_projection_matrix(),
        instances: Vec::new(),
    };

    // 暫定として EntityId(1) を使用していますが、将来は DynamicId との紐付けを行います
    let entity_id = EntityId(1);

    if let Some(mesh_id) = registry.get_mesh_for(&entity_id) {
        // GameState内のすべての動的エンティティの行列をイテレート
        for raw_matrix_array in game_state.dynamic_transforms.values() {
            let model_matrix = Matrix4::from_column_slice(raw_matrix_array);
            snapshot.instances.push(RenderInstance {
                mesh_id,
                transform: model_matrix,
            });
        }
    }

    snapshot
}

// ==========================================
// 3. メインループ (Composition Root)
// ==========================================
#[allow(deprecated)]
fn main() {
    let (event_loop, window, renderer) = setup_engine();
    let mut renderer_opt = Some(renderer);

    // 各ドメインの独立した初期化
    let mut physics = PhysicsWorld::new();
    let mut game_state = GameState::new();
    let mut registry = RenderRegistry::new();

    // --- エンティティの初期化パイプライン ---
    // 1. 物理ワールドにキューブを生成 (高さ10.0から落下)
    let raw_cube_handle = physics.spawn_dynamic_cube(10.0);
    // 2. GameStateに登録し、エンジン全体で管理するIDを発行
    let _cube_id = game_state.dynamic_bodies.insert(raw_cube_handle);
    // 3. レンダリング用のメッシュ登録
    registry.register_entity(EntityId(1), MeshId(1));

    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame_time = Instant::now();

    event_loop.set_control_flow(ControlFlow::Poll);

    // ログ前提のDDDアーキテクチャであれば、将来的に `tracing` 等のロガーへの置換を推奨します
    println!("Rey Engine started. Running at 60 FPS...");

    event_loop
        .run(move |event, elwt| {
            match event {
                // ==========================================
                // 終了フェーズ
                // ==========================================
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    println!("Shutting down Rey Engine...");

                    // take() で中身を抜き取って破棄する（所有権の移動）
                    if let Some(r) = renderer_opt.take() {
                        drop(r);
                    }

                    elwt.exit();
                }

                // ==========================================
                // 物理・状態更新フェーズ
                // ==========================================
                Event::AboutToWait => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);

                    if elapsed >= target_frame_duration {
                        last_frame_time = now;

                        // ① 物理シミュレーションを1ステップ進める
                        physics.step();
                        // ② 物理の結果をGameStateの行列配列 ([f32; 16]) に同期する
                        game_state.sync_transforms(&physics);

                        window.request_redraw();
                    } else {
                        thread::sleep(target_frame_duration - elapsed);
                    }
                }

                // ==========================================
                // 描画フェーズ
                // ==========================================
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    let snapshot = build_render_snapshot(&game_state, &registry);

                    // 🟢 ここが重要！
                    // renderer がまだ存在している場合（破棄されていない場合）のみ描画する。
                    // as_mut() を使うことで、中身を「一時的に借用」してメソッドを呼べる。
                    if let Some(r) = renderer_opt.as_mut() {
                        if let Err(e) = r.draw_frame(&snapshot) {
                            println!("描画中にエラーが発生しました: {}", e);
                        }
                    }
                }

                _ => (),
            }
        })
        .unwrap();
}