#include "graph_executor.h"

namespace vanta::render::fg {

std::expected<void, EngineError> execute_graph(
    const ExecutionPlan& plan,
    const ExecutionContext& context
) noexcept {
    // コマンドバッファが空の場合はエラーとして処理を中断する
    if (context.cmd_buffer == VK_NULL_HANDLE) {
        return std::unexpected(LegacyError("コマンドバッファが空です"));
    }

    // 事前に順番が整理されたパス（描画や計算の命令）を最初から最後まで順番に実行する
    for (const auto& pass : plan.sorted_passes) {

        // ==========================================
        // TODO: ここでメモリバリア（画像やバッファの同期）の命令をVulkanに積む
        //
        // 理想的な設計（データ指向）では、この実行処理の中で複雑な計算は行いません。
        // あらかじめ ExecutionPlan を作る段階で VkImageMemoryBarrier2 などの
        // 構造体の配列を計算して用意しておき、ここでは vkCmdPipelineBarrier2 を
        // 使ってその配列をそのままVulkanに渡すだけの処理にします。
        // ==========================================

        // パスの中身（実際の描画や計算の関数）が設定されていれば実行する
        if (pass.execute != nullptr) {
            pass.execute(context.cmd_buffer);
        }
    }

    // すべての処理が正常にコマンドバッファに記録された場合は成功を返す
    return {};
}

}  // namespace vanta::render::fg
