use ash::vk;
use anyhow::{Result, Context};

pub struct DescriptorSetup {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
}

impl DescriptorSetup {
    pub fn new(
        device: &ash::Device,
        buffer_info: vk::DescriptorBufferInfo,
    ) -> Result<Self> {

        let layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let bindings = [layout_binding];
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);
        
        let layout = unsafe {
            device.create_descriptor_set_layout(&layout_create_info, None)
                .context("設計図 (DescriptorSetLayout) の作成に失敗しました")?
        };

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)];

        let pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);
        
        let pool = unsafe {
            device.create_descriptor_pool(&pool_create_info, None)
                .context("部品箱 (DescriptorPool) の作成に失敗しました")?
        };
        
        let layouts = [layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        
        let sets = unsafe {
            device.allocate_descriptor_sets(&alloc_info)
                .context("接続ケーブル (DescriptorSet) の作成に失敗しました")?
        };
        let set = sets[0];

        let buffer_infos = [buffer_info];
        let write_descriptor_set = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos);

        let write_sets = [write_descriptor_set];

        unsafe {
            device.update_descriptor_sets(&write_sets, &[]);
        }

        Ok(Self { layout, pool, set })
    }
}
