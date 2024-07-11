mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;
mod swapchain;
mod builder;
mod image_views;
mod graphics_pipeline;

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

    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    framebuffers: Vec<vk::Framebuffer>,
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
            for framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(*image_view, None);
            }
            ash::khr::swapchain::Device::new(&self.instance, &self.device)
                .destroy_swapchain(self.swapchain, None);
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
