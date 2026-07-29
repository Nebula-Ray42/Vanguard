use ash::vk;
use render_api::engine_error::EngineError;
use crate::descriptors::layout::GlobalUbo;
use crate::vulkan::context::VulkanContext;

pub fn create_descriptor_pool(context: &VulkanContext) -> Result<vk::DescriptorPool, EngineError> {
    let pool_size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1);

    let pool_sizes = [pool_size];

    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(1);

    unsafe {
        let pool = context.device.create_descriptor_pool(&pool_info, None)
            .map_err(|e| EngineError::Legacy(format!("Descriptor Pool作成失敗: {:?}", e)))?;
        Ok(pool)
    }
}

pub fn create_descriptor_set(
    context: &VulkanContext,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    ubo_buffer: vk::Buffer,
) -> Result<vk::DescriptorSet, EngineError> {
    let layouts = [layout];
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    
    let descriptor_sets = unsafe {
        context.device.allocate_descriptor_sets(&alloc_info)
            .map_err(|e| EngineError::Legacy(format!("Descriptor Set確保失敗: {:?}", e)))?
    };
    let descriptor_set = descriptor_sets[0];

    let buffer_info = vk::DescriptorBufferInfo::default()
        .buffer(ubo_buffer)
        .offset(0)
        .range(size_of::<GlobalUbo>() as vk::DeviceSize);

    let buffer_infos = [buffer_info];
    
    let descriptor_write = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(&buffer_infos);

    unsafe {
        context.device.update_descriptor_sets(&[descriptor_write], &[]);
    }

    Ok(descriptor_set)
}