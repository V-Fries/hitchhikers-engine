mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;
mod swap_chain;
mod builder;
mod image_views;

use ash::vk;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::utils::Result;
use crate::vulkan::builder::VulkanBuilder;

pub struct Vulkan {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

    device: ash::Device,
    queues: device::Queues,

    surface: vk::SurfaceKHR,

    swap_chain: vk::SwapchainKHR,
    swap_chain_images: Vec<vk::Image>,
    swap_chain_format: vk::Format,
    swap_chain_extent: vk::Extent2D,
    image_views: Vec<vk::ImageView>,
}

impl Vulkan {
    pub unsafe fn new(display_handle: RawDisplayHandle,
                      window_handle: RawWindowHandle,
                      window_inner_size: winit::dpi::PhysicalSize<u32>)
                      -> Result<Self> {
        Ok(
            VulkanBuilder::new(display_handle, window_handle, window_inner_size)?
                .build()
        )
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            for image_view in self.image_views.iter() {
                self.device.destroy_image_view(*image_view, None);
            }
            ash::khr::swapchain::Device::new(&self.instance, &self.device)
                .destroy_swapchain(self.swap_chain, None);
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
