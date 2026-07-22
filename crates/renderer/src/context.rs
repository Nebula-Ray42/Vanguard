// crates/renderer/src/context.rs
use ash::khr::surface::Instance as SurfaceLoader;
use ash::{Device, Entry, Instance, vk};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::ffi::CString;

pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: SurfaceLoader,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub graphics_queue: vk::Queue,
}

impl VulkanContext {
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<Self, String> {
        let entry = unsafe {
            Entry::load().map_err(|e| format!("Vulkan APIのロードに失敗しました: {}", e))?
        };

        let app_name = CString::new("Rey Engine").unwrap();
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&app_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let surface_extensions = ash_window::enumerate_required_extensions(display_handle)
            .map_err(|e| format!("ウィンドウ拡張機能の取得に失敗しました: {}", e))?;

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(surface_extensions);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| format!("Vulkan Instanceの生成に失敗しました: {}", e))?
        };

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                .map_err(|e| format!("Vulkan Surfaceの生成に失敗しました: {}", e))?
        };

        let surface_loader = SurfaceLoader::new(&entry, &instance);

        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("物理デバイスの取得に失敗しました: {}", e))?
        };

        let (physical_device, queue_family_index) = physical_devices
            .into_iter()
            .find_map(|p_device| {
                let queue_families =
                    unsafe { instance.get_physical_device_queue_family_properties(p_device) };
                queue_families.iter().enumerate().find_map(|(index, info)| {
                    let supports_graphics = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                    let supports_surface = unsafe {
                        surface_loader
                            .get_physical_device_surface_support(p_device, index as u32, surface)
                            .unwrap_or(false)
                    };
                    if supports_graphics && supports_surface {
                        Some((p_device, index as u32))
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| "適合するGPUとキューファミリーが見つかりませんでした".to_string())?;

        let queue_priorities = [1.0_f32];
        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);
        let queue_create_infos = [queue_create_info];

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| format!("論理デバイスの生成に失敗しました: {}", e))?
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            entry,
            instance,
            surface,
            surface_loader,
            physical_device,
            device,
            graphics_queue,
        })
    }

    pub unsafe fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
