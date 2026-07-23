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
use core::input::InputState;

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
        view_matrix: game_state.camera.get_view_matrix(),
        instances: Vec::new(),
    };

    // ドメイン層から「描画すべきエンティティ」のリストを全てもらう
    let entities = game_state.get_renderable_entities();

    for entity in entities {

        let render_entity_id = EntityId(entity.id.0);

        let Some(mesh_id) = registry.get_mesh_for(&render_entity_id) else {
            continue;
        };

        // TODO 追加: 行列の 12, 13, 14 番目の要素が、それぞれ X, Y, Z の座標（平行移動）を表します
        let x = entity.transform[12];
        let y = entity.transform[13];
        let z = entity.transform[14];
        println!("オブジェクトの現在座標: X:{:.2}, Y:{:.2}, Z:{:.2}", x, y, z);

        snapshot.instances.push(RenderInstance {
            mesh_id,
            transform: entity.transform,
        });

        snapshot.instances.push(RenderInstance {
            mesh_id,
            transform: entity.transform,
        });
    }

    println!("描画するインスタンス数: {}", snapshot.instances.len());
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

    registry.register_entity(EntityId(1), MeshId(1));

    // --- エンティティの初期化パイプライン ---
    // 1. 物理ワールドにキューブを生成 (高さ10.0から落下)
    let raw_cube_handle = physics.spawn_dynamic_cube(10.0);
    // 2. GameStateに登録し、エンジン全体で管理するIDを発行
    let _cube_id = game_state.dynamic_bodies.insert(raw_cube_handle);
    // 3. レンダリング用のメッシュ登録
    registry.register_entity(EntityId(1), MeshId(1));

    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame_time = Instant::now();

    let mut input_state = InputState::default();

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
                //  追加2: マウス入力フェーズ (DeviceEvent)
                // ==========================================
                Event::DeviceEvent { event: winit::event::DeviceEvent::MouseMotion { delta }, .. } => {
                    input_state.mouse_dx += delta.0 as f32;
                    input_state.mouse_dy += delta.1 as f32;
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

                        let dt = elapsed.as_secs_f32();

                        // (将来的にここで game_state.update_camera(&input_state, elapsed) を呼ぶ)

                        // 次のフレームのためにマウス移動量をリセット
                        input_state.reset_relative_state();

                        physics.step();
                        game_state.sync_transforms(&physics);

                        game_state.camera.update(&input_state, dt);
                        input_state.reset_relative_state();

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

                    if let Some(r) = renderer_opt.as_mut() {
                        if let Err(e) = r.draw_frame(&snapshot) {
                            println!("描画中にエラーが発生しました: {}", e);
                        }
                    }
                }

                // ==========================================
                // 追加3: キーボード入力フェーズ (WindowEvent)
                // ==========================================
                Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } => {
                    let is_pressed = key_event.state == winit::event::ElementState::Pressed;

                    if let winit::keyboard::PhysicalKey::Code(keycode) = key_event.physical_key {
                        use winit::keyboard::KeyCode;
                        match keycode {
                            KeyCode::KeyW => input_state.move_forward = is_pressed,
                            KeyCode::KeyS => input_state.move_backward = is_pressed,
                            KeyCode::KeyA => input_state.move_left = is_pressed,
                            KeyCode::KeyD => input_state.move_right = is_pressed,
                            KeyCode::Escape => elwt.exit(), // オマケ: ESCキーで終了
                            _ => {}
                        }
                    }
                }

                _ => (),
            }
        })
        .unwrap();
}