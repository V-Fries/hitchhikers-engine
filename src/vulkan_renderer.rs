mod vulkan_context;

use crate::utils::Result;
use ash::vk;
use vulkan_context::VulkanContext;

pub struct VulkanRenderer {
    context: VulkanContext,
    // render_targets: RenderTargets,
    // interface: Interface,

    // current_frame: usize,
    // nb_of_frames_in_flight: usize,
}

impl VulkanRenderer {
    pub fn new(window: &winit::window::Window, _nb_of_frames_in_flight: u32) -> Result<Self> {
        // TODO use a builder
        Ok(Self {
            context: VulkanContext::new(window)?.0
        })
    }
}

pub struct RenderTargets {
    swapchain_device: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Vec<vk::ImageView>,

    framebuffers: Vec<vk::Framebuffer>,
}

pub struct Interface {
    queues: Queues,

    command_pool: vk::CommandPool,
    command_buffers: Box<[vk::CommandBuffer]>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    sync: Sync,
}

pub struct Queues {
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

pub struct Sync {
    image_available_semaphores: Box<[vk::Semaphore]>,
    render_finished_semaphores: Box<[vk::Semaphore]>,
    in_flight_fences: Box<[vk::Fence]>,
}
