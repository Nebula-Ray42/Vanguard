#pragma once

#include <vulkan/vulkan.h>
#include <expected>

#include "engine_error.h"
#include "renderer_vulkan/pipeline/vulkan_context.h" // パスは適宜合わせてください

namespace rey_engine::render {

    /// バッファ間のデータ転送（One-Time Submit）を同期的に実行します。
    [[nodiscard]] std::expected<void, EngineError> copy_buffer(
        const VulkanContext& context,
        VkCommandPool command_pool,
        VkBuffer src_buffer,
        VkBuffer dst_buffer,
        VkDeviceSize size);

} // namespace rey_engine::render
