use ash::{Device, vk};
use std::cell::Cell;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct SyncContext {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub current_frame: Cell<usize>,
}

impl SyncContext {
    // 引数に image_count を追加
    pub fn new(device: &Device, queue_family_index: u32, image_count: u32) -> Result<Self, String> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = vec![];
        let mut in_flight_fences = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_semaphores
                    .push(device.create_semaphore(&semaphore_info, None).unwrap());
                in_flight_fences.push(device.create_fence(&fence_info, None).unwrap());
            }
        }

        let mut render_finished_semaphores = vec![];
        for _ in 0..image_count {
            unsafe {
                render_finished_semaphores
                    .push(device.create_semaphore(&semaphore_info, None).unwrap());
            }
        }

        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe { device.create_command_pool(&pool_info, None).unwrap() };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info).unwrap() };

        Ok(Self {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            command_pool,
            command_buffers,
            current_frame: Cell::new(0),
        })
    }

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
