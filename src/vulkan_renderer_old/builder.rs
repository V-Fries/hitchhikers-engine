use ash::prelude::VkResult;
use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use crate::vulkan_renderer_old::{device, VulkanRenderer};
use crate::vulkan_renderer_old::instance::create_instance;
#[cfg(feature = "validation_layers")]
use crate::vulkan_renderer_old::validation_layers::{check_validation_layers, setup_debug_messenger};
use crate::utils::{PipeLine, Result};
use crate::vulkan_renderer_old::device::{create_device, create_device_queue, pick_physical_device, QueueFamilies};
use super::graphics_pipeline::create_graphics_pipeline;
use crate::vulkan_renderer_old::image_views::create_image_views;
use crate::vulkan_renderer_old::swapchain::SwapchainBuilder;
use crate::vulkan_renderer_old::sync_objects::SyncObjects;

#[derive(Default)]
pub struct VulkanRendererBuilder {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    #[cfg(feature = "validation_layers")]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    swapchain_builder: Option<SwapchainBuilder>,
    queue_families: Option<QueueFamilies>,
    device: Option<ash::Device>,
    queues: Option<device::Queues>,

    surface: Option<vk::SurfaceKHR>,

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

    command_pool: Option<vk::CommandPool>,
    command_buffers: Option<Box<[vk::CommandBuffer]>>,

    image_available_semaphores: Option<Box<[vk::Semaphore]>>,
    render_finished_semaphores: Option<Box<[vk::Semaphore]>>,
    in_flight_fences: Option<Box<[vk::Fence]>>,

    nb_of_frames_in_flight: u32,
}

impl VulkanRendererBuilder {
    pub fn new(window: &winit::window::Window,
               nb_of_frames_in_flight: u32)
               -> Result<Self> {
        let display_handle = window.display_handle()?.into();
        let window_handle = window.window_handle()?.into();
        let window_inner_size = window.inner_size();

        let mut builder = Self::default()
            .create_entry()?;
        builder.nb_of_frames_in_flight = nb_of_frames_in_flight;

        #[cfg(feature = "validation_layers")] {
            check_validation_layers(builder.entry())?;
        }

