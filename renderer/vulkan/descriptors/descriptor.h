#pragma once

#include <vulkan/vulkan.h>
#include <glm/glm.hpp>
#include <expected>

#include "vulkan/pipeline/vulkan_context.h"
#include "include/engine_error.h"

namespace rey_engine::render {

    struct alignas(16) GlobalUbo {
        glm::mat4 view_proj;
        glm::vec3 camera_pos;
        float padding;
    };

    [[nodiscard]] std::expected<VkDescriptorSetLayout, EngineError> create_global_ubo_layout(
        VkDevice device
    );

    [[nodiscard]] std::expected<VkDescriptorPool, EngineError> create_descriptor_pool(
        const VulkanContext& context
    );

    [[nodiscard]] std::expected<VkDescriptorSet, EngineError> create_descriptor_set(
        const VulkanContext& context,
        VkDescriptorPool pool,
        VkDescriptorSetLayout layout,
        VkBuffer ubo_buffer
    );

} // namespace rey_engine::render
