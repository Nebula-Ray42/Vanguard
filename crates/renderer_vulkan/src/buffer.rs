use crate::pipeline::context::VulkanContext;
use ash::vk;
use render_api::error_pass::engine_error::EngineError;

/// バッファ間のデータ転送（One-Time Submit）を同期的に実行します。
///
/// # Errors
/// コマンドバッファの確保、記録、またはGPUキューへの送信・待機に失敗した場合に `EngineError` を返します。
pub fn copy_buffer(
    context: &VulkanContext,
    command_pool: vk::CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<(), EngineError> {
    // 1. 割り当て設定
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    // FFI呼び出しのみをunsafe化し、エラーは伝播
    let command_buffer = unsafe {
        context
            .device
            .allocate_command_buffers(&alloc_info)
            .map_err(|e| {
                EngineError::Legacy(format!("コマンドバッファの割り当てに失敗: {:?}", e))
            })?[0]
    };

    // 2. 記録開始設定
    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        context
            .device
            .begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| EngineError::Legacy(format!("コマンドバッファの開始に失敗: {:?}", e)))?;
    }

    // 3. コピー実行
    let copy_region = vk::BufferCopy::default().size(size);

    unsafe {
        context
            .device
            .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);

        context
            .device
            .end_command_buffer(command_buffer)
            .map_err(|e| EngineError::Legacy(format!("コマンドバッファの終了に失敗: {:?}", e)))?;
    }

    // 4. キューへの送信と待機）
    let command_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);

    unsafe {
        context
            .device
            .queue_submit(context.graphics_queue, &[submit_info], vk::Fence::null())
            .map_err(|e| EngineError::Legacy(format!("キューの送信に失敗: {:?}", e)))?;

        // 確実な同期
        context
            .device
            .queue_wait_idle(context.graphics_queue)
            .map_err(|e| EngineError::Legacy(format!("キューの待機中にエラーが発生: {:?}", e)))?;

        context
            .device
            .free_command_buffers(command_pool, &command_buffers);
    }

    Ok(())
}
