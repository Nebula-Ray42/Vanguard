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

        // 1. バッファサイズの計算と早期リターン（Always-Valid）
        let size = (size_of::<T>() * data.len()) as vk::DeviceSize;
        if size == 0 {
            return Err(EngineError::Legacy("データサイズが0のバッファは作成できません".to_string()));
        }

        // 2. CPU側の一時バッファ（Staging Buffer）を作成する
        let (staging_buffer, staging_memory) = Self::create_buffer(
            context,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // 3. Staging Bufferにデータを書き込む (Memory Mapping)
        unsafe {
            let data_ptr = context
                .device
                .map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| EngineError::Legacy(format!("メモリマップ失敗: {:?}", e)))?;

            // bytemuck等を使ってスライスをコピー
            let mut align: Align<T> = Align::new(data_ptr, align_of::<T>() as u64, size);
            align.copy_from_slice(bytemuck::cast_slice(data));

            context.device.unmap_memory(staging_memory);
        }

        // 4. GPU側の本命バッファ（Device Local Buffer）を作成する
        let (device_buffer, device_memory) = Self::create_buffer(
            context,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | usage_flags,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // 5. さっきの関数を使って転送コマンドを発行・同期
        copy_buffer( // ※適切なパスに変更してください
                                     context,
                                     command_pool,
                                     staging_buffer,
                                     device_buffer,
                                     size,
        )?;

        // 6. 用済みの Staging Buffer の破棄
        unsafe {
            context.device.destroy_buffer(staging_buffer, None);
            context.device.free_memory(staging_memory, None);
        }

        // 7. 常に正しい状態の Device Local Buffer を返す
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
        // ※ contextから物理デバイスのメモリプロパティを取得する関数があると想定
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