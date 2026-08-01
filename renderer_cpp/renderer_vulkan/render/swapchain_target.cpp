// swapchain_target.cpp
#include "swapchain_target.h"

#include <algorithm>
#include <array>
#include <limits>
#include <ranges>
#include <span>

#include "engine_error.h"

namespace rey_engine::render {

namespace {

    // ------------------------------------------------------------------
    // 選択ロジック (副作用なしの純粋関数)
    // ------------------------------------------------------------------

    VkSurfaceFormatKHR select_surface_format(std::span<const VkSurfaceFormatKHR> formats) {
        auto it = std::ranges::find_if(formats, [](const VkSurfaceFormatKHR& f) {
            return f.format == VK_FORMAT_B8G8R8A8_SRGB
                && f.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
        });
        return it != formats.end() ? *it : formats.front();
    }

    VkPresentModeKHR select_present_mode(std::span<const VkPresentModeKHR> modes) {
        // std::ranges::contains は C++23 で追加された新しいrangeアルゴリズム
        return std::ranges::contains(modes, VK_PRESENT_MODE_MAILBOX_KHR)
            ? VK_PRESENT_MODE_MAILBOX_KHR
            : VK_PRESENT_MODE_FIFO_KHR;
    }

    VkExtent2D compute_extent(const VkSurfaceCapabilitiesKHR& caps, uint32_t width, uint32_t height) {
        if (caps.currentExtent.width != std::numeric_limits<uint32_t>::max()) {
            return caps.currentExtent;
        }
        return VkExtent2D{
            .width = std::clamp(width, caps.minImageExtent.width, caps.maxImageExtent.width),
            .height = std::clamp(height, caps.minImageExtent.height, caps.maxImageExtent.height),
        };
    }

    uint32_t compute_image_count(const VkSurfaceCapabilitiesKHR& caps) {
        const uint32_t wanted = caps.minImageCount + 1;
        return caps.maxImageCount > 0 ? std::min(wanted, caps.maxImageCount) : wanted;
    }

    // ------------------------------------------------------------------
    // メモリタイプ探索
    // 本来は VulkanContext 側 (Rust版の find_memory_type 相当) に
    // 置くのが望ましいが、ここでは自己完結させるためローカルに定義
    // ------------------------------------------------------------------

    std::expected<uint32_t, EngineError> find_memory_type(
        const VkPhysicalDeviceMemoryProperties& mem_props,
        uint32_t type_filter,
        VkMemoryPropertyFlags required_props)
    {
        for (uint32_t i = 0; i < mem_props.memoryTypeCount; ++i) {
            const bool type_ok = (type_filter & (1u << i)) != 0;
            const bool prop_ok = (mem_props.memoryTypes[i].propertyFlags & required_props) == required_props;
            if (type_ok && prop_ok) {
                return i;
            }
        }
        return std::unexpected(EngineError{SwapchainError{
            swapchain_error::CreateDepthResource{"適合するメモリタイプが見つかりません"}}});
    }

    // ------------------------------------------------------------------
    // 各生成ステップの中間結果
    // ------------------------------------------------------------------

    struct SwapchainCreateResult {
        VkSwapchainKHR swapchain;
        VkFormat format;
        VkExtent2D extent;
    };

    struct DepthResources {
        VkFormat format;
        VkImage image;
        VkDeviceMemory memory;
        VkImageView view;
    };

    // ------------------------------------------------------------------
    // 1. Swapchain本体
    // ------------------------------------------------------------------

