mod errors;
mod builder;
mod queue_families;

use ash::vk;
use crate::utils::{PipeLine, Result};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use builder::VulkanContextBuilder;
use crate::vulkan_renderer::vulkan_context::builder::SwapchainBuilder;
use crate::vulkan_renderer::vulkan_context::queue_families::QueueFamilies;

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

    // pub fn get_device_queues(&self) -> Queues {
    //     Queues {
    //         graphics_queue: unsafe {
    //             self.device.get_device_queue(self.device_queue_families.graphics_index(), 0)
    //         },
    //         present_queue: unsafe {
    //             self.device.get_device_queue(self.device_queue_families.present_index(), 0)
    //         },
    //     }
    // }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            #[cfg(feature = "validation_layers")] {
                ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            ash::khr::surface::Instance::new(&self.entry, &self.instance)
                .destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
