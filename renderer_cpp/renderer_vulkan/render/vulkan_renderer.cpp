#include "vulkan_renderer.h"

#include <array>
#include <iostream>
#include <utility>

#include "renderer_vulkan/buffers/buffer.h"

namespace rey_engine::render {

std::expected<void, EngineError> VulkanRenderer::initialize_descriptor_resources() {
    auto layout = create_global_ubo_layout(context_.device);
    if (!layout) {
        return std::unexpected(layout.error());
    }
    descriptor_set_layout_ = *layout;

    auto pool = create_descriptor_pool(context_);
    if (!pool) {
        return std::unexpected(pool.error());
    }
    descriptor_pool_ = *pool;

    auto descriptor_set = create_descriptor_set(
        context_,
        descriptor_pool_,
        descriptor_set_layout_,
        global_ubo_buffer_.buffer);
    if (!descriptor_set) {
        return std::unexpected(descriptor_set.error());
    }
    global_descriptor_set_ = *descriptor_set;
    return {};
}

std::expected<void, EngineError> VulkanRenderer::initialize_pipeline_resources() {
    constexpr VkVertexInputBindingDescription binding_description{
        .binding = 0,
        .stride = sizeof(Vertex),
        .inputRate = VK_VERTEX_INPUT_RATE_VERTEX,
    };

    constexpr std::array attribute_descriptions = {
        VkVertexInputAttributeDescription{
            .location = 0,
            .binding = 0,
            .format = VK_FORMAT_R32G32B32_SFLOAT,
            .offset = offsetof(Vertex, position),
        },
        VkVertexInputAttributeDescription{
            .location = 1,
            .binding = 0,
            .format = VK_FORMAT_R32G32B32_SFLOAT,
            .offset = offsetof(Vertex, color),
        },
        VkVertexInputAttributeDescription{
            .location = 2,
            .binding = 0,
            .format = VK_FORMAT_R32G32B32_SFLOAT,
            .offset = offsetof(Vertex, normal),
        },
    };

    auto pipeline = GraphicsPipeline::create(
        context_.device,
        swapchain_target_.render_pass,
        swapchain_target_.extent,
        descriptor_set_layout_,
        binding_description,
        attribute_descriptions);
    if (!pipeline) {
        return std::unexpected(pipeline.error());
    }

    pipeline_ = std::move(*pipeline);
    return {};
}

GlobalUbo VulkanRenderer::build_global_ubo(const RenderSnapshot& snapshot) {
    return GlobalUbo{
        .view_proj = snapshot.view_matrix,
        .camera_pos = glm::vec3(0.0f),
        ._padding = 0.0f,
    };
}

VulkanRenderer::VulkanRenderer(VulkanRenderer&& other) noexcept = default;
VulkanRenderer& VulkanRenderer::operator=(VulkanRenderer&& other) noexcept = default;

VulkanRenderer::~VulkanRenderer() {
    if (context_.device == VK_NULL_HANDLE) {
        return;
    }

    vkDeviceWaitIdle(context_.device);

    for (auto& mesh : meshes_) {
        mesh.vertex_buffer.destroy(context_);
        mesh.index_buffer.destroy(context_);
    }
    meshes_.clear();

    swapchain_target_.destroy(context_.device);
    pipeline_.destroy(context_.device);
    sync_.destroy(context_.device);

    global_ubo_buffer_.destroy(context_);

    if (descriptor_pool_ != VK_NULL_HANDLE) {
        vkDestroyDescriptorPool(context_.device, descriptor_pool_, nullptr);
        descriptor_pool_ = VK_NULL_HANDLE;
    }
    if (descriptor_set_layout_ != VK_NULL_HANDLE) {
        vkDestroyDescriptorSetLayout(context_.device, descriptor_set_layout_, nullptr);
        descriptor_set_layout_ = VK_NULL_HANDLE;
    }

    context_.destroy();
    std::cout << "VulkanRenderer child objects destroyed cleanly.\n";
}

std::expected<VulkanRenderer, EngineError> VulkanRenderer::create(
    const char* app_name,
    void* window_handle,
    uint32_t window_width,
    uint32_t window_height)
{
    VulkanRenderer renderer;

    auto context = create_vulkan_context(app_name, static_cast<VkSurfaceKHR>(window_handle));
    if (!context) {
        return std::unexpected(context.error());
    }
    renderer.context_ = std::move(*context);

    auto swapchain = create_swapchain_target(renderer.context_, window_width, window_height);
    if (!swapchain) {
        return std::unexpected(swapchain.error());
    }
    renderer.swapchain_target_ = std::move(*swapchain);

    auto sync = SyncContext::create(
        renderer.context_.device,
        renderer.context_.graphics_queue_family_index,
        static_cast<uint32_t>(renderer.swapchain_target_.images.size()));
    if (!sync) {
        return std::unexpected(sync.error());
    }
    renderer.sync_ = std::move(*sync);

    constexpr VmaAllocationCreateInfo ubo_alloc_info{
        .flags = VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT | VMA_ALLOCATION_CREATE_MAPPED_BIT,
        .usage = VMA_MEMORY_USAGE_AUTO,
    };
    auto ubo_buffer = create_buffer(
        renderer.context_,
        sizeof(GlobalUbo),
        VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
        ubo_alloc_info);
    if (!ubo_buffer) {
        return std::unexpected(ubo_buffer.error());
    }
    renderer.global_ubo_buffer_ = std::move(*ubo_buffer);

    if (auto descriptor_result = renderer.initialize_descriptor_resources(); !descriptor_result) {
        return std::unexpected(descriptor_result.error());
    }

    if (auto pipeline_result = renderer.initialize_pipeline_resources(); !pipeline_result) {
        return std::unexpected(pipeline_result.error());
    }

    return renderer;
}

std::expected<MeshId, EngineError> VulkanRenderer::create_mesh_from_data(const MeshData& data) {
    auto vertex_buffer = create_device_local_buffer<Vertex>(
        context_,
        sync_.command_pool,
        std::span(data.vertices),
        VK_BUFFER_USAGE_VERTEX_BUFFER_BIT);
    if (!vertex_buffer) {
        return std::unexpected(vertex_buffer.error());
    }

    auto index_buffer = create_device_local_buffer<uint32_t>(
        context_,
        sync_.command_pool,
        std::span(data.indices),
        VK_BUFFER_USAGE_INDEX_BUFFER_BIT);
    if (!index_buffer) {
        vertex_buffer->destroy(context_);
        return std::unexpected(index_buffer.error());
    }

    const MeshId mesh_id{static_cast<uint32_t>(meshes_.size())};
    meshes_.push_back(GpuMesh{
        .vertex_buffer = std::move(*vertex_buffer),
        .index_buffer = std::move(*index_buffer),
        .index_count = static_cast<uint32_t>(data.indices.size()),
    });

    return mesh_id;
}

std::expected<void, EngineError> VulkanRenderer::draw_frame(const RenderSnapshot& snapshot) {
    auto active_frame_opt = begin_frame();
    if (!active_frame_opt) {
        return std::unexpected(active_frame_opt.error());
    }

    ActiveFrame active_frame = *active_frame_opt;
    CommandRecorder& recorder = active_frame.recorder;

    std::array<VkClearValue, 2> clear_values{};
    clear_values[0].color = {{0.0f, 0.0f, 1.0f, 1.0f}};
    clear_values[1].depthStencil = {1.0f, 0};

    (void)clear_values;

    if (auto res = update_buffer_data(context_, global_ubo_buffer_, build_global_ubo(snapshot)); !res) {
        return std::unexpected(res.error());
    }

    vkCmdBindDescriptorSets(
        recorder.command_buffer,
        VK_PIPELINE_BIND_POINT_GRAPHICS,
        pipeline_.layout,
        0,
        1,
        &global_descriptor_set_,
        0,
        nullptr);

    for (const auto& instance : snapshot.instances) {
        vkCmdPushConstants(
            recorder.command_buffer,
            pipeline_.layout,
            VK_SHADER_STAGE_VERTEX_BIT,
            0,
            sizeof(glm::mat4),
            &instance.model_matrix);

        if (instance.mesh_id.value < meshes_.size()) {
            const auto& mesh = meshes_[instance.mesh_id.value];
            VkBuffer vertex_buffers[] = {mesh.vertex_buffer.buffer};
            VkDeviceSize offsets[] = {0};
            vkCmdBindVertexBuffers(recorder.command_buffer, 0, 1, vertex_buffers, offsets);
            vkCmdBindIndexBuffer(recorder.command_buffer, mesh.index_buffer.buffer, 0, VK_INDEX_TYPE_UINT32);
            vkCmdDrawIndexed(recorder.command_buffer, mesh.index_count, 1, 0, 0, 0);
        }
    }

    return end_frame(active_frame);
}

std::expected<ActiveFrame, EngineError> VulkanRenderer::begin_frame() {
    const uint32_t frame = static_cast<uint32_t>(sync_.current_frame);

    if (vkWaitForFences(context_.device, 1, &sync_.in_flight_fences[frame], VK_TRUE, UINT64_MAX) != VK_SUCCESS) {
        return std::unexpected(EngineError{LegacyError{"Fenceの待機に失敗"}});
    }

    uint32_t image_index = 0;
    const VkResult result = vkAcquireNextImageKHR(
        context_.device,
        swapchain_target_.swapchain,
        UINT64_MAX,
        sync_.image_available_semaphores[frame],
        VK_NULL_HANDLE,
        &image_index);

    if (result == VK_ERROR_OUT_OF_DATE_KHR) {
        // リサイズ時はフレームをスキップする。実装は後続で強化できる。
    } else if (result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR) {
        return std::unexpected(EngineError{LegacyError{"画像の取得に失敗"}});
    }

    vkResetFences(context_.device, 1, &sync_.in_flight_fences[frame]);

    VkCommandBuffer cmd = sync_.command_buffers[frame];
    vkResetCommandBuffer(cmd, 0);

    const VkCommandBufferBeginInfo begin_info{.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO};
    if (vkBeginCommandBuffer(cmd, &begin_info) != VK_SUCCESS) {
        return std::unexpected(EngineError{LegacyError{"コマンドバッファの記録開始に失敗"}});
    }

    return ActiveFrame{
        .recorder = CommandRecorder{context_.device, cmd},
        .image_index = image_index,
        .frame_index = frame,
    };
}

std::expected<void, EngineError> VulkanRenderer::end_frame(const ActiveFrame& active_frame) {
    if (vkEndCommandBuffer(active_frame.recorder.command_buffer) != VK_SUCCESS) {
        return std::unexpected(EngineError{LegacyError{"コマンドバッファの終了に失敗"}});
    }

    VkSemaphore wait_semaphores[] = {sync_.image_available_semaphores[active_frame.frame_index]};
    VkPipelineStageFlags wait_stages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
    VkCommandBuffer command_buffers[] = {active_frame.recorder.command_buffer};
    VkSemaphore signal_semaphores[] = {sync_.render_finished_semaphores[active_frame.image_index]};

    const VkSubmitInfo submit_info{
        .sType = VK_STRUCTURE_TYPE_SUBMIT_INFO,
        .waitSemaphoreCount = 1,
        .pWaitSemaphores = wait_semaphores,
        .pWaitDstStageMask = wait_stages,
        .commandBufferCount = 1,
        .pCommandBuffers = command_buffers,
        .signalSemaphoreCount = 1,
        .pSignalSemaphores = signal_semaphores,
    };

    if (vkQueueSubmit(context_.graphics_queue, 1, &submit_info, sync_.in_flight_fences[active_frame.frame_index]) != VK_SUCCESS) {
        return std::unexpected(EngineError{LegacyError{"キューの送信に失敗"}});
    }

    const VkSwapchainKHR swapchains[] = {swapchain_target_.swapchain};
    const VkPresentInfoKHR present_info{
        .sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
        .waitSemaphoreCount = 1,
        .pWaitSemaphores = signal_semaphores,
        .swapchainCount = 1,
        .pSwapchains = swapchains,
        .pImageIndices = &active_frame.image_index,
    };

    vkQueuePresentKHR(context_.graphics_queue, &present_info);

    sync_.current_frame = (active_frame.frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
    return {};
}

} // namespace rey_engine::render
