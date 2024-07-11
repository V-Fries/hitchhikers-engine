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

    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
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

    fn record_command_buffer(&self, image_index: usize) -> Result<()> {
        // TODO refactor
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.device.begin_command_buffer(self.command_buffer, &begin_info)?;
        }

        let clear_values = [
            vk::ClearValue {
                // TODO check if I should use float32
                color: vk::ClearColorValue { uint32: [0, 0, 0, 1] }
            }
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.cmd_begin_render_pass(
                self.command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                self.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline,
            );
        }

        let viewports = [vk::Viewport::default()
            .x(0.)
            .y(0.)
            .width(self.swapchain_extent.width as f32)
            .height(self.swapchain_extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)];
        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.swapchain_extent)];
        unsafe {
            self.device.cmd_set_viewport(self.command_buffer, 0, &viewports);
            self.device.cmd_set_scissor(self.command_buffer, 0, &scissors);
            self.device.cmd_draw(self.command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_render_pass(self.command_buffer);
            self.device.end_command_buffer(self.command_buffer)?;
        }
        Ok(())
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
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
