// src/vulkan/buffer.rs

use ash::util::Align;
use ash::vk;
use crate::vulkan::context::VulkanContext;
use render_api::engine_error::EngineError;
use crate::buffer::copy_buffer;

pub struct VulkanBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl VulkanBuffer {
    /// Staging Buffer を経由してデータを GPU の Device Local メモリに配置する
    pub fn create_device_local_buffer<T: bytemuck::Pod>(
        context: &VulkanContext,
        command_pool: vk::CommandPool,
        data: &[T],
        usage_flags: vk::BufferUsageFlags,
    ) -> Result<Self, EngineError> {

        let size = (size_of::<T>() * data.len()) as vk::DeviceSize;
        if size == 0 {
            return Err(EngineError::Legacy("データサイズが0のバッファは作成できません".to_string()));
        }

        let (staging_buffer, staging_memory) = Self::create_buffer(
            context,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let data_ptr = context
                .device
                .map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| EngineError::Legacy(format!("メモリマップ失敗: {:?}", e)))?;

            let mut align: Align<T> = Align::new(data_ptr, align_of::<T>() as u64, size);
            align.copy_from_slice(bytemuck::cast_slice(data));

            context.device.unmap_memory(staging_memory);
        }

        let (device_buffer, device_memory) = Self::create_buffer(
            context,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | usage_flags,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(
                                     context,
                                     command_pool,
                                     staging_buffer,
                                     device_buffer,
                                     size,
        )?;

        unsafe {
            context.device.destroy_buffer(staging_buffer, None);
            context.device.free_memory(staging_memory, None);
        }

        Ok(Self {
            buffer: device_buffer,
            memory: device_memory,
            size,
        })
    }

    /// Vulkanバッファとメモリを確保する内部ヘルパー関数
    fn create_buffer(
        context: &VulkanContext,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), EngineError> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            context.device.create_buffer(&buffer_info, None)
                .map_err(|e| EngineError::Legacy(format!("バッファ作成失敗: {:?}", e)))?
        };

        let mem_requirements = unsafe { context.device.get_buffer_memory_requirements(buffer) };
        let memory_type_index = Self::find_memory_type(context, mem_requirements.memory_type_bits, properties)?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            context.device.allocate_memory(&alloc_info, None)
                .map_err(|e| EngineError::Legacy(format!("メモリ割り当て失敗: {:?}", e)))?
        };

        unsafe {
            context.device.bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| EngineError::Legacy(format!("メモリバインド失敗: {:?}", e)))?;
        }

        Ok((buffer, memory))
    }

    /// 必要なプロパティを満たすメモリタイプを探すヘルパー関数
    fn find_memory_type(
        context: &VulkanContext,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, EngineError> {
        let mem_properties = unsafe {
            context.instance.get_physical_device_memory_properties(context.physical_device)
        };

        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
            {
                return Ok(i);
            }
        }

        Err(EngineError::Legacy("適切なメモリタイプが見つかりません".to_string()))
    }
}
