// crates/renderer/src/sync.rs
use ash::{Device, vk};

pub struct SyncContext {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

impl SyncContext {
    pub fn new(device: &Device, queue_family_index: u32) -> Result<Self, String> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let command_pool = unsafe {
            device
                .create_command_pool(&pool_info, None)
                .map_err(|e| format!("CommandPool生成失敗: {}", e))?
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| format!("CommandBuffer確保失敗: {}", e))?[0]
        };

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available_semaphore = unsafe {
            device
                .create_semaphore(&sem_info, None)
                .map_err(|e| format!("Sem失敗: {}", e))?
        };
        let render_finished_semaphore = unsafe {
            device
                .create_semaphore(&sem_info, None)
                .map_err(|e| format!("Sem失敗: {}", e))?
        };
        let in_flight_fence = unsafe {
            device
                .create_fence(&fence_info, None)
                .map_err(|e| format!("Fence失敗: {}", e))?
        };

        Ok(Self {
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        })
    }

    pub unsafe fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.render_finished_semaphore, None);
            device.destroy_fence(self.in_flight_fence, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
