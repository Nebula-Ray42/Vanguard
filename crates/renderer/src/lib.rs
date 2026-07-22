// crates/renderer/src/lib.rs
mod context;
mod pipeline;
mod swapchain;
mod sync;

use context::VulkanContext;
use pipeline::GraphicsPipeline;
use swapchain::SwapchainTarget;
use sync::SyncContext;

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct VulkanRenderer {
    context: VulkanContext,
    swapchain_target: SwapchainTarget,
    pipeline: GraphicsPipeline,
    sync: SyncContext,
}

impl VulkanRenderer {
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        let context = VulkanContext::new(display_handle, window_handle)?;

        // 簡易的にキューファミリーインデックスを再取得（またはcontextに持たせる）
        let queue_family_index = unsafe {
            let families = context
                .instance
                .get_physical_device_queue_family_properties(context.physical_device);
            families
                .iter()
                .enumerate()
                .position(|(i, info)| {
                    info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && context
                            .surface_loader
                            .get_physical_device_surface_support(
                                context.physical_device,
                                i as u32,
                                context.surface,
                            )
                            .unwrap_or(false)
                })
                .unwrap() as u32
        };

        let swapchain_target = SwapchainTarget::new(&context, window_width, window_height)?;
        let pipeline = GraphicsPipeline::new(
            &context.device,
            swapchain_target.render_pass,
            swapchain_target.extent,
        )?;
        let sync = SyncContext::new(&context.device, queue_family_index)?;

        Ok(Self {
            context,
            swapchain_target,
            pipeline,
            sync,
        })
    }

    pub fn destroy(&self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();
            self.sync.destroy(&self.context.device);
            self.pipeline.destroy(&self.context.device);
            self.swapchain_target.destroy(&self.context.device);
            self.context.destroy();
        }
    }

    // ★ 物理演算から渡された Y 座標 (offset_y) を受け取る
    pub fn draw_frame(&self, offset_y: f32) -> Result<(), String> {
        let device = &self.context.device;

        unsafe {
            device
                .wait_for_fences(&[self.sync.in_flight_fence], true, std::u64::MAX)
                .map_err(|e| format!("Fence待機失敗: {}", e))?;
            device
                .reset_fences(&[self.sync.in_flight_fence])
                .map_err(|e| format!("Fenceリセット失敗: {}", e))?;

            let (image_index, _) = self
                .swapchain_target
                .loader
                .acquire_next_image(
                    self.swapchain_target.swapchain,
                    std::u64::MAX,
                    self.sync.image_available_semaphore,
                    vk::Fence::null(),
                )
                .map_err(|e| format!("画像取得失敗: {}", e))?;

            device
                .reset_command_buffer(
                    self.sync.command_buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .map_err(|e| format!("CmdBufリセット失敗: {}", e))?;

            let begin_info = vk::CommandBufferBeginInfo::default();
            device
                .begin_command_buffer(self.sync.command_buffer, &begin_info)
                .map_err(|e| format!("CmdBuf記録開始失敗: {}", e))?;

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.2, 0.4, 1.0],
                },
            }];

            let render_pass_begin = vk::RenderPassBeginInfo::default()
                .render_pass(self.swapchain_target.render_pass)
                .framebuffer(self.swapchain_target.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_target.extent,
                })
                .clear_values(&clear_values);

            device.cmd_begin_render_pass(
                self.sync.command_buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            );

            // 描画パイプラインのバインド
            device.cmd_bind_pipeline(
                self.sync.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            // ★ Push Constants を使ってGPUへ現在のY座標（オフセット）を高速転送する
            let push_constants: [f32; 2] = [0.0, offset_y];
            let bytes = std::slice::from_raw_parts(
                push_constants.as_ptr() as *const u8,
                std::mem::size_of_val(&push_constants),
            );
            device.cmd_push_constants(
                self.sync.command_buffer,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytes,
            );

            // 三角形の描画
            device.cmd_draw(self.sync.command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(self.sync.command_buffer);
            device
                .end_command_buffer(self.sync.command_buffer)
                .map_err(|e| format!("CmdBuf記録終了失敗: {}", e))?;

            let wait_semaphores = [self.sync.image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished_semaphore];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device
                .queue_submit(
                    self.context.graphics_queue,
                    &[submit_info],
                    self.sync.in_flight_fence,
                )
                .map_err(|e| format!("Queue submit失敗: {}", e))?;

            let swapchains = [self.swapchain_target.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_target
                .loader
                .queue_present(self.context.graphics_queue, &present_info)
                .map_err(|e| format!("Present失敗: {}", e))?;

            device.device_wait_idle().unwrap();
        }

        Ok(())
    }
}
