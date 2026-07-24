use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use render_api::{EntityId, MeshData, RenderInstance, RenderRegistry, RenderSnapshot};
use renderer::VulkanRenderer;
use std::thread;
use std::time::{Duration, Instant};
use winit::{
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use core::input::InputState;
use core::state::GameState;

// ==========================================
// 1. 初期化関数
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
fn build_render_snapshot(game_state: &GameState, registry: &RenderRegistry) -> RenderSnapshot {
    let mut snapshot = RenderSnapshot {
        view_matrix: game_state.camera.get_view_matrix(),
        instances: Vec::new(),
    };

    let entities = game_state.get_renderable_entities();

    for entity in entities {
        let render_entity_id = EntityId(entity.id.0);

        let Some(mesh_id) = registry.get_mesh_for(&render_entity_id) else {
            continue;
        };

        // 行列の 12, 13, 14 番目が X, Y, Z の平行移動を表す
        let _x = entity.transform[12];
        let _y = entity.transform[13];
        let _z = entity.transform[14];
        // ログが多すぎる場合はコメントアウト推奨
        // info!("オブジェクトの現在座標: X:{:.2}, Y:{:.2}, Z:{:.2}", x, y, z);

        snapshot.instances.push(RenderInstance {
            mesh_id,
            transform: entity.transform,
        });
    }

    // もしCore側から「床」のエンティティが渡ってこない場合は、手動で原点に描画指示を追加
    if let Some(floor_mesh_id) = registry.get_mesh_for(&EntityId(0)) {
        snapshot.instances.push(RenderInstance {
            mesh_id: floor_mesh_id,
            transform: nalgebra::Matrix4::identity().into(),
        });
    }

    snapshot
}

// ==========================================
// 3. メインループ (Composition Root)
// ==========================================
#[allow(deprecated)]
fn main() {
    // ---------------------------------------------------------
    // A. ドメインとレンダラーの初期化
    // ---------------------------------------------------------
    let (event_loop, window, mut renderer) = setup_engine();

    let mut game_state = GameState::new();
    let mut registry = RenderRegistry::new();
    // 入力状態を管理する構造体の初期化（前回抜けていた部分）
    let mut input_state = InputState::default(); // new() か default() に合わせてください

    // ※もし PhysicsWorld を GameState の外に出している場合はここで初期化
    // let mut physics = PhysicsWorld::new();

    // ---------------------------------------------------------
    // B. メッシュ生成と登録（所有権移動前に行う！）
    // ---------------------------------------------------------
    let floor_data = MeshData::new_plane(50.0, 50.0, [0.6, 0.6, 0.6]);
    let floor_mesh_id = renderer.create_mesh_from_data(&floor_data).unwrap();

    let cube_data = MeshData::new_cube(1.0, [1.0, 0.2, 0.2]);
    let cube_mesh_id = renderer.create_mesh_from_data(&cube_data).unwrap();

    registry.register_entity(EntityId(0), floor_mesh_id); // 床
    registry.register_entity(EntityId(1), cube_mesh_id);  // キューブ

    // メッシュ準備完了後に Option で包む
    let mut renderer_opt = Some(renderer);

    // ---------------------------------------------------------
    // C. メインループ用タイマー変数の初期化（前回抜けていた部分）
    // ---------------------------------------------------------
    let mut last_frame_time = Instant::now();
    let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0);

    println!("Rey Engine started. Running at 60 FPS...");

    // ---------------------------------------------------------
    // D. イベントループ開始
    // ---------------------------------------------------------
    event_loop
        .run(move |event, elwt| {
            match event {
                // --- 終了フェーズ ---
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    println!("Shutting down Rey Engine...");
                    if let Some(r) = renderer_opt.take() {
                        drop(r);
                    }
                    elwt.exit();
                }

                // --- マウス入力フェーズ ---
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    input_state.mouse_dx += delta.0 as f32;
                    input_state.mouse_dy += delta.1 as f32;
                }

                // --- キーボード入力フェーズ ---
                Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } => {
                    let is_pressed = key_event.state == ElementState::Pressed;
                    if let PhysicalKey::Code(keycode) = key_event.physical_key {
                        match keycode {
                            KeyCode::KeyW => input_state.move_forward = is_pressed,
                            KeyCode::KeyS => input_state.move_backward = is_pressed,
                            KeyCode::KeyA => input_state.move_left = is_pressed,
                            KeyCode::KeyD => input_state.move_right = is_pressed,
                            KeyCode::Escape => elwt.exit(),
                            _ => {}
                        }
                    }
                }

                // --- 物理・状態更新フェーズ (60FPS固定) ---
                Event::AboutToWait => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);

                    if elapsed >= target_frame_duration {
                        last_frame_time = now;
                        let dt = elapsed.as_secs_f32();

                        // 1. 物理演算の更新と座標の同期
                        // ※ご自身の設計（physicsが独立しているか否か）に合わせて変更してください
                        // game_state.physics.step();
                        // game_state.sync_transforms();

                        // ※別変数の場合の例:
                        // physics.step();
                        // game_state.sync_transforms(&physics);

                        // 2. カメラ位置の更新
                        game_state.camera.update(&input_state, dt);

                        // 3. 次のフレームのためにマウス移動量をリセット
                        input_state.reset_relative_state();

                        // 4. 描画リクエストを発行
                        window.request_redraw();
                    } else {
                        // CPUの空回りを防ぐためのスリープ
                        thread::sleep(target_frame_duration - elapsed);
                    }
                }

                // --- 描画フェーズ ---
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    let snapshot = build_render_snapshot(&game_state, &registry);

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