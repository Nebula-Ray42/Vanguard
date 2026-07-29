use bytemuck::{Pod, Zeroable};
use ash::vk;


#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlobalUbo {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}

pub fn create_global_ubo_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

    let bindings = [ubo_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(&bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .expect("Failed to create Global UBO Descriptor Set Layout!")
    }
}