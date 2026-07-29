// crates/renderer_vulkan/src/swapchain.rs
use ash::khr::swapchain::Device as SwapchainLoader;
use ash::{Device, vk};
use render_api::render_pass_error::RenderPassError;
use render_api::swapchain::SwapchainError;

use crate::pipeline::context::VulkanContext;
use render_api::engine_error::EngineError;

/// スワップチェーンのライフサイクルと画面更新を管理する構造体
///
/// Vulkanの描画結果をディスプレイに同期して出力するための
/// レスポンス出力バッファキューとして機能します。
#[allow(dead_code)]
pub struct SwapchainTarget {
    pub loader: SwapchainLoader,
    pub swapchain: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub depth_format: vk::Format,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl SwapchainTarget {
    /// 新しいスワップチェーンと関連リソースを生成します。
    pub fn new(context: &VulkanContext, width: u32, height: u32) -> Result<Self, SwapchainError> {
        let loader = SwapchainLoader::new(&context.instance, &context.device);

        // 1. Swapchain本体の生成
        let (swapchain, format, extent) = Self::create_swapchain(context, &loader, width, height)?;

        // 2. 画像とImageViewの取得
        let images = unsafe {
            loader
                .get_swapchain_images(swapchain)
                .map_err(SwapchainError::GetImages)?
        };

        // 内部関数を EngineError を返すように修正したため、ここで Enum にマッピング
        let image_views = Self::create_swapchain_image_views(context, format, &images)
            .map_err(|e| SwapchainError::CreateImageView(e.to_string()))?;

        // 3. Depth Bufferの生成
        let (depth_format, depth_image, depth_image_memory, depth_image_view) =
            Self::create_depth_resources(context, extent)
                .map_err(|e| SwapchainError::CreateDepthResource(e.to_string()))?;

        // 4. RenderPassの生成
        let render_pass = Self::create_render_pass(context, format, depth_format)
            .map_err(SwapchainError::CreateRenderPass)?;

        // 5. Framebufferの生成
        let framebuffers =
            Self::create_framebuffers(context, render_pass, extent, &image_views, depth_image_view)
                .map_err(|e| SwapchainError::CreateFramebuffer(e.to_string()))?;

        Ok(Self {
            loader,
            swapchain,
            format,
            extent,
            images,
            image_views,
            depth_format,
            depth_image,
            depth_image_memory,
            depth_image_view,
            render_pass,
            framebuffers,
        })
    }

    fn select_surface_format(formats: &[vk::SurfaceFormatKHR]) -> Option<vk::SurfaceFormatKHR> {
        formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .or(formats.first())
            .copied()
    }

    fn select_present_mode(modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        modes
            .iter()
            .copied()
            .find(|&m| m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn compute_extent(caps: &vk::SurfaceCapabilitiesKHR, requested: (u32, u32)) -> vk::Extent2D {
        if caps.current_extent.width != u32::MAX {
            caps.current_extent
        } else {
            vk::Extent2D {
                width: requested
                    .0
                    .clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                height: requested
                    .1
                    .clamp(caps.min_image_extent.height, caps.max_image_extent.height),
            }
        }
    }

    fn compute_image_count(caps: &vk::SurfaceCapabilitiesKHR) -> u32 {
        let wanted = caps.min_image_count + 1;
        if caps.max_image_count > 0 {
            wanted.min(caps.max_image_count)
        } else {
            wanted
        }
    }

    fn create_swapchain(
        context: &VulkanContext,
        loader: &SwapchainLoader,
        width: u32,
        height: u32,
    ) -> Result<(vk::SwapchainKHR, vk::Format, vk::Extent2D), SwapchainError> {
        let caps = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_capabilities(context.physical_device, context.surface)
                .map_err(SwapchainError::QueryCapabilities)?
        };
        let formats = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_formats(context.physical_device, context.surface)
                .map_err(SwapchainError::QueryFormats)?
        };
        let present_modes = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_present_modes(context.physical_device, context.surface)
                .map_err(SwapchainError::QueryPresentModes)?
        };

        let format =
            Self::select_surface_format(&formats).ok_or(SwapchainError::NoFormatsAvailable)?;
        let present_mode = Self::select_present_mode(&present_modes);
        let extent = Self::compute_extent(&caps, (width, height));
        let image_count = Self::compute_image_count(&caps);

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(context.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(SwapchainError::CreateSwapchain)?
        };

        Ok((swapchain, format.format, extent))
    }

