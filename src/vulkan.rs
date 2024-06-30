mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;
mod swap_chain;
mod builder;
mod image_views;
mod shader;

use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::utils::{PipeLine, Result};
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
    pub fn new(window: &winit::window::Window)
               -> Result<Self> {
        let display_handle = window.display_handle()?.into();
        let window_handle = window.window_handle()?.into();

        VulkanBuilder::new(display_handle, window_handle, window.inner_size())?
            .build()
            .pipe(Ok)
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
