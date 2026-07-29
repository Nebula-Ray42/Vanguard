// src/lib.rs
mod buffer;
pub mod vulkan;
pub mod render;
pub mod scene;
pub mod descriptors;
pub mod buffers;

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::mem::{align_of, size_of, size_of_val};

use buffer::copy_buffer;
use vulkan::context::{GpuBuffer, VulkanContext};
use vulkan::pipeline::GraphicsPipeline;
use vulkan::swapchain::SwapchainTarget;
use vulkan::sync::SyncContext;
use render::command::CommandRecorder;

use nalgebra::Matrix4;
use render_api::{MeshData, MeshId, RenderSnapshot, Vertex};

use crate::vulkan::sync;
use render_api::engine_error::EngineError;
use tracing::info;
use crate::descriptors::layout::GlobalUbo;

/// GPUに転送済みのメッシュデータ
pub struct GpuMesh {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub index_count: u32,
}

/// 描画中のフレーム状態を保持するトークン
pub struct ActiveFrame<'a> {
    pub recorder: CommandRecorder<'a>,
    pub image_index: u32,
    pub frame_index: usize,
}

pub struct VulkanRenderer {
    context: VulkanContext,
    swapchain_target: SwapchainTarget,
    pipeline: GraphicsPipeline,
    sync: SyncContext,
    pub meshes: Vec<GpuMesh>,
    global_ubo_buffer: GpuBuffer,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    global_descriptor_set: vk::DescriptorSet,
}

