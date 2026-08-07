#pragma once

#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

namespace vanguard::scene {

    // 1. カメラのデータ（状態）
    struct CameraData {
        // 位置と向き
        glm::vec3 position{0.0f, 5.0f, 10.0f}; // デフォルト: 少し高くて後ろ
        glm::vec3 target{0.0f, 0.0f, 0.0f};    // デフォルト: 原点を見つめる
        glm::vec3 up{0.0f, 1.0f, 0.0f};        // デフォルト: Y軸が上

        // レンズの設定
        float fov_degrees = 45.0f;
        float aspect_ratio = 800.0f / 600.0f;
        float near_plane = 0.1f;
        float far_plane = 100.0f;
    };

    // 2. 行列計算（データを変換する純粋な関数）
    [[nodiscard]] inline glm::mat4 compute_view_matrix(const CameraData& camera) noexcept {
        return glm::lookAt(camera.position, camera.target, camera.up);
    }

    [[nodiscard]] inline glm::mat4 compute_projection_matrix(const CameraData& camera) noexcept {
        auto proj = glm::perspective(glm::radians(camera.fov_degrees), camera.aspect_ratio, camera.near_plane, camera.far_plane);
        proj[1][1] *= -1.0f;
        return proj;
    }

} // namespace rey_engine::scene
