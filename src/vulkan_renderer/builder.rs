use crate::{
    utils::{PipeLine, Result},
    vertex::Vertex,
};

use super::{
    render_targets::RenderTargets,
    vulkan_context::{QueueFamilies, SwapchainBuilder, VulkanContext},
    vulkan_interface::VulkanInterface,
    VulkanRenderer,
};

// TODO use generics instead of option in the builders
#[derive(Default)]
pub struct VulkanRendererBuilder {
    context: Option<VulkanContext>,
    interface: Option<VulkanInterface>,
    render_targets: Option<RenderTargets>,

    queue_families: Option<QueueFamilies>,
    swapchain_builder: Option<SwapchainBuilder>,
}

impl VulkanRendererBuilder {
    pub fn build(mut self) -> VulkanRenderer {
        VulkanRenderer {
            context: self.context.take().unwrap(),
            interface: self.interface.take().unwrap(),
            render_targets: self.render_targets.take().unwrap(),
            // TODO remove this
            vertices: vec![
                Vertex::new([0.0, -0.5], [1.0, 0.0, 0.0]),
                Vertex::new([0.5, 0.5], [0.0, 1.0, 0.0]),
                Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.0]),
            ],
            vertex_buffer: None,
            vertex_buffer_memory: None,
            current_frame: 0,
        }
    }

    pub fn create_context(mut self, window: &winit::window::Window) -> Result<Self> {
        let (context, queue_families, swapchain_builder) = VulkanContext::new(window)?;
        self.context = Some(context);
        self.queue_families = Some(queue_families);
        self.swapchain_builder = Some(swapchain_builder);
        Ok(self)
    }

    pub unsafe fn create_interface(mut self) -> Result<Self> {
        self.interface =
            VulkanInterface::new(self.context.as_ref().unwrap(), self.queue_families.unwrap())?
                .pipe(Some);
        Ok(self)
    }

    pub unsafe fn create_render_targets(mut self) -> Result<Self> {
        self.render_targets = RenderTargets::new(
            self.context.as_ref().unwrap(),
            self.swapchain_builder.take().unwrap(),
        )?
        .pipe(Some);
        Ok(self)
    }
}

impl Drop for VulkanRendererBuilder {
    fn drop(&mut self) {
        let Some(context) = &mut self.context else {
            return;
        };

        unsafe {
            if let Some(interface) = self.interface.as_mut() {
                interface.destroy(context);
            }
            if let Some(render_targets) = self.render_targets.as_mut() {
                render_targets.destroy(context);
            }
            context.destroy();
        }
    }
}