        builder = builder.create_instance(display_handle)?;
        #[cfg(feature = "validation_layers")] {
            builder = builder.create_debug_messenger()?;
        }
        builder
            .create_surface(display_handle, window_handle)?
            .create_device(window_inner_size)?
            .create_queues()
            .create_swapchain()?
            .create_image_views()?
            .create_render_pass()?
            .create_graphics_pipeline()?
            .create_framebuffers()?
            .create_command_pool()?
            .create_command_buffers()?
            .create_sync_objects()?
            .pipe(Ok)
    }

    fn create_entry(mut self) -> Result<Self, ash::LoadingError> {
        self.entry = unsafe { ash::Entry::load() }?
            .pipe(Some);
        Ok(self)
    }

    fn create_instance(mut self,
                       display_handle: RawDisplayHandle)
                       -> Result<Self> {
        self.instance = create_instance(self.entry(), display_handle)?
            .pipe(Some);
        Ok(self)
    }

    #[cfg(feature = "validation_layers")]
    fn create_debug_messenger(mut self) -> Result<Self> {
        self.debug_messenger = setup_debug_messenger(self.entry(),
                                                     self.instance())?
            .pipe(Some);
        Ok(self)
    }

    fn create_surface(mut self,
                      display_handle: RawDisplayHandle,
                      window_handle: RawWindowHandle)
                      -> Result<Self> {
        self.surface = unsafe {
            ash_window::create_surface(
                self.entry(),
                self.instance(),
                display_handle,
                window_handle,
                None,
            )?
                .pipe(Some)
        };
        Ok(self)
    }

    fn create_device(mut self,
                     window_inner_size: winit::dpi::PhysicalSize<u32>)
                     -> Result<Self> {
        let device_data = pick_physical_device(
            self.entry(), self.instance(), self.surface(),
            window_inner_size,
        )?;

        self.device = create_device(self.instance(), &device_data)?
            .pipe(Some);

        self.queue_families = Some(device_data.queue_families);
        self.swapchain_builder = Some(device_data.swapchain_builder);

        Ok(self)
    }

    fn create_queues(mut self) -> Self {
        self.queues = create_device_queue(self.device(), self.queue_families())
            .pipe(Some);
        self
    }

    fn create_swapchain(mut self) -> Result<Self> {
        let swapchain_builder = self.take_swapchain_builder();

        let (swapchain, swapchain_device) = swapchain_builder.build(
            self.instance(), self.surface(), self.device(),
        )?;

        self.swapchain = Some(swapchain);
        self.swapchain_device = Some(swapchain_device);

        self.swapchain_images = unsafe {
            ash::khr::swapchain::Device::new(self.instance(), self.device())
                .get_swapchain_images(self.swapchain())?
                .pipe(Some)
        };

        self.swapchain_format = Some(swapchain_builder.format.format);
        self.swapchain_extent = Some(swapchain_builder.extent);
        Ok(self)
    }

    fn create_image_views(mut self) -> VkResult<Self> {
        self.swapchain_image_views = create_image_views(self.device(),
                                                        self.swapchain_images(),
                                                        self.swapchain_format())?
            .pipe(Some);
        Ok(self)
    }

    fn create_render_pass(mut self) -> Result<Self> {
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
            self.device().create_render_pass(&render_pass_create_info, None)?
        }.pipe(Some);

        Ok(self)
    }

    fn create_graphics_pipeline(mut self) -> Result<Self> {
        let (pipeline_layout, pipeline) = create_graphics_pipeline(
            self.device(), self.swapchain_extent(), self.render_pass(),
        )?;
        self.pipeline_layout = Some(pipeline_layout);
        self.pipeline = Some(pipeline);
        Ok(self)
    }

    fn create_framebuffers(mut self) -> Result<Self> {
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
                self.device().create_framebuffer(&create_info, None)
                    .map_err(|err| {
                        for framebuffer in framebuffers.iter() {
                            self.device().destroy_framebuffer(*framebuffer, None);
                        }
                        err
                    })?
            };
            framebuffers.push(framebuffer);
        }

        self.framebuffers = Some(framebuffers);
        Ok(self)
    }

    fn create_command_pool(mut self) -> Result<Self> {
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(self.queue_families().graphics_index);

        self.command_pool = unsafe {
            self.device()
                .create_command_pool(&create_info, None)?
                .pipe(Some)
        };
        Ok(self)
    }

    fn create_command_buffers(mut self)
                              -> Result<Self> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.nb_of_frames_in_flight);

        self.command_buffers = unsafe {
            self.device()
                .allocate_command_buffers(&alloc_info)?
                .into_boxed_slice()
                .pipe(Some)
        };
        Ok(self)
    }

    fn create_sync_objects(mut self)
                           -> Result<Self> {
        let sync_objects = SyncObjects::new(self.device(), self.nb_of_frames_in_flight)?;
        self.image_available_semaphores = Some(sync_objects.image_available_semaphores);
        self.render_finished_semaphores = Some(sync_objects.render_finished_semaphores);
        self.in_flight_fences = Some(sync_objects.in_flight_fences);
        Ok(self)
    }


    pub fn build(mut self) -> VulkanRenderer {
        VulkanRenderer {
            entry: self.entry.take()
                .expect("Vulkan entry was not initialised"),
            instance: self.instance.take()
                .expect("Vulkan instance was not initialised"),
            #[cfg(feature = "validation_layers")]
            debug_messenger: self.debug_messenger.take()
                .expect("Vulkan debug_messenger was not initialised"),

            device: self.device.take()
                .expect("Vulkan device was not initialised"),
            queues: self.queues.take()
                .expect("Vulkan queues was not initialised"),

            surface: self.surface.take()
                .expect("Vulkan surface was not initialised"),

            swapchain_device: self.swapchain_device.take()
                .expect("Vulkan swapchain_device was not initialised"),
            swapchain: self.swapchain.take()
                .expect("Vulkan swapchain was not initialised"),
            swapchain_images: self.swapchain_images.take()
                .expect("Vulkan swapchain_images was not initialised"),
            swapchain_format: self.swapchain_format.take()
                .expect("Vulkan swapchain_format was not initialised"),
            swapchain_extent: self.swapchain_extent.take()
                .expect("Vulkan swapchain_extent was not initialised"),
            swapchain_image_views: self.swapchain_image_views.take()
                .expect("Vulkan image_views was not initialised"),

            render_pass: self.render_pass.take()
                .expect("Vulkan render_pass was not initialised"),
            pipeline_layout: self.pipeline_layout.take()
                .expect("Vulkan pipeline_layout was not initialised"),
            pipeline: self.pipeline.take()
                .expect("Vulkan pipeline was not initialised"),

            framebuffers: self.framebuffers.take()
                .expect("Vulkan swapchain_framebuffers was not initialised"),

            command_pool: self.command_pool.take()
                .expect("Vulkan command_pool was not initialised"),
            command_buffers: self.command_buffers.take()
                .expect("Vulkan command_buffer was not initialised"),

            image_available_semaphores: self.image_available_semaphores.take()
                .expect("Vulkan image_available_semaphore was not initialised"),
            render_finished_semaphores: self.render_finished_semaphores.take()
                .expect("Vulkan render_finished_semaphore was not initialised"),
            in_flight_fences: self.in_flight_fences.take()
                .expect("Vulkan in_flight_fence was not initialised"),

            current_frame: 0,
            nb_of_frames_in_flight: self.nb_of_frames_in_flight as usize,
        }
    }


    fn entry(&self) -> &ash::Entry {
        self.entry.as_ref()
            .expect("entry() was called before the value was initialised")
    }

    fn instance(&self) -> &ash::Instance {
        self.instance.as_ref()
            .expect("instance() was called, but the value is at None")
    }

    fn device(&self) -> &ash::Device {
        self.device.as_ref()
            .expect("device() was called, but the value is at None")
    }

    fn surface(&self) -> vk::SurfaceKHR {
        self.surface
            .expect("surface() was called, but the value is at None")
    }

    fn take_swapchain_builder(&mut self) -> SwapchainBuilder {
        self.swapchain_builder.take()
            .expect("swapchain_builder() was called, but the value is at None")
    }

    fn queue_families(&self) -> &QueueFamilies {
        self.queue_families.as_ref()
            .expect("queue_families() was called, but the value is at None")
    }

    fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
            .expect("swapchain() was called, but the value is at None")
    }

    fn swapchain_images(&self) -> &[vk::Image] {
        self.swapchain_images.as_ref()
            .expect("swapchain_images() was called, but the value is at None")
    }

    fn swapchain_format(&self) -> vk::Format {
        self.swapchain_format
            .expect("swapchain_format() was called, but the value is at None")
    }

    fn swapchain_extent(&self) -> &vk::Extent2D {
        self.swapchain_extent.as_ref()
            .expect("swapchain_extent() was called, but the value is at None")
    }

    fn swapchain_image_views(&self) -> &[vk::ImageView] {
        self.swapchain_image_views.as_ref()
            .expect("swapchain_image_views() was called, but the value is at None")
    }

    fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
            .expect("render_pass() was called, but the value is at None")
    }

    fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
            .expect("command_pool() was called, but the value is at None")
    }
}

