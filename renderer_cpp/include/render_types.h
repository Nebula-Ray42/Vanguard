#pragma once

#include <vulkan/vulkan.hpp>
#include <glm/glm.hpp>
#include <vector>
#include <array>
#include <cstddef>

// ==========================================
// 1. Value Objects (ID群)
// ==========================================
struct EntityId { uint32_t value; };
struct MeshId { uint32_t value; };

// ==========================================
// 2. DTO (描画用データ構造)
// ==========================================

// 頂点1つ分のデータ
struct Vertex {
    glm::vec3 position;
    glm::vec3 color;
    glm::vec3 normal;

    // 構造体の中にVulkanへ教える関数を閉じ込めておくとスッキリします
    static vk::VertexInputBindingDescription get_binding_description() {
        return vk::VertexInputBindingDescription()
            .setBinding(0)
            .setStride(sizeof(Vertex))
            .setInputRate(vk::VertexInputRate::eVertex);
    }

    static std::array<vk::VertexInputAttributeDescription, 3> get_attribute_descriptions() {
        return {
            vk::VertexInputAttributeDescription(0, 0, vk::Format::eR32G32B32Sfloat, offsetof(Vertex, position)),
            vk::VertexInputAttributeDescription(1, 0, vk::Format::eR32G32B32Sfloat, offsetof(Vertex, color)),
            vk::VertexInputAttributeDescription(2, 0, vk::Format::eR32G32B32Sfloat, offsetof(Vertex, normal))
        };
    }
};

// メッシュ全体を表す純粋なデータ
struct MeshData {
    std::vector<Vertex> vertices;
    std::vector<uint32_t> indices;

    // TODO: キューブや平面の生成ロジックは、MeshDataのstaticメソッドとして実装
    static MeshData new_cube(float size, glm::vec3 color);
    static MeshData new_plane(float width, float depth, glm::vec3 color);
};

struct RenderInstance {
    EntityId entity_id;
    MeshId mesh_id;
    glm::mat4 model_matrix;
};

struct RenderSnapshot {
    uint64_t frame_number;
    std::vector<RenderInstance> instances;
    glm::mat4 view_matrix;
};

// --- GPU用データ構造 ---
// C++では alignas(16) を付けることで、Rustで手動計算していた
// 16バイトアライメント（std140/std430レイアウト）をコンパイラに保証させることができます。

struct alignas(16) GpuTransform {
    glm::vec3 position;
    float _pad0;
    glm::vec4 rotation;
    glm::vec3 scale;
    float _pad1;
};

struct alignas(16) GpuEntity {
    uint32_t id;
    uint32_t mesh_id;
    uint32_t _pad0[2];
    GpuTransform transform;
};

struct PushConstants {
    glm::mat4 mvp;

    // Always-Valid: 構築時に必ずMVPを計算して保持する
    PushConstants(const glm::mat4& model, const glm::mat4& view, const glm::mat4& proj) {
        // GLMの行列乗算も数式通り P * V * M
        mvp = proj * view * model;
    }
};
