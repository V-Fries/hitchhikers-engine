mod create_index_buffer;
mod create_vertex_buffer;

use std::sync::LazyLock;

use crate::{
    utils::{Defer, Result, ScopeGuard},
    vertex::Vertex,
};

use super::{buffer::Buffer, vulkan_context::VulkanContext, vulkan_interface::VulkanInterface};
use create_index_buffer::create_index_buffer;
use create_vertex_buffer::create_vertex_buffer;

// TODO remove this
pub static VERTICES: LazyLock<[Vertex; 4]> = LazyLock::new(|| {
    [
        Vertex::new([-0.5, -0.5], [1.0, 0.0, 0.0]),
        Vertex::new([0.5, -0.5], [0.0, 1.0, 0.0]),
        Vertex::new([0.5, 0.5], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5, 0.5], [1.0, 1.0, 1.0]),
    ]
});

// TODO remove this
pub static INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub struct Memory {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Memory {
    pub unsafe fn new(context: &VulkanContext, interface: &VulkanInterface) -> Result<Self> {
        let vertex_buffer = unsafe { create_vertex_buffer(context, interface)? }
            .defer(|mut vertex_buffer| unsafe { vertex_buffer.destroy(context.device()) });

        let index_buffer = unsafe { create_index_buffer(context, interface)? }
            .defer(|mut index_buffer| unsafe { index_buffer.destroy(context.device()) });

        Ok(Self {
            vertex_buffer: ScopeGuard::into_inner(vertex_buffer),
            index_buffer: ScopeGuard::into_inner(index_buffer),
        })
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
    }
}
