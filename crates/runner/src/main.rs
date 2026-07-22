use core::GameState;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use renderer::VulkanRenderer;
use std::thread;
use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[allow(deprecated)]
fn main() {
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

    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame_time = Instant::now();

    let mut game_state = GameState::new();

    // ★ 物理演算から受け取ったY座標を一時的に覚えておく変数
    let mut current_box_y = 0.0_f32;

    event_loop.set_control_flow(ControlFlow::Poll);
    println!("Rey Engine started. Running at 60 FPS...");

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("Shutting down Rey Engine...");
                    renderer.destroy();
                    elwt.exit();
                }

                Event::AboutToWait => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);

                    if elapsed >= target_frame_duration {
                        last_frame_time = now;

                        // 物理演算の世界を1コマ進める
                        let render_commands = game_state.tick();

                        // ★ 計算されたBoxのY座標を、描画用に保存する
                        current_box_y = render_commands.commands[0].position.y;

                        if game_state.frame_count % 10 == 0 {
                            println!(
                                "Frame {}: Box Y = {:.4}",
                                game_state.frame_count, current_box_y
                            );
                        }

                        window.request_redraw();
                    } else {
                        let sleep_duration = target_frame_duration - elapsed;
                        thread::sleep(sleep_duration);
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // ★ 保持しているY座標をレンダラーに渡し、GPUの Push Constants 経由で反映させる！
                    if let Err(e) = renderer.draw_frame(current_box_y) {
                        println!("描画中にエラーが発生しました: {}", e);
                    }
                }

                _ => (),
            }
        })
        .unwrap();
}
