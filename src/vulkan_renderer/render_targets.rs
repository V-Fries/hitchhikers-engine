mod builder;
mod errors;

use crate::utils::{Result, PipeLine};
use ash::vk;

use super::vulkan_context::{SwapchainBuilder, VulkanContext};
use builder::RenderTargetsBuilder;

pub struct RenderTargets {
    swapchain_device: ash::khr::swapchain::Device,
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

impl RenderTargets {
    pub unsafe fn new(context: &VulkanContext,
                      swapchain_builder: SwapchainBuilder)
                      -> Result<Self> {
        RenderTargetsBuilder::new(context)
            .create_swapchain(swapchain_builder)?
            .create_image_views()?
            .create_render_pass()?
            .create_graphics_pipeline()?
            .create_framebuffers()?
            .build()
            .pipe(Ok)
    }

    pub unsafe fn destroy(&self, context: &VulkanContext) {
        for framebuffer in self.framebuffers.iter() {
            unsafe { context.device().destroy_framebuffer(*framebuffer, None) };
        }
        unsafe { context.device().destroy_pipeline(self.pipeline, None) };
        unsafe { context.device().destroy_pipeline_layout(self.pipeline_layout, None) };
        unsafe { context.device().destroy_render_pass(self.render_pass, None) };
        for image_view in self.swapchain_image_views.iter() {
            unsafe { context.device().destroy_image_view(*image_view, None) };
        }
        unsafe {
            ash::khr::swapchain::Device::new(
                context.instance(), context.device()
            )
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
