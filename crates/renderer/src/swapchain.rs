use crate::context::VulkanContext;
use ash::khr::swapchain::Device as SwapchainLoader;
use ash::{Device, vk};

pub struct SwapchainTarget {
    pub loader: SwapchainLoader,
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl SwapchainTarget {
    pub fn new(context: &VulkanContext, width: u32, height: u32) -> Result<Self, String> {
        let loader = SwapchainLoader::new(&context.instance, &context.device);

        let capabilities = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_capabilities(context.physical_device, context.surface)
                .map_err(|e| format!("Surface能力の取得に失敗しました: {}", e))?
        };
        let formats = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_formats(context.physical_device, context.surface)
                .map_err(|e| format!("Surfaceフォーマットの取得に失敗しました: {}", e))?
        };
        let present_modes = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_present_modes(context.physical_device, context.surface)
                .map_err(|e| format!("プレゼンテーションモードの取得に失敗しました: {}", e))?
        };

        let format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0]);

        let present_mode = present_modes
            .into_iter()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(context.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(|e| format!("Swapchainの生成に失敗しました: {}", e))?
        };

        let images = unsafe {
            loader
                .get_swapchain_images(swapchain)
                .map_err(|e| format!("Swapchain画像の取得に失敗しました: {}", e))?
        };

        let image_views = images
            .iter()
            .map(|&image| {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe {
                    context
                        .device
                        .create_image_view(&view_info, None)
                        .map_err(|e| format!("ImageView生成失敗: {}", e))
                }
            })
            .collect::<Result<Vec<_>, String>>()?;

        // RenderPassの構築
        let color_attachment = vk::AttachmentDescription::default()
            .format(format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_refs = [color_attachment_ref];
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe {
            context
                .device
                .create_render_pass(&render_pass_info, None)
                .map_err(|e| format!("RenderPass生成失敗: {}", e))?
        };

        let framebuffers = image_views
            .iter()
            .map(|&view| {
                let fb_attachments = [view];
                let fb_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&fb_attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                unsafe {
                    context
                        .device
                        .create_framebuffer(&fb_info, None)
                        .map_err(|e| format!("Framebuffer生成失敗: {}", e))
                }
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            loader,
            swapchain,
            format: format.format,
            extent,
            images,
            image_views,
            render_pass,
            framebuffers,
        })
    }

    pub unsafe fn destroy(&self, device: &Device) {
        unsafe {
            for &fb in &self.framebuffers {
                device.destroy_framebuffer(fb, None);
            }
            device.destroy_render_pass(self.render_pass, None);
            for &iv in &self.image_views {
                device.destroy_image_view(iv, None);
            }
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
