mod vulkan_context;
mod vulkan_interface;
mod render_targets;

use crate::utils::Result;
use ash::vk;
use render_targets::RenderTargets;
use vulkan_context::VulkanContext;
use vulkan_interface::VulkanInterface;

const NB_OF_FRAMES_IN_FLIGHT: u32 = 2;
const NB_OF_FRAMES_IN_FLIGHT_USIZE: usize = NB_OF_FRAMES_IN_FLIGHT as usize;

pub struct VulkanRenderer {
    context: VulkanContext,
    interface: VulkanInterface,
    render_targets: RenderTargets,

    current_frame: usize,
}

impl VulkanRenderer {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        // TODO use a builder to handle leaks on error
        // TODO use generics instead of option in the builders
        let (context, queue_families, swapchain_builder) = VulkanContext::new(window)?;
        unsafe {
            Ok(Self {
                interface: VulkanInterface::new(&context, queue_families)?,
                render_targets: RenderTargets::new(&context, swapchain_builder)?,
                context,
                current_frame: 0,
            })
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            self.interface.destroy(&self.context);
            self.render_targets.destroy(&self.context);
            self.context.destroy();
        }
    }
}

