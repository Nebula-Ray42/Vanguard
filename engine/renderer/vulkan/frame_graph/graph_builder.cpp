#include "graph_builder.h"

namespace vanta::render::fg {

PassBuilder& PassBuilder::read_image(ImageHandle handle, UsageType usage) noexcept {
    graph_.all_read_images.push_back(PassResource{handle, usage});
    pass_.read_images_count++;

    return *this;
}

PassBuilder& PassBuilder::write_image(ImageHandle handle, UsageType usage) noexcept {
    graph_.all_write_images.push_back(PassResource{handle, usage});
    pass_.write_images_count++;
    return *this;
}

PassBuilder& PassBuilder::read_buffer(BufferHandle handle, UsageType usage) noexcept {
    graph_.all_read_buffers.push_back(PassBufferResource{handle, usage});
    pass_.read_buffers_count++;
    return *this;
}

PassBuilder& PassBuilder::write_buffer(BufferHandle handle, UsageType usage) noexcept {
    graph_.all_write_buffers.push_back(PassBufferResource{handle, usage});
    pass_.write_buffers_count++;
    return *this;
}

void PassBuilder::execute(PassData::ExecuteFunc func) noexcept {
    pass_.execute = func;
}

PassBuilder RenderGraphBuilder::add_pass(std::string_view name) noexcept {
    PassData new_pass{};

    new_pass.read_images_offset = static_cast<uint32_t>(graph_data_.all_read_images.size());
    new_pass.write_images_offset = static_cast<uint32_t>(graph_data_.all_write_images.size());
    new_pass.read_buffers_offset = static_cast<uint32_t>(graph_data_.all_read_buffers.size());
    new_pass.write_buffers_offset = static_cast<uint32_t>(graph_data_.all_write_buffers.size());

    graph_data_.passes.push_back(new_pass);
    graph_data_.pass_names.push_back(name);

    return PassBuilder(graph_data_, graph_data_.passes.back());
}

}  // namespace vanta::render::fg
