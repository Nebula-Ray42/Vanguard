#pragma once

#include <vulkan/vulkan.h>
#include <expected>

#include "engine_error.h"
#include "vulkan/pipeline/vulkan_context.h"

namespace rey_engine::render {

    [[nodiscard]] std::expected<void, EngineError> copy_buffer(
        const VulkanContext& context,
        VkCommandPool command_pool,
        VkBuffer src_buffer,
        VkBuffer dst_buffer,
        VkDeviceSize size);

} // namespace rey_engine::render
