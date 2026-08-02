#pragma once

#include <vulkan/vulkan.h>
#include <string>
#include <variant>
#include <format>

namespace rey_engine::render {

// ============================================================================
// 依存する他のエラードメイン
// ============================================================================
// TODO 後ほど render_pass_error.h などに分割するまでの「一時的な避難所」
struct RenderPassError {
    std::string message;
};

// ============================================================================
// 1. SwapchainError の定義
// ============================================================================
// 各バリアント（列挙子）が持つデータ（ペイロード）を独立した構造体として定義します。
namespace swapchain_error {
    struct QueryCapabilities { VkResult result; };
    struct QueryFormats      { VkResult result; };
    struct QueryPresentModes { VkResult result; };
    struct NoFormatsAvailable {}; // ペイロードなし
    struct CreateSwapchain   { VkResult result; };
    struct GetImages         { VkResult result; };

    // 一時的な避難所（Stringの受け皿）
    struct CreateImageView     { std::string message; };
    struct CreateDepthResource { std::string message; };
    struct CreateRenderPass    { RenderPassError error; CreateRenderPass() = default;

        explicit CreateRenderPass(VkResult res)
            : error{std::format("VkResult: {}", static_cast<int>(res))} {}
    };
    struct CreateFramebuffer   { std::string message; };
}  // namespace swapchain_error

using SwapchainError = std::variant<
    swapchain_error::QueryCapabilities,
    swapchain_error::QueryFormats,
    swapchain_error::QueryPresentModes,
    swapchain_error::NoFormatsAvailable,
    swapchain_error::CreateSwapchain,
    swapchain_error::GetImages,
    swapchain_error::CreateImageView,
    swapchain_error::CreateDepthResource,
    swapchain_error::CreateRenderPass,
    swapchain_error::CreateFramebuffer
>;

// ============================================================================
// 2. エラー文字列化
// ============================================================================
// パターンマッチング（std::visit）を使って、エラーの型ごとにメッセージを生成します。
inline std::string to_string(const SwapchainError& error) {
    return std::visit([]<typename T0>(const T0& e) -> std::string {
        using T = std::decay_t<T0>;
        if constexpr (std::is_same_v<T, swapchain_error::QueryCapabilities>) {
            return std::format("サーフェスの機能(Capabilities)の取得に失敗しました: {}", static_cast<int>(e.result));
        } else if constexpr (std::is_same_v<T, swapchain_error::QueryFormats>) {
            return std::format("対応するフォーマットの取得に失敗しました: {}", static_cast<int>(e.result));
        } else if constexpr (std::is_same_v<T, swapchain_error::QueryPresentModes>) {
            return std::format("対応するプレゼンモードの取得に失敗しました: {}", static_cast<int>(e.result));
        } else if constexpr (std::is_same_v<T, swapchain_error::NoFormatsAvailable>) {
            return "利用可能なフォーマットが1つも見つかりませんでした";
        } else if constexpr (std::is_same_v<T, swapchain_error::CreateSwapchain>) {
            return std::format("Swapchain本体の生成に失敗しました: {}", static_cast<int>(e.result));
        } else if constexpr (std::is_same_v<T, swapchain_error::GetImages>) {
            return std::format("Swapchain画像の取得に失敗しました: {}", static_cast<int>(e.result));
        } else if constexpr (std::is_same_v<T, swapchain_error::CreateImageView>) {
            return std::format("ImageViewの生成に失敗しました: {}", e.message);
        } else if constexpr (std::is_same_v<T, swapchain_error::CreateDepthResource>) {
            return std::format("Depthリソースの生成に失敗しました: {}", e.message);
        } else if constexpr (std::is_same_v<T, swapchain_error::CreateRenderPass>) {
            return std::format("RenderPassの生成に失敗しました: {}", e.error.message);
        } else if constexpr (std::is_same_v<T, swapchain_error::CreateFramebuffer>) {
            return std::format("Framebufferの生成に失敗しました: {}", e.message);
        } else {
            return "不明な SwapchainError";
        }
    }, error);
}

// ============================================================================
// 3. 統合された EngineError の定義
// ============================================================================
// 既存のレガシーエラーと、今回のドメイン特化エラーを束ねます。
struct LegacyError { std::string message; };

using EngineError = std::variant<
    LegacyError,
    SwapchainError
>;

// EngineError用の to_string
inline std::string to_string(const EngineError& error) {
    return std::visit([]<typename T0>(const T0& e) -> std::string {
        using T = std::decay_t<T0>;
        if constexpr (std::is_same_v<T, LegacyError>) {
            return e.message;
        } else if constexpr (std::is_same_v<T, SwapchainError>) {
            return to_string(e);
        } else {
            return "不明な EngineError";
        }
    }, error);
}

} // namespace rey_engine::render
