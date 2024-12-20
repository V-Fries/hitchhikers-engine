mod device;
mod errors;
mod instance;
mod queue_families;
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

pub struct VulkanContext {
    instance: Arc<Instance>,

    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

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

        let (instance, debug_messenger) = create_instance(display_handle)?;
        let debug_messenger = debug_messenger.defer(|debug_messenger| unsafe {
            if let Some(debug_messenger) = debug_messenger {
                instance
                    .debug_utils()
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
        });

        let surface_instance = instance.surface();

        let surface = unsafe {
            ash_window::create_surface(
                instance.vulkan_library(),
                instance.raw_instance(),
                display_handle,
                window_handle,
                None,
            )?
            .defer(|surface| surface_instance.destroy_surface(surface, None))
        };

        let physical_device_data =
            PhysicalDeviceData::new(&instance, *surface, window.inner_size())?;
        let device = unsafe { create_device(&instance, &physical_device_data)? }
            .defer(|device| unsafe { device.destroy_device(None) });

        #[cfg(not(feature = "validation_layers"))]
        {
            let _ = ScopeGuard::into_inner(debug_messenger);
        }
        Ok((
            VulkanContext {
                device: ScopeGuard::into_inner(device),
                physical_device_max_sample_count: physical_device_data.max_sample_count,
                physical_device_features: physical_device_data.physical_device_features,
                physical_device_properties: physical_device_data.physical_device_properties,
                physical_device: physical_device_data.physical_device,
                surface: ScopeGuard::into_inner(surface),
                #[cfg(feature = "validation_layers")]
                debug_messenger: ScopeGuard::into_inner(debug_messenger)
                    .expect("Debug messenger was not initialized"),
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
        #[cfg(feature = "validation_layers")]
        {
            self.instance()
                .debug_utils()
                .destroy_debug_utils_messenger(self.debug_messenger, None);
        }
        self.instance()
            .surface()
            .destroy_surface(self.surface, None);
    }
}