    // 古い String 返却を EngineError に統一
    fn create_swapchain_image_views(
        context: &VulkanContext,
        format: vk::Format,
        images: &[vk::Image],
    ) -> Result<Vec<vk::ImageView>, EngineError> {
        images
            .iter()
            .map(|&image| {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(vk::ComponentMapping::default())
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
                        .map_err(|e| EngineError::Legacy(format!("ImageView生成失敗: {}", e)))
                }
            })
            .collect()
    }

    // find_memory_type の EngineError をそのまま伝播可能に
    fn create_depth_resources(
        context: &VulkanContext,
        extent: vk::Extent2D,
    ) -> Result<(vk::Format, vk::Image, vk::DeviceMemory, vk::ImageView), EngineError> {
        let depth_format = vk::Format::D32_SFLOAT;

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(depth_format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = unsafe {
            context
                .device
                .create_image(&image_info, None)
                .map_err(|e| EngineError::Legacy(format!("Depth Image生成失敗: {}", e)))?
        };

        let mem_reqs = unsafe { context.device.get_image_memory_requirements(depth_image) };
        let mem_props = unsafe {
            context
                .instance
                .get_physical_device_memory_properties(context.physical_device)
        };

        // 先ほど EngineError 対応した find_memory_type なので、そのまま ? で伝播できる
        let mem_type_index = VulkanContext::find_memory_type(
            &mem_props,
            mem_reqs.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_reqs.size)
            .memory_type_index(mem_type_index);

        let depth_image_memory = unsafe {
            context
                .device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| EngineError::Legacy(format!("Depth メモリ確保失敗: {}", e)))?
        };

        unsafe {
            context
                .device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .map_err(|e| EngineError::Legacy(format!("Depth メモリバインド失敗: {}", e)))?
        };

        let view_info = vk::ImageViewCreateInfo::default()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let depth_image_view = unsafe {
            context
                .device
                .create_image_view(&view_info, None)
                .map_err(|e| EngineError::Legacy(format!("Depth ImageView生成失敗: {}", e)))?
        };

        Ok((
            depth_format,
            depth_image,
            depth_image_memory,
            depth_image_view,
        ))
    }

    fn create_render_pass(
        context: &VulkanContext,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<vk::RenderPass, RenderPassError> {
        let color_attachment = vk::AttachmentDescription::default()
            .format(color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);

        let sync_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(sync_stage_mask)
            .dst_stage_mask(sync_stage_mask)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );

        let attachments = [color_attachment, depth_attachment];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        unsafe {
            context
                .device
                .create_render_pass(&render_pass_info, None)
                .map_err(RenderPassError::CreateFailed)
        }
    }

    fn create_framebuffers(
        context: &VulkanContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        image_views: &[vk::ImageView],
        depth_image_view: vk::ImageView,
    ) -> Result<Vec<vk::Framebuffer>, EngineError> {
        image_views
            .iter()
            .map(|&view| {
                let attachments = [view, depth_image_view];
                let fb_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                unsafe {
                    context
                        .device
                        .create_framebuffer(&fb_info, None)
                        .map_err(|e| EngineError::Legacy(format!("Framebuffer生成失敗: {}", e)))
                }
            })
            .collect()
    }

    /// スワップチェーンとその関連リソースを破棄します。
    pub unsafe fn destroy(&self, device: &Device) {
        unsafe {
            for &fb in &self.framebuffers {
                device.destroy_framebuffer(fb, None);
            }
            device.destroy_render_pass(self.render_pass, None);
            for &iv in &self.image_views {
                device.destroy_image_view(iv, None);
            }
            device.destroy_image_view(self.depth_image_view, None);
            device.destroy_image(self.depth_image, None);
            device.free_memory(self.depth_image_memory, None);
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
