use ash::vk;

use crate::vulkan_renderer::{
    buffer::Buffer, memory::INDICES, vulkan_context::VulkanContext,
    vulkan_interface::VulkanInterface,
};
use rs42::{
    scope_guard::{Defer, ScopeGuard},
    Result,
};

pub unsafe fn create_index_buffer(
    context: &VulkanContext,
    interface: &VulkanInterface,
) -> Result<Buffer> {
    let buffer_size = (size_of_val(&INDICES[0]) * INDICES.len()) as vk::DeviceSize;

    let staging_buffer = Buffer::new(
        context,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?
    .defer(|mut staging_buffer| unsafe { staging_buffer.destroy(context.device()) });

    unsafe { staging_buffer.copy_from_ram(0, &INDICES, context.device())? }

    let index_buffer = Buffer::new(
        context,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?
    .defer(|mut vertex_buffer| unsafe { vertex_buffer.destroy(context.device()) });

    unsafe {
        index_buffer.copy_from_buffer(
            0,
            &staging_buffer,
            0,
            buffer_size,
            context.device(),
            interface,
        )?;
    }

    Ok(ScopeGuard::into_inner(index_buffer))
}
