use bytemuck::Pod;
use ash::vk;
use render_api::engine_error::EngineError;
use crate::pipeline::context::VulkanContext;

pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl AllocatedBuffer {
    pub fn update_data<T: Pod>(
        &self,
        context: &VulkanContext,
        data: &T,
    ) -> Result<(), EngineError> {

        let data_size = size_of::<T>() as vk::DeviceSize;

        if data_size > self.size {
            return Err(EngineError::Legacy("バッファサイズを超過しています".to_string()));
        }

        unsafe {
            let data_ptr = context.device.map_memory(
                self.memory,
                0,
                data_size,
                vk::MemoryMapFlags::empty(),
            ).map_err(|e| EngineError::Legacy(format!("メモリマップ失敗: {:?}", e)))?;

            let mut_ptr = data_ptr as *mut T;
            mut_ptr.write_unaligned(*data);

            context.device.unmap_memory(self.memory);
        }

        Ok(())
    }
}
