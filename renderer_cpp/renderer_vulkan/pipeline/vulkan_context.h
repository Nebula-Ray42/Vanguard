#pragma once

#include <vulkan/vulkan.h>
#include <expected>
#include <optional>

#include "include/engine_error.h"
#include "vk_mem_alloc.h"

namespace rey_engine::render {

// ============================================================================
// 1. GPUバッファコンテナ
// ============================================================================
struct GpuBuffer {
    VkBuffer buffer{VK_NULL_HANDLE};
    VmaAllocation allocation{VK_NULL_HANDLE};
    VkDeviceSize size{0};
    std::optional<VkIndexType> index_type{std::nullopt};

    // VMAの解放にはAllocatorが必要なため、引数としてContextを受け取ります。
    void destroy(const struct VulkanContext& context) noexcept;
};

// ============================================================================
// 2. Vulkan コアコンテキスト
// ============================================================================
struct VulkanContext {
    VkInstance instance{VK_NULL_HANDLE};
    VkSurfaceKHR surface{VK_NULL_HANDLE};
    VkPhysicalDevice physical_device{VK_NULL_HANDLE};
    VkDevice device{VK_NULL_HANDLE};
    VkQueue graphics_queue{VK_NULL_HANDLE};
    uint32_t graphics_queue_family_index{0};
    VmaAllocator allocator{VK_NULL_HANDLE};

    void destroy() noexcept {
        // デバイスより先にVMAアロケータを破棄する
        if (allocator != VK_NULL_HANDLE) {
            vmaDestroyAllocator(allocator);
            allocator = VK_NULL_HANDLE;
        }
        if (device != VK_NULL_HANDLE) {
            vkDestroyDevice(device, nullptr);
            device = VK_NULL_HANDLE;
        }
        if (instance != VK_NULL_HANDLE && surface != VK_NULL_HANDLE) {
            vkDestroySurfaceKHR(instance, surface, nullptr);
            surface = VK_NULL_HANDLE;
        }
        if (instance != VK_NULL_HANDLE) {
            vkDestroyInstance(instance, nullptr);
            instance = VK_NULL_HANDLE;
        }
    }
};

// GpuBuffer::destroy の実装 (VulkanContextの定義後に記述)
inline void GpuBuffer::destroy(const VulkanContext& context) noexcept {
    if (buffer != VK_NULL_HANDLE && allocation != VK_NULL_HANDLE) {
        vmaDestroyBuffer(context.allocator, buffer, allocation);
        buffer = VK_NULL_HANDLE;
        allocation = VK_NULL_HANDLE;
    }
}

// ============================================================================
// 3. ステートレスな操作関数群
// ============================================================================
[[nodiscard]] std::expected<VulkanContext, EngineError> create_vulkan_context(
    const char* app_name,
    VkSurfaceKHR surface
);

} // namespace rey_engine::render
