#include "descriptor.h"
#include "vulkan/pipeline/vulkan_context.h"

namespace rey_engine::render {

    std::expected<VkDescriptorSetLayout, EngineError> create_global_ubo_layout(VkDevice device) {
        constexpr VkDescriptorSetLayoutBinding ubo_layout_binding{
            .binding = 0,
            .descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            .descriptorCount = 1,
            .stageFlags = VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_FRAGMENT_BIT,
        };

        const VkDescriptorSetLayoutCreateInfo layout_info{
            .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            .bindingCount = 1,
            .pBindings = &ubo_layout_binding,
        };

        VkDescriptorSetLayout layout{};
        if (vkCreateDescriptorSetLayout(device, &layout_info, nullptr, &layout) != VK_SUCCESS) {
            return std::unexpected(EngineError{LegacyError{"Global UBO Descriptor Set Layout の作成失敗"}});
        }

        return layout;
    }

    std::expected<VkDescriptorPool, EngineError> create_descriptor_pool(const VulkanContext& context) {
        constexpr VkDescriptorPoolSize pool_size{
            .type = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            .descriptorCount = 1,
        };

        const VkDescriptorPoolCreateInfo pool_info{
            .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            .maxSets = 1,
            .poolSizeCount = 1,
            .pPoolSizes = &pool_size,
        };

        VkDescriptorPool pool{};
        if (vkCreateDescriptorPool(context.device, &pool_info, nullptr, &pool) != VK_SUCCESS) {
            return std::unexpected(EngineError{LegacyError{"Descriptor Pool の作成失敗"}});
        }

        return pool;
    }

    std::expected<VkDescriptorSet, EngineError> create_descriptor_set(
        const VulkanContext& context,
        VkDescriptorPool pool,
        VkDescriptorSetLayout layout,
        VkBuffer ubo_buffer)
    {
        const VkDescriptorSetAllocateInfo alloc_info{
            .sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            .descriptorPool = pool,
            .descriptorSetCount = 1,
            .pSetLayouts = &layout,
        };

        VkDescriptorSet descriptor_set{};
        if (vkAllocateDescriptorSets(context.device, &alloc_info, &descriptor_set) != VK_SUCCESS) {
            return std::unexpected(EngineError{LegacyError{"Descriptor Set の確保失敗"}});
        }

        const VkDescriptorBufferInfo buffer_info{
            .buffer = ubo_buffer,
            .offset = 0,
            .range = sizeof(GlobalUbo),
        };

        const VkWriteDescriptorSet descriptor_write{
            .sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            .dstSet = descriptor_set,
            .dstBinding = 0,
            .dstArrayElement = 0,
            .descriptorCount = 1,
            .descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            .pBufferInfo = &buffer_info,
        };

        vkUpdateDescriptorSets(context.device, 1, &descriptor_write, 0, nullptr);

        return descriptor_set;
    }

} // namespace rey_engine::render
