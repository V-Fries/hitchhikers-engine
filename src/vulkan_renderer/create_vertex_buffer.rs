use ash::vk;

use crate::{
    utils::{Defer, Result, ScopeGuard},
    vertex::Vertex,
};

use super::{buffer::Buffer, vulkan_context::VulkanContext, vulkan_interface::VulkanInterface};

pub unsafe fn create_vertex_buffer(
    context: &VulkanContext,
    interface: &VulkanInterface,
) -> Result<Buffer> {
    // TODO remove this
    let vertices = vec![
        Vertex::new([0.0, -0.5], [1.0, 0.0, 0.0]),
        Vertex::new([0.5, 0.5], [0.0, 1.0, 0.0]),
        Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.0]),
    ];

    let buffer_size = (size_of_val(&vertices[0]) * vertices.len()) as vk::DeviceSize;

    let staging_buffer = Buffer::new(
        context,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?
    .defer(|mut staging_buffer| unsafe { staging_buffer.destroy(context.device()) });

    unsafe { staging_buffer.copy_from_ram(0, &vertices, context.device())? }

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
