mod device;
mod errors;
mod instance;
mod queue_families;
#[cfg(feature = "validation_layers")]
mod validation_layers;

use std::sync::Arc;

use ash::{prelude::VkResult, vk};
pub use device::{create_device, PhysicalDeviceData, SwapchainBuilder};
use he42_vulkan::instance::Instance;
use instance::create_instance;
pub use queue_families::QueueFamilies;
use rs42::{
    scope_guard::{Defer, ScopeGuard},
    Result,
};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[cfg(feature = "validation_layers")]
use he42_vulkan::debug_utils_messenger::DebugUtilsMessenger;
#[cfg(feature = "validation_layers")]
use validation_layers::{check_validation_layers, create_debug_messenger};

pub struct VulkanContext {
    instance: Arc<Instance>,

    #[cfg(feature = "validation_layers")]
    _debug_messenger: DebugUtilsMessenger,

    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    physical_device_features: vk::PhysicalDeviceFeatures,
    physical_device_max_sample_count: vk::SampleCountFlags,
    device: ash::Device,
    is_device_destroyed: bool,
}

impl VulkanContext {
    pub fn new(window: &winit::window::Window) -> Result<(Self, QueueFamilies, SwapchainBuilder)> {
        let display_handle = window.display_handle()?.into();
        let window_handle = window.window_handle()?.into();

        let vulkan_library = unsafe { he42_vulkan::VulkanLibrary::new()? };

        #[cfg(feature = "validation_layers")]
        check_validation_layers(&vulkan_library)?;

        let instance = create_instance(vulkan_library, display_handle)?;

        #[cfg(feature = "validation_layers")]
        let debug_messenger = create_debug_messenger(Arc::clone(&instance))?;

        let surface = unsafe {
            ash_window::create_surface(
                instance.vulkan_library(),
                instance.raw_instance(),
                display_handle,
                window_handle,
                None,
            )?
            .defer(|surface| instance.surface().destroy_surface(surface, None))
        };

        let physical_device_data =
            PhysicalDeviceData::new(&instance, *surface, window.inner_size())?;
        let device = unsafe { create_device(&instance, &physical_device_data)? }
            .defer(|device| unsafe { device.destroy_device(None) });

        Ok((
            VulkanContext {
                device: ScopeGuard::into_inner(device),
                physical_device_max_sample_count: physical_device_data.max_sample_count,
                physical_device_features: physical_device_data.physical_device_features,
                physical_device_properties: physical_device_data.physical_device_properties,
                physical_device: physical_device_data.physical_device,
                surface: ScopeGuard::into_inner(surface),
                #[cfg(feature = "validation_layers")]
                _debug_messenger: debug_messenger,
                instance,
                is_device_destroyed: false,
            },
            physical_device_data.queue_families,
            physical_device_data.swapchain_builder,
        ))
    }

    pub fn device_wait_idle(&self) -> VkResult<()> {
        if self.is_device_destroyed {
            return Ok(());
        }

        unsafe { self.device.device_wait_idle() }
    }

    pub fn device(&self) -> &ash::Device {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::device() was called after device destruction"
        );
        &self.device
    }

    pub fn set_device(
        &mut self,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        physical_device_properties: vk::PhysicalDeviceProperties,
        physical_device_features: vk::PhysicalDeviceFeatures,
        physical_device_max_sample_count: vk::SampleCountFlags,
    ) {
        // TODO maybe make a Device struct that holds both those values?

        debug_assert!(
            self.is_device_destroyed,
            "VulkanContext::set_device() was called without device destruction"
        );

        self.physical_device = physical_device;
        self.device = device;
        self.physical_device_properties = physical_device_properties;
        self.physical_device_features = physical_device_features;
        self.physical_device_max_sample_count = physical_device_max_sample_count;
        self.is_device_destroyed = false;
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::physical_device() was called after device destruction"
        );

        self.physical_device
    }

    pub fn physical_device_properties(&self) -> &vk::PhysicalDeviceProperties {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::physical_device_properties() was called after device destruction"
        );

        &self.physical_device_properties
    }

    pub fn physical_device_features(&self) -> &vk::PhysicalDeviceFeatures {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::physical_device_features() was called after device destruction"
        );

        &self.physical_device_features
    }

    pub fn physical_device_max_sample_count(&self) -> vk::SampleCountFlags {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::physical_device_max_sample_count() was called after device destruction"
        );

        self.physical_device_max_sample_count
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub unsafe fn destroy_device(&mut self) {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::destroy_device() was called after device destruction"
        );
        self.device.destroy_device(None);
        self.is_device_destroyed = true;
    }

    pub unsafe fn destroy(&mut self) {
        // TODO add is_destroyed member and do debug_assertions with it (Same for other structs)

        if !self.is_device_destroyed {
            self.destroy_device();
        }

        self.instance()
            .surface()
            .destroy_surface(self.surface, None);
    }
}
