#pragma once
#pragma once

#include <vulkan/vulkan.hpp>
#include <glm/glm.hpp>
#include <array>
#include <cstddef>

// ==========================================
// DTO (描画用データ構造)
// ==========================================

// 頂点1つ分のデータ
struct Vertex {
    glm::vec3 position;
    glm::vec3 color;
    glm::vec3 normal;

    // 頂点データのメモリ上のストライド（歩幅）を定義する
    static vk::VertexInputBindingDescription get_binding_description() {
        return vk::VertexInputBindingDescription()
            .setBinding(0)
            .setStride(sizeof(Vertex))
            .setInputRate(vk::VertexInputRate::eVertex);
    }

    // 頂点データの構造をVulkanに教える
    static std::array<vk::VertexInputAttributeDescription, 3> get_attribute_descriptions() {
        return {
            // 0: Position (location, binding, format, offset の順)
            vk::VertexInputAttributeDescription(
                0,
                0,
                vk::Format::eR32G32B32Sfloat,
                offsetof(Vertex, position)
            ),
            // 1: Color
            vk::VertexInputAttributeDescription(
                1,
                0,
                vk::Format::eR32G32B32Sfloat,
                offsetof(Vertex, color)
            ),
            // 2: Normal
            vk::VertexInputAttributeDescription(
                2,
                0,
                vk::Format::eR32G32B32Sfloat,
                offsetof(Vertex, normal)
            )
        };
    }
};
