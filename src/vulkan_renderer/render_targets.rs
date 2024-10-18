mod create_framebuffers;
mod create_render_pass;
mod errors;
mod graphics_pipeline;
mod image_views;

use crate::utils::{Defer, Result, ScopeGuard};
use ash::vk;
use create_framebuffers::create_framebuffers;
use create_render_pass::create_render_pass;
use graphics_pipeline::create_graphics_pipeline;
use image_views::create_image_views;

use super::vulkan_context::{SwapchainBuilder, VulkanContext};

pub struct RenderTargets {
    is_destroyed: bool,

    swapchain_device: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    #[allow(dead_code)]
    swapchain_images: Box<[vk::Image]>,
    #[allow(dead_code)]
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Box<[vk::ImageView]>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    framebuffers: Box<[vk::Framebuffer]>,
}

impl RenderTargets {
    pub unsafe fn new(
        context: &VulkanContext,
        swapchain_builder: SwapchainBuilder,
    ) -> Result<Self> {
        let (swapchain, swapchain_device) =
            swapchain_builder.build(context.instance(), context.surface(), context.device())?;
        let swapchain = swapchain.defer(|swapchain| {
            swapchain_device.destroy_swapchain(swapchain, None);
        });
        let swapchain_images =
            ash::khr::swapchain::Device::new(context.instance(), context.device())
                .get_swapchain_images(*swapchain)?
                .into_boxed_slice();
        let swapchain_format = swapchain_builder.format.format;
        let swapchain_extent = swapchain_builder.extent;
        let swapchain_image_views =
            create_image_views(context.device(), &swapchain_images, swapchain_format)?
                .defer(|image_views| Self::destroy_image_views(&image_views, context));

        let render_pass = create_render_pass(context.device(), swapchain_format)?
            .defer(|render_pass| context.device().destroy_render_pass(render_pass, None));

        let (pipeline_layout, pipeline) =
            create_graphics_pipeline(context.device(), &swapchain_extent, *render_pass)?;
        let pipeline_layout = pipeline_layout.defer(|pipeline_layout| {
            context
                .device()
                .destroy_pipeline_layout(pipeline_layout, None)
        });
        let pipeline = pipeline.defer(|pipeline| context.device().destroy_pipeline(pipeline, None));

        let framebuffers = create_framebuffers(
            context.device(),
            *render_pass,
            swapchain_extent,
            &swapchain_image_views,
        )?
        .defer(|framebuffers| Self::destroy_framebuffers(&framebuffers, context));

        Ok(RenderTargets {
            framebuffers: ScopeGuard::into_inner(framebuffers),
            pipeline: ScopeGuard::into_inner(pipeline),
            pipeline_layout: ScopeGuard::into_inner(pipeline_layout),
            render_pass: ScopeGuard::into_inner(render_pass),
            swapchain_image_views: ScopeGuard::into_inner(swapchain_image_views),
            swapchain: ScopeGuard::into_inner(swapchain),
            swapchain_extent,
            swapchain_format,
            swapchain_images,
            swapchain_device,
            is_destroyed: false,
        })
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        debug_assert!(
            !self.is_destroyed,
            "RenderTargets::render_pass() was called after render_targets destruction"
        );
        self.render_pass
    }

    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        debug_assert!(
            !self.is_destroyed,
            "RenderTargets::framebuffers() was called after render_targets destruction"
        );
        &self.framebuffers
    }

    pub fn swapchain_extent(&self) -> vk::Extent2D {
        debug_assert!(
            !self.is_destroyed,
            "RenderTargets::swapchain_extent() was called after render_targets destruction"
        );
        self.swapchain_extent
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        debug_assert!(
            !self.is_destroyed,
            "RenderTargets::pipeline() was called after render_targets destruction"
        );
        self.pipeline
    }

    pub fn swapchain_device(&self) -> &ash::khr::swapchain::Device {
        debug_assert!(
            !self.is_destroyed,
            "RenderTargets::swapchain_device() was called after render_targets destruction"
        );
        &self.swapchain_device
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        debug_assert!(
            !self.is_destroyed,
            "RenderTargets::swapchain() was called after render_targets destruction"
        );
        self.swapchain
    }

    pub unsafe fn destroy(&mut self, context: &VulkanContext) {
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

        Self::destroy_framebuffers(&self.framebuffers, context);
        context.device().destroy_pipeline(self.pipeline, None);
        context
            .device()
            .destroy_pipeline_layout(self.pipeline_layout, None);
        context.device().destroy_render_pass(self.render_pass, None);
        Self::destroy_image_views(&self.swapchain_image_views, context);
        ash::khr::swapchain::Device::new(context.instance(), context.device())
            .destroy_swapchain(self.swapchain, None);
    }

    unsafe fn destroy_framebuffers(framebuffers: &[vk::Framebuffer], context: &VulkanContext) {
        for framebuffer in framebuffers {
            context.device().destroy_framebuffer(*framebuffer, None);
        }
    }

    unsafe fn destroy_image_views(image_views: &[vk::ImageView], context: &VulkanContext) {
        for image_view in image_views {
            context.device().destroy_image_view(*image_view, None);
        }
    }
}
