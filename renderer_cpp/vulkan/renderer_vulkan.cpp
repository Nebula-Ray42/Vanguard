#include "rey_renderer.h"
#include "generated/render_command_generated.h"
#include <iostream>

void execute_render_command(const uint8_t* ptr, size_t len) {
    // 1. 生ポインタとサイズからVerifierを構築
    flatbuffers::Verifier verifier(ptr, len);

    if (!rey_engine::render::VerifyRenderCommandBuffer(verifier)) {
        std::cerr << "[C++ Error] Invalid FlatBuffer data!" << std::endl;
        return;
    }

    auto command = rey_engine::render::GetRenderCommand(ptr);
    auto color = command->clear_color();

    if (color) {
        std::cout << "[C++23] RenderCommand Received! ClearColor( R: "
                  << color->r() << ", G: " << color->g()
                  << ", B: " << color->b() << ", A: " << color->a()
                  << " )" << std::endl;
    }
}
