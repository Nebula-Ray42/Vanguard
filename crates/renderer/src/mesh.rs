use ash::vk;
use render_api::Vertex;

pub fn get_vertex_binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(size_of::<Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX)
}

/// 頂点データの構造をVulkanに教える
pub fn get_vertex_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
    [
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<[f32; 3]>() as u32),
    ]
}