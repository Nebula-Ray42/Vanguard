#pragma once

#include <vulkan/vulkan.h>
#include <expected>
#include "render_graph_types.h"
#include "engine_error.h"

namespace vanta::render::fg {

struct ExecutionContext {
    VkCommandBuffer cmd_buffer;
    // TODO 将来的には、ここに「今のフレームの仮想IDと実際のVkImageの紐付け表」などを追加します
};

[[nodiscard]] std::expected<void, EngineError> execute_graph(
    const ExecutionPlan& plan,
    const ExecutionContext& context
) noexcept;

}  // namespace vanta::render::fg
