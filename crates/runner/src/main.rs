use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use render_api::{EntityId, MeshData, RenderInstance, RenderRegistry, RenderSnapshot};
use renderer::VulkanRenderer;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info};
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
    let event_loop = EventLoop::new().expect("EventLoopの生成に失敗しました");
    let window = WindowBuilder::new()
        .with_title("Rey Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .expect("ウィンドウの生成に失敗しました");

    let renderer = VulkanRenderer::new(
        window.display_handle().unwrap().as_raw(),
        window.window_handle().unwrap().as_raw(),
        1280,
        720,
    )
    .expect("Vulkanレンダラーの初期化に失敗しました");

    (event_loop, window, renderer)
}

fn init_logger() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .with_target(true)
        .init();
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
            // 必要に応じて warn! でログを出力できます
            // warn!("EntityId({}) はRegistryに未登録のため描画をスキップします", render_entity_id.0);
            continue;
        };

        snapshot.instances.push(RenderInstance {
            mesh_id,
            transform: entity.transform,
        });
    }

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
    // 1. 最優先でロガーを起動（重複していた初期化を削除）
    init_logger();
    info!("Rey Engine starting...");

    // ---------------------------------------------------------
    // A. ドメインとレンダラーの初期化
    // ---------------------------------------------------------
    let (event_loop, window, mut renderer) = setup_engine();

    let mut game_state = GameState::new();
    let mut registry = RenderRegistry::new();
    let mut input_state = InputState::default();

    // ---------------------------------------------------------
    // B. メッシュ生成と登録
    // ---------------------------------------------------------
    let floor_data = MeshData::new_plane(50.0, 50.0, [0.6, 0.6, 0.6]);
    let floor_mesh_id = renderer
        .create_mesh_from_data(&floor_data)
        .expect("床メッシュの生成に失敗しました");

    let cube_data = MeshData::new_cube(1.0, [1.0, 0.2, 0.2]);
    let cube_mesh_id = renderer
        .create_mesh_from_data(&cube_data)
        .expect("キューブメッシュの生成に失敗しました");

    registry.register_entity(EntityId(0), floor_mesh_id); // 床

    for entity in game_state.get_renderable_entities() {
        registry.register_entity(EntityId(entity.id.0), cube_mesh_id);
    }

    let mut renderer_opt = Some(renderer);

    // ---------------------------------------------------------
    // C. メインループ用タイマー変数の初期化
    // ---------------------------------------------------------
    let mut last_frame_time = Instant::now();
    let target_frame_duration = Duration::from_secs_f32(1.0 / 144.0);

    info!("Rey Engine started. Running at 60 FPS...");

    // ---------------------------------------------------------
    // D. イベントループ開始
    // ---------------------------------------------------------
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("Shutting down Rey Engine...");
                    if let Some(r) = renderer_opt.take() {
                        drop(r);
                    }
                    elwt.exit();
                }

                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    input_state.mouse_dx += delta.0 as f32;
                    input_state.mouse_dy += delta.1 as f32;
                }

                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event: key_event, ..
                        },
                    ..
                } => {
                    let is_pressed = key_event.state == ElementState::Pressed;
                    if let PhysicalKey::Code(keycode) = key_event.physical_key {
                        match keycode {
                            KeyCode::KeyW => input_state.move_forward = is_pressed,
                            KeyCode::KeyS => input_state.move_backward = is_pressed,
                            KeyCode::KeyA => input_state.move_left = is_pressed,
                            KeyCode::KeyD => input_state.move_right = is_pressed,
                            KeyCode::Space => input_state.move_up = is_pressed,
                            KeyCode::ShiftLeft => input_state.move_down = is_pressed,
                            KeyCode::Escape => elwt.exit(),
                            _ => {}
                        }
                    }
                }

                Event::AboutToWait => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);

                    if elapsed >= target_frame_duration {
                        last_frame_time = now;
                        let dt = elapsed.as_secs_f32();

                        game_state.camera.update(&input_state, dt);
                        input_state.reset_relative_state();

                        window.request_redraw();
                    } else {
                        thread::sleep(target_frame_duration - elapsed);
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let snapshot = build_render_snapshot(&game_state, &registry);

                    if let Some(r) = renderer_opt.as_mut() {
                        if let Err(e) = r.draw_frame(&snapshot) {
                            // 修正の成果がここで発揮されます！
                            error!("描画中にエラーが発生しました: {}", e);
                        }
                    }
                }

                _ => (),
            }
        })
        .expect("イベントループの実行中にエラーが発生しました");
}
