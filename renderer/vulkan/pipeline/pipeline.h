#pragma once

#include <vulkan/vulkan.h>
#include <vector>
#include <expected>
#include <span>

#include "engine_error.h"

namespace rey_engine::render {

// =====================================================================
// Layer 3: Pipeline Structures
// =====================================================================

struct PushConstants {
    float model[16];
};

// グラフィックスパイプラインを段階的に構築するためのビルダー
class PipelineBuilder {
private:
    std::vector<VkPipelineShaderStageCreateInfo> shader_stages_;
    VkPipelineVertexInputStateCreateInfo vertex_input_info_{};
    VkPipelineInputAssemblyStateCreateInfo input_assembly_{};
    VkPipelineViewportStateCreateInfo viewport_state_{};
    VkPipelineRasterizationStateCreateInfo rasterizer_{};
    VkPipelineMultisampleStateCreateInfo multisampling_{};
    VkPipelineColorBlendAttachmentState color_blend_attachment_{};
    VkPipelineDepthStencilStateCreateInfo depth_stencil_{};
    VkPipelineLayout pipeline_layout_{VK_NULL_HANDLE};

public:
    PipelineBuilder() noexcept;

    // メソッドチェーンのための参照返し (インライン実装)
    PipelineBuilder& with_shaders(std::vector<VkPipelineShaderStageCreateInfo> stages) noexcept {
        shader_stages_ = std::move(stages);
        return *this;
    }

    PipelineBuilder& with_vertex_input(const VkPipelineVertexInputStateCreateInfo& info) noexcept {
        vertex_input_info_ = info;
        return *this;
    }

    PipelineBuilder& with_viewport_state(const VkPipelineViewportStateCreateInfo& info) noexcept {
        viewport_state_ = info;
        return *this;
    }

    PipelineBuilder& with_layout(VkPipelineLayout layout) noexcept {
        pipeline_layout_ = layout;
        return *this;
    }

    // パイプライン生成 (実装は .cpp へ)
    [[nodiscard]] std::expected<VkPipeline, EngineError> build(
        VkDevice device,
        VkRenderPass render_pass
    ) const noexcept;
};

// =====================================================================
// メインのパイプライン構造体
// =====================================================================

struct GraphicsPipeline {
    VkPipelineLayout layout{VK_NULL_HANDLE};
    VkPipeline pipeline{VK_NULL_HANDLE};

    // 静的ファクトリ関数
    [[nodiscard]] static std::expected<GraphicsPipeline, EngineError> create(
        VkDevice device,
        VkRenderPass render_pass,
        VkExtent2D extent,
        VkDescriptorSetLayout descriptor_set_layout,
        const VkVertexInputBindingDescription& binding_desc,
        std::span<const VkVertexInputAttributeDescription> attrib_desc
    ) noexcept;

    // パイプラインとレイアウトを破棄
    void destroy(VkDevice device) const noexcept;
};

} // namespace rey_engine::render