    std::expected<SwapchainCreateResult, EngineError> create_swapchain_internal(
        const VulkanContext& context, uint32_t width, uint32_t height)
    {
        VkSurfaceCapabilitiesKHR caps{};
        if (auto res = vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
                context.physical_device, context.surface, &caps);
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::QueryCapabilities{res}}});
        }

        uint32_t format_count = 0;
        if (auto res = vkGetPhysicalDeviceSurfaceFormatsKHR(
                context.physical_device, context.surface, &format_count, nullptr);
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::QueryFormats{res}}});
        }
        if (format_count == 0) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::NoFormatsAvailable{}}});
        }
        std::vector<VkSurfaceFormatKHR> formats(format_count);
        if (auto res = vkGetPhysicalDeviceSurfaceFormatsKHR(
                context.physical_device, context.surface, &format_count, formats.data());
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::QueryFormats{res}}});
        }

        uint32_t mode_count = 0;
        if (auto res = vkGetPhysicalDeviceSurfacePresentModesKHR(
                context.physical_device, context.surface, &mode_count, nullptr);
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::QueryPresentModes{res}}});
        }
        std::vector<VkPresentModeKHR> present_modes(mode_count);
        if (auto res = vkGetPhysicalDeviceSurfacePresentModesKHR(
                context.physical_device, context.surface, &mode_count, present_modes.data());
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::QueryPresentModes{res}}});
        }

        const VkSurfaceFormatKHR surface_format = select_surface_format(formats);
        const VkPresentModeKHR present_mode = select_present_mode(present_modes);
        const VkExtent2D extent = compute_extent(caps, width, height);
        const uint32_t image_count = compute_image_count(caps);

        const VkSwapchainCreateInfoKHR create_info{
            .sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            .surface = context.surface,
            .minImageCount = image_count,
            .imageFormat = surface_format.format,
            .imageColorSpace = surface_format.colorSpace,
            .imageExtent = extent,
            .imageArrayLayers = 1,
            .imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            .imageSharingMode = VK_SHARING_MODE_EXCLUSIVE,
            .preTransform = caps.currentTransform,
            .compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            .presentMode = present_mode,
            .clipped = VK_TRUE,
            .oldSwapchain = VK_NULL_HANDLE,
        };

        VkSwapchainKHR swapchain{};
        if (auto res = vkCreateSwapchainKHR(context.device, &create_info, nullptr, &swapchain);
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::CreateSwapchain{res}}});
        }

        return SwapchainCreateResult{
            .swapchain = swapchain,
            .format = surface_format.format,
            .extent = extent,
        };
    }

    // ------------------------------------------------------------------
    // 2. Swapchain画像の取得
    // ------------------------------------------------------------------

    std::expected<std::vector<VkImage>, EngineError> get_swapchain_images(
        VkDevice device, VkSwapchainKHR swapchain)
    {
        uint32_t count = 0;
        if (auto res = vkGetSwapchainImagesKHR(device, swapchain, &count, nullptr);
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::GetImages{res}}});
        }
        std::vector<VkImage> images(count);
        vkGetSwapchainImagesKHR(device, swapchain, &count, images.data());
        return images;
    }

    // ------------------------------------------------------------------
    // 3. ImageView群 (SoA: images[i] -> image_views[i])
    // ------------------------------------------------------------------

    std::expected<std::vector<VkImageView>, EngineError> create_image_views(
        VkDevice device, VkFormat format, std::span<const VkImage> images)
    {
        std::vector<VkImageView> views;
        views.reserve(images.size());

        for (const VkImage image : images) {
            const VkImageViewCreateInfo info{
                .sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                .image = image,
                .viewType = VK_IMAGE_VIEW_TYPE_2D,
                .format = format,
                .subresourceRange = {
                    .aspectMask = VK_IMAGE_ASPECT_COLOR_BIT,
                    .baseMipLevel = 0,
                    .levelCount = 1,
                    .baseArrayLayer = 0,
                    .layerCount = 1,
                },
            };

            VkImageView view{};
            if (vkCreateImageView(device, &info, nullptr, &view) != VK_SUCCESS) {
                return std::unexpected(EngineError{SwapchainError{
                    swapchain_error::CreateImageView{"ImageView生成失敗"}}});
            }
            views.push_back(view);
        }
        return views;
    }

    // ------------------------------------------------------------------
    // 4. Depthリソース (依存関係が一本道なので and_then で合成)
    // ------------------------------------------------------------------

    std::expected<DepthResources, EngineError> create_depth_resources(
        const VulkanContext& context, VkExtent2D extent)
    {
        constexpr VkFormat depth_format = VK_FORMAT_D32_SFLOAT;

        const VkImageCreateInfo image_info{
            .sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            .imageType = VK_IMAGE_TYPE_2D,
            .format = depth_format,
            .extent = {.width = extent.width, .height = extent.height, .depth = 1},
            .mipLevels = 1,
            .arrayLayers = 1,
            .samples = VK_SAMPLE_COUNT_1_BIT,
            .tiling = VK_IMAGE_TILING_OPTIMAL,
            .usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
            .sharingMode = VK_SHARING_MODE_EXCLUSIVE,
            .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
        };

        VkImage depth_image{};
        if (vkCreateImage(context.device, &image_info, nullptr, &depth_image) != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{
                swapchain_error::CreateDepthResource{"Depth Image生成失敗"}}});
        }

        VkMemoryRequirements mem_reqs{};
        vkGetImageMemoryRequirements(context.device, depth_image, &mem_reqs);

        VkPhysicalDeviceMemoryProperties mem_props{};
        vkGetPhysicalDeviceMemoryProperties(context.physical_device, &mem_props);

        return find_memory_type(mem_props, mem_reqs.memoryTypeBits, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)
            .and_then([&](uint32_t mem_type_index) -> std::expected<DepthResources, EngineError> {
                const VkMemoryAllocateInfo alloc_info{
                    .sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                    .allocationSize = mem_reqs.size,
                    .memoryTypeIndex = mem_type_index,
                };

                VkDeviceMemory depth_memory{};
                if (vkAllocateMemory(context.device, &alloc_info, nullptr, &depth_memory) != VK_SUCCESS) {
                    return std::unexpected(EngineError{SwapchainError{
                        swapchain_error::CreateDepthResource{"Depth メモリ確保失敗"}}});
                }
                if (vkBindImageMemory(context.device, depth_image, depth_memory, 0) != VK_SUCCESS) {
                    return std::unexpected(EngineError{SwapchainError{
                        swapchain_error::CreateDepthResource{"Depth メモリバインド失敗"}}});
                }

                const VkImageViewCreateInfo view_info{
                    .sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                    .image = depth_image,
                    .viewType = VK_IMAGE_VIEW_TYPE_2D,
                    .format = depth_format,
                    .subresourceRange = {
                        .aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT,
                        .baseMipLevel = 0,
                        .levelCount = 1,
                        .baseArrayLayer = 0,
                        .layerCount = 1,
                    },
                };

                VkImageView depth_view{};
                if (vkCreateImageView(context.device, &view_info, nullptr, &depth_view) != VK_SUCCESS) {
                    return std::unexpected(EngineError{SwapchainError{
                        swapchain_error::CreateDepthResource{"Depth ImageView生成失敗"}}});
                }

                return DepthResources{
                    .format = depth_format,
                    .image = depth_image,
                    .memory = depth_memory,
                    .view = depth_view,
                };
            });
    }

    // ------------------------------------------------------------------
    // 5. RenderPass
    // ------------------------------------------------------------------

    std::expected<VkRenderPass, EngineError> create_render_pass(
        VkDevice device, VkFormat color_format, VkFormat depth_format)
    {
        const std::array attachments{
            VkAttachmentDescription{
                .format = color_format,
                .samples = VK_SAMPLE_COUNT_1_BIT,
                .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
                .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
                .stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                .stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE,
                .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
                .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
            },
            VkAttachmentDescription{
                .format = depth_format,
                .samples = VK_SAMPLE_COUNT_1_BIT,
                .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
                .storeOp = VK_ATTACHMENT_STORE_OP_DONT_CARE,
                .stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                .stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE,
                .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
                .finalLayout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
        };

        constexpr VkAttachmentReference color_ref{
            .attachment = 0,
            .layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };
        constexpr VkAttachmentReference depth_ref{
            .attachment = 1,
            .layout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        const VkSubpassDescription subpass{
            .pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS,
            .colorAttachmentCount = 1,
            .pColorAttachments = &color_ref,
            .pDepthStencilAttachment = &depth_ref,
        };

        constexpr VkPipelineStageFlags sync_stage_mask =
            VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
            | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;

        const VkSubpassDependency dependency{
            .srcSubpass = VK_SUBPASS_EXTERNAL,
            .dstSubpass = 0,
            .srcStageMask = sync_stage_mask,
            .dstStageMask = sync_stage_mask,
            .srcAccessMask = 0,
            .dstAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT
                | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT,
        };

        const VkRenderPassCreateInfo render_pass_info{
            .sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            .attachmentCount = static_cast<uint32_t>(attachments.size()),
            .pAttachments = attachments.data(),
            .subpassCount = 1,
            .pSubpasses = &subpass,
            .dependencyCount = 1,
            .pDependencies = &dependency,
        };

        VkRenderPass render_pass{};
        if (auto res = vkCreateRenderPass(device, &render_pass_info, nullptr, &render_pass);
            res != VK_SUCCESS) {
            return std::unexpected(EngineError{SwapchainError{swapchain_error::CreateRenderPass{res}}});
        }
        return render_pass;
    }

    // ------------------------------------------------------------------
    // 6. Framebuffer群 (SoA: image_views[i] -> framebuffers[i])
    // ------------------------------------------------------------------

    std::expected<std::vector<VkFramebuffer>, EngineError> create_framebuffers(
        VkDevice device, VkRenderPass render_pass, VkExtent2D extent,
        std::span<const VkImageView> image_views, VkImageView depth_view)
    {
        std::vector<VkFramebuffer> framebuffers;
        framebuffers.reserve(image_views.size());

        for (const VkImageView view : image_views) {
            const std::array fb_attachments{view, depth_view};
            const VkFramebufferCreateInfo fb_info{
                .sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                .renderPass = render_pass,
                .attachmentCount = static_cast<uint32_t>(fb_attachments.size()),
                .pAttachments = fb_attachments.data(),
                .width = extent.width,
                .height = extent.height,
                .layers = 1,
            };

            VkFramebuffer fb{};
            if (vkCreateFramebuffer(device, &fb_info, nullptr, &fb) != VK_SUCCESS) {
                return std::unexpected(EngineError{SwapchainError{
                    swapchain_error::CreateFramebuffer{"Framebuffer生成失敗"}}});
            }
            framebuffers.push_back(fb);
        }
        return framebuffers;
    }

} // namespace anonymous

// ============================================================================
// メインの生成オーケストレーター
// 依存が直列に5段連なるため、Rustの `?` 相当として「失敗したら即return」を
// そのまま採用（タプルを持ち回すモナディック連鎖より読みやすく、コピーも発生しない）
// ============================================================================
std::expected<SwapchainTarget, EngineError> create_swapchain_target(
    const VulkanContext& context, uint32_t width, uint32_t height)
{
    auto sc = create_swapchain_internal(context, width, height);
    if (!sc) { return std::unexpected(sc.error()); }

    auto images = get_swapchain_images(context.device, sc->swapchain);
    if (!images) { return std::unexpected(images.error()); }

    auto views = create_image_views(context.device, sc->format, *images);
    if (!views) { return std::unexpected(views.error()); }

    auto depth = create_depth_resources(context, sc->extent);
    if (!depth) { return std::unexpected(depth.error()); }

    auto render_pass = create_render_pass(context.device, sc->format, depth->format);
    if (!render_pass) { return std::unexpected(render_pass.error()); }

    auto framebuffers = create_framebuffers(
        context.device, *render_pass, sc->extent, *views, depth->view);
    if (!framebuffers) { return std::unexpected(framebuffers.error()); }

    return SwapchainTarget{
        .swapchain = sc->swapchain,
        .format = sc->format,
        .extent = sc->extent,
        .images = std::move(*images),
        .image_views = std::move(*views),
        .framebuffers = std::move(*framebuffers),
        .depth_format = depth->format,
        .depth_image = depth->image,
        .depth_image_memory = depth->memory,
        .depth_image_view = depth->view,
        .render_pass = *render_pass,
    };
}

// ============================================================================
// 破棄処理
// ============================================================================
void SwapchainTarget::destroy(VkDevice device) const noexcept {
    // image_view と framebuffer は 1:1 対応するので zip で1ループにまとめる
    for (auto&& [view, fb] : std::views::zip(image_views, framebuffers)) {
        vkDestroyFramebuffer(device, fb, nullptr);
        vkDestroyImageView(device, view, nullptr);
    }
    vkDestroyRenderPass(device, render_pass, nullptr);
    vkDestroyImageView(device, depth_image_view, nullptr);
    vkDestroyImage(device, depth_image, nullptr);
    vkFreeMemory(device, depth_image_memory, nullptr);
    vkDestroySwapchainKHR(device, swapchain, nullptr);
}

} // namespace rey_engine::render
