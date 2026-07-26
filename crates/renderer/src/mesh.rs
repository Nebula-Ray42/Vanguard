use ash::vk;
use render_api::Vertex;
use std::mem::offset_of;

/// 頂点データのメモリ上のストライド（歩幅）を定義する
pub fn get_vertex_binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::default()
        .binding(0)
        // Self ではなく Vertex を指定
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
            // マジックナンバーを排除し、コンパイラに計算させる
            offset: offset_of!(Vertex, position) as u32,
        },
        // 1: Color
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, color) as u32,
        },
        // 2: Normal
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, normal) as u32,
        },
    ]
}