impl VulkanRenderer {
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, EngineError> {
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

        // 1. UBO用のバッファ（CPUから書き込み可能）を作成
        let global_ubo_buffer = context.create_buffer(
            size_of::<GlobalUbo>() as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // 2. Descriptor Set Layout の作成（Slangの binding(0, 0) に対応）
        let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&ubo_binding));
        let descriptor_set_layout = unsafe {
            context.device.create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| EngineError::Legacy(format!("Layout生成失敗: {}", e)))?
        };

        // 3. Descriptor Pool と Set の確保
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1);
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);
        let descriptor_pool = unsafe {
            context.device.create_descriptor_pool(&pool_info, None).unwrap()
        };

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let global_descriptor_set = unsafe {
            context.device.allocate_descriptor_sets(&alloc_info).unwrap()[0]
        };

        // 4. Set にバッファを紐付け (WriteDescriptorSet)
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(global_ubo_buffer.buffer)
            .offset(0)
            .range(size_of::<GlobalUbo>() as vk::DeviceSize);
        let write_set = vk::WriteDescriptorSet::default()
            .dst_set(global_descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
        unsafe { context.device.update_descriptor_sets(&[write_set], &[]); }

        let pipeline = GraphicsPipeline::new(
            &context.device,
            swapchain_target.render_pass,
            swapchain_target.extent,
            descriptor_set_layout,
        )?;
        let image_count = swapchain_target.images.len() as u32;
        let sync = SyncContext::new(&context.device, queue_family_index, image_count)?;

        Ok(Self {
            context,
            swapchain_target,
            pipeline,
            sync,
            meshes: Vec::new(),
            global_ubo_buffer,
            descriptor_pool,
            descriptor_set_layout,
            global_descriptor_set,
        })
    }

    pub fn create_mesh_from_data(&mut self, data: &MeshData) -> Result<MeshId, EngineError> {
        // --- 1. 頂点バッファの作成と転送 ---
        let vertex_size = (data.vertices.len() * size_of::<Vertex>()) as vk::DeviceSize;

        let v_staging = self.context.create_buffer(
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = self
                .context
                .device
                .map_memory(
                    v_staging.memory,
                    0,
                    vertex_size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let mut align =
                ash::util::Align::new(data_ptr, align_of::<Vertex>() as u64, vertex_size);
            align.copy_from_slice(&data.vertices);
            self.context.device.unmap_memory(v_staging.memory);
        }

        let vertex_buffer = self.context.create_buffer(
            vertex_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(
            &self.context,
            self.sync.command_pool,
            v_staging.buffer,
            vertex_buffer.buffer,
            vertex_size,
        )?;

        // --- 2. インデックスバッファの作成と転送 ---
        let index_size = (data.indices.len() * size_of::<u32>()) as vk::DeviceSize;

        let i_staging = self.context.create_buffer(
            index_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = self
                .context
                .device
                .map_memory(i_staging.memory, 0, index_size, vk::MemoryMapFlags::empty())
                .unwrap();
            let mut align = ash::util::Align::new(data_ptr, align_of::<u32>() as u64, index_size);
            align.copy_from_slice(&data.indices);
            self.context.device.unmap_memory(i_staging.memory);
        }

        let index_buffer = self
            .context
            .create_buffer(
                index_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?
            .with_index_type(vk::IndexType::UINT32);

        copy_buffer(
            &self.context,
            self.sync.command_pool,
            i_staging.buffer,
            index_buffer.buffer,
            index_size,
        )?;

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
    ///
    /// # Errors
    /// フレームの取得、描画コマンドの記録、または送信に失敗した場合に `EngineError` を返します。
    pub fn draw_frame(&self, snapshot: &RenderSnapshot) -> Result<(), EngineError> {
        tracing::trace!(
            "描画リクエスト: {} 個のインスタンス",
            snapshot.instances.len()
        );
        info!("描画対象のインスタンス数: {}", snapshot.instances.len());

        // 1. フレームの開始（エラーが起きたら `?` で即座に返す）
        let active_frame = match self.begin_frame()? {
            Some(frame) => frame,
            None => {
                tracing::warn!("begin_frame が None を返したため、描画をスキップしました");
                return Ok(());
            }
        };

        // 2. コマンド記録フェーズ
        let recorder = &active_frame.recorder;

        let clear_values = [
            // R: 0.0, G: 0.0, B: 1.0, A: 1.0 で真っ青に設定
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 1.0, 1.0],
                },
            },
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
                extent: self.swapchain_target.extent,
            })
            .clear_values(&clear_values);

        recorder.begin_render_pass(&render_pass_begin_info);
        recorder.bind_pipeline(self.pipeline.pipeline);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swapchain_target.extent.width as f32,
            height: self.swapchain_target.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        recorder.set_viewport(0, &[viewport]);

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_target.extent,
        };
        recorder.set_scissor(0, &[scissor]);

        // カメラ行列の計算
        // カメラ行列の計算
        let aspect = self.swapchain_target.extent.width as f32 / self.swapchain_target.extent.height as f32;
        let projection = Matrix4::new_perspective(aspect, std::f32::consts::FRAC_PI_4, 0.1, 1000.0);

        let mut vulkan_clip = Matrix4::identity();
        vulkan_clip[(1, 1)] = -1.0;
        vulkan_clip[(2, 2)] = 0.5;
        vulkan_clip[(2, 3)] = 0.5;

        let view_proj = vulkan_clip * projection * snapshot.view_matrix;

        unsafe {
            let data_ptr = self.context.device.map_memory(
                self.global_ubo_buffer.memory,
                0,
                size_of::<GlobalUbo>() as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            ).unwrap();

            let view_proj_array: [[f32; 4]; 4] = view_proj.into();

            let ubo = GlobalUbo {
                view_proj: view_proj_array,
                camera_pos: [0.0, 0.0, 0.0],
                _padding: 0.0,
            };
            std::ptr::copy_nonoverlapping(&ubo, data_ptr as *mut GlobalUbo, 1);

            self.context.device.unmap_memory(self.global_ubo_buffer.memory);
        }

        unsafe {
            self.context.device.cmd_bind_descriptor_sets(
                recorder.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0, // first_set
                &[self.global_descriptor_set],
                &[], // dynamic_offsets
            );
        }

        for (i, instance) in snapshot.instances.iter().enumerate() {

            let model = instance.transform;

            if i == 0 {
                info!("最終MVP行列:\n{:.2}", model);
            }


            let model_slice = model.as_slice();
            let bytes = unsafe {
                std::slice::from_raw_parts(model_slice.as_ptr() as *const u8, size_of_val(model_slice))
            };

            recorder.push_constants(self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, bytes);

            let mesh_index = instance.mesh_id.0 as usize;
            if let Some(gpu_mesh) = self.meshes.get(mesh_index) {
                recorder.bind_vertex_buffer(&gpu_mesh.vertex_buffer);
                recorder.bind_index_buffer(&gpu_mesh.index_buffer)?;
                recorder.draw_indexed(gpu_mesh.index_count, 1, 0, 0, 0);
            }
        }

        recorder.end_render_pass();

        self.end_frame(active_frame)?;

        Ok(())
    }

    /// ========================================================
    /// フレームの開始（コールドパス）
    /// ========================================================
    /// 描画するフレームの準備を行います。
    ///
    /// # Returns
    /// - `Ok(Some(ActiveFrame))`: 描画の準備が完了し、コマンドの記録が可能な状態。
    /// - `Ok(None)`: ウィンドウサイズが変更された等で、画面の再構築が必要な場合。
    ///
    /// # Errors
    /// Vulkan内部での待機やメモリ確保に失敗した場合に `EngineError` を返します。
    pub fn begin_frame(&self) -> Result<Option<ActiveFrame<'_>>, EngineError> {
        let frame = self.sync.current_frame.get();

        unsafe {
            self.context
                .device
                .wait_for_fences(&[self.sync.in_flight_fences[frame]], true, u64::MAX)
                .map_err(|e| EngineError::Legacy(format!("Fenceの待機に失敗: {:?}", e)))?;
        }

        let acquire_result = unsafe {
            self.swapchain_target.loader.acquire_next_image(
                self.swapchain_target.swapchain,
                u64::MAX,
                self.sync.image_available_semaphores[frame],
                vk::Fence::null(),
            )
        };

        let image_index = match acquire_result {
            Ok((index, _)) => index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Ok(None),
            Err(e) => return Err(EngineError::Legacy(format!("画像の取得に失敗: {:?}", e))),
        };

        unsafe {
            self.context
                .device
                .reset_fences(&[self.sync.in_flight_fences[frame]])
                .map_err(|e| EngineError::Legacy(format!("Fenceのリセットに失敗: {:?}", e)))?;
        }

        let command_buffer = self.sync.command_buffers[frame];

        unsafe {
            self.context
                .device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| {
                    EngineError::Legacy(format!("コマンドバッファのリセットに失敗: {:?}", e))
                })?;

            let begin_info = vk::CommandBufferBeginInfo::default();
            self.context
                .device
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| {
                    EngineError::Legacy(format!("コマンドバッファの記録開始に失敗: {:?}", e))
                })?;
        }

        let recorder = CommandRecorder::new(&self.context.device, command_buffer);

        Ok(Some(ActiveFrame {
            recorder,
            image_index,
            frame_index: frame,
        }))
    }

    /// ========================================================
    /// フレームの終了と送信（コールドパス）
    /// ========================================================
    /// 記録したコマンドをGPUのキューに送信し、画面に表示します。
    ///
    /// # Errors
    /// GPUへのキュー送信、または画面への表示要求に失敗した場合に `EngineError` を返します。
    pub fn end_frame(&self, active_frame: ActiveFrame) -> Result<(), EngineError> {
        unsafe {
            self.context
                .device
                .end_command_buffer(active_frame.recorder.command_buffer)
                .map_err(|e| {
                    EngineError::Legacy(format!("コマンドバッファの終了に失敗: {:?}", e))
                })?;
        }

        let wait_semaphores = [self.sync.image_available_semaphores[active_frame.frame_index]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers_submit = [active_frame.recorder.command_buffer];
        let signal_semaphores =
            [self.sync.render_finished_semaphores[active_frame.image_index as usize]];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers_submit)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context
                .device
                .queue_submit(
                    self.context.graphics_queue,
                    &[submit_info],
                    self.sync.in_flight_fences[active_frame.frame_index],
                )
                .map_err(|e| EngineError::Legacy(format!("キューの送信に失敗: {:?}", e)))?;
        }

        let swapchains = [self.swapchain_target.swapchain];
        let image_indices = [active_frame.image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            let _ = self
                .swapchain_target
                .loader
                .queue_present(self.context.graphics_queue, &present_info);
        }

        self.sync
            .current_frame
            .set((active_frame.frame_index + 1) % sync::MAX_FRAMES_IN_FLIGHT);

        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();
            self.meshes.clear();

            self.sync.destroy(&self.context.device);
            self.pipeline.destroy(&self.context.device);
            self.swapchain_target.destroy(&self.context.device);

            self.context.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.context.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            info!("VulkanRenderer child objects destroyed cleanly.");
        }
    }
}
