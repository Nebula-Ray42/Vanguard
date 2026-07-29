use ash::vk;
use anyhow::{Result, Context};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants {
    pub model: [f32; 16], // 4x4行列
}

pub struct PipelineLayoutSetup {
    pub layout: vk::PipelineLayout,
}

impl PipelineLayoutSetup {
    pub fn new(
        device: &ash::Device,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self> {
        
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX) 
            .offset(0)
            .size(size_of::<PushConstants>() as u32);

        let push_constant_ranges = [push_constant_range];

        let layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        
        let layout = unsafe {
            device.create_pipeline_layout(&layout_create_info, None)
                .context("Pipeline Layoutの作成に失敗しました")?
        };
        
        Ok(Self { layout })
    }
}
