mod errors;
mod builder;
mod queue_families;

use ash::{prelude::VkResult, vk};
use crate::utils::{PipeLine, Result};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use builder::VulkanContextBuilder;
pub use queue_families::QueueFamilies;
pub use builder::{SwapchainBuilder, PhysicalDeviceData, create_device};

pub struct VulkanContext {
    entry: ash::Entry,
    instance: ash::Instance,

    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

    surface: vk::SurfaceKHR,
    surface_instance: ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    is_device_destroyed: bool,
}

impl VulkanContext {
    pub fn new(window: &winit::window::Window)
               -> Result<(Self, QueueFamilies, SwapchainBuilder)> {
        let display_handle = window.display_handle()?.into();
        let window_handle = window.window_handle()?.into();
        let window_inner_size = window.inner_size();

        unsafe {
            VulkanContextBuilder::default()
                .create_entry()?
                .create_instance(display_handle)?
                .create_surface(display_handle, window_handle)?
                .create_device(window_inner_size)?
                .build()
                .pipe(Ok)
        }
    }

    pub fn device_wait_idle(&self) -> VkResult<()> {
        if self.is_device_destroyed {
            return Ok(());
        }

        unsafe { self.device.device_wait_idle() }
    }

    pub fn device(&self) -> &ash::Device {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_device_destroyed,
                    "VulkanContext::device() was called after device destruction");
        }
        &self.device
    }

    pub fn set_device(&mut self, device: ash::Device, physical_device: vk::PhysicalDevice) {
        // TODO maybe make a Device struct that holds both those values?

        #[cfg(feature = "validation_layers")] {
            assert!(self.is_device_destroyed,
                    "VulkanContext::set_device() was called without device destruction");
        }

        self.physical_device = physical_device;
        self.device = device;
        self.is_device_destroyed = false;
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_device_destroyed,
                    "VulkanContext::physical_device() was called after device destruction");
        }

        self.physical_device
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn surface_instance(&self) -> &ash::khr::surface::Instance {
        &self.surface_instance
    }

    pub unsafe fn destroy_device(&mut self) {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_device_destroyed,
                    "VulkanContext::destroy_device() was called after device destruction");
        }
        self.device.destroy_device(None);
        self.is_device_destroyed = true;
    }

    pub unsafe fn destroy(&mut self) {
        if !self.is_device_destroyed {
            self.destroy_device();
        }
        #[cfg(feature = "validation_layers")] {
            ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                .destroy_debug_utils_messenger(self.debug_messenger, None);
        }
        self.surface_instance.destroy_surface(self.surface, None);
        self.instance.destroy_instance(None);
    }
}