impl Drop for VulkanRendererBuilder {
    fn drop(&mut self) {
        self.destroy_sync_objects();
        self.destroy_command_pool();
        self.destroy_framebuffers();
        self.destroy_pipeline();
        self.destroy_pipeline_layout();
        self.destroy_render_pass();
        self.destroy_image_views();
        self.destroy_swapchain();
        self.destroy_device();
        #[cfg(feature = "validation_layers")] {
            self.destroy_debug_messenger();
        }
        self.destroy_surface();
        self.destroy_instance();
    }
}

impl VulkanRendererBuilder {
    fn destroy_sync_objects(&mut self) {
        if let Some(device) = &self.device {
            Self::destroy_semaphores(self.image_available_semaphores.take(), device);
            Self::destroy_semaphores(self.render_finished_semaphores.take(), device);
            Self::destroy_fences(self.in_flight_fences.take(), device);
        }
    }

    fn destroy_semaphores(semaphores: Option<Box<[vk::Semaphore]>>, device: &ash::Device) {
        if let Some(semaphores) = semaphores {
            for semaphore in semaphores.into_iter() {
                unsafe { device.destroy_semaphore(*semaphore, None); }
            }
        }
    }

    fn destroy_fences(fences: Option<Box<[vk::Fence]>>, device: &ash::Device) {
        if let Some(fences) = fences {
            for fence in fences.into_iter() {
                unsafe { device.destroy_fence(*fence, None); }
            }
        }
    }

    fn destroy_command_pool(&mut self) {
        if let Some(command_pool) = self.command_pool.take() {
            unsafe { self.device().destroy_command_pool(command_pool, None) };
        }
    }

    fn destroy_framebuffers(&mut self) {
        if let Some(framebuffers) = self.framebuffers.take() {
            for framebuffer in framebuffers {
                unsafe { self.device().destroy_framebuffer(framebuffer, None) };
            }
        }
    }

    fn destroy_pipeline(&mut self) {
        if let Some(pipeline) = self.pipeline.take() {
            unsafe { self.device().destroy_pipeline(pipeline, None) };
        }
    }

    fn destroy_pipeline_layout(&mut self) {
        if let Some(pipeline_layout) = self.pipeline_layout.take() {
            unsafe { self.device().destroy_pipeline_layout(pipeline_layout, None) };
        }
    }

    fn destroy_render_pass(&mut self) {
        if let Some(render_pass) = self.render_pass.take() {
            unsafe { self.device().destroy_render_pass(render_pass, None) };
        }
    }

    fn destroy_image_views(&mut self) {
        if let Some(image_views) = self.swapchain_image_views.take() {
            let device = self.device();
            for image_view in image_views {
                unsafe { device.destroy_image_view(image_view, None) };
            }
        }
    }

    fn destroy_swapchain(&mut self) {
        if let Some(swapchain) = self.swapchain.take() {
            unsafe {
                ash::khr::swapchain::Device::new(self.instance(), self.device())
                    .destroy_swapchain(swapchain, None);
            }
        }
    }

    fn destroy_device(&mut self) {
        if let Some(device) = &self.device.take() {
            unsafe { device.destroy_device(None) };
        }
    }

    #[cfg(feature = "validation_layers")]
    fn destroy_debug_messenger(&mut self) {
        if let Some(debug_messenger) = self.debug_messenger.take() {
            unsafe {
                ash::ext::debug_utils::Instance::new(self.entry(), self.instance())
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
        }
    }

    fn destroy_surface(&mut self) {
        if let Some(surface) = self.surface.take() {
            unsafe {
                ash::khr::surface::Instance::new(self.entry(), self.instance())
                    .destroy_surface(surface, None);
            }
        }
    }

    fn destroy_instance(&mut self) {
        if let Some(instance) = &self.instance.take() {
            unsafe { instance.destroy_instance(None) };
        }
    }
}
