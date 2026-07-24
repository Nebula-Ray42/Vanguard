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
use nalgebra::Matrix4;
use pipeline::GraphicsPipeline;
use render_api::{MeshData, MeshId, RenderSnapshot, Vertex};
use swapchain::SwapchainTarget;
use sync::SyncContext;

use tracing::{info};

/// GPUに転送済みのメッシュデータ（Vulkanの物理的なリソース）
pub struct GpuMesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
}

pub struct VulkanRenderer {
    context: VulkanContext,
    swapchain_target: SwapchainTarget,
    pipeline: GraphicsPipeline,
    sync: SyncContext,
    // 複数のメッシュを動的に管理するためのリスト
    pub meshes: Vec<GpuMesh>,
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

        // 初期化時は空のリストを持たせるだけ！（ボイラープレートの大掃除完了）
        Ok(Self {
            context,
            swapchain_target,
            pipeline,
            sync,
            meshes: Vec::new(),
        })
    }

    /// 純粋なMeshDataを受け取り、VulkanのGPUバッファを生成してMeshIdを返す万能関数
    pub fn create_mesh_from_data(&mut self, data: &MeshData) -> Result<MeshId, String> {
        // --- 1. 頂点バッファの作成と転送 ---
        let vertex_size = (data.vertices.len() * size_of::<Vertex>()) as vk::DeviceSize;
        let (v_staging_buf, v_staging_mem) = self.context.create_buffer(
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = self.context.device.map_memory(v_staging_mem, 0, vertex_size, vk::MemoryMapFlags::empty()).unwrap();
            let mut align = ash::util::Align::new(data_ptr, align_of::<Vertex>() as u64, vertex_size);
            align.copy_from_slice(&data.vertices);
            self.context.device.unmap_memory(v_staging_mem);
        }

        let (vertex_buffer, vertex_buffer_memory) = self.context.create_buffer(
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(&self.context, self.sync.command_pool, v_staging_buf, vertex_buffer, vertex_size);

        unsafe {
            self.context.device.destroy_buffer(v_staging_buf, None);
            self.context.device.free_memory(v_staging_mem, None);
        }

        // --- 2. インデックスバッファの作成と転送 ---
        let index_size = (data.indices.len() * size_of::<u32>()) as vk::DeviceSize;
        let (i_staging_buf, i_staging_mem) = self.context.create_buffer(
            index_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = self.context.device.map_memory(i_staging_mem, 0, index_size, vk::MemoryMapFlags::empty()).unwrap();
            let mut align = ash::util::Align::new(data_ptr, align_of::<u32>() as u64, index_size);
            align.copy_from_slice(&data.indices);
            self.context.device.unmap_memory(i_staging_mem);
        }

        let (index_buffer, index_buffer_memory) = self.context.create_buffer(
            index_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(&self.context, self.sync.command_pool, i_staging_buf, index_buffer, index_size);

        unsafe {
            self.context.device.destroy_buffer(i_staging_buf, None);
            self.context.device.free_memory(i_staging_mem, None);
        }

        // --- 3. メッシュを登録してIDを返す ---
        let mesh_id = MeshId(self.meshes.len() as u32);
        self.meshes.push(GpuMesh {
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            index_count: data.indices.len() as u32,
        });

        Ok(mesh_id)
    }

    pub fn draw_frame(&self, snapshot: &RenderSnapshot) -> Result<(), String> {
        let device = &self.context.device;
        let frame = self.sync.current_frame.get();

        unsafe {
            device.wait_for_fences(&[self.sync.in_flight_fences[frame]], true, std::u64::MAX).unwrap();
            device.reset_fences(&[self.sync.in_flight_fences[frame]]).unwrap();

            let (image_index, _) = self.swapchain_target.loader.acquire_next_image(
                self.swapchain_target.swapchain,
                std::u64::MAX,
                self.sync.image_available_semaphores[frame],
                vk::Fence::null(),
            ).unwrap();

            let command_buffer = self.sync.command_buffers[frame];
            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()).unwrap();

            let begin_info = vk::CommandBufferBeginInfo::default();
            device.begin_command_buffer(command_buffer, &begin_info).unwrap();

            let clear_values = [
                vk::ClearValue { color: vk::ClearColorValue { float32: [0.3, 0.5, 0.8, 1.0] } },
                vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
            ];

            let render_pass_begin = vk::RenderPassBeginInfo::default()
                .render_pass(self.swapchain_target.render_pass)
                .framebuffer(self.swapchain_target.framebuffers[image_index as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_target.extent })
                .clear_values(&clear_values);

            device.cmd_begin_render_pass(command_buffer, &render_pass_begin, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);

            // ========================================================
            // カメラ行列の計算 (重複を削除してスッキリ)
            // ========================================================
            let aspect = self.swapchain_target.extent.width as f32 / self.swapchain_target.extent.height as f32;
            let projection = Matrix4::new_perspective(aspect, std::f32::consts::FRAC_PI_4, 0.1, 100.0);

            let mut vulkan_clip = Matrix4::identity();
            vulkan_clip[(1, 1)] = -1.0;
            vulkan_clip[(2, 2)] = 0.5;
            vulkan_clip[(2, 3)] = 0.5;

            let view_proj = vulkan_clip * projection * snapshot.view_matrix;

            // 3. DTO（RenderSnapshot）のインスタンスをループで描画！
            for instance in &snapshot.instances {
                let mvp = view_proj * instance.transform;

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
                
                // IDから対象のGPUメッシュを取得
                let mesh_index = instance.mesh_id.0 as usize;
                if let Some(gpu_mesh) = self.meshes.get(mesh_index) {

                    // 1. その図形の頂点データをセット
                    device.cmd_bind_vertex_buffers(command_buffer, 0, &[gpu_mesh.vertex_buffer], &[0]);

                    // 2. その図形のインデックスデータ（頂点を繋ぐ順番）をセット
                    device.cmd_bind_index_buffer(command_buffer, gpu_mesh.index_buffer, 0, vk::IndexType::UINT32);

                    // 3. cmd_draw ではなく、インデックスを使う cmd_draw_indexed で描画！
                    device.cmd_draw_indexed(command_buffer, gpu_mesh.index_count, 1, 0, 0, 0);
                }
            }

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer).unwrap();

            let wait_semaphores = [self.sync.image_available_semaphores[frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers_submit = [command_buffer];
            let signal_semaphores = [self.sync.render_finished_semaphores[image_index as usize]];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers_submit)
                .signal_semaphores(&signal_semaphores);

            device.queue_submit(self.context.graphics_queue, &[submit_info], self.sync.in_flight_fences[frame]).unwrap();

            let swapchains = [self.swapchain_target.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_target.loader.queue_present(self.context.graphics_queue, &present_info).unwrap();
            self.sync.current_frame.set((frame + 1) % sync::MAX_FRAMES_IN_FLIGHT);
        }
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        let device = &self.context.device;

        // unsafe fn の中でも、さらに unsafe { ... } ブロックで囲むのが最新のモダンな書き方です
        unsafe {
            for mesh in &self.meshes {
                device.destroy_buffer(mesh.vertex_buffer, None);
                device.free_memory(mesh.vertex_buffer_memory, None);
                device.destroy_buffer(mesh.index_buffer, None);
                device.free_memory(mesh.index_buffer_memory, None);
            }
            self.meshes.clear();

            self.pipeline.destroy(device);
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();

            for mesh in &self.meshes {
                self.context.device.destroy_buffer(mesh.vertex_buffer, None);
                self.context.device.free_memory(mesh.vertex_buffer_memory, None);
                self.context.device.destroy_buffer(mesh.index_buffer, None);
                self.context.device.free_memory(mesh.index_buffer_memory, None);
            }
            self.meshes.clear();

            self.sync.destroy(&self.context.device);
            self.pipeline.destroy(&self.context.device);
            self.swapchain_target.destroy(&self.context.device);

            info!("VulkanRenderer child objects destroyed cleanly.");
        }
    }
}