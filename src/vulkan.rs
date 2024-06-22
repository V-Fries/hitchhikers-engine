mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;
mod swap_chain;
mod builder;

use ash::vk;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[cfg(feature = "validation_layers")]
use validation_layers::*;
use crate::vulkan::instance::create_instance;
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
    swap_chain_format: vk::SurfaceFormatKHR,
    swap_chain_extent: vk::Extent2D,
}

impl Vulkan {
    pub unsafe fn new(display_handle: RawDisplayHandle,
                      window_handle: RawWindowHandle,
                      window_inner_size: winit::dpi::PhysicalSize<u32>)
                      -> Result<Self> {
        let mut builder = VulkanBuilder::default();
        builder.set_entry(ash::Entry::load()?);

        #[cfg(feature = "validation_layers")] {
            check_validation_layers(builder.get_entry())?;
        }

        builder.set_instance(create_instance(builder.get_entry(),
                                             display_handle)?);

        #[cfg(feature = "validation_layers")] {
            builder.set_debug_messenger(setup_debug_messenger(builder.get_entry(),
                                                              builder.get_instance())?);
        }

        builder.set_surface(ash_window::create_surface(
            builder.get_entry(), builder.get_instance(),
            display_handle, window_handle, None,
        )?);

        let (device, queues, swap_chain_builder) = device::create_device(
            builder.get_entry(), builder.get_instance(), builder.get_surface(),
            window_inner_size,
        )?;
        builder.set_device(device);
        builder.set_queues(queues);

        builder.set_swap_chain(swap_chain_builder.build(
            builder.get_instance(), builder.get_surface(), builder.get_device(),
        )?);
        builder.set_swap_chain_images(
            ash::khr::swapchain::Device::new(builder.get_instance(),
                                             builder.get_device())
                .get_swapchain_images(builder.get_swap_chain())?
        );
        builder.set_swap_chain_format(swap_chain_builder.format);
        builder.set_swap_chain_extent(swap_chain_builder.extent);

        Ok(builder.build())
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
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
