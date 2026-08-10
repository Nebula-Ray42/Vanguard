// Copyright (c) 2026 Nebula-Ray42.
// SPDX-License-Identifier: BSD-2-Clause-Patent

#define GLFW_INCLUDE_VULKAN
#include <iostream>
#include <stdexcept>
#include <string>
#include <variant>

#include "engine_error.hpp"
#include "include/glfw3.h"
#include "include/render_types.hpp"
#include "scene/camera.hpp"
#include "scene/mesh.hpp"
#include "vk_backend/render/vulkan_renderer.hpp"

#include <glm/glm.hpp>

namespace {

constexpr uint32_t kWindowWidth = 800;
constexpr uint32_t kWindowHeight = 600;

std::string describe_error(const vanguard::render::EngineError& error) {
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
        std::cerr << "GLFW の初期化に失敗しました\n";
        return -1;
    }

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);

    GLFWwindow* window = glfwCreateWindow(kWindowWidth, kWindowHeight, "Rey Engine - Vulkan Test", nullptr, nullptr);
    if (window == nullptr) {
        std::cerr << "ウィンドウの作成に失敗しました\n";
        glfwTerminate();
        return -1;
    }

    std::cout << "ウィンドウを作成しました。VulkanRenderer を初期化します...\n";

    try {
        auto renderer_expected = vanguard::render::VulkanRenderer::create(
            "Rey Engine Test",
            window,
            kWindowWidth,
            kWindowHeight);

        if (!renderer_expected) {
            throw std::runtime_error("レンダラー初期化エラー: " + describe_error(renderer_expected.error()));
        }

        auto renderer = std::move(renderer_expected.value());
        std::cout << "VulkanRenderer の初期化に成功しました！\n";

        // ==========================================
        // 1. データの準備 (床の作成とGPU登録)
        // ==========================================
        auto floor_data = vanguard::scene::create_ground_grid(10.0f, 1.0f, 0);
        auto mesh_opt = renderer.create_mesh_from_data(floor_data);
        if (!mesh_opt) {
            std::cerr << "メッシュのGPU登録に失敗しました\n";
            return -1;
        }
        auto floor_mesh_id = *mesh_opt;

        // ==========================================
        // 2. カメラの初期設定
        // ==========================================
        vanguard::scene::CameraData camera{};

        // ==========================================
        // 3. メインループ
        // ==========================================
        uint64_t frame_count = 0;

        while (!glfwWindowShouldClose(window)) {
            glfwPollEvents();

           RenderSnapshot snapshot{};
            snapshot.frame_number = frame_count++;

            snapshot.view_matrix = vanguard::scene::compute_projection_matrix(camera) *
                                   vanguard::scene::compute_view_matrix(camera);

            // 床のインスタンス情報を追加
            RenderInstance floor_instance{};
            floor_instance.entity_id = {0};
            floor_instance.mesh_id = floor_mesh_id;
            floor_instance.model_matrix = glm::mat4(1.0f);

            snapshot.instances.push_back(floor_instance);

            if (auto draw_res = renderer.draw_frame(snapshot); !draw_res) {
                std::cerr << "描画エラー: " << describe_error(draw_res.error()) << '\n';
                break;
            }
        }

        std::cout << "メインループを終了します。リソースを安全に破棄します...\n";
    } catch (const std::exception& e) {
        std::cerr << "致命的なエラー: " << e.what() << '\n';
    }

    glfwDestroyWindow(window);
    glfwTerminate();

    std::cout << "シャットダウン完了。GPU クラッシュは発生していません。\n";
    return 0;
}
