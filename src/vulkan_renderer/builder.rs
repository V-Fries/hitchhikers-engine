use ash::vk;

use crate::{
    utils::{Defer, PipeLine, Result},
    vertex::Vertex,
};

use super::{
    buffer::Buffer,
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
    vertex_buffer: Option<Buffer>,

    queue_families: Option<QueueFamilies>,
    swapchain_builder: Option<SwapchainBuilder>,
}

impl VulkanRendererBuilder {
    pub fn build(mut self) -> VulkanRenderer {
        VulkanRenderer {
            context: self.context.take().unwrap(),
            interface: self.interface.take().unwrap(),
            render_targets: self.render_targets.take().unwrap(),
            vertex_buffer: self.vertex_buffer.take().unwrap(),
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

    pub fn create_interface(mut self) -> Result<Self> {
        unsafe {
            self.interface =
                VulkanInterface::new(self.context.as_ref().unwrap(), self.queue_families.unwrap())?
                    .pipe(Some);
        }
        Ok(self)
    }

    pub fn create_render_targets(mut self) -> Result<Self> {
        unsafe {
            self.render_targets = RenderTargets::new(
                self.context.as_ref().unwrap(),
                self.swapchain_builder.take().unwrap(),
            )?
            .pipe(Some);
        }
        Ok(self)
    }

    pub fn create_vertex_buffer(mut self) -> Result<Self> {
        // TODO remove this
        let vertices = vec![
            Vertex::new([0.0, -0.5], [1.0, 0.0, 0.0]),
            Vertex::new([0.5, 0.5], [0.0, 1.0, 0.0]),
            Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.0]),
        ];

        let context = self.context.as_ref().unwrap();
        let interface = self.interface.as_ref().unwrap();
        let buffer_size = (size_of_val(&vertices[0]) * vertices.len()) as vk::DeviceSize;

        {
            let staging_buffer = Buffer::new(
                context,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::SharingMode::EXCLUSIVE,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?
            .defer(|mut staging_buffer| unsafe {
                staging_buffer.destroy(context.device());
            });

            unsafe {
                staging_buffer.copy_from_ram(&vertices, 0, context.device())?;
            }

            let vertex_buffer = Buffer::new(
                context,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;

            unsafe {
                vertex_buffer.copy_from_buffer(
                    &staging_buffer,
                    0,
                    0,
                    buffer_size,
                    context.device(),
                    interface.command_pool(),
                    interface.queues().graphics_queue(),
                )?;
            }

            self.vertex_buffer = Some(vertex_buffer);
        }

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
