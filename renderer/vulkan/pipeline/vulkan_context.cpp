#include "vulkan_context.h"
#include <vector>

#define VMA_IMPLEMENTATION
#include "glfw3.h"

namespace rey_engine::render {

[[nodiscard]] std::expected<VulkanContext, EngineError> create_vulkan_context(
    const char* app_name,
    void* window_handle
    )
{
    VulkanContext ctx{};

    // ============================================================================
    // 1. Instance の作成
    // ============================================================================
    VkApplicationInfo app_info{};
    app_info.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    app_info.pApplicationName = app_name;
    app_info.applicationVersion = VK_MAKE_API_VERSION(0, 1, 0, 0);
    app_info.pEngineName = "Rey Engine";
    app_info.engineVersion = VK_MAKE_API_VERSION(0, 1, 0, 0);
    app_info.apiVersion = VK_API_VERSION_1_3;

    VkInstanceCreateInfo create_info{};
    create_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    create_info.pApplicationInfo = &app_info;

    // Mac (Apple Silicon / MoltenVK) 環境で必須となる拡張機能
    std::vector<const char*> instance_extensions = {
        VK_KHR_SURFACE_EXTENSION_NAME,
        "VK_EXT_metal_surface", // Mac用
        VK_KHR_PORTABILITY_ENUMERATION_EXTENSION_NAME, // MoltenVK用
    };
    create_info.flags |= VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR;
    create_info.enabledExtensionCount = static_cast<uint32_t>(instance_extensions.size());
    create_info.ppEnabledExtensionNames = instance_extensions.data();

    std::vector<const char*> validation_layers = {
        "VK_LAYER_KHRONOS_validation",
    };
    create_info.enabledLayerCount = static_cast<uint32_t>(validation_layers.size());
    create_info.ppEnabledLayerNames = validation_layers.data();

    if (vkCreateInstance(&create_info, nullptr, &ctx.instance) != VK_SUCCESS) {
        return std::unexpected(EngineError{});
    }

    if (glfwCreateWindowSurface(ctx.instance, static_cast<GLFWwindow*>(window_handle), nullptr, &ctx.surface) != VK_SUCCESS) {
        ctx.destroy();
        return std::unexpected(EngineError{});
    }

    uint32_t device_count = 0;
    vkEnumeratePhysicalDevices(ctx.instance, &device_count, nullptr);
    if (device_count == 0) {
        ctx.destroy();
        return std::unexpected(EngineError{});
    }

    std::vector<VkPhysicalDevice> physical_devices(device_count);
    vkEnumeratePhysicalDevices(ctx.instance, &device_count, physical_devices.data());

    bool device_found = false;
    for (const auto& p_device : physical_devices) {
        uint32_t queue_family_count = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(p_device, &queue_family_count, nullptr);
        std::vector<VkQueueFamilyProperties> queue_families(queue_family_count);
        vkGetPhysicalDeviceQueueFamilyProperties(p_device, &queue_family_count, queue_families.data());

        for (uint32_t i = 0; i < queue_family_count; i++) {
            if (queue_families[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) {
                // Surfaceが有効な場合、そのキューが描画(Present)をサポートしているか確認
                VkBool32 present_support = false;
                if (ctx.surface != VK_NULL_HANDLE) {
                    vkGetPhysicalDeviceSurfaceSupportKHR(p_device, i, ctx.surface, &present_support);
                }

                if (ctx.surface == VK_NULL_HANDLE || present_support) {
                    ctx.physical_device = p_device;
                    ctx.graphics_queue_family_index = i;
                    device_found = true;
                    break;
                }
            }
        }
        if (device_found) break;
    }

    if (!device_found) {
        ctx.destroy();
        return std::unexpected(EngineError{});
    }

    float queue_priority = 1.0f;
    VkDeviceQueueCreateInfo queue_create_info{};
    queue_create_info.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    queue_create_info.queueFamilyIndex = ctx.graphics_queue_family_index;
    queue_create_info.queueCount = 1;
    queue_create_info.pQueuePriorities = &queue_priority;

    VkPhysicalDeviceFeatures device_features{};

    std::vector<const char*> device_extensions = {
        VK_KHR_SWAPCHAIN_EXTENSION_NAME
    };

    // Mac(MoltenVK)環境のための動的チェック
    uint32_t extension_count = 0;
    vkEnumerateDeviceExtensionProperties(ctx.physical_device, nullptr, &extension_count, nullptr);
    std::vector<VkExtensionProperties> available_extensions(extension_count);
    vkEnumerateDeviceExtensionProperties(ctx.physical_device, nullptr, &extension_count, available_extensions.data());

    for (const auto& ext : available_extensions) {
        if (std::string_view(ext.extensionName) == "VK_KHR_portability_subset") {
            device_extensions.push_back("VK_KHR_portability_subset");
            break;
        }
    }

    VkDeviceCreateInfo device_create_info{};
    device_create_info.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    device_create_info.queueCreateInfoCount = 1;
    device_create_info.pQueueCreateInfos = &queue_create_info;
    device_create_info.pEnabledFeatures = &device_features;
    device_create_info.enabledExtensionCount = static_cast<uint32_t>(device_extensions.size());
    device_create_info.ppEnabledExtensionNames = device_extensions.data();

    // 早期リターンで Always-Valid を担保する
    if (vkCreateDevice(ctx.physical_device, &device_create_info, nullptr, &ctx.device) != VK_SUCCESS) {
        ctx.destroy();
        return std::unexpected(EngineError{});
    }

    vkGetDeviceQueue(ctx.device, ctx.graphics_queue_family_index, 0, &ctx.graphics_queue);

    VmaAllocatorCreateInfo allocator_info{};
    allocator_info.physicalDevice = ctx.physical_device;
    allocator_info.device = ctx.device;
    allocator_info.instance = ctx.instance;
    allocator_info.vulkanApiVersion = VK_API_VERSION_1_3;

    if (vmaCreateAllocator(&allocator_info, &ctx.allocator) != VK_SUCCESS) {
        ctx.destroy();
        return std::unexpected(EngineError{});
    }

    return ctx;
}

} // namespace rey_engine::render
