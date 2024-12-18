use ash::vk;
use object_parser::Vertex;

use crate::vulkan_renderer::{
    buffer::Buffer, vulkan_context::VulkanContext, vulkan_interface::VulkanInterface,
};
use rs42::{
    scope_guard::{Defer, ScopeGuard},
    Result,
};

pub unsafe fn create_vertex_buffer(
    context: &VulkanContext,
    interface: &VulkanInterface,
    vertices: &[Vertex],
) -> Result<Buffer> {
    let buffer_size = (size_of_val(&vertices[0]) * vertices.len()) as vk::DeviceSize;

    let staging_buffer = Buffer::new(
        context,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?
    .defer(|mut staging_buffer| unsafe { staging_buffer.destroy(context.device()) });

    unsafe { staging_buffer.copy_from_ram(0, vertices, context.device())? }

    let vertex_buffer = Buffer::new(
        context,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?
    .defer(|mut vertex_buffer| unsafe { vertex_buffer.destroy(context.device()) });

    unsafe {
        vertex_buffer.copy_from_buffer(
            0,
            &staging_buffer,
            0,
            buffer_size,
            context.device(),
            interface,
        )?;
    }

    Ok(ScopeGuard::into_inner(vertex_buffer))
}
