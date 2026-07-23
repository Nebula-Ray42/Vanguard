use crate::context::VulkanContext;
use ash::vk;

pub fn copy_buffer(
    context: &VulkanContext,
    command_pool: vk::CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    unsafe {
        let command_buffer = context
            .device
            .allocate_command_buffers(&alloc_info)
            .unwrap()[0];
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        context
            .device
            .begin_command_buffer(command_buffer, &begin_info)
            .unwrap();

        let copy_region = vk::BufferCopy::default().size(size);
        context
            .device
            .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);

        context.device.end_command_buffer(command_buffer).unwrap();

        let submit_info =
            vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&command_buffer));
        context
            .device
            .queue_submit(context.graphics_queue, &[submit_info], vk::Fence::null())
            .unwrap();
        context
            .device
            .queue_wait_idle(context.graphics_queue)
            .unwrap();

        context
            .device
            .free_command_buffers(command_pool, &[command_buffer]);
    }
}
