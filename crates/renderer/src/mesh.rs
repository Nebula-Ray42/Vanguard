use ash::vk;
use render_api::Vertex;

pub fn get_vertex_binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(size_of::<Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX)
}

/// 頂点データの構造をVulkanに教える
pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
    [
        // 0: Position
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        // 1: Color
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12, // 4 bytes * 3
        },
        // 2: Normal <- NEW
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2, // Slang側の location と一致させる
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 24, // 12 (Position) + 12 (Color)
        },
    ]
}