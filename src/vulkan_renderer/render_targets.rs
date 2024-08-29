mod builder;
mod errors;

use crate::utils::{Result, PipeLine};
use ash::vk;

use super::vulkan_context::{SwapchainBuilder, VulkanContext};
use builder::RenderTargetsBuilder;

pub struct RenderTargets {
    is_destroyed: bool,

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

    pub fn render_pass(&self) -> vk::RenderPass {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_destroyed,
                    "RenderTargets::render_pass() was called after render_targets destruction");
        }
        self.render_pass
    }

    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_destroyed,
                    "RenderTargets::framebuffers() was called after render_targets destruction");
        }
        &self.framebuffers
    }

    pub fn swapchain_extent(&self) -> vk::Extent2D {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_destroyed,
                    "RenderTargets::swapchain_extent() was called after render_targets destruction");
        }
        self.swapchain_extent
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_destroyed,
                    "RenderTargets::pipeline() was called after render_targets destruction");
        }
        self.pipeline
    }

    pub fn swapchain_device(&self) -> &ash::khr::swapchain::Device {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_destroyed,
                    "RenderTargets::swapchain_device() was called after render_targets destruction");
        }
        &self.swapchain_device
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        #[cfg(feature = "validation_layers")] {
            assert!(!self.is_destroyed,
                    "RenderTargets::swapchain() was called after render_targets destruction");
        }
        self.swapchain
    }

    pub unsafe fn destroy(&mut self, context: &VulkanContext) {
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

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
