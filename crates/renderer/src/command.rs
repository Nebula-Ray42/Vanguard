// crates/renderer/src/command.rs

use ash::vk;
use crate::GpuBuffer;

/// 描画コマンドを記録するための薄いラッパー
pub struct CommandRecorder<'a> {
    device: &'a ash::Device,
    pub(crate) command_buffer: vk::CommandBuffer,
}

impl<'a> CommandRecorder<'a> {
    #[inline(always)]
    pub fn new(device: &'a ash::Device, command_buffer: vk::CommandBuffer) -> Self {
        Self { device, command_buffer }
    }

    #[inline(always)]
    pub fn bind_vertex_buffer(&self, buffer: &GpuBuffer) {
        unsafe {
            self.device.cmd_bind_vertex_buffers(self.command_buffer, 0, &[buffer.buffer], &[0]);
        }
    }

    #[inline(always)]
    pub fn bind_index_buffer(&self, buffer: &GpuBuffer) {
        let index_type = buffer
            .index_type
            .expect("bind_index_buffer: 渡されたGpuBufferにindex_typeが設定されていません");

        unsafe {
            self.device.cmd_bind_index_buffer(self.command_buffer, buffer.buffer, 0, index_type);
        }
    }

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

    #[inline(always)]
    pub fn end_render_pass(&self) {
        unsafe {
            self.device.cmd_end_render_pass(self.command_buffer);
        }
    }

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

