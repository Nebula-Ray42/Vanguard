pub mod buffer;
pub mod context;
mod mesh;
mod pipeline;
mod swapchain;
mod sync;
pub mod command;

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::mem::{align_of, size_of, size_of_val};

use buffer::copy_buffer;
use context::{VulkanContext, GpuBuffer};
use nalgebra::Matrix4;
use pipeline::GraphicsPipeline;
use render_api::{MeshData, MeshId, RenderSnapshot, Vertex};
use swapchain::SwapchainTarget;
use sync::SyncContext;
use command::CommandRecorder;

use tracing::info;

/// GPUに転送済みのメッシュデータ
pub struct GpuMesh {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub index_count: u32,
}

pub struct VulkanRenderer {
    context: VulkanContext,
    swapchain_target: SwapchainTarget,
    pipeline: GraphicsPipeline,
    sync: SyncContext,
    pub meshes: Vec<GpuMesh>,
}

/// 描画中のフレーム状態を保持するトークン (Layer 4)
pub struct ActiveFrame<'a> {
    pub recorder: CommandRecorder<'a>,
    pub image_index: u32,
    pub frame_index: usize,
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

        Ok(Self {
            context,
            swapchain_target,
            pipeline,
            sync,
            meshes: Vec::new(),
        })
    }

    pub fn create_mesh_from_data(&mut self, data: &MeshData) -> Result<MeshId, String> {
        // --- 1. 頂点バッファの作成と転送 ---
        let vertex_size = (data.vertices.len() * size_of::<Vertex>()) as vk::DeviceSize;

        let v_staging = self.context.create_buffer(
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = self.context.device.map_memory(v_staging.memory, 0, vertex_size, vk::MemoryMapFlags::empty()).unwrap();
            let mut align = ash::util::Align::new(data_ptr, align_of::<Vertex>() as u64, vertex_size);
            align.copy_from_slice(&data.vertices);
            self.context.device.unmap_memory(v_staging.memory);
        }

        let vertex_buffer = self.context.create_buffer(
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(&self.context, self.sync.command_pool, v_staging.buffer, vertex_buffer.buffer, vertex_size);

        // --- 2. インデックスバッファの作成と転送 ---
        let index_size = (data.indices.len() * size_of::<u32>()) as vk::DeviceSize;

        let i_staging = self.context.create_buffer(
            index_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = self.context.device.map_memory(i_staging.memory, 0, index_size, vk::MemoryMapFlags::empty()).unwrap();
            let mut align = ash::util::Align::new(data_ptr, align_of::<u32>() as u64, index_size);
            align.copy_from_slice(&data.indices);
            self.context.device.unmap_memory(i_staging.memory);
        }

        let index_buffer = self.context.create_buffer(
            index_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?.with_index_type(vk::IndexType::UINT32);

        copy_buffer(&self.context, self.sync.command_pool, i_staging.buffer, index_buffer.buffer, index_size);

        // --- 3. メッシュを登録してIDを返す ---
        let mesh_id = MeshId(self.meshes.len() as u32);
        self.meshes.push(GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
        });

        Ok(mesh_id)
    }

    /// ========================================================
    /// メイン描画ループ
    /// ========================================================
    pub fn draw_frame(&self, snapshot: &RenderSnapshot) -> Result<(), String> {
        // 1. フレームの開始（同期処理の隠蔽）
        let active_frame = match self.begin_frame() {
            Some(frame) => frame,
            None => return Ok(()), // リサイズ時などは描画をスキップ
        };

        // 2. コマンド記録フェーズ（完全に安全なラッパー群）
        let recorder = &active_frame.recorder;

        let clear_values = [
            // 0: カラーバッファのクリア（黒色など）
            vk::ClearValue {
                color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
            },
            // 1: 深度バッファのクリア（1.0 = Zクリップ空間における「一番奥」）
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.swapchain_target.render_pass)
            .framebuffer(self.swapchain_target.framebuffers[active_frame.image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_target.extent
            })
            .clear_values(&clear_values);

        recorder.begin_render_pass(&render_pass_begin_info);
        recorder.bind_pipeline(self.pipeline.pipeline);

        // カメラ行列の計算
        let aspect = self.swapchain_target.extent.width as f32 / self.swapchain_target.extent.height as f32;
        let projection = Matrix4::new_perspective(aspect, std::f32::consts::FRAC_PI_4, 0.1, 100.0);
        let mut vulkan_clip = Matrix4::identity();
        vulkan_clip[(1, 1)] = -1.0;
        vulkan_clip[(2, 2)] = 0.5;
        vulkan_clip[(2, 3)] = 0.5;
        let view_proj = vulkan_clip * projection * snapshot.view_matrix;

        for instance in &snapshot.instances {
            let mvp = view_proj * instance.transform;
            let mvp_slice = mvp.as_slice();
            let bytes = unsafe {
                std::slice::from_raw_parts(mvp_slice.as_ptr() as *const u8, size_of_val(mvp_slice))
            };

            recorder.push_constants(self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, bytes);

            let mesh_index = instance.mesh_id.0 as usize;
            if let Some(gpu_mesh) = self.meshes.get(mesh_index) {
                recorder.bind_vertex_buffer(&gpu_mesh.vertex_buffer);
                recorder.bind_index_buffer(&gpu_mesh.index_buffer);
                recorder.draw_indexed(gpu_mesh.index_count, 1, 0, 0, 0);
            }
        }

        recorder.end_render_pass();

        // 3. フレームの終了（送信処理の隠蔽）
        self.end_frame(active_frame);

        Ok(())
    }

    /// ========================================================
    /// フレームの開始（コールドパス）
    /// ========================================================
    pub fn begin_frame(&self) -> Option<ActiveFrame<'_>> {
        let frame = self.sync.current_frame.get();

        unsafe {
            self.context.device.wait_for_fences(&[self.sync.in_flight_fences[frame]], true, std::u64::MAX).unwrap();

            let acquire_result = self.swapchain_target.loader.acquire_next_image(
                self.swapchain_target.swapchain,
                std::u64::MAX,
                self.sync.image_available_semaphores[frame],
                vk::Fence::null(),
            );

            let image_index = match acquire_result {
                Ok((index, _)) => index,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return None,
                Err(e) => panic!("画像の取得に失敗しました: {:?}", e),
            };

            self.context.device.reset_fences(&[self.sync.in_flight_fences[frame]]).unwrap();

            let command_buffer = self.sync.command_buffers[frame];
            self.context.device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()).unwrap();

            let begin_info = vk::CommandBufferBeginInfo::default();
            self.context.device.begin_command_buffer(command_buffer, &begin_info).unwrap();

            let recorder = CommandRecorder::new(&self.context.device, command_buffer);

            Some(ActiveFrame {
                recorder,
                image_index,
                frame_index: frame,
            })
        }
    }

    /// ========================================================
    /// フレームの終了と送信（コールドパス）
    /// ========================================================
    pub fn end_frame(&self, active_frame: ActiveFrame) {
        unsafe {
            self.context.device.end_command_buffer(active_frame.recorder.command_buffer).unwrap();

            let wait_semaphores = [self.sync.image_available_semaphores[active_frame.frame_index]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers_submit = [active_frame.recorder.command_buffer];
            let signal_semaphores = [self.sync.render_finished_semaphores[active_frame.image_index as usize]];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers_submit)
                .signal_semaphores(&signal_semaphores);

            self.context.device.queue_submit(
                self.context.graphics_queue,
                &[submit_info],
                self.sync.in_flight_fences[active_frame.frame_index]
            ).unwrap();

            let swapchains = [self.swapchain_target.swapchain];
            let image_indices = [active_frame.image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            let _ = self.swapchain_target.loader.queue_present(self.context.graphics_queue, &present_info);

            self.sync.current_frame.set((active_frame.frame_index + 1) % sync::MAX_FRAMES_IN_FLIGHT);
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();
            self.meshes.clear(); // ここで RAII によって GpuBuffer の Drop が呼ばれる

            self.sync.destroy(&self.context.device);
            self.pipeline.destroy(&self.context.device);
            self.swapchain_target.destroy(&self.context.device);

            info!("VulkanRenderer child objects destroyed cleanly.");
        }
    }
}