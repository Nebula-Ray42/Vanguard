// swapchain_target.h
#pragma once

#include <vulkan/vulkan.h>
#include <vector>
#include <ranges>
#include <expected>

#include "../pipeline/vulkan_context.h"
#include "engine_error.h"

namespace rey_engine::render {

    // ============================================================================
    // スワップチェーンとその関連リソースを保持する純粋なデータコンテナ
    // ============================================================================
    struct SwapchainTarget {
        VkSwapchainKHR swapchain{VK_NULL_HANDLE};
        VkFormat format{VK_FORMAT_UNDEFINED};
        VkExtent2D extent{.width = 0, .height = 0};

        // スワップチェーン画像ごとに並行して伸長する配列 (SoA)
        // images[i] / image_views[i] / framebuffers[i] は同一インデックスで対応する
        std::vector<VkImage> images;
        std::vector<VkImageView> image_views;
        std::vector<VkFramebuffer> framebuffers;

        VkFormat depth_format{VK_FORMAT_UNDEFINED};
        VkImage depth_image{VK_NULL_HANDLE};
        VkDeviceMemory depth_image_memory{VK_NULL_HANDLE};
        VkImageView depth_image_view{VK_NULL_HANDLE};

        VkRenderPass render_pass{VK_NULL_HANDLE};

        [[nodiscard]] auto image_resources() const noexcept {
            return std::views::zip(images, image_views, framebuffers);
        }

        // リソースの明示的な破棄
        void destroy(VkDevice device) const noexcept;
    };

    // ============================================================================
    // ステートレスな構築関数
    // ============================================================================
    [[nodiscard]] std::expected<SwapchainTarget, EngineError> create_swapchain_target(
        const VulkanContext& context,
        uint32_t width,
        uint32_t height
    );

} // namespace rey_engine::render
