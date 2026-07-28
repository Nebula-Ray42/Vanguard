// crates/renderer/src/command.rs

use crate::infra::context::GpuBuffer;
use ash::vk;
use render_api::engine_error::EngineError;

/// 描画コマンドを安全に記録するためのラッパー (Layer 4)
///
/// Vulkanの `cmd_` 系関数をカプセル化し、安全なインターフェースを提供します。
pub struct CommandRecorder<'a> {
    device: &'a ash::Device,
    pub(crate) command_buffer: vk::CommandBuffer,
}

impl<'a> CommandRecorder<'a> {
    /// 新しいコマンドレコーダーを生成します。
    #[inline(always)]
    pub fn new(device: &'a ash::Device, command_buffer: vk::CommandBuffer) -> Self {
        Self {
            device,
            command_buffer,
        }
    }

    /// 頂点バッファをバインドします。
    #[inline(always)]
    pub fn bind_vertex_buffer(&self, buffer: &GpuBuffer) {
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
        }
    }

    /// ビューポート（描画領域）を設定します。
    #[inline(always)]
    pub fn set_viewport(&self, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self.device
                .cmd_set_viewport(self.command_buffer, first_viewport, viewports);
        }
    }

    /// シザー（切り抜き領域）を設定します。
    #[inline(always)]
    pub fn set_scissor(&self, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe {
            self.device
                .cmd_set_scissor(self.command_buffer, first_scissor, scissors);
        }
    }

    /// インデックスバッファをバインドします。
    ///
    /// # Errors
    /// 渡された `GpuBuffer` がインデックスバッファとして初期化されていない（`index_type` が無い）場合、
    /// アプリをクラッシュさせず `EngineError` を返します。
    #[inline(always)]
    pub fn bind_index_buffer(&self, buffer: &GpuBuffer) -> Result<(), EngineError> {
        let index_type = buffer.index_type.ok_or_else(|| {
            EngineError::Legacy(
                "bind_index_buffer: 渡されたGpuBufferにindex_typeが設定されていません".to_string(),
            )
        })?;

        unsafe {
            self.device
                .cmd_bind_index_buffer(self.command_buffer, buffer.buffer, 0, index_type);
        }

        Ok(())
    }

    /// インデックス付き描画コマンド（Draw Indexed）を発行します。
    #[inline(always)]
    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.cmd_draw_indexed(
                self.command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    /// レンダーパスの記録を開始します。
    #[inline(always)]
    pub fn begin_render_pass(&self, begin_info: &vk::RenderPassBeginInfo) {
        unsafe {
            self.device.cmd_begin_render_pass(
                self.command_buffer,
                begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    /// レンダーパスの記録を終了します。
    #[inline(always)]
    pub fn end_render_pass(&self) {
        unsafe {
            self.device.cmd_end_render_pass(self.command_buffer);
        }
    }

    /// グラフィックスパイプラインをバインドします。
    #[inline(always)]
    pub fn bind_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
        }
    }

    /// プッシュ定数（Push Constants）をシェーダーに送信します。
    #[inline(always)]
    pub fn push_constants(
        &self,
        layout: vk::PipelineLayout,
        stage_flags: vk::ShaderStageFlags,
        offset: u32,
        constants: &[u8],
    ) {
        unsafe {
            self.device.cmd_push_constants(
                self.command_buffer,
                layout,
                stage_flags,
                offset,
                constants,
            );
        }
    }
}
