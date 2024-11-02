use std::{
    ffi::c_void,
    mem::{self, MaybeUninit},
};

use ash::{prelude::VkResult, vk};

use crate::{
    utils::{PipeLine, Result},
    vulkan_renderer::{
        buffer::Buffer, uniform_buffer_object::UniformBufferObject, vulkan_context::VulkanContext,
        NB_OF_FRAMES_IN_FLIGHT_USIZE,
    },
};

const BUFFER_SIZE: vk::DeviceSize = size_of::<UniformBufferObject>() as vk::DeviceSize;

pub fn create_uniform_buffers(
    context: &VulkanContext,
) -> Result<(
    [Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    [*mut c_void; NB_OF_FRAMES_IN_FLIGHT_USIZE],
)> {
    let mut buffers = [const { MaybeUninit::uninit() }; NB_OF_FRAMES_IN_FLIGHT_USIZE];
    let mut mapped_buffers = [const { MaybeUninit::uninit() }; NB_OF_FRAMES_IN_FLIGHT_USIZE];

    for i in 0..NB_OF_FRAMES_IN_FLIGHT_USIZE {
        create_buffer(context)
            .inspect_err(|_| unsafe {
                destroy_uniform_buffers(context.device(), &mut buffers[..i])
            })?
            .pipe(|buffer| buffers[i].write(buffer));

        unsafe { create_mapped_memory(context, buffers[i].assume_init_ref()) }
            .inspect_err(|_| unsafe {
                buffers[i].assume_init_mut().destroy(context.device());
                destroy_uniform_buffers(context.device(), &mut buffers[..i])
            })?
            .pipe(|mapped_buffer| mapped_buffers[i].write(mapped_buffer));
    }

    unsafe {
        Ok((
            mem::transmute::<
                [MaybeUninit<Buffer>; NB_OF_FRAMES_IN_FLIGHT_USIZE],
                [Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],
            >(buffers),
            mem::transmute::<
                [MaybeUninit<*mut c_void>; NB_OF_FRAMES_IN_FLIGHT_USIZE],
                [*mut c_void; NB_OF_FRAMES_IN_FLIGHT_USIZE],
            >(mapped_buffers),
        ))
    }
}

fn create_buffer(context: &VulkanContext) -> Result<Buffer> {
    Buffer::new(
        context,
        BUFFER_SIZE,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
}

unsafe fn create_mapped_memory(context: &VulkanContext, buffer: &Buffer) -> VkResult<*mut c_void> {
    context
        .device()
        .map_memory(buffer.memory(), 0, BUFFER_SIZE, vk::MemoryMapFlags::empty())
}

unsafe fn destroy_uniform_buffers(device: &ash::Device, buffers: &mut [MaybeUninit<Buffer>]) {
    for buffer in buffers {
        unsafe {
            device.unmap_memory(buffer.assume_init_ref().memory());
            buffer.assume_init_mut().destroy(device);
        }
    }
}
