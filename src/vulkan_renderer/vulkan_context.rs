mod errors;
mod builder;
mod queue_families;

use ash::vk;
use crate::utils::{PipeLine, Result};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use builder::VulkanContextBuilder;
pub use queue_families::QueueFamilies;
pub use builder::SwapchainBuilder;

pub struct VulkanContext {
    entry: ash::Entry,
    instance: ash::Instance,

    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

    surface: vk::SurfaceKHR,

    physical_device: vk::PhysicalDevice,
    device: ash::Device,
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

    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub unsafe fn destroy_device(&mut self) {
        self.device.destroy_device(None);
    }

    pub unsafe fn destroy(&mut self) {
        self.destroy_device();
        #[cfg(feature = "validation_layers")] {
            ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                .destroy_debug_utils_messenger(self.debug_messenger, None);
        }
        ash::khr::surface::Instance::new(&self.entry, &self.instance)
            .destroy_surface(self.surface, None);
        self.instance.destroy_instance(None);
    }
}
