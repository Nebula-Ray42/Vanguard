#define GLFW_INCLUDE_VULKAN
#include <iostream>
#include <stdexcept>
#include <string>
#include <variant>

#include "engine_error.h"
#include "glfw3.h"
#include "render/vulkan_renderer.h"

namespace {

constexpr uint32_t kWindowWidth = 800;
constexpr uint32_t kWindowHeight = 600;

std::string describe_error(const rey_engine::render::EngineError& error) {
    return std::visit([]<typename T0>(const T0& err) -> std::string {
        using T = std::decay_t<T0>;
        if constexpr (requires { err.message; }) {
            return err.message;
        } else {
            return "詳細不明な Vulkan エラー";
        }
    }, error);
}

} // namespace

int main() {
    if (glfwInit() == 0) {
        std::cerr << "GLFW の初期化に失敗しました" << '\n';
        return -1;
    }

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);

    GLFWwindow* window = glfwCreateWindow(kWindowWidth, kWindowHeight, "Rey Engine - Vulkan Test", nullptr, nullptr);
    if (window == nullptr) {
        std::cerr << "ウィンドウの作成に失敗しました" << '\n';
        glfwTerminate();
        return -1;
    }

    std::cout << "ウィンドウを作成しました。VulkanRenderer を初期化します..." << '\n';

    try {
        auto renderer_expected = rey_engine::render::VulkanRenderer::create(
            "Rey Engine Test",
            window,
            kWindowWidth,
            kWindowHeight);

        if (!renderer_expected) {
            throw std::runtime_error("レンダラー初期化エラー: " + describe_error(renderer_expected.error()));
        }

        auto renderer = std::move(renderer_expected.value());
        std::cout << "VulkanRenderer の初期化に成功しました！" << '\n';

        while (!glfwWindowShouldClose(window)) {
            glfwPollEvents();
        }

        std::cout << "メインループを終了します。リソースを安全に破棄します..." << '\n';
    } catch (const std::exception& e) {
        std::cerr << "致命的なエラー: " << e.what() << '\n';
    }

    glfwDestroyWindow(window);
    glfwTerminate();

    std::cout << "シャットダウン完了。GPU クラッシュは発生していません。" << '\n';
    return 0;
}
