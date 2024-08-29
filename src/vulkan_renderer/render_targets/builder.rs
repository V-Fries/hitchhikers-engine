mod image_views;
mod graphics_pipeline;

use ash::{prelude::VkResult, vk};

use crate::vulkan_renderer::vulkan_context::{SwapchainBuilder, VulkanContext};
use graphics_pipeline::create_graphics_pipeline;
use image_views::create_image_views;
use super::RenderTargets;
use crate::utils::{Result, PipeLine};

pub struct RenderTargetsBuilder<'a> {
    context: &'a VulkanContext,
    swapchain_device: Option<ash::khr::swapchain::Device>,
    swapchain: Option<vk::SwapchainKHR>,
    swapchain_images: Option<Vec<vk::Image>>,
    swapchain_format: Option<vk::Format>,
    swapchain_extent: Option<vk::Extent2D>,
    swapchain_image_views: Option<Vec<vk::ImageView>>,

    render_pass: Option<vk::RenderPass>,
    pipeline_layout: Option<vk::PipelineLayout>,
    pipeline: Option<vk::Pipeline>,

    framebuffers: Option<Vec<vk::Framebuffer>>,
}

impl<'a> RenderTargetsBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        Self {
            context,
            swapchain_device: None,
            swapchain: None,
            swapchain_images: None,
            swapchain_format: None,
            swapchain_extent: None,
            swapchain_image_views: None,

            render_pass: None,
            pipeline_layout: None,
            pipeline: None,

            framebuffers: None,
        }
    }

    pub unsafe fn build(mut self) -> RenderTargets {
        RenderTargets {
            is_destroyed: false,

            swapchain_device: self.swapchain_device.take().unwrap(),
            swapchain: self.swapchain.take().unwrap(),
            swapchain_images: self.swapchain_images.take().unwrap(),
            swapchain_format: self.swapchain_format.take().unwrap(),
            swapchain_extent: self.swapchain_extent.take().unwrap(),
            swapchain_image_views: self.swapchain_image_views.take().unwrap(),

            render_pass: self.render_pass.take().unwrap(),
            pipeline_layout: self.pipeline_layout.take().unwrap(),
            pipeline: self.pipeline.take().unwrap(),

            framebuffers: self.framebuffers.take().unwrap(),
        }
    }

    pub unsafe fn create_swapchain(mut self,
                                   swapchain_builder: SwapchainBuilder)
                                   -> Result<Self> {
        let (swapchain, swapchain_device) = swapchain_builder.build(
            self.context.instance(), self.context.surface(), self.context.device(),
        )?;

        self.swapchain = Some(swapchain);
        self.swapchain_device = Some(swapchain_device);

        self.swapchain_images = unsafe {
            ash::khr::swapchain::Device::new(self.context.instance(), self.context.device())
                .get_swapchain_images(self.swapchain())?
                .pipe(Some)
        };

        self.swapchain_format = Some(swapchain_builder.format.format);
        self.swapchain_extent = Some(swapchain_builder.extent);
        Ok(self)
    }

    pub unsafe fn create_image_views(mut self) -> VkResult<Self> {
        self.swapchain_image_views = create_image_views(self.context.device(),
                                                        self.swapchain_images(),
                                                        self.swapchain_format())?
            .pipe(Some);
        Ok(self)
    }

    pub unsafe fn create_render_pass(mut self) -> Result<Self> {
        // TODO refactor
        let attachment_description = [vk::AttachmentDescription::default()
            .format(self.swapchain_format())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let color_attachment = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let subpass = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment)];

        let dependencies = [vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let render_pass_create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment_description)
            .subpasses(&subpass)
            .dependencies(&dependencies);

        self.render_pass = unsafe {
            self.context.device().create_render_pass(&render_pass_create_info, None)?
        }.pipe(Some);

        Ok(self)
    }

     pub unsafe fn create_graphics_pipeline(mut self) -> Result<Self> {
        let (pipeline_layout, pipeline) = create_graphics_pipeline(
            self.context.device(), self.swapchain_extent(), self.render_pass(),
        )?;
        self.pipeline_layout = Some(pipeline_layout);
        self.pipeline = Some(pipeline);
        Ok(self)
    }

    pub unsafe fn create_framebuffers(mut self) -> Result<Self> {
        let mut framebuffers = Vec::with_capacity(self.swapchain_image_views().len());

        for image in self.swapchain_image_views().iter() {
            let attachments = [*image];
            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(self.render_pass())
                .attachments(&attachments)
                .width(self.swapchain_extent().width)
                .height(self.swapchain_extent().height)
                .layers(1);
            let framebuffer = unsafe {
                self.context.device().create_framebuffer(&create_info, None)
                    .map_err(|err| {
                        for framebuffer in framebuffers.iter() {
                            self.context.device().destroy_framebuffer(*framebuffer, None);
                        }
                        err
                    })?
            };
            framebuffers.push(framebuffer);
        }

        self.framebuffers = Some(framebuffers);
        Ok(self)
    }

    unsafe fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain.unwrap()
    }

    unsafe fn swapchain_images(&self) -> &[vk::Image] {
        self.swapchain_images.as_ref().unwrap()
    }

    unsafe fn swapchain_format(&self) -> vk::Format {
        self.swapchain_format.unwrap()
    }

    unsafe fn swapchain_extent(&self) -> &vk::Extent2D {
        self.swapchain_extent.as_ref().unwrap()
    }

    unsafe fn swapchain_image_views(&self) -> &[vk::ImageView] {
        self.swapchain_image_views.as_ref().unwrap()
    }

    unsafe fn render_pass(&self) -> vk::RenderPass {
        self.render_pass.unwrap()
    }
}

impl Drop for RenderTargetsBuilder<'_> {
    fn drop(&mut self) {
        self.destroy_framebuffers();
        self.destroy_pipeline();
        self.destroy_pipeline_layout();
        self.destroy_render_pass();
        self.destroy_image_views();
        self.destroy_swapchain();
    }
}

impl RenderTargetsBuilder<'_> {
    fn destroy_framebuffers(&mut self) {
        if let Some(framebuffers) = self.framebuffers.take() {
            for framebuffer in framebuffers {
                unsafe { self.context.device().destroy_framebuffer(framebuffer, None) };
            }
        }
    }

    fn destroy_pipeline(&mut self) {
        if let Some(pipeline) = self.pipeline.take() {
            unsafe { self.context.device().destroy_pipeline(pipeline, None) };
        }
    }

    fn destroy_pipeline_layout(&mut self) {
        if let Some(pipeline_layout) = self.pipeline_layout.take() {
            unsafe { self.context.device().destroy_pipeline_layout(pipeline_layout, None) };
        }
    }

    fn destroy_render_pass(&mut self) {
        if let Some(render_pass) = self.render_pass.take() {
            unsafe { self.context.device().destroy_render_pass(render_pass, None) };
        }
    }

    fn destroy_image_views(&mut self) {
        if let Some(image_views) = self.swapchain_image_views.take() {
            let device = self.context.device();
            for image_view in image_views {
                unsafe { device.destroy_image_view(image_view, None) };
            }
        }
    }

    fn destroy_swapchain(&mut self) {
        if let Some(swapchain) = self.swapchain.take() {
            unsafe {
                ash::khr::swapchain::Device::new(
                    self.context.instance(), self.context.device()
                )
                    .destroy_swapchain(swapchain, None);
            }
        }
    }
}
