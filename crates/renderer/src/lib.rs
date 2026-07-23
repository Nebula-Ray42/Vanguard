mod buffer;
pub mod context;
mod mesh;
mod pipeline;
mod swapchain;
mod sync;

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use buffer::copy_buffer;
use context::VulkanContext;
use mesh::{VERTICES, Vertex};
use nalgebra::{Matrix4, Point3, Vector3};
use pipeline::GraphicsPipeline;
use render_api::RenderSnapshot;
use swapchain::SwapchainTarget;
use sync::SyncContext;

pub struct VulkanRenderer {
    context: VulkanContext,
    swapchain_target: SwapchainTarget,
    pipeline: GraphicsPipeline,
    sync: SyncContext,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
}

impl VulkanRenderer {
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        let context = VulkanContext::new(display_handle, window_handle)?;

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
        let image_count = swapchain_target.images.len() as u32;
        let sync = SyncContext::new(&context.device, queue_family_index, image_count)?;

        // Staging Buffer 転送処理
        let buffer_size = size_of_val(&VERTICES) as vk::DeviceSize;
        let (staging_buffer, staging_memory) = context.create_buffer(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = context
                .device
                .map_memory(staging_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .unwrap();
            let mut align =
                ash::util::Align::new(data_ptr, align_of::<Vertex>() as u64, buffer_size);
            align.copy_from_slice(&VERTICES);
            context.device.unmap_memory(staging_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = context.create_buffer(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // bufferモジュールから呼び出し
        copy_buffer(
            &context,
            sync.command_pool,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        );

        unsafe {
            context.device.destroy_buffer(staging_buffer, None);
            context.device.free_memory(staging_memory, None);
        }

        Ok(Self {
            context,
            swapchain_target,
            pipeline,
            sync,
            vertex_buffer,
            vertex_buffer_memory,
        })
    }

    pub fn draw_frame(&self, snapshot: &RenderSnapshot) -> Result<(), String> {
        let device = &self.context.device;
        let frame = self.sync.current_frame.get();

        unsafe {
            device
                .wait_for_fences(&[self.sync.in_flight_fences[frame]], true, std::u64::MAX)
                .unwrap();
            device
                .reset_fences(&[self.sync.in_flight_fences[frame]])
                .unwrap();

            let (image_index, _) = self
                .swapchain_target
                .loader
                .acquire_next_image(
                    self.swapchain_target.swapchain,
                    std::u64::MAX,
                    self.sync.image_available_semaphores[frame],
                    vk::Fence::null(),
                )
                .unwrap();

            let command_buffer = self.sync.command_buffers[frame];
            device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();

            let begin_info = vk::CommandBufferBeginInfo::default();
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();

            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.3, 0.5, 0.8, 1.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let render_pass_begin = vk::RenderPassBeginInfo::default()
                .render_pass(self.swapchain_target.render_pass)
                .framebuffer(self.swapchain_target.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_target.extent,
                })
                .clear_values(&clear_values);

            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0_u64]);

            // ========================================================
            // 変更点ここから：カメラの計算とDTOのループ描画
            // ========================================================

            // 1. 画面のアスペクト比を取得
            let aspect = self.swapchain_target.extent.width as f32
                / self.swapchain_target.extent.height as f32;

            // 2. カメラ行列（View）と投影行列（Projection）を nalgebra で計算
            // （※将来的にはこれもDTOとして外部から渡すようにすると更に疎結合になります）
            let projection =
                Matrix4::new_perspective(aspect, std::f32::consts::FRAC_PI_4, 0.1, 100.0);

            // カメラを (0, 3, 5) の位置に置き、原点 (0, 0, 0) を見つめさせる
            let view = Matrix4::look_at_rh(
                &Point3::new(0.0, 3.0, 5.0),
                &Point3::new(0.0, 0.0, 0.0),
                &Vector3::y(),
            );

            // VulkanはY軸が下向き、Zクリップが0〜1なので、nalgebra(OpenGL基準)の行列を補正する
            let mut vulkan_clip = Matrix4::identity();
            vulkan_clip[(1, 1)] = -1.0; // Y軸反転
            vulkan_clip[(2, 2)] = 0.5; // Z軸スケール
            vulkan_clip[(2, 3)] = 0.5; // Z軸平行移動

            // 全てのオブジェクトに共通する ViewProjection 行列
            let view_proj = vulkan_clip * projection * view;

            // 3. DTO（RenderSnapshot）のインスタンスをループで描画！
            for instance in &snapshot.instances {
                // MVP行列 = ViewProjection * Model行列（DTOから取得）
                let mvp = view_proj * instance.transform;

                // Matrix4 を f32 のスライスとして GPU に転送可能なバイト配列に変換
                let mvp_slice = mvp.as_slice();
                let bytes = std::slice::from_raw_parts(
                    mvp_slice.as_ptr() as *const u8,
                    size_of_val(mvp_slice),
                );

                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    bytes,
                );

                // 今はすべてキューブ（36頂点）として描画
                // ※将来は registry.get_mesh_info(instance.mesh_id) 等で頂点数やオフセットを取得します
                device.cmd_draw(command_buffer, 36, 1, 0, 0);
            }

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer).unwrap();

            // ※ ここから下の submit や present 処理もそのまま
            let wait_semaphores = [self.sync.image_available_semaphores[frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers_submit = [command_buffer];
            let signal_semaphores = [self.sync.render_finished_semaphores[image_index as usize]];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers_submit)
                .signal_semaphores(&signal_semaphores);

            device
                .queue_submit(
                    self.context.graphics_queue,
                    &[submit_info],
                    self.sync.in_flight_fences[frame],
                )
                .unwrap();

            let swapchains = [self.swapchain_target.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_target
                .loader
                .queue_present(self.context.graphics_queue, &present_info)
                .unwrap();
            self.sync
                .current_frame
                .set((frame + 1) % sync::MAX_FRAMES_IN_FLIGHT);
        }
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();

            self.context.device.destroy_buffer(self.vertex_buffer, None);
            self.context.device.free_memory(self.vertex_buffer_memory, None);
            
            self.sync.destroy(&self.context.device);
            self.pipeline.destroy(&self.context.device);
            self.swapchain_target.destroy(&self.context.device);

            println!("VulkanRenderer child objects destroyed cleanly.");
        }
    }
}
