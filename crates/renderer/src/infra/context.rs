// crates/renderer/src/context.rs
use ash::khr::surface::Instance as SurfaceLoader;
use ash::{Device, Entry, Instance, vk};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::ffi::CString;
use std::sync::Arc;

use render_api::engine_error::EngineError;

/// Vulkanのコアインスタンスとデバイスを管理するコンテキスト (Layer 1)
///
/// エンジンのライフサイクル全体で生存し、GPUとの通信の基盤を提供します。
#[allow(dead_code)]
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: SurfaceLoader,
    pub physical_device: vk::PhysicalDevice,
    pub device: Arc<Device>,
    pub graphics_queue: vk::Queue,
}

impl VulkanContext {
    /// Vulkan APIを初期化し、描画可能なデバイスコンテキストを生成します。
    ///
    /// # Errors
    /// Vulkanローダーの取得、インスタンスの生成、物理デバイスの選定、または
    /// 論理デバイスの生成に失敗した場合に `EngineError` を返します。
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<Self, EngineError> {
        let entry = unsafe {
            Entry::load().map_err(|e| {
                EngineError::Legacy(format!("Vulkan APIのロードに失敗しました: {}", e))
            })?
        };

        let app_name = CString::new("Rey Engine").unwrap();
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&app_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let instance_extensions = ash_window::enumerate_required_extensions(display_handle)
            .map_err(|e| {
                EngineError::Legacy(format!("ウィンドウ拡張機能の取得に失敗しました: {}", e))
            })?
            .to_vec();

        let layer_names = [c"VK_LAYER_KHRONOS_validation".as_ptr()];

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&layer_names);

        let instance = unsafe {
            entry.create_instance(&create_info, None).map_err(|e| {
                EngineError::Legacy(format!("Vulkan Instanceの生成に失敗しました: {}", e))
            })?
        };

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                .map_err(|e| {
                    EngineError::Legacy(format!("Vulkan Surfaceの生成に失敗しました: {}", e))
                })?
        };

        let surface_loader = SurfaceLoader::new(&entry, &instance);

        let physical_devices = unsafe {
            instance.enumerate_physical_devices().map_err(|e| {
                EngineError::Legacy(format!("物理デバイスの取得に失敗しました: {}", e))
            })?
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
            .ok_or_else(|| {
                EngineError::Legacy(
                    "適合するGPUとキューファミリーが見つかりませんでした".to_string(),
                )
            })?;

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
                .map_err(|e| {
                    EngineError::Legacy(format!("論理デバイスの生成に失敗しました: {}", e))
                })?
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            entry,
            instance,
            surface,
            surface_loader,
            physical_device,
            device: Arc::new(device),
            graphics_queue,
        })
    }

    /// 指定された要件を満たすGPUメモリのタイプインデックスを検索します。
    pub fn find_memory_type(
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, EngineError> {
        for i in 0..memory_properties.memory_type_count {
            let is_type_suitable = (type_filter & (1 << i)) != 0;
            let actual_properties = memory_properties.memory_types[i as usize].property_flags;
            let has_required_properties = actual_properties.contains(properties);

            if is_type_suitable && has_required_properties {
                return Ok(i);
            }
        }
        Err(EngineError::Legacy(
            "条件に適合するメモリタイプが見つかりませんでした".to_string(),
        ))
    }

    /// GPU上にバッファ（頂点データやインデックスデータ用）を確保します。
    ///
    /// # Errors
    /// バッファの作成、メモリの割り当て、またはバインドに失敗した場合に `EngineError` を返します。
    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<GpuBuffer, EngineError> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .map_err(|e| EngineError::Legacy(format!("バッファの生成に失敗しました: {}", e)))?
        };

        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let mem_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        let memory_type_index = Self::find_memory_type(
            &mem_properties,
            mem_requirements.memory_type_bits,
            properties,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| {
                    EngineError::Legacy(format!("バッファ用メモリの確保に失敗しました: {}", e))
                })?
        };

        unsafe {
            self.device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .map_err(|e| {
                    EngineError::Legacy(format!("バッファとメモリのバインドに失敗しました: {}", e))
                })?;
        }

        Ok(GpuBuffer {
            buffer,
            memory: buffer_memory,
            size,
            index_type: None,
            device: Arc::clone(&self.device),
        })
    }
}

/// GPU上に確保されたメモリ領域を表すラッパー
///
/// スコープを抜けると `Drop` トレイトにより自動的にVulkanのメモリが解放されます。
pub struct GpuBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    pub index_type: Option<vk::IndexType>,
    device: Arc<Device>, // Drop時に安全に解放するためDeviceを保持
}

impl GpuBuffer {
    /// このバッファをインデックスバッファとして扱うための型情報を付与します。
    pub fn with_index_type(mut self, index_type: vk::IndexType) -> Self {
        self.index_type = Some(index_type);
        self
    }
}

impl Drop for GpuBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
