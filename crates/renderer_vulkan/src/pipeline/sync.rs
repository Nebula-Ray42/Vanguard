use ash::{Device, vk};
use std::cell::Cell;

// ※ パスは環境に合わせて調整してください
use render_api::error_pass::engine_error::EngineError;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// 描画の同期（セマフォ・フェンス）とコマンドバッファを管理するコンテキスト
///
/// GPUとCPUの実行タイミングを制御し、フレームの重なりや描画の破綻を防ぐ役割を担います。
pub struct SyncContext {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub current_frame: Cell<usize>,
}

impl SyncContext {
    /// 同期オブジェクトとコマンドプールを初期化します。
    ///
    /// # Errors
    /// セマフォ、フェンス、コマンドプール、またはコマンドバッファの生成に失敗した場合に `EngineError` を返します。
    pub fn new(
        device: &Device,
        queue_family_index: u32,
        image_count: u32,
    ) -> Result<Self, EngineError> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        // 要素数が確定しているため、with_capacity でメモリの再確保を防止
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            // FFI呼び出しのみを unsafe にし、エラー時は安全に中断
            let sem = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        EngineError::Legacy(format!("image_available セマフォの作成に失敗: {}", e))
                    })?
            };
            let fence = unsafe {
                device.create_fence(&fence_info, None).map_err(|e| {
                    EngineError::Legacy(format!("in_flight フェンスの作成に失敗: {}", e))
                })?
            };

            image_available_semaphores.push(sem);
            in_flight_fences.push(fence);
        }

        let mut render_finished_semaphores = Vec::with_capacity(image_count as usize);
        for _ in 0..image_count {
            let sem = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        EngineError::Legacy(format!("render_finished セマフォの作成に失敗: {}", e))
                    })?
            };
            render_finished_semaphores.push(sem);
        }

        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device
                .create_command_pool(&pool_info, None)
                .map_err(|e| EngineError::Legacy(format!("コマンドプールの作成に失敗: {}", e)))?
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

        let command_buffers = unsafe {
            device.allocate_command_buffers(&alloc_info).map_err(|e| {
                EngineError::Legacy(format!("コマンドバッファの割り当てに失敗: {}", e))
            })?
        };

        Ok(Self {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            command_pool,
            command_buffers,
            current_frame: Cell::new(0),
        })
    }

    /// 同期オブジェクトとコマンドプールを破棄します。
    ///
    /// # Safety
    /// この関数はGPUが完全に待機状態（Idle）である時にのみ呼び出さなければなりません。
    pub unsafe fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_command_pool(self.command_pool, None);
            for &sem in &self.image_available_semaphores {
                device.destroy_semaphore(sem, None);
            }
            for &sem in &self.render_finished_semaphores {
                device.destroy_semaphore(sem, None);
            }
            for &fence in &self.in_flight_fences {
                device.destroy_fence(fence, None);
            }
        }
    }
}
