#pragma once

#include <vulkan/vulkan.h>

#include <expected>

#include "../core/vulkan_context.hpp"

#include "engine_error.hpp"

namespace vanguard::render {

    [[nodiscard]] std::expected<void, EngineError> copy_buffer(
        const VulkanContext& context,
        VkCommandPool command_pool,
        VkBuffer src_buffer,
        VkBuffer dst_buffer,
        VkDeviceSize size);

} // namespace rey_engine::render
