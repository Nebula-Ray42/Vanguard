use core::GameState;
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

// ==========================================
// 1. 初期化関数の抽出
// ==========================================
/// ウィンドウ、イベントループ、レンダラーの構築を一つの関数にまとめる
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
    )
        .expect("Vulkanレンダラーの初期化に失敗しました");

    (event_loop, window, renderer)
}

// ==========================================
// 2. データ変換関数 (純粋関数) の抽出
// ==========================================

fn build_render_snapshot(game_state: &GameState, registry: &RenderRegistry) -> RenderSnapshot {
    let mut snapshot = RenderSnapshot {
        instances: Vec::new(),
    };

    let entity_id = EntityId(1);

    if let Some(mesh_id) = registry.get_mesh_for(&entity_id) {
        // 1. Coreからは「純粋な数値の配列(16要素)」として受け取る
        let raw_matrix_array = game_state.physics.get_transform_matrix(game_state.cube_handle);
        
        let model_matrix = Matrix4::from_column_slice(&raw_matrix_array);

        snapshot.instances.push(RenderInstance {
            mesh_id,
            transform: model_matrix,
        });
    }

    snapshot
}

// ==========================================
// 3. メインループ (Composition Root)
// ==========================================
#[allow(deprecated)]
fn main() {
    // 抽出した関数でスッキリと初期化
    let (event_loop, window, renderer) = setup_engine();

    let mut game_state = GameState::new();
    let mut registry = RenderRegistry::new();
    registry.register_entity(EntityId(1), MeshId(1));

    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame_time = Instant::now();

    event_loop.set_control_flow(ControlFlow::Poll);
    println!("Rey Engine started. Running at 60 FPS...");

    event_loop
        .run(move |event, elwt| {
            match event {
                // アプリケーション終了
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    println!("Shutting down Rey Engine...");
                    renderer.destroy();
                    elwt.exit();
                }

                // 物理更新フェーズ (Tick)
                Event::AboutToWait => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);

                    if elapsed >= target_frame_duration {
                        last_frame_time = now;
                        game_state.tick();
                        window.request_redraw();
                    } else {
                        thread::sleep(target_frame_duration - elapsed);
                    }
                }

                // 描画フェーズ (Draw)
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    // 抽出した変換関数を呼ぶだけで、コードの意図が明確になる
                    let snapshot = build_render_snapshot(&game_state, &registry);

                    if let Err(e) = renderer.draw_frame(&snapshot) {
                        println!("描画中にエラーが発生しました: {}", e);
                    }
                }

                _ => (),
            }
        })
        .unwrap();
}